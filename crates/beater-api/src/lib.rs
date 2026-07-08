use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use beater_alerts::{
    AlertDecision, AlertEngine, AlertInput, AlertPolicy, OnlineSamplingPolicy, SamplingDecision,
    decide_trace_sampling, validate_webhook_endpoint_url,
};
use beater_archive::{ArchiveManifest, ArchiveQuery, ArchivedSpanRow, ParquetTraceArchive};
use beater_audit::{
    AuditAction, AuditEvent, AuditEventInsert, AuditOutcome, AuditStore,
    ConnectorToolInvokeAuditInput, PiiUnmaskAuditInput, connector_tool_invoke_event,
    pii_unmask_event,
};
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, RevokedApiKey};
#[cfg(feature = "billing")]
use beater_billing::store::BillingStore;
#[cfg(feature = "billing")]
use beater_billing::stripe::{EventApplication, StripeError, StripeSync};
#[cfg(feature = "billing")]
use beater_billing::{
    Billing, BillingError, Invoice, Plan, PlanChange, PlanId, Subscription, SubscriptionStatus,
};
use beater_calibration::{
    CalibrationPolicy, CalibrationReport, CalibrationStore, calibrate_eval_report,
};
use beater_composio::{
    ComposioClient, ComposioError, ConnectionLink, ConnectionStatus, ConnectorTool,
    ConnectorToolPolicy, ConnectorToolPolicyDecision, ToolExecution, Toolkit, skill,
};
#[cfg(feature = "billing")]
use beater_core::OrganizationId;
use beater_core::{
    AgentReleaseId, AnnotationId, ApiKeyId, ArtifactId, DatasetCaseId, DatasetId, DatasetVersionId,
    EnvironmentId, EvaluatorVersionId, ExperimentRunId, GateId, Page, PageRequest, ProjectId,
    PromptId, PromptVersionId, ProviderSecretId, ReviewQueueId, ReviewTaskId, Sha256Hash, SpanId,
    TenantId, TenantScope, TraceId, sha256_json_hash,
};
use beater_datasets::{
    Dataset, DatasetEvalReport, DatasetEvalSpec, DatasetJudgeEvalSpec, DatasetStore,
    DatasetVersionSnapshot, evaluate_dataset_version, evaluate_dataset_version_with_judge,
    promote_trace_span_to_case,
};
use beater_eval::{EvaluationCase, EvaluatorKind, EvaluatorSpec};
use beater_experiments::{
    CaseOutputOverride, ExperimentRunReport, ExperimentRunSpec, ExperimentStore,
    JudgeExperimentRunSpec, run_deterministic_experiment, run_judge_experiment,
};
use beater_gates::{GateDefinition, GateRunReport, GateStore, InconclusivePolicy, run_gate};
use beater_human::{
    CreateReviewQueueRequest as CreateReviewQueueStoreRequest, EnqueueReviewTaskRequest,
    HumanReviewStore, ReviewAnnotation, ReviewQueue, ReviewTask, ReviewTaskState, ReviewVerdict,
    SubmitAnnotationRequest, promote_review_annotation_to_dataset_case,
};
use beater_ingest::{
    DeadLetterReplayReport, IngestError, IngestOutcome, IngestQueueStatus, IngestService,
    NativeIngestRequest, TraceIngestedDrainReport, TraceIngestedReconcileReport,
    TraceWriteDrainReport, anonymous_auth_context,
};
use beater_judge::{
    JudgeBroker, JudgeBrokerError, JudgeBrokerOutcome, JudgeBrokerRequest, JudgeLedgerStore,
};
use beater_oauth::OAuthStore;
use beater_otlp::{
    OtlpScopeHints, decode_export_trace_request, decode_export_trace_request_json,
    export_scope_hints, export_to_raw_trace_ingest_request,
    export_to_raw_trace_ingest_request_with_mime_type,
};
use beater_prompts::{
    AddPromptVersion, CreatePrompt, CreatedPrompt, Prompt, PromptRegistry, PromptRegistryError,
    PromptTemplate, PromptVersion, PromptVersionDiff,
};
use beater_scenarios::{
    FailureMode, PerturbationKnobs, Scenario, ScenarioCluster, ScenarioStore, TraceSummary,
    cluster_failures,
};
use beater_schema::{
    AgentSpanKind, ArtifactRef, AuthContext, CanonicalSpan, RedactionClass, RunFilter, RunSummary,
    SpanStatus, TraceView,
};
use beater_search::{
    NoopSearchIndex, SearchIndex, SearchRequest, SearchResponse, TraceIngestedSearchProcessor,
};
use beater_secrets::{
    ProviderSecretMetadata, ProviderSecretStore, PutProviderSecretRequest, RevokedProviderSecret,
};
use beater_security::{
    ApiScope, CreatedApiKey, SecurityError, api_key_id_from_secret, verify_api_key,
};
use beater_store::{MetadataStore, StoreError, TraceStore};
use beater_store_memory::InMemoryMetadataStore;
use beater_usage::{
    UsageLedgerStore, UsageRecordInsert, UsageSummary, judge_usage_from_dataset_eval_report,
    judge_usage_from_experiment_report, judge_usage_from_outcome, record_usage_batch,
};
use chrono::Utc;
use http::header::{HeaderName, HeaderValue, RETRY_AFTER};
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

pub mod openapi;

const API_KEY_HEADER: &str = "x-beater-api-key";
const TENANT_ID_HEADER: &str = "x-beater-tenant-id";
const PROJECT_ID_HEADER: &str = "x-beater-project-id";
const ENVIRONMENT_ID_HEADER: &str = "x-beater-environment-id";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    Disabled,
    Required,
}

#[derive(Clone)]
pub struct ApiState {
    ingest: IngestService,
    traces: Arc<dyn TraceStore>,
    metadata: Arc<dyn MetadataStore>,
    search: Arc<dyn SearchIndex>,
    archive: Option<ParquetTraceArchive>,
    datasets: Option<Arc<dyn DatasetStore>>,
    scenarios: Option<Arc<dyn ScenarioStore>>,
    experiments: Option<Arc<dyn ExperimentStore>>,
    gates: Option<Arc<dyn GateStore>>,
    human_reviews: Option<Arc<dyn HumanReviewStore>>,
    calibrations: Option<Arc<dyn CalibrationStore>>,
    alerts: Arc<AlertEngine>,
    auth_mode: AuthMode,
    api_keys: Option<Arc<dyn ApiKeyStore>>,
    provider_secrets: Option<Arc<dyn ProviderSecretStore>>,
    /// Beater-owned policy gate for connector execution. The connector provider
    /// supplies metadata and performs execution; this policy decides whether a
    /// caller may use a specific external tool before execution reaches the
    /// provider.
    connector_tool_policy: ConnectorToolPolicy,
    judge_broker: Option<Arc<dyn JudgeBroker>>,
    judge_ledger: Option<Arc<dyn JudgeLedgerStore>>,
    usage: Option<Arc<dyn UsageLedgerStore>>,
    #[cfg(feature = "billing")]
    billing: Option<Arc<dyn BillingStore>>,
    /// Stripe webhook signing secret (HMAC). Required for the Stripe webhook
    /// route to verify inbound deliveries.
    #[cfg(feature = "billing")]
    stripe_webhook_secret: Option<Vec<u8>>,
    audit: Option<Arc<dyn AuditStore>>,
    /// Composio-backed third-party tool provider. When set, the `/v1/connectors`
    /// endpoints broker managed-OAuth connections and tool execution; when unset
    /// (no `COMPOSIO_API_KEY`), those endpoints report 501 and the core runs with
    /// zero cloud dependency.
    connectors: Option<Arc<dyn ComposioClient>>,
    prompts: Option<Arc<dyn PromptRegistry>>,
    /// OAuth 2.1 authorization-server store. When set, `authorize()` also
    /// accepts OAuth access tokens (`bao_...`) as bearer credentials, mapping
    /// the token's tenant scope + OAuth scopes onto the request.
    oauth: Option<Arc<dyn OAuthStore>>,
    /// URL of the OAuth protected-resource metadata document, surfaced in the
    /// `WWW-Authenticate` challenge on 401 (RFC 9728 / MCP auth discovery).
    oauth_metadata_url: Option<String>,
    /// OAuth resource indicator / intended audience for accepted access tokens.
    oauth_resource: Option<String>,
}

impl ApiState {
    pub fn new(ingest: IngestService, traces: Arc<dyn TraceStore>) -> Self {
        Self::base(ingest, traces)
    }

    fn base(ingest: IngestService, traces: Arc<dyn TraceStore>) -> Self {
        Self {
            ingest,
            traces,
            metadata: Arc::new(InMemoryMetadataStore::new()),
            search: Arc::new(NoopSearchIndex),
            archive: None,
            datasets: None,
            scenarios: None,
            experiments: None,
            gates: None,
            human_reviews: None,
            calibrations: None,
            alerts: Arc::new(AlertEngine::new()),
            auth_mode: AuthMode::Disabled,
            api_keys: None,
            provider_secrets: None,
            connector_tool_policy: ConnectorToolPolicy::default(),
            judge_broker: None,
            judge_ledger: None,
            usage: None,
            #[cfg(feature = "billing")]
            billing: None,
            #[cfg(feature = "billing")]
            stripe_webhook_secret: None,
            audit: None,
            connectors: None,
            prompts: None,
            oauth: None,
            oauth_metadata_url: None,
            oauth_resource: None,
        }
    }

    /// Wire an OAuth authorization-server store so `authorize()` accepts OAuth
    /// access tokens in addition to API keys. `metadata_url` is advertised in
    /// the `WWW-Authenticate` challenge on auth failures.
    pub fn with_oauth(
        mut self,
        oauth: Arc<dyn OAuthStore>,
        metadata_url: Option<String>,
        resource: String,
    ) -> Self {
        self.oauth = Some(oauth);
        self.oauth_metadata_url = metadata_url;
        self.oauth_resource = Some(resource);
        self
    }

    /// The OAuth protected-resource metadata URL, if configured (for the MCP
    /// resource server's `WWW-Authenticate` discovery header).
    pub fn oauth_metadata_url(&self) -> Option<&str> {
        self.oauth_metadata_url.as_deref()
    }

    pub fn oauth_resource(&self) -> Option<&str> {
        self.oauth_resource.as_deref()
    }

    pub fn with_search(
        ingest: IngestService,
        traces: Arc<dyn TraceStore>,
        search: Arc<dyn SearchIndex>,
    ) -> Self {
        Self::base(ingest, traces).with_search_index(search)
    }

    pub fn with_search_and_archive(
        ingest: IngestService,
        traces: Arc<dyn TraceStore>,
        search: Arc<dyn SearchIndex>,
        archive: ParquetTraceArchive,
    ) -> Self {
        Self::base(ingest, traces)
            .with_search_index(search)
            .with_archive(archive)
    }

    pub fn with_integrations(
        ingest: IngestService,
        traces: Arc<dyn TraceStore>,
        search: Arc<dyn SearchIndex>,
        archive: ParquetTraceArchive,
        datasets: Arc<dyn DatasetStore>,
        experiments: Arc<dyn ExperimentStore>,
    ) -> Self {
        Self::base(ingest, traces)
            .with_search_index(search)
            .with_archive(archive)
            .with_datasets(datasets)
            .with_experiments(experiments)
    }

    pub fn require_auth(mut self, api_keys: Arc<dyn ApiKeyStore>) -> Self {
        self.auth_mode = AuthMode::Required;
        self.api_keys = Some(api_keys);
        self
    }

    pub fn with_search_index(mut self, search: Arc<dyn SearchIndex>) -> Self {
        self.search = search;
        self
    }

    pub fn with_archive(mut self, archive: ParquetTraceArchive) -> Self {
        self.archive = Some(archive);
        self
    }

    pub fn with_datasets(mut self, datasets: Arc<dyn DatasetStore>) -> Self {
        self.datasets = Some(datasets);
        self
    }

    pub fn with_scenarios(mut self, scenarios: Arc<dyn ScenarioStore>) -> Self {
        self.scenarios = Some(scenarios);
        self
    }

    pub fn with_experiments(mut self, experiments: Arc<dyn ExperimentStore>) -> Self {
        self.experiments = Some(experiments);
        self
    }

    pub fn with_metadata(mut self, metadata: Arc<dyn MetadataStore>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_judge(
        mut self,
        provider_secrets: Arc<dyn ProviderSecretStore>,
        judge_broker: Arc<dyn JudgeBroker>,
        judge_ledger: Arc<dyn JudgeLedgerStore>,
    ) -> Self {
        self.provider_secrets = Some(provider_secrets);
        self.judge_broker = Some(judge_broker);
        self.judge_ledger = Some(judge_ledger);
        self
    }

    pub fn with_gates(mut self, gates: Arc<dyn GateStore>) -> Self {
        self.gates = Some(gates);
        self
    }

    pub fn with_connector_tool_policy(mut self, policy: ConnectorToolPolicy) -> Self {
        self.connector_tool_policy = policy;
        self
    }

    pub fn with_human_reviews(mut self, human_reviews: Arc<dyn HumanReviewStore>) -> Self {
        self.human_reviews = Some(human_reviews);
        self
    }

    pub fn with_calibrations(mut self, calibrations: Arc<dyn CalibrationStore>) -> Self {
        self.calibrations = Some(calibrations);
        self
    }

    pub fn with_usage(mut self, usage: Arc<dyn UsageLedgerStore>) -> Self {
        self.usage = Some(usage);
        self
    }

    /// Wire the billing store and the Stripe webhook signing secret. Enables the
    /// `/v1/plans`, `/v1/subscriptions`, `/v1/billing/invoices` and
    /// `/v1/billing/webhooks/stripe` routes.
    ///
    /// Billing is a hosted concern and is gated behind the non-default `billing`
    /// cargo feature so the open-source build neither compiles billing code nor
    /// advertises Stripe endpoints in the OSS API contract.
    #[cfg(feature = "billing")]
    pub fn with_billing(
        mut self,
        billing: Arc<dyn BillingStore>,
        stripe_webhook_secret: impl Into<Vec<u8>>,
    ) -> Self {
        self.billing = Some(billing);
        self.stripe_webhook_secret = Some(stripe_webhook_secret.into());
        self
    }

    pub fn with_audit(mut self, audit: Arc<dyn AuditStore>) -> Self {
        self.audit = Some(audit);
        self
    }

    /// Wire a Composio-backed connector provider so the `/v1/connectors`
    /// endpoints (and, via the spec→MCP bridge, the `invokeConnectorTool` MCP
    /// tool) can discover, connect, and execute third-party tools.
    pub fn with_connectors(mut self, connectors: Arc<dyn ComposioClient>) -> Self {
        self.connectors = Some(connectors);
        self
    }

    pub fn with_prompts(mut self, prompts: Arc<dyn PromptRegistry>) -> Self {
        self.prompts = Some(prompts);
        self
    }

    fn auth_required(&self) -> bool {
        self.auth_mode == AuthMode::Required
    }
}

/// Number of distinct `/v1/...` HTTP operations (method + path pairs) registered
/// in [`router`]. This excludes the non-versioned `/health` and `/openapi.json`
/// routes. It MUST equal the count of documented `/v1` operations in the OpenAPI
/// spec; the `openapi_coverage` integration test enforces this both ways.
///
/// Update this when adding or removing a `/v1` route in [`router`].
///
/// Billing/Stripe routes are hosted-only and compiled in only under the
/// `billing` feature, so the count is feature-aware: the open-source default
/// build registers 58 `/v1` routes (42 base + 6 always-on `/v1/connectors`
/// routes + 6 `/v1/prompts` routes + 4 `/v1/scenarios` routes); the hosted
/// `billing` build adds 8.
pub const V1_ROUTE_COUNT: usize = if cfg!(feature = "billing") { 66 } else { 58 };

/// See [`V1_ROUTE_COUNT`].
pub fn v1_route_count() -> usize {
    V1_ROUTE_COUNT
}

pub fn router(state: ApiState) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi_json))
        .route("/v1/traces/native", post(ingest_native))
        .route("/v1/traces", post(ingest_otlp_json_collector))
        .route(
            "/v1/api-keys/:tenant_id/:project_id/:environment_id",
            post(create_api_key_route),
        )
        .route(
            "/v1/api-keys/:tenant_id/:project_id/:environment_id/:api_key_id/revoke",
            post(revoke_api_key_route),
        )
        .route(
            "/v1/provider-secrets/:tenant_id/:project_id",
            get(list_provider_secrets_route).post(create_provider_secret_route),
        )
        .route(
            "/v1/provider-secrets/:tenant_id/:project_id/:provider_secret_id/revoke",
            post(revoke_provider_secret_route),
        )
        .route(
            "/v1/connectors/:tenant_id/:project_id",
            get(list_connectors_route),
        )
        .route(
            "/v1/connectors/:tenant_id/:project_id/tools",
            get(list_connector_tools_route),
        )
        .route(
            "/v1/connectors/:tenant_id/:project_id/skills",
            get(connector_skills_route),
        )
        .route(
            "/v1/connectors/:tenant_id/:project_id/connect",
            post(connect_connector_route),
        )
        .route(
            "/v1/connectors/:tenant_id/:project_id/status",
            get(connector_status_route),
        )
        .route(
            "/v1/connectors/:tenant_id/:project_id/invoke",
            post(invoke_connector_tool_route),
        )
        .route(
            "/v1/judge/:tenant_id/:project_id/evaluate",
            post(run_judge_eval_route),
        )
        .route(
            "/v1/judge/:tenant_id/:project_id/ledger",
            get(list_judge_ledger_route),
        )
        .route("/v1/usage/:tenant_id/:project_id", get(get_usage_summary_route))
        .route("/v1/audit/:tenant_id/:project_id", get(list_audit_events_route))
        .route(
            "/v1/ingest/:tenant_id/:project_id/queue",
            get(get_ingest_queue_status_route),
        )
        .route(
            "/v1/ingest/:tenant_id/:project_id/traces/:trace_id/reconcile",
            post(reconcile_trace_ingested_route),
        )
        .route(
            "/v1/ingest/:tenant_id/:project_id/dead-letters/:message_id/replay",
            post(replay_dead_letter_route),
        )
        .route(
            "/v1/ingest/:tenant_id/:project_id/trace-writes/drain",
            post(drain_trace_writes_route),
        )
        .route(
            "/v1/ingest/:tenant_id/:project_id/trace-ingested/drain",
            post(drain_trace_ingested_route),
        )
        .route("/v1/search/:tenant_id/spans", get(search_spans))
        .route("/v1/traces/:tenant_id", get(list_traces))
        .route(
            "/v1/spans/:tenant_id/:trace_id/:span_id",
            get(get_span_route),
        )
        .route(
            "/v1/spans/:tenant_id/:trace_id/:span_id/io",
            get(get_span_io_route),
        )
        .route(
            "/v1/archive/:tenant_id/:project_id/:trace_id",
            post(archive_trace),
        )
        .route(
            "/v1/archive/:tenant_id/:project_id/spans",
            get(query_archive_spans),
        )
        .route(
            "/v1/prompts/:tenant_id/:project_id",
            get(list_prompts_route).post(create_prompt_route),
        )
        .route(
            "/v1/prompts/:tenant_id/:project_id/:prompt_id",
            get(get_prompt_route),
        )
        .route(
            "/v1/prompts/:tenant_id/:project_id/:prompt_id/versions",
            get(list_prompt_versions_route).post(add_prompt_version_route),
        )
        .route(
            "/v1/prompts/:tenant_id/:project_id/:prompt_id/diff",
            get(diff_prompt_versions_route),
        )
        .route("/v1/datasets/:tenant_id/:project_id", post(create_dataset))
        .route(
            "/v1/scenarios/:tenant_id/:project_id",
            post(create_scenario).get(list_scenarios),
        )
        .route(
            "/v1/scenarios/:tenant_id/:project_id/mine",
            post(mine_scenarios),
        )
        .route(
            "/v1/scenarios/:tenant_id/:project_id/:scenario_id",
            get(get_scenario),
        )
        .route(
            "/v1/datasets/:tenant_id/:project_id/:dataset_id/cases/from-trace",
            post(promote_dataset_case),
        )
        .route(
            "/v1/datasets/:tenant_id/:project_id/:dataset_id/versions",
            post(create_dataset_version),
        )
        .route(
            "/v1/datasets/:tenant_id/:project_id/:dataset_id/versions/:version_id/evals/deterministic",
            post(run_deterministic_dataset_eval),
        )
        .route(
            "/v1/datasets/:tenant_id/:project_id/:dataset_id/versions/:version_id/evals/judge",
            post(run_judge_dataset_eval),
        )
        .route(
            "/v1/calibrations/:tenant_id/:project_id/:dataset_id/versions/:version_id",
            post(run_calibration_route),
        )
        .route(
            "/v1/experiments/:tenant_id/:project_id/:dataset_id/versions/:version_id/deterministic",
            post(run_deterministic_experiment_route),
        )
        .route(
            "/v1/experiments/:tenant_id/:project_id/:dataset_id/versions/:version_id/judge",
            post(run_judge_experiment_route),
        )
        .route("/v1/gates/:tenant_id/:project_id", post(create_gate_route))
        .route(
            "/v1/gates/:tenant_id/:project_id/:gate_id/run",
            post(run_gate_route),
        )
        .route(
            "/v1/review-queues/:tenant_id/:project_id",
            post(create_review_queue_route),
        )
        .route(
            "/v1/review-queues/:tenant_id/:project_id/:queue_id/tasks",
            get(list_review_tasks_route),
        )
        .route(
            "/v1/review-queues/:tenant_id/:project_id/:queue_id/tasks/from-trace",
            post(enqueue_review_task_from_trace_route),
        )
        .route(
            "/v1/review-queues/:tenant_id/:project_id/:queue_id/tasks/:task_id/annotations",
            post(submit_review_annotation_route),
        )
        .route(
            "/v1/review-queues/:tenant_id/:project_id/:queue_id/tasks/:task_id/annotations/:annotation_id/promote",
            post(promote_review_annotation_route),
        )
        .route(
            "/v1/online/:tenant_id/:project_id/traces/:trace_id/sampling",
            post(decide_online_sampling),
        )
        .route(
            "/v1/alerts/:tenant_id/:project_id/traces/:trace_id/webhook",
            post(evaluate_alert),
        )
        .route(
            "/v1/otlp/:tenant_id/:project_id/:environment_id/v1/traces",
            post(ingest_otlp_http),
        )
        .route(
            "/v1/import/:tenant_id/:project_id/:environment_id",
            post(import_source_route),
        )
        .route("/v1/traces/:tenant_id/:trace_id", get(get_trace));

    // Billing/Stripe is a hosted concern gated behind the non-default `billing`
    // feature; the OSS build neither registers these routes nor advertises them
    // in the API contract.
    #[cfg(feature = "billing")]
    let router = router
        .route("/v1/plans", get(get_plans_route))
        .route("/v1/plans/:plan_id", get(get_plan_route))
        .route(
            "/v1/subscriptions/:org_id",
            get(get_subscription_route).post(create_subscription_route),
        )
        .route(
            "/v1/subscriptions/:org_id/change-plan",
            post(change_subscription_plan_route),
        )
        .route("/v1/billing/invoices/:org_id", get(get_org_invoices_route))
        .route(
            "/v1/billing/invoices/:org_id/:period_key",
            get(get_invoice_route),
        )
        .route("/v1/billing/webhooks/stripe", post(stripe_webhook_route));

    router.with_state(state)
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    operation_id = "health",
    responses(
        (status = 200, description = "Runtime is accepting requests", body = HealthResponse),
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi::openapi())
}

