//! API-key management handlers (`/v1/api-keys/...`).
//!
//! Migrated out of `lib.rs` as the first resource module per issue #208. The
//! handlers now parse their path segments through [`EnvironmentPath`] instead of
//! repeating `TenantId::new` / `ProjectId::new` / `EnvironmentId::new` inline,
//! while authorization stays routed through [`crate::authorize`] so the 401/403
//! behavior is unchanged.

use axum::extract::{Path, State};
use axum::Json;
use beater_auth::{CreateApiKeyRequest, RevokedApiKey};
use beater_security::ApiScope;
use chrono::Utc;
use http::HeaderMap;

use crate::routes::EnvironmentPath;
use crate::{
    authorize, ensure_environment_exists, ApiError, ApiKeyCreatedResponse, ApiState,
    CreateApiKeyHttpRequest, ErrorResponse,
};

#[utoipa::path(
    post,
    path = "/v1/api-keys/{tenant_id}/{project_id}/{environment_id}",
    tag = "apiKeys",
    operation_id = "createApiKey",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("environment_id" = String, Path, description = "environment_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateApiKeyHttpRequest,
    responses(
        (status = 200, description = "Create a scoped API key", body = ApiKeyCreatedResponse),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
pub(crate) async fn create_api_key_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, environment_id)): Path<(String, String, String)>,
    Json(request): Json<CreateApiKeyHttpRequest>,
) -> Result<Json<ApiKeyCreatedResponse>, ApiError> {
    let api_keys = state
        .api_keys
        .clone()
        .ok_or_else(|| ApiError::not_implemented("api key store is not configured".to_string()))?;
    let EnvironmentPath {
        tenant_id,
        project_id,
        environment_id,
    } = EnvironmentPath::parse(tenant_id, project_id, environment_id)?;
    authorize(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        &environment_id,
        ApiScope::Admin,
    )
    .await?;
    ensure_environment_exists(&state, &tenant_id, &project_id, &environment_id).await?;
    let created = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id,
            project_id,
            environment_id,
            scopes: request.scopes,
        })
        .await?;
    Ok(Json(ApiKeyCreatedResponse::from_created(created)))
}

#[utoipa::path(
    post,
    path = "/v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke",
    tag = "apiKeys",
    operation_id = "revokeApiKey",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("environment_id" = String, Path, description = "environment_id"),
        ("api_key_id" = String, Path, description = "api_key_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Revoke an API key", body = RevokedApiKey),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
pub(crate) async fn revoke_api_key_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, environment_id, api_key_id)): Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Result<Json<RevokedApiKey>, ApiError> {
    let api_keys = state
        .api_keys
        .clone()
        .ok_or_else(|| ApiError::not_implemented("api key store is not configured".to_string()))?;
    // Match the original ordering: validate the environment scope and run auth
    // before parsing the api-key id, so an invalid api-key id on an unauthorized
    // request still surfaces the auth failure first.
    let EnvironmentPath {
        tenant_id,
        project_id,
        environment_id,
    } = EnvironmentPath::parse(tenant_id, project_id, environment_id)?;
    authorize(
        &state,
        &headers,
        &tenant_id,
        &project_id,
        &environment_id,
        ApiScope::Admin,
    )
    .await?;
    let api_key_id = beater_core::ApiKeyId::new(api_key_id)?;
    let record = api_keys
        .get_key(api_key_id.clone())
        .await?
        .ok_or_else(|| ApiError::not_found(format!("api key {} not found", api_key_id.as_str())))?;
    if record.tenant_id != tenant_id
        || record.project_id != project_id
        || record.environment_id != environment_id
    {
        return Err(ApiError::not_found(format!(
            "api key {} not found",
            api_key_id.as_str()
        )));
    }
    let revoked = api_keys
        .revoke_key(api_key_id.clone(), Utc::now())
        .await?
        .ok_or_else(|| ApiError::not_found(format!("api key {} not found", api_key_id.as_str())))?;
    Ok(Json(revoked))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use axum::body::Body;
    use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
    use beater_bus::InMemoryBus;
    use beater_core::{EnvironmentId, ProjectId, TenantId};
    use beater_ingest::{IngestPolicy, IngestService};
    use beater_security::{ApiScope, CreatedApiKey};
    use beater_store::TraceStore;
    use beater_store_obj::FsArtifactStore;
    use beater_store_sql::SqliteTraceStore;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::{router, ApiState};

    /// Build an auth-required app backed by a real SQLite api-key store, plus a
    /// freshly minted key (with the given scopes) the test can present.
    async fn auth_app_with_key(
        scopes: BTreeSet<ApiScope>,
    ) -> (axum::Router, CreatedApiKey, tempfile::TempDir) {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let artifacts = Arc::new(
            FsArtifactStore::new(tempdir.path().join("artifacts"))
                .unwrap_or_else(|err| panic!("{err}")),
        );
        let traces: Arc<dyn TraceStore> =
            Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let bus = Arc::new(InMemoryBus::new(16));
        let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());

        let api_keys =
            Arc::new(SqliteApiKeyStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let created = api_keys
            .create_key(CreateApiKeyRequest {
                tenant_id: TenantId::new("acme").unwrap_or_else(|err| panic!("{err}")),
                project_id: ProjectId::new("proj").unwrap_or_else(|err| panic!("{err}")),
                environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
                scopes,
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let state = ApiState::new(ingest, traces).require_auth(api_keys);
        (router(state), created, tempdir)
    }

    #[tokio::test]
    async fn create_api_key_rejects_invalid_tenant_id_with_400() {
        // Admin-scoped key so auth would otherwise succeed; the whitespace tenant
        // segment must fail id parsing first with a 400.
        let (app, created, _tempdir) = auth_app_with_key(BTreeSet::from([ApiScope::Admin])).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    // `bad%20id` decodes to `bad id` -> whitespace -> invalid id.
                    .uri("/v1/api-keys/bad%20id/proj/prod")
                    .header("x-beater-api-key", &created.secret)
                    .header("x-beater-project-id", "proj")
                    .header("x-beater-environment-id", "prod")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"scopes":["admin"]}"#.to_string()))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_api_key_rejects_insufficient_scope_with_403() {
        // Key only carries TraceRead, but createApiKey requires Admin.
        let (app, created, _tempdir) =
            auth_app_with_key(BTreeSet::from([ApiScope::TraceRead])).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/api-keys/acme/proj/prod")
                    .header("x-beater-api-key", &created.secret)
                    .header("x-beater-project-id", "proj")
                    .header("x-beater-environment-id", "prod")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"scopes":["admin"]}"#.to_string()))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn revoke_api_key_rejects_insufficient_scope_with_403() {
        let (app, created, _tempdir) =
            auth_app_with_key(BTreeSet::from([ApiScope::TraceRead])).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/api-keys/acme/proj/prod/some-key-id/revoke")
                    .header("x-beater-api-key", &created.secret)
                    .header("x-beater-project-id", "proj")
                    .header("x-beater-environment-id", "prod")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
