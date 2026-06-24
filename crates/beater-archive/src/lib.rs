use anyhow::{anyhow, Context};
use arrow_array::{
    Array, ArrayRef, LargeStringArray, RecordBatch, StringArray, StringViewArray, UInt64Array,
};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use beater_core::{EnvironmentId, ProjectId, SpanId, TenantId, Timestamp, TraceId};
use beater_schema::{AgentSpanKind, CanonicalSpan, SpanStatus};
use chrono::Utc;
use datafusion::prelude::{ParquetReadOptions, SessionContext};
use parquet::arrow::ArrowWriter;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

const TABLE_NAME: &str = "spans";

#[derive(Clone, Debug)]
pub struct ParquetTraceArchive {
    root: Arc<PathBuf>,
}

impl ParquetTraceArchive {
    pub fn new(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root)
            .with_context(|| format!("create archive root {}", root.display()))?;
        Ok(Self {
            root: Arc::new(root),
        })
    }

    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }

    pub async fn archive_spans(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        spans: &[CanonicalSpan],
    ) -> anyhow::Result<ArchiveManifest> {
        if spans.is_empty() {
            return Err(anyhow!("cannot archive an empty span set"));
        }
        if spans
            .iter()
            .any(|span| span.tenant_id.as_str() != tenant_id.as_str())
        {
            return Err(anyhow!("archive span set crosses tenant boundary"));
        }
        if spans
            .iter()
            .any(|span| span.project_id.as_str() != project_id.as_str())
        {
            return Err(anyhow!("archive span set crosses project boundary"));
        }

        let path = self.archive_path(tenant_id, project_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create archive dir {}", parent.display()))?;
        }

        let batch = spans_to_batch(spans)?;
        let file = fs::File::create(&path)
            .with_context(|| format!("create parquet archive {}", path.display()))?;
        let mut writer =
            ArrowWriter::try_new(file, batch.schema(), None).context("create parquet writer")?;
        writer.write(&batch).context("write parquet span batch")?;
        writer.close().context("close parquet writer")?;

        Ok(ArchiveManifest {
            path,
            span_count: spans.len(),
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            created_at: Utc::now(),
        })
    }

    pub async fn query_file(
        &self,
        manifest: &ArchiveManifest,
        query: ArchiveQuery,
    ) -> anyhow::Result<Vec<ArchivedSpanRow>> {
        self.query_path(&manifest.path, query).await
    }

    pub async fn query_path(
        &self,
        path: impl AsRef<Path>,
        query: ArchiveQuery,
    ) -> anyhow::Result<Vec<ArchivedSpanRow>> {
        let path = path.as_ref();
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("archive path is not valid UTF-8: {}", path.display()))?;
        let ctx = SessionContext::new();
        ctx.register_parquet(TABLE_NAME, path_str, ParquetReadOptions::default())
            .await
            .with_context(|| format!("register parquet archive {}", path.display()))?;
        let sql = build_query_sql(&query);
        let dataframe = ctx.sql(&sql).await.context("compile archive query")?;
        let batches = dataframe.collect().await.context("run archive query")?;
        rows_from_batches(&batches)
    }

    pub async fn query_project(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        mut query: ArchiveQuery,
    ) -> anyhow::Result<Vec<ArchivedSpanRow>> {
        if query.tenant_id.as_str() != tenant_id.as_str() {
            return Err(anyhow!("archive query crosses tenant boundary"));
        }
        if let Some(query_project) = &query.project_id {
            if query_project.as_str() != project_id.as_str() {
                return Err(anyhow!("archive query crosses project boundary"));
            }
        }
        query.project_id = Some(project_id.clone());
        let project_dir = self.project_dir(tenant_id, project_id);
        if !project_dir_has_parquet(&project_dir)? {
            return Ok(Vec::new());
        }
        self.query_path(project_dir, query).await
    }

    fn archive_path(&self, tenant_id: &TenantId, project_id: &ProjectId) -> PathBuf {
        let file_name = format!(
            "{}-{}.parquet",
            Utc::now().format("%Y%m%dT%H%M%S%.fZ"),
            Uuid::new_v4()
        );
        self.project_dir(tenant_id, project_id).join(file_name)
    }

    fn project_dir(&self, tenant_id: &TenantId, project_id: &ProjectId) -> PathBuf {
        self.root
            .join(safe_segment(tenant_id.as_str()))
            .join(safe_segment(project_id.as_str()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub path: PathBuf,
    pub span_count: usize,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveQuery {
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub environment_id: Option<EnvironmentId>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub kind: Option<AgentSpanKind>,
    pub status: Option<SpanStatus>,
    pub limit: Option<usize>,
}

impl ArchiveQuery {
    pub fn tenant(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            project_id: None,
            environment_id: None,
            trace_id: None,
            span_id: None,
            kind: None,
            status: None,
            limit: Some(100),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedSpanRow {
    pub tenant_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub seq: u64,
    pub kind: String,
    pub status: String,
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub model_provider: Option<String>,
    pub model_name: Option<String>,
    pub cost_amount_micros: Option<String>,
    pub cost_currency: Option<String>,
    pub input_tokens: Option<String>,
    pub output_tokens: Option<String>,
    pub reasoning_tokens: Option<String>,
    pub attributes_json: String,
    pub unmapped_json: String,
    pub input_uri: Option<String>,
    pub output_uri: Option<String>,
    pub raw_uri: String,
}

fn spans_to_batch(spans: &[CanonicalSpan]) -> anyhow::Result<RecordBatch> {
    let schema = archive_schema();
    let attributes_json = spans
        .iter()
        .map(|span| serde_json::to_string(&span.attributes))
        .collect::<Result<Vec<_>, _>>()
        .context("serialize span attributes for archive")?;
    let unmapped_json = spans
        .iter()
        .map(|span| serde_json::to_string(&span.unmapped_attrs))
        .collect::<Result<Vec<_>, _>>()
        .context("serialize unmapped span attributes for archive")?;

    RecordBatch::try_new(
        schema,
        vec![
            strings(spans.iter().map(|span| span.tenant_id.as_str().to_string())),
            strings(
                spans
                    .iter()
                    .map(|span| span.project_id.as_str().to_string()),
            ),
            strings(
                spans
                    .iter()
                    .map(|span| span.environment_id.as_str().to_string()),
            ),
            strings(spans.iter().map(|span| span.trace_id.as_str().to_string())),
            strings(spans.iter().map(|span| span.span_id.as_str().to_string())),
            opt_strings(spans.iter().map(|span| {
                span.parent_span_id
                    .as_ref()
                    .map(|id| id.as_str().to_string())
            })),
            Arc::new(UInt64Array::from(
                spans.iter().map(|span| span.seq).collect::<Vec<_>>(),
            )),
            strings(spans.iter().map(|span| span.kind.as_str().to_string())),
            strings(spans.iter().map(|span| span.status.as_str().to_string())),
            strings(spans.iter().map(|span| span.name.clone())),
            strings(spans.iter().map(|span| span.start_time.to_rfc3339())),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.end_time.map(|time| time.to_rfc3339())),
            ),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.model.as_ref().map(|model| model.provider.clone())),
            ),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.model.as_ref().map(|model| model.name.clone())),
            ),
            opt_strings(spans.iter().map(|span| {
                span.cost
                    .as_ref()
                    .map(|cost| cost.amount_micros.to_string())
            })),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.cost.as_ref().map(|cost| cost.currency.to_string())),
            ),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.tokens.as_ref().map(|tokens| tokens.input.to_string())),
            ),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.tokens.as_ref().map(|tokens| tokens.output.to_string())),
            ),
            opt_strings(spans.iter().map(|span| {
                span.tokens
                    .as_ref()
                    .map(|tokens| tokens.reasoning.to_string())
            })),
            strings(attributes_json),
            strings(unmapped_json),
            opt_strings(
                spans
                    .iter()
                    .map(|span| span.input_ref.as_ref().map(|artifact| artifact.uri.clone())),
            ),
            opt_strings(spans.iter().map(|span| {
                span.output_ref
                    .as_ref()
                    .map(|artifact| artifact.uri.clone())
            })),
            strings(spans.iter().map(|span| span.raw_ref.uri.clone())),
        ],
    )
    .context("build archive record batch")
}

