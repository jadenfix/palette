//! Temporal → Beater normalization.
//!
//! Converts a Temporal `GetWorkflowExecutionHistory` JSON document into Beater's
//! canonical span model, reusing the *exact* same downstream ingest pipeline as
//! the OTLP path (`RawTraceIngestRequest` → `IngestService::ingest_raw_trace_batch`).
//! Nothing here writes storage; it is a pure, deterministic mapping.
//!
//! ## Anti-drift contract
//! This crate is pinned to a specific Temporal history schema
//! ([`TEMPORAL_HISTORY_CONTRACT`]). Every Temporal `EventType` we target is listed
//! in [`KNOWN_EVENT_TYPES`] and classified explicitly by [`classify`] — there is no
//! silent wildcard that drops events. An event type that is not in the pinned set
//! is routed to [`EventClass::Unknown`], counted in [`ConversionStats::unmapped_events`],
//! and still preserved verbatim in the stored raw envelope. The exhaustiveness test
//! (`every_known_event_type_is_classified`) fails the build if a pinned type is left
//! unclassified, so a Temporal schema change surfaces as a loud test failure rather
//! than silent data loss. See `scripts/check-gate0-foundations.py::check_temporal_contract`.

use beater_core::{SpanId, TenantScope, Timestamp, TraceId};
use beater_ingest::{CanonicalSpanDraft, ImportError, RawTraceIngestRequest, SourceImporter};
use beater_schema::{
    AgentSpanKind, AuthContext, CanonicalAttrs, RedactionClass, SourceDialect, SpanStatus,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Pinned Temporal history schema this normalizer targets. Stamped onto every
/// Temporal-sourced span as its `normalizer_version`. Bump explicitly whenever the
/// converter changes how it reads history.
pub const TEMPORAL_HISTORY_CONTRACT: &str = "temporal.api.history.v1@1.27";

/// Source key used to select this importer on the unified import endpoint.
pub const TEMPORAL_HISTORY_SOURCE: &str = "temporal_history";

/// The pinned set of Temporal `EventType` names (canonical SCREAMING_SNAKE, with the
/// `EVENT_TYPE_` prefix stripped). This is the contract: [`classify`] must map every
/// entry to a non-[`EventClass::Unknown`] class. Adding support for a new Temporal
/// version means extending this list and `classify` together.
pub const KNOWN_EVENT_TYPES: &[&str] = &[
    "WORKFLOW_EXECUTION_STARTED",
    "WORKFLOW_EXECUTION_COMPLETED",
    "WORKFLOW_EXECUTION_FAILED",
    "WORKFLOW_EXECUTION_TIMED_OUT",
    "WORKFLOW_EXECUTION_CONTINUED_AS_NEW",
    "WORKFLOW_EXECUTION_CANCEL_REQUESTED",
    "WORKFLOW_EXECUTION_CANCELED",
    "WORKFLOW_EXECUTION_TERMINATED",
    "WORKFLOW_EXECUTION_SIGNALED",
    "WORKFLOW_TASK_SCHEDULED",
    "WORKFLOW_TASK_STARTED",
    "WORKFLOW_TASK_COMPLETED",
    "WORKFLOW_TASK_TIMED_OUT",
    "WORKFLOW_TASK_FAILED",
    "ACTIVITY_TASK_SCHEDULED",
    "ACTIVITY_TASK_STARTED",
    "ACTIVITY_TASK_COMPLETED",
    "ACTIVITY_TASK_FAILED",
    "ACTIVITY_TASK_TIMED_OUT",
    "ACTIVITY_TASK_CANCEL_REQUESTED",
    "ACTIVITY_TASK_CANCELED",
    "TIMER_STARTED",
    "TIMER_FIRED",
    "TIMER_CANCELED",
    "MARKER_RECORDED",
    "START_CHILD_WORKFLOW_EXECUTION_INITIATED",
    "START_CHILD_WORKFLOW_EXECUTION_FAILED",
    "CHILD_WORKFLOW_EXECUTION_STARTED",
    "CHILD_WORKFLOW_EXECUTION_COMPLETED",
    "CHILD_WORKFLOW_EXECUTION_FAILED",
    "CHILD_WORKFLOW_EXECUTION_TIMED_OUT",
    "CHILD_WORKFLOW_EXECUTION_CANCELED",
    "CHILD_WORKFLOW_EXECUTION_TERMINATED",
    "REQUEST_CANCEL_EXTERNAL_WORKFLOW_EXECUTION_INITIATED",
    "REQUEST_CANCEL_EXTERNAL_WORKFLOW_EXECUTION_FAILED",
    "EXTERNAL_WORKFLOW_EXECUTION_CANCEL_REQUESTED",
    "SIGNAL_EXTERNAL_WORKFLOW_EXECUTION_INITIATED",
    "SIGNAL_EXTERNAL_WORKFLOW_EXECUTION_FAILED",
    "EXTERNAL_WORKFLOW_EXECUTION_SIGNALED",
    "UPSERT_WORKFLOW_SEARCH_ATTRIBUTES",
    "WORKFLOW_PROPERTIES_MODIFIED",
    "WORKFLOW_PROPERTIES_MODIFIED_EXTERNALLY",
    "ACTIVITY_PROPERTIES_MODIFIED_EXTERNALLY",
    "WORKFLOW_EXECUTION_UPDATE_ADMITTED",
    "WORKFLOW_EXECUTION_UPDATE_ACCEPTED",
    "WORKFLOW_EXECUTION_UPDATE_REJECTED",
    "WORKFLOW_EXECUTION_UPDATE_COMPLETED",
    "NEXUS_OPERATION_SCHEDULED",
    "NEXUS_OPERATION_STARTED",
    "NEXUS_OPERATION_COMPLETED",
    "NEXUS_OPERATION_FAILED",
    "NEXUS_OPERATION_CANCELED",
    "NEXUS_OPERATION_TIMED_OUT",
    "NEXUS_OPERATION_CANCEL_REQUESTED",
];

#[derive(Debug, thiserror::Error)]
pub enum TemporalError {
    #[error("invalid temporal history json: {0}")]
    Json(String),
    #[error("temporal history is missing an events array")]
    MissingEvents,
    #[error("temporal history is missing a WorkflowExecutionStarted event")]
    MissingWorkflowStart,
    #[error("could not derive a trace id from temporal history (no run id found)")]
    MissingRunId,
    #[error("temporal history event is missing a valid positive eventId")]
    InvalidEventId,
    #[error("temporal history has a duplicate creating eventId: {event_id}")]
    DuplicateEventId { event_id: u64 },
    #[error("invalid identifier derived from temporal history: {0}")]
    Id(String),
}

pub type TemporalResult<T> = Result<T, TemporalError>;

impl From<serde_json::Error> for TemporalError {
    fn from(err: serde_json::Error) -> Self {
        TemporalError::Json(err.to_string())
    }
}

impl From<beater_core::IdError> for TemporalError {
    fn from(err: beater_core::IdError) -> Self {
        TemporalError::Id(err.to_string())
    }
}

/// Coarse classification of a Temporal history event. Each variant maps to a
/// deterministic effect on the canonical span tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventClass {
    /// Root of the trace.
    WorkflowStarted,
    /// Closes the root span with the given status.
    WorkflowTerminal(SpanStatus),
    /// Opens a child activity span (keyed by this event's id).
    ActivityScheduled,
    /// Annotates an in-flight activity span.
    ActivityStarted,
    /// Closes an activity span (referenced via `scheduledEventId`).
    ActivityClosed(SpanStatus),
    /// Opens a nested child-workflow span (keyed by this event's id).
    ChildInitiated,
    /// Annotates a started child-workflow span (referenced via `initiatedEventId`).
    ChildStarted,
    /// Closes a child-workflow span (referenced via `initiatedEventId`).
    ChildClosed(SpanStatus),
    /// Opens a timer span (keyed by this event's id).
    TimerStarted,
    /// Closes a timer span (referenced via `startedEventId`).
    TimerClosed(SpanStatus),
    /// Standalone point-in-time signal span.
    Signal,
    /// Standalone point-in-time marker span.
    Marker,
    /// Recognized but intentionally produces no span (workflow-task bookkeeping,
    /// update/search-attribute lifecycle, external cancel/signal plumbing, nexus).
    Structural,
    /// Not in the pinned contract: counted as unmapped, preserved in the raw envelope.
    Unknown,
}

