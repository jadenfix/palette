//! Orphaned-artifact sweeper.
//!
//! Reconciles a set of candidate artifact references against the artifacts still
//! referenced by live spans in a [`TraceStore`], deleting the bytes for any
//! candidate that is no longer reachable. This reclaims object-store space after
//! traces age out / are deleted from the hot store while their artifacts (raw
//! payloads, span inputs/outputs) linger.
//!
//! The sweep is conservative (delete only what is provably unreferenced) and
//! idempotent (delete of a missing object is a no-op), so it is safe to re-run
//! and safe to race with concurrent deletes.
//!
//! # Memory model
//!
//! The sweep is **incremental and memory-bounded**: it never materializes the
//! whole candidate set, the whole live span set, or a full in-memory index of
//! every referenced artifact URI. Instead it works one candidate *batch* at a
//! time:
//!
//! 1. A [`CandidateSource`] yields candidates a page at a time (so callers can
//!    stream an object-store listing rather than buffering it all).
//! 2. For each candidate batch (at most `batch_size` URIs) the sweeper streams
//!    the live spans of the tenant/project in pages of `batch_size`, loading one
//!    trace at a time, and keeps **only the subset of live URIs that also appear
//!    in the current candidate batch**. Everything else is discarded
//!    immediately, so the reconciliation working set is `O(batch_size)`
//!    regardless of how many traces, spans, or candidates exist in total.
//! 3. Candidates in the batch that are not referenced are deleted.
//!
//! This trades wall-time (the live spans are re-scanned per candidate batch,
//! with an early exit once every candidate in the batch is known to be live) for
//! bounded memory, exactly the budget split a maintenance job wants. Progress is
//! a single opaque candidate cursor, so a pass can be capped
//! ([`SweepConfig::max_batches_per_pass`]) and **resumed** later via the returned
//! [`SweepCheckpoint`] without restarting from the beginning.

use async_trait::async_trait;
use beater_core::{Page, PageRequest, ProjectId, TenantId};
use beater_schema::{ArtifactRef, CanonicalSpan, SpanFilter};
use beater_store::{ArtifactStore, StoreError, StoreResult, TraceStore};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default candidate/span page size for a reconciliation pass. Chosen to bound
/// the per-pass working set while still amortizing per-query overhead. Never
/// `u32::MAX`.
pub const DEFAULT_SWEEP_BATCH_SIZE: u32 = 256;

/// Tuning for a reconciliation pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SweepConfig {
    /// Page size for both candidate fetches and live-span scans. Bounds the
    /// per-pass working set. Always treated as at least `1`; never `u32::MAX`.
    pub batch_size: u32,
    /// Optional wall-time budget expressed as a maximum number of candidate
    /// batches to process before suspending. `None` runs the pass to completion.
    pub max_batches_per_pass: Option<u32>,
    /// When `true` (default) the [`SweepReport`] accumulates the URIs of deleted
    /// artifacts as an audit trail. This list is proportional to the number of
    /// *deletions*, not to the store size; set to `false` to keep the sweep
    /// strictly `O(batch_size)` when an audit trail is not needed.
    pub record_deleted_uris: bool,
}

impl Default for SweepConfig {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_SWEEP_BATCH_SIZE,
            max_batches_per_pass: None,
            record_deleted_uris: true,
        }
    }
}

impl SweepConfig {
    /// A run-to-completion config with the given batch size (minimum `1`).
    pub fn with_batch_size(batch_size: u32) -> Self {
        Self {
            batch_size,
            ..Self::default()
        }
    }

    /// Caps the pass at `max` candidate batches before it suspends.
    pub fn with_max_batches_per_pass(mut self, max: u32) -> Self {
        self.max_batches_per_pass = Some(max);
        self
    }

    fn effective_batch_size(&self) -> u32 {
        self.batch_size.max(1)
    }
}

/// Counters gathered during a sweep. Accumulated across batches (and across
/// resumed passes via the [`SweepCheckpoint`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SweepMetrics {
    /// Number of trace loads performed while scanning live references. Because
    /// the live spans are re-scanned per candidate batch, this reflects work
    /// done, not the number of distinct traces in the store.
    pub scanned_traces: u64,
    /// Number of span summaries iterated while scanning live references.
    pub scanned_spans: u64,
    /// Number of candidate artifacts examined (in and out of scope).
    pub candidate_count: u64,
    /// Number of candidate artifacts deleted as orphans.
    pub deleted: u64,
    /// Number of candidate artifacts retained because a live span referenced
    /// them.
    pub retained: u64,
    /// Number of non-fatal delete failures tolerated during the sweep.
    pub errors: u64,
    /// High-water mark of the reconciliation working set (candidate batch URIs
    /// plus the matched live subset). This is the proof of the memory bound: it
    /// scales with `batch_size`, never with total store size.
    pub peak_tracked_uris: usize,
    /// Wall-clock time spent across all passes contributing to this report.
    pub duration: Duration,
}

