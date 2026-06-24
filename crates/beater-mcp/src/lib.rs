//! Model Context Protocol (MCP) server for the Beater API.
//!
//! This exposes **every** `/v1` API operation as an MCP tool, kept in sync with
//! the API *by construction*: the tool catalog is derived at runtime from
//! [`beater_api::openapi::openapi()`], and `tools/call` dispatches through the
//! real [`beater_api::router`] via `oneshot`. There is no hand-maintained mirror
//! of the routes, so a new `/v1` route automatically becomes a new MCP tool and
//! cannot drift from the HTTP surface.
//!
//! ## Design: hand-rolled JSON-RPC vs `rmcp`
//!
//! We deliberately do **not** use the `rmcp` crate. MCP's transport for an HTTP
//! host is plain JSON-RPC 2.0 over `POST`, which is trivial to implement and
//! keeps the dependency/footprint minimal. More importantly, the in-process
//! approach lets tool calls reuse the *exact* axum handlers, `ApiState`, and
//! auth (`Authorization: Bearer ...` / `x-beater-api-key`) with zero
//! duplication — `rmcp` would add an abstraction layer between MCP and the real
//! handlers and reintroduce the drift risk we are eliminating.
//!
//! ## Auth
//!
//! Auth is identical to the HTTP surface: the inbound MCP request's
//! `Authorization` and `x-beater-api-key` headers are forwarded verbatim onto
//! every synthesized `/v1` request, so the real `authorize()` path runs
//! unchanged. When the server is built with auth disabled, calls are anonymous,
//! exactly as for direct HTTP.

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use beater_api::{router as api_router, ApiState};
use http::{HeaderMap, Method, Request, StatusCode};
use serde_json::{json, Map, Value};
use tower::ServiceExt;

/// MCP protocol version advertised in `initialize`.
const PROTOCOL_VERSION: &str = "2024-11-05";

/// Maximum response body size we will buffer from a dispatched handler.
const MAX_BODY_BYTES: usize = 32 * 1024 * 1024;

/// A single MCP tool derived from one OpenAPI operation.
#[derive(Clone, Debug)]
struct ToolSpec {
    /// MCP tool name = OpenAPI `operationId` (e.g. `traces_list`).
    name: String,
    /// HTTP method for the underlying operation (upper-case).
    method: String,
    /// OpenAPI path template with `{param}` placeholders.
    path_template: String,
    /// JSON Schema describing the tool's input arguments.
    input_schema: Value,
    /// Human-readable description (summary/description from the spec).
    description: String,
    /// Names of path parameters, in template order.
    path_params: Vec<String>,
    /// Names of query parameters.
    query_params: Vec<String>,
    /// Whether the operation accepts a JSON request body.
    has_body: bool,
}

