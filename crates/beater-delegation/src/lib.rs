//! beater-delegation — A2A and inter-agent delegation traces with trust
//! boundaries (issue #268).
//!
//! This crate models cross-agent (agent-to-agent / A2A) delegation as a graph
//! of [`DelegationTask`]s with associated [`DelegationResult`]s, and provides:
//!
//! - trust-boundary classification ([`TrustBoundary`]) and policy enforcement
//!   ([`TrustPolicy`] / [`trust_violations`]),
//! - a delegation graph with cycle detection ([`DelegationGraph`]),
//! - failure attribution across a delegation chain
//!   ([`attribute_chain_failure`]), and
//! - a span-attribute convention mapping ([`delegation_span_attributes`]) so an
//!   exporter can attach delegation context to a span.
//!
//! A real A2A delegation span would use a dedicated span kind. Because this
//! crate cannot edit `beater-schema`, the span kind and attribute keys live here
//! as local convention constants ([`DELEGATION_SPAN_KIND`] and the `attr`
//! module).

use std::collections::{BTreeMap, BTreeSet};

use beater_core::{AgentId, SpanId, TraceId};
use beater_schema::{ArtifactRef, RedactionClass};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Local convention constant for the delegation span kind.
///
/// A production deployment would promote this to a `beater-schema` span kind;
/// it is defined locally here so this crate stays self-contained.
pub const DELEGATION_SPAN_KIND: &str = "agent.delegation";

/// Attribute-key constants for delegation span attributes.
///
/// These are the keys produced by [`delegation_span_attributes`].
pub mod attr {
    /// Source agent identifier.
    pub const SOURCE_ID: &str = "agent.source_id";
    /// Target (delegate) agent identifier.
    pub const TARGET_ID: &str = "agent.target_id";
    /// Delegation protocol (snake_case [`super::Protocol`]).
    pub const PROTOCOL: &str = "agent.protocol";
    /// Human-readable intent of the delegation.
    pub const INTENT: &str = "agent.intent";
    /// Stable task identifier.
    pub const TASK_ID: &str = "agent.delegation.task_id";
    /// Parent trace identifier the delegation belongs to.
    pub const PARENT_TRACE_ID: &str = "agent.delegation.parent_trace_id";
    /// Parent span identifier the delegation was issued from.
    pub const PARENT_SPAN_ID: &str = "agent.delegation.parent_span_id";
    /// Comma-separated, sorted data classes carried by the delegation.
    pub const DATA_CLASSES: &str = "agent.delegation.data_classes";
    /// Whether source and target share a tenant.
    pub const TRUST_SAME_TENANT: &str = "agent.trust.same_tenant";
    /// Whether the delegation crosses a trust boundary.
    pub const TRUST_CROSS_BOUNDARY: &str = "agent.trust.cross_boundary";
    /// Source region, when known.
    pub const TRUST_SOURCE_REGION: &str = "agent.trust.source_region";
    /// Target region, when known.
    pub const TRUST_TARGET_REGION: &str = "agent.trust.target_region";
    /// Source vendor, when known.
    pub const TRUST_SOURCE_VENDOR: &str = "agent.trust.source_vendor";
    /// Target vendor, when known.
    pub const TRUST_TARGET_VENDOR: &str = "agent.trust.target_vendor";
}

/// The protocol used to delegate work from one agent to another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// Agent-to-agent protocol (A2A).
    A2a,
    /// An MCP server invoked as if it were a delegated agent.
    McpAsAgent,
    /// Delegation via an outbound webhook.
    Webhook,
    /// Hand-off to a human in the loop.
    Human,
}

impl Protocol {
    /// Stable snake_case string for span attributes.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A2a => "a2a",
            Self::McpAsAgent => "mcp_as_agent",
            Self::Webhook => "webhook",
            Self::Human => "human",
        }
    }
}

