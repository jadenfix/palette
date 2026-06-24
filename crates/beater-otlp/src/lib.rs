use beater_core::{
    lower_hex, Currency, EnvironmentId, IdempotencyKey, Money, ProjectId, SpanId, TenantId,
    TenantScope, Timestamp, TokenCounts, TraceId,
};
use beater_ingest::{
    anonymous_auth_context, CanonicalSpanDraft, IngestError, IngestService, NativeIngestRequest,
    RawTraceIngestRequest,
};
use beater_schema::{
    AgentSpanKind, AuthContext, CanonicalAttrs, ModelRef, RedactionClass, SourceDialect, SpanStatus,
};
use chrono::{TimeZone, Utc};
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::TraceService, ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use opentelemetry_proto::tonic::common::v1::{
    any_value, AnyValue, ArrayValue, InstrumentationScope,
};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{span, ResourceSpans, ScopeSpans, Span};
use prost::Message;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use tonic::metadata::MetadataMap;
use tonic::{Request, Response, Status};

pub use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::TraceServiceServer;

const TENANT_METADATA_KEY: &str = "x-beater-tenant-id";
const PROJECT_METADATA_KEY: &str = "x-beater-project-id";
const ENVIRONMENT_METADATA_KEY: &str = "x-beater-environment-id";

/// Canonical `browser.*` semantic-convention attribute keys emitted by external
/// instrumentation SDKs (`sdks/python-browser-use`, `sdks/ts-stagehand`) and the
/// Rust capture layer. Kept in sync with `beater_browser::semconv`; hardcoded
/// here to avoid a dependency edge into `beater-otlp`.
const BROWSER_ACTION: &str = "browser.action";
const BROWSER_REASONING: &str = "browser.reasoning";
const BROWSER_STEP_STATUS: &str = "browser.step_status";
/// OTLP span name an SDK uses for the per-step LLM decision span.
const BROWSER_DECISION_SPAN_NAME: &str = "browser.decision";

pub fn decode_export_trace_request(bytes: &[u8]) -> anyhow::Result<ExportTraceServiceRequest> {
    ExportTraceServiceRequest::decode(bytes).map_err(anyhow::Error::from)
}

pub fn encode_export_trace_request(request: &ExportTraceServiceRequest) -> Vec<u8> {
    request.encode_to_vec()
}

#[derive(Clone)]
pub struct OtlpGrpcTraceService {
    ingest: IngestService,
    default_scope: TenantScope,
}

impl OtlpGrpcTraceService {
    pub fn new(ingest: IngestService, default_scope: TenantScope) -> Self {
        Self {
            ingest,
            default_scope,
        }
    }
}

#[tonic::async_trait]
impl TraceService for OtlpGrpcTraceService {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let scope = scope_from_metadata(request.metadata(), &self.default_scope)?;
        let export = request.into_inner();
        let raw_bytes = encode_export_trace_request(&export);
        let raw_request =
            export_to_raw_trace_ingest_request(scope, raw_bytes, export, anonymous_auth_context())
                .map_err(|err| Status::invalid_argument(err.to_string()))?;
        self.ingest
            .buffer_raw_trace_batch(raw_request)
            .await
            .map_err(status_from_ingest_error)?;
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

pub fn export_to_native_requests(
    scope: TenantScope,
    request: ExportTraceServiceRequest,
) -> anyhow::Result<Vec<NativeIngestRequest>> {
    let mut converted = Vec::new();
    for resource_spans in request.resource_spans {
        convert_resource_spans(&scope, resource_spans, &mut converted)?;
    }
    Ok(converted)
}

pub fn export_to_raw_trace_ingest_request(
    scope: TenantScope,
    raw_bytes: Vec<u8>,
    request: ExportTraceServiceRequest,
    auth_context: AuthContext,
) -> anyhow::Result<RawTraceIngestRequest> {
    let source_schema_url = first_schema_url(&request);
    let source_schema_version = source_schema_url
        .as_deref()
        .and_then(schema_version_from_url)
        .map(str::to_string);
    let spans = export_to_native_requests(scope.clone(), request)?
        .into_iter()
        .map(native_to_span_draft)
        .collect();
    Ok(RawTraceIngestRequest {
        scope,
        source: SourceDialect::Otlp,
        source_schema_url,
        source_schema_version,
        normalizer_version: "beater-otlp-v1".to_string(),
        mime_type: "application/x-protobuf".to_string(),
        redaction_class: RedactionClass::Internal,
        raw_bytes,
        raw_idempotency_key: None,
        auth_context: Some(auth_context),
        spans,
    })
}

fn native_to_span_draft(request: NativeIngestRequest) -> CanonicalSpanDraft {
    CanonicalSpanDraft {
        trace_id: request.trace_id,
        span_id: request.span_id,
        parent_span_id: request.parent_span_id,
        seq: request.seq,
        kind: request.kind,
        name: request.name,
        status: request.status,
        start_time: request.start_time,
        end_time: request.end_time,
        model: request.model,
        cost: request.cost,
        tokens: request.tokens,
        input: request.input,
        output: request.output,
        attributes: request.attributes,
    }
}

fn scope_from_metadata(
    metadata: &MetadataMap,
    default_scope: &TenantScope,
) -> Result<TenantScope, Status> {
    let tenant_id = metadata_text(metadata, TENANT_METADATA_KEY)
        .map(TenantId::new)
        .transpose()
        .map_err(|err| Status::invalid_argument(err.to_string()))?
        .unwrap_or_else(|| default_scope.tenant_id.clone());
    let project_id = metadata_text(metadata, PROJECT_METADATA_KEY)
        .map(ProjectId::new)
        .transpose()
        .map_err(|err| Status::invalid_argument(err.to_string()))?
        .unwrap_or_else(|| default_scope.project_id.clone());
    let environment_id = metadata_text(metadata, ENVIRONMENT_METADATA_KEY)
        .map(EnvironmentId::new)
        .transpose()
        .map_err(|err| Status::invalid_argument(err.to_string()))?
        .unwrap_or_else(|| default_scope.environment_id.clone());
    Ok(TenantScope::new(tenant_id, project_id, environment_id))
}

fn metadata_text<'a>(metadata: &'a MetadataMap, key: &str) -> Option<&'a str> {
    metadata.get(key).and_then(|value| value.to_str().ok())
}

