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
//! auth (`Authorization: Bearer ...` / `x-beater-api-key` plus strict-auth
//! project/environment scope headers) with zero duplication — `rmcp` would add
//! an abstraction layer between MCP and the real handlers and reintroduce the
//! drift risk we are eliminating.
//!
//! ## Auth
//!
//! Auth is identical to the HTTP surface: the inbound MCP request's
//! `Authorization`, `x-beater-api-key`, and strict-auth scope headers are
//! forwarded verbatim onto every synthesized `/v1` request, so the real
//! `authorize()` path runs unchanged. When the server is built with auth
//! disabled, calls are anonymous, exactly as for direct HTTP.

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use beater_api::{openapi::urlencode, router as api_router, ApiState};
use http::{HeaderMap, Method, Request, StatusCode};
use serde_json::{json, Map, Value};
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tower::ServiceExt;

/// Latest MCP protocol version this server implements. Advertised by default and
/// returned from `initialize` unless the client requests an older supported one.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Protocol revisions we can speak, newest first. `initialize` echoes back the
/// client's requested version when it is one of these, else falls back to
/// [`PROTOCOL_VERSION`] — standard MCP version negotiation.
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

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
    /// JSON Schema for the tool's structured result, derived from the operation's
    /// lowest 2xx `application/json` response. `None` when there is no JSON body.
    output_schema: Option<Value>,
    /// Human-readable description (summary/description from the spec).
    description: String,
    /// Names of path parameters, in template order.
    path_params: Vec<String>,
    /// Names of query parameters.
    query_params: Vec<String>,
    /// Whether the operation accepts a JSON request body.
    has_body: bool,
}

/// Process-wide tool catalog. The OpenAPI document is fixed for the lifetime of
/// the process, so the (non-trivial) build — serialize + walk the whole spec — is
/// done once and the result shared by `tools/list` and every `tools/call`.
fn tools() -> &'static [ToolSpec] {
    static TOOLS: OnceLock<Vec<ToolSpec>> = OnceLock::new();
    TOOLS.get_or_init(build_tools)
}

/// Build the full MCP tool catalog from the live OpenAPI document.
///
/// This walks the serialized spec as JSON so it is robust to the exact utoipa
/// type shapes and naturally preserves `$ref`s; the resolved component schemas
/// are bundled under `components` in each tool's input/output schema so refs
/// resolve.
fn build_tools() -> Vec<ToolSpec> {
    let spec = beater_api::openapi::openapi();
    // Serialize once; treat the spec as plain JSON from here on.
    let doc: Value = serde_json::to_value(&spec).unwrap_or(Value::Null);

    // Component schemas, bundled into input/output schemas so their refs resolve.
    let component_schemas = doc
        .get("components")
        .and_then(|c| c.get("schemas"))
        .cloned();

    let mut tools = Vec::new();

    for op in beater_api::openapi::operations(&doc) {
        // Only expose versioned API operations as tools.
        if !op.path.starts_with("/v1") {
            continue;
        }
        let operation = op.operation;

        let description = operation
            .get("summary")
            .and_then(Value::as_str)
            .or_else(|| operation.get("description").and_then(Value::as_str))
            .unwrap_or(op.operation_id)
            .to_string();

        let (input_schema, path_params, query_params, has_body) =
            build_input_schema(operation, component_schemas.as_ref());
        let output_schema = build_output_schema(operation, component_schemas.as_ref());

        tools.push(ToolSpec {
            name: op.operation_id.to_string(),
            method: op.method.to_ascii_uppercase(),
            path_template: op.path.to_string(),
            input_schema,
            output_schema,
            description,
            path_params,
            query_params,
            has_body,
        });
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
    // Bundle component schemas so any `$ref` inside the body schema resolves.
    if has_body {
        bundle_component_schemas(&mut schema, component_schemas);
    }

    (Value::Object(schema), path_params, query_params, has_body)
}

/// Build the `outputSchema` for one operation from its lowest 2xx
/// `application/json` response schema, bundling the component schemas so any
/// `$ref` resolves from the schema root.
///
/// Returns `None` unless the success body is **object-rooted**: MCP constrains
/// `outputSchema` (and the `structuredContent` it describes) to a JSON object,
/// so array- or scalar-rooted results (e.g. `Vec<T>` list endpoints) advertise
/// no output schema rather than an invalid one a strict client could reject.
fn build_output_schema(
    op: &Map<String, Value>,
    component_schemas: Option<&Value>,
) -> Option<Value> {
    let responses = op.get("responses")?.as_object()?;
    // Sort so the numerically smallest 2xx status wins (e.g. `200` over `201`),
    // giving one canonical success shape even if several are declared.
    let mut codes: Vec<&String> = responses.keys().filter(|c| c.starts_with('2')).collect();
    codes.sort();
    for code in codes {
        let Some(schema) = responses
            .get(code)
            .and_then(|r| r.get("content"))
            .and_then(|c| c.get("application/json"))
            .and_then(|j| j.get("schema"))
        else {
            continue;
        };
        if !resolves_to_object(schema, component_schemas) {
            return None;
        }
        let mut out = schema.clone();
        // A 2xx schema is often a bare `{ "$ref": "#/components/schemas/X" }`;
        // bundle the schemas as a sibling so the ref resolves from this root.
        if let Some(obj) = out.as_object_mut() {
            bundle_component_schemas(obj, component_schemas);
        }
        return Some(out);
    }
    None
}

/// Whether a schema is object-rooted, resolving one level of `$ref` against the
/// component schemas. Inline schemas are judged by their declared `type`; a
/// `$ref` is resolved and judged by its target's `type`/`properties`. Anything
/// indeterminate is treated as non-object (conservative: omit over mis-advertise).
fn resolves_to_object(schema: &Value, component_schemas: Option<&Value>) -> bool {
    if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        return ty == "object";
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let name = reference.rsplit('/').next().unwrap_or_default();
        if let Some(target) = component_schemas.and_then(|s| s.get(name)) {
            return match target.get("type").and_then(Value::as_str) {
                Some(ty) => ty == "object",
                None => target.get("properties").is_some(),
            };
        }
    }
    false
}

