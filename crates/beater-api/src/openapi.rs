#![allow(dead_code)]

use serde::Serialize;
use utoipa::{IntoParams, OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Beater Read API",
        version = "0.1.0",
        description = "Dashboard-facing trace read APIs for Beater agent observability"
    ),
    paths(
        openapi_health,
        openapi_list_traces,
        openapi_get_trace,
        openapi_get_span,
        openapi_get_span_io
    ),
    components(schemas(
        ArtifactRefDoc,
        CanonicalSpanDoc,
        ErrorResponseDoc,
        HealthResponseDoc,
        MoneyDoc,
        PageRunSummaryDoc,
        RunSummaryDoc,
        SpanIoResponseDoc,
        SpanIoValueDoc,
        TokenCountsDoc,
        TraceViewDoc
    )),
    tags(
        (name = "health", description = "Runtime health"),
        (name = "traces", description = "Trace and span read APIs")
    )
)]
pub struct BeaterReadApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    BeaterReadApi::openapi()
}

pub fn openapi_json_pretty() -> Result<String, serde_json::Error> {
    openapi().to_pretty_json()
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct HealthResponseDoc {
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ErrorResponseDoc {
    pub error: String,
    pub status: u16,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PageRunSummaryDoc {
    pub items: Vec<RunSummaryDoc>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct RunSummaryDoc {
    pub tenant_id: String,
    pub project_id: String,
    pub trace_id: String,
    pub first_span_name: String,
    pub span_count: usize,
    #[schema(example = "ok")]
    pub status: String,
    #[schema(value_type = String, format = DateTime)]
    pub started_at: String,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub ended_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub total_cost: Option<MoneyDoc>,
    pub models: Vec<ModelRefDoc>,
    pub release_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct TraceViewDoc {
    pub tenant_id: String,
    pub trace_id: String,
    pub spans: Vec<CanonicalSpanDoc>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct CanonicalSpanDoc {
    pub schema_version: u32,
    pub normalizer_version: String,
    pub tenant_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub seq: u64,
    #[schema(example = "llm.call")]
    pub kind: String,
    pub name: String,
    #[schema(example = "ok")]
    pub status: String,
    #[schema(value_type = String, format = DateTime)]
    pub start_time: String,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub end_time: Option<String>,
    pub model: Option<ModelRefDoc>,
    pub cost: Option<MoneyDoc>,
    pub tokens: Option<TokenCountsDoc>,
    pub input_ref: Option<ArtifactRefDoc>,
    pub output_ref: Option<ArtifactRefDoc>,
    #[schema(value_type = serde_json::Value)]
    pub attributes: serde_json::Value,
    #[schema(value_type = serde_json::Value)]
    pub unmapped_attrs: serde_json::Value,
    pub raw_ref: ArtifactRefDoc,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ModelRefDoc {
    pub provider: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct MoneyDoc {
    pub amount_micros: i64,
    #[schema(example = "USD")]
    pub currency: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct TokenCountsDoc {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ArtifactRefDoc {
    pub artifact_id: String,
    pub uri: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub mime_type: String,
    #[schema(example = "internal")]
    pub redaction_class: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SpanIoResponseDoc {
    pub tenant_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub input: SpanIoValueDoc,
    pub output: SpanIoValueDoc,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpanIoValueDoc {
    Inline {
        #[schema(value_type = serde_json::Value)]
        value: serde_json::Value,
    },
    Artifact {
        artifact_ref: ArtifactRefDoc,
    },
    Redacted {
        reason: String,
    },
    Missing,
}

#[derive(Clone, Debug, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListTracesParams {
    pub project_id: Option<String>,
    pub environment_id: Option<String>,
    pub trace_id: Option<String>,
    #[param(example = "llm.call")]
    pub kind: Option<String>,
    #[param(example = "ok")]
    pub status: Option<String>,
    #[param(value_type = Option<String>, example = "2026-01-01T00:00:00Z")]
    pub started_after: Option<String>,
    #[param(value_type = Option<String>, example = "2026-01-01T01:00:00Z")]
    pub started_before: Option<String>,
    #[param(example = "gpt-4.1")]
    pub model: Option<String>,
    #[param(example = "release-a")]
    pub release: Option<String>,
    pub min_cost_micros: Option<i64>,
    pub max_cost_micros: Option<i64>,
    pub min_latency_ms: Option<i64>,
    pub max_latency_ms: Option<i64>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TraceReadParams {
    pub unmask: Option<bool>,
    pub reason: Option<String>,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Runtime is accepting requests", body = HealthResponseDoc)
    )
)]
fn openapi_health() {}

#[utoipa::path(
    get,
    path = "/v1/traces/{tenant_id}",
    tag = "traces",
    params(
        ("tenant_id" = String, Path, description = "Tenant id"),
        ListTracesParams,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope")
    ),
    responses(
        (status = 200, description = "Trace run summaries", body = PageRunSummaryDoc),
        (status = 400, description = "Invalid scope or filter", body = ErrorResponseDoc),
        (status = 401, description = "Missing or invalid API key", body = ErrorResponseDoc),
        (status = 403, description = "API key lacks trace read scope", body = ErrorResponseDoc)
    )
)]
fn openapi_list_traces() {}

#[utoipa::path(
    get,
    path = "/v1/traces/{tenant_id}/{trace_id}",
    tag = "traces",
    params(
        ("tenant_id" = String, Path, description = "Tenant id"),
        ("trace_id" = String, Path, description = "Trace id"),
        TraceReadParams,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope")
    ),
    responses(
        (status = 200, description = "Canonical trace with redaction applied unless unmasked", body = TraceViewDoc),
        (status = 400, description = "Invalid scope or query", body = ErrorResponseDoc),
        (status = 401, description = "Missing or invalid API key", body = ErrorResponseDoc),
        (status = 403, description = "API key lacks trace read or unmask scope", body = ErrorResponseDoc),
        (status = 404, description = "Trace not found", body = ErrorResponseDoc)
    )
)]
fn openapi_get_trace() {}

#[utoipa::path(
    get,
    path = "/v1/spans/{tenant_id}/{trace_id}/{span_id}",
    tag = "traces",
    params(
        ("tenant_id" = String, Path, description = "Tenant id"),
        ("trace_id" = String, Path, description = "Trace id"),
        ("span_id" = String, Path, description = "Span id"),
        TraceReadParams,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope")
    ),
    responses(
        (status = 200, description = "Canonical span with redaction applied unless unmasked", body = CanonicalSpanDoc),
        (status = 400, description = "Invalid scope or query", body = ErrorResponseDoc),
        (status = 401, description = "Missing or invalid API key", body = ErrorResponseDoc),
        (status = 403, description = "API key lacks trace read or unmask scope", body = ErrorResponseDoc),
        (status = 404, description = "Span or trace not found", body = ErrorResponseDoc)
    )
)]
fn openapi_get_span() {}

#[utoipa::path(
    get,
    path = "/v1/spans/{tenant_id}/{trace_id}/{span_id}/io",
    tag = "traces",
    params(
        ("tenant_id" = String, Path, description = "Tenant id"),
        ("trace_id" = String, Path, description = "Trace id"),
        ("span_id" = String, Path, description = "Span id"),
        TraceReadParams,
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope")
    ),
    responses(
        (status = 200, description = "Redaction-aware span input and output metadata", body = SpanIoResponseDoc),
        (status = 400, description = "Invalid scope or query", body = ErrorResponseDoc),
        (status = 401, description = "Missing or invalid API key", body = ErrorResponseDoc),
        (status = 403, description = "API key lacks trace read or unmask scope", body = ErrorResponseDoc),
        (status = 404, description = "Span or trace not found", body = ErrorResponseDoc)
    )
)]
fn openapi_get_span_io() {}