fn status_from_ingest_error(error: IngestError) -> Status {
    match error {
        IngestError::QuotaExceeded { .. } | IngestError::Backpressure { .. } => {
            Status::resource_exhausted(error.to_string())
        }
        IngestError::TooManyAttributes { .. } | IngestError::PayloadTooLarge { .. } => {
            Status::invalid_argument(error.to_string())
        }
        IngestError::NotFound(_) => Status::not_found(error.to_string()),
        IngestError::Import(_) => Status::invalid_argument(error.to_string()),
        IngestError::Store(_) => Status::unavailable(error.to_string()),
        IngestError::Other(_) => Status::internal(error.to_string()),
    }
}

fn convert_resource_spans(
    scope: &TenantScope,
    resource_spans: ResourceSpans,
    converted: &mut Vec<NativeIngestRequest>,
) -> anyhow::Result<()> {
    let resource_attrs = resource_attrs(resource_spans.resource.as_ref());
    let resource_schema_url = resource_spans.schema_url;
    for scope_spans in resource_spans.scope_spans {
        convert_scope_spans(
            scope,
            &resource_attrs,
            &resource_schema_url,
            scope_spans,
            converted,
        )?;
    }
    Ok(())
}

fn first_schema_url(request: &ExportTraceServiceRequest) -> Option<String> {
    for resource_spans in &request.resource_spans {
        if !resource_spans.schema_url.is_empty() {
            return Some(resource_spans.schema_url.clone());
        }
        for scope_spans in &resource_spans.scope_spans {
            if !scope_spans.schema_url.is_empty() {
                return Some(scope_spans.schema_url.clone());
            }
        }
    }
    None
}

fn schema_version_from_url(schema_url: &str) -> Option<&str> {
    schema_url
        .rsplit('/')
        .next()
        .filter(|version| !version.is_empty() && version.chars().any(|ch| ch.is_ascii_digit()))
}

fn convert_scope_spans(
    scope: &TenantScope,
    resource_attrs: &CanonicalAttrs,
    resource_schema_url: &str,
    scope_spans: ScopeSpans,
    converted: &mut Vec<NativeIngestRequest>,
) -> anyhow::Result<()> {
    let instrumentation_scope = scope_spans.scope;
    let scope_schema_url = scope_spans.schema_url;
    for (index, span) in scope_spans.spans.into_iter().enumerate() {
        converted.push(convert_span(
            scope,
            resource_attrs,
            resource_schema_url,
            instrumentation_scope.as_ref(),
            &scope_schema_url,
            span,
            index as u64,
        )?);
    }
    Ok(())
}

