//! Langfuse export → Beater canonical span normalization.
//!
//! Converts a [Langfuse] trace/observations export into Beater's canonical span
//! model, reusing the *exact* same downstream ingest pipeline as the OTLP and
//! Temporal paths (`RawTraceIngestRequest` → `IngestService::ingest_raw_trace_batch`).
//! Nothing here writes storage; it is a pure, deterministic mapping. This is the
//! "migrate in, never get locked in" wedge: a user with a Langfuse export gets
//! their traces into Beater as canonical spans WITH the raw envelope retained
//! losslessly, so projections can be re-derived as conventions evolve.
//!
//! ## Targeted Langfuse shape
//!
//! We target the public-API `TraceWithFullDetails` document returned by
//! `GET /api/public/traces/{traceId}` — a trace object whose `observations` array
//! holds the nested observation views. See the Langfuse API reference:
//! <https://api.reference.langfuse.com/#tag/trace/GET/api/public/traces/{traceId}>
//! and the data-model docs: <https://langfuse.com/docs/tracing-data-model>.
//! Langfuse is MIT-licensed and the export schema is publicly documented.
//!
//! A *trace* carries `id`, `timestamp`, `name`, `input`, `output`, `sessionId`,
//! `userId`, `metadata`, `tags`, `release`, `version`. Each *observation*
//! (`ObservationsView`) carries `id`, `traceId`, `type`
//! (`SPAN` | `GENERATION` | `EVENT`), `name`, `startTime`, `endTime`,
//! `parentObservationId`, `level`, `statusMessage`, `model`, `modelParameters`,
//! `input`, `output`, `usage`/`usageDetails`, `costDetails`/`calculatedTotalCost`,
//! and `metadata`.
//!
//! To be forgiving of the several shapes Langfuse exports take, [`convert_export`]
//! accepts any of:
//! - a single trace object (`{ "id": …, "observations": [...] }`),
//! - `{ "trace": { … } }`,
//! - `{ "data": [ trace, … ] }` (the list-endpoint envelope), or
//! - a bare array `[ trace, … ]`.
//!
//! ## Field mapping (Langfuse → canonical)
//!
//! | Langfuse                              | Canonical                                   |
//! | ------------------------------------- | ------------------------------------------- |
//! | trace                                 | `agent.run` (root span)                     |
//! | observation `type=GENERATION`         | `llm.call`                                  |
//! | observation `type=SPAN`               | `tool.call`                                 |
//! | observation `type=EVENT`              | `agent.step`                                |
//! | `timestamp` / `startTime`             | span `start_time`                           |
//! | `endTime`                             | span `end_time`                             |
//! | `model`                               | `ModelRef { provider, name }` (provider inferred) |
//! | `usage`/`usageDetails`                | `TokenCounts { input, output, … }`          |
//! | `calculatedTotalCost`/`costDetails`   | `cost` (USD micros)                         |
//! | `input` / `output`                    | span `input` / `output`                     |
//! | `parentObservationId`                 | `parent_span_id` (else the trace root)      |
//! | `level=ERROR`                         | `SpanStatus::Error`                         |
//! | everything else (`sessionId`,`tags`,  | `langfuse.*` attributes (kept verbatim) and |
//! |  `metadata`, `modelParameters`, raw …)| the IMMUTABLE raw envelope                  |
//!
//! Anything the canonical model cannot represent stays in `langfuse.*`
//! attributes and — losslessly — in the stored raw envelope. The original
//! Langfuse JSON bytes are carried on [`RawTraceIngestRequest::raw_bytes`] and
//! persisted by the ingest pipeline as the immutable raw copy, so the import is
//! re-projectable as conventions evolve (ARCHITECTURE §1 #2/#3, §9).

