use anyhow::Context;
use beater_core::{ProjectId, TenantId, Timestamp, TraceId};
use beater_schema::{CanonicalSpan, SpanStatus, TraceView};
use beater_security::{sign_webhook, webhook_idempotency_key, WEBHOOK_IDEMPOTENCY_KEY_HEADER};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OnlineSamplingPolicy {
    pub sample_rate_per_mille: u16,
    pub keep_errors: bool,
    pub slow_ms_threshold: Option<u64>,
    pub high_cost_micros_threshold: Option<i64>,
}

impl Default for OnlineSamplingPolicy {
    fn default() -> Self {
        Self {
            sample_rate_per_mille: 100,
            keep_errors: true,
            slow_ms_threshold: Some(30_000),
            high_cost_micros_threshold: Some(50_000),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SamplingReason {
    ErrorTrace,
    SlowTrace,
    HighCostTrace,
    RoutineSampled,
    RoutineDropped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SamplingDecision {
    pub selected: bool,
    pub reason: SamplingReason,
    pub stable_score_per_mille: u16,
}

pub fn decide_trace_sampling(trace: &TraceView, policy: &OnlineSamplingPolicy) -> SamplingDecision {
    let stable_score_per_mille = stable_trace_score(&trace.tenant_id, &trace.trace_id);
    if policy.keep_errors
        && trace
            .spans
            .iter()
            .any(|span| span.status == SpanStatus::Error)
    {
        return SamplingDecision {
            selected: true,
            reason: SamplingReason::ErrorTrace,
            stable_score_per_mille,
        };
    }
    if let Some(threshold) = policy.slow_ms_threshold {
        if trace_latency_ms(&trace.spans).is_some_and(|latency| latency >= threshold) {
            return SamplingDecision {
                selected: true,
                reason: SamplingReason::SlowTrace,
                stable_score_per_mille,
            };
        }
    }
    if let Some(threshold) = policy.high_cost_micros_threshold {
        if trace_cost_micros(&trace.spans) >= threshold {
            return SamplingDecision {
                selected: true,
                reason: SamplingReason::HighCostTrace,
                stable_score_per_mille,
            };
        }
    }
    let sample_rate = policy.sample_rate_per_mille.min(1000);
    let selected = stable_score_per_mille < sample_rate;
    SamplingDecision {
        selected,
        reason: if selected {
            SamplingReason::RoutineSampled
        } else {
            SamplingReason::RoutineDropped
        },
        stable_score_per_mille,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertPolicy {
    pub policy_id: String,
    pub endpoint_url: String,
    pub signing_secret: String,
    pub severity: AlertSeverity,
    pub fire_when_score_at_or_below: f64,
    pub dedupe_window_seconds: i64,
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

impl std::fmt::Debug for AlertPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AlertPolicy")
            .field("policy_id", &self.policy_id)
            .field("endpoint_url", &self.endpoint_url)
            .field("signing_secret", &"[redacted]")
            .field("severity", &self.severity)
            .field(
                "fire_when_score_at_or_below",
                &self.fire_when_score_at_or_below,
            )
            .field("dedupe_window_seconds", &self.dedupe_window_seconds)
            .field("maintenance_windows", &self.maintenance_windows)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MaintenanceWindow {
    #[schema(value_type = String, format = DateTime)]
    pub starts_at: Timestamp,
    #[schema(value_type = String, format = DateTime)]
    pub ends_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertLinks {
    pub trace_url: String,
    pub cluster_url: Option<String>,
    pub dataset_url: Option<String>,
    pub gate_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertInput {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub group_key: String,
    pub title: String,
    pub score: f64,
    pub baseline_score: Option<f64>,
    pub links: AlertLinks,
    #[schema(value_type = String, format = DateTime)]
    pub now: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AlertDecision {
    pub emitted: bool,
    pub suppressed_reason: Option<String>,
    pub delivery: Option<WebhookDelivery>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WebhookDelivery {
    pub endpoint_url: String,
    pub headers: BTreeMap<String, String>,
    #[schema(value_type = serde_json::Value)]
    pub body: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlackChannel;

impl SlackChannel {
    pub fn format_alert(&self, policy: &AlertPolicy, input: &AlertInput) -> serde_json::Value {
        slack_alert_payload(policy, input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlertDedupeDecision {
    Recorded,
    Suppressed { last_emitted_at: Timestamp },
}

pub type AlertDedupeStoreResult<T> = Result<T, AlertDedupeStoreError>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum AlertDedupeStoreError {
    #[error("alert dedupe store backend error: {0}")]
    Backend(String),
}

impl AlertDedupeStoreError {
    pub fn backend(error: impl std::fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }
}

impl From<anyhow::Error> for AlertDedupeStoreError {
    fn from(error: anyhow::Error) -> Self {
        Self::backend(error)
    }
}

pub trait AlertDedupeStore: std::fmt::Debug + Send + Sync {
    fn check_and_record(
        &self,
        group_key: &str,
        now: Timestamp,
        dedupe_window: Duration,
    ) -> AlertDedupeStoreResult<AlertDedupeDecision>;
}

#[derive(Clone, Debug, Default)]
pub struct MemoryAlertDedupeStore {
    state: Arc<Mutex<AlertDedupeState>>,
}

impl MemoryAlertDedupeStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AlertDedupeStore for MemoryAlertDedupeStore {
    fn check_and_record(
        &self,
        group_key: &str,
        now: Timestamp,
        dedupe_window: Duration,
    ) -> AlertDedupeStoreResult<AlertDedupeDecision> {
        let mut state = self.state.lock().map_err(|err| {
            AlertDedupeStoreError::backend(format!("alert dedupe store mutex poisoned: {err}"))
        })?;
        Ok(check_and_record_in_state(
            &mut state,
            group_key,
            now,
            dedupe_window,
        ))
    }
}

#[derive(Debug)]
pub struct JsonFileAlertDedupeStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl JsonFileAlertDedupeStore {
    pub fn new(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        ensure_parent_dir(&path)?;
        Ok(Self {
            path,
            lock: Mutex::new(()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load_state(&self) -> anyhow::Result<AlertDedupeState> {
        match fs::read(&self.path) {
            Ok(bytes) if bytes.is_empty() => Ok(AlertDedupeState::default()),
            Ok(bytes) => serde_json::from_slice(&bytes)
                .with_context(|| format!("parse alert dedupe store from {}", self.path.display())),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(AlertDedupeState::default()),
            Err(err) => Err(err)
                .with_context(|| format!("read alert dedupe store from {}", self.path.display())),
        }
    }

    fn write_state(&self, state: &AlertDedupeState) -> anyhow::Result<()> {
        ensure_parent_dir(&self.path)?;
        let bytes = serde_json::to_vec_pretty(state).context("serialize alert dedupe store")?;
        let tmp_path = temp_path_for(&self.path);
        let mut tmp_file = fs::File::create(&tmp_path)
            .with_context(|| format!("create alert dedupe temp file {}", tmp_path.display()))?;
        tmp_file
            .write_all(&bytes)
            .with_context(|| format!("write alert dedupe temp file {}", tmp_path.display()))?;
        tmp_file
            .write_all(b"\n")
            .with_context(|| format!("write alert dedupe temp file {}", tmp_path.display()))?;
        tmp_file
            .sync_all()
            .with_context(|| format!("sync alert dedupe temp file {}", tmp_path.display()))?;
        drop(tmp_file);

        if let Err(err) = fs::rename(&tmp_path, &self.path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(err).with_context(|| {
                format!(
                    "replace alert dedupe store {} with {}",
                    self.path.display(),
                    tmp_path.display()
                )
            });
        }
        Ok(())
    }
}

impl AlertDedupeStore for JsonFileAlertDedupeStore {
    fn check_and_record(
        &self,
        group_key: &str,
        now: Timestamp,
        dedupe_window: Duration,
    ) -> AlertDedupeStoreResult<AlertDedupeDecision> {
        let _guard = self.lock.lock().map_err(|err| {
            AlertDedupeStoreError::backend(format!("alert dedupe file store mutex poisoned: {err}"))
        })?;
        let mut state = self.load_state()?;
        let decision = check_and_record_in_state(&mut state, group_key, now, dedupe_window);
        if decision == AlertDedupeDecision::Recorded {
            self.write_state(&state)?;
        }
        Ok(decision)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct AlertDedupeState {
    #[serde(default)]
    last_emitted_by_group: BTreeMap<String, Timestamp>,
}

fn check_and_record_in_state(
    state: &mut AlertDedupeState,
    group_key: &str,
    now: Timestamp,
    dedupe_window: Duration,
) -> AlertDedupeDecision {
    if let Some(last_emitted) = state.last_emitted_by_group.get(group_key) {
        if in_dedupe_window(*last_emitted, now, dedupe_window) {
            return AlertDedupeDecision::Suppressed {
                last_emitted_at: *last_emitted,
            };
        }
    }
    state
        .last_emitted_by_group
        .insert(group_key.to_string(), now);
    AlertDedupeDecision::Recorded
}

fn in_dedupe_window(last_emitted: Timestamp, now: Timestamp, dedupe_window: Duration) -> bool {
    let age = now.signed_duration_since(last_emitted);
    age >= Duration::zero() && age < dedupe_window
}

fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create alert dedupe store directory {}", parent.display()))?;
    }
    Ok(())
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut temp_name = path
        .file_name()
        .map(std::ffi::OsString::from)
        .unwrap_or_else(|| std::ffi::OsString::from("alert-dedupe-store"));
    temp_name.push(format!(".{}.tmp", std::process::id()));
    path.with_file_name(temp_name)
}

#[derive(Clone, Debug)]
pub struct AlertEngine {
    dedupe_store: Arc<dyn AlertDedupeStore>,
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self {
            dedupe_store: Arc::new(MemoryAlertDedupeStore::new()),
        }
    }
}

impl AlertEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dedupe_store(dedupe_store: Arc<dyn AlertDedupeStore>) -> Self {
        Self { dedupe_store }
    }

    pub fn evaluate(
        &self,
        policy: &AlertPolicy,
        input: AlertInput,
    ) -> anyhow::Result<AlertDecision> {
        if input.score > policy.fire_when_score_at_or_below {
            return Ok(AlertDecision {
                emitted: false,
                suppressed_reason: Some("score_above_threshold".to_string()),
                delivery: None,
            });
        }
        if in_maintenance_window(&policy.maintenance_windows, input.now) {
            return Ok(AlertDecision {
                emitted: false,
                suppressed_reason: Some("maintenance_window".to_string()),
                delivery: None,
            });
        }
        let group_key = dedupe_key(policy, &input);
        let body = alert_payload(policy, &input);
        let body_bytes = serde_json::to_vec(&body).context("serialize alert payload")?;
        let signature = sign_webhook(policy.signing_secret.as_bytes(), &body_bytes, input.now)?;
        // Stable per-delivery idempotency key (R9.4): the delivery identity is the
        // logical firing — dedupe key (tenant:project:policy:group) bucketed by the
        // dedupe window. Retries of the *same* logical delivery (same input, within
        // the same window) recompute the identical id, so the receiver can dedupe;
        // a later firing of the same group (next window) gets a distinct id.
        let delivery_id = delivery_identity(policy, &group_key, input.now);
        let idempotency_key =
            webhook_idempotency_key(policy.signing_secret.as_bytes(), &delivery_id, &body_bytes)?;
        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("beater-signature".to_string(), signature.header_value());
        headers.insert("beater-alert-policy".to_string(), policy.policy_id.clone());
        headers.insert(WEBHOOK_IDEMPOTENCY_KEY_HEADER.to_string(), idempotency_key);
        let delivery = WebhookDelivery {
            endpoint_url: policy.endpoint_url.clone(),
            headers,
            body,
        };
        let dedupe_window = Duration::seconds(policy.dedupe_window_seconds.max(0));
        if let AlertDedupeDecision::Suppressed { .. } =
            self.dedupe_store
                .check_and_record(&group_key, input.now, dedupe_window)?
        {
            return Ok(AlertDecision {
                emitted: false,
                suppressed_reason: Some("dedupe_window".to_string()),
                delivery: None,
            });
        }
        Ok(AlertDecision {
            emitted: true,
            suppressed_reason: None,
            delivery: Some(delivery),
        })
    }
}

fn alert_payload(policy: &AlertPolicy, input: &AlertInput) -> serde_json::Value {
    serde_json::json!({
        "policy_id": policy.policy_id,
        "severity": policy.severity,
        "tenant_id": input.tenant_id,
        "project_id": input.project_id,
        "trace_id": input.trace_id,
        "group_key": input.group_key,
        "title": input.title,
        "score": input.score,
        "baseline_score": input.baseline_score,
        "threshold": policy.fire_when_score_at_or_below,
        "links": input.links,
        "emitted_at": input.now,
    })
}

fn slack_alert_payload(policy: &AlertPolicy, input: &AlertInput) -> serde_json::Value {
    let severity = severity_text(&policy.severity);
    let mut blocks = vec![
        serde_json::json!({
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": format!("{severity} alert"),
                "emoji": true,
            },
        }),
        serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*{}*\n{}", input.title, score_context(input)),
            },
        }),
        serde_json::json!({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": format!(
                        "*Severity:* {severity}  *Tenant:* `{}`  *Project:* `{}`  *Trace:* `{}`",
                        input.tenant_id.as_str(),
                        input.project_id.as_str(),
                        input.trace_id.as_str(),
                    ),
                },
            ],
        }),
    ];

    let trace_url = input.links.trace_url.trim();
    if !trace_url.is_empty() {
        blocks.push(serde_json::json!({
            "type": "actions",
            "elements": [
                {
                    "type": "button",
                    "action_id": "view_trace",
                    "text": {
                        "type": "plain_text",
                        "text": "View trace",
                        "emoji": true,
                    },
                    "url": trace_url,
                },
            ],
        }));
    }

    serde_json::json!({
        "text": format!(
            "{severity} alert: {} (score {})",
            input.title,
            format_score(input.score)
        ),
        "blocks": blocks,
    })
}

fn severity_text(severity: &AlertSeverity) -> &'static str {
    match severity {
        AlertSeverity::Info => "Info",
        AlertSeverity::Warning => "Warning",
        AlertSeverity::Critical => "Critical",
    }
}

fn score_context(input: &AlertInput) -> String {
    let score = format_score(input.score);
    match input.baseline_score {
        Some(baseline) => {
            let delta = input.score - baseline;
            format!(
                "*Score:* `{score}`  *Baseline:* `{}`  *Delta:* `{}`",
                format_score(baseline),
                format_delta(delta),
            )
        }
        None => format!("*Score:* `{score}`"),
    }
}

fn format_score(score: f64) -> String {
    format!("{score:.3}")
}

fn format_delta(delta: f64) -> String {
    format!("{delta:+.3}")
}

/// Logical delivery identity for the idempotency key. Combines the dedupe key
/// with a window bucket so that retries of the same firing (same `now` bucket)
/// are identical while a fresh firing in a later window is distinct.
fn delivery_identity(policy: &AlertPolicy, dedupe_key: &str, now: Timestamp) -> String {
    let window = policy.dedupe_window_seconds.max(1);
    let bucket = now.timestamp().div_euclid(window);
    format!("{dedupe_key}:{bucket}")
}

fn dedupe_key(policy: &AlertPolicy, input: &AlertInput) -> String {
    format!(
        "{}:{}:{}:{}",
        input.tenant_id.as_str(),
        input.project_id.as_str(),
        policy.policy_id,
        input.group_key
    )
}

fn in_maintenance_window(windows: &[MaintenanceWindow], now: Timestamp) -> bool {
    windows
        .iter()
        .any(|window| now >= window.starts_at && now <= window.ends_at)
}

fn stable_trace_score(tenant_id: &TenantId, trace_id: &TraceId) -> u16 {
    let mut hasher = Sha256::new();
    hasher.update(tenant_id.as_str().as_bytes());
    hasher.update(b":");
    hasher.update(trace_id.as_str().as_bytes());
    let digest = hasher.finalize();
    u16::from_be_bytes([digest[0], digest[1]]) % 1000
}

fn trace_cost_micros(spans: &[CanonicalSpan]) -> i64 {
    spans
        .iter()
        .filter_map(|span| span.cost.as_ref())
        .map(|cost| cost.amount_micros)
        .sum()
}

fn trace_latency_ms(spans: &[CanonicalSpan]) -> Option<u64> {
    let start = spans.iter().map(|span| span.start_time).min()?;
    let end = spans.iter().filter_map(|span| span.end_time).max()?;
    let millis = end.signed_duration_since(start).num_milliseconds();
    Some(millis.max(0) as u64)
}

/// Outcome of feeding a span into the [`TailBuffer`].
///
/// Spans for an incomplete trace are *held* (`Buffered`) and never sampled; the
/// online sampling decision is deferred until the trace's
/// [`beater_schema::TraceCompletionState`] flips to a terminal value, at which
/// point the buffered spans are flushed through [`decide_trace_sampling`] exactly
/// once (`Sampled`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TailBufferOutcome {
    /// The span was accumulated; the trace is not yet complete so no sampling
    /// decision has been made. Carries the running buffered span count.
    Buffered { buffered_spans: usize },
    /// The trace reached completion on this span; the accumulated trace was run
    /// through [`decide_trace_sampling`] and is now evicted from the buffer.
    Sampled {
        decision: SamplingDecision,
        sampled_spans: usize,
    },
    /// The current span was buffered, but accepting it pushed the open-trace count
    /// over the in-memory cap, so the *oldest* still-open trace was force-flushed
    /// through [`decide_trace_sampling`] and evicted to reclaim memory. Carries the
    /// running buffered span count for the current trace and the forced decision
    /// (with span count) for the evicted one.
    Evicted {
        buffered_spans: usize,
        evicted_decision: SamplingDecision,
        evicted_spans: usize,
    },
}

/// Tail-sampling buffer: accumulates the spans of in-flight traces and holds the
/// online sampling decision until the trace is observed to be complete.
///
/// A streaming sampler that decided per span would have to keep or drop a trace
/// before its error/latency/cost are known. The tail buffer instead retains every
/// span keyed by `trace_id` and only invokes [`decide_trace_sampling`] once the
/// caller reports a terminal [`beater_schema::TraceCompletionState`] (e.g.
/// `RootEnded`, `Complete`, `IdleComplete`, `LateWindowClosed`). That guarantees
/// the policy observes the *whole* trace — including late-arriving error/slow/
/// costly spans.
///
/// The buffer is purely in-process and ordering-agnostic: spans may be offered in
/// any order, and completion is driven by the caller's
/// `beater_ingest::trace_completion_state` evaluation rather than by wall-clock
/// timers inside the buffer.
/// ponytail: the open-trace ceiling is `DEFAULT_MAX_OPEN_TRACES` traces held
/// purely in process memory. A trace whose completion never flips terminal would
/// otherwise be buffered forever (memory leak); at the cap we force-flush the
/// oldest open trace instead of growing without bound. The upgrade path is durable
/// (spill-to-disk / external) buffering, which would raise or remove this ceiling.
const DEFAULT_MAX_OPEN_TRACES: usize = 100_000;

#[derive(Debug)]
pub struct TailBuffer {
    policy: OnlineSamplingPolicy,
    by_trace: BTreeMap<TraceId, BufferedTrace>,
    /// In-memory ceiling on concurrently open traces. See `ponytail` note above.
    max_open_traces: usize,
    /// Monotonic counter stamped on each newly opened trace so the *oldest* open
    /// trace can be identified for forced eviction at the cap.
    next_insert_seq: u64,
}

impl Default for TailBuffer {
    fn default() -> Self {
        Self {
            policy: OnlineSamplingPolicy::default(),
            by_trace: BTreeMap::new(),
            max_open_traces: DEFAULT_MAX_OPEN_TRACES,
            next_insert_seq: 0,
        }
    }
}

#[derive(Debug)]
struct BufferedTrace {
    tenant_id: TenantId,
    spans: Vec<CanonicalSpan>,
    /// Order in which this trace was first opened; used to evict the oldest.
    insert_seq: u64,
}

/// Terminal completion states that flush the tail buffer. `Open` keeps the trace
/// buffered; everything else is treated as "the trace is done, decide now".
fn completion_is_terminal(state: &beater_schema::TraceCompletionState) -> bool {
    !matches!(state, beater_schema::TraceCompletionState::Open)
}

impl TailBuffer {
    pub fn new(policy: OnlineSamplingPolicy) -> Self {
        Self {
            policy,
            by_trace: BTreeMap::new(),
            max_open_traces: DEFAULT_MAX_OPEN_TRACES,
            next_insert_seq: 0,
        }
    }

    /// Override the in-memory open-trace ceiling (see the `ponytail` note on
    /// [`DEFAULT_MAX_OPEN_TRACES`]). A cap of zero is clamped to one so at least
    /// the current trace can be held. Intended for tests and tuned deployments.
    pub fn with_max_open_traces(mut self, max_open_traces: usize) -> Self {
        self.max_open_traces = max_open_traces.max(1);
        self
    }

    /// Number of traces currently held open (not yet sampled).
    pub fn open_traces(&self) -> usize {
        self.by_trace.len()
    }

    /// Spans currently buffered for `trace_id` (zero once flushed).
    pub fn buffered_spans(&self, trace_id: &TraceId) -> usize {
        self.by_trace
            .get(trace_id)
            .map(|trace| trace.spans.len())
            .unwrap_or(0)
    }

    /// Accumulate `span` and, if `completion` is terminal, flush the buffered
    /// trace through [`decide_trace_sampling`]. Spans are held until completion,
    /// so a non-terminal `completion` normally yields [`TailBufferOutcome::Buffered`].
    ///
    /// If accepting a *new* trace pushes the open-trace count over the in-memory
    /// cap (see the `ponytail` note on [`DEFAULT_MAX_OPEN_TRACES`]), the oldest open
    /// trace is force-flushed and evicted, yielding [`TailBufferOutcome::Evicted`].
    pub fn offer(
        &mut self,
        span: CanonicalSpan,
        completion: &beater_schema::TraceCompletionState,
    ) -> TailBufferOutcome {
        let trace_id = span.trace_id.clone();
        let next_seq = self.next_insert_seq;
        let mut opened_new = false;
        let entry = self.by_trace.entry(trace_id.clone()).or_insert_with(|| {
            opened_new = true;
            BufferedTrace {
                tenant_id: span.tenant_id.clone(),
                spans: Vec::new(),
                insert_seq: next_seq,
            }
        });
        if opened_new {
            self.next_insert_seq += 1;
        }
        entry.spans.push(span);

        if completion_is_terminal(completion) {
            // Terminal: evict and run the whole-trace sampling decision exactly once.
            return self
                .flush_trace(&trace_id)
                .map(|(decision, sampled_spans)| TailBufferOutcome::Sampled {
                    decision,
                    sampled_spans,
                })
                .unwrap_or_else(|| unreachable!("trace was just inserted"));
        }

        let buffered_spans = self.buffered_spans(&trace_id);

        // Bounded guard: only a freshly opened trace can grow the open set, so the
        // cap is only ever breached here. Force-flush the oldest open trace so a
        // never-completing trace cannot leak memory forever.
        if opened_new && self.by_trace.len() > self.max_open_traces {
            if let Some((evicted_decision, evicted_spans)) = self.evict_oldest(&trace_id) {
                return TailBufferOutcome::Evicted {
                    buffered_spans,
                    evicted_decision,
                    evicted_spans,
                };
            }
        }

        TailBufferOutcome::Buffered { buffered_spans }
    }

    /// Remove `trace_id` and run the whole-trace sampling decision over its
    /// buffered spans. Returns `None` if no such trace is buffered.
    fn flush_trace(&mut self, trace_id: &TraceId) -> Option<(SamplingDecision, usize)> {
        let buffered = self.by_trace.remove(trace_id)?;
        let sampled_spans = buffered.spans.len();
        let trace = TraceView {
            tenant_id: buffered.tenant_id,
            trace_id: trace_id.clone(),
            spans: buffered.spans,
        };
        let decision = decide_trace_sampling(&trace, &self.policy);
        Some((decision, sampled_spans))
    }

    /// Force-flush the oldest open trace other than `keep` (the just-opened trace),
    /// reclaiming its memory. Returns the forced decision and its span count.
    fn evict_oldest(&mut self, keep: &TraceId) -> Option<(SamplingDecision, usize)> {
        let oldest = self
            .by_trace
            .iter()
            .filter(|(id, _)| *id != keep)
            .min_by_key(|(_, trace)| trace.insert_seq)
            .map(|(id, _)| id.clone())?;
        self.flush_trace(&oldest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{Money, TokenCounts};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    fn required_link_keys(delivery: &WebhookDelivery) -> BTreeSet<&'static str> {
        let links = &delivery.body["links"];
        ["trace_url", "cluster_url", "dataset_url", "gate_url"]
            .into_iter()
            .filter(|key| links.get(key).and_then(|value| value.as_str()).is_some())
            .collect()
    }

    use beater_schema::{AgentSpanKind, ArtifactRef, RedactionClass, CANONICAL_SCHEMA_VERSION};
    use chrono::Utc;

    #[test]
    fn sampling_keeps_errors_slow_and_high_cost_traces() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut trace = fixture_trace(&tenant, SpanStatus::Ok, 1_000, 0);
        let policy = OnlineSamplingPolicy {
            sample_rate_per_mille: 0,
            keep_errors: true,
            slow_ms_threshold: Some(5_000),
            high_cost_micros_threshold: Some(10_000),
        };
        assert_eq!(
            decide_trace_sampling(&trace, &policy).reason,
            SamplingReason::RoutineDropped
        );

        trace.spans[0].status = SpanStatus::Error;
        assert_eq!(
            decide_trace_sampling(&trace, &policy).reason,
            SamplingReason::ErrorTrace
        );

        let slow_trace = fixture_trace(&tenant, SpanStatus::Ok, 6_000, 0);
        assert_eq!(
            decide_trace_sampling(&slow_trace, &policy).reason,
            SamplingReason::SlowTrace
        );

        let costly_trace = fixture_trace(&tenant, SpanStatus::Ok, 1_000, 20_000);
        assert_eq!(
            decide_trace_sampling(&costly_trace, &policy).reason,
            SamplingReason::HighCostTrace
        );
    }

    #[test]
    fn alert_engine_signs_dedupes_and_respects_maintenance_windows() {
        let engine = AlertEngine::new();
        let now = Utc::now();
        let policy = AlertPolicy {
            policy_id: "low-score".to_string(),
            endpoint_url: "https://example.test/webhook".to_string(),
            signing_secret: "secret".to_string(),
            severity: AlertSeverity::Critical,
            fire_when_score_at_or_below: 0.5,
            dedupe_window_seconds: 60,
            maintenance_windows: Vec::new(),
        };
        let input = fixture_alert_input(now);
        let first = engine
            .evaluate(&policy, input.clone())
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(first.emitted);
        let delivery = first
            .delivery
            .as_ref()
            .unwrap_or_else(|| panic!("expected delivery"));
        assert_eq!(
            required_link_keys(delivery),
            BTreeSet::from(["cluster_url", "dataset_url", "gate_url", "trace_url"])
        );
        let body = serde_json::to_vec(&delivery.body).unwrap_or_else(|err| panic!("{err}"));
        beater_security::verify_webhook(
            b"secret",
            &body,
            delivery
                .headers
                .get("beater-signature")
                .unwrap_or_else(|| panic!("missing signature header")),
            now,
            Duration::seconds(300),
        )
        .unwrap_or_else(|err| panic!("{err}"));

        let second = engine
            .evaluate(&policy, input)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!second.emitted);
        assert_eq!(second.suppressed_reason.as_deref(), Some("dedupe_window"));

        let later = engine
            .evaluate(
                &policy,
                fixture_alert_input(now + Duration::seconds(policy.dedupe_window_seconds)),
            )
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(later.emitted);

        let maintenance_policy = AlertPolicy {
            maintenance_windows: vec![MaintenanceWindow {
                starts_at: now - Duration::minutes(1),
                ends_at: now + Duration::minutes(1),
            }],
            ..policy
        };
        let suppressed = AlertEngine::new()
            .evaluate(&maintenance_policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!suppressed.emitted);
        assert_eq!(
            suppressed.suppressed_reason.as_deref(),
            Some("maintenance_window")
        );
    }

    #[test]
    fn shared_in_memory_dedupe_store_suppresses_across_engines() {
        let now = Utc::now();
        let store: Arc<dyn AlertDedupeStore> = Arc::new(MemoryAlertDedupeStore::new());
        let first_engine = AlertEngine::with_dedupe_store(Arc::clone(&store));
        let second_engine = AlertEngine::with_dedupe_store(store);
        let policy = fixture_alert_policy(60);

        let first = first_engine
            .evaluate(&policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(first.emitted);

        let second = second_engine
            .evaluate(&policy, fixture_alert_input(now + Duration::seconds(10)))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!second.emitted);
        assert_eq!(second.suppressed_reason.as_deref(), Some("dedupe_window"));
    }

    #[test]
    fn dedupe_key_is_scoped_by_tenant_project_and_policy() {
        let now = Utc::now();
        let store: Arc<dyn AlertDedupeStore> = Arc::new(MemoryAlertDedupeStore::new());
        let engine = AlertEngine::with_dedupe_store(store);
        let policy = fixture_alert_policy(60);

        let first = engine
            .evaluate(&policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(first.emitted);

        let same_scope = engine
            .evaluate(&policy, fixture_alert_input(now + Duration::seconds(10)))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!same_scope.emitted);
        assert_eq!(
            same_scope.suppressed_reason.as_deref(),
            Some("dedupe_window")
        );

        let mut other_tenant = fixture_alert_input(now + Duration::seconds(10));
        other_tenant.tenant_id = TenantId::new("tenant-b").unwrap_or_else(|err| panic!("{err}"));
        let tenant_scoped = engine
            .evaluate(&policy, other_tenant)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            tenant_scoped.emitted,
            "same group in another tenant must not be deduped"
        );

        let mut other_project = fixture_alert_input(now + Duration::seconds(10));
        other_project.project_id =
            ProjectId::new("project-b").unwrap_or_else(|err| panic!("{err}"));
        let project_scoped = engine
            .evaluate(&policy, other_project)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            project_scoped.emitted,
            "same group in another project must not be deduped"
        );

        let other_policy = AlertPolicy {
            policy_id: "latency-score".to_string(),
            ..policy
        };
        let policy_scoped = engine
            .evaluate(
                &other_policy,
                fixture_alert_input(now + Duration::seconds(10)),
            )
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            policy_scoped.emitted,
            "same group in another policy must not be deduped"
        );
    }

    #[test]
    fn json_file_dedupe_store_survives_engine_restart() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let store_path = temp_dir.path().join("alert-dedupe.json");
        let now = Utc::now();
        let policy = fixture_alert_policy(60);

        let first_store: Arc<dyn AlertDedupeStore> =
            Arc::new(JsonFileAlertDedupeStore::new(store_path.clone())?);
        let first_engine = AlertEngine::with_dedupe_store(first_store);
        let first = first_engine.evaluate(&policy, fixture_alert_input(now))?;
        assert!(first.emitted);

        drop(first_engine);

        let restarted_store: Arc<dyn AlertDedupeStore> =
            Arc::new(JsonFileAlertDedupeStore::new(store_path)?);
        let restarted_engine = AlertEngine::with_dedupe_store(restarted_store);
        let second =
            restarted_engine.evaluate(&policy, fixture_alert_input(now + Duration::seconds(10)))?;
        assert!(!second.emitted);
        assert_eq!(second.suppressed_reason.as_deref(), Some("dedupe_window"));
        Ok(())
    }

    #[test]
    fn dedupe_store_allows_emission_after_window() {
        let now = Utc::now();
        let store: Arc<dyn AlertDedupeStore> = Arc::new(MemoryAlertDedupeStore::new());
        let engine = AlertEngine::with_dedupe_store(store);
        let policy = fixture_alert_policy(30);

        let first = engine
            .evaluate(&policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(first.emitted);

        let suppressed = engine
            .evaluate(&policy, fixture_alert_input(now + Duration::seconds(29)))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(!suppressed.emitted);
        assert_eq!(
            suppressed.suppressed_reason.as_deref(),
            Some("dedupe_window")
        );

        let emitted = engine
            .evaluate(&policy, fixture_alert_input(now + Duration::seconds(30)))
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(emitted.emitted);
    }

    #[test]
    fn alert_policy_debug_redacts_signing_secret() {
        let policy = AlertPolicy {
            signing_secret: "super-secret-signing-key".to_string(),
            ..fixture_alert_policy(60)
        };

        let debug = format!("{policy:?}");
        assert!(debug.contains("AlertPolicy"));
        assert!(debug.contains("signing_secret: \"[redacted]\""));
        assert!(!debug.contains("super-secret-signing-key"));
    }

    #[test]
    fn webhook_delivery_carries_stable_idempotency_key_across_retries() {
        let now = Utc::now();
        let policy = AlertPolicy {
            policy_id: "low-score".to_string(),
            endpoint_url: "https://example.test/webhook".to_string(),
            signing_secret: "secret".to_string(),
            severity: AlertSeverity::Critical,
            fire_when_score_at_or_below: 0.5,
            dedupe_window_seconds: 300,
            maintenance_windows: Vec::new(),
        };

        // Two independent engines model a redelivery/retry of the same logical
        // firing (e.g. after a worker crash) where dedupe state is not shared.
        let first = AlertEngine::new()
            .evaluate(&policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"))
            .delivery
            .unwrap_or_else(|| panic!("expected delivery"));
        let retry = AlertEngine::new()
            .evaluate(&policy, fixture_alert_input(now))
            .unwrap_or_else(|err| panic!("{err}"))
            .delivery
            .unwrap_or_else(|| panic!("expected delivery"));

        let key = first
            .headers
            .get(WEBHOOK_IDEMPOTENCY_KEY_HEADER)
            .unwrap_or_else(|| panic!("idempotency header must be attached"));
        assert!(key.starts_with("bt_whk_"), "unexpected key shape: {key}");
        // Stable across a retry of the same logical delivery.
        assert_eq!(
            retry.headers.get(WEBHOOK_IDEMPOTENCY_KEY_HEADER),
            Some(key),
            "idempotency key must be stable across retries of the same firing"
        );

        // A logically distinct firing (different group) gets a different key.
        let mut other_input = fixture_alert_input(now);
        other_input.group_key = "eval:exact:other-group".to_string();
        let other = AlertEngine::new()
            .evaluate(&policy, other_input)
            .unwrap_or_else(|err| panic!("{err}"))
            .delivery
            .unwrap_or_else(|| panic!("expected delivery"));
        assert_ne!(
            other.headers.get(WEBHOOK_IDEMPOTENCY_KEY_HEADER),
            Some(key),
            "distinct firings must produce distinct idempotency keys"
        );
    }

    #[test]
    fn slack_channel_formats_block_kit_with_severity_score_and_trace_button() {
        let policy = AlertPolicy {
            policy_id: "low-score".to_string(),
            endpoint_url: "https://example.test/webhook".to_string(),
            signing_secret: "secret".to_string(),
            severity: AlertSeverity::Critical,
            fire_when_score_at_or_below: 0.5,
            dedupe_window_seconds: 300,
            maintenance_windows: Vec::new(),
        };
        let input = fixture_alert_input(Utc::now());

        let payload = SlackChannel.format_alert(&policy, &input);
        assert_eq!(
            payload["text"].as_str(),
            Some("Critical alert: Eval score dropped (score 0.100)")
        );

        let blocks = payload["blocks"]
            .as_array()
            .unwrap_or_else(|| panic!("blocks must be an array"));
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0]["type"].as_str(), Some("header"));
        assert_eq!(blocks[0]["text"]["text"].as_str(), Some("Critical alert"));

        let section_text = blocks[1]["text"]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("section text must be present"));
        assert!(section_text.contains("*Eval score dropped*"));
        assert!(section_text.contains("*Score:* `0.100`"));
        assert!(section_text.contains("*Baseline:* `0.900`"));
        assert!(section_text.contains("*Delta:* `-0.800`"));

        let context_text = blocks[2]["elements"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("context text must be present"));
        assert!(context_text.contains("*Severity:* Critical"));
        assert!(context_text.contains("*Tenant:* `tenant`"));
        assert!(context_text.contains("*Project:* `project`"));
        assert!(context_text.contains("*Trace:* `trace`"));

        let button = &blocks[3]["elements"][0];
        assert_eq!(blocks[3]["type"].as_str(), Some("actions"));
        assert_eq!(button["type"].as_str(), Some("button"));
        assert_eq!(button["action_id"].as_str(), Some("view_trace"));
        assert_eq!(button["text"]["text"].as_str(), Some("View trace"));
        assert_eq!(
            button["url"].as_str(),
            Some("https://beater.test/traces/trace")
        );
    }

    #[test]
    fn slack_channel_omits_trace_button_without_trace_url_and_allows_missing_baseline() {
        let policy = AlertPolicy {
            policy_id: "low-score".to_string(),
            endpoint_url: "https://example.test/webhook".to_string(),
            signing_secret: "secret".to_string(),
            severity: AlertSeverity::Warning,
            fire_when_score_at_or_below: 0.5,
            dedupe_window_seconds: 300,
            maintenance_windows: Vec::new(),
        };
        let mut input = fixture_alert_input(Utc::now());
        input.baseline_score = None;
        input.links.trace_url = "   ".to_string();

        let payload = SlackChannel.format_alert(&policy, &input);
        let blocks = payload["blocks"]
            .as_array()
            .unwrap_or_else(|| panic!("blocks must be an array"));
        assert_eq!(blocks.len(), 3);
        assert!(!blocks.iter().any(|block| block["type"] == "actions"));

        let section_text = blocks[1]["text"]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("section text must be present"));
        assert!(section_text.contains("*Score:* `0.100`"));
        assert!(!section_text.contains("Baseline"));
        assert!(!section_text.contains("Delta"));
        assert_eq!(blocks[0]["text"]["text"].as_str(), Some("Warning alert"));
    }

    #[test]
    fn slack_channel_trims_trace_deep_link_button_url() {
        let policy = fixture_alert_policy(300);
        let mut input = fixture_alert_input(Utc::now());
        input.links.trace_url = "  https://beater.test/traces/trace?from=alert  ".to_string();

        let payload = SlackChannel.format_alert(&policy, &input);
        let blocks = payload["blocks"]
            .as_array()
            .unwrap_or_else(|| panic!("blocks must be an array"));
        let button = blocks
            .iter()
            .find(|block| block["type"] == "actions")
            .and_then(|block| block["elements"].as_array())
            .and_then(|elements| elements.first())
            .unwrap_or_else(|| panic!("trace action button must be present"));

        assert_eq!(
            button["url"].as_str(),
            Some("https://beater.test/traces/trace?from=alert")
        );
    }

    fn fixture_alert_input(now: Timestamp) -> AlertInput {
        AlertInput {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            group_key: "eval:exact:low-score".to_string(),
            title: "Eval score dropped".to_string(),
            score: 0.1,
            baseline_score: Some(0.9),
            links: AlertLinks {
                trace_url: "https://beater.test/traces/trace".to_string(),
                cluster_url: Some("https://beater.test/clusters/cluster".to_string()),
                dataset_url: Some("https://beater.test/datasets/dataset".to_string()),
                gate_url: Some("https://beater.test/gates/gate".to_string()),
            },
            now,
        }
    }

    fn fixture_alert_policy(dedupe_window_seconds: i64) -> AlertPolicy {
        AlertPolicy {
            policy_id: "low-score".to_string(),
            endpoint_url: "https://example.test/webhook".to_string(),
            signing_secret: "secret".to_string(),
            severity: AlertSeverity::Critical,
            fire_when_score_at_or_below: 0.5,
            dedupe_window_seconds,
            maintenance_windows: Vec::new(),
        }
    }

    fn fixture_trace(
        tenant: &TenantId,
        status: SpanStatus,
        latency_ms: i64,
        cost_micros: i64,
    ) -> TraceView {
        let started = Utc::now();
        TraceView {
            tenant_id: tenant.clone(),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            spans: vec![CanonicalSpan {
                schema_version: CANONICAL_SCHEMA_VERSION,
                normalizer_version: "test".to_string(),
                tenant_id: tenant.clone(),
                project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                environment_id: beater_core::EnvironmentId::new("prod")
                    .unwrap_or_else(|err| panic!("{err}")),
                trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
                span_id: beater_core::SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
                parent_span_id: None,
                seq: 1,
                kind: AgentSpanKind::AgentRun,
                name: "run".to_string(),
                status,
                start_time: started,
                end_time: Some(started + Duration::milliseconds(latency_ms)),
                model: None,
                cost: Some(Money::usd_micros(cost_micros)),
                tokens: Some(TokenCounts::default()),
                input_ref: None,
                output_ref: None,
                attributes: BTreeMap::new(),
                unmapped_attrs: serde_json::json!({}),
                raw_ref: ArtifactRef {
                    artifact_id: beater_core::ArtifactId::new("raw")
                        .unwrap_or_else(|err| panic!("{err}")),
                    uri: "artifact://tenant/project/raw".to_string(),
                    sha256: beater_core::Sha256Hash::new("ab".repeat(32))
                        .unwrap_or_else(|err| panic!("{err}")),
                    size_bytes: 2,
                    mime_type: "application/json".to_string(),
                    redaction_class: RedactionClass::Internal,
                },
                sampling_weight: None,
            }],
        }
    }

    fn fixture_span(
        tenant: &TenantId,
        trace_id: &str,
        span_id: &str,
        status: SpanStatus,
        latency_ms: i64,
        cost_micros: i64,
    ) -> CanonicalSpan {
        let started = Utc::now();
        CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "test".to_string(),
            tenant_id: tenant.clone(),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: beater_core::EnvironmentId::new("prod")
                .unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new(trace_id).unwrap_or_else(|err| panic!("{err}")),
            span_id: beater_core::SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "run".to_string(),
            status,
            start_time: started,
            end_time: Some(started + Duration::milliseconds(latency_ms)),
            model: None,
            cost: Some(Money::usd_micros(cost_micros)),
            tokens: Some(TokenCounts::default()),
            input_ref: None,
            output_ref: None,
            attributes: BTreeMap::new(),
            unmapped_attrs: serde_json::json!({}),
            raw_ref: ArtifactRef {
                artifact_id: beater_core::ArtifactId::new("raw")
                    .unwrap_or_else(|err| panic!("{err}")),
                uri: "artifact://tenant/project/raw".to_string(),
                sha256: beater_core::Sha256Hash::new("ab".repeat(32))
                    .unwrap_or_else(|err| panic!("{err}")),
                size_bytes: 2,
                mime_type: "application/json".to_string(),
                redaction_class: RedactionClass::Internal,
            },
            sampling_weight: None,
        }
    }