fn archive_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("tenant_id", DataType::Utf8, false),
        Field::new("project_id", DataType::Utf8, false),
        Field::new("environment_id", DataType::Utf8, false),
        Field::new("trace_id", DataType::Utf8, false),
        Field::new("span_id", DataType::Utf8, false),
        Field::new("parent_span_id", DataType::Utf8, true),
        Field::new("seq", DataType::UInt64, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("start_time", DataType::Utf8, false),
        Field::new("end_time", DataType::Utf8, true),
        Field::new("model_provider", DataType::Utf8, true),
        Field::new("model_name", DataType::Utf8, true),
        Field::new("cost_amount_micros", DataType::Utf8, true),
        Field::new("cost_currency", DataType::Utf8, true),
        Field::new("input_tokens", DataType::Utf8, true),
        Field::new("output_tokens", DataType::Utf8, true),
        Field::new("reasoning_tokens", DataType::Utf8, true),
        Field::new("attributes_json", DataType::Utf8, false),
        Field::new("unmapped_json", DataType::Utf8, false),
        Field::new("input_uri", DataType::Utf8, true),
        Field::new("output_uri", DataType::Utf8, true),
        Field::new("raw_uri", DataType::Utf8, false),
    ]))
}

fn build_query_sql(query: &ArchiveQuery) -> String {
    let mut predicates = vec![format!(
        "tenant_id = '{}'",
        sql_literal(query.tenant_id.as_str())
    )];
    if let Some(project_id) = &query.project_id {
        predicates.push(format!(
            "project_id = '{}'",
            sql_literal(project_id.as_str())
        ));
    }
    if let Some(environment_id) = &query.environment_id {
        predicates.push(format!(
            "environment_id = '{}'",
            sql_literal(environment_id.as_str())
        ));
    }
    if let Some(trace_id) = &query.trace_id {
        predicates.push(format!("trace_id = '{}'", sql_literal(trace_id.as_str())));
    }
    if let Some(span_id) = &query.span_id {
        predicates.push(format!("span_id = '{}'", sql_literal(span_id.as_str())));
    }
    if let Some(kind) = &query.kind {
        predicates.push(format!("kind = '{}'", sql_literal(kind.as_str())));
    }
    if let Some(status) = &query.status {
        predicates.push(format!("status = '{}'", sql_literal(status.as_str())));
    }

    format!(
        "SELECT {} FROM {} WHERE {} ORDER BY start_time ASC, seq ASC LIMIT {}",
        archive_columns().join(", "),
        TABLE_NAME,
        predicates.join(" AND "),
        query.limit.unwrap_or(100).clamp(1, 1000)
    )
}

