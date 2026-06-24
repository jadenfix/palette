use beater_core::{
    AgentReleaseId, ApiKeyId, ArtifactId, DatasetCaseId, DatasetVersionId, EnvironmentId,
    EvalResultId, EvaluatorVersionId, IdempotencyKey, Money, ProjectId, PromptVersionId,
    Sha256Hash, SpanId, TenantId, TenantScope, Timestamp, TokenCounts, TraceId,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const RAW_SCHEMA_VERSION: u32 = 1;
pub const CANONICAL_SCHEMA_VERSION: u32 = 1;

pub type CanonicalAttrs = BTreeMap<String, Value>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDialect {
    Native,
    Otlp,
    OpenInference,
    OpenTelemetryGenAi,
    VercelAiSdk,
    OpenLlmetry,
    PhoenixImport,
    LangSmithImport,
    LangfuseImport,
}

impl SourceDialect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Otlp => "otlp",
            Self::OpenInference => "open_inference",
            Self::OpenTelemetryGenAi => "open_telemetry_gen_ai",
            Self::VercelAiSdk => "vercel_ai_sdk",
            Self::OpenLlmetry => "open_llmetry",
            Self::PhoenixImport => "phoenix_import",
            Self::LangSmithImport => "langsmith_import",
            Self::LangfuseImport => "langfuse_import",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentSpanKind {
    AgentRun,
    AgentTurn,
    AgentPlan,
    AgentStep,
    LlmCall,
    ToolCall,
    McpRequest,
    RetrievalQuery,
    MemoryRead,
    MemoryWrite,
    GuardrailCheck,
    HumanReview,
    EvaluatorRun,
    ReplayRun,
}

impl AgentSpanKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentRun => "agent.run",
            Self::AgentTurn => "agent.turn",
            Self::AgentPlan => "agent.plan",
            Self::AgentStep => "agent.step",
            Self::LlmCall => "llm.call",
            Self::ToolCall => "tool.call",
            Self::McpRequest => "mcp.request",
            Self::RetrievalQuery => "retrieval.query",
            Self::MemoryRead => "memory.read",
            Self::MemoryWrite => "memory.write",
            Self::GuardrailCheck => "guardrail.check",
            Self::HumanReview => "human.review",
            Self::EvaluatorRun => "evaluator.run",
            Self::ReplayRun => "replay.run",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "agent.run" | "agent_run" => Some(Self::AgentRun),
            "agent.turn" | "agent_turn" => Some(Self::AgentTurn),
            "agent.plan" | "agent_plan" => Some(Self::AgentPlan),
            "agent.step" | "agent_step" => Some(Self::AgentStep),
            "llm.call" | "llm_call" => Some(Self::LlmCall),
            "tool.call" | "tool_call" => Some(Self::ToolCall),
            "mcp.request" | "mcp_request" => Some(Self::McpRequest),
            "retrieval.query" | "retrieval_query" => Some(Self::RetrievalQuery),
            "memory.read" | "memory_read" => Some(Self::MemoryRead),
            "memory.write" | "memory_write" => Some(Self::MemoryWrite),
            "guardrail.check" | "guardrail_check" => Some(Self::GuardrailCheck),
            "human.review" | "human_review" => Some(Self::HumanReview),
            "evaluator.run" | "evaluator_run" => Some(Self::EvaluatorRun),
            "replay.run" | "replay_run" => Some(Self::ReplayRun),
            _ => None,
        }
    }
}

