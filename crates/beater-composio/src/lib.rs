//! Thin async client for the [Composio](https://composio.dev) v3 API.
//!
//! Composio brokers managed-OAuth connections and tool execution for 250+
//! third-party apps (Gmail, GitHub, Slack, web search, …). Beater wraps it so
//! that an agent under evaluation — or the RSI loop's `tool_set` lever — can
//! reach those tools through Beater's own `/v1/connectors` surface (and, by
//! extension, the MCP `invokeConnectorTool` tool, since every `/v1` operation
//! becomes an MCP tool).
//!
//! Auth model that this wraps (verified against the live API):
//!
//! * A Composio **entity** (`user_id`) owns connections. Beater keys it per
//!   project: `beater:{tenant_id}:{project_id}`.
//! * Connecting an app is a one-time managed-OAuth handshake — [`connect`]
//!   returns a `redirect_url` the end user opens once; Composio persists and
//!   refreshes the grant, so later [`execute`] calls never prompt again.
//! * [`execute`] runs a tool by slug for an entity and returns Composio's
//!   `{successful, data, error}` envelope verbatim.
//!
//! The [`ComposioClient`] trait keeps the HTTP impl swappable so the API layer
//! can be tested against an in-memory fake with no network.
//!
//! [`connect`]: ComposioClient::connect
//! [`execute`]: ComposioClient::execute

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

pub mod policy;
pub mod skill;

pub use policy::{
    classify_connector_tool, ConnectorToolPolicy, ConnectorToolPolicyDecision,
    ConnectorToolRiskClass,
};

/// Default Composio v3 API base URL.
pub const DEFAULT_BASE_URL: &str = "https://backend.composio.dev/api/v3";

/// Environment variable Beater reads to enable the Composio integration. When
/// unset, the `/v1/connectors` endpoints report "not configured" (501).
pub const API_KEY_ENV: &str = "COMPOSIO_API_KEY";

/// Errors surfaced by a [`ComposioClient`].
#[derive(Debug, thiserror::Error)]
pub enum ComposioError {
    /// Transport-level failure (DNS, TLS, timeout, connection reset).
    #[error("composio transport error: {0}")]
    Transport(String),
    /// The API returned a non-success status. `message` is Composio's own error
    /// message when the body parsed, else the raw body.
    #[error("composio api error ({status}): {message}")]
    Api {
        /// HTTP status code from Composio.
        status: u16,
        /// Best-effort human-readable message.
        message: String,
    },
    /// A success body failed to deserialize into the expected shape.
    #[error("composio decode error: {0}")]
    Decode(String),
}

/// A connectable third-party app (Composio "toolkit"), flattened from the v3
/// `GET /toolkits` shape into the fields Beater exposes.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Toolkit {
    /// Stable slug used everywhere else (e.g. `github`, `gmail`).
    pub slug: String,
    /// Human display name.
    pub name: String,
    /// Short description, if the catalog provides one.
    #[serde(default)]
    pub description: Option<String>,
    /// Number of tools the toolkit exposes, if known.
    #[serde(default)]
    pub tools_count: Option<u32>,
    /// `true` when the toolkit needs no OAuth/connection to execute.
    #[serde(default)]
    pub no_auth: bool,
    /// Supported auth schemes (e.g. `OAUTH2`, `API_KEY`, `NO_AUTH`).
    #[serde(default)]
    pub auth_schemes: Vec<String>,
}

/// A single executable tool within a toolkit, carrying the metadata an agent
/// needs to actually *call* it: the input JSON Schema, tags, and toolkit. This
/// is the raw material for the prompting scaffold in [`crate::skill`].
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ConnectorTool {
    /// Tool slug passed to [`ComposioClient::execute`] (e.g. `GITHUB_CREATE_AN_ISSUE`).
    pub slug: String,
    /// Human display name.
    pub name: String,
    /// What the tool does.
    #[serde(default)]
    pub description: Option<String>,
    /// `true` when the tool executes without a connected account.
    #[serde(default)]
    pub no_auth: bool,
    /// Owning toolkit slug (e.g. `github`), when known.
    #[serde(default)]
    pub toolkit: Option<String>,
    /// Free-form tags Composio assigns (categories, importance, …).
    #[serde(default)]
    pub tags: Vec<String>,
    /// JSON Schema of the tool's `arguments`, verbatim from Composio. The agent
    /// loop uses this to construct valid calls; [`crate::skill`] renders it.
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub input_schema: Option<Value>,
}

