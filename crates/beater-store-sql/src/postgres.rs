//! Postgres-backed [`TraceStore`].
//!
//! The store runs the checked-in `migrations/postgres/0001_initial.sql` contract
//! against a live server and persists raw envelopes + canonical spans into the
//! `raw_envelopes` / `spans` tables defined there. Run summaries are derived by
//! materializing spans through the shared
//! [`query_runs_by_materializing_spans`] helper, keeping the backend behavior
//! identical to the SQLite reference store and satisfying the shared
//! store-conformance suite.
//!
//! The live integration test that boots a real Postgres container is `#[ignore]`d
//! so a Docker-less `cargo test` still passes; CI with Docker runs it explicitly.

use async_trait::async_trait;
use beater_core::{IdempotencyKey, Page, PageRequest, ProjectId, TenantId, TraceId};
use beater_schema::{
    span_summary, CanonicalSpan, CanonicalTraceBatch, RawEnvelope, RunFilter, RunSummary,
    SpanFilter, SpanSummary, TraceView, WriteAck,
};
use beater_store::{
    page_vec, query_runs_by_materializing_spans, StoreError, StoreResult, TraceStore,
};
use tokio_postgres::{Client, NoTls};

/// The Postgres schema contract this store applies and depends on.
pub const POSTGRES_TRACE_STORE_MIGRATION: &str =
    include_str!("../../../migrations/postgres/0001_initial.sql");

/// A [`TraceStore`] backed by a live Postgres server.
pub struct PgTraceStore {
    client: Client,
}

impl PgTraceStore {
    /// Connects to Postgres, runs the trace-store migration, and returns a store.
    ///
    /// The connection string is the standard libpq/`tokio-postgres` form, e.g.
    /// `host=localhost user=beater password=beater dbname=beater`.
    pub async fn connect(connection_string: &str) -> StoreResult<Self> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
            .await
            .map_err(StoreError::backend)?;
        // The connection drives the protocol; it must be polled for the client to
        // make progress, so spawn it and let it own the socket for the lifetime
        // of the program.
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let store = Self { client };
        store.migrate().await?;
        Ok(store)
    }

    /// Applies the checked-in Postgres migration contract. Idempotent: the
    /// migration is written with `CREATE TABLE IF NOT EXISTS`.
    pub async fn migrate(&self) -> StoreResult<()> {
        self.client
            .batch_execute(POSTGRES_TRACE_STORE_MIGRATION)
            .await
            .map_err(StoreError::backend)?;
        Ok(())
    }

    async fn get_trace_with_project(
        &self,
        tenant: TenantId,
        project: Option<ProjectId>,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        let rows = self
            .client
            .query(
                r#"
                SELECT span_json
                FROM spans
                WHERE tenant_id = $1
                  AND ($2::text IS NULL OR project_id = $2)
                  AND trace_id = $3
                ORDER BY seq ASC, start_time ASC
                "#,
                &[
                    &tenant.as_str(),
                    &project.as_ref().map(|project_id| project_id.as_str()),
                    &trace.as_str(),
                ],
            )
            .await
            .map_err(StoreError::backend)?;

        let mut spans = Vec::with_capacity(rows.len());
        for row in rows {
            let json: serde_json::Value = row.get(0);
            spans.push(serde_json::from_value::<CanonicalSpan>(json).map_err(StoreError::backend)?);
        }

        Ok(TraceView {
            tenant_id: tenant,
            trace_id: trace,
            spans,
        })
    }
}

#[async_trait]
impl TraceStore for PgTraceStore {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
        let mut accepted_raw = 0;
        let mut duplicate_raw = 0;
        for raw in &batch.raw_envelopes {
            let raw_json = serde_json::to_value(raw).map_err(StoreError::backend)?;
            let changed = self
                .client
                .execute(
                    r#"
                    INSERT INTO raw_envelopes
                      (tenant_id, project_id, idempotency_key, trace_id, payload_hash, received_at, raw_json)
                    VALUES ($1, $2, $3, NULL, $4, $5, $6)
                    ON CONFLICT (tenant_id, project_id, idempotency_key) DO NOTHING
                    "#,
                    &[
                        &raw.tenant_id.as_str(),
                        &raw.project_id.as_str(),
                        &raw.idempotency_key.as_str(),
                        &raw.payload_hash.as_str(),
                        &raw.received_at,
                        &raw_json,
                    ],
                )
                .await
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
            let span_json = serde_json::to_value(span).map_err(StoreError::backend)?;
            let changed = self
                .client
                .execute(
                    r#"
                    INSERT INTO spans
                      (tenant_id, project_id, environment_id, trace_id, span_id, seq, kind, status,
                       name, start_time, end_time, span_json)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    ON CONFLICT (tenant_id, project_id, trace_id, span_id, seq) DO NOTHING
                    "#,
                    &[
                        &span.tenant_id.as_str(),
                        &span.project_id.as_str(),
                        &span.environment_id.as_str(),
                        &span.trace_id.as_str(),
                        &span.span_id.as_str(),
                        &(span.seq as i64),
                        &span.kind.as_str(),
                        &span.status.as_str(),
                        &span.name,
                        &span.start_time,
                        &span.end_time,
                        &span_json,
                    ],
                )
                .await
                .map_err(StoreError::backend)?;
            if changed == 0 {
                duplicate_spans += 1;
            } else {
                accepted_spans += 1;
            }
        }

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
        let row = self
            .client
            .query_opt(
                r#"
                SELECT raw_json
                FROM raw_envelopes
                WHERE tenant_id = $1 AND project_id = $2 AND idempotency_key = $3
                "#,
                &[
                    &tenant.as_str(),
                    &project.as_str(),
                    &idempotency_key.as_str(),
                ],
            )
            .await
            .map_err(StoreError::backend)?;
        row.map(|row| {
            let json: serde_json::Value = row.get(0);
            serde_json::from_value::<RawEnvelope>(json).map_err(StoreError::backend)
        })
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
        let rows = self
            .client
            .query(
                r#"
                SELECT span_json
                FROM spans
                WHERE tenant_id = $1
                  AND ($2::text IS NULL OR project_id = $2)
                  AND ($3::text IS NULL OR environment_id = $3)
                  AND ($4::text IS NULL OR trace_id = $4)
                  AND ($5::text IS NULL OR span_id = $5)
                  AND ($6::text IS NULL OR kind = $6)
                  AND ($7::text IS NULL OR status = $7)
                ORDER BY start_time DESC, seq ASC
                "#,
                &[
                    &tenant.as_str(),
                    &filter.project_id.as_ref().map(|value| value.as_str()),
                    &filter.environment_id.as_ref().map(|value| value.as_str()),
                    &filter.trace_id.as_ref().map(|value| value.as_str()),
                    &filter.span_id.as_ref().map(|value| value.as_str()),
                    &filter.kind.as_ref().map(|value| value.as_str()),
                    &filter.status.as_ref().map(|value| value.as_str()),
                ],
            )
            .await
            .map_err(StoreError::backend)?;

        let mut spans = Vec::with_capacity(rows.len());
        for row in rows {
            let json: serde_json::Value = row.get(0);
            let span =
                serde_json::from_value::<CanonicalSpan>(json).map_err(StoreError::backend)?;
            spans.push(span_summary(span));
        }

        Ok(page_vec(spans, page))
    }
}