/// Describes the trust relationship between the source and target of a
/// delegation. A delegation is "cross-boundary" when the two parties differ in
/// tenant, region, or vendor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrustBoundary {
    /// Whether source and target belong to the same tenant.
    pub same_tenant: bool,
    /// Region the source agent runs in, when known.
    pub source_region: Option<String>,
    /// Region the target agent runs in, when known.
    pub target_region: Option<String>,
    /// Vendor/operator of the source agent, when known.
    pub source_vendor: Option<String>,
    /// Vendor/operator of the target agent, when known.
    pub target_vendor: Option<String>,
}

impl TrustBoundary {
    /// Returns `true` when the delegation crosses a trust boundary: a different
    /// tenant, or a known-and-differing region, or a known-and-differing vendor.
    pub fn is_cross_boundary(&self) -> bool {
        if !self.same_tenant {
            return true;
        }
        if differs(&self.source_region, &self.target_region) {
            return true;
        }
        if differs(&self.source_vendor, &self.target_vendor) {
            return true;
        }
        false
    }
}

/// Two optional values "differ" only when both are present and unequal. Unknown
/// (None) endpoints are not treated as a boundary on their own.
fn differs(left: &Option<String>, right: &Option<String>) -> bool {
    matches!((left, right), (Some(a), Some(b)) if a != b)
}

/// Ordered, set-friendly wrapper around [`RedactionClass`].
///
/// `beater_schema::RedactionClass` derives only `Eq`/`PartialEq` (no `Ord`), so
/// it cannot live directly in a `BTreeSet`. This crate cannot edit
/// `beater-schema`, so `DataClass` adds a total order (by sensitivity rank) and
/// serializes transparently as the underlying redaction class.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(transparent)]
pub struct DataClass(pub RedactionClass);

impl DataClass {
    /// The wrapped redaction class.
    pub fn class(&self) -> &RedactionClass {
        &self.0
    }

    /// Sensitivity rank used for ordering (lower is less sensitive).
    fn rank(&self) -> u8 {
        match self.0 {
            RedactionClass::Public => 0,
            RedactionClass::Internal => 1,
            RedactionClass::Sensitive => 2,
            RedactionClass::Secret => 3,
        }
    }

    /// Whether the wrapped class is sensitive or secret.
    pub fn is_sensitive(&self) -> bool {
        matches!(self.0, RedactionClass::Sensitive | RedactionClass::Secret)
    }

    /// Stable snake_case label.
    pub fn as_str(&self) -> &'static str {
        match self.0 {
            RedactionClass::Public => "public",
            RedactionClass::Internal => "internal",
            RedactionClass::Sensitive => "sensitive",
            RedactionClass::Secret => "secret",
        }
    }
}

impl From<RedactionClass> for DataClass {
    fn from(class: RedactionClass) -> Self {
        Self(class)
    }
}

impl PartialOrd for DataClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataClass {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

/// A single delegation of work from one agent to another, anchored to the parent
/// trace/span that issued it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DelegationTask {
    /// Stable identifier for this delegation task.
    pub task_id: String,
    /// Trace this delegation is part of.
    pub parent_trace_id: TraceId,
    /// Span the delegation was issued from.
    pub parent_span_id: SpanId,
    /// Agent issuing the delegation.
    pub source_agent: AgentId,
    /// Agent receiving the delegation.
    pub target_agent: AgentId,
    /// Protocol used to delegate.
    pub protocol: Protocol,
    /// Free-form description of what is being delegated.
    pub intent: String,
    /// Data classes carried across the delegation.
    pub data_classes: BTreeSet<DataClass>,
    /// Trust relationship between source and target.
    pub trust: TrustBoundary,
    /// When the delegation was created.
    pub created_at: chrono::DateTime<Utc>,
}

/// Terminal status of a delegated task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    /// The delegate completed the task.
    Completed,
    /// The delegate ran but failed.
    Failed,
    /// The delegate refused the task (e.g. policy / capability).
    Refused,
    /// The delegate did not respond in time.
    Timeout,
}

