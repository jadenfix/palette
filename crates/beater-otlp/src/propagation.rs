//! W3C distributed-context propagation for Beater's async plumbing.
//!
//! Beater hands a trace across process boundaries it does not control with OTLP,
//! but *inside* the service the same logical trace also flows through the durable
//! bus/queue and across `tokio::spawn` boundaries. Tonic/OTLP do not carry that
//! for us on those internal hops, so this module implements the two relevant W3C
//! specifications by hand (no extra dependency, matching the rest of this crate):
//!
//! - **W3C Trace Context** (`traceparent` / `tracestate`) — extracted from and
//!   injected into a string carrier so a queued message or a spawned task keeps
//!   the same trace id and references the right parent span.
//! - **W3C Baggage** (`baggage`) — parses tenant/project/release context that
//!   rides alongside the trace, redacting values flagged as sensitive so secrets
//!   never end up in a propagated header or a stored span attribute.
//!
//! A "carrier" is anything string-keyed: HTTP headers, gRPC metadata, or the
//! `BTreeMap<String, String>` we attach to bus messages and task handles. The
//! [`Carrier`] / [`CarrierMut`] traits keep the inject/extract logic transport
//! agnostic, and [`spawn_with_context`] is the convenience wrapper that captures
//! the current context and re-establishes it inside a spawned future.

use std::collections::BTreeMap;
use std::future::Future;

/// Header/field name for the W3C `traceparent`.
pub const TRACEPARENT_HEADER: &str = "traceparent";
/// Header/field name for the W3C `tracestate`.
pub const TRACESTATE_HEADER: &str = "tracestate";
/// Header/field name for W3C `baggage`.
pub const BAGGAGE_HEADER: &str = "baggage";

const TRACE_ID_HEX_LEN: usize = 32;
const SPAN_ID_HEX_LEN: usize = 16;

/// A read-only string carrier (HTTP headers, gRPC metadata, a map, …).
pub trait Carrier {
    fn get(&self, key: &str) -> Option<&str>;
}

/// A writable string carrier used when injecting context.
pub trait CarrierMut {
    fn set(&mut self, key: &str, value: String);
}

impl Carrier for BTreeMap<String, String> {
    fn get(&self, key: &str) -> Option<&str> {
        BTreeMap::get(self, key).map(String::as_str)
    }
}

impl CarrierMut for BTreeMap<String, String> {
    fn set(&mut self, key: &str, value: String) {
        BTreeMap::insert(self, key.to_string(), value);
    }
}

/// A parsed, validated W3C trace context: the `traceparent` fields plus the
/// optional opaque `tracestate`.
///
/// `trace_id` (16 bytes / 32 hex) and `span_id` (8 bytes / 16 hex) are stored as
/// lowercase hex. The 8-bit `trace_flags` carries the sampled bit (`0x01`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceContext {
    trace_id: String,
    span_id: String,
    trace_flags: u8,
    tracestate: Option<String>,
}

