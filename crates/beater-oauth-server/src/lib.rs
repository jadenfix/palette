//! OAuth 2.1 authorization-server HTTP surface.
//!
//! A self-contained axum [`Router`] (the same pattern as `beater-mcp`) that
//! exposes the OAuth endpoints and is merged into `beaterd` alongside the API
//! and MCP routers. It deliberately lives OUTSIDE the `/v1` OpenAPI contract —
//! these are root-level OAuth/discovery routes, not `/v1` operations.
//!
//! Endpoints:
//! - `GET /.well-known/oauth-authorization-server` — RFC 8414 AS metadata.
//! - `GET /.well-known/oauth-protected-resource` — RFC 9728 resource metadata
//!   (consumed by the MCP resource server).
//! - `POST /oauth/register` — RFC 7591 dynamic client registration.
//! - `GET /oauth/authorize` — authorization endpoint; requires a logged-in user
//!   (accounts session cookie) and issues a PKCE-bound code.
//! - `POST /oauth/token` — token endpoint (authorization_code + refresh_token).
//!
//! Login UI is owned by the dashboard: an unauthenticated `/oauth/authorize`
//! redirects to `login_url` with a `return_to` back to itself.

use std::sync::Arc;

use axum::extract::{Form, OriginalUri, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use beater_accounts::{default_session_ttl, AccountError, AccountStore, OrgMembership, OrgRole};
use beater_auth::{ApiKeyStore, CreateApiKeyRequest};
use beater_core::{
    ApiKeyId, EnvironmentId, OrganizationId, ProjectId, TenantId, TenantScope, UserId,
};
use beater_oauth::{
    validate_redirect_uri, AuthorizationGrant, ClientAuthMethod, ClientRegistration, GrantType,
    IssuedTokens, OAuthError, OAuthStore,
};
use beater_security::ApiScope;
use chrono::Utc;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;

/// Name of the dashboard session cookie carrying the accounts `bs_<id>_<secret>`
/// token used to identify the logged-in user at `/oauth/authorize`.
pub const SESSION_COOKIE: &str = "beater_session";

#[derive(Clone)]
pub struct OAuthServerState {
    pub oauth: Arc<dyn OAuthStore>,
    pub accounts: Arc<dyn AccountStore>,
    /// Absolute issuer URL, no trailing slash, e.g. `https://api.example.com`.
    pub issuer: String,
    /// Dashboard login page. When set, an unauthenticated `/oauth/authorize`
    /// redirects here with `?return_to=<authorize-url>`.
    pub login_url: Option<String>,
    /// Scopes advertised in metadata.
    pub scopes_supported: Vec<String>,
    /// API-key store, used by the session-authorized `/auth/api-keys` endpoints
    /// so a logged-in user can mint/revoke keys for their own tenant. `None`
    /// when the backend runs without strict auth (no key store).
    pub api_keys: Option<Arc<dyn ApiKeyStore>>,
}

impl OAuthServerState {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.issuer.trim_end_matches('/'), path)
    }
}

/// Build the OAuth HTTP router. Mirrors `beater_mcp::router`: self-contained,
/// resolves its own state, ready to `.merge()` into the main server.
pub fn router(state: OAuthServerState) -> Router {
    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource_metadata),
        )
        .route("/oauth/register", post(register))
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/token", post(token))
        // Account auth endpoints used by the dashboard to sign users in/out.
        .route("/auth/register", post(auth_register))
        .route("/auth/login", post(auth_login))
        .route("/auth/logout", post(auth_logout))
        .route("/auth/me", get(auth_me))
        // Session-authorized API-key management for the logged-in user's tenant.
        .route("/auth/api-keys", post(auth_create_api_key))
        .route("/auth/api-keys/revoke", post(auth_revoke_api_key))
        .with_state(state)
}

// ---- session-authorized API keys ----

#[derive(Debug, Deserialize)]
struct CreateApiKeyBody {
    /// Scope names (`trace_read`, `trace_write`, `dataset_write`, `eval_run`,
    /// `pii_unmask`, `admin`).
    scopes: Vec<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    environment_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateApiKeyResult {
    api_key_id: String,
    /// The plaintext key (`bt_...`), shown exactly once.
    secret: String,
    tenant_id: String,
    project_id: String,
    environment_id: String,
    scopes: Vec<String>,
}

fn parse_api_scope(value: &str) -> Option<ApiScope> {
    match value {
        "trace_write" => Some(ApiScope::TraceWrite),
        "trace_read" => Some(ApiScope::TraceRead),
        "dataset_write" => Some(ApiScope::DatasetWrite),
        "eval_run" => Some(ApiScope::EvalRun),
        "pii_unmask" => Some(ApiScope::PiiUnmask),
        "admin" => Some(ApiScope::Admin),
        _ => None,
    }
}

async fn auth_create_api_key(
    State(state): State<OAuthServerState>,
    headers: HeaderMap,
    Json(body): Json<CreateApiKeyBody>,
) -> Response {
    let Some(user_id) = session_user(&state, &headers).await else {
        return oauth_error(StatusCode::UNAUTHORIZED, "not_authenticated", None);
    };
    let Some(api_keys) = state.api_keys.as_ref() else {
        return oauth_error(StatusCode::NOT_IMPLEMENTED, "api_keys_unavailable", None);
    };
    if body.scopes.is_empty() {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            Some("at least one scope is required"),
        );
    }
    let mut scopes = BTreeSet::new();
    for raw in &body.scopes {
        match parse_api_scope(raw) {
            Some(scope) => {
                scopes.insert(scope);
            }
            None => {
                return oauth_error(StatusCode::BAD_REQUEST, "invalid_scope", Some(raw));
            }
        }
    }
    // The key is scoped to the user's personal tenant (== user id). Project /
    // environment default to "default" when omitted.
    let project = body.project_id.unwrap_or_else(|| "default".to_string());
    let environment = body.environment_id.unwrap_or_else(|| "default".to_string());
    let (Ok(tenant_id), Ok(project_id), Ok(environment_id)) = (
        TenantId::new(user_id.as_str().to_string()),
        ProjectId::new(project.clone()),
        EnvironmentId::new(environment.clone()),
    ) else {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_request", None);
    };
    let scope_names: Vec<String> = scopes.iter().map(|s| s.as_str().to_string()).collect();
    match api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            environment_id: environment_id.clone(),
            scopes,
        })
        .await
    {
        Ok(created) => Json(CreateApiKeyResult {
            api_key_id: created.record.api_key_id.as_str().to_string(),
            secret: created.secret,
            tenant_id: tenant_id.as_str().to_string(),
            project_id: project_id.as_str().to_string(),
            environment_id: environment_id.as_str().to_string(),
            scopes: scope_names,
        })
        .into_response(),
        Err(_) => oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    }
}

