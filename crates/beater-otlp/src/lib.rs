use beater_core::{
    EnvironmentId, IdempotencyKey, ProjectId, SpanId, TenantId, TenantScope, Timestamp, TraceId,
};
use beater_ingest::{
    anonymous_auth_context, CanonicalSpanDraft, IngestError, IngestService, NativeIngestRequest,
    RawTraceIngestRequest,
};
use beater_schema::{
    AgentSpanKind, AuthContext, CanonicalAttrs, RedactionClass, SourceDialect, SpanStatus,
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
    let trace_id = TraceId::new(hex(&span.trace_id))?;
    let span_id = SpanId::new(hex(&span.span_id))?;
    let parent_span_id = if span.parent_span_id.is_empty() {
        None
    } else {
        Some(SpanId::new(hex(&span.parent_span_id))?)
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

    let status = convert_status(span.status.as_ref());
    let kind = infer_agent_span_kind(&attributes, span.kind);
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
        model: None,
        cost: None,
        tokens: None,
        input: attributes.remove("input.value"),
        output: attributes.remove("output.value"),
        attributes,
        redaction_class: RedactionClass::Internal,
        idempotency_key: Some(IdempotencyKey::new(format!(
            "otlp:{}:{}:{}:{}",
            scope.tenant_id.as_str(),
            scope.project_id.as_str(),
            hex(&span.trace_id),
            hex(&span.span_id)
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

fn infer_agent_span_kind(attrs: &CanonicalAttrs, otel_kind: i32) -> AgentSpanKind {
    if let Some(value) = attrs
        .get("openinference.span.kind")
        .or_else(|| attrs.get("beater.span.kind"))
        .or_else(|| attrs.get("gen_ai.operation.name"))
        .and_then(Value::as_str)
    {
        return match value.to_ascii_lowercase().as_str() {
            "agent" | "agent.run" => AgentSpanKind::AgentRun,
            "chain" | "agent.step" => AgentSpanKind::AgentStep,
            "llm" | "chat" | "generate_content" | "llm.call" => AgentSpanKind::LlmCall,
            "tool" | "tool.call" => AgentSpanKind::ToolCall,
            "mcp" | "mcp.request" => AgentSpanKind::McpRequest,
            "retriever" | "retrieval.query" => AgentSpanKind::RetrievalQuery,
            "embedding" => AgentSpanKind::RetrievalQuery,
            "guardrail" | "guardrail.check" => AgentSpanKind::GuardrailCheck,
            "evaluator" | "evaluator.run" => AgentSpanKind::EvaluatorRun,
            _ => AgentSpanKind::AgentStep,
        };
    }
    match span::SpanKind::try_from(otel_kind).unwrap_or(span::SpanKind::Internal) {
        span::SpanKind::Client => AgentSpanKind::ToolCall,
        span::SpanKind::Server => AgentSpanKind::AgentRun,
        span::SpanKind::Producer | span::SpanKind::Consumer => AgentSpanKind::AgentStep,
        span::SpanKind::Internal | span::SpanKind::Unspecified => AgentSpanKind::AgentStep,
    }
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
        any_value::Value::BytesValue(value) => Value::String(hex(value)),
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
}
