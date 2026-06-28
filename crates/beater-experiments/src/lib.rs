use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    AgentReleaseId, DatasetCaseId, DatasetId, DatasetVersionId, EvaluatorVersionId,
    ExperimentRunId, JudgeCallId, Money, ProjectId, ProviderSecretId, TenantId, Timestamp,
};
use beater_datasets::DatasetVersionSnapshot;
use beater_eval::{
    compare_paired_scores, evaluate_deterministic, EvaluationCase, EvaluatorSpec,
    ExperimentComparison, GateDecision, GatePolicy, ScoreResult,
};
use beater_judge::{JudgeBroker, JudgeBrokerOutcome, JudgeBrokerRequest};
use beater_schema::EvaluatorLane;
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
    let comparison = compare_paired_scores(&baseline_scores, &candidate_scores, &gate_policy)
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
    let comparison = compare_paired_scores(&baseline_scores, &candidate_scores, &gate_policy)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizerStrategy {
    /// Reflective single-shot LLM rewrite of a prompt lever of π (§6.1) — the
    /// reflective-proposal baseline of §21.3. Implemented (minimally) below.
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

/// Read-only context handed to a [`ProposalStrategy`].
///
/// The strategy reflects on the optimization goal and the indexed agent surface
/// (§21.1 `index_agent`) to emit candidates; it has no ability to accept them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalContext {
    /// The improvement goal in natural language (§21.3 "goal + params").
    pub goal: String,
    /// The current prompt (or other lever text) the optimizer may rewrite.
    pub current_prompt: String,
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
    /// The proposal context was insufficient to produce a candidate.
    #[error("invalid proposal context: {0}")]
    InvalidContext(String),
}

/// A strategy that *proposes* candidate changes for the held-out gate to judge.
///
/// Implementations never accept a change; see [`OptimizerStrategy`] for the
/// gating invariant.
pub trait ProposalStrategy {
    /// Emit zero or more candidate changes from the given context.
    fn propose(&self, ctx: &ProposalContext) -> Result<Vec<CandidateChange>, OptimizerError>;
}

/// Reflective single-shot LLM rewrite of a prompt lever (§21.3).
///
/// Minimal implementation: emits a single placeholder candidate derived from the
/// context. A future revision will delegate to the judge/LLM broker; the
/// candidate it returns still only earns acceptance via the held-out gate.
#[derive(Clone, Copy, Debug, Default)]
pub struct LlmRewrite;

impl ProposalStrategy for LlmRewrite {
    fn propose(&self, ctx: &ProposalContext) -> Result<Vec<CandidateChange>, OptimizerError> {
        if ctx.goal.trim().is_empty() {
            return Err(OptimizerError::InvalidContext(
                "goal must not be empty".to_string(),
            ));
        }
        Ok(vec![CandidateChange {
            kind: ChangeKind::SystemPrompt,
            target: "system_prompt".to_string(),
            description: format!(
                "Rewrite the system prompt to better satisfy goal: {}",
                ctx.goal
            ),
            rationale:
                "reflective LLM rewrite placeholder candidate (§21.3); must clear the held-out \
                 Test gate + beater-stats CI before acceptance"
                    .to_string(),
            proposed_by: OptimizerStrategy::LlmRewrite,
        }])
    }
}

/// Dispatch to the named [`OptimizerStrategy`], returning its proposed candidates.
///
/// Only [`OptimizerStrategy::LlmRewrite`] is implemented; the other variants
/// return a typed [`OptimizerError::NotYetImplemented`]. Whatever a strategy
/// proposes is *only* a proposal — acceptance still requires clearing the
/// held-out Test gate (§21.3) and the planned §21.4 guardrail.
pub fn propose_with(
    strategy: OptimizerStrategy,
    ctx: &ProposalContext,
) -> Result<Vec<CandidateChange>, OptimizerError> {
    match strategy {
        OptimizerStrategy::LlmRewrite => LlmRewrite.propose(ctx),
        OptimizerStrategy::FewShotBayesian
        | OptimizerStrategy::Mipro
        | OptimizerStrategy::Evolutionary
        | OptimizerStrategy::Gepa
        | OptimizerStrategy::ParamSearch => Err(OptimizerError::NotYetImplemented(strategy)),
    }
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
        ProposalContext {
            goal: "reduce hallucinations on factual lookups".to_string(),
            current_prompt: "You are a helpful assistant.".to_string(),
        }
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
        let ctx = ProposalContext {
            goal: "   ".to_string(),
            current_prompt: "x".to_string(),
        };
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
            OptimizerStrategy::ParamSearch,
        ] {
            let err = propose_with(strategy, &ctx)
                .err()
                .unwrap_or_else(|| panic!("expected NotYetImplemented for {strategy:?}"));
            assert_eq!(err, OptimizerError::NotYetImplemented(strategy));
        }
    }

    #[tokio::test]
    async fn experiment_scores_each_case_and_persists_gate_decision() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let dataset = DatasetId::new("dataset").unwrap_or_else(|err| panic!("{err}"));
        let snapshot = DatasetVersionSnapshot {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            version_id: DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            cases: (0..5)
                .map(|index| fixture_case(&tenant, &project, &dataset, index))
                .collect(),
            created_at: Utc::now(),
        };
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
        let snapshot = DatasetVersionSnapshot {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            version_id: DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            cases: (0..5)
                .map(|index| fixture_case(&tenant, &project, &dataset, index))
                .collect(),
            created_at: Utc::now(),
        };
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
        let snapshot = DatasetVersionSnapshot {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            version_id: DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            cases: (0..5)
                .map(|index| fixture_case(&tenant, &project, &dataset, index))
                .collect(),
            created_at: Utc::now(),
        };
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

    #[test]
    fn experiment_refuses_underpowered_gate_policy() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let dataset = DatasetId::new("dataset").unwrap_or_else(|err| panic!("{err}"));
        let snapshot = DatasetVersionSnapshot {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            version_id: DatasetVersionId::new("v1").unwrap_or_else(|err| panic!("{err}")),
            cases: vec![fixture_case(&tenant, &project, &dataset, 0)],
            created_at: Utc::now(),
        };
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