#[derive(Debug, Deserialize)]
struct RevokeApiKeyBody {
    api_key_id: String,
}

async fn auth_revoke_api_key(
    State(state): State<OAuthServerState>,
    headers: HeaderMap,
    Json(body): Json<RevokeApiKeyBody>,
) -> Response {
    let Some(user_id) = session_user(&state, &headers).await else {
        return oauth_error(StatusCode::UNAUTHORIZED, "not_authenticated", None);
    };
    let Some(api_keys) = state.api_keys.as_ref() else {
        return oauth_error(StatusCode::NOT_IMPLEMENTED, "api_keys_unavailable", None);
    };
    let Ok(api_key_id) = ApiKeyId::new(body.api_key_id) else {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_request", None);
    };
    // Only let a user revoke keys in their OWN tenant.
    match api_keys.get_key(api_key_id.clone()).await {
        Ok(Some(record)) if record.tenant_id.as_str() == user_id.as_str() => {}
        Ok(Some(_)) => return oauth_error(StatusCode::FORBIDDEN, "forbidden", None),
        Ok(None) => return oauth_error(StatusCode::NOT_FOUND, "not_found", None),
        Err(_) => return oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    }
    match api_keys.revoke_key(api_key_id, Utc::now()).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    }
}

// ---- account auth (dashboard login/session) ----

#[derive(Debug, Deserialize)]
struct CredentialsRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AccountResponse {
    user_id: String,
    email: String,
    /// The user's personal tenant (== their org id), usable as the
    /// tenant/project scope when authorizing OAuth clients.
    tenant_id: String,
}

fn account_response(user_id: &str, email: &str) -> AccountResponse {
    AccountResponse {
        user_id: user_id.to_string(),
        email: email.to_string(),
        tenant_id: user_id.to_string(),
    }
}

async fn auth_register(
    State(state): State<OAuthServerState>,
    Json(req): Json<CredentialsRequest>,
) -> Response {
    if req.email.trim().is_empty() || req.password.len() < 8 {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            Some("email is required and password must be >= 8 chars"),
        );
    }
    let now = Utc::now();
    let user = match state
        .accounts
        .register(&req.email, &req.password, now)
        .await
    {
        Ok(user) => user,
        Err(AccountError::EmailTaken) => {
            return oauth_error(StatusCode::CONFLICT, "email_taken", None)
        }
        Err(_) => return oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    };
    // Provision a personal organization (org id == user id == tenant) and make
    // the user its owner, so they can immediately authorize for their tenant.
    let org_id = match OrganizationId::new(user.user_id.as_str().to_string()) {
        Ok(org) => org,
        Err(_) => return oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    };
    if state
        .accounts
        .put_membership(OrgMembership {
            organization_id: org_id,
            user_id: user.user_id.clone(),
            role: OrgRole::Owner,
            created_at: now,
        })
        .await
        .is_err()
    {
        return oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None);
    }
    issue_session_response(&state, &user.user_id, &user.email, now).await
}

async fn auth_login(
    State(state): State<OAuthServerState>,
    Json(req): Json<CredentialsRequest>,
) -> Response {
    let now = Utc::now();
    match state.accounts.authenticate(&req.email, &req.password).await {
        Ok(user) => issue_session_response(&state, &user.user_id, &user.email, now).await,
        Err(AccountError::InvalidCredentials) | Err(AccountError::InactiveUser) => {
            oauth_error(StatusCode::UNAUTHORIZED, "invalid_credentials", None)
        }
        Err(_) => oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    }
}

async fn auth_logout(State(state): State<OAuthServerState>, headers: HeaderMap) -> Response {
    // Best-effort: if the cookie maps to a live session, delete it.
    if let Some(token) = session_cookie(&headers) {
        if let Ok((_user, session)) = state.accounts.validate_session(&token, Utc::now()).await {
            let _ = state.accounts.delete_session(&session.session_id).await;
        }
    }
    let mut resp = StatusCode::NO_CONTENT.into_response();
    if let Ok(value) = clear_session_cookie(&state).parse() {
        resp.headers_mut().insert(http::header::SET_COOKIE, value);
    }
    resp
}

async fn auth_me(State(state): State<OAuthServerState>, headers: HeaderMap) -> Response {
    match session_user_full(&state, &headers).await {
        Some((user_id, email)) => Json(account_response(&user_id, &email)).into_response(),
        None => oauth_error(StatusCode::UNAUTHORIZED, "not_authenticated", None),
    }
}