/// Outcome of a completed sweep.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SweepReport {
    /// Counters for the sweep.
    pub metrics: SweepMetrics,
    /// URIs of the artifacts that were deleted, sorted. Empty when
    /// [`SweepConfig::record_deleted_uris`] is `false`.
    pub deleted_uris: Vec<String>,
}

/// Opaque, resumable progress for a sweep that was suspended before completion.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SweepCheckpoint {
    /// Cursor into the candidate source for the next unprocessed batch.
    candidate_cursor: Option<String>,
    /// Metrics accumulated so far.
    metrics: SweepMetrics,
    /// Deleted-URI audit trail accumulated so far.
    deleted_uris: Vec<String>,
}

impl SweepCheckpoint {
    /// Metrics accumulated up to this checkpoint.
    pub fn metrics(&self) -> &SweepMetrics {
        &self.metrics
    }

    /// Whether any progress has been made (i.e. at least one batch processed).
    pub fn started(&self) -> bool {
        self.candidate_cursor.is_some() || self.metrics != SweepMetrics::default()
    }
}

/// Result of a single (possibly budget-capped) sweep pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SweepOutcome {
    /// The sweep finished; the full report is available.
    Completed(SweepReport),
    /// The pass hit its budget; resume with this checkpoint.
    Suspended(SweepCheckpoint),
}

/// A paged source of candidate artifacts to reconcile.
///
/// Implementations stream candidates a page at a time, so the sweeper never has
/// to buffer the whole set. The `cursor` is opaque to the sweeper and is the
/// only state persisted in a [`SweepCheckpoint`], so a source must be able to
/// resume from a previously returned cursor.
#[async_trait]
pub trait CandidateSource: Send {
    /// Fetch the next page of candidates. `cursor` is `None` for the first page
    /// and otherwise the `next_cursor` from the previous page. The returned
    /// page's `next_cursor` is `None` when the source is exhausted.
    async fn fetch(&mut self, limit: u32, cursor: Option<String>)
        -> StoreResult<Page<ArtifactRef>>;
}

/// A [`CandidateSource`] backed by an in-memory slice, using a numeric offset as
/// its cursor. Useful for tests and for callers that already hold the full set
/// but still want batched, resumable reconciliation.
pub struct SliceCandidateSource {
    items: Vec<ArtifactRef>,
}

impl SliceCandidateSource {
    pub fn new(items: impl Into<Vec<ArtifactRef>>) -> Self {
        Self {
            items: items.into(),
        }
    }
}

#[async_trait]
impl CandidateSource for SliceCandidateSource {
    async fn fetch(
        &mut self,
        limit: u32,
        cursor: Option<String>,
    ) -> StoreResult<Page<ArtifactRef>> {
        let limit = limit.max(1) as usize;
        let offset = match cursor {
            Some(cursor) => cursor
                .parse::<usize>()
                .map_err(|err| StoreError::backend(format!("invalid candidate cursor: {err}")))?,
            None => 0,
        };
        if offset >= self.items.len() {
            return Ok(Page::new(Vec::new(), None));
        }
        let end = offset.saturating_add(limit).min(self.items.len());
        let items = self.items[offset..end].to_vec();
        let next_cursor = if end < self.items.len() {
            Some(end.to_string())
        } else {
            None
        };
        Ok(Page::new(items, next_cursor))
    }
}

/// Deletes object-store artifacts that are no longer referenced by any live span.
pub struct OrphanedArtifactSweeper {
    artifacts: Arc<dyn ArtifactStore>,
}

impl OrphanedArtifactSweeper {
    pub fn new(artifacts: Arc<dyn ArtifactStore>) -> Self {
        Self { artifacts }
    }

    /// Reconciles every candidate from `candidates` against the artifacts
    /// referenced by the live spans of `tenant` (optionally scoped to `project`)
    /// in `trace_store`, deleting any candidate whose URI is not referenced.
    ///
    /// `candidates` is the set of artifacts known to exist in the object store
    /// (e.g. an object-store listing). Candidates outside the requested
    /// tenant/project scope are ignored; in-scope candidates not referenced by a
    /// live span are treated as orphaned and deleted.
    ///
    /// Runs to completion regardless of [`SweepConfig::max_batches_per_pass`].
    /// Use [`Self::sweep_pass`] for budget-capped, resumable passes.
    pub async fn sweep(
        &self,
        trace_store: &dyn TraceStore,
        tenant: TenantId,
        project: Option<ProjectId>,
        candidates: &mut dyn CandidateSource,
        config: SweepConfig,
    ) -> StoreResult<SweepReport> {
        let config = SweepConfig {
            max_batches_per_pass: None,
            ..config
        };
        match self
            .sweep_pass(trace_store, tenant, project, candidates, config, None)
            .await?
        {
            SweepOutcome::Completed(report) => Ok(report),
            SweepOutcome::Suspended(_) => Err(StoreError::backend(
                "sweep suspended despite an unbounded pass",
            )),
        }
    }

