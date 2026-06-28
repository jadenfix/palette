use async_trait::async_trait;
use beater_core::{
    sha256_hex, Clock, EnvironmentId, IdempotencyKey, OrganizationId, Page, PageRequest, ProjectId,
    SystemClock, TenantId, Timestamp, TraceId,
};
use beater_schema::{
    span_summary, CanonicalSpan, CanonicalTraceBatch, RawEnvelope, RunFilter, RunSummary,
    SpanFilter, SpanSummary, TraceView, WriteAck,
};
use beater_store::{
    lock_poisoned, query_runs_by_materializing_spans, EnvironmentMetadata, MetadataStore,
    OrganizationMetadata, ProjectMetadata, QuotaDecision, QuotaLimiter, QuotaReservationRequest,
    RoleBinding, StoreError, StoreResult, TraceStore,
};
use chrono::DateTime;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::{PgTraceStore, POSTGRES_TRACE_STORE_MIGRATION};

#[cfg(feature = "clickhouse")]
mod clickhouse;
#[cfg(feature = "clickhouse")]
pub use clickhouse::{ClickHouseTraceStore, CLICKHOUSE_TRACE_STORE_MIGRATION};

const LOCAL_BEATERD_SQLITE_0001: &str =
    include_str!("../../../migrations/sqlite/0001_local_beaterd.sql");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SqliteMigration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SqliteMigrationReport {
    pub applied: usize,
    pub skipped: usize,
}

pub const LOCAL_BEATERD_SQLITE_MIGRATIONS: &[SqliteMigration] = &[SqliteMigration {
    version: 1,
    name: "0001_local_beaterd",
    sql: LOCAL_BEATERD_SQLITE_0001,
}];

pub fn migrate_local_beaterd_sqlite(path: impl AsRef<Path>) -> StoreResult<SqliteMigrationReport> {
    apply_sqlite_migrations(path, LOCAL_BEATERD_SQLITE_MIGRATIONS)
}

pub fn apply_sqlite_migrations(
    path: impl AsRef<Path>,
    migrations: &[SqliteMigration],
) -> StoreResult<SqliteMigrationReport> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(StoreError::backend)?;
    }
    let mut connection = Connection::open(path).map_err(StoreError::backend)?;
    configure_sqlite_connection(&connection)?;
    apply_sqlite_migrations_to_connection(&mut connection, migrations)
}

fn apply_sqlite_migrations_to_connection(
    connection: &mut Connection,
    migrations: &[SqliteMigration],
) -> StoreResult<SqliteMigrationReport> {
    connection
        .execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS _beater_schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(StoreError::backend)?;

    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StoreError::backend)?;
    let mut report = SqliteMigrationReport::default();
    for migration in migrations {
        validate_migration(migration)?;
        let checksum = sha256_hex(migration.sql.as_bytes());
        let existing = tx
            .query_row(
                r#"
                SELECT checksum
                FROM _beater_schema_migrations
                WHERE version = ?1
                "#,
                params![migration.version],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(StoreError::backend)?;
        if let Some(existing) = existing {
            if existing != checksum {
                return Err(StoreError::integrity(format!(
                    "sqlite migration {} checksum mismatch",
                    migration.version
                )));
            }
            report.skipped += 1;
            continue;
        }

        tx.execute_batch(migration.sql)
            .map_err(StoreError::backend)?;
        tx.execute(
            r#"
            INSERT INTO _beater_schema_migrations
              (version, name, checksum, applied_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                migration.version,
                migration.name,
                checksum,
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .map_err(StoreError::backend)?;
        report.applied += 1;
    }
    tx.commit().map_err(StoreError::backend)?;
    Ok(report)
}

fn validate_migration(migration: &SqliteMigration) -> StoreResult<()> {
    if migration.version == 0 {
        return Err(StoreError::integrity(
            "sqlite migration version must be positive",
        ));
    }
    if migration.name.trim().is_empty() {
        return Err(StoreError::integrity("sqlite migration name is empty"));
    }
    if migration.sql.trim().is_empty() {
        return Err(StoreError::integrity("sqlite migration sql is empty"));
    }
    Ok(())
}

#[derive(Clone)]
pub struct SqliteTraceStore {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Clone)]
pub struct SqliteMetadataStore {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Clone)]
pub struct SqliteQuotaLimiter {
    connection: Arc<Mutex<Connection>>,
    clock: Arc<dyn Clock>,
}

