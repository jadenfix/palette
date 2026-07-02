pub mod rsi;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    AgentReleaseId, DatasetCaseId, DatasetId, DatasetVersionId, EvaluatorVersionId,
    ExperimentRunId, JudgeCallId, Money, ProjectId, ProviderSecretId, TenantId, Timestamp,
};
use beater_datasets::DatasetVersionSnapshot;
use beater_eval::{
    compare_paired_scores_cuped, compare_paired_scores_with_design, conservative_gate_design,
    evaluate_deterministic, EvaluationCase, EvaluatorSpec, ExperimentComparison, GateDecision,
    GatePolicy, MultiplicityPolicy, ScoreResult, VarianceReduction,
};
use beater_judge::{
    GenerationRequest, JudgeBroker, JudgeBrokerOutcome, JudgeBrokerRequest, ProviderCredentials,
    TextGenerator,
};
use beater_schema::EvaluatorLane;
use beater_stats::{
    assess_generalization_gap, benjamini_hochberg, hoeffding_race, holm_bonferroni, GapAssessment,
};
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[async_trait]
pub trait ExperimentStore: Send + Sync {
    async fn write_run(&self, report: ExperimentRunReport) -> StoreResult<ExperimentRunReport>;

    async fn get_run(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        experiment_run_id: ExperimentRunId,
    ) -> StoreResult<ExperimentRunReport>;

