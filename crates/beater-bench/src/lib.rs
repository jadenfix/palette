//! `beater-bench` — criterion benchmarks and load-test fixtures for Beater.
//!
//! # Purpose
//!
//! This crate is the home for all performance evidence required by the Tech Rider
//! (§23.10) and the §20.2 gap-closure plan.  Specifically it targets the §20.2 #0.3
//! acceptance criterion: *criterion benches for `write_batch` throughput and
//! `query_*` latency on seeded 1 M / 10 M / 100 M-span fixtures, meeting the §16
//! SLOs in CI*.
//!
//! Nothing in this crate is wired into `beaterd` or any runtime path.  It exists
//! solely to give future bench + load-test work a stable home with the right
//! Cargo plumbing already in place.
//!
//! # Architecture references
//!
//! * **§16** — Self-Observability SLOs (the target numbers benches gate on).
//! * **§20.2 #0.3** — "Measured query p95 SLOs" gap-closure item; the `backend`
//!   Metronome CI gate that runs `cargo bench -p beater-bench`.
//! * **§23.10** — Perf observability + SLO gates (Heartbeat + Tech Rider); describes
//!   the advisory → required promotion path for this bench gate.
//!
//! # Layout
//!
//! ```text
//! crates/beater-bench/
//!   src/lib.rs          — this file; shared helpers / load-fixture builders
//!   benches/smoke.rs    — trivial smoke bench (verifies harness compiles & links)
//!   benches/store.rs    — [planned] write_batch throughput + query_* latency benches
//!                         on in-memory / SQLite / Postgres / ClickHouse backends
//! ```
//!
//! # Adding a new bench
//!
//! 1. Add a `[[bench]]` entry in `Cargo.toml` with `harness = false`.
//! 2. Import `criterion::{criterion_group, criterion_main, Criterion}`.
//! 3. Use [`span_fixtures`] / [`canonical_trace_batch`] to get deterministic fixtures.
//! 4. Wire the SLO assertion as a `criterion` throughput target or a custom
//!    post-bench assertion so CI fails on regression.

use beater_core::{
    ArtifactId, EnvironmentId, IdempotencyKey, Money, ProjectId, Sha256Hash, SpanId, TenantId,
    Timestamp, TokenCounts, TraceId, sha256_hex,
};
use beater_schema::{
    AgentSpanKind, ArtifactRef, AuthContext, CANONICAL_SCHEMA_VERSION, CanonicalAttrs,
    CanonicalSpan, CanonicalTraceBatch, ModelRef, RAW_SCHEMA_VERSION, RawEnvelope, RedactionClass,
    SourceDialect, SpanStatus, conventions,
};
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::time::SystemTime;

const DEFAULT_SPAN_COUNT: usize = 1_024;
const DEFAULT_TRACE_GROUP_SIZE: usize = 16;
const NORMALIZER_VERSION: &str = "beater-bench-fixture-v1";
const DEFAULT_RELEASE_ID: &str = "bench-release-0001";

macro_rules! fixture_id {
    ($ty:ty, $value:expr) => {{
        match <$ty>::new($value) {
            Ok(id) => id,
            Err(err) => unreachable!("bench fixture generated invalid {}: {err}", stringify!($ty)),
        }
    }};
}

/// Configuration for deterministic [`CanonicalSpan`] benchmark fixtures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanFixtureConfig {
    /// Number of spans to generate.
    pub count: usize,
    /// Tenant carried by every generated span.
    pub tenant_id: TenantId,
    /// Project carried by every generated span.
    pub project_id: ProjectId,
    /// Environment carried by every generated span.
    pub environment_id: EnvironmentId,
    /// Number of spans per trace before the trace id advances.
    pub trace_group_size: usize,
    /// Timestamp for the first span.
    pub start_time: Timestamp,
    /// Time range covered by start timestamps.
    pub window: Duration,
    /// Duration added to each span start timestamp.
    pub span_duration: Duration,
    /// Include cost fields on LLM spans.
    pub include_costs: bool,
    /// Include token fields on LLM spans.
    pub include_tokens: bool,
}