use beater_core::{Money, SpanId, TenantScope, Timestamp, TokenCounts, TraceId};
use beater_ingest::{CanonicalSpanDraft, ImportError, RawTraceIngestRequest, SourceImporter};
use beater_schema::{
    AgentSpanKind, AuthContext, CanonicalAttrs, ModelRef, RedactionClass, SourceDialect,
    SpanStatus, conventions::attr,
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

/// Pinned Langfuse export schema this normalizer targets. Stamped onto every
/// Langfuse-sourced span as its `normalizer_version`. Bump explicitly whenever
/// the converter changes how it reads the export.
pub const LANGFUSE_CONTRACT: &str = "langfuse.public-api.observations.v1";

/// Source key selected on the unified `/v1/import` endpoint.
pub const LANGFUSE_SOURCE: &str = "langfuse";

/// Documentation URL recorded on the raw envelope for provenance.
pub const LANGFUSE_SCHEMA_URL: &str =
    "https://api.reference.langfuse.com/#tag/trace/GET/api/public/traces/{traceId}";

/// Errors raised while normalizing a Langfuse export. Every variant is a graceful
/// rejection — the converter never panics on malformed input.
#[derive(Debug, thiserror::Error)]
pub enum LangfuseError {
    #[error("invalid Langfuse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("no Langfuse trace found in export document")]
    MissingTrace,
    #[error("Langfuse trace is missing a string `id`")]
    MissingTraceId,
    #[error("Langfuse trace `id` is not a usable identifier: {0}")]
    InvalidTraceId(String),
    #[error("Langfuse observation is missing a string `id`")]
    MissingObservationId,
    #[error("Langfuse observation `id` is not a usable identifier: {0}")]
    InvalidObservationId(String),
    #[error("Langfuse `observations` field is present but is not an array")]
    ObservationsNotArray,
}

type LangfuseResult<T> = Result<T, LangfuseError>;

/// Per-conversion accounting, proving no observation was silently dropped.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConversionStats {
    pub traces: usize,
    pub observations: usize,
    pub spans: usize,
    /// Observation `type` values outside the pinned set (`SPAN`/`GENERATION`/`EVENT`).
    /// These are mapped to `agent.step` and counted here so an unknown type surfaces
    /// rather than being dropped; the original type is preserved in attributes + raw.
    pub unmapped_observation_types: usize,
}

/// Result of converting one Langfuse export into canonical span drafts.
#[derive(Clone, Debug)]
pub struct ConvertedExport {
    pub drafts: Vec<CanonicalSpanDraft>,
    pub stats: ConversionStats,
}

/// Importer that plugs a Langfuse export into the shared ingest pipeline.
#[derive(Clone, Copy, Debug, Default)]
pub struct LangfuseImporter;

impl SourceImporter for LangfuseImporter {
    fn source(&self) -> &'static str {
        LANGFUSE_SOURCE
    }

    fn normalize(
        &self,
        scope: &TenantScope,
        raw_bytes: &[u8],
        auth: Option<AuthContext>,
    ) -> Result<RawTraceIngestRequest, ImportError> {
        langfuse_export_to_raw_ingest(scope.clone(), raw_bytes.to_vec(), auth).map_err(|err| {
            ImportError::Invalid {
                source_name: LANGFUSE_SOURCE.to_string(),
                message: err.to_string(),
            }
        })
    }
}

/// Convert raw Langfuse export bytes into a `RawTraceIngestRequest` ready for
/// `IngestService::ingest_raw_trace_batch`. The original bytes are carried
/// verbatim on [`RawTraceIngestRequest::raw_bytes`] so the immutable raw envelope
/// is a lossless copy of the source document. Mirrors
/// `beater_temporal::temporal_history_to_raw_ingest`.
pub fn langfuse_export_to_raw_ingest(
    scope: TenantScope,
    raw_bytes: Vec<u8>,
    auth: Option<AuthContext>,
) -> LangfuseResult<RawTraceIngestRequest> {
    let root: Value = serde_json::from_slice(&raw_bytes)?;
    let converted = convert_export(&root)?;
    Ok(RawTraceIngestRequest {
        scope,
        source: SourceDialect::LangfuseImport,
        source_schema_url: Some(LANGFUSE_SCHEMA_URL.to_string()),
        source_schema_version: Some(LANGFUSE_CONTRACT.to_string()),
        normalizer_version: LANGFUSE_CONTRACT.to_string(),
        mime_type: "application/json".to_string(),
        redaction_class: RedactionClass::Sensitive,
        raw_bytes,
        raw_idempotency_key: None,
        auth_context: auth,
        spans: converted.drafts,
    })
}