fn archive_columns() -> Vec<String> {
    archive_schema()
        .fields()
        .iter()
        .map(|field| field.name().clone())
        .collect()
}

fn rows_from_batches(batches: &[RecordBatch]) -> anyhow::Result<Vec<ArchivedSpanRow>> {
    let mut rows = Vec::new();
    for batch in batches {
        let columns = BatchColumns::from_batch(batch)?;
        for row in 0..batch.num_rows() {
            rows.push(ArchivedSpanRow {
                tenant_id: string_value(columns.tenant_id, row),
                project_id: string_value(columns.project_id, row),
                environment_id: string_value(columns.environment_id, row),
                trace_id: string_value(columns.trace_id, row),
                span_id: string_value(columns.span_id, row),
                parent_span_id: optional_string_value(columns.parent_span_id, row),
                seq: columns.seq.value(row),
                kind: string_value(columns.kind, row),
                status: string_value(columns.status, row),
                name: string_value(columns.name, row),
                start_time: string_value(columns.start_time, row),
                end_time: optional_string_value(columns.end_time, row),
                model_provider: optional_string_value(columns.model_provider, row),
                model_name: optional_string_value(columns.model_name, row),
                cost_amount_micros: optional_string_value(columns.cost_amount_micros, row),
                cost_currency: optional_string_value(columns.cost_currency, row),
                input_tokens: optional_string_value(columns.input_tokens, row),
                output_tokens: optional_string_value(columns.output_tokens, row),
                reasoning_tokens: optional_string_value(columns.reasoning_tokens, row),
                attributes_json: string_value(columns.attributes_json, row),
                unmapped_json: string_value(columns.unmapped_json, row),
                input_uri: optional_string_value(columns.input_uri, row),
                output_uri: optional_string_value(columns.output_uri, row),
                raw_uri: string_value(columns.raw_uri, row),
            });
        }
    }
    Ok(rows)
}