/// Classify a canonical (SCREAMING_SNAKE, no `EVENT_TYPE_` prefix) Temporal event
/// type. The single `_ => Unknown` arm is explicit — Unknown events are counted and
/// preserved, never silently dropped.
pub fn classify(canonical_event_type: &str) -> EventClass {
    use EventClass::*;
    use SpanStatus::{Error, Ok, Unset};
    match canonical_event_type {
        "WORKFLOW_EXECUTION_STARTED" => WorkflowStarted,
        "WORKFLOW_EXECUTION_COMPLETED" | "WORKFLOW_EXECUTION_CONTINUED_AS_NEW" => {
            WorkflowTerminal(Ok)
        }
        "WORKFLOW_EXECUTION_FAILED"
        | "WORKFLOW_EXECUTION_TIMED_OUT"
        | "WORKFLOW_EXECUTION_TERMINATED"
        | "WORKFLOW_EXECUTION_CANCELED" => WorkflowTerminal(Error),

        "ACTIVITY_TASK_SCHEDULED" => ActivityScheduled,
        "ACTIVITY_TASK_STARTED" => ActivityStarted,
        "ACTIVITY_TASK_COMPLETED" => ActivityClosed(Ok),
        "ACTIVITY_TASK_FAILED" | "ACTIVITY_TASK_TIMED_OUT" | "ACTIVITY_TASK_CANCELED" => {
            ActivityClosed(Error)
        }

        "START_CHILD_WORKFLOW_EXECUTION_INITIATED" => ChildInitiated,
        "CHILD_WORKFLOW_EXECUTION_STARTED" => ChildStarted,
        "CHILD_WORKFLOW_EXECUTION_COMPLETED" => ChildClosed(Ok),
        "START_CHILD_WORKFLOW_EXECUTION_FAILED"
        | "CHILD_WORKFLOW_EXECUTION_FAILED"
        | "CHILD_WORKFLOW_EXECUTION_TIMED_OUT"
        | "CHILD_WORKFLOW_EXECUTION_CANCELED"
        | "CHILD_WORKFLOW_EXECUTION_TERMINATED" => ChildClosed(Error),

        "TIMER_STARTED" => TimerStarted,
        "TIMER_FIRED" => TimerClosed(Ok),
        "TIMER_CANCELED" => TimerClosed(Unset),

        "WORKFLOW_EXECUTION_SIGNALED" => Signal,
        "MARKER_RECORDED" => Marker,

        // Recognized, no span emitted (control-plane / bookkeeping events).
        "WORKFLOW_TASK_SCHEDULED"
        | "WORKFLOW_TASK_STARTED"
        | "WORKFLOW_TASK_COMPLETED"
        | "WORKFLOW_TASK_TIMED_OUT"
        | "WORKFLOW_TASK_FAILED"
        | "ACTIVITY_TASK_CANCEL_REQUESTED"
        | "WORKFLOW_EXECUTION_CANCEL_REQUESTED"
        | "UPSERT_WORKFLOW_SEARCH_ATTRIBUTES"
        | "WORKFLOW_PROPERTIES_MODIFIED"
        | "WORKFLOW_PROPERTIES_MODIFIED_EXTERNALLY"
        | "ACTIVITY_PROPERTIES_MODIFIED_EXTERNALLY"
        | "WORKFLOW_EXECUTION_UPDATE_ADMITTED"
        | "WORKFLOW_EXECUTION_UPDATE_ACCEPTED"
        | "WORKFLOW_EXECUTION_UPDATE_REJECTED"
        | "WORKFLOW_EXECUTION_UPDATE_COMPLETED"
        | "REQUEST_CANCEL_EXTERNAL_WORKFLOW_EXECUTION_INITIATED"
        | "REQUEST_CANCEL_EXTERNAL_WORKFLOW_EXECUTION_FAILED"
        | "EXTERNAL_WORKFLOW_EXECUTION_CANCEL_REQUESTED"
        | "SIGNAL_EXTERNAL_WORKFLOW_EXECUTION_INITIATED"
        | "SIGNAL_EXTERNAL_WORKFLOW_EXECUTION_FAILED"
        | "EXTERNAL_WORKFLOW_EXECUTION_SIGNALED"
        | "NEXUS_OPERATION_SCHEDULED"
        | "NEXUS_OPERATION_STARTED"
        | "NEXUS_OPERATION_COMPLETED"
        | "NEXUS_OPERATION_FAILED"
        | "NEXUS_OPERATION_CANCELED"
        | "NEXUS_OPERATION_TIMED_OUT"
        | "NEXUS_OPERATION_CANCEL_REQUESTED" => Structural,

        _ => Unknown,
    }
}

/// Per-conversion accounting used to prove no event was silently dropped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConversionStats {
    pub total_events: usize,
    pub mapped_events: usize,
    pub unmapped_events: usize,
    pub spans: usize,
}

/// Result of converting one Temporal history into canonical span drafts.
#[derive(Clone, Debug)]
pub struct ConvertedHistory {
    pub trace_id: TraceId,
    pub drafts: Vec<CanonicalSpanDraft>,
    pub stats: ConversionStats,
}

/// Importer that plugs Temporal history into the shared ingest pipeline.
#[derive(Clone, Copy, Debug, Default)]
pub struct TemporalHistoryImporter;

impl SourceImporter for TemporalHistoryImporter {
    fn source(&self) -> &'static str {
        TEMPORAL_HISTORY_SOURCE
    }

    fn normalize(
        &self,
        scope: &TenantScope,
        raw_bytes: &[u8],
        auth: Option<AuthContext>,
    ) -> Result<RawTraceIngestRequest, ImportError> {
        temporal_history_to_raw_ingest(scope.clone(), raw_bytes.to_vec(), auth).map_err(|err| {
            ImportError::Invalid {
                source_name: TEMPORAL_HISTORY_SOURCE.to_string(),
                message: err.to_string(),
            }
        })
    }
}

/// Convert raw Temporal history bytes into a `RawTraceIngestRequest` ready for
/// `IngestService::ingest_raw_trace_batch`. Mirrors
/// `beater_otlp::export_to_raw_trace_ingest_request`.
pub fn temporal_history_to_raw_ingest(
    scope: TenantScope,
    raw_bytes: Vec<u8>,
    auth: Option<AuthContext>,
) -> TemporalResult<RawTraceIngestRequest> {
    let history: Value = serde_json::from_slice(&raw_bytes)?;
    let converted = convert_history(&scope, &history)?;
    Ok(RawTraceIngestRequest {
        scope,
        source: SourceDialect::TemporalHistoryImport,
        source_schema_url: Some("https://docs.temporal.io/references/events".to_string()),
        source_schema_version: Some(TEMPORAL_HISTORY_CONTRACT.to_string()),
        normalizer_version: TEMPORAL_HISTORY_CONTRACT.to_string(),
        mime_type: "application/json".to_string(),
        redaction_class: RedactionClass::Internal,
        raw_bytes,
        raw_idempotency_key: None,
        auth_context: auth,
        spans: converted.drafts,
    })
}

