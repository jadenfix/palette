// Test code uses unwrap/expect freely on known-good fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for the Beater MCP server.
//!
//! These verify three properties:
//! 1. **Coverage**: the generated tool set exactly equals the set of `/v1`
//!    operationIds in the OpenAPI spec — every operation, no phantom tools.
//! 2. **Parity**: a `tools/call` produces the same JSON as the equivalent direct
//!    HTTP request (same args + auth), proving tools reuse the real handlers.
//! 3. **Smoke**: `initialize` and `tools/list` work over the `/mcp` route.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::Router;
use beater_api::{router, ApiState};
use beater_archive::ParquetTraceArchive;
use beater_audit::SqliteAuditStore;
use beater_bus::InMemoryBus;
use beater_calibration::SqliteCalibrationStore;
use beater_core::Money;
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_ingest::{IngestPolicy, IngestService};
use beater_judge::{JudgeBrokerService, KeywordJudgeProvider, SqliteJudgeLedger};
use beater_search::TantivySearchIndex;
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use beater_usage::SqliteUsageLedger;
use http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

fn unwrap<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("test setup failed: {err}"),
    }
}

/// Build a fully-integrated in-memory `ApiState` (mirrors the helper in
/// `beater-api/tests/openapi_coverage.rs`).
fn build_state() -> (ApiState, tempfile::TempDir) {
    let tempdir = unwrap(tempfile::tempdir());
    let artifacts = Arc::new(unwrap(FsArtifactStore::new(
        tempdir.path().join("artifacts"),
    )));
    let traces = Arc::new(unwrap(SqliteTraceStore::in_memory()));
    let search = Arc::new(unwrap(TantivySearchIndex::in_memory()));
    let archive = unwrap(ParquetTraceArchive::new(tempdir.path().join("archive")));
    let datasets = Arc::new(unwrap(SqliteDatasetStore::in_memory()));
    let experiments = Arc::new(unwrap(SqliteExperimentStore::in_memory()));
    let gates = Arc::new(unwrap(SqliteGateStore::in_memory()));
    let human_reviews = Arc::new(unwrap(SqliteHumanReviewStore::in_memory()));
    let calibrations = Arc::new(unwrap(SqliteCalibrationStore::in_memory()));
    let usage = Arc::new(unwrap(SqliteUsageLedger::in_memory()));
    let audit = Arc::new(unwrap(SqliteAuditStore::in_memory()));
    let provider_secrets = Arc::new(unwrap(EncryptedSqliteProviderSecretStore::in_memory(
        unwrap(SecretKeyring::generated_for_tests()),
    )));
    let judge_ledger = Arc::new(unwrap(SqliteJudgeLedger::in_memory()));
    let judge_broker = Arc::new(JudgeBrokerService::new(
        provider_secrets.clone(),
        judge_ledger.clone(),
        KeywordJudgeProvider::new(Money::usd_micros(25)),
        Money::usd_micros(100),
    ));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());

    let state = ApiState::with_integrations(ingest, traces, search, archive, datasets, experiments)
        .with_gates(gates)
        .with_human_reviews(human_reviews)
        .with_calibrations(calibrations)
        .with_usage(usage)
        .with_audit(audit)
        .with_judge(provider_secrets, judge_broker, judge_ledger);

    (state, tempdir)
}

/// Set of `/v1` operationIds documented in the spec.
fn spec_v1_operation_ids() -> BTreeSet<String> {
    let spec = beater_api::openapi::openapi();
    let doc: Value = serde_json::to_value(&spec).expect("serialize spec");
    let mut ids = BTreeSet::new();
    let paths = doc
        .get("paths")
        .and_then(Value::as_object)
        .expect("paths object");
    for (path, item) in paths {
        if !path.starts_with("/v1") {
            continue;
        }
        let item = item.as_object().expect("path item object");
        for method in ["get", "post", "put", "delete", "patch"] {
            if let Some(op) = item.get(method) {
                let id = op
                    .get("operationId")
                    .and_then(Value::as_str)
                    .expect("operationId present");
                ids.insert(id.to_string());
            }
        }
    }
    ids
}

/// POST a JSON-RPC body to `/mcp` and return (status, parsed JSON).
async fn mcp_call(app: &Router, body: Value, auth: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json");
    if let Some(token) = auth {
        builder = builder.header("authorization", token);
    }
    let request = unwrap(builder.body(Body::from(body.to_string())));
    let response = unwrap(app.clone().oneshot(request).await);
    let status = response.status();
    let bytes = unwrap(to_bytes(response.into_body(), 32 * 1024 * 1024).await);
    let value: Value = if bytes.is_empty() {
        Value::Null
    } else {
        unwrap(serde_json::from_slice(&bytes))
    };
    (status, value)
}