async fn issue_session_response(
    state: &OAuthServerState,
    user_id: &UserId,
    email: &str,
    now: chrono::DateTime<Utc>,
) -> Response {
    let minted = match state
        .accounts
        .start_session(user_id.clone(), default_session_ttl(), now)
        .await
    {
        Ok(minted) => minted,
        Err(_) => return oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    };
    let mut resp = Json(account_response(user_id.as_str(), email)).into_response();
    if let Ok(value) = set_session_cookie(state, &minted.token).parse() {
        resp.headers_mut().insert(http::header::SET_COOKIE, value);
    }
    resp
}

/// Whether to mark the session cookie `Secure` (HTTPS issuers only).
fn cookie_secure(state: &OAuthServerState) -> bool {
    state.issuer.starts_with("https://")
}

fn set_session_cookie(state: &OAuthServerState, token: &str) -> String {
    let secure = if cookie_secure(state) { "; Secure" } else { "" };
    let max_age = default_session_ttl().num_seconds().max(0);
    format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax{secure}; Max-Age={max_age}")
}

fn clear_session_cookie(state: &OAuthServerState) -> String {
    let secure = if cookie_secure(state) { "; Secure" } else { "" };
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax{secure}; Max-Age=0")
}

// ---- metadata ----

async fn authorization_server_metadata(
    State(state): State<OAuthServerState>,
) -> Json<serde_json::Value> {
    Json(json!({
        "issuer": state.issuer,
        "authorization_endpoint": state.url("/oauth/authorize"),
        "token_endpoint": state.url("/oauth/token"),
        "registration_endpoint": state.url("/oauth/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": [
            "none", "client_secret_basic", "client_secret_post"
        ],
        "scopes_supported": state.scopes_supported,
    }))
}

async fn protected_resource_metadata(
    State(state): State<OAuthServerState>,
) -> Json<serde_json::Value> {
    Json(json!({
        "resource": state.issuer,
        "authorization_servers": [state.issuer],
        "scopes_supported": state.scopes_supported,
        "bearer_methods_supported": ["header"],
    }))
}

// ---- dynamic client registration ----

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    client_name: Option<String>,
    redirect_uris: Vec<String>,
    #[serde(default)]
    grant_types: Vec<String>,
    token_endpoint_auth_method: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    client_name: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    token_endpoint_auth_method: String,
    scope: String,
}

async fn register(
    State(state): State<OAuthServerState>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    let auth_method = match req.token_endpoint_auth_method.as_deref() {
        None => ClientAuthMethod::None,
        Some(m) => match parse_auth_method(m) {
            Some(parsed) => parsed,
            None => return oauth_error(StatusCode::BAD_REQUEST, "invalid_client_metadata", None),
        },
    };
    let grant_types = if req.grant_types.is_empty() {
        BTreeSet::from([GrantType::AuthorizationCode, GrantType::RefreshToken])
    } else {
        let mut set = BTreeSet::new();
        for g in &req.grant_types {
            match parse_grant_type(g) {
                Some(parsed) => {
                    set.insert(parsed);
                }
                None => {
                    return oauth_error(StatusCode::BAD_REQUEST, "invalid_client_metadata", None)
                }
            }
        }
        set
    };
    let scopes = parse_scope(req.scope.as_deref());
    if let Some(scope) = unsupported_scope(&scopes, &state.scopes_supported) {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_scope", Some(scope));
    }
    let registration = ClientRegistration {
        client_name: req
            .client_name
            .unwrap_or_else(|| "beater-client".to_string()),
        redirect_uris: req.redirect_uris,
        grant_types,
        scopes,
        token_endpoint_auth_method: auth_method,
    };
    match state.oauth.register_client(registration, Utc::now()).await {
        Ok(registered) => {
            let client = &registered.client;
            Json(RegisterResponse {
                client_id: client.client_id.as_str().to_string(),
                client_secret: registered.client_secret.clone(),
                client_name: client.client_name.clone(),
                redirect_uris: client.redirect_uris.clone(),
                grant_types: client
                    .grant_types
                    .iter()
                    .map(|g| g.as_str().to_string())
                    .collect(),
                token_endpoint_auth_method: client.token_endpoint_auth_method.as_str().to_string(),
                scope: client
                    .scopes
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(" "),
            })
            .into_response()
        }
        Err(err) => oauth_error_from(err),
    }
}

// ---- authorize ----

#[derive(Debug, Deserialize)]
struct AuthorizeParams {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    state: Option<String>,
    code_challenge: String,
    #[serde(default)]
    code_challenge_method: Option<String>,
    /// Tenant/project/environment the user is authorizing the token for. The
    /// user must be a member of the tenant's organization.
    tenant_id: String,
    project_id: String,
    environment_id: String,
}