impl SqliteQuotaLimiter {
    pub fn in_memory() -> StoreResult<Self> {
        let connection = Connection::open_in_memory().map_err(StoreError::backend)?;
        configure_sqlite_connection(&connection)?;
        let limiter = Self {
            connection: Arc::new(Mutex::new(connection)),
            clock: Arc::new(SystemClock),
        };
        limiter.init()?;
        Ok(limiter)
    }

    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(StoreError::backend)?;
        }
        let connection = Connection::open(path).map_err(StoreError::backend)?;
        configure_sqlite_connection(&connection)?;
        let limiter = Self {
            connection: Arc::new(Mutex::new(connection)),
            clock: Arc::new(SystemClock),
        };
        limiter.init()?;
        Ok(limiter)
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    fn init(&self) -> StoreResult<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS quota_counters (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    window_start TEXT NOT NULL,
                    reset_at TEXT NOT NULL,
                    used_events INTEGER NOT NULL,
                    updated_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, window_start)
                );
                "#,
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, Connection>> {
        lock_poisoned(&self.connection, "quota sqlite")
    }
}

#[async_trait]
impl QuotaLimiter for SqliteQuotaLimiter {
    async fn reserve_quota(&self, request: QuotaReservationRequest) -> StoreResult<QuotaDecision> {
        let mut connection = self.lock()?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::backend)?;
        let current_used = tx
            .query_row(
                r#"
                SELECT used_events
                FROM quota_counters
                WHERE tenant_id = ?1 AND project_id = ?2 AND window_start = ?3
                "#,
                params![
                    request.tenant_id.as_str(),
                    request.project_id.as_str(),
                    request.window_start.to_rfc3339(),
                ],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(StoreError::backend)?
            .unwrap_or(0);
        if current_used < 0 {
            return Err(StoreError::integrity("negative quota counter"));
        }
        let current_used = current_used as u64;
        let Some(new_used) = current_used.checked_add(request.amount) else {
            return Err(StoreError::integrity("quota counter overflow"));
        };
        if new_used > request.limit {
            tx.commit().map_err(StoreError::backend)?;
            return Ok(QuotaDecision {
                accepted: false,
                used: current_used,
                limit: request.limit,
                reset_at: request.reset_at,
            });
        }

        tx.execute(
            r#"
            INSERT INTO quota_counters
              (tenant_id, project_id, window_start, reset_at, used_events, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(tenant_id, project_id, window_start) DO UPDATE SET
              reset_at = excluded.reset_at,
              used_events = excluded.used_events,
              updated_at = excluded.updated_at
            "#,
            params![
                request.tenant_id.as_str(),
                request.project_id.as_str(),
                request.window_start.to_rfc3339(),
                request.reset_at.to_rfc3339(),
                new_used as i64,
                self.clock.now().to_rfc3339(),
            ],
        )
        .map_err(StoreError::backend)?;
        tx.commit().map_err(StoreError::backend)?;

        Ok(QuotaDecision {
            accepted: true,
            used: new_used,
            limit: request.limit,
            reset_at: request.reset_at,
        })
    }
}

fn configure_sqlite_connection(connection: &Connection) -> StoreResult<()> {
    connection
        .busy_timeout(StdDuration::from_secs(5))
        .map_err(StoreError::backend)
}

impl SqliteMetadataStore {
    pub fn in_memory() -> StoreResult<Self> {
        let connection = Connection::open_in_memory().map_err(StoreError::backend)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(StoreError::backend)?;
        }
        let connection = Connection::open(path).map_err(StoreError::backend)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> StoreResult<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS organizations (
                    tenant_id TEXT NOT NULL,
                    organization_id TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, organization_id)
                );

                CREATE TABLE IF NOT EXISTS projects (
                    tenant_id TEXT NOT NULL,
                    organization_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id)
                );

                CREATE TABLE IF NOT EXISTS environments (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    environment_id TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, environment_id)
                );

                CREATE TABLE IF NOT EXISTS role_bindings (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT,
                    principal_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    permissions_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, principal_id, role)
                );
                "#,
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, Connection>> {
        lock_poisoned(&self.connection, "metadata sqlite")
    }
}