impl TraceContext {
    /// Build a context from already-validated lowercase-hex ids. Returns `None`
    /// if either id has the wrong length or is all-zero (the W3C "invalid" value).
    pub fn new(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        sampled: bool,
    ) -> Option<Self> {
        let trace_id = normalize_hex(&trace_id.into(), TRACE_ID_HEX_LEN)?;
        let span_id = normalize_hex(&span_id.into(), SPAN_ID_HEX_LEN)?;
        Some(Self {
            trace_id,
            span_id,
            trace_flags: if sampled { 0x01 } else { 0x00 },
            tracestate: None,
        })
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    pub fn is_sampled(&self) -> bool {
        self.trace_flags & 0x01 == 0x01
    }

    pub fn tracestate(&self) -> Option<&str> {
        self.tracestate.as_deref()
    }

    /// Attach an opaque, already-formatted `tracestate` value (multi-vendor list).
    pub fn with_tracestate(mut self, tracestate: impl Into<String>) -> Self {
        let value = tracestate.into();
        self.tracestate = (!value.trim().is_empty()).then_some(value);
        self
    }

    /// Derive the child context a downstream hop should adopt: the same trace id
    /// and tracestate, but the given `child_span_id` becomes the new parent so the
    /// next span links to the work that enqueued/spawned it.
    pub fn child(&self, child_span_id: impl Into<String>) -> Option<Self> {
        let span_id = normalize_hex(&child_span_id.into(), SPAN_ID_HEX_LEN)?;
        Some(Self {
            trace_id: self.trace_id.clone(),
            span_id,
            trace_flags: self.trace_flags,
            tracestate: self.tracestate.clone(),
        })
    }

    /// Render the `traceparent` header value (version `00`).
    pub fn traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Parse a `traceparent` string per W3C version `00`. Unknown future versions
    /// are accepted as long as the first four fields parse (forward compatible),
    /// but a malformed/short value or an all-zero id is rejected.
    pub fn parse_traceparent(value: &str) -> Option<Self> {
        let mut parts = value.trim().split('-');
        let version = parts.next()?;
        if version.len() != 2 || !is_hex(version) {
            return None;
        }
        // Version ff is forbidden by the spec.
        if version == "ff" {
            return None;
        }
        let trace_id = normalize_hex(parts.next()?, TRACE_ID_HEX_LEN)?;
        let span_id = normalize_hex(parts.next()?, SPAN_ID_HEX_LEN)?;
        let flags_field = parts.next()?;
        if flags_field.len() != 2 || !is_hex(flags_field) {
            return None;
        }
        let trace_flags = u8::from_str_radix(flags_field, 16).ok()?;
        Some(Self {
            trace_id,
            span_id,
            trace_flags,
            tracestate: None,
        })
    }

    /// Extract a trace context from any carrier (headers, metadata, a map),
    /// folding in `tracestate` when present.
    pub fn extract<C: Carrier + ?Sized>(carrier: &C) -> Option<Self> {
        let mut context = Self::parse_traceparent(carrier.get(TRACEPARENT_HEADER)?)?;
        if let Some(state) = carrier.get(TRACESTATE_HEADER) {
            context = context.with_tracestate(state.to_string());
        }
        Some(context)
    }

    /// Inject this context into a writable carrier, writing both `traceparent`
    /// and (if present) `tracestate`.
    pub fn inject<C: CarrierMut + ?Sized>(&self, carrier: &mut C) {
        carrier.set(TRACEPARENT_HEADER, self.traceparent());
        if let Some(state) = &self.tracestate {
            carrier.set(TRACESTATE_HEADER, state.clone());
        }
    }
}

/// Capture the trace context from a source carrier and inject it into a fresh
/// `BTreeMap` carrier — the shape attached to a bus message or a spawned task.
/// Returns an empty map when the source has no valid context, so callers can
/// always attach the result unconditionally.
pub fn carrier_from<C: Carrier + ?Sized>(source: &C) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(context) = TraceContext::extract(source) {
        context.inject(&mut out);
    }
    if let Some(baggage) = source.get(BAGGAGE_HEADER) {
        out.insert(BAGGAGE_HEADER.to_string(), baggage.to_string());
    }
    out
}

/// Spawn `future` on the Tokio runtime with `context` propagated into it.
///
/// OTLP carries trace context across the network boundary, but a `tokio::spawn`
/// detaches from the caller's scope, so the context must be moved explicitly. The
/// spawned future receives the parent context as its first argument and is
/// expected to open its child span against it, keeping the queued/async work on
/// the same trace as the request that scheduled it.
pub fn spawn_with_context<F, Fut, T>(
    context: Option<TraceContext>,
    future: F,
) -> tokio::task::JoinHandle<T>
where
    F: FnOnce(Option<TraceContext>) -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move { future(context).await })
}

// ---- W3C Baggage ------------------------------------------------------------

/// Keys whose baggage values Beater treats as sensitive and never propagates in
/// the clear. Matched case-insensitively against the key and against any
/// `key.suffix` segment so `x-tenant.api_key` is caught as well.
const SENSITIVE_BAGGAGE_KEYS: &[&str] = &[
    "secret",
    "api_key",
    "apikey",
    "token",
    "password",
    "authorization",
    "signing_secret",
];

/// Placeholder substituted for a redacted baggage value.
pub const REDACTED_BAGGAGE_VALUE: &str = "[REDACTED]";

/// W3C baggage carrying Beater's tenant/project/release context.
///
/// Parsed from the `baggage` header (`key=value,key2=value2`), with sensitive
/// entries redacted on the way in so a secret accidentally placed in baggage is
/// never stored as a span attribute nor re-emitted downstream. `tenant`,
/// `project`, and `release` are surfaced as typed accessors because they are the
/// fields Beater routes and bills on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Baggage {
    entries: BTreeMap<String, String>,
    redacted_keys: Vec<String>,
}