    async fn latest_run(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        dataset_id: Option<DatasetId>,
        evaluator_version_id: Option<EvaluatorVersionId>,
    ) -> StoreResult<Option<ExperimentRunReport>>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaseOutputOverride {
    pub case_id: DatasetCaseId,
    pub output: Value,
    pub trace: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentRunSpec {
    pub baseline_release_id: AgentReleaseId,
    pub candidate_release_id: AgentReleaseId,
    pub evaluator: EvaluatorSpec,
    pub evaluator_version_id: EvaluatorVersionId,
    pub gate_policy: GatePolicy,
    pub baseline_outputs: Vec<CaseOutputOverride>,
    pub candidate_outputs: Vec<CaseOutputOverride>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeExperimentRunSpec {
    pub experiment: ExperimentRunSpec,
    pub provider_secret_id: ProviderSecretId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CaseExperimentScore {
    pub case_id: DatasetCaseId,
    #[schema(value_type = serde_json::Value)]
    pub baseline_output: Value,
    #[schema(value_type = serde_json::Value)]
    pub candidate_output: Value,
    #[schema(value_type = Option<serde_json::Value>)]
    pub baseline_trace: Option<Value>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub candidate_trace: Option<Value>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub reference: Option<Value>,
    pub baseline_score: f64,
    pub candidate_score: f64,
    pub delta: f64,
    #[schema(value_type = serde_json::Value)]
    pub baseline_evidence: Value,
    #[schema(value_type = serde_json::Value)]
    pub candidate_evidence: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_cost: Option<Money>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_cost: Option<Money>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_judge_call_id: Option<JudgeCallId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_judge_call_id: Option<JudgeCallId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_cached: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_cached: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExperimentRunReport {
    pub experiment_run_id: ExperimentRunId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dataset_id: DatasetId,
    pub dataset_version_id: DatasetVersionId,
    pub baseline_release_id: AgentReleaseId,
    pub candidate_release_id: AgentReleaseId,
    pub evaluator_version_id: EvaluatorVersionId,
    pub case_scores: Vec<CaseExperimentScore>,
    pub comparison: ExperimentComparison,
    pub decision: GateDecision,
    #[serde(default)]
    pub gate_policy: GatePolicy,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HarnessContext {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dataset_id: DatasetId,
    pub dataset_version_id: DatasetVersionId,
    pub agent_release_id: AgentReleaseId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentRunOutput {
    pub output: Value,
    pub trace: Option<Value>,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum AgentAdapterError {
    #[error("agent adapter error: {0}")]
    Backend(String),
}

impl AgentAdapterError {
    pub fn backend(error: impl std::fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }
}

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    async fn run_case(
        &self,
        case: beater_datasets::DatasetCase,
        context: HarnessContext,
    ) -> Result<AgentRunOutput, AgentAdapterError>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentExperimentSpec {
    pub baseline_release_id: AgentReleaseId,
    pub candidate_release_id: AgentReleaseId,
    pub evaluator: EvaluatorSpec,
    pub evaluator_version_id: EvaluatorVersionId,
    pub gate_policy: GatePolicy,
}

#[derive(Clone, Debug)]
pub struct StaticAgentAdapter {
    output: Value,
    adapter_name: String,
}

impl StaticAgentAdapter {
    pub fn new(output: Value, adapter_name: impl Into<String>) -> Self {
        Self {
            output,
            adapter_name: adapter_name.into(),
        }
    }
}

#[async_trait]
impl AgentAdapter for StaticAgentAdapter {
    async fn run_case(
        &self,
        case: beater_datasets::DatasetCase,
        context: HarnessContext,
    ) -> Result<AgentRunOutput, AgentAdapterError> {
        Ok(AgentRunOutput {
            output: self.output.clone(),
            trace: Some(serde_json::json!({
                "adapter": self.adapter_name,
                "agent_release_id": context.agent_release_id,
                "dataset_case_id": case.case_id,
                "input": case.input
            })),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReferenceAgentAdapter {
    adapter_name: String,
}

impl ReferenceAgentAdapter {
    pub fn new(adapter_name: impl Into<String>) -> Self {
        Self {
            adapter_name: adapter_name.into(),
        }
    }
}

#[async_trait]
impl AgentAdapter for ReferenceAgentAdapter {
    async fn run_case(
        &self,
        case: beater_datasets::DatasetCase,
        context: HarnessContext,
    ) -> Result<AgentRunOutput, AgentAdapterError> {
        Ok(AgentRunOutput {
            output: case.reference.clone().unwrap_or(Value::Null),
            trace: Some(serde_json::json!({
                "adapter": self.adapter_name,
                "agent_release_id": context.agent_release_id,
                "dataset_case_id": case.case_id,
                "input": case.input
            })),
        })
    }
}

#[derive(Clone)]
pub struct SqliteExperimentStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteExperimentStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection =
            Connection::open_in_memory().context("open in-memory experiment sqlite")?;
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
                .with_context(|| format!("create experiment sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite experiment store {}", path.display()))?;
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

                CREATE TABLE IF NOT EXISTS experiment_runs (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    experiment_run_id TEXT NOT NULL,
                    dataset_id TEXT NOT NULL,
                    dataset_version_id TEXT NOT NULL,
                    baseline_release_id TEXT NOT NULL,
                    candidate_release_id TEXT NOT NULL,
                    evaluator_version_id TEXT NOT NULL,
                    decision TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    report_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, experiment_run_id)
                );
                "#,
            )
            .context("initialize sqlite experiment store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("sqlite experiment connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl ExperimentStore for SqliteExperimentStore {
    async fn write_run(&self, report: ExperimentRunReport) -> StoreResult<ExperimentRunReport> {
        let report_json = serde_json::to_string(&report)
            .context("serialize experiment report")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO experiment_runs
                  (tenant_id, project_id, experiment_run_id, dataset_id, dataset_version_id,
                   baseline_release_id, candidate_release_id, evaluator_version_id, decision,
                   created_at, report_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    report.tenant_id.as_str(),
                    report.project_id.as_str(),
                    report.experiment_run_id.as_str(),
                    report.dataset_id.as_str(),
                    report.dataset_version_id.as_str(),
                    report.baseline_release_id.as_str(),
                    report.candidate_release_id.as_str(),
                    report.evaluator_version_id.as_str(),
                    report.decision.name(),
                    report.created_at.to_rfc3339(),
                    report_json,
                ],
            )
            .context("insert experiment run")
            .into_store()?;
        Ok(report)
    }

    async fn get_run(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        experiment_run_id: ExperimentRunId,
    ) -> StoreResult<ExperimentRunReport> {
        let connection = self.lock().into_store()?;
        let report_json = connection
            .query_row(
                r#"
                SELECT report_json
                FROM experiment_runs
                WHERE tenant_id = ?1 AND project_id = ?2 AND experiment_run_id = ?3
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    experiment_run_id.as_str()
                ],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query experiment run")
            .into_store()?
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "experiment run {} not found",
                    experiment_run_id.as_str()
                ))
            })?;
        serde_json::from_str(&report_json)
            .context("decode experiment run report")
            .into_store()
    }

    async fn latest_run(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        dataset_id: Option<DatasetId>,
        evaluator_version_id: Option<EvaluatorVersionId>,
    ) -> StoreResult<Option<ExperimentRunReport>> {
        let dataset_id = dataset_id.as_ref().map(|id| id.as_str());
        let evaluator_version_id = evaluator_version_id.as_ref().map(|id| id.as_str());
        let connection = self.lock().into_store()?;
        let report_json = connection
            .query_row(
                r#"
                SELECT report_json
                FROM experiment_runs
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND (?3 IS NULL OR dataset_id = ?3)
                  AND (?4 IS NULL OR evaluator_version_id = ?4)
                ORDER BY created_at DESC, experiment_run_id DESC
                LIMIT 1
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    dataset_id,
                    evaluator_version_id
                ],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query latest experiment run")
            .into_store()?;
        report_json
            .map(|report_json| {
                serde_json::from_str(&report_json).context("decode latest experiment run report")
            })
            .transpose()
            .into_store()
    }
}

pub fn run_deterministic_experiment(
    snapshot: &DatasetVersionSnapshot,
    spec: ExperimentRunSpec,
) -> anyhow::Result<ExperimentRunReport> {
    if spec.evaluator.lane != EvaluatorLane::DeterministicWasi {
        return Err(anyhow!(
            "experiment runner only accepts deterministic evaluator lane"
        ));
    }
    if snapshot.cases.is_empty() {
        return Err(anyhow!(
            "cannot run an experiment over an empty dataset version"
        ));
    }

    let baseline_outputs = output_map(spec.baseline_outputs)?;
    let candidate_outputs = output_map(spec.candidate_outputs)?;
    let mut case_scores = Vec::with_capacity(snapshot.cases.len());
    let mut baseline_scores = Vec::with_capacity(snapshot.cases.len());
    let mut candidate_scores = Vec::with_capacity(snapshot.cases.len());

    for case in &snapshot.cases {
        let baseline = baseline_outputs
            .get(case.case_id.as_str())
            .cloned()
            .unwrap_or_else(|| AgentRunOutput {
                output: case.output.clone(),
                trace: Some(case.trace.clone()),
            });
        let candidate = candidate_outputs
            .get(case.case_id.as_str())
            .cloned()
            .unwrap_or_else(|| AgentRunOutput {
                output: case.output.clone(),
                trace: Some(case.trace.clone()),
            });
        let baseline_score = score_output(snapshot, &spec.evaluator, case, &baseline)
            .with_context(|| format!("score baseline case {}", case.case_id.as_str()))?;
        let candidate_score = score_output(snapshot, &spec.evaluator, case, &candidate)
            .with_context(|| format!("score candidate case {}", case.case_id.as_str()))?;
        baseline_scores.push(baseline_score.score);
        candidate_scores.push(candidate_score.score);
        case_scores.push(CaseExperimentScore {
            case_id: case.case_id.clone(),
            baseline_output: baseline.output.clone(),
            candidate_output: candidate.output.clone(),
            baseline_trace: baseline.trace.clone(),
            candidate_trace: candidate.trace.clone(),
            reference: case.reference.clone(),
            baseline_score: baseline_score.score,
            candidate_score: candidate_score.score,
            delta: candidate_score.score - baseline_score.score,
            baseline_evidence: baseline_score.evidence,
            candidate_evidence: candidate_score.evidence,
            baseline_cost: None,
            candidate_cost: None,
            baseline_judge_call_id: None,
            candidate_judge_call_id: None,
            baseline_cached: None,
            candidate_cached: None,
        });
    }

    let gate_policy = spec.gate_policy.clone();
    let design = conservative_gate_design(&gate_policy, baseline_scores.len());
    let comparison = compare_paired_scores_with_design(
        &baseline_scores,
        &candidate_scores,
        &gate_policy,
        &design,
    )
    .context("compare experiment scores")?;
    Ok(ExperimentRunReport {
        experiment_run_id: ExperimentRunId::new(Uuid::new_v4().to_string())?,
        tenant_id: snapshot.tenant_id.clone(),
        project_id: snapshot.project_id.clone(),
        dataset_id: snapshot.dataset_id.clone(),
        dataset_version_id: snapshot.version_id.clone(),
        baseline_release_id: spec.baseline_release_id,
        candidate_release_id: spec.candidate_release_id,
        evaluator_version_id: spec.evaluator_version_id,
        case_scores,
        decision: comparison.decision.clone(),
        comparison,
        gate_policy,
        created_at: Utc::now(),
    })
}

pub async fn run_judge_experiment<B>(
    snapshot: &DatasetVersionSnapshot,
    spec: JudgeExperimentRunSpec,
    judge_broker: &B,
) -> anyhow::Result<ExperimentRunReport>
where
    B: JudgeBroker + ?Sized,
{
    if spec.experiment.evaluator.lane != EvaluatorLane::JudgeBroker {
        return Err(anyhow!(
            "judge experiment runner only accepts judge broker evaluator lane"
        ));
    }
    if snapshot.cases.is_empty() {
        return Err(anyhow!(
            "cannot run a judge experiment over an empty dataset version"
        ));
    }

    let baseline_outputs = output_map(spec.experiment.baseline_outputs)?;
    let candidate_outputs = output_map(spec.experiment.candidate_outputs)?;
    let mut case_scores = Vec::with_capacity(snapshot.cases.len());
    let mut baseline_scores = Vec::with_capacity(snapshot.cases.len());
    let mut candidate_scores = Vec::with_capacity(snapshot.cases.len());

    for case in &snapshot.cases {
        let baseline = baseline_outputs
            .get(case.case_id.as_str())
            .cloned()
            .unwrap_or_else(|| AgentRunOutput {
                output: case.output.clone(),
                trace: Some(case.trace.clone()),
            });
        let candidate = candidate_outputs
            .get(case.case_id.as_str())
            .cloned()
            .unwrap_or_else(|| AgentRunOutput {
                output: case.output.clone(),
                trace: Some(case.trace.clone()),
            });
        let baseline_outcome = score_output_with_judge(
            snapshot,
            &spec.experiment.evaluator,
            &spec.provider_secret_id,
            spec.experiment.evaluator_version_id.as_str(),
            case,
            &baseline,
            judge_broker,
        )
        .await
        .with_context(|| format!("judge baseline case {}", case.case_id.as_str()))?;
        let candidate_outcome = score_output_with_judge(
            snapshot,
            &spec.experiment.evaluator,
            &spec.provider_secret_id,
            spec.experiment.evaluator_version_id.as_str(),
            case,
            &candidate,
            judge_broker,
        )
        .await
        .with_context(|| format!("judge candidate case {}", case.case_id.as_str()))?;
        baseline_scores.push(baseline_outcome.result.score);
        candidate_scores.push(candidate_outcome.result.score);
        let baseline_audit = baseline_outcome.audit;
        let candidate_audit = candidate_outcome.audit;
        case_scores.push(CaseExperimentScore {
            case_id: case.case_id.clone(),
            baseline_output: baseline.output.clone(),
            candidate_output: candidate.output.clone(),
            baseline_trace: baseline.trace.clone(),
            candidate_trace: candidate.trace.clone(),
            reference: case.reference.clone(),
            baseline_score: baseline_outcome.result.score,
            candidate_score: candidate_outcome.result.score,
            delta: candidate_outcome.result.score - baseline_outcome.result.score,
            baseline_evidence: baseline_outcome.result.evidence,
            candidate_evidence: candidate_outcome.result.evidence,
            baseline_cost: Some(baseline_audit.charged_cost),
            candidate_cost: Some(candidate_audit.charged_cost),
            baseline_judge_call_id: Some(baseline_audit.judge_call_id),
            candidate_judge_call_id: Some(candidate_audit.judge_call_id),
            baseline_cached: Some(baseline_audit.cached),
            candidate_cached: Some(candidate_audit.cached),
        });
    }

    let gate_policy = spec.experiment.gate_policy.clone();
    let design = conservative_gate_design(&gate_policy, baseline_scores.len());
    let comparison = compare_paired_scores_with_design(
        &baseline_scores,
        &candidate_scores,
        &gate_policy,
        &design,
    )
    .context("compare judge experiment scores")?;
    Ok(ExperimentRunReport {
        experiment_run_id: ExperimentRunId::new(Uuid::new_v4().to_string())?,
        tenant_id: snapshot.tenant_id.clone(),
        project_id: snapshot.project_id.clone(),
        dataset_id: snapshot.dataset_id.clone(),
        dataset_version_id: snapshot.version_id.clone(),
        baseline_release_id: spec.experiment.baseline_release_id,
        candidate_release_id: spec.experiment.candidate_release_id,
        evaluator_version_id: spec.experiment.evaluator_version_id,
        case_scores,
        decision: comparison.decision.clone(),
        comparison,
        gate_policy,
        created_at: Utc::now(),
    })
}

pub async fn run_agent_experiment<B, C>(
    snapshot: &DatasetVersionSnapshot,
    spec: AgentExperimentSpec,
    baseline: &B,
    candidate: &C,
) -> anyhow::Result<ExperimentRunReport>
where
    B: AgentAdapter,
    C: AgentAdapter,
{
    if snapshot.cases.is_empty() {
        return Err(anyhow!(
            "cannot run an agent experiment over an empty dataset version"
        ));
    }
    let mut baseline_outputs = Vec::with_capacity(snapshot.cases.len());
    let mut candidate_outputs = Vec::with_capacity(snapshot.cases.len());
    for case in &snapshot.cases {
        let baseline_output = baseline
            .run_case(
                case.clone(),
                HarnessContext {
                    tenant_id: snapshot.tenant_id.clone(),
                    project_id: snapshot.project_id.clone(),
                    dataset_id: snapshot.dataset_id.clone(),
                    dataset_version_id: snapshot.version_id.clone(),
                    agent_release_id: spec.baseline_release_id.clone(),
                },
            )
            .await
            .with_context(|| format!("run baseline agent case {}", case.case_id.as_str()))?;
        let candidate_output = candidate
            .run_case(
                case.clone(),
                HarnessContext {
                    tenant_id: snapshot.tenant_id.clone(),
                    project_id: snapshot.project_id.clone(),
                    dataset_id: snapshot.dataset_id.clone(),
                    dataset_version_id: snapshot.version_id.clone(),
                    agent_release_id: spec.candidate_release_id.clone(),
                },
            )
            .await
            .with_context(|| format!("run candidate agent case {}", case.case_id.as_str()))?;
        baseline_outputs.push(CaseOutputOverride {
            case_id: case.case_id.clone(),
            output: baseline_output.output,
            trace: baseline_output.trace,
        });
        candidate_outputs.push(CaseOutputOverride {
            case_id: case.case_id.clone(),
            output: candidate_output.output,
            trace: candidate_output.trace,
        });
    }

    run_deterministic_experiment(
        snapshot,
        ExperimentRunSpec {
            baseline_release_id: spec.baseline_release_id,
            candidate_release_id: spec.candidate_release_id,
            evaluator: spec.evaluator,
            evaluator_version_id: spec.evaluator_version_id,
            gate_policy: spec.gate_policy,
            baseline_outputs,
            candidate_outputs,
        },
    )
}

fn output_map(
    overrides: Vec<CaseOutputOverride>,
) -> anyhow::Result<BTreeMap<String, AgentRunOutput>> {
    let mut map = BTreeMap::new();
    for override_value in overrides {
        let previous = map.insert(
            override_value.case_id.as_str().to_string(),
            AgentRunOutput {
                output: override_value.output,
                trace: override_value.trace,
            },
        );
        if previous.is_some() {
            return Err(anyhow!(
                "duplicate output override for case {}",
                override_value.case_id.as_str()
            ));
        }
    }
    Ok(map)
}

fn score_output(
    snapshot: &DatasetVersionSnapshot,
    evaluator: &EvaluatorSpec,
    case: &beater_datasets::DatasetCase,
    output: &AgentRunOutput,
) -> anyhow::Result<ScoreResult> {
    if case.tenant_id.as_str() != snapshot.tenant_id.as_str()
        || case.project_id.as_str() != snapshot.project_id.as_str()
        || case.dataset_id.as_str() != snapshot.dataset_id.as_str()
    {
        return Err(anyhow!("dataset case crosses snapshot boundary"));
    }
    evaluate_deterministic(
        evaluator,
        &EvaluationCase {
            input: case.input.clone(),
            output: output.output.clone(),
            reference: case.reference.clone(),
            trace: output.trace.clone().or_else(|| Some(case.trace.clone())),
        },
    )
    .map_err(anyhow::Error::from)
}

async fn score_output_with_judge<B>(
    snapshot: &DatasetVersionSnapshot,
    evaluator: &EvaluatorSpec,
    provider_secret_id: &ProviderSecretId,
    cache_namespace: &str,
    case: &beater_datasets::DatasetCase,
    output: &AgentRunOutput,
    judge_broker: &B,
) -> anyhow::Result<JudgeBrokerOutcome>
where
    B: JudgeBroker + ?Sized,
{
    if case.tenant_id.as_str() != snapshot.tenant_id.as_str()
        || case.project_id.as_str() != snapshot.project_id.as_str()
        || case.dataset_id.as_str() != snapshot.dataset_id.as_str()
    {
        return Err(anyhow!("dataset case crosses snapshot boundary"));
    }
    judge_broker
        .evaluate(JudgeBrokerRequest {
            tenant_id: snapshot.tenant_id.clone(),
            project_id: snapshot.project_id.clone(),
            evaluator: evaluator.clone(),
            case: EvaluationCase {
                input: case.input.clone(),
                output: output.output.clone(),
                reference: case.reference.clone(),
                trace: output.trace.clone().or_else(|| Some(case.trace.clone())),
            },
            provider_secret_id: provider_secret_id.clone(),
            cache_namespace: Some(cache_namespace.to_string()),
        })
        .await
        .map_err(|err| anyhow!(err))
}

/// Pluggable optimizer strategies for the recursive self-improvement loop.
///
/// Each variant names a concrete prompt/agent optimizer family called for by
/// ARCHITECTURE §20.10 #7.6 ("named prompt/agent optimizer strategies, gated by
/// held-out statistics") and REQUIREMENTS R18.6. The names mirror the
/// reflective-proposal direction of §21.3 and the deferred population search of
/// §21.6c.
///
/// **Gating invariant — the differentiator vs. un-gated optimizers.** A strategy
/// only *proposes* [`CandidateChange`]s; it never *accepts* one. Every candidate
/// from every strategy MUST flow through the existing held-out **Test** gate plus
/// the `beater-stats` confidence interval already implemented here
/// (`run_deterministic_experiment` / `run_judge_experiment` / `run_agent_experiment`
/// → [`compare_paired_scores`] → [`GateDecision`], §21.3) and the planned §21.4
/// anti-overfitting guardrail before it can be accepted. Proposal is not
/// acceptance: the strategy emits candidates, the gate decides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizerStrategy {
    /// Reflective single-shot LLM rewrite of a prompt lever of π (§6.1) — the
    /// reflective-proposal baseline of §21.3. Implemented (minimally) below.
    #[default]
    LlmRewrite,
    /// Few-shot exemplar selection driven by a Bayesian acquisition function.
    FewShotBayesian,
    /// MIPRO-style joint optimization of instructions and few-shot exemplars.
    Mipro,
    /// Population / evolutionary search over agent configs (deferred, §21.6c).
    Evolutionary,
    /// GEPA-style reflective evolutionary prompt optimization.
    Gepa,
    /// Hyperparameter / model-params search over the model-params lever of π (§6.1).
    ParamSearch,
}

/// The policy-π (§6.1) lever a [`CandidateChange`] targets, mirroring the planned
/// §21.1 `ChangeKind` taxonomy. Kept internal to this crate; intentionally not a
/// `/v1` contract type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// The system-prompt lever of π (§6.1).
    SystemPrompt,
    /// The customer-prompt lever of π (§6.1).
    CustomerPrompt,
    /// Agent code.
    Code,
    /// Add a tool — the tool_set lever of π (§6.1).
    ToolAdd,
    /// Remove a tool — the tool_set lever of π (§6.1).
    ToolRemove,
    /// The memory lever of π (§6.1).
    MemoryConfig,
    /// The model-params lever of π (§6.1).
    ModelParams,
    /// Not a π lever — challenges a dataset label (§21.1 `challenge_labels`).
    DataLabel,
}

/// A single proposed change to the target agent's policy π (§6.1).
///
/// This is the *proposal* a strategy emits — never an applied edit. It carries a
/// rationale and the exact target (§21.1: "the exact file/symbol/span it
/// targets") so the held-out gate has full provenance for the audit trail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateChange {
    /// Which policy lever this change touches.
    pub kind: ChangeKind,
    /// The file / symbol / prompt the change targets (§21.1).
    pub target: String,
    /// Human-readable description of the proposed change.
    pub description: String,
    /// Why the strategy believes this change helps — carried to the gate for audit.
    pub rationale: String,
    /// Which strategy emitted this candidate.
    pub proposed_by: OptimizerStrategy,
}

/// One failing example handed to a proposer as reflective signal.
///
/// Carries the high-signal fields a reflective rewrite needs — what went in,
/// what we wanted vs. what we got, the numeric score, and any error/exception
/// text — without depending on the richer (unmerged) `beater-scenarios`
/// clustering. Excerpts are truncated by [`FailureExample::from_parts`] so a
/// single huge output cannot blow up a reflective prompt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FailureExample {
    /// A truncated excerpt of the case input.
    pub input_excerpt: String,
    /// The expected / reference output, truncated, if the case had one.
    pub expected: Option<String>,
    /// A truncated excerpt of the actual output produced by the agent.
    pub actual: String,
    /// The numeric score this example received from the evaluator.
    pub score: f64,
    /// Any error / exception text surfaced for this example, if present.
    pub error: Option<String>,
}

impl FailureExample {
    /// Maximum characters retained per excerpt before truncation.
    pub const EXCERPT_LIMIT: usize = 280;

    /// Build a [`FailureExample`], truncating every free-text field to
    /// [`EXCERPT_LIMIT`](Self::EXCERPT_LIMIT) characters so a pathological case
    /// cannot dominate a reflective prompt.
    pub fn from_parts(
        input: impl Into<String>,
        expected: Option<String>,
        actual: impl Into<String>,
        score: f64,
        error: Option<String>,
    ) -> Self {
        Self {
            input_excerpt: truncate_excerpt(&input.into()),
            expected: expected.map(|e| truncate_excerpt(&e)),
            actual: truncate_excerpt(&actual.into()),
            score,
            error: error.map(|e| truncate_excerpt(&e)),
        }
    }

    /// The deterministic failure signature for this example — see
    /// [`ProposalContext`] for how signatures are aggregated. When error text is
    /// present we normalize it (lowercased, volatile tokens masked); otherwise
    /// we fall back to the first divergent token between expected and actual.
    pub fn signature(&self) -> String {
        if let Some(error) = self.error.as_deref().filter(|e| !e.trim().is_empty()) {
            return format!("error: {}", normalize_signature(error));
        }
        match self.expected.as_deref() {
            Some(expected) => format!(
                "divergence: {}",
                first_divergent_token(expected, &self.actual)
            ),
            None => format!("low_score: {}", normalize_signature(&self.actual)),
        }
    }
}

/// Truncate a free-text excerpt to [`FailureExample::EXCERPT_LIMIT`] characters,
/// appending an ellipsis marker when truncation occurred. Operates on `char`
/// boundaries so it never splits a multi-byte code point.
fn truncate_excerpt(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= FailureExample::EXCERPT_LIMIT {
        return trimmed.to_string();
    }
    let head: String = trimmed
        .chars()
        .take(FailureExample::EXCERPT_LIMIT)
        .collect();
    format!("{head}…")
}

/// Normalize a string into a stable failure signature: lowercase, collapse
/// whitespace, and mask volatile tokens (numbers, hex, uuids) to `<n>` so that
/// "timeout after 1200ms" and "timeout after 950ms" bucket together. Kept
/// deliberately simple and deterministic — this is the local, on-`main`
/// substitute for `beater-scenarios` clustering, not a reimplementation of it.
fn normalize_signature(text: &str) -> String {
    let lowered = text.trim().to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut prev_space = false;
    for word in lowered.split_whitespace() {
        if prev_space {
            out.push(' ');
        }
        out.push_str(&mask_token(word));
        prev_space = true;
    }
    // Cap signature length so an unbounded message cannot become a unique bucket.
    out.chars().take(120).collect()
}

/// Mask volatile substrings within a single token so values that differ only by
/// number/hex/uuid bucket together: every maximal run of digits/hex is replaced
/// by `<n>`. So "1200ms" and "950ms" both become "<n>ms", and a bare uuid /
/// hex id collapses to "<n>". Non-numeric tokens pass through unchanged.
fn mask_token(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut in_run = false;
    for ch in word.chars() {
        // Treat a hex-ish character as part of a numeric run only when the token
        // actually contains a digit somewhere (so plain words like "deaf" or
        // "cab" are not mangled).
        let numericish = ch.is_ascii_digit()
            || ((ch.is_ascii_hexdigit() || ch == '-') && word.chars().any(|c| c.is_ascii_digit()));
        if numericish {
            if !in_run {
                out.push_str("<n>");
                in_run = true;
            }
        } else {
            out.push(ch);
            in_run = false;
        }
    }
    out
}

/// Return the first token of `actual` that diverges from `expected`, used as a
/// coarse failure signature when there is no explicit error string. Tokens are
/// whitespace-delimited; returns `<missing>`/`<extra>` for length mismatches.
fn first_divergent_token(expected: &str, actual: &str) -> String {
    let mut exp = expected.split_whitespace();
    let mut act = actual.split_whitespace();
    loop {
        match (exp.next(), act.next()) {
            (Some(e), Some(a)) if e == a => continue,
            (Some(e), Some(a)) => {
                return normalize_signature(&format!("{e} -> {a}"));
            }
            (Some(e), None) => return normalize_signature(&format!("{e} -> <missing>")),
            (None, Some(a)) => return normalize_signature(&format!("<extra> -> {a}")),
            (None, None) => return "<no_divergence>".to_string(),
        }
    }
}

/// A counted failure signature: a normalized failure bucket and how many of the
/// failing examples fell into it. See [`ProposalContext::failure_signatures`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FailureSignature {
    /// The normalized signature string.
    pub signature: String,
    /// How many failing examples matched this signature.
    pub count: usize,
}

/// Aggregate statistics over the failing examples in a [`ProposalContext`].
///
/// All fields are derived deterministically from the supplied failing examples;
/// `score_buckets` is a fixed-width histogram over `[0,1]` in 0.2-wide buckets
/// (index 0 = `[0.0,0.2)` … index 4 = `[0.8,1.0]`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FailureStats {
    /// Number of failing examples summarized.
    pub n_failures: usize,
    /// Mean score across the failing examples (0.0 when there are none).
    pub mean_score: f64,
    /// Fixed five-bucket score histogram over `[0,1]` (0.2-wide buckets).
    pub score_buckets: [usize; 5],
}