    /// Convenience wrapper over [`Self::sweep`] for callers that already hold the
    /// full candidate slice but still want batched, memory-bounded reconciliation.
    pub async fn sweep_slice(
        &self,
        trace_store: &dyn TraceStore,
        tenant: TenantId,
        project: Option<ProjectId>,
        candidates: &[ArtifactRef],
        config: SweepConfig,
    ) -> StoreResult<SweepReport> {
        let mut source = SliceCandidateSource::new(candidates.to_vec());
        self.sweep(trace_store, tenant, project, &mut source, config)
            .await
    }

    /// Runs a single, optionally budget-capped reconciliation pass.
    ///
    /// Resume a previously suspended sweep by threading its [`SweepCheckpoint`]
    /// back in via `checkpoint`. With `config.max_batches_per_pass == Some(n)`
    /// the pass processes at most `n` candidate batches before returning
    /// [`SweepOutcome::Suspended`]; otherwise it returns
    /// [`SweepOutcome::Completed`].
    pub async fn sweep_pass(
        &self,
        trace_store: &dyn TraceStore,
        tenant: TenantId,
        project: Option<ProjectId>,
        candidates: &mut dyn CandidateSource,
        config: SweepConfig,
        checkpoint: Option<SweepCheckpoint>,
    ) -> StoreResult<SweepOutcome> {
        let batch_size = config.effective_batch_size();
        let mut state = checkpoint.unwrap_or_default();
        let start = Instant::now();
        let mut batches_this_pass: u32 = 0;

        loop {
            if let Some(max) = config.max_batches_per_pass {
                if batches_this_pass >= max {
                    state.metrics.duration += start.elapsed();
                    return Ok(SweepOutcome::Suspended(state));
                }
            }

            let page = candidates
                .fetch(batch_size, state.candidate_cursor.clone())
                .await?;
            let is_last = page.next_cursor.is_none();

            self.reconcile_batch(
                trace_store,
                &tenant,
                project.as_ref(),
                &page.items,
                &config,
                &mut state,
            )
            .await?;

            batches_this_pass = batches_this_pass.saturating_add(1);

            if is_last {
                if config.record_deleted_uris {
                    state.deleted_uris.sort();
                    state.deleted_uris.dedup();
                }
                state.metrics.duration += start.elapsed();
                return Ok(SweepOutcome::Completed(SweepReport {
                    metrics: state.metrics,
                    deleted_uris: state.deleted_uris,
                }));
            }

            state.candidate_cursor = page.next_cursor;
        }
    }

    /// Reconciles a single candidate batch in bounded memory: builds the set of
    /// in-scope candidate URIs, finds which of *those* are referenced by a live
    /// span (keeping only that subset), and deletes the rest.
    async fn reconcile_batch(
        &self,
        trace_store: &dyn TraceStore,
        tenant: &TenantId,
        project: Option<&ProjectId>,
        candidates: &[ArtifactRef],
        config: &SweepConfig,
        state: &mut SweepCheckpoint,
    ) -> StoreResult<()> {
        // In-scope candidate URIs only; bounded by the batch size.
        let mut pending = BTreeSet::new();
        for candidate in candidates {
            if artifact_uri_in_scope(&candidate.uri, tenant, project) {
                pending.insert(candidate.uri.clone());
            }
        }

        let referenced = self
            .referenced_subset(
                trace_store,
                tenant,
                project,
                &pending,
                config.effective_batch_size(),
                &mut state.metrics,
            )
            .await?;

        // Working-set high-water mark: candidate batch + matched live subset.
        // Both are bounded by `batch_size`, never by total store size.
        let working_set = pending.len().saturating_add(referenced.len());
        state.metrics.peak_tracked_uris = state.metrics.peak_tracked_uris.max(working_set);

        for candidate in candidates {
            state.metrics.candidate_count = state.metrics.candidate_count.saturating_add(1);
            if !artifact_uri_in_scope(&candidate.uri, tenant, project) {
                continue;
            }
            if referenced.contains(&candidate.uri) {
                state.metrics.retained = state.metrics.retained.saturating_add(1);
                continue;
            }
            match self.artifacts.delete_bytes(candidate).await {
                Ok(()) => {
                    state.metrics.deleted = state.metrics.deleted.saturating_add(1);
                    if config.record_deleted_uris {
                        state.deleted_uris.push(candidate.uri.clone());
                    }
                }
                Err(_) => {
                    // A maintenance sweep must be resilient: one object that
                    // refuses to delete should not abort the whole pass. Count
                    // it and continue; the next sweep retries (delete is
                    // idempotent).
                    state.metrics.errors = state.metrics.errors.saturating_add(1);
                }
            }
        }

        Ok(())
    }

