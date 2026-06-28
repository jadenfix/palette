use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{ProjectId, SpanId, TenantId, TraceId};
use beater_schema::{CanonicalSpan, RedactionClass};
use beater_store::{IntoStoreResult, StoreError, StoreResult, TraceStore};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, BooleanQuery, EmptyQuery, Occur, Query, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, Schema, TantivyDocument, Value, STORED, STRING, TEXT,
};
use tantivy::tokenizer::TokenStream;
use tantivy::{doc, Index, IndexWriter, Term};

#[async_trait]
pub trait SearchIndex: Send + Sync {
    async fn index_spans(&self, spans: &[CanonicalSpan]) -> StoreResult<()>;
    async fn search(&self, query: SearchRequest) -> StoreResult<SearchResponse>;
}

#[derive(Clone)]
pub struct TraceIngestedSearchProcessor {
    traces: Arc<dyn TraceStore>,
    search: Arc<dyn SearchIndex>,
}

impl TraceIngestedSearchProcessor {
    pub fn new(traces: Arc<dyn TraceStore>, search: Arc<dyn SearchIndex>) -> Self {
        Self { traces, search }
    }

    pub async fn process_trace(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        trace_id: TraceId,
    ) -> Result<(), String> {
        index_project_trace(
            self.traces.as_ref(),
            self.search.as_ref(),
            tenant_id,
            project_id,
            trace_id,
        )
        .await
        .map_err(|err| err.to_string())
    }
}

pub async fn index_project_trace(
    traces: &dyn TraceStore,
    search: &dyn SearchIndex,
    tenant_id: TenantId,
    project_id: ProjectId,
    trace_id: TraceId,
) -> StoreResult<()> {
    let trace = traces
        .get_project_trace(tenant_id.clone(), project_id.clone(), trace_id.clone())
        .await
        .map_err(|err| {
            context_store_error(
                err,
                format!(
                    "trace.ingested readback failed for tenant={} project={} trace={}",
                    tenant_id, project_id, trace_id
                ),
            )
        })?;
    search.index_spans(&trace.spans).await.map_err(|err| {
        context_store_error(
            err,
            format!(
                "trace.ingested indexing failed for tenant={} project={} trace={}",
                tenant_id, project_id, trace_id
            ),
        )
    })
}

fn context_store_error(error: StoreError, context: String) -> StoreError {
    match error {
        StoreError::NotFound(message) => StoreError::NotFound(format!("{context}: {message}")),
        StoreError::Conflict(message) => StoreError::Conflict(format!("{context}: {message}")),
        StoreError::Backpressure(message) => {
            StoreError::Backpressure(format!("{context}: {message}"))
        }
        StoreError::LimitExceeded(message) => {
            StoreError::LimitExceeded(format!("{context}: {message}"))
        }
        StoreError::Integrity(message) => StoreError::Integrity(format!("{context}: {message}")),
        StoreError::Backend(message) => StoreError::Backend(format!("{context}: {message}")),
    }
}

#[derive(Clone, Default)]
pub struct NoopSearchIndex;

#[async_trait]
impl SearchIndex for NoopSearchIndex {
    async fn index_spans(&self, _spans: &[CanonicalSpan]) -> StoreResult<()> {
        Ok(())
    }

    async fn search(&self, _query: SearchRequest) -> StoreResult<SearchResponse> {
        Ok(SearchResponse { hits: Vec::new() })
    }
}

#[derive(Clone)]
pub struct TantivySearchIndex {
    index: Index,
    fields: SearchFields,
    writer: Arc<Mutex<IndexWriter>>,
}

impl TantivySearchIndex {
    pub fn in_memory() -> anyhow::Result<Self> {
        let (schema, fields) = build_schema();
        let index = Index::create_in_ram(schema);
        Self::from_index(index, fields)
    }

    pub fn open_or_create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        std::fs::create_dir_all(path)
            .with_context(|| format!("create search index dir {}", path.display()))?;
        let (schema, fields) = build_schema();
        let index = match Index::open_in_dir(path) {
            Ok(index) => index,
            Err(_) => Index::create_in_dir(path, schema)
                .with_context(|| format!("create tantivy search index in {}", path.display()))?,
        };
        Self::from_index(index, fields)
    }

    fn from_index(index: Index, fields: SearchFields) -> anyhow::Result<Self> {
        let writer = index.writer(50_000_000).context("create tantivy writer")?;
        Ok(Self {
            index,
            fields,
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}

#[async_trait]
impl SearchIndex for TantivySearchIndex {
    async fn index_spans(&self, spans: &[CanonicalSpan]) -> StoreResult<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|err| StoreError::backend(format!("search writer mutex poisoned: {err}")))?;
        for span in spans {
            let doc_key = doc_key(span);
            writer.delete_term(Term::from_field_text(self.fields.doc_key, &doc_key));
            writer
                .add_document(doc!(
                    self.fields.doc_key => doc_key,
                    self.fields.tenant_id => span.tenant_id.as_str(),
                    self.fields.project_id => span.project_id.as_str(),
                    self.fields.environment_id => span.environment_id.as_str(),
                    self.fields.trace_id => span.trace_id.as_str(),
                    self.fields.span_id => span.span_id.as_str(),
                    self.fields.kind => span.kind.as_str(),
                    self.fields.status => span.status.as_str(),
                    self.fields.name => span.name.as_str(),
                    self.fields.model => model_text(span),
                    self.fields.tool => tool_text(span).unwrap_or_default(),
                    self.fields.input_body => input_body_text(span),
                    self.fields.output_body => output_body_text(span),
                    self.fields.error => error_text(span),
                    self.fields.text => searchable_text(span),
                ))
                .map_err(StoreError::backend)?;
        }
        writer
            .commit()
            .context("commit search index")
            .into_store()?;
        Ok(())
    }

    async fn search(&self, query: SearchRequest) -> StoreResult<SearchResponse> {
        let reader = self
            .index
            .reader()
            .context("open search reader")
            .into_store()?;
        let searcher = reader.searcher();
        let parsed = self.filtered_query(&query)?;
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let top_docs = searcher
            .search(
                parsed.as_ref(),
                &TopDocs::with_limit(limit as usize).order_by_score(),
            )
            .context("search tantivy index")
            .into_store()?;

        let mut hits = Vec::new();
        for (score, address) in top_docs {
            let doc = searcher
                .doc::<TantivyDocument>(address)
                .context("load search hit document")
                .into_store()?;
            let hit = SearchHit {
                score,
                tenant_id: text_field(&doc, self.fields.tenant_id).into_store()?,
                project_id: text_field(&doc, self.fields.project_id).into_store()?,
                environment_id: text_field(&doc, self.fields.environment_id).into_store()?,
                trace_id: text_field(&doc, self.fields.trace_id).into_store()?,
                span_id: text_field(&doc, self.fields.span_id).into_store()?,
                kind: text_field(&doc, self.fields.kind).into_store()?,
                status: text_field(&doc, self.fields.status).into_store()?,
                name: text_field(&doc, self.fields.name).into_store()?,
                model: text_field(&doc, self.fields.model).into_store()?,
                tool: text_field(&doc, self.fields.tool).into_store()?,
            };
            hits.push(hit);
        }
        Ok(SearchResponse { hits })
    }
}