#[utoipa::path(
    post,
    path = "/v1/traces/native",
    tag = "ingest",
    operation_id = "ingestNative",
    params(
        IngestDurabilityQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = NativeIngestRequest,
    responses(
        (status = 200, description = "Ingest native canonical spans", body = IngestOutcome),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 413, description = "Payload or attribute cardinality too large", body = ErrorResponse),
        (status = 429, description = "Per-project quota exceeded or backpressure", body = ErrorResponse),
    )
)]
async fn ingest_native(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<IngestDurabilityQuery>,
    Json(mut request): Json<NativeIngestRequest>,
) -> Result<Json<IngestOutcome>, ApiError> {
    let auth = authorize(
        &state,
        &headers,
        &request.scope.tenant_id,
        &request.scope.project_id,
        &request.scope.environment_id,
        ApiScope::TraceWrite,
    )
    .await?;
    request.auth_context = Some(auth.context);
    let outcome = if ingest_buffered(&params)? {
        state.ingest.buffer_native(request).await?
    } else {
        state.ingest.ingest_native(request).await?
    };
    Ok(Json(outcome))
}

#[utoipa::path(
    post,
    path = "/v1/api-keys/{tenant_id}/{project_id}/{environment_id}",
    tag = "apiKeys",
    operation_id = "createApiKey",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("environment_id" = String, Path, description = "environment_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateApiKeyHttpRequest,
    responses(
        (status = 200, description = "Create a scoped API key", body = ApiKeyCreatedResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_api_key_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, environment_id)): Path<(String, String, String)>,
    Json(request): Json<CreateApiKeyHttpRequest>,
) -> Result<Json<ApiKeyCreatedResponse>, ApiError> {
    let api_keys = state
        .api_keys
        .clone()
        .ok_or_else(|| ApiError::not_implemented("api key store is not configured".to_string()))?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let environment_id = EnvironmentId::new(environment_id)?;
    let auth = authorize(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        &environment_id,
        ApiScope::Admin,
    )
    .await?;
    ensure_environment_exists(&state, &tenant_id, &project_id, &environment_id).await?;
    let created = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            environment_id: environment_id.clone(),
            scopes: request.scopes,
        })
        .await?;
    append_optional_audit_event(
        &state,
        AuditEventInsert {
            tenant_id,
            project_id,
            environment_id: Some(environment_id),
            actor_api_key_id: auth.context.api_key_id.clone(),
            action: AuditAction::ApiKeyCreate,
            resource_type: "api_key".to_string(),
            resource_id: created.record.api_key_id.as_str().to_string(),
            outcome: AuditOutcome::Allowed,
            reason: None,
            attributes: serde_json::json!({
                "scopes": created
                    .record
                    .scopes
                    .iter()
                    .map(ApiScope::as_str)
                    .collect::<Vec<_>>(),
                "active": created.record.active,
            }),
        },
    )
    .await?;
    Ok(Json(ApiKeyCreatedResponse::from_created(created)))
}

#[utoipa::path(
    post,
    path = "/v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke",
    tag = "apiKeys",
    operation_id = "revokeApiKey",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("environment_id" = String, Path, description = "environment_id"),
        ("api_key_id" = String, Path, description = "api_key_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Revoke an API key", body = RevokedApiKey),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn revoke_api_key_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, environment_id, api_key_id)): Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Result<Json<RevokedApiKey>, ApiError> {
    let api_keys = state
        .api_keys
        .clone()
        .ok_or_else(|| ApiError::not_implemented("api key store is not configured".to_string()))?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let environment_id = EnvironmentId::new(environment_id)?;
    let auth = authorize(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        &environment_id,
        ApiScope::Admin,
    )
    .await?;
    let api_key_id = ApiKeyId::new(api_key_id)?;
    let record = api_keys
        .get_key(api_key_id.clone())
        .await?
        .ok_or_else(|| ApiError::not_found(format!("api key {} not found", api_key_id.as_str())))?;
    if record.tenant_id != tenant_id
        || record.project_id != project_id
        || record.environment_id != environment_id
    {
        return Err(ApiError::not_found(format!(
            "api key {} not found",
            api_key_id.as_str()
        )));
    }
    let revoked = api_keys
        .revoke_key(api_key_id.clone(), Utc::now())
        .await?
        .ok_or_else(|| ApiError::not_found(format!("api key {} not found", api_key_id.as_str())))?;
    append_optional_audit_event(
        &state,
        AuditEventInsert {
            tenant_id,
            project_id,
            environment_id: Some(environment_id),
            actor_api_key_id: auth.context.api_key_id.clone(),
            action: AuditAction::ApiKeyRevoke,
            resource_type: "api_key".to_string(),
            resource_id: api_key_id.as_str().to_string(),
            outcome: AuditOutcome::Allowed,
            reason: None,
            attributes: serde_json::json!({
                "active": revoked.active,
                "rotated_at": revoked.rotated_at.to_rfc3339(),
            }),
        },
    )
    .await?;
    Ok(Json(revoked))
}

#[utoipa::path(
    post,
    path = "/v1/provider-secrets/{tenant_id}/{project_id}",
    tag = "providerSecrets",
    operation_id = "createProviderSecret",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateProviderSecretHttpRequest,
    responses(
        (status = 200, description = "Store an encrypted provider secret", body = ProviderSecretMetadata),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_provider_secret_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreateProviderSecretHttpRequest>,
) -> Result<Json<ProviderSecretMetadata>, ApiError> {
    let provider_secrets = provider_secret_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let auth =
        authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let metadata = provider_secrets
        .put_secret(PutProviderSecretRequest {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            provider: request.provider,
            display_name: request.display_name,
            secret_value: request.secret_value,
        })
        .await?;
    append_optional_audit_event(
        &state,
        AuditEventInsert {
            tenant_id,
            project_id,
            environment_id: auth.environment_id.clone(),
            actor_api_key_id: auth.context.api_key_id.clone(),
            action: AuditAction::ProviderSecretCreate,
            resource_type: "provider_secret".to_string(),
            resource_id: metadata.provider_secret_id.as_str().to_string(),
            outcome: AuditOutcome::Allowed,
            reason: None,
            attributes: serde_json::json!({
                "provider": metadata.provider.as_str(),
                "display_name": metadata.display_name.as_str(),
                "active": metadata.active,
            }),
        },
    )
    .await?;
    Ok(Json(metadata))
}

#[utoipa::path(
    get,
    path = "/v1/provider-secrets/{tenant_id}/{project_id}",
    tag = "providerSecrets",
    operation_id = "listProviderSecrets",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List provider secret metadata", body = Vec < ProviderSecretMetadata >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn list_provider_secrets_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<Vec<ProviderSecretMetadata>>, ApiError> {
    let provider_secrets = provider_secret_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let secrets = provider_secrets
        .list_secret_metadata(tenant_id, project_id)
        .await?;
    Ok(Json(secrets))
}

#[utoipa::path(
    post,
    path = "/v1/provider-secrets/{tenant_id}/{project_id}/{provider_secret_id}/revoke",
    tag = "providerSecrets",
    operation_id = "revokeProviderSecret",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("provider_secret_id" = String, Path, description = "provider_secret_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Revoke a provider secret", body = RevokedProviderSecret),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn revoke_provider_secret_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, provider_secret_id)): Path<(String, String, String)>,
) -> Result<Json<RevokedProviderSecret>, ApiError> {
    let provider_secrets = provider_secret_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let provider_secret_id = ProviderSecretId::new(provider_secret_id)?;
    let auth =
        authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let revoked = provider_secrets
        .revoke_secret(
            tenant_id.clone(),
            project_id.clone(),
            provider_secret_id.clone(),
            Utc::now(),
        )
        .await?
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "provider secret {} not found",
                provider_secret_id.as_str()
            ))
        })?;
    append_optional_audit_event(
        &state,
        AuditEventInsert {
            tenant_id,
            project_id,
            environment_id: auth.environment_id.clone(),
            actor_api_key_id: auth.context.api_key_id.clone(),
            action: AuditAction::ProviderSecretRevoke,
            resource_type: "provider_secret".to_string(),
            resource_id: provider_secret_id.as_str().to_string(),
            outcome: AuditOutcome::Allowed,
            reason: None,
            attributes: serde_json::json!({
                "active": revoked.active,
                "rotated_at": revoked.rotated_at.to_rfc3339(),
            }),
        },
    )
    .await?;
    Ok(Json(revoked))
}

// Connectors (Composio-backed third-party tools)
//
// These broker managed-OAuth connections and tool execution for an agent's
// `tool_set` lever. Every operation here auto-becomes an MCP tool (the MCP
// catalog is derived from the `/v1` surface), so an RSI agent session reaches
// Composio tools through the same path it already uses for Beater's own tools.
// The Composio "entity" is keyed per project: `beater:{tenant}:{project}`, so
// connections are isolated per project under one shared Composio API key.
// ---------------------------------------------------------------------------

/// Maximum apps returned from the catalog listing.
const CONNECTOR_CATALOG_LIMIT: u32 = 100;
/// Maximum tools returned for a single toolkit.
const CONNECTOR_TOOL_LIMIT: u32 = 100;

/// Composio entity id Beater uses for a project's connections.
fn connector_user_id(tenant_id: &TenantId, project_id: &ProjectId) -> String {
    format!("beater:{}:{}", tenant_id.as_str(), project_id.as_str())
}

fn connector_client(state: &ApiState) -> Result<Arc<dyn ComposioClient>, ApiError> {
    require(
        &state.connectors,
        "connector provider (set COMPOSIO_API_KEY)",
    )
}

/// Map a Composio client error onto an API error. Upstream 4xx (bad arguments)
/// surface as 400; auth/transport/decode failures are our configuration problem
/// and surface as 500.
fn map_composio_err(err: ComposioError) -> ApiError {
    match err {
        ComposioError::Api { status, message }
            if (400..500).contains(&status) && status != 401 && status != 403 =>
        {
            ApiError::bad_request(format!("composio: {message}"))
        }
        other => ApiError::internal(format!("composio: {other}")),
    }
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct ConnectConnectorRequest {
    /// Toolkit slug to connect (e.g. `github`, `gmail`, `slack`).
    toolkit: String,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct InvokeConnectorRequest {
    /// Tool slug to execute (e.g. `GITHUB_CREATE_AN_ISSUE`).
    tool: String,
    /// Arguments object matching the tool's input schema.
    #[serde(default)]
    #[schema(value_type = Object)]
    arguments: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ToolkitQuery {
    /// Toolkit slug to scope the request to.
    toolkit: String,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct CatalogQuery {
    /// Maximum number of apps to return (page size).
    limit: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ToolListQuery {
    /// Toolkit slug to list tools for.
    toolkit: String,
    /// Maximum number of tools to return (page size).
    limit: Option<u32>,
}

/// Generated prompting scaffold ("skills.md") for a toolkit's tools.
#[derive(Clone, Debug, Serialize, ToSchema)]
struct ConnectorSkillsResponse {
    /// Toolkit the skills document covers.
    toolkit: String,
    /// Markdown document: one skill card per tool, ready to splice into an
    /// agent's system prompt.
    skills: String,
}

#[utoipa::path(
    get,
    path = "/v1/connectors/{tenant_id}/{project_id}",
    tag = "connectors",
    operation_id = "listConnectors",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        CatalogQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List connectable third-party apps (catalog)", body = Vec < Toolkit >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 501, description = "Connector provider not configured", body = ErrorResponse),
    )
)]
async fn list_connectors_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<Vec<Toolkit>>, ApiError> {
    let connectors = connector_client(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let limit = query.limit.unwrap_or(CONNECTOR_CATALOG_LIMIT);
    let toolkits = connectors
        .list_toolkits(limit)
        .await
        .map_err(map_composio_err)?;
    Ok(Json(toolkits))
}

#[utoipa::path(
    get,
    path = "/v1/connectors/{tenant_id}/{project_id}/tools",
    tag = "connectors",
    operation_id = "listConnectorTools",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ToolListQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List a toolkit's executable tools with input schemas", body = Vec < ConnectorTool >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 501, description = "Connector provider not configured", body = ErrorResponse),
    )
)]
async fn list_connector_tools_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Query(query): Query<ToolListQuery>,
) -> Result<Json<Vec<ConnectorTool>>, ApiError> {
    let connectors = connector_client(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let limit = query.limit.unwrap_or(CONNECTOR_TOOL_LIMIT);
    let tools = connectors
        .list_tools(&query.toolkit, limit)
        .await
        .map_err(map_composio_err)?;
    Ok(Json(tools))
}

#[utoipa::path(
    get,
    path = "/v1/connectors/{tenant_id}/{project_id}/skills",
    tag = "connectors",
    operation_id = "getConnectorSkills",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ToolkitQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Generated prompting scaffold (skill cards) for a toolkit", body = ConnectorSkillsResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 501, description = "Connector provider not configured", body = ErrorResponse),
    )
)]
async fn connector_skills_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Query(query): Query<ToolkitQuery>,
) -> Result<Json<ConnectorSkillsResponse>, ApiError> {
    let connectors = connector_client(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let tools = connectors
        .list_tools(&query.toolkit, CONNECTOR_TOOL_LIMIT)
        .await
        .map_err(map_composio_err)?;
    let allowed_tools: Vec<ConnectorTool> = tools
        .into_iter()
        .filter(|tool| state.connector_tool_policy.evaluate(tool).allowed)
        .collect();
    Ok(Json(ConnectorSkillsResponse {
        toolkit: query.toolkit,
        skills: skill::skills_doc(&allowed_tools),
    }))
}

#[utoipa::path(
    post,
    path = "/v1/connectors/{tenant_id}/{project_id}/connect",
    tag = "connectors",
    operation_id = "connectConnector",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = ConnectConnectorRequest,
    responses(
        (status = 200, description = "One-time login link to authorize the app", body = ConnectionLink),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 501, description = "Connector provider not configured", body = ErrorResponse),
    )
)]
async fn connect_connector_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<ConnectConnectorRequest>,
) -> Result<Json<ConnectionLink>, ApiError> {
    let connectors = connector_client(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let user_id = connector_user_id(&tenant_id, &project_id);
    let link = connectors
        .connect(&request.toolkit, &user_id)
        .await
        .map_err(map_composio_err)?;
    Ok(Json(link))
}

#[utoipa::path(
    get,
    path = "/v1/connectors/{tenant_id}/{project_id}/status",
    tag = "connectors",
    operation_id = "connectorStatus",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ToolkitQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Connection status of a toolkit for this project", body = ConnectionStatus),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 501, description = "Connector provider not configured", body = ErrorResponse),
    )
)]
async fn connector_status_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Query(query): Query<ToolkitQuery>,
) -> Result<Json<ConnectionStatus>, ApiError> {
    let connectors = connector_client(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let user_id = connector_user_id(&tenant_id, &project_id);
    let status = connectors
        .connection_status(&query.toolkit, &user_id)
        .await
        .map_err(map_composio_err)?;
    Ok(Json(status))
}

#[utoipa::path(
    post,
    path = "/v1/connectors/{tenant_id}/{project_id}/invoke",
    tag = "connectors",
    operation_id = "invokeConnectorTool",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = InvokeConnectorRequest,
    responses(
        (status = 200, description = "Execute a connector tool and return its result envelope", body = ToolExecution),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 501, description = "Connector provider not configured", body = ErrorResponse),
    )
)]
async fn invoke_connector_tool_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<InvokeConnectorRequest>,
) -> Result<Json<ToolExecution>, ApiError> {
    let connectors = connector_client(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let auth =
        authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun)
            .await?;
    let tool = connectors
        .get_tool(&request.tool)
        .await
        .map_err(map_composio_err)?;
    if !tool.slug.eq_ignore_ascii_case(request.tool.trim()) {
        return Err(ApiError::internal(format!(
            "connector provider returned metadata for {} while resolving {}",
            tool.slug, request.tool
        )));
    }
    let policy_decision = state.connector_tool_policy.evaluate(&tool);
    record_connector_tool_policy_audit_if_configured(
        &state,
        &tenant_id,
        &project_id,
        &auth,
        &tool,
        &request.arguments,
        &policy_decision,
    )
    .await?;
    if !policy_decision.allowed {
        return Err(ApiError::forbidden(policy_decision.reason));
    }
    let user_id = connector_user_id(&tenant_id, &project_id);
    let execution = connectors
        .execute(&request.tool, &user_id, request.arguments)
        .await
        .map_err(map_composio_err)?;
    Ok(Json(execution))
}

