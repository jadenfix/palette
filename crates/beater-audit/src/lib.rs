use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{ApiKeyId, AuditEventId, EnvironmentId, ProjectId, TenantId, Timestamp, TraceId};
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    PiiUnmask,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PiiUnmask => "pii_unmask",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Allowed,
    Denied,
}

impl AuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditEventInsert {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: Option<EnvironmentId>,
    pub actor_api_key_id: Option<ApiKeyId>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub attributes: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub audit_event_id: AuditEventId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: Option<EnvironmentId>,
    pub actor_api_key_id: Option<ApiKeyId>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub attributes: Value,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PiiUnmaskAuditInput {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: Option<EnvironmentId>,
    pub actor_api_key_id: Option<ApiKeyId>,
    pub trace_id: TraceId,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub attributes: Value,
}

#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn append_event(&self, insert: AuditEventInsert) -> StoreResult<AuditEvent>;

    async fn list_events(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<AuditEvent>>;
}

#[derive(Clone)]
pub struct SqliteAuditStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAuditStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory audit sqlite")?;
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
                .with_context(|| format!("create audit sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite audit store {}", path.display()))?;
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

                CREATE TABLE IF NOT EXISTS audit_events (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    audit_event_id TEXT NOT NULL,
                    environment_id TEXT,
                    actor_api_key_id TEXT,
                    action TEXT NOT NULL,
                    resource_type TEXT NOT NULL,
                    resource_id TEXT NOT NULL,
                    outcome TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    event_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, audit_event_id)
                );

                CREATE INDEX IF NOT EXISTS idx_audit_events_list
                  ON audit_events (tenant_id, project_id, created_at, audit_event_id);

                CREATE INDEX IF NOT EXISTS idx_audit_events_resource
                  ON audit_events (tenant_id, project_id, resource_type, resource_id);
                "#,
            )
            .context("initialize sqlite audit store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("sqlite audit connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl AuditStore for SqliteAuditStore {
    async fn append_event(&self, insert: AuditEventInsert) -> StoreResult<AuditEvent> {
        let event = AuditEvent {
            audit_event_id: AuditEventId::new(Uuid::new_v4().to_string())
                .map_err(StoreError::backend)?,
            tenant_id: insert.tenant_id,
            project_id: insert.project_id,
            environment_id: insert.environment_id,
            actor_api_key_id: insert.actor_api_key_id,
            action: insert.action,
            resource_type: insert.resource_type,
            resource_id: insert.resource_id,
            outcome: insert.outcome,
            reason: insert.reason,
            attributes: insert.attributes,
            created_at: Utc::now(),
        };
        let event_json = serde_json::to_string(&event)
            .context("serialize audit event")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO audit_events
                  (tenant_id, project_id, audit_event_id, environment_id, actor_api_key_id,
                   action, resource_type, resource_id, outcome, created_at, event_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    event.tenant_id.as_str(),
                    event.project_id.as_str(),
                    event.audit_event_id.as_str(),
                    event.environment_id.as_ref().map(|id| id.as_str()),
                    event.actor_api_key_id.as_ref().map(|id| id.as_str()),
                    event.action.as_str(),
                    event.resource_type.as_str(),
                    event.resource_id.as_str(),
                    event.outcome.as_str(),
                    event.created_at.to_rfc3339(),
                    event_json
                ],
            )
            .context("insert audit event")
            .into_store()?;
        Ok(event)
    }

    async fn list_events(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<AuditEvent>> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT event_json
                FROM audit_events
                WHERE tenant_id = ?1 AND project_id = ?2
                ORDER BY created_at ASC, audit_event_id ASC
                "#,
            )
            .context("prepare audit event list query")
            .into_store()?;
        let rows = statement
            .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
                row.get::<_, String>(0)
            })
            .context("query audit events")
            .into_store()?;

        let mut events = Vec::new();
        for row in rows {
            let event_json = row.context("read audit event row").into_store()?;
            events.push(
                serde_json::from_str(&event_json)
                    .context("decode audit event")
                    .into_store()?,
            );
        }
        Ok(events)
    }
}

pub fn pii_unmask_event(input: PiiUnmaskAuditInput) -> AuditEventInsert {
    AuditEventInsert {
        tenant_id: input.tenant_id,
        project_id: input.project_id,
        environment_id: input.environment_id,
        actor_api_key_id: input.actor_api_key_id,
        action: AuditAction::PiiUnmask,
        resource_type: "trace".to_string(),
        resource_id: input.trace_id.as_str().to_string(),
        outcome: input.outcome,
        reason: input.reason,
        attributes: input.attributes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn sqlite_audit_store_persists_privileged_unmask_events() -> anyhow::Result<()> {
        let store = SqliteAuditStore::in_memory()?;
        let event = store
            .append_event(pii_unmask_event(PiiUnmaskAuditInput {
                tenant_id: TenantId::new("tenant")?,
                project_id: ProjectId::new("project")?,
                environment_id: Some(EnvironmentId::new("prod")?),
                actor_api_key_id: Some(ApiKeyId::new("key")?),
                trace_id: TraceId::new("trace")?,
                outcome: AuditOutcome::Allowed,
                reason: Some("debugging incident".to_string()),
                attributes: json!({"sensitive_refs": 2}),
            }))
            .await?;
        assert_eq!(event.action, AuditAction::PiiUnmask);
        assert_eq!(event.outcome, AuditOutcome::Allowed);

        let events = store
            .list_events(TenantId::new("tenant")?, ProjectId::new("project")?)
            .await?;
        assert_eq!(events, vec![event]);
        Ok(())
    }
}
