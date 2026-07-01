//! OAuth 2.1 authorization-server core: clients, authorization codes (with
//! mandatory PKCE), and access/refresh tokens.
//!
//! This crate owns the records, the crypto (PKCE S256 verification, opaque
//! token minting + hashing), and the SQLite store. The HTTP surface —
//! `/.well-known/oauth-authorization-server`, `/.well-known/oauth-protected-resource`,
//! `/authorize`, `/token`, and dynamic client registration (`/register`) — wires
//! this in a later PR, and the MCP server validates access tokens against it as
//! an OAuth resource server.
//!
//! Security posture (OAuth 2.1):
//! - PKCE is REQUIRED for the authorization-code grant; only `S256` is accepted
//!   (`plain` is rejected).
//! - Authorization codes are single-use and short-lived; reuse is rejected
//!   atomically.
//! - Refresh tokens are ROTATED on use (the presented refresh token is revoked
//!   and a new one issued).
//! - All token secrets are stored only as SHA-256 hashes and compared in
//!   constant time; the plaintext is shown once at mint time.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use beater_core::{
    AccessTokenId, AuthCodeId, OAuthClientId, RefreshTokenId, TenantScope, Timestamp,
    TokenFamilyId, UserId,
};
use beater_store::StoreError;
use chrono::{DateTime, Duration, Utc};
use http::Uri;
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use subtle::ConstantTimeEq;
use uuid::Uuid;

/// Authorization codes live 10 minutes (RFC 6749 recommends <= 10 min).
pub fn authorization_code_ttl() -> Duration {
    Duration::minutes(10)
}

/// Access tokens live 1 hour.
pub fn access_token_ttl() -> Duration {
    Duration::hours(1)
}

/// Refresh tokens live 30 days.
pub fn refresh_token_ttl() -> Duration {
    Duration::days(30)
}

const ACCESS_TOKEN_PREFIX: &str = "bao";
const REFRESH_TOKEN_PREFIX: &str = "bro";
const AUTH_CODE_PREFIX: &str = "bac";

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    /// Maps to OAuth `invalid_grant` (bad/expired/used code or refresh token,
    /// PKCE failure, redirect/client mismatch). Deliberately coarse so it does
    /// not reveal which check failed.
    #[error("invalid_grant")]
    InvalidGrant,
    /// Maps to OAuth `invalid_client`.
    #[error("invalid_client")]
    InvalidClient,
    /// Maps to OAuth `invalid_request`.
    #[error("invalid_request: {0}")]
    InvalidRequest(String),
    /// Maps to OAuth `invalid_scope`.
    #[error("invalid_scope")]
    InvalidScope,
    /// Maps to the resource-server `invalid_token` (expired/revoked/unknown).
    #[error("invalid_token")]
    InvalidToken,
    /// `unsupported_grant_type` / method the client is not allowed to use.
    #[error("unauthorized_client")]
    UnauthorizedClient,
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, OAuthError>;

fn backend(msg: impl std::fmt::Display) -> OAuthError {
    OAuthError::Store(StoreError::backend(msg.to_string()))
}

/// OAuth grant types this server supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

impl GrantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorizationCode => "authorization_code",
            Self::RefreshToken => "refresh_token",
        }
    }
}

/// Token-endpoint client authentication method (RFC 7591). `None` marks a
/// public client that authenticates purely via PKCE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAuthMethod {
    None,
    ClientSecretBasic,
    ClientSecretPost,
}

impl ClientAuthMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ClientSecretBasic => "client_secret_basic",
            Self::ClientSecretPost => "client_secret_post",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "none" => Ok(Self::None),
            "client_secret_basic" => Ok(Self::ClientSecretBasic),
            "client_secret_post" => Ok(Self::ClientSecretPost),
            other => Err(backend(format!("unknown auth method {other}"))),
        }
    }

    fn is_public(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// A registered OAuth client. Confidential clients carry a `client_secret_hash`;
/// public clients (e.g. native MCP clients) carry `None` and rely on PKCE.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthClient {
    pub client_id: OAuthClientId,
    #[serde(skip_serializing, default)]
    pub client_secret_hash: Option<String>,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: BTreeSet<GrantType>,
    pub scopes: BTreeSet<String>,
    pub token_endpoint_auth_method: ClientAuthMethod,
    pub created_at: Timestamp,
}

impl OAuthClient {
    pub fn is_public(&self) -> bool {
        self.token_endpoint_auth_method.is_public()
    }

    pub fn allows_redirect(&self, redirect_uri: &str) -> bool {
        self.redirect_uris.iter().any(|uri| uri == redirect_uri)
    }
}

/// A new-client registration request (dynamic client registration input).
#[derive(Clone, Debug)]
pub struct ClientRegistration {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: BTreeSet<GrantType>,
    pub scopes: BTreeSet<String>,
    pub token_endpoint_auth_method: ClientAuthMethod,
}

/// Validate an OAuth redirect URI before it can be registered or used to issue
/// a code. Hosted clients must use HTTPS; native/local clients may use HTTP only
/// on the loopback hosts used by OAuth 2.1 development flows.
pub fn validate_redirect_uri(redirect_uri: &str) -> std::result::Result<(), OAuthError> {
    if redirect_uri.is_empty() || redirect_uri.contains('#') {
        return Err(invalid_redirect_uri());
    }

    let uri = redirect_uri
        .parse::<Uri>()
        .map_err(|_| invalid_redirect_uri())?;
    let Some(scheme) = uri.scheme_str() else {
        return Err(invalid_redirect_uri());
    };
    let Some(host) = uri.host().filter(|host| !host.is_empty()) else {
        return Err(invalid_redirect_uri());
    };

    if scheme.eq_ignore_ascii_case("https") {
        return Ok(());
    }
    if scheme.eq_ignore_ascii_case("http") && is_loopback_redirect_host(host) {
        return Ok(());
    }

    Err(invalid_redirect_uri())
}

fn invalid_redirect_uri() -> OAuthError {
    OAuthError::InvalidRequest(
        "redirect_uri must use https or loopback http and must not contain a fragment".to_string(),
    )
}

fn is_loopback_redirect_host(host: &str) -> bool {
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1"
}

/// The result of registering a client. `client_secret` is `Some` only for
/// confidential clients and is shown exactly once.
#[derive(Clone)]
pub struct RegisteredClient {
    pub client: OAuthClient,
    pub client_secret: Option<String>,
}

/// An issued authorization code. Single-use; bound to a client, user, redirect
/// URI, scope, and a PKCE `S256` challenge.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub code_id: AuthCodeId,
    pub client_id: OAuthClientId,
    pub user_id: UserId,
    pub redirect_uri: String,
    pub scope: BTreeSet<String>,
    /// Tenant/project/environment this authorization is bound to (chosen at
    /// `/authorize`). Tokens minted from this code inherit it.
    pub tenant_scope: TenantScope,
    pub code_challenge: String,
    #[serde(skip_serializing, default)]
    pub secret_hash: String,
    pub expires_at: Timestamp,
    pub consumed: bool,
}

