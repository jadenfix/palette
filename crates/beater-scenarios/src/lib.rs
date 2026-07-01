//! beater-scenarios — scenario mining & replay data engine.
//!
//! This crate mines recurring failure patterns out of agent traces and promotes
//! them into reusable [`Scenario`]s for replay/eval. Everything here is
//! deterministic and non-LLM: identical input always yields identical output.
//!
//! The pipeline is:
//!
//! 1. Reduce raw [`CanonicalSpan`]s to lightweight [`TraceSummary`]s.
//! 2. Compute a structural [`Signature`] (shingled fingerprint of failing spans).
//! 3. Infer a [`FailureMode`] from span kinds/statuses.
//! 4. [`cluster_failures`] groups failing traces by Jaccard signature similarity.
//! 5. [`promote_cluster_to_scenario`] turns a [`ScenarioCluster`] into a
//!    [`Scenario`].

use std::collections::BTreeSet;

use beater_core::{sha256_json_hash, JsonHashError, TenantScope, Timestamp, TraceId};
use beater_schema::{AgentSpanKind, CanonicalSpan, RedactionClass, SpanStatus};
use serde::{Deserialize, Serialize};

/// Errors that can occur while mining scenarios.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    /// Hashing a signature failed.
    #[error("hash signature: {0}")]
    Hash(#[from] JsonHashError),
}

/// A single span reduced to the fields scenario mining cares about.
///
/// `kind` and `status` are stored as the canonical wire strings
/// (`AgentSpanKind::as_str` / `SpanStatus::as_str`) so that callers that only
/// have string telemetry can build summaries without depending on the schema
/// enums.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SpanSummary {
    /// Monotonic per-trace ordering key.
    pub seq: u64,
    /// Canonical span kind string, e.g. `"tool.call"`.
    pub kind: String,
    /// Canonical span status string, e.g. `"error"`.
    pub status: String,
    /// Human-readable span name.
    pub name: String,
}

impl SpanSummary {
    /// Build a summary from a canonical span.
    pub fn from_span(span: &CanonicalSpan) -> Self {
        Self {
            seq: span.seq,
            kind: span.kind.as_str().to_string(),
            status: span.status.as_str().to_string(),
            name: span.name.clone(),
        }
    }

    /// Whether this span represents a failure.
    fn is_failing(&self) -> bool {
        self.status == SpanStatus::Error.as_str()
    }
}

/// A trace reduced to an ordered list of [`SpanSummary`]s.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TraceSummary {
    /// The trace this summary describes.
    pub trace_id: TraceId,
    /// Spans in ascending `seq` order.
    pub spans: Vec<SpanSummary>,
}

impl TraceSummary {
    /// Build a summary from canonical spans.
    ///
    /// All spans are assumed to belong to a single trace; the `trace_id` is taken
    /// from the first span. Spans are sorted by `seq` (ties broken by span id) so
    /// that output is independent of input ordering. Returns `None` when `spans`
    /// is empty (no trace id to attribute).
    pub fn from_spans(spans: &[CanonicalSpan]) -> Option<Self> {
        let first = spans.first()?;
        let trace_id = first.trace_id.clone();
        let mut sorted: Vec<&CanonicalSpan> = spans.iter().collect();
        sorted.sort_by(|a, b| a.seq.cmp(&b.seq).then_with(|| a.span_id.cmp(&b.span_id)));
        let spans = sorted.iter().map(|s| SpanSummary::from_span(s)).collect();
        Some(Self { trace_id, spans })
    }

    /// Whether the trace contains at least one failing span.
    pub fn is_failing(&self) -> bool {
        self.spans.iter().any(SpanSummary::is_failing)
    }

    /// The ordered failing-span `(kind, status)` shingles used to fingerprint the
    /// failure shape. Spans are visited in `seq` order; only failing spans count.
    fn failing_shingles(&self) -> Vec<String> {
        let mut failing: Vec<&SpanSummary> = self.spans.iter().filter(|s| s.is_failing()).collect();
        failing.sort_by_key(|s| s.seq);
        failing
            .iter()
            .map(|s| format!("{}|{}", s.kind, s.status))
            .collect()
    }
}

/// A structural fingerprint of a trace's failure shape.
///
/// Two traces with the same ordered failing-span shingles share a [`Signature`]
/// (and therefore the same [`Signature::hash`]). The `shingles` set is also used
/// for Jaccard similarity during clustering.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct Signature {
    /// Ordered `(kind|status)` shingles of failing spans.
    pub shingles: Vec<String>,
    /// Stable sha256 hash of the ordered shingles.
    pub hash: String,
}

