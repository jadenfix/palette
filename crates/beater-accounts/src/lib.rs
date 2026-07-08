//! Accounts: users, password authentication, browser sessions, and org
//! membership.
//!
//! This is the identity layer that sits *above* the existing
//! `(tenant, project, environment)`-scoped API keys in `beater-security` /
//! `beater-auth`. A person registers with an email + password, signs in to get
//! an opaque session token (carried in a dashboard cookie), and belongs to one
//! or more organizations with a role. The OAuth authorization server and the
//! HTTP/dashboard wiring live in later layers; this crate owns only the records,
//! the password/session crypto, and the SQLite store.
//!
//! Crypto choices:
//! - Passwords are hashed with Argon2 (same as API-key secrets in
//!   `beater-security`) — slow on purpose, salted per password.
//! - Session tokens are high-entropy random strings; we store only a SHA-256
//!   hash and compare in constant time. Argon2 would be wasteful per request for
//!   a 256-bit random secret.

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use beater_core::{OrganizationId, SessionId, Timestamp, UserId};
use beater_store::StoreError;
use chrono::{DateTime, Duration, Utc};
use rand_core::{OsRng, RngCore};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use subtle::ConstantTimeEq;
use uuid::Uuid;

/// Default session lifetime: 14 days.
pub fn default_session_ttl() -> Duration {
    Duration::days(14)
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("email is already registered")]
    EmailTaken,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("session token is malformed")]
    MalformedSession,
    #[error("session not found or expired")]
    SessionInvalid,
    #[error("user is inactive")]
    InactiveUser,
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, AccountError>;

fn backend(msg: impl std::fmt::Display) -> AccountError {
    AccountError::Store(StoreError::backend(msg.to_string()))
}

/// Role of a user within an organization. `Owner` is a superset of `Admin`,
/// which is a superset of `Member`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Member,
    Admin,
    Owner,
}

impl OrgRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "member" => Ok(Self::Member),
            "admin" => Ok(Self::Admin),
            "owner" => Ok(Self::Owner),
            other => Err(backend(format!("unknown org role {other}"))),
        }
    }
}

/// A registered user. `password_hash` is the Argon2 PHC string; it is
/// `skip_serializing` so it can never leak into a JSON response/log/event even
/// if a `User` is accidentally serialized. The SQLite store binds columns
/// explicitly (not via serde), so skipping it is safe.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub user_id: UserId,
    pub email: String,
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    pub active: bool,
    pub created_at: Timestamp,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("user_id", &self.user_id)
            .field("email", &self.email)
            .field("password_hash", &"<redacted>")
            .field("active", &self.active)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// A browser session. `secret_hash` is the SHA-256 hex of the random token
/// secret; the plaintext token is shown only once at mint time.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub session_id: SessionId,
    pub user_id: UserId,
    #[serde(skip_serializing, default)]
    pub secret_hash: String,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub last_seen_at: Timestamp,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session_id", &self.session_id)
            .field("user_id", &self.user_id)
            .field("secret_hash", &"<redacted>")
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("last_seen_at", &self.last_seen_at)
            .finish()
    }
}

impl Session {
    pub fn is_expired(&self, now: Timestamp) -> bool {
        now >= self.expires_at
    }
}

/// A freshly minted session plus its one-time plaintext token (`bs_<id>_<secret>`).
#[derive(Clone)]
pub struct MintedSession {
    pub session: Session,
    pub token: String,
}

impl std::fmt::Debug for MintedSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MintedSession")
            .field("session", &self.session)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// Membership of a user in an organization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrgMembership {
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub role: OrgRole,
    pub created_at: Timestamp,
}

/// Normalize an email for uniqueness/lookup: trim + lowercase.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| backend(format!("hash password: {err}")))?
        .to_string())
}

fn verify_password(password_hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// A fixed, valid Argon2 hash used only to equalize `authenticate` timing on the
/// unknown-email path. Computed once; never matches any real password.
fn dummy_password_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY
        .get_or_init(|| {
            hash_password("beater-accounts timing-equalization dummy").unwrap_or_default()
        })
        .as_str()
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing into a String is infallible; avoids a temporary String per byte.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    to_hex(&hasher.finalize())
}

