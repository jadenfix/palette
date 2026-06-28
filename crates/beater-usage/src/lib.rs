use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{Money, ProjectId, TenantId, Timestamp, UsageRecordId};
use beater_datasets::DatasetEvalReport;
use beater_experiments::{CaseExperimentScore, ExperimentRunReport};
use beater_judge::JudgeBrokerOutcome;
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageMeter {
    JudgeCostMicros,
}

impl UsageMeter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JudgeCostMicros => "judge_cost_micros",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageRecordSourceKind {
    JudgeCall,
    DatasetEvalReport,
    ExperimentRun,
    Manual,
}

impl UsageRecordSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JudgeCall => "judge_call",
            Self::DatasetEvalReport => "dataset_eval_report",
            Self::ExperimentRun => "experiment_run",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsageRecordInsert {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub meter: UsageMeter,
    pub quantity: i64,
    pub unit: String,
    pub source_kind: UsageRecordSourceKind,
    pub source_id: String,
    pub attributes: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub usage_record_id: UsageRecordId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub meter: UsageMeter,
    pub quantity: i64,
    pub unit: String,
    pub source_kind: UsageRecordSourceKind,
    pub source_id: String,
    pub attributes: Value,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UsageTotal {
    pub quantity: i64,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UsageSummary {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub totals: BTreeMap<String, UsageTotal>,
}

#[async_trait]
pub trait UsageLedgerStore: Send + Sync {
    async fn record_usage(&self, insert: UsageRecordInsert) -> StoreResult<UsageRecord>;

    async fn list_usage(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<UsageRecord>>;

    async fn summarize_usage(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<UsageSummary>;
}

#[derive(Clone)]
pub struct SqliteUsageLedger {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteUsageLedger {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory usage sqlite")?;
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
                .with_context(|| format!("create usage sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite usage ledger {}", path.display()))?;
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

                CREATE TABLE IF NOT EXISTS usage_records (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    usage_record_id TEXT NOT NULL,
                    meter TEXT NOT NULL,
                    quantity INTEGER NOT NULL,
                    unit TEXT NOT NULL,
                    source_kind TEXT NOT NULL,
                    source_id TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    record_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, usage_record_id),
                    UNIQUE (tenant_id, project_id, meter, source_kind, source_id)
                );

                CREATE INDEX IF NOT EXISTS idx_usage_records_list
                  ON usage_records (tenant_id, project_id, created_at, usage_record_id);
                "#,
            )
            .context("initialize sqlite usage ledger")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("sqlite usage connection mutex poisoned: {err}"))
    }

    fn select_by_unique(
        connection: &Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        meter: UsageMeter,
        source_kind: UsageRecordSourceKind,
        source_id: &str,
    ) -> anyhow::Result<UsageRecord> {
        let record_json = connection
            .query_row(
                r#"
                SELECT record_json
                FROM usage_records
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND meter = ?3
                  AND source_kind = ?4
                  AND source_id = ?5
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    meter.as_str(),
                    source_kind.as_str(),
                    source_id
                ],
                |row| row.get::<_, String>(0),
            )
            .with_context(|| format!("usage record source {source_id} not found"))?;
        serde_json::from_str(&record_json).context("decode usage record")
    }
}

#[async_trait]
impl UsageLedgerStore for SqliteUsageLedger {
    async fn record_usage(&self, insert: UsageRecordInsert) -> StoreResult<UsageRecord> {
        let record = UsageRecord {
            usage_record_id: UsageRecordId::new(Uuid::new_v4().to_string())
                .map_err(StoreError::backend)?,
            tenant_id: insert.tenant_id,
            project_id: insert.project_id,
            meter: insert.meter,
            quantity: insert.quantity,
            unit: insert.unit,
            source_kind: insert.source_kind,
            source_id: insert.source_id,
            attributes: insert.attributes,
            created_at: Utc::now(),
        };
        let record_json = serde_json::to_string(&record)
            .context("serialize usage record")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT OR IGNORE INTO usage_records
                  (tenant_id, project_id, usage_record_id, meter, quantity, unit,
                   source_kind, source_id, created_at, record_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
                params![
                    record.tenant_id.as_str(),
                    record.project_id.as_str(),
                    record.usage_record_id.as_str(),
                    record.meter.as_str(),
                    record.quantity,
                    record.unit,
                    record.source_kind.as_str(),
                    record.source_id,
                    record.created_at.to_rfc3339(),
                    record_json
                ],
            )
            .context("insert usage record")
            .into_store()?;
        Self::select_by_unique(
            &connection,
            &record.tenant_id,
            &record.project_id,
            record.meter,
            record.source_kind,
            &record.source_id,
        )
        .into_store()
    }