impl Default for SpanFixtureConfig {
    fn default() -> Self {
        Self {
            count: DEFAULT_SPAN_COUNT,
            tenant_id: fixture_id!(TenantId, "bench-tenant"),
            project_id: fixture_id!(ProjectId, "bench-project"),
            environment_id: fixture_id!(EnvironmentId, "bench-prod"),
            trace_group_size: DEFAULT_TRACE_GROUP_SIZE,
            start_time: DateTime::<Utc>::from(SystemTime::UNIX_EPOCH),
            window: Duration::minutes(10),
            span_duration: Duration::milliseconds(25),
            include_costs: true,
            include_tokens: true,
        }
    }
}

impl SpanFixtureConfig {
    /// Create a fixture config with the default shape and a custom span count.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            ..Self::default()
        }
    }

    /// Return a copy of this config with a custom start-time window.
    pub fn with_window(mut self, window: Duration) -> Self {
        self.window = window;
        self
    }

    /// Return a copy of this config with a custom trace grouping size.
    pub fn with_trace_group_size(mut self, trace_group_size: usize) -> Self {
        self.trace_group_size = trace_group_size;
        self
    }

    /// Return a copy of this config with cost fields enabled or disabled.
    pub fn with_costs(mut self, include_costs: bool) -> Self {
        self.include_costs = include_costs;
        self
    }

    /// Return a copy of this config with token fields enabled or disabled.
    pub fn with_tokens(mut self, include_tokens: bool) -> Self {
        self.include_tokens = include_tokens;
        self
    }

    fn spans_per_trace(&self) -> usize {
        self.trace_group_size.max(1)
    }
}

/// Generate deterministic canonical spans for future store benchmarks.
///
/// The fixture intentionally varies status, kind, model/cost/token fields,
/// artifact refs, and attributes while keeping IDs and timestamps stable across
/// runs. It does not seed any store by itself.
pub fn span_fixtures(config: &SpanFixtureConfig) -> Vec<CanonicalSpan> {
    (0..config.count)
        .map(|index| fixture_span(config, index))
        .collect()
}

/// Generate a default-shaped deterministic span batch with `count` spans.
pub fn span_batch(count: usize) -> Vec<CanonicalSpan> {
    span_fixtures(&SpanFixtureConfig::new(count))
}

/// Generate a write-ready trace batch for future store throughput benchmarks.
///
/// Each span gets a matching raw envelope with a unique idempotency key, so a
/// benchmark can pass the returned value directly to `TraceStore::write_batch`
/// and measure raw-envelope and canonical-span insertion together.
pub fn canonical_trace_batch(config: &SpanFixtureConfig) -> CanonicalTraceBatch {
    let spans = span_fixtures(config);
    let raw_envelopes = spans
        .iter()
        .map(|span| raw_envelope_for(config, span))
        .collect();

    CanonicalTraceBatch {
        raw_envelopes,
        spans,
    }
}

/// Generate a default-shaped write-ready trace batch with `count` spans.
pub fn trace_batch(count: usize) -> CanonicalTraceBatch {
    canonical_trace_batch(&SpanFixtureConfig::new(count))
}