struct BatchColumns<'a> {
    tenant_id: StringColumn<'a>,
    project_id: StringColumn<'a>,
    environment_id: StringColumn<'a>,
    trace_id: StringColumn<'a>,
    span_id: StringColumn<'a>,
    parent_span_id: StringColumn<'a>,
    seq: &'a UInt64Array,
    kind: StringColumn<'a>,
    status: StringColumn<'a>,
    name: StringColumn<'a>,
    start_time: StringColumn<'a>,
    end_time: StringColumn<'a>,
    model_provider: StringColumn<'a>,
    model_name: StringColumn<'a>,
    cost_amount_micros: StringColumn<'a>,
    cost_currency: StringColumn<'a>,
    input_tokens: StringColumn<'a>,
    output_tokens: StringColumn<'a>,
    reasoning_tokens: StringColumn<'a>,
    attributes_json: StringColumn<'a>,
    unmapped_json: StringColumn<'a>,
    input_uri: StringColumn<'a>,
    output_uri: StringColumn<'a>,
    raw_uri: StringColumn<'a>,
}

impl<'a> BatchColumns<'a> {
    fn from_batch(batch: &'a RecordBatch) -> anyhow::Result<Self> {
        Ok(Self {
            tenant_id: string_column(batch, "tenant_id")?,
            project_id: string_column(batch, "project_id")?,
            environment_id: string_column(batch, "environment_id")?,
            trace_id: string_column(batch, "trace_id")?,
            span_id: string_column(batch, "span_id")?,
            parent_span_id: string_column(batch, "parent_span_id")?,
            seq: u64_column(batch, "seq")?,
            kind: string_column(batch, "kind")?,
            status: string_column(batch, "status")?,
            name: string_column(batch, "name")?,
            start_time: string_column(batch, "start_time")?,
            end_time: string_column(batch, "end_time")?,
            model_provider: string_column(batch, "model_provider")?,
            model_name: string_column(batch, "model_name")?,
            cost_amount_micros: string_column(batch, "cost_amount_micros")?,
            cost_currency: string_column(batch, "cost_currency")?,
            input_tokens: string_column(batch, "input_tokens")?,
            output_tokens: string_column(batch, "output_tokens")?,
            reasoning_tokens: string_column(batch, "reasoning_tokens")?,
            attributes_json: string_column(batch, "attributes_json")?,
            unmapped_json: string_column(batch, "unmapped_json")?,
            input_uri: string_column(batch, "input_uri")?,
            output_uri: string_column(batch, "output_uri")?,
            raw_uri: string_column(batch, "raw_uri")?,
        })
    }
}

#[derive(Clone, Copy)]
enum StringColumn<'a> {
    Utf8(&'a StringArray),
    LargeUtf8(&'a LargeStringArray),
    Utf8View(&'a StringViewArray),
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> anyhow::Result<StringColumn<'a>> {
    let index = batch
        .schema()
        .index_of(name)
        .with_context(|| format!("find archive column {name}"))?;
    let column = batch.column(index);
    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(StringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<LargeStringArray>() {
        return Ok(StringColumn::LargeUtf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(StringColumn::Utf8View(array));
    }
    Err(anyhow!(
        "archive column {name} is not a string column; data_type={:?}",
        column.data_type()
    ))
}

fn u64_column<'a>(batch: &'a RecordBatch, name: &str) -> anyhow::Result<&'a UInt64Array> {
    let index = batch
        .schema()
        .index_of(name)
        .with_context(|| format!("find archive column {name}"))?;
    batch
        .column(index)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| anyhow!("archive column {name} is not u64"))
}

fn string_value(column: StringColumn<'_>, row: usize) -> String {
    match column {
        StringColumn::Utf8(array) => array.value(row).to_string(),
        StringColumn::LargeUtf8(array) => array.value(row).to_string(),
        StringColumn::Utf8View(array) => array.value(row).to_string(),
    }
}

fn optional_string_value(column: StringColumn<'_>, row: usize) -> Option<String> {
    if column.is_null(row) {
        None
    } else {
        Some(string_value(column, row))
    }
}

impl StringColumn<'_> {
    fn is_null(self, row: usize) -> bool {
        match self {
            Self::Utf8(array) => array.is_null(row),
            Self::LargeUtf8(array) => array.is_null(row),
            Self::Utf8View(array) => array.is_null(row),
        }
    }
}