fn convert_span(
    scope: &TenantScope,
    resource_attrs: &CanonicalAttrs,
    resource_schema_url: &str,
    instrumentation_scope: Option<&InstrumentationScope>,
    scope_schema_url: &str,
    span: Span,
    fallback_seq: u64,
) -> anyhow::Result<NativeIngestRequest> {
    let trace_id = TraceId::new(lower_hex(&span.trace_id))?;
    let span_id = SpanId::new(lower_hex(&span.span_id))?;
    let parent_span_id = if span.parent_span_id.is_empty() {
        None
    } else {
        Some(SpanId::new(lower_hex(&span.parent_span_id))?)
    };
    let mut attributes = BTreeMap::new();
    for (key, value) in resource_attrs {
        attributes.insert(format!("resource.{key}"), value.clone());
    }
    attributes.insert(
        "otel.resource_schema_url".to_string(),
        Value::String(resource_schema_url.to_string()),
    );
    attributes.insert(
        "otel.scope_schema_url".to_string(),
        Value::String(scope_schema_url.to_string()),
    );
    if let Some(scope) = instrumentation_scope {
        if !scope.name.is_empty() {
            attributes.insert(
                "otel.scope.name".to_string(),
                Value::String(scope.name.clone()),
            );
        }
        if !scope.version.is_empty() {
            attributes.insert(
                "otel.scope.version".to_string(),
                Value::String(scope.version.clone()),
            );
        }
    }
    for attr in span.attributes {
        attributes.insert(attr.key, any_value_to_json(attr.value.as_ref()));
    }
    attributes.insert(
        "otel.span.kind".to_string(),
        json!(span_kind_name(span.kind)),
    );
    attributes.insert(
        "otel.dropped_attributes_count".to_string(),
        json!(span.dropped_attributes_count),
    );
    attributes.insert(
        "otel.dropped_events_count".to_string(),
        json!(span.dropped_events_count),
    );
    attributes.insert(
        "otel.dropped_links_count".to_string(),
        json!(span.dropped_links_count),
    );
    if !span.trace_state.is_empty() {
        attributes.insert(
            "w3c.tracestate".to_string(),
            Value::String(span.trace_state),
        );
    }

    // The explicit OTLP span status is authoritative; the browser step status
    // only fills an unset status, and a browser-reported error must never be
    // downgraded by (nor allowed to mask) the transport status.
    let status = match convert_status(span.status.as_ref()) {
        SpanStatus::Error => SpanStatus::Error,
        SpanStatus::Ok if browser_status_override(&attributes) == Some(SpanStatus::Error) => {
            SpanStatus::Error
        }
        SpanStatus::Ok => SpanStatus::Ok,
        SpanStatus::Unset => browser_status_override(&attributes).unwrap_or(SpanStatus::Unset),
    };
    let kind = infer_agent_span_kind(&attributes, &span.name, span.kind);
    if temporal_span_kind(&attributes, &span.name).is_some() {
        attributes
            .entry("beater.framework".to_string())
            .or_insert_with(|| Value::String("temporal".to_string()));
    }
    let model = extract_model(&attributes);
    let cost = extract_cost(&attributes);
    let tokens = extract_tokens(&attributes);
    let start_time = unix_nano_to_timestamp(span.start_time_unix_nano);
    let end_time = unix_nano_to_timestamp(span.end_time_unix_nano);
    let seq = attributes
        .get("beater.seq")
        .and_then(Value::as_u64)
        .unwrap_or(fallback_seq + 1);

    Ok(NativeIngestRequest {
        scope: scope.clone(),
        trace_id,
        span_id,
        parent_span_id,
        seq,
        kind,
        name: span.name,
        status,
        start_time,
        end_time,
        model,
        cost,
        tokens,
        input: attributes.remove("input.value"),
        output: attributes.remove("output.value"),
        attributes,
        redaction_class: RedactionClass::Internal,
        idempotency_key: Some(IdempotencyKey::new(format!(
            "otlp:{}:{}:{}:{}",
            scope.tenant_id.as_str(),
            scope.project_id.as_str(),
            lower_hex(&span.trace_id),
            lower_hex(&span.span_id)
        ))?),
        auth_context: None,
    })
}

fn resource_attrs(resource: Option<&Resource>) -> CanonicalAttrs {
    let mut attrs = BTreeMap::new();
    if let Some(resource) = resource {
        for attr in &resource.attributes {
            attrs.insert(attr.key.clone(), any_value_to_json(attr.value.as_ref()));
        }
        attrs.insert(
            "otel.dropped_resource_attributes_count".to_string(),
            json!(resource.dropped_attributes_count),
        );
    }
    attrs
}

fn convert_status(status: Option<&opentelemetry_proto::tonic::trace::v1::Status>) -> SpanStatus {
    let Some(status) = status else {
        return SpanStatus::Unset;
    };
    match opentelemetry_proto::tonic::trace::v1::status::StatusCode::try_from(status.code) {
        Ok(opentelemetry_proto::tonic::trace::v1::status::StatusCode::Ok) => SpanStatus::Ok,
        Ok(opentelemetry_proto::tonic::trace::v1::status::StatusCode::Error) => SpanStatus::Error,
        _ => SpanStatus::Unset,
    }
}