/// One-time login link returned when initiating a managed-OAuth connection.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct ConnectionLink {
    /// URL the end user opens once to authorize the app.
    pub redirect_url: String,
    /// Composio connection id (`ca_…`) created for this handshake.
    pub connected_account_id: String,
    /// When the link expires (RFC 3339), if provided.
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Connection status of one app for one entity.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct ConnectionStatus {
    /// Toolkit slug this status is for.
    pub toolkit: String,
    /// `true` only when an account exists and is `ACTIVE`.
    pub connected: bool,
    /// Raw Composio status (`ACTIVE`, `INITIALIZING`, `FAILED`, …) or
    /// `not_connected` when no account exists yet.
    pub status: String,
    /// The connected-account id, when one exists.
    #[serde(default)]
    pub connected_account_id: Option<String>,
}

impl ConnectionStatus {
    /// Status for a toolkit that has no connection yet.
    pub fn not_connected(toolkit: impl Into<String>) -> Self {
        Self {
            toolkit: toolkit.into(),
            connected: false,
            status: "not_connected".to_string(),
            connected_account_id: None,
        }
    }
}

/// Result of executing a tool — Composio's `{successful, data, error}` envelope.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolExecution {
    /// Whether the tool reported success.
    pub successful: bool,
    /// Tool output payload (shape is tool-specific).
    #[serde(default)]
    #[schema(value_type = Object)]
    pub data: Value,
    /// Error message when `successful` is false.
    #[serde(default)]
    pub error: Option<String>,
    /// Composio execution log id, for tracing.
    #[serde(default)]
    pub log_id: Option<String>,
}

/// Async interface over the Composio operations Beater needs. Implemented by
/// [`HttpComposioClient`] (real) and by test fakes.
#[async_trait]
pub trait ComposioClient: Send + Sync {
    /// List connectable apps (catalog). `limit` caps the page size.
    async fn list_toolkits(&self, limit: u32) -> Result<Vec<Toolkit>, ComposioError>;

    /// List executable tools within a toolkit (including input schemas).
    async fn list_tools(
        &self,
        toolkit_slug: &str,
        limit: u32,
    ) -> Result<Vec<ConnectorTool>, ComposioError>;

    /// Fetch a single tool's full detail (name, description, input schema) by
    /// slug — used to prepare an invocation or build a skill card.
    async fn get_tool(&self, tool_slug: &str) -> Result<ConnectorTool, ComposioError>;

    /// Initiate a managed-OAuth connection for `user_id` to `toolkit_slug`,
    /// returning the one-time login link. Reuses an existing managed auth
    /// config for the toolkit when present, else creates one.
    async fn connect(
        &self,
        toolkit_slug: &str,
        user_id: &str,
    ) -> Result<ConnectionLink, ComposioError>;

    /// Current connection status of `toolkit_slug` for `user_id`.
    async fn connection_status(
        &self,
        toolkit_slug: &str,
        user_id: &str,
    ) -> Result<ConnectionStatus, ComposioError>;

    /// Execute `tool_slug` for `user_id` with `arguments`.
    async fn execute(
        &self,
        tool_slug: &str,
        user_id: &str,
        arguments: Value,
    ) -> Result<ToolExecution, ComposioError>;
}