/// Inputs to mint an authorization code (after the user has authenticated and
/// consented at `/authorize`). PKCE `code_challenge` is required and must be
/// `S256` — there is no `plain` path.
#[derive(Clone, Debug)]
pub struct AuthorizationGrant {
    pub client_id: OAuthClientId,
    pub user_id: UserId,
    pub redirect_uri: String,
    pub scope: BTreeSet<String>,
    /// The tenant/project/environment the user selected at `/authorize`.
    pub tenant_scope: TenantScope,
    pub code_challenge: String,
}

/// An access token's persisted record. The opaque secret is only in the hash.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessToken {
    pub token_id: AccessTokenId,
    pub client_id: OAuthClientId,
    pub user_id: UserId,
    pub scope: BTreeSet<String>,
    /// Tenant/project/environment the token is authorized for.
    pub tenant_scope: TenantScope,
    /// Lineage shared with the refresh token that minted it, so a detected
    /// refresh-token reuse can revoke the whole family.
    pub family_id: TokenFamilyId,
    #[serde(skip_serializing, default)]
    pub secret_hash: String,
    pub expires_at: Timestamp,
    pub revoked: bool,
}

/// A refresh token's persisted record.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token_id: RefreshTokenId,
    pub client_id: OAuthClientId,
    pub user_id: UserId,
    pub scope: BTreeSet<String>,
    /// Tenant/project/environment the token is authorized for.
    pub tenant_scope: TenantScope,
    /// Rotation lineage. All tokens minted from the same original authorization
    /// share a family; reuse of a rotated (revoked) refresh token revokes it.
    pub family_id: TokenFamilyId,
    #[serde(skip_serializing, default)]
    pub secret_hash: String,
    pub expires_at: Timestamp,
    pub revoked: bool,
}

/// The validated identity behind an access token, as seen by a resource server.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessTokenClaims {
    pub token_id: AccessTokenId,
    pub client_id: OAuthClientId,
    pub user_id: UserId,
    pub scope: BTreeSet<String>,
    /// Tenant/project/environment the token is authorized for — a resource
    /// server MUST check this against the resource it is protecting.
    pub tenant_scope: TenantScope,
    pub expires_at: Timestamp,
}

/// A freshly issued access + refresh token pair, with one-time plaintext values.
#[derive(Clone)]
pub struct IssuedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub scope: BTreeSet<String>,
}

fn redacted_option(value: &Option<String>) -> Option<&'static str> {
    value.as_ref().map(|_| "<redacted>")
}

impl std::fmt::Debug for OAuthClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthClient")
            .field("client_id", &self.client_id)
            .field(
                "client_secret_hash",
                &redacted_option(&self.client_secret_hash),
            )
            .field("client_name", &self.client_name)
            .field("redirect_uris", &self.redirect_uris)
            .field("grant_types", &self.grant_types)
            .field("scopes", &self.scopes)
            .field(
                "token_endpoint_auth_method",
                &self.token_endpoint_auth_method,
            )
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl std::fmt::Debug for RegisteredClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredClient")
            .field("client", &self.client)
            .field("client_secret", &redacted_option(&self.client_secret))
            .finish()
    }
}

impl std::fmt::Debug for AuthorizationCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthorizationCode")
            .field("code_id", &self.code_id)
            .field("client_id", &self.client_id)
            .field("user_id", &self.user_id)
            .field("redirect_uri", &self.redirect_uri)
            .field("scope", &self.scope)
            .field("tenant_scope", &self.tenant_scope)
            .field("code_challenge", &self.code_challenge)
            .field("secret_hash", &"<redacted>")
            .field("expires_at", &self.expires_at)
            .field("consumed", &self.consumed)
            .finish()
    }
}

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessToken")
            .field("token_id", &self.token_id)
            .field("client_id", &self.client_id)
            .field("user_id", &self.user_id)
            .field("scope", &self.scope)
            .field("tenant_scope", &self.tenant_scope)
            .field("family_id", &self.family_id)
            .field("secret_hash", &"<redacted>")
            .field("expires_at", &self.expires_at)
            .field("revoked", &self.revoked)
            .finish()
    }
}

impl std::fmt::Debug for RefreshToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshToken")
            .field("token_id", &self.token_id)
            .field("client_id", &self.client_id)
            .field("user_id", &self.user_id)
            .field("scope", &self.scope)
            .field("tenant_scope", &self.tenant_scope)
            .field("family_id", &self.family_id)
            .field("secret_hash", &"<redacted>")
            .field("expires_at", &self.expires_at)
            .field("revoked", &self.revoked)
            .finish()
    }
}

impl std::fmt::Debug for IssuedTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IssuedTokens")
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field("scope", &self.scope)
            .finish()
    }
}

// ---- crypto helpers ----

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing into a String is infallible; avoids a temporary String per byte.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn sha256_raw(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn sha256_hex(input: &[u8]) -> String {
    to_hex(&sha256_raw(input))
}

/// Verify a PKCE `S256` challenge: `BASE64URL(SHA256(verifier)) == challenge`,
/// compared in constant time. Returns false for a verifier outside the RFC 7636
/// §4.1 length/charset bounds (43–128 chars of `[A-Za-z0-9-._~]`).
pub fn verify_pkce_s256(code_verifier: &str, code_challenge: &str) -> bool {
    if !is_valid_code_verifier(code_verifier) {
        return false;
    }
    let computed = URL_SAFE_NO_PAD.encode(sha256_raw(code_verifier.as_bytes()));
    computed.as_bytes().ct_eq(code_challenge.as_bytes()).into()
}

/// RFC 7636 §4.1: code_verifier is 43–128 chars of the unreserved set.
fn is_valid_code_verifier(verifier: &str) -> bool {
    (43..=128).contains(&verifier.len())
        && verifier
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
}

/// A valid `S256` challenge is the base64url-no-pad of a 32-byte digest: exactly
/// 43 chars of the base64url alphabet.
fn is_valid_code_challenge(challenge: &str) -> bool {
    challenge.len() == 43
        && challenge
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

fn random_secret() -> String {
    let mut raw = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut raw);
    to_hex(&raw)
}

/// Mint a `<prefix>_<id>_<secret>` token; returns the plaintext token and the
/// SHA-256 hash to persist.
fn mint_token(prefix: &str, id: &str) -> (String, String) {
    let secret = random_secret();
    let token = format!("{prefix}_{id}_{secret}");
    let secret_hash = sha256_hex(secret.as_bytes());
    (token, secret_hash)
}

/// Split a `<prefix>_<id>_<secret>` token into id + secret, validating prefix.
fn parse_token<'a>(prefix: &str, token: &'a str) -> Option<(&'a str, &'a str)> {
    let rest = token.strip_prefix(prefix)?.strip_prefix('_')?;
    let (id, secret) = rest.split_once('_')?;
    if id.is_empty() || secret.is_empty() {
        return None;
    }
    Some((id, secret))
}

fn secret_matches(stored_hash: &str, presented_secret: &str) -> bool {
    sha256_hex(presented_secret.as_bytes())
        .as_bytes()
        .ct_eq(stored_hash.as_bytes())
        .into()
}

