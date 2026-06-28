use async_trait::async_trait;
use beater_core::{
    EnvironmentId, IdempotencyKey, Money, OrganizationId, Page, PageRequest, ProjectId, TenantId,
    Timestamp, TraceId,
};
use beater_schema::{
    filter_run_summaries, roll_up_runs, AgentSpanKind, ArtifactRef, CanonicalTraceBatch, ModelRef,
    RawEnvelope, RunFilter, RunSummary, SpanFilter, SpanStatus, SpanSummary, TraceView, WriteAck,
};
use std::collections::BTreeSet;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum StoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("backpressure: {0}")]
    Backpressure(String),
    #[error("limit exceeded: {0}")]
    LimitExceeded(String),
    #[error("integrity error: {0}")]
    Integrity(String),
    #[error("backend error: {0}")]
    Backend(String),
}

impl StoreError {
    pub fn backend(error: impl std::fmt::Display) -> Self {
        Self::Backend(error.to_string())
    }

    pub fn integrity(error: impl std::fmt::Display) -> Self {
        Self::Integrity(error.to_string())
    }
}

/// Lock a [`Mutex`](std::sync::Mutex), mapping a poisoned lock into a
/// [`StoreError::backend`] tagged with `what` (e.g. `"sqlite connection"`) so the
/// originating store is identifiable in the message. Centralizes the
/// `lock().map_err(...)` boilerplate every backend's guard accessor repeated.
pub fn lock_poisoned<'a, T>(
    mutex: &'a std::sync::Mutex<T>,
    what: &str,
) -> StoreResult<std::sync::MutexGuard<'a, T>> {
    mutex
        .lock()
        .map_err(|err| StoreError::backend(format!("{what} mutex poisoned: {err}")))
}

/// Maps a fallible `anyhow` result into a [`StoreResult`] backend error.
pub trait IntoStoreResult<T> {
    fn into_store(self) -> StoreResult<T>;
}

impl<T> IntoStoreResult<T> for anyhow::Result<T> {
    fn into_store(self) -> StoreResult<T> {
        self.map_err(StoreError::backend)
    }
}

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn put_bytes(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        mime_type: &str,
        redaction_class: beater_schema::RedactionClass,
        bytes: &[u8],
    ) -> StoreResult<ArtifactRef>;

    async fn get_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<Vec<u8>>;

    /// Deletes the bytes backing an artifact reference.
    ///
    /// Deleting an artifact that does not exist is a no-op and succeeds, so the
    /// orphaned-artifact sweeper can be safely re-run (it may race a previous
    /// sweep or a concurrent delete). Implementors should treat a missing object
    /// as already-deleted rather than an error.
    async fn delete_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<()>;
}

#[async_trait]
pub trait TraceStore: Send + Sync {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck>;

    async fn get_trace(&self, tenant: TenantId, trace: TraceId) -> StoreResult<TraceView>;

    async fn get_project_trace(
        &self,
        tenant: TenantId,
        project: ProjectId,
        trace: TraceId,
    ) -> StoreResult<TraceView>;

    async fn get_raw_envelope(
        &self,
        tenant: TenantId,
        project: ProjectId,
        idempotency_key: IdempotencyKey,
    ) -> StoreResult<Option<RawEnvelope>>;

    async fn query_runs(
        &self,
        tenant: TenantId,
        filter: RunFilter,
        page: PageRequest,
    ) -> StoreResult<Page<RunSummary>>;

    async fn query_spans(
        &self,
        tenant: TenantId,
        filter: SpanFilter,
        page: PageRequest,
    ) -> StoreResult<Page<SpanSummary>>;
}

/// Development fallback for trace stores that cannot aggregate run summaries in
/// the backend. Production OLAP stores should implement `query_runs` with
/// backend aggregation instead of materializing all matching spans.
pub async fn query_runs_by_materializing_spans<S>(
    store: &S,
    tenant: TenantId,
    filter: RunFilter,
    page: PageRequest,
) -> StoreResult<Page<RunSummary>>
where
    S: TraceStore + ?Sized,
{
    let spans = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                project_id: filter.project_id.clone(),
                environment_id: filter.environment_id.clone(),
                trace_id: filter.trace_id.clone(),
                span_id: None,
                kind: None,
                status: None,
            },
            PageRequest {
                limit: u32::MAX,
                cursor: None,
            },
        )
        .await?
        .items;

    let runs = filter_run_summaries(roll_up_runs(tenant, spans.clone()), &spans, &filter);
    Ok(page_vec(runs, page))
}