/// Pure converter: Langfuse export JSON → canonical span drafts + accounting.
pub fn convert_export(root: &Value) -> LangfuseResult<ConvertedExport> {
    let traces = collect_traces(root);
    if traces.is_empty() {
        return Err(LangfuseError::MissingTrace);
    }
    let mut drafts = Vec::new();
    let mut stats = ConversionStats::default();
    for trace in traces {
        convert_one_trace(trace, &mut drafts, &mut stats)?;
    }
    stats.spans = drafts.len();
    Ok(ConvertedExport { drafts, stats })
}

/// Extract every trace object from the supported export envelopes.
fn collect_traces(root: &Value) -> Vec<&Value> {
    if let Some(array) = root.as_array() {
        return array.iter().collect();
    }
    if let Some(trace) = root.get("trace") {
        return vec![trace];
    }
    if let Some(data) = root.get("data").and_then(Value::as_array) {
        return data.iter().collect();
    }
    // A bare trace object: recognized by carrying an `id` (and usually `observations`).
    if root.get("id").is_some() {
        return vec![root];
    }
    Vec::new()
}

fn convert_one_trace(
    trace: &Value,
    drafts: &mut Vec<CanonicalSpanDraft>,
    stats: &mut ConversionStats,
) -> LangfuseResult<()> {
    stats.traces += 1;
    let trace_id_str = trace
        .get("id")
        .and_then(Value::as_str)
        .ok_or(LangfuseError::MissingTraceId)?;
    let trace_id = TraceId::new(trace_id_str)
        .map_err(|_| LangfuseError::InvalidTraceId(trace_id_str.to_string()))?;
    let root_span_id = trace_span_id(trace_id_str)
        .map_err(|_| LangfuseError::InvalidTraceId(trace_id_str.to_string()))?;

    let observations = match trace.get("observations") {
        Some(Value::Array(items)) => items.as_slice(),
        Some(Value::Null) | None => &[],
        Some(_) => return Err(LangfuseError::ObservationsNotArray),
    };
    stats.observations += observations.len();

    // Deterministic ordering: root first, then observations sorted by (startTime, id).
    // `seq` is assigned in this order; parent linkage uses ids and is order-independent.
    let mut sorted: Vec<&Value> = observations.iter().collect();
    sorted.sort_by(|a, b| {
        observation_start(a)
            .cmp(&observation_start(b))
            .then_with(|| {
                obs_id(a)
                    .unwrap_or_default()
                    .cmp(&obs_id(b).unwrap_or_default())
            })
    });

    let trace_start = parse_time(trace.get("timestamp")).or_else(|| {
        sorted
            .iter()
            .find_map(|obs| parse_time(obs.get("startTime")))
    });
    // The trace has no end time of its own; bound it by the latest observation end.
    let trace_end = sorted
        .iter()
        .filter_map(|obs| parse_time(obs.get("endTime")))
        .max();

    let mut seq: u64 = 0;
    drafts.push(build_root_draft(
        trace,
        &trace_id,
        &root_span_id,
        trace_start,
        trace_end,
        seq,
    ));

    for obs in sorted {
        seq += 1;
        drafts.push(build_observation_draft(
            obs,
            &trace_id,
            &root_span_id,
            seq,
            stats,
        )?);
    }
    Ok(())
}

