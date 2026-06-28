//! Beater LLM gateway ("Patchbay") — a model-agnostic, BYOK-capable chat
//! completions surface over the Beater provider/cache/budget substrate.
//!
//! Architecture: ARCHITECTURE §20.10 #7.3 / REQUIREMENTS R18.3.
//!
//! The product requirement is: *"we should be ABLE to use any model in AI mode
//! but should NOT have to."* Concretely this gateway is:
//!
//! * **Model-agnostic** — any provider/model is chosen per request via
//!   [`ChatCompletionRequest::model`] plus a [`ModelRouting`] policy.
//! * **BYOK-capable** — [`ModelRouting::Byok`] resolves the tenant's own opaque
//!   [`beater_core::ProviderSecretId`] through [`ProviderSecretStore`]; raw key
//!   material is never logged, serialized, or returned (it travels only inside
//!   [`beater_judge::ProviderCredentials`], whose `Debug` is redacted).
//! * **Optional / managed-default** — if the caller brings no key, the gateway
//!   falls back to a configured [`ManagedDefault`] model so a hosted tenant does
//!   not *have* to configure anything. In OSS there is no managed default, so the
//!   absence of a key returns the typed [`GatewayError::NoKeyAndNoManagedDefault`]
//!   (mirrors the §2 editions table).
//! * **Robust** — request-hash caching, per-tenant budget reservation via
//!   [`QuotaLimiter`], retry/backoff on 429/5xx inside each provider, failover
//!   across an ordered list of keys/providers, a per-request timeout, and a
//!   canonical `llm.call` span emitted per proxied call (§5.2).
//!
//! The gateway deliberately *reuses* the [`beater_judge`] substrate
//! ([`ProviderCredentials`] redaction + [`RetryPolicy`]) and the
//! [`beater_secrets`] BYOK store and the [`beater_store::QuotaLimiter`] rather
//! than building a parallel stack; it adds only the chat-completions surface.

use async_trait::async_trait;
use beater_core::{
    sha256_json_hash, Money, ProjectId, ProviderSecretId, Sha256Hash, TenantId, Timestamp,
    TokenCounts,
};
use beater_judge::{ProviderCredentials, RetryPolicy};
use beater_schema::{AgentSpanKind, CanonicalAttrs, ModelRef, SpanStatus};
use beater_secrets::ProviderSecretStore;
use beater_store::{QuotaLimiter, QuotaReservationRequest};
use chrono::Utc;
use reqwest::StatusCode as ReqwestStatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

mod cache;
mod provider;

pub use cache::{CachedCompletion, GatewayCache, InMemoryGatewayCache, SqliteGatewayCache};
pub use provider::{
    AnthropicChatProvider, ChatProvider, ChatProviderError, ChatProviderResult,
    HttpChatProviderConfig, OpenAiChatProvider, ProviderCompletion, RoutingChatProvider,
};

/// Default per-request timeout applied to a single provider attempt.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

// ---------------------------------------------------------------------------
// OpenAI-compatible request / response surface
// ---------------------------------------------------------------------------

/// A single chat message in the OpenAI-compatible request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// An OpenAI-compatible chat completion request. The `model` field is a free-form
/// model identifier, making the gateway model-agnostic: any provider/model can be
/// named per request, paired with a [`ModelRouting`] credential policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    /// OpenAI-style reasoning effort hint (`low` | `medium` | `high`), forwarded
    /// to providers that understand it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl ChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            top_p: None,
            max_tokens: None,
            reasoning_effort: None,
        }
    }
}

/// Token usage in the OpenAI-compatible response shape.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

impl ChatCompletionUsage {
    /// Convert into the canonical [`TokenCounts`] used by Beater spans/cost.
    pub fn to_token_counts(&self) -> TokenCounts {
        TokenCounts {
            input: self.prompt_tokens,
            output: self.completion_tokens,
            reasoning: 0,
            cache_read: 0,
        }
    }
}

/// One choice in an OpenAI-compatible response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// An OpenAI-compatible chat completion response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: ChatCompletionUsage,
}

// ---------------------------------------------------------------------------
// Routing policy: any model, optionally BYOK, else managed default
// ---------------------------------------------------------------------------