/// One aggregated run row produced by a backend `GROUP BY` over the `spans`
/// table, keyed on `(project_id, trace_id)`.
///
/// Columnar backends (Postgres, ClickHouse) compute the scalar run rollups
/// (`span_count`, `status`, `started_at`, `ended_at`, `first_span_name`) in the
/// database and return only the per-run model / cost / release occurrences as
/// small arrays, ordered by `start_time DESC, seq ASC` to mirror the iteration
/// order of [`beater_schema::roll_up_runs`]. This is the §8.1-compliant
/// alternative to [`query_runs_by_materializing_spans`]: the backend reduces
/// each run to a single row instead of streaming every matching span into
/// process memory. [`finalize_run_aggregates`] folds these rows into the exact
/// same `Page<RunSummary>` the in-memory rollup produces.
#[derive(Clone, Debug)]
pub struct RunAggregateRow {
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub span_count: usize,
    pub status: SpanStatus,
    pub first_span_name: String,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    /// Per-span models in `start_time DESC, seq ASC` order (spans without a
    /// model omitted); de-duplicated by `(provider, name)` in
    /// [`finalize_run_aggregates`].
    pub models: Vec<ModelRef>,
    /// Per-span costs in `start_time DESC, seq ASC` order (spans without a cost
    /// omitted); folded in [`finalize_run_aggregates`].
    pub costs: Vec<Money>,
    /// Per-span release ids in `start_time DESC, seq ASC` order (spans without a
    /// release id omitted); de-duplicated in [`finalize_run_aggregates`].
    pub release_ids: Vec<String>,
    /// The distinct span kinds present in the run, used for the `RunFilter::kind`
    /// roll-up filter.
    pub kinds: BTreeSet<AgentSpanKind>,
}

/// Folds backend-aggregated [`RunAggregateRow`]s into the same
/// `Page<RunSummary>` that [`query_runs_by_materializing_spans`] would produce
/// for the same spans and `filter`, but without ever materializing every span.
///
/// The dedup / cost-merge / sort / filter / paginate steps mirror
/// [`beater_schema::roll_up_runs`] and [`beater_schema::filter_run_summaries`]
/// exactly; the conformance suite and the `finalize_matches_materialized_rollup`
/// unit test guard against drift.
pub fn finalize_run_aggregates(
    tenant: TenantId,
    rows: Vec<RunAggregateRow>,
    filter: RunFilter,
    page: PageRequest,
) -> Page<RunSummary> {
    let mut runs: Vec<RunSummary> = rows
        .iter()
        // `roll_up_runs`/`filter_run_summaries` keep a run when ANY of its spans
        // matches `filter.kind`; the backend has already reduced that predicate
        // to the run's distinct-kind set.
        .filter(|row| match &filter.kind {
            Some(kind) => row.kinds.contains(kind),
            None => true,
        })
        .map(|row| RunSummary {
            tenant_id: tenant.clone(),
            project_id: row.project_id.clone(),
            trace_id: row.trace_id.clone(),
            first_span_name: row.first_span_name.clone(),
            span_count: row.span_count,
            status: row.status.clone(),
            started_at: row.started_at,
            ended_at: row.ended_at,
            duration_ms: run_duration_ms(row.started_at, row.ended_at),
            total_cost: fold_run_cost(&row.costs),
            models: dedup_models(&row.models),
            release_ids: dedup_release_ids(&row.release_ids),
        })
        .collect();

    // `roll_up_runs` sorts most-recent-first by run start; backends return rows
    // pre-ordered by a first-appearance proxy so this stable sort reproduces the
    // reference tie-break for the common case of distinct run start times.
    runs.sort_by(|left, right| right.started_at.cmp(&left.started_at));

    // `filter.kind` is already applied above; clearing it lets the shared
    // run-level filter run with an empty span slice (it only consults spans for
    // the kind predicate).
    let runs = filter_run_summaries(
        runs,
        &[],
        &RunFilter {
            kind: None,
            ..filter
        },
    );
    page_vec(runs, page)
}

/// Mirrors `beater_schema`'s private `run_duration_ms`: clamps a non-negative
/// run duration in milliseconds, or `None` when the run has not ended.
fn run_duration_ms(started_at: Timestamp, ended_at: Option<Timestamp>) -> Option<i64> {
    ended_at.map(|ended_at| (ended_at - started_at).num_milliseconds().max(0))
}

/// Mirrors `beater_schema`'s private `merge_cost` fold over the run's per-span
/// costs (already ordered as `roll_up_runs` would visit them): sums same-currency
/// amounts and keeps the running total when a span's currency or an overflow
/// would make the add fail.
fn fold_run_cost(costs: &[Money]) -> Option<Money> {
    let mut total: Option<Money> = None;
    for cost in costs {
        total = match total {
            Some(current) => Some(current.try_add(cost).unwrap_or(current)),
            None => Some(cost.clone()),
        };
    }
    total
}

