use async_trait::async_trait;
use beater_core::{
    EnvironmentId, IdempotencyKey, OrganizationId, Page, PageRequest, ProjectId, TenantId,
    Timestamp, TraceId,
};
use beater_schema::{
    filter_run_summaries, roll_up_runs, ArtifactRef, CanonicalTraceBatch, RawEnvelope, RunFilter,
    RunSummary, SpanFilter, SpanSummary, TraceView, WriteAck,
};
use std::collections::BTreeSet;
use std::sync::Arc;

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
impl<T> QuotaLimiter for Arc<T>
where
    T: QuotaLimiter + ?Sized,
{
    async fn reserve_quota(&self, request: QuotaReservationRequest) -> StoreResult<QuotaDecision> {
        (**self).reserve_quota(request).await
    }
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
