//! ClickHouse-backed [`TraceStore`].
//!
//! The store drives ClickHouse over its HTTP interface (no native driver) and
//! runs the checked-in `migrations/clickhouse/0001_trace_store.sql` contract.
//! Raw envelopes and canonical spans are persisted into the `beater.raw_envelopes`
//! / `beater.spans` tables defined there with the full canonical object kept in
//! the `raw_json` / `span_json` columns.
//!
//! ClickHouse `MergeTree` engines do not enforce primary-key uniqueness on
//! insert, so idempotency (the `WriteAck` duplicate counts the conformance suite
//! asserts) is enforced at the application layer: existing keys are looked up
//! before insert and only genuinely new rows are written.
//!
//! Run summaries are derived by materializing spans through the shared
//! [`query_runs_by_materializing_spans`] helper, keeping behavior identical to
//! the SQLite reference store. The live integration test that boots a real
//! ClickHouse container is `#[ignore]`d so a Docker-less `cargo test` still
//! passes; CI with Docker runs it explicitly.

use async_trait::async_trait;
use beater_core::{IdempotencyKey, Page, PageRequest, ProjectId, TenantId, TraceId};
use beater_schema::{
    span_summary, CanonicalSpan, CanonicalTraceBatch, RawEnvelope, RunFilter, RunSummary,
    SpanFilter, SpanSummary, TraceView, WriteAck,
};
use beater_store::{
    page_vec, query_runs_by_materializing_spans, StoreError, StoreResult, TraceStore,
};
use std::collections::BTreeSet;

/// The ClickHouse schema contract this store applies and depends on.
pub const CLICKHOUSE_TRACE_STORE_MIGRATION: &str =
    include_str!("../../../migrations/clickhouse/0001_trace_store.sql");

/// A [`TraceStore`] backed by a live ClickHouse server over the HTTP interface.
pub struct ClickHouseTraceStore {
    http: reqwest::Client,
    endpoint: String,
}

impl ClickHouseTraceStore {
    /// Connects to ClickHouse at `endpoint` (e.g. `http://localhost:8123`) and
    /// runs the trace-store migration. Idempotent: the migration uses
    /// `CREATE ... IF NOT EXISTS`.
    pub async fn connect(endpoint: &str) -> StoreResult<Self> {
        let store = Self {
            http: reqwest::Client::new(),
            endpoint: endpoint.trim_end_matches('/').to_string(),
        };
        store.migrate().await?;
        Ok(store)
    }

    /// Applies the checked-in ClickHouse migration contract statement by
    /// statement (the HTTP interface accepts a single statement per request).
    pub async fn migrate(&self) -> StoreResult<()> {
        for statement in split_sql_statements(CLICKHOUSE_TRACE_STORE_MIGRATION) {
            self.execute(&statement).await?;
        }
        Ok(())
    }

    /// Sends a statement that returns no rows.
    async fn execute(&self, sql: &str) -> StoreResult<()> {
        let response = self
            .http
            .post(&self.endpoint)
            .body(sql.to_string())
            .send()
            .await
            .map_err(StoreError::backend)?;
        Self::check_response(response).await.map(|_| ())
    }

    /// Runs an `INSERT ... FORMAT JSONEachRow` statement, sending the row data as
    /// the request body and the statement itself in the `query` URL parameter.
    ///
    /// ClickHouse's HTTP interface treats a bare POST body as the entire query,
    /// but inlining bulk row data into the body that way is fragile for larger
    /// payloads; the `query` parameter + body split is the supported pattern for
    /// streaming insert data.
    async fn insert(&self, statement: &str, rows: String) -> StoreResult<()> {
        let response = self
            .http
            .post(&self.endpoint)
            .query(&[
                ("query", statement),
                // Make the insert fully synchronous and durable before the
                // request returns so the app-level idempotency lookups on a
                // subsequent write observe these rows, and disable ClickHouse's
                // identical-block deduplication (our `WriteAck` duplicate counts
                // are enforced in-app instead).
                ("async_insert", "0"),
                ("wait_end_of_query", "1"),
                ("insert_deduplicate", "0"),
            ])
            .body(rows)
            .send()
            .await
            .map_err(StoreError::backend)?;
        Self::check_response(response).await.map(|_| ())
    }

    /// Sends a query and returns the raw response body.
    async fn query_raw(&self, sql: &str) -> StoreResult<String> {
        let response = self
            .http
            .post(&self.endpoint)
            .body(sql.to_string())
            .send()
            .await
            .map_err(StoreError::backend)?;
        Self::check_response(response).await
    }