#[async_trait]
impl MetadataStore for SqliteMetadataStore {
    async fn put_organization(&self, organization: OrganizationMetadata) -> StoreResult<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO organizations (tenant_id, organization_id, display_name, created_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(tenant_id, organization_id) DO UPDATE SET
                  display_name = excluded.display_name,
                  created_at = excluded.created_at
                "#,
                params![
                    organization.tenant_id.as_str(),
                    organization.organization_id.as_str(),
                    organization.display_name,
                    organization.created_at.to_rfc3339(),
                ],
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    async fn get_organization(
        &self,
        tenant_id: TenantId,
        organization_id: OrganizationId,
    ) -> StoreResult<Option<OrganizationMetadata>> {
        let connection = self.lock()?;
        connection
            .query_row(
                r#"
                SELECT tenant_id, organization_id, display_name, created_at
                FROM organizations
                WHERE tenant_id = ?1 AND organization_id = ?2
                "#,
                params![tenant_id.as_str(), organization_id.as_str()],
                decode_organization,
            )
            .optional()
            .map_err(StoreError::backend)
    }

    async fn put_project(&self, project: ProjectMetadata) -> StoreResult<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO projects
                  (tenant_id, organization_id, project_id, display_name, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(tenant_id, project_id) DO UPDATE SET
                  organization_id = excluded.organization_id,
                  display_name = excluded.display_name,
                  created_at = excluded.created_at
                "#,
                params![
                    project.tenant_id.as_str(),
                    project.organization_id.as_str(),
                    project.project_id.as_str(),
                    project.display_name,
                    project.created_at.to_rfc3339(),
                ],
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    async fn get_project(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Option<ProjectMetadata>> {
        let connection = self.lock()?;
        connection
            .query_row(
                r#"
                SELECT tenant_id, organization_id, project_id, display_name, created_at
                FROM projects
                WHERE tenant_id = ?1 AND project_id = ?2
                "#,
                params![tenant_id.as_str(), project_id.as_str()],
                decode_project,
            )
            .optional()
            .map_err(StoreError::backend)
    }

    async fn put_environment(&self, environment: EnvironmentMetadata) -> StoreResult<()> {
        let connection = self.lock()?;
        connection
            .execute(
                r#"
                INSERT INTO environments
                  (tenant_id, project_id, environment_id, display_name, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(tenant_id, project_id, environment_id) DO UPDATE SET
                  display_name = excluded.display_name,
                  created_at = excluded.created_at
                "#,
                params![
                    environment.tenant_id.as_str(),
                    environment.project_id.as_str(),
                    environment.environment_id.as_str(),
                    environment.display_name,
                    environment.created_at.to_rfc3339(),
                ],
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    async fn get_environment(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        environment_id: EnvironmentId,
    ) -> StoreResult<Option<EnvironmentMetadata>> {
        let connection = self.lock()?;
        connection
            .query_row(
                r#"
                SELECT tenant_id, project_id, environment_id, display_name, created_at
                FROM environments
                WHERE tenant_id = ?1 AND project_id = ?2 AND environment_id = ?3
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    environment_id.as_str()
                ],
                decode_environment,
            )
            .optional()
            .map_err(StoreError::backend)
    }

    async fn put_role_binding(&self, binding: RoleBinding) -> StoreResult<()> {
        let connection = self.lock()?;
        let permissions_json =
            serde_json::to_string(&binding.permissions).map_err(StoreError::backend)?;
        connection
            .execute(
                r#"
                INSERT INTO role_bindings
                  (tenant_id, project_id, principal_id, role, permissions_json, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(tenant_id, project_id, principal_id, role) DO UPDATE SET
                  permissions_json = excluded.permissions_json,
                  created_at = excluded.created_at
                "#,
                params![
                    binding.tenant_id.as_str(),
                    binding.project_id.as_ref().map(|project| project.as_str()),
                    binding.principal_id,
                    binding.role,
                    permissions_json,
                    binding.created_at.to_rfc3339(),
                ],
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    async fn list_role_bindings(
        &self,
        tenant_id: TenantId,
        project_id: Option<ProjectId>,
        principal_id: String,
    ) -> StoreResult<Vec<RoleBinding>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT tenant_id, project_id, principal_id, role, permissions_json, created_at
                FROM role_bindings
                WHERE tenant_id = ?1
                  AND ((?2 IS NULL AND project_id IS NULL) OR project_id = ?2)
                  AND principal_id = ?3
                ORDER BY role ASC
                "#,
            )
            .map_err(StoreError::backend)?;
        let rows = statement
            .query_map(
                params![
                    tenant_id.as_str(),
                    project_id.as_ref().map(|project| project.as_str()),
                    principal_id,
                ],
                decode_role_binding,
            )
            .map_err(StoreError::backend)?;
        let mut bindings = Vec::new();
        for row in rows {
            bindings.push(row.map_err(StoreError::backend)?);
        }
        Ok(bindings)
    }
}

impl SqliteTraceStore {
    pub fn in_memory() -> StoreResult<Self> {
        let connection = Connection::open_in_memory().map_err(StoreError::backend)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> StoreResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(StoreError::backend)?;
        }
        let connection = Connection::open(path).map_err(StoreError::backend)?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> StoreResult<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS raw_envelopes (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    idempotency_key TEXT NOT NULL,
                    trace_id TEXT,
                    payload_hash TEXT NOT NULL,
                    received_at TEXT NOT NULL,
                    raw_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, idempotency_key)
                );

                CREATE TABLE IF NOT EXISTS spans (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    environment_id TEXT NOT NULL,
                    trace_id TEXT NOT NULL,
                    span_id TEXT NOT NULL,
                    seq INTEGER NOT NULL,
                    kind TEXT NOT NULL,
                    status TEXT NOT NULL,
                    name TEXT NOT NULL,
                    start_time TEXT NOT NULL,
                    end_time TEXT,
                    span_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, trace_id, span_id, seq)
                );

                CREATE INDEX IF NOT EXISTS idx_spans_tenant_trace
                ON spans (tenant_id, trace_id, seq);

                CREATE INDEX IF NOT EXISTS idx_spans_tenant_kind_status
                ON spans (tenant_id, kind, status, start_time);
                "#,
            )
            .map_err(StoreError::backend)?;
        Ok(())
    }

    async fn get_trace_with_project(
        &self,
        tenant: TenantId,
        project: Option<ProjectId>,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT span_json
                FROM spans
                WHERE tenant_id = ?1
                  AND (?2 IS NULL OR project_id = ?2)
                  AND trace_id = ?3
                ORDER BY seq ASC, start_time ASC
                "#,
            )
            .map_err(StoreError::backend)?;
        let rows = statement
            .query_map(
                params![
                    tenant.as_str(),
                    project.as_ref().map(|project_id| project_id.as_str()),
                    trace.as_str()
                ],
                |row| row.get::<_, String>(0),
            )
            .map_err(StoreError::backend)?;

        let mut spans = Vec::new();
        for row in rows {
            let json = row.map_err(StoreError::backend)?;
            spans.push(serde_json::from_str::<CanonicalSpan>(&json).map_err(StoreError::backend)?);
        }

        Ok(TraceView {
            tenant_id: tenant,
            trace_id: trace,
            spans,
        })
    }

    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, Connection>> {
        lock_poisoned(&self.connection, "sqlite connection")
    }
}

#[async_trait]
impl TraceStore for SqliteTraceStore {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(StoreError::backend)?;