impl DelegationStatus {
    /// Stable snake_case string for span attributes / reporting.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Refused => "refused",
            Self::Timeout => "timeout",
        }
    }

    /// Whether this status represents a failure of the delegate (anything other
    /// than [`DelegationStatus::Completed`]).
    pub fn is_failure(&self) -> bool {
        !matches!(self, Self::Completed)
    }
}

/// The outcome of a [`DelegationTask`], including any downstream trace the
/// delegate produced.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DelegationResult {
    /// Identifier of the task this result is for (matches
    /// [`DelegationTask::task_id`]).
    pub task_id: String,
    /// Terminal status of the delegation.
    pub status: DelegationStatus,
    /// Trace the delegate produced, when it emitted one.
    pub downstream_trace_id: Option<TraceId>,
    /// Artifacts returned by the delegate.
    pub artifacts: Vec<ArtifactRef>,
    /// Confidence in the result, when reported, in `[0, 1]`.
    pub confidence: Option<f64>,
    /// Reason supplied when the delegate refused.
    pub refusal_reason: Option<String>,
    /// Number of tool calls the delegate made.
    pub tool_calls: u32,
}

/// Builds the span attribute map for a delegation task so an exporter can attach
/// it to a [`DELEGATION_SPAN_KIND`] span.
pub fn delegation_span_attributes(task: &DelegationTask) -> BTreeMap<String, Value> {
    let mut attrs = BTreeMap::new();
    attrs.insert(
        attr::SOURCE_ID.to_string(),
        Value::String(task.source_agent.to_string()),
    );
    attrs.insert(
        attr::TARGET_ID.to_string(),
        Value::String(task.target_agent.to_string()),
    );
    attrs.insert(
        attr::PROTOCOL.to_string(),
        Value::String(task.protocol.as_str().to_string()),
    );
    attrs.insert(attr::INTENT.to_string(), Value::String(task.intent.clone()));
    attrs.insert(
        attr::TASK_ID.to_string(),
        Value::String(task.task_id.clone()),
    );
    attrs.insert(
        attr::PARENT_TRACE_ID.to_string(),
        Value::String(task.parent_trace_id.to_string()),
    );
    attrs.insert(
        attr::PARENT_SPAN_ID.to_string(),
        Value::String(task.parent_span_id.to_string()),
    );
    attrs.insert(
        attr::DATA_CLASSES.to_string(),
        Value::String(data_classes_str(&task.data_classes)),
    );
    attrs.insert(
        attr::TRUST_SAME_TENANT.to_string(),
        Value::Bool(task.trust.same_tenant),
    );
    attrs.insert(
        attr::TRUST_CROSS_BOUNDARY.to_string(),
        Value::Bool(task.trust.is_cross_boundary()),
    );
    insert_opt(
        &mut attrs,
        attr::TRUST_SOURCE_REGION,
        &task.trust.source_region,
    );
    insert_opt(
        &mut attrs,
        attr::TRUST_TARGET_REGION,
        &task.trust.target_region,
    );
    insert_opt(
        &mut attrs,
        attr::TRUST_SOURCE_VENDOR,
        &task.trust.source_vendor,
    );
    insert_opt(
        &mut attrs,
        attr::TRUST_TARGET_VENDOR,
        &task.trust.target_vendor,
    );
    attrs
}

fn insert_opt(attrs: &mut BTreeMap<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        attrs.insert(key.to_string(), Value::String(value.clone()));
    }
}

/// Renders a stable, sorted, comma-separated string for a set of data classes.
fn data_classes_str(classes: &BTreeSet<DataClass>) -> String {
    classes
        .iter()
        .map(DataClass::as_str)
        .collect::<Vec<_>>()
        .join(",")
}

/// A delegation graph keyed by agent: an adjacency map from each source agent to
/// the set of agents it delegates to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DelegationGraph {
    adjacency: BTreeMap<AgentId, BTreeSet<AgentId>>,
}

