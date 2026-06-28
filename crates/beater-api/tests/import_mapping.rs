use std::sync::Arc;

use axum::body::{to_bytes, Body};
use beater_api::{router, ApiState};
use beater_bus::InMemoryBus;
use beater_ingest::{IngestOutcome, IngestPolicy, IngestService};
use beater_schema::{AgentSpanKind, SpanStatus, TraceView};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn import_source_mapping_projects_foreign_trace() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(ApiState::new(ingest, traces));

    let import_body = json!({
        "source": "mapping",
        "payload": {
            "config": {
                "spans_path": "rows",
                "fields": {
                    "trace_id": "tid",
                    "span_id": "sid",
                    "kind": "kind",
                    "name": "name",
                    "status": "status",
                    "start_time": "ts",
                    "output": "result"
                },
                "kind_map": {
                    "vendor-chat": "llm.call"
                },
                "attributes_path": "meta",
                "attribute_map": {
                    "legacy.case_id": "dataset.case_id"
                }
            },
            "document": {
                "rows": [{
                    "tid": "api-mapped-trace",
                    "sid": "api-mapped-span",
                    "kind": "vendor-chat",
                    "name": "mapped chat",
                    "status": "ok",
                    "ts": "2026-01-01T00:00:00Z",
                    "result": {"answer": "mapped"},
                    "meta": {
                        "legacy.case_id": "case-1",
                        "vendor.score": 0.9
                    }
                }]
            }
        }
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/import/tenant/project/prod")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&import_body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let outcome: IngestOutcome =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(outcome.ack.accepted_raw, 1);
    assert_eq!(outcome.ack.accepted_spans, 1);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/api-mapped-trace")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let trace: TraceView = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(trace.spans.len(), 1);
    let span = &trace.spans[0];
    assert_eq!(span.trace_id.as_str(), "api-mapped-trace");
    assert_eq!(span.span_id.as_str(), "api-mapped-span");
    assert_eq!(span.kind, AgentSpanKind::LlmCall);
    assert_eq!(span.name, "mapped chat");
    assert_eq!(span.status, SpanStatus::Ok);
    assert_eq!(span.normalizer_version, "beater-mapping-import-v1");
    assert_eq!(span.attributes["dataset.case_id"], json!("case-1"));
    assert_eq!(span.attributes["vendor.score"], json!(0.9));
    assert_eq!(span.attributes["output.value"], json!("[redacted]"));
}