    async fn check_response(response: reqwest::Response) -> StoreResult<String> {
        let status = response.status();
        let body = response.text().await.map_err(StoreError::backend)?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(StoreError::Backend(format!(
                "clickhouse responded {status}: {body}"
            )))
        }
    }

    /// Returns the set of existing `(tenant, project, idempotency_key)` raw keys
    /// among the candidates so duplicate inserts can be filtered out.
    ///
    /// Batched into a single `SELECT ... WHERE (tenant, project, key) IN (...)`
    /// round-trip rather than one query per candidate. Only the matching keys are
    /// returned (`SELECT DISTINCT` of the tuple), so the response is bounded by
    /// the number of genuine duplicates.
    async fn existing_raw_keys(
        &self,
        candidates: &[(String, String, String)],
    ) -> StoreResult<BTreeSet<(String, String, String)>> {
        if candidates.is_empty() {
            return Ok(BTreeSet::new());
        }
        // Each candidate is rendered with the explicit `tuple(...)` constructor.
        // ClickHouse's `IN` has a historical-compatibility ambiguity: when the
        // right side is a *single* parenthesised tuple — as it is whenever a
        // batch carries one raw envelope — the redundant outer parentheses in
        // `(('a','b','c'))` collapse and the set is parsed as three scalar
        // values rather than one composite-key tuple, so the tuple-valued left
        // side never matches and the duplicate goes undetected. `tuple(...)`
        // forces the set element to stay a tuple regardless of cardinality.
        let tuples = candidates
            .iter()
            .map(|(tenant, project, key)| {
                format!(
                    "tuple('{}','{}','{}')",
                    escape(tenant),
                    escape(project),
                    escape(key)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT DISTINCT tenant_id, project_id, idempotency_key FROM beater.raw_envelopes WHERE (tenant_id, project_id, idempotency_key) IN ({tuples}) FORMAT JSONEachRow"
        );
        let body = self.query_raw(&sql).await?;
        let mut found = BTreeSet::new();
        for line in body.lines().filter(|line| !line.trim().is_empty()) {
            let row: serde_json::Value = serde_json::from_str(line).map_err(StoreError::backend)?;
            found.insert((
                json_str(&row, "tenant_id")?,
                json_str(&row, "project_id")?,
                json_str(&row, "idempotency_key")?,
            ));
        }
        Ok(found)
    }

    /// Returns the set of existing span primary keys among the candidates.
    ///
    /// Batched into a single `SELECT ... WHERE (...) IN (...)` round-trip rather
    /// than one query per candidate.
    async fn existing_span_keys(&self, candidates: &[SpanKey]) -> StoreResult<BTreeSet<SpanKey>> {
        if candidates.is_empty() {
            return Ok(BTreeSet::new());
        }
        // Use the explicit `tuple(...)` constructor for the same reason as
        // `existing_raw_keys`: a single-candidate batch otherwise produces a
        // single-tuple `IN` set whose redundant parentheses collapse, defeating
        // the composite-key match and the in-app duplicate detection.
        //
        // Compare `seq` as a String on BOTH sides instead of as a UInt64. The
        // raw-envelope dedup (an all-String tuple) matches correctly, but the
        // 5-element span tuple mixing four Strings with a UInt64 `seq` does not:
        // ClickHouse's element-wise tuple `IN` is sensitive to the numeric type
        // of the literal `seq` vs the UInt64 column (a bare literal infers as a
        // narrower type, and even `toUInt64(...)` left the just-inserted span
        // keys unmatched, so `duplicate_spans` stayed 0). Casting the column
        // with `toString(seq)` and quoting the literal makes the entire tuple
        // all-String — identical in shape to the proven raw path — so the
        // composite key matches reliably regardless of numeric type inference.
        let tuples = candidates
            .iter()
            .map(|key| {
                format!(
                    "tuple('{}','{}','{}','{}','{}')",
                    escape(&key.tenant),
                    escape(&key.project),
                    escape(&key.trace),
                    escape(&key.span),
                    key.seq
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT DISTINCT tenant_id, project_id, trace_id, span_id, seq FROM beater.spans WHERE (tenant_id, project_id, trace_id, span_id, toString(seq)) IN ({tuples}) FORMAT JSONEachRow"
        );
        let body = self.query_raw(&sql).await?;
        let mut found = BTreeSet::new();
        for line in body.lines().filter(|line| !line.trim().is_empty()) {
            let row: serde_json::Value = serde_json::from_str(line).map_err(StoreError::backend)?;
            // `seq` is a UInt64 column; ClickHouse JSONEachRow renders large
            // unsigned ints as quoted strings, so accept either representation.
            let seq = row
                .get("seq")
                .and_then(json_u64)
                .ok_or_else(|| StoreError::Backend("missing or invalid seq column".to_string()))?;
            found.insert(SpanKey {
                tenant: json_str(&row, "tenant_id")?,
                project: json_str(&row, "project_id")?,
                trace: json_str(&row, "trace_id")?,
                span: json_str(&row, "span_id")?,
                seq,
            });
        }
        Ok(found)
    }

    async fn span_json_query(&self, sql: &str) -> StoreResult<Vec<CanonicalSpan>> {
        let body = self.query_raw(sql).await?;
        let mut spans = Vec::new();
        for line in body.lines().filter(|line| !line.trim().is_empty()) {
            // Each row is a single JSON object: {"span_json": "<escaped json>"}.
            let row: serde_json::Value = serde_json::from_str(line).map_err(StoreError::backend)?;
            let inner = row
                .get("span_json")
                .and_then(|value| value.as_str())
                .ok_or_else(|| StoreError::Backend("missing span_json column".to_string()))?;
            spans.push(serde_json::from_str::<CanonicalSpan>(inner).map_err(StoreError::backend)?);
        }
        Ok(spans)
    }

    async fn get_trace_with_project(
        &self,
        tenant: TenantId,
        project: Option<ProjectId>,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        let mut predicates = vec![
            format!("tenant_id = '{}'", escape(tenant.as_str())),
            format!("trace_id = '{}'", escape(trace.as_str())),
        ];
        if let Some(project) = &project {
            predicates.push(format!("project_id = '{}'", escape(project.as_str())));
        }
        let sql = format!(
            "SELECT span_json FROM beater.spans WHERE {} ORDER BY seq ASC, start_time ASC FORMAT JSONEachRow",
            predicates.join(" AND ")
        );
        let spans = self.span_json_query(&sql).await?;
        Ok(TraceView {
            tenant_id: tenant,
            trace_id: trace,
            spans,
        })
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SpanKey {
    tenant: String,
    project: String,
    trace: String,
    span: String,
    seq: u64,
}

#[async_trait]
impl TraceStore for ClickHouseTraceStore {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
        // De-duplicate against what is already persisted (MergeTree does not
        // enforce uniqueness on insert).
        let raw_candidates: Vec<(String, String, String)> = batch
            .raw_envelopes
            .iter()
            .map(|raw| {
                (
                    raw.tenant_id.as_str().to_string(),
                    raw.project_id.as_str().to_string(),
                    raw.idempotency_key.as_str().to_string(),
                )
            })
            .collect();
        let existing_raw = self.existing_raw_keys(&raw_candidates).await?;

        let mut accepted_raw = 0;
        let mut duplicate_raw = 0;
        let mut seen_raw: BTreeSet<(String, String, String)> = BTreeSet::new();
        let mut raw_rows = String::new();
        for raw in &batch.raw_envelopes {
            let key = (
                raw.tenant_id.as_str().to_string(),
                raw.project_id.as_str().to_string(),
                raw.idempotency_key.as_str().to_string(),
            );
            if existing_raw.contains(&key) || !seen_raw.insert(key) {
                duplicate_raw += 1;
                continue;
            }
            let raw_json = serde_json::to_string(raw).map_err(StoreError::backend)?;
            let row = serde_json::json!({
                "tenant_id": raw.tenant_id.as_str(),
                "project_id": raw.project_id.as_str(),
                "idempotency_key": raw.idempotency_key.as_str(),
                "trace_id": serde_json::Value::Null,
                "payload_hash": raw.payload_hash.as_str(),
                "received_at": format_clickhouse_datetime(&raw.received_at),
                "source": raw.source.as_str(),
                "normalizer_version": "",
                "raw_json": raw_json,
            });
            raw_rows.push_str(&serde_json::to_string(&row).map_err(StoreError::backend)?);
            raw_rows.push('\n');
            accepted_raw += 1;
        }
        if !raw_rows.is_empty() {
            self.insert(
                "INSERT INTO beater.raw_envelopes FORMAT JSONEachRow",
                raw_rows,
            )
            .await?;
        }

        let span_candidates: Vec<SpanKey> = batch
            .spans
            .iter()
            .map(|span| SpanKey {
                tenant: span.tenant_id.as_str().to_string(),
                project: span.project_id.as_str().to_string(),
                trace: span.trace_id.as_str().to_string(),
                span: span.span_id.as_str().to_string(),
                seq: span.seq,
            })
            .collect();
        let existing_spans = self.existing_span_keys(&span_candidates).await?;

        let mut accepted_spans = 0;
        let mut duplicate_spans = 0;
        let mut seen_spans: BTreeSet<SpanKey> = BTreeSet::new();
        let mut span_rows = String::new();
        for span in &batch.spans {
            let key = SpanKey {
                tenant: span.tenant_id.as_str().to_string(),
                project: span.project_id.as_str().to_string(),
                trace: span.trace_id.as_str().to_string(),
                span: span.span_id.as_str().to_string(),
                seq: span.seq,
            };
            if existing_spans.contains(&key) || !seen_spans.insert(key) {
                duplicate_spans += 1;
                continue;
            }
            let span_json = serde_json::to_string(span).map_err(StoreError::backend)?;
            let row = serde_json::json!({
                "tenant_id": span.tenant_id.as_str(),
                "project_id": span.project_id.as_str(),
                "environment_id": span.environment_id.as_str(),
                "trace_id": span.trace_id.as_str(),
                "span_id": span.span_id.as_str(),
                "parent_span_id": span.parent_span_id.as_ref().map(|id| id.as_str()),
                "seq": span.seq,
                "kind": span.kind.as_str(),
                "status": span.status.as_str(),
                "name": span.name,
                "start_time": format_clickhouse_datetime(&span.start_time),
                "end_time": span.end_time.as_ref().map(format_clickhouse_datetime),
                "span_json": span_json,
            });
            span_rows.push_str(&serde_json::to_string(&row).map_err(StoreError::backend)?);
            span_rows.push('\n');
            accepted_spans += 1;
        }
        if !span_rows.is_empty() {
            self.insert("INSERT INTO beater.spans FORMAT JSONEachRow", span_rows)
                .await?;
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
        let sql = format!(
            "SELECT raw_json FROM beater.raw_envelopes WHERE tenant_id = '{}' AND project_id = '{}' AND idempotency_key = '{}' LIMIT 1 FORMAT JSONEachRow",
            escape(tenant.as_str()),
            escape(project.as_str()),
            escape(idempotency_key.as_str())
        );
        let body = self.query_raw(&sql).await?;
        let Some(line) = body.lines().find(|line| !line.trim().is_empty()) else {
            return Ok(None);
        };
        let row: serde_json::Value = serde_json::from_str(line).map_err(StoreError::backend)?;
        let inner = row
            .get("raw_json")
            .and_then(|value| value.as_str())
            .ok_or_else(|| StoreError::Backend("missing raw_json column".to_string()))?;
        Ok(Some(
            serde_json::from_str::<RawEnvelope>(inner).map_err(StoreError::backend)?,
        ))
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
        let mut predicates = vec![format!("tenant_id = '{}'", escape(tenant.as_str()))];
        if let Some(project) = &filter.project_id {
            predicates.push(format!("project_id = '{}'", escape(project.as_str())));
        }
        if let Some(environment) = &filter.environment_id {
            predicates.push(format!(
                "environment_id = '{}'",
                escape(environment.as_str())
            ));
        }
        if let Some(trace) = &filter.trace_id {
            predicates.push(format!("trace_id = '{}'", escape(trace.as_str())));
        }
        if let Some(span) = &filter.span_id {
            predicates.push(format!("span_id = '{}'", escape(span.as_str())));
        }
        if let Some(kind) = &filter.kind {
            predicates.push(format!("kind = '{}'", escape(kind.as_str())));
        }
        if let Some(status) = &filter.status {
            predicates.push(format!("status = '{}'", escape(status.as_str())));
        }
        let sql = format!(
            "SELECT span_json FROM beater.spans WHERE {} ORDER BY start_time DESC, seq ASC FORMAT JSONEachRow",
            predicates.join(" AND ")
        );
        let spans = self.span_json_query(&sql).await?;
        let summaries = spans.into_iter().map(span_summary).collect();
        Ok(page_vec(summaries, page))
    }
}

/// Splits a multi-statement migration into individual statements. ClickHouse's
/// HTTP interface accepts only one statement per request.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    for line in sql.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("--") {
            continue;
        }
        current.push_str(line);
        current.push('\n');
        if line.trim_end().ends_with(';') {
            let statement = current.trim().trim_end_matches(';').trim().to_string();
            if !statement.is_empty() {
                statements.push(statement);
            }
            current.clear();
        }
    }
    let tail = current.trim().trim_end_matches(';').trim().to_string();
    if !tail.is_empty() {
        statements.push(tail);
    }
    statements
}

/// Extracts a required string column from a ClickHouse `JSONEachRow` object.
fn json_str(row: &serde_json::Value, column: &str) -> StoreResult<String> {
    row.get(column)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| StoreError::Backend(format!("missing {column} column")))
}

/// Reads a `UInt64` column from a ClickHouse `JSONEachRow` value, accepting both
/// the numeric and the quoted-string renderings ClickHouse may emit.
fn json_u64(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<u64>().ok()))
}

/// Escapes a string for inclusion in a single-quoted ClickHouse SQL literal.
fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Formats a timestamp for ClickHouse `DateTime64(6, 'UTC')` columns.
fn format_clickhouse_datetime(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}