fn fixture_span(config: &SpanFixtureConfig, index: usize) -> CanonicalSpan {
    let spans_per_trace = config.spans_per_trace();
    let trace_index = index / spans_per_trace;
    let offset_in_trace = index % spans_per_trace;
    let trace_id = make_trace_id(trace_index);
    let span_id = make_span_id(index);
    let parent_span_id = if offset_in_trace == 0 {
        None
    } else {
        Some(make_span_id(trace_index * spans_per_trace))
    };
    let seq = index as u64;
    let kind = kind_for(offset_in_trace, index);
    let status = status_for(index);
    let start_time = timestamp_at(config, index);
    let end_time = add_duration(start_time, non_negative_duration(config.span_duration));
    let is_llm = kind == AgentSpanKind::LlmCall;
    let model = model_for(&kind, index);
    let cost = cost_for(config, &kind, index);
    let tokens = tokens_for(config, &kind, index);
    let mut attributes = attributes_for(&kind, &status, index, trace_index, &cost, &tokens);
    let attach_io = matches!(&kind, AgentSpanKind::LlmCall | AgentSpanKind::ToolCall);
    let input_ref = if attach_io {
        Some(artifact_ref(
            config, &trace_id, &span_id, "input", index, 256,
        ))
    } else {
        None
    };
    let output_ref = if attach_io {
        Some(artifact_ref(
            config, &trace_id, &span_id, "output", index, 384,
        ))
    } else {
        None
    };

    if is_llm {
        attributes.insert("benchmark.model_lane".to_string(), json!(index % 3));
    }

    CanonicalSpan {
        schema_version: CANONICAL_SCHEMA_VERSION,
        normalizer_version: NORMALIZER_VERSION.to_string(),
        tenant_id: config.tenant_id.clone(),
        project_id: config.project_id.clone(),
        environment_id: config.environment_id.clone(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        parent_span_id,
        seq,
        kind,
        name: format!("bench span {index:012}"),
        status,
        start_time,
        end_time: Some(end_time),
        model,
        cost,
        tokens,
        input_ref,
        output_ref,
        attributes,
        unmapped_attrs: Value::Object(serde_json::Map::new()),
        raw_ref: artifact_ref(config, &trace_id, &span_id, "raw", index, 512),
    }
}

fn raw_envelope_for(config: &SpanFixtureConfig, span: &CanonicalSpan) -> RawEnvelope {
    let idempotency_key = fixture_id!(IdempotencyKey, format!("bench-raw-{:012}", span.seq));

    RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: config.tenant_id.clone(),
        project_id: config.project_id.clone(),
        environment_id: config.environment_id.clone(),
        source: SourceDialect::Native,
        source_schema_url: Some("beater://bench/native/v1".to_string()),
        source_schema_version: Some("1".to_string()),
        received_at: span.start_time,
        idempotency_key,
        payload_hash: span.raw_ref.sha256.clone(),
        body_ref: span.raw_ref.clone(),
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    }
}

fn make_trace_id(index: usize) -> TraceId {
    fixture_id!(TraceId, format!("bench-trace-{index:08}"))
}

fn make_span_id(index: usize) -> SpanId {
    fixture_id!(SpanId, format!("bench-span-{index:012}"))
}

fn kind_for(offset_in_trace: usize, index: usize) -> AgentSpanKind {
    if offset_in_trace == 0 {
        return AgentSpanKind::AgentRun;
    }

    match index % 6 {
        0 => AgentSpanKind::AgentStep,
        1 => AgentSpanKind::LlmCall,
        2 => AgentSpanKind::ToolCall,
        3 => AgentSpanKind::RetrievalQuery,
        4 => AgentSpanKind::McpRequest,
        _ => AgentSpanKind::GuardrailCheck,
    }
}

fn status_for(index: usize) -> SpanStatus {
    match index % 4 {
        0 | 2 => SpanStatus::Ok,
        1 => SpanStatus::Error,
        _ => SpanStatus::Unset,
    }
}

fn timestamp_at(config: &SpanFixtureConfig, index: usize) -> Timestamp {
    let window_nanos = match config.window.num_nanoseconds() {
        Some(nanos) if nanos > 0 => nanos as u128,
        _ => 0,
    };
    let offset_nanos = if config.count <= 1 || window_nanos == 0 {
        0
    } else {
        let denominator = config.count.saturating_sub(1) as u128;
        window_nanos.saturating_mul(index as u128) / denominator
    };
    let clamped = offset_nanos.min(i64::MAX as u128) as i64;

    add_duration(config.start_time, Duration::nanoseconds(clamped))
}

fn non_negative_duration(duration: Duration) -> Duration {
    if duration < Duration::zero() {
        Duration::zero()
    } else {
        duration
    }
}

fn add_duration(timestamp: Timestamp, duration: Duration) -> Timestamp {
    match timestamp.checked_add_signed(duration) {
        Some(value) => value,
        None => timestamp,
    }
}

fn model_for(kind: &AgentSpanKind, index: usize) -> Option<ModelRef> {
    if kind != &AgentSpanKind::LlmCall {
        return None;
    }

    Some(ModelRef {
        provider: "openai".to_string(),
        name: format!("gpt-bench-{}", index % 3),
    })
}

fn cost_for(config: &SpanFixtureConfig, kind: &AgentSpanKind, index: usize) -> Option<Money> {
    if !config.include_costs || kind != &AgentSpanKind::LlmCall {
        return None;
    }

    Some(Money::usd_micros(100 + ((index % 17) as i64 * 10)))
}