impl Serialize for AgentSpanKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AgentSpanKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| {
            serde::de::Error::custom(format!("unsupported agent span kind: {value}"))
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

impl SpanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Unset => "unset",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            "unset" => Some(Self::Unset),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub artifact_id: ArtifactId,
    pub uri: String,
    pub sha256: Sha256Hash,
    pub size_bytes: u64,
    pub mime_type: String,
    pub redaction_class: RedactionClass,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionClass {
    Public,
    Internal,
    Sensitive,
    Secret,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthContext {
    pub api_key_id: Option<ApiKeyId>,
    pub scopes: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawEnvelope {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub source: SourceDialect,
    pub source_schema_url: Option<String>,
    pub source_schema_version: Option<String>,
    pub received_at: Timestamp,
    pub idempotency_key: IdempotencyKey,
    pub payload_hash: Sha256Hash,
    pub body_ref: ArtifactRef,
    pub auth_context: AuthContext,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanonicalSpan {
    pub schema_version: u32,
    pub normalizer_version: String,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub seq: u64,
    pub kind: AgentSpanKind,
    pub name: String,
    pub status: SpanStatus,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub model: Option<ModelRef>,
    pub cost: Option<Money>,
    pub tokens: Option<TokenCounts>,
    pub input_ref: Option<ArtifactRef>,
    pub output_ref: Option<ArtifactRef>,
    pub attributes: CanonicalAttrs,
    pub unmapped_attrs: Value,
    pub raw_ref: ArtifactRef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanonicalTraceBatch {
    pub raw_envelopes: Vec<RawEnvelope>,
    pub spans: Vec<CanonicalSpan>,
}

impl CanonicalTraceBatch {
    pub fn one(raw_envelope: RawEnvelope, span: CanonicalSpan) -> Self {
        Self {
            raw_envelopes: vec![raw_envelope],
            spans: vec![span],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteAck {
    pub accepted_raw: usize,
    pub accepted_spans: usize,
    pub duplicate_raw: usize,
    pub duplicate_spans: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceView {
    pub tenant_id: TenantId,
    pub trace_id: TraceId,
    pub spans: Vec<CanonicalSpan>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub first_span_name: String,
    pub span_count: usize,
    pub status: SpanStatus,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub duration_ms: Option<i64>,
    pub total_cost: Option<Money>,
    pub models: Vec<ModelRef>,
    pub release_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpanSummary {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub kind: AgentSpanKind,
    pub name: String,
    pub status: SpanStatus,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub model: Option<ModelRef>,
    pub cost: Option<Money>,
    pub release_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunFilter {
    pub project_id: Option<ProjectId>,
    pub environment_id: Option<EnvironmentId>,
    pub trace_id: Option<TraceId>,
    pub status: Option<SpanStatus>,
    pub kind: Option<AgentSpanKind>,
    pub started_after: Option<Timestamp>,
    pub started_before: Option<Timestamp>,
    pub model: Option<String>,
    pub release: Option<String>,
    pub min_cost_micros: Option<i64>,
    pub max_cost_micros: Option<i64>,
    pub min_latency_ms: Option<i64>,
    pub max_latency_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanFilter {
    pub project_id: Option<ProjectId>,
    pub environment_id: Option<EnvironmentId>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub kind: Option<AgentSpanKind>,
    pub status: Option<SpanStatus>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluatorLane {
    DeterministicWasi,
    JudgeBroker,
    Human,
    Hybrid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvalReproducibility {
    pub dataset_version_id: DatasetVersionId,
    pub dataset_case_id: DatasetCaseId,
    pub agent_release_id: AgentReleaseId,
    pub prompt_version_id: Option<PromptVersionId>,
    pub evaluator_version_id: EvaluatorVersionId,
    pub code_hash: Option<Sha256Hash>,
    pub wasm_hash: Option<Sha256Hash>,
    pub wasi_abi_version: Option<String>,
    pub judge_model_id: Option<String>,
    pub judge_provider: Option<String>,
    pub judge_parameters: Value,
    pub judge_seed: Option<u64>,
    pub judge_rubric_version: Option<String>,
    pub normalizer_version: String,
    pub trace_schema_version: u32,
    pub input_artifact_hashes: Vec<Sha256Hash>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvalResult {
    pub eval_result_id: EvalResultId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub span_id: Option<SpanId>,
    pub score: f64,
    pub label: Option<String>,
    pub evidence: Value,
    pub reproducibility: EvalReproducibility,
    pub cost: Option<Money>,
    pub tokens: Option<TokenCounts>,
    pub created_at: Timestamp,
    pub non_reproducible_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceCompletionState {
    Open,
    RootEnded,
    IdleComplete,
    LateWindowClosed,
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCassette {
    pub tenant_id: TenantId,
    pub trace_id: TraceId,
    pub provider_events: usize,
    pub tool_events: usize,
    pub memory_events: usize,
    pub retrieval_events: usize,
    pub clock_events: usize,
    pub random_events: usize,
    pub missing_required_kinds: Vec<String>,
}

pub fn make_idempotency_key(
    scope: &TenantScope,
    trace_id: &TraceId,
    span_id: &SpanId,
    seq: u64,
    payload_hash: &Sha256Hash,
) -> Result<IdempotencyKey, beater_core::IdError> {
    IdempotencyKey::new(format!(
        "{}:{}:{}:{}:{}:{}",
        scope.tenant_id.as_str(),
        scope.project_id.as_str(),
        trace_id.as_str(),
        span_id.as_str(),
        seq,
        payload_hash.as_str()
    ))
}

pub fn span_matches(span: &CanonicalSpan, filter: &SpanFilter) -> bool {
    if let Some(project_id) = &filter.project_id {
        if span.project_id != *project_id {
            return false;
        }
    }
    if let Some(environment_id) = &filter.environment_id {
        if span.environment_id != *environment_id {
            return false;
        }
    }
    if let Some(trace_id) = &filter.trace_id {
        if span.trace_id != *trace_id {
            return false;
        }
    }
    if let Some(span_id) = &filter.span_id {
        if span.span_id != *span_id {
            return false;
        }
    }
    if let Some(kind) = &filter.kind {
        if span.kind != *kind {
            return false;
        }
    }
    if let Some(status) = &filter.status {
        if span.status != *status {
            return false;
        }
    }
    true
}

pub fn span_summary(span: CanonicalSpan) -> SpanSummary {
    let release_id = span_release_id(&span);
    SpanSummary {
        tenant_id: span.tenant_id,
        project_id: span.project_id,
        trace_id: span.trace_id,
        span_id: span.span_id,
        kind: span.kind,
        name: span.name,
        status: span.status,
        started_at: span.start_time,
        ended_at: span.end_time,
        model: span.model,
        cost: span.cost,
        release_id,
    }
}

pub fn span_release_id(span: &CanonicalSpan) -> Option<String> {
    [
        "beater.release_id",
        "agent.release_id",
        "deployment.release_id",
        "release.id",
        "release_id",
    ]
    .iter()
    .find_map(|key| {
        span.attributes
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string)
    })
}

pub fn roll_up_runs(tenant: TenantId, spans: Vec<SpanSummary>) -> Vec<RunSummary> {
    let mut runs = Vec::<RunSummary>::new();
    for span in &spans {
        if let Some(run) = runs
            .iter_mut()
            .find(|run| run.project_id == span.project_id && run.trace_id == span.trace_id)
        {
            run.span_count += 1;
            if span.started_at < run.started_at {
                run.started_at = span.started_at;
                run.first_span_name = span.name.clone();
            }
            run.status = aggregate_run_status(&run.status, &span.status);
            run.total_cost = merge_cost(run.total_cost.clone(), span.cost.clone());
            push_model(&mut run.models, span.model.clone());
            push_release_id(&mut run.release_ids, span.release_id.clone());
            run.ended_at = match (run.ended_at, span.ended_at) {
                (Some(left), Some(right)) => Some(left.max(right)),
                (None, Some(right)) => Some(right),
                (left, None) => left,
            };
            run.duration_ms = run_duration_ms(run.started_at, run.ended_at);
        } else {
            let ended_at = span.ended_at;
            runs.push(RunSummary {
                tenant_id: tenant.clone(),
                project_id: span.project_id.clone(),
                trace_id: span.trace_id.clone(),
                first_span_name: span.name.clone(),
                span_count: 1,
                status: span.status.clone(),
                started_at: span.started_at,
                ended_at,
                duration_ms: run_duration_ms(span.started_at, ended_at),
                total_cost: span.cost.clone(),
                models: span.model.clone().into_iter().collect(),
                release_ids: span.release_id.clone().into_iter().collect(),
            });
        }
    }

    runs.sort_by(|left, right| right.started_at.cmp(&left.started_at));
    runs
}

pub fn filter_run_summaries(
    runs: Vec<RunSummary>,
    spans: &[SpanSummary],
    filter: &RunFilter,
) -> Vec<RunSummary> {
    let kind_trace_ids = filter.kind.as_ref().map(|kind| {
        spans
            .iter()
            .filter(|span| &span.kind == kind)
            .map(|span| (span.project_id.clone(), span.trace_id.clone()))
            .collect::<BTreeSet<_>>()
    });
    runs.into_iter()
        .filter(|run| match &filter.trace_id {
            Some(trace_id) => &run.trace_id == trace_id,
            None => true,
        })
        .filter(|run| match &filter.status {
            Some(status) => &run.status == status,
            None => true,
        })
        .filter(|run| match &filter.started_after {
            Some(started_after) => run.started_at >= *started_after,
            None => true,
        })
        .filter(|run| match &filter.started_before {
            Some(started_before) => run.started_at <= *started_before,
            None => true,
        })
        .filter(|run| match filter.min_latency_ms {
            Some(min_latency_ms) => run
                .duration_ms
                .is_some_and(|duration_ms| duration_ms >= min_latency_ms),
            None => true,
        })
        .filter(|run| match filter.max_latency_ms {
            Some(max_latency_ms) => run
                .duration_ms
                .is_some_and(|duration_ms| duration_ms <= max_latency_ms),
            None => true,
        })
        .filter(|run| match filter.min_cost_micros {
            Some(min_cost_micros) => run
                .total_cost
                .as_ref()
                .is_some_and(|cost| cost.amount_micros >= min_cost_micros),
            None => true,
        })
        .filter(|run| match filter.max_cost_micros {
            Some(max_cost_micros) => run
                .total_cost
                .as_ref()
                .is_some_and(|cost| cost.amount_micros <= max_cost_micros),
            None => true,
        })
        .filter(|run| match &filter.model {
            Some(model) => run_model_matches(run, model),
            None => true,
        })
        .filter(|run| match &filter.release {
            Some(release) => run.release_ids.iter().any(|candidate| candidate == release),
            None => true,
        })
        .filter(|run| match &kind_trace_ids {
            Some(trace_ids) => trace_ids.contains(&(run.project_id.clone(), run.trace_id.clone())),
            None => true,
        })
        .collect()
}

fn aggregate_run_status(current: &SpanStatus, next: &SpanStatus) -> SpanStatus {
    if current == &SpanStatus::Error || next == &SpanStatus::Error {
        return SpanStatus::Error;
    }
    if current == &SpanStatus::Ok || next == &SpanStatus::Ok {
        return SpanStatus::Ok;
    }
    SpanStatus::Unset
}

fn run_duration_ms(started_at: Timestamp, ended_at: Option<Timestamp>) -> Option<i64> {
    ended_at.map(|ended_at| (ended_at - started_at).num_milliseconds().max(0))
}

fn merge_cost(current: Option<Money>, next: Option<Money>) -> Option<Money> {
    match (current, next) {
        (Some(current), Some(next)) => current.try_add(&next).ok().or(Some(current)),
        (Some(current), None) => Some(current),
        (None, Some(next)) => Some(next),
        (None, None) => None,
    }
}

fn push_model(models: &mut Vec<ModelRef>, model: Option<ModelRef>) {
    let Some(model) = model else {
        return;
    };
    if !models
        .iter()
        .any(|existing| existing.provider == model.provider && existing.name == model.name)
    {
        models.push(model);
    }
}

fn push_release_id(release_ids: &mut Vec<String>, release_id: Option<String>) {
    let Some(release_id) = release_id else {
        return;
    };
    if !release_ids.contains(&release_id) {
        release_ids.push(release_id);
    }
}

fn run_model_matches(run: &RunSummary, model: &str) -> bool {
    let needle = model.to_ascii_lowercase();
    run.models.iter().any(|candidate| {
        candidate.provider.to_ascii_lowercase().contains(&needle)
            || candidate.name.to_ascii_lowercase().contains(&needle)
            || format!("{}/{}", candidate.provider, candidate.name)
                .to_ascii_lowercase()
                .contains(&needle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{EnvironmentId, ProjectId, TenantId};
    use chrono::{TimeZone, Utc};

    #[test]
    fn span_taxonomy_is_agent_native() {
        assert_eq!(AgentSpanKind::AgentRun.as_str(), "agent.run");
        assert_eq!(AgentSpanKind::McpRequest.as_str(), "mcp.request");
        assert_eq!(AgentSpanKind::ReplayRun.as_str(), "replay.run");
        assert_eq!(
            AgentSpanKind::parse("agent.step"),
            Some(AgentSpanKind::AgentStep)
        );
        assert_eq!(
            AgentSpanKind::parse("agent_step"),
            Some(AgentSpanKind::AgentStep)
        );
        assert_eq!(
            serde_json::to_value(&AgentSpanKind::LlmCall).unwrap_or_else(|err| panic!("{err}")),
            Value::String("llm.call".to_string())
        );
        assert_eq!(
            serde_json::from_value::<AgentSpanKind>(Value::String("tool_call".to_string()))
                .unwrap_or_else(|err| panic!("{err}")),
            AgentSpanKind::ToolCall
        );
        assert_eq!(AgentSpanKind::parse("bogus"), None);
        assert_eq!(SpanStatus::Error.as_str(), "error");
        assert_eq!(SpanStatus::parse("unset"), Some(SpanStatus::Unset));
    }

    #[test]
    fn idempotency_key_includes_tenant_and_hash() {
        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        let trace_id = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
        let span_id = SpanId::new("span").unwrap_or_else(|err| panic!("{err}"));
        let hash = Sha256Hash::new("abc").unwrap_or_else(|err| panic!("{err}"));

        let key = make_idempotency_key(&scope, &trace_id, &span_id, 7, &hash)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(key.as_str(), "tenant:project:trace:span:7:abc");
    }

    #[test]
    fn rollups_use_complete_trace_order_and_filter_after_aggregation() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let other_project = ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}"));
        let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
        let other_trace = TraceId::new("other-trace").unwrap_or_else(|err| panic!("{err}"));
        let early = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"));
        let middle = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 1)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"));
        let late = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 2)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"));
        let spans = vec![
            SpanSummary {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                trace_id: trace.clone(),
                span_id: SpanId::new("child").unwrap_or_else(|err| panic!("{err}")),
                kind: AgentSpanKind::AgentStep,
                name: "latest child".to_string(),
                status: SpanStatus::Ok,
                started_at: late,
                ended_at: Some(late),
                model: Some(ModelRef {
                    provider: "openai".to_string(),
                    name: "gpt-test".to_string(),
                }),
                cost: Some(Money::usd_micros(300)),
                release_id: Some("release-a".to_string()),
            },
            SpanSummary {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                trace_id: trace.clone(),
                span_id: SpanId::new("root").unwrap_or_else(|err| panic!("{err}")),
                kind: AgentSpanKind::AgentRun,
                name: "earliest root".to_string(),
                status: SpanStatus::Error,
                started_at: early,
                ended_at: Some(middle),
                model: None,
                cost: Some(Money::usd_micros(200)),
                release_id: Some("release-a".to_string()),
            },
            SpanSummary {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                trace_id: other_trace.clone(),
                span_id: SpanId::new("other").unwrap_or_else(|err| panic!("{err}")),
                kind: AgentSpanKind::LlmCall,
                name: "other run".to_string(),
                status: SpanStatus::Ok,
                started_at: middle,
                ended_at: Some(middle),
                model: Some(ModelRef {
                    provider: "anthropic".to_string(),
                    name: "claude-test".to_string(),
                }),
                cost: Some(Money::usd_micros(50)),
                release_id: Some("release-b".to_string()),
            },
            SpanSummary {
                tenant_id: tenant.clone(),
                project_id: other_project.clone(),
                trace_id: trace.clone(),
                span_id: SpanId::new("same-trace-other-project")
                    .unwrap_or_else(|err| panic!("{err}")),
                kind: AgentSpanKind::LlmCall,
                name: "same trace other project".to_string(),
                status: SpanStatus::Ok,
                started_at: middle,
                ended_at: Some(middle),
                model: None,
                cost: Some(Money::usd_micros(25)),
                release_id: Some("release-c".to_string()),
            },
        ];

        let runs = roll_up_runs(tenant, spans.clone());
        assert_eq!(runs.len(), 3);
        let run = runs
            .iter()
            .find(|candidate| candidate.project_id == project && candidate.trace_id == trace)
            .unwrap_or_else(|| panic!("trace run exists"));
        assert_eq!(run.project_id.as_str(), project.as_str());
        assert_eq!(run.first_span_name, "earliest root");
        assert_eq!(run.span_count, 2);
        assert_eq!(run.status, SpanStatus::Error);
        assert_eq!(run.started_at, early);
        assert_eq!(run.ended_at, Some(late));
        assert_eq!(run.duration_ms, Some(2000));
        assert_eq!(run.total_cost, Some(Money::usd_micros(500)));
        assert_eq!(run.models.len(), 1);
        assert_eq!(run.models[0].name, "gpt-test");
        assert_eq!(run.release_ids, vec!["release-a".to_string()]);

        let error_agent_step_runs = filter_run_summaries(
            runs.clone(),
            &spans,
            &RunFilter {
                kind: Some(AgentSpanKind::AgentStep),
                status: Some(SpanStatus::Error),
                model: Some("gpt".to_string()),
                release: Some("release-a".to_string()),
                min_cost_micros: Some(400),
                max_cost_micros: Some(600),
                min_latency_ms: Some(1500),
                max_latency_ms: Some(2500),
                started_after: Some(early),
                started_before: Some(middle),
                ..RunFilter::default()
            },
        );
        assert_eq!(error_agent_step_runs.len(), 1);
        assert_eq!(
            error_agent_step_runs[0].project_id.as_str(),
            project.as_str()
        );
        assert_eq!(error_agent_step_runs[0].trace_id, trace);
        assert_eq!(error_agent_step_runs[0].span_count, 2);

        let ok_runs = filter_run_summaries(
            runs,
            &spans,
            &RunFilter {
                status: Some(SpanStatus::Ok),
                release: Some("release-b".to_string()),
                model: Some("anthropic/claude".to_string()),
                max_cost_micros: Some(100),
                ..RunFilter::default()
            },
        );
        assert_eq!(ok_runs.len(), 1);
        assert_eq!(ok_runs[0].project_id.as_str(), project.as_str());
        assert_eq!(ok_runs[0].trace_id, other_trace);
    }
}
