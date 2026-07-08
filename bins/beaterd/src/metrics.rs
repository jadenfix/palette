//! Self-observability metrics facade for `beaterd`.
//!
//! This is a small, dependency-free Prometheus exposition layer. It deliberately
//! does NOT pull in an external metrics/OTLP SDK: it keeps the self-observability
//! foundation additive and localized to the binary that owns process init, and it
//! renders the standard Prometheus text format (v0.0.4) consumed by any scraper.
//!
//! The facade backs the requirement track `self-observability-metrics`:
//!
//! * R13.1 — [`init_observability`] installs the process-wide [`MetricsRegistry`]
//!   plus a structured log writer (a `tracing_subscriber`-style stderr layer that
//!   stays dependency-free).
//! * R13.2 — ingest-to-queryable lag histogram ([`Metrics::observe_ingest_lag`]).
//! * R13.3 — write success rate counters ([`Metrics::record_write`]).
//! * R13.4 — eval queue depth / age gauges ([`Metrics::set_eval_queue`]).
//! * R13.5 — query latency histogram fed by an axum middleware
//!   ([`Metrics::observe_query_latency`]).
//! * R13.6 — DLQ age / count gauges ([`Metrics::set_dlq`]).
//! * R13.7 — normalizer failures by dialect/version
//!   ([`Metrics::record_normalizer_failure`]).
//! * R13.8 — per-lane / per-tenant queue lag with cardinality-safe labels
//!   ([`Metrics::set_queue_lag`]).
//! * R13.9 — object-store read/write failures
//!   ([`Metrics::record_object_store_op`]).
//!
//! All label values are passed through [`safe_label`], which clamps cardinality by
//! length and by a per-family series cap, so a hostile or unbounded attribute
//! (e.g. a raw trace id) can never explode the time-series count.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

/// Default histogram buckets in seconds, covering sub-millisecond to ~30s.
pub const DEFAULT_SECONDS_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
];

/// Maximum number of distinct label-sets retained per metric family. Beyond this
/// limit new label-sets collapse into a single `overflow="true"` series, so a
/// runaway label (a bug, or an attacker) cannot exhaust memory or the scraper.
const MAX_SERIES_PER_FAMILY: usize = 2048;

/// Maximum length of any single label value before truncation.
const MAX_LABEL_VALUE_LEN: usize = 64;

type Labels = BTreeMap<&'static str, String>;