fn build_root_draft(
    trace: &Value,
    trace_id: &TraceId,
    root_span_id: &SpanId,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    seq: u64,
) -> CanonicalSpanDraft {
    let mut attributes = CanonicalAttrs::new();
    attr_str(&mut attributes, "langfuse.kind", "trace");
    attr_str(&mut attributes, "langfuse.trace.id", trace_id.as_str());
    copy_str(&mut attributes, trace, "sessionId", "langfuse.session_id");
    copy_str(&mut attributes, trace, "userId", "langfuse.user_id");
    copy_str(&mut attributes, trace, "release", "langfuse.release");
    copy_str(&mut attributes, trace, "release", attr::RELEASE_ID);
    copy_str(&mut attributes, trace, "version", "langfuse.version");
    copy_value(&mut attributes, trace, "tags", "langfuse.tags");
    copy_value(&mut attributes, trace, "metadata", "langfuse.metadata");

    let name = trace
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("langfuse-trace")
        .to_string();

    CanonicalSpanDraft {
        trace_id: trace_id.clone(),
        span_id: root_span_id.clone(),
        parent_span_id: None,
        seq,
        kind: AgentSpanKind::AgentRun,
        name,
        status: SpanStatus::Ok,
        start_time,
        end_time,
        model: None,
        cost: None,
        tokens: None,
        input: trace.get("input").cloned(),
        output: trace.get("output").cloned(),
        attributes,
    }
}

fn build_observation_draft(
    obs: &Value,
    trace_id: &TraceId,
    root_span_id: &SpanId,
    seq: u64,
    stats: &mut ConversionStats,
) -> LangfuseResult<CanonicalSpanDraft> {
    let obs_id = obs
        .get("id")
        .and_then(Value::as_str)
        .ok_or(LangfuseError::MissingObservationId)?;
    let span_id = observation_span_id(obs_id)
        .map_err(|_| LangfuseError::InvalidObservationId(obs_id.to_string()))?;

    // parentObservationId → parent observation span; absent → the trace root.
    let parent_span_id = match obs.get("parentObservationId").and_then(Value::as_str) {
        Some(pid) => Some(
            observation_span_id(pid)
                .map_err(|_| LangfuseError::InvalidObservationId(pid.to_string()))?,
        ),
        None => Some(root_span_id.clone()),
    };

    let raw_type = obs.get("type").and_then(Value::as_str).unwrap_or("");
    let (kind, recognized) = classify_observation(raw_type);
    if !recognized {
        stats.unmapped_observation_types += 1;
    }

    let mut attributes = CanonicalAttrs::new();
    attr_str(&mut attributes, "langfuse.kind", "observation");
    attr_str(&mut attributes, "langfuse.observation.id", obs_id);
    if !raw_type.is_empty() {
        attr_str(&mut attributes, "langfuse.observation.type", raw_type);
    }
    copy_str(&mut attributes, obs, "level", "langfuse.level");
    copy_str(
        &mut attributes,
        obs,
        "statusMessage",
        "langfuse.status_message",
    );
    copy_str(&mut attributes, obs, "version", "langfuse.version");
    copy_str(&mut attributes, obs, "promptName", "langfuse.prompt.name");
    copy_value(
        &mut attributes,
        obs,
        "promptVersion",
        "langfuse.prompt.version",
    );
    copy_value(
        &mut attributes,
        obs,
        "modelParameters",
        "langfuse.model_parameters",
    );
    copy_value(&mut attributes, obs, "metadata", "langfuse.metadata");
    copy_value(&mut attributes, obs, "usage", "langfuse.usage");
    copy_value(
        &mut attributes,
        obs,
        "usageDetails",
        "langfuse.usage_details",
    );
    copy_value(&mut attributes, obs, "costDetails", "langfuse.cost_details");

    let model = extract_model(obs);
    if let Some(model) = &model {
        attr_str(&mut attributes, "gen_ai.system", &model.provider);
        attr_str(&mut attributes, "gen_ai.request.model", &model.name);
    }

    let tokens = extract_tokens(obs);
    let cost = extract_cost(obs);
    if let Some(cost) = &cost {
        attr_num(
            &mut attributes,
            "langfuse.cost.total_micros_usd",
            cost.amount_micros,
        );
    }

    let name = obs
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| default_observation_name(raw_type));

    // An absent start_time is back-filled to "now" downstream, which for a
    // historical import is always later than the recorded end and yields a
    // negative duration. Anchor an unparseable start to the end instead.
    let end_time = parse_time(obs.get("endTime"));
    let start_time = parse_time(obs.get("startTime")).or(end_time);

    Ok(CanonicalSpanDraft {
        trace_id: trace_id.clone(),
        span_id,
        parent_span_id,
        seq,
        kind,
        name,
        status: observation_status(obs, end_time),
        start_time,
        end_time,
        model,
        cost,
        tokens,
        input: obs.get("input").cloned(),
        output: obs.get("output").cloned(),
        attributes,
    })
}