/// HTTP implementation of [`ComposioClient`] against the Composio v3 REST API.
pub struct HttpComposioClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl HttpComposioClient {
    /// Build a client with the default base URL.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Build a client against a custom base URL (no trailing slash).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Build a client from `COMPOSIO_API_KEY`, returning `None` when unset/empty
    /// so callers can leave the integration disabled.
    pub fn from_env() -> Option<Self> {
        match std::env::var(API_KEY_ENV) {
            Ok(key) if !key.trim().is_empty() => Some(Self::new(key)),
            _ => None,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    /// Send a request and decode the JSON body into `T`, mapping Composio's
    /// error envelope onto [`ComposioError::Api`].
    async fn send<T: for<'de> Deserialize<'de>>(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<T, ComposioError> {
        let resp = builder
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| ComposioError::Transport(e.to_string()))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| ComposioError::Transport(e.to_string()))?;
        if !status.is_success() {
            return Err(ComposioError::Api {
                status: status.as_u16(),
                message: extract_error_message(&body),
            });
        }
        serde_json::from_str(&body).map_err(|e| ComposioError::Decode(e.to_string()))
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.http.get(self.url(path))
    }

    fn post_json(&self, path: &str, body: &Value) -> reqwest::RequestBuilder {
        self.http.post(self.url(path)).json(body)
    }

    /// Find an existing Composio-managed auth config for a toolkit, or create
    /// one. Returns the auth config id (`ac_…`).
    async fn ensure_auth_config(&self, toolkit_slug: &str) -> Result<String, ComposioError> {
        let existing: AuthConfigList = self
            .send(self.get(&format!(
                "auth_configs?toolkit_slug={toolkit_slug}&limit=10"
            )))
            .await?;
        if let Some(id) = existing
            .items
            .into_iter()
            .find(|c| c.is_composio_managed)
            .map(|c| c.id)
        {
            return Ok(id);
        }
        let created: CreateAuthConfigResponse = self
            .send(self.post_json(
                "auth_configs",
                &serde_json::json!({
                    "toolkit": { "slug": toolkit_slug },
                    "auth_config": { "type": "use_composio_managed_auth" },
                }),
            ))
            .await?;
        Ok(created.auth_config.id)
    }
}

#[async_trait]
impl ComposioClient for HttpComposioClient {
    async fn list_toolkits(&self, limit: u32) -> Result<Vec<Toolkit>, ComposioError> {
        let list: ToolkitList = self
            .send(self.get(&format!("toolkits?limit={limit}")))
            .await?;
        Ok(list.items.into_iter().map(Toolkit::from).collect())
    }

    async fn list_tools(
        &self,
        toolkit_slug: &str,
        limit: u32,
    ) -> Result<Vec<ConnectorTool>, ComposioError> {
        let list: ToolList = self
            .send(self.get(&format!("tools?toolkit_slug={toolkit_slug}&limit={limit}")))
            .await?;
        Ok(list.items.into_iter().map(ConnectorTool::from).collect())
    }

    async fn get_tool(&self, tool_slug: &str) -> Result<ConnectorTool, ComposioError> {
        let tool: WireTool = self.send(self.get(&format!("tools/{tool_slug}"))).await?;
        Ok(ConnectorTool::from(tool))
    }

    async fn connect(
        &self,
        toolkit_slug: &str,
        user_id: &str,
    ) -> Result<ConnectionLink, ComposioError> {
        let auth_config_id = self.ensure_auth_config(toolkit_slug).await?;
        let link: ConnectionLink = self
            .send(self.post_json(
                "connected_accounts/link",
                &serde_json::json!({
                    "auth_config_id": auth_config_id,
                    "user_id": user_id,
                }),
            ))
            .await?;
        Ok(link)
    }

    async fn connection_status(
        &self,
        toolkit_slug: &str,
        user_id: &str,
    ) -> Result<ConnectionStatus, ComposioError> {
        let list: ConnectedAccountList = self
            .send(self.get(&format!(
                "connected_accounts?user_ids={user_id}&toolkit_slugs={toolkit_slug}"
            )))
            .await?;
        match list.items.into_iter().next() {
            Some(acct) => Ok(ConnectionStatus {
                toolkit: toolkit_slug.to_string(),
                connected: acct.status.eq_ignore_ascii_case("ACTIVE"),
                status: acct.status,
                connected_account_id: Some(acct.id),
            }),
            None => Ok(ConnectionStatus::not_connected(toolkit_slug)),
        }
    }

    async fn execute(
        &self,
        tool_slug: &str,
        user_id: &str,
        arguments: Value,
    ) -> Result<ToolExecution, ComposioError> {
        let exec: ToolExecution = self
            .send(self.post_json(
                &format!("tools/execute/{tool_slug}"),
                &serde_json::json!({ "user_id": user_id, "arguments": arguments }),
            ))
            .await?;
        Ok(exec)
    }
}

/// Pull a human-readable message out of a Composio error body, falling back to
/// the raw body. Composio uses both `{"error":{"message":...}}` and
/// `{"error":"..."}` shapes.
fn extract_error_message(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "empty response body".to_string();
    }
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if let Some(obj) = v.get("error") {
            if let Some(msg) = obj.get("message").and_then(Value::as_str) {
                return msg.to_string();
            }
            if let Some(msg) = obj.as_str() {
                return msg.to_string();
            }
        }
    }
    trimmed.chars().take(500).collect()
}

// ----- Wire shapes (private; mapped onto the public domain types above) -----