    async fn list_usage(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<UsageRecord>> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT record_json
                FROM usage_records
                WHERE tenant_id = ?1 AND project_id = ?2
                ORDER BY created_at ASC, usage_record_id ASC
                "#,
            )
            .context("prepare usage list query")
            .into_store()?;
        let rows = statement
            .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
                row.get::<_, String>(0)
            })
            .context("query usage list")
            .into_store()?;
        let mut records = Vec::new();
        for row in rows {
            let record_json = row.context("read usage record row").into_store()?;
            records.push(
                serde_json::from_str(&record_json)
                    .context("decode usage record")
                    .into_store()?,
            );
        }
        Ok(records)
    }

    async fn summarize_usage(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<UsageSummary> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT meter, unit, COALESCE(SUM(quantity), 0)
                FROM usage_records
                WHERE tenant_id = ?1 AND project_id = ?2
                GROUP BY meter, unit
                ORDER BY meter ASC, unit ASC
                "#,
            )
            .context("prepare usage summary query")
            .into_store()?;
        let rows = statement
            .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .context("query usage summary")
            .into_store()?;
        let mut totals = BTreeMap::new();
        for row in rows {
            let (meter, unit, quantity) = row.context("read usage summary row").into_store()?;
            totals.insert(meter, UsageTotal { quantity, unit });
        }
        Ok(UsageSummary {
            tenant_id,
            project_id,
            totals,
        })
    }
}

pub async fn record_usage_batch(
    store: &dyn UsageLedgerStore,
    inserts: Vec<UsageRecordInsert>,
) -> anyhow::Result<Vec<UsageRecord>> {
    let mut records = Vec::with_capacity(inserts.len());
    for insert in inserts {
        records.push(store.record_usage(insert).await?);
    }
    Ok(records)
}

pub fn judge_usage_from_outcome(outcome: &JudgeBrokerOutcome) -> UsageRecordInsert {
    let audit = &outcome.audit;
    UsageRecordInsert {
        tenant_id: audit.tenant_id.clone(),
        project_id: audit.project_id.clone(),
        meter: UsageMeter::JudgeCostMicros,
        quantity: audit.charged_cost.amount_micros,
        unit: money_unit(&audit.charged_cost),
        source_kind: UsageRecordSourceKind::JudgeCall,
        source_id: audit.judge_call_id.as_str().to_string(),
        attributes: json!({
            "provider": audit.provider,
            "provider_secret_id": audit.provider_secret_id,
            "model": audit.model,
            "evaluator_id": audit.evaluator_id,
            "request_hash": audit.request_hash,
            "response_hash": audit.response_hash,
            "score": audit.score,
            "provider_cost_micros": audit.provider_cost.amount_micros,
            "charged_cost_micros": audit.charged_cost.amount_micros,
            "currency": audit.charged_cost.currency.as_str(),
            "cached": audit.cached,
            "remaining_budget_micros": outcome.remaining_budget.amount_micros
        }),
    }
}