/// Map a Langfuse observation `type` to a canonical span kind. Returns
/// `(kind, recognized)`; an unrecognized type maps to `agent.step` so it is never
/// dropped, and `recognized=false` flags it for [`ConversionStats`].
fn classify_observation(raw_type: &str) -> (AgentSpanKind, bool) {
    match raw_type.to_ascii_uppercase().as_str() {
        "GENERATION" => (AgentSpanKind::LlmCall, true),
        "SPAN" => (AgentSpanKind::ToolCall, true),
        "EVENT" => (AgentSpanKind::AgentStep, true),
        _ => (AgentSpanKind::AgentStep, false),
    }
}

fn default_observation_name(raw_type: &str) -> String {
    match raw_type.to_ascii_uppercase().as_str() {
        "GENERATION" => "generation".to_string(),
        "SPAN" => "span".to_string(),
        "EVENT" => "event".to_string(),
        _ => "observation".to_string(),
    }
}

fn observation_status(obs: &Value, end_time: Option<Timestamp>) -> SpanStatus {
    match obs.get("level").and_then(Value::as_str) {
        Some(level) if level.eq_ignore_ascii_case("ERROR") => SpanStatus::Error,
        _ if end_time.is_some() => SpanStatus::Ok,
        _ => SpanStatus::Unset,
    }
}