fn hash_client_secret(secret: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|err| backend(format!("hash client secret: {err}")))?
        .to_string())
}

fn verify_client_secret(hash: &str, secret: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed)
        .is_ok()
}

fn new_uuid_id<T, F>(ctor: F) -> T
where
    F: Fn(String) -> std::result::Result<T, beater_core::IdError>,
{
    ctor(Uuid::new_v4().to_string()).unwrap_or_else(|err| panic!("uuid v4 is a valid id: {err}"))
}

/// Store for OAuth clients, authorization codes, and tokens. Low-level CRUD is
/// on the trait; the security-critical flows ([`OAuthStore::exchange_code`],
/// [`OAuthStore::validate_access_token`], [`OAuthStore::refresh`]) are default
/// methods built on top of it.
#[async_trait::async_trait]
pub trait OAuthStore: Send + Sync {
    async fn put_client(&self, client: OAuthClient) -> Result<()>;
    async fn get_client(&self, client_id: &OAuthClientId) -> Result<Option<OAuthClient>>;
    async fn put_code(&self, code: AuthorizationCode) -> Result<()>;
    async fn get_code(&self, code_id: &AuthCodeId) -> Result<Option<AuthorizationCode>>;
    /// Atomically mark a code consumed. Returns `true` if this call performed the
    /// transition (i.e. the code was previously unconsumed) — the single-use
    /// guarantee. A `false` means it was already consumed (replay).
    async fn consume_code(&self, code_id: &AuthCodeId) -> Result<bool>;
    async fn put_access_token(&self, token: AccessToken) -> Result<()>;
    async fn get_access_token(&self, token_id: &AccessTokenId) -> Result<Option<AccessToken>>;
    async fn revoke_access_token(&self, token_id: &AccessTokenId) -> Result<()>;
    async fn put_refresh_token(&self, token: RefreshToken) -> Result<()>;
    async fn get_refresh_token(&self, token_id: &RefreshTokenId) -> Result<Option<RefreshToken>>;
    async fn revoke_refresh_token(&self, token_id: &RefreshTokenId) -> Result<()>;
    /// Revoke every access + refresh token sharing `family_id` (compromise
    /// containment on detected refresh-token reuse).
    async fn revoke_token_family(&self, family_id: &TokenFamilyId) -> Result<()>;

    /// Register a new client. Returns the one-time `client_secret` for
    /// confidential clients.
    async fn register_client(
        &self,
        registration: ClientRegistration,
        now: Timestamp,
    ) -> Result<RegisteredClient> {
        if registration.redirect_uris.is_empty() {
            return Err(OAuthError::InvalidRequest(
                "at least one redirect_uri is required".to_string(),
            ));
        }
        for redirect_uri in &registration.redirect_uris {
            validate_redirect_uri(redirect_uri)?;
        }
        let client_id = new_uuid_id(OAuthClientId::new);
        let (client_secret, client_secret_hash) =
            if registration.token_endpoint_auth_method.is_public() {
                (None, None)
            } else {
                let secret = random_secret();
                let hash = hash_client_secret(&secret)?;
                (Some(secret), Some(hash))
            };
        let client = OAuthClient {
            client_id,
            client_secret_hash,
            client_name: registration.client_name,
            redirect_uris: registration.redirect_uris,
            grant_types: registration.grant_types,
            scopes: registration.scopes,
            token_endpoint_auth_method: registration.token_endpoint_auth_method,
            created_at: now,
        };
        self.put_client(client.clone()).await?;
        Ok(RegisteredClient {
            client,
            client_secret,
        })
    }