    #[test]
    fn tail_buffer_holds_spans_until_completion_then_samples_whole_trace() {
        use beater_schema::TraceCompletionState;

        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        // Sample nothing routinely + keep errors: the keep decision can only come
        // from the *late* error span, proving the whole trace is observed at tail.
        let policy = OnlineSamplingPolicy {
            sample_rate_per_mille: 0,
            keep_errors: true,
            slow_ms_threshold: None,
            high_cost_micros_threshold: None,
        };
        let mut buffer = TailBuffer::new(policy);
        let trace_id = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));

        // First (healthy) span arrives while the trace is still Open: it must be
        // held, never sampled.
        let held = buffer.offer(
            fixture_span(&tenant, "trace", "span-1", SpanStatus::Ok, 10, 0),
            &TraceCompletionState::Open,
        );
        assert_eq!(held, TailBufferOutcome::Buffered { buffered_spans: 1 });
        assert_eq!(buffer.open_traces(), 1);
        assert_eq!(buffer.buffered_spans(&trace_id), 1);

        // A second, still-Open span: still buffered, still no decision.
        let still_held = buffer.offer(
            fixture_span(&tenant, "trace", "span-2", SpanStatus::Ok, 10, 0),
            &TraceCompletionState::Open,
        );
        assert_eq!(
            still_held,
            TailBufferOutcome::Buffered { buffered_spans: 2 }
        );

        // The root ends and carries the error: now the trace is complete and the
        // accumulated three spans are sampled together as ErrorTrace.
        let outcome = buffer.offer(
            fixture_span(&tenant, "trace", "root", SpanStatus::Error, 10, 0),
            &TraceCompletionState::RootEnded,
        );
        match outcome {
            TailBufferOutcome::Sampled {
                decision,
                sampled_spans,
            } => {
                assert_eq!(sampled_spans, 3);
                assert!(decision.selected);
                assert_eq!(decision.reason, SamplingReason::ErrorTrace);
            }
            other => panic!("expected Sampled at completion, got {other:?}"),
        }
        // Trace evicted after sampling.
        assert_eq!(buffer.open_traces(), 0);
        assert_eq!(buffer.buffered_spans(&trace_id), 0);
    }

    #[test]
    fn tail_buffer_keeps_traces_independent_and_drops_uninteresting_completed_trace() {
        use beater_schema::TraceCompletionState;

        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let policy = OnlineSamplingPolicy {
            sample_rate_per_mille: 0,
            keep_errors: true,
            slow_ms_threshold: None,
            high_cost_micros_threshold: None,
        };
        let mut buffer = TailBuffer::new(policy);

        // Two interleaved traces buffer independently.
        buffer.offer(
            fixture_span(&tenant, "trace-a", "a1", SpanStatus::Ok, 1, 0),
            &TraceCompletionState::Open,
        );
        buffer.offer(
            fixture_span(&tenant, "trace-b", "b1", SpanStatus::Ok, 1, 0),
            &TraceCompletionState::Open,
        );
        assert_eq!(buffer.open_traces(), 2);

        // trace-a completes with no error/slow/cost signal and routine rate 0:
        // it is sampled (decided) but RoutineDropped.
        let outcome = buffer.offer(
            fixture_span(&tenant, "trace-a", "a2", SpanStatus::Ok, 1, 0),
            &TraceCompletionState::LateWindowClosed,
        );
        match outcome {
            TailBufferOutcome::Sampled { decision, .. } => {
                assert!(!decision.selected);
                assert_eq!(decision.reason, SamplingReason::RoutineDropped);
            }
            other => panic!("expected Sampled, got {other:?}"),
        }
        // trace-b is untouched and still open.
        assert_eq!(buffer.open_traces(), 1);
        assert_eq!(
            buffer.buffered_spans(&TraceId::new("trace-b").unwrap_or_else(|err| panic!("{err}"))),
            1
        );
    }

    /// ponytail guard: a trace whose completion never flips terminal would buffer
    /// forever. At the in-memory cap, opening one more trace force-flushes the
    /// *oldest* open trace and evicts it, so the open set stays bounded.
    #[test]
    fn tail_buffer_evicts_oldest_open_trace_at_the_cap() {
        use beater_schema::TraceCompletionState;

        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let policy = OnlineSamplingPolicy {
            sample_rate_per_mille: 0,
            keep_errors: true,
            slow_ms_threshold: None,
            high_cost_micros_threshold: None,
        };
        // Cap of 2 open traces. None of these ever report a terminal completion,
        // so without the guard they would accumulate without bound.
        let mut buffer = TailBuffer::new(policy).with_max_open_traces(2);
        let trace_a = TraceId::new("trace-a").unwrap_or_else(|err| panic!("{err}"));
        let trace_b = TraceId::new("trace-b").unwrap_or_else(|err| panic!("{err}"));
        let trace_c = TraceId::new("trace-c").unwrap_or_else(|err| panic!("{err}"));

        // trace-a (oldest) then trace-b: both within the cap, both buffered.
        assert_eq!(
            buffer.offer(
                fixture_span(&tenant, "trace-a", "a1", SpanStatus::Ok, 1, 0),
                &TraceCompletionState::Open,
            ),
            TailBufferOutcome::Buffered { buffered_spans: 1 }
        );
        assert_eq!(
            buffer.offer(
                fixture_span(&tenant, "trace-b", "b1", SpanStatus::Ok, 1, 0),
                &TraceCompletionState::Open,
            ),
            TailBufferOutcome::Buffered { buffered_spans: 1 }
        );
        assert_eq!(buffer.open_traces(), 2);

        // trace-c opens past the cap: the oldest open trace (trace-a) is force-
        // flushed and evicted. trace-c itself is buffered.
        let outcome = buffer.offer(
            fixture_span(&tenant, "trace-c", "c1", SpanStatus::Ok, 1, 0),
            &TraceCompletionState::Open,
        );
        match outcome {
            TailBufferOutcome::Evicted {
                buffered_spans,
                evicted_decision,
                evicted_spans,
            } => {
                assert_eq!(buffered_spans, 1, "trace-c span is held");
                assert_eq!(evicted_spans, 1, "trace-a's single span was flushed");
                // Routine rate 0 with no error/slow/cost signal -> dropped on flush.
                assert!(!evicted_decision.selected);
                assert_eq!(evicted_decision.reason, SamplingReason::RoutineDropped);
            }
            other => panic!("expected Evicted at the cap, got {other:?}"),
        }

        // Open set stayed bounded at the cap; the oldest (trace-a) was evicted while
        // the newer trace-b and trace-c remain.
        assert_eq!(buffer.open_traces(), 2);
        assert_eq!(buffer.buffered_spans(&trace_a), 0);
        assert_eq!(buffer.buffered_spans(&trace_b), 1);
        assert_eq!(buffer.buffered_spans(&trace_c), 1);
    }
}