impl FailureStats {
    fn from_examples(examples: &[FailureExample]) -> Self {
        let n_failures = examples.len();
        let mut score_buckets = [0usize; 5];
        let mut sum = 0.0;
        for ex in examples {
            sum += ex.score;
            let clamped = ex.score.clamp(0.0, 1.0);
            // 1.0 lands in the top bucket rather than overflowing to index 5.
            let idx = ((clamped * 5.0) as usize).min(4);
            score_buckets[idx] += 1;
        }
        let mean_score = if n_failures == 0 {
            0.0
        } else {
            sum / n_failures as f64
        };
        Self {
            n_failures,
            mean_score,
            score_buckets,
        }
    }
}

/// Read-only context handed to a [`ProposalStrategy`].
///
/// The strategy reflects on the optimization goal, the current lever text, and a
/// set of *failing examples* (§21.1 `index_agent`) to emit candidates; it has no
/// ability to accept them. The failure features here are derived locally and
/// deterministically — this is the on-`main` substitute for the richer
/// `beater-scenarios` clustering (unmerged PR #470), not a dependency on it.
///
/// `Eq`/`Hash` are intentionally not derived: the aggregate stats carry an
/// `f64` mean, so the struct is `PartialEq` only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProposalContext {
    /// The improvement goal in natural language (§21.3 "goal + params").
    pub goal: String,
    /// The current prompt (or other lever text) the optimizer may rewrite.
    pub current_prompt: String,
    /// The failing examples that motivate this optimization round.
    pub failing_examples: Vec<FailureExample>,
    /// Aggregate statistics over [`failing_examples`](Self::failing_examples).
    pub stats: FailureStats,
    /// The most common failure signatures, most frequent first, ties broken by
    /// signature string for determinism.
    pub failure_signatures: Vec<FailureSignature>,
}

impl ProposalContext {
    /// Construct a context from a goal, current lever text, and the failing
    /// examples, computing the aggregate stats and failure signatures
    /// deterministically.
    pub fn new(
        goal: impl Into<String>,
        current_prompt: impl Into<String>,
        failing_examples: Vec<FailureExample>,
    ) -> Self {
        let stats = FailureStats::from_examples(&failing_examples);
        let failure_signatures = Self::compute_signatures(&failing_examples);
        Self {
            goal: goal.into(),
            current_prompt: current_prompt.into(),
            failing_examples,
            stats,
            failure_signatures,
        }
    }

    /// Construct a context with no failing examples — preserves the old
    /// two-field call sites and the empty-failure case.
    pub fn from_goal(goal: impl Into<String>, current_prompt: impl Into<String>) -> Self {
        Self::new(goal, current_prompt, Vec::new())
    }

    /// Production constructor (#435): build a [`ProposalContext`] from the real
    /// failing evaluation cases of an optimization round.
    ///
    /// This is the non-test entry point that populates the enriched
    /// [`stats`](Self::stats) and [`failure_signatures`](Self::failure_signatures)
    /// fields from real data — the aggregate statistics and failure signatures are
    /// recomputed deterministically from `failures`, exactly as [`Self::new`] does
    /// for tests. The caller passes the minimal failing-case data already modeled
    /// by [`FailureExample`] (`input` / `expected` / `actual` / `score` / `error`),
    /// which avoids a dependency on `beater-eval` and the resulting crate cycle.
    ///
    /// The eval loop that owns `EvaluationResult`s constructs each
    /// [`FailureExample`] via [`FailureExample::from_parts`] and hands the slice
    /// here; the fields are therefore not test-only plumbing.
    pub fn from_failures(
        goal: impl Into<String>,
        current_prompt: impl Into<String>,
        failures: &[FailureExample],
    ) -> Self {
        Self::new(goal, current_prompt, failures.to_vec())
    }

    /// Group the failing examples by [`FailureExample::signature`] and return the
    /// buckets ordered by descending count (ties broken by signature string).
    fn compute_signatures(examples: &[FailureExample]) -> Vec<FailureSignature> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for ex in examples {
            *counts.entry(ex.signature()).or_insert(0) += 1;
        }
        let mut signatures: Vec<FailureSignature> = counts
            .into_iter()
            .map(|(signature, count)| FailureSignature { signature, count })
            .collect();
        // Descending count; BTreeMap already gives ascending-signature order so
        // the final sort keeps signatures sorted within equal counts.
        signatures.sort_by(|a, b| b.count.cmp(&a.count).then(a.signature.cmp(&b.signature)));
        signatures
    }
}

/// Errors a [`ProposalStrategy`] or [`propose_with`] can return.
///
/// Unimplemented strategies return [`OptimizerError::NotYetImplemented`] rather
/// than panicking, so the dispatch never aborts the process.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum OptimizerError {
    /// The strategy is named in the architecture (§20.10 #7.6) but not yet
    /// implemented. Returned as a typed error — never a panic / `unimplemented!()`.
    #[error("optimizer strategy {0:?} is not yet implemented")]
    NotYetImplemented(OptimizerStrategy),
    /// A candidate was emitted with provenance that does not match the selected
    /// dispatch strategy.
    #[error("optimizer strategy mismatch: expected {expected:?}, candidate recorded {actual:?}")]
    StrategyMismatch {
        expected: OptimizerStrategy,
        actual: OptimizerStrategy,
    },
    /// The proposal context was insufficient to produce a candidate.
    #[error("invalid proposal context: {0}")]
    InvalidContext(String),
    /// A proposer that consults an external broker/model failed to produce a
    /// candidate (e.g. the broker errored or returned nothing usable).
    #[error("proposer failed: {0}")]
    ProposerFailed(String),
    /// The injected [`CandidateEvaluator`] failed to score a candidate's cases.
    #[error("candidate evaluation failed: {0}")]
    EvaluationFailed(String),
    /// The held-out gate / anti-overfit statistics could not be computed for a
    /// candidate (e.g. mismatched per-split score lengths, non-finite scores, or
    /// an under-powered comparison). Surfaced as a typed error rather than a
    /// silent accept — a candidate the gate cannot judge is never accepted.
    #[error("gate evaluation failed: {0}")]
    GateFailed(String),
    /// The round configuration was internally inconsistent (e.g. an empty goal or
    /// no failing cases to reflect on).
    #[error("invalid optimization-round config: {0}")]
    InvalidConfig(String),
}

/// A strategy that *proposes* candidate changes for the held-out gate to judge.
///
/// Implementations never accept a change; see [`OptimizerStrategy`] for the
/// gating invariant.
pub trait ProposalStrategy {
    /// Emit zero or more candidate changes from the given context.
    fn propose(&self, ctx: &ProposalContext) -> Result<Vec<CandidateChange>, OptimizerError>;
}

/// An LLM-backed proposal strategy whose `propose` requires an async generation
/// call.
///
/// This complements the sync [`ProposalStrategy`] trait: strategies that must
/// consult a live model (e.g. [`LlmRewrite`]) implement this instead, taking a
/// [`TextGenerator`] (the plain text-generation seam in `beater-judge` — NOT the
/// scoring/judge path) plus the [`ProviderCredentials`] needed to authenticate
/// the call. The candidates returned are *still only proposals* — acceptance is
/// decided by the held-out gate, never by the proposer.
#[async_trait]
pub trait AsyncProposalStrategy {
    /// Emit zero or more candidate changes, consulting `generator` as needed.
    async fn propose_async(
        &self,
        ctx: &ProposalContext,
        generator: &dyn TextGenerator,
        credentials: ProviderCredentials,
    ) -> Result<Vec<CandidateChange>, OptimizerError>;
}

/// Reflective single-shot LLM rewrite of a prompt lever (§21.3).
///
/// Two entry points, both honest about what they do:
/// * [`ProposalStrategy::propose`] (sync) emits a single *scaffold* candidate
///   that records the reflective brief built from the context but does not call
///   a model — useful when no broker is wired (e.g. dry runs / planning).
/// * [`AsyncProposalStrategy::propose_async`] (async) builds the same reflective
///   brief and sends it to a real [`TextGenerator`] as a plain completion (its
///   own system + user prompt — never the judge/scoring contract), returning the
///   model's improved prompt as the candidate's `target`/`description`.
///
/// Either way the candidate is *only a proposal*; it must clear the held-out
/// Test gate (§21.3) + the beater-stats CI before it can be accepted. Proposal
/// is not acceptance: the gate decides.
#[derive(Clone, Copy, Debug, Default)]
pub struct LlmRewrite;

/// The model the reflective rewrite asks for when none is otherwise configured.
const LLM_REWRITE_MODEL: &str = "gpt-4o-mini";

/// The system prompt for the reflective rewrite generation call. It frames the
/// model as a prompt engineer and constrains the output to the rewritten prompt
/// only — this is a generation instruction, NOT a scoring/judge contract.
const LLM_REWRITE_SYSTEM: &str =
    "You are an expert prompt engineer. Given an optimization goal, a current system \
     prompt, and observed failures, you produce an improved system prompt. Respond with \
     ONLY the improved system prompt text — no preamble, no commentary, no scores.";

impl LlmRewrite {
    /// Build the reflective brief sent to (or recorded for) the model: goal,
    /// current prompt, failure stats, the top failure signatures, and a few
    /// concrete failing examples. Deterministic given the context.
    fn reflective_brief(ctx: &ProposalContext) -> String {
        let mut brief = String::new();
        brief.push_str(
            "You are improving an LLM agent's system prompt. Rewrite the CURRENT PROMPT so it \
             better achieves the GOAL and fixes the observed failures. Respond with ONLY the \
             improved system prompt text, no preamble.\n\n",
        );
        brief.push_str(&format!("GOAL:\n{}\n\n", ctx.goal.trim()));
        brief.push_str(&format!(
            "CURRENT PROMPT:\n{}\n\n",
            ctx.current_prompt.trim()
        ));
        brief.push_str(&format!(
            "FAILURE STATS: {} failing examples, mean score {:.3}, score buckets {:?}\n",
            ctx.stats.n_failures, ctx.stats.mean_score, ctx.stats.score_buckets
        ));
        if !ctx.failure_signatures.is_empty() {
            brief.push_str("TOP FAILURE SIGNATURES:\n");
            for sig in ctx.failure_signatures.iter().take(5) {
                brief.push_str(&format!("  - [{}x] {}\n", sig.count, sig.signature));
            }
        }
        if !ctx.failing_examples.is_empty() {
            brief.push_str("\nFAILING EXAMPLES:\n");
            for (i, ex) in ctx.failing_examples.iter().take(5).enumerate() {
                brief.push_str(&format!("  Example {} (score {:.3}):\n", i + 1, ex.score));
                brief.push_str(&format!("    input:    {}\n", ex.input_excerpt));
                if let Some(expected) = &ex.expected {
                    brief.push_str(&format!("    expected: {expected}\n"));
                }
                brief.push_str(&format!("    actual:   {}\n", ex.actual));
                if let Some(error) = &ex.error {
                    brief.push_str(&format!("    error:    {error}\n"));
                }
            }
        }
        brief
    }