/// Bundle the resolved component schemas into a schema root so any `$ref`
/// resolves. The spec's refs use the OpenAPI pointer space
/// (`#/components/schemas/...`), so that is the single location we expose;
/// JSON Schema 2020-12 permits these sibling keywords alongside a root `$ref`.
fn bundle_component_schemas(target: &mut Map<String, Value>, component_schemas: Option<&Value>) {
    if let Some(schemas) = component_schemas {
        target.insert(
            "components".to_string(),
            json!({ "schemas": schemas.clone() }),
        );
    }
}

/// Serialize a tool to its MCP `tools/list` JSON shape: name, description,
/// input/output schemas, and method-derived behavioural annotations.
fn tool_to_json(tool: &ToolSpec) -> Value {
    let mut obj = Map::new();
    obj.insert("name".to_string(), Value::String(tool.name.clone()));
    obj.insert(
        "description".to_string(),
        Value::String(tool.description.clone()),
    );
    obj.insert("inputSchema".to_string(), tool.input_schema.clone());
    if let Some(output) = &tool.output_schema {
        obj.insert("outputSchema".to_string(), output.clone());
    }
    obj.insert(
        "annotations".to_string(),
        annotations_for(&tool.method, &tool.description),
    );
    Value::Object(obj)
}

/// Behavioural hints derived purely from the HTTP method, per the MCP tool
/// annotations contract: `GET` is read-only; `PUT`/`DELETE` are idempotent and
/// may overwrite/remove state, so they are flagged destructive; `POST` is a
/// non-idempotent, non-destructive write. These are advisory hints clients use
/// to gate or batch calls — they are not a security boundary.
fn annotations_for(method: &str, title: &str) -> Value {
    let read_only = method == "GET";
    let idempotent = matches!(method, "GET" | "PUT" | "DELETE");
    let destructive = matches!(method, "PUT" | "DELETE");
    json!({
        "title": title,
        "readOnlyHint": read_only,
        "destructiveHint": destructive,
        "idempotentHint": idempotent,
    })
}

