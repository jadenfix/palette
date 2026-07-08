//! Golden-fixture conformance test. Drives the converter against an unmodified
//! Temporal `GetWorkflowExecutionHistory` document (protojson `EVENT_TYPE_*` encoding)
//! and asserts the reconstructed span tree plus the no-drop accounting invariant.

use beater_core::{EnvironmentId, ProjectId, TenantId, TenantScope};
use beater_schema::{AgentSpanKind, SpanStatus};
use beater_temporal::{ConvertedHistory, convert_history, temporal_history_to_raw_ingest};
use serde_json::{Value, json};

const HISTORY: &str = include_str!("fixtures/order_workflow_history.json");

fn scope() -> TenantScope {
    TenantScope::new(
        TenantId::new("acme").unwrap_or_else(|e| panic!("{e}")),
        ProjectId::new("orders").unwrap_or_else(|e| panic!("{e}")),
        EnvironmentId::new("prod").unwrap_or_else(|e| panic!("{e}")),
    )
}

fn convert() -> ConvertedHistory {
    let history = fixture_history();
    convert_history(&scope(), &history).unwrap_or_else(|e| panic!("convert: {e}"))
}

fn fixture_history() -> Value {
    serde_json::from_str(HISTORY).unwrap_or_else(|e| panic!("fixture json: {e}"))
}

#[test]
fn reconstructs_workflow_span_tree() {
    let converted = convert();
    assert_eq!(
        converted.trace_id.as_str(),
        "11111111-1111-1111-1111-111111111111"
    );

    // Spans: workflow root, activity, timer, child workflow, signal = 5.
    assert_eq!(converted.drafts.len(), 5);

    let by_name = |needle: &str| {
        converted
            .drafts
            .iter()
            .find(|d| d.name.contains(needle))
            .unwrap_or_else(|| panic!("missing span {needle}"))
    };

    let root = &converted.drafts[0];
    assert_eq!(root.kind, AgentSpanKind::AgentRun);
    assert_eq!(root.name, "OrderWorkflow");
    assert_eq!(root.parent_span_id, None);
    assert_eq!(root.status, SpanStatus::Ok);
    assert!(root.end_time.is_some());

    let activity = by_name("ValidateOrder");
    assert_eq!(activity.kind, AgentSpanKind::ToolCall);
    assert_eq!(activity.parent_span_id, Some(root.span_id.clone()));
    assert_eq!(activity.status, SpanStatus::Ok);
    assert!(activity.output.is_some());

    let child = by_name("FulfillmentWorkflow");
    assert_eq!(child.kind, AgentSpanKind::AgentRun);
    assert_eq!(child.parent_span_id, Some(root.span_id.clone()));
    assert_eq!(child.status, SpanStatus::Ok);

    let timer = by_name("timer:settle-delay");
    assert_eq!(timer.kind, AgentSpanKind::AgentStep);
    assert!(timer.end_time.is_some());

    let signal = by_name("signal:addNote");
    assert_eq!(signal.kind, AgentSpanKind::AgentStep);

    // Every span is correctly parented under the workflow root.
    for draft in converted.drafts.iter().skip(1) {
        assert_eq!(
            draft.parent_span_id,
            Some(root.span_id.clone()),
            "span {} should be parented under the workflow root",
            draft.name
        );
    }
}

#[test]
fn no_event_is_silently_dropped() {
    let converted = convert();
    // The golden fixture is fully covered by the pinned contract.
    assert_eq!(converted.stats.unmapped_events, 0);
    // Accounting invariant: every input event is either mapped or counted as unmapped.
    assert_eq!(
        converted.stats.mapped_events + converted.stats.unmapped_events,
        converted.stats.total_events
    );
    assert_eq!(converted.stats.total_events, 14);
}

#[test]
fn unknown_event_is_counted_and_raw_preserved_without_breaking_known_conversion() {
    let mut history = fixture_history();
    history
        .get_mut("events")
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("events array"))
        .push(json!({
            "eventId": "15",
            "eventTime": "2026-06-23T12:00:43Z",
            "eventType": "EVENT_TYPE_WORKFLOW_EXECUTION_PATCHED",
            "workflowExecutionPatchedEventAttributes": {
                "patchId": "temporal-1.28-new-event",
                "details": {"still": "available only in raw"}
            }
        }));

    let converted = convert_history(&scope(), &history).unwrap_or_else(|e| panic!("convert: {e}"));
    assert_eq!(converted.stats.total_events, 15);
    assert_eq!(converted.stats.mapped_events, 14);
    assert_eq!(converted.stats.unmapped_events, 1);
    assert_eq!(
        converted.stats.mapped_events + converted.stats.unmapped_events,
        converted.stats.total_events
    );

    // Known events still reconstruct the golden span tree.
    assert_eq!(
        converted
            .drafts
            .iter()
            .map(|draft| draft.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "OrderWorkflow",
            "ValidateOrder",
            "timer:settle-delay",
            "FulfillmentWorkflow",
            "signal:addNote",
        ]
    );

    let root = &converted.drafts[0];
    assert_eq!(root.status, SpanStatus::Ok);
    assert_eq!(
        root.attributes.get("temporal.unmapped_event_count"),
        Some(&json!(1u64))
    );
    assert_eq!(
        root.attributes.get("temporal.unmapped_event_types"),
        Some(&json!(["WORKFLOW_EXECUTION_PATCHED"]))
    );

    let raw_bytes = serde_json::to_vec(&history).unwrap_or_else(|e| panic!("history json: {e}"));
    let request = temporal_history_to_raw_ingest(scope(), raw_bytes.clone(), None)
        .unwrap_or_else(|e| panic!("raw ingest: {e}"));
    assert_eq!(request.raw_bytes, raw_bytes);
    assert_eq!(request.spans.len(), 5);
    assert_eq!(
        request.spans[0]
            .attributes
            .get("temporal.unmapped_event_count"),
        Some(&json!(1u64))
    );

    let preserved_raw: Value =
        serde_json::from_slice(&request.raw_bytes).unwrap_or_else(|e| panic!("raw json: {e}"));
    let raw_unknown = preserved_raw
        .get("events")
        .and_then(Value::as_array)
        .and_then(|events| {
            events.iter().find(|event| {
                event.get("eventType").and_then(Value::as_str)
                    == Some("EVENT_TYPE_WORKFLOW_EXECUTION_PATCHED")
            })
        })
        .unwrap_or_else(|| panic!("unknown event preserved in raw bytes"));
    assert_eq!(
        raw_unknown
            .get("workflowExecutionPatchedEventAttributes")
            .and_then(|attrs| attrs.get("patchId"))
            .and_then(Value::as_str),
        Some("temporal-1.28-new-event")
    );
}
