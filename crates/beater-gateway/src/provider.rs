//! Chat-completion provider clients for the gateway.
//!
//! These mirror the `beater-judge` provider clients (OpenAI-compatible +
//! Anthropic) but speak the OpenAI chat-completions surface used by the gateway.
//! Each client applies retry/backoff on 429/5xx via [`crate::send_json_with_retries`]
//! and returns *typed* errors so the gateway can fail over to the next route.

use async_trait::async_trait;
use beater_core::Money;
use beater_judge::{ProviderCredentials, RetryPolicy};
use beater_schema::ModelRef;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    send_json_with_retries, ChatCompletionChoice, ChatCompletionRequest, ChatCompletionResponse,
    ChatCompletionUsage, ChatMessage,
};

pub type ChatProviderResult<T> = Result<T, ChatProviderError>;

/// A typed provider error. Distinguishing transport/status/decode lets the
/// gateway treat all of them as failover-eligible while still reporting cause.
#[derive(Debug, thiserror::Error)]
pub enum ChatProviderError {
    #[error("provider transport error: {0}")]
    Transport(String),
    #[error("provider returned HTTP {status}: {body}")]
    Status { status: u16, body: String },
    #[error("provider response decode error: {0}")]
    Decode(String),
    #[error("unsupported provider: {0}")]
    Unsupported(String),
}

impl ChatProviderError {
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport(message.into())
    }

    pub fn decode(message: impl Into<String>) -> Self {
        Self::Decode(message.into())
    }
}

/// A provider completion: the OpenAI-compatible response plus the metered cost.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderCompletion {
    pub response: ChatCompletionResponse,
    pub cost: Money,
}

/// A chat-completion provider. Implementors call a concrete model with the given
/// (already-resolved) credentials and return the OpenAI-compatible response.
#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// The most this provider could charge for `request` — used to reserve budget
    /// before the call. `model` lets a routing provider pick the right ceiling.
    fn max_cost(&self, model: &ModelRef, request: &ChatCompletionRequest) -> Money;

    async fn complete(
        &self,
        model: &ModelRef,
        request: &ChatCompletionRequest,
        credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion>;
}