/// Pure converter: Temporal history JSON → canonical span drafts + accounting.
pub fn convert_history(_scope: &TenantScope, history: &Value) -> TemporalResult<ConvertedHistory> {
    let events = events_array(history)?;
    let trace_id = derive_trace_id(history, events)?;

    // Builders keyed by the *creating* event id (workflow start, activity scheduled,
    // child initiated, timer started, signal, marker). Update events reference those
    // ids via scheduledEventId / initiatedEventId / startedEventId.
    let mut spans: BTreeMap<u64, DraftBuilder> = BTreeMap::new();
    let mut order: Vec<u64> = Vec::new();
    let mut root_key: Option<u64> = None;
    let mut root_span_id: Option<SpanId> = None;
    let mut stats = ConversionStats {
        total_events: events.len(),
        ..ConversionStats::default()
    };
    let mut unmapped_types: Vec<String> = Vec::new();

    for event in events {
        let canonical = normalize_event_type(event_type(event));
        let class = classify(&canonical);
        if class == EventClass::Unknown {
            stats.unmapped_events += 1;
            if !unmapped_types.contains(&canonical) {
                unmapped_types.push(canonical);
            }
            continue;
        }
        stats.mapped_events += 1;
        // Every real Temporal history event carries a positive, monotonic eventId.
        // Reject malformed input loudly instead of defaulting to 0 (which would collide
        // span keys / span_ids and silently drop spans).
        let eid = event_id(event).ok_or(TemporalError::InvalidEventId)?;
        let when = event_time(event);

        match class {
            EventClass::WorkflowStarted => {
                let attrs = attributes(event, "workflowExecutionStartedEventAttributes");
                let span_id = span_id_for(eid)?;
                root_span_id = Some(span_id.clone());
                root_key = Some(eid);
                let mut builder = DraftBuilder::new(span_id, None, eid, AgentSpanKind::AgentRun);
                builder.name = type_name(attrs, "workflowType")
                    .unwrap_or("workflow")
                    .to_string();
                builder.start_time = when;
                builder.input = payload_value(attrs, "input");
                builder.attr_str("temporal.kind", "workflow");
                if let Some(name) = type_name(attrs, "workflowType") {
                    builder.attr_str("temporal.workflow.type", name);
                }
                if let Some(tq) = name_field(attrs, "taskQueue") {
                    builder.attr_str("temporal.task_queue", tq);
                }
                builder.attr_str("temporal.run_id", trace_id.as_str());
                insert_builder(&mut spans, &mut order, eid, builder)?;
            }
            EventClass::WorkflowTerminal(status) => {
                if let Some(rk) = root_key {
                    if let Some(builder) = spans.get_mut(&rk) {
                        builder.end_time = when;
                        builder.status = status;
                        builder.output = terminal_output(event);
                    }
                }
            }
            EventClass::ActivityScheduled => {
                let attrs = attributes(event, "activityTaskScheduledEventAttributes");
                let mut builder = DraftBuilder::new(
                    span_id_for(eid)?,
                    root_span_id.clone(),
                    eid,
                    AgentSpanKind::ToolCall,
                );
                builder.name = type_name(attrs, "activityType")
                    .unwrap_or("activity")
                    .to_string();
                builder.start_time = when;
                builder.input = payload_value(attrs, "input");
                builder.attr_str("temporal.kind", "activity");
                if let Some(name) = type_name(attrs, "activityType") {
                    builder.attr_str("temporal.activity.type", name);
                }
                if let Some(id) = str_field(attrs, "activityId") {
                    builder.attr_str("temporal.activity.id", id);
                }
                if let Some(tq) = name_field(attrs, "taskQueue") {
                    builder.attr_str("temporal.task_queue", tq);
                }
                insert_builder(&mut spans, &mut order, eid, builder)?;
            }
            EventClass::ActivityStarted => {
                let attrs = attributes(event, "activityTaskStartedEventAttributes");
                if let Some(builder) = ref_builder(&mut spans, attrs, "scheduledEventId") {
                    if let Some(attempt) = attrs.get("attempt").and_then(as_u64) {
                        builder.attr_num("temporal.attempt", attempt);
                    }
                }
            }
            EventClass::ActivityClosed(status) => {
                let attrs = attributes(event, activity_close_key(&canonical));
                if let Some(builder) = ref_builder(&mut spans, attrs, "scheduledEventId") {
                    builder.end_time = when;
                    builder.status = status;
                    builder.output = closing_output(attrs);
                }
            }
            EventClass::ChildInitiated => {
                let attrs =
                    attributes(event, "startChildWorkflowExecutionInitiatedEventAttributes");
                let mut builder = DraftBuilder::new(
                    span_id_for(eid)?,
                    root_span_id.clone(),
                    eid,
                    AgentSpanKind::AgentRun,
                );
                builder.name = type_name(attrs, "workflowType")
                    .unwrap_or("child-workflow")
                    .to_string();
                builder.start_time = when;
                builder.input = payload_value(attrs, "input");
                builder.attr_str("temporal.kind", "child_workflow");
                if let Some(name) = type_name(attrs, "workflowType") {
                    builder.attr_str("temporal.workflow.type", name);
                }
                insert_builder(&mut spans, &mut order, eid, builder)?;
            }
            EventClass::ChildStarted => {
                let attrs = attributes(event, "childWorkflowExecutionStartedEventAttributes");
                if let Some(builder) = ref_builder(&mut spans, attrs, "initiatedEventId") {
                    if let Some(run_id) = attrs
                        .get("workflowExecution")
                        .and_then(|we| we.get("runId"))
                        .and_then(Value::as_str)
                    {
                        builder.attr_str("temporal.child.run_id", run_id);
                    }
                }
            }
            EventClass::ChildClosed(status) => {
                let attrs = attributes(event, child_close_key(&canonical));
                if let Some(builder) = ref_builder(&mut spans, attrs, "initiatedEventId") {
                    builder.end_time = when;
                    builder.status = status;
                    builder.output = closing_output(attrs);
                }
            }
            EventClass::TimerStarted => {
                let attrs = attributes(event, "timerStartedEventAttributes");
                let mut builder = DraftBuilder::new(
                    span_id_for(eid)?,
                    root_span_id.clone(),
                    eid,
                    AgentSpanKind::AgentStep,
                );
                let timer_id = str_field(attrs, "timerId").unwrap_or("timer");
                builder.name = format!("timer:{timer_id}");
                builder.start_time = when;
                builder.attr_str("temporal.kind", "timer");
                builder.attr_str("temporal.timer.id", timer_id);
                insert_builder(&mut spans, &mut order, eid, builder)?;
            }
            EventClass::TimerClosed(status) => {
                let attrs = attributes(event, timer_close_key(&canonical));
                if let Some(builder) = ref_builder(&mut spans, attrs, "startedEventId") {
                    builder.end_time = when;
                    if status != SpanStatus::Unset {
                        builder.status = status;
                    }
                }
            }
            EventClass::Signal => {
                let attrs = attributes(event, "workflowExecutionSignaledEventAttributes");
                let mut builder = DraftBuilder::new(
                    span_id_for(eid)?,
                    root_span_id.clone(),
                    eid,
                    AgentSpanKind::AgentStep,
                );
                let signal_name = str_field(attrs, "signalName").unwrap_or("signal");
                builder.name = format!("signal:{signal_name}");
                builder.start_time = when;
                builder.end_time = when;
                builder.status = SpanStatus::Ok;
                builder.input = payload_value(attrs, "input");
                builder.attr_str("temporal.kind", "signal");
                builder.attr_str("temporal.signal.name", signal_name);
                insert_builder(&mut spans, &mut order, eid, builder)?;
            }
            EventClass::Marker => {
                let attrs = attributes(event, "markerRecordedEventAttributes");
                let mut builder = DraftBuilder::new(
                    span_id_for(eid)?,
                    root_span_id.clone(),
                    eid,
                    AgentSpanKind::AgentStep,
                );
                let marker_name = str_field(attrs, "markerName").unwrap_or("marker");
                builder.name = format!("marker:{marker_name}");
                builder.start_time = when;
                builder.end_time = when;
                builder.status = SpanStatus::Ok;
                builder.attr_str("temporal.kind", "marker");
                builder.attr_str("temporal.marker.name", marker_name);
                insert_builder(&mut spans, &mut order, eid, builder)?;
            }
            EventClass::Structural => {}
            EventClass::Unknown => unreachable!("Unknown handled before match"),
        }
    }

    if root_key.is_none() {
        return Err(TemporalError::MissingWorkflowStart);
    }

    // Record unmapped coverage on the root span so it is visible without re-parsing raw.
    if !unmapped_types.is_empty() {
        if let Some(rk) = root_key {
            if let Some(builder) = spans.get_mut(&rk) {
                builder.attr_num(
                    "temporal.unmapped_event_count",
                    stats.unmapped_events as u64,
                );
                builder.attributes.insert(
                    "temporal.unmapped_event_types".to_string(),
                    json!(unmapped_types),
                );
            }
        }
    }

    let drafts: Vec<CanonicalSpanDraft> = order
        .into_iter()
        .filter_map(|key| spans.remove(&key))
        .map(|builder| builder.finish(trace_id.clone()))
        .collect();
    stats.spans = drafts.len();

    Ok(ConvertedHistory {
        trace_id,
        drafts,
        stats,
    })
}