/// Recognize spans emitted by Temporal's built-in OpenTelemetry tracing interceptor
/// (any SDK language). Temporal encodes the unit of work both in `temporal.*`
/// attributes and in the span name prefix (`RunWorkflow:`, `StartActivity:`, …), so we
/// check attributes first and fall back to the name. Returns `None` for non-Temporal
/// spans so normal inference proceeds.
fn temporal_span_kind(attrs: &CanonicalAttrs, name: &str) -> Option<AgentSpanKind> {
    let has = |key: &str| attrs.contains_key(key);
    if has("temporal.workflow.type") || has("temporalWorkflowType") {
        return Some(AgentSpanKind::AgentRun);
    }
    if has("temporal.activity.type") || has("temporalActivityType") {
        return Some(AgentSpanKind::ToolCall);
    }
    // Temporal interceptor span names are always the `Verb:Type` form (e.g.
    // `RunWorkflow:OrderWorkflow`). Require the colon so a non-Temporal span merely
    // named "StartActivity" is not misclassified.
    let prefix = name.split_once(':').map(|(prefix, _)| prefix)?;
    match prefix {
        "RunWorkflow" | "StartWorkflow" | "StartChildWorkflow" | "RunChildWorkflow" => {
            Some(AgentSpanKind::AgentRun)
        }
        "StartActivity" | "RunActivity" => Some(AgentSpanKind::ToolCall),
        "HandleSignal" | "HandleQuery" | "HandleUpdate" => Some(AgentSpanKind::AgentStep),
        _ => None,
    }
}

fn infer_agent_span_kind(attrs: &CanonicalAttrs, name: &str, otel_kind: i32) -> AgentSpanKind {
    // An explicit, operator/SDK-declared kind is authoritative and must win
    // over browser-marker inference — e.g. a browser decision span sets both
    // `beater.span.kind=llm.call` and `browser.action`, and the declared kind
    // is correct.
    if let Some(value) = attrs
        .get("openinference.span.kind")
        .or_else(|| attrs.get("beater.span.kind"))
        .or_else(|| attrs.get("gen_ai.operation.name"))
        .and_then(Value::as_str)
    {
        let value = value.to_ascii_lowercase();
        if let Some(kind) = AgentSpanKind::parse(&value) {
            return kind;
        }
        return match value.as_str() {
            "agent" | "agent.run" => AgentSpanKind::AgentRun,
            "turn" | "agent.turn" => AgentSpanKind::AgentTurn,
            "plan" | "agent.plan" => AgentSpanKind::AgentPlan,
            "chain" | "agent.step" => AgentSpanKind::AgentStep,
            "llm" | "chat" | "generate_content" | "llm.call" => AgentSpanKind::LlmCall,
            "tool" | "tool.call" => AgentSpanKind::ToolCall,
            "mcp" | "mcp.request" => AgentSpanKind::McpRequest,
            "retriever" | "retrieval.query" => AgentSpanKind::RetrievalQuery,
            "embedding" => AgentSpanKind::RetrievalQuery,
            "memory_read" | "memory.read" => AgentSpanKind::MemoryRead,
            "memory_write" | "memory.write" => AgentSpanKind::MemoryWrite,
            "guardrail" | "guardrail.check" => AgentSpanKind::GuardrailCheck,
            "human" | "human_review" | "human.review" => AgentSpanKind::HumanReview,
            "evaluator" | "evaluator.run" => AgentSpanKind::EvaluatorRun,
            "replay" | "replay.run" => AgentSpanKind::ReplayRun,
            _ => AgentSpanKind::AgentStep,
        };
    }
    // No explicit kind: browser-step spans from external SDKs carry `browser.*`
    // markers but a non-canonical OTLP name, so classify them from the markers.
    if let Some(kind) = infer_browser_span_kind(attrs, name) {
        return kind;
    }
    // Temporal interceptor spans, recognized by `temporal.*` attrs or name prefix.
    if let Some(kind) = temporal_span_kind(attrs, name) {
        return kind;
    }
    match span::SpanKind::try_from(otel_kind).unwrap_or(span::SpanKind::Internal) {
        span::SpanKind::Client => AgentSpanKind::ToolCall,
        span::SpanKind::Server => AgentSpanKind::AgentRun,
        span::SpanKind::Producer | span::SpanKind::Consumer => AgentSpanKind::AgentStep,
        span::SpanKind::Internal | span::SpanKind::Unspecified => AgentSpanKind::AgentStep,
    }
}

/// Classifies a browser-step span from its `browser.*` markers.
///
/// - A span carrying `browser.action` is a browser tool invocation
///   (`ToolCall`), regardless of its OTLP span name.
/// - A span representing the agent's LLM decision — name `browser.decision`, or
///   carrying `browser.reasoning` — is an `LlmCall`.
///
/// Returns `None` for spans that carry no browser markers, leaving the existing
/// classification untouched.
fn infer_browser_span_kind(attrs: &CanonicalAttrs, name: &str) -> Option<AgentSpanKind> {
    let is_decision = name.eq_ignore_ascii_case(BROWSER_DECISION_SPAN_NAME)
        || attrs.contains_key(BROWSER_REASONING);
    if is_decision {
        return Some(AgentSpanKind::LlmCall);
    }
    if attrs.contains_key(BROWSER_ACTION) {
        return Some(AgentSpanKind::ToolCall);
    }
    None
}