/// Maximum byte length of a user-supplied query string.  Strings longer than
/// this are rejected before reaching the Tantivy parser, preventing slow or
/// memory-hungry tokenisation passes on adversarially large inputs.
const MAX_QUERY_LEN: usize = 1_000;

impl TantivySearchIndex {
    fn filtered_query(&self, query: &SearchRequest) -> StoreResult<Box<dyn Query>> {
        ensure_query_len("search query", &query.text)?;

        let mut clauses: Vec<(Occur, Box<dyn Query>)> = vec![(
            Occur::Must,
            exact_field_query(self.fields.tenant_id, query.tenant_id.as_str()),
        )];
        if query.text.trim().is_empty() {
            clauses.push((Occur::Must, Box::new(AllQuery)));
        } else {
            clauses.push((
                Occur::Must,
                self.literal_any_text_query(
                    &[
                        self.fields.text,
                        self.fields.name,
                        self.fields.model,
                        self.fields.tool,
                    ],
                    &query.text,
                )?,
            ));
        }
        if let Some(project_id) = &query.project_id {
            clauses.push((
                Occur::Must,
                exact_field_query(self.fields.project_id, project_id.as_str()),
            ));
        }
        if let Some(environment_id) = &query.environment_id {
            clauses.push((
                Occur::Must,
                exact_field_query(self.fields.environment_id, environment_id),
            ));
        }
        if let Some(trace_id) = &query.trace_id {
            clauses.push((
                Occur::Must,
                exact_field_query(self.fields.trace_id, trace_id.as_str()),
            ));
        }
        if let Some(span_id) = &query.span_id {
            clauses.push((
                Occur::Must,
                exact_field_query(self.fields.span_id, span_id.as_str()),
            ));
        }
        if let Some(kind) = &query.kind {
            clauses.push((Occur::Must, exact_field_query(self.fields.kind, kind)));
        }
        if let Some(status) = &query.status {
            clauses.push((Occur::Must, exact_field_query(self.fields.status, status)));
        }
        if let Some(model) = query.model.as_ref().filter(|value| !value.is_empty()) {
            ensure_query_len("model search filter", model)?;
            clauses.push((
                Occur::Must,
                self.literal_all_text_query(self.fields.model, model)?,
            ));
        }
        if let Some(tool) = query.tool.as_ref().filter(|value| !value.is_empty()) {
            ensure_query_len("tool search filter", tool)?;
            clauses.push((
                Occur::Must,
                self.literal_all_text_query(self.fields.tool, tool)?,
            ));
        }

        if clauses.len() == 1 {
            Ok(clauses
                .pop()
                .unwrap_or_else(|| (Occur::Must, Box::new(AllQuery)))
                .1)
        } else {
            Ok(Box::new(BooleanQuery::new(clauses)))
        }
    }

    fn literal_any_text_query(&self, fields: &[Field], text: &str) -> StoreResult<Box<dyn Query>> {
        let mut clauses = Vec::new();
        for field in fields {
            for token in self.literal_tokens(*field, text)? {
                clauses.push((Occur::Should, text_term_query(*field, &token)));
            }
        }
        Ok(boolean_or_single(clauses))
    }

    fn literal_all_text_query(&self, field: Field, text: &str) -> StoreResult<Box<dyn Query>> {
        let clauses = self
            .literal_tokens(field, text)?
            .into_iter()
            .map(|token| (Occur::Must, text_term_query(field, &token)))
            .collect();
        Ok(boolean_or_single(clauses))
    }

    fn literal_tokens(&self, field: Field, text: &str) -> StoreResult<Vec<String>> {
        let mut analyzer = self
            .index
            .tokenizer_for_field(field)
            .context("load search field tokenizer")
            .into_store()?;
        let mut stream = analyzer.token_stream(text);
        let mut tokens = Vec::new();
        while stream.advance() {
            let token = stream.token().text.trim();
            if !token.is_empty() {
                tokens.push(token.to_string());
            }
        }
        Ok(tokens)
    }
}

