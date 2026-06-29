use async_trait::async_trait;
use beater_core::{
    IdempotencyKey, Page, PageRequest, ProjectId, SpanId, TenantId, Timestamp, TraceId,
};
use beater_schema::{
    span_matches, span_summary, CanonicalSpan, CanonicalTraceBatch, RawEnvelope, RunFilter,
    RunSummary, SpanFilter, SpanSummary, TraceView, WriteAck,
};
use beater_store::{
    lock_poisoned, page_vec, query_runs_by_materializing_spans, EnvironmentMetadata, MetadataStore,
    OrganizationMetadata, ProjectMetadata, QuotaDecision, QuotaLimiter, QuotaReservationRequest,
    RoleBinding, StoreError, StoreResult, TraceStore,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub struct InMemoryTraceStore {
    state: Arc<Mutex<InMemoryTraceState>>,
}

#[derive(Clone, Default)]
pub struct InMemoryMetadataStore {
    state: Arc<Mutex<InMemoryMetadataState>>,
}

#[derive(Clone, Default)]
pub struct InMemoryQuotaLimiter {
    state: Arc<Mutex<InMemoryQuotaState>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InMemoryQuotaCounter {
    tenant_id: TenantId,
    project_id: ProjectId,
    window_start: Timestamp,
    used: u64,
}

/// Recorded outcome of a keyed reservation, mirroring the SQLite
/// `quota_reservations` ledger so a client retry replays the same decision
/// instead of double-counting.
#[derive(Clone, Debug, PartialEq, Eq)]
struct InMemoryQuotaReservation {
    tenant_id: TenantId,
    project_id: ProjectId,
    window_start: Timestamp,
    idempotency_key: String,
    accepted: bool,
    used_after: u64,
    reset_at: Timestamp,
}

#[derive(Clone, Default)]
struct InMemoryQuotaState {
    counters: Vec<InMemoryQuotaCounter>,
    reservations: Vec<InMemoryQuotaReservation>,
}

#[derive(Clone, Default)]
struct InMemoryMetadataState {
    organizations: Vec<OrganizationMetadata>,
    projects: Vec<ProjectMetadata>,
    environments: Vec<EnvironmentMetadata>,
    role_bindings: Vec<RoleBinding>,
}

#[derive(Clone, Default)]
struct InMemoryTraceState {
    raw_envelopes: HashMap<RawEnvelopeKey, RawEnvelope>,
    spans: HashMap<SpanDedupKey, CanonicalSpan>,
    span_order: Vec<SpanDedupKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RawEnvelopeKey {
    tenant_id: TenantId,
    project_id: ProjectId,
    idempotency_key: IdempotencyKey,
}

impl From<&RawEnvelope> for RawEnvelopeKey {
    fn from(raw: &RawEnvelope) -> Self {
        Self {
            tenant_id: raw.tenant_id.clone(),
            project_id: raw.project_id.clone(),
            idempotency_key: raw.idempotency_key.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanDedupKey {
    tenant_id: TenantId,
    project_id: ProjectId,
    trace_id: TraceId,
    span_id: SpanId,
    seq: u64,
}

impl From<&CanonicalSpan> for SpanDedupKey {
    fn from(span: &CanonicalSpan) -> Self {
        Self {
            tenant_id: span.tenant_id.clone(),
            project_id: span.project_id.clone(),
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            seq: span.seq,
        }
    }
}

impl InMemoryTraceState {
    fn spans_in_insert_order(&self) -> impl Iterator<Item = &CanonicalSpan> {
        self.span_order.iter().filter_map(|key| self.spans.get(key))
    }
}

impl InMemoryTraceStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, InMemoryTraceState>> {
        lock_poisoned(&self.state, "in-memory trace store")
    }
}

impl InMemoryMetadataStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, InMemoryMetadataState>> {
        lock_poisoned(&self.state, "metadata store")
    }
}

impl InMemoryQuotaLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, InMemoryQuotaState>> {
        lock_poisoned(&self.state, "quota limiter")
    }
}

#[async_trait]
impl TraceStore for InMemoryTraceStore {
    async fn write_batch(&self, batch: Arc<CanonicalTraceBatch>) -> StoreResult<WriteAck> {
        // Take ownership without a deep copy when we hold the only handle (the
        // common ingest path); fall back to cloning the inner batch only when a
        // retry handle is still alive.
        let batch = Arc::try_unwrap(batch).unwrap_or_else(|shared| (*shared).clone());
        let mut state = self.lock()?;
        let mut accepted_raw = 0;
        let mut duplicate_raw = 0;
        for raw in batch.raw_envelopes {
            let key = RawEnvelopeKey::from(&raw);
            if let Entry::Vacant(entry) = state.raw_envelopes.entry(key) {
                entry.insert(raw);
                accepted_raw += 1;
            } else {
                duplicate_raw += 1;
            }
        }

        let mut accepted_spans = 0;
        let mut duplicate_spans = 0;
        for span in batch.spans {
            let key = SpanDedupKey::from(&span);
            let inserted_key = if let Entry::Vacant(entry) = state.spans.entry(key) {
                let key = entry.key().clone();
                entry.insert(span);
                Some(key)
            } else {
                None
            };
            if let Some(key) = inserted_key {
                state.span_order.push(key);
                accepted_spans += 1;
            } else {
                duplicate_spans += 1;
            }
        }

        Ok(WriteAck {
            accepted_raw,
            accepted_spans,
            duplicate_raw,
            duplicate_spans,
        })
    }

    async fn get_trace(&self, tenant: TenantId, trace: TraceId) -> StoreResult<TraceView> {
        let state = self.lock()?;
        let mut spans = state
            .spans_in_insert_order()
            .filter(|span| span.tenant_id == tenant && span.trace_id == trace)
            .cloned()
            .collect::<Vec<_>>();
        sort_trace_spans(&mut spans);
        Ok(TraceView {
            tenant_id: tenant,
            trace_id: trace,
            spans,
        })
    }

    async fn get_project_trace(
        &self,
        tenant: TenantId,
        project: ProjectId,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        let state = self.lock()?;
        let mut spans = state
            .spans_in_insert_order()
            .filter(|span| {
                span.tenant_id == tenant && span.project_id == project && span.trace_id == trace
            })
            .cloned()
            .collect::<Vec<_>>();
        sort_trace_spans(&mut spans);
        Ok(TraceView {
            tenant_id: tenant,
            trace_id: trace,
            spans,
        })
    }

    async fn get_raw_envelope(
        &self,
        tenant: TenantId,
        project: ProjectId,
        idempotency_key: IdempotencyKey,
    ) -> StoreResult<Option<RawEnvelope>> {
        let state = self.lock()?;
        let key = RawEnvelopeKey {
            tenant_id: tenant,
            project_id: project,
            idempotency_key,
        };
        Ok(state.raw_envelopes.get(&key).cloned())
    }

    async fn query_runs(
        &self,
        tenant: TenantId,
        filter: RunFilter,
        page: PageRequest,
    ) -> StoreResult<Page<RunSummary>> {
        query_runs_by_materializing_spans(self, tenant, filter, page).await
    }

    async fn query_spans(
        &self,
        tenant: TenantId,
        filter: SpanFilter,
        page: PageRequest,
    ) -> StoreResult<Page<SpanSummary>> {
        let state = self.lock()?;
        let mut spans = state
            .spans_in_insert_order()
            .filter(|span| span.tenant_id == tenant && span_matches(span, &filter))
            .cloned()
            .map(span_summary)
            .collect::<Vec<_>>();
        spans.sort_by(|left, right| {
            right
                .started_at
                .cmp(&left.started_at)
                .then_with(|| left.trace_id.cmp(&right.trace_id))
                .then_with(|| left.span_id.cmp(&right.span_id))
        });
        Ok(page_vec(spans, page))
    }
}

fn sort_trace_spans(spans: &mut [CanonicalSpan]) {
    spans.sort_by(|left, right| {
        left.seq
            .cmp(&right.seq)
            .then_with(|| left.start_time.cmp(&right.start_time))
    });
}

#[async_trait]
impl MetadataStore for InMemoryMetadataStore {
    async fn put_organization(&self, organization: OrganizationMetadata) -> StoreResult<()> {
        let mut state = self.lock()?;
        if let Some(existing) = state.organizations.iter_mut().find(|existing| {
            existing.tenant_id == organization.tenant_id
                && existing.organization_id == organization.organization_id
        }) {
            *existing = organization;
        } else {
            state.organizations.push(organization);
        }
        Ok(())
    }

    async fn get_organization(
        &self,
        tenant_id: TenantId,
        organization_id: beater_core::OrganizationId,
    ) -> StoreResult<Option<OrganizationMetadata>> {
        let state = self.lock()?;
        Ok(state
            .organizations
            .iter()
            .find(|organization| {
                organization.tenant_id == tenant_id
                    && organization.organization_id == organization_id
            })
            .cloned())
    }

    async fn put_project(&self, project: ProjectMetadata) -> StoreResult<()> {
        let mut state = self.lock()?;
        if let Some(existing) = state.projects.iter_mut().find(|existing| {
            existing.tenant_id == project.tenant_id && existing.project_id == project.project_id
        }) {
            *existing = project;
        } else {
            state.projects.push(project);
        }
        Ok(())
    }

    async fn get_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Option<ProjectMetadata>> {
        let state = self.lock()?;
        Ok(state
            .projects
            .iter()
            .find(|project| project.tenant_id == tenant_id && project.project_id == project_id)
            .cloned())
    }

    async fn put_environment(&self, environment: EnvironmentMetadata) -> StoreResult<()> {
        let mut state = self.lock()?;
        if let Some(existing) = state.environments.iter_mut().find(|existing| {
            existing.tenant_id == environment.tenant_id
                && existing.project_id == environment.project_id
                && existing.environment_id == environment.environment_id
        }) {
            *existing = environment;
        } else {
            state.environments.push(environment);
        }
        Ok(())
    }

    async fn get_environment(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        environment_id: beater_core::EnvironmentId,
    ) -> StoreResult<Option<EnvironmentMetadata>> {
        let state = self.lock()?;
        Ok(state
            .environments
            .iter()
            .find(|environment| {
                environment.tenant_id == tenant_id
                    && environment.project_id == project_id
                    && environment.environment_id == environment_id
            })
            .cloned())
    }

    async fn put_role_binding(&self, binding: RoleBinding) -> StoreResult<()> {
        let mut state = self.lock()?;
        if let Some(existing) = state.role_bindings.iter_mut().find(|existing| {
            existing.tenant_id == binding.tenant_id
                && existing.project_id == binding.project_id
                && existing.principal_id == binding.principal_id
                && existing.role == binding.role
        }) {
            *existing = binding;
        } else {
            state.role_bindings.push(binding);
        }
        Ok(())
    }

    async fn list_role_bindings(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
        principal_id: String,
    ) -> StoreResult<Vec<RoleBinding>> {
        let state = self.lock()?;
        let mut bindings = state
            .role_bindings
            .iter()
            .filter(|binding| {
                binding.tenant_id == tenant_id
                    && binding.project_id == project_id
                    && binding.principal_id == principal_id
            })
            .cloned()
            .collect::<Vec<_>>();
        bindings.sort_by(|left, right| left.role.cmp(&right.role));
        Ok(bindings)
    }
}

#[async_trait]
impl QuotaLimiter for InMemoryQuotaLimiter {
    async fn reserve_quota(&self, request: QuotaReservationRequest) -> StoreResult<QuotaDecision> {
        let mut state = self.lock()?;

        // Idempotent replay: a retry of the same logical reservation (same key,
        // same window) returns the recorded decision without advancing the
        // counter again. The whole method holds the state mutex, so the lookup
        // and the counter update are atomic.
        if let Some(key) = request.idempotency_key.as_deref() {
            if let Some(existing) = state.reservations.iter().find(|reservation| {
                reservation.tenant_id == request.tenant_id
                    && reservation.project_id == request.project_id
                    && reservation.window_start == request.window_start
                    && reservation.idempotency_key == key
            }) {
                return Ok(QuotaDecision {
                    accepted: existing.accepted,
                    used: existing.used_after,
                    limit: request.limit,
                    reset_at: existing.reset_at,
                });
            }
        }

        let counter = state.counters.iter().position(|counter| {
            counter.tenant_id == request.tenant_id
                && counter.project_id == request.project_id
                && counter.window_start == request.window_start
        });
        let current_used = counter.map(|idx| state.counters[idx].used).unwrap_or(0);
        let Some(new_used) = current_used.checked_add(request.amount) else {
            return Err(StoreError::integrity("quota counter overflow"));
        };

        let decision = if new_used > request.limit {
            QuotaDecision {
                accepted: false,
                used: current_used,
                limit: request.limit,
                reset_at: request.reset_at,
            }
        } else {
            match counter {
                Some(idx) => state.counters[idx].used = new_used,
                None => state.counters.push(InMemoryQuotaCounter {
                    tenant_id: request.tenant_id.clone(),
                    project_id: request.project_id.clone(),
                    window_start: request.window_start,
                    used: new_used,
                }),
            }
            QuotaDecision {
                accepted: true,
                used: new_used,
                limit: request.limit,
                reset_at: request.reset_at,
            }
        };

        // Record the keyed outcome (accepted or rejected) so a later retry of this
        // key replays it verbatim.
        if let Some(key) = request.idempotency_key.as_deref() {
            state.reservations.push(InMemoryQuotaReservation {
                tenant_id: request.tenant_id,
                project_id: request.project_id,
                window_start: request.window_start,
                idempotency_key: key.to_string(),
                accepted: decision.accepted,
                used_after: decision.used,
                reset_at: decision.reset_at,
            });
        }

        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_store_conformance::{
        assert_metadata_store_conformance, assert_quota_limiter_concurrency_conformance,
        assert_quota_limiter_conformance, assert_quota_limiter_idempotency_conformance,
        assert_trace_store_conformance,
    };
    use std::collections::BTreeSet;

    #[tokio::test]
    async fn in_memory_trace_store_conforms() {
        assert_trace_store_conformance(InMemoryTraceStore::new()).await;
    }

    #[tokio::test]
    async fn in_memory_metadata_store_conforms() {
        assert_metadata_store_conformance(InMemoryMetadataStore::new()).await;
    }

    #[tokio::test]
    async fn list_role_bindings_orders_by_role() {
        let store = InMemoryMetadataStore::new();
        let tenant = tenant_id("tenant");
        let project = project_id("project");

        for role in ["viewer", "admin", "editor"] {
            store
                .put_role_binding(role_binding(
                    &tenant,
                    Some(&project),
                    "api-key:ordered",
                    role,
                ))
                .await
                .unwrap_or_else(|err| panic!("{err}"));
        }

        let bindings = store
            .list_role_bindings(tenant, Some(project), "api-key:ordered".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let roles = bindings
            .iter()
            .map(|binding| binding.role.as_str())
            .collect::<Vec<_>>();

        assert_eq!(roles, vec!["admin", "editor", "viewer"]);
    }

    #[tokio::test]
    async fn list_role_bindings_isolates_tenant_project_and_principal() {
        let store = InMemoryMetadataStore::new();
        let tenant = tenant_id("tenant");
        let other_tenant = tenant_id("other-tenant");
        let project = project_id("project");
        let other_project = project_id("other-project");

        for binding in [
            role_binding(&tenant, Some(&project), "api-key:target", "viewer"),
            role_binding(
                &other_tenant,
                Some(&project),
                "api-key:target",
                "tenant-leak",
            ),
            role_binding(
                &tenant,
                Some(&other_project),
                "api-key:target",
                "project-leak",
            ),
            role_binding(&tenant, Some(&project), "api-key:other", "principal-leak"),
            role_binding(&tenant, None, "api-key:target", "tenant-wide"),
        ] {
            store
                .put_role_binding(binding)
                .await
                .unwrap_or_else(|err| panic!("{err}"));
        }

        let project_bindings = store
            .list_role_bindings(
                tenant.clone(),
                Some(project.clone()),
                "api-key:target".to_string(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            project_bindings
                .iter()
                .map(|binding| binding.role.as_str())
                .collect::<Vec<_>>(),
            vec!["viewer"]
        );

        let tenant_wide_bindings = store
            .list_role_bindings(tenant, None, "api-key:target".to_string())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            tenant_wide_bindings
                .iter()
                .map(|binding| binding.role.as_str())
                .collect::<Vec<_>>(),
            vec!["tenant-wide"]
        );
    }

    #[tokio::test]
    async fn in_memory_quota_limiter_conforms() {
        assert_quota_limiter_conformance(InMemoryQuotaLimiter::new()).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn in_memory_quota_limiter_concurrency_conforms() {
        assert_quota_limiter_concurrency_conformance(InMemoryQuotaLimiter::new()).await;
    }

    #[tokio::test]
    async fn in_memory_quota_limiter_idempotency_conforms() {
        assert_quota_limiter_idempotency_conformance(InMemoryQuotaLimiter::new()).await;
    }

    fn tenant_id(value: &str) -> TenantId {
        TenantId::new(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn project_id(value: &str) -> ProjectId {
        ProjectId::new(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn role_binding(
        tenant_id: &TenantId,
        project_id: Option<&ProjectId>,
        principal_id: &str,
        role: &str,
    ) -> RoleBinding {
        RoleBinding {
            tenant_id: tenant_id.clone(),
            project_id: project_id.cloned(),
            principal_id: principal_id.to_string(),
            role: role.to_string(),
            permissions: BTreeSet::from([format!("{role}:permission")]),
            created_at: std::time::SystemTime::UNIX_EPOCH.into(),
        }
    }
}