/// Maps `browser.step_status` to a canonical span status, letting the
/// browser-reported step outcome surface even when the OTLP span status is unset.
/// Returns `None` when the span carries no `browser.step_status`.
fn browser_status_override(attrs: &CanonicalAttrs) -> Option<SpanStatus> {
    let status = attrs.get(BROWSER_STEP_STATUS).and_then(Value::as_str)?;
    if status.eq_ignore_ascii_case("error") {
        Some(SpanStatus::Error)
    } else if status.eq_ignore_ascii_case("ok") {
        Some(SpanStatus::Ok)
    } else {
        None
    }
}

fn extract_model(attrs: &CanonicalAttrs) -> Option<ModelRef> {
    let name = string_attr(
        attrs,
        &[
            "llm.model_name",
            "llm.model",
            "gen_ai.request.model",
            "gen_ai.response.model",
            "model_name",
            "model",
        ],
    )?;
    let provider = string_attr(
        attrs,
        &[
            "llm.provider",
            "gen_ai.system",
            "model.provider",
            "provider",
        ],
    )
    .or_else(|| infer_provider_from_model(&name))
    .unwrap_or_else(|| "unknown".to_string());
    Some(ModelRef { provider, name })
}

fn infer_provider_from_model(model: &str) -> Option<String> {
    let lower = model.to_ascii_lowercase();
    if lower.starts_with("gpt-") || lower.starts_with("o1") || lower.starts_with("o3") {
        return Some("openai".to_string());
    }
    if lower.starts_with("claude-") {
        return Some("anthropic".to_string());
    }
    None
}

fn extract_cost(attrs: &CanonicalAttrs) -> Option<Money> {
    let amount_micros = i64_attr(
        attrs,
        &[
            "llm.cost.amount_micros",
            "llm.cost.micros",
            "gen_ai.usage.cost_micros",
            "cost.amount_micros",
            "cost_micros",
        ],
    )?;
    let currency = string_attr(
        attrs,
        &[
            "llm.cost.currency",
            "gen_ai.usage.cost_currency",
            "cost.currency",
            "currency",
        ],
    )
    .unwrap_or_else(|| "USD".to_string());
    if currency.eq_ignore_ascii_case("USD") {
        Some(Money::new(amount_micros, Currency::Usd))
    } else {
        None
    }
}

fn extract_tokens(attrs: &CanonicalAttrs) -> Option<TokenCounts> {
    let input = u64_attr(
        attrs,
        &[
            "llm.token_count.prompt",
            "llm.usage.prompt_tokens",
            "gen_ai.usage.input_tokens",
            "input_tokens",
        ],
    )
    .unwrap_or(0);
    let output = u64_attr(
        attrs,
        &[
            "llm.token_count.completion",
            "llm.usage.completion_tokens",
            "gen_ai.usage.output_tokens",
            "output_tokens",
        ],
    )
    .unwrap_or(0);
    let reasoning = u64_attr(
        attrs,
        &[
            "llm.token_count.reasoning",
            "llm.usage.reasoning_tokens",
            "gen_ai.usage.reasoning_tokens",
            "reasoning_tokens",
        ],
    )
    .unwrap_or(0);
    let cache_read = u64_attr(
        attrs,
        &[
            "llm.token_count.cache_read",
            "llm.usage.cache_read_tokens",
            "gen_ai.usage.cache_read_input_tokens",
            "cache_read_tokens",
        ],
    )
    .unwrap_or(0);
    if input == 0 && output == 0 && reasoning == 0 && cache_read == 0 {
        None
    } else {
        Some(TokenCounts {
            input,
            output,
            reasoning,
            cache_read,
        })
    }
}

fn string_attr(attrs: &CanonicalAttrs, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| attrs.get(*key).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn i64_attr(attrs: &CanonicalAttrs, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| {
        attrs.get(*key).and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
                .or_else(|| value.as_str().and_then(|value| value.parse::<i64>().ok()))
        })
    })
}

fn u64_attr(attrs: &CanonicalAttrs, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|key| {
        attrs.get(*key).and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_i64().and_then(|value| u64::try_from(value).ok()))
                .or_else(|| value.as_str().and_then(|value| value.parse::<u64>().ok()))
        })
    })
}

