use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{Money, ProjectId, TenantId, Timestamp, UsageRecordId};
use beater_datasets::DatasetEvalReport;
use beater_experiments::{CaseExperimentScore, ExperimentRunReport};
use beater_judge::JudgeBrokerOutcome;
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
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
    IngestSpans,
    IngestBytes,
    StoredArtifactBytes,
    EvalCasesScored,
}

impl UsageMeter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JudgeCostMicros => "judge_cost_micros",
            Self::IngestSpans => "ingest_spans",
            Self::IngestBytes => "ingest_bytes",
            Self::StoredArtifactBytes => "stored_artifact_bytes",
            Self::EvalCasesScored => "eval_cases_scored",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageRecordSourceKind {
    JudgeCall,
    TraceIngest,
    ArtifactStore,
    DatasetEvalReport,
    ExperimentRun,
    Manual,
    /// A compensating entry that credits (refunds/voids) a prior charge.
    ///
    /// The ledger is append-only: corrections are never expressed by mutating or
    /// deleting an existing row. Instead a `Refund` record is appended whose
    /// `quantity` is typically negative so that the rollup nets out the original
    /// charge. `Refund` is the only source kind permitted to carry a negative
    /// quantity (see [`SqliteUsageLedger::record_usage`]).
    Refund,
}