impl Signature {
    /// The distinct shingles as a set, for Jaccard similarity.
    fn shingle_set(&self) -> BTreeSet<&str> {
        self.shingles.iter().map(String::as_str).collect()
    }
}

/// Extract the structural [`Signature`] from a trace summary.
pub fn extract_signature(trace: &TraceSummary) -> Result<Signature, ScenarioError> {
    let shingles = trace.failing_shingles();
    let hash = sha256_json_hash(&shingles)?.as_str().to_string();
    Ok(Signature { shingles, hash })
}

/// A deterministically-inferred class of failure.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FailureMode {
    /// A `tool.call` (or `mcp.request`) errored.
    ToolError,
    /// A span name/status indicates a timeout/deadline.
    Timeout,
    /// A `guardrail.check` blocked the run.
    GuardrailBlock,
    /// The agent produced an incorrect output (e.g. an `agent.run`/`llm.call`
    /// error with no more specific signal).
    WrongOutput,
    /// A `retrieval.query` returned nothing useful or errored.
    RetrievalMiss,
    /// No more specific class could be inferred.
    Other,
}

impl FailureMode {
    /// Canonical snake_case string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ToolError => "tool_error",
            Self::Timeout => "timeout",
            Self::GuardrailBlock => "guardrail_block",
            Self::WrongOutput => "wrong_output",
            Self::RetrievalMiss => "retrieval_miss",
            Self::Other => "other",
        }
    }

    /// Classify a single span, if it looks like a failure. The order of checks is
    /// the priority used when a trace has several failing spans.
    fn classify_span(span: &SpanSummary) -> Option<Self> {
        if !span.is_failing() {
            return None;
        }
        let name = span.name.to_ascii_lowercase();
        let is_timeout = name.contains("timeout") || name.contains("deadline");
        let kind = AgentSpanKind::parse(&span.kind);
        match kind {
            Some(AgentSpanKind::GuardrailCheck) => Some(Self::GuardrailBlock),
            _ if is_timeout => Some(Self::Timeout),
            Some(AgentSpanKind::ToolCall) | Some(AgentSpanKind::McpRequest) => {
                Some(Self::ToolError)
            }
            Some(AgentSpanKind::RetrievalQuery) | Some(AgentSpanKind::MemoryRead) => {
                Some(Self::RetrievalMiss)
            }
            Some(AgentSpanKind::AgentRun)
            | Some(AgentSpanKind::AgentTurn)
            | Some(AgentSpanKind::AgentPlan)
            | Some(AgentSpanKind::AgentStep)
            | Some(AgentSpanKind::LlmCall) => Some(Self::WrongOutput),
            _ => Some(Self::Other),
        }
    }
}

/// Numeric priority for picking the dominant failure mode (lower wins). This is a
/// total, deterministic ordering so ties never depend on iteration order.
fn failure_mode_priority(mode: FailureMode) -> u8 {
    match mode {
        FailureMode::GuardrailBlock => 0,
        FailureMode::Timeout => 1,
        FailureMode::ToolError => 2,
        FailureMode::RetrievalMiss => 3,
        FailureMode::WrongOutput => 4,
        FailureMode::Other => 5,
    }
}

/// Infer the dominant [`FailureMode`] for a whole trace. Returns `None` for a
/// trace with no failing spans. The most specific (lowest-priority-number) class
/// across all failing spans wins.
pub fn infer_failure_mode(trace: &TraceSummary) -> Option<FailureMode> {
    trace
        .spans
        .iter()
        .filter_map(FailureMode::classify_span)
        .min_by_key(|m| failure_mode_priority(*m))
}

/// A cluster of failing traces that share a similar failure signature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ScenarioCluster {
    /// The signature of the cluster's exemplar.
    pub signature: Signature,
    /// All member trace ids, sorted ascending.
    pub member_trace_ids: Vec<TraceId>,
    /// The representative trace for the cluster.
    pub exemplar_trace_id: TraceId,
    /// Number of member traces.
    pub size: usize,
    /// The most common failure mode across members.
    pub dominant_failure_mode: FailureMode,
}