fn labels(pairs: &[(&'static str, &str)]) -> Labels {
    pairs.iter().map(|(k, v)| (*k, safe_label(v))).collect()
}

/// Clamp a label value to a cardinality-safe, escaped form.
///
/// Empty values become `unknown`; long values are truncated; characters that
/// would break Prometheus exposition (`\`, `"`, newline) are escaped.
pub fn safe_label(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }
    let mut out = String::with_capacity(trimmed.len().min(MAX_LABEL_VALUE_LEN));
    for ch in trimmed.chars().take(MAX_LABEL_VALUE_LEN) {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

#[derive(Default)]
struct CounterFamily {
    series: Mutex<BTreeMap<Labels, AtomicU64>>,
}

impl CounterFamily {
    fn incr(&self, labels: Labels, by: u64) {
        let mut guard = self.series.lock().unwrap_or_else(|e| e.into_inner());
        if !guard.contains_key(&labels) && guard.len() >= MAX_SERIES_PER_FAMILY {
            let overflow: Labels = [("overflow", "true".to_string())].into_iter().collect();
            guard
                .entry(overflow)
                .or_default()
                .fetch_add(by, Ordering::Relaxed);
            return;
        }
        guard
            .entry(labels)
            .or_default()
            .fetch_add(by, Ordering::Relaxed);
    }

    fn snapshot(&self) -> Vec<(Labels, u64)> {
        let guard = self.series.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .iter()
            .map(|(l, v)| (l.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }
}

#[derive(Default)]
struct GaugeFamily {
    // Gauges store f64 bits in a u64 per series; the map mutex guards insertion.
    series: Mutex<BTreeMap<Labels, AtomicU64>>,
}

impl GaugeFamily {
    fn set(&self, labels: Labels, value: f64) {
        let mut guard = self.series.lock().unwrap_or_else(|e| e.into_inner());
        if !guard.contains_key(&labels) && guard.len() >= MAX_SERIES_PER_FAMILY {
            return;
        }
        guard
            .entry(labels)
            .or_default()
            .store(value.to_bits(), Ordering::Relaxed);
    }

    fn snapshot(&self) -> Vec<(Labels, f64)> {
        let guard = self.series.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .iter()
            .map(|(l, v)| (l.clone(), f64::from_bits(v.load(Ordering::Relaxed))))
            .collect()
    }
}

struct HistogramSeries {
    bucket_counts: Vec<AtomicU64>,
    sum: Mutex<f64>,
    count: AtomicU64,
}

impl HistogramSeries {
    fn new(buckets: usize) -> Self {
        Self {
            bucket_counts: (0..buckets).map(|_| AtomicU64::new(0)).collect(),
            sum: Mutex::new(0.0),
            count: AtomicU64::new(0),
        }
    }
}

struct HistogramFamily {
    buckets: Vec<f64>,
    series: Mutex<BTreeMap<Labels, Arc<HistogramSeries>>>,
}

type HistogramBucketSnapshot = Vec<(f64, u64)>;
type HistogramSeriesSnapshot = (Labels, HistogramBucketSnapshot, f64, u64);

impl HistogramFamily {
    fn new(buckets: Vec<f64>) -> Self {
        Self {
            buckets,
            series: Mutex::new(BTreeMap::new()),
        }
    }

    fn observe(&self, labels: Labels, value: f64) {
        let series = {
            let mut guard = self.series.lock().unwrap_or_else(|e| e.into_inner());
            if !guard.contains_key(&labels) && guard.len() >= MAX_SERIES_PER_FAMILY {
                return;
            }
            guard
                .entry(labels)
                .or_insert_with(|| Arc::new(HistogramSeries::new(self.buckets.len())))
                .clone()
        };
        for (idx, bound) in self.buckets.iter().enumerate() {
            if value <= *bound {
                series.bucket_counts[idx].fetch_add(1, Ordering::Relaxed);
            }
        }
        series.count.fetch_add(1, Ordering::Relaxed);
        let mut sum = series.sum.lock().unwrap_or_else(|e| e.into_inner());
        *sum += value;
    }

    fn snapshot(&self) -> Vec<HistogramSeriesSnapshot> {
        let guard = self.series.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .iter()
            .map(|(l, s)| {
                let buckets = self
                    .buckets
                    .iter()
                    .enumerate()
                    .map(|(idx, bound)| (*bound, s.bucket_counts[idx].load(Ordering::Relaxed)))
                    .collect();
                let sum = *s.sum.lock().unwrap_or_else(|e| e.into_inner());
                let count = s.count.load(Ordering::Relaxed);
                (l.clone(), buckets, sum, count)
            })
            .collect()
    }
}

/// A registered metric family with its Prometheus metadata.
struct Family {
    help: &'static str,
    kind: FamilyKind,
}

enum FamilyKind {
    Counter(CounterFamily),
    Gauge(GaugeFamily),
    Histogram(HistogramFamily),
}

/// Process-wide metrics registry.
pub struct MetricsRegistry {
    families: BTreeMap<&'static str, Family>,
}

impl MetricsRegistry {
    fn new() -> Self {
        let mut families = BTreeMap::new();

        // R13.3 — write success rate.
        families.insert(
            "beater_trace_writes_total",
            Family {
                help: "Trace store write attempts by result (success|failure).",
                kind: FamilyKind::Counter(CounterFamily::default()),
            },
        );
        // R13.7 — normalizer failures by dialect/version.
        families.insert(
            "beater_normalizer_failures_total",
            Family {
                help: "Normalizer failures by source dialect and version.",
                kind: FamilyKind::Counter(CounterFamily::default()),
            },
        );
        // R13.9 — object-store read/write failures.
        families.insert(
            "beater_object_store_ops_total",
            Family {
                help: "Object-store operations by op (read|write) and result.",
                kind: FamilyKind::Counter(CounterFamily::default()),
            },
        );
        // R13.4 — eval queue depth/age.
        families.insert(
            "beater_eval_queue_depth",
            Family {
                help: "Eval queue depth (pending evaluation cases).",
                kind: FamilyKind::Gauge(GaugeFamily::default()),
            },
        );
        families.insert(
            "beater_eval_queue_oldest_age_seconds",
            Family {
                help: "DLQ-DERIVED: age in seconds of the oldest dead-lettered eval-lane \
                       message. This is NOT live-backlog age of non-failed pending items \
                       (the bus exposes no live peek), so a growing healthy backlog reads 0.",
                kind: FamilyKind::Gauge(GaugeFamily::default()),
            },
        );
        // R13.6 — DLQ age/count. The depth gauge reflects the WHOLE deployment
        // (unfiltered global DLQ), not a single tenant; see the sampler.
        families.insert(
            "beater_dlq_depth",
            Family {
                help: "Dead-letter queue depth by lane, across all tenants (global DLQ).",
                kind: FamilyKind::Gauge(GaugeFamily::default()),
            },
        );
        families.insert(
            "beater_dlq_oldest_age_seconds",
            Family {
                help: "Age in seconds of the oldest dead-lettered message by lane, across all \
                       tenants (global DLQ).",
                kind: FamilyKind::Gauge(GaugeFamily::default()),
            },
        );
        // R13.8 — per-lane/per-tenant queue lag (cardinality-safe labels).
        families.insert(
            "beater_queue_lag_seconds",
            Family {
                help: "DLQ-DERIVED queue lag in seconds by lane and tenant (cardinality-safe). \
                       Measured from the oldest dead-lettered message's enqueue time, NOT from \
                       a live backlog peek, so a growing non-failed backlog reads 0.",
                kind: FamilyKind::Gauge(GaugeFamily::default()),
            },
        );
        // R13.2 — ingest-to-queryable lag.
        families.insert(
            "beater_ingest_to_queryable_lag_seconds",
            Family {
                help: "Seconds from ingest acceptance to a trace becoming queryable.",
                kind: FamilyKind::Histogram(HistogramFamily::new(DEFAULT_SECONDS_BUCKETS.to_vec())),
            },
        );
        // R13.5 — query latency (drives p95 in the scraper / recording rules).
        families.insert(
            "beater_http_request_duration_seconds",
            Family {
                help: "HTTP request latency in seconds by route and method.",
                kind: FamilyKind::Histogram(HistogramFamily::new(DEFAULT_SECONDS_BUCKETS.to_vec())),
            },
        );

        Self { families }
    }

    fn counter(&self, name: &'static str) -> Option<&CounterFamily> {
        match self.families.get(name).map(|f| &f.kind) {
            Some(FamilyKind::Counter(c)) => Some(c),
            _ => None,
        }
    }

    fn gauge(&self, name: &'static str) -> Option<&GaugeFamily> {
        match self.families.get(name).map(|f| &f.kind) {
            Some(FamilyKind::Gauge(g)) => Some(g),
            _ => None,
        }
    }

    fn histogram(&self, name: &'static str) -> Option<&HistogramFamily> {
        match self.families.get(name).map(|f| &f.kind) {
            Some(FamilyKind::Histogram(h)) => Some(h),
            _ => None,
        }
    }

    /// Render the registry as Prometheus text exposition format (v0.0.4).
    pub fn render(&self) -> String {
        let mut out = String::new();
        for (name, family) in &self.families {
            match &family.kind {
                FamilyKind::Counter(c) => {
                    render_help_type(&mut out, name, family.help, "counter");
                    for (labels, value) in c.snapshot() {
                        let _ = writeln!(out, "{}{} {}", name, render_labels(&labels), value);
                    }
                }
                FamilyKind::Gauge(g) => {
                    render_help_type(&mut out, name, family.help, "gauge");
                    for (labels, value) in g.snapshot() {
                        let _ =
                            writeln!(out, "{}{} {}", name, render_labels(&labels), fmt_f64(value));
                    }
                }
                FamilyKind::Histogram(h) => {
                    render_help_type(&mut out, name, family.help, "histogram");
                    for (labels, buckets, sum, count) in h.snapshot() {
                        for (bound, cumulative) in &buckets {
                            let bucket_labels = with_le(&labels, &fmt_f64(*bound));
                            let _ = writeln!(
                                out,
                                "{}_bucket{} {}",
                                name,
                                render_labels(&bucket_labels),
                                cumulative
                            );
                        }
                        let inf_labels = with_le(&labels, "+Inf");
                        let _ = writeln!(
                            out,
                            "{}_bucket{} {}",
                            name,
                            render_labels(&inf_labels),
                            count
                        );
                        let _ = writeln!(
                            out,
                            "{}_sum{} {}",
                            name,
                            render_labels(&labels),
                            fmt_f64(sum)
                        );
                        let _ = writeln!(out, "{}_count{} {}", name, render_labels(&labels), count);
                    }
                }
            }
        }
        out
    }
}

fn render_help_type(out: &mut String, name: &str, help: &str, kind: &str) {
    let _ = writeln!(out, "# HELP {name} {help}");
    let _ = writeln!(out, "# TYPE {name} {kind}");
}

fn render_labels(labels: &Labels) -> String {
    if labels.is_empty() {
        return String::new();
    }
    let mut out = String::from("{");
    for (idx, (k, v)) in labels.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let _ = write!(out, "{k}=\"{v}\"");
    }
    out.push('}');
    out
}

fn with_le(labels: &Labels, le: &str) -> Labels {
    let mut next = labels.clone();
    next.insert("le", le.to_string());
    next
}

fn fmt_f64(value: f64) -> String {
    if value == f64::INFINITY {
        "+Inf".to_string()
    } else if value == value.trunc() && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// Result of a fallible operation, for success-rate counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpResult {
    Success,
    Failure,
}

impl OpResult {
    fn as_str(self) -> &'static str {
        match self {
            OpResult::Success => "success",
            OpResult::Failure => "failure",
        }
    }
}

/// Object-store operation kind (R13.9).
///
/// Only the operations `ArtifactStore` actually performs are represented. There
/// is intentionally no `Delete` variant: `ArtifactStore` has no delete operation,
/// so a `delete` label would be a documented-but-never-produced series. If a
/// delete operation is added to `ArtifactStore`, add the variant here (and its
/// label) at the same time so the label domain stays truthful.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectStoreOp {
    Read,
    Write,
    Delete,
}

impl ObjectStoreOp {
    fn as_str(self) -> &'static str {
        match self {
            ObjectStoreOp::Read => "read",
            ObjectStoreOp::Write => "write",
            ObjectStoreOp::Delete => "delete",
        }
    }
}

/// Cheaply-clonable handle to the process metrics registry.
#[derive(Clone)]
pub struct Metrics {
    registry: Arc<MetricsRegistry>,
}

impl Metrics {
    /// Create a fresh, isolated metrics handle (used by [`init_observability`]
    /// and by tests). Production code should use [`metrics()`] for the global.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(MetricsRegistry::new()),
        }
    }

    /// Render the current state as Prometheus text exposition format.
    pub fn render(&self) -> String {
        self.registry.render()
    }

    // ---- R13.2: ingest-to-queryable lag ---------------------------------

    /// Record seconds elapsed from ingest acceptance to queryability.
    pub fn observe_ingest_lag(&self, seconds: f64) {
        if let Some(h) = self
            .registry
            .histogram("beater_ingest_to_queryable_lag_seconds")
        {
            h.observe(Labels::new(), seconds.max(0.0));
        }
    }

    // ---- R13.3: write success rate --------------------------------------

    /// Record a trace-store write outcome.
    pub fn record_write(&self, result: OpResult, count: u64) {
        if count == 0 {
            return;
        }
        if let Some(c) = self.registry.counter("beater_trace_writes_total") {
            c.incr(labels(&[("result", result.as_str())]), count);
        }
    }

    // ---- R13.4: eval queue depth/age ------------------------------------

    /// Set eval queue depth and oldest-item age (seconds).
    pub fn set_eval_queue(&self, depth: usize, oldest_age_seconds: f64) {
        if let Some(g) = self.registry.gauge("beater_eval_queue_depth") {
            g.set(Labels::new(), depth as f64);
        }
        if let Some(g) = self.registry.gauge("beater_eval_queue_oldest_age_seconds") {
            g.set(Labels::new(), oldest_age_seconds.max(0.0));
        }
    }

    // ---- R13.5: query latency -------------------------------------------

    /// Observe an HTTP request latency (seconds) labelled by route + method.
    pub fn observe_query_latency(&self, route: &str, method: &str, seconds: f64) {
        if let Some(h) = self
            .registry
            .histogram("beater_http_request_duration_seconds")
        {
            h.observe(
                labels(&[("route", route), ("method", method)]),
                seconds.max(0.0),
            );
        }
    }

    // ---- R13.6: DLQ age/count -------------------------------------------

    /// Set dead-letter queue depth (count) for a lane.
    ///
    /// Depth is owned by whoever observes a fresh count: drain workers report the
    /// per-drain DLQ delta for their lane, while the queue-stats sampler reports
    /// the unfiltered global DLQ count. The companion `*_oldest_age_seconds`
    /// gauge is owned EXCLUSIVELY by the sampler (see [`Metrics::set_dlq_oldest_age`])
    /// so the two writers never race on the age series.
    pub fn set_dlq_depth(&self, lane: &str, depth: usize) {
        if let Some(g) = self.registry.gauge("beater_dlq_depth") {
            g.set(labels(&[("lane", lane)]), depth as f64);
        }
    }

    /// Set the oldest dead-lettered message age (seconds) for a lane.
    ///
    /// Owned EXCLUSIVELY by the queue-stats sampler so it is the single writer of
    /// the age series; drain workers only set depth via [`Metrics::set_dlq_depth`].
    pub fn set_dlq_oldest_age(&self, lane: &str, oldest_age_seconds: f64) {
        if let Some(g) = self.registry.gauge("beater_dlq_oldest_age_seconds") {
            g.set(labels(&[("lane", lane)]), oldest_age_seconds.max(0.0));
        }
    }

    // ---- R13.7: normalizer failures by dialect/version ------------------

    /// Record a normalizer failure labelled by source dialect and version.
    pub fn record_normalizer_failure(&self, dialect: &str, version: &str) {
        if let Some(c) = self.registry.counter("beater_normalizer_failures_total") {
            c.incr(labels(&[("dialect", dialect), ("version", version)]), 1);
        }
    }

    // ---- R13.8: per-lane/per-tenant queue lag ---------------------------

    /// Set queue lag (seconds) for a (lane, tenant) pair. Labels are
    /// cardinality-safe via [`safe_label`] and the per-family series cap.
    pub fn set_queue_lag(&self, lane: &str, tenant: &str, lag_seconds: f64) {
        if let Some(g) = self.registry.gauge("beater_queue_lag_seconds") {
            g.set(
                labels(&[("lane", lane), ("tenant", tenant)]),
                lag_seconds.max(0.0),
            );
        }
    }

    // ---- R13.9: object-store failures -----------------------------------

    /// Record an object-store operation outcome (read|write|delete).
    pub fn record_object_store_op(&self, op: ObjectStoreOp, result: OpResult) {
        if let Some(c) = self.registry.counter("beater_object_store_ops_total") {
            c.incr(
                labels(&[("op", op.as_str()), ("result", result.as_str())]),
                1,
            );
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL: OnceLock<Metrics> = OnceLock::new();

/// Access the process-wide metrics handle, initializing it on first use.
pub fn metrics() -> Metrics {
    GLOBAL.get_or_init(Metrics::new).clone()
}

static LOG_INIT: OnceLock<()> = OnceLock::new();

/// R13.1 — install the process-wide observability foundation: the global
/// [`MetricsRegistry`] and a structured stderr log writer. Returns the metrics
/// handle. Idempotent: repeated calls return the already-installed handle.
///
/// The log writer is a minimal, dependency-free stand-in for a
/// `tracing_subscriber` fmt layer / OpenTelemetry MeterProvider: it emits a
/// single structured init line so operators can confirm self-observability is
/// active, and keeps the door open to swapping in an OTLP self-exporter later
/// without touching call sites.
pub fn init_observability() -> Metrics {
    let handle = metrics();
    LOG_INIT.get_or_init(|| {
        log_event(
            "info",
            "self-observability metrics initialized",
            &[("component", "beaterd"), ("exposition", "prometheus")],
        );
    });
    handle
}

/// Emit a single structured (logfmt-style) event to stderr. This is the seam a
/// real `tracing`/OTLP backend would replace; keeping it here means call sites
/// never change.
pub fn log_event(level: &str, message: &str, fields: &[(&str, &str)]) {
    let mut line = format!("level={level} msg=\"{}\"", safe_label(message));
    for (k, v) in fields {
        let _ = write!(line, " {k}=\"{}\"", safe_label(v));
    }
    eprintln!("{line}");
}

/// A simple stopwatch returning elapsed seconds, used by the query-latency
/// middleware (R13.5) and lag observations (R13.2).
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_label_clamps_empty_and_truncates() {
        assert_eq!(safe_label("  "), "unknown");
        assert_eq!(safe_label("ok"), "ok");
        let long = "x".repeat(200);
        assert_eq!(safe_label(&long).len(), MAX_LABEL_VALUE_LEN);
    }

    #[test]
    fn safe_label_escapes_quotes_and_newlines() {
        assert_eq!(safe_label("a\"b"), "a\\\"b");
        assert_eq!(safe_label("a\nb"), "a\\nb");
        assert_eq!(safe_label("a\\b"), "a\\\\b");
    }

    #[test]
    fn write_success_rate_counts_both_results() {
        let m = Metrics::new();
        m.record_write(OpResult::Success, 3);
        m.record_write(OpResult::Failure, 1);
        let out = m.render();
        assert!(out.contains("beater_trace_writes_total{result=\"success\"} 3"));
        assert!(out.contains("beater_trace_writes_total{result=\"failure\"} 1"));
        assert!(out.contains("# TYPE beater_trace_writes_total counter"));
    }

    #[test]
    fn ingest_lag_histogram_renders_buckets_sum_count() {
        let m = Metrics::new();
        m.observe_ingest_lag(0.02);
        m.observe_ingest_lag(2.0);
        let out = m.render();
        assert!(out.contains("# TYPE beater_ingest_to_queryable_lag_seconds histogram"));
        assert!(out.contains("beater_ingest_to_queryable_lag_seconds_count 2"));
        assert!(out.contains("beater_ingest_to_queryable_lag_seconds_bucket{le=\"+Inf\"} 2"));
        // 0.02 falls in <=0.025 bucket; 2.0 does not.
        assert!(out.contains("beater_ingest_to_queryable_lag_seconds_bucket{le=\"0.025\"} 1"));
    }

    #[test]
    fn query_latency_labelled_by_route_and_method() {
        let m = Metrics::new();
        m.observe_query_latency("/v1/traces/:tenant_id", "GET", 0.3);
        let out = m.render();
        assert!(out.contains("route=\"/v1/traces/:tenant_id\""));
        assert!(out.contains("method=\"GET\""));
    }

    #[test]
    fn eval_queue_and_dlq_gauges() {
        let m = Metrics::new();
        m.set_eval_queue(7, 42.5);
        m.set_dlq_depth("trace.write", 2);
        m.set_dlq_oldest_age("trace.write", 90.0);
        let out = m.render();
        assert!(out.contains("beater_eval_queue_depth 7"));
        assert!(out.contains("beater_eval_queue_oldest_age_seconds 42.5"));
        assert!(out.contains("beater_dlq_depth{lane=\"trace.write\"} 2"));
        assert!(out.contains("beater_dlq_oldest_age_seconds{lane=\"trace.write\"} 90"));
    }

    #[test]
    fn normalizer_failures_by_dialect_version() {
        let m = Metrics::new();
        m.record_normalizer_failure("temporal", "v1.2");
        m.record_normalizer_failure("temporal", "v1.2");
        let out = m.render();
        assert!(
            out.contains(
                "beater_normalizer_failures_total{dialect=\"temporal\",version=\"v1.2\"} 2"
            )
        );
    }

    #[test]
    fn queue_lag_labels_are_cardinality_safe() {
        let m = Metrics::new();
        m.set_queue_lag("trace.ingested", "tenant-a", 12.0);
        // An unbounded value gets truncated rather than creating a huge label.
        m.set_queue_lag("trace.ingested", &"z".repeat(500), 5.0);
        let out = m.render();
        assert!(out.contains("lane=\"trace.ingested\",tenant=\"tenant-a\""));
        assert!(!out.contains(&"z".repeat(500)));
    }

    #[test]
    fn object_store_failures_by_op_and_result() {
        let m = Metrics::new();
        m.record_object_store_op(ObjectStoreOp::Read, OpResult::Failure);
        m.record_object_store_op(ObjectStoreOp::Write, OpResult::Success);
        let out = m.render();
        assert!(out.contains("beater_object_store_ops_total{op=\"read\",result=\"failure\"} 1"));
        assert!(out.contains("beater_object_store_ops_total{op=\"write\",result=\"success\"} 1"));
    }

    #[test]
    fn counter_family_overflow_is_bounded() {
        let m = Metrics::new();
        for i in 0..(MAX_SERIES_PER_FAMILY + 50) {
            m.record_normalizer_failure("d", &format!("v{i}"));
        }
        let out = m.render();
        assert!(out.contains("overflow=\"true\""));
    }

    #[test]
    fn init_observability_is_idempotent() {
        let a = init_observability();
        let b = init_observability();
        // Both share the same global registry, so writes are visible across handles.
        a.record_write(OpResult::Success, 1);
        assert!(
            b.render()
                .contains("beater_trace_writes_total{result=\"success\"}")
        );
    }
}
