use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{ProjectId, ProviderSecretId, TenantId, Timestamp};
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod encrypted;

pub use encrypted::{EncryptedSqliteProviderSecretStore, SecretEncryptionKey, SecretKeyring};

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct PutProviderSecretRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub provider: String,
    pub display_name: String,
    pub secret_value: String,
}

impl Debug for PutProviderSecretRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PutProviderSecretRequest")
            .field("tenant_id", &self.tenant_id)
            .field("project_id", &self.project_id)
            .field("provider", &self.provider)
            .field("display_name", &self.display_name)
            .field("secret_value", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProviderSecretMetadata {
    pub provider_secret_id: ProviderSecretId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub provider: String,
    pub display_name: String,
    pub active: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub rotated_at: Option<Timestamp>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProviderSecret {
    pub metadata: ProviderSecretMetadata,
    secret_value: String,
}

impl ProviderSecret {
    pub(crate) fn from_decrypted(
        metadata: ProviderSecretMetadata,
        secret_value: impl Into<String>,
    ) -> Self {
        Self {
            metadata,
            secret_value: secret_value.into(),
        }
    }

    pub fn secret_value(&self) -> &str {
        &self.secret_value
    }
}

impl Debug for ProviderSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderSecret")
            .field("metadata", &self.metadata)
            .field("secret_value", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RevokedProviderSecret {
    pub provider_secret_id: ProviderSecretId,
    pub active: bool,
    #[schema(value_type = String, format = DateTime)]
    pub rotated_at: Timestamp,
}

#[async_trait]
pub trait ProviderSecretStore: Send + Sync {
    async fn put_secret(
        &self,
        request: PutProviderSecretRequest,
    ) -> StoreResult<ProviderSecretMetadata>;

    async fn get_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
    ) -> StoreResult<Option<ProviderSecret>>;

    async fn revoke_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
        rotated_at: Timestamp,
    ) -> StoreResult<Option<RevokedProviderSecret>>;

    async fn list_secret_metadata(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<ProviderSecretMetadata>>;
}

#[async_trait]
impl<T> ProviderSecretStore for Arc<T>
where
    T: ProviderSecretStore + ?Sized,
{
    async fn put_secret(
        &self,
        request: PutProviderSecretRequest,
    ) -> StoreResult<ProviderSecretMetadata> {
        (**self).put_secret(request).await
    }

    async fn get_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
    ) -> StoreResult<Option<ProviderSecret>> {
        (**self)
            .get_secret(tenant_id, project_id, provider_secret_id)
            .await
    }

    async fn revoke_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
        rotated_at: Timestamp,
    ) -> StoreResult<Option<RevokedProviderSecret>> {
        (**self)
            .revoke_secret(tenant_id, project_id, provider_secret_id, rotated_at)
            .await
    }

    async fn list_secret_metadata(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<ProviderSecretMetadata>> {
        (**self).list_secret_metadata(tenant_id, project_id).await
    }
}

#[derive(Clone)]
pub struct SqliteProviderSecretStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteProviderSecretStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection =
            Connection::open_in_memory().context("open in-memory provider secret sqlite")?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("create provider secret sqlite dir {}", parent.display())
            })?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open provider secret sqlite store {}", path.display()))?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> anyhow::Result<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS provider_secrets (
                    provider_secret_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    provider TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    secret_value TEXT NOT NULL,
                    active INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    rotated_at TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_provider_secrets_scope
                ON provider_secrets (tenant_id, project_id, provider, active);
                "#,
            )
            .context("initialize provider secret sqlite store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("provider secret sqlite connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl ProviderSecretStore for SqliteProviderSecretStore {
    async fn put_secret(
        &self,
        request: PutProviderSecretRequest,
    ) -> StoreResult<ProviderSecretMetadata> {
        let metadata = ProviderSecretMetadata {
            provider_secret_id: ProviderSecretId::new(Uuid::new_v4().to_string())
                .map_err(StoreError::backend)?,
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            provider: request.provider,
            display_name: request.display_name,
            active: true,
            created_at: Utc::now(),
            rotated_at: None,
        };
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO provider_secrets
                  (provider_secret_id, tenant_id, project_id, provider, display_name,
                   secret_value, active, created_at, rotated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    metadata.provider_secret_id.as_str(),
                    metadata.tenant_id.as_str(),
                    metadata.project_id.as_str(),
                    metadata.provider.as_str(),
                    metadata.display_name.as_str(),
                    request.secret_value,
                    if metadata.active { 1_i64 } else { 0_i64 },
                    metadata.created_at.to_rfc3339(),
                    metadata.rotated_at.as_ref().map(|time| time.to_rfc3339()),
                ],
            )
            .context("insert provider secret")
            .into_store()?;
        Ok(metadata)
    }

    async fn get_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
    ) -> StoreResult<Option<ProviderSecret>> {
        let connection = self.lock().into_store()?;
        connection
            .query_row(
                r#"
                SELECT provider_secret_id, tenant_id, project_id, provider, display_name,
                       active, created_at, rotated_at, secret_value
                FROM provider_secrets
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND provider_secret_id = ?3
                  AND active = 1
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    provider_secret_id.as_str()
                ],
                decode_secret,
            )
            .optional()
            .context("get provider secret")
            .into_store()
    }

    async fn revoke_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
        rotated_at: Timestamp,
    ) -> StoreResult<Option<RevokedProviderSecret>> {
        let connection = self.lock().into_store()?;
        let changed = connection
            .execute(
                r#"
                UPDATE provider_secrets
                SET active = 0, rotated_at = ?4
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND provider_secret_id = ?3
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    provider_secret_id.as_str(),
                    rotated_at.to_rfc3339()
                ],
            )
            .context("revoke provider secret")
            .into_store()?;
        if changed == 0 {
            return Ok(None);
        }
        Ok(Some(RevokedProviderSecret {
            provider_secret_id,
            active: false,
            rotated_at,
        }))
    }

    async fn list_secret_metadata(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<ProviderSecretMetadata>> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT provider_secret_id, tenant_id, project_id, provider, display_name,
                       active, created_at, rotated_at
                FROM provider_secrets
                WHERE tenant_id = ?1 AND project_id = ?2
                ORDER BY created_at DESC, provider_secret_id ASC
                "#,
            )
            .context("prepare list provider secrets")
            .into_store()?;
        let rows = statement
            .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
                decode_metadata(row)
            })
            .context("query provider secret metadata")
            .into_store()?;
        let mut records = Vec::new();
        for row in rows {
            records.push(
                row.context("decode provider secret metadata")
                    .into_store()?,
            );
        }
        Ok(records)
    }
}

