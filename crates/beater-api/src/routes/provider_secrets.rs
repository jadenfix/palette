//! Provider-secret management handlers (`/v1/provider-secrets/...`).
//!
//! Migrated out of `lib.rs` per issue #208. Path segments are parsed through
//! [`ProjectPath`] / [`ProviderSecretPath`] instead of repeating
//! `TenantId::new` / `ProjectId::new` inline; authorization stays routed through
//! [`crate::authorize_project_route`].

use axum::extract::{Path, State};
use axum::Json;
use beater_secrets::{ProviderSecretMetadata, PutProviderSecretRequest, RevokedProviderSecret};
use beater_security::ApiScope;
use chrono::Utc;
use http::HeaderMap;

use crate::routes::{ProjectPath, ProviderSecretPath};
use crate::{
    authorize_project_route, provider_secret_store, ApiError, ApiState,
    CreateProviderSecretHttpRequest, ErrorResponse,
};

#[utoipa::path(
    post,
    path = "/v1/provider-secrets/{tenant_id}/{project_id}",
    tag = "providerSecrets",
    operation_id = "createProviderSecret",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    request_body = CreateProviderSecretHttpRequest,
    responses(
        (status = 200, description = "Store an encrypted provider secret", body = ProviderSecretMetadata),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
pub(crate) async fn create_provider_secret_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
    Json(request): Json<CreateProviderSecretHttpRequest>,
) -> Result<Json<ProviderSecretMetadata>, ApiError> {
    let provider_secrets = provider_secret_store(&state)?;
    let ProjectPath {
        tenant_id,
        project_id,
    } = ProjectPath::parse(tenant_id, project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let metadata = provider_secrets
        .put_secret(PutProviderSecretRequest {
            tenant_id,
            project_id,
            provider: request.provider,
            display_name: request.display_name,
            secret_value: request.secret_value,
        })
        .await?;
    Ok(Json(metadata))
}

#[utoipa::path(
    get,
    path = "/v1/provider-secrets/{tenant_id}/{project_id}",
    tag = "providerSecrets",
    operation_id = "listProviderSecrets",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "List provider secret metadata", body = Vec < ProviderSecretMetadata >),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
    )
)]
pub(crate) async fn list_provider_secrets_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id)): Path<(String, String)>,
) -> Result<Json<Vec<ProviderSecretMetadata>>, ApiError> {
    let provider_secrets = provider_secret_store(&state)?;
    let ProjectPath {
        tenant_id,
        project_id,
    } = ProjectPath::parse(tenant_id, project_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let secrets = provider_secrets
        .list_secret_metadata(tenant_id, project_id)
        .await?;
    Ok(Json(secrets))
}

#[utoipa::path(
    post,
    path = "/v1/provider-secrets/{tenant_id}/{project_id}/{provider_secret_id}/revoke",
    tag = "providerSecrets",
    operation_id = "revokeProviderSecret",
    params(
        ("tenant_id" = String, Path, description = "tenant_id"),
        ("project_id" = String, Path, description = "project_id"),
        ("provider_secret_id" = String, Path, description = "provider_secret_id"),
        ("authorization" = Option<String>, Header, description = "Bearer API token for strict auth"),
        ("x-beater-api-key" = Option<String>, Header, description = "API key alternative for strict auth"),
        ("x-beater-project-id" = Option<String>, Header, description = "Strict-auth project scope"),
        ("x-beater-environment-id" = Option<String>, Header, description = "Strict-auth environment scope"),
    ),
    responses(
        (status = 200, description = "Revoke a provider secret", body = RevokedProviderSecret),
        (status = 400, description = "Invalid request, scope, or filter", body = ErrorResponse),
        (status = 401, description = "Missing or invalid credentials", body = ErrorResponse),
        (status = 403, description = "Credentials lack the required scope", body = ErrorResponse),
        (status = 404, description = "Resource not found", body = ErrorResponse),
    )
)]
pub(crate) async fn revoke_provider_secret_route(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path((tenant_id, project_id, provider_secret_id)): Path<(String, String, String)>,
) -> Result<Json<RevokedProviderSecret>, ApiError> {
    let provider_secrets = provider_secret_store(&state)?;
    let ProviderSecretPath {
        project: ProjectPath {
            tenant_id,
            project_id,
        },
        provider_secret_id,
    } = ProviderSecretPath::parse(tenant_id, project_id, provider_secret_id)?;
    authorize_project_route(&state, &headers, &tenant_id, &project_id, ApiScope::Admin).await?;
    let revoked = provider_secrets
        .revoke_secret(
            tenant_id,
            project_id,
            provider_secret_id.clone(),
            Utc::now(),
        )
        .await?
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "provider secret {} not found",
                provider_secret_id.as_str()
            ))
        })?;
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
    use beater_secrets::{ProviderSecretStore, SqliteProviderSecretStore};
    use beater_security::{ApiScope, CreatedApiKey};
    use beater_store::TraceStore;
    use beater_store_obj::FsArtifactStore;
    use beater_store_sql::SqliteTraceStore;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::{router, ApiState};

    /// Auth-required app with a configured provider-secret store and a minted key.
    async fn auth_app_with_secrets(
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

        let secrets: Arc<dyn ProviderSecretStore> =
            Arc::new(SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let mut state = ApiState::new(ingest, traces).require_auth(api_keys);
        // No public `with_provider_secrets` builder yet; set the field directly
        // (the test is a descendant module of the `ApiState` definition).
        state.provider_secrets = Some(secrets);
        (router(state), created, tempdir)
    }

    #[tokio::test]
    async fn create_provider_secret_rejects_invalid_tenant_id_with_400() {
        let (app, created, _tempdir) =
            auth_app_with_secrets(BTreeSet::from([ApiScope::Admin])).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/provider-secrets/bad%20id/proj")
                    .header("x-beater-api-key", &created.secret)
                    .header("x-beater-project-id", "proj")
                    .header("x-beater-environment-id", "prod")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"provider":"openai","display_name":"k","secret_value":"s"}"#
                            .to_string(),
                    ))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn list_provider_secrets_rejects_insufficient_scope_with_403() {
        let (app, created, _tempdir) =
            auth_app_with_secrets(BTreeSet::from([ApiScope::TraceRead])).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/provider-secrets/acme/proj")
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

    #[tokio::test]
    async fn revoke_provider_secret_rejects_invalid_secret_id_with_400() {
        let (app, created, _tempdir) =
            auth_app_with_secrets(BTreeSet::from([ApiScope::Admin])).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/provider-secrets/acme/proj/bad%20secret/revoke")
                    .header("x-beater-api-key", &created.secret)
                    .header("x-beater-project-id", "proj")
                    .header("x-beater-environment-id", "prod")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