    fn validate(ctx: &ProposalContext) -> Result<(), OptimizerError> {
        if ctx.goal.trim().is_empty() {
            return Err(OptimizerError::InvalidContext(
                "goal must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

impl ProposalStrategy for LlmRewrite {
    fn propose(&self, ctx: &ProposalContext) -> Result<Vec<CandidateChange>, OptimizerError> {
        Self::validate(ctx)?;
        Ok(vec![CandidateChange {
            kind: ChangeKind::SystemPrompt,
            target: "system_prompt".to_string(),
            description: format!(
                "Rewrite the system prompt to better satisfy goal: {}",
                ctx.goal
            ),
            rationale: format!(
                "reflective LLM-rewrite scaffold (§21.3) over {} failing example(s); no model \
                 was called on this sync path — call `propose_async` with a broker for the live \
                 rewrite. Either way the candidate must clear the held-out Test gate + \
                 beater-stats CI before acceptance.",
                ctx.stats.n_failures
            ),
            proposed_by: OptimizerStrategy::LlmRewrite,
        }])
    }
}

#[async_trait]
impl AsyncProposalStrategy for LlmRewrite {
    async fn propose_async(
        &self,
        ctx: &ProposalContext,
        generator: &dyn TextGenerator,
        credentials: ProviderCredentials,
    ) -> Result<Vec<CandidateChange>, OptimizerError> {
        Self::validate(ctx)?;
        // Honest generation seam: the reflective brief is sent as a PLAIN
        // completion (system + user prompt) via `TextGenerator::generate`. The
        // model's raw text IS the rewritten prompt. This does NOT go through the
        // judge/scoring path, so the model is asked to rewrite — not to score.
        let request = GenerationRequest::new(LLM_REWRITE_MODEL, Self::reflective_brief(ctx))
            .with_system(LLM_REWRITE_SYSTEM)
            .with_temperature(0.3)
            .with_max_tokens(1024);
        let response = generator
            .generate(request, credentials)
            .await
            .map_err(|err| OptimizerError::ProposerFailed(err.to_string()))?;
        let rewritten = response.text.trim();
        if rewritten.is_empty() {
            return Err(OptimizerError::ProposerFailed(
                "generator returned an empty rewritten prompt".to_string(),
            ));
        }
        Ok(vec![CandidateChange {
            kind: ChangeKind::SystemPrompt,
            target: rewritten.to_string(),
            description: format!(
                "Generator-proposed system-prompt rewrite for goal: {}",
                ctx.goal
            ),
            rationale: format!(
                "reflective LLM rewrite via the beater-judge text-generation seam over {} failing \
                 example(s) (model {}); a proposal only — must clear the held-out Test gate + \
                 beater-stats CI before acceptance. Proposal is not acceptance: the gate decides.",
                ctx.stats.n_failures, LLM_REWRITE_MODEL
            ),
            proposed_by: OptimizerStrategy::LlmRewrite,
        }])
    }
}

/// Deterministic grid search over the model-params lever of π (§6.1).
///
/// This is a real (LLM-free) strategy: it emits one candidate per point of a
/// small fixed temperature/top-p grid, deterministically ordered. It does not
/// run the candidates — that is the gate's job. Proposal is not acceptance.
#[derive(Clone, Copy, Debug, Default)]
pub struct ParamSearch;

impl ParamSearch {
    /// The fixed temperature grid swept by [`ParamSearch`].
    const TEMPERATURES: [f64; 3] = [0.0, 0.3, 0.7];
    /// The fixed top-p grid swept by [`ParamSearch`].
    const TOP_PS: [f64; 2] = [0.9, 1.0];
}

impl ProposalStrategy for ParamSearch {
    fn propose(&self, ctx: &ProposalContext) -> Result<Vec<CandidateChange>, OptimizerError> {
        if ctx.goal.trim().is_empty() {
            return Err(OptimizerError::InvalidContext(
                "goal must not be empty".to_string(),
            ));
        }
        let mut candidates = Vec::new();
        for &temperature in &Self::TEMPERATURES {
            for &top_p in &Self::TOP_PS {
                candidates.push(CandidateChange {
                    kind: ChangeKind::ModelParams,
                    target: format!("model_params(temperature={temperature},top_p={top_p})"),
                    description: format!(
                        "Set model params to temperature={temperature}, top_p={top_p}"
                    ),
                    rationale: format!(
                        "deterministic param-grid point (§6.1 model-params lever) proposed for \
                         goal: {}; a proposal only — the held-out Test gate + beater-stats CI \
                         decide acceptance.",
                        ctx.goal
                    ),
                    proposed_by: OptimizerStrategy::ParamSearch,
                });
            }
        }
        Ok(candidates)
    }
}

/// Dispatch to the named [`OptimizerStrategy`], returning its proposed candidates.
///
/// Two strategies are implemented synchronously here:
/// * [`OptimizerStrategy::LlmRewrite`] — emits the reflective-rewrite *scaffold*
///   (no model call). For the live, generation-backed rewrite call
///   [`LlmRewrite::propose_async`] directly, which needs an async [`TextGenerator`].
/// * [`OptimizerStrategy::ParamSearch`] — a deterministic model-params grid.
///
/// The remaining variants are genuinely deferred and return a typed
/// [`OptimizerError::NotYetImplemented`]. Whatever a strategy proposes is *only*
/// a proposal — acceptance still requires clearing the held-out Test gate
/// (§21.3) and the planned §21.4 guardrail.
pub fn propose_with(
    strategy: OptimizerStrategy,
    ctx: &ProposalContext,
) -> Result<Vec<CandidateChange>, OptimizerError> {
    let candidates = match strategy {
        OptimizerStrategy::LlmRewrite => LlmRewrite.propose(ctx),
        OptimizerStrategy::ParamSearch => ParamSearch.propose(ctx),
        OptimizerStrategy::FewShotBayesian
        | OptimizerStrategy::Mipro
        | OptimizerStrategy::Evolutionary
        | OptimizerStrategy::Gepa => Err(OptimizerError::NotYetImplemented(strategy)),
    }?;
    ensure_candidates_record_strategy(strategy, candidates)
}

fn ensure_candidates_record_strategy(
    strategy: OptimizerStrategy,
    candidates: Vec<CandidateChange>,
) -> Result<Vec<CandidateChange>, OptimizerError> {
    if let Some(candidate) = candidates
        .iter()
        .find(|candidate| candidate.proposed_by != strategy)
    {
        return Err(OptimizerError::StrategyMismatch {
            expected: strategy,
            actual: candidate.proposed_by,
        });
    }
    Ok(candidates)
}

/// Which split of the optimization substrate a [`CaseScore`] belongs to.
///
/// The RSI optimizer searches candidates against `Train`/`Val` and decides
/// acceptance only on the held-out `Test` split (§21.4). The split assignment is
/// the *caller's* responsibility — it owns the dataset and its train/val/test
/// partition — so [`run_optimization_round`] never reshuffles or peeks at the
/// split substrate; it merely routes each [`CaseScore`] to the gate by its tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    /// Optimization split the proposer is allowed to fit against.
    Train,
    /// Validation split used for in-loop model selection / early signal.
    Val,
    /// Held-out **Test** split — the only split that can grant acceptance.
    Test,
}

/// One case's paired baseline-vs-candidate score, tagged with its [`Split`].
///
/// This is the unit the injected [`CandidateEvaluator`] returns. `baseline_score`
/// is the current policy's score on the case and `candidate_score` is the
/// candidate policy's score on the *same* case (paired), so the gate can compute
/// a paired lift. Scores are the evaluator's own metric in `[0, 1]` (higher is
/// better), matching the convention of `compare_paired_scores`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CaseScore {
    /// Which split this case belongs to.
    pub split: Split,
    /// Score of the current (baseline) policy on this case.
    pub baseline_score: f64,
    /// Score of the candidate policy on the same case (paired with baseline).
    pub candidate_score: f64,
    /// Optional pre-experiment covariate for CUPED variance reduction (§10.3 #4 /
    /// #436 item 4), consumed by the gate only when the round's design declares
    /// [`VarianceReduction::Cuped`]. It MUST be independent of the candidate under
    /// test — a per-case difficulty measured before this experiment is the
    /// canonical choice; **never** an arm's own score (that un-pairs the design,
    /// see [`beater_stats::cuped_adjust`]). `None` disables CUPED for the case; the
    /// gate only applies CUPED when *every* Test case carries one.
    #[serde(default)]
    pub covariate: Option<f64>,
}

/// Scores a proposed [`CandidateChange`] against a set of cases — the seam the
/// caller injects so [`run_optimization_round`] can drive the proposer → gate
/// loop *without* itself executing the candidate agent / LLM.
///
/// **Honest boundary.** Actually running the candidate policy over the cases
/// (re-prompting the agent, calling the model, executing tools, scoring the
/// outputs) is the caller's responsibility — it owns the agent runtime, the
/// provider credentials, and the dataset. This crate only orchestrates the
/// proposal and the statistical gate, so the "run the candidate" step is an
/// injected trait, not faked in-tree. Production wires a real evaluator (e.g. one
/// built on [`run_agent_experiment`] / [`run_judge_experiment`]); tests inject a
/// deterministic stub.
///
/// Implementations must return one [`CaseScore`] per case they were given,
/// tagged with the case's [`Split`]; the orchestrator partitions them and feeds
/// the held-out Test split to the gate.
#[async_trait]
pub trait CandidateEvaluator {
    /// Score `candidate`'s effect on `cases`, returning paired baseline-vs-candidate
    /// scores keyed by split. The slice is opaque (`serde_json::Value`) so the
    /// evaluator can carry whatever case identity / payload its runtime needs
    /// without coupling this crate to a concrete case type.
    async fn evaluate(
        &self,
        candidate: &CandidateChange,
        cases: &[Value],
    ) -> Result<Vec<CaseScore>, String>;
}

/// Configuration for a single [`run_optimization_round`].
///
/// The round reflects on `failures` to build a [`ProposalContext`], asks the
/// configured [`OptimizerStrategy`] for candidates, and routes each through the
/// held-out gate. The `cases` are handed verbatim to the [`CandidateEvaluator`];
/// the caller is responsible for their train/val/test split tagging.
#[derive(Clone, Debug)]
pub struct OptimizationRoundConfig {
    /// The improvement goal in natural language (§21.3 "goal + params").
    pub goal: String,
    /// The current prompt / lever text the proposer may rewrite.
    pub current_prompt: String,
    /// The round's failing examples, used to build the reflective context.
    pub failures: Vec<FailureExample>,
    /// The cases to score each candidate against (opaque payloads the injected
    /// evaluator understands). The evaluator tags each returned [`CaseScore`]
    /// with its [`Split`].
    pub cases: Vec<Value>,
    /// Which proposer to drive. `LlmRewrite` uses the async generation seam;
    /// every other implemented strategy uses the deterministic [`propose_with`]
    /// dispatch.
    pub strategy: OptimizerStrategy,
    /// Gate policy applied to the held-out **Test** split via
    /// [`compare_paired_scores`].
    pub gate_policy: GatePolicy,
    /// Multiple-comparison control across the *candidate family* proposed in this
    /// round (§10.3 #4 / #436). When a strategy emits more than one candidate,
    /// screening each against the same baseline inflates the family-wise false-win
    /// rate; this policy corrects for it. [`MultiplicityPolicy::Holm`] (the default)
    /// controls the family-wise error rate and uniformly dominates the single-step
    /// `alpha / comparison_count` Bonferroni already applied inside the gate;
    /// [`MultiplicityPolicy::BenjaminiHochberg`] controls the false-discovery rate;
    /// [`MultiplicityPolicy::None`] disables the family guard. It is orthogonal to
    /// `gate_policy.comparison_count` (which pre-declares a fixed comparison
    /// budget): the guard only ever *withdraws* a within-round `Pass`, never grants
    /// one, and is a no-op when the round proposes a single candidate.
    pub multiplicity: MultiplicityPolicy,
    /// Variance-reduction policy for the held-out gate (§10.3 #4 / #436 item 4).
    /// [`VarianceReduction::Cuped`] regresses a pre-experiment covariate (carried
    /// per-case on [`CaseScore::covariate`]) out of the paired difference before
    /// the gate's t-test, tightening the CI without moving the point estimate — so
    /// a borderline-underpowered round can resolve to a `Pass` on the same data.
    /// [`VarianceReduction::None`] (the default) leaves the gate unchanged. Like
    /// the multiplicity guard it is safe-by-construction: it never manufactures a
    /// win, and it is a no-op unless every Test case carries a covariate.
    pub variance_reduction: VarianceReduction,
    /// Largest benign generalization gap for [`assess_generalization_gap`]
    /// (e.g. `0.0` — "held-out lift must not be significantly below the
    /// optimization-split lift").
    pub overfit_tolerance: f64,
    /// Bootstrap confidence level for the generalization-gap CI.
    pub overfit_confidence: f64,
    /// Number of bootstrap resamples for the generalization-gap CI.
    pub overfit_resamples: usize,
    /// Seed for the (deterministic) generalization-gap bootstrap.
    pub overfit_seed: u64,
}

impl OptimizationRoundConfig {
    /// Sensible anti-overfit defaults: zero-tolerance gap, 95% bootstrap CI,
    /// 2000 resamples, fixed seed, and Holm family-wise multiplicity control
    /// across the candidate family. The caller still must set `goal`,
    /// `current_prompt`, `failures`, `cases`, `strategy`, and `gate_policy`.
    pub fn new(
        goal: impl Into<String>,
        current_prompt: impl Into<String>,
        failures: Vec<FailureExample>,
        cases: Vec<Value>,
        strategy: OptimizerStrategy,
        gate_policy: GatePolicy,
    ) -> Self {
        Self {
            goal: goal.into(),
            current_prompt: current_prompt.into(),
            failures,
            cases,
            strategy,
            gate_policy,
            multiplicity: MultiplicityPolicy::Holm,
            variance_reduction: VarianceReduction::None,
            overfit_tolerance: 0.0,
            overfit_confidence: 0.95,
            overfit_resamples: 2000,
            overfit_seed: 1,
        }
    }
}

/// The per-candidate verdict produced by [`run_optimization_round`].
///
/// Carries both gate decisions so the audit trail records *why* a candidate was
/// or wasn't accepted: the held-out **Test** [`GateDecision`] AND the
/// generalization-gap [`GapAssessment`] (the §21.4 anti-overfit check). A
/// candidate is `accepted` only when the Test gate `Pass`es **and** the gap
/// assessment does not flag overfitting — proposal is never acceptance.
#[derive(Clone, Debug)]
pub struct CandidateEvaluation {
    /// The proposed change that was evaluated.
    pub candidate: CandidateChange,
    /// The held-out **Test**-split gate comparison (`compare_paired_scores`).
    pub gate: ExperimentComparison,
    /// The generalization-gap assessment (optimization split vs. held-out split).
    pub overfit: GapAssessment,
    /// `true` iff the Test gate passed AND no significant generalization gap was
    /// detected. This is the only path to acceptance.
    pub accepted: bool,
}

/// The outcome of one [`run_optimization_round`].
///
/// `accepted` is the single candidate (if any) that cleared *both* gates; when
/// multiple candidates clear, the first in proposal order wins (deterministic).
/// `evaluated` records every candidate's full verdict for the audit trail.
#[derive(Clone, Debug)]
pub struct OptimizationOutcome {
    /// The accepted candidate, or `None` when no candidate cleared the gates.
    pub accepted: Option<CandidateChange>,
    /// Per-candidate verdicts, in proposal order.
    pub evaluated: Vec<CandidateEvaluation>,
}

/// Drive one optimization round end-to-end: **propose → evaluate → gate**.
///
/// This is the first real (non-test) caller of the proposer seam landed in
/// 5e299f1: it builds a [`ProposalContext`] from the round's failing cases,
/// asks the configured [`OptimizerStrategy`] for candidate changes (the live
/// [`LlmRewrite::propose_async`] generation path for `LlmRewrite`, the
/// deterministic [`propose_with`] dispatch for strategies like `ParamSearch`),
/// and then runs **every** candidate through the existing held-out gate before
/// any acceptance.
///
/// # The gate is reused, not reinvented
///
/// Each candidate is scored by the injected [`CandidateEvaluator`] (which the
/// caller owns — see its docs), yielding paired baseline-vs-candidate
/// [`CaseScore`]s tagged by [`Split`]. Acceptance then requires **both**:
///
/// 1. The held-out **Test** split must `Pass` the existing
///    [`compare_paired_scores`] gate (§21.3): a real paired test + CI against the
///    regression bound. The Test split is the *only* split that can grant
///    acceptance.
/// 2. The §21.4 anti-overfit guardrail [`assess_generalization_gap`] must NOT
///    flag a significant gap between the optimization (Train+Val) lift and the
///    held-out (Test) lift. A candidate that looks good only on data the
///    optimizer could see is rejected even if it marginally passes (1).
///
/// Both come straight from `beater-eval` / `beater-stats`; this function does no
/// statistics of its own — it routes scores to the existing functions and
/// records their verdicts.
///
/// # Proposal is not acceptance
///
/// The proposer only emits [`CandidateChange`]s; the gate decides. A candidate is
/// returned in [`OptimizationOutcome::accepted`] only when it clears both checks,
/// and a candidate the gate *cannot judge* (e.g. an under-powered Test split)
/// surfaces as a typed [`OptimizerError`] rather than a silent accept.
///
/// # What this is NOT
///
/// This orchestrates proposal + gating only. Actually executing the candidate
/// agent/LLM over the cases is the [`CandidateEvaluator`]'s job (caller-supplied),
/// and the production CLI / HTTP endpoint that invokes this round, persists the
/// outcome, and applies an accepted change is the next layer up — intentionally
/// not built here.
pub async fn run_optimization_round(
    cfg: OptimizationRoundConfig,
    generator: &dyn TextGenerator,
    credentials: ProviderCredentials,
    evaluate_candidate: &dyn CandidateEvaluator,
) -> Result<OptimizationOutcome, OptimizerError> {
    if cfg.goal.trim().is_empty() {
        return Err(OptimizerError::InvalidConfig(
            "goal must not be empty".to_string(),
        ));
    }

    // 1. Build the reflective context from the round's real failing cases.
    let ctx = ProposalContext::from_failures(&cfg.goal, &cfg.current_prompt, &cfg.failures);

    // 2. Propose candidate(s) via the configured strategy. LlmRewrite consults
    //    the live generation seam; every other implemented strategy is
    //    deterministic via `propose_with`.
    let candidates = match cfg.strategy {
        OptimizerStrategy::LlmRewrite => {
            LlmRewrite
                .propose_async(&ctx, generator, credentials)
                .await?
        }
        other => propose_with(other, &ctx)?,
    };

    let mut evaluated = Vec::with_capacity(candidates.len());
    // Each candidate's held-out Test-split candidate scores, kept parallel to
    // `evaluated` so the best-arm race can compare arms after gating.
    let mut test_arms: Vec<Vec<f64>> = Vec::with_capacity(candidates.len());

    // 3 + 4. For each candidate: score its cases (injected evaluator), then run
    //         the REAL held-out gate + anti-overfit assessment.
    for candidate in candidates {
        let scores = evaluate_candidate
            .evaluate(&candidate, &cfg.cases)
            .await
            .map_err(OptimizerError::EvaluationFailed)?;

        let evaluation = gate_candidate(&candidate, &scores, &cfg)?;
        // gate_candidate already proved the Test split is non-empty.
        let (_, test_candidate) = split_scores(&scores, Split::Test);
        evaluated.push(evaluation);
        test_arms.push(test_candidate);
    }

    // 5. Family-wise multiple-comparison control across the candidates proposed
    //    this round (§10.3 #4 / #436 item 3). Screening N candidates against one
    //    baseline inflates the family-wise false-win rate; a candidate that clears
    //    the gate on its own but does not survive the correction has its `Pass`
    //    withdrawn to `Inconclusive` (and loses acceptance). No-op for a single
    //    candidate — a family of one needs no correction — so the pre-declared
    //    `comparison_count` path (and its e2e coverage) is unchanged.
    apply_family_multiplicity(&mut evaluated, cfg.multiplicity, cfg.gate_policy.alpha)?;

    // 6. Best-arm race across the *accepted* candidates (§10.3 / #436 item 2):
    //    drop any that a strictly-better accepted candidate confidently dominates
    //    on the held-out split, so acceptance never lands on a dominated arm. A
    //    no-op unless two or more candidates are accepted and their intervals are
    //    disjoint, so single-candidate and tied rounds are unchanged.
    apply_best_arm_race(&mut evaluated, &test_arms, cfg.gate_policy.alpha)?;

    // First candidate (in proposal order) that still clears every gate — the
    // held-out gate, anti-overfit guardrail, family correction, and best-arm
    // race — wins. Deterministic in proposal order.
    let accepted = evaluated
        .iter()
        .find(|evaluation| evaluation.accepted)
        .map(|evaluation| evaluation.candidate.clone());

    Ok(OptimizationOutcome {
        accepted,
        evaluated,
    })
}

/// Apply a family-wise / false-discovery multiple-comparison correction across a
/// round's candidate evaluations (§10.3 #4 / #436 item 3).
///
/// The gate already applies the pre-declared single-step Bonferroni
/// (`alpha / comparison_count`) to each candidate's CI. This adds the *within-round*
/// family correction across the candidates actually proposed: the raw per-candidate
/// p-values (`ExperimentComparison::p_value`, which is independent of the CI's alpha)
/// are run through Holm (FWER) or Benjamini-Hochberg (FDR). A candidate whose gate
/// decision is `Pass` but which does **not** survive the correction has its decision
/// downgraded to `Inconclusive` and its acceptance withdrawn — mirroring the
/// `EvalDesign::permit_pass` downgrade discipline: the guard can only ever remove a
/// win, never manufacture one.
///
/// It is a deliberate no-op when:
/// * the policy is [`MultiplicityPolicy::None`], or
/// * fewer than two candidates were evaluated (a family of one has no multiplicity),
///
/// so single-candidate rounds — including the pre-declared `comparison_count` gate
/// path — are byte-for-byte unchanged.
fn apply_family_multiplicity(
    evaluated: &mut [CandidateEvaluation],
    policy: MultiplicityPolicy,
    alpha: f64,
) -> Result<(), OptimizerError> {
    if matches!(policy, MultiplicityPolicy::None) || evaluated.len() < 2 {
        return Ok(());
    }

    let p_values: Vec<f64> = evaluated
        .iter()
        .map(|evaluation| evaluation.gate.p_value)
        .collect();

    let decisions = match policy {
        MultiplicityPolicy::Holm => holm_bonferroni(&p_values, alpha),
        MultiplicityPolicy::BenjaminiHochberg => benjamini_hochberg(&p_values, alpha),
        // `None` handled above.
        MultiplicityPolicy::None => return Ok(()),
    }
    .map_err(|err| OptimizerError::GateFailed(err.to_string()))?;

    for (evaluation, decision) in evaluated.iter_mut().zip(decisions) {
        // Only a genuine `Pass` can be withdrawn; a regression failure or an
        // already-inconclusive verdict is left untouched. Survival of the family
        // correction is the `reject`-the-null (candidate really beats baseline)
        // decision at the family level.
        if evaluation.gate.decision == GateDecision::Pass && !decision.reject {
            evaluation.gate.decision = GateDecision::Inconclusive;
            evaluation.accepted = false;
        }
    }

    Ok(())
}

/// Best-arm race across a round's *accepted* candidates (§10.3 / #436 item 2).
///
/// Among the candidates that cleared the gate, the anti-overfit guardrail, and
/// the family correction, a Hoeffding race on their held-out Test scores drops
/// any candidate that a strictly-better accepted candidate *confidently
/// dominates* — so the round never accepts a candidate that a demonstrably better
/// one beats at the same eval budget. The gate decision of a dropped candidate is
/// left as `Pass` (it genuinely cleared its own gate) for the audit trail; only
/// its acceptance is withdrawn.
///
/// A deliberate no-op unless at least two candidates are still accepted, and even
/// then it only eliminates arms with *disjoint* confidence intervals — tied or
/// statistically-indistinguishable candidates all survive, so deterministic
/// proposal-order selection is preserved. The Hoeffding range is `1.0` because
/// eval scores are bounded to `[0, 1]`. The empirically-best arm can never be
/// eliminated, so the accepted set is never emptied.
fn apply_best_arm_race(
    evaluated: &mut [CandidateEvaluation],
    test_arms: &[Vec<f64>],
    alpha: f64,
) -> Result<(), OptimizerError> {
    let accepted_indices: Vec<usize> = evaluated
        .iter()
        .enumerate()
        .filter(|(_, evaluation)| evaluation.accepted)
        .map(|(index, _)| index)
        .collect();
    if accepted_indices.len() < 2 {
        return Ok(());
    }

    let arms: Vec<&[f64]> = accepted_indices
        .iter()
        .map(|&index| test_arms[index].as_slice())
        .collect();
    let outcome = hoeffding_race(&arms, 1.0, alpha)
        .map_err(|err| OptimizerError::GateFailed(err.to_string()))?;

    // `survivors` are positions within `arms`, i.e. into `accepted_indices`.
    for (position, &eval_index) in accepted_indices.iter().enumerate() {
        if !outcome.survivors.contains(&position) {
            evaluated[eval_index].accepted = false;
        }
    }

    Ok(())
}

/// Route one candidate's per-case scores through the held-out Test gate and the
/// generalization-gap guardrail, returning the combined verdict. Pure plumbing
/// over `compare_paired_scores` + `assess_generalization_gap` — no bespoke stats.
fn gate_candidate(
    candidate: &CandidateChange,
    scores: &[CaseScore],
    cfg: &OptimizationRoundConfig,
) -> Result<CandidateEvaluation, OptimizerError> {
    // Held-out Test split: the only split that can grant acceptance.
    let (test_baseline, test_candidate) = split_scores(scores, Split::Test);
    if test_baseline.is_empty() {
        return Err(OptimizerError::GateFailed(
            "no held-out Test cases were scored; the gate cannot grant acceptance".to_string(),
        ));
    }

    // Optimization split = Train + Val (everything the proposer could see). The
    // generalization gap compares this lift against the held-out Test lift.
    let (optimize_baseline, optimize_candidate): (Vec<f64>, Vec<f64>) = scores
        .iter()
        .filter(|s| matches!(s.split, Split::Train | Split::Val))
        .map(|s| (s.baseline_score, s.candidate_score))
        .unzip();
    if optimize_baseline.is_empty() {
        return Err(OptimizerError::GateFailed(
            "no Train/Val cases were scored; cannot assess the generalization gap".to_string(),
        ));
    }

    // (1) Real held-out Test gate — reused verbatim from beater-eval, and now
    // routed through the pre-registration design so the RSI acceptance decision
    // enforces `EvalDesign::permit_pass` too (a structurally-invalid design can
    // never certify a Pass). The conservative default design always permits pass,
    // so this changes no accept/reject outcome (§1 #9, §10.3).
    let mut gate_design = conservative_gate_design(&cfg.gate_policy, test_baseline.len());
    gate_design.variance_reduction = cfg.variance_reduction.clone();
    // Pre-experiment covariate for CUPED, only when *every* Test case carries one
    // (a partial covariate is a data bug — fall back to the unadjusted gate rather
    // than impute). `compare_paired_scores_cuped` further no-ops unless the design
    // pre-registers CUPED, so this changes nothing for the default (None) policy.
    let test_covariate = split_covariate(scores, Split::Test);
    let gate = compare_paired_scores_cuped(
        &test_baseline,
        &test_candidate,
        test_covariate.as_deref(),
        &cfg.gate_policy,
        &gate_design,
    )
    .map_err(|err| OptimizerError::GateFailed(err.to_string()))?;

    // (2) Real anti-overfit guardrail — reused verbatim from beater-stats.
    let overfit = assess_generalization_gap(
        &optimize_baseline,
        &optimize_candidate,
        &test_baseline,
        &test_candidate,
        cfg.overfit_tolerance,
        cfg.overfit_confidence,
        cfg.overfit_resamples,
        cfg.overfit_seed,
    )
    .map_err(|err| OptimizerError::GateFailed(err.to_string()))?;

    // Acceptance requires BOTH: Test passes AND no significant generalization gap.
    let accepted = gate.decision == GateDecision::Pass && !overfit.overfit;

    Ok(CandidateEvaluation {
        candidate: candidate.clone(),
        gate,
        overfit,
        accepted,
    })
}

/// Collect the paired (baseline, candidate) score vectors for a single [`Split`],
/// preserving case order so the pairing stays aligned.
fn split_scores(scores: &[CaseScore], split: Split) -> (Vec<f64>, Vec<f64>) {
    scores
        .iter()
        .filter(|s| s.split == split)
        .map(|s| (s.baseline_score, s.candidate_score))
        .unzip()
}

/// The per-case CUPED covariates for a single [`Split`], in the same case order
/// as [`split_scores`]. Returns `Some` only when *every* case in the split carries
/// a covariate (and the split is non-empty); a partial covariate is a data bug, so
/// the gate falls back to the unadjusted path rather than imputing a value.
fn split_covariate(scores: &[CaseScore], split: Split) -> Option<Vec<f64>> {
    let in_split = scores.iter().filter(|s| s.split == split);
    let covariates: Vec<f64> = in_split.clone().filter_map(|s| s.covariate).collect();
    let count = in_split.count();
    (count > 0 && covariates.len() == count).then_some(covariates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{
        DatasetId, EnvironmentId, ProjectId, ProviderSecretId, Sha256Hash, SpanId, TenantId,
        TraceId,
    };
    use beater_datasets::DatasetCase;
    use beater_eval::{EvalError, EvaluatorKind, StatisticalTest};
    use beater_judge::{JudgeAuditRecord, JudgeBrokerError};
    use serde_json::json;

    fn proposal_context() -> ProposalContext {
        ProposalContext::new(
            "reduce hallucinations on factual lookups",
            "You are a helpful assistant.",
            vec![
                FailureExample::from_parts(
                    "What year did the Eiffel Tower open?",
                    Some("1889".to_string()),
                    "1887",
                    0.0,
                    None,
                ),
                FailureExample::from_parts(
                    "Who wrote Hamlet?",
                    Some("Shakespeare".to_string()),
                    "Marlowe",
                    0.1,
                    None,
                ),
                FailureExample::from_parts(
                    "Capital of Australia?",
                    Some("Canberra".to_string()),
                    "",
                    0.0,
                    Some("tool timeout after 1200ms".to_string()),
                ),
                FailureExample::from_parts(
                    "Capital of Canada?",
                    Some("Ottawa".to_string()),
                    "",
                    0.0,
                    Some("tool timeout after 950ms".to_string()),
                ),
            ],
        )
    }

    /// A fake [`TextGenerator`] that returns a canned completion as raw text,
    /// standing in for a live model so [`LlmRewrite::propose_async`] can be
    /// tested without network access. The production path calls the real
    /// generation seam; only the test substitutes this mock.
    struct FakeRewriteGenerator {
        completion: String,
    }

    #[async_trait]
    impl TextGenerator for FakeRewriteGenerator {
        async fn generate(
            &self,
            req: beater_judge::GenerationRequest,
            _credentials: ProviderCredentials,
        ) -> beater_judge::JudgeProviderResult<beater_judge::GenerationResponse> {
            // Assert the reflective brief actually reached the generator as the
            // PLAIN user prompt — not a judge rubric / scoring request.
            assert!(req.prompt.contains("GOAL:"));
            assert!(req.prompt.contains("FAILURE STATS:"));
            // And the call carries a generation system prompt, not the judge one.
            let system = req.system.as_deref().unwrap_or("");
            assert!(system.contains("prompt engineer"));
            assert!(!system.contains("strict evaluation judge"));
            Ok(beater_judge::GenerationResponse {
                text: self.completion.clone(),
                model: Some(req.model),
            })
        }
    }

    #[test]
    fn optimizer_strategy_defaults_to_llm_rewrite() {
        assert_eq!(OptimizerStrategy::default(), OptimizerStrategy::LlmRewrite);
    }

    #[test]
    fn optimizer_strategy_serializes_named_variants() {
        for (strategy, serialized) in [
            (OptimizerStrategy::LlmRewrite, "llm_rewrite"),
            (OptimizerStrategy::FewShotBayesian, "few_shot_bayesian"),
            (OptimizerStrategy::Mipro, "mipro"),
            (OptimizerStrategy::Evolutionary, "evolutionary"),
            (OptimizerStrategy::Gepa, "gepa"),
            (OptimizerStrategy::ParamSearch, "param_search"),
        ] {
            let value = serde_json::to_value(strategy).unwrap_or_else(|err| panic!("{err}"));
            assert_eq!(value, json!(serialized));
            let round_trip: OptimizerStrategy =
                serde_json::from_value(value).unwrap_or_else(|err| panic!("{err}"));
            assert_eq!(round_trip, strategy);
        }
    }

    #[test]
    fn candidate_change_serializes_strategy_provenance() {
        let candidate = CandidateChange {
            kind: ChangeKind::SystemPrompt,
            target: "system_prompt".to_string(),
            description: "Try a MIPRO candidate".to_string(),
            rationale: "records the optimizer family for the gate audit".to_string(),
            proposed_by: OptimizerStrategy::Mipro,
        };

        let value = serde_json::to_value(&candidate).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(value["proposed_by"], json!("mipro"));

        let round_trip: CandidateChange =
            serde_json::from_value(value).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(round_trip.proposed_by, OptimizerStrategy::Mipro);
    }

    #[test]
    fn candidate_strategy_guard_rejects_mismatched_provenance() {
        let candidate = CandidateChange {
            kind: ChangeKind::SystemPrompt,
            target: "system_prompt".to_string(),
            description: "Mis-stamped candidate".to_string(),
            rationale: "would confuse gate audit provenance".to_string(),
            proposed_by: OptimizerStrategy::LlmRewrite,
        };

        let err = ensure_candidates_record_strategy(OptimizerStrategy::Mipro, vec![candidate])
            .err()
            .unwrap_or_else(|| panic!("expected strategy mismatch"));
        assert_eq!(
            err,
            OptimizerError::StrategyMismatch {
                expected: OptimizerStrategy::Mipro,
                actual: OptimizerStrategy::LlmRewrite,
            }
        );
    }

    #[test]
    fn llm_rewrite_proposes_at_least_one_candidate() {
        let candidates = LlmRewrite
            .propose(&proposal_context())
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].proposed_by, OptimizerStrategy::LlmRewrite);
        assert_eq!(candidates[0].kind, ChangeKind::SystemPrompt);
        assert!(!candidates[0].rationale.is_empty());
    }

    #[test]
    fn llm_rewrite_rejects_empty_goal() {
        let ctx = ProposalContext::from_goal("   ", "x");
        let err = LlmRewrite
            .propose(&ctx)
            .err()
            .unwrap_or_else(|| panic!("expected invalid-context error"));
        assert!(matches!(err, OptimizerError::InvalidContext(_)));
    }

    #[test]
    fn dispatch_routes_llm_rewrite_to_implementation() {
        let candidates = propose_with(OptimizerStrategy::LlmRewrite, &proposal_context())
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].proposed_by, OptimizerStrategy::LlmRewrite);
    }

