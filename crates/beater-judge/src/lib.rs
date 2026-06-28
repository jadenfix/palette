use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    JudgeCallId, Money, ProjectId, ProviderSecretId, Sha256Hash, TenantId, Timestamp,
};
use beater_eval::{
    EvaluationCase, EvaluatorKind, EvaluatorSpec, JudgeRequest, JudgeResponse, ScoreResult,
};
use beater_schema::EvaluatorLane;
use beater_secrets::ProviderSecretStore;
use beater_store::{StoreError, StoreResult};
use chrono::{DateTime, Utc};
use reqwest::StatusCode as ReqwestStatusCode;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type JudgeProviderResult<T> = Result<T, JudgeProviderError>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum JudgeProviderError {
    #[error("judge provider error: {0}")]
    Backend(String),
}

impl JudgeProviderError {
    pub fn backend(error: impl std::fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProviderCredentials {
    provider: String,
    secret_value: String,
}

impl ProviderCredentials {
    pub fn new(provider: impl Into<String>, secret_value: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            secret_value: secret_value.into(),
        }
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn secret_value(&self) -> &str {
        &self.secret_value
    }
}

impl Debug for ProviderCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderCredentials")
            .field("provider", &self.provider)
            .field("secret_value", &"<redacted>")
            .finish()
    }
}

#[async_trait]
pub trait JudgeProvider: Send + Sync {
    fn max_cost(&self, provider: &str, request: &JudgeRequest) -> Money;