struct DraftBuilder {
    span_id: SpanId,
    parent_span_id: Option<SpanId>,
    seq: u64,
    kind: AgentSpanKind,
    name: String,
    status: SpanStatus,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    input: Option<Value>,
    output: Option<Value>,
    attributes: CanonicalAttrs,
}

impl DraftBuilder {
    fn new(span_id: SpanId, parent_span_id: Option<SpanId>, seq: u64, kind: AgentSpanKind) -> Self {
        Self {
            span_id,
            parent_span_id,
            seq,
            kind,
            name: String::new(),
            status: SpanStatus::Unset,
            start_time: None,
            end_time: None,
            input: None,
            output: None,
            attributes: CanonicalAttrs::new(),
        }
    }

    fn attr_str(&mut self, key: &str, value: &str) {
        self.attributes
            .insert(key.to_string(), Value::String(value.to_string()));
    }

    fn attr_num(&mut self, key: &str, value: u64) {
        self.attributes.insert(key.to_string(), json!(value));
    }

    fn finish(self, trace_id: TraceId) -> CanonicalSpanDraft {
        // An absent start_time is back-filled to "now" downstream; for historical
        // imports that is always later than the recorded end_time and would yield a
        // negative duration. Anchor an unparseable start to the end instead.
        let start_time = self.start_time.or(self.end_time);
        CanonicalSpanDraft {
            trace_id,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            seq: self.seq,
            kind: self.kind,
            name: self.name,
            status: self.status,
            start_time,
            end_time: self.end_time,
            model: None,
            cost: None,
            tokens: None,
            input: self.input,
            output: self.output,
            attributes: self.attributes,
        }
    }
}

fn insert_builder(
    spans: &mut BTreeMap<u64, DraftBuilder>,
    order: &mut Vec<u64>,
    key: u64,
    builder: DraftBuilder,
) -> TemporalResult<()> {
    if spans.contains_key(&key) {
        // Two creating events sharing an eventId would otherwise silently overwrite
        // the first span. Real histories never do this; reject loudly.
        return Err(TemporalError::DuplicateEventId { event_id: key });
    }
    spans.insert(key, builder);
    order.push(key);
    Ok(())
}

fn ref_builder<'a>(
    spans: &'a mut BTreeMap<u64, DraftBuilder>,
    attrs: &Value,
    ref_field: &str,
) -> Option<&'a mut DraftBuilder> {
    let key = attrs.get(ref_field).and_then(as_u64)?;
    spans.get_mut(&key)
}

fn events_array(history: &Value) -> TemporalResult<&Vec<Value>> {
    history
        .get("history")
        .and_then(|h| h.get("events"))
        .or_else(|| history.get("events"))
        .and_then(Value::as_array)
        .ok_or(TemporalError::MissingEvents)
}

fn derive_trace_id(history: &Value, events: &[Value]) -> TemporalResult<TraceId> {
    // Prefer explicit run ids, then the WorkflowExecutionStarted attributes.
    let candidate = history
        .get("workflowExecution")
        .and_then(|we| we.get("runId"))
        .and_then(Value::as_str)
        .or_else(|| history.get("runId").and_then(Value::as_str))
        .map(str::to_string)
        .or_else(|| {
            events.iter().find_map(|event| {
                let attrs = attributes(event, "workflowExecutionStartedEventAttributes");
                attrs
                    .get("originalExecutionRunId")
                    .or_else(|| attrs.get("firstExecutionRunId"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
        })
        .ok_or(TemporalError::MissingRunId)?;
    Ok(TraceId::new(candidate)?)
}

fn span_id_for(event_id: u64) -> TemporalResult<SpanId> {
    Ok(SpanId::new(format!("event-{event_id}"))?)
}

fn event_id(event: &Value) -> Option<u64> {
    let id = event.get("eventId").and_then(as_u64)?;
    (id > 0).then_some(id)
}

fn event_type(event: &Value) -> &str {
    event.get("eventType").and_then(Value::as_str).unwrap_or("")
}

fn event_time(event: &Value) -> Option<Timestamp> {
    let raw = event.get("eventTime").and_then(Value::as_str)?;
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Normalize any Temporal `eventType` encoding (protojson `EVENT_TYPE_FOO_BAR`,
/// SCREAMING_SNAKE `FOO_BAR`, or PascalCase `FooBar`) to canonical `FOO_BAR`.
pub fn normalize_event_type(raw: &str) -> String {
    let stripped = raw.strip_prefix("EVENT_TYPE_").unwrap_or(raw);
    if stripped.contains('_') || stripped.chars().all(|c| !c.is_ascii_lowercase()) {
        return stripped.to_ascii_uppercase();
    }
    let mut out = String::with_capacity(stripped.len() + 8);
    for (index, ch) in stripped.chars().enumerate() {
        if ch.is_ascii_uppercase() && index != 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_uppercase());
    }
    out
}

fn attributes<'a>(event: &'a Value, expected_key: &str) -> &'a Value {
    static NULL: Value = Value::Null;
    if let Some(value) = event.get(expected_key) {
        return value;
    }
    if let Some(snake_key) = snake_case_event_attributes_key(expected_key) {
        if let Some(value) = event.get(&snake_key) {
            return value;
        }
    }
    // Encoding-robust fallback: any `*EventAttributes` field on the event.
    if let Some(object) = event.as_object() {
        for (key, value) in object {
            if key.ends_with("EventAttributes") {
                return value;
            }
        }
        for (key, value) in object {
            if key.ends_with("_event_attributes") {
                return value;
            }
        }
    }
    &NULL
}

fn snake_case_event_attributes_key(expected_key: &str) -> Option<String> {
    if !expected_key.ends_with("EventAttributes") {
        return None;
    }
    let mut out = String::with_capacity(expected_key.len() + 8);
    for (index, ch) in expected_key.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

fn type_name<'a>(attrs: &'a Value, field: &str) -> Option<&'a str> {
    attrs
        .get(field)
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
}

fn name_field<'a>(attrs: &'a Value, field: &str) -> Option<&'a str> {
    attrs
        .get(field)
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
}