impl DelegationGraph {
    /// Builds the adjacency graph from a slice of delegation tasks. Every agent
    /// that appears as a source or target gets a (possibly empty) node.
    pub fn from_tasks(tasks: &[DelegationTask]) -> Self {
        let mut adjacency: BTreeMap<AgentId, BTreeSet<AgentId>> = BTreeMap::new();
        for task in tasks {
            adjacency
                .entry(task.source_agent.clone())
                .or_default()
                .insert(task.target_agent.clone());
            adjacency.entry(task.target_agent.clone()).or_default();
        }
        Self { adjacency }
    }

    /// The set of agents the given agent delegates to directly.
    pub fn targets_of(&self, agent: &AgentId) -> Option<&BTreeSet<AgentId>> {
        self.adjacency.get(agent)
    }

    /// All agents that appear in the graph.
    pub fn agents(&self) -> impl Iterator<Item = &AgentId> {
        self.adjacency.keys()
    }

    /// Number of agent nodes in the graph.
    pub fn len(&self) -> usize {
        self.adjacency.len()
    }

    /// Whether the graph has no nodes.
    pub fn is_empty(&self) -> bool {
        self.adjacency.is_empty()
    }

    /// Detects whether the delegation graph contains a cycle (e.g. an agent that
    /// transitively delegates back to itself). Self-loops count as cycles.
    pub fn has_cycle(&self) -> bool {
        let mut visiting: BTreeSet<AgentId> = BTreeSet::new();
        let mut visited: BTreeSet<AgentId> = BTreeSet::new();
        for node in self.adjacency.keys() {
            if !visited.contains(node) && self.dfs_cycle(node, &mut visiting, &mut visited) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle(
        &self,
        node: &AgentId,
        visiting: &mut BTreeSet<AgentId>,
        visited: &mut BTreeSet<AgentId>,
    ) -> bool {
        visiting.insert(node.clone());
        if let Some(targets) = self.adjacency.get(node) {
            for target in targets {
                if visiting.contains(target) {
                    return true;
                }
                if !visited.contains(target) && self.dfs_cycle(target, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.clone());
        false
    }
}

/// Policy applied to delegations to flag trust-boundary violations.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrustPolicy {
    /// When `true`, delegating sensitive/secret data across a trust boundary is a
    /// violation.
    pub block_sensitive_cross_boundary: bool,
    /// When `Some`, the target vendor must appear in this allowlist; a target
    /// with a vendor outside the set (or an unknown vendor) is a violation.
    pub allowed_target_vendors: Option<BTreeSet<String>>,
}

/// The kind of trust-policy violation a delegation triggered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustViolationKind {
    /// Sensitive or secret data was delegated across a trust boundary.
    SensitiveCrossBoundary,
    /// The target vendor is not in the configured allowlist.
    VendorNotAllowed,
}

/// A single trust-policy violation tied to a specific delegation task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrustViolation {
    /// Task that violated the policy.
    pub task_id: String,
    /// Why the task violated the policy.
    pub kind: TrustViolationKind,
    /// Human-readable detail.
    pub detail: String,
}

/// Evaluates a set of delegation tasks against a [`TrustPolicy`], returning every
/// violation found (in task order, kind order).
pub fn trust_violations(tasks: &[DelegationTask], policy: &TrustPolicy) -> Vec<TrustViolation> {
    let mut violations = Vec::new();
    for task in tasks {
        if policy.block_sensitive_cross_boundary
            && task.trust.is_cross_boundary()
            && task.data_classes.iter().any(DataClass::is_sensitive)
        {
            let classes: Vec<&'static str> = task
                .data_classes
                .iter()
                .filter(|c| c.is_sensitive())
                .map(DataClass::as_str)
                .collect();
            violations.push(TrustViolation {
                task_id: task.task_id.clone(),
                kind: TrustViolationKind::SensitiveCrossBoundary,
                detail: format!(
                    "{} data delegated across trust boundary to {}",
                    classes.join(","),
                    task.target_agent
                ),
            });
        }

        if let Some(allowed) = &policy.allowed_target_vendors {
            let permitted = matches!(&task.trust.target_vendor, Some(v) if allowed.contains(v));
            if !permitted {
                let vendor = task
                    .trust
                    .target_vendor
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string());
                violations.push(TrustViolation {
                    task_id: task.task_id.clone(),
                    kind: TrustViolationKind::VendorNotAllowed,
                    detail: format!("target vendor {vendor} is not in the allowlist"),
                });
            }
        }
    }
    violations
}

