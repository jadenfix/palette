//! Resource-oriented route modules and shared typed-path helpers.
//!
//! `crate::lib` historically parsed every `/v1` route's path segments into typed
//! ids inline inside each handler, then ran the same authorization / project-scope
//! checks by hand. This module centralizes the *path-parsing* half of that pattern
//! in a small set of typed structs (`ProjectPath`, `EnvironmentPath`, …) and hosts
//! the per-resource handler modules that consume them.
//!
//! The structs deliberately do **not** perform authorization themselves — auth is
//! still routed through [`crate::authorize`] and friends so the 401/403 behavior
//! stays in one place. What the typed paths own is the "parse the raw string
//! segments into validated ids, returning `400` on an invalid id" half, which was
//! the most-duplicated boilerplate across handlers.

use crate::ApiError;
use beater_core::{EnvironmentId, ProjectId, ProviderSecretId, TenantId};

pub mod api_keys;
pub mod provider_secrets;

/// The `{tenant_id}/{project_id}` prefix shared by project-scoped routes
/// (provider secrets, judge, usage, audit, datasets, gates, review queues, …).
///
/// Parsing centralizes the `TenantId::new` / `ProjectId::new` calls that every
/// project-scoped handler used to repeat, so an invalid id segment yields the
/// same `400 Bad Request` everywhere via [`ApiError`]'s `From<IdError>`.
#[derive(Clone, Debug)]
pub struct ProjectPath {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
}

impl ProjectPath {
    /// Validate the raw `(tenant_id, project_id)` path tuple, returning `400` on
    /// an invalid id segment.
    pub fn parse(tenant_id: String, project_id: String) -> Result<Self, ApiError> {
        Ok(Self {
            tenant_id: TenantId::new(tenant_id)?,
            project_id: ProjectId::new(project_id)?,
        })
    }
}

/// The `{tenant_id}/{project_id}/{environment_id}` prefix shared by
/// environment-scoped routes (api-keys create, OTLP ingest, import, …).
#[derive(Clone, Debug)]
pub struct EnvironmentPath {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
}

impl EnvironmentPath {
    /// Validate the raw `(tenant_id, project_id, environment_id)` path tuple,
    /// returning `400` on an invalid id segment.
    pub fn parse(
        tenant_id: String,
        project_id: String,
        environment_id: String,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            tenant_id: TenantId::new(tenant_id)?,
            project_id: ProjectId::new(project_id)?,
            environment_id: EnvironmentId::new(environment_id)?,
        })
    }
}

/// A `{provider_secret_id}` suffix on top of a [`ProjectPath`], used by the
/// provider-secret revoke route.
#[derive(Clone, Debug)]
pub struct ProviderSecretPath {
    pub project: ProjectPath,
    pub provider_secret_id: ProviderSecretId,
}

impl ProviderSecretPath {
    /// Validate the raw `(tenant, project, provider_secret)` path tuple,
    /// returning `400` on an invalid id segment.
    pub fn parse(
        tenant_id: String,
        project_id: String,
        provider_secret_id: String,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            project: ProjectPath::parse(tenant_id, project_id)?,
            provider_secret_id: ProviderSecretId::new(provider_secret_id)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{EnvironmentPath, ProjectPath, ProviderSecretPath};
    use http::StatusCode;

    #[test]
    fn project_path_rejects_invalid_id_with_400() {
        let ok = ProjectPath::parse("acme".to_string(), "proj".to_string());
        assert!(ok.is_ok());
        // Whitespace is an invalid id segment.
        let err = ProjectPath::parse("bad id".to_string(), "proj".to_string())
            .err()
            .expect("whitespace tenant id must be rejected");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        // Empty is invalid too.
        let err = ProjectPath::parse("acme".to_string(), String::new())
            .err()
            .expect("empty project id must be rejected");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn environment_path_rejects_invalid_id_with_400() {
        assert!(
            EnvironmentPath::parse("acme".to_string(), "proj".to_string(), "prod".to_string())
                .is_ok()
        );
        let err = EnvironmentPath::parse(
            "acme".to_string(),
            "proj".to_string(),
            "bad env".to_string(),
        )
        .err()
        .expect("whitespace environment id must be rejected");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn provider_secret_path_rejects_invalid_id_with_400() {
        let err = ProviderSecretPath::parse(
            "acme".to_string(),
            "proj".to_string(),
            "bad secret".to_string(),
        )
        .err()
        .expect("whitespace provider-secret id must be rejected");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }
}
