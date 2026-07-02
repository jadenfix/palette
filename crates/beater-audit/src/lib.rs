use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    sha256_json_hash, ApiKeyId, AuditEventId, EnvironmentId, ProjectId, Sha256Hash, TenantId,
    Timestamp, TraceId,
};
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    PiiUnmask,
    ApiKeyCreate,
    ApiKeyRevoke,
    ProviderSecretCreate,
    ProviderSecretRevoke,
    ConnectorToolInvoke,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PiiUnmask => "pii_unmask",
            Self::ApiKeyCreate => "api_key_create",
            Self::ApiKeyRevoke => "api_key_revoke",
            Self::ProviderSecretCreate => "provider_secret_create",
            Self::ProviderSecretRevoke => "provider_secret_revoke",
            Self::ConnectorToolInvoke => "connector_tool_invoke",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
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
    #[schema(value_type = serde_json::Value)]
    pub attributes: Value,
    #[schema(value_type = String, format = DateTime)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnectorToolInvokeAuditInput {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: Option<EnvironmentId>,
    pub actor_api_key_id: Option<ApiKeyId>,
    pub tool_slug: String,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub attributes: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditChainVerification {
    pub valid: bool,
    pub checked_events: usize,
    pub legacy_events: usize,
    pub failure: Option<AuditChainFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditChainFailure {
    pub audit_event_id: AuditEventId,
    pub expected_previous_event_hash: Option<Sha256Hash>,
    pub actual_previous_event_hash: Option<Sha256Hash>,
    pub expected_event_hash: Sha256Hash,
    pub actual_event_hash: Option<Sha256Hash>,
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
                    previous_event_hash TEXT,
                    event_hash TEXT,
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
        ensure_audit_chain_columns(&connection)?;
        Ok(())
    }

    pub fn verify_chain(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<AuditChainVerification> {
        let connection = self.lock().into_store()?;
        verify_audit_chain(&connection, &tenant_id, &project_id)
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
        let mut connection = self.lock().into_store()?;
        let transaction = connection
            .transaction()
            .context("begin audit event append transaction")
            .into_store()?;
        let previous_event_hash =
            latest_audit_event_hash(&transaction, &event.tenant_id, &event.project_id)
                .into_store()?;
        let event_hash = audit_event_chain_hash(&event_json, previous_event_hash.as_ref())
            .context("hash audit event chain entry")
            .into_store()?;
        transaction
            .execute(
                r#"
                INSERT INTO audit_events
                  (tenant_id, project_id, audit_event_id, environment_id, actor_api_key_id,
                   action, resource_type, resource_id, outcome, created_at,
                   previous_event_hash, event_hash, event_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
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
                    previous_event_hash.as_ref().map(|hash| hash.as_str()),
                    event_hash.as_str(),
                    event_json
                ],
            )
            .context("insert audit event")
            .into_store()?;
        transaction
            .commit()
            .context("commit audit event append")
            .into_store()?;
        Ok(event)
    }

    async fn list_events(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<AuditEvent>> {
        let connection = self.lock().into_store()?;
        ensure_audit_chain_readable(verify_audit_chain(&connection, &tenant_id, &project_id)?)?;
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct AuditChainRow {
    audit_event_id: AuditEventId,
    event_json: String,
    previous_event_hash: Option<Sha256Hash>,
    event_hash: Option<Sha256Hash>,
}

fn verify_audit_chain(
    connection: &Connection,
    tenant_id: &TenantId,
    project_id: &ProjectId,
) -> StoreResult<AuditChainVerification> {
    let mut statement = connection
        .prepare(
            r#"
            SELECT audit_event_id, event_json, previous_event_hash, event_hash
            FROM audit_events
            WHERE tenant_id = ?1 AND project_id = ?2
            ORDER BY created_at ASC, audit_event_id ASC
            "#,
        )
        .context("prepare audit chain verification query")
        .into_store()?;
    let rows = statement
        .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
            Ok(AuditChainRow {
                audit_event_id: AuditEventId::new(row.get::<_, String>(0)?)
                    .map_err(sql_decode_error)?,
                event_json: row.get(1)?,
                previous_event_hash: row
                    .get::<_, Option<String>>(2)?
                    .map(Sha256Hash::new)
                    .transpose()
                    .map_err(sql_decode_error)?,
                event_hash: row
                    .get::<_, Option<String>>(3)?
                    .map(Sha256Hash::new)
                    .transpose()
                    .map_err(sql_decode_error)?,
            })
        })
        .context("query audit chain rows")
        .into_store()?;

    let mut previous_event_hash = None;
    let mut checked_events = 0;
    let mut legacy_events = 0;
    for row in rows {
        let row = row.context("read audit chain row").into_store()?;
        let expected_previous_event_hash = previous_event_hash.clone();
        if row.event_hash.is_none() && expected_previous_event_hash.is_none() {
            legacy_events += 1;
            continue;
        }

        let expected_event_hash =
            audit_event_chain_hash(&row.event_json, expected_previous_event_hash.as_ref())
                .into_store()?;
        let actual_event_hash = row.event_hash.clone();
        if row.previous_event_hash != expected_previous_event_hash
            || row.event_hash.as_ref() != Some(&expected_event_hash)
        {
            return Ok(AuditChainVerification {
                valid: false,
                checked_events,
                legacy_events,
                failure: Some(AuditChainFailure {
                    audit_event_id: row.audit_event_id,
                    expected_previous_event_hash,
                    actual_previous_event_hash: row.previous_event_hash,
                    expected_event_hash,
                    actual_event_hash,
                }),
            });
        }

        checked_events += 1;
        previous_event_hash = row.event_hash;
    }

    Ok(AuditChainVerification {
        valid: true,
        checked_events,
        legacy_events,
        failure: None,
    })
}

fn ensure_audit_chain_readable(verification: AuditChainVerification) -> StoreResult<()> {
    if verification.valid {
        return Ok(());
    }

    let failed_event = verification
        .failure
        .map(|failure| failure.audit_event_id.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    Err(StoreError::Integrity(format!(
        "audit chain verification failed at event {failed_event}"
    )))
}

#[derive(Serialize)]
struct AuditChainEntry<'a> {
    previous_event_hash: Option<&'a str>,
    event_json: &'a str,
}

fn ensure_audit_chain_columns(connection: &Connection) -> anyhow::Result<()> {
    if !sqlite_column_exists(connection, "audit_events", "previous_event_hash")? {
        connection
            .execute(
                "ALTER TABLE audit_events ADD COLUMN previous_event_hash TEXT",
                [],
            )
            .context("add audit previous event hash column")?;
    }
    if !sqlite_column_exists(connection, "audit_events", "event_hash")? {
        connection
            .execute("ALTER TABLE audit_events ADD COLUMN event_hash TEXT", [])
            .context("add audit event hash column")?;
    }
    Ok(())
}

fn sqlite_column_exists(
    connection: &Connection,
    table: &str,
    column: &str,
) -> anyhow::Result<bool> {
    let escaped_table = table.replace('"', "\"\"");
    let mut statement = connection
        .prepare(&format!(r#"PRAGMA table_info("{escaped_table}")"#))
        .with_context(|| format!("inspect sqlite table {table} columns"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .with_context(|| format!("query sqlite table {table} columns"))?;
    for row in rows {
        if row.with_context(|| format!("read sqlite table {table} column"))? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn latest_audit_event_hash(
    connection: &Connection,
    tenant_id: &TenantId,
    project_id: &ProjectId,
) -> anyhow::Result<Option<Sha256Hash>> {
    connection
        .query_row(
            r#"
            SELECT event_hash
            FROM audit_events
            WHERE tenant_id = ?1 AND project_id = ?2 AND event_hash IS NOT NULL
            ORDER BY created_at DESC, audit_event_id DESC
            LIMIT 1
            "#,
            params![tenant_id.as_str(), project_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .context("load previous audit event hash")?
        .map(Sha256Hash::new)
        .transpose()
        .context("decode previous audit event hash")
}

fn audit_event_chain_hash(
    event_json: &str,
    previous_event_hash: Option<&Sha256Hash>,
) -> anyhow::Result<Sha256Hash> {
    sha256_json_hash(&AuditChainEntry {
        previous_event_hash: previous_event_hash.map(Sha256Hash::as_str),
        event_json,
    })
    .context("hash audit event chain entry")
}

fn sql_decode_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
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

pub fn connector_tool_invoke_event(input: ConnectorToolInvokeAuditInput) -> AuditEventInsert {
    AuditEventInsert {
        tenant_id: input.tenant_id,
        project_id: input.project_id,
        environment_id: input.environment_id,
        actor_api_key_id: input.actor_api_key_id,
        action: AuditAction::ConnectorToolInvoke,
        resource_type: "connector_tool".to_string(),
        resource_id: input.tool_slug,
        outcome: input.outcome,
        reason: input.reason,
        attributes: input.attributes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn audit_action_names_cover_admin_lifecycle_events() {
        assert_eq!(AuditAction::PiiUnmask.as_str(), "pii_unmask");
        assert_eq!(AuditAction::ApiKeyCreate.as_str(), "api_key_create");
        assert_eq!(AuditAction::ApiKeyRevoke.as_str(), "api_key_revoke");
        assert_eq!(
            AuditAction::ProviderSecretCreate.as_str(),
            "provider_secret_create"
        );
        assert_eq!(
            AuditAction::ProviderSecretRevoke.as_str(),
            "provider_secret_revoke"
        );
    }

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

    #[tokio::test]
    async fn sqlite_audit_store_hash_chains_appended_events() -> anyhow::Result<()> {
        let store = SqliteAuditStore::in_memory()?;
        let tenant_id = TenantId::new("tenant-chain")?;
        let project_id = ProjectId::new("project-chain")?;

        store
            .append_event(sample_pii_unmask_event(
                &tenant_id,
                &project_id,
                "trace-one",
            )?)
            .await?;
        store
            .append_event(sample_pii_unmask_event(
                &tenant_id,
                &project_id,
                "trace-two",
            )?)
            .await?;

        let verification = store.verify_chain(tenant_id.clone(), project_id.clone())?;
        assert_eq!(
            verification,
            AuditChainVerification {
                valid: true,
                checked_events: 2,
                legacy_events: 0,
                failure: None,
            }
        );

        let hash_rows = stored_hash_rows(&store, &tenant_id, &project_id)?;
        assert_eq!(hash_rows.len(), 2);
        assert!(hash_rows[0].previous_event_hash.is_none());
        assert_eq!(
            hash_rows[1].previous_event_hash.as_deref(),
            hash_rows[0].event_hash.as_deref()
        );
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_audit_store_detects_tampered_event_json() -> anyhow::Result<()> {
        let store = SqliteAuditStore::in_memory()?;
        let tenant_id = TenantId::new("tenant-tamper")?;
        let project_id = ProjectId::new("project-tamper")?;
        let mut event = store
            .append_event(sample_pii_unmask_event(
                &tenant_id,
                &project_id,
                "trace-tamper",
            )?)
            .await?;
        event.reason = Some("changed after append".to_string());
        let tampered_event_json = serde_json::to_string(&event)?;

        {
            let connection = store.lock()?;
            connection.execute(
                r#"
                UPDATE audit_events
                SET event_json = ?1
                WHERE tenant_id = ?2 AND project_id = ?3 AND audit_event_id = ?4
                "#,
                rusqlite::params![
                    tampered_event_json,
                    tenant_id.as_str(),
                    project_id.as_str(),
                    event.audit_event_id.as_str(),
                ],
            )?;
        }

        let verification = store.verify_chain(tenant_id, project_id)?;
        assert!(!verification.valid);
        let Some(failure) = verification.failure else {
            panic!("expected a tamper failure to be reported");
        };
        assert_eq!(failure.audit_event_id, event.audit_event_id);
        assert_ne!(
            Some(&failure.expected_event_hash),
            failure.actual_event_hash.as_ref()
        );
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_audit_store_refuses_tampered_readback() -> anyhow::Result<()> {
        let store = SqliteAuditStore::in_memory()?;
        let tenant_id = TenantId::new("tenant-readback-tamper")?;
        let project_id = ProjectId::new("project-readback-tamper")?;
        let mut event = store
            .append_event(sample_pii_unmask_event(
                &tenant_id,
                &project_id,
                "trace-readback-tamper",
            )?)
            .await?;
        event.reason = Some("changed after append".to_string());
        let tampered_event_json = serde_json::to_string(&event)?;

        {
            let connection = store.lock()?;
            connection.execute(
                r#"
                UPDATE audit_events
                SET event_json = ?1
                WHERE tenant_id = ?2 AND project_id = ?3 AND audit_event_id = ?4
                "#,
                rusqlite::params![
                    tampered_event_json,
                    tenant_id.as_str(),
                    project_id.as_str(),
                    event.audit_event_id.as_str(),
                ],
            )?;
        }

        let result = store.list_events(tenant_id, project_id).await;
        let Err(StoreError::Integrity(message)) = result else {
            panic!("expected integrity error for tampered audit readback, got {result:?}");
        };
        assert!(
            message.contains(event.audit_event_id.as_str()),
            "integrity message should name the failed audit event: {message}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_audit_store_migrates_legacy_tables_before_hashing_new_events(
    ) -> anyhow::Result<()> {
        let dir = std::env::temp_dir().join(format!("beater-audit-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir)?;
        let path = dir.join("audit.sqlite");
        let tenant_id = TenantId::new("tenant-legacy")?;
        let project_id = ProjectId::new("project-legacy")?;
        {
            let connection = Connection::open(&path)?;
            create_legacy_audit_table(&connection)?;
            insert_legacy_event(&connection, &tenant_id, &project_id)?;
        }

        let store = SqliteAuditStore::open(&path)?;
        {
            let connection = store.lock()?;
            assert!(sqlite_column_exists(
                &connection,
                "audit_events",
                "previous_event_hash"
            )?);
            assert!(sqlite_column_exists(
                &connection,
                "audit_events",
                "event_hash"
            )?);
        }

        store
            .append_event(sample_pii_unmask_event(
                &tenant_id,
                &project_id,
                "trace-after-migration",
            )?)
            .await?;

        let verification = store.verify_chain(tenant_id, project_id)?;
        assert_eq!(
            verification,
            AuditChainVerification {
                valid: true,
                checked_events: 1,
                legacy_events: 1,
                failure: None,
            }
        );

        drop(store);
        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct StoredHashRow {
        previous_event_hash: Option<String>,
        event_hash: Option<String>,
    }

    fn sample_pii_unmask_event(
        tenant_id: &TenantId,
        project_id: &ProjectId,
        trace_id: &str,
    ) -> anyhow::Result<AuditEventInsert> {
        Ok(pii_unmask_event(PiiUnmaskAuditInput {
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            environment_id: Some(EnvironmentId::new("prod")?),
            actor_api_key_id: Some(ApiKeyId::new("key")?),
            trace_id: TraceId::new(trace_id)?,
            outcome: AuditOutcome::Allowed,
            reason: Some("debugging incident".to_string()),
            attributes: json!({"sensitive_refs": 2}),
        }))
    }

    fn stored_hash_rows(
        store: &SqliteAuditStore,
        tenant_id: &TenantId,
        project_id: &ProjectId,
    ) -> anyhow::Result<Vec<StoredHashRow>> {
        let connection = store.lock()?;
        let mut statement = connection.prepare(
            r#"
            SELECT previous_event_hash, event_hash
            FROM audit_events
            WHERE tenant_id = ?1 AND project_id = ?2
            ORDER BY created_at ASC, audit_event_id ASC
            "#,
        )?;
        let rows = statement.query_map(
            rusqlite::params![tenant_id.as_str(), project_id.as_str()],
            |row| {
                Ok(StoredHashRow {
                    previous_event_hash: row.get(0)?,
                    event_hash: row.get(1)?,
                })
            },
        )?;

        let mut hash_rows = Vec::new();
        for row in rows {
            hash_rows.push(row?);
        }
        Ok(hash_rows)
    }

    fn create_legacy_audit_table(connection: &Connection) -> anyhow::Result<()> {
        connection.execute_batch(
            r#"
            CREATE TABLE audit_events (
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
            "#,
        )?;
        Ok(())
    }

    fn insert_legacy_event(
        connection: &Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
    ) -> anyhow::Result<()> {
        let event = AuditEvent {
            audit_event_id: AuditEventId::new("legacy-event")?,
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            environment_id: None,
            actor_api_key_id: None,
            action: AuditAction::PiiUnmask,
            resource_type: "trace".to_string(),
            resource_id: "legacy-trace".to_string(),
            outcome: AuditOutcome::Allowed,
            reason: Some("legacy row".to_string()),
            attributes: json!({"legacy": true}),
            created_at: Utc::now() - chrono::Duration::seconds(60),
        };
        let event_json = serde_json::to_string(&event)?;
        connection.execute(
            r#"
            INSERT INTO audit_events
              (tenant_id, project_id, audit_event_id, environment_id, actor_api_key_id,
               action, resource_type, resource_id, outcome, created_at, event_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            rusqlite::params![
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
                event_json,
            ],
        )?;
        Ok(())
    }
}
