use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    AgentReleaseId, DatasetId, EvaluatorVersionId, ExperimentRunId, GateId, GateRunId, ProjectId,
    TenantId, Timestamp,
};
use beater_eval::{ExperimentComparison, GateDecision, GatePolicy};
use beater_experiments::{ExperimentRunReport, ExperimentStore};
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InconclusivePolicy {
    Pass,
    #[default]
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GateDefinition {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub gate_id: GateId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<DatasetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluator_version_id: Option<EvaluatorVersionId>,
    #[serde(default)]
    pub inconclusive_policy: InconclusivePolicy,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GateRunReport {
    pub gate_run_id: GateRunId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub gate_id: GateId,
    pub gate_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_dataset_id: Option<DatasetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_evaluator_version_id: Option<EvaluatorVersionId>,
    pub inconclusive_policy: InconclusivePolicy,
    pub experiment_run_id: ExperimentRunId,
    pub dataset_id: DatasetId,
    pub evaluator_version_id: EvaluatorVersionId,
    pub baseline_release_id: AgentReleaseId,
    pub candidate_release_id: AgentReleaseId,
    pub experiment_decision: GateDecision,
    pub experiment_gate_policy: GatePolicy,
    pub passed: bool,
    pub reason: String,
    pub comparison: ExperimentComparison,
    #[schema(value_type = String, format = DateTime)]
    pub experiment_created_at: Timestamp,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

#[async_trait]
pub trait GateStore: Send + Sync {
    async fn put_gate(&self, gate: GateDefinition) -> StoreResult<GateDefinition>;

    async fn get_gate(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        gate_id: GateId,
    ) -> StoreResult<GateDefinition>;

    async fn write_run(&self, report: GateRunReport) -> StoreResult<GateRunReport>;

    async fn latest_run(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        gate_id: GateId,
    ) -> StoreResult<Option<GateRunReport>>;
}

#[derive(Clone)]
pub struct SqliteGateStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteGateStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory gate sqlite")?;
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
                .with_context(|| format!("create gate sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite gate store {}", path.display()))?;
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

                CREATE TABLE IF NOT EXISTS gates (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    gate_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    dataset_id TEXT,
                    evaluator_version_id TEXT,
                    inconclusive_policy TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    definition_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, gate_id)
                );

                CREATE TABLE IF NOT EXISTS gate_runs (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    gate_run_id TEXT NOT NULL,
                    gate_id TEXT NOT NULL,
                    experiment_run_id TEXT NOT NULL,
                    experiment_decision TEXT NOT NULL,
                    passed INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    report_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, gate_run_id)
                );

                CREATE INDEX IF NOT EXISTS idx_gate_runs_latest
                  ON gate_runs (tenant_id, project_id, gate_id, created_at DESC, gate_run_id DESC);
                "#,
            )
            .context("initialize sqlite gate store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("sqlite gate connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl GateStore for SqliteGateStore {
    async fn put_gate(&self, gate: GateDefinition) -> StoreResult<GateDefinition> {
        let definition_json = serde_json::to_string(&gate)
            .context("serialize gate definition")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO gates
                  (tenant_id, project_id, gate_id, name, dataset_id, evaluator_version_id,
                   inconclusive_policy, created_at, definition_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(tenant_id, project_id, gate_id) DO UPDATE SET
                  name = excluded.name,
                  dataset_id = excluded.dataset_id,
                  evaluator_version_id = excluded.evaluator_version_id,
                  inconclusive_policy = excluded.inconclusive_policy,
                  created_at = excluded.created_at,
                  definition_json = excluded.definition_json
                "#,
                params![
                    gate.tenant_id.as_str(),
                    gate.project_id.as_str(),
                    gate.gate_id.as_str(),
                    gate.name.as_str(),
                    gate.dataset_id.as_ref().map(|id| id.as_str()),
                    gate.evaluator_version_id.as_ref().map(|id| id.as_str()),
                    inconclusive_policy_name(&gate.inconclusive_policy),
                    gate.created_at.to_rfc3339(),
                    definition_json,
                ],
            )
            .context("upsert gate definition")
            .into_store()?;
        Ok(gate)
    }

    async fn get_gate(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        gate_id: GateId,
    ) -> StoreResult<GateDefinition> {
        let connection = self.lock().into_store()?;
        let definition_json = connection
            .query_row(
                r#"
                SELECT definition_json
                FROM gates
                WHERE tenant_id = ?1 AND project_id = ?2 AND gate_id = ?3
                "#,
                params![tenant_id.as_str(), project_id.as_str(), gate_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query gate definition")
            .into_store()?
            .ok_or_else(|| StoreError::NotFound(format!("gate {} not found", gate_id.as_str())))?;
        serde_json::from_str(&definition_json)
            .context("decode gate definition")
            .into_store()
    }

    async fn write_run(&self, report: GateRunReport) -> StoreResult<GateRunReport> {
        let report_json = serde_json::to_string(&report)
            .context("serialize gate run report")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO gate_runs
                  (tenant_id, project_id, gate_run_id, gate_id, experiment_run_id,
                   experiment_decision, passed, created_at, report_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    report.tenant_id.as_str(),
                    report.project_id.as_str(),
                    report.gate_run_id.as_str(),
                    report.gate_id.as_str(),
                    report.experiment_run_id.as_str(),
                    report.experiment_decision.name(),
                    if report.passed { 1_i64 } else { 0_i64 },
                    report.created_at.to_rfc3339(),
                    report_json,
                ],
            )
            .context("insert gate run")
            .into_store()?;
        Ok(report)
    }

    async fn latest_run(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        gate_id: GateId,
    ) -> StoreResult<Option<GateRunReport>> {
        let connection = self.lock().into_store()?;
        let report_json = connection
            .query_row(
                r#"
                SELECT report_json
                FROM gate_runs
                WHERE tenant_id = ?1 AND project_id = ?2 AND gate_id = ?3
                ORDER BY created_at DESC, gate_run_id DESC
                LIMIT 1
                "#,
                params![tenant_id.as_str(), project_id.as_str(), gate_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query latest gate run")
            .into_store()?;
        report_json
            .map(|report_json| serde_json::from_str(&report_json).context("decode gate run report"))
            .transpose()
            .into_store()
    }
}

pub async fn run_gate(
    gate_store: &dyn GateStore,
    experiment_store: &dyn ExperimentStore,
    tenant_id: TenantId,
    project_id: ProjectId,
    gate_id: GateId,
    experiment_run_id: Option<ExperimentRunId>,
) -> anyhow::Result<GateRunReport> {
    let gate = gate_store
        .get_gate(tenant_id.clone(), project_id.clone(), gate_id)
        .await?;
    let experiment = match experiment_run_id {
        Some(experiment_run_id) => {
            experiment_store
                .get_run(tenant_id, project_id, experiment_run_id)
                .await?
        }
        None => experiment_store
            .latest_run(
                tenant_id,
                project_id,
                gate.dataset_id.clone(),
                gate.evaluator_version_id.clone(),
            )
            .await?
            .ok_or_else(|| anyhow!("no experiment run found for gate {}", gate.gate_id.as_str()))?,
    };
    let report = evaluate_gate(&gate, &experiment)?;
    Ok(gate_store.write_run(report).await?)
}

pub fn evaluate_gate(
    gate: &GateDefinition,
    experiment: &ExperimentRunReport,
) -> anyhow::Result<GateRunReport> {
    if gate.tenant_id != experiment.tenant_id || gate.project_id != experiment.project_id {
        return Err(anyhow!(
            "gate {} cannot evaluate experiment {} from another tenant/project",
            gate.gate_id.as_str(),
            experiment.experiment_run_id.as_str()
        ));
    }
    if let Some(dataset_id) = &gate.dataset_id {
        if dataset_id != &experiment.dataset_id {
            return Err(anyhow!(
                "gate {} expected dataset {}, experiment {} used {}",
                gate.gate_id.as_str(),
                dataset_id.as_str(),
                experiment.experiment_run_id.as_str(),
                experiment.dataset_id.as_str()
            ));
        }
    }
    if let Some(evaluator_version_id) = &gate.evaluator_version_id {
        if evaluator_version_id != &experiment.evaluator_version_id {
            return Err(anyhow!(
                "gate {} expected evaluator {}, experiment {} used {}",
                gate.gate_id.as_str(),
                evaluator_version_id.as_str(),
                experiment.experiment_run_id.as_str(),
                experiment.evaluator_version_id.as_str()
            ));
        }
    }

    let (passed, reason) = gate_result(&experiment.decision, &gate.inconclusive_policy, experiment);
    Ok(GateRunReport {
        gate_run_id: GateRunId::new(Uuid::new_v4().to_string())?,
        tenant_id: experiment.tenant_id.clone(),
        project_id: experiment.project_id.clone(),
        gate_id: gate.gate_id.clone(),
        gate_name: gate.name.clone(),
        gate_dataset_id: gate.dataset_id.clone(),
        gate_evaluator_version_id: gate.evaluator_version_id.clone(),
        inconclusive_policy: gate.inconclusive_policy.clone(),
        experiment_run_id: experiment.experiment_run_id.clone(),
        dataset_id: experiment.dataset_id.clone(),
        evaluator_version_id: experiment.evaluator_version_id.clone(),
        baseline_release_id: experiment.baseline_release_id.clone(),
        candidate_release_id: experiment.candidate_release_id.clone(),
        experiment_decision: experiment.decision.clone(),
        experiment_gate_policy: experiment.gate_policy.clone(),
        passed,
        reason,
        comparison: experiment.comparison.clone(),
        experiment_created_at: experiment.created_at,
        created_at: Utc::now(),
    })
}

fn gate_result(
    decision: &GateDecision,
    _inconclusive_policy: &InconclusivePolicy,
    experiment: &ExperimentRunReport,
) -> (bool, String) {
    match decision {
        GateDecision::Pass => (
            true,
            format!(
                "experiment {} passed with delta {}",
                experiment.experiment_run_id.as_str(),
                experiment.comparison.delta
            ),
        ),
        GateDecision::FailRegression => (
            false,
            format!(
                "experiment {} failed regression with delta {}",
                experiment.experiment_run_id.as_str(),
                experiment.comparison.delta
            ),
        ),
        GateDecision::Inconclusive => (
            false,
            format!(
                "experiment {} was inconclusive; deploy gates never pass inconclusive results",
                experiment.experiment_run_id.as_str()
            ),
        ),
    }
}

fn inconclusive_policy_name(policy: &InconclusivePolicy) -> &'static str {
    match policy {
        InconclusivePolicy::Pass => "pass",
        InconclusivePolicy::Fail => "fail",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_eval::{ExperimentComparison, StatisticalTest};

    #[tokio::test]
    async fn gate_run_fails_regression_and_persists_report() -> anyhow::Result<()> {
        let gates = SqliteGateStore::in_memory()?;
        let experiments = beater_experiments::SqliteExperimentStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let dataset = DatasetId::new("dataset")?;
        let evaluator = EvaluatorVersionId::new("exact-v1")?;
        let gate = gates
            .put_gate(GateDefinition {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                gate_id: GateId::new("main")?,
                name: "main".to_string(),
                dataset_id: Some(dataset.clone()),
                evaluator_version_id: Some(evaluator.clone()),
                inconclusive_policy: InconclusivePolicy::Fail,
                created_at: Utc::now(),
            })
            .await?;
        let experiment = experiments
            .write_run(experiment_report(
                tenant.clone(),
                project.clone(),
                "exp-1",
                dataset,
                evaluator,
                GateDecision::FailRegression,
                -0.25,
            )?)
            .await?;

        let report = run_gate(
            &gates,
            &experiments,
            tenant.clone(),
            project.clone(),
            gate.gate_id.clone(),
            None,
        )
        .await?;

        assert!(!report.passed);
        assert_eq!(report.experiment_run_id, experiment.experiment_run_id);
        assert_eq!(
            gates
                .latest_run(tenant, project, gate.gate_id)
                .await?
                .map(|report| report.gate_run_id),
            Some(report.gate_run_id)
        );
        Ok(())
    }

    #[tokio::test]
    async fn gate_rejects_mismatched_explicit_experiment() -> anyhow::Result<()> {
        let gates = SqliteGateStore::in_memory()?;
        let experiments = beater_experiments::SqliteExperimentStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let expected_dataset = DatasetId::new("expected-dataset")?;
        let actual_dataset = DatasetId::new("actual-dataset")?;
        let evaluator = EvaluatorVersionId::new("exact-v1")?;
        let gate = gates
            .put_gate(GateDefinition {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                gate_id: GateId::new("main")?,
                name: "main".to_string(),
                dataset_id: Some(expected_dataset),
                evaluator_version_id: Some(evaluator.clone()),
                inconclusive_policy: InconclusivePolicy::Fail,
                created_at: Utc::now(),
            })
            .await?;
        let experiment = experiments
            .write_run(experiment_report(
                tenant.clone(),
                project.clone(),
                "exp-1",
                actual_dataset,
                evaluator,
                GateDecision::Pass,
                0.1,
            )?)
            .await?;

        let error = run_gate(
            &gates,
            &experiments,
            tenant,
            project,
            gate.gate_id,
            Some(experiment.experiment_run_id),
        )
        .await
        .err()
        .ok_or_else(|| anyhow!("expected gate mismatch to fail"))?;

        assert!(error.to_string().contains("expected dataset"));
        Ok(())
    }

    #[test]
    fn gate_never_passes_inconclusive_even_if_policy_allows_it() -> anyhow::Result<()> {
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let gate = GateDefinition {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            gate_id: GateId::new("main")?,
            name: "main".to_string(),
            dataset_id: Some(DatasetId::new("dataset")?),
            evaluator_version_id: Some(EvaluatorVersionId::new("exact-v1")?),
            inconclusive_policy: InconclusivePolicy::Pass,
            created_at: Utc::now(),
        };
        let experiment = experiment_report(
            tenant,
            project,
            "exp-inconclusive",
            DatasetId::new("dataset")?,
            EvaluatorVersionId::new("exact-v1")?,
            GateDecision::Inconclusive,
            0.01,
        )?;

        let report = evaluate_gate(&gate, &experiment)?;

        assert_eq!(report.inconclusive_policy, InconclusivePolicy::Pass);
        assert_eq!(report.experiment_decision, GateDecision::Inconclusive);
        assert!(!report.passed);
        assert_eq!(
            report.reason,
            "experiment exp-inconclusive was inconclusive; deploy gates never pass inconclusive results"
        );
        Ok(())
    }

    fn experiment_report(
        tenant_id: TenantId,
        project_id: ProjectId,
        experiment_run_id: &str,
        dataset_id: DatasetId,
        evaluator_version_id: EvaluatorVersionId,
        decision: GateDecision,
        delta: f64,
    ) -> anyhow::Result<ExperimentRunReport> {
        Ok(ExperimentRunReport {
            experiment_run_id: ExperimentRunId::new(experiment_run_id)?,
            tenant_id,
            project_id,
            dataset_id,
            dataset_version_id: beater_core::DatasetVersionId::new("version")?,
            baseline_release_id: AgentReleaseId::new("baseline")?,
            candidate_release_id: AgentReleaseId::new("candidate")?,
            evaluator_version_id,
            case_scores: Vec::new(),
            comparison: ExperimentComparison {
                sample_size: 1,
                baseline_mean: 1.0,
                candidate_mean: 1.0 + delta,
                delta,
                ci_low: delta,
                ci_high: delta,
                p_value: 1.0,
                decision: decision.clone(),
                test: StatisticalTest::PairedT,
                adjusted_alpha: 0.05,
            },
            decision,
            gate_policy: GatePolicy {
                min_sample_size: 1,
                ..GatePolicy::default()
            },
            created_at: Utc::now(),
        })
    }
}