        let mut accepted_raw = 0;
        let mut duplicate_raw = 0;
        for raw in &batch.raw_envelopes {
            let raw_json = serde_json::to_string(raw).map_err(StoreError::backend)?;
            let changed = tx
                .execute(
                    r#"
                    INSERT OR IGNORE INTO raw_envelopes
                      (tenant_id, project_id, idempotency_key, trace_id, payload_hash, received_at, raw_json)
                    VALUES
                      (?1, ?2, ?3, NULL, ?4, ?5, ?6)
                    "#,
                    params![
                        raw.tenant_id.as_str(),
                        raw.project_id.as_str(),
                        raw.idempotency_key.as_str(),
                        raw.payload_hash.as_str(),
                        raw.received_at.to_rfc3339(),
                        raw_json
                    ],
                )
                .map_err(StoreError::backend)?;
            if changed == 0 {
                duplicate_raw += 1;
            } else {
                accepted_raw += 1;
            }
        }

        let mut accepted_spans = 0;
        let mut duplicate_spans = 0;
        for span in &batch.spans {
            let span_json = serde_json::to_string(span).map_err(StoreError::backend)?;
            let changed = tx
                .execute(
                    r#"
                    INSERT OR IGNORE INTO spans
                      (tenant_id, project_id, environment_id, trace_id, span_id, seq, kind, status,
                       name, start_time, end_time, span_json)
                    VALUES
                      (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    "#,
                    params![
                        span.tenant_id.as_str(),
                        span.project_id.as_str(),
                        span.environment_id.as_str(),
                        span.trace_id.as_str(),
                        span.span_id.as_str(),
                        span.seq as i64,
                        span.kind.as_str(),
                        span.status.as_str(),
                        span.name,
                        span.start_time.to_rfc3339(),
                        span.end_time.map(|time| time.to_rfc3339()),
                        span_json
                    ],
                )
                .map_err(StoreError::backend)?;
            if changed == 0 {
                duplicate_spans += 1;
            } else {
                accepted_spans += 1;
            }
        }

        tx.commit().map_err(StoreError::backend)?;
        Ok(WriteAck {
            accepted_raw,
            accepted_spans,
            duplicate_raw,
            duplicate_spans,
        })
    }