/// Name of the one synthetic, non-spec meta tool. Every other tool maps 1:1 to a
/// `/v1` operation derived from the spec; `help` is the deliberate exception — a
/// local discovery aid that never dispatches to the API. It is therefore added at
/// the `tools/list`/`tools/call` layer and is intentionally absent from
/// [`tool_names`]/[`tools`], which remain the authoritative spec-coverage answer.
const HELP_TOOL_NAME: &str = "help";

/// The `tools/list` entry for the synthetic [`HELP_TOOL_NAME`] tool.
fn help_tool_json() -> Value {
    json!({
        "name": HELP_TOOL_NAME,
        "description": "Discover Beater MCP tools. Call with no arguments for an \
            overview of every tool, `query` to filter by keyword, or `tool` to get \
            one operation's full input/output schema and annotations. Resolved \
            locally — this makes no API call.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Case-insensitive keyword to filter tools by name or description."
                },
                "tool": {
                    "type": "string",
                    "description": "An operationId to describe in full (method, path, schemas, annotations)."
                }
            }
        },
        "outputSchema": {
            "type": "object",
            "properties": {
                "server": { "type": "object" },
                "protocolVersion": { "type": "string" },
                "toolCount": { "type": "integer" },
                "tools": { "type": "array" },
                "tool": { "type": "object" }
            }
        },
        // Pure read of static, in-process data: read-only and idempotent.
        "annotations": annotations_for("GET", "Discover Beater MCP tools"),
    })
}

/// Compact one-line view of a tool for catalog/overview listings.
fn tool_summary(tool: &ToolSpec) -> Value {
    json!({
        "name": tool.name,
        "method": tool.method,
        "description": tool.description,
    })
}

/// Full descriptor of one tool: its `tools/list` shape plus the underlying HTTP
/// method and path template.
fn describe_tool(tool: &ToolSpec) -> Value {
    let mut value = tool_to_json(tool);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("method".to_string(), Value::String(tool.method.clone()));
        obj.insert(
            "path".to_string(),
            Value::String(tool.path_template.clone()),
        );
    }
    value
}

/// Handle a `tools/call` for the synthetic `help` tool. Read-only and local: it
/// never dispatches to the API. With `tool` it returns that operation's full
/// descriptor; with `query` it returns matching tools; otherwise an overview of
/// the whole catalog. The structured result is always a JSON object.
fn handle_help(arguments: &Map<String, Value>) -> Result<Value, (i64, String)> {
    let structured = if let Some(name) = arguments.get("tool").and_then(Value::as_str) {
        let tool = tools()
            .iter()
            .find(|t| t.name == name)
            .ok_or((INVALID_PARAMS, format!("unknown tool: {name}")))?;
        json!({ "tool": describe_tool(tool) })
    } else {
        let query = arguments
            .get("query")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase);
        let matches: Vec<Value> = tools()
            .iter()
            .filter(|t| match &query {
                Some(q) => {
                    t.name.to_ascii_lowercase().contains(q)
                        || t.description.to_ascii_lowercase().contains(q)
                }
                None => true,
            })
            .map(tool_summary)
            .collect();
        json!({
            "server": server_info(),
            "protocolVersion": PROTOCOL_VERSION,
            "toolCount": matches.len(),
            "tools": matches,
        })
    };

    // Mirror the `tools/call` result shape (text + structured), minus the HTTP
    // `_meta` since no request was dispatched.
    let text = serde_json::to_string_pretty(&structured).unwrap_or_else(|_| structured.to_string());
    Ok(json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": structured,
        "isError": false,
    }))
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

/// Serve MCP JSON-RPC over stdin/stdout.
///
/// The stdio transport is line-delimited JSON-RPC: each non-empty stdin line is
/// one request, and every response is written as exactly one stdout line. All
/// diagnostic output belongs on stderr in the caller so stdout remains valid MCP
/// traffic for local clients.
pub async fn serve_stdio(state: ApiState) -> std::io::Result<()> {
    serve_stdio_streams(state, tokio::io::stdin(), tokio::io::stdout()).await
}

