use super::*;
use beater_core::{EnvironmentId, Money};
use beater_schema::ModelRef;
use beater_secrets::{PutProviderSecretRequest, SqliteProviderSecretStore};
use beater_store_memory::InMemoryQuotaLimiter;
use chrono::Utc;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

/// A mock provider that fails for a configured set of secret values, optionally
/// hangs (for timeout tests), and otherwise echoes a deterministic completion.
#[derive(Clone)]
struct MockChatProvider {
    calls: Arc<AtomicUsize>,
    fail_secret_values: HashSet<String>,
    cost: Money,
    hang: bool,
}

impl MockChatProvider {
    fn new(cost: Money) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            fail_secret_values: HashSet::new(),
            cost,
            hang: false,
        }
    }

    fn failing_on(mut self, secret_value: impl Into<String>) -> Self {
        self.fail_secret_values.insert(secret_value.into());
        self
    }

    fn hanging(mut self) -> Self {
        self.hang = true;
        self
    }
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
        credentials: &ProviderCredentials,
    ) -> ChatProviderResult<ProviderCompletion> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.hang {
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
        if self.fail_secret_values.contains(credentials.secret_value()) {
            return Err(ChatProviderError::Status {
                status: 503,
                body: "mock provider failure".to_string(),
            });
        }
        Ok(ProviderCompletion {
            response: ChatCompletionResponse {
                id: "cmpl-mock".to_string(),
                object: "chat.completion".to_string(),
                created: 0,
                model: model.name.clone(),
                choices: vec![ChatCompletionChoice {
                    index: 0,
                    message: ChatMessage::new("assistant", "mock answer"),
                    finish_reason: Some("stop".to_string()),
                }],
                usage: ChatCompletionUsage {
                    prompt_tokens: 3,
                    completion_tokens: 5,
                    total_tokens: 8,
                },
            },
            cost: self.cost.clone(),
        })
    }
}

/// A span sink that records every emitted span for assertions.
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

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn tenant() -> TenantId {
    TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"))
}

fn project() -> ProjectId {
    ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"))
}

fn environment() -> EnvironmentId {
    EnvironmentId::new("env").unwrap_or_else(|err| panic!("{err}"))
}

fn completion() -> ChatCompletionRequest {
    ChatCompletionRequest::new(
        "gpt-test",
        vec![ChatMessage::new("user", "hello there")],
    )
}

fn gateway_request(routing: Vec<ModelRouting>, budget_micros: i64) -> GatewayRequest {
    GatewayRequest {
        tenant_id: tenant(),
        project_id: project(),
        environment_id: environment(),
        completion: completion(),
        routing,
        budget_micros,
        window_start: Utc::now(),
        reset_at: Utc::now() + chrono::Duration::hours(1),
    }
}

async fn put_secret(
    secrets: &SqliteProviderSecretStore,
    provider: &str,
    secret_value: &str,
) -> ProviderSecretId {
    secrets
        .put_secret(PutProviderSecretRequest {
            tenant_id: tenant(),
            project_id: project(),
            provider: provider.to_string(),
            display_name: "byok".to_string(),
            secret_value: secret_value.to_string(),
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .provider_secret_id
}

fn managed_default() -> ManagedDefault {
    ManagedDefault::new(
        ProviderCredentials::new("openai", "managed-secret-xyz"),
        ModelRef {
            provider: "openai".to_string(),
            name: "managed-default-model".to_string(),
        },
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cache_hit_returns_cached_without_calling_provider_twice() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let secret_id = put_secret(&secrets, "openai", "sk-live-secret").await;
    let provider = MockChatProvider::new(Money::usd_micros(40));
    let calls = provider.calls.clone();
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    );

    let routing = vec![ModelRouting::Byok {
        provider_secret_id: secret_id,
    }];
    let first = gateway
        .complete(gateway_request(routing.clone(), 1_000))
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let second = gateway
        .complete(gateway_request(routing, 1_000))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert!(!first.cached);
    assert!(second.cached);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(first.request_hash, second.request_hash);
    assert_eq!(first.response, second.response);
    assert_eq!(second.cost, Money::usd_micros(40));
}

#[tokio::test]
async fn first_key_failure_fails_over_to_second_key() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let bad = put_secret(&secrets, "openai", "sk-bad").await;
    let good = put_secret(&secrets, "openai", "sk-good").await;
    let provider = MockChatProvider::new(Money::usd_micros(10)).failing_on("sk-bad");
    let calls = provider.calls.clone();
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    );

    let outcome = gateway
        .complete(gateway_request(
            vec![
                ModelRouting::Byok {
                    provider_secret_id: bad,
                },
                ModelRouting::Byok {
                    provider_secret_id: good,
                },
            ],
            1_000,
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert!(!outcome.cached);
    assert_eq!(outcome.attempts, 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn budget_exceeded_returns_typed_error() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let secret_id = put_secret(&secrets, "openai", "sk-live").await;
    // max_cost (75) exceeds the budget ceiling (40).
    let provider = MockChatProvider::new(Money::usd_micros(75));
    let calls = provider.calls.clone();
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    );

    let result = gateway
        .complete(gateway_request(
            vec![ModelRouting::Byok {
                provider_secret_id: secret_id,
            }],
            40,
        ))
        .await;

    assert!(matches!(
        result,
        Err(GatewayError::BudgetExceeded {
            attempted_micros: 75,
            limit_micros: 40,
            ..
        })
    ));
    // The provider is never called once the budget reservation is refused.
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn no_byok_with_managed_default_uses_default_model() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let provider = MockChatProvider::new(Money::usd_micros(10));
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        Some(managed_default()),
    );

    // Empty routing == "I did not bring a key".
    let outcome = gateway
        .complete(gateway_request(Vec::new(), 1_000))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(outcome.model.name, "managed-default-model");
    assert_eq!(outcome.provider, "openai");
    assert_eq!(outcome.attempts, 1);
}

#[tokio::test]
async fn no_byok_without_managed_default_is_typed_no_key_error_oss() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let provider = MockChatProvider::new(Money::usd_micros(10));
    // managed = None mirrors the OSS edition where BYOK is required.
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    );

    let result = gateway.complete(gateway_request(Vec::new(), 1_000)).await;
    assert!(matches!(
        result,
        Err(GatewayError::NoKeyAndNoManagedDefault)
    ));
}