fn strings(values: impl IntoIterator<Item = String>) -> ArrayRef {
    Arc::new(StringArray::from(values.into_iter().collect::<Vec<_>>()))
}

fn opt_strings(values: impl IntoIterator<Item = Option<String>>) -> ArrayRef {
    Arc::new(StringArray::from(
        values.into_iter().collect::<Vec<Option<String>>>(),
    ))
}

fn safe_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn project_dir_has_parquet(path: &Path) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    if !path.is_dir() {
        return Ok(false);
    }
    for entry in
        fs::read_dir(path).with_context(|| format!("read archive dir {}", path.display()))?
    {
        let entry = entry.with_context(|| format!("read archive entry {}", path.display()))?;
        if entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("parquet")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{
        ArtifactId, EnvironmentId, IdempotencyKey, Money, Sha256Hash, TenantScope, TokenCounts,
    };
    use beater_schema::{
        ArtifactRef, AuthContext, CanonicalTraceBatch, RawEnvelope, RedactionClass, SourceDialect,
        CANONICAL_SCHEMA_VERSION, RAW_SCHEMA_VERSION,
    };
    use beater_store::TraceStore;
    use beater_store_sql::SqliteTraceStore;
    use serde_json::json;
    use std::collections::{BTreeMap, BTreeSet};

    #[tokio::test]
    async fn writes_parquet_and_queries_with_datafusion_filters() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-a").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project-a").unwrap_or_else(|err| panic!("{err}"));
        let spans = vec![
            fixture_span(
                &tenant,
                &project,
                "prod",
                "trace-a",
                "span-a",
                1,
                SpanStatus::Ok,
            ),
            fixture_span(
                &tenant,
                &project,
                "prod",
                "trace-a",
                "span-b",
                2,
                SpanStatus::Error,
            ),
        ];

        let manifest = archive
            .archive_spans(&tenant, &project, &spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let rows = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant.clone(),
                    project_id: Some(project),
                    environment_id: None,
                    trace_id: Some(TraceId::new("trace-a").unwrap_or_else(|err| panic!("{err}"))),
                    span_id: None,
                    kind: Some(AgentSpanKind::ToolCall),
                    status: Some(SpanStatus::Error),
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(manifest.span_count, 2);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].span_id, "span-b");
        assert_eq!(rows[0].status, "error");
        assert_eq!(rows[0].cost_amount_micros.as_deref(), Some("2500"));
        assert_eq!(rows[0].input_tokens.as_deref(), Some("7"));
    }

    #[tokio::test]
    async fn archives_hot_trace_store_view_to_cold_datafusion_query() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive = ParquetTraceArchive::new(tempdir.path().join("archive"))
            .unwrap_or_else(|err| panic!("{err}"));
        let store = SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-hot").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project-hot").unwrap_or_else(|err| panic!("{err}"));
        let trace = TraceId::new("trace-hot").unwrap_or_else(|err| panic!("{err}"));
        let raw = fixture_raw(&tenant, &project, "prod");
        let span = fixture_span(
            &tenant,
            &project,
            "prod",
            trace.as_str(),
            "span-hot",
            1,
            SpanStatus::Ok,
        );
        let batch = CanonicalTraceBatch::one(raw, span);

        store
            .write_batch(batch)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let trace_view = store
            .get_trace(tenant.clone(), trace.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let manifest = archive
            .archive_spans(&tenant, &project, &trace_view.spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let rows = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant,
                    project_id: None,
                    environment_id: None,
                    trace_id: Some(trace),
                    span_id: None,
                    kind: None,
                    status: None,
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].trace_id, "trace-hot");
        assert_eq!(rows[0].raw_uri, "artifact://tenant-hot/project-hot/raw");
    }

    #[tokio::test]
    async fn project_query_without_archive_files_returns_empty_rows() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-empty").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project-empty").unwrap_or_else(|err| panic!("{err}"));

        let rows = archive
            .query_project(&tenant, &project, ArchiveQuery::tenant(tenant.clone()))
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn rejects_cross_tenant_archive_batches() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-a").unwrap_or_else(|err| panic!("{err}"));
        let other = TenantId::new("tenant-b").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project-a").unwrap_or_else(|err| panic!("{err}"));
        let spans = vec![fixture_span(
            &other,
            &project,
            "prod",
            "trace-a",
            "span-a",
            1,
            SpanStatus::Ok,
        )];

        let error = archive
            .archive_spans(&tenant, &project, &spans)
            .await
            .err()
            .unwrap_or_else(|| panic!("expected tenant boundary error"));
        assert!(error.to_string().contains("tenant boundary"));
    }

    fn fixture_span(
        tenant: &TenantId,
        project: &ProjectId,
        environment: &str,
        trace: &str,
        span: &str,
        seq: u64,
        status: SpanStatus,
    ) -> CanonicalSpan {
        let environment_id = EnvironmentId::new(environment).unwrap_or_else(|err| panic!("{err}"));
        let trace_id = TraceId::new(trace).unwrap_or_else(|err| panic!("{err}"));
        let span_id = SpanId::new(span).unwrap_or_else(|err| panic!("{err}"));
        let mut attributes = BTreeMap::new();
        attributes.insert("tool.name".to_string(), json!("search"));
        attributes.insert("beater.scope".to_string(), json!("archive-test"));
        CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "beater-archive-test".to_string(),
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            environment_id,
            trace_id,
            span_id,
            parent_span_id: None,
            seq,
            kind: AgentSpanKind::ToolCall,
            name: format!("tool-{span}"),
            status,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            model: None,
            cost: Some(Money::usd_micros(2500)),
            tokens: Some(TokenCounts {
                input: 7,
                output: 11,
                reasoning: 3,
                cache_read: 0,
            }),
            input_ref: None,
            output_ref: None,
            attributes,
            unmapped_attrs: json!({"source": "fixture"}),
            raw_ref: artifact_ref(tenant, project, "raw"),
        }
    }

    fn fixture_raw(tenant: &TenantId, project: &ProjectId, environment: &str) -> RawEnvelope {
        RawEnvelope {
            schema_version: RAW_SCHEMA_VERSION,
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            environment_id: EnvironmentId::new(environment).unwrap_or_else(|err| panic!("{err}")),
            source: SourceDialect::Native,
            source_schema_url: Some("beater://native/v1".to_string()),
            source_schema_version: Some("1".to_string()),
            received_at: Utc::now(),
            idempotency_key: IdempotencyKey::new("archive-test-key")
                .unwrap_or_else(|err| panic!("{err}")),
            payload_hash: Sha256Hash::new("ab".repeat(32)).unwrap_or_else(|err| panic!("{err}")),
            body_ref: artifact_ref(tenant, project, "raw"),
            auth_context: AuthContext {
                api_key_id: None,
                scopes: BTreeSet::new(),
            },
        }
    }

    fn artifact_ref(tenant: &TenantId, project: &ProjectId, name: &str) -> ArtifactRef {
        let _scope = TenantScope::new(
            tenant.clone(),
            project.clone(),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        );
        ArtifactRef {
            artifact_id: ArtifactId::new(name).unwrap_or_else(|err| panic!("{err}")),
            uri: format!(
                "artifact://{}/{}/{}",
                tenant.as_str(),
                project.as_str(),
                name
            ),
            sha256: Sha256Hash::new("cd".repeat(32)).unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 2,
            mime_type: "application/json".to_string(),
            redaction_class: RedactionClass::Internal,
        }
    }
}