    #[test]
    fn unimplemented_strategies_return_typed_not_yet_implemented() {
        let ctx = proposal_context();
        for strategy in [
            OptimizerStrategy::FewShotBayesian,
            OptimizerStrategy::Mipro,
            OptimizerStrategy::Evolutionary,
            OptimizerStrategy::Gepa,
        ] {
            let err = propose_with(strategy, &ctx)
                .err()
                .unwrap_or_else(|| panic!("expected NotYetImplemented for {strategy:?}"));
            assert_eq!(err, OptimizerError::NotYetImplemented(strategy));
        }
    }

    #[test]
    fn proposal_context_computes_stats_and_signatures() {
        let ctx = proposal_context();
        assert_eq!(ctx.stats.n_failures, 4);
        // Two timeouts (masked to the same signature) should be the top bucket.
        let top = &ctx.failure_signatures[0];
        assert!(top.signature.contains("timeout after <n>ms"), "{top:?}");
        assert_eq!(top.count, 2);
        // Mean score is below the failing band.
        assert!(ctx.stats.mean_score < 0.2);
        // All four failures live in the lowest score bucket.
        assert_eq!(ctx.stats.score_buckets[0], 4);
    }

    #[test]
    fn proposal_context_roundtrips_serde() {
        let ctx = proposal_context();
        let json = serde_json::to_string(&ctx).unwrap_or_else(|err| panic!("{err}"));
        let back: ProposalContext =
            serde_json::from_str(&json).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(ctx, back);
    }