async fn authorize(
    State(state): State<OAuthServerState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    Query(params): Query<AuthorizeParams>,
) -> Response {
    let client_id = match beater_core::OAuthClientId::new(params.client_id.clone()) {
        Ok(id) => id,
        Err(_) => {
            return oauth_error(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                Some("invalid client_id"),
            )
        }
    };
    // Resolve + validate the client and redirect_uri BEFORE we ever redirect, so
    // we never bounce a code/error to an unregistered URI (open-redirect guard).
    let client = match state.oauth.get_client(&client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => {
            return oauth_error(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                Some("unknown client"),
            )
        }
        Err(err) => return oauth_error_from(err),
    };
    if !client.allows_redirect(&params.redirect_uri) {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            Some("redirect_uri not registered"),
        );
    }
    if let Err(err) = validate_redirect_uri(&params.redirect_uri) {
        return oauth_error_from(err);
    }

    // From here, recoverable errors are returned to the client via redirect.
    if params.response_type != "code" {
        return redirect_error(
            &params.redirect_uri,
            "unsupported_response_type",
            params.state.as_deref(),
        );
    }
    if params.code_challenge_method.as_deref().unwrap_or("S256") != "S256" {
        return redirect_error(
            &params.redirect_uri,
            "invalid_request",
            params.state.as_deref(),
        );
    }

    // Identify the logged-in user from the accounts session cookie.
    let user_id = match session_user(&state, &headers).await {
        Some(user_id) => user_id,
        None => {
            // Not logged in: bounce to the dashboard login with a return_to.
            return match &state.login_url {
                Some(login) => {
                    let return_to = state.url(
                        uri.path_and_query()
                            .map(|pq| pq.as_str())
                            .unwrap_or("/oauth/authorize"),
                    );
                    let sep = if login.contains('?') { '&' } else { '?' };
                    Redirect::to(&format!(
                        "{login}{sep}return_to={}",
                        utf8_percent_encode(&return_to, NON_ALPHANUMERIC)
                    ))
                    .into_response()
                }
                None => oauth_error(StatusCode::UNAUTHORIZED, "login_required", None),
            };
        }
    };

    // The user picks the tenant/project/environment to authorize for. Validate
    // the ids and enforce that the user is a MEMBER of the tenant's org — this
    // is the privilege-escalation guard: a logged-in user cannot mint a token
    // for a tenant they don't belong to.
    let (Ok(tenant_id), Ok(project_id), Ok(environment_id)) = (
        TenantId::new(params.tenant_id.clone()),
        ProjectId::new(params.project_id.clone()),
        EnvironmentId::new(params.environment_id.clone()),
    ) else {
        return redirect_error(
            &params.redirect_uri,
            "invalid_request",
            params.state.as_deref(),
        );
    };
    let org_id = match OrganizationId::new(params.tenant_id.clone()) {
        Ok(org) => org,
        Err(_) => {
            return redirect_error(
                &params.redirect_uri,
                "invalid_request",
                params.state.as_deref(),
            )
        }
    };
    match state.accounts.get_membership(&org_id, &user_id).await {
        Ok(Some(_membership)) => {}
        Ok(None) => {
            return redirect_error(
                &params.redirect_uri,
                "access_denied",
                params.state.as_deref(),
            )
        }
        Err(_) => return oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None),
    }
    let tenant_scope = TenantScope::new(tenant_id, project_id, environment_id);

    // Default the scope to the client's full registered set when omitted.
    let requested = parse_scope(params.scope.as_deref());
    let scope = if requested.is_empty() {
        client.scopes.clone()
    } else {
        requested
    };
    if unsupported_scope(&scope, &state.scopes_supported).is_some() {
        return redirect_error(
            &params.redirect_uri,
            "invalid_scope",
            params.state.as_deref(),
        );
    }

    let grant = AuthorizationGrant {
        client_id,
        user_id,
        redirect_uri: params.redirect_uri.clone(),
        scope,
        tenant_scope,
        code_challenge: params.code_challenge.clone(),
    };
    match state
        .oauth
        .issue_authorization_code(grant, Utc::now())
        .await
    {
        Ok(code) => {
            let mut url = format!(
                "{}?code={}",
                params.redirect_uri,
                utf8_percent_encode(&code, NON_ALPHANUMERIC)
            );
            if let Some(s) = &params.state {
                url.push_str(&format!(
                    "&state={}",
                    utf8_percent_encode(s, NON_ALPHANUMERIC)
                ));
            }
            Redirect::to(&url).into_response()
        }
        Err(OAuthError::InvalidScope) => redirect_error(
            &params.redirect_uri,
            "invalid_scope",
            params.state.as_deref(),
        ),
        Err(OAuthError::InvalidRequest(_)) => redirect_error(
            &params.redirect_uri,
            "invalid_request",
            params.state.as_deref(),
        ),
        Err(err) => oauth_error_from(err),
    }
}

// ---- token ----

#[derive(Debug, Deserialize)]
struct TokenForm {
    grant_type: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    redirect_uri: Option<String>,
    #[serde(default)]
    code_verifier: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    client_secret: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: i64,
    refresh_token: String,
    scope: String,
}

async fn token(
    State(state): State<OAuthServerState>,
    headers: HeaderMap,
    Form(form): Form<TokenForm>,
) -> Response {
    // Client credentials may arrive via HTTP Basic (client_secret_basic) or in
    // the body (client_secret_post). Basic takes precedence when present.
    let (client_id_str, client_secret) = match basic_auth(&headers) {
        Some((id, secret)) => (Some(id), Some(secret)),
        None => (form.client_id.clone(), form.client_secret.clone()),
    };
    let Some(client_id_str) = client_id_str else {
        return oauth_error(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            Some("client_id is required"),
        );
    };
    let client_id = match beater_core::OAuthClientId::new(client_id_str) {
        Ok(id) => id,
        Err(_) => return oauth_error(StatusCode::UNAUTHORIZED, "invalid_client", None),
    };
    let now = Utc::now();

    let result: Result<IssuedTokens, OAuthError> = match form.grant_type.as_str() {
        "authorization_code" => {
            let (Some(code), Some(redirect_uri), Some(verifier)) =
                (&form.code, &form.redirect_uri, &form.code_verifier)
            else {
                return oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    Some("code, redirect_uri, code_verifier are required"),
                );
            };
            state
                .oauth
                .exchange_code(
                    &client_id,
                    client_secret.as_deref(),
                    code,
                    redirect_uri,
                    verifier,
                    now,
                )
                .await
        }
        "refresh_token" => {
            let Some(refresh) = &form.refresh_token else {
                return oauth_error(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    Some("refresh_token is required"),
                );
            };
            state
                .oauth
                .refresh(&client_id, client_secret.as_deref(), refresh, now)
                .await
        }
        _ => return oauth_error(StatusCode::BAD_REQUEST, "unsupported_grant_type", None),
    };

    match result {
        Ok(tokens) => Json(TokenResponse {
            access_token: tokens.access_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
            refresh_token: tokens.refresh_token,
            scope: tokens
                .scope
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" "),
        })
        .into_response(),
        Err(err) => oauth_error_from(err),
    }
}

