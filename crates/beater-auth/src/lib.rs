use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{ApiKeyId, EnvironmentId, ProjectId, TenantId, Timestamp};
use beater_security::{create_api_key, ApiKeyRecord, ApiScope, CreatedApiKey};
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub scopes: BTreeSet<ApiScope>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokedApiKey {
    pub api_key_id: ApiKeyId,
    pub active: bool,
    pub rotated_at: Timestamp,
}

#[async_trait]
pub trait ApiKeyStore: Send + Sync {
    async fn put_key(&self, record: ApiKeyRecord) -> StoreResult<()>;

    async fn get_key(&self, api_key_id: ApiKeyId) -> StoreResult<Option<ApiKeyRecord>>;

    async fn create_key(&self, request: CreateApiKeyRequest) -> StoreResult<CreatedApiKey> {
        let created = create_api_key(
            request.tenant_id,
            request.project_id,
            request.environment_id,
            request.scopes,
        )
        .map_err(StoreError::backend)?;
        self.put_key(created.record.clone()).await?;
        Ok(created)
    }

    async fn revoke_key(
        &self,
        api_key_id: ApiKeyId,
        rotated_at: Timestamp,
    ) -> StoreResult<Option<RevokedApiKey>>;

    async fn touch_last_used(&self, api_key_id: ApiKeyId, used_at: Timestamp) -> StoreResult<()>;
}

#[derive(Clone)]
pub struct SqliteApiKeyStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteApiKeyStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory api key sqlite")?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create api key sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open api key sqlite store {}", path.display()))?;
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

                CREATE TABLE IF NOT EXISTS api_keys (
                    api_key_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    environment_id TEXT NOT NULL,
                    secret_hash TEXT NOT NULL,
                    scopes_json TEXT NOT NULL,
                    active INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    rotated_at TEXT,
                    last_used_at TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_api_keys_scope
                ON api_keys (tenant_id, project_id, environment_id, active);
                "#,
            )
            .context("initialize api key sqlite store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("api key sqlite connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl ApiKeyStore for SqliteApiKeyStore {
    async fn put_key(&self, record: ApiKeyRecord) -> StoreResult<()> {
        let connection = self.lock().into_store()?;
        let scopes_json = serde_json::to_string(&record.scopes)
            .context("serialize api scopes")
            .into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO api_keys
                  (api_key_id, tenant_id, project_id, environment_id, secret_hash, scopes_json,
                   active, created_at, rotated_at, last_used_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(api_key_id) DO UPDATE SET
                  tenant_id = excluded.tenant_id,
                  project_id = excluded.project_id,
                  environment_id = excluded.environment_id,
                  secret_hash = excluded.secret_hash,
                  scopes_json = excluded.scopes_json,
                  active = excluded.active,
                  created_at = excluded.created_at,
                  rotated_at = excluded.rotated_at,
                  last_used_at = excluded.last_used_at
                "#,
                params![
                    record.api_key_id.as_str(),
                    record.tenant_id.as_str(),
                    record.project_id.as_str(),
                    record.environment_id.as_str(),
                    record.secret_hash,
                    scopes_json,
                    if record.active { 1_i64 } else { 0_i64 },
                    record.created_at.to_rfc3339(),
                    record.rotated_at.map(|time| time.to_rfc3339()),
                    record.last_used_at.map(|time| time.to_rfc3339()),
                ],
            )
            .context("upsert api key")
            .into_store()?;
        Ok(())
    }

    async fn get_key(&self, api_key_id: ApiKeyId) -> StoreResult<Option<ApiKeyRecord>> {
        let connection = self.lock().into_store()?;
        connection
            .query_row(
                r#"
                SELECT api_key_id, tenant_id, project_id, environment_id, secret_hash, scopes_json,
                       active, created_at, rotated_at, last_used_at
                FROM api_keys
                WHERE api_key_id = ?1
                "#,
                params![api_key_id.as_str()],
                decode_record,
            )
            .optional()
            .context("get api key")
            .into_store()
    }

    async fn revoke_key(
        &self,
        api_key_id: ApiKeyId,
        rotated_at: Timestamp,
    ) -> StoreResult<Option<RevokedApiKey>> {
        let connection = self.lock().into_store()?;
        let changed = connection
            .execute(
                r#"
                UPDATE api_keys
                SET active = 0, rotated_at = ?2
                WHERE api_key_id = ?1
                "#,
                params![api_key_id.as_str(), rotated_at.to_rfc3339()],
            )
            .context("revoke api key")
            .into_store()?;
        if changed == 0 {
            return Ok(None);
        }
        Ok(Some(RevokedApiKey {
            api_key_id,
            active: false,
            rotated_at,
        }))
    }

    async fn touch_last_used(&self, api_key_id: ApiKeyId, used_at: Timestamp) -> StoreResult<()> {
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                UPDATE api_keys
                SET last_used_at = ?2
                WHERE api_key_id = ?1
                "#,
                params![api_key_id.as_str(), used_at.to_rfc3339()],
            )
            .context("touch api key last_used_at")
            .into_store()?;
        Ok(())
    }
}

fn decode_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<ApiKeyRecord> {
    let scopes_json: String = row.get(5)?;
    let scopes = serde_json::from_str::<BTreeSet<ApiScope>>(&scopes_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            scopes_json.len(),
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })?;
    let created_at = parse_time(row.get::<_, String>(7)?)?;
    let rotated_at = row
        .get::<_, Option<String>>(8)?
        .map(parse_time)
        .transpose()?;
    let last_used_at = row
        .get::<_, Option<String>>(9)?
        .map(parse_time)
        .transpose()?;
    Ok(ApiKeyRecord {
        api_key_id: ApiKeyId::new(row.get::<_, String>(0)?).map_err(sql_decode_error)?,
        tenant_id: TenantId::new(row.get::<_, String>(1)?).map_err(sql_decode_error)?,
        project_id: ProjectId::new(row.get::<_, String>(2)?).map_err(sql_decode_error)?,
        environment_id: EnvironmentId::new(row.get::<_, String>(3)?).map_err(sql_decode_error)?,
        secret_hash: row.get(4)?,
        scopes,
        active: row.get::<_, i64>(6)? != 0,
        created_at,
        rotated_at,
        last_used_at,
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
mod tests {
    use super::*;
    use beater_security::{verify_api_key, SecurityError};

    #[tokio::test]
    async fn sqlite_store_creates_reads_and_revokes_keys() {
        let store = SqliteApiKeyStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let mut scopes = BTreeSet::new();
        scopes.insert(ApiScope::TraceWrite);
        let created = store
            .create_key(CreateApiKeyRequest {
                tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
                scopes,
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let loaded = store
            .get_key(created.record.api_key_id.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("api key should exist"));
        assert_eq!(loaded.api_key_id, created.record.api_key_id);
        assert_ne!(loaded.secret_hash, created.secret);
        verify_api_key(
            &loaded,
            &created.secret,
            &TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            &ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            &EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            ApiScope::TraceWrite,
        )
        .unwrap_or_else(|err| panic!("{err}"));

        let revoked = store
            .revoke_key(created.record.api_key_id.clone(), Utc::now())
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("api key should be revoked"));
        assert!(!revoked.active);

        let loaded = store
            .get_key(created.record.api_key_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("api key should still exist"));
        assert!(matches!(
            verify_api_key(
                &loaded,
                &created.secret,
                &TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                &ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                &EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
                ApiScope::TraceWrite,
            ),
            Err(SecurityError::InactiveApiKey)
        ));
    }
}