fn is_session_secret(secret: &str) -> bool {
    secret.len() == 64
        && secret
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_hexdigit())
}

/// Mint a new session token for `user_id`. The returned plaintext token is the
/// only time the secret is available; only its hash is persisted.
pub fn mint_session(user_id: UserId, ttl: Duration, now: Timestamp) -> MintedSession {
    // A v4 UUID is always a valid id (non-empty, no whitespace), so this is
    // infallible in practice; `unwrap_or_else` keeps it clippy-clean.
    let session_id = SessionId::new(Uuid::new_v4().to_string())
        .unwrap_or_else(|err| panic!("uuid v4 is a valid session id: {err}"));
    let mut raw = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut raw);
    let secret = to_hex(&raw);
    let token = format!("bs_{}_{}", session_id.as_str(), secret);
    let secret_hash = sha256_hex(secret.as_bytes());
    MintedSession {
        session: Session {
            session_id,
            user_id,
            secret_hash,
            created_at: now,
            expires_at: now + ttl,
            last_seen_at: now,
        },
        token,
    }
}

/// Split a `bs_<session_id>_<secret>` token into its id and secret parts.
pub fn parse_session_token(token: &str) -> Result<(SessionId, &str)> {
    let rest = token
        .strip_prefix("bs_")
        .ok_or(AccountError::MalformedSession)?;
    let (id, secret) = rest.split_once('_').ok_or(AccountError::MalformedSession)?;
    if id.is_empty() || !is_session_secret(secret) {
        return Err(AccountError::MalformedSession);
    }
    let session_id = SessionId::new(id.to_string()).map_err(|_| AccountError::MalformedSession)?;
    Ok((session_id, secret))
}

fn secret_matches(session: &Session, presented_secret: &str) -> bool {
    let presented_hash = sha256_hex(presented_secret.as_bytes());
    presented_hash
        .as_bytes()
        .ct_eq(session.secret_hash.as_bytes())
        .into()
}

/// Store for users, sessions, and org membership. Low-level CRUD lives on the
/// trait; the higher-level [`AccountStore::register`], [`AccountStore::authenticate`],
/// [`AccountStore::start_session`], and [`AccountStore::validate_session`] flows
/// are default methods built on top of it.
#[async_trait::async_trait]
pub trait AccountStore: Send + Sync {
    async fn put_user(&self, user: User) -> Result<()>;
    async fn get_user(&self, user_id: &UserId) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn put_session(&self, session: Session) -> Result<()>;
    async fn get_session(&self, session_id: &SessionId) -> Result<Option<Session>>;
    async fn delete_session(&self, session_id: &SessionId) -> Result<()>;
    async fn touch_session(&self, session_id: &SessionId, last_seen_at: Timestamp) -> Result<()>;
    async fn put_membership(&self, membership: OrgMembership) -> Result<()>;
    async fn get_membership(
        &self,
        organization_id: &OrganizationId,
        user_id: &UserId,
    ) -> Result<Option<OrgMembership>>;
    async fn list_memberships(&self, user_id: &UserId) -> Result<Vec<OrgMembership>>;

    /// Register a new user. Fails with [`AccountError::EmailTaken`] if the
    /// normalized email already exists.
    async fn register(&self, email: &str, password: &str, now: Timestamp) -> Result<User> {
        if self.get_user_by_email(email).await?.is_some() {
            return Err(AccountError::EmailTaken);
        }
        let user = User {
            user_id: UserId::new(Uuid::new_v4().to_string())
                .map_err(|err| backend(format!("mint user id: {err}")))?,
            email: email.trim().to_string(),
            password_hash: hash_password(password)?,
            active: true,
            created_at: now,
        };
        self.put_user(user.clone()).await?;
        Ok(user)
    }