// ---- helpers ----

async fn session_user(state: &OAuthServerState, headers: &HeaderMap) -> Option<UserId> {
    let token = session_cookie(headers)?;
    match state.accounts.validate_session(&token, Utc::now()).await {
        Ok((user, _session)) => Some(user.user_id),
        Err(_) => None,
    }
}

/// Resolve the logged-in user's id + email from the session cookie.
async fn session_user_full(
    state: &OAuthServerState,
    headers: &HeaderMap,
) -> Option<(String, String)> {
    let token = session_cookie(headers)?;
    match state.accounts.validate_session(&token, Utc::now()).await {
        Ok((user, _session)) => Some((user.user_id.as_str().to_string(), user.email)),
        Err(_) => None,
    }
}

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(http::header::COOKIE)?.to_str().ok()?;
    for pair in raw.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn basic_auth(headers: &HeaderMap) -> Option<(String, String)> {
    let raw = headers.get(http::header::AUTHORIZATION)?.to_str().ok()?;
    let encoded = raw.strip_prefix("Basic ")?;
    let decoded = BASE64_STANDARD.decode(encoded).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (id, secret) = decoded.split_once(':')?;
    Some((id.to_string(), secret.to_string()))
}

fn parse_scope(scope: Option<&str>) -> BTreeSet<String> {
    scope
        .unwrap_or("")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn unsupported_scope<'a>(
    scopes: &'a BTreeSet<String>,
    scopes_supported: &[String],
) -> Option<&'a str> {
    let supported: BTreeSet<&str> = scopes_supported.iter().map(String::as_str).collect();
    scopes
        .iter()
        .find(|scope| !supported.contains(scope.as_str()))
        .map(String::as_str)
}

fn parse_grant_type(value: &str) -> Option<GrantType> {
    match value {
        "authorization_code" => Some(GrantType::AuthorizationCode),
        "refresh_token" => Some(GrantType::RefreshToken),
        _ => None,
    }
}

fn parse_auth_method(value: &str) -> Option<ClientAuthMethod> {
    match value {
        "none" => Some(ClientAuthMethod::None),
        "client_secret_basic" => Some(ClientAuthMethod::ClientSecretBasic),
        "client_secret_post" => Some(ClientAuthMethod::ClientSecretPost),
        _ => None,
    }
}

fn oauth_error(status: StatusCode, error: &str, description: Option<&str>) -> Response {
    let mut body = json!({ "error": error });
    if let Some(desc) = description {
        body["error_description"] = json!(desc);
    }
    (status, Json(body)).into_response()
}

fn oauth_error_from(err: OAuthError) -> Response {
    match err {
        OAuthError::InvalidClient => oauth_error(StatusCode::UNAUTHORIZED, "invalid_client", None),
        OAuthError::InvalidToken => oauth_error(StatusCode::UNAUTHORIZED, "invalid_token", None),
        OAuthError::InvalidGrant => oauth_error(StatusCode::BAD_REQUEST, "invalid_grant", None),
        OAuthError::InvalidScope => oauth_error(StatusCode::BAD_REQUEST, "invalid_scope", None),
        OAuthError::UnauthorizedClient => {
            oauth_error(StatusCode::BAD_REQUEST, "unauthorized_client", None)
        }
        OAuthError::InvalidRequest(msg) => {
            oauth_error(StatusCode::BAD_REQUEST, "invalid_request", Some(&msg))
        }
        OAuthError::Store(_) | OAuthError::Other(_) => {
            oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None)
        }
    }
}