fn decode_secret(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderSecret> {
    Ok(ProviderSecret {
        metadata: decode_metadata(row)?,
        secret_value: row.get(8)?,
    })
}

fn decode_metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderSecretMetadata> {
    let created_at = parse_time(row.get::<_, String>(6)?)?;
    let rotated_at = row
        .get::<_, Option<String>>(7)?
        .map(parse_time)
        .transpose()?;
    Ok(ProviderSecretMetadata {
        provider_secret_id: ProviderSecretId::new(row.get::<_, String>(0)?)
            .map_err(sql_decode_error)?,
        tenant_id: TenantId::new(row.get::<_, String>(1)?).map_err(sql_decode_error)?,
        project_id: ProjectId::new(row.get::<_, String>(2)?).map_err(sql_decode_error)?,
        provider: row.get(3)?,
        display_name: row.get(4)?,
        active: row.get::<_, i64>(5)? != 0,
        created_at,
        rotated_at,
    })
}

fn parse_time(value: String) -> rusqlite::Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                value.len(),
                rusqlite::types::Type::Text,
                Box::new(err),
            )
        })
}

fn sql_decode_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[cfg(test)]
pub(crate) async fn assert_provider_secret_scope_isolation<S>(store: &S)
where
    S: ProviderSecretStore + ?Sized,
{
    let owner_tenant = test_tenant_id("tenant-a");
    let other_tenant = test_tenant_id("tenant-b");
    let owner_project = test_project_id("project-a");
    let other_project = test_project_id("project-b");

    let owner_secret =
        put_colliding_provider_secret(store, &owner_tenant, &owner_project, "sk-owner").await;
    let tenant_collision =
        put_colliding_provider_secret(store, &other_tenant, &owner_project, "sk-tenant").await;
    let project_collision =
        put_colliding_provider_secret(store, &owner_tenant, &other_project, "sk-project").await;

    let cross_tenant_get = store
        .get_secret(
            other_tenant.clone(),
            owner_project.clone(),
            owner_secret.provider_secret_id.clone(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(cross_tenant_get.is_none());

    let cross_project_get = store
        .get_secret(
            owner_tenant.clone(),
            other_project.clone(),
            owner_secret.provider_secret_id.clone(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(cross_project_get.is_none());

    let cross_tenant_revoke = store
        .revoke_secret(
            other_tenant.clone(),
            owner_project.clone(),
            owner_secret.provider_secret_id.clone(),
            Utc::now(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(cross_tenant_revoke.is_none());

    let cross_project_revoke = store
        .revoke_secret(
            owner_tenant.clone(),
            other_project.clone(),
            owner_secret.provider_secret_id.clone(),
            Utc::now(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(cross_project_revoke.is_none());

    let loaded = store
        .get_secret(
            owner_tenant.clone(),
            owner_project.clone(),
            owner_secret.provider_secret_id.clone(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("owner-scoped provider secret should remain active"));
    assert_eq!(loaded.secret_value(), "sk-owner");
    assert!(loaded.metadata.active);

    assert_single_scoped_metadata(
        store,
        owner_tenant.clone(),
        owner_project.clone(),
        &owner_secret,
    )
    .await;
    assert_single_scoped_metadata(
        store,
        other_tenant.clone(),
        owner_project,
        &tenant_collision,
    )
    .await;
    assert_single_scoped_metadata(store, owner_tenant, other_project, &project_collision).await;
}

#[cfg(test)]
async fn put_colliding_provider_secret<S>(
    store: &S,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    secret_value: &str,
) -> ProviderSecretMetadata
where
    S: ProviderSecretStore + ?Sized,
{
    store
        .put_secret(PutProviderSecretRequest {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            provider: "openai".to_string(),
            display_name: "shared judge".to_string(),
            secret_value: secret_value.to_string(),
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"))
}

#[cfg(test)]
async fn assert_single_scoped_metadata<S>(
    store: &S,
    tenant_id: TenantId,
    project_id: ProjectId,
    expected: &ProviderSecretMetadata,
) where
    S: ProviderSecretStore + ?Sized,
{
    let listed = store
        .list_secret_metadata(tenant_id.clone(), project_id.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(listed.len(), 1);
    let actual = listed
        .first()
        .unwrap_or_else(|| panic!("scope should contain one provider secret"));
    assert_eq!(
        actual.provider_secret_id.as_str(),
        expected.provider_secret_id.as_str()
    );
    assert_eq!(actual.tenant_id.as_str(), tenant_id.as_str());
    assert_eq!(actual.project_id.as_str(), project_id.as_str());
    assert_eq!(actual.provider, "openai");
    assert_eq!(actual.display_name, "shared judge");
    assert!(actual.active);
}

#[cfg(test)]
fn test_tenant_id(value: &str) -> TenantId {
    TenantId::new(value).unwrap_or_else(|err| panic!("{err}"))
}

#[cfg(test)]
fn test_project_id(value: &str) -> ProjectId {
    ProjectId::new(value).unwrap_or_else(|err| panic!("{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn provider_secret_metadata_never_serializes_secret_material() {
        let store = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let request = PutProviderSecretRequest {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            provider: "openai".to_string(),
            display_name: "production judge".to_string(),
            secret_value: "sk-test-secret".to_string(),
        };
        assert!(!format!("{request:?}").contains("sk-test-secret"));

        let metadata = store
            .put_secret(request)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let serialized = serde_json::to_string(&metadata).unwrap_or_else(|err| panic!("{err}"));
        assert!(!serialized.contains("sk-test-secret"));

        let loaded = store
            .get_secret(
                metadata.tenant_id.clone(),
                metadata.project_id.clone(),
                metadata.provider_secret_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("provider secret should exist"));
        assert_eq!(loaded.secret_value(), "sk-test-secret");
        assert!(!format!("{loaded:?}").contains("sk-test-secret"));
    }

    #[tokio::test]
    async fn revoked_provider_secrets_are_not_returned_to_broker_path() {
        let store = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let metadata = store
            .put_secret(PutProviderSecretRequest {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                provider: "anthropic".to_string(),
                display_name: "judge".to_string(),
                secret_value: "sk-ant-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let revoked = store
            .revoke_secret(
                tenant_id.clone(),
                project_id.clone(),
                metadata.provider_secret_id.clone(),
                Utc::now(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("provider secret should be revoked"));
        assert!(!revoked.active);

        let loaded = store
            .get_secret(
                tenant_id.clone(),
                project_id.clone(),
                metadata.provider_secret_id,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(loaded.is_none());

        let listed = store
            .list_secret_metadata(tenant_id, project_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(listed.len(), 1);
        assert!(!listed[0].active);
        assert!(listed[0].rotated_at.is_some());
    }

    #[tokio::test]
    async fn sqlite_provider_secret_store_enforces_tenant_project_scope() {
        let store = SqliteProviderSecretStore::in_memory().unwrap_or_else(|err| panic!("{err}"));

        assert_provider_secret_scope_isolation(&store).await;
    }
}