    /// Verify an email + password, returning the user on success.
    ///
    /// Hardened against user enumeration: unknown email, wrong password, and
    /// inactive account all collapse to a uniform [`AccountError::InvalidCredentials`],
    /// and an Argon2 verify runs on every path — including the unknown-email path
    /// (against a fixed dummy hash) — so login timing does not reveal whether an
    /// email exists. The `active` flag is only consulted *after* a correct
    /// password, so a deactivated account is indistinguishable from a wrong
    /// password to an attacker who lacks the password.
    async fn authenticate(&self, email: &str, password: &str) -> Result<User> {
        let maybe_user = self.get_user_by_email(email).await?;
        let password_ok = match &maybe_user {
            Some(user) => verify_password(&user.password_hash, password),
            // Equalize timing: spend the same Argon2 cost as a real verify.
            None => {
                let _ = verify_password(dummy_password_hash(), password);
                false
            }
        };
        match maybe_user {
            Some(user) if password_ok && user.active => Ok(user),
            _ => Err(AccountError::InvalidCredentials),
        }
    }

    /// Mint and persist a session for `user_id`.
    async fn start_session(
        &self,
        user_id: UserId,
        ttl: Duration,
        now: Timestamp,
    ) -> Result<MintedSession> {
        let minted = mint_session(user_id, ttl, now);
        self.put_session(minted.session.clone()).await?;
        Ok(minted)
    }

    /// Validate a presented session token: parse it, look up the session,
    /// constant-time compare the secret, and reject if expired/inactive. On
    /// success returns the user and refreshes `last_seen_at`.
    async fn validate_session(&self, token: &str, now: Timestamp) -> Result<(User, Session)> {
        let (session_id, secret) = parse_session_token(token)?;
        let session = self
            .get_session(&session_id)
            .await?
            .ok_or(AccountError::SessionInvalid)?;
        if !secret_matches(&session, secret) {
            return Err(AccountError::SessionInvalid);
        }
        if session.is_expired(now) {
            // Best-effort cleanup; ignore delete errors.
            let _ = self.delete_session(&session_id).await;
            return Err(AccountError::SessionInvalid);
        }
        let user = self
            .get_user(&session.user_id)
            .await?
            .ok_or(AccountError::SessionInvalid)?;
        if !user.active {
            return Err(AccountError::InactiveUser);
        }
        self.touch_session(&session_id, now).await?;
        Ok((user, session))
    }
}

#[derive(Clone)]
pub struct SqliteAccountStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAccountStore {
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
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS users (
                user_id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                email_key TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                active INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                secret_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);

            CREATE TABLE IF NOT EXISTS org_members (
                organization_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (organization_id, user_id)
            );
            CREATE INDEX IF NOT EXISTS idx_org_members_user ON org_members(user_id);
            "#,
        )?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow::anyhow!("account sqlite mutex poisoned: {err}"))
    }
}

fn parse_time(value: String) -> Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|err| backend(format!("parse timestamp {value}: {err}")))
}

#[async_trait::async_trait]
impl AccountStore for SqliteAccountStore {
    async fn put_user(&self, user: User) -> Result<()> {
        let connection = self.lock()?;
        let email_key = normalize_email(&user.email);
        let inserted = connection.execute(
            r#"
            INSERT INTO users (user_id, email, email_key, password_hash, active, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(user_id) DO UPDATE SET
                email = excluded.email,
                email_key = excluded.email_key,
                password_hash = excluded.password_hash,
                active = excluded.active
            "#,
            params![
                user.user_id.as_str(),
                user.email,
                email_key,
                user.password_hash,
                if user.active { 1_i64 } else { 0_i64 },
                user.created_at.to_rfc3339(),
            ],
        );
        match inserted {
            Ok(_) => Ok(()),
            // Unique violation on email_key -> the email belongs to another user.
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(AccountError::EmailTaken)
            }
            Err(err) => Err(backend(format!("insert user: {err}"))),
        }
    }