/// Per-request credential routing policy.
///
/// This enum is the encoding of *"any model, optionally BYOK, else managed
/// default"*:
///
/// * [`ModelRouting::Byok`] — use the tenant's own opaque provider secret. The
///   model named in the [`ChatCompletionRequest`] is used as-is, so any model the
///   key can reach is reachable.
/// * [`ModelRouting::Managed`] — use Beater's managed credentials with the given
///   default model. Only available when the gateway is built with a
///   [`ManagedDefault`] (hosted edition).
///
/// A [`GatewayRequest`] carries an *ordered* `Vec<ModelRouting>` so the gateway
/// can fail over from one key/provider to the next. An empty list means *"I did
/// not bring a key"*: the gateway then uses the managed default if configured, or
/// returns [`GatewayError::NoKeyAndNoManagedDefault`] in OSS.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelRouting {
    /// Bring-your-own-key: resolve the tenant's opaque provider secret.
    Byok { provider_secret_id: ProviderSecretId },
    /// Use the managed default model (hosted only).
    Managed { default_model: ModelRef },
}

/// Beater-managed default credentials + model, present only in the hosted
/// edition. When absent (OSS), callers MUST bring their own key.
#[derive(Clone)]
pub struct ManagedDefault {
    credentials: ProviderCredentials,
    default_model: ModelRef,
}

impl ManagedDefault {
    pub fn new(credentials: ProviderCredentials, default_model: ModelRef) -> Self {
        Self {
            credentials,
            default_model,
        }
    }
}

impl std::fmt::Debug for ManagedDefault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ProviderCredentials already redacts its secret; default to its Debug.
        f.debug_struct("ManagedDefault")
            .field("credentials", &self.credentials)
            .field("default_model", &self.default_model)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Gateway request / outcome
// ---------------------------------------------------------------------------

/// A request to proxy a chat completion through the gateway.
#[derive(Clone, Debug)]
pub struct GatewayRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: beater_core::EnvironmentId,
    pub completion: ChatCompletionRequest,
    /// Ordered failover list of credential policies. Empty means "no key
    /// brought" — fall back to the managed default if configured.
    pub routing: Vec<ModelRouting>,
    /// Per-tenant budget ceiling (in micros) enforced via the [`QuotaLimiter`].
    pub budget_micros: i64,
    /// Quota window start; reservations accumulate within a window.
    pub window_start: Timestamp,
    /// When the quota window resets (surfaced in budget-exceeded errors).
    pub reset_at: Timestamp,
}

/// The result of a successfully proxied completion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GatewayOutcome {
    pub response: ChatCompletionResponse,
    pub cost: Money,
    pub cached: bool,
    pub provider: String,
    pub model: ModelRef,
    pub tokens: TokenCounts,
    pub request_hash: Sha256Hash,
    pub attempts: u32,
}

// ---------------------------------------------------------------------------
// Canonical llm.call span emission
// ---------------------------------------------------------------------------

/// A canonical `llm.call` span describing one proxied completion (§5.2).
///
/// This carries exactly the fields the Beater canonical span model records for an
/// LLM call, keyed by the single-sourced [`beater_schema::conventions::attr`]
/// keys via [`LlmCallSpan::canonical_attributes`], so proxied traffic is natively
/// observed with zero SDK.
#[derive(Clone, Debug, PartialEq)]
pub struct LlmCallSpan {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: beater_core::EnvironmentId,
    pub model: ModelRef,
    pub status: SpanStatus,
    pub tokens: TokenCounts,
    pub cost: Money,
    pub cached: bool,
    pub attempts: u32,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

impl LlmCallSpan {
    /// The canonical span kind for a proxied LLM call.
    pub fn kind(&self) -> AgentSpanKind {
        AgentSpanKind::LlmCall
    }

