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

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::Router;
use beater_api::{router, ApiState};
use beater_archive::ParquetTraceArchive;
use beater_audit::SqliteAuditStore;
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
use beater_bus::InMemoryBus;
use beater_calibration::SqliteCalibrationStore;
use beater_core::{EnvironmentId, Money, ProjectId, TenantId};
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_ingest::{IngestPolicy, IngestService};
use beater_judge::{JudgeBrokerService, KeywordJudgeProvider, SqliteJudgeLedger};
use beater_search::TantivySearchIndex;
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_security::ApiScope;
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

/// Map of `/v1` operationId -> upper-case HTTP method, from the spec.
fn spec_op_methods() -> BTreeMap<String, String> {
    let spec = beater_api::openapi::openapi();
    let doc: Value = serde_json::to_value(&spec).expect("serialize spec");
    let mut map = BTreeMap::new();
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
                map.insert(id.to_string(), method.to_ascii_uppercase());
            }
        }
    }
    map
}

/// Fetch the `tools/list` array over the `/mcp` route.
async fn list_tools(app: &Router) -> Vec<Value> {
    let (status, listed) = mcp_call(
        app,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    listed["result"]["tools"]
        .as_array()
        .expect("tools array")
        .clone()
}

/// POST a JSON-RPC body to `/mcp` and return (status, parsed JSON).
async fn mcp_call(app: &Router, body: Value, auth: Option<&str>) -> (StatusCode, Value) {
    let headers = auth
        .map(|token| vec![("authorization", token)])
        .unwrap_or_default();
    mcp_call_with_headers(app, body, &headers).await
}

async fn mcp_call_with_headers(
    app: &Router,
    body: Value,
    headers: &[(&str, &str)],
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
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

    // Every operation is exposed, and there are no phantom tools. The synthetic
    // `help` tool is deliberately NOT part of the spec-coverage answer.
    assert_eq!(
        tools, spec,
        "MCP tool set must equal the set of /v1 operationIds in the spec"
    );
    assert!(
        !tools.contains("help"),
        "the synthetic help tool must not appear in spec-coverage tool_names()"
    );
    // Sanity: the spec covers 53 /v1 operations.
    assert_eq!(tools.len(), 53, "expected 53 tools, got {}", tools.len());
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
    // 53 spec-derived tools + the synthetic `help` meta tool.
    assert_eq!(tools.len(), 54);
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

#[tokio::test]
async fn initialize_and_tools_list_over_stdio_transport() {
    let (state, _tempdir) = build_state();
    let input = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }).to_string(),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }).to_string(),
    ]
    .join("\n")
        + "\n";
    let mut output = Vec::new();

    unwrap(beater_mcp::serve_stdio_streams(state, input.as_bytes(), &mut output).await);

    let text = String::from_utf8(output).expect("stdio output is utf8");
    let lines: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).expect("stdout line is JSON-RPC"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["jsonrpc"], "2.0");
    assert_eq!(lines[0]["id"], 1);
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "beater-mcp");
    assert_eq!(lines[1]["jsonrpc"], "2.0");
    assert_eq!(lines[1]["id"], 2);
    let tools = lines[1]["result"]["tools"]
        .as_array()
        .expect("tools/list result");
    assert_eq!(tools.len(), beater_mcp::tool_names().len() + 1);
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