async fn record_connector_tool_policy_audit_if_configured(
    state: &ApiState,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    auth: &AuthDecision,
    tool: &ConnectorTool,
    arguments: &serde_json::Value,
    policy_decision: &ConnectorToolPolicyDecision,
) -> Result<(), ApiError> {
    let Some(audit) = state.audit.as_ref().cloned() else {
        return Ok(());
    };
    let arguments_hash = sha256_json_hash(arguments)
        .map_err(|err| ApiError::internal(format!("hash connector tool arguments: {err}")))?;
    let outcome = if policy_decision.allowed {
        AuditOutcome::Allowed
    } else {
        AuditOutcome::Denied
    };
    audit
        .append_event(connector_tool_invoke_event(ConnectorToolInvokeAuditInput {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            environment_id: auth.environment_id.clone(),
            actor_api_key_id: auth.context.api_key_id.clone(),
            tool_slug: tool.slug.clone(),
            outcome,
            reason: Some(policy_decision.reason.clone()),
            attributes: serde_json::json!({
                "arguments_hash": arguments_hash.as_str(),
                "approval_id": policy_decision.approval_id,
                "connector": "composio",
                "no_auth": tool.no_auth,
                "policy_decision": if policy_decision.allowed { "allowed" } else { "denied" },
                "risk_class": policy_decision.risk_class.as_str(),
                "tool": tool.slug,
                "toolkit": tool.toolkit,
            }),
        }))
        .await?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/v1/judge/{tenant_id}/{project_id}/evaluate",
    tag = "judge",
    operation_id = "evaluateJudge",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunJudgeEvalHttpRequest,
    responses(
        (status = 200, description = "Run an ad-hoc judge evaluation", body = JudgeBrokerOutcome),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn run_judge_eval_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<RunJudgeEvalHttpRequest>,
) -> Result<Json<JudgeBrokerOutcome>, ApiError> {
    let judge_broker = judge_broker(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let outcome = judge_broker
        .evaluate(JudgeBrokerRequest {
            tenant_id,
            project_id,
            evaluator: request.evaluator,
            case: request.case,
            provider_secret_id: request.provider_secret_id,
            cache_namespace: request.cache_namespace,
        })
        .await
        .map_err(judge_failure)?;
    record_usage_if_configured(&state, vec![judge_usage_from_outcome(&outcome)]).await?;
    Ok(Json(outcome))
}

#[utoipa::path(
    get,
    path = "/v1/usage/{tenant_id}/{project_id}",
    tag = "usage",
    operation_id = "getUsageSummary",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get usage summary", body = UsageSummary),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn get_usage_summary_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<UsageSummary>, ApiError> {
    let usage = usage_ledger(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let summary = usage.summarize_usage(tenant_id, project_id).await?;
    Ok(Json(summary))
}

// ---------------------------------------------------------------------------
// Billing (Bandwidth, §20.7 #5.8 / §21.7 / R15)
// ---------------------------------------------------------------------------

/// Request body for creating a subscription.
#[cfg(feature = "billing")]
#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateSubscriptionRequest {
    /// Plan to subscribe the org to.
    plan_id: String,
    /// RFC3339 period start.
    period_start: String,
    /// RFC3339 period end.
    period_end: String,
    /// Optional initial status (`active` default).
    status: Option<String>,
}

/// Request body for changing an org's plan with proration.
#[cfg(feature = "billing")]
#[derive(Clone, Debug, Deserialize, ToSchema)]
struct ChangePlanRequest {
    /// New plan id.
    new_plan_id: String,
    /// RFC3339 instant at which the change takes effect (defaults to now).
    at: Option<String>,
}

/// Acknowledgement for an inbound Stripe webhook delivery.
#[cfg(feature = "billing")]
#[derive(Clone, Debug, Serialize, ToSchema)]
struct StripeWebhookAck {
    /// `applied`, `duplicate`, or `stale`.
    outcome: String,
}

#[cfg(feature = "billing")]
#[utoipa::path(
    get,
    path = "/v1/plans",
    tag = "billing",
    operation_id = "getPlans",
    responses(
        (status = 200, description = "List billing plans", body = Vec < Plan >),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_plans_route(State(state): State<ApiState>) -> Result<Json<Vec<Plan>>, ApiError> {
    let billing = billing_store(&state)?;
    let plans = billing.list_plans().await?;
    Ok(Json(plans))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    get,
    path = "/v1/plans/{plan_id}",
    tag = "billing",
    operation_id = "getPlan",
    params(("plan_id" = String, Path, description = "plan_id")),
    responses(
        (status = 200, description = "Get a billing plan", body = Plan),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_plan_route(
    State(state): State<ApiState>,
    Path(plan_id): Path<String>,
) -> Result<Json<Plan>, ApiError> {
    let billing = billing_store(&state)?;
    let plan_id = PlanId::new(plan_id).map_err(ApiError::from)?;
    let plan = billing
        .get_plan(&plan_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("plan {} not found", plan_id.as_str())))?;
    Ok(Json(plan))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    get,
    path = "/v1/subscriptions/{org_id}",
    tag = "billing",
    operation_id = "getSubscription",
    params(
        ("org_id" = String, Path, description = "org_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get an org subscription", body = Subscription),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_subscription_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<Subscription>, ApiError> {
    let billing = billing_store(&state)?;
    let org_id = OrganizationId::new(org_id)?;
    authorize_org_route(&state, &headers, &org_id, ApiScope::Admin).await?;
    let subscription = billing.get_subscription(&org_id).await?.ok_or_else(|| {
        ApiError::not_found(format!(
            "subscription for org {} not found",
            org_id.as_str()
        ))
    })?;
    Ok(Json(subscription))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    post,
    path = "/v1/subscriptions/{org_id}",
    tag = "billing",
    operation_id = "createSubscription",
    params(
        ("org_id" = String, Path, description = "org_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateSubscriptionRequest,
    responses(
        (status = 200, description = "Create an org subscription", body = Subscription),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 409, description = "Subscription already exists", body = ErrorResponse),
    )
)]
async fn create_subscription_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<Json<Subscription>, ApiError> {
    let billing = billing_store(&state)?;
    let org_id = OrganizationId::new(org_id)?;
    authorize_org_route(&state, &headers, &org_id, ApiScope::Admin).await?;
    let plan_id = PlanId::new(request.plan_id).map_err(ApiError::from)?;
    let period_start = parse_required_timestamp(&request.period_start, "period_start")?;
    let period_end = parse_required_timestamp(&request.period_end, "period_end")?;
    if period_end <= period_start {
        return Err(ApiError::bad_request(
            "period_end must be after period_start".to_string(),
        ));
    }
    if billing.get_plan(&plan_id).await?.is_none() {
        return Err(ApiError::not_found(format!(
            "plan {} not found",
            plan_id.as_str()
        )));
    }
    let status = match request.status.as_deref() {
        None | Some("active") => SubscriptionStatus::Active,
        Some(other) => SubscriptionStatus::parse(other).ok_or_else(|| {
            ApiError::bad_request(format!("unknown subscription status: {other}"))
        })?,
    };
    let subscription = billing
        .create_subscription(Subscription {
            org_id,
            plan_id,
            status,
            period_start,
            period_end,
            version: 1,
        })
        .await?;
    Ok(Json(subscription))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    post,
    path = "/v1/subscriptions/{org_id}/change-plan",
    tag = "billing",
    operation_id = "changeSubscriptionPlan",
    params(
        ("org_id" = String, Path, description = "org_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = ChangePlanRequest,
    responses(
        (status = 200, description = "Change an org plan with proration", body = PlanChange),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
        (status = 409, description = "Concurrent modification", body = ErrorResponse),
    )
)]
async fn change_subscription_plan_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
    Json(request): Json<ChangePlanRequest>,
) -> Result<Json<PlanChange>, ApiError> {
    let billing = billing_service(&state)?;
    let org_id = OrganizationId::new(org_id)?;
    authorize_org_route(&state, &headers, &org_id, ApiScope::Admin).await?;
    let new_plan_id = PlanId::new(request.new_plan_id).map_err(ApiError::from)?;
    let at = match request.at {
        Some(value) => parse_required_timestamp(&value, "at")?,
        None => Utc::now(),
    };
    let change = billing.change_plan(&org_id, &new_plan_id, at).await?;
    Ok(Json(change))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    get,
    path = "/v1/billing/invoices/{org_id}",
    tag = "billing",
    operation_id = "getOrgInvoices",
    params(
        ("org_id" = String, Path, description = "org_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List org invoices", body = Vec < Invoice >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn get_org_invoices_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<Invoice>>, ApiError> {
    let billing = billing_store(&state)?;
    let org_id = OrganizationId::new(org_id)?;
    authorize_org_route(&state, &headers, &org_id, ApiScope::Admin).await?;
    let invoices = billing.list_invoices(&org_id).await?;
    Ok(Json(invoices))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    get,
    path = "/v1/billing/invoices/{org_id}/{period_key}",
    tag = "billing",
    operation_id = "getInvoice",
    params(
        ("org_id" = String, Path, description = "org_id"),
        ("period_key" = String, Path, description = "period_key (YYYY-MM)"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get an org invoice for a period", body = Invoice),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_invoice_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((org_id, period_key)): Path<(String, String)>,
) -> Result<Json<Invoice>, ApiError> {
    let billing = billing_store(&state)?;
    let org_id = OrganizationId::new(org_id)?;
    authorize_org_route(&state, &headers, &org_id, ApiScope::Admin).await?;
    let invoice = billing
        .get_invoice(&org_id, &period_key)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "invoice for org {} period {period_key} not found",
                org_id.as_str()
            ))
        })?;
    Ok(Json(invoice))
}

#[cfg(feature = "billing")]
#[utoipa::path(
    post,
    path = "/v1/billing/webhooks/stripe",
    tag = "billing",
    operation_id = "handleStripeWebhook",
    params(
        ("stripe-signature" = String, Header, description = "Stripe signed-webhook header (t=...,v1=...)"),
    ),
    request_body(content = String, description = "Raw Stripe event JSON (signature-verified)"),
    responses(
        (status = 200, description = "Webhook accepted (applied/duplicate/stale)", body = StripeWebhookAck),
        (status = 400, description = "Invalid signature or malformed event", body = ErrorResponse),
    )
)]
async fn stripe_webhook_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<StripeWebhookAck>, ApiError> {
    let sync = stripe_sync(&state)?;
    let signature = headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::bad_request("missing stripe-signature header".to_string()))?;
    let outcome = sync.apply_event(&body, signature).await?;
    let outcome = match outcome {
        EventApplication::Applied => "applied",
        EventApplication::Duplicate => "duplicate",
        EventApplication::Stale => "stale",
    };
    Ok(Json(StripeWebhookAck {
        outcome: outcome.to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/v1/judge/{tenant_id}/{project_id}/ledger",
    tag = "judge",
    operation_id = "listJudgeLedger",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List judge ledger audit records", body = Vec < beater_judge :: JudgeAuditRecord >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn list_judge_ledger_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<Vec<beater_judge::JudgeAuditRecord>>, ApiError> {
    let judge_ledger = judge_ledger(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let records = judge_ledger.list_records(tenant_id, project_id).await?;
    Ok(Json(records))
}

#[utoipa::path(
    get,
    path = "/v1/ingest/{tenant_id}/{project_id}/queue",
    tag = "ingest",
    operation_id = "getIngestQueueStatus",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get ingest queue status", body = IngestQueueStatus),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn get_ingest_queue_status_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<IngestQueueStatus>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    Ok(Json(
        state.ingest.queue_status(tenant_id, project_id).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay",
    tag = "ingest",
    operation_id = "replayDeadLetter",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("message_id" = String, Path, description = "message_id"),
        ReplayDeadLetterQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Replay a dead-letter message", body = DeadLetterReplayReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn replay_dead_letter_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, message_id)): Path<(String, String, String)>,
    Query(params): Query<ReplayDeadLetterQuery>,
) -> Result<Json<DeadLetterReplayReport>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let reset_attempts = params.reset_attempts.unwrap_or(true);
    let report = state
        .ingest
        .replay_dead_letter(&tenant_id, &project_id, &message_id, reset_attempts)
        .await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile",
    tag = "ingest",
    operation_id = "reconcileTrace",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Reconcile a trace-ingested record", body = TraceIngestedReconcileReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn reconcile_trace_ingested_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, trace_id)): Path<(String, String, String)>,
) -> Result<Json<TraceIngestedReconcileReport>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let trace_id = TraceId::new(trace_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    Ok(Json(
        state
            .ingest
            .reconcile_trace_ingested(tenant_id, project_id, trace_id)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/v1/ingest/{tenant_id}/{project_id}/trace-writes/drain",
    tag = "ingest",
    operation_id = "drainTraceWrites",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        DrainTraceWritesQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Drain pending trace writes", body = TraceWriteDrainReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 422, description = "Drained with dead-letters", body = TraceWriteDrainReport),
    )
)]
async fn drain_trace_writes_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Query(params): Query<DrainTraceWritesQuery>,
) -> Result<(StatusCode, Json<TraceWriteDrainReport>), ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let limit = params.limit.unwrap_or(100).min(1000);
    let report = state
        .ingest
        .drain_trace_writes_for(&tenant_id, &project_id, limit)
        .await?;
    Ok((drain_status(report.dead_lettered), Json(report)))
}

#[utoipa::path(
    post,
    path = "/v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain",
    tag = "ingest",
    operation_id = "drainTraceIngested",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        DrainTraceWritesQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Drain pending trace-ingested events", body = TraceIngestedDrainReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 422, description = "Drained with dead-letters", body = TraceIngestedDrainReport),
    )
)]
async fn drain_trace_ingested_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Query(params): Query<DrainTraceWritesQuery>,
) -> Result<(StatusCode, Json<TraceIngestedDrainReport>), ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let limit = params.limit.unwrap_or(100).min(1000);
    let search_processor =
        TraceIngestedSearchProcessor::new(state.traces.clone(), state.search.clone());
    let report = state
        .ingest
        .drain_trace_ingested_for(&tenant_id, &project_id, limit, move |trace_ref| {
            let search_processor = search_processor.clone();
            async move {
                search_processor
                    .process_trace(
                        trace_ref.tenant_id,
                        trace_ref.project_id,
                        trace_ref.trace_id,
                    )
                    .await
            }
        })
        .await?;
    Ok((drain_status(report.dead_lettered), Json(report)))
}

fn drain_status(dead_lettered: usize) -> StatusCode {
    if dead_lettered > 0 {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    }
}

#[utoipa::path(
    post,
    path = "/v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces",
    tag = "ingest",
    operation_id = "ingestOtlp",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("environment_id" = String, Path, description = "environment_id"),
        IngestDurabilityQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Ingest OTLP/HTTP protobuf traces", body = OtlpIngestOutcome),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 413, description = "Payload or attribute cardinality too large", body = ErrorResponse),
        (status = 429, description = "Per-project quota exceeded or backpressure", body = ErrorResponse),
    )
)]
async fn ingest_otlp_http(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, environment_id)): Path<(String, String, String)>,
    Query(params): Query<IngestDurabilityQuery>,
    body: Bytes,
) -> Result<Json<OtlpIngestOutcome>, ApiError> {
    let scope = TenantScope::new(
        TenantId::new(tenant_id)?,
        ProjectId::new(project_id)?,
        EnvironmentId::new(environment_id)?,
    );
    let auth = authorize(
        &state,
        &headers,
        &scope.tenant_id,
        &scope.project_id,
        &scope.environment_id,
        ApiScope::TraceWrite,
    )
    .await?;
    let export = decode_export_trace_request(&body).map_err(invalid_otlp_export)?;
    let raw_request =
        export_to_raw_trace_ingest_request(scope.clone(), body.to_vec(), export, auth.context)
            .map_err(invalid_otlp_export)?;
    let buffered = ingest_buffered(&params)?;
    let outcome = if buffered {
        state.ingest.buffer_raw_trace_batch(raw_request).await?
    } else {
        state.ingest.ingest_raw_trace_batch(raw_request).await?
    };
    Ok(Json(OtlpIngestOutcome {
        accepted_raw: outcome.ack.accepted_raw,
        accepted_spans: outcome.ack.accepted_spans,
        duplicate_raw: outcome.ack.duplicate_raw,
        duplicate_spans: outcome.ack.duplicate_spans,
        downstream_queued: outcome.downstream_queued,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/traces",
    tag = "ingest",
    operation_id = "ingestOtlpJsonCollector",
    params(
        IngestDurabilityQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-tenant-id" = Option<String>, Header, description = "Tenant scope override for collector-style OTLP JSON"),
        ("x-beater-project-id" = Option<String>, Header, description = "Project scope override for collector-style OTLP JSON"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Environment scope override for collector-style OTLP JSON"),
    ),
    responses(
        (status = 200, description = "Ingest collector-style OTLP/HTTP JSON traces", body = OtlpIngestOutcome),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 413, description = "Payload or attribute cardinality too large", body = ErrorResponse),
        (status = 429, description = "Per-project quota exceeded or backpressure", body = ErrorResponse),
    )
)]
async fn ingest_otlp_json_collector(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<IngestDurabilityQuery>,
    body: Bytes,
) -> Result<Json<OtlpIngestOutcome>, ApiError> {
    let export = decode_export_trace_request_json(&body).map_err(invalid_otlp_export)?;
    let scope = collector_otlp_scope(&headers, export_scope_hints(&export))?;
    let auth = authorize(
        &state,
        &headers,
        &scope.tenant_id,
        &scope.project_id,
        &scope.environment_id,
        ApiScope::TraceWrite,
    )
    .await?;
    let raw_request = export_to_raw_trace_ingest_request_with_mime_type(
        scope.clone(),
        body.to_vec(),
        export,
        auth.context,
        "application/json",
    )
    .map_err(invalid_otlp_export)?;
    let buffered = ingest_buffered(&params)?;
    let outcome = if buffered {
        state.ingest.buffer_raw_trace_batch(raw_request).await?
    } else {
        state.ingest.ingest_raw_trace_batch(raw_request).await?
    };
    Ok(Json(OtlpIngestOutcome {
        accepted_raw: outcome.ack.accepted_raw,
        accepted_spans: outcome.ack.accepted_spans,
        duplicate_raw: outcome.ack.duplicate_raw,
        duplicate_spans: outcome.ack.duplicate_spans,
        downstream_queued: outcome.downstream_queued,
    }))
}

/// Request body for the unified import endpoint. The `source` field selects a
/// registered [`beater_ingest::SourceImporter`] (e.g. `temporal_history`, `native`);
/// `payload` is that source's document (Temporal `History` JSON, a native span list,
/// …). Everything flows through the same downstream ingest pipeline as OTLP — there are
/// no source-specific routes.
#[derive(Clone, Debug, Deserialize, ToSchema)]
struct ImportSourceHttpRequest {
    /// Registered importer key, e.g. `temporal_history` or `native`.
    source: String,
    /// The source-specific document to normalize.
    #[serde(default)]
    payload: serde_json::Value,
}

#[utoipa::path(
    post,
    path = "/v1/import/{tenant_id}/{project_id}/{environment_id}",
    tag = "ingest",
    operation_id = "importSource",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("environment_id" = String, Path, description = "environment_id"),
        IngestDurabilityQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
    ),
    request_body = ImportSourceHttpRequest,
    responses(
        (status = 200, description = "Normalize an imported source document into canonical spans", body = IngestOutcome),
        (status = 400, description = "Invalid request, scope, or unknown source", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 413, description = "Payload or attribute cardinality too large", body = ErrorResponse),
        (status = 429, description = "Per-project quota exceeded or backpressure", body = ErrorResponse),
    )
)]
async fn import_source_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, environment_id)): Path<(String, String, String)>,
    Query(params): Query<IngestDurabilityQuery>,
    Json(request): Json<ImportSourceHttpRequest>,
) -> Result<Json<IngestOutcome>, ApiError> {
    let scope = TenantScope::new(
        TenantId::new(tenant_id)?,
        ProjectId::new(project_id)?,
        EnvironmentId::new(environment_id)?,
    );
    let auth = authorize(
        &state,
        &headers,
        &scope.tenant_id,
        &scope.project_id,
        &scope.environment_id,
        ApiScope::TraceWrite,
    )
    .await?;
    // The stored raw envelope is the canonical-JSON re-serialization of `payload`,
    // not the verbatim request bytes. That's intentional here: it normalizes
    // formatting for the idempotency hash and is lossless for the source documents we
    // import (Temporal history uses small integer ids + base64 payloads). Switch to
    // capturing the raw body if byte-exact archival ever becomes a requirement.
    let raw_bytes = serde_json::to_vec(&request.payload)
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    let buffered = ingest_buffered(&params)?;
    let outcome = state
        .ingest
        .import_source(
            &request.source,
            scope,
            raw_bytes,
            Some(auth.context),
            buffered,
        )
        .await?;
    Ok(Json(outcome))
}

#[utoipa::path(
    get,
    path = "/v1/search/{tenant_id}/spans",
    tag = "search",
    operation_id = "searchSpans",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        SearchQueryParams,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Search spans", body = SearchResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn search_spans(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<SearchQueryParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = params.project_id.clone().map(ProjectId::new).transpose()?;
    let environment_id = params
        .environment_id
        .clone()
        .map(EnvironmentId::new)
        .transpose()?;
    authorize_query_scope(
        &state,
        &headers,
        &tenant_id,
        project_id.as_ref(),
        environment_id.as_ref(),
        ApiScope::TraceRead,
    )
    .await?;
    let request = SearchRequest {
        tenant_id,
        text: params.q.unwrap_or_default(),
        project_id,
        environment_id: params.environment_id,
        trace_id: params.trace_id.map(TraceId::new).transpose()?,
        span_id: params.span_id.map(beater_core::SpanId::new).transpose()?,
        kind: params
            .kind
            .map(parse_span_kind)
            .transpose()?
            .map(|kind| kind.as_str().to_string()),
        status: params
            .status
            .map(parse_span_status)
            .transpose()?
            .map(|status| status.as_str().to_string()),
        model: params.model,
        tool: params.tool,
        limit: params.limit,
    };
    Ok(Json(state.search.search(request).await?))
}

