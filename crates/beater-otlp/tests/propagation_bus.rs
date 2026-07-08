//! Integration test: a W3C trace context survives a real durable-bus hop and a
//! spawned async task, end to end through `beater_bus::InMemoryBus`.
//!
//! Beater's `BusMessage` payload is opaque bytes, so the producer serializes the
//! W3C carrier (traceparent/tracestate/baggage) alongside the application payload
//! into the message body. The consumer pulls the message, rebuilds the trace
//! context, and spawns work that must land on the *same* trace id — which is what
//! `spawn_with_context` guarantees across the `tokio::spawn` boundary.

use beater_bus::{BusMessage, DurableBus, InMemoryBus};
use beater_core::{IdempotencyKey, ProjectId, TenantId};
use beater_otlp::propagation::{
    BAGGAGE_HEADER, Baggage, TRACEPARENT_HEADER, TRACESTATE_HEADER, TraceContext,
    spawn_with_context,
};
use std::collections::BTreeMap;

const TRACEPARENT: &str = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";

#[tokio::test]
async fn trace_context_survives_bus_enqueue_and_spawned_consumer() {
    // ---- Producer: capture the inbound request context + baggage ----
    let inbound: BTreeMap<String, String> = BTreeMap::from([
        (TRACEPARENT_HEADER.to_string(), TRACEPARENT.to_string()),
        (TRACESTATE_HEADER.to_string(), "vendor=opaque".to_string()),
        (
            BAGGAGE_HEADER.to_string(),
            "tenant=acme,project=checkout,release=2026.06.23,api_key=sk-leak".to_string(),
        ),
    ]);
    let context = TraceContext::extract(&inbound).unwrap_or_else(|| panic!("inbound context"));
    let baggage = Baggage::extract(&inbound);

    // Carry the context inside the message body (the BusMessage contract is
    // untouched: payload stays opaque bytes).
    let mut carrier: BTreeMap<String, String> = BTreeMap::new();
    context.inject(&mut carrier);
    // Inject the already-redacted baggage so no secret crosses the queue.
    carrier.insert(BAGGAGE_HEADER.to_string(), baggage.to_header());
    let body = serde_json::to_vec(&carrier).unwrap_or_else(|err| panic!("{err}"));
    assert!(
        !String::from_utf8_lossy(&body).contains("sk-leak"),
        "secret baggage must not be enqueued in the clear"
    );

    let bus = InMemoryBus::new(8);
    let message = BusMessage::new(
        TenantId::new("acme").unwrap_or_else(|err| panic!("{err}")),
        ProjectId::new("checkout").unwrap_or_else(|err| panic!("{err}")),
        IdempotencyKey::new("delivery-1").unwrap_or_else(|err| panic!("{err}")),
        "trace.propagation_demo",
        body,
    );
    bus.publish(message)
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    // ---- Consumer: pull the message, rebuild context, spawn child work ----
    let mut consumed = bus
        .consume_batch(1)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(consumed.len(), 1);
    let delivered = consumed.remove(0);

    let restored: BTreeMap<String, String> =
        serde_json::from_slice(&delivered.payload).unwrap_or_else(|err| panic!("{err}"));
    let consumer_context =
        TraceContext::extract(&restored).unwrap_or_else(|| panic!("restored context"));
    assert_eq!(consumer_context.trace_id(), context.trace_id());
    assert_eq!(consumer_context.tracestate(), Some("vendor=opaque"));

    let consumer_baggage = Baggage::extract(&restored);
    assert_eq!(consumer_baggage.tenant(), Some("acme"));
    assert_eq!(consumer_baggage.project(), Some("checkout"));
    assert_eq!(consumer_baggage.release(), Some("2026.06.23"));
    // The secret survived the hop only as a redaction marker.
    assert_eq!(consumer_baggage.get("api_key"), Some("[REDACTED]"));

    // The worker opens a child span and spawns async work under it.
    let child = consumer_context
        .child("00f067aa0ba902b7")
        .unwrap_or_else(|| panic!("child context"));
    let handle = spawn_with_context(Some(child), |ctx| async move {
        let ctx = ctx.unwrap_or_else(|| panic!("context propagated into spawned task"));
        (ctx.trace_id().to_string(), ctx.span_id().to_string())
    });
    let (trace_in_task, parent_in_task) = handle.await.unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(trace_in_task, "0af7651916cd43dd8448eb211c80319c");
    assert_eq!(trace_in_task, context.trace_id());
    assert_eq!(parent_in_task, "00f067aa0ba902b7");

    bus.ack(delivered)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
}