/// Build the full MCP tool catalog from the live OpenAPI document.
///
/// This walks the serialized spec as JSON so it is robust to the exact utoipa
/// type shapes and naturally preserves `$ref`s; the resolved component schemas
/// are bundled under `$defs` in each tool's `inputSchema` so body refs resolve.
fn build_tools() -> Vec<ToolSpec> {
    let spec = beater_api::openapi::openapi();
    // Serialize once; treat the spec as plain JSON from here on.
    let doc: Value = serde_json::to_value(&spec).unwrap_or(Value::Null);

    // Component schemas, used to bundle `$defs` for body references.
    let component_schemas = doc
        .get("components")
        .and_then(|c| c.get("schemas"))
        .cloned();

    let mut tools = Vec::new();

    let Some(paths) = doc.get("paths").and_then(Value::as_object) else {
        return tools;
    };

    for (path, item) in paths {
        // Only expose versioned API operations as tools.
        if !path.starts_with("/v1") {
            continue;
        }
        let Some(item) = item.as_object() else {
            continue;
        };
        for method in ["get", "post", "put", "delete", "patch"] {
            let Some(op) = item.get(method).and_then(Value::as_object) else {
                continue;
            };
            let Some(operation_id) = op.get("operationId").and_then(Value::as_str) else {
                continue;
            };

            let description = op
                .get("summary")
                .and_then(Value::as_str)
                .or_else(|| op.get("description").and_then(Value::as_str))
                .unwrap_or(operation_id)
                .to_string();

            let (input_schema, path_params, query_params, has_body) =
                build_input_schema(op, component_schemas.as_ref());

            tools.push(ToolSpec {
                name: operation_id.to_string(),
                method: method.to_ascii_uppercase(),
                path_template: path.clone(),
                input_schema,
                description,
                path_params,
                query_params,
                has_body,
            });
        }
    }

    // Deterministic ordering keeps tools/list stable across runs.
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

/// Construct the JSON Schema `inputSchema` for one operation, plus the metadata
/// needed to later reconstruct the HTTP request.
///
/// Properties = path params + query params + (if a requestBody exists) a `body`
/// property carrying the requestBody schema. Path params are required.
fn build_input_schema(
    op: &Map<String, Value>,
    component_schemas: Option<&Value>,
) -> (Value, Vec<String>, Vec<String>, bool) {
    let mut properties = Map::new();
    let mut required: Vec<Value> = Vec::new();
    let mut path_params: Vec<String> = Vec::new();
    let mut query_params: Vec<String> = Vec::new();

    if let Some(params) = op.get("parameters").and_then(Value::as_array) {
        for param in params {
            let Some(param) = param.as_object() else {
                continue;
            };
            let Some(name) = param.get("name").and_then(Value::as_str) else {
                continue;
            };
            let location = param.get("in").and_then(Value::as_str).unwrap_or("");
            // Synthesized requests carry auth headers separately; header/cookie
            // params (e.g. the optional api-key header) are never tool inputs.
            if location != "path" && location != "query" {
                continue;
            }

            let mut prop = param
                .get("schema")
                .cloned()
                .unwrap_or_else(|| json!({ "type": "string" }));
            if let (Some(prop_obj), Some(desc)) = (
                prop.as_object_mut(),
                param.get("description").and_then(Value::as_str),
            ) {
                prop_obj
                    .entry("description")
                    .or_insert_with(|| Value::String(desc.to_string()));
            }
            properties.insert(name.to_string(), prop);

            match location {
                "path" => {
                    path_params.push(name.to_string());
                    required.push(Value::String(name.to_string()));
                }
                "query" => {
                    query_params.push(name.to_string());
                    if param
                        .get("required")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        required.push(Value::String(name.to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    let mut has_body = false;
    if let Some(body_schema) = op
        .get("requestBody")
        .and_then(|b| b.get("content"))
        .and_then(|c| c.get("application/json"))
        .and_then(|j| j.get("schema"))
    {
        has_body = true;
        properties.insert("body".to_string(), body_schema.clone());
        let body_required = op
            .get("requestBody")
            .and_then(|b| b.get("required"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if body_required {
            required.push(Value::String("body".to_string()));
        }
    }

    let mut schema = Map::new();
    schema.insert("type".to_string(), Value::String("object".to_string()));
    schema.insert("properties".to_string(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".to_string(), Value::Array(required));
    }
    // Bundle component schemas so any `$ref: #/components/schemas/X` inside the
    // body schema resolves. We expose them under both the OpenAPI pointer space
    // (via `components`) and JSON Schema `$defs` for maximum client compat.
    if has_body {
        if let Some(schemas) = component_schemas {
            schema.insert(
                "components".to_string(),
                json!({ "schemas": schemas.clone() }),
            );
        }
    }

    (Value::Object(schema), path_params, query_params, has_body)
}

/// Serialize a tool to its MCP `tools/list` JSON shape.
fn tool_to_json(tool: &ToolSpec) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "inputSchema": tool.input_schema,
    })
}

/// Build a [`Router`] that serves the MCP endpoint at `POST /mcp` (and `GET`
/// for capability probing), backed by the supplied [`ApiState`].
///
/// Merge this into the main app, e.g. `beater_api::router(state).merge(
/// beater_mcp::router(state))`, so `/mcp` is served alongside the HTTP API and
/// shares the same `ApiState` and auth.
pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp).get(handle_mcp_get))
        .with_state(state)
}

/// JSON-RPC error codes (subset used here).
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

/// `GET /mcp` returns minimal server info for capability probing.
async fn handle_mcp_get() -> Response {
    axum::Json(json!({
        "protocolVersion": PROTOCOL_VERSION,
        "serverInfo": server_info(),
        "capabilities": capabilities(),
    }))
    .into_response()
}

fn server_info() -> Value {
    json!({ "name": "beater-mcp", "version": env!("CARGO_PKG_VERSION") })
}

fn capabilities() -> Value {
    json!({ "tools": { "listChanged": false } })
}

/// `POST /mcp` — the JSON-RPC 2.0 entry point.
async fn handle_mcp(State(state): State<ApiState>, headers: HeaderMap, body: Body) -> Response {
    let bytes = match to_bytes(body, MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => {
            return json_response(rpc_error(
                Value::Null,
                PARSE_ERROR,
                "failed to read request body",
            ))
        }
    };
    let request: Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return json_response(rpc_error(Value::Null, PARSE_ERROR, "invalid JSON")),
    };

    // Note: JSON-RPC batching is not used by current MCP clients; handle a
    // single request object.
    let response = dispatch_rpc(&state, &headers, &request).await;
    match response {
        Some(resp) => json_response(resp),
        // Notification (no id) — respond 202 with empty body.
        None => StatusCode::ACCEPTED.into_response(),
    }
}

fn json_response(value: Value) -> Response {
    axum::Json(value).into_response()
}

/// Route a single JSON-RPC request. Returns `None` for notifications (no `id`).
async fn dispatch_rpc(state: &ApiState, headers: &HeaderMap, request: &Value) -> Option<Value> {
    let id = request.get("id").cloned();
    let is_notification = id.is_none();
    let id = id.unwrap_or(Value::Null);

    let Some(method) = request.get("method").and_then(Value::as_str) else {
        if is_notification {
            return None;
        }
        return Some(rpc_error(id, INVALID_REQUEST, "missing method"));
    };

    let params = request.get("params").cloned().unwrap_or(Value::Null);

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "serverInfo": server_info(),
            "capabilities": capabilities(),
        })),
        "notifications/initialized" | "initialized" => {
            // Lifecycle notification: acknowledge, no result.
            return if is_notification {
                None
            } else {
                Some(rpc_result(id, json!({})))
            };
        }
        "ping" => Ok(json!({})),
        "tools/list" => {
            let tools: Vec<Value> = build_tools().iter().map(tool_to_json).collect();
            Ok(json!({ "tools": tools }))
        }
        "tools/call" => call_tool(state, headers, &params).await,
        other => Err((METHOD_NOT_FOUND, format!("unknown method: {other}"))),
    };

    if is_notification {
        return None;
    }

    Some(match result {
        Ok(value) => rpc_result(id, value),
        Err((code, message)) => rpc_error(id, code, &message),
    })
}

