//! Enforced RBAC (§20.7 #5.2, invariant A20): `authorize()` consults the
//! caller's `RoleBinding`s on mutating routes and denies (`403`) a principal
//! that no binding grants. Read-only routes are never RBAC-gated. These tests
//! also prove the change is orthogonal to — and preserves — the object-level
//! scope guard on the API-key revoke route (PR #251): an out-of-scope revoke
//! still returns `404` for a caller that passes RBAC.

use axum::body::Body;
use axum::Router;
use beater_api::{router, ApiState};
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
use beater_bus::InMemoryBus;
use beater_core::{
    EnvironmentId, OrganizationId, ProjectId, SpanId, TenantId, TenantScope, TraceId,
};
use beater_ingest::{IngestPolicy, IngestService, NativeIngestRequest};
use beater_schema::{AgentSpanKind, RedactionClass, SpanStatus};
use beater_security::{ApiScope, CreatedApiKey};
use beater_store::{
    EnvironmentMetadata, MetadataStore, OrganizationMetadata, ProjectMetadata, RoleBinding,
};
use beater_store_memory::InMemoryMetadataStore;
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use chrono::Utc;
use http::{Request, StatusCode};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tower::ServiceExt;

const TENANT: &str = "tenant";
const PROJECT: &str = "project";
const ENVIRONMENT: &str = "prod";

fn tenant_id() -> TenantId {
    TenantId::new(TENANT).unwrap_or_else(|err| panic!("{err}"))
}

fn project_id() -> ProjectId {
    ProjectId::new(PROJECT).unwrap_or_else(|err| panic!("{err}"))
}

fn environment_id() -> EnvironmentId {
    EnvironmentId::new(ENVIRONMENT).unwrap_or_else(|err| panic!("{err}"))
}

/// An RBAC-enforcing app: auth required (#127) *and* RBAC required (§20.7 #5.2).
async fn rbac_app() -> (
    Router,
    Arc<SqliteApiKeyStore>,
    Arc<InMemoryMetadataStore>,
) {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts")).unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let api_keys = Arc::new(SqliteApiKeyStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let metadata = Arc::new(InMemoryMetadataStore::new());
    seed_scope(&metadata).await;
    let app = router(
        ApiState::new(ingest, traces)
            .with_metadata(metadata.clone())
            .require_auth(api_keys.clone())
            .require_rbac(),
    );
    (app, api_keys, metadata)
}