/// Mirrors `beater_schema`'s private `push_model`: distinct models by
/// `(provider, name)`, keeping first-occurrence order.
fn dedup_models(models: &[ModelRef]) -> Vec<ModelRef> {
    let mut deduped: Vec<ModelRef> = Vec::new();
    for model in models {
        if !deduped
            .iter()
            .any(|existing| existing.provider == model.provider && existing.name == model.name)
        {
            deduped.push(model.clone());
        }
    }
    deduped
}

/// Mirrors `beater_schema`'s private `push_release_id`: distinct release ids,
/// keeping first-occurrence order.
fn dedup_release_ids(release_ids: &[String]) -> Vec<String> {
    let mut deduped: Vec<String> = Vec::new();
    for release_id in release_ids {
        if !deduped.contains(release_id) {
            deduped.push(release_id.clone());
        }
    }
    deduped
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganizationMetadata {
    pub tenant_id: TenantId,
    pub organization_id: OrganizationId,
    pub display_name: String,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub tenant_id: TenantId,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub display_name: String,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentMetadata {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub display_name: String,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleBinding {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub principal_id: String,
    pub role: String,
    pub permissions: BTreeSet<String>,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotaReservationRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub amount: u64,
    pub limit: u64,
    pub window_start: Timestamp,
    pub reset_at: Timestamp,
    /// Optional client-supplied token that makes a reservation idempotent within
    /// its window. Concurrency between *distinct* reservers is already race-free
    /// (the limiter serializes the read-modify-write), but a client that retries
    /// the *same logical reservation* after a network timeout would otherwise be
    /// counted twice — the retry is a legitimately new request to the server. When
    /// a key is supplied the limiter records the outcome under
    /// `(tenant, project, window_start, idempotency_key)`, and a retry replays the
    /// original decision without advancing the counter again. `None` preserves the
    /// historical behavior for callers that do not retry.
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotaDecision {
    pub accepted: bool,
    pub used: u64,
    pub limit: u64,
    pub reset_at: Timestamp,
}

#[async_trait]
pub trait QuotaLimiter: Send + Sync {
    async fn reserve_quota(&self, request: QuotaReservationRequest) -> StoreResult<QuotaDecision>;
}

#[async_trait]
pub trait MetadataStore: Send + Sync {
    async fn put_organization(&self, organization: OrganizationMetadata) -> StoreResult<()>;

    async fn get_organization(
        &self,
        tenant_id: TenantId,
        organization_id: OrganizationId,
    ) -> StoreResult<Option<OrganizationMetadata>>;

    async fn put_project(&self, project: ProjectMetadata) -> StoreResult<()>;

    async fn get_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Option<ProjectMetadata>>;

    async fn put_environment(&self, environment: EnvironmentMetadata) -> StoreResult<()>;

    async fn get_environment(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        environment_id: EnvironmentId,
    ) -> StoreResult<Option<EnvironmentMetadata>>;

    async fn put_role_binding(&self, binding: RoleBinding) -> StoreResult<()>;

    async fn list_role_bindings(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
        principal_id: String,
    ) -> StoreResult<Vec<RoleBinding>>;
}

pub fn page_vec<T>(mut items: Vec<T>, page: PageRequest) -> Page<T> {
    let limit = page.limit.max(1) as usize;
    let offset = page
        .cursor
        .and_then(|cursor| cursor.parse::<usize>().ok())
        .unwrap_or(0);

    if offset >= items.len() {
        return Page::new(Vec::new(), None);
    }

    let next_offset = offset.saturating_add(limit);
    let next_cursor = if next_offset < items.len() {
        Some(next_offset.to_string())
    } else {
        None
    };
    let end = next_offset.min(items.len());
    let selected = items.drain(offset..end).collect();
    Page::new(selected, next_cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::SpanId;
    use beater_schema::roll_up_runs;
    use chrono::{TimeZone, Utc};
    use std::collections::BTreeMap;

    fn ts(seconds: i64) -> Timestamp {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid base timestamp"))
            + chrono::Duration::seconds(seconds)
    }

    // A test-only span constructor; the wide parameter list keeps the fixture
    // table-like and readable.
    #[allow(clippy::too_many_arguments)]
    fn span(
        project: &str,
        trace: &str,
        span_id: &str,
        kind: AgentSpanKind,
        name: &str,
        status: SpanStatus,
        started: i64,
        ended: Option<i64>,
        model: Option<(&str, &str)>,
        cost: Option<i64>,
        release: Option<&str>,
    ) -> SpanSummary {
        SpanSummary {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new(project).unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new(trace).unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}")),
            kind,
            name: name.to_string(),
            status,
            started_at: ts(started),
            ended_at: ended.map(ts),
            model: model.map(|(provider, name)| ModelRef {
                provider: provider.to_string(),
                name: name.to_string(),
            }),
            cost: cost.map(Money::usd_micros),
            release_id: release.map(ToString::to_string),
        }
    }

    /// Mimics the columnar backends' `GROUP BY (project_id, trace_id)`: the rows
    /// it emits carry exactly what Postgres/ClickHouse aggregate (scalars in the
    /// database, ordered model/cost/release arrays), and they are pushed in the
    /// same first-appearance order the backends' `ORDER BY max(start) DESC`
    /// yields. `spans` must already be in `start_time DESC, seq ASC` order, as
    /// `query_spans` returns them.
    fn aggregate_rows(spans: &[SpanSummary]) -> Vec<RunAggregateRow> {
        let mut order: Vec<(String, String)> = Vec::new();
        let mut groups: BTreeMap<(String, String), Vec<&SpanSummary>> = BTreeMap::new();
        for span in spans {
            let key = (
                span.project_id.as_str().to_string(),
                span.trace_id.as_str().to_string(),
            );
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push(span);
        }

        order
            .into_iter()
            .map(|key| {
                let members = &groups[&key];
                let status = if members.iter().any(|s| s.status == SpanStatus::Error) {
                    SpanStatus::Error
                } else if members.iter().any(|s| s.status == SpanStatus::Ok) {
                    SpanStatus::Ok
                } else {
                    SpanStatus::Unset
                };
                let started_at = members
                    .iter()
                    .map(|s| s.started_at)
                    .min()
                    .unwrap_or_else(|| panic!("non-empty group"));
                let ended_at = members.iter().filter_map(|s| s.ended_at).max();
                let first_span_name = members
                    .iter()
                    .filter(|s| s.started_at == started_at)
                    .map(|s| s.name.clone())
                    .next()
                    .unwrap_or_else(|| panic!("group has earliest span"));
                RunAggregateRow {
                    project_id: members[0].project_id.clone(),
                    trace_id: members[0].trace_id.clone(),
                    span_count: members.len(),
                    status,
                    first_span_name,
                    started_at,
                    ended_at,
                    models: members.iter().filter_map(|s| s.model.clone()).collect(),
                    costs: members.iter().filter_map(|s| s.cost.clone()).collect(),
                    release_ids: members
                        .iter()
                        .filter_map(|s| s.release_id.clone())
                        .collect(),
                    kinds: members.iter().map(|s| s.kind.clone()).collect(),
                }
            })
            .collect()
    }

    /// The reference run page: the in-memory rollup the columnar backends must
    /// reproduce exactly.
    fn reference(spans: &[SpanSummary], filter: &RunFilter, page: PageRequest) -> Page<RunSummary> {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let runs = filter_run_summaries(roll_up_runs(tenant, spans.to_vec()), spans, filter);
        page_vec(runs, page)
    }

    fn fixture_spans() -> Vec<SpanSummary> {
        // Ordered `start_time DESC, seq ASC`, as `query_spans` returns them.
        vec![
            span(
                "project",
                "trace-a",
                "a-late",
                AgentSpanKind::LlmCall,
                "a latest",
                SpanStatus::Ok,
                30,
                Some(40),
                Some(("openai", "gpt-4")),
                Some(300),
                Some("rel-2"),
            ),
            span(
                "project",
                "trace-b",
                "b-late",
                AgentSpanKind::ToolCall,
                "b latest",
                SpanStatus::Error,
                25,
                Some(25),
                None,
                None,
                None,
            ),
            span(
                "project",
                "trace-a",
                "a-mid",
                AgentSpanKind::LlmCall,
                "a middle",
                SpanStatus::Error,
                20,
                Some(35),
                Some(("anthropic", "claude")),
                Some(200),
                Some("rel-1"),
            ),
            span(
                "project",
                "trace-a",
                "a-early",
                AgentSpanKind::AgentRun,
                "a earliest",
                SpanStatus::Ok,
                10,
                Some(15),
                Some(("openai", "gpt-4")),
                Some(100),
                Some("rel-1"),
            ),
            span(
                "project",
                "trace-b",
                "b-early",
                AgentSpanKind::AgentRun,
                "b earliest",
                SpanStatus::Ok,
                5,
                None,
                None,
                Some(50),
                Some("rel-3"),
            ),
            span(
                "other",
                "trace-c",
                "c-only",
                AgentSpanKind::LlmCall,
                "c only",
                SpanStatus::Ok,
                12,
                Some(18),
                Some(("openai", "gpt-4o")),
                Some(70),
                Some("rel-4"),
            ),
        ]
    }

    fn assert_parity(filter: RunFilter, page: PageRequest) {
        let spans = fixture_spans();
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let expected = reference(&spans, &filter, page.clone());
        let actual = finalize_run_aggregates(tenant, aggregate_rows(&spans), filter, page);
        assert_eq!(actual.items, expected.items);
        assert_eq!(actual.next_cursor, expected.next_cursor);
    }

    #[test]
    fn finalize_matches_materialized_rollup() {
        // Unfiltered roll-up, including the multi-span run's summed cost, distinct
        // models/release ids, status precedence, earliest-span name, and duration.
        assert_parity(RunFilter::default(), PageRequest::default());

        let expected = reference(
            &fixture_spans(),
            &RunFilter::default(),
            PageRequest::default(),
        );
        let run_a = expected
            .items
            .iter()
            .find(|run| run.trace_id.as_str() == "trace-a")
            .unwrap_or_else(|| panic!("trace-a present"));
        assert_eq!(run_a.span_count, 3);
        assert_eq!(run_a.first_span_name, "a earliest");
        assert_eq!(run_a.status, SpanStatus::Error);
        assert_eq!(run_a.total_cost, Some(Money::usd_micros(600)));
        assert_eq!(run_a.duration_ms, Some(30_000));
        assert_eq!(
            run_a.models,
            vec![
                ModelRef {
                    provider: "openai".to_string(),
                    name: "gpt-4".to_string()
                },
                ModelRef {
                    provider: "anthropic".to_string(),
                    name: "claude".to_string()
                },
            ]
        );
        assert_eq!(
            run_a.release_ids,
            vec!["rel-2".to_string(), "rel-1".to_string()]
        );
    }

    #[test]
    fn finalize_matches_under_run_level_filters_and_paging() {
        let trace_a = TraceId::new("trace-a").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let cases = [
            RunFilter {
                status: Some(SpanStatus::Error),
                ..RunFilter::default()
            },
            RunFilter {
                kind: Some(AgentSpanKind::ToolCall),
                ..RunFilter::default()
            },
            RunFilter {
                kind: Some(AgentSpanKind::AgentRun),
                ..RunFilter::default()
            },
            RunFilter {
                project_id: Some(project),
                ..RunFilter::default()
            },
            RunFilter {
                trace_id: Some(trace_a),
                ..RunFilter::default()
            },
            RunFilter {
                model: Some("openai".to_string()),
                ..RunFilter::default()
            },
            RunFilter {
                release: Some("rel-1".to_string()),
                ..RunFilter::default()
            },
            RunFilter {
                min_cost_micros: Some(100),
                ..RunFilter::default()
            },
            RunFilter {
                max_cost_micros: Some(80),
                ..RunFilter::default()
            },
            RunFilter {
                min_latency_ms: Some(20_000),
                ..RunFilter::default()
            },
            RunFilter {
                max_latency_ms: Some(10_000),
                ..RunFilter::default()
            },
            RunFilter {
                started_after: Some(ts(8)),
                ..RunFilter::default()
            },
            RunFilter {
                started_before: Some(ts(8)),
                ..RunFilter::default()
            },
        ];
        for filter in cases {
            assert_parity(filter, PageRequest::default());
        }

        // Pagination parity, including the opaque offset cursor round-trip.
        let spans = fixture_spans();
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let first = finalize_run_aggregates(
            tenant.clone(),
            aggregate_rows(&spans),
            RunFilter::default(),
            PageRequest {
                limit: 1,
                cursor: None,
            },
        );
        let expected_first = reference(
            &spans,
            &RunFilter::default(),
            PageRequest {
                limit: 1,
                cursor: None,
            },
        );
        assert_eq!(first.items, expected_first.items);
        let cursor = first
            .next_cursor
            .clone()
            .unwrap_or_else(|| panic!("first page has a cursor"));
        let second = finalize_run_aggregates(
            tenant,
            aggregate_rows(&spans),
            RunFilter::default(),
            PageRequest {
                limit: 2,
                cursor: Some(cursor.clone()),
            },
        );
        let expected_second = reference(
            &spans,
            &RunFilter::default(),
            PageRequest {
                limit: 2,
                cursor: Some(cursor),
            },
        );
        assert_eq!(second.items, expected_second.items);
        assert_eq!(second.next_cursor, expected_second.next_cursor);
    }
}
