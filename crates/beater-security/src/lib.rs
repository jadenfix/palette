use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use beater_core::{ApiKeyId, EnvironmentId, ProjectId, TenantId, Timestamp};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeSet;
use subtle::ConstantTimeEq;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("api key is inactive")]
    InactiveApiKey,
    #[error("api key is malformed")]
    MalformedApiKey,
    #[error("api key scope {0} is missing")]
    MissingScope(String),
    #[error("api key tenant/project/environment mismatch")]
    ScopeMismatch,
    #[error("api key verification failed")]
    ApiKeyVerificationFailed,
    #[error("webhook signature is malformed")]
    MalformedSignature,
    #[error("webhook timestamp is outside replay window")]
    WebhookReplayWindow,
    #[error("webhook signature verification failed")]
    WebhookSignatureFailed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ApiScope {
    TraceWrite,
    TraceRead,
    DatasetWrite,
    EvalRun,
    PiiUnmask,
    Admin,
}

impl ApiScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TraceWrite => "trace:write",
            Self::TraceRead => "trace:read",
            Self::DatasetWrite => "dataset:write",
            Self::EvalRun => "eval:run",
            Self::PiiUnmask => "pii:unmask",
            Self::Admin => "admin",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub api_key_id: ApiKeyId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub secret_hash: String,
    pub scopes: BTreeSet<ApiScope>,
    pub active: bool,
    pub created_at: Timestamp,
    pub rotated_at: Option<Timestamp>,
    pub last_used_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedApiKey {
    pub record: ApiKeyRecord,
    pub secret: String,
}

pub fn create_api_key(
    tenant_id: TenantId,
    project_id: ProjectId,
    environment_id: EnvironmentId,
    scopes: BTreeSet<ApiScope>,
) -> Result<CreatedApiKey, SecurityError> {
    let api_key_id =
        ApiKeyId::new(Uuid::new_v4().to_string()).map_err(|err| anyhow::anyhow!(err))?;
    let secret = format!("bt_{}_{}", api_key_id.as_str(), Uuid::new_v4().simple());
    let salt = SaltString::generate(&mut OsRng);
    let secret_hash = Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|err| SecurityError::Other(anyhow::anyhow!(err.to_string())))?
        .to_string();
    Ok(CreatedApiKey {
        record: ApiKeyRecord {
            api_key_id,
            tenant_id,
            project_id,
            environment_id,
            secret_hash,
            scopes,
            active: true,
            created_at: Utc::now(),
            rotated_at: None,
            last_used_at: None,
        },
        secret,
    })
}

pub fn api_key_id_from_secret(secret: &str) -> Result<ApiKeyId, SecurityError> {
    let Some(rest) = secret.strip_prefix("bt_") else {
        return Err(SecurityError::MalformedApiKey);
    };
    let Some((api_key_id, token_secret)) = rest.split_once('_') else {
        return Err(SecurityError::MalformedApiKey);
    };
    if api_key_id.is_empty() || token_secret.is_empty() {
        return Err(SecurityError::MalformedApiKey);
    }
    ApiKeyId::new(api_key_id.to_string()).map_err(|_| SecurityError::MalformedApiKey)
}