    async fn get_user(&self, user_id: &UserId) -> Result<Option<User>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT user_id, email, password_hash, active, created_at FROM users WHERE user_id = ?1",
                params![user_id.as_str()],
                decode_user,
            )
            .optional()
            .map_err(|err| backend(format!("get user: {err}")))?
            .transpose()
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT user_id, email, password_hash, active, created_at FROM users WHERE email_key = ?1",
                params![normalize_email(email)],
                decode_user,
            )
            .optional()
            .map_err(|err| backend(format!("get user by email: {err}")))?
            .transpose()
    }

    async fn put_session(&self, session: Session) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO sessions
                    (session_id, user_id, secret_hash, created_at, expires_at, last_seen_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(session_id) DO UPDATE SET
                    secret_hash = excluded.secret_hash,
                    expires_at = excluded.expires_at,
                    last_seen_at = excluded.last_seen_at
                "#,
                params![
                    session.session_id.as_str(),
                    session.user_id.as_str(),
                    session.secret_hash,
                    session.created_at.to_rfc3339(),
                    session.expires_at.to_rfc3339(),
                    session.last_seen_at.to_rfc3339(),
                ],
            )
            .map_err(|err| backend(format!("insert session: {err}")))?;
        Ok(())
    }

    async fn get_session(&self, session_id: &SessionId) -> Result<Option<Session>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT session_id, user_id, secret_hash, created_at, expires_at, last_seen_at \
                 FROM sessions WHERE session_id = ?1",
                params![session_id.as_str()],
                decode_session,
            )
            .optional()
            .map_err(|err| backend(format!("get session: {err}")))?
            .transpose()
    }

    async fn delete_session(&self, session_id: &SessionId) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "DELETE FROM sessions WHERE session_id = ?1",
                params![session_id.as_str()],
            )
            .map_err(|err| backend(format!("delete session: {err}")))?;
        Ok(())
    }

    async fn touch_session(&self, session_id: &SessionId, last_seen_at: Timestamp) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "UPDATE sessions SET last_seen_at = ?2 WHERE session_id = ?1",
                params![session_id.as_str(), last_seen_at.to_rfc3339()],
            )
            .map_err(|err| backend(format!("touch session: {err}")))?;
        Ok(())
    }

    async fn put_membership(&self, membership: OrgMembership) -> Result<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO org_members (organization_id, user_id, role, created_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(organization_id, user_id) DO UPDATE SET role = excluded.role
                "#,
                params![
                    membership.organization_id.as_str(),
                    membership.user_id.as_str(),
                    membership.role.as_str(),
                    membership.created_at.to_rfc3339(),
                ],
            )
            .map_err(|err| backend(format!("insert membership: {err}")))?;
        Ok(())
    }

    async fn get_membership(
        &self,
        organization_id: &OrganizationId,
        user_id: &UserId,
    ) -> Result<Option<OrgMembership>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT organization_id, user_id, role, created_at FROM org_members \
                 WHERE organization_id = ?1 AND user_id = ?2",
                params![organization_id.as_str(), user_id.as_str()],
                decode_membership,
            )
            .optional()
            .map_err(|err| backend(format!("get membership: {err}")))?
            .transpose()
    }

    async fn list_memberships(&self, user_id: &UserId) -> Result<Vec<OrgMembership>> {
        let connection = self.lock()?;
        let mut stmt = connection
            .prepare(
                "SELECT organization_id, user_id, role, created_at FROM org_members \
                 WHERE user_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|err| backend(format!("prepare list memberships: {err}")))?;
        let rows = stmt
            .query_map(params![user_id.as_str()], decode_membership)
            .map_err(|err| backend(format!("query memberships: {err}")))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| backend(format!("decode membership row: {err}")))??);
        }
        Ok(out)
    }
}

fn decode_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<User>> {
    let user_id = row.get::<_, String>(0)?;
    let email = row.get::<_, String>(1)?;
    let password_hash = row.get::<_, String>(2)?;
    let active = row.get::<_, i64>(3)? != 0;
    let created_at = row.get::<_, String>(4)?;
    Ok((|| {
        Ok(User {
            user_id: UserId::new(user_id)
                .map_err(|err| backend(format!("decode user id: {err}")))?,
            email,
            password_hash,
            active,
            created_at: parse_time(created_at)?,
        })
    })())
}