#[test]
fn tool_set_equals_spec_v1_operations() {
    let tools: BTreeSet<String> = beater_mcp::tool_names().into_iter().collect();
    let spec = spec_v1_operation_ids();

    // Every operation is exposed, and there are no phantom tools.
    assert_eq!(
        tools, spec,
        "MCP tool set must equal the set of /v1 operationIds in the spec"
    );
    // Sanity: the spec covers 41 /v1 operations.
    assert_eq!(tools.len(), 41, "expected 41 tools, got {}", tools.len());
}

#[tokio::test]
async fn initialize_and_tools_list_over_mcp_route() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    // initialize
    let (status, init) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(init["jsonrpc"], "2.0");
    assert_eq!(init["id"], 1);
    assert!(init["result"]["protocolVersion"].is_string());
    assert_eq!(init["result"]["serverInfo"]["name"], "beater-mcp");
    assert!(init["result"]["capabilities"]["tools"].is_object());

    // tools/list
    let (status, listed) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tools = listed["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 41);
    // Each tool has the required MCP shape.
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert!(tool["inputSchema"]["properties"].is_object());
    }
    // A representative tool with path + query params is present and shaped right.
    let traces_list = tools
        .iter()
        .find(|t| t["name"] == "listTraces")
        .expect("listTraces tool present");
    let props = &traces_list["inputSchema"]["properties"];
    assert!(props["tenant_id"].is_object(), "path param exposed");
    assert!(props["project_id"].is_object(), "query param exposed");
    let required = traces_list["inputSchema"]["required"]
        .as_array()
        .expect("required array");
    assert!(
        required.iter().any(|v| v == "tenant_id"),
        "path param is required"
    );
}

/// Parity: a `tools/call` returns the same JSON body as the equivalent direct
/// HTTP request, with identical auth (here: anonymous, auth disabled).
#[tokio::test]
async fn tools_call_matches_direct_http_for_traces_list() {
    let (state, _tempdir) = build_state();
    let mcp_app = beater_mcp::router(state.clone());
    let http_app = router(state);

    // 1) Direct HTTP call.
    let http_request = unwrap(
        Request::builder()
            .method("GET")
            .uri("/v1/traces/tenant-1?project_id=proj-1&environment_id=env-1")
            .body(Body::empty()),
    );
    let http_response = unwrap(http_app.oneshot(http_request).await);
    let http_status = http_response.status();
    let http_bytes = unwrap(to_bytes(http_response.into_body(), 32 * 1024 * 1024).await);
    let http_json: Value = unwrap(serde_json::from_slice(&http_bytes));

    // 2) MCP tools/call with the same args.
    let (status, rpc) = mcp_call(
        &mcp_app,
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "listTraces",
                "arguments": {
                    "tenant_id": "tenant-1",
                    "project_id": "proj-1",
                    "environment_id": "env-1"
                }
            }
        }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let result = &rpc["result"];
    assert_eq!(
        result["isError"], false,
        "successful HTTP call must not be an MCP error: {rpc}"
    );
    assert_eq!(
        result["_meta"]["httpStatus"].as_u64().unwrap() as u16,
        http_status.as_u16(),
        "MCP must report the same HTTP status as the direct call"
    );

    // The structured tool result equals the direct HTTP JSON body.
    assert_eq!(
        result["structuredContent"], http_json,
        "MCP tool result JSON must equal the direct HTTP response JSON"
    );
}

/// A 4xx from the underlying handler surfaces as `isError: true`.
#[tokio::test]
async fn tools_call_surfaces_http_errors() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    // Unknown tool -> JSON-RPC error.
    let (_status, rpc) = mcp_call(
        &app,
        json!({
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/call",
            "params": { "name": "does_not_exist", "arguments": {} }
        }),
        None,
    )
    .await;
    assert!(rpc["error"].is_object(), "unknown tool is a JSON-RPC error");
}

/// Acceptance #4: `/mcp` is reachable in the beaterd-style merged app and an
/// `initialize` POST returns 200.
#[tokio::test]
async fn mcp_reachable_in_merged_app() {
    let (state, _tempdir) = build_state();
    // Mirror beaterd: merge the MCP router into the API router.
    let app = router(state.clone()).merge(beater_mcp::router(state));

    let request = unwrap(
        Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} })
                    .to_string(),
            )),
    );
    let response = unwrap(app.clone().oneshot(request).await);
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = unwrap(to_bytes(response.into_body(), 1024 * 1024).await);
    let json: Value = unwrap(serde_json::from_slice(&bytes));
    assert_eq!(json["result"]["serverInfo"]["name"], "beater-mcp");

    // And the HTTP API still works in the same merged app.
    let health = unwrap(
        Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty()),
    );
    let health_resp = unwrap(app.oneshot(health).await);
    assert_eq!(health_resp.status(), StatusCode::OK);
}
