use anyhow::{anyhow, Context};
use arrow_array::{
    Array, ArrayRef, LargeStringArray, RecordBatch, StringArray, StringViewArray, UInt64Array,
};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use beater_core::{EnvironmentId, ProjectId, SpanId, TenantId, Timestamp, TraceId};
use beater_schema::{AgentSpanKind, CanonicalSpan, SpanStatus};
use chrono::Utc;
use datafusion::dataframe::DataFrame;
use datafusion::prelude::{col, lit, ParquetReadOptions, SessionContext};
use parquet::arrow::ArrowWriter;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

mod sweeper;
pub use sweeper::{OrphanedArtifactSweeper, SweepReport};

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

        let path = self.archive_path(tenant_id, project_id, spans)?;
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
        let dataframe = ctx.table(TABLE_NAME).await.context("load archive table")?;
        query_dataframe(dataframe, query).await
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
        let parquet_paths = parquet_files(&project_dir)?;
        if parquet_paths.is_empty() {
            return Ok(Vec::new());
        }
        self.query_paths(&parquet_paths, query).await
    }

    async fn query_paths(
        &self,
        paths: &[PathBuf],
        query: ArchiveQuery,
    ) -> anyhow::Result<Vec<ArchivedSpanRow>> {
        let path_strings = paths
            .iter()
            .map(|path| path_to_utf8(path.as_path()))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let ctx = SessionContext::new();
        let dataframe = ctx
            .read_parquet(path_strings, ParquetReadOptions::default())
            .await
            .context("read parquet archive files")?;
        query_dataframe(dataframe, query).await
    }

    fn archive_path(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        spans: &[CanonicalSpan],
    ) -> anyhow::Result<PathBuf> {
        let partition_month = archive_partition_month(spans)?;
        let file_name = format!("{}.parquet", Uuid::new_v4());
        Ok(self
            .project_dir(tenant_id, project_id)
            .join(partition_month)
            .join(file_name))
    }

    fn project_dir(&self, tenant_id: &TenantId, project_id: &ProjectId) -> PathBuf {
        self.root
            .join(safe_segment(tenant_id.as_str()))
            .join(safe_segment(project_id.as_str()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ArchiveManifest {
    #[schema(value_type = String)]
    pub path: PathBuf,
    pub span_count: usize,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    #[schema(value_type = String, format = DateTime)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
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

/// Build the archive query as a structured DataFusion `DataFrame` instead of
/// interpolated SQL text. Table and column identifiers are compile-time
/// constants; only user-supplied VALUES enter the plan, and they do so through
/// `lit(...)` (a bound `ScalarValue`), never string concatenation. This makes
/// attacker-influenced tenant/project/environment/trace/span IDs and filters
/// structurally incapable of altering the query shape.
fn build_query_dataframe(dataframe: DataFrame, query: &ArchiveQuery) -> anyhow::Result<DataFrame> {
    let column_names = archive_columns();
    let columns: Vec<&str> = column_names.iter().map(String::as_str).collect();
    let mut dataframe = dataframe
        .select_columns(&columns)
        .context("select archive columns")?;

    dataframe = dataframe
        .filter(col("tenant_id").eq(lit(query.tenant_id.as_str())))
        .context("filter tenant_id")?;
    if let Some(project_id) = &query.project_id {
        dataframe = dataframe
            .filter(col("project_id").eq(lit(project_id.as_str())))
            .context("filter project_id")?;
    }
    if let Some(environment_id) = &query.environment_id {
        dataframe = dataframe
            .filter(col("environment_id").eq(lit(environment_id.as_str())))
            .context("filter environment_id")?;
    }
    if let Some(trace_id) = &query.trace_id {
        dataframe = dataframe
            .filter(col("trace_id").eq(lit(trace_id.as_str())))
            .context("filter trace_id")?;
    }
    if let Some(span_id) = &query.span_id {
        dataframe = dataframe
            .filter(col("span_id").eq(lit(span_id.as_str())))
            .context("filter span_id")?;
    }
    if let Some(kind) = &query.kind {
        dataframe = dataframe
            .filter(col("kind").eq(lit(kind.as_str())))
            .context("filter kind")?;
    }
    if let Some(status) = &query.status {
        dataframe = dataframe
            .filter(col("status").eq(lit(status.as_str())))
            .context("filter status")?;
    }

    let dataframe = dataframe
        .sort(vec![
            col("start_time").sort(true, false),
            col("seq").sort(true, false),
        ])
        .context("sort archive rows")?;

    let limit = query.limit.unwrap_or(100).clamp(1, 1000);
    dataframe
        .limit(0, Some(limit))
        .context("limit archive rows")
}

async fn query_dataframe(
    dataframe: DataFrame,
    query: ArchiveQuery,
) -> anyhow::Result<Vec<ArchivedSpanRow>> {
    let dataframe = build_query_dataframe(dataframe, &query)?;
    let batches = dataframe.collect().await.context("run archive query")?;
    rows_from_batches(&batches)
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
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(value.len());
    for &byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    encoded
}

fn archive_partition_month(spans: &[CanonicalSpan]) -> anyhow::Result<String> {
    let earliest_start_time = spans
        .iter()
        .map(|span| span.start_time)
        .min()
        .ok_or_else(|| anyhow!("cannot partition an empty span set"))?;
    Ok(earliest_start_time.format("%Y%m").to_string())
}

fn parquet_files(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_parquet_files(path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_parquet_files(path: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        if is_parquet_path(path) {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if !path.is_dir() {
        return Ok(());
    }

    for entry in
        fs::read_dir(path).with_context(|| format!("read archive dir {}", path.display()))?
    {
        let entry = entry.with_context(|| format!("read archive entry {}", path.display()))?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("read archive entry type {}", entry_path.display()))?;
        if file_type.is_dir() {
            collect_parquet_files(&entry_path, files)?;
        } else if file_type.is_file() && is_parquet_path(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_parquet_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("parquet")
}

fn path_to_utf8(path: &Path) -> anyhow::Result<String> {
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("archive path is not valid UTF-8: {}", path.display()))
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
    async fn archive_path_uses_safe_segments_and_earliest_span_month_partition() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant/path").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project:alpha").unwrap_or_else(|err| panic!("{err}"));
        let mut later_span = fixture_span(
            &tenant,
            &project,
            "prod",
            "trace-partition",
            "span-later",
            2,
            SpanStatus::Ok,
        );
        later_span.start_time = timestamp("2026-03-03T10:00:00Z");
        let mut earliest_span = fixture_span(
            &tenant,
            &project,
            "prod",
            "trace-partition",
            "span-earliest",
            1,
            SpanStatus::Ok,
        );
        earliest_span.start_time = timestamp("2026-02-28T23:59:59Z");

        let manifest = archive
            .archive_spans(&tenant, &project, &[later_span, earliest_span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let relative_path = manifest
            .path
            .strip_prefix(tempdir.path())
            .unwrap_or_else(|err| panic!("{err}"));
        let parent = relative_path
            .parent()
            .unwrap_or_else(|| panic!("archive path has no parent"));
        let file_stem = relative_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("archive file stem is not utf-8"));

        assert_eq!(
            parent,
            PathBuf::from("tenant%2Fpath")
                .join("project%3Aalpha")
                .join("202602")
        );
        assert_eq!(
            relative_path
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("parquet")
        );
        Uuid::parse_str(file_stem).unwrap_or_else(|err| panic!("{err}"));
    }

    #[tokio::test]
    async fn query_project_reads_partitioned_archive_files_recursively() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-recursive").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project-recursive").unwrap_or_else(|err| panic!("{err}"));
        let mut january_span = fixture_span(
            &tenant,
            &project,
            "prod",
            "trace-recursive",
            "span-january",
            1,
            SpanStatus::Ok,
        );
        january_span.start_time = timestamp("2026-01-05T00:00:00Z");
        let mut february_span = fixture_span(
            &tenant,
            &project,
            "prod",
            "trace-recursive",
            "span-february",
            2,
            SpanStatus::Ok,
        );
        february_span.start_time = timestamp("2026-02-05T00:00:00Z");

        archive
            .archive_spans(&tenant, &project, &[february_span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        archive
            .archive_spans(&tenant, &project, &[january_span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let rows = archive
            .query_project(&tenant, &project, ArchiveQuery::tenant(tenant.clone()))
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            rows.iter()
                .map(|row| row.span_id.as_str())
                .collect::<Vec<_>>(),
            vec!["span-january", "span-february"]
        );
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
    async fn archive_path_segments_escape_dot_dot_under_archive_root() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("..").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("..").unwrap_or_else(|err| panic!("{err}"));
        let span = fixture_span(
            &tenant,
            &project,
            "prod",
            "trace",
            "span",
            1,
            SpanStatus::Ok,
        );

        let manifest = archive
            .archive_spans(&tenant, &project, &[span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let root = archive
            .root()
            .canonicalize()
            .unwrap_or_else(|err| panic!("{err}"));
        let archive_dir = manifest
            .path
            .parent()
            .unwrap_or_else(|| panic!("archive path must have a parent"))
            .canonicalize()
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            archive_dir.starts_with(&root),
            "archive dir {} must stay under root {}",
            archive_dir.display(),
            root.display()
        );
        assert!(manifest
            .path
            .components()
            .any(|component| { component.as_os_str() == std::ffi::OsStr::new("%2E%2E") }));
    }

    #[test]
    fn archive_path_segments_do_not_collapse_distinct_ids() {
        assert_eq!(safe_segment("tenant"), "tenant");
        assert_eq!(safe_segment("tenant-1_A"), "tenant-1_A");
        assert_eq!(safe_segment(".."), "%2E%2E");
        assert_eq!(safe_segment("a/b"), "a%2Fb");
        assert_ne!(safe_segment("a/b"), safe_segment("a_b"));
        assert_ne!(safe_segment("a.b"), safe_segment("a_b"));
        assert_ne!(safe_segment("a%b"), safe_segment("a_b"));
    }

    // ── SQL-injection regression tests ────────────────────────────────────────
    //
    // The archive query is now assembled as a structured DataFusion `DataFrame`
    // (`build_query_dataframe`): table/column identifiers are compile-time
    // constants and user-supplied filter VALUES enter the plan exclusively via
    // `lit(...)` as bound `ScalarValue`s. No filter value is ever concatenated
    // into SQL text, so an attacker-influenced tenant/project/environment/
    // trace/span ID cannot change the query shape — it can only fail to match.
    //
    // Note: `TenantId::new` / `TraceId::new` etc. already reject whitespace, so
    // the classic `' OR 1=1 --` form is blocked at construction. The tests below
    // use the harder no-whitespace variants (e.g. `'OR'a'='a`) that do pass the
    // type constructor; the structured path must bind them as opaque literals.

    // --- unit: plan inspection (value bound as literal, not interpolated) ------

    #[tokio::test]
    async fn hostile_id_is_bound_as_literal_not_interpolated_into_sql() {
        // Prove the migration's core invariant directly on the logical plan: a
        // hostile value containing SQL metacharacters is carried verbatim as a
        // bound `Utf8` literal, never escaped-and-spliced into SQL text the way
        // the old string builder required.
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let real_tenant = TenantId::new("real-tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj-lit").unwrap_or_else(|err| panic!("{err}"));
        let manifest = archive
            .archive_spans(
                &real_tenant,
                &project,
                &[fixture_span(
                    &real_tenant,
                    &project,
                    "prod",
                    "trace-l",
                    "span-l",
                    1,
                    SpanStatus::Ok,
                )],
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let hostile_raw = "'OR'1'='1";
        let query = ArchiveQuery {
            tenant_id: TenantId::new(hostile_raw).unwrap_or_else(|err| panic!("{err}")),
            project_id: None,
            environment_id: None,
            trace_id: None,
            span_id: None,
            kind: None,
            status: None,
            limit: Some(10),
        };

        let path_str = manifest
            .path
            .to_str()
            .unwrap_or_else(|| panic!("archive path is not utf-8"));
        let ctx = SessionContext::new();
        ctx.register_parquet(TABLE_NAME, path_str, ParquetReadOptions::default())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let dataframe = ctx
            .table(TABLE_NAME)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let dataframe =
            build_query_dataframe(dataframe, &query).unwrap_or_else(|err| panic!("{err}"));
        let plan = format!("{}", dataframe.logical_plan().display_indent());

        // The raw hostile value survives verbatim as a bound literal …
        assert!(
            plan.contains(hostile_raw),
            "hostile value must be a bound literal; plan={plan}"
        );
        // … and is NOT doubled/escaped the way interpolated-SQL would require,
        // proving no `sql_literal`-style text splicing happens anywhere.
        assert!(
            !plan.contains("''OR''1''=''1"),
            "value must not be SQL-escaped text; plan={plan}"
        );

        // And end-to-end the hostile tenant matches no real-tenant rows.
        let rows = archive
            .query_file(&manifest, query)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            rows.len(),
            0,
            "hostile tenant_id must not return real-tenant rows"
        );
    }

    #[tokio::test]
    async fn hostile_quotes_in_all_id_filter_positions_match_no_rows() {
        // Exercises every ID-typed filter position with a leading-quote marker;
        // through the structured path each value is bound as a literal and
        // matches nothing in the real-tenant archive.
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-allpos").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj-allpos").unwrap_or_else(|err| panic!("{err}"));
        let manifest = archive
            .archive_spans(
                &tenant,
                &project,
                &[fixture_span(
                    &tenant,
                    &project,
                    "prod",
                    "trace-allpos",
                    "span-allpos",
                    1,
                    SpanStatus::Ok,
                )],
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let rows = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant.clone(),
                    project_id: Some(ProjectId::new("'proj").unwrap_or_else(|err| panic!("{err}"))),
                    environment_id: Some(
                        EnvironmentId::new("'env").unwrap_or_else(|err| panic!("{err}")),
                    ),
                    trace_id: Some(TraceId::new("'tr").unwrap_or_else(|err| panic!("{err}"))),
                    span_id: Some(SpanId::new("'sp").unwrap_or_else(|err| panic!("{err}"))),
                    kind: None,
                    status: None,
                    limit: Some(5),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            rows.len(),
            0,
            "hostile quotes in any id filter must return no rows"
        );
    }

    // --- integration: hostile values must return zero rows (not all rows) -----
    //
    // If injection succeeded the WHERE clause would become a tautology and
    // the real rows in the archive would be returned.  Safe escaping makes
    // DataFusion evaluate `column = <escaped-literal>`, which matches nothing.

    #[tokio::test]
    async fn hostile_tenant_id_with_quotes_matches_no_rows() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let real_tenant = TenantId::new("real-tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj-rt").unwrap_or_else(|err| panic!("{err}"));

        let manifest = archive
            .archive_spans(
                &real_tenant,
                &project,
                &[fixture_span(
                    &real_tenant,
                    &project,
                    "prod",
                    "trace-r",
                    "span-r",
                    1,
                    SpanStatus::Ok,
                )],
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // "'OR'a'='a" has no whitespace so TenantId::new accepts it.
        // Bound through `lit(...)` as an opaque literal, it matches no real row.
        let hostile = TenantId::new("'OR'a'='a").unwrap_or_else(|err| panic!("{err}"));
        let rows = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: hostile,
                    project_id: None,
                    environment_id: None,
                    trace_id: None,
                    span_id: None,
                    kind: None,
                    status: None,
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            rows.len(),
            0,
            "injection in tenant_id must not return real-tenant rows"
        );
    }

    #[tokio::test]
    async fn hostile_trace_id_with_quotes_matches_no_rows() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-inj-tr").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj-inj-tr").unwrap_or_else(|err| panic!("{err}"));

        let manifest = archive
            .archive_spans(
                &tenant,
                &project,
                &[fixture_span(
                    &tenant,
                    &project,
                    "prod",
                    "trace-real",
                    "span-real",
                    1,
                    SpanStatus::Ok,
                )],
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let hostile_trace = TraceId::new("'OR'1'='1").unwrap_or_else(|err| panic!("{err}"));
        let rows = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant.clone(),
                    project_id: None,
                    environment_id: None,
                    trace_id: Some(hostile_trace),
                    span_id: None,
                    kind: None,
                    status: None,
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            rows.len(),
            0,
            "injection in trace_id must not return real rows"
        );
    }

    #[tokio::test]
    async fn semicolon_injection_in_span_id_does_not_corrupt_query() {
        // Closest no-whitespace SQL-terminator pattern: ';DROPTABLEspans;--
        // The leading `'` is doubled, keeping the rest inside the string literal.
        // DataFusion must not crash, and must return 0 rows.
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-sem").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj-sem").unwrap_or_else(|err| panic!("{err}"));

        let manifest = archive
            .archive_spans(
                &tenant,
                &project,
                &[fixture_span(
                    &tenant,
                    &project,
                    "prod",
                    "trace-sem",
                    "span-sem",
                    1,
                    SpanStatus::Ok,
                )],
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let hostile_span = SpanId::new("';DROPTABLEspans;--").unwrap_or_else(|err| panic!("{err}"));
        let rows = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant.clone(),
                    project_id: None,
                    environment_id: None,
                    trace_id: None,
                    span_id: Some(hostile_span),
                    kind: None,
                    status: None,
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            rows.len(),
            0,
            "semicolon-injection span_id must not corrupt the DataFusion query"
        );
    }

    #[tokio::test]
    async fn hostile_project_and_environment_ids_match_no_rows() {
        // Covers the two remaining ID-filter positions: project_id and
        // environment_id.  Both are bound through `lit(...)` as literals.
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let archive =
            ParquetTraceArchive::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant-pe").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("proj-pe").unwrap_or_else(|err| panic!("{err}"));

        let manifest = archive
            .archive_spans(
                &tenant,
                &project,
                &[fixture_span(
                    &tenant,
                    &project,
                    "prod",
                    "trace-pe",
                    "span-pe",
                    1,
                    SpanStatus::Ok,
                )],
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let hostile_proj = ProjectId::new("'OR'b'='b").unwrap_or_else(|err| panic!("{err}"));
        let rows_proj = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant.clone(),
                    project_id: Some(hostile_proj),
                    environment_id: None,
                    trace_id: None,
                    span_id: None,
                    kind: None,
                    status: None,
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(rows_proj.len(), 0, "hostile project_id must return no rows");

        let hostile_env = EnvironmentId::new("'OR'c'='c").unwrap_or_else(|err| panic!("{err}"));
        let rows_env = archive
            .query_file(
                &manifest,
                ArchiveQuery {
                    tenant_id: tenant.clone(),
                    project_id: None,
                    environment_id: Some(hostile_env),
                    trace_id: None,
                    span_id: None,
                    kind: None,
                    status: None,
                    limit: Some(10),
                },
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            rows_env.len(),
            0,
            "hostile environment_id must return no rows"
        );
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

    fn timestamp(value: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(value)
            .unwrap_or_else(|err| panic!("{err}"))
            .with_timezone(&Utc)
    }
}