impl Baggage {
    /// Parse a W3C `baggage` header value, redacting sensitive entries.
    pub fn parse(value: &str) -> Self {
        let mut entries = BTreeMap::new();
        let mut redacted_keys = Vec::new();
        for member in value.split(',') {
            let member = member.trim();
            if member.is_empty() {
                continue;
            }
            // Drop any `;`-delimited member properties; we only keep the value.
            let pair = member.split(';').next().unwrap_or(member);
            let Some((raw_key, raw_value)) = pair.split_once('=') else {
                continue;
            };
            let key = raw_key.trim();
            if key.is_empty() {
                continue;
            }
            let value = raw_value.trim();
            if is_sensitive_baggage_key(key) {
                entries.insert(key.to_string(), REDACTED_BAGGAGE_VALUE.to_string());
                redacted_keys.push(key.to_string());
            } else {
                entries.insert(key.to_string(), value.to_string());
            }
        }
        redacted_keys.sort();
        redacted_keys.dedup();
        Self {
            entries,
            redacted_keys,
        }
    }

    /// Extract baggage from any carrier; an absent header yields empty baggage.
    pub fn extract<C: Carrier + ?Sized>(carrier: &C) -> Self {
        carrier
            .get(BAGGAGE_HEADER)
            .map(Self::parse)
            .unwrap_or_default()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub fn tenant(&self) -> Option<&str> {
        self.get("tenant")
    }

    pub fn project(&self) -> Option<&str> {
        self.get("project")
    }

    pub fn release(&self) -> Option<&str> {
        self.get("release")
    }

    /// Keys whose values were redacted because they were flagged sensitive.
    pub fn redacted_keys(&self) -> &[String] {
        &self.redacted_keys
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Render back to a W3C `baggage` header value (sorted, redactions applied).
    /// Safe to inject downstream: sensitive values were already replaced.
    pub fn to_header(&self) -> String {
        self.entries
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn is_sensitive_baggage_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SENSITIVE_BAGGAGE_KEYS.iter().any(|needle| {
        // Match the whole key, or any dot-delimited segment of it, so both
        // `api_key` and a namespaced `x-tenant.api_key` are redacted while a
        // segment like `api_key` is not torn apart on its own underscore.
        lower == *needle || lower.split('.').any(|segment| segment == *needle)
    })
}

fn is_hex(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Lowercase + validate a hex id of exactly `expected_len` chars, rejecting the
/// all-zero "invalid" value mandated by the W3C spec.
fn normalize_hex(value: &str, expected_len: usize) -> Option<String> {
    let value = value.trim();
    if value.len() != expected_len || !is_hex(value) {
        return None;
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.bytes().all(|b| b == b'0') {
        return None;
    }
    Some(lowered)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
    const SPAN_ID: &str = "b7ad6b7169203331";

    #[test]
    fn traceparent_round_trips_through_a_carrier() {
        let context = TraceContext::new(TRACE_ID, SPAN_ID, true)
            .unwrap_or_else(|| panic!("valid context"))
            .with_tracestate("vendor=opaque");
        let mut carrier: BTreeMap<String, String> = BTreeMap::new();
        context.inject(&mut carrier);

        assert_eq!(
            carrier.get(TRACEPARENT_HEADER).map(String::as_str),
            Some("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
        );
        assert_eq!(
            carrier.get(TRACESTATE_HEADER).map(String::as_str),
            Some("vendor=opaque")
        );

        let extracted = TraceContext::extract(&carrier).unwrap_or_else(|| panic!("extract"));
        assert_eq!(extracted, context);
        assert!(extracted.is_sampled());
        assert_eq!(extracted.tracestate(), Some("vendor=opaque"));
    }

    #[test]
    fn malformed_or_invalid_traceparent_is_rejected() {
        // Wrong field count, bad hex, wrong lengths, all-zero ids, forbidden ff.
        for bad in [
            "",
            "00-0af7651916cd43dd8448eb211c80319c",
            "00-not-hex-aaaaaaaaaaaaaaaa-01",
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b71-01",
            "00-00000000000000000000000000000000-b7ad6b7169203331-01",
            "00-0af7651916cd43dd8448eb211c80319c-0000000000000000-01",
            "ff-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
        ] {
            assert!(
                TraceContext::parse_traceparent(bad).is_none(),
                "{bad:?} should be rejected"
            );
        }
        // No traceparent in the carrier -> no context.
        let empty: BTreeMap<String, String> = BTreeMap::new();
        assert!(TraceContext::extract(&empty).is_none());
    }

    #[test]
    fn child_context_keeps_trace_and_swaps_parent_span() {
        let parent = TraceContext::new(TRACE_ID, SPAN_ID, true)
            .unwrap_or_else(|| panic!("valid"))
            .with_tracestate("vendor=opaque");
        let child = parent
            .child("00f067aa0ba902b7")
            .unwrap_or_else(|| panic!("child"));
        assert_eq!(child.trace_id(), parent.trace_id());
        assert_eq!(child.span_id(), "00f067aa0ba902b7");
        assert_eq!(child.tracestate(), Some("vendor=opaque"));
        assert!(child.is_sampled());
    }

    #[tokio::test]
    async fn context_propagates_across_a_bus_hop_and_a_spawned_task() {
        // Producer side: inject the request's context onto the message carrier.
        let producer_headers: BTreeMap<String, String> = BTreeMap::from([
            (
                TRACEPARENT_HEADER.to_string(),
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string(),
            ),
            (TRACESTATE_HEADER.to_string(), "vendor=opaque".to_string()),
            (
                BAGGAGE_HEADER.to_string(),
                "tenant=acme,project=checkout".to_string(),
            ),
        ]);
        // This is what we'd persist alongside a BusMessage (a string map), without
        // touching the BusMessage contract.
        let message_carrier = carrier_from(&producer_headers);
        assert_eq!(
            message_carrier.get(TRACEPARENT_HEADER).map(String::as_str),
            Some("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
        );

        // Consumer side: a worker pulls the message and spawns async work. The
        // context must survive the spawn boundary and reference the same trace.
        let consumed = TraceContext::extract(&message_carrier).unwrap_or_else(|| panic!("extract"));
        let child = consumed
            .child("00f067aa0ba902b7")
            .unwrap_or_else(|| panic!("child"));

        let handle = spawn_with_context(Some(child.clone()), |ctx| async move {
            let ctx = ctx.unwrap_or_else(|| panic!("context propagated into task"));
            // The spawned task is on the same trace as the producer.
            ctx.trace_id().to_string()
        });
        let trace_in_task = handle.await.unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(trace_in_task, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(trace_in_task, consumed.trace_id());
        assert_eq!(child.span_id(), "00f067aa0ba902b7");
    }

    #[test]
    fn baggage_parses_tenant_project_release_and_redacts_secrets() {
        let baggage = Baggage::parse(
            "tenant=acme, project=checkout ,release=2026.06.23, api_key=sk-leak;meta=1, token=xyz",
        );
        assert_eq!(baggage.tenant(), Some("acme"));
        assert_eq!(baggage.project(), Some("checkout"));
        assert_eq!(baggage.release(), Some("2026.06.23"));

        // Sensitive entries are present but redacted, never carrying the raw value.
        assert_eq!(baggage.get("api_key"), Some(REDACTED_BAGGAGE_VALUE));
        assert_eq!(baggage.get("token"), Some(REDACTED_BAGGAGE_VALUE));
        assert_eq!(
            baggage.redacted_keys(),
            &["api_key".to_string(), "token".to_string()]
        );

        let header = baggage.to_header();
        assert!(!header.contains("sk-leak"));
        assert!(!header.contains("xyz"));
        assert!(header.contains("tenant=acme"));
        assert!(header.contains("api_key=[REDACTED]"));
    }

    #[test]
    fn baggage_redacts_dotted_sensitive_segments_and_handles_empty() {
        let baggage = Baggage::parse("x-tenant.api_key=sk-leak,release=v1");
        assert_eq!(
            baggage.get("x-tenant.api_key"),
            Some(REDACTED_BAGGAGE_VALUE)
        );
        assert_eq!(baggage.release(), Some("v1"));

        let empty = Baggage::extract(&BTreeMap::<String, String>::new());
        assert!(empty.is_empty());
        assert!(empty.tenant().is_none());
    }
}