    /// Authenticate a client at the token endpoint. Public clients pass
    /// `secret = None`; confidential clients must present their secret.
    async fn authenticate_client(
        &self,
        client_id: &OAuthClientId,
        secret: Option<&str>,
    ) -> Result<OAuthClient> {
        let client = self
            .get_client(client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;
        match (&client.client_secret_hash, secret) {
            (None, _) => Ok(client), // public client
            (Some(hash), Some(presented)) if verify_client_secret(hash, presented) => Ok(client),
            _ => Err(OAuthError::InvalidClient),
        }
    }

    /// Issue an authorization code after the user has authenticated + consented.
    /// Validates the redirect URI against the client and requires a non-empty
    /// PKCE challenge. Returns the one-time code value.
    async fn issue_authorization_code(
        &self,
        grant: AuthorizationGrant,
        now: Timestamp,
    ) -> Result<String> {
        let client = self
            .get_client(&grant.client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;
        if !client.allows_redirect(&grant.redirect_uri) {
            return Err(OAuthError::InvalidRequest(
                "redirect_uri not registered for client".to_string(),
            ));
        }
        validate_redirect_uri(&grant.redirect_uri)?;
        if !is_valid_code_challenge(&grant.code_challenge) {
            return Err(OAuthError::InvalidRequest(
                "PKCE code_challenge must be a base64url SHA-256 (S256) digest".to_string(),
            ));
        }
        if !grant.scope.is_subset(&client.scopes) {
            return Err(OAuthError::InvalidScope);
        }
        let code_id = new_uuid_id(AuthCodeId::new);
        let (token, secret_hash) = mint_token(AUTH_CODE_PREFIX, code_id.as_str());
        self.put_code(AuthorizationCode {
            code_id,
            client_id: grant.client_id,
            user_id: grant.user_id,
            redirect_uri: grant.redirect_uri,
            scope: grant.scope,
            tenant_scope: grant.tenant_scope,
            code_challenge: grant.code_challenge,
            secret_hash,
            expires_at: now + authorization_code_ttl(),
            consumed: false,
        })
        .await?;
        Ok(token)
    }

    /// Exchange an authorization code for tokens (the `authorization_code`
    /// grant). Authenticates the client first (confidential clients MUST present
    /// their secret; public clients pass `None` and rely on PKCE), then performs
    /// every OAuth 2.1 code check: authenticity, single-use, expiry, client +
    /// redirect-URI binding, and PKCE `S256` verification.
    async fn exchange_code(
        &self,
        client_id: &OAuthClientId,
        client_secret: Option<&str>,
        code: &str,
        redirect_uri: &str,
        code_verifier: &str,
        now: Timestamp,
    ) -> Result<IssuedTokens> {
        // Confidential-client authentication at the token endpoint (RFC 6749
        // §4.1.3 / OAuth 2.1). Returns invalid_client on failure.
        self.authenticate_client(client_id, client_secret).await?;

        let (code_id_str, secret) =
            parse_token(AUTH_CODE_PREFIX, code).ok_or(OAuthError::InvalidGrant)?;
        let code_id = AuthCodeId::new(code_id_str).map_err(|_| OAuthError::InvalidGrant)?;
        let record = self
            .get_code(&code_id)
            .await?
            .ok_or(OAuthError::InvalidGrant)?;

        // Authenticity, binding, freshness — all collapse to invalid_grant.
        if !secret_matches(&record.secret_hash, secret)
            || record.client_id.as_str() != client_id.as_str()
            || record.redirect_uri != redirect_uri
            || record.consumed
            || now >= record.expires_at
        {
            return Err(OAuthError::InvalidGrant);
        }
        if !verify_pkce_s256(code_verifier, &record.code_challenge) {
            return Err(OAuthError::InvalidGrant);
        }
        // Atomically burn the code; a lost race means it was already used.
        if !self.consume_code(&code_id).await? {
            return Err(OAuthError::InvalidGrant);
        }
        // A successful code exchange roots a new token-rotation family.
        let family_id = new_uuid_id(TokenFamilyId::new);
        self.issue_token_pair(
            record.client_id,
            record.user_id,
            record.scope,
            record.tenant_scope,
            family_id,
            now,
        )
        .await
    }

    /// Validate a bearer access token (resource-server entry point).
    async fn validate_access_token(
        &self,
        token: &str,
        now: Timestamp,
    ) -> Result<AccessTokenClaims> {
        let (id_str, secret) =
            parse_token(ACCESS_TOKEN_PREFIX, token).ok_or(OAuthError::InvalidToken)?;
        let token_id = AccessTokenId::new(id_str).map_err(|_| OAuthError::InvalidToken)?;
        let record = self
            .get_access_token(&token_id)
            .await?
            .ok_or(OAuthError::InvalidToken)?;
        if !secret_matches(&record.secret_hash, secret)
            || record.revoked
            || now >= record.expires_at
        {
            return Err(OAuthError::InvalidToken);
        }
        Ok(AccessTokenClaims {
            token_id: record.token_id,
            client_id: record.client_id,
            user_id: record.user_id,
            scope: record.scope,
            tenant_scope: record.tenant_scope,
            expires_at: record.expires_at,
        })
    }

    /// Refresh-token grant with rotation and reuse detection. Authenticates the
    /// client, then validates and REVOKES the presented refresh token before
    /// issuing a new pair (carrying the same family). If a *rotated* (already
    /// revoked) refresh token with a valid secret is replayed, that signals
    /// theft: the entire token family is revoked (RFC 9700 / OAuth 2.1 §4.13.2).
    async fn refresh(
        &self,
        client_id: &OAuthClientId,
        client_secret: Option<&str>,
        refresh_token: &str,
        now: Timestamp,
    ) -> Result<IssuedTokens> {
        self.authenticate_client(client_id, client_secret).await?;

        let (id_str, secret) =
            parse_token(REFRESH_TOKEN_PREFIX, refresh_token).ok_or(OAuthError::InvalidGrant)?;
        let token_id = RefreshTokenId::new(id_str).map_err(|_| OAuthError::InvalidGrant)?;
        let record = self
            .get_refresh_token(&token_id)
            .await?
            .ok_or(OAuthError::InvalidGrant)?;
        // Wrong secret, wrong client, or expired: reject without a family signal.
        if !secret_matches(&record.secret_hash, secret)
            || record.client_id.as_str() != client_id.as_str()
            || now >= record.expires_at
        {
            return Err(OAuthError::InvalidGrant);
        }
        // Valid secret but already revoked => a rotated token is being replayed.
        // Treat as compromise and burn the whole family.
        if record.revoked {
            self.revoke_token_family(&record.family_id).await?;
            return Err(OAuthError::InvalidGrant);
        }
        // Rotate: burn the presented token before issuing the replacement, and
        // carry the family + tenant scope forward.
        self.revoke_refresh_token(&token_id).await?;
        self.issue_token_pair(
            record.client_id,
            record.user_id,
            record.scope,
            record.tenant_scope,
            record.family_id,
            now,
        )
        .await
    }

    /// Mint + persist a fresh access + refresh token pair within `family_id`,
    /// bound to `tenant_scope`.
    async fn issue_token_pair(
        &self,
        client_id: OAuthClientId,
        user_id: UserId,
        scope: BTreeSet<String>,
        tenant_scope: TenantScope,
        family_id: TokenFamilyId,
        now: Timestamp,
    ) -> Result<IssuedTokens> {
        let access_id = new_uuid_id(AccessTokenId::new);
        let (access_token, access_hash) = mint_token(ACCESS_TOKEN_PREFIX, access_id.as_str());
        let access_expires = now + access_token_ttl();
        self.put_access_token(AccessToken {
            token_id: access_id,
            client_id: client_id.clone(),
            user_id: user_id.clone(),
            scope: scope.clone(),
            tenant_scope: tenant_scope.clone(),
            family_id: family_id.clone(),
            secret_hash: access_hash,
            expires_at: access_expires,
            revoked: false,
        })
        .await?;

        let refresh_id = new_uuid_id(RefreshTokenId::new);
        let (refresh_token, refresh_hash) = mint_token(REFRESH_TOKEN_PREFIX, refresh_id.as_str());
        self.put_refresh_token(RefreshToken {
            token_id: refresh_id,
            client_id,
            user_id,
            scope: scope.clone(),
            tenant_scope,
            family_id,
            secret_hash: refresh_hash,
            expires_at: now + refresh_token_ttl(),
            revoked: false,
        })
        .await?;

        Ok(IssuedTokens {
            access_token,
            refresh_token,
            token_type: "Bearer",
            expires_in: access_token_ttl().num_seconds(),
            scope,
        })
    }
}

#[derive(Clone)]
pub struct SqliteOAuthStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteOAuthStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory()?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> anyhow::Result<()> {
        let connection = self.lock()?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS oauth_clients (
                client_id TEXT PRIMARY KEY,
                client_secret_hash TEXT,
                client_name TEXT NOT NULL,
                redirect_uris TEXT NOT NULL,
                grant_types TEXT NOT NULL,
                scopes TEXT NOT NULL,
                token_endpoint_auth_method TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS oauth_codes (
                code_id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                redirect_uri TEXT NOT NULL,
                scope TEXT NOT NULL,
                tenant_scope TEXT NOT NULL,
                code_challenge TEXT NOT NULL,
                secret_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                consumed INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS oauth_access_tokens (
                token_id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                scope TEXT NOT NULL,
                tenant_scope TEXT NOT NULL,
                family_id TEXT NOT NULL,
                secret_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                revoked INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_access_family ON oauth_access_tokens(family_id);

            CREATE TABLE IF NOT EXISTS oauth_refresh_tokens (
                token_id TEXT PRIMARY KEY,
                client_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                scope TEXT NOT NULL,
                tenant_scope TEXT NOT NULL,
                family_id TEXT NOT NULL,
                secret_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                revoked INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_refresh_family ON oauth_refresh_tokens(family_id);
            "#,
        )?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow::anyhow!("oauth sqlite mutex poisoned: {err}"))
    }
}

fn parse_time(value: String) -> Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|err| backend(format!("parse timestamp {value}: {err}")))
}

fn encode_set<T: Serialize>(values: &T) -> Result<String> {
    serde_json::to_string(values).map_err(|err| backend(format!("encode json: {err}")))
}

fn decode_set<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T> {
    serde_json::from_str(value).map_err(|err| backend(format!("decode json: {err}")))
}

#[async_trait::async_trait]
impl OAuthStore for SqliteOAuthStore {
    async fn put_client(&self, client: OAuthClient) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                -- Insert-only: registration must never silently overwrite an
                -- existing client's secret/redirect_uris/scopes. A reused id is
                -- a hard error, not an upsert.
                INSERT INTO oauth_clients
                  (client_id, client_secret_hash, client_name, redirect_uris, grant_types,
                   scopes, token_endpoint_auth_method, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    client.client_id.as_str(),
                    client.client_secret_hash,
                    client.client_name,
                    encode_set(&client.redirect_uris)?,
                    encode_set(&client.grant_types)?,
                    encode_set(&client.scopes)?,
                    client.token_endpoint_auth_method.as_str(),
                    client.created_at.to_rfc3339(),
                ],
            )
            .map_err(|err| backend(format!("insert client: {err}")))?;
        Ok(())
    }

    async fn get_client(&self, client_id: &OAuthClientId) -> Result<Option<OAuthClient>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT client_id, client_secret_hash, client_name, redirect_uris, grant_types, \
                 scopes, token_endpoint_auth_method, created_at FROM oauth_clients WHERE client_id = ?1",
                params![client_id.as_str()],
                decode_client,
            )
            .optional()
            .map_err(|err| backend(format!("get client: {err}")))?
            .transpose()
    }

    async fn put_code(&self, code: AuthorizationCode) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO oauth_codes
                  (code_id, client_id, user_id, redirect_uri, scope, tenant_scope, code_challenge,
                   secret_hash, expires_at, consumed)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
                params![
                    code.code_id.as_str(),
                    code.client_id.as_str(),
                    code.user_id.as_str(),
                    code.redirect_uri,
                    encode_set(&code.scope)?,
                    encode_set(&code.tenant_scope)?,
                    code.code_challenge,
                    code.secret_hash,
                    code.expires_at.to_rfc3339(),
                    if code.consumed { 1_i64 } else { 0_i64 },
                ],
            )
            .map_err(|err| backend(format!("insert code: {err}")))?;
        Ok(())
    }

    async fn get_code(&self, code_id: &AuthCodeId) -> Result<Option<AuthorizationCode>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT code_id, client_id, user_id, redirect_uri, scope, tenant_scope, \
                 code_challenge, secret_hash, expires_at, consumed FROM oauth_codes WHERE code_id = ?1",
                params![code_id.as_str()],
                decode_code,
            )
            .optional()
            .map_err(|err| backend(format!("get code: {err}")))?
            .transpose()
    }

    async fn consume_code(&self, code_id: &AuthCodeId) -> Result<bool> {
        let connection = self.lock()?;
        let changed = connection
            .execute(
                "UPDATE oauth_codes SET consumed = 1 WHERE code_id = ?1 AND consumed = 0",
                params![code_id.as_str()],
            )
            .map_err(|err| backend(format!("consume code: {err}")))?;
        Ok(changed == 1)
    }

    async fn put_access_token(&self, token: AccessToken) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO oauth_access_tokens
                  (token_id, client_id, user_id, scope, tenant_scope, family_id, secret_hash,
                   expires_at, revoked)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    token.token_id.as_str(),
                    token.client_id.as_str(),
                    token.user_id.as_str(),
                    encode_set(&token.scope)?,
                    encode_set(&token.tenant_scope)?,
                    token.family_id.as_str(),
                    token.secret_hash,
                    token.expires_at.to_rfc3339(),
                    if token.revoked { 1_i64 } else { 0_i64 },
                ],
            )
            .map_err(|err| backend(format!("insert access token: {err}")))?;
        Ok(())
    }

    async fn get_access_token(&self, token_id: &AccessTokenId) -> Result<Option<AccessToken>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT token_id, client_id, user_id, scope, tenant_scope, family_id, secret_hash, \
                 expires_at, revoked FROM oauth_access_tokens WHERE token_id = ?1",
                params![token_id.as_str()],
                decode_access_token,
            )
            .optional()
            .map_err(|err| backend(format!("get access token: {err}")))?
            .transpose()
    }

    async fn revoke_access_token(&self, token_id: &AccessTokenId) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "UPDATE oauth_access_tokens SET revoked = 1 WHERE token_id = ?1",
                params![token_id.as_str()],
            )
            .map_err(|err| backend(format!("revoke access token: {err}")))?;
        Ok(())
    }

    async fn put_refresh_token(&self, token: RefreshToken) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO oauth_refresh_tokens
                  (token_id, client_id, user_id, scope, tenant_scope, family_id, secret_hash,
                   expires_at, revoked)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    token.token_id.as_str(),
                    token.client_id.as_str(),
                    token.user_id.as_str(),
                    encode_set(&token.scope)?,
                    encode_set(&token.tenant_scope)?,
                    token.family_id.as_str(),
                    token.secret_hash,
                    token.expires_at.to_rfc3339(),
                    if token.revoked { 1_i64 } else { 0_i64 },
                ],
            )
            .map_err(|err| backend(format!("insert refresh token: {err}")))?;
        Ok(())
    }

    async fn get_refresh_token(&self, token_id: &RefreshTokenId) -> Result<Option<RefreshToken>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT token_id, client_id, user_id, scope, tenant_scope, family_id, secret_hash, \
                 expires_at, revoked FROM oauth_refresh_tokens WHERE token_id = ?1",
                params![token_id.as_str()],
                decode_refresh_token,
            )
            .optional()
            .map_err(|err| backend(format!("get refresh token: {err}")))?
            .transpose()
    }

    async fn revoke_refresh_token(&self, token_id: &RefreshTokenId) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "UPDATE oauth_refresh_tokens SET revoked = 1 WHERE token_id = ?1",
                params![token_id.as_str()],
            )
            .map_err(|err| backend(format!("revoke refresh token: {err}")))?;
        Ok(())
    }

    async fn revoke_token_family(&self, family_id: &TokenFamilyId) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "UPDATE oauth_access_tokens SET revoked = 1 WHERE family_id = ?1",
                params![family_id.as_str()],
            )
            .map_err(|err| backend(format!("revoke access family: {err}")))?;
        connection
            .execute(
                "UPDATE oauth_refresh_tokens SET revoked = 1 WHERE family_id = ?1",
                params![family_id.as_str()],
            )
            .map_err(|err| backend(format!("revoke refresh family: {err}")))?;
        Ok(())
    }
}