pub fn judge_usage_from_dataset_eval_report(report: &DatasetEvalReport) -> Vec<UsageRecordInsert> {
    report
        .results
        .iter()
        .filter_map(|result| {
            let cost = result.cost.as_ref()?;
            Some(UsageRecordInsert {
                tenant_id: report.tenant_id.clone(),
                project_id: report.project_id.clone(),
                meter: UsageMeter::JudgeCostMicros,
                quantity: cost.amount_micros,
                unit: money_unit(cost),
                source_kind: UsageRecordSourceKind::JudgeCall,
                source_id: result
                    .reproducibility
                    .judge_parameters
                    .get("judge_call_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| result.eval_result_id.as_str().to_string()),
                attributes: json!({
                    "report_id": report.report_id,
                    "eval_result_id": result.eval_result_id,
                    "dataset_id": report.dataset_id,
                    "dataset_version_id": report.dataset_version_id,
                    "dataset_case_id": result.reproducibility.dataset_case_id,
                    "evaluator_version_id": report.evaluator_version_id,
                    "agent_release_id": result.reproducibility.agent_release_id,
                    "trace_id": result.trace_id,
                    "span_id": result.span_id,
                    "judge_provider": result.reproducibility.judge_provider,
                    "judge_model_id": result.reproducibility.judge_model_id,
                    "judge_parameters": result.reproducibility.judge_parameters
                }),
            })
        })
        .collect()
}

pub fn judge_usage_from_experiment_report(report: &ExperimentRunReport) -> Vec<UsageRecordInsert> {
    report
        .case_scores
        .iter()
        .flat_map(|score| {
            [
                experiment_score_usage(report, score, ExperimentRole::Baseline),
                experiment_score_usage(report, score, ExperimentRole::Candidate),
            ]
            .into_iter()
            .flatten()
        })
        .collect()
}

fn experiment_score_usage(
    report: &ExperimentRunReport,
    score: &CaseExperimentScore,
    role: ExperimentRole,
) -> Option<UsageRecordInsert> {
    let (cost, judge_call_id, cached, release_id) = match role {
        ExperimentRole::Baseline => (
            score.baseline_cost.as_ref()?,
            score.baseline_judge_call_id.as_ref()?,
            score.baseline_cached,
            &report.baseline_release_id,
        ),
        ExperimentRole::Candidate => (
            score.candidate_cost.as_ref()?,
            score.candidate_judge_call_id.as_ref()?,
            score.candidate_cached,
            &report.candidate_release_id,
        ),
    };
    Some(UsageRecordInsert {
        tenant_id: report.tenant_id.clone(),
        project_id: report.project_id.clone(),
        meter: UsageMeter::JudgeCostMicros,
        quantity: cost.amount_micros,
        unit: money_unit(cost),
        source_kind: UsageRecordSourceKind::JudgeCall,
        source_id: judge_call_id.as_str().to_string(),
        attributes: json!({
            "experiment_run_id": report.experiment_run_id,
            "role": role.as_str(),
            "dataset_id": report.dataset_id,
            "dataset_version_id": report.dataset_version_id,
            "dataset_case_id": score.case_id,
            "agent_release_id": release_id,
            "evaluator_version_id": report.evaluator_version_id,
            "cached": cached
        }),
    })
}

fn money_unit(cost: &Money) -> String {
    format!("{}_micros", cost.currency.as_str().to_ascii_lowercase())
}

#[derive(Clone, Copy)]
enum ExperimentRole {
    Baseline,
    Candidate,
}

impl ExperimentRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Candidate => "candidate",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{
        AgentReleaseId, DatasetCaseId, DatasetId, DatasetVersionId, EvaluatorVersionId,
    };

    #[tokio::test]
    async fn sqlite_usage_records_are_idempotent_by_source() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let insert = UsageRecordInsert {
            tenant_id: TenantId::new("tenant")?,
            project_id: ProjectId::new("project")?,
            meter: UsageMeter::JudgeCostMicros,
            quantity: 25,
            unit: "usd_micros".to_string(),
            source_kind: UsageRecordSourceKind::JudgeCall,
            source_id: "judge-call-1".to_string(),
            attributes: json!({"cached": false}),
        };

        let first = store.record_usage(insert.clone()).await?;
        let second = store.record_usage(insert).await?;
        assert_eq!(first.usage_record_id, second.usage_record_id);

        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(records.len(), 1);

        let summary = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(
            summary.totals.get(UsageMeter::JudgeCostMicros.as_str()),
            Some(&UsageTotal {
                quantity: 25,
                unit: "usd_micros".to_string()
            })
        );
        Ok(())
    }

    #[test]
    fn experiment_usage_keeps_zero_cost_cached_calls_for_audit() -> anyhow::Result<()> {
        let report = ExperimentRunReport {
            experiment_run_id: beater_core::ExperimentRunId::new("run-1")?,
            tenant_id: TenantId::new("tenant")?,
            project_id: ProjectId::new("project")?,
            dataset_id: DatasetId::new("dataset")?,
            dataset_version_id: DatasetVersionId::new("version")?,
            baseline_release_id: AgentReleaseId::new("baseline")?,
            candidate_release_id: AgentReleaseId::new("candidate")?,
            evaluator_version_id: EvaluatorVersionId::new("judge-v1")?,
            case_scores: vec![CaseExperimentScore {
                case_id: DatasetCaseId::new("case-1")?,
                baseline_output: json!("old"),
                candidate_output: json!("new"),
                baseline_trace: None,
                candidate_trace: None,
                reference: None,
                baseline_score: 0.0,
                candidate_score: 1.0,
                delta: 1.0,
                baseline_evidence: json!({}),
                candidate_evidence: json!({}),
                baseline_cost: Some(Money::usd_micros(25)),
                candidate_cost: Some(Money::usd_micros(0)),
                baseline_judge_call_id: Some(beater_core::JudgeCallId::new("judge-call-a")?),
                candidate_judge_call_id: Some(beater_core::JudgeCallId::new("judge-call-b")?),
                baseline_cached: Some(false),
                candidate_cached: Some(true),
            }],
            comparison: beater_eval::ExperimentComparison {
                sample_size: 1,
                baseline_mean: 0.0,
                candidate_mean: 1.0,
                delta: 1.0,
                ci_low: 1.0,
                ci_high: 1.0,
                p_value: 1.0,
                decision: beater_eval::GateDecision::Pass,
                test: beater_eval::StatisticalTest::PairedT,
                adjusted_alpha: 0.05,
            },
            decision: beater_eval::GateDecision::Pass,
            gate_policy: beater_eval::GatePolicy::default(),
            created_at: Utc::now(),
        };

        let inserts = judge_usage_from_experiment_report(&report);
        assert_eq!(inserts.len(), 2);
        assert_eq!(inserts[0].quantity, 25);
        assert_eq!(inserts[1].quantity, 0);
        assert_eq!(inserts[1].source_id, "judge-call-b");
        Ok(())
    }
}