    async fn judge(
        &self,
        request: JudgeRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<JudgeResponse>;
}

#[async_trait]
impl<T> JudgeProvider for Arc<T>
where
    T: JudgeProvider + ?Sized,
{
    fn max_cost(&self, provider: &str, request: &JudgeRequest) -> Money {
        (**self).max_cost(provider, request)
    }

    async fn judge(
        &self,
        request: JudgeRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<JudgeResponse> {
        (**self).judge(request, credentials).await
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeBrokerRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub evaluator: EvaluatorSpec,
    pub case: EvaluationCase,
    pub provider_secret_id: ProviderSecretId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JudgeBrokerOutcome {
    pub result: ScoreResult,
    pub audit: JudgeAuditRecord,
    pub remaining_budget: Money,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JudgeAuditRecord {
    pub judge_call_id: JudgeCallId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub evaluator_id: String,
    pub provider: String,
    pub provider_secret_id: ProviderSecretId,
    pub model: String,
    pub request_hash: Sha256Hash,
    pub response_hash: Sha256Hash,
    pub score: f64,
    pub provider_cost: Money,
    pub charged_cost: Money,
    pub cached: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CachedJudgeResponse {
    pub response: JudgeResponse,
    pub response_hash: Sha256Hash,
    pub provider_cost: Money,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeAuditInsert {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub evaluator_id: String,
    pub provider: String,
    pub provider_secret_id: ProviderSecretId,
    pub model: String,
    pub request_hash: Sha256Hash,
    pub response_hash: Sha256Hash,
    pub response: JudgeResponse,
    pub provider_cost: Money,
    pub charged_cost: Money,
    pub cached: bool,
}

#[async_trait]
pub trait JudgeLedgerStore: Send + Sync {
    async fn cached_response(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> StoreResult<Option<CachedJudgeResponse>>;

    async fn append_record(&self, insert: JudgeAuditInsert) -> StoreResult<JudgeAuditRecord>;

    async fn list_records(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<JudgeAuditRecord>>;
}

#[async_trait]
impl<T> JudgeLedgerStore for Arc<T>
where
    T: JudgeLedgerStore + ?Sized,
{
    async fn cached_response(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> StoreResult<Option<CachedJudgeResponse>> {
        (**self)
            .cached_response(tenant_id, project_id, request_hash)
            .await
    }

    async fn append_record(&self, insert: JudgeAuditInsert) -> StoreResult<JudgeAuditRecord> {
        (**self).append_record(insert).await
    }

    async fn list_records(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<JudgeAuditRecord>> {
        (**self).list_records(tenant_id, project_id).await
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum JudgeBrokerError {
    #[error("evaluator {0} must use the judge broker lane")]
    RequiresJudgeBrokerLane(String),
    #[error("evaluator {0} must be an llm_judge evaluator")]
    RequiresLlmJudge(String),
    #[error("provider secret {0} was not found or is inactive")]
    ProviderSecretNotFound(ProviderSecretId),
    #[error("judge budget exceeded: attempted {attempted_micros} micros, remaining {remaining_micros} micros")]
    JudgeBudgetExceeded {
        attempted_micros: i64,
        remaining_micros: i64,
    },
    #[error("judge provider exceeded preflight max cost: estimated {estimated_micros} micros, actual {actual_micros} micros")]
    ProviderExceededPreflightCost {
        estimated_micros: i64,
        actual_micros: i64,
    },
    #[error("judge provider error: {0}")]
    Provider(String),
    #[error("judge storage error: {0}")]
    Store(String),
}

pub struct JudgeBrokerService<S, L, P> {
    secrets: S,
    ledger: L,
    provider: P,
    remaining_budget: Mutex<Money>,
}

#[async_trait]
pub trait JudgeBroker: Send + Sync {
    async fn evaluate(
        &self,
        request: JudgeBrokerRequest,
    ) -> Result<JudgeBrokerOutcome, JudgeBrokerError>;

    fn remaining_budget(&self) -> Result<Money, JudgeBrokerError>;
}

impl<S, L, P> JudgeBrokerService<S, L, P>
where
    S: ProviderSecretStore,
    L: JudgeLedgerStore,
    P: JudgeProvider,
{
    pub fn new(secrets: S, ledger: L, provider: P, budget: Money) -> Self {
        Self {
            secrets,
            ledger,
            provider,
            remaining_budget: Mutex::new(budget),
        }
    }

    pub fn remaining_budget(&self) -> Result<Money, JudgeBrokerError> {
        let remaining = self.remaining_budget.lock().map_err(|err| {
            JudgeBrokerError::Store(format!("judge budget mutex poisoned: {err}"))
        })?;
        Ok(remaining.clone())
    }

    pub async fn evaluate(
        &self,
        request: JudgeBrokerRequest,
    ) -> Result<JudgeBrokerOutcome, JudgeBrokerError> {
        if request.evaluator.lane != EvaluatorLane::JudgeBroker {
            return Err(JudgeBrokerError::RequiresJudgeBrokerLane(
                request.evaluator.id,
            ));
        }
        let (rubric, model) = match request.evaluator.kind {
            EvaluatorKind::LlmJudge { rubric, model } => (rubric, model),
            _ => return Err(JudgeBrokerError::RequiresLlmJudge(request.evaluator.id)),
        };
        let provider_secret = self
            .secrets
            .get_secret(
                request.tenant_id.clone(),
                request.project_id.clone(),
                request.provider_secret_id.clone(),
            )
            .await
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?
            .ok_or_else(|| {
                JudgeBrokerError::ProviderSecretNotFound(request.provider_secret_id.clone())
            })?;
        let provider = provider_secret.metadata.provider.clone();
        let judge_request = JudgeRequest {
            rubric,
            model: model.clone(),
            input: request.case.input,
            output: request.case.output,
            reference: request.case.reference,
        };
        let request_hash = judge_request_hash(
            &request.tenant_id,
            &request.project_id,
            &request.evaluator.id,
            &provider,
            &request.provider_secret_id,
            &judge_request,
        )?;

        if let Some(cached) = self
            .ledger
            .cached_response(
                request.tenant_id.clone(),
                request.project_id.clone(),
                request_hash.clone(),
            )
            .await
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?
        {
            let charged_cost = Money::usd_micros(0);
            let audit = self
                .ledger
                .append_record(JudgeAuditInsert {
                    tenant_id: request.tenant_id,
                    project_id: request.project_id,
                    evaluator_id: request.evaluator.id,
                    provider,
                    provider_secret_id: request.provider_secret_id,
                    model,
                    request_hash,
                    response_hash: cached.response_hash,
                    response: cached.response.clone(),
                    provider_cost: cached.provider_cost.clone(),
                    charged_cost: charged_cost.clone(),
                    cached: true,
                })
                .await
                .map_err(|err| JudgeBrokerError::Store(err.to_string()))?;
            return Ok(JudgeBrokerOutcome {
                result: score_from_response(&cached.response, &audit),
                audit,
                remaining_budget: self.remaining_budget()?,
            });
        }

        let max_cost = self.provider.max_cost(&provider, &judge_request);
        self.reserve_budget(&max_cost)?;
        let response = match self
            .provider
            .judge(
                judge_request,
                ProviderCredentials::new(provider.clone(), provider_secret.secret_value()),
            )
            .await
        {
            Ok(response) => response,
            Err(err) => {
                self.refund_budget(&max_cost)?;
                return Err(JudgeBrokerError::Provider(err.to_string()));
            }
        };
        if response.cost.amount_micros > max_cost.amount_micros {
            self.refund_budget(&max_cost)?;
            return Err(JudgeBrokerError::ProviderExceededPreflightCost {
                estimated_micros: max_cost.amount_micros,
                actual_micros: response.cost.amount_micros,
            });
        }
        let remaining_budget = self.finalize_budget_reservation(&max_cost, &response.cost)?;
        let response_hash = judge_response_hash(&response)?;
        let score_response = response.clone();
        let audit = self
            .ledger
            .append_record(JudgeAuditInsert {
                tenant_id: request.tenant_id,
                project_id: request.project_id,
                evaluator_id: request.evaluator.id,
                provider,
                provider_secret_id: request.provider_secret_id,
                model,
                request_hash,
                response_hash,
                provider_cost: response.cost.clone(),
                charged_cost: response.cost.clone(),
                response,
                cached: false,
            })
            .await
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?;
        Ok(JudgeBrokerOutcome {
            result: score_from_response(&score_response, &audit),
            audit,
            remaining_budget,
        })
    }

    fn reserve_budget(&self, max_cost: &Money) -> Result<Money, JudgeBrokerError> {
        let mut remaining = self.remaining_budget.lock().map_err(|err| {
            JudgeBrokerError::Store(format!("judge budget mutex poisoned: {err}"))
        })?;
        if max_cost.amount_micros > remaining.amount_micros {
            return Err(JudgeBrokerError::JudgeBudgetExceeded {
                attempted_micros: max_cost.amount_micros,
                remaining_micros: remaining.amount_micros,
            });
        }
        *remaining = remaining
            .try_sub(max_cost)
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?;
        Ok(remaining.clone())
    }

    fn refund_budget(&self, cost: &Money) -> Result<Money, JudgeBrokerError> {
        let mut remaining = self.remaining_budget.lock().map_err(|err| {
            JudgeBrokerError::Store(format!("judge budget mutex poisoned: {err}"))
        })?;
        *remaining = remaining
            .try_add(cost)
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?;
        Ok(remaining.clone())
    }

    fn finalize_budget_reservation(
        &self,
        max_cost: &Money,
        actual_cost: &Money,
    ) -> Result<Money, JudgeBrokerError> {
        let refund = max_cost
            .try_sub(actual_cost)
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?;
        let mut remaining = self.remaining_budget.lock().map_err(|err| {
            JudgeBrokerError::Store(format!("judge budget mutex poisoned: {err}"))
        })?;
        *remaining = remaining
            .try_add(&refund)
            .map_err(|err| JudgeBrokerError::Store(err.to_string()))?;
        Ok(remaining.clone())
    }
}

#[async_trait]
impl<S, L, P> JudgeBroker for JudgeBrokerService<S, L, P>
where
    S: ProviderSecretStore,
    L: JudgeLedgerStore,
    P: JudgeProvider,
{
    async fn evaluate(
        &self,
        request: JudgeBrokerRequest,
    ) -> Result<JudgeBrokerOutcome, JudgeBrokerError> {
        JudgeBrokerService::evaluate(self, request).await
    }

    fn remaining_budget(&self) -> Result<Money, JudgeBrokerError> {
        JudgeBrokerService::remaining_budget(self)
    }
}

#[async_trait]
impl<T> JudgeBroker for Arc<T>
where
    T: JudgeBroker + ?Sized,
{
    async fn evaluate(
        &self,
        request: JudgeBrokerRequest,
    ) -> Result<JudgeBrokerOutcome, JudgeBrokerError> {
        (**self).evaluate(request).await
    }

    fn remaining_budget(&self) -> Result<Money, JudgeBrokerError> {
        (**self).remaining_budget()
    }
}

#[derive(Clone)]
pub struct SqliteJudgeLedger {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteJudgeLedger {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory judge ledger")?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create judge ledger sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open judge ledger sqlite store {}", path.display()))?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> anyhow::Result<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS judge_audit_records (
                    judge_call_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    evaluator_id TEXT NOT NULL,
                    provider TEXT NOT NULL,
                    provider_secret_id TEXT NOT NULL,
                    model TEXT NOT NULL,
                    request_hash TEXT NOT NULL,
                    response_hash TEXT NOT NULL,
                    score REAL NOT NULL,
                    provider_cost_json TEXT NOT NULL,
                    charged_cost_json TEXT NOT NULL,
                    cached INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    response_json TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_judge_audit_scope
                ON judge_audit_records (tenant_id, project_id, created_at, judge_call_id);

                CREATE INDEX IF NOT EXISTS idx_judge_cache_lookup
                ON judge_audit_records (tenant_id, project_id, request_hash, cached, created_at);
                "#,
            )
            .context("initialize judge ledger sqlite store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("judge ledger sqlite connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl JudgeLedgerStore for SqliteJudgeLedger {
    async fn cached_response(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> StoreResult<Option<CachedJudgeResponse>> {
        let connection = self.lock().map_err(StoreError::backend)?;
        connection
            .query_row(
                r#"
                SELECT response_json, response_hash, provider_cost_json
                FROM judge_audit_records
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND request_hash = ?3
                  AND cached = 0
                ORDER BY created_at DESC, judge_call_id DESC
                LIMIT 1
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    request_hash.as_str()
                ],
                |row| {
                    let response_json: String = row.get(0)?;
                    let provider_cost_json: String = row.get(2)?;
                    let response = serde_json::from_str::<JudgeResponse>(&response_json)
                        .map_err(|err| json_decode_error(response_json.len(), err))?;
                    let provider_cost = serde_json::from_str::<Money>(&provider_cost_json)
                        .map_err(|err| json_decode_error(provider_cost_json.len(), err))?;
                    Ok(CachedJudgeResponse {
                        response,
                        response_hash: Sha256Hash::new(row.get::<_, String>(1)?)
                            .map_err(sql_decode_error)?,
                        provider_cost,
                    })
                },
            )
            .optional()
            .map_err(StoreError::backend)
    }

    async fn append_record(&self, insert: JudgeAuditInsert) -> StoreResult<JudgeAuditRecord> {
        let record = JudgeAuditRecord {
            judge_call_id: JudgeCallId::new(Uuid::new_v4().to_string())
                .map_err(StoreError::backend)?,
            tenant_id: insert.tenant_id,
            project_id: insert.project_id,
            evaluator_id: insert.evaluator_id,
            provider: insert.provider,
            provider_secret_id: insert.provider_secret_id,
            model: insert.model,
            request_hash: insert.request_hash,
            response_hash: insert.response_hash,
            score: insert.response.score,
            provider_cost: insert.provider_cost,
            charged_cost: insert.charged_cost,
            cached: insert.cached,
            created_at: Utc::now(),
        };
        let response_json = serde_json::to_string(&insert.response).map_err(StoreError::backend)?;
        let provider_cost_json =
            serde_json::to_string(&record.provider_cost).map_err(StoreError::backend)?;
        let charged_cost_json =
            serde_json::to_string(&record.charged_cost).map_err(StoreError::backend)?;
        let connection = self.lock().map_err(StoreError::backend)?;
        connection
            .execute(
                r#"
                INSERT INTO judge_audit_records
                  (judge_call_id, tenant_id, project_id, evaluator_id, provider,
                   provider_secret_id, model, request_hash, response_hash, score,
                   provider_cost_json, charged_cost_json, cached, created_at, response_json)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                "#,
                params![
                    record.judge_call_id.as_str(),
                    record.tenant_id.as_str(),
                    record.project_id.as_str(),
                    record.evaluator_id.as_str(),
                    record.provider.as_str(),
                    record.provider_secret_id.as_str(),
                    record.model.as_str(),
                    record.request_hash.as_str(),
                    record.response_hash.as_str(),
                    record.score,
                    provider_cost_json,
                    charged_cost_json,
                    if record.cached { 1_i64 } else { 0_i64 },
                    record.created_at.to_rfc3339(),
                    response_json,
                ],
            )
            .map_err(StoreError::backend)?;
        Ok(record)
    }

    async fn list_records(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<JudgeAuditRecord>> {
        let connection = self.lock().map_err(StoreError::backend)?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT judge_call_id, tenant_id, project_id, evaluator_id, provider,
                       provider_secret_id, model, request_hash, response_hash, score,
                       provider_cost_json, charged_cost_json, cached, created_at
                FROM judge_audit_records
                WHERE tenant_id = ?1 AND project_id = ?2
                ORDER BY created_at ASC, judge_call_id ASC
                "#,
            )
            .map_err(StoreError::backend)?;
        let rows = statement
            .query_map(
                params![tenant_id.as_str(), project_id.as_str()],
                decode_record,
            )
            .map_err(StoreError::backend)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(StoreError::backend)?);
        }
        Ok(records)
    }
}

#[derive(Clone, Debug)]
pub struct KeywordJudgeProvider {
    cost: Money,
}

impl KeywordJudgeProvider {
    pub fn new(cost: Money) -> Self {
        Self { cost }
    }
}

impl Default for KeywordJudgeProvider {
    fn default() -> Self {
        Self {
            cost: Money::usd_micros(25),
        }
    }
}

#[async_trait]
impl JudgeProvider for KeywordJudgeProvider {
    fn max_cost(&self, _provider: &str, _request: &JudgeRequest) -> Money {
        self.cost.clone()
    }

    async fn judge(
        &self,
        request: JudgeRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<JudgeResponse> {
        if credentials.secret_value().is_empty() {
            return Err(JudgeProviderError::backend("provider credential is empty"));
        }
        let score = if request.reference.as_ref() == Some(&request.output) {
            1.0
        } else if output_mentions_rubric(&request.output, &request.rubric) {
            0.75
        } else {
            0.0
        };
        Ok(JudgeResponse {
            score,
            rationale: format!(
                "keyword judge via provider {} scored output against rubric {}",
                credentials.provider(),
                request.rubric
            ),
            cost: self.cost.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct HttpJudgeProviderConfig {
    pub endpoint_url: String,
    pub max_cost: Money,
    pub retry_policy: RetryPolicy,
}

impl HttpJudgeProviderConfig {
    pub fn openai_default() -> Self {
        Self {
            endpoint_url: "https://api.openai.com/v1/chat/completions".to_string(),
            max_cost: Money::usd_micros(100_000),
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn anthropic_default() -> Self {
        Self {
            endpoint_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_cost: Money::usd_micros(100_000),
            retry_policy: RetryPolicy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff_ms: 250,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpenAiJudgeProvider {
    client: reqwest::Client,
    config: HttpJudgeProviderConfig,
}

impl OpenAiJudgeProvider {
    pub fn new(config: HttpJudgeProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }
}

#[async_trait]
impl JudgeProvider for OpenAiJudgeProvider {
    fn max_cost(&self, _provider: &str, _request: &JudgeRequest) -> Money {
        self.config.max_cost.clone()
    }

    async fn judge(
        &self,
        request: JudgeRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<JudgeResponse> {
        let prompt = judge_prompt(&request).map_err(JudgeProviderError::backend)?;
        let body = serde_json::json!({
            "model": request.model,
            "temperature": 0,
            "response_format": { "type": "json_object" },
            "messages": [
                {
                    "role": "system",
                    "content": "You are a strict evaluation judge. Return only JSON with numeric score in [0,1] and string rationale."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });
        let response = send_json_with_retries(
            || {
                self.client
                    .post(&self.config.endpoint_url)
                    .bearer_auth(credentials.secret_value())
                    .json(&body)
            },
            self.config.retry_policy,
        )
        .await
        .map_err(JudgeProviderError::backend)?;
        let payload: OpenAiChatCompletionResponse = response
            .json()
            .await
            .context("decode openai judge response")
            .map_err(JudgeProviderError::backend)?;
        let content = payload
            .choices
            .first()
            .map(|choice| choice.message.content.as_str())
            .ok_or_else(|| {
                JudgeProviderError::backend("openai judge response did not include a choice")
            })?;
        let scored = parse_scored_judge_json(content).map_err(JudgeProviderError::backend)?;
        Ok(JudgeResponse {
            score: scored.score,
            rationale: scored.rationale,
            cost: self.config.max_cost.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct AnthropicJudgeProvider {
    client: reqwest::Client,
    config: HttpJudgeProviderConfig,
    anthropic_version: String,
}

impl AnthropicJudgeProvider {
    pub fn new(config: HttpJudgeProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            anthropic_version: "2023-06-01".to_string(),
        }
    }
}

#[async_trait]
impl JudgeProvider for AnthropicJudgeProvider {
    fn max_cost(&self, _provider: &str, _request: &JudgeRequest) -> Money {
        self.config.max_cost.clone()
    }

    async fn judge(
        &self,
        request: JudgeRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<JudgeResponse> {
        let prompt = judge_prompt(&request).map_err(JudgeProviderError::backend)?;
        let body = serde_json::json!({
            "model": request.model,
            "max_tokens": 512,
            "temperature": 0,
            "system": "You are a strict evaluation judge. Return only JSON with numeric score in [0,1] and string rationale.",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });
        let response = send_json_with_retries(
            || {
                self.client
                    .post(&self.config.endpoint_url)
                    .header("x-api-key", credentials.secret_value())
                    .header("anthropic-version", self.anthropic_version.as_str())
                    .json(&body)
            },
            self.config.retry_policy,
        )
        .await
        .map_err(JudgeProviderError::backend)?;
        let payload: AnthropicMessagesResponse = response
            .json()
            .await
            .context("decode anthropic judge response")
            .map_err(JudgeProviderError::backend)?;
        let content = payload
            .content
            .iter()
            .find(|block| block.kind == "text")
            .map(|block| block.text.as_str())
            .ok_or_else(|| {
                JudgeProviderError::backend("anthropic judge response did not include text content")
            })?;
        let scored = parse_scored_judge_json(content).map_err(JudgeProviderError::backend)?;
        Ok(JudgeResponse {
            score: scored.score,
            rationale: scored.rationale,
            cost: self.config.max_cost.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct HttpRoutingJudgeProvider {
    openai: OpenAiJudgeProvider,
    anthropic: AnthropicJudgeProvider,
}

impl HttpRoutingJudgeProvider {
    pub fn new(openai: OpenAiJudgeProvider, anthropic: AnthropicJudgeProvider) -> Self {
        Self { openai, anthropic }
    }
}

impl Default for HttpRoutingJudgeProvider {
    fn default() -> Self {
        Self {
            openai: OpenAiJudgeProvider::new(HttpJudgeProviderConfig::openai_default()),
            anthropic: AnthropicJudgeProvider::new(HttpJudgeProviderConfig::anthropic_default()),
        }
    }
}

#[async_trait]
impl JudgeProvider for HttpRoutingJudgeProvider {
    fn max_cost(&self, provider: &str, request: &JudgeRequest) -> Money {
        if provider.eq_ignore_ascii_case("openai")
            || provider.eq_ignore_ascii_case("openai-compatible")
        {
            self.openai.max_cost(provider, request)
        } else if provider.eq_ignore_ascii_case("anthropic") {
            self.anthropic.max_cost(provider, request)
        } else {
            Money::usd_micros(0)
        }
    }

    async fn judge(
        &self,
        request: JudgeRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<JudgeResponse> {
        if credentials.provider().eq_ignore_ascii_case("openai")
            || credentials
                .provider()
                .eq_ignore_ascii_case("openai-compatible")
        {
            self.openai.judge(request, credentials).await
        } else if credentials.provider().eq_ignore_ascii_case("anthropic") {
            self.anthropic.judge(request, credentials).await
        } else {
            Err(JudgeProviderError::backend(format!(
                "unsupported judge provider {}",
                credentials.provider()
            )))
        }
    }
}

async fn send_json_with_retries<F>(
    mut build_request: F,
    retry_policy: RetryPolicy,
) -> anyhow::Result<reqwest::Response>
where
    F: FnMut() -> reqwest::RequestBuilder,
{
    let max_attempts = retry_policy.max_attempts.max(1);
    for attempt in 1..=max_attempts {
        let response = build_request()
            .send()
            .await
            .context("send judge provider request")?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        if !is_retryable_status(status) || attempt == max_attempts {
            return Err(provider_http_error(response).await);
        }
        let delay = retry_delay(&response, retry_policy, attempt);
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }
    Err(anyhow!("judge provider request exhausted retries"))
}

fn is_retryable_status(status: ReqwestStatusCode) -> bool {
    status == ReqwestStatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retry_delay(
    response: &reqwest::Response,
    retry_policy: RetryPolicy,
    attempt: u32,
) -> std::time::Duration {
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(std::time::Duration::from_secs);
    retry_after.unwrap_or_else(|| {
        std::time::Duration::from_millis(
            retry_policy
                .base_backoff_ms
                .saturating_mul(u64::from(attempt)),
        )
    })
}

async fn provider_http_error(response: reqwest::Response) -> anyhow::Error {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    anyhow!(
        "judge provider returned {}: {}",
        status.as_u16(),
        truncate_error_body(&body)
    )
}

fn truncate_error_body(body: &str) -> String {
    const LIMIT: usize = 512;
    if body.chars().count() <= LIMIT {
        return body.to_string();
    }
    let truncated = body.chars().take(LIMIT).collect::<String>();
    format!("{truncated}...")
}

fn judge_prompt(request: &JudgeRequest) -> anyhow::Result<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "rubric": request.rubric,
        "input": request.input,
        "output": request.output,
        "reference": request.reference
    }))
    .context("serialize judge prompt")
}

#[derive(Clone, Debug, Deserialize)]
struct OpenAiChatCompletionResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenAiMessage {
    content: String,
}

#[derive(Clone, Debug, Deserialize)]
struct AnthropicMessagesResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Clone, Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: String,
}

#[derive(Clone, Debug, Deserialize)]
struct StructuredJudgePayload {
    score: f64,
    rationale: String,
}

fn parse_scored_judge_json(content: &str) -> anyhow::Result<StructuredJudgePayload> {
    let normalized = strip_json_fence(content.trim());
    let payload: StructuredJudgePayload =
        serde_json::from_str(normalized).context("parse structured judge JSON")?;
    if !(0.0..=1.0).contains(&payload.score) || !payload.score.is_finite() {
        return Err(anyhow!(
            "judge score must be a finite number in [0,1], got {}",
            payload.score
        ));
    }
    if payload.rationale.trim().is_empty() {
        return Err(anyhow!("judge rationale cannot be empty"));
    }
    Ok(payload)
}

fn strip_json_fence(content: &str) -> &str {
    let Some(without_prefix) = content.strip_prefix("```") else {
        return content;
    };
    let without_language = without_prefix
        .strip_prefix("json")
        .unwrap_or(without_prefix)
        .trim_start();
    without_language
        .strip_suffix("```")
        .map(str::trim_end)
        .unwrap_or(content)
}

pub fn judge_request_hash(
    tenant_id: &TenantId,
    project_id: &ProjectId,
    evaluator_id: &str,
    provider: &str,
    provider_secret_id: &ProviderSecretId,
    request: &JudgeRequest,
) -> Result<Sha256Hash, JudgeBrokerError> {
    #[derive(Serialize)]
    struct HashableJudgeRequest<'a> {
        tenant_id: &'a TenantId,
        project_id: &'a ProjectId,
        evaluator_id: &'a str,
        provider: &'a str,
        provider_secret_id: &'a ProviderSecretId,
        request: &'a JudgeRequest,
    }

    judge_json_hash(&HashableJudgeRequest {
        tenant_id,
        project_id,
        evaluator_id,
        provider,
        provider_secret_id,
        request,
    })
}

pub fn judge_response_hash(response: &JudgeResponse) -> Result<Sha256Hash, JudgeBrokerError> {
    judge_json_hash(response)
}

fn judge_json_hash<T: Serialize>(value: &T) -> Result<Sha256Hash, JudgeBrokerError> {
    beater_core::sha256_json_hash(value).map_err(|err| JudgeBrokerError::Store(err.to_string()))
}

fn score_from_response(response: &JudgeResponse, audit: &JudgeAuditRecord) -> ScoreResult {
    ScoreResult {
        score: response.score,
        label: None,
        evidence: serde_json::json!({
            "rationale": response.rationale,
            "provider": audit.provider,
            "provider_secret_id": audit.provider_secret_id,
            "request_hash": audit.request_hash,
            "response_hash": audit.response_hash,
            "cached": audit.cached,
            "provider_cost_micros": audit.provider_cost.amount_micros,
            "charged_cost_micros": audit.charged_cost.amount_micros,
            "currency": audit.provider_cost.currency.as_str()
        }),
    }
}

fn output_mentions_rubric(output: &serde_json::Value, rubric: &str) -> bool {
    let output_text = output.to_string().to_lowercase();
    rubric
        .split_whitespace()
        .map(str::to_lowercase)
        .any(|word| !word.is_empty() && output_text.contains(&word))
}

fn decode_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<JudgeAuditRecord> {
    let provider_cost_json: String = row.get(10)?;
    let charged_cost_json: String = row.get(11)?;
    let created_at = parse_time(row.get::<_, String>(13)?)?;
    Ok(JudgeAuditRecord {
        judge_call_id: JudgeCallId::new(row.get::<_, String>(0)?).map_err(sql_decode_error)?,
        tenant_id: TenantId::new(row.get::<_, String>(1)?).map_err(sql_decode_error)?,
        project_id: ProjectId::new(row.get::<_, String>(2)?).map_err(sql_decode_error)?,
        evaluator_id: row.get(3)?,
        provider: row.get(4)?,
        provider_secret_id: ProviderSecretId::new(row.get::<_, String>(5)?)
            .map_err(sql_decode_error)?,
        model: row.get(6)?,
        request_hash: Sha256Hash::new(row.get::<_, String>(7)?).map_err(sql_decode_error)?,
        response_hash: Sha256Hash::new(row.get::<_, String>(8)?).map_err(sql_decode_error)?,
        score: row.get(9)?,
        provider_cost: serde_json::from_str(&provider_cost_json)
            .map_err(|err| json_decode_error(provider_cost_json.len(), err))?,
        charged_cost: serde_json::from_str(&charged_cost_json)
            .map_err(|err| json_decode_error(charged_cost_json.len(), err))?,
        cached: row.get::<_, i64>(12)? != 0,
        created_at,
    })
}

fn parse_time(value: String) -> rusqlite::Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                value.len(),
                rusqlite::types::Type::Text,
                Box::new(err),
            )
        })
}

fn json_decode_error(
    index: usize,
    error: impl std::error::Error + Send + Sync + 'static,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(error))
}

fn sql_decode_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::{header, HeaderMap, StatusCode};
    use axum::response::{IntoResponse, Response};
    use axum::{routing::post, Json, Router};
    use beater_secrets::{PutProviderSecretRequest, SqliteProviderSecretStore};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn judge_broker_caches_and_audits_without_exposing_provider_secret() {
        let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let ledger = SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let metadata = secrets
            .put_secret(PutProviderSecretRequest {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                provider: "openai".to_string(),
                display_name: "judge".to_string(),
                secret_value: "sk-live-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let calls = Arc::new(AtomicUsize::new(0));
        let broker = JudgeBrokerService::new(
            secrets.clone(),
            ledger.clone(),
            CountingJudgeProvider {
                calls: calls.clone(),
                cost: Money::usd_micros(40),
            },
            Money::usd_micros(100),
        );
        let first = broker
            .evaluate(fixture_request(
                tenant_id.clone(),
                project_id.clone(),
                metadata.provider_secret_id.clone(),
            ))
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let second = broker
            .evaluate(fixture_request(
                tenant_id.clone(),
                project_id.clone(),
                metadata.provider_secret_id,
            ))
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!first.audit.cached);
        assert!(second.audit.cached);
        assert_eq!(first.audit.charged_cost.amount_micros, 40);
        assert_eq!(second.audit.charged_cost.amount_micros, 0);
        assert_eq!(second.remaining_budget.amount_micros, 60);
        assert_eq!(first.audit.request_hash, second.audit.request_hash);
        assert_eq!(first.audit.response_hash, second.audit.response_hash);

        let records = ledger
            .list_records(tenant_id, project_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(records.len(), 2);
        let serialized = serde_json::to_string(&records).unwrap_or_else(|err| panic!("{err}"));
        assert!(!serialized.contains("sk-live-secret"));
        assert!(
            !format!("{:?}", ProviderCredentials::new("openai", "sk-live-secret"))
                .contains("sk-live-secret")
        );
    }

    #[tokio::test]
    async fn judge_broker_rejects_inactive_secret_references() {
        let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let ledger = SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let metadata = secrets
            .put_secret(PutProviderSecretRequest {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                provider: "openai".to_string(),
                display_name: "judge".to_string(),
                secret_value: "sk-live-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        secrets
            .revoke_secret(
                tenant_id.clone(),
                project_id.clone(),
                metadata.provider_secret_id.clone(),
                Utc::now(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let broker = JudgeBrokerService::new(
            secrets,
            ledger,
            KeywordJudgeProvider::default(),
            Money::usd_micros(100),
        );

        assert!(matches!(
            broker
                .evaluate(fixture_request(
                    tenant_id,
                    project_id,
                    metadata.provider_secret_id
                ))
                .await,
            Err(JudgeBrokerError::ProviderSecretNotFound(_))
        ));
    }

    #[tokio::test]
    async fn judge_broker_does_not_cache_budget_rejected_calls() {
        let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let ledger = SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let metadata = secrets
            .put_secret(PutProviderSecretRequest {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                provider: "openai".to_string(),
                display_name: "judge".to_string(),
                secret_value: "sk-live-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let broker = JudgeBrokerService::new(
            secrets,
            ledger.clone(),
            KeywordJudgeProvider::new(Money::usd_micros(75)),
            Money::usd_micros(40),
        );

        assert!(matches!(
            broker
                .evaluate(fixture_request(
                    tenant_id.clone(),
                    project_id.clone(),
                    metadata.provider_secret_id
                ))
                .await,
            Err(JudgeBrokerError::JudgeBudgetExceeded {
                attempted_micros: 75,
                remaining_micros: 40
            })
        ));
        let records = ledger
            .list_records(tenant_id, project_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn sqlite_judge_ledger_list_records_is_scoped_to_tenant_and_project() {
        let ledger = SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant-a").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project-a").unwrap_or_else(|err| panic!("{err}"));
        let other_tenant_id = TenantId::new("tenant-b").unwrap_or_else(|err| panic!("{err}"));
        let other_project_id = ProjectId::new("project-b").unwrap_or_else(|err| panic!("{err}"));
        let request_hash =
            Sha256Hash::new("shared-request-hash").unwrap_or_else(|err| panic!("{err}"));

        let target_first = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "target-first-response",
                rationale: "target first",
                score: 0.25,
                cost_micros: 25,
                cached: false,
            },
        )
        .await;
        set_record_created_at(&ledger, &target_first, "2026-01-01T00:00:01Z");
        append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &other_project_id,
                request_hash: &request_hash,
                response_hash: "same-tenant-other-project-response",
                rationale: "same tenant other project",
                score: 0.5,
                cost_micros: 50,
                cached: false,
            },
        )
        .await;
        append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &other_tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "other-tenant-same-project-response",
                rationale: "other tenant same project",
                score: 0.75,
                cost_micros: 75,
                cached: false,
            },
        )
        .await;
        let target_second = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "target-second-response",
                rationale: "target second",
                score: 1.0,
                cost_micros: 100,
                cached: true,
            },
        )
        .await;
        set_record_created_at(&ledger, &target_second, "2026-01-01T00:00:04Z");

        let records = ledger
            .list_records(tenant_id.clone(), project_id.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            records
                .iter()
                .map(|record| record.judge_call_id.clone())
                .collect::<Vec<_>>(),
            vec![target_first.judge_call_id, target_second.judge_call_id]
        );
        assert!(records
            .iter()
            .all(|record| record.tenant_id == tenant_id && record.project_id == project_id));
    }

    #[tokio::test]
    async fn sqlite_judge_ledger_cached_response_uses_latest_uncached_record_in_scope() {
        let ledger = SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant-a").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project-a").unwrap_or_else(|err| panic!("{err}"));
        let other_tenant_id = TenantId::new("tenant-b").unwrap_or_else(|err| panic!("{err}"));
        let other_project_id = ProjectId::new("project-b").unwrap_or_else(|err| panic!("{err}"));
        let request_hash =
            Sha256Hash::new("shared-request-hash").unwrap_or_else(|err| panic!("{err}"));

        let target_old = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "target-old-response",
                rationale: "target old",
                score: 0.2,
                cost_micros: 20,
                cached: false,
            },
        )
        .await;
        set_record_created_at(&ledger, &target_old, "2026-01-01T00:00:01Z");
        let target_latest = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "target-latest-response",
                rationale: "target latest",
                score: 0.8,
                cost_micros: 80,
                cached: false,
            },
        )
        .await;
        set_record_created_at(&ledger, &target_latest, "2026-01-01T00:00:02Z");
        let target_cached = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "target-cached-response",
                rationale: "target cached",
                score: 0.99,
                cost_micros: 99,
                cached: true,
            },
        )
        .await;
        set_record_created_at(&ledger, &target_cached, "2026-01-01T00:00:03Z");
        let other_tenant_newer = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &other_tenant_id,
                project_id: &project_id,
                request_hash: &request_hash,
                response_hash: "other-tenant-newer-response",
                rationale: "other tenant newer",
                score: 0.1,
                cost_micros: 10,
                cached: false,
            },
        )
        .await;
        set_record_created_at(&ledger, &other_tenant_newer, "2026-01-01T00:00:04Z");
        let other_project_newer = append_ledger_record(
            &ledger,
            TestLedgerRecord {
                tenant_id: &tenant_id,
                project_id: &other_project_id,
                request_hash: &request_hash,
                response_hash: "other-project-newer-response",
                rationale: "other project newer",
                score: 0.3,
                cost_micros: 30,
                cached: false,
            },
        )
        .await;
        set_record_created_at(&ledger, &other_project_newer, "2026-01-01T00:00:05Z");

        let cached = ledger
            .cached_response(tenant_id, project_id, request_hash)
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("expected cached judge response"));

        assert_eq!(cached.response_hash, target_latest.response_hash);
        assert_eq!(cached.response.score, 0.8);
        assert_eq!(cached.response.rationale, "target latest");
        assert_eq!(cached.provider_cost, Money::usd_micros(80));
    }

    #[tokio::test]
    async fn openai_provider_retries_rate_limit_and_parses_structured_score() {
        let calls = Arc::new(AtomicUsize::new(0));
        let endpoint = spawn_mock_provider(Arc::new(MockHttpJudgeState {
            calls: calls.clone(),
            expected_auth_header: "Bearer sk-openai".to_string(),
            auth_header_name: "authorization".to_string(),
            required_secondary_header: None,
            rate_limit_first: true,
            response: MockProviderResponse::OpenAi,
        }))
        .await
        .unwrap_or_else(|err| panic!("{err}"));
        let provider = OpenAiJudgeProvider::new(HttpJudgeProviderConfig {
            endpoint_url: endpoint,
            max_cost: Money::usd_micros(33),
            retry_policy: RetryPolicy {
                max_attempts: 2,
                base_backoff_ms: 0,
            },
        });

        let response = provider
            .judge(
                fixture_judge_request(),
                ProviderCredentials::new("openai", "sk-openai"),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(response.score, 0.82);
        assert_eq!(response.rationale, "openai mock rationale");
        assert_eq!(response.cost, Money::usd_micros(33));
    }

    #[tokio::test]
    async fn anthropic_provider_sends_api_key_header_and_parses_fenced_json() {
        let calls = Arc::new(AtomicUsize::new(0));
        let endpoint = spawn_mock_provider(Arc::new(MockHttpJudgeState {
            calls: calls.clone(),
            expected_auth_header: "sk-anthropic".to_string(),
            auth_header_name: "x-api-key".to_string(),
            required_secondary_header: Some((
                "anthropic-version".to_string(),
                "2023-06-01".to_string(),
            )),
            rate_limit_first: false,
            response: MockProviderResponse::Anthropic,
        }))
        .await
        .unwrap_or_else(|err| panic!("{err}"));
        let provider = AnthropicJudgeProvider::new(HttpJudgeProviderConfig {
            endpoint_url: endpoint,
            max_cost: Money::usd_micros(44),
            retry_policy: RetryPolicy {
                max_attempts: 1,
                base_backoff_ms: 0,
            },
        });

        let response = provider
            .judge(
                fixture_judge_request(),
                ProviderCredentials::new("anthropic", "sk-anthropic"),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(response.score, 0.67);
        assert_eq!(response.rationale, "anthropic mock rationale");
        assert_eq!(response.cost, Money::usd_micros(44));
    }

    fn fixture_request(
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
    ) -> JudgeBrokerRequest {
        JudgeBrokerRequest {
            tenant_id,
            project_id,
            evaluator: EvaluatorSpec {
                id: "judge-correctness".to_string(),
                lane: EvaluatorLane::JudgeBroker,
                kind: EvaluatorKind::LlmJudge {
                    rubric: "correctness".to_string(),
                    model: "judge-model".to_string(),
                },
            },
            case: EvaluationCase {
                input: serde_json::json!("question"),
                output: serde_json::json!("answer"),
                reference: Some(serde_json::json!("answer")),
                trace: None,
            },
            provider_secret_id,
        }
    }

    fn fixture_judge_request() -> JudgeRequest {
        JudgeRequest {
            rubric: "correctness".to_string(),
            model: "judge-model".to_string(),
            input: serde_json::json!("question"),
            output: serde_json::json!("answer"),
            reference: Some(serde_json::json!("answer")),
        }
    }

    struct TestLedgerRecord<'a> {
        tenant_id: &'a TenantId,
        project_id: &'a ProjectId,
        request_hash: &'a Sha256Hash,
        response_hash: &'a str,
        rationale: &'a str,
        score: f64,
        cost_micros: i64,
        cached: bool,
    }

    async fn append_ledger_record(
        ledger: &SqliteJudgeLedger,
        record: TestLedgerRecord<'_>,
    ) -> JudgeAuditRecord {
        let provider_cost = Money::usd_micros(record.cost_micros);
        ledger
            .append_record(JudgeAuditInsert {
                tenant_id: record.tenant_id.clone(),
                project_id: record.project_id.clone(),
                evaluator_id: "judge-correctness".to_string(),
                provider: "openai".to_string(),
                provider_secret_id: ProviderSecretId::new("secret")
                    .unwrap_or_else(|err| panic!("{err}")),
                model: "judge-model".to_string(),
                request_hash: record.request_hash.clone(),
                response_hash: Sha256Hash::new(record.response_hash)
                    .unwrap_or_else(|err| panic!("{err}")),
                response: JudgeResponse {
                    score: record.score,
                    rationale: record.rationale.to_string(),
                    cost: provider_cost.clone(),
                },
                provider_cost,
                charged_cost: if record.cached {
                    Money::usd_micros(0)
                } else {
                    Money::usd_micros(record.cost_micros)
                },
                cached: record.cached,
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"))
    }

    fn set_record_created_at(
        ledger: &SqliteJudgeLedger,
        record: &JudgeAuditRecord,
        created_at: &str,
    ) {
        let connection = ledger.lock().unwrap_or_else(|err| panic!("{err}"));
        connection
            .execute(
                "UPDATE judge_audit_records SET created_at = ?1 WHERE judge_call_id = ?2",
                rusqlite::params![created_at, record.judge_call_id.as_str()],
            )
            .unwrap_or_else(|err| panic!("{err}"));
    }

    async fn spawn_mock_provider(state: Arc<MockHttpJudgeState>) -> anyhow::Result<String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("bind mock judge provider")?;
        let addr = listener.local_addr().context("read mock provider addr")?;
        let app = Router::new()
            .route("/judge", post(mock_judge_handler))
            .with_state(state);
        tokio::spawn(async move {
            if let Err(err) = axum::serve(listener, app).await {
                panic!("{err}");
            }
        });
        Ok(format!("http://{addr}/judge"))
    }

    #[derive(Debug)]
    struct MockHttpJudgeState {
        calls: Arc<AtomicUsize>,
        expected_auth_header: String,
        auth_header_name: String,
        required_secondary_header: Option<(String, String)>,
        rate_limit_first: bool,
        response: MockProviderResponse,
    }

    #[derive(Clone, Debug)]
    enum MockProviderResponse {
        OpenAi,
        Anthropic,
    }

    async fn mock_judge_handler(
        State(state): State<Arc<MockHttpJudgeState>>,
        headers: HeaderMap,
        Json(body): Json<serde_json::Value>,
    ) -> Response {
        let call_index = state.calls.fetch_add(1, Ordering::SeqCst);
        if state.rate_limit_first && call_index == 0 {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, "0")],
                Json(serde_json::json!({ "error": "rate limited" })),
            )
                .into_response();
        }
        let auth_header = headers
            .get(state.auth_header_name.as_str())
            .and_then(|value| value.to_str().ok());
        if auth_header != Some(state.expected_auth_header.as_str()) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "bad auth" })),
            )
                .into_response();
        }
        if let Some((name, expected_value)) = &state.required_secondary_header {
            let actual = headers
                .get(name.as_str())
                .and_then(|value| value.to_str().ok());
            if actual != Some(expected_value.as_str()) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "missing secondary header" })),
                )
                    .into_response();
            }
        }
        if body.get("model").and_then(serde_json::Value::as_str) != Some("judge-model") {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "bad model" })),
            )
                .into_response();
        }
        match state.response {
            MockProviderResponse::OpenAi => Json(serde_json::json!({
                "choices": [
                    {
                        "message": {
                            "content": "{\"score\":0.82,\"rationale\":\"openai mock rationale\"}"
                        }
                    }
                ]
            }))
            .into_response(),
            MockProviderResponse::Anthropic => Json(serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": "```json\n{\"score\":0.67,\"rationale\":\"anthropic mock rationale\"}\n```"
                    }
                ]
            }))
            .into_response(),
        }
    }

    #[derive(Clone)]
    struct CountingJudgeProvider {
        calls: Arc<AtomicUsize>,
        cost: Money,
    }

    #[async_trait]
    impl JudgeProvider for CountingJudgeProvider {
        fn max_cost(&self, _provider: &str, _request: &JudgeRequest) -> Money {
            self.cost.clone()
        }

        async fn judge(
            &self,
            _request: JudgeRequest,
            credentials: ProviderCredentials,
        ) -> JudgeProviderResult<JudgeResponse> {
            if credentials.secret_value() != "sk-live-secret" {
                return Err(JudgeProviderError::backend(
                    "unexpected provider credential",
                ));
            }
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(JudgeResponse {
                score: 0.9,
                rationale: "counted".to_string(),
                cost: self.cost.clone(),
            })
        }
    }
}
