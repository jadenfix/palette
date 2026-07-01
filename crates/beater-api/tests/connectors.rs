//! Integration tests for the `/v1/connectors` surface (Composio-backed
//! third-party tools) and the RSI use cases it enables.
//!
//! These use an in-memory [`FakeComposio`] client so they exercise the full
//! Beater wiring (router → auth → handler → connector trait → response) with no
//! network. The live wire contract is covered by `beater-composio`'s
//! `COMPOSIO_API_KEY`-gated tests.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use beater_api::{router, ApiState};
use beater_bus::InMemoryBus;
use beater_composio::{
    skill, ComposioClient, ComposioError, ConnectionLink, ConnectionStatus, ConnectorTool,
    ToolExecution, Toolkit,
};
use beater_ingest::{IngestPolicy, IngestService};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

/// In-memory connector provider that records calls for assertions.
#[derive(Default)]
struct FakeComposio {
    calls: Mutex<Vec<String>>,
}

impl FakeComposio {
    fn record(&self, call: impl Into<String>) {
        self.calls.lock().unwrap().push(call.into());
    }
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
    fn issue_tool(toolkit: &str) -> ConnectorTool {
        ConnectorTool {
            slug: "GITHUB_CREATE_AN_ISSUE".to_string(),
            name: "Create an issue".to_string(),
            description: Some("Open a new GitHub issue.".to_string()),
            no_auth: false,
            toolkit: Some(toolkit.to_string()),
            tags: vec!["important".to_string()],
            input_schema: Some(json!({
                "type": "object",
                "required": ["title"],
                "properties": { "title": {"type": "string", "description": "Issue title"} }
            })),
        }
    }
}

#[async_trait]
impl ComposioClient for FakeComposio {
    async fn list_toolkits(&self, limit: u32) -> Result<Vec<Toolkit>, ComposioError> {
        self.record(format!("list_toolkits:{limit}"));
        Ok(vec![Toolkit {
            slug: "github".to_string(),
            name: "GitHub".to_string(),
            description: Some("Git hosting".to_string()),
            tools_count: Some(61),
            no_auth: false,
            auth_schemes: vec!["OAUTH2".to_string()],
        }])
    }

    async fn list_tools(
        &self,
        toolkit_slug: &str,
        limit: u32,
    ) -> Result<Vec<ConnectorTool>, ComposioError> {
        self.record(format!("list_tools:{toolkit_slug}:{limit}"));
        Ok(vec![Self::issue_tool(toolkit_slug)])
    }

    async fn get_tool(&self, tool_slug: &str) -> Result<ConnectorTool, ComposioError> {
        self.record(format!("get_tool:{tool_slug}"));
        Ok(Self::issue_tool("github"))
    }

    async fn connect(
        &self,
        toolkit_slug: &str,
        user_id: &str,
    ) -> Result<ConnectionLink, ComposioError> {
        self.record(format!("connect:{toolkit_slug}:{user_id}"));
        Ok(ConnectionLink {
            redirect_url: "https://connect.composio.dev/link/lk_test".to_string(),
            connected_account_id: "ca_test".to_string(),
            expires_at: None,
        })
    }

    async fn connection_status(
        &self,
        toolkit_slug: &str,
        user_id: &str,
    ) -> Result<ConnectionStatus, ComposioError> {
        self.record(format!("status:{toolkit_slug}:{user_id}"));
        Ok(ConnectionStatus::not_connected(toolkit_slug))
    }

    async fn execute(
        &self,
        tool_slug: &str,
        user_id: &str,
        arguments: Value,
    ) -> Result<ToolExecution, ComposioError> {
        self.record(format!("execute:{tool_slug}:{user_id}:{arguments}"));
        Ok(ToolExecution {
            successful: true,
            data: json!({ "echo": arguments }),
            error: None,
            log_id: Some("log_test".to_string()),
        })
    }
}

fn state_with(connectors: Option<Arc<FakeComposio>>) -> ApiState {
    // `into_path()` keeps the temp dir alive for the test process; connector
    // endpoints never touch artifacts, so cleanup timing is irrelevant.
    let dir = tempfile::tempdir().unwrap().keep();
    let artifacts = Arc::new(FsArtifactStore::new(dir.join("artifacts")).unwrap());
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap());
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let mut state = ApiState::new(ingest, traces);
    if let Some(c) = connectors {
        state = state.with_connectors(c as Arc<dyn ComposioClient>);
    }
    state
}

async fn send(
    state: ApiState,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let app = router(state);
    let req = Request::builder().method(method).uri(uri);
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&b).unwrap())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