#[derive(Clone, Copy)]
struct SearchFields {
    doc_key: tantivy::schema::Field,
    tenant_id: tantivy::schema::Field,
    project_id: tantivy::schema::Field,
    environment_id: tantivy::schema::Field,
    trace_id: tantivy::schema::Field,
    span_id: tantivy::schema::Field,
    kind: tantivy::schema::Field,
    status: tantivy::schema::Field,
    name: tantivy::schema::Field,
    model: tantivy::schema::Field,
    tool: tantivy::schema::Field,
    input_body: tantivy::schema::Field,
    output_body: tantivy::schema::Field,
    error: tantivy::schema::Field,
    text: tantivy::schema::Field,
}

fn build_schema() -> (Schema, SearchFields) {
    let mut builder = Schema::builder();
    let doc_key = builder.add_text_field("doc_key", STRING | STORED);
    let tenant_id = builder.add_text_field("tenant_id", STRING | STORED);
    let project_id = builder.add_text_field("project_id", STRING | STORED);
    let environment_id = builder.add_text_field("environment_id", STRING | STORED);
    let trace_id = builder.add_text_field("trace_id", STRING | STORED);
    let span_id = builder.add_text_field("span_id", STRING | STORED);
    let kind = builder.add_text_field("kind", STRING | STORED);
    let status = builder.add_text_field("status", STRING | STORED);
    let name = builder.add_text_field("name", TEXT | STORED);
    let model = builder.add_text_field("model", TEXT | STORED);
    let tool = builder.add_text_field("tool", TEXT | STORED);
    let input_body = builder.add_text_field("input_body", TEXT | STORED);
    let output_body = builder.add_text_field("output_body", TEXT | STORED);
    let error = builder.add_text_field("error", TEXT | STORED);
    let text = builder.add_text_field("text", TEXT | STORED);
    (
        builder.build(),
        SearchFields {
            doc_key,
            tenant_id,
            project_id,
            environment_id,
            trace_id,
            span_id,
            kind,
            status,
            name,
            model,
            tool,
            input_body,
            output_body,
            error,
            text,
        },
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRequest {
    pub tenant_id: TenantId,
    pub text: String,
    pub project_id: Option<ProjectId>,
    pub environment_id: Option<String>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub model: Option<String>,
    pub tool: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchHit {
    pub score: f32,
    pub tenant_id: String,
    pub project_id: String,
    pub environment_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub kind: String,
    pub status: String,
    pub name: String,
    pub model: String,
    pub tool: String,
}

fn text_field(doc: &TantivyDocument, field: tantivy::schema::Field) -> anyhow::Result<String> {
    doc.get_first(field)
        .and_then(|value| value.as_str().map(ToString::to_string))
        .ok_or_else(|| anyhow!("search hit missing stored text field {:?}", field))
}

fn exact_field_query(field: tantivy::schema::Field, value: &str) -> Box<dyn Query> {
    Box::new(TermQuery::new(
        Term::from_field_text(field, value),
        IndexRecordOption::Basic,
    ))
}

fn text_term_query(field: tantivy::schema::Field, value: &str) -> Box<dyn Query> {
    Box::new(TermQuery::new(
        Term::from_field_text(field, value),
        IndexRecordOption::Basic,
    ))
}

fn boolean_or_single(mut clauses: Vec<(Occur, Box<dyn Query>)>) -> Box<dyn Query> {
    match clauses.len() {
        0 => Box::new(EmptyQuery),
        1 => {
            clauses
                .pop()
                .unwrap_or_else(|| (Occur::Must, Box::new(EmptyQuery)))
                .1
        }
        _ => Box::new(BooleanQuery::new(clauses)),
    }
}

fn ensure_query_len(label: &str, value: &str) -> StoreResult<()> {
    if value.len() > MAX_QUERY_LEN {
        return Err(StoreError::backend(format!(
            "{label} too long: {} bytes (limit {MAX_QUERY_LEN})",
            value.len(),
        )));
    }
    Ok(())
}

fn doc_key(span: &CanonicalSpan) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        span.tenant_id.as_str(),
        span.project_id.as_str(),
        span.trace_id.as_str(),
        span.span_id.as_str(),
        span.seq
    )
}

fn model_text(span: &CanonicalSpan) -> String {
    span.model
        .as_ref()
        .map(|model| format!("{} {}", model.provider, model.name))
        .unwrap_or_default()
}

fn tool_text(span: &CanonicalSpan) -> Option<String> {
    span.attributes
        .get("tool.name")
        .and_then(JsonValue::as_str)
        .map(ToString::to_string)
}

const INPUT_BODY_ATTRS: &[&str] = &[
    "input.value",
    "llm.prompts",
    "llm.input_messages",
    "gen_ai.prompt",
    "gen_ai.input.messages",
    "gen_ai.system",
];

const OUTPUT_BODY_ATTRS: &[&str] = &[
    "output.value",
    "llm.completions",
    "llm.output_messages",
    "gen_ai.completion",
    "gen_ai.output.messages",
];

const ERROR_BODY_ATTRS: &[&str] = &[
    "error",
    "error.message",
    "error.type",
    "exception.message",
    "exception.type",
    "exception.stacktrace",
];

fn input_body_text(span: &CanonicalSpan) -> String {
    if !input_body_indexable(span) {
        return String::new();
    }
    canonical_body_text(span, INPUT_BODY_ATTRS)
}

fn output_body_text(span: &CanonicalSpan) -> String {
    if !output_body_indexable(span) {
        return String::new();
    }
    canonical_body_text(span, OUTPUT_BODY_ATTRS)
}

fn error_text(span: &CanonicalSpan) -> String {
    canonical_body_text(span, ERROR_BODY_ATTRS)
}

fn canonical_body_text(span: &CanonicalSpan, keys: &[&str]) -> String {
    span.attributes
        .iter()
        .filter(|(key, _)| keys.iter().any(|candidate| attr_matches(key, candidate)))
        .map(|(_, value)| value_to_text(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn attr_matches(key: &str, candidate: &str) -> bool {
    key == candidate
        || key
            .strip_prefix(candidate)
            .is_some_and(|suffix| suffix.starts_with('.') || suffix.starts_with('['))
}

fn input_body_indexable(span: &CanonicalSpan) -> bool {
    body_indexable(span, span.input_ref.as_ref())
}

fn output_body_indexable(span: &CanonicalSpan) -> bool {
    body_indexable(span, span.output_ref.as_ref())
}

fn body_indexable(span: &CanonicalSpan, body_ref: Option<&beater_schema::ArtifactRef>) -> bool {
    !is_sensitive_redaction(&span.raw_ref.redaction_class)
        && !body_ref
            .is_some_and(|artifact_ref| is_sensitive_redaction(&artifact_ref.redaction_class))
}

fn is_sensitive_redaction(redaction_class: &RedactionClass) -> bool {
    matches!(
        redaction_class,
        RedactionClass::Sensitive | RedactionClass::Secret
    )
}

fn body_attr_value_indexable(span: &CanonicalSpan, key: &str) -> bool {
    let input_body_attr = INPUT_BODY_ATTRS
        .iter()
        .any(|candidate| attr_matches(key, candidate));
    let output_body_attr = OUTPUT_BODY_ATTRS
        .iter()
        .any(|candidate| attr_matches(key, candidate));
    (!input_body_attr || input_body_indexable(span))
        && (!output_body_attr || output_body_indexable(span))
}

fn searchable_text(span: &CanonicalSpan) -> String {
    let mut pieces = vec![
        span.name.clone(),
        span.kind.as_str().to_string(),
        span.status.as_str().to_string(),
        model_text(span),
        tool_text(span).unwrap_or_default(),
        input_body_text(span),
        output_body_text(span),
        error_text(span),
    ];
    for (key, value) in &span.attributes {
        pieces.push(key.clone());
        if body_attr_value_indexable(span, key) {
            pieces.push(value_to_text(value));
        }
    }
    pieces.push(value_to_text(&span.unmapped_attrs));
    pieces.join(" ")
}

fn value_to_text(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => value.clone(),
        JsonValue::Array(values) => values
            .iter()
            .map(value_to_text)
            .collect::<Vec<_>>()
            .join(" "),
        JsonValue::Object(values) => values
            .iter()
            .flat_map(|(key, value)| [key.clone(), value_to_text(value)])
            .collect::<Vec<_>>()
            .join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{EnvironmentId, ProjectId, TenantId};
    use beater_schema::CanonicalTraceBatch;
    use beater_schema::{
        AgentSpanKind, ModelRef, RedactionClass, SpanStatus, CANONICAL_SCHEMA_VERSION,
    };
    use beater_store_memory::InMemoryTraceStore;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn tantivy_search_indexes_text_and_enforces_tenant_filters() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let other_tenant = TenantId::new("other").unwrap_or_else(|err| panic!("{err}"));
        index
            .index_spans(&[
                fixture_span(
                    &tenant,
                    "trace-a",
                    "span-a",
                    "refund tool failed",
                    SpanStatus::Error,
                ),
                fixture_span(
                    &other_tenant,
                    "trace-b",
                    "span-b",
                    "refund tool failed",
                    SpanStatus::Error,
                ),
            ])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "refund".to_string(),
                status: Some("error".to_string()),
                kind: Some("tool.call".to_string()),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "trace-a");
    }

    #[tokio::test]
    async fn structured_filters_are_applied_inside_the_search_query() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let other_project = ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}"));
        let mut spans = Vec::new();
        for index_id in 0..20 {
            spans.push(fixture_span_with_project(
                &tenant,
                &other_project,
                &format!("other-trace-{index_id}"),
                &format!("other-span-{index_id}"),
                "refund refund refund refund refund failed loudly",
                SpanStatus::Error,
            ));
        }
        spans.push(fixture_span_with_project(
            &tenant,
            &project,
            "target-trace",
            "target-span",
            "refund",
            SpanStatus::Error,
        ));
        index
            .index_spans(&spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "refund".to_string(),
                project_id: Some(project),
                limit: Some(1),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "target-trace");
        assert_eq!(response.hits[0].project_id, "project");
    }

    #[tokio::test]
    async fn tenant_filter_is_applied_inside_the_search_query() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let other_tenant = TenantId::new("other-tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let mut spans = Vec::new();
        for index_id in 0..20 {
            spans.push(fixture_span_with_project(
                &other_tenant,
                &project,
                &format!("other-tenant-trace-{index_id}"),
                &format!("other-tenant-span-{index_id}"),
                "refund refund refund refund refund failed loudly",
                SpanStatus::Error,
            ));
        }
        spans.push(fixture_span_with_project(
            &tenant,
            &project,
            "target-tenant-trace",
            "target-tenant-span",
            "refund",
            SpanStatus::Error,
        ));
        index
            .index_spans(&spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "refund".to_string(),
                limit: Some(1),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "target-tenant-trace");
        assert_eq!(response.hits[0].tenant_id, "tenant");
    }

    #[tokio::test]
    async fn kind_and_status_filters_are_applied_inside_the_search_query() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let mut spans = Vec::new();
        for index_id in 0..20 {
            let mut span = fixture_span_with_project(
                &tenant,
                &project,
                &format!("wrong-kind-trace-{index_id}"),
                &format!("wrong-kind-span-{index_id}"),
                "refund refund refund refund refund failed loudly",
                SpanStatus::Ok,
            );
            span.kind = AgentSpanKind::LlmCall;
            spans.push(span);
        }
        spans.push(fixture_span_with_project(
            &tenant,
            &project,
            "target-kind-trace",
            "target-kind-span",
            "refund",
            SpanStatus::Error,
        ));
        index
            .index_spans(&spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "refund".to_string(),
                kind: Some("tool.call".to_string()),
                status: Some("error".to_string()),
                limit: Some(1),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "target-kind-trace");
        assert_eq!(response.hits[0].kind, "tool.call");
        assert_eq!(response.hits[0].status, "error");
    }

    #[tokio::test]
    async fn trace_and_span_filters_are_applied_inside_the_search_query() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let mut spans = Vec::new();
        for index_id in 0..20 {
            spans.push(fixture_span_with_project(
                &tenant,
                &project,
                &format!("wrong-trace-{index_id}"),
                &format!("wrong-span-{index_id}"),
                "refund refund refund refund refund failed loudly",
                SpanStatus::Error,
            ));
        }
        spans.push(fixture_span_with_project(
            &tenant,
            &project,
            "target-trace-filter",
            "target-span-filter",
            "refund",
            SpanStatus::Error,
        ));
        index
            .index_spans(&spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "refund".to_string(),
                trace_id: Some(
                    TraceId::new("target-trace-filter").unwrap_or_else(|err| panic!("{err}")),
                ),
                span_id: Some(
                    SpanId::new("target-span-filter").unwrap_or_else(|err| panic!("{err}")),
                ),
                limit: Some(1),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "target-trace-filter");
        assert_eq!(response.hits[0].span_id, "target-span-filter");
    }

    #[tokio::test]
    async fn environment_model_and_tool_filters_are_applied_inside_the_search_query() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let wrong_environment = EnvironmentId::new("staging").unwrap_or_else(|err| panic!("{err}"));
        let mut spans = Vec::new();
        for index_id in 0..20 {
            let mut span = fixture_span_with_project(
                &tenant,
                &project,
                &format!("wrong-env-trace-{index_id}"),
                &format!("wrong-env-span-{index_id}"),
                "refund refund refund refund refund failed loudly",
                SpanStatus::Error,
            );
            span.environment_id = wrong_environment.clone();
            span.model = Some(ModelRef {
                provider: "openai".to_string(),
                name: "othermodel".to_string(),
            });
            span.attributes
                .insert("tool.name".to_string(), json!("other_tool"));
            spans.push(span);
        }
        let mut target = fixture_span_with_project(
            &tenant,
            &project,
            "target-env-model-tool-trace",
            "target-env-model-tool-span",
            "refund",
            SpanStatus::Error,
        );
        target.model = Some(ModelRef {
            provider: "openai".to_string(),
            name: "targetmodel".to_string(),
        });
        target
            .attributes
            .insert("tool.name".to_string(), json!("target_tool"));
        spans.push(target);
        index
            .index_spans(&spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "refund".to_string(),
                environment_id: Some("prod".to_string()),
                model: Some("targetmodel".to_string()),
                tool: Some("target_tool".to_string()),
                limit: Some(1),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "target-env-model-tool-trace");
        assert!(response.hits[0].model.contains("targetmodel"));
        assert_eq!(response.hits[0].tool, "target_tool");
    }

    #[tokio::test]
    async fn fielded_body_search_indexes_inline_prompt_output_and_error_attrs() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let mut prompt_span = fixture_span_with_project(
            &tenant,
            &project,
            "prompt-trace",
            "prompt-span",
            "chat completion",
            SpanStatus::Ok,
        );
        prompt_span.attributes.insert(
            "llm.input_messages".to_string(),
            json!([
                { "role": "system", "content": "Follow the invoicequake policy" },
                { "role": "user", "content": "Can this refund be approved?" }
            ]),
        );

        let mut output_span = fixture_span_with_project(
            &tenant,
            &project,
            "output-trace",
            "output-span",
            "chat completion",
            SpanStatus::Ok,
        );
        output_span.attributes.insert(
            "gen_ai.completion".to_string(),
            json!("shipmentcalc approved"),
        );

        let mut error_span = fixture_span_with_project(
            &tenant,
            &project,
            "error-trace",
            "error-span",
            "tool failed",
            SpanStatus::Error,
        );
        error_span.attributes.insert(
            "exception.message".to_string(),
            json!("cardboom gateway timeout"),
        );

        index
            .index_spans(&[prompt_span, output_span, error_span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // Inline body attributes flow into the combined `text` field via
        // `searchable_text`, so they are findable by their literal content.
        // #125 tokenizes the query literally, so field-DSL prefixes like
        // `input_body:` are no longer interpreted as field selectors; we search
        // by the distinctive body terms directly.
        let prompt = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "invoicequake".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let output = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "shipmentcalc".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let error = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "cardboom".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(prompt.hits.len(), 1);
        assert_eq!(prompt.hits[0].trace_id, "prompt-trace");
        assert_eq!(output.hits.len(), 1);
        assert_eq!(output.hits[0].trace_id, "output-trace");
        assert_eq!(error.hits.len(), 1);
        assert_eq!(error.hits[0].trace_id, "error-trace");
    }

    #[tokio::test]
    async fn redacted_body_attrs_are_not_indexed_as_search_text() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut span = fixture_span(
            &tenant,
            "redacted-trace",
            "redacted-span",
            "visibleanchor chat completion",
            SpanStatus::Ok,
        );
        span.raw_ref.redaction_class = RedactionClass::Sensitive;
        span.attributes.insert(
            "llm.input_messages".to_string(),
            json!([{ "role": "user", "content": "secretinputquake" }]),
        );
        span.attributes
            .insert("gen_ai.completion".to_string(), json!("secretoutputquake"));

        index
            .index_spans(&[span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let input = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "secretinputquake".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let output = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "secretoutputquake".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let metadata = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "visibleanchor".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(input.hits.is_empty());
        assert!(output.hits.is_empty());
        assert_eq!(metadata.hits.len(), 1);
        assert_eq!(metadata.hits[0].trace_id, "redacted-trace");
    }

    #[test]
    fn body_text_extractors_ignore_artifact_refs_and_use_inline_attrs_only() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut span = fixture_span(&tenant, "trace", "span", "chat completion", SpanStatus::Ok);
        span.input_ref = Some(span.raw_ref.clone());
        span.output_ref = Some(span.raw_ref.clone());
        span.attributes = BTreeMap::from([
            ("input.value".to_string(), json!("inline prompt body")),
            ("output.value".to_string(), json!("inline output body")),
            ("error.message".to_string(), json!("inline error body")),
        ]);

        assert_eq!(input_body_text(&span), "inline prompt body");
        assert_eq!(output_body_text(&span), "inline output body");
        assert_eq!(error_text(&span), "inline error body");
    }

    #[test]
    fn body_text_extractors_skip_sensitive_body_refs() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut span = fixture_span(&tenant, "trace", "span", "chat completion", SpanStatus::Ok);
        let mut sensitive_ref = span.raw_ref.clone();
        sensitive_ref.redaction_class = RedactionClass::Secret;
        span.input_ref = Some(sensitive_ref.clone());
        span.output_ref = Some(sensitive_ref);
        span.attributes = BTreeMap::from([
            ("input.value".to_string(), json!("inline prompt body")),
            ("output.value".to_string(), json!("inline output body")),
            ("error.message".to_string(), json!("inline error body")),
        ]);

        assert_eq!(input_body_text(&span), "");
        assert_eq!(output_body_text(&span), "");
        assert_eq!(error_text(&span), "inline error body");
    }

    #[tokio::test]
    async fn reindex_replaces_existing_span_document() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut span = fixture_span(&tenant, "trace", "span", "first message", SpanStatus::Ok);
        index
            .index_spans(&[span.clone()])
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        span.name = "second message".to_string();
        index
            .index_spans(&[span])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let old = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "first".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let new = index
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "second".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(old.hits.is_empty());
        assert_eq!(new.hits.len(), 1);
    }

    // ── query-hardening regression tests ──────────────────────────────────────

    /// An unbalanced double-quote is a DSL metacharacter. User text is
    /// tokenized literally, so it must not reach Tantivy's query parser.
    #[tokio::test]
    async fn metacharacter_query_is_tokenized_without_parse_error() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        index
            .index_spans(&[fixture_span(
                &tenant,
                "trace-quoted",
                "span-quoted",
                "unbalanced quote marker",
                SpanStatus::Ok,
            )])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                text: "\"unbalanced".to_string(),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "trace-quoted");
    }

    /// A raw `"` character is punctuation-only input. It should produce an
    /// empty literal term query, not a parse error and not an all-docs query.
    #[tokio::test]
    async fn punctuation_only_query_matches_no_rows_without_parse_error() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        index
            .index_spans(&[fixture_span(
                &tenant,
                "trace-punctuation",
                "span-punctuation",
                "quote marker",
                SpanStatus::Ok,
            )])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                text: "\"".to_string(),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(response.hits.is_empty());
    }

    /// Prefix/wildcard syntax is not executed. The tokenizer extracts `ref`,
    /// which does not match the indexed `refund` token.
    #[tokio::test]
    async fn wildcard_syntax_is_not_executed() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        index
            .index_spans(&[fixture_span(
                &tenant,
                "trace-refund",
                "span-refund",
                "refund issued",
                SpanStatus::Ok,
            )])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                text: "ref*".to_string(),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(response.hits.is_empty());
    }

    /// Model/tool filters are literal analyzed terms. Boolean words like `OR`
    /// cannot broaden a structured filter into multiple alternatives.
    #[tokio::test]
    async fn model_filter_does_not_execute_boolean_syntax() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let mut target = fixture_span(
            &tenant,
            "trace-target-model",
            "span-target-model",
            "target model span",
            SpanStatus::Ok,
        );
        target.model = Some(ModelRef {
            provider: "openai".to_string(),
            name: "targetmodel".to_string(),
        });
        let mut other = fixture_span(
            &tenant,
            "trace-other-model",
            "span-other-model",
            "other model span",
            SpanStatus::Ok,
        );
        other.model = Some(ModelRef {
            provider: "openai".to_string(),
            name: "othermodel".to_string(),
        });
        index
            .index_spans(&[target, other])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                model: Some("targetmodel OR othermodel".to_string()),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(response.hits.is_empty());
    }

    /// A query string longer than MAX_QUERY_LEN bytes is rejected before it
    /// reaches the Tantivy parser, preventing slow tokenisation passes.
    #[tokio::test]
    async fn oversized_query_returns_error() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let long_query = "a ".repeat(MAX_QUERY_LEN + 1);
        let result = index
            .search(SearchRequest {
                text: long_query,
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await;
        let Err(err) = result else {
            panic!("expected error for oversized query, got ok");
        };
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("too long"),
            "error should mention 'too long', got: {err_msg}"
        );
    }

    /// The per-request result limit is clamped to at most 200 regardless of
    /// what the caller supplies.  Index more than 200 docs and request 9999;
    /// the response must cap at 200.
    #[tokio::test]
    async fn result_limit_is_clamped_to_200() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let spans: Vec<_> = (0..250)
            .map(|i| {
                fixture_span(
                    &tenant,
                    &format!("trace-{i}"),
                    &format!("span-{i}"),
                    "needle",
                    SpanStatus::Ok,
                )
            })
            .collect();
        index
            .index_spans(&spans)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                text: "needle".to_string(),
                limit: Some(9_999),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert!(
            response.hits.len() <= 200,
            "expected ≤200 hits (limit clamp), got {}",
            response.hits.len()
        );
    }

    /// A well-formed normal query succeeds as a baseline regression guard.
    #[tokio::test]
    async fn normal_query_succeeds() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        index
            .index_spans(&[fixture_span(
                &tenant,
                "trace-ok",
                "span-ok",
                "healthy span",
                SpanStatus::Ok,
            )])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = index
            .search(SearchRequest {
                text: "healthy".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "trace-ok");
    }

    // ── end query-hardening regression tests ──────────────────────────────────

    // ── H7 cross-tenant DSL field-injection regression tests ──────────────────
    //
    // These tests prove that the mandatory `Occur::Must` tenant clause in
    // `filtered_query` cannot be escaped by anything a caller places in the
    // free-text query string.  All three variants must return zero hits from
    // tenant B's documents when the search is executed as tenant A.

    /// Happy path: tenant A can see its own spans, tenant B can see its own.
    /// This baseline confirms the shared index actually has both tenants' data.
    #[tokio::test]
    async fn cross_tenant_baseline_each_tenant_sees_own_data() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_a = TenantId::new("tenant-alpha").unwrap_or_else(|err| panic!("{err}"));
        let tenant_b = TenantId::new("tenant-bravo").unwrap_or_else(|err| panic!("{err}"));

        index
            .index_spans(&[
                fixture_span(
                    &tenant_a,
                    "trace-a",
                    "span-a",
                    "alpha secret",
                    SpanStatus::Ok,
                ),
                fixture_span(
                    &tenant_b,
                    "trace-b",
                    "span-b",
                    "bravo secret",
                    SpanStatus::Ok,
                ),
            ])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let resp_a = index
            .search(SearchRequest {
                text: "secret".to_string(),
                limit: Some(50),
                ..SearchRequest::default_for_tenant(tenant_a.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let resp_b = index
            .search(SearchRequest {
                text: "secret".to_string(),
                limit: Some(50),
                ..SearchRequest::default_for_tenant(tenant_b.clone())
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // Each tenant sees exactly its own span.
        assert_eq!(
            resp_a.hits.len(),
            1,
            "tenant A should see 1 hit, got {:?}",
            resp_a.hits.iter().map(|h| &h.span_id).collect::<Vec<_>>()
        );
        assert_eq!(resp_a.hits[0].span_id, "span-a");
        assert_eq!(
            resp_b.hits.len(),
            1,
            "tenant B should see 1 hit, got {:?}",
            resp_b.hits.iter().map(|h| &h.span_id).collect::<Vec<_>>()
        );
        assert_eq!(resp_b.hits[0].span_id, "span-b");
    }

    /// Attempt 1 — direct field injection: user query string contains
    /// `tenant_id:tenant-bravo`.  Tantivy's QueryParser can reference any
    /// schema field with the `field:value` DSL syntax.  Even if Tantivy
    /// parses this clause, the outer `Occur::Must` tenant guard must prevent
    /// any tenant-B document from appearing in tenant-A's results.
    #[tokio::test]
    async fn cross_tenant_dsl_direct_field_injection_returns_no_hits() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_a = TenantId::new("tenant-alpha").unwrap_or_else(|err| panic!("{err}"));
        let tenant_b = TenantId::new("tenant-bravo").unwrap_or_else(|err| panic!("{err}"));

        index
            .index_spans(&[
                fixture_span(&tenant_a, "trace-a", "span-a", "alpha data", SpanStatus::Ok),
                fixture_span(&tenant_b, "trace-b", "span-b", "bravo data", SpanStatus::Ok),
            ])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // Attempt to reference tenant B's tenant_id field directly.
        // May return an error (field not in parser defaults) or zero hits —
        // either outcome is acceptable; returning tenant-B docs is not.
        let result = index
            .search(SearchRequest {
                text: "tenant_id:tenant-bravo".to_string(),
                limit: Some(50),
                ..SearchRequest::default_for_tenant(tenant_a.clone())
            })
            .await;

        match result {
            Err(_) => {
                // Parser rejected the injected field reference — fine.
            }
            Ok(resp) => {
                let cross_tenant_hits: Vec<_> =
                    resp.hits.iter().filter(|h| h.span_id == "span-b").collect();
                assert!(
                    cross_tenant_hits.is_empty(),
                    "SECURITY LEAK: tenant-A query with DSL injection returned tenant-B span: \
                     {:?}",
                    cross_tenant_hits
                );
            }
        }
    }

    /// Attempt 2 — OR injection: user query string is `alpha OR
    /// tenant_id:tenant-bravo`.  An attacker hopes the OR arms are evaluated
    /// without the tenant guard, leaking docs that match the second arm.
    /// The mandatory `Occur::Must` tenant clause must prevent this.
    #[tokio::test]
    async fn cross_tenant_dsl_or_injection_returns_no_tenant_b_hits() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_a = TenantId::new("tenant-alpha").unwrap_or_else(|err| panic!("{err}"));
        let tenant_b = TenantId::new("tenant-bravo").unwrap_or_else(|err| panic!("{err}"));

        index
            .index_spans(&[
                fixture_span(
                    &tenant_a,
                    "trace-a",
                    "span-a",
                    "alpha payload",
                    SpanStatus::Ok,
                ),
                fixture_span(
                    &tenant_b,
                    "trace-b",
                    "span-b",
                    "bravo payload",
                    SpanStatus::Ok,
                ),
            ])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // OR-injection: attempt to also match tenant B spans via an OR clause.
        let result = index
            .search(SearchRequest {
                text: "alpha OR tenant_id:tenant-bravo".to_string(),
                limit: Some(50),
                ..SearchRequest::default_for_tenant(tenant_a.clone())
            })
            .await;

        match result {
            Err(_) => {
                // Rejected as malformed / unknown field — acceptable.
            }
            Ok(resp) => {
                let cross_tenant_hits: Vec<_> =
                    resp.hits.iter().filter(|h| h.span_id == "span-b").collect();
                assert!(
                    cross_tenant_hits.is_empty(),
                    "SECURITY LEAK: OR-injection returned tenant-B span in tenant-A results: \
                     {:?}",
                    cross_tenant_hits
                );
                // Tenant A's own document MAY appear (the OR's left arm matches).
                // What must NOT appear is any tenant-B document.
            }
        }
    }

    /// Attempt 3 — raw field override: user query string contains the literal
    /// `tenant_id` field name but targeting tenant B, using quoted-phrase DSL
    /// so that it is unlikely to be parsed as a default-field text search.
    /// This exercises the case where the QueryParser can resolve named fields
    /// from the Tantivy schema regardless of the default-field list.
    #[tokio::test]
    async fn cross_tenant_dsl_quoted_field_injection_returns_no_tenant_b_hits() {
        let index = TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}"));
        let tenant_a = TenantId::new("tenant-alpha").unwrap_or_else(|err| panic!("{err}"));
        let tenant_b = TenantId::new("tenant-bravo").unwrap_or_else(|err| panic!("{err}"));

        index
            .index_spans(&[
                fixture_span(
                    &tenant_a,
                    "trace-a",
                    "span-a",
                    "alpha content here",
                    SpanStatus::Ok,
                ),
                fixture_span(
                    &tenant_b,
                    "trace-b",
                    "span-b",
                    "bravo content here",
                    SpanStatus::Ok,
                ),
            ])
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // Quoted phrase targeting the tenant_id field for tenant B.
        let result = index
            .search(SearchRequest {
                text: r#"tenant_id:"tenant-bravo""#.to_string(),
                limit: Some(50),
                ..SearchRequest::default_for_tenant(tenant_a.clone())
            })
            .await;

        match result {
            Err(_) => {
                // Parse error — field reference rejected. Acceptable.
            }
            Ok(resp) => {
                let cross_tenant_hits: Vec<_> =
                    resp.hits.iter().filter(|h| h.span_id == "span-b").collect();
                assert!(
                    cross_tenant_hits.is_empty(),
                    "SECURITY LEAK: quoted field injection returned tenant-B span in \
                     tenant-A results: {:?}",
                    cross_tenant_hits
                );
            }
        }
    }

    // ── end H7 cross-tenant DSL field-injection regression tests ──────────────

    #[tokio::test]
    async fn trace_ingested_processor_reads_project_trace_and_indexes_spans() {
        let traces = Arc::new(InMemoryTraceStore::new());
        let search =
            Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let span = fixture_span_with_project(
            &tenant,
            &project,
            "helper-trace",
            "helper-span",
            "shared downstream indexing",
            SpanStatus::Ok,
        );
        traces
            .write_batch(CanonicalTraceBatch {
                raw_envelopes: Vec::new(),
                spans: vec![span],
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let processor = TraceIngestedSearchProcessor::new(traces, search.clone());
        processor
            .process_trace(
                tenant.clone(),
                project,
                TraceId::new("helper-trace").unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let response = search
            .search(SearchRequest {
                tenant_id: tenant.clone(),
                text: "downstream".to_string(),
                limit: Some(10),
                ..SearchRequest::default_for_tenant(tenant)
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].trace_id, "helper-trace");
        assert_eq!(response.hits[0].span_id, "helper-span");
    }

    impl SearchRequest {
        fn default_for_tenant(tenant_id: TenantId) -> Self {
            Self {
                tenant_id,
                text: String::new(),
                project_id: None,
                environment_id: None,
                trace_id: None,
                span_id: None,
                kind: None,
                status: None,
                model: None,
                tool: None,
                limit: None,
            }
        }
    }

    fn fixture_span(
        tenant_id: &TenantId,
        trace_id: &str,
        span_id: &str,
        name: &str,
        status: SpanStatus,
    ) -> CanonicalSpan {
        fixture_span_with_project(
            tenant_id,
            &ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            trace_id,
            span_id,
            name,
            status,
        )
    }

    fn fixture_span_with_project(
        tenant_id: &TenantId,
        project_id: &ProjectId,
        trace_id: &str,
        span_id: &str,
        name: &str,
        status: SpanStatus,
    ) -> CanonicalSpan {
        CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "test".to_string(),
            tenant_id: tenant_id.clone(),
            project_id: project_id.clone(),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new(trace_id).unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::ToolCall,
            name: name.to_string(),
            status,
            start_time: Utc::now(),
            end_time: None,
            model: Some(ModelRef {
                provider: "openai".to_string(),
                name: "gpt-test".to_string(),
            }),
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref: None,
            attributes: BTreeMap::from([
                ("tool.name".to_string(), json!("refund_lookup")),
                ("input.value".to_string(), json!("customer asks for refund")),
                ("output.value".to_string(), json!("refund denied")),
            ]),
            unmapped_attrs: json!({ "error": "timeout talking to billing" }),
            raw_ref: beater_schema::ArtifactRef {
                artifact_id: beater_core::ArtifactId::new("artifact")
                    .unwrap_or_else(|err| panic!("{err}")),
                uri: "artifact://tenant/project/artifact".to_string(),
                sha256: beater_core::Sha256Hash::new("hash").unwrap_or_else(|err| panic!("{err}")),
                size_bytes: 0,
                mime_type: "application/json".to_string(),
                redaction_class: beater_schema::RedactionClass::Internal,
            },
        }
    }
}