/// The root cause of a chain failure: the first downstream delegate (reachable
/// from the parent task) that failed, timed out, or refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FailureAttribution {
    /// Task identifier of the delegate that actually failed.
    pub failing_task_id: String,
    /// The failure status of that delegate.
    pub status: DelegationStatus,
    /// Reason for the failure, when available.
    pub reason: Option<String>,
}

/// Walks a delegation chain rooted at `parent_task_id` and attributes a failure
/// to the first downstream delegate that failed/timed out/refused.
///
/// The chain is reconstructed by linking a task's [`DelegationResult::downstream_trace_id`]
/// to the [`DelegationTask::parent_trace_id`] of further tasks. Both `tasks` and
/// `results` are provided; `results` are keyed by task id.
///
/// Returns `None` when the parent task itself did not fail, or when no failing
/// delegate can be found downstream.
pub fn attribute_chain_failure(
    parent_task_id: &str,
    tasks: &[DelegationTask],
    results: &[DelegationResult],
) -> Option<FailureAttribution> {
    let result_by_task: BTreeMap<&str, &DelegationResult> =
        results.iter().map(|r| (r.task_id.as_str(), r)).collect();
    let task_by_id: BTreeMap<&str, &DelegationTask> =
        tasks.iter().map(|t| (t.task_id.as_str(), t)).collect();

    // Map a downstream trace to the tasks that ran inside it (i.e. whose parent
    // trace is that trace), so we can follow the chain by trace linkage.
    let mut tasks_by_parent_trace: BTreeMap<&str, Vec<&DelegationTask>> = BTreeMap::new();
    for task in tasks {
        tasks_by_parent_trace
            .entry(task.parent_trace_id.as_str())
            .or_default()
            .push(task);
    }

    let parent_task = task_by_id.get(parent_task_id)?;
    let parent_result = result_by_task.get(parent_task_id)?;

    // Only attribute when the parent actually failed.
    if !parent_result.status.is_failure() {
        return None;
    }

    // BFS down the chain, preferring the deepest concrete failure: a delegate
    // that failed but whose own downstream produced no further failing delegate
    // is the culprit.
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    attribute_recursive(
        parent_task,
        parent_result,
        &result_by_task,
        &tasks_by_parent_trace,
        &mut visited,
    )
}