fn str_field<'a>(attrs: &'a Value, field: &str) -> Option<&'a str> {
    attrs.get(field).and_then(Value::as_str)
}

fn payload_value(attrs: &Value, field: &str) -> Option<Value> {
    attrs.get(field).cloned()
}

fn closing_output(attrs: &Value) -> Option<Value> {
    attrs
        .get("result")
        .or_else(|| attrs.get("failure"))
        .cloned()
}

fn terminal_output(event: &Value) -> Option<Value> {
    let attrs = attributes(event, "");
    closing_output(attrs)
}

fn activity_close_key(canonical: &str) -> &'static str {
    match canonical {
        "ACTIVITY_TASK_COMPLETED" => "activityTaskCompletedEventAttributes",
        "ACTIVITY_TASK_FAILED" => "activityTaskFailedEventAttributes",
        "ACTIVITY_TASK_TIMED_OUT" => "activityTaskTimedOutEventAttributes",
        _ => "activityTaskCanceledEventAttributes",
    }
}

fn child_close_key(canonical: &str) -> &'static str {
    match canonical {
        "CHILD_WORKFLOW_EXECUTION_COMPLETED" => "childWorkflowExecutionCompletedEventAttributes",
        "CHILD_WORKFLOW_EXECUTION_FAILED" => "childWorkflowExecutionFailedEventAttributes",
        "CHILD_WORKFLOW_EXECUTION_TIMED_OUT" => "childWorkflowExecutionTimedOutEventAttributes",
        "CHILD_WORKFLOW_EXECUTION_CANCELED" => "childWorkflowExecutionCanceledEventAttributes",
        "CHILD_WORKFLOW_EXECUTION_TERMINATED" => "childWorkflowExecutionTerminatedEventAttributes",
        _ => "startChildWorkflowExecutionFailedEventAttributes",
    }
}

fn timer_close_key(canonical: &str) -> &'static str {
    match canonical {
        "TIMER_FIRED" => "timerFiredEventAttributes",
        _ => "timerCanceledEventAttributes",
    }
}