#[utoipa::path(
    get,
    path = "/v1/traces/{tenant_id}",
    tag = "traces",
    operation_id = "listTraces",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ListTracesQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List trace run summaries", body = Page < RunSummary >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn list_traces(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ListTracesQuery>,
) -> Result<Json<Page<RunSummary>>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = params.project_id.map(ProjectId::new).transpose()?;
    let environment_id = params.environment_id.map(EnvironmentId::new).transpose()?;
    let auth = authorize_query_scope(
        &state,
        &headers,
        &tenant_id,
        project_id.as_ref(),
        environment_id.as_ref(),
        ApiScope::TraceRead,
    )
    .await?;
    let filter = RunFilter {
        project_id: auth.project_id.or(project_id),
        environment_id: auth.environment_id.or(environment_id),
        trace_id: params.trace_id.map(TraceId::new).transpose()?,
        kind: params.kind.map(parse_span_kind).transpose()?,
        status: params.status.map(parse_span_status).transpose()?,
        started_after: parse_optional_timestamp(params.started_after, "started_after")?,
        started_before: parse_optional_timestamp(params.started_before, "started_before")?,
        model: params.model,
        release: params.release,
        min_cost_micros: params.min_cost_micros,
        max_cost_micros: params.max_cost_micros,
        min_latency_ms: params.min_latency_ms,
        max_latency_ms: params.max_latency_ms,
    };
    let page = PageRequest {
        limit: params.limit.unwrap_or(50).clamp(1, 200),
        cursor: params.cursor,
    };
    Ok(Json(
        state.traces.query_runs(tenant_id, filter, page).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/v1/traces/{tenant_id}/{trace_id}",
    tag = "traces",
    operation_id = "getTrace",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        TraceReadQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get a canonical trace", body = TraceView),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_trace(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, trace_id)): Path<(String, String)>,
    Query(params): Query<TraceReadQuery>,
) -> Result<Json<TraceView>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let trace_id = TraceId::new(trace_id)?;
    let auth = authorize_tenant_route(&state, &headers, &tenant_id, ApiScope::TraceRead).await?;
    // A trace with no spans is intentionally returned as an empty 200, not a 404.
    // Ingest is asynchronous and eventually consistent, so clients (e.g. the
    // live-smoke `wait_for_trace` poller) read this endpoint immediately after
    // submitting and poll until spans appear. The store cannot distinguish a
    // genuinely-unknown trace id from one whose spans have not landed yet, so
    // 404-ing on empty would break that polling contract.
    let trace = load_trace_for_auth_scope(&state, tenant_id, trace_id, &auth).await?;
    ensure_trace_auth_scope(&trace, &auth)?;
    if params.unmask.unwrap_or(false) {
        let trace =
            authorize_and_audit_trace_unmask(&state, &headers, trace, auth, params.reason).await?;
        return Ok(Json(trace));
    }
    Ok(Json(redact_trace_view(trace)))
}

#[utoipa::path(
    get,
    path = "/v1/spans/{tenant_id}/{trace_id}/{span_id}",
    tag = "spans",
    operation_id = "getSpan",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        ("span_id" = String, Path, description = "span_id"),
        TraceReadQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get a canonical span", body = CanonicalSpan),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_span_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, trace_id, span_id)): Path<(String, String, String)>,
    Query(params): Query<TraceReadQuery>,
) -> Result<Json<CanonicalSpan>, ApiError> {
    let span = load_span_for_route(state, headers, tenant_id, trace_id, span_id, params).await?;
    Ok(Json(span))
}

#[utoipa::path(
    get,
    path = "/v1/spans/{tenant_id}/{trace_id}/{span_id}/io",
    tag = "spans",
    operation_id = "getSpanIo",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        ("span_id" = String, Path, description = "span_id"),
        TraceReadQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get span input/output metadata", body = SpanIoResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_span_io_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, trace_id, span_id)): Path<(String, String, String)>,
    Query(params): Query<TraceReadQuery>,
) -> Result<Json<SpanIoResponse>, ApiError> {
    let span = load_span_for_route(state, headers, tenant_id, trace_id, span_id, params).await?;
    Ok(Json(SpanIoResponse {
        tenant_id: span.tenant_id.clone(),
        trace_id: span.trace_id.clone(),
        span_id: span.span_id.clone(),
        input: span_io_value(&span, "input.value", span.input_ref.as_ref()),
        output: span_io_value(&span, "output.value", span.output_ref.as_ref()),
    }))
}

#[utoipa::path(
    get,
    path = "/v1/audit/{tenant_id}/{project_id}",
    tag = "audit",
    operation_id = "listAuditEvents",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List audit events", body = Vec < AuditEvent >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn list_audit_events_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<Vec<AuditEvent>>, ApiError> {
    let audit = audit_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let events = audit.list_events(tenant_id, project_id).await?;
    Ok(Json(events))
}

#[utoipa::path(
    post,
    path = "/v1/archive/{tenant_id}/{project_id}/{trace_id}",
    tag = "archive",
    operation_id = "archiveTrace",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Archive a trace to object storage", body = ArchiveManifest),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn archive_trace(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, trace_id)): Path<(String, String, String)>,
) -> Result<Json<ArchiveManifest>, ApiError> {
    let archive = state
        .archive
        .clone()
        .ok_or_else(|| ApiError::not_implemented("archive tier is not configured".to_string()))?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let trace_id = TraceId::new(trace_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let trace = state
        .traces
        .get_project_trace(tenant_id.clone(), project_id.clone(), trace_id.clone())
        .await?;
    ensure_trace_project(&trace, &project_id)?;
    ensure_trace_auth_scope(&trace, &auth)?;
    if trace.spans.is_empty() {
        return Err(ApiError::not_found(format!(
            "trace {} not found for tenant {}",
            trace_id.as_str(),
            tenant_id.as_str()
        )));
    }
    let manifest = archive
        .archive_spans(&tenant_id, &project_id, &trace.spans)
        .await?;
    Ok(Json(manifest))
}

#[utoipa::path(
    get,
    path = "/v1/archive/{tenant_id}/{project_id}/spans",
    tag = "archive",
    operation_id = "queryArchiveSpans",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ArchiveQueryParams,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Query archived spans", body = ArchiveQueryResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn query_archive_spans(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<ArchiveQueryParams>,
) -> Result<Json<ArchiveQueryResponse>, ApiError> {
    let archive = state
        .archive
        .clone()
        .ok_or_else(|| ApiError::not_implemented("archive tier is not configured".to_string()))?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let requested_environment_id = params.environment_id.map(EnvironmentId::new).transpose()?;
    let environment_id = match (&auth.environment_id, requested_environment_id) {
        (Some(auth_environment_id), Some(requested_environment_id)) => {
            if auth_environment_id.as_str() != requested_environment_id.as_str() {
                return Err(ApiError::forbidden(
                    "archive query environment does not match authenticated environment"
                        .to_string(),
                ));
            }
            Some(requested_environment_id)
        }
        (Some(auth_environment_id), None) => Some(auth_environment_id.clone()),
        (None, requested_environment_id) => requested_environment_id,
    };
    let query = ArchiveQuery {
        tenant_id: tenant_id.clone(),
        project_id: Some(project_id.clone()),
        environment_id,
        trace_id: params.trace_id.map(TraceId::new).transpose()?,
        span_id: params.span_id.map(SpanId::new).transpose()?,
        kind: params.kind.map(parse_span_kind).transpose()?,
        status: params.status.map(parse_span_status).transpose()?,
        limit: params.limit,
    };
    let rows = archive
        .query_project(&tenant_id, &project_id, query)
        .await?;
    Ok(Json(ArchiveQueryResponse { rows }))
}

/// Request body for `createPrompt`: the new prompt's metadata plus its initial
/// (version 1) template.
#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreatePromptRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    template: PromptTemplate,
    #[serde(default)]
    created_by: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

