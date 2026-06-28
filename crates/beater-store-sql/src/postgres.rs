//! Postgres-backed [`TraceStore`].
//!
//! The store runs the checked-in `migrations/postgres/0001_initial.sql` contract
//! against a live server and persists raw envelopes + canonical spans into the
//! `raw_envelopes` / `spans` tables defined there. Each span's model, cost, and
//! release id are projected into dedicated columns at write time so `query_runs`
//! aggregates run summaries with a backend `GROUP BY (project_id, trace_id)`
//! rather than materializing every matching span in process (ARCHITECTURE.md
//! §8.1). The shared [`finalize_run_aggregates`] helper folds the aggregated
//! rows into the same `Page<RunSummary>` the SQLite reference store produces, so
//! the store still satisfies the shared store-conformance suite.
//!
//! The live integration test that boots a real Postgres container is `#[ignore]`d
//! so a Docker-less `cargo test` still passes; CI with Docker runs it explicitly.

use async_trait::async_trait;
use beater_core::{
    IdempotencyKey, Money, Page, PageRequest, ProjectId, TenantId, Timestamp, TraceId,
};
use beater_schema::{
    span_release_id, span_summary, AgentSpanKind, CanonicalSpan, CanonicalTraceBatch, ModelRef,
    RawEnvelope, RunFilter, RunSummary, SpanFilter, SpanStatus, SpanSummary, TraceView, WriteAck,
};
use beater_store::{
    finalize_run_aggregates, page_vec, RunAggregateRow, StoreError, StoreResult, TraceStore,
};
use std::collections::BTreeSet;
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
            // Project the roll-up dimensions into columns at write time (derived
            // from the canonical span exactly as `roll_up_runs` reads them) so
            // `query_runs` can GROUP BY without re-parsing `span_json`.
            let model_provider = span.model.as_ref().map(|model| model.provider.as_str());
            let model_name = span.model.as_ref().map(|model| model.name.as_str());
            let cost_currency = span.cost.as_ref().map(|cost| cost.currency.as_str());
            let cost_micros = span.cost.as_ref().map(|cost| cost.amount_micros);
            let release_id = span_release_id(span);
            let changed = self
                .client
                .execute(
                    r#"
                    INSERT INTO spans
                      (tenant_id, project_id, environment_id, trace_id, span_id, seq, kind, status,
                       name, start_time, end_time, model_provider, model_name, cost_currency,
                       cost_micros, release_id, span_json)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
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
                        &model_provider,
                        &model_name,
                        &cost_currency,
                        &cost_micros,
                        &release_id,
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
        // Backend GROUP BY over the span-scope columns (§8.1): the database
        // reduces each run to one row with scalar rollups plus the per-span
        // model/cost/release occurrences in `start_time DESC, seq ASC` order
        // (matching `roll_up_runs`). Run-level filters that depend on the rolled
        // up values (status/kind/cost/latency/model/release) and pagination are
        // applied by `finalize_run_aggregates`, exactly as the materializing
        // fallback applies them.
        let rows = self
            .client
            .query(
                r#"
                SELECT
                  project_id,
                  trace_id,
                  COUNT(*)::bigint AS span_count,
                  CASE
                    WHEN bool_or(status = 'error') THEN 'error'
                    WHEN bool_or(status = 'ok') THEN 'ok'
                    ELSE 'unset'
                  END AS status,
                  (array_agg(name ORDER BY start_time ASC, seq ASC))[1] AS first_span_name,
                  MIN(start_time) AS started_at,
                  MAX(end_time) AS ended_at,
                  COALESCE(
                    jsonb_agg(jsonb_build_array(model_provider, model_name)
                      ORDER BY start_time DESC, seq ASC)
                      FILTER (WHERE model_provider IS NOT NULL AND model_name IS NOT NULL),
                    '[]'::jsonb
                  ) AS models,
                  COALESCE(
                    jsonb_agg(jsonb_build_array(cost_currency, cost_micros)
                      ORDER BY start_time DESC, seq ASC)
                      FILTER (WHERE cost_currency IS NOT NULL AND cost_micros IS NOT NULL),
                    '[]'::jsonb
                  ) AS costs,
                  COALESCE(
                    jsonb_agg(to_jsonb(release_id) ORDER BY start_time DESC, seq ASC)
                      FILTER (WHERE release_id IS NOT NULL),
                    '[]'::jsonb
                  ) AS release_ids,
                  COALESCE(jsonb_agg(DISTINCT kind), '[]'::jsonb) AS kinds
                FROM spans
                WHERE tenant_id = $1
                  AND ($2::text IS NULL OR project_id = $2)
                  AND ($3::text IS NULL OR environment_id = $3)
                  AND ($4::text IS NULL OR trace_id = $4)
                GROUP BY project_id, trace_id
                ORDER BY MAX(start_time) DESC, project_id ASC, trace_id ASC
                "#,
                &[
                    &tenant.as_str(),
                    &filter.project_id.as_ref().map(|value| value.as_str()),
                    &filter.environment_id.as_ref().map(|value| value.as_str()),
                    &filter.trace_id.as_ref().map(|value| value.as_str()),
                ],
            )
            .await
            .map_err(StoreError::backend)?;

        let mut aggregates = Vec::with_capacity(rows.len());
        for row in rows {
            aggregates.push(run_aggregate_from_row(&row)?);
        }
        Ok(finalize_run_aggregates(tenant, aggregates, filter, page))
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

/// Builds a [`RunAggregateRow`] from one row of the `query_runs` GROUP BY.
fn run_aggregate_from_row(row: &tokio_postgres::Row) -> StoreResult<RunAggregateRow> {
    let project_id: String = row.get("project_id");
    let trace_id: String = row.get("trace_id");
    let span_count: i64 = row.get("span_count");
    let status: String = row.get("status");
    let first_span_name: String = row.get("first_span_name");
    let started_at: Timestamp = row.get("started_at");
    let ended_at: Option<Timestamp> = row.get("ended_at");
    let models: serde_json::Value = row.get("models");
    let costs: serde_json::Value = row.get("costs");
    let release_ids: serde_json::Value = row.get("release_ids");
    let kinds: serde_json::Value = row.get("kinds");

    Ok(RunAggregateRow {
        project_id: ProjectId::new(project_id).map_err(StoreError::integrity)?,
        trace_id: TraceId::new(trace_id).map_err(StoreError::integrity)?,
        span_count: usize::try_from(span_count).map_err(StoreError::integrity)?,
        status: SpanStatus::parse(&status)
            .ok_or_else(|| StoreError::integrity(format!("unknown span status {status}")))?,
        first_span_name,
        started_at,
        ended_at,
        models: parse_models(&models)?,
        costs: parse_costs(&costs)?,
        release_ids: parse_release_ids(&release_ids)?,
        kinds: parse_kinds(&kinds)?,
    })
}

/// Reads the JSON array a `COALESCE(jsonb_agg(...), '[]')` column always yields.
fn jsonb_array(value: &serde_json::Value) -> StoreResult<&Vec<serde_json::Value>> {
    value
        .as_array()
        .ok_or_else(|| StoreError::integrity("expected a JSON array aggregate column"))
}

/// Parses the `[ [provider, name], ... ]` models aggregate.
fn parse_models(value: &serde_json::Value) -> StoreResult<Vec<ModelRef>> {
    jsonb_array(value)?
        .iter()
        .map(|entry| {
            let (provider, name) = json_pair(entry)?;
            Ok(ModelRef {
                provider: provider.to_string(),
                name: name.to_string(),
            })
        })
        .collect()
}

/// Parses the `[ [currency, micros], ... ]` costs aggregate.
fn parse_costs(value: &serde_json::Value) -> StoreResult<Vec<Money>> {
    jsonb_array(value)?
        .iter()
        .map(|entry| {
            let currency = entry
                .get(0)
                .ok_or_else(|| StoreError::integrity("cost tuple missing currency"))?;
            let amount_micros = entry
                .get(1)
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| StoreError::integrity("cost tuple missing amount_micros"))?;
            let currency =
                serde_json::from_value(currency.clone()).map_err(StoreError::integrity)?;
            Ok(Money::new(amount_micros, currency))
        })
        .collect()
}

/// Parses the `[ release_id, ... ]` release-id aggregate.
fn parse_release_ids(value: &serde_json::Value) -> StoreResult<Vec<String>> {
    jsonb_array(value)?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| StoreError::integrity("release id is not a string"))
        })
        .collect()
}

/// Parses the distinct-`kind` aggregate.
fn parse_kinds(value: &serde_json::Value) -> StoreResult<BTreeSet<AgentSpanKind>> {
    jsonb_array(value)?
        .iter()
        .map(|entry| {
            let kind = entry
                .as_str()
                .ok_or_else(|| StoreError::integrity("span kind is not a string"))?;
            AgentSpanKind::parse(kind)
                .ok_or_else(|| StoreError::integrity(format!("unknown span kind {kind}")))
        })
        .collect()
}

/// Extracts a `[first, second]` string pair from a JSON aggregate tuple.
fn json_pair(entry: &serde_json::Value) -> StoreResult<(&str, &str)> {
    let first = entry
        .get(0)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| StoreError::integrity("aggregate tuple missing first element"))?;
    let second = entry
        .get(1)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| StoreError::integrity("aggregate tuple missing second element"))?;
    Ok((first, second))
}