    async fn get_trace(&self, tenant: TenantId, trace: TraceId) -> StoreResult<TraceView> {
        self.get_trace_with_project(tenant, None, trace).await
    }

    async fn get_project_trace(
        &self,
        tenant: TenantId,
        project: ProjectId,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        self.get_trace_with_project(tenant, Some(project), trace)
            .await
    }

    async fn get_raw_envelope(
        &self,
        tenant: TenantId,
        project: ProjectId,
        idempotency_key: IdempotencyKey,
    ) -> StoreResult<Option<RawEnvelope>> {
        let connection = self.lock()?;
        let raw_json = connection
            .query_row(
                r#"
                SELECT raw_json
                FROM raw_envelopes
                WHERE tenant_id = ?1 AND project_id = ?2 AND idempotency_key = ?3
                "#,
                params![tenant.as_str(), project.as_str(), idempotency_key.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(StoreError::backend)?;
        raw_json
            .map(|json| serde_json::from_str::<RawEnvelope>(&json).map_err(StoreError::backend))
            .transpose()
    }
    async fn query_runs(
        &self,
        tenant: TenantId,
        filter: RunFilter,
        page: PageRequest,
    ) -> StoreResult<Page<RunSummary>> {
        query_runs_by_materializing_spans(self, tenant, filter, page).await
    }

    async fn query_spans(
        &self,
        tenant: TenantId,
        filter: SpanFilter,
        page: PageRequest,
    ) -> StoreResult<Page<SpanSummary>> {
        let limit = page.limit.max(1) as usize;
        let offset = page
            .cursor
            .as_deref()
            .and_then(|cursor| cursor.parse::<usize>().ok())
            .unwrap_or(0);
        let fetch_limit = limit.saturating_add(1);
        let fetch_limit_i64 = i64::try_from(fetch_limit).unwrap_or(i64::MAX);
        let offset_i64 = i64::try_from(offset).unwrap_or(i64::MAX);
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT span_json
                FROM spans
                WHERE tenant_id = ?1
                  AND (?2 IS NULL OR project_id = ?2)
                  AND (?3 IS NULL OR environment_id = ?3)
                  AND (?4 IS NULL OR trace_id = ?4)
                  AND (?5 IS NULL OR span_id = ?5)
                  AND (?6 IS NULL OR kind = ?6)
                  AND (?7 IS NULL OR status = ?7)
                ORDER BY start_time DESC, seq ASC
                LIMIT ?8 OFFSET ?9
                "#,
            )
            .map_err(StoreError::backend)?;
        let rows = statement
            .query_map(
                params![
                    tenant.as_str(),
                    filter
                        .project_id
                        .as_ref()
                        .map(|project_id| project_id.as_str()),
                    filter
                        .environment_id
                        .as_ref()
                        .map(|environment_id| environment_id.as_str()),
                    filter.trace_id.as_ref().map(|trace_id| trace_id.as_str()),
                    filter.span_id.as_ref().map(|span_id| span_id.as_str()),
                    filter.kind.as_ref().map(|kind| kind.as_str()),
                    filter.status.as_ref().map(|status| status.as_str()),
                    fetch_limit_i64,
                    offset_i64,
                ],
                |row| row.get::<_, String>(0),
            )
            .map_err(StoreError::backend)?;

        let mut spans = Vec::new();
        let mut has_more = false;
        for row in rows {
            let json = row.map_err(StoreError::backend)?;
            if spans.len() == limit {
                has_more = true;
                break;
            }
            let span = serde_json::from_str::<CanonicalSpan>(&json).map_err(StoreError::backend)?;
            spans.push(span_summary(span));
        }

        let next_cursor = if has_more {
            Some(offset.saturating_add(limit).to_string())
        } else {
            None
        };

        Ok(Page::new(spans, next_cursor))
    }
}

fn decode_organization(row: &rusqlite::Row<'_>) -> rusqlite::Result<OrganizationMetadata> {
    Ok(OrganizationMetadata {
        tenant_id: id_from_row(row, 0)?,
        organization_id: id_from_row(row, 1)?,
        display_name: row.get(2)?,
        created_at: parse_timestamp(row.get(3)?)?,
    })
}

fn decode_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectMetadata> {
    Ok(ProjectMetadata {
        tenant_id: id_from_row(row, 0)?,
        organization_id: id_from_row(row, 1)?,
        project_id: id_from_row(row, 2)?,
        display_name: row.get(3)?,
        created_at: parse_timestamp(row.get(4)?)?,
    })
}

fn decode_environment(row: &rusqlite::Row<'_>) -> rusqlite::Result<EnvironmentMetadata> {
    Ok(EnvironmentMetadata {
        tenant_id: id_from_row(row, 0)?,
        project_id: id_from_row(row, 1)?,
        environment_id: id_from_row(row, 2)?,
        display_name: row.get(3)?,
        created_at: parse_timestamp(row.get(4)?)?,
    })
}

fn decode_role_binding(row: &rusqlite::Row<'_>) -> rusqlite::Result<RoleBinding> {
    let project_id = row
        .get::<_, Option<String>>(1)?
        .map(|value| parse_id::<ProjectId>(value, 1))
        .transpose()?;
    let permissions_json: String = row.get(4)?;
    let permissions = serde_json::from_str::<BTreeSet<String>>(&permissions_json)
        .map_err(|err| conversion_error(4, err))?;
    Ok(RoleBinding {
        tenant_id: id_from_row(row, 0)?,
        project_id,
        principal_id: row.get(2)?,
        role: row.get(3)?,
        permissions,
        created_at: parse_timestamp(row.get(5)?)?,
    })
}

fn id_from_row<T>(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<T>
where
    T: TryFrom<String>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    parse_id(row.get(index)?, index)
}

fn parse_id<T>(value: String, index: usize) -> rusqlite::Result<T>
where
    T: TryFrom<String>,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    value.try_into().map_err(|err| conversion_error(index, err))
}

fn parse_timestamp(value: String) -> rusqlite::Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&chrono::Utc))
        .map_err(|err| conversion_error(0, err))
}

