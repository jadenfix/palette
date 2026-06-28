//! Request-hash cache for the gateway.
//!
//! Mirrors the `beater-judge` ledger cache: a successful completion is keyed by
//! the request hash so an identical request returns the cached response at zero
//! incremental cost. Two backends are provided — an in-memory map (tests / dev)
//! and a sqlite-backed store (the same shape `beater-judge` uses).

use crate::ChatCompletionResponse;
use async_trait::async_trait;
use beater_core::{Money, ProjectId, Sha256Hash, TenantId};
use beater_schema::ModelRef;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// A cached completion: the OpenAI-compatible response plus its metered cost.
#[derive(Clone, Debug, PartialEq)]
pub struct CachedCompletion {
    pub response: ChatCompletionResponse,
    pub cost: Money,
}

/// Error type for the cache layer.
#[derive(Debug, thiserror::Error)]
#[error("gateway cache error: {0}")]
pub struct CacheError(pub String);

impl CacheError {
    fn new(message: impl std::fmt::Display) -> Self {
        Self(message.to_string())
    }
}

/// The gateway's request-hash cache abstraction.
#[async_trait]
pub trait GatewayCache: Send + Sync {
    async fn get(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> Result<Option<CachedCompletion>, CacheError>;

    async fn put(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
        model: ModelRef,
        value: CachedCompletion,
    ) -> Result<(), CacheError>;
}

#[async_trait]
impl<T> GatewayCache for Arc<T>
where
    T: GatewayCache + ?Sized,
{
    async fn get(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> Result<Option<CachedCompletion>, CacheError> {
        (**self).get(tenant_id, project_id, request_hash).await
    }

    async fn put(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
        model: ModelRef,
        value: CachedCompletion,
    ) -> Result<(), CacheError> {
        (**self)
            .put(tenant_id, project_id, request_hash, model, value)
            .await
    }
}

// ---------------------------------------------------------------------------
// In-memory backend
// ---------------------------------------------------------------------------

type CacheKey = (String, String, String);

/// An in-memory request-hash cache (tests / dev).
#[derive(Clone, Default)]
pub struct InMemoryGatewayCache {
    entries: Arc<Mutex<HashMap<CacheKey, CachedCompletion>>>,
}

impl InMemoryGatewayCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(tenant_id: &TenantId, project_id: &ProjectId, request_hash: &Sha256Hash) -> CacheKey {
        (
            tenant_id.as_str().to_string(),
            project_id.as_str().to_string(),
            request_hash.as_str().to_string(),
        )
    }
}

#[async_trait]
impl GatewayCache for InMemoryGatewayCache {
    async fn get(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> Result<Option<CachedCompletion>, CacheError> {
        let entries = self
            .entries
            .lock()
            .map_err(|err| CacheError::new(format!("cache mutex poisoned: {err}")))?;
        Ok(entries
            .get(&Self::key(&tenant_id, &project_id, &request_hash))
            .cloned())
    }

    async fn put(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
        _model: ModelRef,
        value: CachedCompletion,
    ) -> Result<(), CacheError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|err| CacheError::new(format!("cache mutex poisoned: {err}")))?;
        entries.insert(Self::key(&tenant_id, &project_id, &request_hash), value);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Sqlite backend
// ---------------------------------------------------------------------------

/// A sqlite-backed request-hash cache, mirroring the `beater-judge` ledger shape.
#[derive(Clone)]
pub struct SqliteGatewayCache {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteGatewayCache {
    pub fn in_memory() -> Result<Self, CacheError> {
        let connection = Connection::open_in_memory().map_err(CacheError::new)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, CacheError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(CacheError::new)?;
        }
        let connection = Connection::open(path).map_err(CacheError::new)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> Result<(), CacheError> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;

                CREATE TABLE IF NOT EXISTS gateway_cache (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    request_hash TEXT NOT NULL,
                    model_provider TEXT NOT NULL,
                    model_name TEXT NOT NULL,
                    response_json TEXT NOT NULL,
                    cost_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, request_hash)
                );
                "#,
            )
            .map_err(CacheError::new)?;
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CacheError> {
        self.connection
            .lock()
            .map_err(|err| CacheError::new(format!("gateway cache mutex poisoned: {err}")))
    }
}

#[async_trait]
impl GatewayCache for SqliteGatewayCache {
    async fn get(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
    ) -> Result<Option<CachedCompletion>, CacheError> {
        let connection = self.lock()?;
        let row = connection
            .query_row(
                r#"
                SELECT response_json, cost_json
                FROM gateway_cache
                WHERE tenant_id = ?1 AND project_id = ?2 AND request_hash = ?3
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    request_hash.as_str()
                ],
                |row| {
                    let response_json: String = row.get(0)?;
                    let cost_json: String = row.get(1)?;
                    Ok((response_json, cost_json))
                },
            )
            .optional()
            .map_err(CacheError::new)?;
        let Some((response_json, cost_json)) = row else {
            return Ok(None);
        };
        let response = serde_json::from_str::<ChatCompletionResponse>(&response_json)
            .map_err(CacheError::new)?;
        let cost = serde_json::from_str::<Money>(&cost_json).map_err(CacheError::new)?;
        Ok(Some(CachedCompletion { response, cost }))
    }

    async fn put(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        request_hash: Sha256Hash,
        model: ModelRef,
        value: CachedCompletion,
    ) -> Result<(), CacheError> {
        let response_json = serde_json::to_string(&value.response).map_err(CacheError::new)?;
        let cost_json = serde_json::to_string(&value.cost).map_err(CacheError::new)?;
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT OR REPLACE INTO gateway_cache
                  (tenant_id, project_id, request_hash, model_provider, model_name,
                   response_json, cost_json, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    request_hash.as_str(),
                    model.provider.as_str(),
                    model.name.as_str(),
                    response_json,
                    cost_json,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(CacheError::new)?;
        Ok(())
    }
}
