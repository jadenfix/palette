use beater_core::{
    ArtifactId, EnvironmentId, IdempotencyKey, OrganizationId, PageRequest, ProjectId, Sha256Hash,
    SpanId, TenantId, TraceId,
};
use beater_schema::{
    AgentSpanKind, ArtifactRef, AuthContext, CanonicalSpan, CanonicalTraceBatch, RawEnvelope,
    RedactionClass, RunFilter, SourceDialect, SpanFilter, SpanStatus, RAW_SCHEMA_VERSION,
};
use beater_store::{
    MetadataStore, OrganizationMetadata, ProjectMetadata, QuotaLimiter, QuotaReservationRequest,
    RoleBinding, TraceStore,
};
use chrono::{TimeZone, Utc};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

pub async fn assert_trace_store_conformance<S>(store: S)
where
    S: TraceStore,
{
    let (batch, tenant, project, trace, idempotency_key) = fixture_batch();

    let first = store
        .write_batch(batch.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let second = store
        .write_batch(batch)
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(first.accepted_raw, 1);
    assert_eq!(first.accepted_spans, 2);
    assert_eq!(second.duplicate_raw, 1);
    assert_eq!(second.duplicate_spans, 2);

    let trace_view = store
        .get_trace(tenant.clone(), trace.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(trace_view.spans.len(), 2);
    assert_eq!(trace_view.spans[0].span_id.as_str(), "root");
    assert_eq!(trace_view.spans[1].span_id.as_str(), "child");

    let raw = store
        .get_raw_envelope(tenant.clone(), project.clone(), idempotency_key)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("raw envelope should exist"));
    assert_eq!(raw.source, SourceDialect::Native);

    let spans = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                project_id: None,
                environment_id: None,
                trace_id: Some(trace.clone()),
                span_id: None,
                kind: Some(AgentSpanKind::AgentStep),
                status: Some(SpanStatus::Ok),
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(spans.items.len(), 1);
    assert_eq!(spans.items[0].span_id.as_str(), "child");

    let other_tenant = TenantId::new("other").unwrap_or_else(|err| panic!("{err}"));
    let empty = store
        .query_spans(other_tenant, SpanFilter::default(), PageRequest::default())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(empty.items.is_empty());

    let runs = store
        .query_runs(tenant, RunFilter::default(), PageRequest::default())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(runs.items.len(), 1);
    assert_eq!(runs.items[0].project_id.as_str(), project.as_str());
    assert_eq!(runs.items[0].span_count, 2);

    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let other_project = ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}"));
    let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
    let other_project_batch =
        fixture_project_batch(&tenant, &other_project, &trace, "other-project-root", 3);
    let write_other_project = store
        .write_batch(other_project_batch)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(write_other_project.accepted_spans, 1);

    let same_trace_runs = store
        .query_runs(
            tenant.clone(),
            RunFilter {
                trace_id: Some(trace.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(same_trace_runs.items.len(), 2);
    assert!(same_trace_runs
        .items
        .iter()
        .any(|run| run.project_id.as_str() == project.as_str() && run.span_count == 2));
    assert!(same_trace_runs
        .items
        .iter()
        .any(|run| run.project_id.as_str() == other_project.as_str() && run.span_count == 1));

    let project_runs = store
        .query_runs(
            tenant.clone(),
            RunFilter {
                project_id: Some(project.clone()),
                trace_id: Some(trace.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(project_runs.items.len(), 1);
    assert_eq!(project_runs.items[0].project_id.as_str(), project.as_str());
    assert_eq!(project_runs.items[0].span_count, 2);

    let other_project_runs = store
        .query_runs(
            tenant.clone(),
            RunFilter {
                project_id: Some(other_project.clone()),
                trace_id: Some(trace.clone()),
                ..RunFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(other_project_runs.items.len(), 1);
    assert_eq!(
        other_project_runs.items[0].project_id.as_str(),
        other_project.as_str()
    );
    assert_eq!(other_project_runs.items[0].span_count, 1);

    let scoped_spans = store
        .query_spans(
            tenant.clone(),
            SpanFilter {
                project_id: Some(project.clone()),
                trace_id: Some(trace.clone()),
                ..SpanFilter::default()
            },
            PageRequest {
                limit: 10,
                cursor: None,
            },
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(scoped_spans.items.len(), 2);
    assert!(scoped_spans
        .items
        .iter()
        .all(|span| span.project_id.as_str() == project.as_str()));

    let scoped_project = store
        .get_project_trace(tenant.clone(), project, trace.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let scoped_other_project = store
        .get_project_trace(tenant, other_project, trace)
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(scoped_project.spans.len(), 2);
    assert!(scoped_project
        .spans
        .iter()
        .all(|span| span.project_id.as_str() == "project"));
    assert_eq!(scoped_other_project.spans.len(), 1);
    assert_eq!(
        scoped_other_project.spans[0].span_id.as_str(),
        "other-project-root"
    );
    assert_eq!(
        scoped_other_project.spans[0].project_id.as_str(),
        "other-project"
    );
}

pub async fn assert_metadata_store_conformance<S>(store: S)
where
    S: MetadataStore,
{
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let organization = OrganizationId::new("org").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let created_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));

    store
        .put_organization(OrganizationMetadata {
            tenant_id: tenant.clone(),
            organization_id: organization.clone(),
            display_name: "Org".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    store
        .put_project(ProjectMetadata {
            tenant_id: tenant.clone(),
            organization_id: organization.clone(),
            project_id: project.clone(),
            display_name: "Project".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    store
        .put_environment(beater_store::EnvironmentMetadata {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            environment_id: environment.clone(),
            display_name: "Production".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    store
        .put_role_binding(RoleBinding {
            tenant_id: tenant.clone(),
            project_id: Some(project.clone()),
            principal_id: "api-key:admin".to_string(),
            role: "project_admin".to_string(),
            permissions: BTreeSet::from(["admin".to_string()]),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let loaded_environment = store
        .get_environment(tenant.clone(), project.clone(), environment)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("environment should exist"));
    assert_eq!(loaded_environment.display_name, "Production");

    let bindings = store
        .list_role_bindings(tenant, Some(project), "api-key:admin".to_string())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(bindings.len(), 1);
    assert!(bindings[0].permissions.contains("admin"));
}

pub async fn assert_quota_limiter_conformance<L>(limiter: L)
where
    L: QuotaLimiter,
{
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let window_start = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let reset_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));

    let first = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            amount: 2,
            limit: 3,
            window_start,
            reset_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(first.accepted);
    assert_eq!(first.used, 2);

    let rejected = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            amount: 2,
            limit: 3,
            window_start,
            reset_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(!rejected.accepted);
    assert_eq!(rejected.used, 2);
    assert_eq!(rejected.reset_at, reset_at);

    let next_window = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let next_reset = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 2, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let after_reset = limiter
        .reserve_quota(QuotaReservationRequest {
            tenant_id: tenant,
            project_id: project,
            amount: 3,
            limit: 3,
            window_start: next_window,
            reset_at: next_reset,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(after_reset.accepted);
    assert_eq!(after_reset.used, 3);
}

fn fixture_batch() -> (
    CanonicalTraceBatch,
    TenantId,
    ProjectId,
    TraceId,
    IdempotencyKey,
) {
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
    let idempotency_key =
        IdempotencyKey::new("tenant:project:trace:raw").unwrap_or_else(|err| panic!("{err}"));
    let body_ref = artifact_ref("raw");
    let raw = RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: environment.clone(),
        source: SourceDialect::Native,
        source_schema_url: Some("beater://native/v1".to_string()),
        source_schema_version: Some("1".to_string()),
        received_at: Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp")),
        idempotency_key: idempotency_key.clone(),
        payload_hash: body_ref.sha256.clone(),
        body_ref: body_ref.clone(),
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    };
    let root = canonical_span(CanonicalSpanFixture {
        tenant: &tenant,
        project: &project,
        environment: &environment,
        trace: &trace,
        span: "root",
        seq: 1,
        kind: AgentSpanKind::AgentRun,
        name: "run",
        raw_ref: body_ref.clone(),
    });
    let child = canonical_span(CanonicalSpanFixture {
        tenant: &tenant,
        project: &project,
        environment: &environment,
        trace: &trace,
        span: "child",
        seq: 2,
        kind: AgentSpanKind::AgentStep,
        name: "step",
        raw_ref: body_ref,
    });
    (
        CanonicalTraceBatch {
            raw_envelopes: vec![raw],
            spans: vec![child, root],
        },
        tenant,
        project,
        trace,
        idempotency_key,
    )
}

fn fixture_project_batch(
    tenant: &TenantId,
    project: &ProjectId,
    trace: &TraceId,
    span_id: &str,
    seq: u64,
) -> CanonicalTraceBatch {
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let idempotency_key = IdempotencyKey::new(format!(
        "{}:{}:{}:raw",
        tenant.as_str(),
        project.as_str(),
        trace.as_str()
    ))
    .unwrap_or_else(|err| panic!("{err}"));
    let body_ref = artifact_ref("other-project-raw");
    let raw = RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: environment.clone(),
        source: SourceDialect::Native,
        source_schema_url: Some("beater://native/v1".to_string()),
        source_schema_version: Some("1".to_string()),
        received_at: Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp")),
        idempotency_key,
        payload_hash: body_ref.sha256.clone(),
        body_ref: body_ref.clone(),
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    };
    let span = canonical_span(CanonicalSpanFixture {
        tenant,
        project,
        environment: &environment,
        trace,
        span: span_id,
        seq,
        kind: AgentSpanKind::AgentRun,
        name: "other project run",
        raw_ref: body_ref,
    });
    CanonicalTraceBatch {
        raw_envelopes: vec![raw],
        spans: vec![span],
    }
}

struct CanonicalSpanFixture<'a> {
    tenant: &'a TenantId,
    project: &'a ProjectId,
    environment: &'a EnvironmentId,
    trace: &'a TraceId,
    span: &'a str,
    seq: u64,
    kind: AgentSpanKind,
    name: &'a str,
    raw_ref: ArtifactRef,
}

fn canonical_span(fixture: CanonicalSpanFixture<'_>) -> CanonicalSpan {
    CanonicalSpan {
        schema_version: beater_schema::CANONICAL_SCHEMA_VERSION,
        normalizer_version: "beater-native-v1".to_string(),
        tenant_id: fixture.tenant.clone(),
        project_id: fixture.project.clone(),
        environment_id: fixture.environment.clone(),
        trace_id: fixture.trace.clone(),
        span_id: SpanId::new(fixture.span).unwrap_or_else(|err| panic!("{err}")),
        parent_span_id: None,
        seq: fixture.seq,
        kind: fixture.kind,
        name: fixture.name.to_string(),
        status: SpanStatus::Ok,
        start_time: Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, fixture.seq as u32)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp")),
        end_time: None,
        model: None,
        cost: None,
        tokens: None,
        input_ref: None,
        output_ref: None,
        attributes: BTreeMap::new(),
        unmapped_attrs: json!({}),
        raw_ref: fixture.raw_ref,
    }
}

fn artifact_ref(name: &str) -> ArtifactRef {
    ArtifactRef {
        artifact_id: ArtifactId::new(name).unwrap_or_else(|err| panic!("{err}")),
        uri: format!("artifact://tenant/project/{name}"),
        sha256: Sha256Hash::new("ab".repeat(32)).unwrap_or_else(|err| panic!("{err}")),
        size_bytes: 2,
        mime_type: "application/json".to_string(),
        redaction_class: RedactionClass::Internal,
    }
}