fn any_value_to_json(value: Option<&AnyValue>) -> Value {
    let Some(value) = value.and_then(|value| value.value.as_ref()) else {
        return Value::Null;
    };
    match value {
        any_value::Value::StringValue(value) => Value::String(value.clone()),
        any_value::Value::BoolValue(value) => Value::Bool(*value),
        any_value::Value::IntValue(value) => json!(value),
        any_value::Value::DoubleValue(value) => json!(value),
        any_value::Value::BytesValue(value) => Value::String(lower_hex(value)),
        any_value::Value::StringValueStrindex(value) => {
            Value::String(format!("string-table-index:{value}"))
        }
        any_value::Value::ArrayValue(value) => array_to_json(value),
        any_value::Value::KvlistValue(value) => {
            let mut map = Map::new();
            for item in &value.values {
                map.insert(item.key.clone(), any_value_to_json(item.value.as_ref()));
            }
            Value::Object(map)
        }
    }
}

fn array_to_json(value: &ArrayValue) -> Value {
    Value::Array(
        value
            .values
            .iter()
            .map(|value| any_value_to_json(Some(value)))
            .collect(),
    )
}

fn unix_nano_to_timestamp(value: u64) -> Option<Timestamp> {
    if value == 0 {
        return None;
    }
    let secs = (value / 1_000_000_000) as i64;
    let nanos = (value % 1_000_000_000) as u32;
    Utc.timestamp_opt(secs, nanos).single()
}