/// Request body for `addPromptVersion`: a new immutable template revision.
#[derive(Clone, Debug, Deserialize, ToSchema)]
struct AddPromptVersionRequest {
    template: PromptTemplate,
    #[serde(default)]
    created_by: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct PromptListResponse {
    prompts: Vec<Prompt>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct PromptVersionListResponse {
    versions: Vec<PromptVersion>,
}

/// Query parameters for `diffPromptVersions`: the two version ids to compare.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DiffPromptVersionsQuery {
    from: String,
    to: String,
}

#[utoipa::path(
    post,
    path = "/v1/prompts/{tenant_id}/{project_id}",
    tag = "prompts",
    operation_id = "createPrompt",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreatePromptRequest,
    responses(
        (status = 200, description = "Create a prompt and its initial version", body = CreatedPrompt),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_prompt_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreatePromptRequest>,
) -> Result<Json<CreatedPrompt>, ApiError> {
    let prompts = prompt_registry(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let created = prompts.create_prompt(CreatePrompt {
        tenant_id,
        project_id,
        name: request.name,
        description: request.description,
        template: request.template,
        created_by: request.created_by,
        message: request.message,
    })?;
    Ok(Json(created))
}

#[utoipa::path(
    get,
    path = "/v1/prompts/{tenant_id}/{project_id}",
    tag = "prompts",
    operation_id = "listPrompts",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List prompts in a project", body = PromptListResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn list_prompts_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<PromptListResponse>, ApiError> {
    let prompts = prompt_registry(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let prompts = prompts.list_prompts(&tenant_id, &project_id)?;
    Ok(Json(PromptListResponse { prompts }))
}

#[utoipa::path(
    get,
    path = "/v1/prompts/{tenant_id}/{project_id}/{prompt_id}",
    tag = "prompts",
    operation_id = "getPrompt",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("prompt_id" = String, Path, description = "prompt_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get a prompt's metadata", body = Prompt),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_prompt_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, prompt_id)): Path<(String, String, String)>,
) -> Result<Json<Prompt>, ApiError> {
    let prompts = prompt_registry(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let prompt_id = PromptId::new(prompt_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let prompt = prompts.get_prompt(&tenant_id, &project_id, &prompt_id)?;
    Ok(Json(prompt))
}

#[utoipa::path(
    post,
    path = "/v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions",
    tag = "prompts",
    operation_id = "addPromptVersion",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("prompt_id" = String, Path, description = "prompt_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = AddPromptVersionRequest,
    responses(
        (status = 200, description = "Append an immutable prompt version", body = PromptVersion),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn add_prompt_version_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, prompt_id)): Path<(String, String, String)>,
    Json(request): Json<AddPromptVersionRequest>,
) -> Result<Json<PromptVersion>, ApiError> {
    let prompts = prompt_registry(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let prompt_id = PromptId::new(prompt_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let version = prompts.add_version(AddPromptVersion {
        tenant_id,
        project_id,
        prompt_id,
        template: request.template,
        created_by: request.created_by,
        message: request.message,
    })?;
    Ok(Json(version))
}

#[utoipa::path(
    get,
    path = "/v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions",
    tag = "prompts",
    operation_id = "listPromptVersions",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("prompt_id" = String, Path, description = "prompt_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List a prompt's versions oldest-first", body = PromptVersionListResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn list_prompt_versions_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, prompt_id)): Path<(String, String, String)>,
) -> Result<Json<PromptVersionListResponse>, ApiError> {
    let prompts = prompt_registry(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let prompt_id = PromptId::new(prompt_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let versions = prompts.list_versions(&tenant_id, &project_id, &prompt_id)?;
    Ok(Json(PromptVersionListResponse { versions }))
}

#[utoipa::path(
    get,
    path = "/v1/prompts/{tenant_id}/{project_id}/{prompt_id}/diff",
    tag = "prompts",
    operation_id = "diffPromptVersions",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("prompt_id" = String, Path, description = "prompt_id"),
        DiffPromptVersionsQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Line diff between two prompt versions", body = PromptVersionDiff),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn diff_prompt_versions_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, prompt_id)): Path<(String, String, String)>,
    Query(query): Query<DiffPromptVersionsQuery>,
) -> Result<Json<PromptVersionDiff>, ApiError> {
    let prompts = prompt_registry(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let prompt_id = PromptId::new(prompt_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let from_version_id = PromptVersionId::new(query.from)?;
    let to_version_id = PromptVersionId::new(query.to)?;
    let diff = prompts.diff_versions(
        &tenant_id,
        &project_id,
        &prompt_id,
        &from_version_id,
        &to_version_id,
    )?;
    Ok(Json(diff))
}

#[utoipa::path(
    post,
    path = "/v1/datasets/{tenant_id}/{project_id}",
    tag = "datasets",
    operation_id = "createDataset",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateDatasetRequest,
    responses(
        (status = 200, description = "Create a dataset", body = Dataset),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_dataset(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreateDatasetRequest>,
) -> Result<Json<Dataset>, ApiError> {
    let datasets = dataset_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let dataset = datasets
        .create_dataset(tenant_id, project_id, request.name)
        .await?;
    Ok(Json(dataset))
}

#[utoipa::path(
    post,
    path = "/v1/scenarios/{tenant_id}/{project_id}",
    tag = "scenarios",
    operation_id = "createScenario",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateScenarioRequest,
    responses(
        (status = 200, description = "Create a scenario", body = Scenario),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_scenario(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreateScenarioRequest>,
) -> Result<Json<Scenario>, ApiError> {
    let scenarios = scenario_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::ScenarioWrite,
    )
    .await?;
    let scope = TenantScope::new(tenant_id, project_id, EnvironmentId::new("default")?);
    let failure_mode = request.failure_mode.unwrap_or(FailureMode::Other);
    let mut source_trace_ids = request
        .source_trace_ids
        .into_iter()
        .map(TraceId::new)
        .collect::<Result<Vec<_>, _>>()?;
    source_trace_ids.sort();
    let exemplar_trace_id = match request.exemplar_trace_id {
        Some(id) => TraceId::new(id)?,
        None => source_trace_ids.first().cloned().ok_or_else(|| {
            ApiError::bad_request("scenario needs at least one source trace".to_string())
        })?,
    };
    let scenario = Scenario {
        scenario_id: uuid::Uuid::new_v4().to_string(),
        scope,
        title: request.title,
        failure_mode,
        recurrence_count: source_trace_ids.len(),
        source_trace_ids,
        exemplar_trace_id,
        perturbation_knobs: PerturbationKnobs::for_failure_mode(failure_mode),
        expected_outcome: request.expected_outcome,
        redaction_class: RedactionClass::Internal,
        created_at: Utc::now(),
    };
    let scenario = scenarios.put_scenario(scenario).await?;
    Ok(Json(scenario))
}

#[utoipa::path(
    get,
    path = "/v1/scenarios/{tenant_id}/{project_id}",
    tag = "scenarios",
    operation_id = "listScenarios",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ListScenariosQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List scenarios", body = ListScenariosResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn list_scenarios(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<ListScenariosQuery>,
) -> Result<Json<ListScenariosResponse>, ApiError> {
    let scenarios = scenario_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::ScenarioRead,
    )
    .await?;
    // Store returns scenarios ordered by (created_at, scenario_id); paginate with
    // an opaque cursor equal to the last scenario_id returned on the prior page.
    let limit = params.limit.unwrap_or(50).clamp(1, 200) as usize;
    let all = scenarios.list_scenarios(&tenant_id, &project_id).await?;
    let start = match params.cursor.as_deref() {
        Some(cursor) => all
            .iter()
            .position(|s| s.scenario_id == cursor)
            .map(|i| i + 1)
            .unwrap_or(all.len()),
        None => 0,
    };
    let page: Vec<Scenario> = all.iter().skip(start).take(limit).cloned().collect();
    let next_cursor = if start + page.len() < all.len() {
        page.last().map(|s| s.scenario_id.clone())
    } else {
        None
    };
    Ok(Json(ListScenariosResponse {
        scenarios: page,
        next_cursor,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/scenarios/{tenant_id}/{project_id}/{scenario_id}",
    tag = "scenarios",
    operation_id = "getScenario",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("scenario_id" = String, Path, description = "scenario_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Get a scenario", body = Scenario),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn get_scenario(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, scenario_id)): Path<(String, String, String)>,
) -> Result<Json<Scenario>, ApiError> {
    let scenarios = scenario_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::ScenarioRead,
    )
    .await?;
    let scenario = scenarios
        .get_scenario(&tenant_id, &project_id, &scenario_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("scenario {scenario_id} not found")))?;
    Ok(Json(scenario))
}

#[utoipa::path(
    post,
    path = "/v1/scenarios/{tenant_id}/{project_id}/mine",
    tag = "scenarios",
    operation_id = "mineScenarios",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = MineScenariosRequest,
    responses(
        (status = 200, description = "Mine scenario clusters from traces", body = MineScenariosResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn mine_scenarios(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<MineScenariosRequest>,
) -> Result<Json<MineScenariosResponse>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::ScenarioRead,
    )
    .await?;
    let mut summaries = Vec::with_capacity(request.trace_ids.len());
    for id in request.trace_ids {
        let trace_id = TraceId::new(id)?;
        let trace = state
            .traces
            .get_project_trace(tenant_id.clone(), project_id.clone(), trace_id)
            .await?;
        if let Some(summary) = TraceSummary::from_spans(&trace.spans) {
            summaries.push(summary);
        }
    }
    let clusters = cluster_failures(&summaries, request.jaccard_threshold.unwrap_or(0.6))
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    Ok(Json(MineScenariosResponse { clusters }))
}

#[utoipa::path(
    post,
    path = "/v1/datasets/{tenant_id}/{project_id}/{dataset_id}/cases/from-trace",
    tag = "datasets",
    operation_id = "promoteDatasetCaseFromTrace",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = PromoteTraceCaseRequest,
    responses(
        (status = 200, description = "Promote a trace span to a dataset case", body = beater_datasets :: DatasetCase),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn promote_dataset_case(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id)): Path<(String, String, String)>,
    Json(request): Json<PromoteTraceCaseRequest>,
) -> Result<Json<beater_datasets::DatasetCase>, ApiError> {
    let datasets = dataset_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let trace_id = TraceId::new(request.trace_id)?;
    let span_id = request.span_id.map(SpanId::new).transpose()?;
    let trace = state
        .traces
        .get_project_trace(tenant_id.clone(), project_id.clone(), trace_id)
        .await?;
    ensure_trace_not_empty(&trace)?;
    ensure_trace_project(&trace, &project_id)?;
    ensure_trace_auth_scope(&trace, &auth)?;
    if let Some(span_id) = &span_id {
        ensure_trace_has_span(&trace, span_id)?;
    }
    let case = promote_trace_span_to_case(
        tenant_id,
        project_id,
        dataset_id,
        &trace,
        span_id,
        request.reference,
    )?;
    let case = datasets.put_case(case).await?;
    Ok(Json(case))
}

#[utoipa::path(
    post,
    path = "/v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions",
    tag = "datasets",
    operation_id = "createDatasetVersion",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateDatasetVersionRequest,
    responses(
        (status = 200, description = "Create a dataset version snapshot", body = DatasetVersionSnapshot),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn create_dataset_version(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id)): Path<(String, String, String)>,
    Json(request): Json<CreateDatasetVersionRequest>,
) -> Result<Json<DatasetVersionSnapshot>, ApiError> {
    let datasets = dataset_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let case_ids = request
        .case_ids
        .map(|ids| {
            ids.into_iter()
                .map(DatasetCaseId::new)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let version = datasets
        .create_version(tenant_id, project_id, dataset_id, case_ids)
        .await?;
    Ok(Json(version))
}

#[utoipa::path(
    post,
    path = "/v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/deterministic",
    tag = "evals",
    operation_id = "runDeterministicEval",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("version_id" = String, Path, description = "version_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunDeterministicEvalRequest,
    responses(
        (status = 200, description = "Run a deterministic dataset evaluation", body = DatasetEvalReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn run_deterministic_dataset_eval(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id, version_id)): Path<(String, String, String, String)>,
    Json(request): Json<RunDeterministicEvalRequest>,
) -> Result<Json<DatasetEvalReport>, ApiError> {
    let datasets = dataset_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    let version_id = DatasetVersionId::new(version_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let snapshot = datasets
        .get_version(
            tenant_id.clone(),
            project_id.clone(),
            dataset_id.clone(),
            version_id,
        )
        .await?;
    let report = evaluate_dataset_version(
        &snapshot,
        DatasetEvalSpec {
            evaluator: EvaluatorSpec {
                id: request.evaluator_id,
                lane: beater_schema::EvaluatorLane::DeterministicWasi,
                kind: request.kind,
            },
            evaluator_version_id: EvaluatorVersionId::new(request.evaluator_version_id)?,
            agent_release_id: AgentReleaseId::new(request.agent_release_id)?,
            prompt_version_id: request
                .prompt_version_id
                .map(PromptVersionId::new)
                .transpose()?,
            code_hash: request.code_hash.map(Sha256Hash::new).transpose()?,
            wasm_hash: request.wasm_hash.map(Sha256Hash::new).transpose()?,
        },
    )?;
    let report = datasets.write_eval_report(report).await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/judge",
    tag = "evals",
    operation_id = "runJudgeEval",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("version_id" = String, Path, description = "version_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunJudgeDatasetEvalRequest,
    responses(
        (status = 200, description = "Run a judge dataset evaluation", body = DatasetEvalReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn run_judge_dataset_eval(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id, version_id)): Path<(String, String, String, String)>,
    Json(request): Json<RunJudgeDatasetEvalRequest>,
) -> Result<Json<DatasetEvalReport>, ApiError> {
    let datasets = dataset_store(&state)?;
    let judge_broker = judge_broker(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    let version_id = DatasetVersionId::new(version_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let snapshot = datasets
        .get_version(
            tenant_id.clone(),
            project_id.clone(),
            dataset_id.clone(),
            version_id,
        )
        .await?;
    let report = evaluate_dataset_version_with_judge(
        &snapshot,
        DatasetJudgeEvalSpec {
            eval: DatasetEvalSpec {
                evaluator: EvaluatorSpec {
                    id: request.evaluator_id,
                    lane: beater_schema::EvaluatorLane::JudgeBroker,
                    kind: request.kind,
                },
                evaluator_version_id: EvaluatorVersionId::new(request.evaluator_version_id)?,
                agent_release_id: AgentReleaseId::new(request.agent_release_id)?,
                prompt_version_id: request
                    .prompt_version_id
                    .map(PromptVersionId::new)
                    .transpose()?,
                code_hash: request.code_hash.map(Sha256Hash::new).transpose()?,
                wasm_hash: None,
            },
            provider_secret_id: request.provider_secret_id,
        },
        judge_broker.as_ref(),
    )
    .await
    .map_err(|err| ApiError::internal(err.to_string()))?;
    let report = datasets.write_eval_report(report).await?;
    record_usage_if_configured(&state, judge_usage_from_dataset_eval_report(&report)).await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}",
    tag = "calibrations",
    operation_id = "runCalibration",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("version_id" = String, Path, description = "version_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunCalibrationHttpRequest,
    responses(
        (status = 200, description = "Run a calibration over an eval report", body = CalibrationReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn run_calibration_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id, version_id)): Path<(String, String, String, String)>,
    Json(request): Json<RunCalibrationHttpRequest>,
) -> Result<Json<CalibrationReport>, ApiError> {
    let datasets = dataset_store(&state)?;
    let calibrations = calibration_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    let version_id = DatasetVersionId::new(version_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let snapshot = datasets
        .get_version(
            tenant_id.clone(),
            project_id.clone(),
            dataset_id.clone(),
            version_id.clone(),
        )
        .await?;
    let requested_evaluator = request
        .evaluator_version_id
        .map(EvaluatorVersionId::new)
        .transpose()?;
    let eval_report = if let Some(eval_report_id) = request.eval_report_id {
        let report = datasets
            .get_eval_report(tenant_id.clone(), project_id.clone(), eval_report_id)
            .await?;
        if let Some(evaluator_version_id) = &requested_evaluator
            && &report.evaluator_version_id != evaluator_version_id
        {
            return Err(ApiError::bad_request(
                "eval_report_id does not match requested evaluator_version_id".to_string(),
            ));
        }
        report
    } else {
        datasets
            .latest_eval_report(
                tenant_id.clone(),
                project_id.clone(),
                dataset_id,
                version_id,
                requested_evaluator,
            )
            .await?
            .ok_or_else(|| ApiError::not_found("dataset eval report not found".to_string()))?
    };
    let report = calibrate_eval_report(
        &snapshot,
        &eval_report,
        CalibrationPolicy {
            pass_threshold: request.pass_threshold.unwrap_or(0.5),
        },
    )
    .map_err(|err| ApiError::bad_request(err.to_string()))?;
    let report = calibrations.write_report(report).await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/deterministic",
    tag = "experiments",
    operation_id = "runDeterministicExperiment",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("version_id" = String, Path, description = "version_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunExperimentRequest,
    responses(
        (status = 200, description = "Run a deterministic experiment", body = ExperimentRunReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn run_deterministic_experiment_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id, version_id)): Path<(String, String, String, String)>,
    Json(request): Json<RunExperimentRequest>,
) -> Result<Json<ExperimentRunReport>, ApiError> {
    let datasets = dataset_store(&state)?;
    let experiments = experiment_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    let version_id = DatasetVersionId::new(version_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let snapshot = datasets
        .get_version(
            tenant_id.clone(),
            project_id.clone(),
            dataset_id.clone(),
            version_id,
        )
        .await?;
    let report = run_deterministic_experiment(
        &snapshot,
        ExperimentRunSpec {
            baseline_release_id: AgentReleaseId::new(request.baseline_release_id)?,
            candidate_release_id: AgentReleaseId::new(request.candidate_release_id)?,
            evaluator: EvaluatorSpec {
                id: request.evaluator_id,
                lane: beater_schema::EvaluatorLane::DeterministicWasi,
                kind: request.kind,
            },
            evaluator_version_id: EvaluatorVersionId::new(request.evaluator_version_id)?,
            gate_policy: request.gate_policy.unwrap_or_default(),
            baseline_outputs: parse_case_outputs(request.baseline_outputs)?,
            candidate_outputs: parse_case_outputs(request.candidate_outputs)?,
        },
    )?;
    let report = experiments.write_run(report).await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/judge",
    tag = "experiments",
    operation_id = "runJudgeExperiment",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("dataset_id" = String, Path, description = "dataset_id"),
        ("version_id" = String, Path, description = "version_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunJudgeExperimentRequest,
    responses(
        (status = 200, description = "Run a judge experiment", body = ExperimentRunReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn run_judge_experiment_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, dataset_id, version_id)): Path<(String, String, String, String)>,
    Json(request): Json<RunJudgeExperimentRequest>,
) -> Result<Json<ExperimentRunReport>, ApiError> {
    let datasets = dataset_store(&state)?;
    let experiments = experiment_store(&state)?;
    let judge_broker = judge_broker(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let dataset_id = DatasetId::new(dataset_id)?;
    let version_id = DatasetVersionId::new(version_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let snapshot = datasets
        .get_version(
            tenant_id.clone(),
            project_id.clone(),
            dataset_id.clone(),
            version_id,
        )
        .await?;
    let report = run_judge_experiment(
        &snapshot,
        JudgeExperimentRunSpec {
            experiment: ExperimentRunSpec {
                baseline_release_id: AgentReleaseId::new(request.baseline_release_id)?,
                candidate_release_id: AgentReleaseId::new(request.candidate_release_id)?,
                evaluator: EvaluatorSpec {
                    id: request.evaluator_id,
                    lane: beater_schema::EvaluatorLane::JudgeBroker,
                    kind: request.kind,
                },
                evaluator_version_id: EvaluatorVersionId::new(request.evaluator_version_id)?,
                gate_policy: request.gate_policy.unwrap_or_default(),
                baseline_outputs: parse_case_outputs(request.baseline_outputs)?,
                candidate_outputs: parse_case_outputs(request.candidate_outputs)?,
            },
            provider_secret_id: request.provider_secret_id,
        },
        judge_broker.as_ref(),
    )
    .await
    .map_err(|err| ApiError::internal(err.to_string()))?;
    let report = experiments.write_run(report).await?;
    record_usage_if_configured(&state, judge_usage_from_experiment_report(&report)).await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/gates/{tenant_id}/{project_id}",
    tag = "gates",
    operation_id = "createGate",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateGateRequest,
    responses(
        (status = 200, description = "Create a release gate", body = GateDefinition),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_gate_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreateGateRequest>,
) -> Result<Json<GateDefinition>, ApiError> {
    let gates = gate_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let gate = gates
        .put_gate(GateDefinition {
            tenant_id,
            project_id,
            gate_id: GateId::new(request.gate_id)?,
            name: request.name,
            dataset_id: request.dataset_id.map(DatasetId::new).transpose()?,
            evaluator_version_id: request
                .evaluator_version_id
                .map(EvaluatorVersionId::new)
                .transpose()?,
            inconclusive_policy: request.inconclusive_policy.unwrap_or_default(),
            created_at: Utc::now(),
        })
        .await?;
    Ok(Json(gate))
}

#[utoipa::path(
    post,
    path = "/v1/gates/{tenant_id}/{project_id}/{gate_id}/run",
    tag = "gates",
    operation_id = "runGate",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("gate_id" = String, Path, description = "gate_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = RunGateRequest,
    responses(
        (status = 200, description = "Run a gate against an experiment", body = GateRunReport),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn run_gate_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, gate_id)): Path<(String, String, String)>,
    Json(request): Json<RunGateRequest>,
) -> Result<Json<GateRunReport>, ApiError> {
    let gates = gate_store(&state)?;
    let experiments = experiment_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::EvalRun).await?;
    let report = run_gate(
        gates.as_ref(),
        experiments.as_ref(),
        tenant_id,
        project_id,
        GateId::new(gate_id)?,
        request
            .experiment_run_id
            .map(ExperimentRunId::new)
            .transpose()?,
    )
    .await?;
    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/v1/review-queues/{tenant_id}/{project_id}",
    tag = "reviews",
    operation_id = "createReviewQueue",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateReviewQueueHttpRequest,
    responses(
        (status = 200, description = "Create a human review queue", body = ReviewQueue),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn create_review_queue_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreateReviewQueueHttpRequest>,
) -> Result<Json<ReviewQueue>, ApiError> {
    let reviews = human_review_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let queue = reviews
        .create_queue(CreateReviewQueueStoreRequest {
            tenant_id,
            project_id,
            queue_id: request.queue_id.map(ReviewQueueId::new).transpose()?,
            name: request.name,
            annotation_schema: request.annotation_schema,
        })
        .await?;
    Ok(Json(queue))
}

#[utoipa::path(
    get,
    path = "/v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks",
    tag = "reviews",
    operation_id = "listReviewTasks",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("queue_id" = String, Path, description = "queue_id"),
        ListReviewTasksQuery,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List review tasks", body = Vec < ReviewTask >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn list_review_tasks_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, queue_id)): Path<(String, String, String)>,
    Query(query): Query<ListReviewTasksQuery>,
) -> Result<Json<Vec<ReviewTask>>, ApiError> {
    let reviews = human_review_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let tasks = reviews
        .list_tasks(
            tenant_id,
            project_id,
            ReviewQueueId::new(queue_id)?,
            query.state,
        )
        .await?;
    Ok(Json(tasks))
}

#[utoipa::path(
    post,
    path = "/v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace",
    tag = "reviews",
    operation_id = "enqueueReviewTaskFromTrace",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("queue_id" = String, Path, description = "queue_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = EnqueueReviewTaskFromTraceHttpRequest,
    responses(
        (status = 200, description = "Enqueue a review task from a trace", body = ReviewTask),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn enqueue_review_task_from_trace_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, queue_id)): Path<(String, String, String)>,
    Json(request): Json<EnqueueReviewTaskFromTraceHttpRequest>,
) -> Result<Json<ReviewTask>, ApiError> {
    let reviews = human_review_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let trace_id = TraceId::new(request.trace_id)?;
    let span_id = request.span_id.map(SpanId::new).transpose()?;
    let trace = state
        .traces
        .get_project_trace(tenant_id.clone(), project_id.clone(), trace_id.clone())
        .await?;
    ensure_trace_project(&trace, &project_id)?;
    ensure_trace_auth_scope(&trace, &auth)?;
    if let Some(span_id) = &span_id {
        ensure_trace_has_span(&trace, span_id)?;
    }
    let task = reviews
        .enqueue_task(EnqueueReviewTaskRequest {
            tenant_id,
            project_id,
            queue_id: ReviewQueueId::new(queue_id)?,
            task_id: request.task_id.map(ReviewTaskId::new).transpose()?,
            trace_id,
            span_id,
            dataset_id: request.dataset_id.map(DatasetId::new).transpose()?,
            dataset_case_id: request
                .dataset_case_id
                .map(DatasetCaseId::new)
                .transpose()?,
            priority: request.priority.unwrap_or_default(),
        })
        .await?;
    Ok(Json(task))
}

#[utoipa::path(
    post,
    path = "/v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations",
    tag = "reviews",
    operation_id = "submitReviewAnnotation",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("queue_id" = String, Path, description = "queue_id"),
        ("task_id" = String, Path, description = "task_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = SubmitReviewAnnotationHttpRequest,
    responses(
        (status = 200, description = "Submit a review annotation", body = ReviewAnnotation),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn submit_review_annotation_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, queue_id, task_id)): Path<(String, String, String, String)>,
    Json(request): Json<SubmitReviewAnnotationHttpRequest>,
) -> Result<Json<ReviewAnnotation>, ApiError> {
    let reviews = human_review_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let annotation = reviews
        .submit_annotation(SubmitAnnotationRequest {
            tenant_id,
            project_id,
            queue_id: ReviewQueueId::new(queue_id)?,
            task_id: ReviewTaskId::new(task_id)?,
            annotation_id: request.annotation_id.map(AnnotationId::new).transpose()?,
            reviewer_id: request.reviewer_id,
            verdict: request.verdict,
            payload: request.payload,
        })
        .await?;
    Ok(Json(annotation))
}

#[utoipa::path(
    post,
    path = "/v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote",
    tag = "reviews",
    operation_id = "promoteReviewAnnotation",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("queue_id" = String, Path, description = "queue_id"),
        ("task_id" = String, Path, description = "task_id"),
        ("annotation_id" = String, Path, description = "annotation_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = PromoteReviewAnnotationHttpRequest,
    responses(
        (status = 200, description = "Promote a review annotation to a dataset case", body = beater_datasets :: DatasetCase),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
async fn promote_review_annotation_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, queue_id, task_id, annotation_id)): Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
    Json(request): Json<PromoteReviewAnnotationHttpRequest>,
) -> Result<Json<beater_datasets::DatasetCase>, ApiError> {
    let reviews = human_review_store(&state)?;
    let datasets = dataset_store(&state)?;
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::DatasetWrite,
    )
    .await?;
    let queue_id = ReviewQueueId::new(queue_id)?;
    let task_id = ReviewTaskId::new(task_id)?;
    let annotation_id = AnnotationId::new(annotation_id)?;
    let task = reviews
        .get_task(
            tenant_id.clone(),
            project_id.clone(),
            queue_id.clone(),
            task_id.clone(),
        )
        .await?;
    let annotation = reviews
        .get_annotation(
            tenant_id.clone(),
            project_id.clone(),
            queue_id,
            task_id,
            annotation_id,
        )
        .await?;
    let trace = state
        .traces
        .get_project_trace(tenant_id.clone(), project_id.clone(), task.trace_id.clone())
        .await?;
    ensure_trace_project(&trace, &project_id)?;
    ensure_trace_auth_scope(&trace, &auth)?;
    let dataset_id = DatasetId::new(request.dataset_id)?;
    let case = promote_review_annotation_to_dataset_case(
        tenant_id,
        project_id,
        dataset_id,
        &trace,
        &task,
        &annotation,
        request.reference,
    )?;
    let case = datasets.put_case(case).await?;
    Ok(Json(case))
}

#[utoipa::path(
    post,
    path = "/v1/online/{tenant_id}/{project_id}/traces/{trace_id}/sampling",
    tag = "online",
    operation_id = "decideOnlineSampling",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = OnlineSamplingPolicy,
    responses(
        (status = 200, description = "Decide online sampling for a trace", body = SamplingDecision),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn decide_online_sampling(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, trace_id)): Path<(String, String, String)>,
    Json(policy): Json<OnlineSamplingPolicy>,
) -> Result<Json<SamplingDecision>, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let project_id = ProjectId::new(project_id)?;
    let trace_id = TraceId::new(trace_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        ApiScope::TraceRead,
    )
    .await?;
    let trace = state
        .traces
        .get_project_trace(tenant_id, project_id.clone(), trace_id)
        .await?;
    ensure_trace_project(&trace, &project_id)?;
    ensure_trace_auth_scope(&trace, &auth)?;
    Ok(Json(decide_trace_sampling(&trace, &policy)))
}

#[utoipa::path(
    post,
    path = "/v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook",
    tag = "alerts",
    operation_id = "evaluateAlert",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("trace_id" = String, Path, description = "trace_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = EvaluateAlertRequest,
    responses(
        (status = 200, description = "Evaluate an alert policy for a trace", body = AlertDecision),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
async fn evaluate_alert(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, trace_id)): Path<(String, String, String)>,
    Json(request): Json<EvaluateAlertRequest>,
) -> Result<Json<AlertDecision>, ApiError> {
    let path_tenant = TenantId::new(tenant_id)?;
    let path_project = ProjectId::new(project_id)?;
    let path_trace = TraceId::new(trace_id)?;
    let auth = authorize_project_route(
        &state,
        &headers,
        &path_tenant,
        &path_project,
        ApiScope::TraceRead,
    )
    .await?;
    if request.input.tenant_id.as_str() != path_tenant.as_str()
        || request.input.project_id.as_str() != path_project.as_str()
        || request.input.trace_id.as_str() != path_trace.as_str()
    {
        return Err(ApiError::bad_request(
            "alert input must match route tenant/project/trace".to_string(),
        ));
    }
    let trace = state
        .traces
        .get_project_trace(path_tenant, path_project.clone(), path_trace)
        .await?;
    ensure_trace_project(&trace, &path_project)?;
    ensure_trace_auth_scope(&trace, &auth)?;
    if trace.spans.is_empty() {
        return Err(ApiError::not_found("trace not found for alert".to_string()));
    }
    validate_webhook_endpoint_url(&request.policy.endpoint_url)
        .map_err(|error| ApiError::bad_request(error.to_string()))?;
    let decision = state.alerts.evaluate(&request.policy, request.input)?;
    Ok(Json(decision))
}

/// Returns the configured store, or a 501 `not_implemented` error naming it.
fn require<T: ?Sized>(value: &Option<Arc<T>>, what: &str) -> Result<Arc<T>, ApiError> {
    value
        .clone()
        .ok_or_else(|| ApiError::not_implemented(format!("{what} is not configured")))
}

fn dataset_store(state: &ApiState) -> Result<Arc<dyn DatasetStore>, ApiError> {
    require(&state.datasets, "dataset store")
}

fn prompt_registry(state: &ApiState) -> Result<Arc<dyn PromptRegistry>, ApiError> {
    require(&state.prompts, "prompt registry")
}

fn scenario_store(state: &ApiState) -> Result<Arc<dyn ScenarioStore>, ApiError> {
    require(&state.scenarios, "scenario store")
}

fn provider_secret_store(state: &ApiState) -> Result<Arc<dyn ProviderSecretStore>, ApiError> {
    require(&state.provider_secrets, "provider secret store")
}

fn judge_broker(state: &ApiState) -> Result<Arc<dyn JudgeBroker>, ApiError> {
    require(&state.judge_broker, "judge broker")
}

fn judge_ledger(state: &ApiState) -> Result<Arc<dyn JudgeLedgerStore>, ApiError> {
    require(&state.judge_ledger, "judge ledger")
}

fn experiment_store(state: &ApiState) -> Result<Arc<dyn ExperimentStore>, ApiError> {
    require(&state.experiments, "experiment store")
}

fn gate_store(state: &ApiState) -> Result<Arc<dyn GateStore>, ApiError> {
    require(&state.gates, "gate store")
}

fn human_review_store(state: &ApiState) -> Result<Arc<dyn HumanReviewStore>, ApiError> {
    require(&state.human_reviews, "human review store")
}

fn calibration_store(state: &ApiState) -> Result<Arc<dyn CalibrationStore>, ApiError> {
    require(&state.calibrations, "calibration store")
}

fn usage_ledger(state: &ApiState) -> Result<Arc<dyn UsageLedgerStore>, ApiError> {
    require(&state.usage, "usage ledger")
}

#[cfg(feature = "billing")]
fn billing_store(state: &ApiState) -> Result<Arc<dyn BillingStore>, ApiError> {
    require(&state.billing, "billing store")
}

/// Build the [`Billing`] service from the wired billing + usage stores.
#[cfg(feature = "billing")]
fn billing_service(state: &ApiState) -> Result<Billing, ApiError> {
    let billing = billing_store(state)?;
    let usage = usage_ledger(state)?;
    Ok(Billing::new(billing, usage))
}

#[cfg(feature = "billing")]
fn stripe_sync(state: &ApiState) -> Result<StripeSync, ApiError> {
    let billing = billing_store(state)?;
    let secret = state.stripe_webhook_secret.clone().ok_or_else(|| {
        ApiError::not_implemented("stripe webhook secret is not configured".to_string())
    })?;
    Ok(StripeSync::new(billing, secret))
}

fn audit_store(state: &ApiState) -> Result<Arc<dyn AuditStore>, ApiError> {
    require(&state.audit, "audit store")
}

async fn append_optional_audit_event(
    state: &ApiState,
    insert: AuditEventInsert,
) -> Result<(), ApiError> {
    if let Some(audit) = state.audit.clone() {
        audit.append_event(insert).await?;
    }
    Ok(())
}

async fn record_usage_if_configured(
    state: &ApiState,
    inserts: Vec<UsageRecordInsert>,
) -> Result<(), ApiError> {
    if let Some(usage) = state.usage.clone() {
        record_usage_batch(usage.as_ref(), inserts).await?;
    }
    Ok(())
}

fn parse_case_outputs(
    outputs: Vec<CaseOutputOverrideRequest>,
) -> Result<Vec<CaseOutputOverride>, ApiError> {
    outputs
        .into_iter()
        .map(|output| {
            Ok(CaseOutputOverride {
                case_id: DatasetCaseId::new(output.case_id)?,
                output: output.output,
                trace: output.trace,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct HealthResponse {
    ok: bool,
}

/// Error envelope returned by every fallible endpoint.
#[derive(Clone, Debug, Serialize, ToSchema)]
struct ErrorResponse {
    /// Human-readable error message.
    error: String,
    /// HTTP status code, duplicated in the body for convenience.
    status: u16,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct OtlpIngestOutcome {
    accepted_raw: usize,
    accepted_spans: usize,
    duplicate_raw: usize,
    duplicate_spans: usize,
    downstream_queued: bool,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct ArchiveQueryResponse {
    rows: Vec<ArchivedSpanRow>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateDatasetRequest {
    name: String,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateScenarioRequest {
    title: String,
    failure_mode: Option<FailureMode>,
    source_trace_ids: Vec<String>,
    exemplar_trace_id: Option<String>,
    expected_outcome: Option<String>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct MineScenariosRequest {
    trace_ids: Vec<String>,
    jaccard_threshold: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListScenariosQuery {
    limit: Option<u32>,
    cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct ListScenariosResponse {
    scenarios: Vec<Scenario>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
struct MineScenariosResponse {
    clusters: Vec<ScenarioCluster>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct PromoteTraceCaseRequest {
    trace_id: String,
    span_id: Option<String>,
    #[schema(value_type = Option<serde_json::Value>)]
    reference: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateDatasetVersionRequest {
    case_ids: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunDeterministicEvalRequest {
    evaluator_id: String,
    evaluator_version_id: String,
    agent_release_id: String,
    prompt_version_id: Option<String>,
    code_hash: Option<String>,
    wasm_hash: Option<String>,
    kind: EvaluatorKind,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunJudgeDatasetEvalRequest {
    evaluator_id: String,
    evaluator_version_id: String,
    agent_release_id: String,
    prompt_version_id: Option<String>,
    code_hash: Option<String>,
    kind: EvaluatorKind,
    provider_secret_id: ProviderSecretId,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunCalibrationHttpRequest {
    eval_report_id: Option<String>,
    evaluator_version_id: Option<String>,
    pass_threshold: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CaseOutputOverrideRequest {
    case_id: String,
    #[schema(value_type = serde_json::Value)]
    output: serde_json::Value,
    #[schema(value_type = Option<serde_json::Value>)]
    trace: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunExperimentRequest {
    baseline_release_id: String,
    candidate_release_id: String,
    evaluator_id: String,
    evaluator_version_id: String,
    kind: EvaluatorKind,
    gate_policy: Option<beater_eval::GatePolicy>,
    baseline_outputs: Vec<CaseOutputOverrideRequest>,
    candidate_outputs: Vec<CaseOutputOverrideRequest>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunJudgeExperimentRequest {
    baseline_release_id: String,
    candidate_release_id: String,
    evaluator_id: String,
    evaluator_version_id: String,
    kind: EvaluatorKind,
    gate_policy: Option<beater_eval::GatePolicy>,
    baseline_outputs: Vec<CaseOutputOverrideRequest>,
    candidate_outputs: Vec<CaseOutputOverrideRequest>,
    provider_secret_id: ProviderSecretId,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateGateRequest {
    gate_id: String,
    name: String,
    dataset_id: Option<String>,
    evaluator_version_id: Option<String>,
    inconclusive_policy: Option<InconclusivePolicy>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunGateRequest {
    experiment_run_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateReviewQueueHttpRequest {
    queue_id: Option<String>,
    name: String,
    #[schema(value_type = serde_json::Value)]
    annotation_schema: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListReviewTasksQuery {
    state: Option<ReviewTaskState>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct EnqueueReviewTaskFromTraceHttpRequest {
    task_id: Option<String>,
    trace_id: String,
    span_id: Option<String>,
    dataset_id: Option<String>,
    dataset_case_id: Option<String>,
    priority: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct SubmitReviewAnnotationHttpRequest {
    annotation_id: Option<String>,
    reviewer_id: String,
    verdict: ReviewVerdict,
    #[schema(value_type = serde_json::Value)]
    payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct PromoteReviewAnnotationHttpRequest {
    dataset_id: String,
    #[schema(value_type = Option<serde_json::Value>)]
    reference: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct EvaluateAlertRequest {
    policy: AlertPolicy,
    input: AlertInput,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateApiKeyHttpRequest {
    scopes: BTreeSet<ApiScope>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct CreateProviderSecretHttpRequest {
    provider: String,
    display_name: String,
    secret_value: String,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct RunJudgeEvalHttpRequest {
    evaluator: EvaluatorSpec,
    case: EvaluationCase,
    provider_secret_id: ProviderSecretId,
    /// Calibration-map / judge-instrument version folded into the judge cache
    /// key; bumping it on recalibration invalidates stale cached scores.
    #[serde(default)]
    cache_namespace: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct ApiKeyCreatedResponse {
    api_key_id: ApiKeyId,
    tenant_id: TenantId,
    project_id: ProjectId,
    environment_id: EnvironmentId,
    scopes: BTreeSet<ApiScope>,
    active: bool,
    #[schema(value_type = String, format = DateTime)]
    created_at: beater_core::Timestamp,
    secret: String,
}

impl ApiKeyCreatedResponse {
    fn from_created(created: CreatedApiKey) -> Self {
        Self {
            api_key_id: created.record.api_key_id,
            tenant_id: created.record.tenant_id,
            project_id: created.record.project_id,
            environment_id: created.record.environment_id,
            scopes: created.record.scopes,
            active: created.record.active,
            created_at: created.record.created_at,
            secret: created.secret,
        }
    }
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SearchQueryParams {
    q: Option<String>,
    project_id: Option<String>,
    environment_id: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
    kind: Option<String>,
    status: Option<String>,
    model: Option<String>,
    tool: Option<String>,
    limit: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListTracesQuery {
    project_id: Option<String>,
    environment_id: Option<String>,
    trace_id: Option<String>,
    kind: Option<String>,
    status: Option<String>,
    started_after: Option<String>,
    started_before: Option<String>,
    model: Option<String>,
    release: Option<String>,
    min_cost_micros: Option<i64>,
    max_cost_micros: Option<i64>,
    min_latency_ms: Option<i64>,
    max_latency_ms: Option<i64>,
    limit: Option<u32>,
    cursor: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct TraceReadQuery {
    unmask: Option<bool>,
    reason: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct IngestDurabilityQuery {
    durability: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DrainTraceWritesQuery {
    limit: Option<usize>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ReplayDeadLetterQuery {
    reset_attempts: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
struct SpanIoResponse {
    tenant_id: TenantId,
    trace_id: TraceId,
    span_id: SpanId,
    input: SpanIoValue,
    output: SpanIoValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SpanIoValue {
    Inline {
        #[schema(value_type = serde_json::Value)]
        value: serde_json::Value,
    },
    Artifact {
        artifact_ref: ArtifactRef,
    },
    Redacted {
        reason: String,
    },
    Missing,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ArchiveQueryParams {
    environment_id: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
    kind: Option<String>,
    status: Option<String>,
    limit: Option<usize>,
}

#[derive(Clone, Debug)]
struct AuthDecision {
    context: AuthContext,
    project_id: Option<ProjectId>,
    environment_id: Option<EnvironmentId>,
}

impl AuthDecision {
    fn anonymous() -> Self {
        Self {
            context: anonymous_auth_context(),
            project_id: None,
            environment_id: None,
        }
    }
}

async fn authorize_project_route(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    required_scope: ApiScope,
) -> Result<AuthDecision, ApiError> {
    if !state.auth_required() {
        return Ok(AuthDecision::anonymous());
    }
    let token_scope = oauth_tenant_scope_from_headers(state, headers).await?;
    let environment_id = match optional_environment_id_from_header(headers)? {
        Some(environment_id) => environment_id,
        None => match token_scope {
            Some(scope) => scope.environment_id,
            None => return Err(missing_strict_auth_header(ENVIRONMENT_ID_HEADER)),
        },
    };
    authorize(
        state,
        headers,
        tenant_id,
        project_id,
        &environment_id,
        required_scope,
    )
    .await
}

async fn authorize_tenant_route(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &TenantId,
    required_scope: ApiScope,
) -> Result<AuthDecision, ApiError> {
    if !state.auth_required() {
        return Ok(AuthDecision::anonymous());
    }
    let token_scope = oauth_tenant_scope_from_headers(state, headers).await?;
    let project_id = match optional_project_id_from_header(headers)? {
        Some(project_id) => project_id,
        None => match token_scope.as_ref() {
            Some(scope) => scope.project_id.clone(),
            None => return Err(missing_strict_auth_header(PROJECT_ID_HEADER)),
        },
    };
    let environment_id = match optional_environment_id_from_header(headers)? {
        Some(environment_id) => environment_id,
        None => match token_scope {
            Some(scope) => scope.environment_id,
            None => return Err(missing_strict_auth_header(ENVIRONMENT_ID_HEADER)),
        },
    };
    authorize(
        state,
        headers,
        tenant_id,
        &project_id,
        &environment_id,
        required_scope,
    )
    .await
}

/// Authorize an org-scoped billing route. Billing is org-scoped; auth is
/// project-scoped, so we map the organization id onto the tenant scope for the
/// credential check. A no-op when auth is disabled.
#[cfg(feature = "billing")]
async fn authorize_org_route(
    state: &ApiState,
    headers: &HeaderMap,
    org_id: &OrganizationId,
    required_scope: ApiScope,
) -> Result<AuthDecision, ApiError> {
    if !state.auth_required() {
        return Ok(AuthDecision::anonymous());
    }
    let tenant_id = TenantId::new(org_id.as_str())?;
    authorize_tenant_route(state, headers, &tenant_id, required_scope).await
}

async fn authorize_query_scope(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &TenantId,
    project_id: Option<&ProjectId>,
    environment_id: Option<&EnvironmentId>,
    required_scope: ApiScope,
) -> Result<AuthDecision, ApiError> {
    if !state.auth_required() {
        return Ok(AuthDecision::anonymous());
    }
    let token_scope = oauth_tenant_scope_from_headers(state, headers).await?;
    let token_project_id;
    let project_id = match project_id {
        Some(project_id) => project_id,
        None => match token_scope.as_ref() {
            Some(scope) => {
                token_project_id = scope.project_id.clone();
                &token_project_id
            }
            None => {
                return Err(ApiError::bad_request(
                    "strict auth requires project_id query parameter".to_string(),
                ));
            }
        },
    };
    let token_environment_id;
    let environment_id = match environment_id {
        Some(environment_id) => environment_id,
        None => match token_scope.as_ref() {
            Some(scope) => {
                token_environment_id = scope.environment_id.clone();
                &token_environment_id
            }
            None => {
                return Err(ApiError::bad_request(
                    "strict auth requires environment_id query parameter".to_string(),
                ));
            }
        },
    };
    authorize(
        state,
        headers,
        tenant_id,
        project_id,
        environment_id,
        required_scope,
    )
    .await
}

async fn oauth_tenant_scope_from_headers(
    state: &ApiState,
    headers: &HeaderMap,
) -> Result<Option<TenantScope>, ApiError> {
    let Some(secret) = bearer_secret(headers)? else {
        return Ok(None);
    };
    if !secret.starts_with("bao_") {
        return Ok(None);
    }
    let Some(oauth) = state.oauth.as_ref() else {
        return Ok(None);
    };
    let claims = oauth
        .validate_access_token(secret, Utc::now())
        .await
        .map_err(|_| ApiError::unauthorized("invalid or expired access token".to_string()))?;
    if state
        .oauth_resource()
        .is_none_or(|resource| claims.resource != resource)
    {
        return Err(ApiError::unauthorized(
            "access token audience does not match this resource".to_string(),
        ));
    }
    Ok(Some(claims.tenant_scope))
}

fn bearer_secret(headers: &HeaderMap) -> Result<Option<&str>, ApiError> {
    let Some(value) = headers
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Ok(None);
    };
    let Some(secret) = value.strip_prefix("Bearer ") else {
        return Err(ApiError::unauthorized(
            "authorization header must use bearer scheme".to_string(),
        ));
    };
    Ok(Some(secret))
}

async fn authorize(
    state: &ApiState,
    headers: &HeaderMap,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    environment_id: &EnvironmentId,
    required_scope: ApiScope,
) -> Result<AuthDecision, ApiError> {
    if !state.auth_required() {
        return Ok(AuthDecision::anonymous());
    }
    let secret = presented_api_key(headers)?;
    // OAuth access tokens (`bao_...`) are accepted when an OAuth store is wired.
    // This is the bridge that lets MCP/HTTP callers authenticate with an OAuth
    // token instead of a raw API key.
    if secret.starts_with("bao_")
        && let Some(oauth) = state.oauth.as_ref()
    {
        return authorize_oauth(
            oauth.as_ref(),
            secret,
            state.oauth_resource(),
            tenant_id,
            project_id,
            environment_id,
            required_scope,
        )
        .await;
    }
    let api_keys = state
        .api_keys
        .as_ref()
        .ok_or_else(|| ApiError::internal("api key store is not configured".to_string()))?;
    let api_key_id = api_key_id_from_secret(secret).map_err(auth_failure)?;
    let record = api_keys
        .get_key(api_key_id.clone())
        .await?
        .ok_or_else(|| ApiError::unauthorized("api key not found".to_string()))?;
    verify_api_key(
        &record,
        secret,
        tenant_id,
        project_id,
        environment_id,
        required_scope,
    )
    .map_err(auth_failure)?;
    api_keys
        .touch_last_used(record.api_key_id.clone(), Utc::now())
        .await?;
    Ok(AuthDecision {
        context: AuthContext {
            api_key_id: Some(record.api_key_id),
            scopes: record
                .scopes
                .iter()
                .map(|scope| scope.as_str().to_string())
                .collect(),
        },
        project_id: Some(record.project_id),
        environment_id: Some(record.environment_id),
    })
}

/// Authorize via an OAuth 2.1 access token (`bao_...`). Validates the token,
/// checks it is bound to the requested tenant/project/environment, and maps the
/// OAuth scopes (which use the same names as [`ApiScope::as_str`]) onto the
/// required scope. `admin` satisfies any scope.
async fn authorize_oauth(
    oauth: &dyn OAuthStore,
    secret: &str,
    expected_resource: Option<&str>,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    environment_id: &EnvironmentId,
    required_scope: ApiScope,
) -> Result<AuthDecision, ApiError> {
    let claims = oauth
        .validate_access_token(secret, Utc::now())
        .await
        .map_err(|_| ApiError::unauthorized("invalid or expired access token".to_string()))?;
    if expected_resource.is_none_or(|resource| claims.resource != resource) {
        return Err(ApiError::unauthorized(
            "access token audience does not match this resource".to_string(),
        ));
    }
    if &claims.tenant_scope.tenant_id != tenant_id
        || &claims.tenant_scope.project_id != project_id
        || &claims.tenant_scope.environment_id != environment_id
    {
        return Err(ApiError::forbidden(
            "access token is not scoped to this tenant/project/environment".to_string(),
        ));
    }
    if !claims.scope.contains("mcp:invoke") {
        return Err(ApiError::forbidden(
            "access token is missing scope mcp:invoke".to_string(),
        ));
    }
    let is_admin = claims.scope.contains(ApiScope::Admin.as_str());
    let has_scope = claims.scope.contains(required_scope.as_str()) || is_admin;
    if !has_scope {
        return Err(ApiError::forbidden(format!(
            "access token is missing scope {}",
            required_scope.as_str()
        )));
    }
    Ok(AuthDecision {
        context: AuthContext {
            api_key_id: None,
            scopes: claims.scope,
        },
        project_id: Some(claims.tenant_scope.project_id),
        environment_id: Some(claims.tenant_scope.environment_id),
    })
}

fn presented_api_key(headers: &HeaderMap) -> Result<&str, ApiError> {
    if let Some(value) = headers
        .get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        let Some(secret) = value.strip_prefix("Bearer ") else {
            return Err(ApiError::unauthorized(
                "authorization header must use bearer scheme".to_string(),
            ));
        };
        return Ok(secret);
    }
    headers
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing api key".to_string()))
}

fn optional_environment_id_from_header(
    headers: &HeaderMap,
) -> Result<Option<EnvironmentId>, ApiError> {
    headers
        .get(ENVIRONMENT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| EnvironmentId::new(value.to_string()).map_err(ApiError::from))
        .transpose()
}

fn optional_project_id_from_header(headers: &HeaderMap) -> Result<Option<ProjectId>, ApiError> {
    headers
        .get(PROJECT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| ProjectId::new(value.to_string()).map_err(ApiError::from))
        .transpose()
}

fn missing_strict_auth_header(header: &str) -> ApiError {
    ApiError::bad_request(format!("strict auth requires {header} header"))
}

fn ensure_trace_project(trace: &TraceView, project_id: &ProjectId) -> Result<(), ApiError> {
    if trace
        .spans
        .iter()
        .any(|span| span.project_id.as_str() != project_id.as_str())
    {
        return Err(ApiError::forbidden(
            "trace contains spans outside requested project".to_string(),
        ));
    }
    Ok(())
}

async fn ensure_environment_exists(
    state: &ApiState,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    environment_id: &EnvironmentId,
) -> Result<(), ApiError> {
    let environment = state
        .metadata
        .get_environment(
            tenant_id.clone(),
            project_id.clone(),
            environment_id.clone(),
        )
        .await?;
    if environment.is_some() {
        Ok(())
    } else {
        Err(ApiError::not_found(format!(
            "environment {}/{}/{} not found",
            tenant_id.as_str(),
            project_id.as_str(),
            environment_id.as_str()
        )))
    }
}

fn ensure_trace_has_span(trace: &TraceView, span_id: &SpanId) -> Result<(), ApiError> {
    if trace.spans.iter().any(|span| &span.span_id == span_id) {
        return Ok(());
    }
    Err(ApiError::not_found(format!(
        "span {} not found in trace {}",
        span_id.as_str(),
        trace.trace_id.as_str()
    )))
}

fn ensure_trace_not_empty(trace: &TraceView) -> Result<(), ApiError> {
    if !trace.spans.is_empty() {
        return Ok(());
    }
    Err(ApiError::not_found(format!(
        "trace {} not found",
        trace.trace_id.as_str()
    )))
}

fn ensure_trace_auth_scope(trace: &TraceView, auth: &AuthDecision) -> Result<(), ApiError> {
    if let Some(project_id) = &auth.project_id {
        ensure_trace_project(trace, project_id)?;
    }
    if let Some(environment_id) = &auth.environment_id
        && trace
            .spans
            .iter()
            .any(|span| span.environment_id.as_str() != environment_id.as_str())
    {
        return Err(ApiError::forbidden(
            "trace contains spans outside authenticated environment".to_string(),
        ));
    }
    Ok(())
}

async fn load_trace_for_auth_scope(
    state: &ApiState,
    tenant_id: TenantId,
    trace_id: TraceId,
    auth: &AuthDecision,
) -> Result<TraceView, ApiError> {
    if let Some(project_id) = &auth.project_id {
        return Ok(state
            .traces
            .get_project_trace(tenant_id, project_id.clone(), trace_id)
            .await?);
    }
    Ok(state.traces.get_trace(tenant_id, trace_id).await?)
}

async fn load_span_for_route(
    state: ApiState,
    headers: HeaderMap,
    tenant_id: String,
    trace_id: String,
    span_id: String,
    params: TraceReadQuery,
) -> Result<CanonicalSpan, ApiError> {
    let tenant_id = TenantId::new(tenant_id)?;
    let trace_id = TraceId::new(trace_id)?;
    let span_id = SpanId::new(span_id)?;
    let auth = authorize_tenant_route(&state, &headers, &tenant_id, ApiScope::TraceRead).await?;
    let trace =
        load_trace_for_auth_scope(&state, tenant_id.clone(), trace_id.clone(), &auth).await?;
    ensure_trace_auth_scope(&trace, &auth)?;
    let trace = if params.unmask.unwrap_or(false) {
        authorize_and_audit_trace_unmask(&state, &headers, trace, auth, params.reason).await?
    } else {
        redact_trace_view(trace)
    };
    trace
        .spans
        .into_iter()
        .find(|span| span.span_id == span_id)
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "span {} not found in trace {}",
                span_id.as_str(),
                trace_id.as_str()
            ))
        })
}

fn span_io_value(
    span: &CanonicalSpan,
    inline_key: &str,
    artifact_ref: Option<&ArtifactRef>,
) -> SpanIoValue {
    if let Some(artifact_ref) = artifact_ref {
        if is_sensitive_redaction(&artifact_ref.redaction_class) {
            return SpanIoValue::Redacted {
                reason: format!(
                    "{} payload is {:?}",
                    inline_key, artifact_ref.redaction_class
                ),
            };
        }
        return SpanIoValue::Artifact {
            artifact_ref: artifact_ref.clone(),
        };
    }
    if let Some(value) = span.attributes.get(inline_key) {
        return SpanIoValue::Inline {
            value: value.clone(),
        };
    }
    SpanIoValue::Missing
}

async fn authorize_and_audit_trace_unmask(
    state: &ApiState,
    headers: &HeaderMap,
    trace: TraceView,
    trace_auth: AuthDecision,
    reason: Option<String>,
) -> Result<TraceView, ApiError> {
    let audit = audit_store(state)?;
    let project_id = trace_project_id(&trace)?;
    let environment_id = trace_auth
        .environment_id
        .clone()
        .or_else(|| trace.spans.first().map(|span| span.environment_id.clone()));
    let sensitive_refs = count_sensitive_refs(&trace);
    let pii_auth =
        authorize_tenant_route(state, headers, &trace.tenant_id, ApiScope::PiiUnmask).await;
    match pii_auth {
        Ok(pii_auth) => {
            ensure_trace_auth_scope(&trace, &pii_auth)?;
            audit
                .append_event(pii_unmask_event(PiiUnmaskAuditInput {
                    tenant_id: trace.tenant_id.clone(),
                    project_id,
                    environment_id,
                    actor_api_key_id: pii_auth.context.api_key_id.clone(),
                    trace_id: trace.trace_id.clone(),
                    outcome: AuditOutcome::Allowed,
                    reason,
                    attributes: serde_json::json!({
                        "sensitive_ref_count": sensitive_refs,
                        "unmasked": true
                    }),
                }))
                .await?;
            Ok(trace)
        }
        Err(error) => {
            let error_message = error.message.clone();
            audit
                .append_event(pii_unmask_event(PiiUnmaskAuditInput {
                    tenant_id: trace.tenant_id.clone(),
                    project_id,
                    environment_id,
                    actor_api_key_id: trace_auth.context.api_key_id.clone(),
                    trace_id: trace.trace_id.clone(),
                    outcome: AuditOutcome::Denied,
                    reason,
                    attributes: serde_json::json!({
                        "sensitive_ref_count": sensitive_refs,
                        "error": error_message
                    }),
                }))
                .await?;
            Err(error)
        }
    }
}

fn trace_project_id(trace: &TraceView) -> Result<ProjectId, ApiError> {
    trace
        .spans
        .first()
        .map(|span| span.project_id.clone())
        .ok_or_else(|| {
            ApiError::not_found(format!("trace {} has no spans", trace.trace_id.as_str()))
        })
}

fn redact_trace_view(mut trace: TraceView) -> TraceView {
    for span in &mut trace.spans {
        let span_sensitive = is_sensitive_redaction(&span.raw_ref.redaction_class);
        span.raw_ref = redact_artifact_ref(&span.raw_ref);
        span.input_ref = span.input_ref.as_ref().map(redact_artifact_ref);
        span.output_ref = span.output_ref.as_ref().map(redact_artifact_ref);
        if span_sensitive {
            redact_sensitive_span_attributes(span);
        }
    }
    trace
}

fn redact_sensitive_span_attributes(span: &mut CanonicalSpan) {
    for (key, value) in &mut span.attributes {
        if sensitive_read_safe_attribute(key) {
            continue;
        }
        *value = serde_json::json!("[redacted]");
    }
    redact_json_values(&mut span.unmapped_attrs);
}

fn sensitive_read_safe_attribute(key: &str) -> bool {
    matches!(
        key,
        "agent.release_id" | "beater.release_id" | "deployment.release_id" | "release_id"
    )
}

fn redact_json_values(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for child in map.values_mut() {
                redact_json_values(child);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                redact_json_values(child);
            }
        }
        serde_json::Value::Null => {}
        scalar => {
            *scalar = serde_json::json!("[redacted]");
        }
    }
}

fn redact_artifact_ref(artifact_ref: &ArtifactRef) -> ArtifactRef {
    if !is_sensitive_redaction(&artifact_ref.redaction_class) {
        return artifact_ref.clone();
    }
    ArtifactRef {
        artifact_id: match ArtifactId::new("redacted") {
            Ok(id) => id,
            Err(_) => artifact_ref.artifact_id.clone(),
        },
        uri: "artifact://redacted".to_string(),
        sha256: match Sha256Hash::new("redacted") {
            Ok(hash) => hash,
            Err(_) => artifact_ref.sha256.clone(),
        },
        size_bytes: 0,
        mime_type: "application/x.beater-redacted".to_string(),
        redaction_class: artifact_ref.redaction_class.clone(),
    }
}

fn count_sensitive_refs(trace: &TraceView) -> usize {
    trace
        .spans
        .iter()
        .map(|span| {
            let mut count = usize::from(is_sensitive_redaction(&span.raw_ref.redaction_class));
            if span
                .input_ref
                .as_ref()
                .is_some_and(|artifact_ref| is_sensitive_redaction(&artifact_ref.redaction_class))
            {
                count += 1;
            }
            if span
                .output_ref
                .as_ref()
                .is_some_and(|artifact_ref| is_sensitive_redaction(&artifact_ref.redaction_class))
            {
                count += 1;
            }
            count
        })
        .sum()
}

fn is_sensitive_redaction(redaction_class: &RedactionClass) -> bool {
    matches!(
        redaction_class,
        RedactionClass::Sensitive | RedactionClass::Secret
    )
}

fn auth_failure(error: SecurityError) -> ApiError {
    match error {
        SecurityError::MalformedApiKey | SecurityError::ApiKeyVerificationFailed => {
            ApiError::unauthorized(error.to_string())
        }
        SecurityError::InactiveApiKey
        | SecurityError::MissingScope(_)
        | SecurityError::ScopeMismatch => ApiError::forbidden(error.to_string()),
        SecurityError::MalformedSignature
        | SecurityError::WebhookReplayWindow
        | SecurityError::WebhookSignatureFailed => ApiError::bad_request(error.to_string()),
        SecurityError::Other(error) => ApiError::internal(error.to_string()),
    }
}

fn judge_failure(error: JudgeBrokerError) -> ApiError {
    match error {
        JudgeBrokerError::RequiresJudgeBrokerLane(_)
        | JudgeBrokerError::RequiresLlmJudge(_)
        | JudgeBrokerError::JudgeBudgetExceeded { .. }
        | JudgeBrokerError::ProviderExceededPreflightCost { .. } => {
            ApiError::bad_request(error.to_string())
        }
        JudgeBrokerError::ProviderSecretNotFound(_) => ApiError::not_found(error.to_string()),
        JudgeBrokerError::Provider(_) | JudgeBrokerError::Store(_) => {
            ApiError::internal(error.to_string())
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
    headers: Vec<(HeaderName, HeaderValue)>,
}

impl ApiError {
    fn with_status(status: StatusCode, message: String) -> Self {
        Self {
            status,
            message,
            headers: Vec::new(),
        }
    }

    fn with_header(mut self, name: HeaderName, value: impl ToString) -> Self {
        if let Ok(value) = HeaderValue::from_str(&value.to_string()) {
            self.headers.push((name, value));
        }
        self
    }

    fn bad_request(message: String) -> Self {
        Self::with_status(StatusCode::BAD_REQUEST, message)
    }

    fn internal(message: String) -> Self {
        Self::with_status(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    fn unauthorized(message: String) -> Self {
        Self::with_status(StatusCode::UNAUTHORIZED, message)
    }

    fn forbidden(message: String) -> Self {
        Self::with_status(StatusCode::FORBIDDEN, message)
    }

    fn not_found(message: String) -> Self {
        Self::with_status(StatusCode::NOT_FOUND, message)
    }

    fn not_implemented(message: String) -> Self {
        Self::with_status(StatusCode::NOT_IMPLEMENTED, message)
    }
}

fn parse_span_kind(value: String) -> Result<AgentSpanKind, ApiError> {
    AgentSpanKind::parse(&value)
        .ok_or_else(|| ApiError::bad_request(format!("unsupported span kind: {value}")))
}

fn parse_optional_timestamp(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<beater_core::Timestamp>, ApiError> {
    value
        .map(|value| {
            chrono::DateTime::parse_from_rfc3339(&value)
                .map(|timestamp| timestamp.with_timezone(&Utc))
                .map_err(|err| {
                    ApiError::bad_request(format!("{field_name} must be RFC3339: {err}"))
                })
        })
        .transpose()
}

#[cfg(feature = "billing")]
fn parse_required_timestamp(
    value: &str,
    field_name: &str,
) -> Result<beater_core::Timestamp, ApiError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|err| ApiError::bad_request(format!("{field_name} must be RFC3339: {err}")))
}

fn parse_span_status(value: String) -> Result<SpanStatus, ApiError> {
    SpanStatus::parse(&value)
        .ok_or_else(|| ApiError::bad_request(format!("unsupported span status: {value}")))
}

fn ingest_buffered(params: &IngestDurabilityQuery) -> Result<bool, ApiError> {
    match params.durability.as_deref() {
        None | Some("direct") | Some("sync") => Ok(false),
        Some("buffered") | Some("durable") => Ok(true),
        Some(value) => Err(ApiError::bad_request(format!(
            "unsupported ingest durability: {value}"
        ))),
    }
}

fn collector_otlp_scope(
    headers: &HeaderMap,
    hints: OtlpScopeHints,
) -> Result<TenantScope, ApiError> {
    let tenant_id = header_string(headers, TENANT_ID_HEADER)
        .or(hints.tenant_id)
        .ok_or_else(|| {
            ApiError::bad_request(format!(
                "collector OTLP JSON requires {TENANT_ID_HEADER} header or beater.tenant_id resource attribute"
            ))
        })?;
    let project_id = header_string(headers, PROJECT_ID_HEADER)
        .or(hints.project_id)
        .ok_or_else(|| {
            ApiError::bad_request(format!(
                "collector OTLP JSON requires {PROJECT_ID_HEADER} header or beater.project_id resource attribute"
            ))
        })?;
    let environment_id = header_string(headers, ENVIRONMENT_ID_HEADER)
        .or(hints.environment_id)
        .ok_or_else(|| {
            ApiError::bad_request(format!(
                "collector OTLP JSON requires {ENVIRONMENT_ID_HEADER} header, beater.environment_id resource attribute, or deployment.environment.name resource attribute"
            ))
        })?;

    Ok(TenantScope::new(
        TenantId::new(tenant_id)?,
        ProjectId::new(project_id)?,
        EnvironmentId::new(environment_id)?,
    ))
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
}

fn invalid_otlp_export(error: impl std::fmt::Display) -> ApiError {
    ApiError::bad_request(format!("invalid OTLP trace export: {error}"))
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        Self::internal(error.to_string())
    }
}

impl From<beater_core::IdError> for ApiError {
    fn from(error: beater_core::IdError) -> Self {
        Self::bad_request(error.to_string())
    }
}

impl From<IngestError> for ApiError {
    fn from(error: IngestError) -> Self {
        match error {
            IngestError::QuotaExceeded {
                limit, reset_at, ..
            } => {
                let retry_after = reset_at
                    .signed_duration_since(Utc::now())
                    .num_seconds()
                    .max(0);
                Self::with_status(StatusCode::TOO_MANY_REQUESTS, error.to_string())
                    .with_header(RETRY_AFTER, retry_after)
                    .with_header(HeaderName::from_static("x-ratelimit-limit"), limit)
                    .with_header(HeaderName::from_static("x-ratelimit-remaining"), 0)
                    .with_header(
                        HeaderName::from_static("x-ratelimit-reset"),
                        reset_at.timestamp(),
                    )
            }
            IngestError::Backpressure { .. } => {
                Self::with_status(StatusCode::TOO_MANY_REQUESTS, error.to_string())
            }
            IngestError::TooManyAttributes { .. } | IngestError::PayloadTooLarge { .. } => {
                Self::with_status(StatusCode::PAYLOAD_TOO_LARGE, error.to_string())
            }
            IngestError::NotFound(_) => Self::with_status(StatusCode::NOT_FOUND, error.to_string()),
            IngestError::Import(_) => Self::bad_request(error.to_string()),
            IngestError::Store(error) => error.into(),
            IngestError::Other(error) => Self::internal(error.to_string()),
        }
    }
}

impl From<PromptRegistryError> for ApiError {
    fn from(error: PromptRegistryError) -> Self {
        match error {
            PromptRegistryError::PromptNotFound { .. }
            | PromptRegistryError::VersionNotFound { .. } => Self::not_found(error.to_string()),
            PromptRegistryError::VersionPromptMismatch { .. } => {
                Self::bad_request(error.to_string())
            }
            PromptRegistryError::LockPoisoned | PromptRegistryError::Backend(_) => {
                Self::internal(error.to_string())
            }
        }
    }
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::NotFound(_) => Self::with_status(StatusCode::NOT_FOUND, error.to_string()),
            StoreError::Conflict(_) => Self::with_status(StatusCode::CONFLICT, error.to_string()),
            StoreError::Backpressure(_) => {
                Self::with_status(StatusCode::SERVICE_UNAVAILABLE, error.to_string())
            }
            StoreError::LimitExceeded(_) => {
                Self::with_status(StatusCode::PAYLOAD_TOO_LARGE, error.to_string())
            }
            StoreError::Integrity(_) | StoreError::Backend(_) => Self::internal(error.to_string()),
        }
    }
}

#[cfg(feature = "billing")]
impl From<BillingError> for ApiError {
    fn from(error: BillingError) -> Self {
        match error {
            BillingError::NotFound(_) => {
                Self::with_status(StatusCode::NOT_FOUND, error.to_string())
            }
            BillingError::Conflict(_) | BillingError::ConcurrentModification(_) => {
                Self::with_status(StatusCode::CONFLICT, error.to_string())
            }
            BillingError::InvalidPlan(_)
            | BillingError::InvalidPeriod(_)
            | BillingError::Money(_)
            | BillingError::Overflow => Self::bad_request(error.to_string()),
            BillingError::Store(store) => store.into(),
            BillingError::Backend(_) => Self::internal(error.to_string()),
        }
    }
}

#[cfg(feature = "billing")]
impl From<StripeError> for ApiError {
    fn from(error: StripeError) -> Self {
        match error {
            StripeError::Signature(_) | StripeError::Malformed(_) => {
                Self::bad_request(error.to_string())
            }
            StripeError::Store(store) => store.into(),
            StripeError::Billing(message) => Self::bad_request(message),
            StripeError::Transport(_) => {
                Self::with_status(StatusCode::BAD_GATEWAY, error.to_string())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "error": self.message,
            "status": self.status.as_u16()
        }));
        let mut response = (self.status, body).into_response();
        for (name, value) in self.headers {
            response.headers_mut().insert(name, value);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use beater_bus::InMemoryBus;
    use beater_core::sha256_hex;
    use beater_core::{EnvironmentId, IdempotencyKey, ProjectId, SpanId, TenantScope, TraceId};
    use beater_ingest::IngestPolicy;
    use beater_otlp::encode_export_trace_request;
    use beater_schema::{AgentSpanKind, RedactionClass, SourceDialect, SpanStatus, TraceView};
    use beater_search::{SearchResponse, TantivySearchIndex};
    use beater_store::ArtifactStore;
    use beater_store_obj::FsArtifactStore;
    use beater_store_sql::SqliteTraceStore;
    use http::{Request, StatusCode};
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use opentelemetry_proto::tonic::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, span, status,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    use tower::ServiceExt;

    #[tokio::test]
    async fn oauth_access_token_authorizes_only_for_its_tenant_and_scope() {
        use beater_core::{OAuthClientId, TokenFamilyId, UserId};
        use beater_oauth::{OAuthStore, SqliteOAuthStore};
        use std::collections::BTreeSet;

        let store = SqliteOAuthStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let now = Utc::now();
        let tenant = TenantId::new("acme").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj").unwrap_or_else(|err| panic!("{err}"));
        let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
        let resource = "https://api.example.com";
        let issued = store
            .issue_token_pair(
                OAuthClientId::new("client").unwrap_or_else(|err| panic!("{err}")),
                UserId::new("user").unwrap_or_else(|err| panic!("{err}")),
                BTreeSet::from(["mcp:invoke".to_string(), "trace:read".to_string()]),
                resource.to_string(),
                TenantScope::new(tenant.clone(), project.clone(), environment.clone()),
                TokenFamilyId::new("fam").unwrap_or_else(|err| panic!("{err}")),
                true,
                now,
            )
            .await
            .unwrap_or_else(|err| panic!("{err:?}"));

        // Correct tenant + scope -> authorized; identity is OAuth (no api_key_id).
        let decision = authorize_oauth(
            &store,
            &issued.access_token,
            Some(resource),
            &tenant,
            &project,
            &environment,
            ApiScope::TraceRead,
        )
        .await
        .unwrap_or_else(|err| panic!("expected authorized: {err:?}"));
        assert!(decision.context.api_key_id.is_none());
        assert!(decision.context.scopes.contains("trace:read"));
        assert!(decision.context.scopes.contains("mcp:invoke"));

        let no_mcp = store
            .issue_token_pair(
                OAuthClientId::new("client").unwrap_or_else(|err| panic!("{err}")),
                UserId::new("user").unwrap_or_else(|err| panic!("{err}")),
                BTreeSet::from(["trace:read".to_string()]),
                resource.to_string(),
                TenantScope::new(tenant.clone(), project.clone(), environment.clone()),
                TokenFamilyId::new("fam-no-mcp").unwrap_or_else(|err| panic!("{err}")),
                true,
                now,
            )
            .await
            .unwrap_or_else(|err| panic!("{err:?}"));
        assert!(
            authorize_oauth(
                &store,
                &no_mcp.access_token,
                Some(resource),
                &tenant,
                &project,
                &environment,
                ApiScope::TraceRead,
            )
            .await
            .is_err(),
            "OAuth tokens must carry mcp:invoke before operation scopes matter"
        );

        let admin_without_mcp = store
            .issue_token_pair(
                OAuthClientId::new("client").unwrap_or_else(|err| panic!("{err}")),
                UserId::new("user").unwrap_or_else(|err| panic!("{err}")),
                BTreeSet::from(["admin".to_string()]),
                resource.to_string(),
                TenantScope::new(tenant.clone(), project.clone(), environment.clone()),
                TokenFamilyId::new("fam-admin-no-mcp").unwrap_or_else(|err| panic!("{err}")),
                true,
                now,
            )
            .await
            .unwrap_or_else(|err| panic!("{err:?}"));
        assert!(
            authorize_oauth(
                &store,
                &admin_without_mcp.access_token,
                Some(resource),
                &tenant,
                &project,
                &environment,
                ApiScope::TraceWrite,
            )
            .await
            .is_err(),
            "admin still must carry mcp:invoke to use MCP-issued OAuth tokens"
        );

        let admin_with_mcp = store
            .issue_token_pair(
                OAuthClientId::new("client").unwrap_or_else(|err| panic!("{err}")),
                UserId::new("user").unwrap_or_else(|err| panic!("{err}")),
                BTreeSet::from(["admin".to_string(), "mcp:invoke".to_string()]),
                resource.to_string(),
                TenantScope::new(tenant.clone(), project.clone(), environment.clone()),
                TokenFamilyId::new("fam-admin-mcp").unwrap_or_else(|err| panic!("{err}")),
                true,
                now,
            )
            .await
            .unwrap_or_else(|err| panic!("{err:?}"));
        authorize_oauth(
            &store,
            &admin_with_mcp.access_token,
            Some(resource),
            &tenant,
            &project,
            &environment,
            ApiScope::TraceWrite,
        )
        .await
        .unwrap_or_else(|err| panic!("admin+mcp should satisfy operation scope: {err:?}"));

        // Wrong tenant -> forbidden (token is bound to its tenant scope).
        let other = TenantId::new("evil").unwrap_or_else(|err| panic!("{err}"));
        assert!(
            authorize_oauth(
                &store,
                &issued.access_token,
                Some(resource),
                &other,
                &project,
                &environment,
                ApiScope::TraceRead,
            )
            .await
            .is_err()
        );

        // Missing scope -> forbidden.
        assert!(
            authorize_oauth(
                &store,
                &issued.access_token,
                Some(resource),
                &tenant,
                &project,
                &environment,
                ApiScope::TraceWrite,
            )
            .await
            .is_err()
        );

        // Wrong resource audience -> rejected.
        assert!(
            authorize_oauth(
                &store,
                &issued.access_token,
                Some("https://other.example.com"),
                &tenant,
                &project,
                &environment,
                ApiScope::TraceRead,
            )
            .await
            .is_err()
        );

        // Garbage token -> rejected.
        assert!(
            authorize_oauth(
                &store,
                "bao_nope_nope",
                Some(resource),
                &tenant,
                &project,
                &environment,
                ApiScope::TraceRead,
            )
            .await
            .is_err()
        );
    }

    #[test]
    fn api_state_constructors_share_defaults_and_set_integrations() {
        let (ingest, traces, _tempdir) = api_state_fixture();
        let state = ApiState::new(ingest, traces);
        assert_api_state_optional_defaults(&state);

        let (ingest, traces, _tempdir) = api_state_fixture();
        let search: Arc<dyn SearchIndex> =
            Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let state = ApiState::with_search(ingest, traces, search.clone());
        assert!(Arc::ptr_eq(&state.search, &search));
        assert_api_state_optional_defaults(&state);

        let (ingest, traces, tempdir) = api_state_fixture();
        let search: Arc<dyn SearchIndex> =
            Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let archive = ParquetTraceArchive::new(tempdir.path().join("archive"))
            .unwrap_or_else(|err| panic!("{err}"));
        let archive_root = archive.root().to_path_buf();
        let state = ApiState::with_search_and_archive(ingest, traces, search.clone(), archive);
        assert!(Arc::ptr_eq(&state.search, &search));
        assert_eq!(
            state
                .archive
                .as_ref()
                .unwrap_or_else(|| panic!("archive should be configured"))
                .root(),
            archive_root.as_path()
        );
        assert_api_state_optional_defaults_except_archive(&state);

        let (ingest, traces, tempdir) = api_state_fixture();
        let search: Arc<dyn SearchIndex> =
            Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let archive = ParquetTraceArchive::new(tempdir.path().join("archive"))
            .unwrap_or_else(|err| panic!("{err}"));
        let archive_root = archive.root().to_path_buf();
        let datasets: Arc<dyn DatasetStore> = Arc::new(
            beater_datasets::SqliteDatasetStore::in_memory().unwrap_or_else(|err| panic!("{err}")),
        );
        let experiments: Arc<dyn ExperimentStore> = Arc::new(
            beater_experiments::SqliteExperimentStore::in_memory()
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let state = ApiState::with_integrations(
            ingest,
            traces,
            search.clone(),
            archive,
            datasets.clone(),
            experiments.clone(),
        );
        assert!(Arc::ptr_eq(&state.search, &search));
        assert_eq!(
            state
                .archive
                .as_ref()
                .unwrap_or_else(|| panic!("archive should be configured"))
                .root(),
            archive_root.as_path()
        );
        assert!(Arc::ptr_eq(
            state
                .datasets
                .as_ref()
                .unwrap_or_else(|| panic!("datasets should be configured")),
            &datasets
        ));
        assert!(Arc::ptr_eq(
            state
                .experiments
                .as_ref()
                .unwrap_or_else(|| panic!("experiments should be configured")),
            &experiments
        ));
        assert_api_state_common_defaults(&state);
    }

    #[tokio::test]
    async fn openapi_json_documents_dashboard_read_surface() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
        let app = router(ApiState::new(ingest, traces));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let spec: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        assert!(spec["paths"].get("/v1/traces/{tenant_id}").is_some());
        assert!(
            spec["paths"]
                .get("/v1/spans/{tenant_id}/{trace_id}/{span_id}/io")
                .is_some()
        );
        let trace_params = spec["paths"]["/v1/traces/{tenant_id}"]["get"]["parameters"]
            .as_array()
            .unwrap_or_else(|| panic!("trace list params must be an array"));
        assert!(
            trace_params
                .iter()
                .any(|param| param["name"] == json!("started_after"))
        );
        assert!(
            trace_params
                .iter()
                .any(|param| param["name"] == json!("min_cost_micros"))
        );
    }

    #[tokio::test]
    async fn api_ingests_and_reads_trace() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let search =
            Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
        let app = router(ApiState::with_search(ingest, traces, search));
        let request = fixture_request();
        let body = serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}"));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/traces/native")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/traces/tenant/trace")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/search/tenant/spans?q=hello&status=ok&kind=agent.run")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let search: SearchResponse =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(search.hits.len(), 1);
        assert_eq!(search.hits[0].trace_id, "trace");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/search/tenant/spans?q=hello&status=ok&kind=agent_run")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let search: SearchResponse =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(search.hits.len(), 1);
        assert_eq!(search.hits[0].trace_id, "trace");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/search/tenant/spans?q=hello&kind=not.a.kind")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/search/tenant/spans?q=hello&status=passed")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn api_accepts_otlp_http_protobuf_and_reads_canonical_trace() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let ingest = IngestService::new(
            artifacts.clone(),
            traces.clone(),
            bus,
            IngestPolicy::default(),
        );
        let app = router(ApiState::new(ingest, traces.clone()));
        let body = encode_export_trace_request(&fixture_otlp_export());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/otlp/tenant/project/prod/v1/traces")
                    .header("content-type", "application/x-protobuf")
                    .body(Body::from(body.clone()))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/traces/tenant/0102030405060708090a0b0c0d0e0f10")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
        let response_body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace: TraceView =
            serde_json::from_slice(&response_body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].normalizer_version, "beater-otlp-v1");
        assert_eq!(trace.spans[0].raw_ref.uri, "artifact://redacted");

        let stored_trace = traces
            .get_trace(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                TraceId::new("0102030405060708090a0b0c0d0e0f10")
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let raw_bytes = artifacts
            .get_bytes(&stored_trace.spans[0].raw_ref)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(raw_bytes, body);

        let raw_idempotency_key = IdempotencyKey::new(format!(
            "raw:otlp:tenant:project:{}",
            sha256_hex(&raw_bytes)
        ))
        .unwrap_or_else(|err| panic!("{err}"));
        let raw = traces
            .get_raw_envelope(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                raw_idempotency_key,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("raw otlp envelope should be stored"));
        assert_eq!(raw.source, SourceDialect::Otlp);
        assert_eq!(raw.source_schema_version.as_deref(), Some("1.37.0"));
    }

    #[tokio::test]
    async fn api_accepts_collector_otlp_json_and_reads_canonical_trace() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let ingest = IngestService::new(
            artifacts.clone(),
            traces.clone(),
            bus,
            IngestPolicy::default(),
        );
        let app = router(ApiState::new(ingest, traces.clone()));
        let body =
            serde_json::to_vec(&fixture_otlp_json_export()).unwrap_or_else(|err| panic!("{err}"));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/traces")
                    .header("content-type", "application/json")
                    .body(Body::from(body.clone()))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/traces/tenant/0102030405060708090a0b0c0d0e0f10")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
        let response_body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace: TraceView =
            serde_json::from_slice(&response_body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].kind, AgentSpanKind::AgentRun);
        assert_eq!(trace.spans[0].raw_ref.uri, "artifact://redacted");
        assert_eq!(
            trace.spans[0].attributes.get("beateros.payment_mandate_id"),
            Some(&json!("[redacted]"))
        );
        assert_eq!(
            trace.spans[0].attributes.get("aether.payment_envelope_id"),
            Some(&json!("[redacted]"))
        );
        assert_eq!(
            trace.spans[0]
                .attributes
                .get("aether.agent_payment_authorization"),
            Some(&json!("[redacted]"))
        );

        let stored_trace = traces
            .get_trace(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                TraceId::new("0102030405060708090a0b0c0d0e0f10")
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            stored_trace.spans[0]
                .attributes
                .get("beateros.payment_mandate_id"),
            Some(&json!("mandate-demo"))
        );
        assert_eq!(
            stored_trace.spans[0]
                .attributes
                .get("aether.payment_envelope_id"),
            Some(&json!("payment-envelope-demo"))
        );
        assert_eq!(
            stored_trace.spans[0]
                .attributes
                .get("aether.agent_payment_authorization"),
            Some(&json!("observed"))
        );
        assert_eq!(stored_trace.spans[0].raw_ref.mime_type, "application/json");
        let raw_bytes = artifacts
            .get_bytes(&stored_trace.spans[0].raw_ref)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(raw_bytes, body);

        let raw_idempotency_key = IdempotencyKey::new(format!(
            "raw:otlp:tenant:project:{}",
            sha256_hex(&raw_bytes)
        ))
        .unwrap_or_else(|err| panic!("{err}"));
        let raw = traces
            .get_raw_envelope(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                raw_idempotency_key,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("raw otlp json envelope should be stored"));
        assert_eq!(raw.source, SourceDialect::Otlp);
        assert_eq!(raw.body_ref.mime_type, "application/json");
    }

    #[tokio::test]
    async fn api_rejects_malformed_otlp_http_as_bad_request() {
        let (ingest, traces, _tempdir) = api_state_fixture();
        let app = router(ApiState::new(ingest, traces));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/otlp/tenant/project/prod/v1/traces")
                    .header("content-type", "application/x-protobuf")
                    .body(Body::from(vec![0xff]))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let error: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(error["status"], serde_json::json!(400));
        assert!(
            error["error"]
                .as_str()
                .unwrap_or_default()
                .contains("invalid OTLP trace export")
        );
    }

    #[tokio::test]
    async fn api_rejects_unusable_otlp_semantics_as_bad_request() {
        let (ingest, traces, _tempdir) = api_state_fixture();
        let app = router(ApiState::new(ingest, traces));
        let mut export = fixture_otlp_export();
        export.resource_spans[0].scope_spans[0].spans[0]
            .trace_id
            .clear();
        let body = encode_export_trace_request(&export);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/otlp/tenant/project/prod/v1/traces")
                    .header("content-type", "application/x-protobuf")
                    .body(Body::from(body))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let error: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(error["status"], serde_json::json!(400));
        assert!(
            error["error"]
                .as_str()
                .unwrap_or_default()
                .contains("invalid OTLP trace export")
        );
    }

    fn api_state_fixture() -> (IngestService, Arc<dyn TraceStore>, tempfile::TempDir) {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces: Arc<dyn TraceStore> =
            Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
        (ingest, traces, tempdir)
    }

    fn fixture_otlp_json_export() -> serde_json::Value {
        json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [
                        {"key": "service.name", "value": {"stringValue": "beater.js"}},
                        {"key": "beater.tenant_id", "value": {"stringValue": "tenant"}},
                        {"key": "beater.project_id", "value": {"stringValue": "project"}},
                        {"key": "deployment.environment.name", "value": {"stringValue": "prod"}}
                    ]
                },
                "scopeSpans": [{
                    "scope": {"name": "beater-agent", "version": "0.1.0"},
                    "spans": [{
                        "traceId": "0102030405060708090a0b0c0d0e0f10",
                        "spanId": "1112131415161718",
                        "name": "beater.js agent run",
                        "kind": "SPAN_KIND_INTERNAL",
                        "startTimeUnixNano": "1704067200000000000",
                        "endTimeUnixNano": "1704067201000000000",
                        "attributes": [
                            {"key": "beater.span.kind", "value": {"stringValue": "agent.run"}},
                            {"key": "beater.run_id", "value": {"stringValue": "run-json"}},
                            {"key": "beateros.payment_mandate_id", "value": {"stringValue": "mandate-demo"}},
                            {"key": "beateros.receipt_requirement", "value": {"stringValue": "required"}},
                            {"key": "aether.payment_envelope_id", "value": {"stringValue": "payment-envelope-demo"}},
                            {"key": "aether.agent_payment_authorization", "value": {"stringValue": "observed"}}
                        ],
                        "events": [
                            {
                                "name": "beater.input",
                                "timeUnixNano": "1704067200000000000",
                                "attributes": [
                                    {"key": "beater.payload_json", "value": {"stringValue": "{\"prompt\":\"hello\"}"}}
                                ]
                            },
                            {
                                "name": "beater.output",
                                "timeUnixNano": "1704067201000000000",
                                "attributes": [
                                    {"key": "beater.payload_json", "value": {"stringValue": "{\"answer\":\"world\"}"}}
                                ]
                            }
                        ],
                        "status": {"code": "STATUS_CODE_OK"}
                    }]
                }]
            }]
        })
    }

    fn assert_api_state_optional_defaults(state: &ApiState) {
        assert!(state.archive.is_none());
        assert_api_state_optional_defaults_except_archive(state);
    }

    fn assert_api_state_optional_defaults_except_archive(state: &ApiState) {
        assert!(state.datasets.is_none());
        assert!(state.experiments.is_none());
        assert_api_state_common_defaults(state);
    }

    fn assert_api_state_common_defaults(state: &ApiState) {
        assert!(state.gates.is_none());
        assert!(state.human_reviews.is_none());
        assert!(state.calibrations.is_none());
        assert!(state.provider_secrets.is_none());
        assert!(state.judge_broker.is_none());
        assert!(state.judge_ledger.is_none());
        assert!(state.usage.is_none());
        assert!(state.audit.is_none());
        assert!(!state.auth_required());
        assert!(state.api_keys.is_none());
    }

    fn fixture_request() -> NativeIngestRequest {
        NativeIngestRequest {
            scope: TenantScope::new(
                TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            ),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "agent run".to_string(),
            status: SpanStatus::Ok,
            start_time: None,
            end_time: None,
            model: None,
            cost: None,
            tokens: None,
            input: Some(json!({"hello": "world"})),
            output: None,
            attributes: BTreeMap::new(),
            redaction_class: RedactionClass::Internal,
            idempotency_key: None,
            auth_context: None,
        }
    }

    fn fixture_otlp_export() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![kv("service.name", string_value("checkout-agent"))],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "beater-test".to_string(),
                        version: "1.0.0".to_string(),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                    }),
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![17, 18, 19, 20, 21, 22, 23, 24],
                        trace_state: String::new(),
                        parent_span_id: Vec::new(),
                        flags: 0,
                        name: "llm call".to_string(),
                        kind: span::SpanKind::Client as i32,
                        start_time_unix_nano: 1_700_000_000_000_000_000,
                        end_time_unix_nano: 1_700_000_001_000_000_000,
                        attributes: vec![
                            kv("openinference.span.kind", string_value("llm")),
                            kv("input.value", string_value("hello")),
                            kv("output.value", string_value("world")),
                        ],
                        dropped_attributes_count: 0,
                        events: Vec::new(),
                        dropped_events_count: 0,
                        links: Vec::new(),
                        dropped_links_count: 0,
                        status: Some(Status {
                            message: String::new(),
                            code: status::StatusCode::Ok as i32,
                        }),
                    }],
                    schema_url: "https://opentelemetry.io/schemas/1.37.0".to_string(),
                }],
                schema_url: "https://opentelemetry.io/schemas/1.37.0".to_string(),
            }],
        }
    }

    fn kv(key: &str, value: AnyValue) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            key_strindex: 0,
            value: Some(value),
        }
    }

    fn string_value(value: &str) -> AnyValue {
        AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }
    }

    fn redaction_fixture_artifact(redaction_class: RedactionClass) -> ArtifactRef {
        ArtifactRef {
            artifact_id: ArtifactId::new("artifact").unwrap_or_else(|err| panic!("{err}")),
            uri: "artifact://raw".to_string(),
            sha256: Sha256Hash::new("test-sha").unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 42,
            mime_type: "application/json".to_string(),
            redaction_class,
        }
    }

    fn redaction_fixture_span(
        redaction_class: RedactionClass,
        attributes: BTreeMap<String, serde_json::Value>,
        unmapped_attrs: serde_json::Value,
    ) -> CanonicalSpan {
        CanonicalSpan {
            schema_version: 1,
            normalizer_version: "test".to_string(),
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "span".to_string(),
            status: SpanStatus::Ok,
            start_time: Utc::now(),
            end_time: None,
            model: None,
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref: None,
            attributes,
            unmapped_attrs,
            raw_ref: redaction_fixture_artifact(redaction_class),
        }
    }

    fn leaky_sensitive_attributes() -> BTreeMap<String, serde_json::Value> {
        BTreeMap::from([
            ("input.value".to_string(), json!("question")),
            ("output.value".to_string(), json!("answer")),
            (
                "http.request.header.x-auth-token".to_string(),
                json!("Bearer super-secret"),
            ),
            ("agent.release_id".to_string(), json!("compose-demo")),
            (
                "db.statement".to_string(),
                json!("SELECT * FROM users WHERE password = 'hunter2'"),
            ),
            ("api_key".to_string(), json!("sk-live-abc123")),
            ("gen_ai.prompt.0.content".to_string(), json!("raw prompt")),
        ])
    }

    #[test]
    fn redact_trace_view_redacts_sensitive_attribute_values_and_provenance() {
        let span = redaction_fixture_span(
            RedactionClass::Sensitive,
            leaky_sensitive_attributes(),
            json!({
                "dropped_attributes": {
                    "unlisted.secret": "allow-list miss leaked before #314"
                },
                "unmapped": {
                    "custom.token": "leak-me",
                    "nested": { "token": "nested-leak" },
                    "array": ["array-leak"]
                }
            }),
        );
        let trace = TraceView {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            spans: vec![span],
        };

        let redacted = redact_trace_view(trace);
        let span = &redacted.spans[0];
        for key in [
            "input.value",
            "output.value",
            "http.request.header.x-auth-token",
            "db.statement",
            "api_key",
            "gen_ai.prompt.0.content",
        ] {
            assert_eq!(
                span.attributes[key],
                json!("[redacted]"),
                "{key} value must be redacted on sensitive reads"
            );
        }
        assert_eq!(span.attributes["agent.release_id"], json!("compose-demo"));

        let redacted_text = serde_json::to_string(span).unwrap_or_else(|err| panic!("{err}"));
        for secret in [
            "question",
            "answer",
            "super-secret",
            "hunter2",
            "sk-live-abc123",
            "raw prompt",
            "allow-list miss leaked before #314",
            "leak-me",
            "nested-leak",
            "array-leak",
        ] {
            assert!(
                !redacted_text.contains(secret),
                "redacted sensitive read leaked {secret}"
            );
        }
        assert_eq!(span.raw_ref.uri, "artifact://redacted");
        assert_eq!(
            span.unmapped_attrs["dropped_attributes"]["unlisted.secret"],
            json!("[redacted]")
        );
        assert_eq!(
            span.unmapped_attrs["unmapped"]["custom.token"],
            json!("[redacted]")
        );
        assert_eq!(
            span.unmapped_attrs["unmapped"]["nested"]["token"],
            json!("[redacted]")
        );
        assert_eq!(
            span.unmapped_attrs["unmapped"]["array"][0],
            json!("[redacted]")
        );
    }

    #[test]
    fn redact_trace_view_redacts_secret_spans_like_sensitive_spans() {
        let span = redaction_fixture_span(
            RedactionClass::Secret,
            leaky_sensitive_attributes(),
            json!({ "unmapped": { "custom.token": "leak-me" } }),
        );
        let trace = TraceView {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            spans: vec![span],
        };

        let redacted = redact_trace_view(trace);
        assert_eq!(
            redacted.spans[0].attributes["agent.release_id"],
            json!("compose-demo")
        );
        assert_eq!(redacted.spans[0].attributes["api_key"], json!("[redacted]"));
        assert_eq!(
            redacted.spans[0].unmapped_attrs["unmapped"]["custom.token"],
            json!("[redacted]")
        );
    }

    #[test]
    fn redact_trace_view_leaves_internal_spans_readable() {
        let unmapped = json!({ "unmapped": { "custom.flag": true } });
        let span = redaction_fixture_span(
            RedactionClass::Internal,
            leaky_sensitive_attributes(),
            unmapped.clone(),
        );
        let trace = TraceView {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            spans: vec![span],
        };

        let redacted = redact_trace_view(trace);
        let attributes = &redacted.spans[0].attributes;
        assert_eq!(attributes["input.value"], json!("question"));
        assert_eq!(
            attributes["http.request.header.x-auth-token"],
            json!("Bearer super-secret")
        );
        assert_eq!(attributes["agent.release_id"], json!("compose-demo"));
        assert_eq!(attributes["api_key"], json!("sk-live-abc123"));
        assert_eq!(redacted.spans[0].unmapped_attrs, unmapped);
        assert_eq!(redacted.spans[0].raw_ref.uri, "artifact://raw");
    }
}