#[tokio::test]
async fn provider_secret_not_found_is_typed_error() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let missing = ProviderSecretId::new("does-not-exist").unwrap_or_else(|err| panic!("{err}"));
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        MockChatProvider::new(Money::usd_micros(10)),
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    );

    let result = gateway
        .complete(gateway_request(
            vec![ModelRouting::Byok {
                provider_secret_id: missing,
            }],
            1_000,
        ))
        .await;
    assert!(matches!(
        result,
        Err(GatewayError::ProviderSecretNotFound(_))
    ));
}

#[tokio::test]
async fn all_providers_failing_returns_typed_error() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let bad = put_secret(&secrets, "openai", "sk-bad").await;
    let provider = MockChatProvider::new(Money::usd_micros(10)).failing_on("sk-bad");
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    );

    let result = gateway
        .complete(gateway_request(
            vec![ModelRouting::Byok {
                provider_secret_id: bad,
            }],
            1_000,
        ))
        .await;
    assert!(matches!(result, Err(GatewayError::AllProvidersFailed(_))));
}

#[tokio::test]
async fn hanging_provider_times_out_with_typed_error() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let secret_id = put_secret(&secrets, "openai", "sk-live").await;
    let provider = MockChatProvider::new(Money::usd_micros(10)).hanging();
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        provider,
        InMemoryQuotaLimiter::new(),
        RecordingSpanSink::default(),
        None,
    )
    .with_request_timeout(Duration::from_millis(50));

    let result = gateway
        .complete(gateway_request(
            vec![ModelRouting::Byok {
                provider_secret_id: secret_id,
            }],
            1_000,
        ))
        .await;
    assert!(matches!(result, Err(GatewayError::Timeout { .. })));
}

#[tokio::test]
async fn emits_canonical_llm_call_span_per_call() {
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let secret_id = put_secret(&secrets, "openai", "sk-live").await;
    let sink = RecordingSpanSink::default();
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        MockChatProvider::new(Money::usd_micros(10)),
        InMemoryQuotaLimiter::new(),
        sink.clone(),
        None,
    );

    gateway
        .complete(gateway_request(
            vec![ModelRouting::Byok {
                provider_secret_id: secret_id,
            }],
            1_000,
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let spans = sink.snapshot();
    assert_eq!(spans.len(), 1);
    let span = spans.first().unwrap_or_else(|| panic!("expected one span"));
    assert_eq!(span.kind(), AgentSpanKind::LlmCall);
    assert_eq!(span.status, SpanStatus::Ok);
    let attrs = span.canonical_attributes();
    assert_eq!(
        attrs
            .get(beater_schema::conventions::attr::SPAN_KIND)
            .and_then(serde_json::Value::as_str),
        Some("llm.call")
    );
    assert_eq!(
        attrs
            .get(beater_schema::conventions::attr::LLM_PROVIDER)
            .and_then(serde_json::Value::as_str),
        Some("openai")
    );
}

#[tokio::test]
async fn provider_key_material_is_never_exposed() {
    let secret_value = "sk-super-secret-byok-value";
    let secrets = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
    let secret_id = put_secret(&secrets, "openai", secret_value).await;
    let sink = RecordingSpanSink::default();
    let gateway = Gateway::new(
        secrets,
        InMemoryGatewayCache::new(),
        MockChatProvider::new(Money::usd_micros(10)),
        InMemoryQuotaLimiter::new(),
        sink.clone(),
        Some(managed_default()),
    );

    let outcome = gateway
        .complete(gateway_request(
            vec![ModelRouting::Byok {
                provider_secret_id: secret_id,
            }],
            1_000,
        ))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    // Outcome Debug + serialized JSON must not leak key material.
    assert!(!format!("{outcome:?}").contains(secret_value));
    let serialized = serde_json::to_string(&outcome).unwrap_or_else(|err| panic!("{err}"));
    assert!(!serialized.contains(secret_value));

    // Emitted spans (and their Debug) must not leak key material.
    let spans = sink.snapshot();
    assert!(!format!("{spans:?}").contains(secret_value));

    // The reused redacted credential type must not leak, and neither must the
    // managed default config Debug.
    assert!(!format!(
        "{:?}",
        ProviderCredentials::new("openai", secret_value)
    )
    .contains(secret_value));
    assert!(!format!("{:?}", managed_default()).contains("managed-secret-xyz"));
}
