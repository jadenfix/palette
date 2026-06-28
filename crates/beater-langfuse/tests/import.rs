//! End-to-end import through the shared ingest dispatch: register the
//! [`LangfuseImporter`] on a real [`IngestService`] and drive it via
//! `import_source("langfuse", …)` — the exact entrypoint the `POST /v1/import`
//! HTTP handler calls. Asserts canonical spans land in the trace store and the
//! immutable raw envelope is a lossless copy of the original Langfuse document.

use std::sync::Arc;

use beater_bus::{DurableBus, InMemoryBus};
use beater_core::{EnvironmentId, PageRequest, ProjectId, TenantId, TenantScope};
use beater_ingest::{IngestPolicy, IngestService};
use beater_langfuse::LangfuseImporter;
use beater_schema::{conventions::attr, AgentSpanKind, RunFilter};
use beater_store::{ArtifactStore, TraceStore};
use beater_store_memory::InMemoryTraceStore;
use beater_store_obj::FsArtifactStore;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/langfuse_trace_export.json");

#[tokio::test]
async fn import_langfuse_export_end_to_end() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let artifacts: Arc<dyn ArtifactStore> =
        Arc::new(FsArtifactStore::new(tempdir.path()).unwrap_or_else(|e| panic!("artifacts: {e}")));
    let traces: Arc<dyn TraceStore> = Arc::new(InMemoryTraceStore::new());
    let bus: Arc<dyn DurableBus> = Arc::new(InMemoryBus::new(1024));

    let ingest = IngestService::new(
        artifacts.clone(),
        traces.clone(),
        bus,
        IngestPolicy::default(),
    )
    .with_importer(Arc::new(LangfuseImporter));

    assert!(ingest.registered_import_sources().contains(&"langfuse"));

    let scope = TenantScope::new(
        TenantId::new("acme").unwrap_or_else(|e| panic!("{e}")),
        ProjectId::new("support").unwrap_or_else(|e| panic!("{e}")),
        EnvironmentId::new("prod").unwrap_or_else(|e| panic!("{e}")),
    );

    let outcome = ingest
        .import_source(
            "langfuse",
            scope.clone(),
            FIXTURE.as_bytes().to_vec(),
            None,
            false,
        )
        .await
        .unwrap_or_else(|e| panic!("import_source: {e}"));
    // root + 4 observations.
    assert_eq!(outcome.ack.accepted_spans, 5);

    let trace = traces
        .get_project_trace(
            scope.tenant_id.clone(),
            scope.project_id.clone(),
            beater_core::TraceId::new("trace-001").unwrap_or_else(|e| panic!("{e}")),
        )
        .await
        .unwrap_or_else(|e| panic!("get_project_trace: {e}"));
    assert_eq!(trace.spans.len(), 5);

    let find = |span_id: &str| {
        trace
            .spans
            .iter()
            .find(|s| s.span_id.as_str() == span_id)
            .unwrap_or_else(|| panic!("missing span {span_id}"))
            .clone()
    };

    let root = find("lf-trace-trace-001");
    assert_eq!(root.kind, AgentSpanKind::AgentRun);
    assert_eq!(root.parent_span_id, None);
    assert_eq!(
        root.attributes.get("langfuse.release"),
        Some(&json!("v1.4.2"))
    );
    assert_eq!(
        root.attributes.get(attr::RELEASE_ID),
        Some(&json!("v1.4.2"))
    );

    let generation = find("lf-obs-obs-gen-1");
    assert_eq!(generation.kind, AgentSpanKind::LlmCall);
    assert_eq!(
        generation.model.as_ref().map(|m| m.name.as_str()),
        Some("gpt-4o")
    );
    assert_eq!(generation.tokens.map(|t| t.input), Some(120));
    assert_eq!(
        generation.parent_span_id.as_ref().map(|s| s.as_str()),
        Some("lf-obs-obs-span-retrieve")
    );

    assert_eq!(
        find("lf-obs-obs-span-weather").kind,
        AgentSpanKind::ToolCall
    );
    assert_eq!(
        find("lf-obs-obs-event-cache").kind,
        AgentSpanKind::AgentStep
    );

    // Lossless guarantee: the stored raw envelope re-parses to the exact original
    // Langfuse document, so projections can be re-derived as conventions evolve.
    let raw_bytes = artifacts
        .get_bytes(&root.raw_ref)
        .await
        .unwrap_or_else(|e| panic!("get_bytes: {e}"));
    let from_raw: Value = serde_json::from_slice(&raw_bytes).unwrap_or_else(|e| panic!("{e}"));
    let from_src: Value = serde_json::from_str(FIXTURE).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(from_raw, from_src);

    let release_runs = traces
        .query_runs(
            scope.tenant_id.clone(),
            RunFilter {
                project_id: Some(scope.project_id.clone()),
                environment_id: Some(scope.environment_id.clone()),
                release: Some("v1.4.2".to_string()),
                ..RunFilter::default()
            },
            PageRequest::default(),
        )
        .await
        .unwrap_or_else(|e| panic!("query_runs release filter: {e}"));
    assert_eq!(release_runs.items.len(), 1);
    assert_eq!(
        release_runs.items[0].release_ids,
        vec!["v1.4.2".to_string()]
    );
}