fn span_kind_name(value: i32) -> &'static str {
    span::SpanKind::try_from(value)
        .unwrap_or(span::SpanKind::Unspecified)
        .as_str_name()
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_bus::{DurableBus, InMemoryBus};
    use beater_core::{EnvironmentId, ProjectId, TenantId};
    use beater_ingest::{IngestPolicy, TRACE_WRITE_BATCH_KIND};
    use beater_store_obj::FsArtifactStore;
    use beater_store_sql::SqliteTraceStore;
    use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, KeyValue};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use opentelemetry_proto::tonic::trace::v1::{status, ResourceSpans, ScopeSpans, Span, Status};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    #[tokio::test]
    async fn grpc_trace_service_buffers_otlp_export_from_metadata_scope() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let ingest = IngestService::new(artifacts, traces, bus.clone(), IngestPolicy::default());
        let default_scope = TenantScope::new(
            TenantId::new("default-tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("default-project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("local").unwrap_or_else(|err| panic!("{err}")),
        );
        let service = OtlpGrpcTraceService::new(ingest, default_scope);
        let mut request = Request::new(fixture_export());
        request.metadata_mut().insert(
            TENANT_METADATA_KEY,
            "tenant".parse().unwrap_or_else(|err| panic!("{err}")),
        );
        request.metadata_mut().insert(
            PROJECT_METADATA_KEY,
            "project".parse().unwrap_or_else(|err| panic!("{err}")),
        );
        request.metadata_mut().insert(
            ENVIRONMENT_METADATA_KEY,
            "prod".parse().unwrap_or_else(|err| panic!("{err}")),
        );

        let response = TraceService::export(&service, request)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(response.into_inner().partial_success.is_none());
        assert_eq!(bus.depth_for_kind(TRACE_WRITE_BATCH_KIND).await, Ok(1));
    }

    #[test]
    fn normalizer_accepts_every_canonical_span_kind() {
        // Guard tying beater-otlp <-> beater-schema: every span kind in the
        // cross-language ingest contract must be understood by the normalizer's
        // kind parser and round-trip to the same canonical kind. If someone adds
        // a canonical kind to beater-schema without teaching the normalizer, this
        // fails (it would silently fall through to AgentStep otherwise).
        for wire in beater_schema::conventions::span_kinds() {
            let mut attrs = CanonicalAttrs::new();
            attrs.insert(
                "openinference.span.kind".to_string(),
                Value::String(wire.to_string()),
            );
            // otel_kind and name are irrelevant when the attribute is present.
            let kind = infer_agent_span_kind(&attrs, "", span::SpanKind::Unspecified as i32);
            assert_eq!(
                kind.as_str(),
                wire,
                "normalizer did not round-trip canonical span kind {wire:?}"
            );
        }
    }

    #[test]
    fn decodes_real_otlp_export_request_and_maps_agent_kinds() {
        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        let request = fixture_export();
        let bytes = encode_export_trace_request(&request);
        let decoded = decode_export_trace_request(&bytes).unwrap_or_else(|err| panic!("{err}"));
        let native = export_to_native_requests(scope.clone(), decoded.clone())
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(native.len(), 1);
        let span = &native[0];
        assert_eq!(span.trace_id.as_str(), "0102030405060708090a0b0c0d0e0f10");
        assert_eq!(span.span_id.as_str(), "1112131415161718");
        assert_eq!(span.kind, AgentSpanKind::LlmCall);
        assert_eq!(span.status, SpanStatus::Ok);
        assert_eq!(
            span.model,
            Some(ModelRef {
                provider: "openai".to_string(),
                name: "gpt-demo".to_string(),
            })
        );
        assert_eq!(span.cost, Some(Money::usd_micros(2500)));
        assert_eq!(
            span.tokens,
            Some(TokenCounts {
                input: 12,
                output: 7,
                reasoning: 3,
                cache_read: 2,
            })
        );
        assert_eq!(
            span.attributes["resource.service.name"],
            json!("checkout-agent")
        );
        assert_eq!(span.input, Some(json!("hello")));
        assert_eq!(span.output, Some(json!("world")));

        let raw_request = export_to_raw_trace_ingest_request(
            scope,
            bytes.clone(),
            decoded,
            AuthContext {
                api_key_id: None,
                scopes: BTreeSet::from(["trace:write".to_string()]),
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(raw_request.source, SourceDialect::Otlp);
        assert_eq!(raw_request.raw_bytes, bytes);
        assert_eq!(
            raw_request.source_schema_url.as_deref(),
            Some("https://opentelemetry.io/schemas/1.37.0")
        );
        assert_eq!(raw_request.source_schema_version.as_deref(), Some("1.37.0"));
        assert_eq!(raw_request.normalizer_version, "beater-otlp-v1");
        assert_eq!(raw_request.spans.len(), 1);
        assert_eq!(raw_request.spans[0].kind, AgentSpanKind::LlmCall);
    }

    #[test]
    fn maps_every_canonical_beater_span_kind_from_otlp_attrs() {
        let cases = [
            ("agent.run", AgentSpanKind::AgentRun),
            ("agent.turn", AgentSpanKind::AgentTurn),
            ("agent.plan", AgentSpanKind::AgentPlan),
            ("agent.step", AgentSpanKind::AgentStep),
            ("llm.call", AgentSpanKind::LlmCall),
            ("tool.call", AgentSpanKind::ToolCall),
            ("mcp.request", AgentSpanKind::McpRequest),
            ("retrieval.query", AgentSpanKind::RetrievalQuery),
            ("memory.read", AgentSpanKind::MemoryRead),
            ("memory.write", AgentSpanKind::MemoryWrite),
            ("guardrail.check", AgentSpanKind::GuardrailCheck),
            ("human.review", AgentSpanKind::HumanReview),
            ("evaluator.run", AgentSpanKind::EvaluatorRun),
            ("replay.run", AgentSpanKind::ReplayRun),
        ];
        for (value, expected) in cases {
            let attrs = BTreeMap::from([(
                "openinference.span.kind".to_string(),
                Value::String(value.to_string()),
            )]);
            assert_eq!(
                infer_agent_span_kind(&attrs, "", span::SpanKind::Internal as i32),
                expected,
                "{value} should map to {expected:?}"
            );
        }
    }

    #[test]
    fn infers_temporal_span_kinds() {
        // Attribute-based recognition.
        let workflow_attrs = BTreeMap::from([(
            "temporal.workflow.type".to_string(),
            Value::String("OrderWorkflow".to_string()),
        )]);
        assert_eq!(
            infer_agent_span_kind(&workflow_attrs, "RunWorkflow:OrderWorkflow", 0),
            AgentSpanKind::AgentRun
        );
        let activity_attrs = BTreeMap::from([(
            "temporal.activity.type".to_string(),
            Value::String("ChargeCard".to_string()),
        )]);
        assert_eq!(
            infer_agent_span_kind(&activity_attrs, "RunActivity:ChargeCard", 0),
            AgentSpanKind::ToolCall
        );
        // Name-prefix recognition (stock interceptor sets no temporal.* attributes).
        let empty = BTreeMap::new();
        assert_eq!(
            infer_agent_span_kind(&empty, "StartActivity:ChargeCard", 0),
            AgentSpanKind::ToolCall
        );
        assert_eq!(
            infer_agent_span_kind(&empty, "StartChildWorkflow:Sub", 0),
            AgentSpanKind::AgentRun
        );
    }

    #[test]
    fn normalizes_browser_step_spans_from_external_sdk_attributes() {
        // A browser-step span emitted by an external instrumentation SDK carries
        // `browser.*` attributes and a non-canonical OTLP span name. It must
        // normalize to a `ToolCall` with every `browser.*` attribute preserved
        // and `browser.step_status == "error"` surfaced as `SpanStatus::Error`.
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![kv("service.name", string_value("browse-agent"))],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "browser-use".to_string(),
                        version: "1.0.0".to_string(),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                    }),
                    spans: vec![
                        // Tool span: name is the SDK's own verb, not "tool.call".
                        Span {
                            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            span_id: vec![1, 1, 1, 1, 1, 1, 1, 1],
                            trace_state: String::new(),
                            parent_span_id: Vec::new(),
                            flags: 0,
                            name: "click".to_string(),
                            kind: span::SpanKind::Internal as i32,
                            start_time_unix_nano: 1_700_000_000_000_000_000,
                            end_time_unix_nano: 1_700_000_001_000_000_000,
                            attributes: vec![
                                kv("browser.engine", string_value("chromium")),
                                kv("browser.action", string_value("click")),
                                kv("browser.selector", string_value("#submit")),
                                kv("browser.url", string_value("https://example.com")),
                                kv("browser.title", string_value("Example")),
                                kv("browser.selector_existed", bool_value(true)),
                                kv("browser.matched_element", bool_value(false)),
                                kv("browser.step_seq", int_value(3)),
                                kv("browser.step_status", string_value("error")),
                                kv("browser.dom_artifact_id", string_value("art-dom-1")),
                                kv("browser.screenshot_artifact_id", string_value("art-shot-1")),
                            ],
                            dropped_attributes_count: 0,
                            events: Vec::new(),
                            dropped_events_count: 0,
                            links: Vec::new(),
                            dropped_links_count: 0,
                            // OTLP span status is unset; the browser status drives it.
                            status: None,
                        },
                        // Decision span: the agent's LLM reasoning for the step.
                        Span {
                            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            span_id: vec![2, 2, 2, 2, 2, 2, 2, 2],
                            trace_state: String::new(),
                            parent_span_id: Vec::new(),
                            flags: 0,
                            name: "browser.decision".to_string(),
                            kind: span::SpanKind::Internal as i32,
                            start_time_unix_nano: 1_700_000_000_000_000_000,
                            end_time_unix_nano: 1_700_000_001_000_000_000,
                            attributes: vec![
                                kv(
                                    "browser.reasoning",
                                    string_value("the submit button is visible"),
                                ),
                                kv("browser.step_status", string_value("ok")),
                            ],
                            dropped_attributes_count: 0,
                            events: Vec::new(),
                            dropped_events_count: 0,
                            links: Vec::new(),
                            dropped_links_count: 0,
                            status: None,
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let scope = TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        let native =
            export_to_native_requests(scope, request).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(native.len(), 2);

        let tool = &native[0];
        assert_eq!(tool.kind, AgentSpanKind::ToolCall);
        assert_eq!(tool.status, SpanStatus::Error);
        assert_eq!(tool.attributes["browser.engine"], json!("chromium"));
        assert_eq!(tool.attributes["browser.action"], json!("click"));
        assert_eq!(tool.attributes["browser.selector"], json!("#submit"));
        assert_eq!(tool.attributes["browser.url"], json!("https://example.com"));
        assert_eq!(tool.attributes["browser.title"], json!("Example"));
        assert_eq!(tool.attributes["browser.selector_existed"], json!(true));
        assert_eq!(tool.attributes["browser.matched_element"], json!(false));
        assert_eq!(tool.attributes["browser.step_seq"], json!(3));
        assert_eq!(tool.attributes["browser.step_status"], json!("error"));
        assert_eq!(
            tool.attributes["browser.dom_artifact_id"],
            json!("art-dom-1")
        );
        assert_eq!(
            tool.attributes["browser.screenshot_artifact_id"],
            json!("art-shot-1")
        );

        let decision = &native[1];
        assert_eq!(decision.kind, AgentSpanKind::LlmCall);
        assert_eq!(decision.status, SpanStatus::Ok);
        assert_eq!(
            decision.attributes["browser.reasoning"],
            json!("the submit button is visible")
        );
    }

    pub fn fixture_export() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![kv("service.name", string_value("checkout-agent"))],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope {
                        name: "fixture".to_string(),
                        version: "1.0.0".to_string(),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                    }),
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![17, 18, 19, 20, 21, 22, 23, 24],
                        trace_state: "tenant=tenant".to_string(),
                        parent_span_id: Vec::new(),
                        flags: 0,
                        name: "llm call".to_string(),
                        kind: span::SpanKind::Client as i32,
                        start_time_unix_nano: 1_700_000_000_000_000_000,
                        end_time_unix_nano: 1_700_000_001_000_000_000,
                        attributes: vec![
                            kv("openinference.span.kind", string_value("llm")),
                            kv("llm.provider", string_value("openai")),
                            kv("llm.model_name", string_value("gpt-demo")),
                            kv("llm.cost.amount_micros", int_value(2500)),
                            kv("llm.cost.currency", string_value("USD")),
                            kv("llm.token_count.prompt", int_value(12)),
                            kv("llm.token_count.completion", int_value(7)),
                            kv("llm.token_count.reasoning", int_value(3)),
                            kv("llm.token_count.cache_read", int_value(2)),
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

    fn int_value(value: i64) -> AnyValue {
        AnyValue {
            value: Some(any_value::Value::IntValue(value)),
        }
    }

    fn bool_value(value: bool) -> AnyValue {
        AnyValue {
            value: Some(any_value::Value::BoolValue(value)),
        }
    }
}