    #[test]
    fn from_goal_constructs_empty_failure_context() {
        let ctx = ProposalContext::from_goal("g", "p");
        assert_eq!(ctx.stats.n_failures, 0);
        assert_eq!(ctx.stats.mean_score, 0.0);
        assert!(ctx.failure_signatures.is_empty());
    }

    #[test]
    fn from_failures_populates_stats_and_signatures_from_real_cases() {
        // The production (#435) entry point: real failing cases in, enriched
        // context out — stats and signatures computed from the supplied data,
        // not hand-set by a test fixture.
        let failures = vec![
            FailureExample::from_parts("What is 2+2?", Some("4".to_string()), "5", 0.0, None),
            FailureExample::from_parts(
                "Call the API",
                None,
                "boom",
                0.1,
                Some("timeout after 1200ms".to_string()),
            ),
            FailureExample::from_parts(
                "Call the API again",
                None,
                "boom",
                0.15,
                Some("timeout after 950ms".to_string()),
            ),
        ];
        let ctx = ProposalContext::from_failures(
            "make the agent reliable",
            "You are an assistant.",
            &failures,
        );
        assert_eq!(ctx.stats.n_failures, 3);
        assert!(ctx.stats.mean_score < 0.2);
        // The two masked timeouts collapse into one signature with count 2.
        let top = &ctx.failure_signatures[0];
        assert!(top.signature.contains("timeout after <n>ms"), "{top:?}");
        assert_eq!(top.count, 2);
        // The context is usable by the proposer with no further plumbing.
        let candidates = LlmRewrite
            .propose(&ctx)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(candidates.len(), 1);
    }