fn redirect_error(redirect_uri: &str, error: &str, state: Option<&str>) -> Response {
    // `error` is always a controlled OAuth error-code constant (safe token), so
    // it is written verbatim; only the client-supplied `state` is encoded.
    let mut url = format!("{redirect_uri}?error={error}");
    if let Some(s) = state {
        url.push_str(&format!(
            "&state={}",
            utf8_percent_encode(s, NON_ALPHANUMERIC)
        ));
    }
    Redirect::to(&url).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use beater_accounts::SqliteAccountStore;
    use beater_oauth::{OAuthClient, SqliteOAuthStore};
    use http::header::LOCATION;
    use http::header::SET_COOKIE;
    use tower::ServiceExt;

    fn ok<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        result.unwrap_or_else(|err| panic!("expected Ok, got {err:?}"))
    }

    fn test_state() -> OAuthServerState {
        OAuthServerState {
            oauth: Arc::new(ok(SqliteOAuthStore::in_memory())),
            accounts: Arc::new(ok(SqliteAccountStore::in_memory())),
            issuer: "https://api.example.test".to_string(),
            login_url: Some("https://app.example.test/login".to_string()),
            scopes_supported: vec!["traces:read".to_string(), "mcp:invoke".to_string()],
            api_keys: Some(Arc::new(ok(beater_auth::SqliteApiKeyStore::in_memory()))),
        }
    }

    async fn body_json(resp: Response) -> serde_json::Value {
        let bytes = ok(axum::body::to_bytes(resp.into_body(), 1 << 20).await);
        ok(serde_json::from_slice(&bytes))
    }

    // RFC 7636 fixture verifier + its S256 challenge.
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    fn challenge() -> String {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        URL_SAFE_NO_PAD.encode(sha2_256(VERIFIER.as_bytes()))
    }
    fn sha2_256(input: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.finalize().into()
    }

    async fn post_json(
        app: &Router,
        uri: &str,
        body: serde_json::Value,
        cookie: Option<&str>,
    ) -> Response {
        let mut builder = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(c) = cookie {
            builder = builder.header(http::header::COOKIE, c);
        }
        ok(app
            .clone()
            .oneshot(
                builder
                    .body(Body::from(body.to_string()))
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await)
    }

    fn cookie_token(resp: &Response) -> String {
        let set = resp
            .headers()
            .get(SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        set.split(';')
            .next()
            .and_then(|kv| kv.strip_prefix(&format!("{SESSION_COOKIE}=")))
            .unwrap_or("")
            .to_string()
    }

    #[tokio::test]
    async fn auth_register_login_me_logout_flow() {
        let app = router(test_state());

        // Register -> 200, sets session cookie, personal tenant == user id.
        let resp = post_json(
            &app,
            "/auth/register",
            json!({"email": "dev@example.test", "password": "supersecret"}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let token = cookie_token(&resp);
        assert!(token.starts_with("bs_"), "expected session cookie");
        let body = body_json(resp).await;
        let user_id = body["user_id"].as_str().unwrap_or("").to_string();
        assert_eq!(body["tenant_id"], user_id);

        // /auth/me with the cookie returns the same user.
        let cookie = format!("{SESSION_COOKIE}={token}");
        let resp = ok(app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header(http::header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(body_json(resp).await["user_id"], user_id);

        // Duplicate register -> 409.
        let resp = post_json(
            &app,
            "/auth/register",
            json!({"email": "DEV@example.test", "password": "supersecret"}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        // Wrong password -> 401.
        let resp = post_json(
            &app,
            "/auth/login",
            json!({"email": "dev@example.test", "password": "wrong"}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Correct login -> 200 + fresh cookie.
        let resp = post_json(
            &app,
            "/auth/login",
            json!({"email": "dev@example.test", "password": "supersecret"}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(cookie_token(&resp).starts_with("bs_"));

        // Logout -> 204 + clears cookie; the old session no longer validates.
        let resp = post_json(&app, "/auth/logout", json!({}), Some(&cookie)).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let resp = ok(app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header(http::header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn session_user_mints_and_revokes_api_key_for_own_tenant() {
        let app = router(test_state());
        // Register to get a session cookie.
        let resp = post_json(
            &app,
            "/auth/register",
            json!({"email": "dev@example.test", "password": "supersecret"}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let token = cookie_token(&resp);
        let body = body_json(resp).await;
        let user_id = body["user_id"].as_str().unwrap_or("").to_string();
        let cookie = format!("{SESSION_COOKIE}={token}");

        // Mint a key for the user's own tenant.
        let resp = post_json(
            &app,
            "/auth/api-keys",
            json!({"scopes": ["trace_read", "trace_write"]}),
            Some(&cookie),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let key = body_json(resp).await;
        assert!(key["secret"].as_str().unwrap_or("").starts_with("bt_"));
        assert_eq!(key["tenant_id"], user_id); // scoped to the user's tenant
        let api_key_id = key["api_key_id"].as_str().unwrap_or("").to_string();

        // Unauthenticated mint -> 401.
        let resp = post_json(
            &app,
            "/auth/api-keys",
            json!({"scopes": ["trace_read"]}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Invalid scope -> 400.
        let resp = post_json(
            &app,
            "/auth/api-keys",
            json!({"scopes": ["wat"]}),
            Some(&cookie),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Revoke own key -> 204.
        let resp = post_json(
            &app,
            "/auth/api-keys/revoke",
            json!({"api_key_id": api_key_id}),
            Some(&cookie),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn session_user_cannot_revoke_api_key_from_another_tenant() {
        let state = test_state();
        let api_keys = state
            .api_keys
            .as_ref()
            .unwrap_or_else(|| panic!("missing api key store"))
            .clone();
        let foreign_key = ok(api_keys
            .create_key(CreateApiKeyRequest {
                tenant_id: ok(TenantId::new("other-user")),
                project_id: ok(ProjectId::new("default")),
                environment_id: ok(EnvironmentId::new("default")),
                scopes: BTreeSet::from([ApiScope::TraceRead]),
            })
            .await);
        let app = router(state);

        let resp = post_json(
            &app,
            "/auth/register",
            json!({"email": "dev@example.test", "password": "supersecret"}),
            None,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let cookie = format!("{SESSION_COOKIE}={}", cookie_token(&resp));

        let resp = post_json(
            &app,
            "/auth/api-keys/revoke",
            json!({"api_key_id": foreign_key.record.api_key_id.as_str()}),
            Some(&cookie),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let loaded = ok(api_keys
            .get_key(foreign_key.record.api_key_id.clone())
            .await)
        .unwrap_or_else(|| panic!("expected foreign key to remain stored"));
        assert!(loaded.active, "foreign tenant key must not be revoked");
        assert_eq!(loaded.tenant_id, foreign_key.record.tenant_id);
    }

    #[tokio::test]
    async fn authorization_server_metadata_advertises_s256_and_endpoints() {
        let app = router(test_state());
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/oauth-authorization-server")
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["issuer"], "https://api.example.test");
        assert_eq!(
            body["token_endpoint"],
            "https://api.example.test/oauth/token"
        );
        assert_eq!(body["code_challenge_methods_supported"][0], "S256");
    }

    #[tokio::test]
    async fn register_returns_public_client_without_secret() {
        let app = router(test_state());
        let req_body = json!({
            "client_name": "mcp",
            "redirect_uris": ["https://app.example.test/cb"],
            "token_endpoint_auth_method": "none",
            "scope": "traces:read mcp:invoke"
        });
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(req_body.to_string()))
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["client_id"].as_str().is_some());
        assert_eq!(body["scope"], "mcp:invoke traces:read");
        assert!(
            body.get("client_secret").is_none(),
            "public client has no secret"
        );
    }

    #[tokio::test]
    async fn register_rejects_unsupported_scope() {
        let app = router(test_state());
        let req_body = json!({
            "client_name": "mcp",
            "redirect_uris": ["https://app.example.test/cb"],
            "token_endpoint_auth_method": "none",
            "scope": "traces:read traces:delete"
        });
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(req_body.to_string()))
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = body_json(resp).await;
        assert_eq!(body["error"], "invalid_scope");
        assert_eq!(body["error_description"], "traces:delete");
    }

    #[tokio::test]
    async fn register_rejects_unsafe_redirect_uris() {
        let app = router(test_state());
        let cases = [
            "",
            "not a uri",
            "https://app.example.test/cb#fragment",
            "http://app.example.test/cb",
            "ftp://app.example.test/cb",
        ];

        for redirect_uri in cases {
            let resp = post_json(
                &app,
                "/oauth/register",
                json!({
                    "client_name": "mcp",
                    "redirect_uris": [redirect_uri],
                    "token_endpoint_auth_method": "none",
                    "scope": "traces:read"
                }),
                None,
            )
            .await;
            assert_eq!(
                resp.status(),
                StatusCode::BAD_REQUEST,
                "expected {redirect_uri:?} to be rejected"
            );
        }
    }

    #[tokio::test]
    async fn authorize_rejects_unsafe_registered_redirect_uri_without_redirecting() {
        let state = test_state();
        let now = Utc::now();
        let client_id = ok(beater_core::OAuthClientId::new("unsafe-client"));
        ok(state
            .oauth
            .put_client(OAuthClient {
                client_id: client_id.clone(),
                client_secret_hash: None,
                client_name: "legacy-client".to_string(),
                redirect_uris: vec!["http://app.example.test/cb".to_string()],
                grant_types: BTreeSet::from([GrantType::AuthorizationCode]),
                scopes: BTreeSet::from(["traces:read".to_string()]),
                token_endpoint_auth_method: ClientAuthMethod::None,
                created_at: now,
            })
            .await);

        let app = router(state);
        let uri = format!(
            "/oauth/authorize?response_type=token&client_id={client_id}&redirect_uri={}&tenant_id=demo&project_id=demo&environment_id=local&code_challenge={}&code_challenge_method=S256",
            utf8_percent_encode("http://app.example.test/cb", NON_ALPHANUMERIC),
            challenge()
        );
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert!(
            resp.headers().get(LOCATION).is_none(),
            "must not redirect to an unsafe registered URI"
        );
        let body = body_json(resp).await;
        assert_eq!(body["error"], "invalid_request");
    }

    #[tokio::test]
    async fn authorize_without_session_redirects_to_login() {
        let state = test_state();
        // Register a client directly via the store.
        let registered = ok(state
            .oauth
            .register_client(
                ClientRegistration {
                    client_name: "mcp".to_string(),
                    redirect_uris: vec!["https://app.example.test/cb".to_string()],
                    grant_types: BTreeSet::from([GrantType::AuthorizationCode]),
                    scopes: BTreeSet::from(["traces:read".to_string()]),
                    token_endpoint_auth_method: ClientAuthMethod::None,
                },
                Utc::now(),
            )
            .await);
        let client_id = registered.client.client_id.as_str().to_string();
        let app = router(state);
        let uri = format!(
            "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={}&tenant_id=demo&project_id=demo&environment_id=local&code_challenge={}&code_challenge_method=S256",
            utf8_percent_encode("https://app.example.test/cb", NON_ALPHANUMERIC),
            challenge()
        );
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let loc = resp
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            loc.starts_with("https://app.example.test/login"),
            "got {loc}"
        );
        assert!(loc.contains("return_to="));
    }

    #[tokio::test]
    async fn authorize_denies_non_member_of_tenant() {
        let state = test_state();
        let now = Utc::now();
        // Logged-in user, but NO membership in the "demo" org/tenant.
        let user = ok(state
            .accounts
            .register("outsider@example.test", "pw", now)
            .await);
        let session = ok(state
            .accounts
            .start_session(user.user_id.clone(), default_session_ttl(), now)
            .await);
        let registered = ok(state
            .oauth
            .register_client(
                ClientRegistration {
                    client_name: "mcp".to_string(),
                    redirect_uris: vec!["https://app.example.test/cb".to_string()],
                    grant_types: BTreeSet::from([GrantType::AuthorizationCode]),
                    scopes: BTreeSet::from(["traces:read".to_string()]),
                    token_endpoint_auth_method: ClientAuthMethod::None,
                },
                now,
            )
            .await);
        let client_id = registered.client.client_id.as_str().to_string();
        let app = router(state);
        let uri = format!(
            "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={}&tenant_id=demo&project_id=demo&environment_id=local&code_challenge={}&code_challenge_method=S256",
            utf8_percent_encode("https://app.example.test/cb", NON_ALPHANUMERIC),
            challenge()
        );
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(
                        http::header::COOKIE,
                        format!("{SESSION_COOKIE}={}", session.token),
                    )
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        // Redirected back to the client with error=access_denied (no code).
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let loc = resp
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(loc.contains("error=access_denied"), "got {loc}");
        assert!(!loc.contains("code="), "must not issue a code: {loc}");
    }

    #[tokio::test]
    async fn authorize_rejects_client_with_unsupported_scope() {
        let state = test_state();
        let now = Utc::now();
        let user = ok(state.accounts.register("dev@example.test", "pw", now).await);
        let session = ok(state
            .accounts
            .start_session(user.user_id.clone(), default_session_ttl(), now)
            .await);
        ok(state
            .accounts
            .put_membership(beater_accounts::OrgMembership {
                organization_id: ok(OrganizationId::new("demo")),
                user_id: user.user_id.clone(),
                role: beater_accounts::OrgRole::Member,
                created_at: now,
            })
            .await);
        // Seed the store directly to simulate a client that predates the
        // HTTP registration allowlist or was inserted by an admin tool.
        let registered = ok(state
            .oauth
            .register_client(
                ClientRegistration {
                    client_name: "mcp".to_string(),
                    redirect_uris: vec!["https://app.example.test/cb".to_string()],
                    grant_types: BTreeSet::from([GrantType::AuthorizationCode]),
                    scopes: BTreeSet::from([
                        "traces:read".to_string(),
                        "traces:delete".to_string(),
                    ]),
                    token_endpoint_auth_method: ClientAuthMethod::None,
                },
                now,
            )
            .await);
        let client_id = registered.client.client_id.as_str().to_string();
        let app = router(state);
        let uri = format!(
            "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={}&tenant_id=demo&project_id=demo&environment_id=local&code_challenge={}&code_challenge_method=S256",
            utf8_percent_encode("https://app.example.test/cb", NON_ALPHANUMERIC),
            challenge()
        );
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(
                        http::header::COOKIE,
                        format!("{SESSION_COOKIE}={}", session.token),
                    )
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let loc = resp
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(loc.contains("error=invalid_scope"), "got {loc}");
        assert!(!loc.contains("code="), "must not issue a code: {loc}");
    }

    #[tokio::test]
    async fn full_authorize_then_token_flow() {
        let state = test_state();
        // A logged-in user + session.
        let now = Utc::now();
        let user = ok(state.accounts.register("dev@example.test", "pw", now).await);
        let session = ok(state
            .accounts
            .start_session(user.user_id.clone(), default_session_ttl(), now)
            .await);
        // The user must be a member of the tenant's org to authorize for it.
        ok(state
            .accounts
            .put_membership(beater_accounts::OrgMembership {
                organization_id: ok(OrganizationId::new("demo")),
                user_id: user.user_id.clone(),
                role: beater_accounts::OrgRole::Member,
                created_at: now,
            })
            .await);
        // A public client.
        let registered = ok(state
            .oauth
            .register_client(
                ClientRegistration {
                    client_name: "mcp".to_string(),
                    redirect_uris: vec!["https://app.example.test/cb".to_string()],
                    grant_types: BTreeSet::from([
                        GrantType::AuthorizationCode,
                        GrantType::RefreshToken,
                    ]),
                    scopes: BTreeSet::from(["traces:read".to_string()]),
                    token_endpoint_auth_method: ClientAuthMethod::None,
                },
                now,
            )
            .await);
        let client_id = registered.client.client_id.as_str().to_string();
        let app = router(state);

        // GET /oauth/authorize with the session cookie -> 303 to redirect_uri?code=...
        let uri = format!(
            "/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={}&scope=traces%3Aread&state=xyz&tenant_id=demo&project_id=demo&environment_id=local&code_challenge={}&code_challenge_method=S256",
            utf8_percent_encode("https://app.example.test/cb", NON_ALPHANUMERIC),
            challenge()
        );
        let resp = ok(app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header(
                        http::header::COOKIE,
                        format!("{SESSION_COOKIE}={}", session.token),
                    )
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let loc = resp
            .headers()
            .get(LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            loc.starts_with("https://app.example.test/cb?code="),
            "got {loc}"
        );
        assert!(loc.contains("state=xyz"));
        // Extract the code.
        let code = loc
            .split("code=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .unwrap_or("");
        let code = percent_encoding::percent_decode_str(code)
            .decode_utf8_lossy()
            .to_string();

        // POST /oauth/token (authorization_code) -> access + refresh tokens.
        let form = format!(
            "grant_type=authorization_code&client_id={client_id}&code={}&redirect_uri={}&code_verifier={VERIFIER}",
            utf8_percent_encode(&code, NON_ALPHANUMERIC),
            utf8_percent_encode("https://app.example.test/cb", NON_ALPHANUMERIC),
        );
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(form))
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["token_type"], "Bearer");
        assert!(body["access_token"]
            .as_str()
            .unwrap_or("")
            .starts_with("bao_"));
        assert!(body["refresh_token"]
            .as_str()
            .unwrap_or("")
            .starts_with("bro_"));
        assert_eq!(body["scope"], "traces:read");
    }

    #[tokio::test]
    async fn token_rejects_bad_code_with_invalid_grant() {
        let state = test_state();
        let registered = ok(state
            .oauth
            .register_client(
                ClientRegistration {
                    client_name: "mcp".to_string(),
                    redirect_uris: vec!["https://app.example.test/cb".to_string()],
                    grant_types: BTreeSet::from([GrantType::AuthorizationCode]),
                    scopes: BTreeSet::from(["traces:read".to_string()]),
                    token_endpoint_auth_method: ClientAuthMethod::None,
                },
                Utc::now(),
            )
            .await);
        let client_id = registered.client.client_id.as_str().to_string();
        let app = router(state);
        let form = format!(
            "grant_type=authorization_code&client_id={client_id}&code=bac_nope_nope&redirect_uri={}&code_verifier={VERIFIER}",
            utf8_percent_encode("https://app.example.test/cb", NON_ALPHANUMERIC),
        );
        let resp = ok(app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(form))
                    .unwrap_or_else(|e| panic!("{e}")),
            )
            .await);
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = body_json(resp).await;
        assert_eq!(body["error"], "invalid_grant");
    }
}