#[derive(Deserialize)]
struct ToolkitList {
    #[serde(default)]
    items: Vec<WireToolkit>,
}

#[derive(Deserialize)]
struct WireToolkit {
    slug: String,
    name: String,
    #[serde(default)]
    no_auth: bool,
    #[serde(default)]
    auth_schemes: Vec<String>,
    #[serde(default)]
    meta: WireToolkitMeta,
}

#[derive(Deserialize, Default)]
struct WireToolkitMeta {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tools_count: Option<u32>,
}

impl From<WireToolkit> for Toolkit {
    fn from(w: WireToolkit) -> Self {
        Toolkit {
            slug: w.slug,
            name: w.name,
            description: w.meta.description,
            tools_count: w.meta.tools_count,
            no_auth: w.no_auth,
            auth_schemes: w.auth_schemes,
        }
    }
}

#[derive(Deserialize)]
struct ToolList {
    #[serde(default)]
    items: Vec<WireTool>,
}

#[derive(Deserialize)]
struct WireTool {
    slug: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    no_auth: bool,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    toolkit: Option<WireToolkitRef>,
    #[serde(default)]
    input_parameters: Option<Value>,
}

#[derive(Deserialize)]
struct WireToolkitRef {
    #[serde(default)]
    slug: Option<String>,
}

impl From<WireTool> for ConnectorTool {
    fn from(w: WireTool) -> Self {
        ConnectorTool {
            slug: w.slug,
            name: w.name,
            description: w.description,
            no_auth: w.no_auth,
            toolkit: w.toolkit.and_then(|t| t.slug),
            tags: w.tags,
            input_schema: w.input_parameters,
        }
    }
}

#[derive(Deserialize)]
struct AuthConfigList {
    #[serde(default)]
    items: Vec<WireAuthConfig>,
}

#[derive(Deserialize)]
struct WireAuthConfig {
    id: String,
    #[serde(default)]
    is_composio_managed: bool,
}

#[derive(Deserialize)]
struct CreateAuthConfigResponse {
    auth_config: WireAuthConfig,
}

#[derive(Deserialize)]
struct ConnectedAccountList {
    #[serde(default)]
    items: Vec<WireConnectedAccount>,
}

#[derive(Deserialize)]
struct WireConnectedAccount {
    id: String,
    #[serde(default)]
    status: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    // Fixtures captured verbatim from the live Composio v3 API.