/// Jaccard similarity between two shingle sets. Two empty sets are defined as
/// fully similar (`1.0`) so that traces that fail without any error spans still
/// cluster together.
fn jaccard(a: &BTreeSet<&str>, b: &BTreeSet<&str>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        return 1.0;
    }
    intersection as f64 / union as f64
}

/// One failing trace plus its precomputed signature and failure mode.
struct Failing {
    trace_id: TraceId,
    signature: Signature,
    failure_mode: FailureMode,
}

/// Cluster failing traces by signature (Jaccard over failing-span shingles).
///
/// Non-failing traces are ignored. Clustering is a deterministic single-pass
/// greedy assignment over traces sorted by `(trace_id)`: each trace joins the
/// first existing cluster whose exemplar similarity meets `jaccard_threshold`,
/// otherwise it seeds a new cluster. Output clusters are sorted by `size`
/// descending, then by `signature` ascending, for stable ordering.
pub fn cluster_failures(
    traces: &[TraceSummary],
    jaccard_threshold: f64,
) -> Result<Vec<ScenarioCluster>, ScenarioError> {
    // Reduce to failing traces with precomputed signatures, in deterministic order.
    let mut failing: Vec<Failing> = Vec::new();
    for trace in traces {
        if !trace.is_failing() {
            continue;
        }
        let signature = extract_signature(trace)?;
        let failure_mode = infer_failure_mode(trace).unwrap_or(FailureMode::Other);
        failing.push(Failing {
            trace_id: trace.trace_id.clone(),
            signature,
            failure_mode,
        });
    }
    failing.sort_by(|a, b| a.trace_id.cmp(&b.trace_id));

    // Greedy single-pass assignment. Each working cluster keeps its exemplar's
    // shingle set so similarity is measured against a fixed representative.
    struct Working {
        exemplar: Failing,
        members: Vec<Failing>,
    }
    let mut working: Vec<Working> = Vec::new();
    for item in failing {
        let item_set = item.signature.shingle_set();
        let target = working.iter().position(|cluster| {
            let exemplar_set = cluster.exemplar.signature.shingle_set();
            jaccard(&item_set, &exemplar_set) >= jaccard_threshold
        });
        drop(item_set);
        match target.and_then(|idx| working.get_mut(idx)) {
            Some(cluster) => cluster.members.push(item),
            None => working.push(Working {
                exemplar: Failing {
                    trace_id: item.trace_id.clone(),
                    signature: item.signature.clone(),
                    failure_mode: item.failure_mode,
                },
                members: vec![item],
            }),
        }
    }

    let mut clusters: Vec<ScenarioCluster> = working
        .into_iter()
        .map(|w| {
            let mut member_trace_ids: Vec<TraceId> =
                w.members.iter().map(|m| m.trace_id.clone()).collect();
            member_trace_ids.sort();
            let dominant_failure_mode = dominant_mode(&w.members);
            ScenarioCluster {
                size: member_trace_ids.len(),
                signature: w.exemplar.signature,
                exemplar_trace_id: w.exemplar.trace_id,
                member_trace_ids,
                dominant_failure_mode,
            }
        })
        .collect();

    clusters.sort_by(|a, b| {
        b.size
            .cmp(&a.size)
            .then_with(|| a.signature.cmp(&b.signature))
    });
    Ok(clusters)
}

/// Pick the most common failure mode among members; ties broken by the stable
/// [`failure_mode_priority`].
fn dominant_mode(members: &[Failing]) -> FailureMode {
    let modes = [
        FailureMode::ToolError,
        FailureMode::Timeout,
        FailureMode::GuardrailBlock,
        FailureMode::WrongOutput,
        FailureMode::RetrievalMiss,
        FailureMode::Other,
    ];
    modes
        .into_iter()
        .map(|mode| {
            let count = members.iter().filter(|m| m.failure_mode == mode).count();
            (mode, count)
        })
        .filter(|(_, count)| *count > 0)
        .max_by(|(mode_a, count_a), (mode_b, count_b)| {
            count_a.cmp(count_b).then_with(|| {
                // Higher priority (lower number) should win on ties, so reverse.
                failure_mode_priority(*mode_b).cmp(&failure_mode_priority(*mode_a))
            })
        })
        .map(|(mode, _)| mode)
        .unwrap_or(FailureMode::Other)
}