    /// Returns the subset of `pending` URIs that are referenced by at least one
    /// live span, streaming the live spans in pages so memory stays bounded by
    /// `batch_size`. Only URIs already in `pending` are ever retained; every
    /// other referenced URI is discarded immediately. Exits early once every
    /// candidate in `pending` is known to be live.
    async fn referenced_subset(
        &self,
        trace_store: &dyn TraceStore,
        tenant: &TenantId,
        project: Option<&ProjectId>,
        pending: &BTreeSet<String>,
        batch_size: u32,
        metrics: &mut SweepMetrics,
    ) -> StoreResult<BTreeSet<String>> {
        let mut referenced = BTreeSet::new();
        if pending.is_empty() {
            return Ok(referenced);
        }

        let mut cursor = None;
        loop {
            let page = trace_store
                .query_spans(
                    tenant.clone(),
                    SpanFilter {
                        project_id: project.cloned(),
                        ..SpanFilter::default()
                    },
                    PageRequest {
                        limit: batch_size,
                        cursor,
                    },
                )
                .await?;
            metrics.scanned_spans = metrics
                .scanned_spans
                .saturating_add(page.items.len() as u64);

            // Dedupe trace ids within the page (bounded by batch size) so we do
            // not load the same trace twice for adjacent spans.
            let mut trace_ids = BTreeSet::new();
            for summary in &page.items {
                trace_ids.insert(summary.trace_id.clone());
            }

            for trace_id in trace_ids {
                let view = match project {
                    Some(project) => {
                        trace_store
                            .get_project_trace(tenant.clone(), project.clone(), trace_id)
                            .await?
                    }
                    None => trace_store.get_trace(tenant.clone(), trace_id).await?,
                };
                metrics.scanned_traces = metrics.scanned_traces.saturating_add(1);
                retain_referenced(&view.spans, pending, &mut referenced);
                if referenced.len() == pending.len() {
                    // Every candidate in this batch is live; nothing left to
                    // confirm, so stop scanning.
                    return Ok(referenced);
                }
            }

            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        Ok(referenced)
    }
}

fn artifact_uri_in_scope(uri: &str, tenant: &TenantId, project: Option<&ProjectId>) -> bool {
    let Some(path) = uri.strip_prefix("artifact://") else {
        return false;
    };
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() != 3
        || segments
            .iter()
            .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return false;
    }
    if segments[0] != tenant.as_str() {
        return false;
    }
    match project {
        Some(project) => segments[1] == project.as_str(),
        None => true,
    }
}