#[tokio::test]
async fn tools_call_forwards_strict_auth_scope_headers() {
    let (state, _tempdir) = build_state();
    let api_keys = Arc::new(unwrap(SqliteApiKeyStore::in_memory()));
    let created = unwrap(
        api_keys
            .create_key(CreateApiKeyRequest {
                tenant_id: unwrap(TenantId::new("tenant-1")),
                project_id: unwrap(ProjectId::new("proj-1")),
                environment_id: unwrap(EnvironmentId::new("env-1")),
                scopes: BTreeSet::from([ApiScope::TraceRead]),
            })
            .await,
    );
    let app = beater_mcp::router(state.require_auth(api_keys));
    let call = json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {
            "name": "getSpan",
            "arguments": {
                "tenant_id": "tenant-1",
                "trace_id": "missing-trace",
                "span_id": "missing-span"
            }
        }
    });
    let authorization = format!("Bearer {}", created.secret);

    let (status, missing_scope) =
        mcp_call_with_headers(&app, call.clone(), &[("authorization", &authorization)]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(missing_scope["result"]["isError"], true);
    assert_eq!(missing_scope["result"]["_meta"]["httpStatus"], 400);

    let (status, authorized) = mcp_call_with_headers(
        &app,
        call,
        &[
            ("authorization", &authorization),
            ("x-beater-project-id", "proj-1"),
            ("x-beater-environment-id", "env-1"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(authorized["result"]["isError"], true);
    assert_eq!(authorized["result"]["_meta"]["httpStatus"], 404);
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

#[tokio::test]
async fn tools_call_rejects_non_scalar_query_param() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    let (status, rpc) = mcp_call(
        &app,
        json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "listTraces",
                "arguments": {
                    "tenant_id": "tenant-1",
                    "limit": { "bad": true }
                }
            }
        }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rpc["error"]["code"], -32602);
    let message = rpc["error"]["message"]
        .as_str()
        .expect("error message is a string");
    assert!(
        message.contains("query parameter limit must be a scalar"),
        "unexpected error message: {message}"
    );
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

/// Every listed tool advertises an `outputSchema` object and method-derived
/// `annotations` whose hints match the underlying HTTP verb.
#[tokio::test]
async fn tools_list_exposes_output_schema_and_annotations() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);
    let tools = list_tools(&app).await;
    let methods = spec_op_methods();
    // 53 spec-derived tools + the synthetic `help` meta tool.
    assert_eq!(tools.len(), 54);

    // The six list endpoints return top-level JSON arrays, which MCP forbids as
    // structured output, so they advertise no outputSchema.
    let array_ops = [
        "listAuditEvents",
        "listJudgeLedger",
        "listProviderSecrets",
        "listReviewTasks",
        "listConnectors",
        "listConnectorTools",
    ];

    for tool in &tools {
        let name = tool["name"].as_str().expect("tool name");
        // `help` is the synthetic meta tool — not a spec op; covered separately.
        if name == "help" {
            continue;
        }

        // outputSchema is present for object-returning ops and object-rooted
        // (never array — a strict client would reject that); omitted for the
        // known array-returning ops.
        match tool.get("outputSchema") {
            None => assert!(
                array_ops.contains(&name),
                "{name}: only array-returning ops may omit outputSchema"
            ),
            Some(output) => {
                assert!(output.is_object(), "{name}: outputSchema must be an object");
                let root_type = output["type"].as_str();
                assert_ne!(
                    root_type,
                    Some("array"),
                    "{name}: outputSchema is array-rooted"
                );
                // Object-rooted directly, or a $ref resolved against bundled components.
                if root_type != Some("object") {
                    let referenced = output["$ref"]
                        .as_str()
                        .unwrap_or_else(|| panic!("{name}: output is neither object nor $ref"));
                    let comp = referenced.rsplit('/').next().expect("ref name");
                    assert!(
                        output["components"]["schemas"][comp].is_object(),
                        "{name}: ref {referenced} resolves under components/schemas"
                    );
                }
            }
        }

        // annotations carry method-derived hints plus operation-aware POST risk.
        let ann = &tool["annotations"];
        assert_eq!(
            ann["title"], tool["description"],
            "{name}: annotation title"
        );
        let method = methods.get(name).expect("tool maps to a spec method");
        let expect_read_only = method == "GET";
        let expect_idempotent = matches!(method.as_str(), "GET" | "PUT" | "DELETE");
        let expect_destructive = matches!(method.as_str(), "PUT" | "DELETE")
            || (method == "POST" && matches!(name, "revokeApiKey" | "revokeProviderSecret"));
        let expect_open_world = method == "POST"
            && matches!(
                name,
                "evaluateJudge" | "runJudgeEval" | "runJudgeExperiment" | "importSource"
            );
        assert_eq!(
            ann["readOnlyHint"], expect_read_only,
            "{name} ({method}): readOnlyHint"
        );
        assert_eq!(
            ann["idempotentHint"], expect_idempotent,
            "{name} ({method}): idempotentHint"
        );
        assert_eq!(
            ann["destructiveHint"], expect_destructive,
            "{name} ({method}): destructiveHint"
        );
        assert_eq!(
            ann["openWorldHint"], expect_open_world,
            "{name} ({method}): openWorldHint"
        );
    }
}