#[tokio::test]
async fn lists_connector_catalog() {
    let (status, body) = send(
        state_with(Some(Arc::new(FakeComposio::default()))),
        "GET",
        "/v1/connectors/acme/proj",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let toolkits: Vec<Toolkit> = serde_json::from_value(body).unwrap();
    assert_eq!(toolkits[0].slug, "github");
    assert_eq!(toolkits[0].tools_count, Some(61));
}

#[tokio::test]
async fn lists_tools_with_input_schema() {
    let (status, body) = send(
        state_with(Some(Arc::new(FakeComposio::default()))),
        "GET",
        "/v1/connectors/acme/proj/tools?toolkit=github",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tools: Vec<ConnectorTool> = serde_json::from_value(body).unwrap();
    assert_eq!(tools[0].slug, "GITHUB_CREATE_AN_ISSUE");
    // The agent loop needs the schema to construct a valid call.
    assert_eq!(
        tools[0].input_schema.as_ref().unwrap()["properties"]["title"]["type"],
        "string"
    );
}

#[tokio::test]
async fn generates_skill_scaffold() {
    let (status, body) = send(
        state_with(Some(Arc::new(FakeComposio::default()))),
        "GET",
        "/v1/connectors/acme/proj/skills?toolkit=github",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["toolkit"], "github");
    let skills = body["skills"].as_str().unwrap();
    // The padded prompting context the agent splices into its system prompt.
    assert!(skills.contains("GITHUB_CREATE_AN_ISSUE"));
    assert!(skills.contains("When to use:"));
    assert!(skills.contains(skill::INVOKE_OPERATION));
    assert!(skills.contains("`title` (string, required)"));
}

#[tokio::test]
async fn connect_returns_login_link_with_project_entity() {
    let fake = Arc::new(FakeComposio::default());
    let (status, body) = send(
        state_with(Some(fake.clone())),
        "POST",
        "/v1/connectors/acme/proj/connect",
        Some(json!({ "toolkit": "github" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let link: ConnectionLink = serde_json::from_value(body).unwrap();
    assert!(link
        .redirect_url
        .starts_with("https://connect.composio.dev/link/"));
    // The Composio entity must be the per-project id.
    assert!(fake
        .calls()
        .iter()
        .any(|c| c == "connect:github:beater:acme:proj"));
}

#[tokio::test]
async fn reports_connection_status() {
    let (status, body) = send(
        state_with(Some(Arc::new(FakeComposio::default()))),
        "GET",
        "/v1/connectors/acme/proj/status?toolkit=github",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let st: ConnectionStatus = serde_json::from_value(body).unwrap();
    assert_eq!(st.toolkit, "github");
    assert!(!st.connected);
}

#[tokio::test]
async fn invokes_tool_and_scopes_entity_per_project() {
    let fake = Arc::new(FakeComposio::default());
    let (status, body) = send(
        state_with(Some(fake.clone())),
        "POST",
        "/v1/connectors/acme/proj/invoke",
        Some(json!({ "tool": "GITHUB_CREATE_AN_ISSUE", "arguments": { "title": "bug" } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let exec: ToolExecution = serde_json::from_value(body).unwrap();
    assert!(exec.successful);
    assert_eq!(exec.log_id.as_deref(), Some("log_test"));
    // Verify the handler forwarded tool, project-scoped entity, and arguments.
    assert!(fake.calls().iter().any(|c| c
        .starts_with("execute:GITHUB_CREATE_AN_ISSUE:beater:acme:proj:")
        && c.contains("\"title\":\"bug\"")));
}

#[tokio::test]
async fn returns_501_when_connectors_unconfigured() {
    // No COMPOSIO_API_KEY → provider absent → endpoints are not implemented.
    let (status, _) = send(state_with(None), "GET", "/v1/connectors/acme/proj", None).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);

    let (status, _) = send(
        state_with(None),
        "POST",
        "/v1/connectors/acme/proj/invoke",
        Some(json!({ "tool": "X", "arguments": {} })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn missing_toolkit_query_is_rejected() {
    // `toolkit` is a required query param for tool/skill/status listing.
    let (status, _) = send(
        state_with(Some(Arc::new(FakeComposio::default()))),
        "GET",
        "/v1/connectors/acme/proj/tools",
        None,
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 4xx for missing toolkit, got {status}"
    );
}

/// RSI use case: the meta-loop discovers a capability gap, picks a Composio
/// tool, and lands it in the agent's `tool_set` as a complete `tools.json`
/// entry (schema + skill card), then the agent executes it. This test walks
/// that chain through the public API + the `skill` emitter.
#[tokio::test]
async fn rsi_tool_add_then_execute_flow() {
    let fake = Arc::new(FakeComposio::default());

    // 1. Discover tools for a toolkit the loop wants to add.
    let (status, body) = send(
        state_with(Some(fake.clone())),
        "GET",
        "/v1/connectors/acme/proj/tools?toolkit=github",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tools: Vec<ConnectorTool> = serde_json::from_value(body).unwrap();
    let chosen = &tools[0];

    // 2. Emit the tools.json entry the RSI ToolAdd writes — must be complete.
    let entry = skill::tool_definition_json(chosen);
    assert_eq!(entry["name"], "GITHUB_CREATE_AN_ISSUE");
    assert_eq!(entry["source"], "composio");
    assert_eq!(
        entry["symbol"],
        "invokeConnectorTool(GITHUB_CREATE_AN_ISSUE)"
    );
    assert!(entry["input_schema"]["properties"]["title"].is_object());
    assert!(entry["skill_card"]
        .as_str()
        .unwrap()
        .contains("When to use:"));

    // 3. The agent now executes the freshly-added tool via the same surface.
    let (status, body) = send(
        state_with(Some(fake.clone())),
        "POST",
        "/v1/connectors/acme/proj/invoke",
        Some(json!({
            "tool": entry["name"],
            "arguments": { "title": "RSI-added issue" }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let exec: ToolExecution = serde_json::from_value(body).unwrap();
    assert!(exec.successful);
}