pub fn verify_api_key(
    record: &ApiKeyRecord,
    presented_secret: &str,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    environment_id: &EnvironmentId,
    required_scope: ApiScope,
) -> Result<(), SecurityError> {
    if !record.active {
        return Err(SecurityError::InactiveApiKey);
    }
    if &record.tenant_id != tenant_id
        || &record.project_id != project_id
        || &record.environment_id != environment_id
    {
        return Err(SecurityError::ScopeMismatch);
    }
    if !record.scopes.contains(&required_scope) && !record.scopes.contains(&ApiScope::Admin) {
        return Err(SecurityError::MissingScope(
            required_scope.as_str().to_string(),
        ));
    }
    let parsed_hash = PasswordHash::new(&record.secret_hash)
        .map_err(|err| SecurityError::Other(anyhow::anyhow!(err.to_string())))?;
    Argon2::default()
        .verify_password(presented_secret.as_bytes(), &parsed_hash)
        .map_err(|_| SecurityError::ApiKeyVerificationFailed)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookSignature {
    pub timestamp: i64,
    pub signature: String,
}

impl WebhookSignature {
    pub fn header_value(&self) -> String {
        format!("t={},v1={}", self.timestamp, self.signature)
    }
}

pub fn sign_webhook(
    signing_secret: &[u8],
    body: &[u8],
    timestamp: Timestamp,
) -> Result<WebhookSignature, SecurityError> {
    let payload = webhook_payload(timestamp.timestamp(), body);
    let mut mac = HmacSha256::new_from_slice(signing_secret)
        .map_err(|err| SecurityError::Other(anyhow::anyhow!(err.to_string())))?;
    mac.update(&payload);
    let signature = STANDARD_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(WebhookSignature {
        timestamp: timestamp.timestamp(),
        signature,
    })
}

pub fn verify_webhook(
    signing_secret: &[u8],
    body: &[u8],
    header: &str,
    now: Timestamp,
    tolerance: Duration,
) -> Result<(), SecurityError> {
    let parsed = parse_webhook_header(header)?;
    let signed_at = chrono::DateTime::from_timestamp(parsed.timestamp, 0)
        .ok_or(SecurityError::MalformedSignature)?;
    let age = now.signed_duration_since(signed_at).num_seconds().abs();
    if age > tolerance.num_seconds() {
        return Err(SecurityError::WebhookReplayWindow);
    }
    let expected = sign_webhook(signing_secret, body, signed_at)?;
    if expected
        .signature
        .as_bytes()
        .ct_eq(parsed.signature.as_bytes())
        .unwrap_u8()
        == 1
    {
        Ok(())
    } else {
        Err(SecurityError::WebhookSignatureFailed)
    }
}

fn parse_webhook_header(header: &str) -> Result<WebhookSignature, SecurityError> {
    let mut timestamp = None;
    let mut signature = None;
    for part in header.split(',') {
        let Some((key, value)) = part.split_once('=') else {
            return Err(SecurityError::MalformedSignature);
        };
        match key {
            "t" => {
                timestamp = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| SecurityError::MalformedSignature)?,
                );
            }
            "v1" => signature = Some(value.to_string()),
            _ => {}
        }
    }
    Ok(WebhookSignature {
        timestamp: timestamp.ok_or(SecurityError::MalformedSignature)?,
        signature: signature.ok_or(SecurityError::MalformedSignature)?,
    })
}

fn webhook_payload(timestamp: i64, body: &[u8]) -> Vec<u8> {
    let mut payload = timestamp.to_string().into_bytes();
    payload.push(b'.');
    payload.extend_from_slice(body);
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_keys_are_hashed_scoped_and_rotatable() {
        let mut scopes = BTreeSet::new();
        scopes.insert(ApiScope::TraceWrite);
        let created = create_api_key(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            scopes,
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_ne!(created.record.secret_hash, created.secret);
        assert_eq!(
            api_key_id_from_secret(&created.secret).unwrap_or_else(|err| panic!("{err}")),
            created.record.api_key_id
        );
        verify_api_key(
            &created.record,
            &created.secret,
            &TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            &ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            &EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            ApiScope::TraceWrite,
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert!(matches!(
            verify_api_key(
                &created.record,
                &created.secret,
                &TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                &ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                &EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
                ApiScope::PiiUnmask,
            ),
            Err(SecurityError::MissingScope(scope)) if scope == "pii:unmask"
        ));
    }

    #[test]
    fn webhooks_are_signed_and_replay_protected() {
        let body = br#"{"event":"trace.alert"}"#;
        let now = Utc::now();
        let signature = sign_webhook(b"secret", body, now).unwrap_or_else(|err| panic!("{err}"));
        let header = signature.header_value();

        verify_webhook(b"secret", body, &header, now, Duration::minutes(5))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(matches!(
            verify_webhook(b"other", body, &header, now, Duration::minutes(5)),
            Err(SecurityError::WebhookSignatureFailed)
        ));
        assert!(matches!(
            verify_webhook(
                b"secret",
                body,
                &header,
                now + Duration::minutes(10),
                Duration::minutes(5)
            ),
            Err(SecurityError::WebhookReplayWindow)
        ));
    }
}