/// Tunable knobs describing how a scenario may be perturbed during replay.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PerturbationKnobs {
    /// Serve a stale version of a context source.
    pub stale_source: bool,
    /// Inject a contradictory context source.
    pub contradictory_source: bool,
    /// Present a tool whose schema mismatches expectations.
    pub tool_schema_mismatch: bool,
    /// Force an auth failure on a dependency.
    pub auth_failure: bool,
    /// Force a timeout on a dependency.
    pub timeout: bool,
    /// Attempt a prompt-injection payload.
    pub prompt_injection: bool,
}

impl PerturbationKnobs {
    /// Default knobs biased toward the given failure mode, so a promoted scenario
    /// suggests the most relevant perturbation to exercise.
    pub fn for_failure_mode(mode: FailureMode) -> Self {
        let mut knobs = Self::default();
        match mode {
            FailureMode::ToolError => knobs.tool_schema_mismatch = true,
            FailureMode::Timeout => knobs.timeout = true,
            FailureMode::GuardrailBlock => knobs.prompt_injection = true,
            FailureMode::RetrievalMiss => knobs.stale_source = true,
            FailureMode::WrongOutput => knobs.contradictory_source = true,
            FailureMode::Other => {}
        }
        knobs
    }
}

/// A reusable failure scenario mined from production traces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Scenario {
    /// Stable, deterministic identifier for the scenario.
    pub scenario_id: String,
    /// Tenant/project/environment scope this scenario belongs to.
    pub scope: TenantScope,
    /// Human-readable title.
    pub title: String,
    /// The dominant failure mode this scenario reproduces.
    pub failure_mode: FailureMode,
    /// Trace ids the scenario was mined from, sorted ascending.
    pub source_trace_ids: Vec<TraceId>,
    /// The representative trace.
    pub exemplar_trace_id: TraceId,
    /// How many traces exhibited this scenario.
    pub recurrence_count: usize,
    /// Suggested perturbation knobs for replay.
    pub perturbation_knobs: PerturbationKnobs,
    /// Expected outcome for replay assertions, if known.
    pub expected_outcome: Option<String>,
    /// Redaction classification of the scenario payload.
    pub redaction_class: RedactionClass,
    /// When the scenario was created.
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