fn decode_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<Session>> {
    let session_id = row.get::<_, String>(0)?;
    let user_id = row.get::<_, String>(1)?;
    let secret_hash = row.get::<_, String>(2)?;
    let created_at = row.get::<_, String>(3)?;
    let expires_at = row.get::<_, String>(4)?;
    let last_seen_at = row.get::<_, String>(5)?;
    Ok((|| {
        Ok(Session {
            session_id: SessionId::new(session_id)
                .map_err(|err| backend(format!("decode session id: {err}")))?,
            user_id: UserId::new(user_id)
                .map_err(|err| backend(format!("decode session user id: {err}")))?,
            secret_hash,
            created_at: parse_time(created_at)?,
            expires_at: parse_time(expires_at)?,
            last_seen_at: parse_time(last_seen_at)?,
        })
    })())
}

fn decode_membership(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<OrgMembership>> {
    let organization_id = row.get::<_, String>(0)?;
    let user_id = row.get::<_, String>(1)?;
    let role = row.get::<_, String>(2)?;
    let created_at = row.get::<_, String>(3)?;
    Ok((|| {
        Ok(OrgMembership {
            organization_id: OrganizationId::new(organization_id)
                .map_err(|err| backend(format!("decode org id: {err}")))?,
            user_id: UserId::new(user_id)
                .map_err(|err| backend(format!("decode membership user id: {err}")))?,
            role: OrgRole::parse(&role)?,
            created_at: parse_time(created_at)?,
        })
    })())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The workspace denies `unwrap_used`/`expect_used` (clippy), so tests use
    // `ok(..)`/`some(..)` panic helpers and `assert!(matches!(..))` for errors.
    fn ok<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        result.unwrap_or_else(|err| panic!("expected Ok, got {err:?}"))
    }

    fn some<T>(value: Option<T>) -> T {
        value.unwrap_or_else(|| panic!("expected Some, got None"))
    }

    fn store() -> SqliteAccountStore {
        ok(SqliteAccountStore::in_memory())
    }

    #[tokio::test]
    async fn register_then_authenticate() {
        let store = store();
        let now = Utc::now();
        let user = ok(store
            .register("Alice@Example.com", "correct horse battery", now)
            .await);
        assert!(user.active);
        assert_ne!(user.password_hash, "correct horse battery");

        // Case/space-insensitive lookup.
        let authed = ok(store
            .authenticate("  alice@example.com ", "correct horse battery")
            .await);
        assert_eq!(authed.user_id, user.user_id);

        // Wrong password is rejected with the uniform error.
        assert!(matches!(
            store.authenticate("alice@example.com", "wrong").await,
            Err(AccountError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn duplicate_email_is_rejected() {
        let store = store();
        let now = Utc::now();
        ok(store.register("bob@example.com", "pw1", now).await);
        assert!(matches!(
            store.register("BOB@example.com", "pw2", now).await,
            Err(AccountError::EmailTaken)
        ));
    }

    #[tokio::test]
    async fn unknown_email_is_invalid_credentials() {
        let store = store();
        assert!(matches!(
            store.authenticate("nobody@example.com", "pw").await,
            Err(AccountError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn inactive_user_login_is_uniform_error() {
        let store = store();
        let now = Utc::now();
        let user = ok(store.register("frank@example.com", "pw", now).await);
        // Deactivate the account.
        let mut deactivated = user.clone();
        deactivated.active = false;
        ok(store.put_user(deactivated).await);
        // Even with the CORRECT password, an inactive account is indistinguishable
        // from a wrong password: no `InactiveUser` enumeration oracle on login.
        assert!(matches!(
            store.authenticate("frank@example.com", "pw").await,
            Err(AccountError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn session_lifecycle() {
        let store = store();
        let now = Utc::now();
        let user = ok(store.register("carol@example.com", "pw", now).await);

        let minted = ok(store
            .start_session(user.user_id.clone(), default_session_ttl(), now)
            .await);
        assert!(minted.token.starts_with("bs_"));

        let (validated_user, _session) = ok(store.validate_session(&minted.token, now).await);
        assert_eq!(validated_user.user_id, user.user_id);

        // Tampered secret is rejected.
        let mut tampered = minted.token.clone();
        let replacement = if tampered.ends_with('0') { '1' } else { '0' };
        tampered.pop();
        tampered.push(replacement);
        assert!(matches!(
            store.validate_session(&tampered, now).await,
            Err(AccountError::SessionInvalid)
        ));

        // Logout removes it.
        ok(store.delete_session(&minted.session.session_id).await);
        assert!(matches!(
            store.validate_session(&minted.token, now).await,
            Err(AccountError::SessionInvalid)
        ));
    }

    #[tokio::test]
    async fn serialized_user_and_session_do_not_leak_secret_hashes() {
        let store = store();
        let now = Utc::now();
        let user = ok(store.register("privacy@example.com", "pw", now).await);
        let minted = ok(store
            .start_session(user.user_id.clone(), default_session_ttl(), now)
            .await);

        let user_json = ok(serde_json::to_string(&user));
        assert!(!user_json.contains("password_hash"));
        assert!(!user_json.contains(&user.password_hash));

        let session_json = ok(serde_json::to_string(&minted.session));
        assert!(!session_json.contains("secret_hash"));
        assert!(!session_json.contains(&minted.session.secret_hash));
        assert!(!session_json.contains(&minted.token));
    }

    #[tokio::test]
    async fn debug_user_and_session_surfaces_redact_secrets() {
        let store = store();
        let now = Utc::now();
        let user = ok(store.register("debug-privacy@example.com", "pw", now).await);
        let minted = ok(store
            .start_session(user.user_id.clone(), default_session_ttl(), now)
            .await);
        let (_session_id, token_secret) = ok(parse_session_token(&minted.token));

        let user_debug = format!("{user:?}");
        assert!(!user_debug.contains(&user.password_hash));
        assert!(user_debug.contains("<redacted>"));

        let session_debug = format!("{:?}", minted.session);
        assert!(!session_debug.contains(&minted.session.secret_hash));
        assert!(session_debug.contains("<redacted>"));

        let minted_debug = format!("{minted:?}");
        assert!(!minted_debug.contains(&minted.token));
        assert!(!minted_debug.contains(token_secret));
        assert!(!minted_debug.contains(&minted.session.secret_hash));
        assert!(minted_debug.contains("<redacted>"));
    }

    #[tokio::test]
    async fn expired_session_is_rejected_and_cleaned() {
        let store = store();
        let now = Utc::now();
        let user = ok(store.register("dave@example.com", "pw", now).await);
        let minted = ok(store
            .start_session(user.user_id.clone(), Duration::seconds(1), now)
            .await);

        let later = now + Duration::seconds(2);
        assert!(matches!(
            store.validate_session(&minted.token, later).await,
            Err(AccountError::SessionInvalid)
        ));
        // The expired row was cleaned up.
        assert!(ok(store.get_session(&minted.session.session_id).await).is_none());
    }

    #[test]
    fn minted_session_token_parses() {
        let user_id = ok(UserId::new(Uuid::new_v4().to_string()));
        let minted = mint_session(user_id, default_session_ttl(), Utc::now());

        let (session_id, secret) = ok(parse_session_token(&minted.token));

        assert_eq!(session_id, minted.session.session_id);
        assert_eq!(secret.len(), 64);
        assert!(
            secret
                .as_bytes()
                .iter()
                .all(|byte| byte.is_ascii_hexdigit())
        );
        assert_eq!(minted.session.secret_hash, sha256_hex(secret.as_bytes()));
    }

    #[test]
    fn parse_session_token_rejects_bad_secret_format() {
        let id = Uuid::new_v4();
        let short_secret = "a".repeat(63);
        let long_secret = "a".repeat(65);
        let non_hex_secret = format!("{}g", "a".repeat(63));

        for secret in [short_secret, long_secret, non_hex_secret] {
            let token = format!("bs_{id}_{secret}");
            assert!(matches!(
                parse_session_token(&token),
                Err(AccountError::MalformedSession)
            ));
        }
    }

    #[tokio::test]
    async fn malformed_session_tokens() {
        let store = store();
        let now = Utc::now();
        let id = Uuid::new_v4();
        let short_secret = "a".repeat(63);
        let long_secret = "a".repeat(65);
        let non_hex_secret = format!("{}g", "a".repeat(63));
        let bad_tokens = [
            String::new(),
            "nope".to_string(),
            "bs_".to_string(),
            "bs_only".to_string(),
            "bs__secret".to_string(),
            "bs_id_".to_string(),
            format!("bs_{id}_{short_secret}"),
            format!("bs_{id}_{long_secret}"),
            format!("bs_{id}_{non_hex_secret}"),
        ];
        for bad in bad_tokens {
            assert!(matches!(
                store.validate_session(&bad, now).await,
                Err(AccountError::MalformedSession)
            ));
        }
    }

    #[tokio::test]
    async fn org_membership_roundtrip() {
        let store = store();
        let now = Utc::now();
        let user = ok(store.register("erin@example.com", "pw", now).await);
        let org = ok(OrganizationId::new("org-1"));

        ok(store
            .put_membership(OrgMembership {
                organization_id: org.clone(),
                user_id: user.user_id.clone(),
                role: OrgRole::Owner,
                created_at: now,
            })
            .await);

        let got = some(ok(store.get_membership(&org, &user.user_id).await));
        assert_eq!(got.role, OrgRole::Owner);

        // Upsert changes the role.
        ok(store
            .put_membership(OrgMembership {
                organization_id: org.clone(),
                user_id: user.user_id.clone(),
                role: OrgRole::Admin,
                created_at: now,
            })
            .await);

        let memberships = ok(store.list_memberships(&user.user_id).await);
        assert_eq!(memberships.len(), 1);
        assert_eq!(memberships[0].role, OrgRole::Admin);
    }

    #[tokio::test]
    async fn org_membership_list_is_user_scoped() {
        let store = store();
        let now = Utc::now();
        let first_user = ok(store.register("first@example.com", "pw", now).await);
        let second_user = ok(store.register("second@example.com", "pw", now).await);
        let shared_org = ok(OrganizationId::new("shared-org"));
        let first_only_org = ok(OrganizationId::new("first-only-org"));

        ok(store
            .put_membership(OrgMembership {
                organization_id: shared_org.clone(),
                user_id: first_user.user_id.clone(),
                role: OrgRole::Owner,
                created_at: now,
            })
            .await);
        ok(store
            .put_membership(OrgMembership {
                organization_id: first_only_org.clone(),
                user_id: first_user.user_id.clone(),
                role: OrgRole::Admin,
                created_at: now + Duration::seconds(1),
            })
            .await);
        ok(store
            .put_membership(OrgMembership {
                organization_id: shared_org.clone(),
                user_id: second_user.user_id.clone(),
                role: OrgRole::Member,
                created_at: now + Duration::seconds(2),
            })
            .await);

        let first_memberships = ok(store.list_memberships(&first_user.user_id).await);
        assert_eq!(first_memberships.len(), 2);
        assert!(
            first_memberships
                .iter()
                .all(|membership| membership.user_id == first_user.user_id)
        );
        assert_eq!(first_memberships[0].organization_id, shared_org);
        assert_eq!(first_memberships[0].role, OrgRole::Owner);
        assert_eq!(first_memberships[1].organization_id, first_only_org);
        assert_eq!(first_memberships[1].role, OrgRole::Admin);

        let second_memberships = ok(store.list_memberships(&second_user.user_id).await);
        assert_eq!(second_memberships.len(), 1);
        assert_eq!(second_memberships[0].user_id, second_user.user_id);
        assert_eq!(second_memberships[0].organization_id, shared_org);
        assert_eq!(second_memberships[0].role, OrgRole::Member);

        assert!(
            ok(store
                .get_membership(&first_only_org, &second_user.user_id)
                .await)
            .is_none()
        );
    }
}