/// Handle `tools/call`: resolve the operation, synthesize an HTTP request,
/// dispatch through the real router, and wrap the response as an MCP tool
/// result.
async fn call_tool(
    state: &ApiState,
    headers: &HeaderMap,
    params: &Value,
) -> Result<Value, (i64, String)> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or((INVALID_PARAMS, "missing tool name".to_string()))?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let arguments = arguments
        .as_object()
        .cloned()
        .ok_or((INVALID_PARAMS, "arguments must be an object".to_string()))?;

    let tools = build_tools();
    let tool = tools
        .iter()
        .find(|t| t.name == name)
        .ok_or((INVALID_PARAMS, format!("unknown tool: {name}")))?;

    // Substitute path params.
    let mut path = tool.path_template.clone();
    for param in &tool.path_params {
        let value = arguments.get(param).ok_or((
            INVALID_PARAMS,
            format!("missing required path parameter: {param}"),
        ))?;
        let rendered = value_to_path_segment(value).ok_or((
            INVALID_PARAMS,
            format!("path parameter {param} must be a scalar"),
        ))?;
        path = path.replace(&format!("{{{param}}}"), &urlencode(&rendered));
    }

    // Append query params.
    let mut query_pairs: Vec<String> = Vec::new();
    for param in &tool.query_params {
        if let Some(value) = arguments.get(param) {
            if value.is_null() {
                continue;
            }
            if let Some(rendered) = value_to_path_segment(value) {
                query_pairs.push(format!("{}={}", urlencode(param), urlencode(&rendered)));
            }
        }
    }
    let uri = if query_pairs.is_empty() {
        path
    } else {
        format!("{path}?{}", query_pairs.join("&"))
    };

    // Build request body from `arguments.body`.
    let body = if tool.has_body {
        match arguments.get("body") {
            Some(b) => Body::from(b.to_string()),
            None => Body::from("{}"),
        }
    } else {
        Body::empty()
    };

    let method = Method::from_bytes(tool.method.as_bytes())
        .map_err(|_| (INTERNAL_ERROR, "invalid method".to_string()))?;

    let mut builder = Request::builder().method(method).uri(&uri);
    if tool.has_body {
        builder = builder.header(http::header::CONTENT_TYPE, "application/json");
    }
    // Forward auth headers verbatim so the real authorize() path runs unchanged.
    for header_name in [http::header::AUTHORIZATION.as_str(), "x-beater-api-key"] {
        if let Some(value) = headers.get(header_name) {
            builder = builder.header(header_name, value);
        }
    }

    let request = builder
        .body(body)
        .map_err(|e| (INTERNAL_ERROR, format!("failed to build request: {e}")))?;

    // Dispatch through the *real* router — same handlers, state, and auth.
    let response = api_router(state.clone())
        .oneshot(request)
        .await
        .map_err(|e| (INTERNAL_ERROR, format!("dispatch failed: {e}")))?;

    let status = response.status();
    let resp_bytes = to_bytes(response.into_body(), MAX_BODY_BYTES)
        .await
        .map_err(|e| (INTERNAL_ERROR, format!("failed to read response: {e}")))?;
    let text = String::from_utf8_lossy(&resp_bytes).to_string();

    // Parse JSON if possible so callers get structured content; else raw text.
    let structured: Value = serde_json::from_str(&text).unwrap_or(Value::Null);

    let is_error = status.as_u16() >= 400;
    let mut result = Map::new();
    result.insert(
        "content".to_string(),
        json!([{ "type": "text", "text": text }]),
    );
    if !structured.is_null() {
        result.insert("structuredContent".to_string(), structured);
    }
    result.insert("isError".to_string(), Value::Bool(is_error));
    // Surface the HTTP status for debuggability / parity assertions.
    result.insert(
        "_meta".to_string(),
        json!({ "httpStatus": status.as_u16() }),
    );

    Ok(Value::Object(result))
}

/// Render a scalar JSON value as a path/query segment. Returns `None` for
/// arrays/objects (which are not valid scalar params).
fn value_to_path_segment(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Minimal percent-encoding for path/query segments (RFC 3986 unreserved set
/// passes through; everything else is `%XX`-encoded). Kept dependency-free.
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push_str(&format!("{byte:02X}"));
            }
        }
    }
    out
}

/// The set of MCP tool names exposed, for tests and introspection. This is the
/// authoritative "what does the MCP surface cover" answer, derived from the spec.
pub fn tool_names() -> Vec<String> {
    build_tools().into_iter().map(|t| t.name).collect()
}