/// Promote a [`ScenarioCluster`] into a [`Scenario`].
///
/// The `scenario_id` is a deterministic sha256 of the scope, exemplar trace id,
/// failure mode, and signature hash, so re-promoting the same cluster within the
/// same scope yields the same id.
pub fn promote_cluster_to_scenario(
    cluster: &ScenarioCluster,
    scope: TenantScope,
    title: impl Into<String>,
    now: Timestamp,
) -> Result<Scenario, ScenarioError> {
    let failure_mode = cluster.dominant_failure_mode;
    let id_seed = (
        scope.tenant_id.as_str(),
        scope.project_id.as_str(),
        scope.environment_id.as_str(),
        cluster.exemplar_trace_id.as_str(),
        failure_mode.as_str(),
        cluster.signature.hash.as_str(),
    );
    let scenario_id = sha256_json_hash(&id_seed)?.as_str().to_string();
    Ok(Scenario {
        scenario_id,
        scope,
        title: title.into(),
        failure_mode,
        source_trace_ids: cluster.member_trace_ids.clone(),
        exemplar_trace_id: cluster.exemplar_trace_id.clone(),
        recurrence_count: cluster.size,
        perturbation_knobs: PerturbationKnobs::for_failure_mode(failure_mode),
        expected_outcome: None,
        redaction_class: RedactionClass::Internal,
        created_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{EnvironmentId, ProjectId, TenantId};

    fn span(seq: u64, kind: &str, status: &str, name: &str) -> SpanSummary {
        SpanSummary {
            seq,
            kind: kind.to_string(),
            status: status.to_string(),
            name: name.to_string(),
        }
    }

    fn trace(id: &str, spans: Vec<SpanSummary>) -> TraceSummary {
        TraceSummary {
            trace_id: TraceId::new(id).unwrap_or_else(|e| panic!("{e}")),
            spans,
        }
    }

    fn scope() -> TenantScope {
        TenantScope::new(
            TenantId::new("t1").unwrap_or_else(|e| panic!("{e}")),
            ProjectId::new("p1").unwrap_or_else(|e| panic!("{e}")),
            EnvironmentId::new("e1").unwrap_or_else(|e| panic!("{e}")),
        )
    }

    fn now() -> Timestamp {
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms(2026, 6, 28, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"))
    }

    #[test]
    fn span_summary_failure_detection() {
        assert!(span(0, "tool.call", "error", "x").is_failing());
        assert!(!span(0, "tool.call", "ok", "x").is_failing());
    }

    #[test]
    fn from_spans_empty_is_none() {
        assert!(TraceSummary::from_spans(&[]).is_none());
    }

    #[test]
    fn signature_is_deterministic() -> anyhow::Result<()> {
        let t = trace(
            "a",
            vec![
                span(1, "agent.run", "ok", "run"),
                span(2, "tool.call", "error", "fetch"),
            ],
        );
        let s1 = extract_signature(&t)?;
        let s2 = extract_signature(&t)?;
        assert_eq!(s1, s2);
        assert_eq!(s1.shingles, vec!["tool.call|error".to_string()]);
        assert_eq!(s1.hash.len(), 64);
        Ok(())
    }

    #[test]
    fn signature_ignores_ok_spans() -> anyhow::Result<()> {
        let with_ok = trace(
            "a",
            vec![
                span(1, "llm.call", "ok", "x"),
                span(2, "tool.call", "error", "fetch"),
            ],
        );
        let only_fail = trace("b", vec![span(2, "tool.call", "error", "fetch")]);
        assert_eq!(
            extract_signature(&with_ok)?.hash,
            extract_signature(&only_fail)?.hash
        );
        Ok(())
    }

    #[test]
    fn infer_tool_error() {
        let t = trace(
            "a",
            vec![
                span(1, "agent.run", "ok", "run"),
                span(2, "tool.call", "error", "search"),
            ],
        );
        assert_eq!(infer_failure_mode(&t), Some(FailureMode::ToolError));
    }

    #[test]
    fn infer_guardrail_block() {
        let t = trace("a", vec![span(1, "guardrail.check", "error", "policy")]);
        assert_eq!(infer_failure_mode(&t), Some(FailureMode::GuardrailBlock));
    }

    #[test]
    fn infer_timeout_from_name() {
        let t = trace("a", vec![span(1, "llm.call", "error", "request timeout")]);
        assert_eq!(infer_failure_mode(&t), Some(FailureMode::Timeout));
    }

    #[test]
    fn infer_retrieval_miss() {
        let t = trace("a", vec![span(1, "retrieval.query", "error", "vector")]);
        assert_eq!(infer_failure_mode(&t), Some(FailureMode::RetrievalMiss));
    }

    #[test]
    fn infer_wrong_output_for_llm() {
        let t = trace("a", vec![span(1, "llm.call", "error", "completion")]);
        assert_eq!(infer_failure_mode(&t), Some(FailureMode::WrongOutput));
    }

    #[test]
    fn infer_dominant_picks_most_specific() {
        // Both guardrail and tool errors present; guardrail wins by priority.
        let t = trace(
            "a",
            vec![
                span(1, "tool.call", "error", "search"),
                span(2, "guardrail.check", "error", "policy"),
            ],
        );
        assert_eq!(infer_failure_mode(&t), Some(FailureMode::GuardrailBlock));
    }

    #[test]
    fn no_failure_mode_for_clean_trace() {
        let t = trace("a", vec![span(1, "agent.run", "ok", "run")]);
        assert_eq!(infer_failure_mode(&t), None);
        assert!(!t.is_failing());
    }

    #[test]
    fn clustering_groups_similar_separates_dissimilar() -> anyhow::Result<()> {
        let tool_fail_a = trace(
            "a",
            vec![
                span(1, "agent.run", "ok", "run"),
                span(2, "tool.call", "error", "search"),
            ],
        );
        let tool_fail_b = trace(
            "b",
            vec![
                span(1, "agent.turn", "ok", "turn"),
                span(2, "tool.call", "error", "lookup"),
            ],
        );
        let guardrail_fail = trace("c", vec![span(1, "guardrail.check", "error", "policy")]);

        let clusters = cluster_failures(&[tool_fail_a, tool_fail_b, guardrail_fail], 1.0)?;
        assert_eq!(clusters.len(), 2);
        // Largest cluster first: the two tool errors.
        assert_eq!(clusters[0].size, 2);
        assert_eq!(clusters[0].dominant_failure_mode, FailureMode::ToolError);
        assert_eq!(
            clusters[0].member_trace_ids,
            vec![
                TraceId::new("a").unwrap_or_else(|e| panic!("{e}")),
                TraceId::new("b").unwrap_or_else(|e| panic!("{e}")),
            ]
        );
        assert_eq!(clusters[1].size, 1);
        assert_eq!(
            clusters[1].dominant_failure_mode,
            FailureMode::GuardrailBlock
        );
        Ok(())
    }

    #[test]
    fn clustering_ignores_passing_traces() -> anyhow::Result<()> {
        let ok = trace("a", vec![span(1, "agent.run", "ok", "run")]);
        let fail = trace("b", vec![span(1, "tool.call", "error", "x")]);
        let clusters = cluster_failures(&[ok, fail], 0.5)?;
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].size, 1);
        Ok(())
    }

    #[test]
    fn clustering_threshold_controls_merging() -> anyhow::Result<()> {
        // Two failing spans each, sharing one shingle -> jaccard = 1/3.
        let a = trace(
            "a",
            vec![
                span(1, "tool.call", "error", "x"),
                span(2, "llm.call", "error", "y"),
            ],
        );
        let b = trace(
            "b",
            vec![
                span(1, "tool.call", "error", "x"),
                span(2, "retrieval.query", "error", "z"),
            ],
        );
        // High threshold keeps them apart.
        assert_eq!(cluster_failures(&[a.clone(), b.clone()], 0.9)?.len(), 2);
        // Low threshold merges them.
        assert_eq!(cluster_failures(&[a, b], 0.3)?.len(), 1);
        Ok(())
    }

    #[test]
    fn clustering_is_deterministic() -> anyhow::Result<()> {
        let traces = vec![
            trace("c", vec![span(1, "tool.call", "error", "x")]),
            trace("a", vec![span(1, "guardrail.check", "error", "p")]),
            trace("b", vec![span(1, "tool.call", "error", "y")]),
        ];
        let r1 = cluster_failures(&traces, 1.0)?;
        let r2 = cluster_failures(&traces, 1.0)?;
        assert_eq!(r1, r2);
        Ok(())
    }

    #[test]
    fn promotion_fills_fields() -> anyhow::Result<()> {
        let fail_a = trace("a", vec![span(1, "tool.call", "error", "search")]);
        let fail_b = trace("b", vec![span(1, "tool.call", "error", "lookup")]);
        let clusters = cluster_failures(&[fail_a, fail_b], 1.0)?;
        assert_eq!(clusters.len(), 1);
        let scenario = promote_cluster_to_scenario(&clusters[0], scope(), "Tool failures", now())?;

        assert_eq!(scenario.title, "Tool failures");
        assert_eq!(scenario.failure_mode, FailureMode::ToolError);
        assert_eq!(scenario.recurrence_count, 2);
        assert_eq!(scenario.source_trace_ids.len(), 2);
        assert!(scenario.perturbation_knobs.tool_schema_mismatch);
        assert!(!scenario.perturbation_knobs.timeout);
        assert_eq!(scenario.redaction_class, RedactionClass::Internal);
        assert_eq!(scenario.expected_outcome, None);
        assert_eq!(scenario.created_at, now());
        assert_eq!(scenario.scenario_id.len(), 64);
        Ok(())
    }

    #[test]
    fn promotion_id_is_stable() -> anyhow::Result<()> {
        let fail = trace("a", vec![span(1, "tool.call", "error", "search")]);
        let clusters = cluster_failures(&[fail], 1.0)?;
        let s1 = promote_cluster_to_scenario(&clusters[0], scope(), "t", now())?;
        let s2 = promote_cluster_to_scenario(&clusters[0], scope(), "t", now())?;
        assert_eq!(s1.scenario_id, s2.scenario_id);
        Ok(())
    }

    #[test]
    fn perturbation_knobs_track_failure_mode() {
        assert!(PerturbationKnobs::for_failure_mode(FailureMode::Timeout).timeout);
        assert!(PerturbationKnobs::for_failure_mode(FailureMode::GuardrailBlock).prompt_injection);
        assert!(PerturbationKnobs::for_failure_mode(FailureMode::RetrievalMiss).stale_source);
        assert!(PerturbationKnobs::for_failure_mode(FailureMode::WrongOutput).contradictory_source);
        assert_eq!(
            PerturbationKnobs::for_failure_mode(FailureMode::Other),
            PerturbationKnobs::default()
        );
    }

    #[test]
    fn failure_mode_serde_snake_case() -> anyhow::Result<()> {
        let json = serde_json::to_string(&FailureMode::GuardrailBlock)?;
        assert_eq!(json, "\"guardrail_block\"");
        Ok(())
    }
}
