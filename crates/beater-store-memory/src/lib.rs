use async_trait::async_trait;
use beater_core::{IdempotencyKey, Page, PageRequest, ProjectId, TenantId, Timestamp, TraceId};
use beater_schema::{
    span_matches, span_summary, CanonicalSpan, CanonicalTraceBatch, RawEnvelope, RunFilter,
    RunSummary, SpanFilter, SpanSummary, TraceView, WriteAck,
};
use beater_store::{
    lock_poisoned, page_vec, query_runs_by_materializing_spans, EnvironmentMetadata, MetadataStore,
    OrganizationMetadata, ProjectMetadata, QuotaDecision, QuotaLimiter, QuotaReservationRequest,
    RoleBinding, StoreError, StoreResult, TraceStore,
};
use std::sync::{Arc, Mutex};

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

#[derive(Clone, Default)]
struct InMemoryQuotaState {
    counters: Vec<InMemoryQuotaCounter>,
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
    raw_envelopes: Vec<RawEnvelope>,
    spans: Vec<CanonicalSpan>,
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
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
        let mut state = self.lock()?;
        let mut accepted_raw = 0;
        let mut duplicate_raw = 0;
        for raw in batch.raw_envelopes {
            let exists = state.raw_envelopes.iter().any(|existing| {
                existing.tenant_id == raw.tenant_id
                    && existing.project_id == raw.project_id
                    && existing.idempotency_key == raw.idempotency_key
            });
            if exists {
                duplicate_raw += 1;
            } else {
                state.raw_envelopes.push(raw);
                accepted_raw += 1;
            }
        }

        let mut accepted_spans = 0;
        let mut duplicate_spans = 0;
        for span in batch.spans {
            let exists = state.spans.iter().any(|existing| {
                existing.tenant_id == span.tenant_id
                    && existing.project_id == span.project_id
                    && existing.trace_id == span.trace_id
                    && existing.span_id == span.span_id
                    && existing.seq == span.seq
            });
            if exists {
                duplicate_spans += 1;
            } else {
                state.spans.push(span);
                accepted_spans += 1;
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
            .spans
            .iter()
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
            .spans
            .iter()
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
        Ok(state
            .raw_envelopes
            .iter()
            .find(|raw| {
                raw.tenant_id == tenant
                    && raw.project_id == project
                    && raw.idempotency_key == idempotency_key
            })
            .cloned())
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
            .spans
            .iter()
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
        Ok(state
            .role_bindings
            .iter()
            .filter(|binding| {
                binding.tenant_id == tenant_id
                    && binding.project_id == project_id
                    && binding.principal_id == principal_id
            })
            .cloned()
            .collect())
    }
}

#[async_trait]
impl QuotaLimiter for InMemoryQuotaLimiter {
    async fn reserve_quota(&self, request: QuotaReservationRequest) -> StoreResult<QuotaDecision> {
        let mut state = self.lock()?;
        let counter = state.counters.iter_mut().find(|counter| {
            counter.tenant_id == request.tenant_id
                && counter.project_id == request.project_id
                && counter.window_start == request.window_start
        });
        let current_used = counter.as_ref().map(|counter| counter.used).unwrap_or(0);
        let Some(new_used) = current_used.checked_add(request.amount) else {
            return Err(StoreError::integrity("quota counter overflow"));
        };
        if new_used > request.limit {
            return Ok(QuotaDecision {
                accepted: false,
                used: current_used,
                limit: request.limit,
                reset_at: request.reset_at,
            });
        }

        if let Some(counter) = counter {
            counter.used = new_used;
        } else {
            state.counters.push(InMemoryQuotaCounter {
                tenant_id: request.tenant_id,
                project_id: request.project_id,
                window_start: request.window_start,
                used: new_used,
            });
        }
        Ok(QuotaDecision {
            accepted: true,
            used: new_used,
            limit: request.limit,
            reset_at: request.reset_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_store_conformance::{
        assert_metadata_store_conformance, assert_quota_limiter_concurrency_conformance,
        assert_quota_limiter_conformance, assert_trace_store_conformance,
    };

    #[tokio::test]
    async fn in_memory_trace_store_conforms() {
        assert_trace_store_conformance(InMemoryTraceStore::new()).await;
    }

    #[tokio::test]
    async fn in_memory_metadata_store_conforms() {
        assert_metadata_store_conformance(InMemoryMetadataStore::new()).await;
    }

    #[tokio::test]
    async fn in_memory_quota_limiter_conforms() {
        assert_quota_limiter_conformance(InMemoryQuotaLimiter::new()).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn in_memory_quota_limiter_concurrency_conforms() {
        assert_quota_limiter_concurrency_conformance(InMemoryQuotaLimiter::new()).await;
    }
}