fn tokens_for(
    config: &SpanFixtureConfig,
    kind: &AgentSpanKind,
    index: usize,
) -> Option<TokenCounts> {
    if !config.include_tokens || kind != &AgentSpanKind::LlmCall {
        return None;
    }

    Some(TokenCounts {
        input: 120 + (index % 31) as u64,
        output: 32 + (index % 19) as u64,
        reasoning: if index.is_multiple_of(2) { 8 } else { 0 },
        cache_read: if index.is_multiple_of(3) { 16 } else { 0 },
    })
}

fn attributes_for(
    kind: &AgentSpanKind,
    status: &SpanStatus,
    index: usize,
    trace_index: usize,
    cost: &Option<Money>,
    tokens: &Option<TokenCounts>,
) -> CanonicalAttrs {
    let mut attributes = BTreeMap::from([
        (
            conventions::attr::SPAN_KIND.to_string(),
            json!(kind.as_str()),
        ),
        (conventions::attr::SEQ.to_string(), json!(index as u64)),
        (
            conventions::attr::RELEASE_ID.to_string(),
            json!(DEFAULT_RELEASE_ID),
        ),
        ("benchmark.fixture".to_string(), json!("span-store-v1")),
        ("benchmark.bucket".to_string(), json!(index % 8)),
        ("benchmark.trace_index".to_string(), json!(trace_index)),
        ("span.status".to_string(), json!(status.as_str())),
    ]);

    match kind {
        AgentSpanKind::LlmCall => {
            attributes.insert(conventions::attr::LLM_PROVIDER.to_string(), json!("openai"));
            attributes.insert(
                conventions::attr::LLM_MODEL_NAME.to_string(),
                json!(format!("gpt-bench-{}", index % 3)),
            );
        }
        AgentSpanKind::ToolCall => {
            attributes.insert(
                "tool.name".to_string(),
                json!(format!("bench-tool-{}", index % 5)),
            );
        }
        AgentSpanKind::RetrievalQuery => {
            attributes.insert(
                "retrieval.collection".to_string(),
                json!(format!("kb-{}", index % 4)),
            );
        }
        AgentSpanKind::McpRequest => {
            attributes.insert(
                "mcp.tool.name".to_string(),
                json!(format!("mcp-tool-{}", index % 3)),
            );
        }
        AgentSpanKind::GuardrailCheck => {
            attributes.insert("guardrail.policy".to_string(), json!("bench-policy"));
        }
        _ => {}
    }

    if status == &SpanStatus::Error {
        attributes.insert("error.type".to_string(), json!("bench_fixture_error"));
    }

    if let Some(cost) = cost {
        attributes.insert(
            conventions::attr::LLM_COST_MICROS.to_string(),
            json!(cost.amount_micros),
        );
        attributes.insert(
            conventions::attr::LLM_COST_CURRENCY.to_string(),
            json!(cost.currency.as_str()),
        );
    }

    if let Some(tokens) = tokens {
        attributes.insert(
            conventions::attr::LLM_TOKEN_PROMPT.to_string(),
            json!(tokens.input),
        );
        attributes.insert(
            conventions::attr::LLM_TOKEN_COMPLETION.to_string(),
            json!(tokens.output),
        );
        attributes.insert(
            conventions::attr::LLM_TOKEN_REASONING.to_string(),
            json!(tokens.reasoning),
        );
        attributes.insert(
            conventions::attr::LLM_TOKEN_CACHE_READ.to_string(),
            json!(tokens.cache_read),
        );
    }

    attributes
}