fn as_u64(value: &Value) -> Option<u64> {
    if let Some(n) = value.as_u64() {
        return Some(n);
    }
    value.as_str().and_then(|s| s.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> TenantScope {
        TenantScope::new(
            beater_core::TenantId::new("t1").unwrap_or_else(|e| panic!("{e}")),
            beater_core::ProjectId::new("p1").unwrap_or_else(|e| panic!("{e}")),
            beater_core::EnvironmentId::new("e1").unwrap_or_else(|e| panic!("{e}")),
        )
    }

    #[test]
    fn every_known_event_type_is_classified() {
        // Anti-drift guard: no pinned Temporal event type may fall through to Unknown.
        for name in KNOWN_EVENT_TYPES {
            assert_ne!(
                classify(name),
                EventClass::Unknown,
                "pinned event type {name} is not classified — update classify()"
            );
        }
    }

    #[test]
    fn pinned_set_has_no_duplicates() {
        let mut seen = std::collections::BTreeSet::new();
        for name in KNOWN_EVENT_TYPES {
            assert!(seen.insert(*name), "duplicate pinned event type {name}");
        }
    }

    #[test]
    fn normalizes_event_type_encodings() {
        assert_eq!(
            normalize_event_type("EVENT_TYPE_WORKFLOW_EXECUTION_STARTED"),
            "WORKFLOW_EXECUTION_STARTED"
        );
        assert_eq!(
            normalize_event_type("WorkflowExecutionStarted"),
            "WORKFLOW_EXECUTION_STARTED"
        );
        assert_eq!(
            normalize_event_type("ACTIVITY_TASK_COMPLETED"),
            "ACTIVITY_TASK_COMPLETED"
        );
        assert_eq!(normalize_event_type("TimedOut"), "TIMED_OUT");
    }

    fn sample_history() -> Value {
        json!({
            "events": [
                {
                    "eventId": "1",
                    "eventTime": "2026-06-23T00:00:00Z",
                    "eventType": "WorkflowExecutionStarted",
                    "workflowExecutionStartedEventAttributes": {
                        "workflowType": {"name": "OrderWorkflow"},
                        "taskQueue": {"name": "orders"},
                        "originalExecutionRunId": "run-abc",
                        "input": {"payloads": [{"data": "eyJvcmRlciI6IDF9"}]}
                    }
                },
                {"eventId": "2", "eventTime": "2026-06-23T00:00:00Z", "eventType": "WorkflowTaskScheduled",
                 "workflowTaskScheduledEventAttributes": {}},
                {
                    "eventId": "5",
                    "eventTime": "2026-06-23T00:00:01Z",
                    "eventType": "ActivityTaskScheduled",
                    "activityTaskScheduledEventAttributes": {
                        "activityId": "1",
                        "activityType": {"name": "ChargeCard"},
                        "taskQueue": {"name": "orders"},
                        "input": {"payloads": [{"data": "e30="}]}
                    }
                },
                {"eventId": "6", "eventTime": "2026-06-23T00:00:02Z", "eventType": "ActivityTaskStarted",
                 "activityTaskStartedEventAttributes": {"scheduledEventId": "5", "attempt": 1}},
                {
                    "eventId": "7",
                    "eventTime": "2026-06-23T00:00:03Z",
                    "eventType": "ActivityTaskCompleted",
                    "activityTaskCompletedEventAttributes": {
                        "scheduledEventId": "5",
                        "startedEventId": "6",
                        "result": {"payloads": [{"data": "eyJvayI6IHRydWV9"}]}
                    }
                },
                {
                    "eventId": "8",
                    "eventTime": "2026-06-23T00:00:04Z",
                    "eventType": "WorkflowExecutionCompleted",
                    "workflowExecutionCompletedEventAttributes": {
                        "result": {"payloads": [{"data": "eyJkb25lIjogdHJ1ZX0="}]}
                    }
                }
            ]
        })
    }

    #[test]
    fn converts_workflow_activity_tree() {
        let converted = convert_history(&scope(), &sample_history())
            .unwrap_or_else(|e| panic!("convert failed: {e}"));
        assert_eq!(converted.trace_id.as_str(), "run-abc");
        // 6 events; the WorkflowTaskScheduled and terminal events do not create spans.
        assert_eq!(converted.stats.total_events, 6);
        assert_eq!(converted.stats.unmapped_events, 0);
        // Two spans: the workflow root and one activity.
        assert_eq!(converted.drafts.len(), 2);

        let root = &converted.drafts[0];
        assert_eq!(root.kind, AgentSpanKind::AgentRun);
        assert_eq!(root.name, "OrderWorkflow");
        assert_eq!(root.parent_span_id, None);
        assert_eq!(root.seq, 1);
        assert_eq!(root.status, SpanStatus::Ok);
        assert!(root.end_time.is_some());

        let activity = &converted.drafts[1];
        assert_eq!(activity.kind, AgentSpanKind::ToolCall);
        assert_eq!(activity.name, "ChargeCard");
        assert_eq!(activity.parent_span_id, Some(root.span_id.clone()));
        assert_eq!(activity.seq, 5);
        assert_eq!(activity.status, SpanStatus::Ok);
        assert!(activity.start_time.is_some());
        assert!(activity.end_time.is_some());
        assert!(activity.output.is_some());
    }

    #[test]
    fn unknown_events_are_counted_not_dropped() {
        let mut history = sample_history();
        let events = history
            .get_mut("events")
            .and_then(Value::as_array_mut)
            .unwrap_or_else(|| panic!("events array"));
        events.push(json!({
            "eventId": "9",
            "eventTime": "2026-06-23T00:00:05Z",
            "eventType": "SomeFutureTemporalEvent",
            "someFutureEventAttributes": {}
        }));
        let converted =
            convert_history(&scope(), &history).unwrap_or_else(|e| panic!("convert failed: {e}"));
        assert_eq!(converted.stats.unmapped_events, 1);
        assert_eq!(converted.stats.total_events, 7);
        // Still produces the same two spans; nothing silently breaks.
        assert_eq!(converted.drafts.len(), 2);
        let root = &converted.drafts[0];
        assert_eq!(
            root.attributes.get("temporal.unmapped_event_count"),
            Some(&json!(1u64))
        );
        assert_eq!(
            root.attributes.get("temporal.unmapped_event_types"),
            Some(&json!(["SOME_FUTURE_TEMPORAL_EVENT"]))
        );
    }

    #[test]
    fn missing_run_id_is_rejected() {
        let history = json!({
            "events": [
                {"eventId": "1", "eventTime": "2026-06-23T00:00:00Z",
                 "eventType": "WorkflowExecutionStarted",
                 "workflowExecutionStartedEventAttributes": {"workflowType": {"name": "W"}}}
            ]
        });
        let err = convert_history(&scope(), &history).err();
        assert!(matches!(err, Some(TemporalError::MissingRunId)));
    }

    #[test]
    fn builds_raw_ingest_request_with_pinned_normalizer() {
        let bytes = serde_json::to_vec(&sample_history()).unwrap_or_else(|e| panic!("{e}"));
        let request =
            temporal_history_to_raw_ingest(scope(), bytes, None).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(request.source, SourceDialect::TemporalHistoryImport);
        assert_eq!(request.normalizer_version, TEMPORAL_HISTORY_CONTRACT);
        assert_eq!(request.spans.len(), 2);
    }

    // ---- Failure-path and edge-case coverage -----------------------------------

    fn history(events: Vec<Value>) -> Value {
        json!({ "events": events })
    }

    fn started(eid: u64, run: &str) -> Value {
        json!({
            "eventId": eid.to_string(),
            "eventTime": "2026-06-23T00:00:00Z",
            "eventType": "WorkflowExecutionStarted",
            "workflowExecutionStartedEventAttributes": {
                "workflowType": {"name": "W"},
                "taskQueue": {"name": "q"},
                "originalExecutionRunId": run
            }
        })
    }

    fn activity_scheduled(eid: u64, name: &str) -> Value {
        json!({
            "eventId": eid.to_string(),
            "eventTime": "2026-06-23T00:00:01Z",
            "eventType": "ActivityTaskScheduled",
            "activityTaskScheduledEventAttributes": {
                "activityId": "a",
                "activityType": {"name": name},
                "taskQueue": {"name": "q"}
            }
        })
    }

    fn event(eid: u64, etype: &str, attrs_key: &str, attrs: Value) -> Value {
        json!({
            "eventId": eid.to_string(),
            "eventTime": "2026-06-23T00:00:05Z",
            "eventType": etype,
            attrs_key: attrs
        })
    }

    fn convert_ok(events: Vec<Value>) -> ConvertedHistory {
        convert_history(&scope(), &history(events)).unwrap_or_else(|e| panic!("convert: {e}"))
    }

    fn find<'a>(c: &'a ConvertedHistory, name: &str) -> &'a CanonicalSpanDraft {
        c.drafts
            .iter()
            .find(|d| d.name == name)
            .unwrap_or_else(|| panic!("missing span {name}"))
    }

    #[test]
    fn activity_failure_marks_error_span_but_keeps_others() {
        let c = convert_ok(vec![
            started(1, "r1"),
            activity_scheduled(5, "Flaky"),
            activity_scheduled(8, "Healthy"),
            event(
                9,
                "ActivityTaskFailed",
                "activityTaskFailedEventAttributes",
                json!({"scheduledEventId": "5", "failure": {"message": "boom"}}),
            ),
            event(
                10,
                "ActivityTaskCompleted",
                "activityTaskCompletedEventAttributes",
                json!({"scheduledEventId": "8", "result": {"payloads": []}}),
            ),
            event(
                11,
                "WorkflowExecutionCompleted",
                "workflowExecutionCompletedEventAttributes",
                json!({"result": {"payloads": []}}),
            ),
        ]);
        let flaky = find(&c, "Flaky");
        assert_eq!(flaky.status, SpanStatus::Error);
        assert_eq!(flaky.seq, 5);
        assert!(flaky.output.is_some(), "failure detail captured as output");
        let healthy = find(&c, "Healthy");
        assert_eq!(healthy.status, SpanStatus::Ok);
        // Workflow still completed Ok even though one activity failed.
        assert_eq!(c.drafts[0].status, SpanStatus::Ok);
        assert_eq!(c.stats.unmapped_events, 0);
    }

    #[test]
    fn workflow_failure_timeout_terminate_cancel_all_mark_root_error() {
        for (etype, key) in [
            (
                "WorkflowExecutionFailed",
                "workflowExecutionFailedEventAttributes",
            ),
            (
                "WorkflowExecutionTimedOut",
                "workflowExecutionTimedOutEventAttributes",
            ),
            (
                "WorkflowExecutionTerminated",
                "workflowExecutionTerminatedEventAttributes",
            ),
            (
                "WorkflowExecutionCanceled",
                "workflowExecutionCanceledEventAttributes",
            ),
        ] {
            let c = convert_ok(vec![
                started(1, "r1"),
                event(2, etype, key, json!({"failure": {"message": "x"}})),
            ]);
            assert_eq!(
                c.drafts[0].status,
                SpanStatus::Error,
                "{etype} should mark root error"
            );
            assert!(c.drafts[0].end_time.is_some());
        }
    }

    #[test]
    fn activity_timeout_and_cancel_mark_error() {
        for (etype, key) in [
            (
                "ActivityTaskTimedOut",
                "activityTaskTimedOutEventAttributes",
            ),
            (
                "ActivityTaskCanceled",
                "activityTaskCanceledEventAttributes",
            ),
        ] {
            let c = convert_ok(vec![
                started(1, "r1"),
                activity_scheduled(5, "A"),
                event(7, etype, key, json!({"scheduledEventId": "5"})),
            ]);
            assert_eq!(find(&c, "A").status, SpanStatus::Error, "{etype}");
        }
    }

    #[test]
    fn open_activity_has_no_end_and_unset_status() {
        // Scheduled but never closed (workflow still running / history truncated).
        let c = convert_ok(vec![started(1, "r1"), activity_scheduled(5, "Pending")]);
        let pending = find(&c, "Pending");
        assert_eq!(pending.status, SpanStatus::Unset);
        assert_eq!(pending.end_time, None);
        assert!(pending.start_time.is_some());
    }

    #[test]
    fn child_workflow_failure_marks_error() {
        let c = convert_ok(vec![
            started(1, "r1"),
            event(
                5,
                "StartChildWorkflowExecutionInitiated",
                "startChildWorkflowExecutionInitiatedEventAttributes",
                json!({"workflowType": {"name": "Child"}}),
            ),
            event(
                6,
                "ChildWorkflowExecutionFailed",
                "childWorkflowExecutionFailedEventAttributes",
                json!({"initiatedEventId": "5", "failure": {"message": "child boom"}}),
            ),
        ]);
        let child = find(&c, "Child");
        assert_eq!(child.kind, AgentSpanKind::AgentRun);
        assert_eq!(child.status, SpanStatus::Error);
    }

    #[test]
    fn start_child_workflow_failed_marks_error() {
        let c = convert_ok(vec![
            started(1, "r1"),
            event(
                5,
                "StartChildWorkflowExecutionInitiated",
                "startChildWorkflowExecutionInitiatedEventAttributes",
                json!({"workflowType": {"name": "Child"}}),
            ),
            event(
                6,
                "StartChildWorkflowExecutionFailed",
                "startChildWorkflowExecutionFailedEventAttributes",
                json!({"initiatedEventId": "5", "cause": "WORKFLOW_ALREADY_EXISTS"}),
            ),
        ]);
        assert_eq!(find(&c, "Child").status, SpanStatus::Error);
    }

    #[test]
    fn continued_as_new_is_ok_terminal() {
        let c = convert_ok(vec![
            started(1, "r1"),
            event(
                2,
                "WorkflowExecutionContinuedAsNew",
                "workflowExecutionContinuedAsNewEventAttributes",
                json!({"newExecutionRunId": "r2"}),
            ),
        ]);
        assert_eq!(c.drafts[0].status, SpanStatus::Ok);
        assert!(c.drafts[0].end_time.is_some());
    }

    #[test]
    fn retried_activity_records_attempt() {
        let c = convert_ok(vec![
            started(1, "r1"),
            activity_scheduled(5, "Retrying"),
            event(
                6,
                "ActivityTaskStarted",
                "activityTaskStartedEventAttributes",
                json!({"scheduledEventId": "5", "attempt": 3}),
            ),
            event(
                7,
                "ActivityTaskCompleted",
                "activityTaskCompletedEventAttributes",
                json!({"scheduledEventId": "5", "result": {"payloads": []}}),
            ),
        ]);
        let span = find(&c, "Retrying");
        assert_eq!(span.attributes.get("temporal.attempt"), Some(&json!(3u64)));
        assert_eq!(span.status, SpanStatus::Ok);
    }

    #[test]
    fn many_activities_preserve_seq_order() {
        let mut events = vec![started(1, "r1")];
        for i in 0..10u64 {
            let sched = 5 + i * 2;
            events.push(activity_scheduled(sched, &format!("Act{i}")));
            events.push(event(
                sched + 1,
                "ActivityTaskCompleted",
                "activityTaskCompletedEventAttributes",
                json!({"scheduledEventId": sched.to_string(), "result": {"payloads": []}}),
            ));
        }
        let c = convert_ok(events);
        assert_eq!(c.drafts.len(), 11); // root + 10 activities
                                        // seq strictly increasing in draft order (root first at seq 1).
        let seqs: Vec<u64> = c.drafts.iter().map(|d| d.seq).collect();
        let mut sorted = seqs.clone();
        sorted.sort_unstable();
        assert_eq!(seqs, sorted, "spans emitted in event-id order");
        assert_eq!(seqs[0], 1);
    }

    #[test]
    fn wrapped_history_form_supported() {
        let wrapped = json!({"history": {"events": [started(1, "wrapped-run")]}});
        let c = convert_history(&scope(), &wrapped).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(c.trace_id.as_str(), "wrapped-run");
        assert_eq!(c.drafts.len(), 1);
    }

    #[test]
    fn pascal_case_encoding_fully_converts() {
        // Some tooling emits PascalCase eventType instead of EVENT_TYPE_* protojson.
        let c = convert_ok(vec![
            started(1, "r1"),
            activity_scheduled(5, "A"),
            event(
                6,
                "ActivityTaskCompleted",
                "activityTaskCompletedEventAttributes",
                json!({"scheduledEventId": "5", "result": {"payloads": []}}),
            ),
        ]);
        assert_eq!(c.stats.unmapped_events, 0);
        assert_eq!(find(&c, "A").status, SpanStatus::Ok);
    }

    #[test]
    fn snake_case_event_attribute_keys_convert_workflow_activity_tree() {
        let c = convert_ok(vec![
            json!({
                "eventId": "1",
                "eventTime": "2026-06-23T00:00:00Z",
                "eventType": "WorkflowExecutionStarted",
                "workflow_execution_started_event_attributes": {
                    "workflowType": {"name": "SnakeWorkflow"},
                    "taskQueue": {"name": "snake-q"},
                    "originalExecutionRunId": "snake-run"
                }
            }),
            json!({
                "eventId": "5",
                "eventTime": "2026-06-23T00:00:01Z",
                "eventType": "ActivityTaskScheduled",
                "activity_task_scheduled_event_attributes": {
                    "activityId": "snake-activity-id",
                    "activityType": {"name": "SnakeActivity"},
                    "taskQueue": {"name": "snake-q"}
                }
            }),
            json!({
                "eventId": "7",
                "eventTime": "2026-06-23T00:00:03Z",
                "eventType": "ActivityTaskCompleted",
                "activity_task_completed_event_attributes": {
                    "scheduledEventId": "5",
                    "result": {"payloads": []}
                }
            }),
            json!({
                "eventId": "8",
                "eventTime": "2026-06-23T00:00:04Z",
                "eventType": "WorkflowExecutionCompleted",
                "workflow_execution_completed_event_attributes": {
                    "result": {"payloads": []}
                }
            }),
        ]);

        assert_eq!(c.trace_id.as_str(), "snake-run");
        assert_eq!(c.stats.unmapped_events, 0);
        assert_eq!(c.drafts.len(), 2);

        let root = &c.drafts[0];
        assert_eq!(root.name, "SnakeWorkflow");
        assert_eq!(root.parent_span_id, None);
        assert_eq!(root.seq, 1);
        assert_eq!(root.status, SpanStatus::Ok);
        assert_eq!(
            root.attributes.get("temporal.workflow.type"),
            Some(&json!("SnakeWorkflow"))
        );

        let activity = &c.drafts[1];
        assert_eq!(activity.name, "SnakeActivity");
        assert_eq!(activity.parent_span_id, Some(root.span_id.clone()));
        assert_eq!(activity.seq, 5);
        assert_eq!(activity.status, SpanStatus::Ok);
        assert_eq!(
            activity.attributes.get("temporal.activity.type"),
            Some(&json!("SnakeActivity"))
        );
        assert!(activity.output.is_some());
    }

    #[test]
    fn exact_lower_camel_attribute_key_wins_over_snake_case_alias() {
        let c = convert_ok(vec![json!({
            "eventId": "1",
            "eventTime": "2026-06-23T00:00:00Z",
            "eventType": "WorkflowExecutionStarted",
            "workflowExecutionStartedEventAttributes": {
                "workflowType": {"name": "LowerCamelWorkflow"},
                "originalExecutionRunId": "lower-camel-run"
            },
            "workflow_execution_started_event_attributes": {
                "workflowType": {"name": "SnakeWorkflow"},
                "originalExecutionRunId": "snake-run"
            }
        })]);

        assert_eq!(c.trace_id.as_str(), "lower-camel-run");
        assert_eq!(c.drafts[0].name, "LowerCamelWorkflow");
    }

    #[test]
    fn malformed_json_is_rejected() {
        let err = temporal_history_to_raw_ingest(scope(), b"{not json".to_vec(), None).err();
        assert!(matches!(err, Some(TemporalError::Json(_))));
    }

    #[test]
    fn missing_events_array_is_rejected() {
        let err = convert_history(&scope(), &json!({"foo": 1})).err();
        assert!(matches!(err, Some(TemporalError::MissingEvents)));
    }

    #[test]
    fn empty_events_reports_missing_run_id() {
        // No events → no derivable run id, rejected before span construction.
        let err = convert_history(&scope(), &json!({"events": []})).err();
        assert!(matches!(err, Some(TemporalError::MissingRunId)));
    }

    #[test]
    fn run_id_present_but_no_workflow_start_is_rejected() {
        // A top-level run id makes the trace id derivable, but with no
        // WorkflowExecutionStarted there is no root span → MissingWorkflowStart.
        let h = json!({
            "workflowExecution": {"runId": "r1"},
            "events": [activity_scheduled(5, "A")]
        });
        let err = convert_history(&scope(), &h).err();
        assert!(matches!(err, Some(TemporalError::MissingWorkflowStart)));
    }

    #[test]
    fn events_without_workflow_start_or_run_id_are_rejected() {
        // Activity events but no WorkflowExecutionStarted and no run id → rejected early.
        let err = convert_history(&scope(), &history(vec![activity_scheduled(5, "A")])).err();
        assert!(matches!(err, Some(TemporalError::MissingRunId)));
    }

    #[test]
    fn signal_and_marker_become_point_spans() {
        let c = convert_ok(vec![
            started(1, "r1"),
            event(
                5,
                "WorkflowExecutionSignaled",
                "workflowExecutionSignaledEventAttributes",
                json!({"signalName": "cancelOrder", "input": {"payloads": []}}),
            ),
            event(
                6,
                "MarkerRecorded",
                "markerRecordedEventAttributes",
                json!({"markerName": "Version"}),
            ),
        ]);
        let signal = find(&c, "signal:cancelOrder");
        assert_eq!(signal.kind, AgentSpanKind::AgentStep);
        assert_eq!(signal.start_time, signal.end_time);
        let marker = find(&c, "marker:Version");
        assert_eq!(marker.kind, AgentSpanKind::AgentStep);
    }

    #[test]
    fn input_and_output_payloads_are_captured() {
        let c = convert_ok(vec![
            json!({
                "eventId": "1",
                "eventTime": "2026-06-23T00:00:00Z",
                "eventType": "WorkflowExecutionStarted",
                "workflowExecutionStartedEventAttributes": {
                    "workflowType": {"name": "W"},
                    "originalExecutionRunId": "r1",
                    "input": {"payloads": [{"data": "aGk="}]}
                }
            }),
            event(
                2,
                "WorkflowExecutionCompleted",
                "workflowExecutionCompletedEventAttributes",
                json!({"result": {"payloads": [{"data": "Ynll"}]}}),
            ),
        ]);
        let root = &c.drafts[0];
        assert!(root.input.is_some(), "workflow input captured");
        assert!(root.output.is_some(), "workflow result captured");
    }

    #[test]
    fn unmapped_events_recorded_on_root_and_counted() {
        let c = convert_ok(vec![
            started(1, "r1"),
            event(
                2,
                "BrandNewTemporalEvent",
                "brandNewEventAttributes",
                json!({}),
            ),
            event(3, "AnotherUnknown", "anotherEventAttributes", json!({})),
            event(
                4,
                "WorkflowExecutionCompleted",
                "workflowExecutionCompletedEventAttributes",
                json!({}),
            ),
        ]);
        assert_eq!(c.stats.unmapped_events, 2);
        assert_eq!(c.stats.total_events, 4);
        assert_eq!(
            c.drafts[0].attributes.get("temporal.unmapped_event_count"),
            Some(&json!(2u64))
        );
        // The unknown event TYPES are surfaced for visibility.
        let types = c.drafts[0]
            .attributes
            .get("temporal.unmapped_event_types")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("unmapped types recorded"));
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn structural_events_emit_no_span() {
        // Pure control-plane history (workflow tasks only) → just the root span.
        let c = convert_ok(vec![
            started(1, "r1"),
            event(
                2,
                "WorkflowTaskScheduled",
                "workflowTaskScheduledEventAttributes",
                json!({}),
            ),
            event(
                3,
                "WorkflowTaskStarted",
                "workflowTaskStartedEventAttributes",
                json!({"scheduledEventId": "2"}),
            ),
            event(
                4,
                "WorkflowTaskCompleted",
                "workflowTaskCompletedEventAttributes",
                json!({"scheduledEventId": "2", "startedEventId": "3"}),
            ),
            event(
                5,
                "UpsertWorkflowSearchAttributes",
                "upsertWorkflowSearchAttributesEventAttributes",
                json!({}),
            ),
        ]);
        assert_eq!(c.drafts.len(), 1);
        assert_eq!(c.stats.unmapped_events, 0);
    }

    #[test]
    fn missing_or_zero_event_id_is_rejected_not_collapsed() {
        // A mapped event with no usable eventId must error, not silently default to 0
        // (which would collide span keys and drop spans).
        for bad in [json!("not-a-number"), json!(0), Value::Null] {
            let started = json!({
                "eventId": bad,
                "eventTime": "2026-06-23T00:00:00Z",
                "eventType": "WorkflowExecutionStarted",
                "workflowExecutionStartedEventAttributes": {
                    "workflowType": {"name": "W"}, "originalExecutionRunId": "r1"
                }
            });
            let err = convert_history(&scope(), &history(vec![started])).err();
            assert!(
                matches!(err, Some(TemporalError::InvalidEventId)),
                "eventId {bad:?} should be rejected, got {err:?}"
            );
        }
    }

    #[test]
    fn duplicate_creating_event_id_is_rejected_not_overwritten() {
        // Two creating events sharing an eventId must error rather than silently
        // overwriting the first span.
        let c = convert_history(
            &scope(),
            &history(vec![
                started(1, "r1"),
                activity_scheduled(5, "First"),
                activity_scheduled(5, "Second"), // duplicate creating id 5
            ]),
        );
        assert!(matches!(
            c.err(),
            Some(TemporalError::DuplicateEventId { event_id: 5 })
        ));
    }

    #[test]
    fn missing_start_time_does_not_produce_negative_duration() {
        // Activity with no parseable scheduled time but a real completed time: the
        // start must anchor to the end, never leave a None that back-fills to "now".
        let c = convert_ok(vec![
            started(1, "r1"),
            json!({
                "eventId": "5",
                "eventType": "ActivityTaskScheduled",
                "activityTaskScheduledEventAttributes": {
                    "activityType": {"name": "NoStartTime"}, "taskQueue": {"name": "q"}
                }
            }),
            event(
                7,
                "ActivityTaskCompleted",
                "activityTaskCompletedEventAttributes",
                json!({"scheduledEventId": "5", "result": {"payloads": []}}),
            ),
        ]);
        let span = find(&c, "NoStartTime");
        assert!(span.end_time.is_some());
        assert_eq!(
            span.start_time, span.end_time,
            "missing start anchored to end (non-negative duration)"
        );
    }
}