/// Langfuse records only the model *name*; we infer a provider from common name
/// prefixes for a useful `ModelRef` and keep the verbatim model string in
/// `gen_ai.request.model` regardless. Unknown families get `provider="unknown"`.
fn extract_model(obs: &Value) -> Option<ModelRef> {
    let name = obs.get("model").and_then(Value::as_str)?;
    if name.is_empty() {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    let provider = if lower.starts_with("gpt")
        || lower.starts_with("o1")
        || lower.starts_with("o3")
        || lower.starts_with("text-")
        || lower.contains("davinci")
    {
        "openai"
    } else if lower.starts_with("claude") {
        "anthropic"
    } else if lower.starts_with("gemini") || lower.starts_with("palm") {
        "google"
    } else if lower.starts_with("mistral") || lower.starts_with("mixtral") {
        "mistral"
    } else if lower.starts_with("llama") {
        "meta"
    } else {
        "unknown"
    };
    Some(ModelRef {
        provider: provider.to_string(),
        name: name.to_string(),
    })
}

/// Token usage from `usage` (current `{input,output,total,unit}` or legacy
/// `{promptTokens,completionTokens,totalTokens}`) or `usageDetails`.
fn extract_tokens(obs: &Value) -> Option<TokenCounts> {
    let usage = obs.get("usage");
    let details = obs.get("usageDetails");
    let pick = |keys: &[&str]| -> Option<u64> {
        for source in [usage, details].into_iter().flatten() {
            for key in keys {
                if let Some(n) = source.get(*key).and_then(json_u64) {
                    return Some(n);
                }
            }
        }
        None
    };
    let input = pick(&["input", "promptTokens", "input_tokens"]);
    let output = pick(&["output", "completionTokens", "output_tokens"]);
    if input.is_none() && output.is_none() {
        return None;
    }
    Some(TokenCounts {
        input: input.unwrap_or(0),
        output: output.unwrap_or(0),
        reasoning: 0,
        cache_read: 0,
    })
}

/// Cost in USD micros from `calculatedTotalCost` or `costDetails.total`.
fn extract_cost(obs: &Value) -> Option<Money> {
    let usd = obs
        .get("calculatedTotalCost")
        .and_then(Value::as_f64)
        .or_else(|| {
            obs.get("costDetails")
                .and_then(|c| c.get("total"))
                .and_then(Value::as_f64)
        })?;
    if !usd.is_finite() {
        return None;
    }
    let micros = (usd * 1_000_000.0).round();
    // Clamp to i64 range defensively; real costs never approach this bound.
    let micros = micros.clamp(i64::MIN as f64, i64::MAX as f64) as i64;
    Some(Money::usd_micros(micros))
}

fn trace_span_id(trace_id: &str) -> Result<SpanId, beater_core::IdError> {
    SpanId::new(format!("lf-trace-{trace_id}"))
}

fn observation_span_id(obs_id: &str) -> Result<SpanId, beater_core::IdError> {
    SpanId::new(format!("lf-obs-{obs_id}"))
}

fn obs_id(obs: &Value) -> Option<String> {
    obs.get("id").and_then(Value::as_str).map(str::to_string)
}

fn observation_start(obs: &Value) -> Option<Timestamp> {
    parse_time(obs.get("startTime"))
}

fn parse_time(value: Option<&Value>) -> Option<Timestamp> {
    let raw = value?.as_str()?;
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn json_u64(value: &Value) -> Option<u64> {
    if let Some(n) = value.as_u64() {
        return Some(n);
    }
    // Some exporters emit token counts as floats; accept non-negative whole values.
    value
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .map(|n| n.round() as u64)
}

fn attr_str(attributes: &mut CanonicalAttrs, key: &str, value: &str) {
    attributes.insert(key.to_string(), Value::String(value.to_string()));
}

fn attr_num(attributes: &mut CanonicalAttrs, key: &str, value: i64) {
    attributes.insert(key.to_string(), json!(value));
}

fn copy_str(attributes: &mut CanonicalAttrs, source: &Value, field: &str, key: &str) {
    if let Some(value) = source.get(field).and_then(Value::as_str)
        && !value.is_empty()
    {
        attr_str(attributes, key, value);
    }
}

fn copy_value(attributes: &mut CanonicalAttrs, source: &Value, field: &str, key: &str) {
    match source.get(field) {
        Some(Value::Null) | None => {}
        Some(value) => {
            attributes.insert(key.to_string(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../tests/fixtures/langfuse_trace_export.json");

    fn converted() -> ConvertedExport {
        let root: Value = serde_json::from_str(FIXTURE).unwrap_or_else(|e| panic!("fixture: {e}"));
        convert_export(&root).unwrap_or_else(|e| panic!("convert: {e}"))
    }

    fn draft<'a>(c: &'a ConvertedExport, span_id: &str) -> &'a CanonicalSpanDraft {
        c.drafts
            .iter()
            .find(|d| d.span_id.as_str() == span_id)
            .unwrap_or_else(|| panic!("missing span {span_id}"))
    }

    #[test]
    fn maps_trace_root_to_agent_run() {
        let c = converted();
        let root = draft(&c, "lf-trace-trace-001");
        assert_eq!(root.kind, AgentSpanKind::AgentRun);
        assert_eq!(root.parent_span_id, None);
        assert_eq!(root.name, "support-agent-run");
        assert_eq!(root.seq, 0);
        // Root start = trace timestamp; end = latest observation end (10:00:07).
        assert_eq!(
            root.start_time.map(|t| t.to_rfc3339()),
            Some("2026-06-20T10:00:00+00:00".to_string())
        );
        assert_eq!(
            root.end_time.map(|t| t.to_rfc3339()),
            Some("2026-06-20T10:00:07+00:00".to_string())
        );
        assert_eq!(
            root.attributes.get("langfuse.session_id"),
            Some(&json!("sess-42"))
        );
        assert_eq!(
            root.attributes.get("langfuse.release"),
            Some(&json!("v1.4.2"))
        );
        assert_eq!(
            root.attributes.get(attr::RELEASE_ID),
            Some(&json!("v1.4.2"))
        );
    }

    #[test]
    fn maps_generation_to_llm_call_with_model_and_tokens() {
        let c = converted();
        let generation = draft(&c, "lf-obs-obs-gen-1");
        assert_eq!(generation.kind, AgentSpanKind::LlmCall);
        let model = generation.model.as_ref().unwrap_or_else(|| panic!("model"));
        assert_eq!(model.provider, "openai");
        assert_eq!(model.name, "gpt-4o");
        let tokens = generation
            .tokens
            .as_ref()
            .unwrap_or_else(|| panic!("tokens"));
        assert_eq!(tokens.input, 120);
        assert_eq!(tokens.output, 45);
        let cost = generation.cost.as_ref().unwrap_or_else(|| panic!("cost"));
        assert_eq!(cost.amount_micros, 2400); // 0.0024 USD
        assert_eq!(
            generation.attributes.get("gen_ai.request.model"),
            Some(&json!("gpt-4o"))
        );
    }

    #[test]
    fn maps_span_to_tool_call_and_event_to_agent_step() {
        let c = converted();
        assert_eq!(
            draft(&c, "lf-obs-obs-span-retrieve").kind,
            AgentSpanKind::ToolCall
        );
        assert_eq!(
            draft(&c, "lf-obs-obs-span-weather").kind,
            AgentSpanKind::ToolCall
        );
        assert_eq!(
            draft(&c, "lf-obs-obs-event-cache").kind,
            AgentSpanKind::AgentStep
        );
    }

    #[test]
    fn preserves_parent_child_nesting() {
        let c = converted();
        // generation is nested under the retrieve span...
        assert_eq!(
            draft(&c, "lf-obs-obs-gen-1")
                .parent_span_id
                .as_ref()
                .map(|s| s.as_str()),
            Some("lf-obs-obs-span-retrieve")
        );
        // ...the event under the generation...
        assert_eq!(
            draft(&c, "lf-obs-obs-event-cache")
                .parent_span_id
                .as_ref()
                .map(|s| s.as_str()),
            Some("lf-obs-obs-gen-1")
        );
        // ...and a parent-less observation under the trace root.
        assert_eq!(
            draft(&c, "lf-obs-obs-span-weather")
                .parent_span_id
                .as_ref()
                .map(|s| s.as_str()),
            Some("lf-trace-trace-001")
        );
    }

    #[test]
    fn maps_error_level_to_error_status() {
        let c = converted();
        assert_eq!(
            draft(&c, "lf-obs-obs-span-weather").status,
            SpanStatus::Error
        );
        assert_eq!(draft(&c, "lf-obs-obs-gen-1").status, SpanStatus::Ok);
    }

    #[test]
    fn timestamps_are_parsed() {
        let c = converted();
        let generation = draft(&c, "lf-obs-obs-gen-1");
        assert_eq!(
            generation.start_time.map(|t| t.to_rfc3339()),
            Some("2026-06-20T10:00:02+00:00".to_string())
        );
        assert_eq!(
            generation.end_time.map(|t| t.to_rfc3339()),
            Some("2026-06-20T10:00:05+00:00".to_string())
        );
    }

    #[test]
    fn stats_account_for_every_observation() {
        let c = converted();
        assert_eq!(c.stats.traces, 1);
        assert_eq!(c.stats.observations, 4);
        assert_eq!(c.stats.spans, 5); // root + 4 observations
        assert_eq!(c.stats.unmapped_observation_types, 0);
    }

    #[test]
    fn raw_bytes_are_retained_losslessly() {
        let scope = TenantScope::new(
            beater_core::TenantId::new("t1").unwrap_or_else(|e| panic!("{e}")),
            beater_core::ProjectId::new("p1").unwrap_or_else(|e| panic!("{e}")),
            beater_core::EnvironmentId::new("e1").unwrap_or_else(|e| panic!("{e}")),
        );
        let original = FIXTURE.as_bytes().to_vec();
        let request = langfuse_export_to_raw_ingest(scope, original.clone(), None)
            .unwrap_or_else(|e| panic!("normalize: {e}"));
        assert_eq!(request.source, SourceDialect::LangfuseImport);
        assert_eq!(request.raw_bytes, original);
        // The retained bytes re-parse to the exact original document → re-projectable.
        let from_raw: Value =
            serde_json::from_slice(&request.raw_bytes).unwrap_or_else(|e| panic!("{e}"));
        let from_src: Value = serde_json::from_str(FIXTURE).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(from_raw, from_src);
    }

    #[test]
    fn accepts_list_and_data_envelopes() {
        let bare: Value = serde_json::from_str(FIXTURE).unwrap_or_else(|e| panic!("{e}"));
        let wrapped = json!({ "data": [bare.clone()] });
        let array = json!([bare.clone()]);
        assert_eq!(
            convert_export(&wrapped)
                .unwrap_or_else(|e| panic!("{e}"))
                .drafts
                .len(),
            5
        );
        assert_eq!(
            convert_export(&array)
                .unwrap_or_else(|e| panic!("{e}"))
                .drafts
                .len(),
            5
        );
    }

    #[test]
    fn unknown_observation_type_is_mapped_not_dropped() {
        let doc = json!({
            "id": "trace-x",
            "timestamp": "2026-06-20T10:00:00Z",
            "observations": [
                {"id": "o1", "type": "RETRIEVER", "startTime": "2026-06-20T10:00:01Z"}
            ]
        });
        let c = convert_export(&doc).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(c.stats.unmapped_observation_types, 1);
        let obs = draft(&c, "lf-obs-o1");
        assert_eq!(obs.kind, AgentSpanKind::AgentStep);
        assert_eq!(
            obs.attributes.get("langfuse.observation.type"),
            Some(&json!("RETRIEVER"))
        );
    }

    #[test]
    fn malformed_inputs_error_without_panicking() {
        // Not JSON at all.
        assert!(matches!(
            convert_export(&json!("not a trace")),
            Err(LangfuseError::MissingTrace)
        ));
        // Trace without an id.
        assert!(matches!(
            convert_export(&json!({"observations": []})),
            Err(LangfuseError::MissingTrace)
        ));
        // Observations present but not an array.
        assert!(matches!(
            convert_export(&json!({"id": "t", "observations": {}})),
            Err(LangfuseError::ObservationsNotArray)
        ));
        // Observation missing its id.
        assert!(matches!(
            convert_export(&json!({"id": "t", "observations": [{"type": "SPAN"}]})),
            Err(LangfuseError::MissingObservationId)
        ));
        // Raw bytes that are not valid JSON surface as a graceful error.
        let scope = TenantScope::new(
            beater_core::TenantId::new("t1").unwrap_or_else(|e| panic!("{e}")),
            beater_core::ProjectId::new("p1").unwrap_or_else(|e| panic!("{e}")),
            beater_core::EnvironmentId::new("e1").unwrap_or_else(|e| panic!("{e}")),
        );
        assert!(langfuse_export_to_raw_ingest(scope, b"{ broken".to_vec(), None).is_err());
    }
}