/// Inserts into `referenced` every URI referenced by `spans` that is present in
/// `pending`. URIs not in `pending` are ignored, keeping memory bounded by the
/// candidate batch.
fn retain_referenced(
    spans: &[CanonicalSpan],
    pending: &BTreeSet<String>,
    referenced: &mut BTreeSet<String>,
) {
    let mut consider = |uri: &str| {
        if pending.contains(uri) {
            referenced.insert(uri.to_string());
        }
    };
    for span in spans {
        consider(&span.raw_ref.uri);
        if let Some(input) = &span.input_ref {
            consider(&input.uri);
        }
        if let Some(output) = &span.output_ref {
            consider(&output.uri);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{ArtifactId, EnvironmentId, IdempotencyKey, Sha256Hash, SpanId, TraceId};
    use beater_schema::{
        AgentSpanKind, CanonicalTraceBatch, RawEnvelope, RedactionClass, RunFilter, RunSummary,
        SpanStatus, SpanSummary, TraceView, WriteAck,
    };
    use chrono::Utc;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    fn tenant_id() -> TenantId {
        TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"))
    }

    fn project_id() -> ProjectId {
        ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"))
    }

    fn artifact(uri: &str) -> ArtifactRef {
        ArtifactRef {
            artifact_id: ArtifactId::new(uri.replace("://", "-").replace('/', "-"))
                .unwrap_or_else(|err| panic!("{err}")),
            uri: uri.to_string(),
            sha256: Sha256Hash::new("test-sha").unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 1,
            mime_type: "application/octet-stream".to_string(),
            redaction_class: RedactionClass::Internal,
        }
    }

    /// A trace consisting of a single span whose `raw_ref` is `uri`.
    fn trace_with_uri(trace: &str, uri: &str) -> CanonicalSpan {
        let tenant = tenant_id();
        let project = project_id();
        CanonicalSpan {
            schema_version: 1,
            normalizer_version: "test".to_string(),
            tenant_id: tenant,
            project_id: project,
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new(trace).unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new(format!("span-{trace}")).unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq: 0,
            kind: AgentSpanKind::ToolCall,
            name: format!("span-{trace}"),
            status: SpanStatus::Ok,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            model: None,
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref: None,
            attributes: BTreeMap::new(),
            unmapped_attrs: serde_json::Value::Null,
            raw_ref: artifact(uri),
        }
    }

    /// A [`TraceStore`] that serves a fixed set of single-span traces and records
    /// the largest `PageRequest::limit` it was ever asked for, so tests can prove
    /// the sweeper never requests `u32::MAX` (or more than `batch_size`).
    struct FakeTraceStore {
        spans: Vec<CanonicalSpan>,
        max_limit_seen: Mutex<u32>,
    }

    impl FakeTraceStore {
        fn new(spans: Vec<CanonicalSpan>) -> Self {
            Self {
                spans,
                max_limit_seen: Mutex::new(0),
            }
        }

        fn max_limit_seen(&self) -> u32 {
            *self
                .max_limit_seen
                .lock()
                .unwrap_or_else(|err| panic!("{err}"))
        }

        fn view_for(&self, trace: &TraceId) -> StoreResult<TraceView> {
            let spans: Vec<CanonicalSpan> = self
                .spans
                .iter()
                .filter(|span| &span.trace_id == trace)
                .cloned()
                .collect();
            if spans.is_empty() {
                return Err(StoreError::NotFound(trace.as_str().to_string()));
            }
            Ok(TraceView {
                tenant_id: tenant_id(),
                trace_id: trace.clone(),
                spans,
            })
        }
    }

    #[async_trait]
    impl TraceStore for FakeTraceStore {
        async fn write_batch(&self, _batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
            Err(StoreError::backend("write_batch not used in sweeper tests"))
        }

        async fn get_trace(&self, _tenant: TenantId, trace: TraceId) -> StoreResult<TraceView> {
            self.view_for(&trace)
        }

        async fn get_project_trace(
            &self,
            _tenant: TenantId,
            _project: ProjectId,
            trace: TraceId,
        ) -> StoreResult<TraceView> {
            self.view_for(&trace)
        }

        async fn get_raw_envelope(
            &self,
            _tenant: TenantId,
            _project: ProjectId,
            _idempotency_key: IdempotencyKey,
        ) -> StoreResult<Option<RawEnvelope>> {
            Err(StoreError::backend(
                "get_raw_envelope not used in sweeper tests",
            ))
        }

        async fn query_runs(
            &self,
            _tenant: TenantId,
            _filter: RunFilter,
            _page: PageRequest,
        ) -> StoreResult<Page<RunSummary>> {
            Err(StoreError::backend("query_runs not used in sweeper tests"))
        }

        async fn query_spans(
            &self,
            _tenant: TenantId,
            _filter: SpanFilter,
            page: PageRequest,
        ) -> StoreResult<Page<SpanSummary>> {
            {
                let mut max = self
                    .max_limit_seen
                    .lock()
                    .unwrap_or_else(|err| panic!("{err}"));
                *max = (*max).max(page.limit);
            }
            let limit = page.limit.max(1) as usize;
            let offset = match &page.cursor {
                Some(cursor) => cursor
                    .parse::<usize>()
                    .map_err(|err| StoreError::backend(format!("bad cursor: {err}")))?,
                None => 0,
            };
            if offset >= self.spans.len() {
                return Ok(Page::new(Vec::new(), None));
            }
            let end = offset.saturating_add(limit).min(self.spans.len());
            let items: Vec<SpanSummary> = self.spans[offset..end]
                .iter()
                .map(|span| SpanSummary {
                    tenant_id: span.tenant_id.clone(),
                    project_id: span.project_id.clone(),
                    trace_id: span.trace_id.clone(),
                    span_id: span.span_id.clone(),
                    kind: span.kind.clone(),
                    name: span.name.clone(),
                    status: span.status.clone(),
                    started_at: span.start_time,
                    ended_at: span.end_time,
                    model: span.model.clone(),
                    cost: span.cost.clone(),
                    release_id: None,
                })
                .collect();
            let next_cursor = if end < self.spans.len() {
                Some(end.to_string())
            } else {
                None
            };
            Ok(Page::new(items, next_cursor))
        }
    }

    /// An [`ArtifactStore`] that records deletes and can be told to fail a
    /// specific URI's delete (to exercise the error path).
    #[derive(Default)]
    struct RecordingArtifactStore {
        deleted: Mutex<Vec<String>>,
        fail_uri: Option<String>,
    }

    impl RecordingArtifactStore {
        fn failing(uri: &str) -> Self {
            Self {
                deleted: Mutex::new(Vec::new()),
                fail_uri: Some(uri.to_string()),
            }
        }

        fn deleted(&self) -> Vec<String> {
            self.deleted
                .lock()
                .unwrap_or_else(|err| panic!("{err}"))
                .clone()
        }
    }

    #[async_trait]
    impl ArtifactStore for RecordingArtifactStore {
        async fn put_bytes(
            &self,
            _tenant_id: &TenantId,
            _project_id: &ProjectId,
            _mime_type: &str,
            _redaction_class: RedactionClass,
            _bytes: &[u8],
        ) -> StoreResult<ArtifactRef> {
            Err(StoreError::backend("put_bytes not used in sweeper tests"))
        }

        async fn get_bytes(&self, _artifact_ref: &ArtifactRef) -> StoreResult<Vec<u8>> {
            Err(StoreError::backend("get_bytes not used in sweeper tests"))
        }

        async fn delete_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<()> {
            if self.fail_uri.as_deref() == Some(artifact_ref.uri.as_str()) {
                return Err(StoreError::backend("simulated delete failure"));
            }
            self.deleted
                .lock()
                .unwrap_or_else(|err| panic!("{err}"))
                .push(artifact_ref.uri.clone());
            Ok(())
        }
    }

    #[test]
    fn artifact_uri_scope_uses_exact_path_segments() {
        let tenant = tenant_id();
        let other_tenant = TenantId::new("tenant-other").unwrap_or_else(|err| panic!("{err}"));
        let project = project_id();

        assert!(artifact_uri_in_scope(
            "artifact://tenant/project/artifact",
            &tenant,
            Some(&project)
        ));
        assert!(artifact_uri_in_scope(
            "artifact://tenant/other-project/artifact",
            &tenant,
            None
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/projectile/artifact",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant-other/project/artifact",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/project/artifact/extra",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/project",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/../project",
            &tenant,
            None
        ));
        assert!(!artifact_uri_in_scope(
            "https://tenant/project/artifact",
            &tenant,
            Some(&project)
        ));

        let slash_tenant = TenantId::new("tenant/slash").unwrap_or_else(|err| panic!("{err}"));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/slash/project/artifact",
            &slash_tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/project/artifact",
            &other_tenant,
            None
        ));
    }

    // Acceptance: candidates can be streamed/paged, orphans deleted, live
    // retained, and metrics report scanned/candidate/delete/retain counts plus
    // duration. Also exercises out-of-scope skipping.
    #[tokio::test]
    async fn sweep_deletes_orphans_retains_live_and_reports_metrics() {
        let live = trace_with_uri("trace-live", "artifact://tenant/project/live");
        let store = FakeTraceStore::new(vec![live]);
        let artifacts = Arc::new(RecordingArtifactStore::default());
        let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());

        let candidates = vec![
            artifact("artifact://tenant/project/live"),
            artifact("artifact://tenant/project/orphan-a"),
            artifact("artifact://tenant/project/orphan-b"),
            // out of scope -> neither deleted nor retained
            artifact("artifact://tenant/other-project/orphan"),
            artifact("https://tenant/project/orphan"),
        ];

        let report = sweeper
            .sweep_slice(
                &store,
                tenant_id(),
                Some(project_id()),
                &candidates,
                SweepConfig::with_batch_size(2),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.metrics.deleted, 2);
        assert_eq!(report.metrics.retained, 1);
        assert_eq!(report.metrics.candidate_count, 5);
        assert_eq!(report.metrics.errors, 0);
        assert!(report.metrics.scanned_spans >= 1);
        assert!(report.metrics.scanned_traces >= 1);
        assert_eq!(
            report.deleted_uris,
            vec![
                "artifact://tenant/project/orphan-a".to_string(),
                "artifact://tenant/project/orphan-b".to_string(),
            ]
        );
        assert_eq!(
            artifacts.deleted().into_iter().collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "artifact://tenant/project/orphan-a".to_string(),
                "artifact://tenant/project/orphan-b".to_string(),
            ])
        );
        // Never u32::MAX, and never more than the batch size.
        assert_eq!(store.max_limit_seen(), 2);
        assert_ne!(store.max_limit_seen(), u32::MAX);
    }

    // Acceptance: tenant-wide sweep (no project) reconciles across projects.
    #[tokio::test]
    async fn sweep_tenant_wide_without_project_scope() {
        let live_a = trace_with_uri("trace-a", "artifact://tenant/project/live-a");
        let live_b = trace_with_uri("trace-b", "artifact://tenant/other/live-b");
        let store = FakeTraceStore::new(vec![live_a, live_b]);
        let artifacts = Arc::new(RecordingArtifactStore::default());
        let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());

        let candidates = vec![
            artifact("artifact://tenant/project/live-a"),
            artifact("artifact://tenant/other/live-b"),
            artifact("artifact://tenant/project/orphan"),
        ];

        let report = sweeper
            .sweep_slice(
                &store,
                tenant_id(),
                None,
                &candidates,
                SweepConfig::with_batch_size(4),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.metrics.deleted, 1);
        assert_eq!(report.metrics.retained, 2);
        assert_eq!(
            report.deleted_uris,
            vec!["artifact://tenant/project/orphan"]
        );
    }

    // Acceptance: a delete failure is tolerated and counted in `errors`, and the
    // sweep keeps making progress.
    #[tokio::test]
    async fn sweep_counts_delete_errors_and_continues() {
        let store = FakeTraceStore::new(vec![]);
        let artifacts = Arc::new(RecordingArtifactStore::failing(
            "artifact://tenant/project/orphan-a",
        ));
        let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());

        let candidates = vec![
            artifact("artifact://tenant/project/orphan-a"),
            artifact("artifact://tenant/project/orphan-b"),
        ];

        let report = sweeper
            .sweep_slice(
                &store,
                tenant_id(),
                Some(project_id()),
                &candidates,
                SweepConfig::with_batch_size(8),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.metrics.errors, 1);
        assert_eq!(report.metrics.deleted, 1);
        assert_eq!(
            report.deleted_uris,
            vec!["artifact://tenant/project/orphan-b"]
        );
        assert_eq!(
            artifacts.deleted(),
            vec!["artifact://tenant/project/orphan-b"]
        );
    }

    // Acceptance: a sweep can checkpoint and resume without restarting, and the
    // resumed result matches a single run-to-completion sweep.
    #[tokio::test]
    async fn sweep_checkpoints_and_resumes() {
        let live = trace_with_uri("trace-live", "artifact://tenant/project/live");
        let store = FakeTraceStore::new(vec![live]);

        let candidates: Vec<ArtifactRef> = (0..10)
            .map(|i| artifact(&format!("artifact://tenant/project/orphan-{i:02}")))
            .chain(std::iter::once(artifact("artifact://tenant/project/live")))
            .collect();

        // One-shot reference run.
        let oneshot = {
            let artifacts = Arc::new(RecordingArtifactStore::default());
            let sweeper = OrphanedArtifactSweeper::new(artifacts);
            sweeper
                .sweep_slice(
                    &store,
                    tenant_id(),
                    Some(project_id()),
                    &candidates,
                    SweepConfig::with_batch_size(3),
                )
                .await
                .unwrap_or_else(|err| panic!("{err}"))
        };

        // Resumable run: one batch at a time.
        let artifacts = Arc::new(RecordingArtifactStore::default());
        let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());
        let mut source = SliceCandidateSource::new(candidates.clone());
        let config = SweepConfig::with_batch_size(3).with_max_batches_per_pass(1);

        let mut checkpoint = None;
        let mut passes = 0;
        let report = loop {
            passes += 1;
            assert!(passes < 100, "resume loop did not terminate");
            match sweeper
                .sweep_pass(
                    &store,
                    tenant_id(),
                    Some(project_id()),
                    &mut source,
                    config,
                    checkpoint.take(),
                )
                .await
                .unwrap_or_else(|err| panic!("{err}"))
            {
                SweepOutcome::Completed(report) => break report,
                SweepOutcome::Suspended(cp) => {
                    // Progress must advance, not restart.
                    assert!(cp.started());
                    checkpoint = Some(cp);
                }
            }
        };

        // 11 candidates / batch of 3 => at least 4 passes were needed.
        assert!(
            passes >= 4,
            "expected multiple resumed passes, got {passes}"
        );
        assert_eq!(report.metrics.deleted, oneshot.metrics.deleted);
        assert_eq!(report.metrics.retained, oneshot.metrics.retained);
        assert_eq!(
            report.metrics.candidate_count,
            oneshot.metrics.candidate_count
        );
        assert_eq!(report.deleted_uris, oneshot.deleted_uris);
        assert_eq!(report.metrics.deleted, 10);
        assert_eq!(report.metrics.retained, 1);
    }

    // Acceptance: memory (peak reconciliation working set) scales with the batch
    // size, not the total store size. We sweep two stores that differ 25x in size
    // with the same batch size and assert the peak tracked set is bounded by the
    // batch, identical across both, and never grows with the number of
    // traces/candidates.
    //
    // Both stores are sized strictly larger than `batch_size` (in both live
    // traces and candidates) so the working set actually saturates its bound on
    // at least one full batch: this is what makes the comparison a real proof
    // rather than a vacuous one over a store too small to fill a batch.
    #[tokio::test]
    async fn peak_memory_scales_with_batch_not_store_size() {
        async fn peak_for(num_live: usize, batch_size: u32) -> usize {
            let live_spans: Vec<CanonicalSpan> = (0..num_live)
                .map(|i| {
                    trace_with_uri(
                        &format!("trace-{i:04}"),
                        &format!("artifact://tenant/project/live-{i:04}"),
                    )
                })
                .collect();
            let store = FakeTraceStore::new(live_spans);

            // Candidates: all live URIs plus an equal number of orphans, so the
            // candidate set also grows with the store.
            let mut candidates: Vec<ArtifactRef> = (0..num_live)
                .map(|i| artifact(&format!("artifact://tenant/project/live-{i:04}")))
                .collect();
            candidates.extend(
                (0..num_live)
                    .map(|i| artifact(&format!("artifact://tenant/project/orphan-{i:04}"))),
            );

            let artifacts = Arc::new(RecordingArtifactStore::default());
            let sweeper = OrphanedArtifactSweeper::new(artifacts);
            let report = sweeper
                .sweep_slice(
                    &store,
                    tenant_id(),
                    Some(project_id()),
                    &candidates,
                    SweepConfig::with_batch_size(batch_size),
                )
                .await
                .unwrap_or_else(|err| panic!("{err}"));

            // Sanity: it actually did the work.
            assert_eq!(report.metrics.deleted as usize, num_live);
            assert_eq!(report.metrics.retained as usize, num_live);
            // Page requests never exceeded the batch size (and never u32::MAX).
            assert_eq!(store.max_limit_seen(), batch_size);
            report.metrics.peak_tracked_uris
        }

        let batch_size = 4u32;
        // Both stores hold strictly more live traces than `batch_size`, so the
        // first (all-live) candidate batch fully saturates the working set. The
        // large store is 25x the small one.
        let small_live = (batch_size * 2) as usize; // 8 live + 8 orphans = 16 candidates
        let large_live = small_live * 25; // 200 live + 200 orphans = 400 candidates
        let small = peak_for(small_live, batch_size).await;
        let large = peak_for(large_live, batch_size).await;

        // The working-set high-water mark is bounded by the batch (candidate
        // batch + matched subset <= 2 * batch_size).
        let bound = (2 * batch_size) as usize;
        assert!(small <= bound, "small peak {small} exceeded bound {bound}");
        assert!(large <= bound, "large peak {large} exceeded bound {bound}");
        // Both stores are large enough to saturate the bound, so the peak is not
        // vacuously small: it genuinely reaches `2 * batch_size`.
        assert_eq!(large, bound, "large peak must saturate the batch bound");
        // The crux: a 25x-larger store yields the *identical* peak working set.
        assert_eq!(
            small, large,
            "peak working set must not grow with store size"
        );
        // And it is dwarfed by the candidate set (400 candidates in the large run).
        assert!(large < large_live, "peak {large} must not track store size");
    }

    // Acceptance: the streamed candidate source is genuinely paged - the source
    // is asked for pages of at most `batch_size` and reconciliation still works.
    #[tokio::test]
    async fn candidate_source_is_paged() {
        struct CountingSource {
            inner: SliceCandidateSource,
            max_page_len: Arc<Mutex<usize>>,
            fetches: Arc<Mutex<usize>>,
        }

        #[async_trait]
        impl CandidateSource for CountingSource {
            async fn fetch(
                &mut self,
                limit: u32,
                cursor: Option<String>,
            ) -> StoreResult<Page<ArtifactRef>> {
                *self.fetches.lock().unwrap_or_else(|err| panic!("{err}")) += 1;
                let page = self.inner.fetch(limit, cursor).await?;
                let mut max = self
                    .max_page_len
                    .lock()
                    .unwrap_or_else(|err| panic!("{err}"));
                *max = (*max).max(page.items.len());
                Ok(page)
            }
        }

        let store = FakeTraceStore::new(vec![]);
        let artifacts = Arc::new(RecordingArtifactStore::default());
        let sweeper = OrphanedArtifactSweeper::new(artifacts);

        let candidates: Vec<ArtifactRef> = (0..20)
            .map(|i| artifact(&format!("artifact://tenant/project/orphan-{i:02}")))
            .collect();
        let max_page_len = Arc::new(Mutex::new(0));
        let fetches = Arc::new(Mutex::new(0));
        let mut source = CountingSource {
            inner: SliceCandidateSource::new(candidates),
            max_page_len: max_page_len.clone(),
            fetches: fetches.clone(),
        };

        let report = sweeper
            .sweep(
                &store,
                tenant_id(),
                Some(project_id()),
                &mut source,
                SweepConfig::with_batch_size(5),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.metrics.deleted, 20);
        assert!(*max_page_len.lock().unwrap_or_else(|err| panic!("{err}")) <= 5);
        // 20 candidates / 5 per page => at least 4 fetches (paged, not one slice).
        assert!(*fetches.lock().unwrap_or_else(|err| panic!("{err}")) >= 4);
    }

    // record_deleted_uris=false keeps the audit list empty (strict O(batch)).
    #[tokio::test]
    async fn deleted_uris_can_be_suppressed() {
        let store = FakeTraceStore::new(vec![]);
        let artifacts = Arc::new(RecordingArtifactStore::default());
        let sweeper = OrphanedArtifactSweeper::new(artifacts);

        let candidates = vec![artifact("artifact://tenant/project/orphan")];
        let config = SweepConfig {
            record_deleted_uris: false,
            ..SweepConfig::with_batch_size(4)
        };
        let report = sweeper
            .sweep_slice(&store, tenant_id(), Some(project_id()), &candidates, config)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.metrics.deleted, 1);
        assert!(report.deleted_uris.is_empty());
    }
}