fn decode_client(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<OAuthClient>> {
    let client_id = row.get::<_, String>(0)?;
    let client_secret_hash = row.get::<_, Option<String>>(1)?;
    let client_name = row.get::<_, String>(2)?;
    let redirect_uris = row.get::<_, String>(3)?;
    let grant_types = row.get::<_, String>(4)?;
    let scopes = row.get::<_, String>(5)?;
    let auth_method = row.get::<_, String>(6)?;
    let created_at = row.get::<_, String>(7)?;
    Ok((|| {
        Ok(OAuthClient {
            client_id: OAuthClientId::new(client_id)
                .map_err(|err| backend(format!("decode client id: {err}")))?,
            client_secret_hash,
            client_name,
            redirect_uris: decode_set(&redirect_uris)?,
            grant_types: decode_set(&grant_types)?,
            scopes: decode_set(&scopes)?,
            token_endpoint_auth_method: ClientAuthMethod::parse(&auth_method)?,
            created_at: parse_time(created_at)?,
        })
    })())
}

fn decode_code(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<AuthorizationCode>> {
    let code_id = row.get::<_, String>(0)?;
    let client_id = row.get::<_, String>(1)?;
    let user_id = row.get::<_, String>(2)?;
    let redirect_uri = row.get::<_, String>(3)?;
    let scope = row.get::<_, String>(4)?;
    let tenant_scope = row.get::<_, String>(5)?;
    let code_challenge = row.get::<_, String>(6)?;
    let secret_hash = row.get::<_, String>(7)?;
    let expires_at = row.get::<_, String>(8)?;
    let consumed = row.get::<_, i64>(9)? != 0;
    Ok((|| {
        Ok(AuthorizationCode {
            code_id: AuthCodeId::new(code_id)
                .map_err(|err| backend(format!("decode code id: {err}")))?,
            client_id: OAuthClientId::new(client_id)
                .map_err(|err| backend(format!("decode code client id: {err}")))?,
            user_id: UserId::new(user_id)
                .map_err(|err| backend(format!("decode code user id: {err}")))?,
            redirect_uri,
            scope: decode_set(&scope)?,
            tenant_scope: decode_set(&tenant_scope)?,
            code_challenge,
            secret_hash,
            expires_at: parse_time(expires_at)?,
            consumed,
        })
    })())
}

fn decode_access_token(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<AccessToken>> {
    let token_id = row.get::<_, String>(0)?;
    let client_id = row.get::<_, String>(1)?;
    let user_id = row.get::<_, String>(2)?;
    let scope = row.get::<_, String>(3)?;
    let tenant_scope = row.get::<_, String>(4)?;
    let family_id = row.get::<_, String>(5)?;
    let secret_hash = row.get::<_, String>(6)?;
    let expires_at = row.get::<_, String>(7)?;
    let revoked = row.get::<_, i64>(8)? != 0;
    Ok((|| {
        Ok(AccessToken {
            token_id: AccessTokenId::new(token_id)
                .map_err(|err| backend(format!("decode access token id: {err}")))?,
            client_id: OAuthClientId::new(client_id)
                .map_err(|err| backend(format!("decode access client id: {err}")))?,
            user_id: UserId::new(user_id)
                .map_err(|err| backend(format!("decode access user id: {err}")))?,
            scope: decode_set(&scope)?,
            tenant_scope: decode_set(&tenant_scope)?,
            family_id: TokenFamilyId::new(family_id)
                .map_err(|err| backend(format!("decode access family id: {err}")))?,
            secret_hash,
            expires_at: parse_time(expires_at)?,
            revoked,
        })
    })())
}

fn decode_refresh_token(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<RefreshToken>> {
    let token_id = row.get::<_, String>(0)?;
    let client_id = row.get::<_, String>(1)?;
    let user_id = row.get::<_, String>(2)?;
    let scope = row.get::<_, String>(3)?;
    let tenant_scope = row.get::<_, String>(4)?;
    let family_id = row.get::<_, String>(5)?;
    let secret_hash = row.get::<_, String>(6)?;
    let expires_at = row.get::<_, String>(7)?;
    let revoked = row.get::<_, i64>(8)? != 0;
    Ok((|| {
        Ok(RefreshToken {
            token_id: RefreshTokenId::new(token_id)
                .map_err(|err| backend(format!("decode refresh token id: {err}")))?,
            client_id: OAuthClientId::new(client_id)
                .map_err(|err| backend(format!("decode refresh client id: {err}")))?,
            user_id: UserId::new(user_id)
                .map_err(|err| backend(format!("decode refresh user id: {err}")))?,
            scope: decode_set(&scope)?,
            tenant_scope: decode_set(&tenant_scope)?,
            family_id: TokenFamilyId::new(family_id)
                .map_err(|err| backend(format!("decode refresh family id: {err}")))?,
            secret_hash,
            expires_at: parse_time(expires_at)?,
            revoked,
        })
    })())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        result.unwrap_or_else(|err| panic!("expected Ok, got {err:?}"))
    }

    fn assert_debug_redacts<T: std::fmt::Debug>(label: &str, value: &T, secret_values: &[&str]) {
        let debug = format!("{value:?}");
        assert!(
            debug.contains("<redacted>"),
            "{label} debug output should make redaction explicit: {debug}"
        );
        for secret in secret_values {
            assert!(
                !secret.is_empty(),
                "{label} test secret should not be empty"
            );
            assert!(
                !debug.contains(secret),
                "{label} debug output leaked secret material: {debug}"
            );
        }
    }

    fn store() -> SqliteOAuthStore {
        ok(SqliteOAuthStore::in_memory())
    }

    fn test_tenant_scope() -> TenantScope {
        TenantScope::new(
            ok(beater_core::TenantId::new("demo")),
            ok(beater_core::ProjectId::new("demo")),
            ok(beater_core::EnvironmentId::new("local")),
        )
    }

    // RFC 7636 Appendix B fixture verifier (43 chars, unreserved set).
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    fn challenge_for(verifier: &str) -> String {
        URL_SAFE_NO_PAD.encode(sha256_raw(verifier.as_bytes()))
    }

    fn public_client_registration() -> ClientRegistration {
        ClientRegistration {
            client_name: "mcp-client".to_string(),
            redirect_uris: vec!["https://app.example.com/cb".to_string()],
            grant_types: BTreeSet::from([GrantType::AuthorizationCode, GrantType::RefreshToken]),
            scopes: BTreeSet::from(["traces:read".to_string(), "mcp:invoke".to_string()]),
            token_endpoint_auth_method: ClientAuthMethod::None,
        }
    }

    async fn issue_code(
        store: &SqliteOAuthStore,
        client_id: &OAuthClientId,
        challenge: &str,
        now: Timestamp,
    ) -> String {
        ok(store
            .issue_authorization_code(
                AuthorizationGrant {
                    client_id: client_id.clone(),
                    user_id: ok(UserId::new("user-1")),
                    redirect_uri: "https://app.example.com/cb".to_string(),
                    scope: BTreeSet::from(["traces:read".to_string()]),
                    tenant_scope: test_tenant_scope(),
                    code_challenge: challenge.to_string(),
                },
                now,
            )
            .await)
    }

    #[test]
    fn pkce_s256_roundtrip() {
        assert!(verify_pkce_s256(VERIFIER, &challenge_for(VERIFIER)));
        assert!(!verify_pkce_s256(
            "wrong-verifier",
            &challenge_for(VERIFIER)
        ));
    }

    #[tokio::test]
    async fn register_client_accepts_https_and_loopback_http_redirects() {
        let cases = [
            "https://app.example.com/cb",
            "http://localhost:8765/callback",
            "http://127.0.0.1:8765/callback",
            "http://[::1]:8765/callback",
        ];

        for redirect_uri in cases {
            let store = store();
            let mut registration = public_client_registration();
            registration.redirect_uris = vec![redirect_uri.to_string()];

            let registered = ok(store.register_client(registration, Utc::now()).await);
            assert_eq!(registered.client.redirect_uris, vec![redirect_uri]);
        }
    }

    #[tokio::test]
    async fn register_client_rejects_unsafe_redirect_uris() {
        let cases = [
            "",
            "not a uri",
            "https://app.example.com/cb#fragment",
            "http://app.example.com/cb",
            "ftp://app.example.com/cb",
            "javascript:alert(1)",
        ];

        for redirect_uri in cases {
            let store = store();
            let mut registration = public_client_registration();
            registration.redirect_uris = vec![redirect_uri.to_string()];

            assert!(
                matches!(
                    store.register_client(registration, Utc::now()).await,
                    Err(OAuthError::InvalidRequest(_))
                ),
                "expected {redirect_uri:?} to be rejected"
            );
        }
    }

    #[tokio::test]
    async fn issue_authorization_code_rejects_unsafe_registered_redirect_uri() {
        let store = store();
        let now = Utc::now();
        let client_id = ok(OAuthClientId::new("unsafe-client"));
        ok(store
            .put_client(OAuthClient {
                client_id: client_id.clone(),
                client_secret_hash: None,
                client_name: "legacy-client".to_string(),
                redirect_uris: vec!["http://app.example.com/cb".to_string()],
                grant_types: BTreeSet::from([GrantType::AuthorizationCode]),
                scopes: BTreeSet::from(["traces:read".to_string()]),
                token_endpoint_auth_method: ClientAuthMethod::None,
                created_at: now,
            })
            .await);

        assert!(matches!(
            store
                .issue_authorization_code(
                    AuthorizationGrant {
                        client_id,
                        user_id: ok(UserId::new("user-1")),
                        redirect_uri: "http://app.example.com/cb".to_string(),
                        scope: BTreeSet::from(["traces:read".to_string()]),
                        tenant_scope: test_tenant_scope(),
                        code_challenge: challenge_for(VERIFIER),
                    },
                    now,
                )
                .await,
            Err(OAuthError::InvalidRequest(_))
        ));
    }

    #[tokio::test]
    async fn full_auth_code_pkce_flow() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        assert!(
            registered.client_secret.is_none(),
            "public client has no secret"
        );
        let client_id = registered.client.client_id.clone();

        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;
        let tokens = ok(store
            .exchange_code(
                &client_id,
                None,
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);
        assert_eq!(tokens.token_type, "Bearer");
        assert!(tokens.access_token.starts_with("bao_"));
        assert!(tokens.refresh_token.starts_with("bro_"));

        let claims = ok(store.validate_access_token(&tokens.access_token, now).await);
        assert_eq!(claims.user_id.as_str(), "user-1");
        assert!(claims.scope.contains("traces:read"));
        // The tenant scope chosen at authorize-time rides through to the token.
        assert_eq!(claims.tenant_scope, test_tenant_scope());
    }

    #[tokio::test]
    async fn authorization_code_is_single_use() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;

        ok(store
            .exchange_code(
                &client_id,
                None,
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);
        // Second exchange of the same code is rejected (replay).
        assert!(matches!(
            store
                .exchange_code(
                    &client_id,
                    None,
                    &code,
                    "https://app.example.com/cb",
                    VERIFIER,
                    now
                )
                .await,
            Err(OAuthError::InvalidGrant)
        ));
    }

    #[tokio::test]
    async fn wrong_pkce_verifier_is_rejected() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;

        assert!(matches!(
            store
                .exchange_code(
                    &client_id,
                    None,
                    &code,
                    "https://app.example.com/cb",
                    "attacker",
                    now
                )
                .await,
            Err(OAuthError::InvalidGrant)
        ));
    }

    #[tokio::test]
    async fn redirect_uri_mismatch_is_rejected() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;

        assert!(matches!(
            store
                .exchange_code(
                    &client_id,
                    None,
                    &code,
                    "https://evil.example.com/cb",
                    VERIFIER,
                    now
                )
                .await,
            Err(OAuthError::InvalidGrant)
        ));
    }

    #[tokio::test]
    async fn expired_code_is_rejected() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;

        let later = now + Duration::minutes(11);
        assert!(matches!(
            store
                .exchange_code(
                    &client_id,
                    None,
                    &code,
                    "https://app.example.com/cb",
                    VERIFIER,
                    later
                )
                .await,
            Err(OAuthError::InvalidGrant)
        ));
    }

    #[tokio::test]
    async fn refresh_rotates_and_old_token_is_dead() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;
        let first = ok(store
            .exchange_code(
                &client_id,
                None,
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);

        let rotated = ok(store
            .refresh(&client_id, None, &first.refresh_token, now)
            .await);
        assert_ne!(rotated.refresh_token, first.refresh_token);
        // The rotated access token validates (checked before any reuse, which
        // would trigger family revocation — see the dedicated reuse test).
        let claims = ok(store
            .validate_access_token(&rotated.access_token, now)
            .await);
        assert_eq!(claims.tenant_scope, test_tenant_scope());
        // The old refresh token is now revoked (rotation) — reuse is rejected.
        assert!(matches!(
            store
                .refresh(&client_id, None, &first.refresh_token, now)
                .await,
            Err(OAuthError::InvalidGrant)
        ));
    }

    #[tokio::test]
    async fn reusing_rotated_refresh_token_revokes_the_family() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;
        let first = ok(store
            .exchange_code(
                &client_id,
                None,
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);

        // Legitimate rotation: RT1 -> RT2/AT2.
        let rotated = ok(store
            .refresh(&client_id, None, &first.refresh_token, now)
            .await);
        // AT2 is valid right now.
        ok(store
            .validate_access_token(&rotated.access_token, now)
            .await);

        // Attacker replays the already-rotated RT1: reuse is detected and the
        // whole family is burned (RFC 9700 / OAuth 2.1 §4.13.2).
        assert!(matches!(
            store
                .refresh(&client_id, None, &first.refresh_token, now)
                .await,
            Err(OAuthError::InvalidGrant)
        ));
        // AT2 (a descendant in the family) is now revoked too.
        assert!(matches!(
            store
                .validate_access_token(&rotated.access_token, now)
                .await,
            Err(OAuthError::InvalidToken)
        ));
        // And RT2 can no longer be used.
        assert!(matches!(
            store
                .refresh(&client_id, None, &rotated.refresh_token, now)
                .await,
            Err(OAuthError::InvalidGrant)
        ));
    }

    #[tokio::test]
    async fn confidential_client_secret_is_required() {
        let store = store();
        let now = Utc::now();
        let mut reg = public_client_registration();
        reg.token_endpoint_auth_method = ClientAuthMethod::ClientSecretBasic;
        let registered = ok(store.register_client(reg, now).await);
        let secret = ok(registered.client_secret.clone().ok_or("expected secret"));
        let client_id = registered.client.client_id.clone();

        ok(store.authenticate_client(&client_id, Some(&secret)).await);
        assert!(matches!(
            store.authenticate_client(&client_id, Some("wrong")).await,
            Err(OAuthError::InvalidClient)
        ));
        assert!(matches!(
            store.authenticate_client(&client_id, None).await,
            Err(OAuthError::InvalidClient)
        ));
    }

    #[tokio::test]
    async fn serialized_oauth_records_do_not_leak_secret_hashes() {
        let store = store();
        let now = Utc::now();
        let mut reg = public_client_registration();
        reg.token_endpoint_auth_method = ClientAuthMethod::ClientSecretBasic;
        let registered = ok(store.register_client(reg, now).await);
        let client_secret = ok(registered.client_secret.clone().ok_or("expected secret"));
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;
        let code_id = AuthCodeId::new(
            parse_token(AUTH_CODE_PREFIX, &code)
                .unwrap_or_else(|| panic!("expected minted authorization code"))
                .0
                .to_string(),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let tokens = ok(store
            .exchange_code(
                &client_id,
                Some(&client_secret),
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);
        let access_id = AccessTokenId::new(
            parse_token(ACCESS_TOKEN_PREFIX, &tokens.access_token)
                .unwrap_or_else(|| panic!("expected minted access token"))
                .0
                .to_string(),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let refresh_id = RefreshTokenId::new(
            parse_token(REFRESH_TOKEN_PREFIX, &tokens.refresh_token)
                .unwrap_or_else(|| panic!("expected minted refresh token"))
                .0
                .to_string(),
        )
        .unwrap_or_else(|err| panic!("{err}"));

        let client = ok(store.get_client(&client_id).await)
            .unwrap_or_else(|| panic!("expected registered client"));
        let code_record =
            ok(store.get_code(&code_id).await).unwrap_or_else(|| panic!("expected auth code"));
        let access = ok(store.get_access_token(&access_id).await)
            .unwrap_or_else(|| panic!("expected access token"));
        let refresh = ok(store.get_refresh_token(&refresh_id).await)
            .unwrap_or_else(|| panic!("expected refresh token"));

        let client_json = ok(serde_json::to_string(&client));
        assert!(!client_json.contains("client_secret_hash"));
        assert!(!client_json.contains(&client_secret));
        assert!(!client_json.contains(client.client_secret_hash.as_deref().unwrap_or("")));

        let code_json = ok(serde_json::to_string(&code_record));
        assert!(!code_json.contains("secret_hash"));
        assert!(!code_json.contains(&code));
        assert!(!code_json.contains(&code_record.secret_hash));

        let access_json = ok(serde_json::to_string(&access));
        assert!(!access_json.contains("secret_hash"));
        assert!(!access_json.contains(&tokens.access_token));
        assert!(!access_json.contains(&access.secret_hash));

        let refresh_json = ok(serde_json::to_string(&refresh));
        assert!(!refresh_json.contains("secret_hash"));
        assert!(!refresh_json.contains(&tokens.refresh_token));
        assert!(!refresh_json.contains(&refresh.secret_hash));
    }

    #[tokio::test]
    async fn debug_output_redacts_secret_material() {
        let store = store();
        let now = Utc::now();
        let mut reg = public_client_registration();
        reg.token_endpoint_auth_method = ClientAuthMethod::ClientSecretPost;
        let registered = ok(store.register_client(reg, now).await);
        let client_secret = ok(registered.client_secret.clone().ok_or("expected secret"));
        let client_secret_hash = ok(registered
            .client
            .client_secret_hash
            .clone()
            .ok_or("expected secret hash"));
        let client_id = registered.client.client_id.clone();
        assert_debug_redacts(
            "registered client",
            &registered,
            &[&client_secret, &client_secret_hash],
        );

        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;
        let (code_id_raw, code_secret) = parse_token(AUTH_CODE_PREFIX, &code)
            .unwrap_or_else(|| panic!("expected minted authorization code"));
        let code_id = ok(AuthCodeId::new(code_id_raw));
        let code_record =
            ok(store.get_code(&code_id).await).unwrap_or_else(|| panic!("expected auth code"));
        let code_secret_hash = code_record.secret_hash.clone();
        assert_debug_redacts(
            "authorization code",
            &code_record,
            &[code_secret, &code_secret_hash],
        );

        let tokens = ok(store
            .exchange_code(
                &client_id,
                Some(&client_secret),
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);
        assert_debug_redacts(
            "issued token pair",
            &tokens,
            &[&tokens.access_token, &tokens.refresh_token],
        );

        let (access_id_raw, access_secret) = parse_token(ACCESS_TOKEN_PREFIX, &tokens.access_token)
            .unwrap_or_else(|| panic!("expected minted access token"));
        let access_id = ok(AccessTokenId::new(access_id_raw));
        let access = ok(store.get_access_token(&access_id).await)
            .unwrap_or_else(|| panic!("expected access token"));
        let access_secret_hash = access.secret_hash.clone();
        assert_debug_redacts(
            "access token",
            &access,
            &[access_secret, &access_secret_hash],
        );

        let (refresh_id_raw, refresh_secret) =
            parse_token(REFRESH_TOKEN_PREFIX, &tokens.refresh_token)
                .unwrap_or_else(|| panic!("expected minted refresh token"));
        let refresh_id = ok(RefreshTokenId::new(refresh_id_raw));
        let refresh = ok(store.get_refresh_token(&refresh_id).await)
            .unwrap_or_else(|| panic!("expected refresh token"));
        let refresh_secret_hash = refresh.secret_hash.clone();
        assert_debug_redacts(
            "refresh token",
            &refresh,
            &[refresh_secret, &refresh_secret_hash],
        );
    }

    #[tokio::test]
    async fn revoked_access_token_fails_validation() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        let code = issue_code(&store, &client_id, &challenge_for(VERIFIER), now).await;
        let tokens = ok(store
            .exchange_code(
                &client_id,
                None,
                &code,
                "https://app.example.com/cb",
                VERIFIER,
                now,
            )
            .await);

        let claims = ok(store.validate_access_token(&tokens.access_token, now).await);
        ok(store.revoke_access_token(&claims.token_id).await);
        assert!(matches!(
            store.validate_access_token(&tokens.access_token, now).await,
            Err(OAuthError::InvalidToken)
        ));
    }

    #[tokio::test]
    async fn scope_must_be_subset_of_client() {
        let store = store();
        let now = Utc::now();
        let registered = ok(store
            .register_client(public_client_registration(), now)
            .await);
        let client_id = registered.client.client_id.clone();
        assert!(matches!(
            store
                .issue_authorization_code(
                    AuthorizationGrant {
                        client_id,
                        user_id: ok(UserId::new("user-1")),
                        redirect_uri: "https://app.example.com/cb".to_string(),
                        scope: BTreeSet::from(["admin:everything".to_string()]),
                        tenant_scope: test_tenant_scope(),
                        code_challenge: challenge_for(VERIFIER),
                    },
                    now,
                )
                .await,
            Err(OAuthError::InvalidScope)
        ));
    }
}
