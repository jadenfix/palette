//! End-to-end test for the LLM gateway endpoint (§20.10 #7.3 / R18.3).
//!
//! Drives `POST /v1/gateway/{tenant}/{project}/{environment}/chat/completions`
//! through the real axum router with a *mock* chat provider (no real network)
//! and asserts: the OpenAI-compatible response shape, that a canonical `llm.call`
//! span is emitted, a cache hit on a second identical call, and a budget-exceeded
//! 4xx.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::Router;
use beater_api::{router, ApiState};
use beater_bus::InMemoryBus;
use beater_core::Money;
use beater_gateway::{
    ChatCompletionChoice, ChatCompletionRequest, ChatCompletionResponse, ChatCompletionUsage,
    ChatMessage, ChatProvider, ChatProviderError, ChatProviderResult, Gateway, InMemoryGatewayCache,
    LlmCallSpan, LlmCallSpanSink, ManagedDefault, ProviderCompletion, SpanSinkError,
};
use beater_ingest::{IngestPolicy, IngestService};
use beater_judge::ProviderCredentials;
use beater_schema::{AgentSpanKind, ModelRef};
use beater_secrets::SqliteProviderSecretStore;
use beater_store_memory::{InMemoryQuotaLimiter, InMemoryTraceStore};
use beater_store_obj::FsArtifactStore;
use http::{Request, StatusCode};
use tower::ServiceExt;

fn unwrap<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("test setup failed: {err}"),
    }
}

/// A mock provider: echoes a deterministic OpenAI-compatible completion, with a
/// configurable flat cost, and counts calls so cache hits are observable.
#[derive(Clone)]
struct MockChatProvider {
    calls: Arc<AtomicUsize>,
    cost: Money,
}

#[async_trait]
impl ChatProvider for MockChatProvider {
    fn max_cost(&self, _model: &ModelRef, _request: &ChatCompletionRequest) -> Money {
        self.cost.clone()
    }

    async fn complete(
        &self,
        model: &ModelRef,
        _request: &ChatCompletionRequest,
        _credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if model.name.is_empty() {
            return Err(ChatProviderError::Unsupported("empty model".to_string()));
        }
        Ok(ProviderCompletion {
            response: ChatCompletionResponse {
                id: "cmpl-e2e".to_string(),
                object: "chat.completion".to_string(),
                created: 0,
                model: model.name.clone(),
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage::new("assistant", "hello from the gateway"),
                    finish_reason: Some("stop".to_string()),
                }],
                usage: ChatCompletionUsage {
                    prompt_tokens: 4,
                    completion_tokens: 6,
                    total_tokens: 10,
                },
            },
            cost: self.cost.clone(),
        })
    }
}

#[derive(Clone, Default)]
struct RecordingSpanSink {
    spans: Arc<Mutex<Vec<LlmCallSpan>>>,
}

#[async_trait]
impl LlmCallSpanSink for RecordingSpanSink {
    async fn record(&self, span: LlmCallSpan) -> Result<(), SpanSinkError> {
        let mut spans = self
            .spans
            .lock()
            .map_err(|err| SpanSinkError(err.to_string()))?;
        spans.push(span);
        Ok(())
    }
}

impl RecordingSpanSink {
    fn snapshot(&self) -> Vec<LlmCallSpan> {
        self.spans
            .lock()
            .map(|spans| spans.clone())
            .unwrap_or_default()
    }
}

fn build_app(cost_micros: i64) -> (Router, Arc<AtomicUsize>, RecordingSpanSink, tempfile::TempDir) {
    let tempdir = unwrap(tempfile::tempdir());
    let artifacts = Arc::new(unwrap(FsArtifactStore::new(tempdir.path().join("artifacts"))));
    let traces = Arc::new(InMemoryTraceStore::new());
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());

    let calls = Arc::new(AtomicUsize::new(0));
    let sink = RecordingSpanSink::default();
    let gateway = Arc::new(Gateway::new(
        unwrap(SqliteProviderSecretStore::in_memory()),
        InMemoryGatewayCache::new(),
        MockChatProvider {
            calls: calls.clone(),
            cost: Money::usd_micros(cost_micros),
        },
        InMemoryQuotaLimiter::new(),
        sink.clone(),
        // Hosted-style managed default so callers do not have to bring a key.
        Some(ManagedDefault::new(
            ProviderCredentials::new("openai", "managed-secret"),
            ModelRef {
                provider: "openai".to_string(),
                name: "managed-default-model".to_string(),
            },
        )),
    ));

    let state = ApiState::new(ingest, traces).with_gateway(gateway);
    (router(state), calls, sink, tempdir)
}

fn chat_body(budget_micros: Option<i64>) -> String {
    let mut body = serde_json::json!({
        "completion": {
            "model": "gpt-test",
            "messages": [{ "role": "user", "content": "hello there" }]
        },
        "routing": []
    });
    if let Some(budget) = budget_micros {
        body["budget_micros"] = serde_json::Value::from(budget);
    }
    body.to_string()
}

async fn post_chat(app: &Router, body: String) -> (StatusCode, serde_json::Value) {
    let request = unwrap(
        Request::builder()
            .method("POST")
            .uri("/v1/gateway/acme/proj/prod/chat/completions")
            .header("content-type", "application/json")
            .body(Body::from(body)),
    );
    let response = unwrap(app.clone().oneshot(request).await);
    let status = response.status();
    let bytes = unwrap(to_bytes(response.into_body(), 1024 * 1024).await);
    let json = if bytes.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    };
    (status, json)
}

#[tokio::test]
async fn gateway_returns_openai_shape_caches_and_emits_span() {
    let (app, calls, sink, _tempdir) = build_app(10);

    // First call: OpenAI-compatible response, not cached.
    let (status, json) = post_chat(&app, chat_body(Some(1_000_000))).await;
    assert_eq!(status, StatusCode::OK, "unexpected body: {json}");
    assert_eq!(json["cached"], serde_json::Value::Bool(false));
    assert_eq!(json["response"]["object"], "chat.completion");
    assert_eq!(json["response"]["model"], "managed-default-model");
    assert_eq!(
        json["response"]["choices"][0]["message"]["content"],
        "hello from the gateway"
    );
    assert_eq!(json["response"]["usage"]["total_tokens"], 10);

    // Second identical call: served from cache, provider not called again.
    let (status, json) = post_chat(&app, chat_body(Some(1_000_000))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["cached"], serde_json::Value::Bool(true));
    assert_eq!(calls.load(Ordering::SeqCst), 1, "cache should prevent a 2nd provider call");

    // A canonical llm.call span is emitted per proxied call (live + cached).
    let spans = sink.snapshot();
    assert_eq!(spans.len(), 2);
    for span in &spans {
        assert_eq!(span.kind(), AgentSpanKind::LlmCall);
        let attrs = span.canonical_attributes();
        assert_eq!(
            attrs
                .get(beater_schema::conventions::attr::SPAN_KIND)
                .and_then(serde_json::Value::as_str),
            Some("llm.call")
        );
    }
}

#[tokio::test]
async fn gateway_budget_exceeded_returns_4xx() {
    // Provider's flat cost (50) exceeds the per-request budget ceiling (5).
    let (app, _calls, _sink, _tempdir) = build_app(50);
    let (status, _json) = post_chat(&app, chat_body(Some(5))).await;
    assert!(status.is_client_error(), "expected 4xx, got {status}");
    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
}