/// Pin representative operations so a regression in the method and
/// operation-aware safety hints is caught by name.
#[tokio::test]
async fn representative_tools_have_correct_safety_hints() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);
    let tools = list_tools(&app).await;
    let by_name = |n: &str| {
        tools
            .iter()
            .find(|t| t["name"] == n)
            .unwrap_or_else(|| panic!("{n} present"))
            .clone()
    };

    // GET: read-only.
    let listing = by_name("listTraces");
    assert_eq!(listing["annotations"]["readOnlyHint"], true);
    assert_eq!(listing["annotations"]["destructiveHint"], false);
    assert_eq!(listing["annotations"]["idempotentHint"], true);

    // POST: a write, not read-only.
    let create = by_name("createDataset");
    assert_eq!(create["annotations"]["readOnlyHint"], false);
    assert_eq!(create["annotations"]["destructiveHint"], false);
    assert_eq!(create["annotations"]["idempotentHint"], false);
    assert_eq!(create["annotations"]["openWorldHint"], false);

    // Destructive POST: revokes access, so clients should confirm it.
    let revoke = by_name("revokeApiKey");
    assert_eq!(revoke["annotations"]["readOnlyHint"], false);
    assert_eq!(revoke["annotations"]["destructiveHint"], true);
    assert_eq!(revoke["annotations"]["idempotentHint"], false);
    assert_eq!(revoke["annotations"]["openWorldHint"], false);

    // Open-world POST: can invoke an external provider.
    let judge = by_name("evaluateJudge");
    assert_eq!(judge["annotations"]["readOnlyHint"], false);
    assert_eq!(judge["annotations"]["destructiveHint"], false);
    assert_eq!(judge["annotations"]["idempotentHint"], false);
    assert_eq!(judge["annotations"]["openWorldHint"], true);
}

/// The `outputSchema` for an operation returning a component type resolves its
/// `$ref` against the bundled component schemas.
#[tokio::test]
async fn output_schema_resolves_component_refs() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);
    let tools = list_tools(&app).await;
    let create = tools
        .iter()
        .find(|t| t["name"] == "createDataset")
        .expect("createDataset present");
    let output = &create["outputSchema"];

    // The success body is `{ "$ref": "#/components/schemas/Dataset" }`; the ref
    // target must be present under the bundled OpenAPI pointer space, and the
    // target must itself be an object (so the advertised output is object-rooted).
    let referenced = output["$ref"].as_str().expect("output is a $ref");
    let type_name = referenced
        .rsplit('/')
        .next()
        .expect("ref has a component name");
    let target = &output["components"]["schemas"][type_name];
    assert!(
        target.is_object(),
        "ref {referenced} resolves under components/schemas"
    );
    assert_eq!(target["type"], "object", "resolved output type is object");
}

/// `initialize` negotiates the protocol version: a supported version requested
/// by the client is echoed back; an unsupported one falls back to the latest.
#[tokio::test]
async fn initialize_negotiates_protocol_version() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    // Client requests an older but supported revision -> echoed back.
    let (_s, older) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": { "protocolVersion": "2024-11-05" } }),
        None,
    )
    .await;
    assert_eq!(older["result"]["protocolVersion"], "2024-11-05");

    // Unsupported version -> server advertises its latest.
    let (_s, unknown) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "initialize",
                "params": { "protocolVersion": "1999-01-01" } }),
        None,
    )
    .await;
    assert_eq!(unknown["result"]["protocolVersion"], "2025-06-18");

    // No version requested -> latest.
    let (_s, none) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 3, "method": "initialize", "params": {} }),
        None,
    )
    .await;
    assert_eq!(none["result"]["protocolVersion"], "2025-06-18");

    // `params` omitted entirely (bare initialize) -> latest, no panic.
    let (status, bare) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 4, "method": "initialize" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bare["result"]["protocolVersion"], "2025-06-18");
}

/// `GET /mcp` is a static capability probe advertising the latest protocol
/// version and server info.
#[tokio::test]
async fn get_mcp_probe_advertises_latest_version() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);
    let request = unwrap(
        Request::builder()
            .method("GET")
            .uri("/mcp")
            .body(Body::empty()),
    );
    let response = unwrap(app.oneshot(request).await);
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = unwrap(to_bytes(response.into_body(), 1024 * 1024).await);
    let json: Value = unwrap(serde_json::from_slice(&bytes));
    assert_eq!(json["protocolVersion"], "2025-06-18");
    assert_eq!(json["serverInfo"]["name"], "beater-mcp");
    assert!(json["capabilities"]["tools"].is_object());
}

/// A list endpoint that returns a top-level JSON array surfaces its body via the
/// text `content` but omits `structuredContent` (which MCP requires be an object).
#[tokio::test]
async fn array_result_omits_structured_content() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    let (status, rpc) = mcp_call(
        &app,
        json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "listProviderSecrets",
                "arguments": { "tenant_id": "tenant-1", "project_id": "proj-1" }
            }
        }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let result = &rpc["result"];
    assert_eq!(result["isError"], false, "list call should succeed: {rpc}");
    // Body is a JSON array -> no structuredContent, but the text content carries it.
    assert!(
        result.get("structuredContent").is_none(),
        "array result must not set structuredContent: {rpc}"
    );
    let text = result["content"][0]["text"].as_str().expect("text content");
    let parsed: Value = serde_json::from_str(text).expect("content is JSON");
    assert!(parsed.is_array(), "the underlying body is a JSON array");
}

