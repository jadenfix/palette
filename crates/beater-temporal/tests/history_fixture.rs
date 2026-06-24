//! Golden-fixture conformance test. Drives the converter against an unmodified
//! Temporal `GetWorkflowExecutionHistory` document (protojson `EVENT_TYPE_*` encoding)
//! and asserts the reconstructed span tree plus the no-drop accounting invariant.

use beater_core::{EnvironmentId, ProjectId, TenantId, TenantScope};
use beater_schema::{AgentSpanKind, SpanStatus};
use beater_temporal::{convert_history, ConvertedHistory};

const HISTORY: &str = include_str!("fixtures/order_workflow_history.json");

fn scope() -> TenantScope {
    TenantScope::new(
        TenantId::new("acme").unwrap_or_else(|e| panic!("{e}")),
        ProjectId::new("orders").unwrap_or_else(|e| panic!("{e}")),
        EnvironmentId::new("prod").unwrap_or_else(|e| panic!("{e}")),
    )
}

fn convert() -> ConvertedHistory {
    let history: serde_json::Value =
        serde_json::from_str(HISTORY).unwrap_or_else(|e| panic!("fixture json: {e}"));
    convert_history(&scope(), &history).unwrap_or_else(|e| panic!("convert: {e}"))
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