#[async_trait]
impl<T> ChatProvider for Arc<T>
where
    T: ChatProvider + ?Sized,
{
    fn max_cost(&self, model: &ModelRef, request: &ChatCompletionRequest) -> Money {
        (**self).max_cost(model, request)
    }

    async fn complete(
        &self,
        model: &ModelRef,
        request: &ChatCompletionRequest,
        credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion> {
        (**self).complete(model, request, credentials).await
    }
}

/// Shared HTTP provider configuration.
#[derive(Clone, Debug)]
pub struct HttpChatProviderConfig {
    pub endpoint_url: String,
    /// Flat cost charged/metered per successful call (micros). Kept simple and
    /// deterministic, matching the `beater-judge` provider clients.
    pub max_cost: Money,
    pub retry_policy: RetryPolicy,
}

impl HttpChatProviderConfig {
    pub fn openai_default() -> Self {
        Self {
            endpoint_url: "https://api.openai.com/v1/chat/completions".to_string(),
            max_cost: Money::usd_micros(100_000),
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn anthropic_default() -> Self {
        Self {
            endpoint_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_cost: Money::usd_micros(100_000),
            retry_policy: RetryPolicy::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible provider
// ---------------------------------------------------------------------------

/// OpenAI-compatible chat-completions client. Works against any endpoint that
/// implements the OpenAI `/chat/completions` contract.
#[derive(Clone, Debug)]
pub struct OpenAiChatProvider {
    client: reqwest::Client,
    config: HttpChatProviderConfig,
}

impl OpenAiChatProvider {
    pub fn new(config: HttpChatProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }
}

#[async_trait]
impl ChatProvider for OpenAiChatProvider {
    fn max_cost(&self, _model: &ModelRef, _request: &ChatCompletionRequest) -> Money {
        self.config.max_cost.clone()
    }

    async fn complete(
        &self,
        model: &ModelRef,
        request: &ChatCompletionRequest,
        credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion> {
        let mut body = serde_json::json!({
            "model": model.name,
            "messages": request.messages,
        });
        insert_optional(&mut body, "temperature", request.temperature.map(into_json));
        insert_optional(&mut body, "top_p", request.top_p.map(into_json));
        insert_optional(
            &mut body,
            "max_tokens",
            request.max_tokens.map(serde_json::Value::from),
        );
        insert_optional(
            &mut body,
            "reasoning_effort",
            request.reasoning_effort.clone().map(serde_json::Value::from),
        );

        let response = send_json_with_retries(
            || {
                self.client
                    .post(&self.config.endpoint_url)
                    .bearer_auth(credentials.secret_value())
                    .json(&body)
            },
            self.config.retry_policy,
        )
        .await?;
        let payload: OpenAiResponse = response
            .json()
            .await
            .map_err(|err| ChatProviderError::decode(err.to_string()))?;
        Ok(ProviderCompletion {
            response: payload.into_response(&model.name),
            cost: self.config.max_cost.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Anthropic provider
// ---------------------------------------------------------------------------

/// Anthropic Messages-API client, mapped onto the OpenAI-compatible surface.
#[derive(Clone, Debug)]
pub struct AnthropicChatProvider {
    client: reqwest::Client,
    config: HttpChatProviderConfig,
    anthropic_version: String,
}

impl AnthropicChatProvider {
    pub fn new(config: HttpChatProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            anthropic_version: "2023-06-01".to_string(),
        }
    }
}

#[async_trait]
impl ChatProvider for AnthropicChatProvider {
    fn max_cost(&self, _model: &ModelRef, _request: &ChatCompletionRequest) -> Money {
        self.config.max_cost.clone()
    }

    async fn complete(
        &self,
        model: &ModelRef,
        request: &ChatCompletionRequest,
        credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion> {
        // Anthropic separates the system prompt from the message list.
        let system = request
            .messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        let messages = request
            .messages
            .iter()
            .filter(|message| message.role != "system")
            .map(|message| {
                serde_json::json!({ "role": message.role, "content": message.content })
            })
            .collect::<Vec<_>>();
        let mut body = serde_json::json!({
            "model": model.name,
            "max_tokens": request.max_tokens.unwrap_or(1024),
            "messages": messages,
        });
        if !system.is_empty() {
            body["system"] = serde_json::Value::from(system);
        }
        insert_optional(&mut body, "temperature", request.temperature.map(into_json));
        insert_optional(&mut body, "top_p", request.top_p.map(into_json));

        let response = send_json_with_retries(
            || {
                self.client
                    .post(&self.config.endpoint_url)
                    .header("x-api-key", credentials.secret_value())
                    .header("anthropic-version", self.anthropic_version.as_str())
                    .json(&body)
            },
            self.config.retry_policy,
        )
        .await?;
        let payload: AnthropicResponse = response
            .json()
            .await
            .map_err(|err| ChatProviderError::decode(err.to_string()))?;
        Ok(ProviderCompletion {
            response: payload.into_response(&model.name),
            cost: self.config.max_cost.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Routing provider: dispatch by ModelRef.provider
// ---------------------------------------------------------------------------

/// Dispatches to the OpenAI-compatible or Anthropic client based on the resolved
/// [`ModelRef::provider`], so a single gateway is model-agnostic across both.
#[derive(Clone, Debug)]
pub struct RoutingChatProvider {
    openai: OpenAiChatProvider,
    anthropic: AnthropicChatProvider,
}

impl RoutingChatProvider {
    pub fn new(openai: OpenAiChatProvider, anthropic: AnthropicChatProvider) -> Self {
        Self { openai, anthropic }
    }

    fn is_openai(provider: &str) -> bool {
        provider.eq_ignore_ascii_case("openai")
            || provider.eq_ignore_ascii_case("openai-compatible")
    }

    fn is_anthropic(provider: &str) -> bool {
        provider.eq_ignore_ascii_case("anthropic")
    }
}

impl Default for RoutingChatProvider {
    fn default() -> Self {
        Self {
            openai: OpenAiChatProvider::new(HttpChatProviderConfig::openai_default()),
            anthropic: AnthropicChatProvider::new(HttpChatProviderConfig::anthropic_default()),
        }
    }
}

#[async_trait]
impl ChatProvider for RoutingChatProvider {
    fn max_cost(&self, model: &ModelRef, request: &ChatCompletionRequest) -> Money {
        if Self::is_anthropic(&model.provider) {
            self.anthropic.max_cost(model, request)
        } else if Self::is_openai(&model.provider) {
            self.openai.max_cost(model, request)
        } else {
            // Unknown provider: reserve the larger ceiling defensively.
            self.openai.max_cost(model, request)
        }
    }

    async fn complete(
        &self,
        model: &ModelRef,
        request: &ChatCompletionRequest,
        credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion> {
        if Self::is_openai(&model.provider) {
            self.openai.complete(model, request, credentials).await
        } else if Self::is_anthropic(&model.provider) {
            self.anthropic.complete(model, request, credentials).await
        } else {
            Err(ChatProviderError::Unsupported(model.provider.clone()))
        }
    }
}

// ---------------------------------------------------------------------------
// Wire formats
// ---------------------------------------------------------------------------

fn into_json(value: f64) -> serde_json::Value {
    serde_json::Number::from_f64(value)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

/// Insert a JSON field only when `value` is `Some` and not `Null`.
fn insert_optional(body: &mut serde_json::Value, key: &str, value: Option<serde_json::Value>) {
    if let (Some(value), Some(object)) = (value, body.as_object_mut()) {
        if !value.is_null() {
            object.insert(key.to_string(), value);
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    created: Option<i64>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    #[serde(default)]
    index: u32,
    message: OpenAiMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    #[serde(default = "assistant_role")]
    role: String,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

fn assistant_role() -> String {
    "assistant".to_string()
}

impl OpenAiResponse {
    fn into_response(self, requested_model: &str) -> ChatCompletionResponse {
        let usage = self.usage.unwrap_or_default();
        let total_tokens = if usage.total_tokens == 0 {
            usage.prompt_tokens.saturating_add(usage.completion_tokens)
        } else {
            usage.total_tokens
        };
        ChatCompletionResponse {
            id: self.id.unwrap_or_default(),
            object: "chat.completion".to_string(),
            created: self.created.unwrap_or(0),
            model: self.model.unwrap_or_else(|| requested_model.to_string()),
            choices: self
                .choices
                .into_iter()
                .map(|choice| ChatCompletionChoice {
                    index: choice.index,
                    message: ChatMessage::new(
                        choice.message.role,
                        choice.message.content.unwrap_or_default(),
                    ),
                    finish_reason: choice.finish_reason,
                })
                .collect(),
            usage: ChatCompletionUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Default, Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

impl AnthropicResponse {
    fn into_response(self, requested_model: &str) -> ChatCompletionResponse {
        let usage = self.usage.unwrap_or_default();
        let text = self
            .content
            .iter()
            .filter(|block| block.kind == "text")
            .map(|block| block.text.clone())
            .collect::<Vec<_>>()
            .join("");
        ChatCompletionResponse {
            id: self.id.unwrap_or_default(),
            object: "chat.completion".to_string(),
            created: 0,
            model: self.model.unwrap_or_else(|| requested_model.to_string()),
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage::new("assistant", text),
                finish_reason: self.stop_reason,
            }],
            usage: ChatCompletionUsage {
                prompt_tokens: usage.input_tokens,
                completion_tokens: usage.output_tokens,
                total_tokens: usage.input_tokens.saturating_add(usage.output_tokens),
            },
        }
    }
}
