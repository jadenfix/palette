use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

pub type Timestamp = DateTime<Utc>;

pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        DateTime::<Utc>::from(SystemTime::now())
    }
}

#[derive(Clone, Debug)]
pub struct FixedClock {
    now: Timestamp,
}

impl FixedClock {
    pub fn new(now: Timestamp) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("identifier cannot be empty")]
    Empty,
    #[error("identifier contains whitespace: {0}")]
    Whitespace(String),
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Clone,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            utoipa::ToSchema,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                if value.is_empty() {
                    return Err(IdError::Empty);
                }
                if value.chars().any(char::is_whitespace) {
                    return Err(IdError::Whitespace(value));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IdError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

id_type!(TenantId);
id_type!(OrganizationId);
id_type!(UserId);
id_type!(SessionId);
id_type!(ProjectId);
id_type!(EnvironmentId);
id_type!(AgentId);
id_type!(AgentReleaseId);
id_type!(RunId);
id_type!(TraceId);
id_type!(SpanId);
id_type!(ArtifactId);
id_type!(DatasetId);
id_type!(DatasetVersionId);
id_type!(DatasetCaseId);
id_type!(ExperimentId);
id_type!(ExperimentRunId);
id_type!(EvaluatorId);
id_type!(EvaluatorVersionId);
id_type!(EvalResultId);
id_type!(GateId);
id_type!(GateRunId);
id_type!(ReviewQueueId);
id_type!(ReviewTaskId);
id_type!(AnnotationId);
id_type!(CalibrationReportId);
id_type!(PromptId);
id_type!(PromptVersionId);
id_type!(ApiKeyId);
id_type!(ProviderSecretId);
id_type!(JudgeCallId);
id_type!(UsageRecordId);
id_type!(AuditEventId);
id_type!(WebhookEndpointId);
id_type!(IdempotencyKey);
id_type!(Sha256Hash);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TenantScope {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
}

impl TenantScope {
    pub fn new(tenant_id: TenantId, project_id: ProjectId, environment_id: EnvironmentId) -> Self {
        Self {
            tenant_id,
            project_id,
            environment_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum Currency {
    #[serde(rename = "USD")]
    Usd,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Usd => "USD",
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum MoneyError {
    #[error("currency mismatch: {left} != {right}")]
    CurrencyMismatch { left: Currency, right: Currency },
    #[error("money amount overflow")]
    Overflow,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Money {
    pub amount_micros: i64,
    pub currency: Currency,
}

impl Money {
    pub fn new(amount_micros: i64, currency: Currency) -> Self {
        Self {
            amount_micros,
            currency,
        }
    }

    pub fn usd_micros(amount_micros: i64) -> Self {
        Self::new(amount_micros, Currency::Usd)
    }

    pub fn try_add(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;
        let amount_micros = self
            .amount_micros
            .checked_add(other.amount_micros)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::new(amount_micros, self.currency))
    }

    pub fn try_sub(&self, other: &Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;
        let amount_micros = self
            .amount_micros
            .checked_sub(other.amount_micros)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self::new(amount_micros, self.currency))
    }

    fn ensure_same_currency(&self, other: &Self) -> Result<(), MoneyError> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            })
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TokenCounts {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
}

impl TokenCounts {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.reasoning
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    lower_hex(digest.as_ref())
}

#[derive(Debug, thiserror::Error)]
pub enum JsonHashError {
    #[error("serialize json for hash: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("sha256 digest was not a valid hash identifier: {0}")]
    HashId(#[from] IdError),
}

pub fn sha256_json_hash<T: Serialize>(value: &T) -> Result<Sha256Hash, JsonHashError> {
    let bytes = serde_json::to_vec(value)?;
    Sha256Hash::new(sha256_hex(&bytes)).map_err(JsonHashError::from)
}

pub fn lower_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PageRequest {
    pub limit: u32,
    pub cursor: Option<String>,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            limit: 100,
            cursor: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<String>) -> Self {
        Self { items, next_cursor }
    }
}

/// Self-host telemetry posture (R12.5).
///
/// Beater is opt-out: a self-hosted `beaterd` reports **no** anonymous usage
/// telemetry to Beater Cloud unless the operator explicitly opts in. This type
/// is the single source of truth for the disabled-by-default posture and the
/// fixed endpoint. Operators opt in via the `--self-host-telemetry` flag (env
/// `BEATER_SELF_HOST_TELEMETRY`); until then no outbound endpoint is exposed, so
/// an offline self-host can firewall it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelfHostTelemetryConfig {
    enabled: bool,
}

impl SelfHostTelemetryConfig {
    /// The env var operators set to opt **in** to anonymous self-host telemetry.
    pub const ENV_VAR: &'static str = "BEATER_SELF_HOST_TELEMETRY";

    /// The only destination self-host telemetry is ever sent to, and only when
    /// it is explicitly enabled. Kept here so offline deployments can block it.
    pub const ENDPOINT: &'static str = "https://telemetry.beater.dev/v1/usage";

    /// Construct an explicit posture. The binary resolves `enabled` from the
    /// `--self-host-telemetry` flag (which reads `ENV_VAR` via clap), so opt-in
    /// parsing lives in one place rather than being duplicated here.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Whether anonymous self-host telemetry reporting is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The destination usage telemetry is reported to, or `None` when disabled.
    /// Self-host installs that never opt in therefore make no outbound call.
    pub fn endpoint(&self) -> Option<&'static str> {
        if self.enabled {
            Some(Self::ENDPOINT)
        } else {
            None
        }
    }
}

impl Default for SelfHostTelemetryConfig {
    /// Opt-out by default: self-host telemetry is disabled.
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_reject_empty_and_whitespace() {
        assert_eq!(TenantId::new(""), Err(IdError::Empty));
        assert!(matches!(
            TenantId::new("tenant one"),
            Err(IdError::Whitespace(value)) if value == "tenant one"
        ));
        assert_eq!(
            TenantId::new("tenant-one").map(|id| id.to_string()),
            Ok("tenant-one".to_string())
        );
    }

    #[test]
    fn token_total_excludes_cache_read() {
        let counts = TokenCounts {
            input: 10,
            output: 20,
            reasoning: 5,
            cache_read: 100,
        };
        assert_eq!(counts.total(), 35);
    }

    #[test]
    fn json_hash_uses_stable_serde_json_bytes() {
        let counts = TokenCounts {
            input: 1,
            output: 2,
            reasoning: 3,
            cache_read: 4,
        };
        let hash = sha256_json_hash(&counts).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            hash.as_str(),
            "e078d77c009b2f97482cb3e09d6230236a802e5c28feebc7ad7ce6634434cf13"
        );
    }

    #[test]
    fn money_math_requires_matching_currency() {
        let left = Money::usd_micros(100);
        let right = Money::usd_micros(40);

        assert_eq!(left.try_sub(&right), Ok(Money::usd_micros(60)));
        assert_eq!(right.try_add(&left), Ok(Money::usd_micros(140)));
        assert_eq!(
            serde_json::to_value(Money::usd_micros(1)).unwrap_or_else(|err| panic!("{err}"))
                ["currency"],
            "USD"
        );
    }

    #[test]
    fn self_host_telemetry_defaults_to_opt_out() {
        // The whole point of R12.5: self-host telemetry is OFF unless opted in,
        // so an un-configured self-host has no outbound telemetry target.
        let default = SelfHostTelemetryConfig::default();
        assert!(!default.is_enabled());
        assert_eq!(default.endpoint(), None);

        // Opting in is the only thing that exposes the endpoint.
        let opted_in = SelfHostTelemetryConfig::new(true);
        assert!(opted_in.is_enabled());
        assert_eq!(opted_in.endpoint(), Some(SelfHostTelemetryConfig::ENDPOINT));
    }
}