fn conversion_error(
    index: usize,
    err: impl std::error::Error + Send + Sync + 'static,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::FixedClock;
    use beater_store_conformance::{
        assert_metadata_store_conformance, assert_quota_limiter_conformance,
        assert_trace_store_conformance,
    };
    use beater_store_memory::{InMemoryMetadataStore, InMemoryQuotaLimiter, InMemoryTraceStore};
    use chrono::{TimeZone, Utc};

    #[test]
    fn local_sqlite_migration_executes_and_is_idempotent() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("beater.sqlite");

        let first = migrate_local_beaterd_sqlite(&path).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(first.applied, 1);
        assert_eq!(first.skipped, 0);

        let connection = Connection::open(&path).unwrap_or_else(|err| panic!("{err}"));
        assert!(sqlite_object_exists(&connection, "table", "raw_envelopes"));
        assert!(sqlite_object_exists(
            &connection,
            "table",
            "judge_audit_records"
        ));
        assert!(sqlite_object_exists(
            &connection,
            "table",
            "calibration_reports"
        ));
        assert!(sqlite_object_exists(
            &connection,
            "table",
            "_beater_schema_migrations"
        ));
        assert!(sqlite_column_exists(
            &connection,
            "judge_audit_records",
            "evaluator_id"
        ));
        assert!(sqlite_column_exists(
            &connection,
            "calibration_reports",
            "evaluator_version_id"
        ));

        let second = migrate_local_beaterd_sqlite(&path).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(second.applied, 0);
        assert_eq!(second.skipped, 1);
    }

    #[test]
    fn sqlite_migration_checksum_drift_is_rejected() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("beater.sqlite");
        let first = SqliteMigration {
            version: 42,
            name: "test",
            sql: "CREATE TABLE checksum_test (id TEXT PRIMARY KEY);",
        };
        let changed = SqliteMigration {
            version: 42,
            name: "test",
            sql: "CREATE TABLE checksum_test_changed (id TEXT PRIMARY KEY);",
        };

        let report = apply_sqlite_migrations(&path, &[first]).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(report.applied, 1);

        let error = apply_sqlite_migrations(&path, &[changed])
            .err()
            .unwrap_or_else(|| panic!("expected checksum drift rejection"));
        assert!(error.to_string().contains("checksum mismatch"));
    }

    #[tokio::test]
    async fn sqlite_trace_store_conforms() {
        assert_trace_store_conformance(
            SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")),
        )
        .await;
    }

    #[tokio::test]
    async fn in_memory_trace_store_conforms() {
        assert_trace_store_conformance(InMemoryTraceStore::new()).await;
    }

    #[tokio::test]
    async fn sqlite_metadata_store_conforms() {
        assert_metadata_store_conformance(
            SqliteMetadataStore::in_memory().unwrap_or_else(|err| panic!("{err}")),
        )
        .await;
    }

    #[tokio::test]
    async fn in_memory_metadata_store_conforms() {
        assert_metadata_store_conformance(InMemoryMetadataStore::new()).await;
    }

    #[tokio::test]
    async fn sqlite_quota_limiter_conforms() {
        assert_quota_limiter_conformance(
            SqliteQuotaLimiter::in_memory().unwrap_or_else(|err| panic!("{err}")),
        )
        .await;
    }

    #[tokio::test]
    async fn sqlite_quota_limiter_uses_injected_clock_for_updates() {
        let now = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 42)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp"));
        let limiter = SqliteQuotaLimiter::in_memory()
            .unwrap_or_else(|err| panic!("{err}"))
            .with_clock(Arc::new(FixedClock::new(now)));
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

        let decision = limiter
            .reserve_quota(QuotaReservationRequest {
                tenant_id: tenant,
                project_id: project,
                amount: 1,
                limit: 2,
                window_start,
                reset_at,
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(decision.accepted);

        let connection = limiter.lock().unwrap_or_else(|err| panic!("{err}"));
        let updated_at = connection
            .query_row("SELECT updated_at FROM quota_counters", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(updated_at, now.to_rfc3339());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn sqlite_quota_limiter_serializes_independent_connections() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("quotas.sqlite");
        let first = SqliteQuotaLimiter::open(&path).unwrap_or_else(|err| panic!("{err}"));
        let second = SqliteQuotaLimiter::open(&path).unwrap_or_else(|err| panic!("{err}"));
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
        let request = QuotaReservationRequest {
            tenant_id: tenant,
            project_id: project,
            amount: 1,
            limit: 1,
            window_start,
            reset_at,
        };
        let first_request = request.clone();
        let second_request = request;

        let first_task = tokio::spawn(async move { first.reserve_quota(first_request).await });
        let second_task = tokio::spawn(async move { second.reserve_quota(second_request).await });
        let (first_result, second_result) = tokio::join!(first_task, second_task);
        let decisions = [
            first_result
                .unwrap_or_else(|err| panic!("{err}"))
                .unwrap_or_else(|err| panic!("{err}")),
            second_result
                .unwrap_or_else(|err| panic!("{err}"))
                .unwrap_or_else(|err| panic!("{err}")),
        ];

        assert_eq!(
            decisions
                .iter()
                .filter(|decision| decision.accepted)
                .count(),
            1
        );
        assert!(decisions
            .iter()
            .any(|decision| !decision.accepted && decision.used == 1));
    }

    #[tokio::test]
    async fn in_memory_quota_limiter_conforms() {
        assert_quota_limiter_conformance(InMemoryQuotaLimiter::new()).await;
    }

    fn sqlite_object_exists(connection: &Connection, object_type: &str, name: &str) -> bool {
        connection
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM sqlite_master
                WHERE type = ?1 AND name = ?2
                "#,
                params![object_type, name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or_else(|err| panic!("{err}"))
            > 0
    }

    fn sqlite_column_exists(connection: &Connection, table: &str, column: &str) -> bool {
        let escaped_table = table.replace('"', "\"\"");
        let mut statement = connection
            .prepare(&format!(r#"PRAGMA table_info("{escaped_table}")"#))
            .unwrap_or_else(|err| panic!("{err}"));
        let mut rows = statement.query([]).unwrap_or_else(|err| panic!("{err}"));
        while let Some(row) = rows.next().unwrap_or_else(|err| panic!("{err}")) {
            let name: String = row.get(1).unwrap_or_else(|err| panic!("{err}"));
            if name == column {
                return true;
            }
        }
        false
    }
}