    /// Build the canonical attribute map for this span using the single-sourced
    /// convention keys, so it matches what an SDK-emitted `llm.call` span carries.
    pub fn canonical_attributes(&self) -> CanonicalAttrs {
        use beater_schema::conventions::attr;
        let mut attrs = CanonicalAttrs::new();
        attrs.insert(
            attr::SPAN_KIND.to_string(),
            serde_json::Value::from(AgentSpanKind::LlmCall.as_str()),
        );
        attrs.insert(
            attr::LLM_PROVIDER.to_string(),
            serde_json::Value::from(self.model.provider.clone()),
        );
        attrs.insert(
            attr::LLM_MODEL_NAME.to_string(),
            serde_json::Value::from(self.model.name.clone()),
        );
        attrs.insert(
            attr::LLM_TOKEN_PROMPT.to_string(),
            serde_json::Value::from(self.tokens.input),
        );
        attrs.insert(
            attr::LLM_TOKEN_COMPLETION.to_string(),
            serde_json::Value::from(self.tokens.output),
        );
        attrs.insert(
            attr::LLM_TOKEN_REASONING.to_string(),
            serde_json::Value::from(self.tokens.reasoning),
        );
        attrs.insert(
            attr::LLM_TOKEN_CACHE_READ.to_string(),
            serde_json::Value::from(self.tokens.cache_read),
        );
        attrs.insert(
            attr::LLM_COST_MICROS.to_string(),
            serde_json::Value::from(self.cost.amount_micros),
        );
        attrs.insert(
            attr::LLM_COST_CURRENCY.to_string(),
            serde_json::Value::from(self.cost.currency.as_str()),
        );
        attrs
    }
}

/// Error returned by a [`LlmCallSpanSink`].
#[derive(Debug, thiserror::Error)]
#[error("llm.call span sink error: {0}")]
pub struct SpanSinkError(pub String);

/// Sink for canonical `llm.call` spans emitted by the gateway.
#[async_trait]
pub trait LlmCallSpanSink: Send + Sync {
    async fn record(&self, span: LlmCallSpan) -> Result<(), SpanSinkError>;
}

#[async_trait]
impl<T> LlmCallSpanSink for Arc<T>
where
    T: LlmCallSpanSink + ?Sized,
{
    async fn record(&self, span: LlmCallSpan) -> Result<(), SpanSinkError> {
        (**self).record(span).await
    }
}

/// A span sink that drops spans. Useful where observation is not wired.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopSpanSink;

#[async_trait]
impl LlmCallSpanSink for NoopSpanSink {
    async fn record(&self, _span: LlmCallSpan) -> Result<(), SpanSinkError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Typed errors
// ---------------------------------------------------------------------------

/// Typed errors surfaced by [`Gateway::complete`].
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// The caller brought no key and the gateway has no managed default (OSS).
    #[error("no provider key was supplied and no managed default model is configured")]
    NoKeyAndNoManagedDefault,
    /// A BYOK secret reference did not resolve to an active secret.
    #[error("provider secret {0} was not found or is inactive")]
    ProviderSecretNotFound(ProviderSecretId),
    /// A managed route was requested but the gateway has no managed credentials.
    #[error("managed routing requested but this gateway has no managed default configured")]
    ManagedDefaultUnavailable,
    /// The estimated cost would exceed the per-tenant budget.
    #[error(
        "gateway budget exceeded: attempted {attempted_micros} micros, used {used_micros}/{limit_micros} micros"
    )]
    BudgetExceeded {
        attempted_micros: i64,
        used_micros: i64,
        limit_micros: i64,
    },
    /// Every configured key/provider failed (after per-provider retries).
    #[error("all providers failed: {0}")]
    AllProvidersFailed(String),
    /// The provider call(s) timed out.
    #[error("gateway request timed out after {timeout_ms} ms")]
    Timeout { timeout_ms: u64 },
    /// The cache layer failed.
    #[error("gateway cache error: {0}")]
    Cache(String),
    /// The secret store failed.
    #[error("gateway store error: {0}")]
    Store(String),
    /// Hashing the request failed.
    #[error("gateway hash error: {0}")]
    Hash(String),
    /// The span sink failed.
    #[error("gateway span error: {0}")]
    Span(String),
}

// ---------------------------------------------------------------------------
// Object-safe service trait (so the API can hold `Arc<dyn GatewayService>`)
// ---------------------------------------------------------------------------

/// Object-safe gateway surface for wiring into HTTP state.
#[async_trait]
pub trait GatewayService: Send + Sync {
    async fn complete(&self, request: GatewayRequest) -> Result<GatewayOutcome, GatewayError>;
}

#[async_trait]
impl<T> GatewayService for Arc<T>
where
    T: GatewayService + ?Sized,
{
    async fn complete(&self, request: GatewayRequest) -> Result<GatewayOutcome, GatewayError> {
        (**self).complete(request).await
    }
}

// ---------------------------------------------------------------------------
// The Gateway service
// ---------------------------------------------------------------------------

/// A resolved credential/model attempt produced from a [`ModelRouting`] entry.
struct ResolvedRoute {
    provider: String,
    model: ModelRef,
    credentials: ProviderCredentials,
}