fn artifact_ref(
    config: &SpanFixtureConfig,
    trace_id: &TraceId,
    span_id: &SpanId,
    role: &str,
    index: usize,
    size_bytes: u64,
) -> ArtifactRef {
    let artifact_id = fixture_id!(ArtifactId, format!("bench-artifact-{role}-{index:012}"));
    let hash_input = format!(
        "{}:{}:{}:{}:{}",
        config.tenant_id.as_str(),
        config.project_id.as_str(),
        trace_id.as_str(),
        span_id.as_str(),
        role
    );
    let sha256 = fixture_id!(Sha256Hash, sha256_hex(hash_input.as_bytes()));

    ArtifactRef {
        artifact_id,
        uri: format!(
            "artifact://{}/{}/{}/{}/{}",
            config.tenant_id.as_str(),
            config.project_id.as_str(),
            trace_id.as_str(),
            span_id.as_str(),
            role
        ),
        sha256,
        size_bytes,
        mime_type: "application/json".to_string(),
        redaction_class: RedactionClass::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_batch_uses_requested_count() {
        assert_eq!(span_batch(0).len(), 0);
        assert_eq!(span_batch(3).len(), 3);
    }

    #[test]
    fn trace_batch_pairs_raw_envelopes_with_spans() {
        let batch = trace_batch(5);

        assert_eq!(batch.raw_envelopes.len(), 5);
        assert_eq!(batch.spans.len(), 5);

        for (index, (raw, span)) in batch.raw_envelopes.iter().zip(&batch.spans).enumerate() {
            assert_eq!(raw.schema_version, RAW_SCHEMA_VERSION);
            assert_eq!(raw.tenant_id, span.tenant_id);
            assert_eq!(raw.project_id, span.project_id);
            assert_eq!(raw.environment_id, span.environment_id);
            assert_eq!(raw.source, SourceDialect::Native);
            assert_eq!(raw.received_at, span.start_time);
            assert_eq!(raw.payload_hash, span.raw_ref.sha256);
            assert_eq!(raw.body_ref, span.raw_ref);
            assert_eq!(
                raw.idempotency_key.as_str(),
                format!("bench-raw-{index:012}")
            );
            assert!(raw.auth_context.scopes.is_empty());
        }
    }

    #[test]
    fn fixtures_have_deterministic_ids_and_timestamps() {
        let config = SpanFixtureConfig::new(3)
            .with_trace_group_size(2)
            .with_window(Duration::seconds(2));

        let first = span_fixtures(&config);
        let second = span_fixtures(&config);

        assert_eq!(first, second);
        assert_eq!(first[0].tenant_id.as_str(), "bench-tenant");
        assert_eq!(first[0].project_id.as_str(), "bench-project");
        assert_eq!(first[0].environment_id.as_str(), "bench-prod");
        assert_eq!(first[0].trace_id.as_str(), "bench-trace-00000000");
        assert_eq!(first[0].span_id.as_str(), "bench-span-000000000000");
        assert_eq!(
            first[1].parent_span_id.as_ref().map(SpanId::as_str),
            Some("bench-span-000000000000")
        );
        assert_eq!(first[2].trace_id.as_str(), "bench-trace-00000001");
        assert_eq!(first[0].start_time, config.start_time);
        assert_eq!(
            first[1].start_time,
            config.start_time + Duration::seconds(1)
        );
        assert_eq!(
            first[2].start_time,
            config.start_time + Duration::seconds(2)
        );
    }

    #[test]
    fn fixtures_mix_queryable_fields() {
        let spans = span_fixtures(&SpanFixtureConfig::new(8).with_trace_group_size(4));

        assert!(
            spans
                .iter()
                .any(|span| span.kind == AgentSpanKind::AgentRun)
        );
        assert!(spans.iter().any(|span| span.kind == AgentSpanKind::LlmCall));
        assert!(
            spans
                .iter()
                .any(|span| span.kind == AgentSpanKind::ToolCall)
        );
        assert!(spans.iter().any(|span| span.status == SpanStatus::Ok));
        assert!(spans.iter().any(|span| span.status == SpanStatus::Error));
        assert!(spans.iter().any(|span| span.status == SpanStatus::Unset));
        assert!(spans.iter().any(|span| span.cost.is_some()));
        assert!(spans.iter().any(|span| span.tokens.is_some()));
        assert!(spans.iter().any(|span| span.cost.is_none()));
        assert!(spans.iter().any(|span| span.input_ref.is_some()));
        assert!(spans.iter().any(|span| span.output_ref.is_some()));
        assert!(
            spans
                .iter()
                .all(|span| span.raw_ref.uri.starts_with("artifact://bench-tenant/"))
        );
        assert!(
            spans
                .iter()
                .any(|span| span.attributes.contains_key("benchmark.bucket"))
        );
        assert!(spans.iter().any(|span| {
            span.attributes
                .contains_key(conventions::attr::LLM_TOKEN_PROMPT)
        }));
    }
}