async fn seed_scope(metadata: &InMemoryMetadataStore) {
    let created_at = Utc::now();
    metadata
        .put_organization(OrganizationMetadata {
            tenant_id: tenant_id(),
            organization_id: OrganizationId::new("org").unwrap_or_else(|err| panic!("{err}")),
            display_name: "Org".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    metadata
        .put_project(ProjectMetadata {
            tenant_id: tenant_id(),
            organization_id: OrganizationId::new("org").unwrap_or_else(|err| panic!("{err}")),
            project_id: project_id(),
            display_name: "Project".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    metadata
        .put_environment(EnvironmentMetadata {
            tenant_id: tenant_id(),
            project_id: project_id(),
            environment_id: environment_id(),
            display_name: "Production".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
}

async fn create_key(api_keys: &SqliteApiKeyStore, scopes: &[ApiScope]) -> CreatedApiKey {
    let scopes: BTreeSet<ApiScope> = scopes.iter().copied().collect();
    api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: tenant_id(),
            project_id: project_id(),
            environment_id: environment_id(),
            scopes,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"))
}

async fn grant_role(
    metadata: &InMemoryMetadataStore,
    principal_id: &str,
    role: &str,
    permissions: &[&str],
) {
    metadata
        .put_role_binding(RoleBinding {
            tenant_id: tenant_id(),
            project_id: Some(project_id()),
            principal_id: principal_id.to_string(),
            role: role.to_string(),
            permissions: permissions.iter().map(|perm| perm.to_string()).collect(),
            created_at: Utc::now(),
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
}

fn native_request() -> NativeIngestRequest {
    NativeIngestRequest {
        scope: TenantScope::new(tenant_id(), project_id(), environment_id()),
        trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
        span_id: SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
        parent_span_id: None,
        seq: 1,
        kind: AgentSpanKind::AgentRun,
        name: "rbac agent run".to_string(),
        status: SpanStatus::Ok,
        start_time: None,
        end_time: None,
        model: None,
        cost: None,
        tokens: None,
        input: Some(json!("question")),
        output: Some(json!("answer")),
        attributes: BTreeMap::new(),
        redaction_class: RedactionClass::Internal,
        idempotency_key: None,
        auth_context: None,
    }
}

async fn post_native(app: &Router, secret: &str) -> StatusCode {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("authorization", format!("Bearer {secret}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&native_request()).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .status()
}

/// Caller WITH a binding granting `trace:write` may perform the mutating
/// ingest route (2xx). Owner-style `"*"` grants likewise pass.
#[tokio::test]
async fn caller_with_binding_can_mutate() {
    let (app, api_keys, metadata) = rbac_app().await;
    let key = create_key(&api_keys, &[ApiScope::TraceWrite, ApiScope::TraceRead]).await;
    grant_role(
        &metadata,
        key.record.api_key_id.as_str(),
        "writer",
        &["trace:write"],
    )
    .await;
    assert_eq!(post_native(&app, &key.secret).await, StatusCode::OK);
}

/// Same route, same valid credential + scope, but NO `RoleBinding`: the
/// subject-level RBAC check denies with `403` (A20: a non-owner is denied a
/// mutating route by `authorize()`).
#[tokio::test]
async fn caller_without_binding_is_forbidden() {
    let (app, api_keys, _metadata) = rbac_app().await;
    let key = create_key(&api_keys, &[ApiScope::TraceWrite, ApiScope::TraceRead]).await;
    assert_eq!(post_native(&app, &key.secret).await, StatusCode::FORBIDDEN);
}

/// Read-only routes are never RBAC-gated: a caller with the read scope but no
/// binding still reads successfully.
#[tokio::test]
async fn read_only_route_is_not_rbac_gated() {
    let (app, api_keys, _metadata) = rbac_app().await;
    let key = create_key(&api_keys, &[ApiScope::TraceRead]).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant?project_id=project&environment_id=prod&limit=10")
                .header("authorization", format!("Bearer {}", key.secret))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
}

/// Regression for PR #251: RBAC is orthogonal to the object-level scope guard.
/// An owner (passes RBAC) revoking an out-of-scope key still gets `404`, and the
/// target key stays active — the cross-tenant revoke bypass remains closed.
#[tokio::test]
async fn out_of_scope_revoke_still_404_under_rbac() {
    let (app, api_keys, metadata) = rbac_app().await;
    let admin = create_key(&api_keys, &[ApiScope::Admin]).await;
    grant_role(&metadata, admin.record.api_key_id.as_str(), "owner", &["*"]).await;

    // A key belonging to a different tenant/project — out of scope for the
    // `tenant/project/prod` revoke path below.
    let foreign = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: TenantId::new("other-tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: environment_id(),
            scopes: BTreeSet::from([ApiScope::TraceWrite]),
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/api-keys/tenant/project/prod/{}/revoke",
                    foreign.record.api_key_id.as_str()
                ))
                .header("authorization", format!("Bearer {}", admin.secret))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let still_there = api_keys
        .get_key(foreign.record.api_key_id)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("out-of-scope key must still exist"));
    assert!(still_there.active);
}

/// An owner (`"*"`) passing RBAC can revoke an in-scope key (2xx) — proves the
/// admin/mutating route is reachable once both the subject- and object-level
/// checks pass, so RBAC did not over-deny.
#[tokio::test]
async fn owner_can_revoke_in_scope_key() {
    let (app, api_keys, metadata) = rbac_app().await;
    let admin = create_key(&api_keys, &[ApiScope::Admin]).await;
    grant_role(&metadata, admin.record.api_key_id.as_str(), "owner", &["*"]).await;
    let target = create_key(&api_keys, &[ApiScope::TraceWrite]).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/api-keys/tenant/project/prod/{}/revoke",
                    target.record.api_key_id.as_str()
                ))
                .header("authorization", format!("Bearer {}", admin.secret))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
}