impl UsageRecordSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JudgeCall => "judge_call",
            Self::TraceIngest => "trace_ingest",
            Self::ArtifactStore => "artifact_store",
            Self::DatasetEvalReport => "dataset_eval_report",
            Self::ExperimentRun => "experiment_run",
            Self::Manual => "manual",
            Self::Refund => "refund",
        }
    }

    /// Whether this source kind represents a compensating (crediting) entry that
    /// may carry a negative quantity.
    pub fn is_compensating(self) -> bool {
        matches!(self, Self::Refund)
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

/// Billing-grade usage/metering ledger.
///
/// # Invariants
///
/// * **Append-only.** Rows are only ever inserted. There is no `UPDATE` or
///   `DELETE` path anywhere in this trait or its implementations. Corrections,
///   refunds and voids are expressed as *compensating entries*: a new row (see
///   [`UsageRecordSourceKind::Refund`]) whose quantity nets against the original
///   charge in the rollup. The original charge row is never touched.
/// * **Exactly-once recording.** `(tenant_id, project_id, meter, source_kind,
///   source_id)` is unique. Re-recording the same dedup tuple is idempotent and
///   returns the *canonical stored row* (same `usage_record_id`, same stored
///   quantity), never a freshly generated one.
/// * **Overflow-safe rollups.** Quantities are folded into a per-`(meter, unit)`
///   rollup that is maintained transactionally on each append, so
///   [`summarize_usage`](Self::summarize_usage) reads O(distinct meter x unit)
///   buckets instead of scanning the whole ledger. The fold uses checked
///   arithmetic: an addition that would overflow `i64` returns a typed error
///   (rolling the append back) rather than wrapping or promoting to a float, and
///   summarize still rejects mixed units within a single meter.
#[async_trait]
pub trait UsageLedgerStore: Send + Sync {
    /// Append a usage record, idempotently on its dedup tuple.
    ///
    /// On a duplicate dedup tuple this is a no-op write and returns the existing
    /// canonical stored row rather than the in-memory candidate.
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

                -- Incremental rollup of usage_records, maintained transactionally
                -- on each *new* append (see record_usage). It exists so that
                -- summarize_usage reads O(distinct meter x unit) rows instead of
                -- scanning every usage row for the tenant/project (#181). It is
                -- derived state: it can always be rebuilt from usage_records by
                -- folding quantities per (meter, unit).
                CREATE TABLE IF NOT EXISTS usage_rollups (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    meter TEXT NOT NULL,
                    unit TEXT NOT NULL,
                    total_quantity INTEGER NOT NULL,
                    updated_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, meter, unit)
                );
                "#,
            )
            .context("initialize sqlite usage ledger")?;
        // Backfill the rollup for ledgers written before it existed: if the
        // rollup is empty but records exist, this is a pre-#181 database, so fold
        // the historical rows in once. A net-zero meter still leaves a (zero)
        // rollup row, so "rollup empty AND records present" reliably means
        // "never rolled up", not "legitimately nets to zero".
        Self::backfill_rollups(&connection)?;
        Ok(())
    }

    /// One-time migration: populate `usage_rollups` from existing `usage_records`
    /// when the rollup table is empty but records already exist. Folds quantities
    /// in Rust with checked arithmetic (SQLite's `SUM` silently promotes to float
    /// on i64 overflow, which would corrupt money), mirroring `summarize_usage`.
    fn backfill_rollups(connection: &Connection) -> anyhow::Result<()> {
        let rollup_rows: i64 = connection
            .query_row("SELECT COUNT(*) FROM usage_rollups", [], |row| row.get(0))
            .context("count usage rollups")?;
        if rollup_rows > 0 {
            return Ok(());
        }
        let record_rows: i64 = connection
            .query_row("SELECT COUNT(*) FROM usage_records", [], |row| row.get(0))
            .context("count usage records")?;
        if record_rows == 0 {
            return Ok(());
        }

        // (tenant, project, meter, unit) -> running total, folded with checked_add.
        let mut totals: BTreeMap<(String, String, String, String), i64> = BTreeMap::new();
        let mut statement = connection
            .prepare(
                r#"
                SELECT tenant_id, project_id, meter, unit, quantity
                FROM usage_records
                "#,
            )
            .context("prepare usage rollup backfill scan")?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .context("scan usage records for rollup backfill")?;
        for row in rows {
            let (tenant, project, meter, unit, quantity) =
                row.context("read usage record row for backfill")?;
            let entry = totals.entry((tenant, project, meter, unit)).or_insert(0);
            *entry = entry.checked_add(quantity).ok_or_else(|| {
                anyhow!("usage rollup backfill overflow folding historical quantities")
            })?;
        }

        let now = Utc::now().to_rfc3339();
        for ((tenant, project, meter, unit), total) in totals {
            connection
                .execute(
                    r#"
                    INSERT INTO usage_rollups
                      (tenant_id, project_id, meter, unit, total_quantity, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                    "#,
                    params![tenant, project, meter, unit, total, now],
                )
                .context("insert backfilled usage rollup")?;
        }
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

    /// Folds `quantity` into the `(tenant, project, meter, unit)` rollup bucket
    /// with checked arithmetic, inserting the bucket if absent. Overflow surfaces
    /// as a typed [`StoreError::Integrity`] rather than wrapping or promoting to a
    /// float; the caller's transaction is left uncommitted so the originating
    /// append rolls back too, keeping the rollup an exact running total. Refund
    /// rows (negative quantities) net the bucket downward.
    fn bump_rollup(
        connection: &Connection,
        tenant_id: &str,
        project_id: &str,
        meter: &str,
        unit: &str,
        quantity: i64,
    ) -> StoreResult<()> {
        let current: Option<i64> = connection
            .query_row(
                r#"
                SELECT total_quantity
                FROM usage_rollups
                WHERE tenant_id = ?1 AND project_id = ?2 AND meter = ?3 AND unit = ?4
                "#,
                params![tenant_id, project_id, meter, unit],
                |row| row.get(0),
            )
            .optional()
            .context("read usage rollup bucket")
            .into_store()?;
        let next = match current {
            Some(total) => total.checked_add(quantity).ok_or_else(|| {
                StoreError::Integrity(format!(
                    "usage rollup overflow summing quantities for meter {meter}"
                ))
            })?,
            None => quantity,
        };
        connection
            .execute(
                r#"
                INSERT INTO usage_rollups
                  (tenant_id, project_id, meter, unit, total_quantity, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(tenant_id, project_id, meter, unit) DO UPDATE SET
                  total_quantity = excluded.total_quantity,
                  updated_at = excluded.updated_at
                "#,
                params![
                    tenant_id,
                    project_id,
                    meter,
                    unit,
                    next,
                    Utc::now().to_rfc3339()
                ],
            )
            .context("upsert usage rollup bucket")
            .into_store()?;
        Ok(())
    }
}

#[async_trait]
impl UsageLedgerStore for SqliteUsageLedger {
    async fn record_usage(&self, insert: UsageRecordInsert) -> StoreResult<UsageRecord> {
        // Only compensating entries (refunds/voids) may carry a negative
        // quantity. A negative charge on any other source kind would silently
        // drain the ledger and is rejected as an integrity violation; the
        // correct way to credit a charge is to append a `Refund` record.
        if insert.quantity < 0 && !insert.source_kind.is_compensating() {
            return Err(StoreError::Integrity(format!(
                "usage quantity must be non-negative for meter {} (source kind {}); \
                 record a refund compensating entry instead",
                insert.meter.as_str(),
                insert.source_kind.as_str()
            )));
        }
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
        // Insert-or-ignore and the canonical read-back run inside a single
        // transaction so the returned row is exactly what is durably stored,
        // even under concurrent identical inserts: the loser of the dedup race
        // still reads back the winner's canonical row.
        let mut connection = self.lock().into_store()?;
        let transaction = connection
            .transaction()
            .context("begin usage record transaction")
            .into_store()?;
        let inserted = transaction
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
        // Maintain the incremental rollup, but only when this call actually
        // appended a new row. On a dedup no-op (`INSERT OR IGNORE` ignored, 0 rows
        // changed) the quantity was already folded by the original append, so
        // re-applying it would double-count. The connection is mutex-serialized,
        // so this read-modify-write of the rollup is atomic with the append.
        if inserted == 1 {
            Self::bump_rollup(
                &transaction,
                record.tenant_id.as_str(),
                record.project_id.as_str(),
                record.meter.as_str(),
                &record.unit,
                record.quantity,
            )?;
        }
        let stored = Self::select_by_unique(
            &transaction,
            &record.tenant_id,
            &record.project_id,
            record.meter,
            record.source_kind,
            &record.source_id,
        )
        .into_store()?;
        transaction
            .commit()
            .context("commit usage record transaction")
            .into_store()?;
        Ok(stored)
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
        // Read the pre-aggregated rollup instead of scanning every usage row
        // (#181): this returns O(distinct meter x unit) buckets, not O(rows). The
        // rollup is maintained transactionally on each append (see `bump_rollup`),
        // so it is an exact running total. Each bucket is one (meter, unit), so a
        // meter that ever recorded mixed units yields two buckets and surfaces as
        // a typed mismatch error here — preserving the invariant that a meter is
        // never silently summed across currencies/units.
        let mut statement = connection
            .prepare(
                r#"
                SELECT meter, unit, total_quantity
                FROM usage_rollups
                WHERE tenant_id = ?1 AND project_id = ?2
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
        let mut totals: BTreeMap<String, UsageTotal> = BTreeMap::new();
        for row in rows {
            let (meter, unit, quantity) = row.context("read usage summary row").into_store()?;
            match totals.get_mut(&meter) {
                None => {
                    totals.insert(meter, UsageTotal { quantity, unit });
                }
                Some(total) => {
                    if total.unit != unit {
                        return Err(StoreError::Integrity(format!(
                            "usage rollup unit mismatch for meter {meter}: \
                             cannot sum {} and {}",
                            total.unit, unit
                        )));
                    }
                    total.quantity = total.quantity.checked_add(quantity).ok_or_else(|| {
                        StoreError::Integrity(format!(
                            "usage rollup overflow summing quantities for meter {meter}"
                        ))
                    })?;
                }
            }
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
        // A second insert on the same dedup tuple, even with a *different*
        // candidate quantity, must return the canonical stored row unchanged.
        let mut conflicting = insert.clone();
        conflicting.quantity = 9_999;
        let second = store.record_usage(conflicting).await?;
        assert_eq!(first.usage_record_id, second.usage_record_id);
        assert_eq!(first.quantity, second.quantity);
        assert_eq!(second.quantity, 25);
        assert_eq!(first, second);

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

    #[tokio::test]
    async fn sqlite_usage_rejects_negative_quantities() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let insert = UsageRecordInsert {
            tenant_id: TenantId::new("tenant")?,
            project_id: ProjectId::new("project")?,
            meter: UsageMeter::JudgeCostMicros,
            quantity: -1,
            unit: "usd_micros".to_string(),
            source_kind: UsageRecordSourceKind::Manual,
            source_id: "bad-adjustment".to_string(),
            attributes: json!({}),
        };

        let error = store
            .record_usage(insert)
            .await
            .err()
            .ok_or_else(|| anyhow!("negative usage quantity should be rejected"))?;
        assert!(matches!(
            error,
            StoreError::Integrity(message)
                if message.contains("usage quantity must be non-negative")
                    && message.contains("judge_cost_micros")
        ));

        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert!(records.is_empty());
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
                mde: None,
                required_n: None,
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

    fn charge(source_id: &str, quantity: i64) -> anyhow::Result<UsageRecordInsert> {
        Ok(UsageRecordInsert {
            tenant_id: TenantId::new("tenant")?,
            project_id: ProjectId::new("project")?,
            meter: UsageMeter::JudgeCostMicros,
            quantity,
            unit: "usd_micros".to_string(),
            source_kind: UsageRecordSourceKind::JudgeCall,
            source_id: source_id.to_string(),
            attributes: json!({}),
        })
    }

    fn summary_quantity(summary: &UsageSummary) -> Option<i64> {
        summary
            .totals
            .get(UsageMeter::JudgeCostMicros.as_str())
            .map(|total| total.quantity)
    }

    #[tokio::test]
    async fn sqlite_usage_summarizes_ingest_storage_and_eval_meters() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let inserts = [
            UsageRecordInsert {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                meter: UsageMeter::IngestSpans,
                quantity: 7,
                unit: "spans".to_string(),
                source_kind: UsageRecordSourceKind::TraceIngest,
                source_id: "trace-a".to_string(),
                attributes: json!({"environment_id": "prod"}),
            },
            UsageRecordInsert {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                meter: UsageMeter::IngestBytes,
                quantity: 2048,
                unit: "bytes".to_string(),
                source_kind: UsageRecordSourceKind::TraceIngest,
                source_id: "raw-envelope-a".to_string(),
                attributes: json!({"content_type": "application/x-protobuf"}),
            },
            UsageRecordInsert {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                meter: UsageMeter::StoredArtifactBytes,
                quantity: 4096,
                unit: "bytes".to_string(),
                source_kind: UsageRecordSourceKind::ArtifactStore,
                source_id: "artifact-a".to_string(),
                attributes: json!({"redaction_class": "sensitive"}),
            },
            UsageRecordInsert {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                meter: UsageMeter::EvalCasesScored,
                quantity: 3,
                unit: "cases".to_string(),
                source_kind: UsageRecordSourceKind::DatasetEvalReport,
                source_id: "report-a".to_string(),
                attributes: json!({"dataset_id": "dataset-a"}),
            },
        ];

        for insert in inserts {
            store.record_usage(insert).await?;
        }

        let summary = store.summarize_usage(tenant, project).await?;
        assert_eq!(
            summary.totals.get(UsageMeter::IngestSpans.as_str()),
            Some(&UsageTotal {
                quantity: 7,
                unit: "spans".to_string(),
            })
        );
        assert_eq!(
            summary.totals.get(UsageMeter::IngestBytes.as_str()),
            Some(&UsageTotal {
                quantity: 2048,
                unit: "bytes".to_string(),
            })
        );
        assert_eq!(
            summary.totals.get(UsageMeter::StoredArtifactBytes.as_str()),
            Some(&UsageTotal {
                quantity: 4096,
                unit: "bytes".to_string(),
            })
        );
        assert_eq!(
            summary.totals.get(UsageMeter::EvalCasesScored.as_str()),
            Some(&UsageTotal {
                quantity: 3,
                unit: "cases".to_string(),
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn zero_quantity_is_recorded_for_audit() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let stored = store.record_usage(charge("cached-call", 0)?).await?;
        assert_eq!(stored.quantity, 0);

        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(records.len(), 1);

        let summary = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(0));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn concurrent_identical_inserts_yield_exactly_one_row() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let insert = charge("judge-call-race", 25)?;

        let mut handles = Vec::new();
        for _ in 0..50 {
            let store = store.clone();
            let insert = insert.clone();
            handles.push(tokio::spawn(
                async move { store.record_usage(insert).await },
            ));
        }

        let mut results = Vec::new();
        for handle in handles {
            let record = handle
                .await
                .map_err(|err| anyhow!("join error: {err}"))?
                .map_err(|err| anyhow!("record_usage error: {err}"))?;
            results.push(record);
        }

        let canonical = results
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("expected at least one result"))?;
        for record in &results {
            assert_eq!(
                record, &canonical,
                "all concurrent winners must be identical"
            );
        }

        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0], canonical);

        // The rollup must reflect the charge applied exactly once (25), not once
        // per concurrent winner: a dedup no-op append must not re-fold the
        // quantity into the rollup.
        let summary = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(25));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn concurrent_distinct_inserts_yield_exactly_n_rows() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        const N: i64 = 50;

        let mut handles = Vec::new();
        for index in 0..N {
            let store = store.clone();
            let insert = charge(&format!("judge-call-{index}"), index + 1)?;
            handles.push(tokio::spawn(
                async move { store.record_usage(insert).await },
            ));
        }
        for handle in handles {
            handle
                .await
                .map_err(|err| anyhow!("join error: {err}"))?
                .map_err(|err| anyhow!("record_usage error: {err}"))?;
        }

        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(records.len(), N as usize);

        // Sum 1..=N == N*(N+1)/2.
        let expected = N * (N + 1) / 2;
        let summary = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(expected));
        Ok(())
    }

    #[tokio::test]
    async fn refund_is_a_compensating_append_only_entry() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let original = store.record_usage(charge("judge-call-1", 1_000)?).await?;

        let refund = UsageRecordInsert {
            tenant_id: TenantId::new("tenant")?,
            project_id: ProjectId::new("project")?,
            meter: UsageMeter::JudgeCostMicros,
            quantity: -400,
            unit: "usd_micros".to_string(),
            source_kind: UsageRecordSourceKind::Refund,
            source_id: "refund-of-judge-call-1".to_string(),
            attributes: json!({"refunds": "judge-call-1"}),
        };
        let refund_record = store.record_usage(refund).await?;
        assert_eq!(refund_record.quantity, -400);

        // Both rows present; the original charge is untouched.
        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(records.len(), 2);
        let stored_original = records
            .iter()
            .find(|record| record.source_id == "judge-call-1")
            .ok_or_else(|| anyhow!("original charge row missing"))?;
        assert_eq!(stored_original, &original);

        // Net rollup reflects the compensating entry.
        let summary = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(600));
        Ok(())
    }

    #[tokio::test]
    async fn non_refund_negative_quantity_is_rejected() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        let error = store
            .record_usage(charge("bad", -1)?)
            .await
            .err()
            .ok_or_else(|| anyhow!("negative non-refund quantity should be rejected"))?;
        assert!(matches!(
            error,
            StoreError::Integrity(message)
                if message.contains("usage quantity must be non-negative")
        ));
        Ok(())
    }

    #[tokio::test]
    async fn rollup_overflow_returns_typed_error_not_panic() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        store.record_usage(charge("near-max", i64::MAX)?).await?;

        // The rollup is now maintained incrementally on append, so an addition
        // that would overflow the i64 running total fails fast at write time with
        // a typed error (never a panic or silent float promotion) and rolls the
        // append back rather than surfacing only later at summarize.
        let error = store
            .record_usage(charge("one-more", 1)?)
            .await
            .err()
            .ok_or_else(|| anyhow!("overflowing rollup should error"))?;
        assert!(matches!(
            &error,
            StoreError::Integrity(message) if message.contains("overflow")
        ));

        // The overflowing append rolled back: the record is absent and the rollup
        // still reflects only the first charge.
        let records = store
            .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(records.len(), 1, "overflowing append must not persist");
        let summary = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(i64::MAX));
        Ok(())
    }

    #[tokio::test]
    async fn rollup_unit_mismatch_returns_typed_error() -> anyhow::Result<()> {
        let store = SqliteUsageLedger::in_memory()?;
        store.record_usage(charge("usd-call", 100)?).await?;
        let mut other_currency = charge("eur-call", 100)?;
        other_currency.unit = "eur_micros".to_string();
        store.record_usage(other_currency).await?;

        let error = store
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await
            .err()
            .ok_or_else(|| anyhow!("mixed units in a meter should error"))?;
        assert!(matches!(
            error,
            StoreError::Integrity(message) if message.contains("unit mismatch")
        ));
        Ok(())
    }

    #[tokio::test]
    async fn backfills_rollup_for_pre_existing_ledger_on_open() -> anyhow::Result<()> {
        let mut path = std::env::temp_dir();
        path.push(format!("beater-usage-backfill-{}.sqlite", Uuid::new_v4()));

        // Write a ledger, then simulate a database created before the rollup
        // table existed by dropping it directly. Reopening must rebuild the
        // rollup from the surviving records (including the netting refund).
        {
            let store = SqliteUsageLedger::open(&path)?;
            store.record_usage(charge("call-1", 100)?).await?;
            store.record_usage(charge("call-2", 250)?).await?;
            let refund = UsageRecordInsert {
                tenant_id: TenantId::new("tenant")?,
                project_id: ProjectId::new("project")?,
                meter: UsageMeter::JudgeCostMicros,
                quantity: -50,
                unit: "usd_micros".to_string(),
                source_kind: UsageRecordSourceKind::Refund,
                source_id: "refund-call-1".to_string(),
                attributes: json!({}),
            };
            store.record_usage(refund).await?;
        }
        {
            let connection = Connection::open(&path)?;
            connection.execute_batch("DROP TABLE usage_rollups;")?;
        }

        let reopened = SqliteUsageLedger::open(&path)?;
        let summary = reopened
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(300));

        // A fresh append after backfill keeps folding into the rebuilt rollup.
        reopened.record_usage(charge("call-3", 5)?).await?;
        let summary = reopened
            .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(summary_quantity(&summary), Some(305));

        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
        Ok(())
    }

    // Table-driven property test (proptest is not a workspace dependency).
    // For any sequence of inserts with repeats, the final row set equals the set
    // of distinct dedup tuples (keeping the first quantity per tuple), and the
    // rollup equals the checked sum of those distinct quantities.
    #[tokio::test]
    async fn property_distinct_tuples_define_rows_and_rollup() -> anyhow::Result<()> {
        use std::collections::BTreeMap as Map;

        // Deterministic LCG so the test is reproducible without a PRNG crate.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state
        };

        for trial in 0..32 {
            let store = SqliteUsageLedger::in_memory()?;
            let mut first_quantity: Map<String, i64> = Map::new();

            let ops = 20 + (next() % 40);
            for _ in 0..ops {
                let source_id = format!("src-{}", next() % 7);
                let quantity = (next() % 1_000) as i64;
                store.record_usage(charge(&source_id, quantity)?).await?;
                first_quantity.entry(source_id).or_insert(quantity);
            }

            let records = store
                .list_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
                .await?;
            assert_eq!(
                records.len(),
                first_quantity.len(),
                "trial {trial}: row count must equal distinct dedup tuples"
            );
            for record in &records {
                let expected = first_quantity
                    .get(&record.source_id)
                    .ok_or_else(|| anyhow!("unexpected source {}", record.source_id))?;
                assert_eq!(
                    &record.quantity, expected,
                    "trial {trial}: stored quantity must be the first inserted"
                );
            }

            let mut expected_sum: i64 = 0;
            for quantity in first_quantity.values() {
                expected_sum = expected_sum
                    .checked_add(*quantity)
                    .ok_or_else(|| anyhow!("test sum overflow"))?;
            }
            let summary = store
                .summarize_usage(TenantId::new("tenant")?, ProjectId::new("project")?)
                .await?;
            let rollup = if first_quantity.is_empty() {
                None
            } else {
                Some(expected_sum)
            };
            assert_eq!(
                summary_quantity(&summary),
                rollup,
                "trial {trial}: rollup must equal checked sum of distinct quantities"
            );
        }
        Ok(())
    }
}