    #[test]
    fn maps_toolkit_list_from_wire() {
        let body = r#"{"items":[
            {"name":"Gmail","slug":"gmail","auth_schemes":["OAUTH2"],"no_auth":false,
             "meta":{"tools_count":61,"description":"Gmail is Google's email service"}},
            {"name":"Composio","slug":"composio","auth_schemes":["NO_AUTH"],"no_auth":true,
             "meta":{"tools_count":18,"description":"Composio enables AI Agents"}}
        ]}"#;
        let list: ToolkitList = serde_json::from_str(body).unwrap();
        let toolkits: Vec<Toolkit> = list.items.into_iter().map(Toolkit::from).collect();
        assert_eq!(toolkits.len(), 2);
        assert_eq!(toolkits[0].slug, "gmail");
        assert_eq!(toolkits[0].tools_count, Some(61));
        assert!(!toolkits[0].no_auth);
        assert_eq!(toolkits[0].auth_schemes, vec!["OAUTH2"]);
        assert!(toolkits[1].no_auth);
    }

    #[test]
    fn maps_tool_list_from_wire() {
        let body = r#"{"items":[
            {"slug":"GITHUB_CREATE_AN_ISSUE","name":"Create an issue",
             "description":"Open a GitHub issue","no_auth":false,
             "tags":["important"],"toolkit":{"slug":"github"},
             "input_parameters":{"type":"object","required":["title"],
               "properties":{"title":{"type":"string","description":"Issue title"}}}}
        ]}"#;
        let list: ToolList = serde_json::from_str(body).unwrap();
        let tools: Vec<ConnectorTool> = list.items.into_iter().map(ConnectorTool::from).collect();
        assert_eq!(tools[0].slug, "GITHUB_CREATE_AN_ISSUE");
        assert!(!tools[0].no_auth);
        assert_eq!(tools[0].toolkit.as_deref(), Some("github"));
        assert_eq!(tools[0].tags, vec!["important"]);
        // The input schema must round-trip so the agent loop can build a call.
        let schema = tools[0].input_schema.as_ref().expect("schema present");
        assert_eq!(schema["properties"]["title"]["type"], "string");
    }

    #[test]
    fn parses_connection_link() {
        let body = r#"{"link_token":"lk_x","redirect_url":"https://connect.composio.dev/link/lk_x",
            "expires_at":"2026-06-28T01:01:50.471Z","connected_account_id":"ca_55T05PSMifi1"}"#;
        let link: ConnectionLink = serde_json::from_str(body).unwrap();
        assert_eq!(link.connected_account_id, "ca_55T05PSMifi1");
        assert!(link
            .redirect_url
            .starts_with("https://connect.composio.dev/link/"));
        assert_eq!(link.expires_at.as_deref(), Some("2026-06-28T01:01:50.471Z"));
    }

    #[test]
    fn parses_tool_execution_envelope() {
        let ok = r#"{"data":{"results":[1,2]},"successful":true,"error":null,"log_id":"log_x"}"#;
        let exec: ToolExecution = serde_json::from_str(ok).unwrap();
        assert!(exec.successful);
        assert_eq!(exec.log_id.as_deref(), Some("log_x"));

        let fail = r#"{"data":{"message":"key missing"},"successful":false,
            "error":"COMPOSIO_SERPAPI_API_KEY is not set","log_id":"log_y"}"#;
        let exec: ToolExecution = serde_json::from_str(fail).unwrap();
        assert!(!exec.successful);
        assert!(exec.error.unwrap().contains("SERPAPI"));
    }

    #[test]
    fn connection_status_active_vs_initializing() {
        let active = WireConnectedAccount {
            id: "ca_1".into(),
            status: "ACTIVE".into(),
        };
        let s = ConnectionStatus {
            toolkit: "github".into(),
            connected: active.status.eq_ignore_ascii_case("ACTIVE"),
            status: active.status,
            connected_account_id: Some(active.id),
        };
        assert!(s.connected);

        let not = ConnectionStatus::not_connected("gmail");
        assert!(!not.connected);
        assert_eq!(not.status, "not_connected");
        assert_eq!(not.connected_account_id, None);
    }

    #[test]
    fn extracts_error_messages_both_shapes() {
        let nested = r#"{"error":{"message":"No authentication provided","code":10401}}"#;
        assert_eq!(extract_error_message(nested), "No authentication provided");
        let flat = r#"{"error":"This endpoint is no longer available."}"#;
        assert_eq!(
            extract_error_message(flat),
            "This endpoint is no longer available."
        );
        assert_eq!(extract_error_message(""), "empty response body");
    }

    // ----- Live integration tests (skipped unless COMPOSIO_API_KEY is set) ---
    //
    // Run locally with:  COMPOSIO_API_KEY=ak_... cargo test -p beater-composio -- --nocapture

    fn live_client() -> Option<HttpComposioClient> {
        HttpComposioClient::from_env()
    }

    #[tokio::test]
    async fn live_list_toolkits() {
        let Some(client) = live_client() else {
            eprintln!("skipping: {API_KEY_ENV} not set");
            return;
        };
        let toolkits = client.list_toolkits(5).await.expect("list toolkits");
        assert!(!toolkits.is_empty(), "expected a non-empty catalog");
        assert!(toolkits.iter().all(|t| !t.slug.is_empty()));
    }

    #[tokio::test]
    async fn live_execute_search_tool() {
        let Some(client) = live_client() else {
            eprintln!("skipping: {API_KEY_ENV} not set");
            return;
        };
        // A no-auth tool: execution path must return the envelope (success of
        // the tool itself depends on Composio-side provider keys).
        let exec = client
            .execute(
                "COMPOSIO_SEARCH_DUCK_DUCK_GO_SEARCH",
                "beater:test:ci",
                serde_json::json!({ "query": "agent observability" }),
            )
            .await
            .expect("execute returns an envelope");
        // We assert the call round-tripped (got a log id), not that the upstream
        // search succeeded.
        assert!(exec.log_id.is_some() || exec.successful);
    }

    #[tokio::test]
    async fn live_connect_returns_login_url() {
        let Some(client) = live_client() else {
            eprintln!("skipping: {API_KEY_ENV} not set");
            return;
        };
        let link = client
            .connect("github", "beater:test:ci")
            .await
            .expect("connect");
        assert!(link.redirect_url.starts_with("https://"));
        assert!(link.connected_account_id.starts_with("ca_"));
    }
}