/// The model-agnostic, BYOK-capable chat completions gateway.
///
/// Generic over the [`ProviderSecretStore`] (BYOK), [`GatewayCache`]
/// (request-hash cache), [`ChatProvider`] (the actual caller, with retry/backoff
/// and failover-friendly errors), [`QuotaLimiter`] (per-tenant budget), and the
/// [`LlmCallSpanSink`] (canonical span emission).
pub struct Gateway<S, C, P, Q, K> {
    secrets: S,
    cache: C,
    provider: P,
    quota: Q,
    span_sink: K,
    managed: Option<ManagedDefault>,
    request_timeout: Duration,
}

impl<S, C, P, Q, K> Gateway<S, C, P, Q, K>
where
    S: ProviderSecretStore,
    C: GatewayCache,
    P: ChatProvider,
    Q: QuotaLimiter,
    K: LlmCallSpanSink,
{
    /// Build a gateway. Pass `managed = Some(..)` for the hosted edition (so
    /// callers do not *have* to bring a key) or `None` for OSS (BYOK required).
    pub fn new(
        secrets: S,
        cache: C,
        provider: P,
        quota: Q,
        span_sink: K,
        managed: Option<ManagedDefault>,
    ) -> Self {
        Self {
            secrets,
            cache,
            provider,
            quota,
            span_sink,
            managed,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        }
    }

    /// Override the per-request (per-attempt) timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Proxy a chat completion: resolve routing, check cache, reserve budget,
    /// call providers with retry/backoff + failover, record cost, and emit a
    /// canonical `llm.call` span.
    pub async fn complete(
        &self,
        request: GatewayRequest,
    ) -> Result<GatewayOutcome, GatewayError> {
        let routes = self.resolve_routes(&request).await?;

        // The primary (first) route's model identifies the call for hashing,
        // observation, and budget estimation.
        let primary_model = routes
            .first()
            .map(|route| route.model.clone())
            .ok_or(GatewayError::NoKeyAndNoManagedDefault)?;

        let request_hash = self.request_hash(&request, &routes)?;
        let start_time = Utc::now();

        // 1. Request-hash cache: identical requests return the cached completion
        //    at zero incremental cost.
        if let Some(cached) = self
            .cache
            .get(
                request.tenant_id.clone(),
                request.project_id.clone(),
                request_hash.clone(),
            )
            .await
            .map_err(|err| GatewayError::Cache(err.to_string()))?
        {
            let tokens = cached.response.usage.to_token_counts();
            self.emit_span(&request, &primary_model, SpanStatus::Ok, &tokens, &cached.cost, true, 0, start_time)
                .await?;
            return Ok(GatewayOutcome {
                response: cached.response,
                cost: cached.cost,
                cached: true,
                provider: primary_model.provider.clone(),
                model: primary_model,
                tokens,
                request_hash,
                attempts: 0,
            });
        }

        // 2. Reserve the estimated max cost against the per-tenant budget.
        let max_cost = self.provider.max_cost(&primary_model, &request.completion);
        self.reserve_budget(&request, &max_cost).await?;

        // 3. Try each resolved route in order (failover), each provider applying
        //    its own retry/backoff on 429/5xx.
        let mut failures: Vec<String> = Vec::new();
        let mut all_timeouts = true;
        let mut attempts: u32 = 0;
        for route in &routes {
            attempts = attempts.saturating_add(1);
            let call = self.provider.complete(
                &route.model,
                &request.completion,
                &route.credentials,
            );
            match tokio::time::timeout(self.request_timeout, call).await {
                Err(_elapsed) => {
                    failures.push(format!(
                        "{}/{}: timed out after {} ms",
                        route.provider,
                        route.model.name,
                        self.request_timeout.as_millis()
                    ));
                }
                Ok(Err(err)) => {
                    all_timeouts = false;
                    failures.push(format!("{}/{}: {err}", route.provider, route.model.name));
                }
                Ok(Ok(completion)) => {
                    let tokens = completion.response.usage.to_token_counts();
                    // Persist to the request-hash cache for subsequent hits.
                    self.cache
                        .put(
                            request.tenant_id.clone(),
                            request.project_id.clone(),
                            request_hash.clone(),
                            route.model.clone(),
                            CachedCompletion {
                                response: completion.response.clone(),
                                cost: completion.cost.clone(),
                            },
                        )
                        .await
                        .map_err(|err| GatewayError::Cache(err.to_string()))?;
                    self.emit_span(
                        &request,
                        &route.model,
                        SpanStatus::Ok,
                        &tokens,
                        &completion.cost,
                        false,
                        attempts,
                        start_time,
                    )
                    .await?;
                    return Ok(GatewayOutcome {
                        response: completion.response,
                        cost: completion.cost,
                        cached: false,
                        provider: route.provider.clone(),
                        model: route.model.clone(),
                        tokens,
                        request_hash,
                        attempts,
                    });
                }
            }
        }

        // 4. Everything failed — emit an error span and return a typed error.
        self.emit_span(
            &request,
            &primary_model,
            SpanStatus::Error,
            &TokenCounts::default(),
            &Money::usd_micros(0),
            false,
            attempts,
            start_time,
        )
        .await?;
        if all_timeouts {
            return Err(GatewayError::Timeout {
                timeout_ms: u64::try_from(self.request_timeout.as_millis()).unwrap_or(u64::MAX),
            });
        }
        Err(GatewayError::AllProvidersFailed(failures.join("; ")))
    }

    /// Resolve the ordered routing list into concrete credential/model attempts.
    async fn resolve_routes(
        &self,
        request: &GatewayRequest,
    ) -> Result<Vec<ResolvedRoute>, GatewayError> {
        if request.routing.is_empty() {
            // No key brought: use the managed default if one exists, else OSS error.
            let managed = self
                .managed
                .as_ref()
                .ok_or(GatewayError::NoKeyAndNoManagedDefault)?;
            return Ok(vec![ResolvedRoute {
                provider: managed.default_model.provider.clone(),
                model: managed.default_model.clone(),
                credentials: managed.credentials.clone(),
            }]);
        }

        let mut routes = Vec::with_capacity(request.routing.len());
        for routing in &request.routing {
            match routing {
                ModelRouting::Byok { provider_secret_id } => {
                    let secret = self
                        .secrets
                        .get_secret(
                            request.tenant_id.clone(),
                            request.project_id.clone(),
                            provider_secret_id.clone(),
                        )
                        .await
                        .map_err(|err| GatewayError::Store(err.to_string()))?
                        .ok_or_else(|| {
                            GatewayError::ProviderSecretNotFound(provider_secret_id.clone())
                        })?;
                    let provider = secret.metadata.provider.clone();
                    routes.push(ResolvedRoute {
                        model: ModelRef {
                            provider: provider.clone(),
                            name: request.completion.model.clone(),
                        },
                        credentials: ProviderCredentials::new(provider.clone(), secret.secret_value()),
                        provider,
                    });
                }
                ModelRouting::Managed { default_model } => {
                    let managed = self
                        .managed
                        .as_ref()
                        .ok_or(GatewayError::ManagedDefaultUnavailable)?;
                    routes.push(ResolvedRoute {
                        provider: default_model.provider.clone(),
                        model: default_model.clone(),
                        credentials: managed.credentials.clone(),
                    });
                }
            }
        }
        Ok(routes)
    }

    /// Compute the request-hash used for caching. Hashes the tenant scope, the
    /// OpenAI-compatible request, and each route's (provider, model) — never the
    /// secret material — so identical requests cache-hit and different
    /// keys/providers do not collide.
    fn request_hash(
        &self,
        request: &GatewayRequest,
        routes: &[ResolvedRoute],
    ) -> Result<Sha256Hash, GatewayError> {
        #[derive(Serialize)]
        struct RouteKey<'a> {
            provider: &'a str,
            model: &'a ModelRef,
        }
        #[derive(Serialize)]
        struct HashableRequest<'a> {
            tenant_id: &'a TenantId,
            project_id: &'a ProjectId,
            environment_id: &'a beater_core::EnvironmentId,
            completion: &'a ChatCompletionRequest,
            routes: Vec<RouteKey<'a>>,
        }
        let hashable = HashableRequest {
            tenant_id: &request.tenant_id,
            project_id: &request.project_id,
            environment_id: &request.environment_id,
            completion: &request.completion,
            routes: routes
                .iter()
                .map(|route| RouteKey {
                    provider: &route.provider,
                    model: &route.model,
                })
                .collect(),
        };
        sha256_json_hash(&hashable).map_err(|err| GatewayError::Hash(err.to_string()))
    }

    /// Reserve the estimated max cost against the per-tenant budget ceiling via
    /// the [`QuotaLimiter`]. A non-accepted decision yields a typed
    /// [`GatewayError::BudgetExceeded`].
    async fn reserve_budget(
        &self,
        request: &GatewayRequest,
        max_cost: &Money,
    ) -> Result<(), GatewayError> {
        let amount = u64::try_from(max_cost.amount_micros).unwrap_or(0);
        let limit = u64::try_from(request.budget_micros).unwrap_or(0);
        let decision = self
            .quota
            .reserve_quota(QuotaReservationRequest {
                tenant_id: request.tenant_id.clone(),
                project_id: request.project_id.clone(),
                amount,
                limit,
                window_start: request.window_start,
                reset_at: request.reset_at,
            })
            .await
            .map_err(|err| GatewayError::Store(err.to_string()))?;
        if !decision.accepted {
            return Err(GatewayError::BudgetExceeded {
                attempted_micros: max_cost.amount_micros,
                used_micros: i64::try_from(decision.used).unwrap_or(i64::MAX),
                limit_micros: i64::try_from(decision.limit).unwrap_or(i64::MAX),
            });
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn emit_span(
        &self,
        request: &GatewayRequest,
        model: &ModelRef,
        status: SpanStatus,
        tokens: &TokenCounts,
        cost: &Money,
        cached: bool,
        attempts: u32,
        start_time: Timestamp,
    ) -> Result<(), GatewayError> {
        self.span_sink
            .record(LlmCallSpan {
                tenant_id: request.tenant_id.clone(),
                project_id: request.project_id.clone(),
                environment_id: request.environment_id.clone(),
                model: model.clone(),
                status,
                tokens: tokens.clone(),
                cost: cost.clone(),
                cached,
                attempts,
                start_time,
                end_time: Utc::now(),
            })
            .await
            .map_err(|err| GatewayError::Span(err.to_string()))
    }
}

#[async_trait]
impl<S, C, P, Q, K> GatewayService for Gateway<S, C, P, Q, K>
where
    S: ProviderSecretStore,
    C: GatewayCache,
    P: ChatProvider,
    Q: QuotaLimiter,
    K: LlmCallSpanSink,
{
    async fn complete(&self, request: GatewayRequest) -> Result<GatewayOutcome, GatewayError> {
        Gateway::complete(self, request).await
    }
}

// ---------------------------------------------------------------------------
// Shared retry/backoff helper (mirrors the beater-judge provider clients)
// ---------------------------------------------------------------------------

/// Send a request with retry/backoff on 429/5xx, honoring `Retry-After`.
///
/// This mirrors the retry loop used by the `beater-judge` provider clients,
/// reusing the shared [`RetryPolicy`] type, so the gateway's HTTP providers are
/// robust the same way the judge providers are.
pub(crate) async fn send_json_with_retries<F>(
    mut build_request: F,
    retry_policy: RetryPolicy,
) -> Result<reqwest::Response, ChatProviderError>
where
    F: FnMut() -> reqwest::RequestBuilder,
{
    let max_attempts = retry_policy.max_attempts.max(1);
    for attempt in 1..=max_attempts {
        let response = build_request()
            .send()
            .await
            .map_err(|err| ChatProviderError::transport(err.to_string()))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        if !is_retryable_status(status) || attempt == max_attempts {
            return Err(provider_http_error(response).await);
        }
        let delay = retry_delay(&response, retry_policy, attempt);
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }
    Err(ChatProviderError::transport(
        "provider request exhausted retries".to_string(),
    ))
}

fn is_retryable_status(status: ReqwestStatusCode) -> bool {
    status == ReqwestStatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retry_delay(response: &reqwest::Response, retry_policy: RetryPolicy, attempt: u32) -> Duration {
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs);
    retry_after.unwrap_or_else(|| {
        Duration::from_millis(
            retry_policy
                .base_backoff_ms
                .saturating_mul(u64::from(attempt)),
        )
    })
}

async fn provider_http_error(response: reqwest::Response) -> ChatProviderError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    ChatProviderError::Status {
        status: status.as_u16(),
        body: truncate_error_body(&body),
    }
}

fn truncate_error_body(body: &str) -> String {
    const LIMIT: usize = 512;
    if body.chars().count() <= LIMIT {
        return body.to_string();
    }
    let truncated = body.chars().take(LIMIT).collect::<String>();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests;