    #[tokio::test]
    async fn llm_rewrite_async_calls_generator_and_returns_rewrite() {
        let generator = FakeRewriteGenerator {
            completion: "You are a meticulous assistant. Cite a source for every factual claim \
                         and say 'I am not sure' when uncertain."
                .to_string(),
        };
        let credentials = ProviderCredentials::new("openai", "sk-test");
        let candidates = LlmRewrite
            .propose_async(&proposal_context(), &generator, credentials)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, ChangeKind::SystemPrompt);
        assert_eq!(candidates[0].proposed_by, OptimizerStrategy::LlmRewrite);
        // The generator's raw text becomes the candidate target (the new prompt).
        assert!(candidates[0].target.contains("meticulous assistant"));
        // Proposal-not-acceptance invariant is preserved in the rationale.
        assert!(candidates[0].rationale.contains("gate decides"));
    }

    #[tokio::test]
    async fn llm_rewrite_async_rejects_empty_generator_output() {
        let generator = FakeRewriteGenerator {
            completion: "   ".to_string(),
        };
        let credentials = ProviderCredentials::new("openai", "sk-test");
        let err = LlmRewrite
            .propose_async(&proposal_context(), &generator, credentials)
            .await
            .err()
            .unwrap_or_else(|| panic!("expected ProposerFailed"));
        assert!(matches!(err, OptimizerError::ProposerFailed(_)));
    }

    /// A deterministic [`CandidateEvaluator`] driven by two closures: one that
    /// produces the per-case baseline/candidate scores for the optimization
    /// (Train+Val) split, and one for the held-out Test split. This lets a test
    /// dial in "improves everywhere" vs. "improves only on the optimization
    /// split" without any live agent or network.
    struct ScriptedEvaluator {
        /// (baseline, candidate) score for each Train/Val case.
        optimize: Vec<(f64, f64)>,
        /// (baseline, candidate) score for each held-out Test case.
        test: Vec<(f64, f64)>,
    }

    #[async_trait]
    impl CandidateEvaluator for ScriptedEvaluator {
        async fn evaluate(
            &self,
            _candidate: &CandidateChange,
            _cases: &[Value],
        ) -> Result<Vec<CaseScore>, String> {
            // Split Train/Val arbitrarily; both count as the optimization split
            // for the generalization-gap assessment.
            let mut out = Vec::new();
            for (i, (b, c)) in self.optimize.iter().enumerate() {
                let split = if i % 2 == 0 { Split::Train } else { Split::Val };
                out.push(CaseScore {
                    split,
                    baseline_score: *b,
                    candidate_score: *c,
                    covariate: None,
                });
            }
            for (b, c) in &self.test {
                out.push(CaseScore {
                    split: Split::Test,
                    baseline_score: *b,
                    candidate_score: *c,
                    covariate: None,
                });
            }
            Ok(out)
        }
    }

    fn round_config(strategy: OptimizerStrategy) -> OptimizationRoundConfig {
        OptimizationRoundConfig::new(
            "reduce hallucinations on factual lookups",
            "You are a helpful assistant.",
            proposal_context().failing_examples,
            // Cases are opaque to the orchestrator; the scripted evaluator ignores them.
            (0..12).map(|i| json!({ "case": i })).collect(),
            strategy,
            GatePolicy {
                min_sample_size: 6,
                max_regression: 0.0,
                alpha: 0.05,
                comparison_count: 1,
            },
        )
    }

    /// A candidate + per-case scores where the held-out Test lift is a fixed
    /// +0.05 buried under noise that a pre-experiment covariate almost fully
    /// explains, plus a clean matching optimization-split lift (so there is no
    /// generalization gap). This is the fixture the CUPED wiring test drives
    /// straight through `gate_candidate`.
    fn cuped_round_scores() -> Vec<CaseScore> {
        let covariate = [0.0, 1.0, 0.1, 0.9, 0.2, 0.8, 0.3, 0.7, 0.4, 0.6, 0.45, 0.55];
        let candidate = [
            0.405, 0.695, 0.435, 0.665, 0.465, 0.635, 0.495, 0.605, 0.525, 0.575, 0.54, 0.56,
        ];
        let mut scores = Vec::new();
        // Held-out Test split: noisy but covariate-explained (baseline constant, so
        // the covariate is independent of it — a valid difficulty proxy).
        for (x, c) in covariate.iter().zip(candidate.iter()) {
            scores.push(CaseScore {
                split: Split::Test,
                baseline_score: 0.5,
                candidate_score: *c,
                covariate: Some(*x),
            });
        }
        // Optimization split: the same +0.05 mean lift, cleanly, so the
        // generalization gap is ~0 and the anti-overfit guardrail stays quiet.
        for i in 0..8 {
            let split = if i % 2 == 0 { Split::Train } else { Split::Val };
            scores.push(CaseScore {
                split,
                baseline_score: 0.50,
                candidate_score: 0.55,
                covariate: None,
            });
        }
        scores
    }

    fn cuped_candidate() -> CandidateChange {
        CandidateChange {
            kind: ChangeKind::SystemPrompt,
            target: "system_prompt".to_string(),
            description: "tighten the instructions".to_string(),
            rationale: "reduce variance on hard cases".to_string(),
            proposed_by: OptimizerStrategy::LlmRewrite,
        }
    }

    /// The load-bearing wiring test: the SAME held-out scores are `Inconclusive`
    /// through the default (no variance reduction) gate, but a pre-registered CUPED
    /// covariate — threaded via `CaseScore::covariate` and
    /// `OptimizationRoundConfig::variance_reduction` — resolves it to a `Pass` via
    /// the regression estimator. Proves the covariate and its population mean
    /// actually reach `compare_paired_scores_cuped` inside `gate_candidate`.
    #[test]
    fn cuped_covariate_flips_the_round_gate_from_inconclusive_to_pass() {
        let scores = cuped_round_scores();
        let candidate = cuped_candidate();

        // Default policy: no variance reduction → the noise leaves it underpowered.
        let plain_cfg = round_config(OptimizerStrategy::LlmRewrite);
        assert_eq!(plain_cfg.variance_reduction, VarianceReduction::None);
        let plain =
            gate_candidate(&candidate, &scores, &plain_cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            plain.gate.decision,
            GateDecision::Inconclusive,
            "without CUPED the round gate is underpowered"
        );
        assert!(!plain.accepted);

        // Same scores + a pre-registered covariate (known population mean 0.7,
        // above this sample's 0.5) → the regression estimator corrects the lift
        // upward and the variance-reduced CI clears the bound → Pass.
        let mut cuped_cfg = round_config(OptimizerStrategy::LlmRewrite);
        cuped_cfg.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean: 0.7,
        };
        let cuped =
            gate_candidate(&candidate, &scores, &cuped_cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            cuped.gate.decision,
            GateDecision::Pass,
            "CUPED resolves the round gate to a Pass"
        );
        // The regression estimator MOVES the estimate off the plain mean and the
        // variance-reduced CI is narrower — both are how CUPED earns the Pass.
        assert!(
            cuped.gate.delta > plain.gate.delta,
            "estimate corrected upward"
        );
        assert!(
            cuped.gate.ci_high - cuped.gate.ci_low < plain.gate.ci_high - plain.gate.ci_low,
            "CUPED must narrow the round gate's CI"
        );
        // No generalization gap in this fixture, so the Pass is a real acceptance.
        assert!(
            cuped.accepted,
            "a covariate-resolved Pass with no overfit gap must be accepted"
        );
    }

    /// A partial covariate (not every Test case carries one) must fall back to the
    /// unadjusted gate rather than impute — identical to the default policy.
    #[test]
    fn cuped_round_falls_back_when_covariate_is_partial() {
        let mut scores = cuped_round_scores();
        // Drop the covariate on one Test case.
        if let Some(first_test) = scores.iter_mut().find(|s| s.split == Split::Test) {
            first_test.covariate = None;
        }
        let candidate = cuped_candidate();
        let mut cfg = round_config(OptimizerStrategy::LlmRewrite);
        cfg.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean: 0.7,
        };
        let evaluated =
            gate_candidate(&candidate, &scores, &cfg).unwrap_or_else(|err| panic!("{err}"));
        // Fell back to the plain paired-t, so still Inconclusive on the raw noise.
        assert_eq!(evaluated.gate.decision, GateDecision::Inconclusive);
        assert_eq!(evaluated.gate.test, beater_eval::StatisticalTest::PairedT);
    }

    /// Test A: a candidate that improves uniformly across every split clears the
    /// held-out Test gate AND the anti-overfit guardrail → accepted.
    #[tokio::test]
    async fn round_accepts_candidate_that_improves_across_all_splits() {
        let generator = FakeRewriteGenerator {
            completion: "You are a meticulous assistant. Cite a source for every claim."
                .to_string(),
        };
        // Baseline 0.5 everywhere, candidate 0.9 everywhere — same lift on the
        // optimization and held-out splits, so no generalization gap.
        let evaluator = ScriptedEvaluator {
            optimize: vec![(0.5, 0.9); 6],
            test: vec![(0.5, 0.9); 6],
        };
        let outcome = run_optimization_round(
            round_config(OptimizerStrategy::LlmRewrite),
            &generator,
            ProviderCredentials::new("openai", "sk-test"),
            &evaluator,
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(outcome.evaluated.len(), 1, "LlmRewrite emits one candidate");
        let eval = &outcome.evaluated[0];
        assert_eq!(eval.gate.decision, GateDecision::Pass);
        assert!(
            !eval.overfit.overfit,
            "uniform lift has no generalization gap"
        );
        assert!(eval.accepted);
        let accepted = outcome
            .accepted
            .unwrap_or_else(|| panic!("expected an accepted candidate"));
        assert_eq!(accepted.proposed_by, OptimizerStrategy::LlmRewrite);
        assert!(accepted.target.contains("meticulous assistant"));
    }

    /// Test B: a candidate that improves only on Train/Val but reverts to
    /// baseline on the held-out Test split is REJECTED — proving the loop honors
    /// the §21.4 anti-overfit gate (a candidate that looks good only on data the
    /// optimizer could see is never accepted).
    #[tokio::test]
    async fn round_rejects_candidate_that_overfits_the_optimization_split() {
        let generator = FakeRewriteGenerator {
            completion: "You are a meticulous assistant.".to_string(),
        };
        // Big lift on the optimization split (0.2 -> 0.95) but ZERO lift on the
        // held-out Test split (0.2 -> 0.2): the classic overfit signature.
        let evaluator = ScriptedEvaluator {
            optimize: vec![(0.2, 0.95); 6],
            test: vec![(0.2, 0.2); 6],
        };
        let outcome = run_optimization_round(
            round_config(OptimizerStrategy::LlmRewrite),
            &generator,
            ProviderCredentials::new("openai", "sk-test"),
            &evaluator,
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        let eval = &outcome.evaluated[0];
        // The generalization-gap guardrail flags the candidate.
        assert!(
            eval.overfit.overfit,
            "expected an overfit flag: gap={:?}",
            eval.overfit
        );
        assert!(eval.overfit.gap_ci_low > 0.0);
        // Even though the Test split shows no regression (so the Test gate alone
        // would Pass), acceptance is denied because the gap guardrail fired.
        assert!(!eval.accepted, "overfit candidate must not be accepted");
        assert!(
            outcome.accepted.is_none(),
            "no candidate should be accepted in the overfit round"
        );
    }

    /// A round with a deterministic (LLM-free) strategy still drives the gate:
    /// ParamSearch proposes a grid, each point is gated, and the generator is
    /// never consulted.
    #[tokio::test]
    async fn round_drives_deterministic_param_search_through_the_gate() {
        // A generator that panics if called — proves ParamSearch never touches it.
        struct PanicGenerator;
        #[async_trait]
        impl TextGenerator for PanicGenerator {
            async fn generate(
                &self,
                _req: beater_judge::GenerationRequest,
                _credentials: ProviderCredentials,
            ) -> beater_judge::JudgeProviderResult<beater_judge::GenerationResponse> {
                panic!("deterministic strategy must not call the generator");
            }
        }
        let evaluator = ScriptedEvaluator {
            optimize: vec![(0.5, 0.9); 6],
            test: vec![(0.5, 0.9); 6],
        };
        let outcome = run_optimization_round(
            round_config(OptimizerStrategy::ParamSearch),
            &PanicGenerator,
            ProviderCredentials::new("openai", "sk-test"),
            &evaluator,
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        // 3 temperatures x 2 top_p = 6 grid points, each gated.
        assert_eq!(outcome.evaluated.len(), 6);
        assert!(outcome.evaluated.iter().all(|e| e.accepted));
        let accepted = outcome
            .accepted
            .unwrap_or_else(|| panic!("expected an accepted grid point"));
        assert_eq!(accepted.proposed_by, OptimizerStrategy::ParamSearch);
    }

    /// A held-out Test split smaller than `min_sample_size` is a candidate the
    /// gate cannot judge — surfaced as a typed error, never a silent accept.
    #[tokio::test]
    async fn round_errors_when_test_split_underpowers_the_gate() {
        let generator = FakeRewriteGenerator {
            completion: "You are a meticulous assistant.".to_string(),
        };
        let evaluator = ScriptedEvaluator {
            optimize: vec![(0.5, 0.9); 6],
            test: vec![(0.5, 0.9); 2], // below min_sample_size = 6
        };
        let err = run_optimization_round(
            round_config(OptimizerStrategy::LlmRewrite),
            &generator,
            ProviderCredentials::new("openai", "sk-test"),
            &evaluator,
        )
        .await
        .err()
        .unwrap_or_else(|| panic!("expected a GateFailed error"));
        assert!(matches!(err, OptimizerError::GateFailed(_)), "{err:?}");
    }

    /// Build a minimal `CandidateEvaluation` that `Pass`ed the gate on its own,
    /// carrying a chosen raw p-value — the only field the family multiplicity
    /// correction reads.
    fn passing_eval(p_value: f64) -> CandidateEvaluation {
        CandidateEvaluation {
            candidate: CandidateChange {
                kind: ChangeKind::SystemPrompt,
                target: "prompt".to_string(),
                description: "desc".to_string(),
                rationale: "why".to_string(),
                proposed_by: OptimizerStrategy::ParamSearch,
            },
            gate: ExperimentComparison {
                sample_size: 6,
                baseline_mean: 0.5,
                candidate_mean: 0.6,
                delta: 0.1,
                ci_low: 0.0,
                ci_high: 0.2,
                p_value,
                decision: GateDecision::Pass,
                test: beater_eval::StatisticalTest::PairedT,
                adjusted_alpha: 0.05,
                mde: None,
                required_n: None,
            },
            overfit: GapAssessment {
                optimize_lift: 0.1,
                holdout_lift: 0.1,
                gap: 0.0,
                gap_ci_low: -0.1,
                gap_ci_high: 0.1,
                overfit: false,
            },
            accepted: true,
        }
    }

    /// Holm withdraws a family of three marginal passers: each clears α=0.05 alone
    /// (p ∈ {0.02, 0.03, 0.04}) but none survives the family-wise correction
    /// (Holm's tightest step is α/3 ≈ 0.0167), so all are downgraded and none is
    /// accepted. Proves the §10.3 #4 / #436 guard is load-bearing.
    #[test]
    fn holm_withdraws_a_family_of_marginal_passers() {
        let mut evals = vec![passing_eval(0.02), passing_eval(0.03), passing_eval(0.04)];
        apply_family_multiplicity(&mut evals, MultiplicityPolicy::Holm, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            evals
                .iter()
                .all(|e| e.gate.decision == GateDecision::Inconclusive && !e.accepted),
            "every marginal candidate must be withdrawn under Holm"
        );
    }

    /// Benjamini-Hochberg (FDR) is more lenient than Holm (FWER) on the *same*
    /// family: the three marginal passers all clear the BH threshold (adjusted
    /// p ≈ 0.04 ≤ 0.05), so they keep their `Pass`. Same input, different policy,
    /// different outcome — both branches are load-bearing.
    #[test]
    fn benjamini_hochberg_keeps_family_that_holm_withdraws() {
        let mut evals = vec![passing_eval(0.02), passing_eval(0.03), passing_eval(0.04)];
        apply_family_multiplicity(&mut evals, MultiplicityPolicy::BenjaminiHochberg, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            evals
                .iter()
                .all(|e| e.gate.decision == GateDecision::Pass && e.accepted),
            "BH controls FDR, not FWER, so this family survives"
        );
    }

    /// A strongly-significant family (p ≈ 0.001) survives Holm intact — the guard
    /// only removes noise, it never withdraws a genuine win.
    #[test]
    fn holm_keeps_a_strongly_significant_family() {
        let mut evals = vec![
            passing_eval(0.001),
            passing_eval(0.002),
            passing_eval(0.003),
        ];
        apply_family_multiplicity(&mut evals, MultiplicityPolicy::Holm, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(evals
            .iter()
            .all(|e| e.gate.decision == GateDecision::Pass && e.accepted));
    }

    /// The guard is a deliberate no-op for a single candidate (a family of one has
    /// no multiplicity) and when the policy is `None` — so the pre-declared
    /// `comparison_count` gate path is never double-corrected.
    #[test]
    fn family_guard_is_a_noop_for_single_candidate_or_none_policy() {
        // Single candidate with a terrible p-value: still untouched.
        let mut one = vec![passing_eval(0.9)];
        apply_family_multiplicity(&mut one, MultiplicityPolicy::Holm, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(one[0].gate.decision == GateDecision::Pass && one[0].accepted);

        // Policy None disables the correction even for a marginal family.
        let mut none = vec![passing_eval(0.04), passing_eval(0.04)];
        apply_family_multiplicity(&mut none, MultiplicityPolicy::None, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(none
            .iter()
            .all(|e| e.gate.decision == GateDecision::Pass && e.accepted));
    }

    /// The best-arm race withdraws acceptance from a candidate that a strictly
    /// better accepted candidate confidently dominates on the held-out split —
    /// proving §10.3 / #436 item 2 is load-bearing in the round.
    #[test]
    fn best_arm_race_drops_a_dominated_accepted_candidate() {
        let mut evals = vec![passing_eval(0.001), passing_eval(0.001)];
        // 40 held-out cases each: arm 0 ≈ 0.95, arm 1 ≈ 0.05 → disjoint intervals.
        let test_arms = vec![vec![0.95; 40], vec![0.05; 40]];
        apply_best_arm_race(&mut evals, &test_arms, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(evals[0].accepted, "the dominant arm survives");
        assert!(!evals[1].accepted, "the dominated arm loses acceptance");
    }

    /// Tied (statistically-indistinguishable) accepted candidates all survive the
    /// race, so proposal-order selection is preserved.
    #[test]
    fn best_arm_race_keeps_tied_candidates() {
        let mut evals = vec![passing_eval(0.001), passing_eval(0.001)];
        let test_arms = vec![vec![0.6; 30], vec![0.6; 30]];
        apply_best_arm_race(&mut evals, &test_arms, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(evals.iter().all(|e| e.accepted), "tied arms both survive");
    }

    #[test]
    fn param_search_emits_deterministic_grid() {
        let candidates = propose_with(OptimizerStrategy::ParamSearch, &proposal_context())
            .unwrap_or_else(|err| panic!("{err}"));
        // 3 temperatures x 2 top_p = 6 grid points.
        assert_eq!(candidates.len(), 6);
        assert!(candidates.iter().all(|c| c.kind == ChangeKind::ModelParams));
        assert_eq!(candidates[0].proposed_by, OptimizerStrategy::ParamSearch);
        // Deterministic ordering: first point is the lowest temperature/top_p.
        assert!(candidates[0].target.contains("temperature=0,top_p=0.9"));
        // A second call yields the identical grid.
        let again = propose_with(OptimizerStrategy::ParamSearch, &proposal_context())
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(candidates, again);
    }

    #[tokio::test]
    async fn experiment_scores_each_case_and_persists_gate_decision() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let dataset = DatasetId::new("dataset").unwrap_or_else(|err| panic!("{err}"));
        let snapshot = DatasetVersionSnapshot::try_new(
            tenant.clone(),
            project.clone(),
            dataset.clone(),
            DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            (0..5)
                .map(|index| fixture_case(&tenant, &project, &dataset, index))
                .collect(),
            Utc::now(),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let baseline_outputs = snapshot
            .cases
            .iter()
            .map(|case| CaseOutputOverride {
                case_id: case.case_id.clone(),
                output: json!("wrong"),
                trace: None,
            })
            .collect();
        let candidate_outputs = snapshot
            .cases
            .iter()
            .map(|case| CaseOutputOverride {
                case_id: case.case_id.clone(),
                output: json!("answer"),
                trace: None,
            })
            .collect();

        let report = run_deterministic_experiment(
            &snapshot,
            ExperimentRunSpec {
                baseline_release_id: AgentReleaseId::new("baseline")
                    .unwrap_or_else(|err| panic!("{err}")),
                candidate_release_id: AgentReleaseId::new("candidate")
                    .unwrap_or_else(|err| panic!("{err}")),
                evaluator: EvaluatorSpec {
                    id: "exact".to_string(),
                    lane: EvaluatorLane::DeterministicWasi,
                    kind: EvaluatorKind::ExactMatch,
                },
                evaluator_version_id: EvaluatorVersionId::new("exact-v1")
                    .unwrap_or_else(|err| panic!("{err}")),
                gate_policy: GatePolicy {
                    min_sample_size: 5,
                    max_regression: 0.05,
                    ..GatePolicy::default()
                },
                baseline_outputs,
                candidate_outputs,
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.case_scores.len(), 5);
        assert_eq!(report.comparison.baseline_mean, 0.0);
        assert_eq!(report.comparison.candidate_mean, 1.0);
        assert_eq!(report.decision, GateDecision::Pass);

        let store = SqliteExperimentStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let stored = store
            .write_run(report.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let loaded = store
            .get_run(tenant, project, stored.experiment_run_id.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(loaded.experiment_run_id, report.experiment_run_id);
        assert_eq!(loaded.case_scores.len(), 5);
    }

    #[tokio::test]
    async fn agent_harness_runs_releases_and_persists_traced_gate_report() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let dataset = DatasetId::new("dataset").unwrap_or_else(|err| panic!("{err}"));
        let snapshot = DatasetVersionSnapshot::try_new(
            tenant.clone(),
            project.clone(),
            dataset.clone(),
            DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            (0..5)
                .map(|index| fixture_case(&tenant, &project, &dataset, index))
                .collect(),
            Utc::now(),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let baseline = StaticAgentAdapter::new(json!("wrong"), "static-baseline");
        let candidate = ReferenceAgentAdapter::new("reference-candidate");

        let report = run_agent_experiment(
            &snapshot,
            AgentExperimentSpec {
                baseline_release_id: AgentReleaseId::new("baseline-release")
                    .unwrap_or_else(|err| panic!("{err}")),
                candidate_release_id: AgentReleaseId::new("candidate-release")
                    .unwrap_or_else(|err| panic!("{err}")),
                evaluator: EvaluatorSpec {
                    id: "exact".to_string(),
                    lane: EvaluatorLane::DeterministicWasi,
                    kind: EvaluatorKind::ExactMatch,
                },
                evaluator_version_id: EvaluatorVersionId::new("exact-v1")
                    .unwrap_or_else(|err| panic!("{err}")),
                gate_policy: GatePolicy {
                    min_sample_size: 5,
                    max_regression: 0.05,
                    ..GatePolicy::default()
                },
            },
            &baseline,
            &candidate,
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.decision, GateDecision::Pass);
        assert_eq!(report.case_scores.len(), 5);
        assert_eq!(report.case_scores[0].baseline_output, json!("wrong"));
        assert_eq!(report.case_scores[0].candidate_output, json!("answer"));
        assert_eq!(
            report.case_scores[0]
                .baseline_trace
                .as_ref()
                .unwrap_or_else(|| panic!("missing baseline trace"))["adapter"],
            json!("static-baseline")
        );
        assert_eq!(
            report.case_scores[0]
                .candidate_trace
                .as_ref()
                .unwrap_or_else(|| panic!("missing candidate trace"))["agent_release_id"],
            json!("candidate-release")
        );

        let store = SqliteExperimentStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let stored = store
            .write_run(report.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let loaded = store
            .get_run(tenant, project, stored.experiment_run_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            loaded.case_scores[0].candidate_trace,
            report.case_scores[0].candidate_trace
        );
    }

    #[tokio::test]
    async fn judge_experiment_scores_outputs_and_persists_audited_gate_report() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let dataset = DatasetId::new("dataset").unwrap_or_else(|err| panic!("{err}"));
        let snapshot = DatasetVersionSnapshot::try_new(
            tenant.clone(),
            project.clone(),
            dataset.clone(),
            DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            (0..5)
                .map(|index| fixture_case(&tenant, &project, &dataset, index))
                .collect(),
            Utc::now(),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let baseline_outputs = snapshot
            .cases
            .iter()
            .map(|case| CaseOutputOverride {
                case_id: case.case_id.clone(),
                output: json!("wrong"),
                trace: None,
            })
            .collect();
        let candidate_outputs = snapshot
            .cases
            .iter()
            .map(|case| CaseOutputOverride {
                case_id: case.case_id.clone(),
                output: json!("answer"),
                trace: None,
            })
            .collect();

        let report = run_judge_experiment(
            &snapshot,
            JudgeExperimentRunSpec {
                experiment: ExperimentRunSpec {
                    baseline_release_id: AgentReleaseId::new("baseline")
                        .unwrap_or_else(|err| panic!("{err}")),
                    candidate_release_id: AgentReleaseId::new("candidate")
                        .unwrap_or_else(|err| panic!("{err}")),
                    evaluator: EvaluatorSpec {
                        id: "judge-correctness".to_string(),
                        lane: EvaluatorLane::JudgeBroker,
                        kind: EvaluatorKind::LlmJudge {
                            rubric: "correctness".to_string(),
                            model: "judge-model".to_string(),
                        },
                    },
                    evaluator_version_id: EvaluatorVersionId::new("judge-v1")
                        .unwrap_or_else(|err| panic!("{err}")),
                    gate_policy: GatePolicy {
                        min_sample_size: 5,
                        max_regression: 0.05,
                        ..GatePolicy::default()
                    },
                    baseline_outputs,
                    candidate_outputs,
                },
                provider_secret_id: ProviderSecretId::new("judge-secret")
                    .unwrap_or_else(|err| panic!("{err}")),
            },
            &ReferenceScoringJudgeBroker,
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.decision, GateDecision::Pass);
        assert_eq!(report.comparison.baseline_mean, 0.0);
        assert_eq!(report.comparison.candidate_mean, 1.0);
        assert_eq!(
            report.case_scores[0].baseline_cost,
            Some(Money::usd_micros(13))
        );
        assert_eq!(
            report.case_scores[0].candidate_cost,
            Some(Money::usd_micros(13))
        );
        assert_eq!(report.case_scores[0].baseline_cached, Some(false));
        assert_eq!(report.case_scores[0].candidate_cached, Some(false));
        assert!(report.case_scores[0].baseline_judge_call_id.is_some());
        assert!(report.case_scores[0].candidate_judge_call_id.is_some());

        let store = SqliteExperimentStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let stored = store
            .write_run(report.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let loaded = store
            .get_run(tenant, project, stored.experiment_run_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(loaded.case_scores[0].candidate_score, 1.0);
        assert_eq!(
            loaded.case_scores[0].candidate_cost,
            Some(Money::usd_micros(13))
        );
    }

    #[tokio::test]
    async fn experiment_store_selects_latest_run_by_gate_selectors() -> anyhow::Result<()> {
        let store = SqliteExperimentStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let dataset = DatasetId::new("dataset")?;
        let other_dataset = DatasetId::new("other-dataset")?;
        let exact = EvaluatorVersionId::new("exact-v1")?;
        let judge = EvaluatorVersionId::new("judge-v1")?;
        let older = report_with_created_at(
            tenant.clone(),
            project.clone(),
            "older-exact",
            dataset.clone(),
            exact.clone(),
            "2026-06-19T10:00:00Z",
        )?;
        let newest_non_matching = report_with_created_at(
            tenant.clone(),
            project.clone(),
            "newest-other-dataset",
            other_dataset,
            exact.clone(),
            "2026-06-19T12:00:00Z",
        )?;
        let newest_matching = report_with_created_at(
            tenant.clone(),
            project.clone(),
            "newest-judge",
            dataset.clone(),
            judge.clone(),
            "2026-06-19T11:00:00Z",
        )?;
        store.write_run(older).await?;
        store.write_run(newest_non_matching).await?;
        store.write_run(newest_matching.clone()).await?;

        let latest = store
            .latest_run(
                tenant.clone(),
                project.clone(),
                Some(dataset.clone()),
                Some(judge),
            )
            .await?
            .ok_or_else(|| anyhow!("expected latest judge report"))?;
        assert_eq!(latest.experiment_run_id, newest_matching.experiment_run_id);

        let latest_dataset_any_evaluator = store
            .latest_run(tenant, project, Some(dataset), None)
            .await?
            .ok_or_else(|| anyhow!("expected latest dataset report"))?;
        assert_eq!(
            latest_dataset_any_evaluator.experiment_run_id,
            newest_matching.experiment_run_id
        );
        Ok(())
    }

    #[tokio::test]
    async fn experiment_store_scopes_get_run_by_tenant_and_project() -> anyhow::Result<()> {
        let store = SqliteExperimentStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let other_tenant = TenantId::new("other-tenant")?;
        let other_project = ProjectId::new("other-project")?;
        let dataset = DatasetId::new("dataset")?;
        let other_tenant_dataset = DatasetId::new("other-tenant-dataset")?;
        let other_project_dataset = DatasetId::new("other-project-dataset")?;
        let evaluator = EvaluatorVersionId::new("exact-v1")?;
        let run_id = ExperimentRunId::new("shared-run")?;

        let target = report_with_created_at(
            tenant.clone(),
            project.clone(),
            run_id.as_str(),
            dataset.clone(),
            evaluator.clone(),
            "2026-06-19T10:00:00Z",
        )?;
        let tenant_neighbor = report_with_created_at(
            other_tenant.clone(),
            project.clone(),
            run_id.as_str(),
            other_tenant_dataset.clone(),
            evaluator.clone(),
            "2026-06-19T11:00:00Z",
        )?;
        let project_neighbor = report_with_created_at(
            tenant.clone(),
            other_project.clone(),
            run_id.as_str(),
            other_project_dataset.clone(),
            evaluator,
            "2026-06-19T12:00:00Z",
        )?;
        store.write_run(target).await?;
        store.write_run(tenant_neighbor).await?;
        store.write_run(project_neighbor).await?;

        let loaded = store
            .get_run(tenant.clone(), project.clone(), run_id.clone())
            .await?;
        assert_eq!(loaded.tenant_id, tenant.clone());
        assert_eq!(loaded.project_id, project.clone());
        assert_eq!(loaded.dataset_id, dataset);

        let loaded_tenant_neighbor = store
            .get_run(other_tenant.clone(), project.clone(), run_id.clone())
            .await?;
        assert_eq!(loaded_tenant_neighbor.tenant_id, other_tenant);
        assert_eq!(loaded_tenant_neighbor.project_id, project.clone());
        assert_eq!(loaded_tenant_neighbor.dataset_id, other_tenant_dataset);

        let loaded_project_neighbor = store
            .get_run(tenant.clone(), other_project.clone(), run_id.clone())
            .await?;
        assert_eq!(loaded_project_neighbor.tenant_id, tenant.clone());
        assert_eq!(loaded_project_neighbor.project_id, other_project);
        assert_eq!(loaded_project_neighbor.dataset_id, other_project_dataset);

        let result = store
            .get_run(tenant, ProjectId::new("missing-project")?, run_id)
            .await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));
        Ok(())
    }

    #[tokio::test]
    async fn experiment_store_scopes_latest_run_by_tenant_and_project() -> anyhow::Result<()> {
        let store = SqliteExperimentStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let other_tenant = TenantId::new("other-tenant")?;
        let other_project = ProjectId::new("other-project")?;
        let dataset = DatasetId::new("dataset")?;
        let evaluator = EvaluatorVersionId::new("exact-v1")?;

        let target = report_with_created_at(
            tenant.clone(),
            project.clone(),
            "target-run",
            dataset.clone(),
            evaluator.clone(),
            "2026-06-19T10:00:00Z",
        )?;
        let tenant_neighbor = report_with_created_at(
            other_tenant.clone(),
            project.clone(),
            "other-tenant-run",
            dataset.clone(),
            evaluator.clone(),
            "2026-06-19T11:00:00Z",
        )?;
        let project_neighbor = report_with_created_at(
            tenant.clone(),
            other_project.clone(),
            "other-project-run",
            dataset.clone(),
            evaluator.clone(),
            "2026-06-19T12:00:00Z",
        )?;
        store.write_run(target.clone()).await?;
        store.write_run(tenant_neighbor.clone()).await?;
        store.write_run(project_neighbor.clone()).await?;

        let latest = store
            .latest_run(
                tenant.clone(),
                project.clone(),
                Some(dataset.clone()),
                Some(evaluator.clone()),
            )
            .await?
            .ok_or_else(|| anyhow!("expected scoped latest report"))?;
        assert_eq!(latest.experiment_run_id, target.experiment_run_id);

        let latest_tenant_neighbor = store
            .latest_run(
                other_tenant,
                project.clone(),
                Some(dataset.clone()),
                Some(evaluator.clone()),
            )
            .await?
            .ok_or_else(|| anyhow!("expected tenant-neighbor latest report"))?;
        assert_eq!(
            latest_tenant_neighbor.experiment_run_id,
            tenant_neighbor.experiment_run_id
        );

        let latest_project_neighbor = store
            .latest_run(
                tenant.clone(),
                other_project,
                Some(dataset.clone()),
                Some(evaluator.clone()),
            )
            .await?
            .ok_or_else(|| anyhow!("expected project-neighbor latest report"))?;
        assert_eq!(
            latest_project_neighbor.experiment_run_id,
            project_neighbor.experiment_run_id
        );

        let missing_scope = store
            .latest_run(
                TenantId::new("missing-tenant")?,
                project,
                Some(dataset),
                Some(evaluator),
            )
            .await?;
        assert!(missing_scope.is_none());
        Ok(())
    }

    #[test]
    fn experiment_refuses_underpowered_gate_policy() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let dataset = DatasetId::new("dataset").unwrap_or_else(|err| panic!("{err}"));
        let snapshot = DatasetVersionSnapshot::try_new(
            tenant.clone(),
            project.clone(),
            dataset.clone(),
            DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            vec![fixture_case(&tenant, &project, &dataset, 0)],
            Utc::now(),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let error = run_deterministic_experiment(
            &snapshot,
            ExperimentRunSpec {
                baseline_release_id: AgentReleaseId::new("baseline")
                    .unwrap_or_else(|err| panic!("{err}")),
                candidate_release_id: AgentReleaseId::new("candidate")
                    .unwrap_or_else(|err| panic!("{err}")),
                evaluator: EvaluatorSpec {
                    id: "exact".to_string(),
                    lane: EvaluatorLane::DeterministicWasi,
                    kind: EvaluatorKind::ExactMatch,
                },
                evaluator_version_id: EvaluatorVersionId::new("exact-v1")
                    .unwrap_or_else(|err| panic!("{err}")),
                gate_policy: GatePolicy {
                    min_sample_size: 2,
                    ..GatePolicy::default()
                },
                baseline_outputs: Vec::new(),
                candidate_outputs: Vec::new(),
            },
        )
        .err()
        .unwrap_or_else(|| panic!("expected underpowered experiment error"));
        let underpowered = error
            .chain()
            .find_map(|cause| cause.downcast_ref::<EvalError>());
        assert!(matches!(
            underpowered,
            Some(EvalError::Underpowered {
                sample_size: 1,
                min_sample_size: 2
            })
        ));
    }

    fn fixture_case(
        tenant: &TenantId,
        project: &ProjectId,
        dataset: &DatasetId,
        index: usize,
    ) -> DatasetCase {
        DatasetCase {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            case_id: DatasetCaseId::new(format!("case-{index}"))
                .unwrap_or_else(|err| panic!("{err}")),
            source_trace_id: TraceId::new(format!("trace-{index}"))
                .unwrap_or_else(|err| panic!("{err}")),
            source_span_id: SpanId::new(format!("span-{index}"))
                .unwrap_or_else(|err| panic!("{err}")),
            source_environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            input: json!("question"),
            output: json!("answer"),
            reference: Some(json!("answer")),
            trace: json!({}),
            normalizer_version: "test".to_string(),
            trace_schema_version: 1,
            input_artifact_hashes: Vec::new(),
            created_at: Utc::now(),
        }
    }

    fn report_with_created_at(
        tenant_id: TenantId,
        project_id: ProjectId,
        experiment_run_id: &str,
        dataset_id: DatasetId,
        evaluator_version_id: EvaluatorVersionId,
        created_at: &str,
    ) -> anyhow::Result<ExperimentRunReport> {
        let created_at = chrono::DateTime::parse_from_rfc3339(created_at)?.with_timezone(&Utc);
        Ok(ExperimentRunReport {
            experiment_run_id: ExperimentRunId::new(experiment_run_id)?,
            tenant_id,
            project_id,
            dataset_id,
            dataset_version_id: DatasetVersionId::new("v1")?,
            baseline_release_id: AgentReleaseId::new("baseline")?,
            candidate_release_id: AgentReleaseId::new("candidate")?,
            evaluator_version_id,
            case_scores: Vec::new(),
            comparison: ExperimentComparison {
                sample_size: 1,
                baseline_mean: 1.0,
                candidate_mean: 1.0,
                delta: 0.0,
                ci_low: 0.0,
                ci_high: 0.0,
                p_value: 1.0,
                decision: GateDecision::Pass,
                test: StatisticalTest::PairedT,
                adjusted_alpha: 0.05,
                mde: None,
                required_n: None,
            },
            decision: GateDecision::Pass,
            gate_policy: GatePolicy {
                min_sample_size: 1,
                ..GatePolicy::default()
            },
            created_at,
        })
    }

    struct ReferenceScoringJudgeBroker;

    #[async_trait]
    impl JudgeBroker for ReferenceScoringJudgeBroker {
        async fn evaluate(
            &self,
            request: JudgeBrokerRequest,
        ) -> Result<JudgeBrokerOutcome, JudgeBrokerError> {
            let score = if request.case.reference.as_ref() == Some(&request.case.output) {
                1.0
            } else {
                0.0
            };
            Ok(JudgeBrokerOutcome {
                result: ScoreResult {
                    score,
                    label: Some(if score == 1.0 { "pass" } else { "fail" }.to_string()),
                    evidence: json!({ "score_source": "fixed-reference-judge" }),
                },
                audit: JudgeAuditRecord {
                    judge_call_id: JudgeCallId::new(Uuid::new_v4().to_string())
                        .unwrap_or_else(|err| panic!("{err}")),
                    tenant_id: request.tenant_id,
                    project_id: request.project_id,
                    evaluator_id: request.evaluator.id,
                    provider: "openai".to_string(),
                    provider_secret_id: request.provider_secret_id,
                    model: "judge-model".to_string(),
                    request_hash: Sha256Hash::new("11".repeat(32))
                        .unwrap_or_else(|err| panic!("{err}")),
                    response_hash: Sha256Hash::new("22".repeat(32))
                        .unwrap_or_else(|err| panic!("{err}")),
                    score,
                    provider_cost: Money::usd_micros(13),
                    charged_cost: Money::usd_micros(13),
                    cached: false,
                    created_at: Utc::now(),
                },
                remaining_budget: Money::usd_micros(100),
            })
        }

        fn remaining_budget(&self) -> Result<Money, JudgeBrokerError> {
            Ok(Money::usd_micros(100))
        }
    }
}