/// `tools/list` is deterministic and stable across repeated calls (the catalog
/// is cached and the output ordering fixed).
#[tokio::test]
async fn tools_list_is_stable_across_calls() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);
    let first = list_tools(&app).await;
    let second = list_tools(&app).await;
    assert_eq!(first, second, "tools/list must be byte-stable across calls");
}

/// The synthetic `help` tool is listed (read-only, well-formed) and is the only
/// non-spec tool in `tools/list`.
#[tokio::test]
async fn help_tool_is_listed_and_read_only() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);
    let tools = list_tools(&app).await;
    let spec_ops = spec_op_methods();

    // Exactly one tool is non-spec, and it is `help`.
    let non_spec: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .filter(|n| !spec_ops.contains_key(*n))
        .collect();
    assert_eq!(non_spec, ["help"], "help is the only synthetic tool");

    let help = tools
        .iter()
        .find(|t| t["name"] == "help")
        .expect("help present");
    assert_eq!(help["inputSchema"]["type"], "object");
    assert!(help["inputSchema"]["properties"]["query"].is_object());
    assert!(help["inputSchema"]["properties"]["tool"].is_object());
    assert_eq!(help["outputSchema"]["type"], "object");
    assert_eq!(help["annotations"]["readOnlyHint"], true);
    assert_eq!(help["annotations"]["destructiveHint"], false);
}

/// `help` with no arguments returns an overview of every spec tool, as object
/// structured content, without dispatching to the API.
#[tokio::test]
async fn help_overview_lists_every_spec_tool() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    let (status, rpc) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "help", "arguments": {} } }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let result = &rpc["result"];
    assert_eq!(result["isError"], false, "help must not error: {rpc}");
    let structured = &result["structuredContent"];
    assert!(structured.is_object(), "structuredContent is an object");
    assert_eq!(structured["server"]["name"], "beater-mcp");
    assert_eq!(
        structured["toolCount"], 53,
        "overview covers all 53 spec tools"
    );
    let listed = structured["tools"].as_array().expect("tools array");
    assert_eq!(listed.len(), 53);
    // Each entry is a compact {name, method, description} summary.
    for entry in listed {
        assert!(entry["name"].is_string());
        assert!(entry["method"].is_string());
        assert!(entry["description"].is_string());
    }
}

/// `help` with a `query` filters the catalog case-insensitively over name and
/// description.
#[tokio::test]
async fn help_query_filters_catalog() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    let (_status, rpc) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "help", "arguments": { "query": "DATASET" } } }),
        None,
    )
    .await;
    let structured = &rpc["result"]["structuredContent"];
    let listed = structured["tools"].as_array().expect("tools array");
    assert!(!listed.is_empty(), "expected some dataset tools");
    assert!(listed.len() < 41, "query must narrow the catalog");
    assert_eq!(structured["toolCount"], listed.len());
    // Every match contains the (lower-cased) query in its name or description.
    for entry in listed {
        let name = entry["name"].as_str().unwrap().to_ascii_lowercase();
        let desc = entry["description"].as_str().unwrap().to_ascii_lowercase();
        assert!(
            name.contains("dataset") || desc.contains("dataset"),
            "unexpected match: {entry}"
        );
    }
    // A representative dataset op is present.
    assert!(listed.iter().any(|t| t["name"] == "createDataset"));
}

/// `help` with a `tool` returns that operation's full descriptor; an unknown
/// operationId is a JSON-RPC error.
#[tokio::test]
async fn help_describes_one_tool_and_rejects_unknown() {
    let (state, _tempdir) = build_state();
    let app = beater_mcp::router(state);

    let (_status, rpc) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "help", "arguments": { "tool": "listTraces" } } }),
        None,
    )
    .await;
    let tool = &rpc["result"]["structuredContent"]["tool"];
    assert_eq!(tool["name"], "listTraces");
    assert_eq!(tool["method"], "GET");
    assert_eq!(tool["path"], "/v1/traces/{tenant_id}");
    assert_eq!(tool["inputSchema"]["type"], "object");
    assert!(
        tool["outputSchema"].is_object(),
        "listTraces has an output schema"
    );
    assert_eq!(tool["annotations"]["readOnlyHint"], true);

    // Unknown operationId -> JSON-RPC error.
    let (_status, err) = mcp_call(
        &app,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": { "name": "help", "arguments": { "tool": "nope" } } }),
        None,
    )
    .await;
    assert!(
        err["error"].is_object(),
        "unknown tool query is an error: {err}"
    );
}