fn attribute_recursive<'a>(
    task: &'a DelegationTask,
    result: &'a DelegationResult,
    result_by_task: &BTreeMap<&'a str, &'a DelegationResult>,
    tasks_by_parent_trace: &BTreeMap<&'a str, Vec<&'a DelegationTask>>,
    visited: &mut BTreeSet<&'a str>,
) -> Option<FailureAttribution> {
    if !visited.insert(task.task_id.as_str()) {
        return None;
    }

    // Follow the downstream trace this delegate produced, if any, and look for a
    // failing delegate one level down. The deepest failure wins.
    if let Some(trace) = &result.downstream_trace_id
        && let Some(children) = tasks_by_parent_trace.get(trace.as_str())
    {
        for child in children {
            if let Some(child_result) = result_by_task.get(child.task_id.as_str())
                && child_result.status.is_failure()
                && let Some(deeper) = attribute_recursive(
                    child,
                    child_result,
                    result_by_task,
                    tasks_by_parent_trace,
                    visited,
                )
            {
                return Some(deeper);
            }
        }
    }

    // No deeper failing delegate: this task is the culprit.
    Some(FailureAttribution {
        failing_task_id: task.task_id.clone(),
        status: result.status,
        reason: result.refusal_reason.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{ArtifactId, Sha256Hash};

    fn agent(s: &str) -> AgentId {
        AgentId::new(s).unwrap_or_else(|e| panic!("{e}"))
    }

    fn trace(s: &str) -> TraceId {
        TraceId::new(s).unwrap_or_else(|e| panic!("{e}"))
    }

    fn span(s: &str) -> SpanId {
        SpanId::new(s).unwrap_or_else(|e| panic!("{e}"))
    }

    fn task(id: &str, src: &str, tgt: &str) -> DelegationTask {
        DelegationTask {
            task_id: id.to_string(),
            parent_trace_id: trace(&format!("trace-{id}")),
            parent_span_id: span(&format!("span-{id}")),
            source_agent: agent(src),
            target_agent: agent(tgt),
            protocol: Protocol::A2a,
            intent: "do work".to_string(),
            data_classes: BTreeSet::new(),
            trust: TrustBoundary {
                same_tenant: true,
                source_region: None,
                target_region: None,
                source_vendor: None,
                target_vendor: None,
            },
            created_at: Utc::now(),
        }
    }

    fn artifact() -> ArtifactRef {
        ArtifactRef {
            artifact_id: ArtifactId::new("art-1").unwrap_or_else(|e| panic!("{e}")),
            uri: "mem://art-1".to_string(),
            sha256: Sha256Hash::new("a".repeat(64)).unwrap_or_else(|e| panic!("{e}")),
            size_bytes: 1,
            mime_type: "text/plain".to_string(),
            redaction_class: RedactionClass::Public,
        }
    }

    fn result(id: &str, status: DelegationStatus) -> DelegationResult {
        DelegationResult {
            task_id: id.to_string(),
            status,
            downstream_trace_id: None,
            artifacts: vec![],
            confidence: None,
            refusal_reason: None,
            tool_calls: 0,
        }
    }

    #[test]
    fn graph_builds_adjacency_keyed_by_agent() {
        let tasks = vec![
            task("t1", "a", "b"),
            task("t2", "a", "c"),
            task("t3", "b", "c"),
        ];
        let g = DelegationGraph::from_tasks(&tasks);
        assert_eq!(g.len(), 3); // a, b, c
        let a_targets = g.targets_of(&agent("a")).unwrap_or_else(|| panic!("no a"));
        assert!(a_targets.contains(&agent("b")));
        assert!(a_targets.contains(&agent("c")));
        assert_eq!(a_targets.len(), 2);
        // c is a leaf: present but no outgoing edges.
        assert_eq!(g.targets_of(&agent("c")).map(|s| s.len()), Some(0));
    }

    #[test]
    fn graph_acyclic_returns_no_cycle() {
        let tasks = vec![task("t1", "a", "b"), task("t2", "b", "c")];
        let g = DelegationGraph::from_tasks(&tasks);
        assert!(!g.has_cycle());
    }

    #[test]
    fn graph_detects_back_edge_cycle() {
        let tasks = vec![
            task("t1", "a", "b"),
            task("t2", "b", "c"),
            task("t3", "c", "a"),
        ];
        let g = DelegationGraph::from_tasks(&tasks);
        assert!(g.has_cycle());
    }

    #[test]
    fn graph_detects_self_loop_cycle() {
        let tasks = vec![task("t1", "a", "a")];
        let g = DelegationGraph::from_tasks(&tasks);
        assert!(g.has_cycle());
    }

    #[test]
    fn trust_boundary_cross_when_different_tenant() {
        let tb = TrustBoundary {
            same_tenant: false,
            source_region: None,
            target_region: None,
            source_vendor: None,
            target_vendor: None,
        };
        assert!(tb.is_cross_boundary());
    }

    #[test]
    fn trust_boundary_cross_on_region_and_vendor_diff() {
        let region = TrustBoundary {
            same_tenant: true,
            source_region: Some("us-east".to_string()),
            target_region: Some("eu-west".to_string()),
            source_vendor: None,
            target_vendor: None,
        };
        assert!(region.is_cross_boundary());

        let vendor = TrustBoundary {
            same_tenant: true,
            source_region: None,
            target_region: None,
            source_vendor: Some("acme".to_string()),
            target_vendor: Some("globex".to_string()),
        };
        assert!(vendor.is_cross_boundary());
    }

    #[test]
    fn trust_boundary_not_cross_when_aligned_or_unknown() {
        let tb = TrustBoundary {
            same_tenant: true,
            source_region: Some("us-east".to_string()),
            target_region: Some("us-east".to_string()),
            source_vendor: None,
            target_vendor: Some("only-target-known".to_string()),
        };
        assert!(!tb.is_cross_boundary());
    }

    #[test]
    fn sensitive_data_crossing_boundary_is_flagged() {
        let mut t = task("t1", "a", "b");
        t.trust.same_tenant = false;
        t.data_classes.insert(DataClass(RedactionClass::Sensitive));
        let policy = TrustPolicy {
            block_sensitive_cross_boundary: true,
            allowed_target_vendors: None,
        };
        let v = trust_violations(&[t], &policy);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, TrustViolationKind::SensitiveCrossBoundary);
        assert_eq!(v[0].task_id, "t1");
    }

    #[test]
    fn sensitive_data_same_boundary_not_flagged() {
        let mut t = task("t1", "a", "b");
        t.data_classes.insert(DataClass(RedactionClass::Secret));
        // same_tenant defaults true and no region/vendor diff -> not cross.
        let policy = TrustPolicy {
            block_sensitive_cross_boundary: true,
            allowed_target_vendors: None,
        };
        assert!(trust_violations(&[t], &policy).is_empty());
    }

    #[test]
    fn vendor_allowlist_flags_disallowed_target() {
        let mut allowed = BTreeSet::new();
        allowed.insert("trusted-vendor".to_string());

        let mut ok = task("t-ok", "a", "b");
        ok.trust.target_vendor = Some("trusted-vendor".to_string());
        let mut bad = task("t-bad", "a", "c");
        bad.trust.target_vendor = Some("rogue-vendor".to_string());
        let mut unknown = task("t-unknown", "a", "d");
        unknown.trust.target_vendor = None;

        let policy = TrustPolicy {
            block_sensitive_cross_boundary: false,
            allowed_target_vendors: Some(allowed),
        };
        let v = trust_violations(&[ok, bad, unknown], &policy);
        let flagged: BTreeSet<&str> = v.iter().map(|x| x.task_id.as_str()).collect();
        assert!(flagged.contains("t-bad"));
        assert!(flagged.contains("t-unknown"));
        assert!(!flagged.contains("t-ok"));
        assert!(
            v.iter()
                .all(|x| x.kind == TrustViolationKind::VendorNotAllowed)
        );
    }

    #[test]
    fn attribute_chain_finds_downstream_culprit() {
        // parent t1 (trace-t1) delegates; its downstream trace is "ds1".
        // child t2 runs in "ds1" (parent_trace = ds1) and itself fails with
        // downstream "ds2"; grandchild t3 runs in "ds2" and times out (leaf).
        let mut t1 = task("t1", "a", "b");
        t1.parent_trace_id = trace("root");
        let mut t2 = task("t2", "b", "c");
        t2.parent_trace_id = trace("ds1");
        let mut t3 = task("t3", "c", "d");
        t3.parent_trace_id = trace("ds2");

        let mut r1 = result("t1", DelegationStatus::Failed);
        r1.downstream_trace_id = Some(trace("ds1"));
        let mut r2 = result("t2", DelegationStatus::Failed);
        r2.downstream_trace_id = Some(trace("ds2"));
        let mut r3 = result("t3", DelegationStatus::Timeout);
        r3.refusal_reason = Some("deadline exceeded".to_string());

        let tasks = vec![t1, t2, t3];
        let results = vec![r1, r2, r3];

        let attribution = attribute_chain_failure("t1", &tasks, &results)
            .unwrap_or_else(|| panic!("expected attribution"));
        assert_eq!(attribution.failing_task_id, "t3");
        assert_eq!(attribution.status, DelegationStatus::Timeout);
        assert_eq!(attribution.reason.as_deref(), Some("deadline exceeded"));
    }

    #[test]
    fn attribute_chain_returns_none_when_parent_succeeded() {
        let t1 = task("t1", "a", "b");
        let r1 = result("t1", DelegationStatus::Completed);
        assert!(attribute_chain_failure("t1", &[t1], &[r1]).is_none());
    }

    #[test]
    fn attribute_chain_blames_parent_when_no_failing_child() {
        let mut t1 = task("t1", "a", "b");
        t1.parent_trace_id = trace("root");
        let mut t2 = task("t2", "b", "c");
        t2.parent_trace_id = trace("ds1");

        let mut r1 = result("t1", DelegationStatus::Refused);
        r1.downstream_trace_id = Some(trace("ds1"));
        r1.refusal_reason = Some("policy".to_string());
        let r2 = result("t2", DelegationStatus::Completed);

        let attribution = attribute_chain_failure("t1", &[t1, t2], &[r1, r2])
            .unwrap_or_else(|| panic!("expected attribution"));
        assert_eq!(attribution.failing_task_id, "t1");
        assert_eq!(attribution.status, DelegationStatus::Refused);
    }

    #[test]
    fn span_attributes_contain_expected_keys() {
        let mut t = task("t1", "src-agent", "tgt-agent");
        t.protocol = Protocol::McpAsAgent;
        t.trust.same_tenant = false;
        t.trust.target_vendor = Some("globex".to_string());
        t.data_classes.insert(DataClass(RedactionClass::Sensitive));
        t.data_classes.insert(DataClass(RedactionClass::Public));

        let attrs = delegation_span_attributes(&t);
        assert_eq!(
            attrs.get(attr::SOURCE_ID),
            Some(&Value::String("src-agent".to_string()))
        );
        assert_eq!(
            attrs.get(attr::TARGET_ID),
            Some(&Value::String("tgt-agent".to_string()))
        );
        assert_eq!(
            attrs.get(attr::PROTOCOL),
            Some(&Value::String("mcp_as_agent".to_string()))
        );
        assert_eq!(
            attrs.get(attr::TRUST_SAME_TENANT),
            Some(&Value::Bool(false))
        );
        assert_eq!(
            attrs.get(attr::TRUST_CROSS_BOUNDARY),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            attrs.get(attr::TRUST_TARGET_VENDOR),
            Some(&Value::String("globex".to_string()))
        );
        // Sorted, comma-joined data classes.
        assert_eq!(
            attrs.get(attr::DATA_CLASSES),
            Some(&Value::String("public,sensitive".to_string()))
        );
        // Unknown optional fields are omitted.
        assert!(!attrs.contains_key(attr::TRUST_SOURCE_VENDOR));
        assert_eq!(DELEGATION_SPAN_KIND, "agent.delegation");
    }

    #[test]
    fn result_carries_artifacts_and_confidence() {
        let mut r = result("t1", DelegationStatus::Completed);
        r.artifacts.push(artifact());
        r.confidence = Some(0.9);
        r.tool_calls = 3;
        let json = serde_json::to_value(&r).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(json["status"], "completed");
        assert_eq!(json["tool_calls"], 3);
        assert_eq!(json["artifacts"][0]["mime_type"], "text/plain");
    }
}