/// Testable stdio transport implementation over arbitrary async streams.
pub async fn serve_stdio_streams<R, W>(
    state: ApiState,
    input: R,
    mut output: W,
) -> std::io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = BufReader::new(input).lines();
    let headers = HeaderMap::new();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(&line) {
            Ok(request) => dispatch_rpc(&state, &headers, &request).await,
            Err(_) => Some(rpc_error(Value::Null, PARSE_ERROR, "invalid JSON")),
        };
        if let Some(response) = response {
            output.write_all(response.to_string().as_bytes()).await?;
            output.write_all(b"\n").await?;
            output.flush().await?;
        }
    }

    output.flush().await
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
    let mut resp = match response {
        Some(resp) => json_response(resp),
        // Notification (no id) — respond 202 with empty body.
        None => StatusCode::ACCEPTED.into_response(),
    };
    // MCP OAuth discovery (RFC 9728): when the caller presented no credentials
    // and the server advertises OAuth, point it at the protected-resource
    // metadata via a `WWW-Authenticate` challenge so it can start the flow.
    if let Some(url) = state.oauth_metadata_url() {
        let has_creds = headers.contains_key(http::header::AUTHORIZATION)
            || headers.contains_key("x-beater-api-key");
        if !has_creds {
            if let Ok(value) = format!("Bearer resource_metadata=\"{url}\"").parse() {
                resp.headers_mut()
                    .insert(http::header::WWW_AUTHENTICATE, value);
            }
        }
    }
    resp
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
        "initialize" => {
            // Negotiate: echo the client's requested version when we support it,
            // otherwise advertise our latest.
            let version = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .filter(|v| SUPPORTED_PROTOCOL_VERSIONS.contains(v))
                .unwrap_or(PROTOCOL_VERSION);
            Ok(json!({
                "protocolVersion": version,
                "serverInfo": server_info(),
                "capabilities": capabilities(),
            }))
        }
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
            let mut list: Vec<Value> = tools().iter().map(tool_to_json).collect();
            // Append the one synthetic meta tool after the spec-derived tools.
            list.push(help_tool_json());
            Ok(json!({ "tools": list }))
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

    // The synthetic `help` tool is served locally and never hits the API.
    if name == HELP_TOOL_NAME {
        return handle_help(&arguments);
    }

    let tool = tools()
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
    for header_name in [
        http::header::AUTHORIZATION.as_str(),
        "x-beater-api-key",
        "x-beater-project-id",
        "x-beater-environment-id",
    ] {
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
    // MCP requires `structuredContent` to be a JSON object; array/scalar bodies
    // (e.g. list endpoints) are conveyed via the text content only. The full
    // body is always present in `content` regardless.
    if structured.is_object() {
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

/// The set of MCP tool names exposed, for tests and introspection. This is the
/// authoritative "what does the MCP surface cover" answer, derived from the spec.
pub fn tool_names() -> Vec<String> {
    tools().iter().map(|t| t.name.clone()).collect()
}

#[cfg(test)]
mod tests {
    // Test code uses unwrap/expect freely on known-good fixtures.
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    /// Method-derived annotations follow HTTP semantics for every method,
    /// including `PUT`/`DELETE` (which the spec does not currently use, so they
    /// are only exercised here).
    #[test]
    fn annotations_track_http_method_semantics() {
        let cases = [
            // (method, read_only, destructive, idempotent)
            ("GET", true, false, true),
            ("POST", false, false, false),
            ("PUT", false, true, true),
            ("DELETE", false, true, true),
        ];
        for (method, read_only, destructive, idempotent) in cases {
            let a = annotations_for(method, "do a thing");
            assert_eq!(a["title"], "do a thing", "{method}: title carried through");
            assert_eq!(a["readOnlyHint"], read_only, "{method}: readOnlyHint");
            assert_eq!(
                a["destructiveHint"], destructive,
                "{method}: destructiveHint"
            );
            assert_eq!(a["idempotentHint"], idempotent, "{method}: idempotentHint");
        }
    }

    /// A read-only method is never also flagged destructive (the two hints must
    /// not contradict for any method we emit).
    #[test]
    fn read_only_is_never_destructive() {
        for method in ["GET", "POST", "PUT", "DELETE"] {
            let a = annotations_for(method, "t");
            if a["readOnlyHint"] == Value::Bool(true) {
                assert_eq!(a["destructiveHint"], Value::Bool(false), "{method}");
            }
        }
    }

    /// `bundle_component_schemas` exposes the resolved schemas under the OpenAPI
    /// pointer space the spec's refs use, and nothing else. Without component
    /// schemas it is a no-op.
    #[test]
    fn bundle_exposes_components_only() {
        let schemas = json!({ "Foo": { "type": "object" } });
        let mut target = Map::new();
        bundle_component_schemas(&mut target, Some(&schemas));
        assert_eq!(target["components"]["schemas"]["Foo"]["type"], "object");
        // No dead `$defs` duplicate — the spec's refs never point there.
        assert!(target.get("$defs").is_none(), "no redundant $defs key");

        let mut empty = Map::new();
        bundle_component_schemas(&mut empty, None);
        assert!(empty.is_empty(), "no schemas -> no keys added");
    }

    /// The catalog is built once and the cached slice is identical across calls.
    #[test]
    fn tools_catalog_is_cached_and_stable() {
        let first = tools();
        let second = tools();
        assert!(
            std::ptr::eq(first, second),
            "tools() must return the same cached slice"
        );
        assert_eq!(first.len(), second.len());
    }

    /// `resolves_to_object` resolves inline types and one level of `$ref`, and is
    /// conservative (false) for arrays, scalars, and danging refs.
    #[test]
    fn resolves_to_object_handles_inline_and_refs() {
        let comps = json!({
            "Obj": { "type": "object" },
            "Bag": { "properties": { "a": {} } }, // object by shape, no explicit type
            "List": { "type": "array" },
        });
        let c = Some(&comps);
        assert!(resolves_to_object(&json!({ "type": "object" }), c));
        assert!(!resolves_to_object(&json!({ "type": "array" }), c));
        assert!(resolves_to_object(
            &json!({ "$ref": "#/components/schemas/Obj" }),
            c
        ));
        assert!(resolves_to_object(
            &json!({ "$ref": "#/components/schemas/Bag" }),
            c
        ));
        assert!(!resolves_to_object(
            &json!({ "$ref": "#/components/schemas/List" }),
            c
        ));
        assert!(!resolves_to_object(
            &json!({ "$ref": "#/components/schemas/Missing" }),
            c
        ));
    }

    /// `build_output_schema`: lowest 2xx wins, only object-rooted bodies are
    /// emitted (with components bundled), and absent/array/scalar bodies → None.
    #[test]
    fn build_output_schema_selects_and_filters() {
        let comps = json!({ "A": { "type": "object" }, "B": { "type": "object" } });
        let json_resp = |r: Value| json!({ "content": { "application/json": { "schema": r } } });

        // Two 2xx codes declared -> the lower (200) wins.
        let multi = json!({
            "responses": {
                "201": json_resp(json!({ "$ref": "#/components/schemas/B" })),
                "200": json_resp(json!({ "$ref": "#/components/schemas/A" })),
            }
        });
        let out = build_output_schema(multi.as_object().unwrap(), Some(&comps))
            .expect("object-rooted 200 is emitted");
        assert_eq!(out["$ref"], "#/components/schemas/A", "lowest 2xx wins");
        assert_eq!(
            out["components"]["schemas"]["A"]["type"], "object",
            "components bundled so the ref resolves"
        );

        // Array-rooted success body -> no output schema.
        let array = json!({ "responses": { "200": json_resp(json!({ "type": "array" })) } });
        assert!(build_output_schema(array.as_object().unwrap(), Some(&comps)).is_none());

        // No application/json success body -> None.
        let empty = json!({ "responses": { "204": { "description": "no content" } } });
        assert!(build_output_schema(empty.as_object().unwrap(), Some(&comps)).is_none());
    }

    /// `tool_to_json` omits `outputSchema` entirely when the tool has none, and
    /// includes it (plus annotations) when present.
    #[test]
    fn tool_to_json_omits_absent_output_schema() {
        let mut tool = ToolSpec {
            name: "x".into(),
            method: "POST".into(),
            path_template: "/v1/x".into(),
            input_schema: json!({ "type": "object" }),
            output_schema: None,
            description: "x".into(),
            path_params: vec![],
            query_params: vec![],
            has_body: false,
        };
        let without = tool_to_json(&tool);
        assert!(without.get("outputSchema").is_none(), "omitted when None");
        assert!(
            without["annotations"].is_object(),
            "annotations always present"
        );

        tool.output_schema = Some(json!({ "type": "object" }));
        let with = tool_to_json(&tool);
        assert_eq!(with["outputSchema"]["type"], "object");
    }

    /// Every emitted output schema is object-rooted; the known array-returning
    /// list operations advertise none. Guards the MCP object-only invariant.
    #[test]
    fn output_schemas_are_object_rooted_or_absent() {
        let array_ops = [
            "listAuditEvents",
            "listJudgeLedger",
            "listProviderSecrets",
            "listReviewTasks",
        ];
        for tool in tools() {
            match &tool.output_schema {
                None => assert!(
                    array_ops.contains(&tool.name.as_str()),
                    "{} unexpectedly has no output schema",
                    tool.name
                ),
                Some(schema) => {
                    // Resolve the root type against the bundled components.
                    let comps = schema.get("components").and_then(|c| c.get("schemas"));
                    assert!(
                        resolves_to_object(schema, comps),
                        "{}: emitted output schema must be object-rooted",
                        tool.name
                    );
                }
            }
        }
        // And the array ops are definitely omitted.
        for name in array_ops {
            let tool = tools().iter().find(|t| t.name == name).unwrap();
            assert!(
                tool.output_schema.is_none(),
                "{name} returns an array and must not advertise an output schema"
            );
        }
    }

    /// The synthetic `help` tool advertises read-only/idempotent annotations and
    /// the documented input/output schema shape.
    #[test]
    fn help_tool_json_is_well_formed() {
        let help = help_tool_json();
        assert_eq!(help["name"], HELP_TOOL_NAME);
        assert_eq!(help["inputSchema"]["type"], "object");
        assert!(help["inputSchema"]["properties"]["query"].is_object());
        assert!(help["inputSchema"]["properties"]["tool"].is_object());
        assert_eq!(help["outputSchema"]["type"], "object");
        assert_eq!(help["annotations"]["readOnlyHint"], true);
        assert_eq!(help["annotations"]["idempotentHint"], true);
        assert_eq!(help["annotations"]["destructiveHint"], false);
    }

    /// `handle_help` resolves locally to object structured content: overview,
    /// keyword filter, and single-tool description; unknown ops are an error.
    #[test]
    fn handle_help_modes() {
        let total = tools().len();

        // Overview (no args).
        let overview = handle_help(&Map::new()).unwrap();
        let s = &overview["structuredContent"];
        assert!(s.is_object());
        assert_eq!(overview["isError"], false);
        assert_eq!(s["toolCount"], total);
        assert_eq!(s["tools"].as_array().unwrap().len(), total);

        // Query filter narrows the set and matches name/description.
        let mut args = Map::new();
        args.insert("query".to_string(), json!("trace"));
        let filtered = handle_help(&args).unwrap();
        let matched = filtered["structuredContent"]["tools"].as_array().unwrap();
        assert!(!matched.is_empty() && matched.len() < total);
        for entry in matched {
            let n = entry["name"].as_str().unwrap().to_ascii_lowercase();
            let d = entry["description"].as_str().unwrap().to_ascii_lowercase();
            assert!(n.contains("trace") || d.contains("trace"));
        }

        // Describe one tool.
        let mut one = Map::new();
        one.insert("tool".to_string(), json!("listTraces"));
        let described = handle_help(&one).unwrap();
        let t = &described["structuredContent"]["tool"];
        assert_eq!(t["name"], "listTraces");
        assert_eq!(t["method"], "GET");
        assert!(t["path"].as_str().unwrap().starts_with("/v1/"));

        // Unknown op -> error.
        let mut bad = Map::new();
        bad.insert("tool".to_string(), json!("does-not-exist"));
        assert!(handle_help(&bad).is_err());
    }
}
