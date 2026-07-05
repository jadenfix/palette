//! Integration test for the orphaned-artifact sweeper.
//!
//! Wires a real SQLite `TraceStore` to a real filesystem `ArtifactStore`: writes
//! a trace whose spans reference some artifacts, puts both referenced and
//! orphaned artifacts in the object store, then sweeps and asserts only the
//! orphans are deleted while referenced artifacts survive.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use beater_archive::{OrphanedArtifactSweeper, SweepConfig};
use beater_core::{EnvironmentId, IdempotencyKey, ProjectId, SpanId, TenantId, TraceId};
use beater_schema::{
    AgentSpanKind, ArtifactRef, AuthContext, CanonicalSpan, CanonicalTraceBatch, RawEnvelope,
    RedactionClass, SourceDialect, SpanStatus, CANONICAL_SCHEMA_VERSION, RAW_SCHEMA_VERSION,
};
use beater_store::{ArtifactStore, TraceStore};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use chrono::{TimeZone, Utc};
use serde_json::json;

#[tokio::test]
async fn sweeps_orphaned_artifacts_against_live_spans() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts =
        Arc::new(FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}")));
    let trace_store = SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}"));

    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

    // Three real artifacts: raw payload + span output are referenced by the live
    // trace; the third is an orphan with no referencing span.
    let raw_ref = put(&artifacts, &tenant, &project, b"raw-payload").await;
    let output_ref = put(&artifacts, &tenant, &project, b"span-output").await;
    let orphan_ref = put(&artifacts, &tenant, &project, b"orphaned-bytes").await;

    // Write a trace whose span references raw_ref + output_ref but not orphan_ref.
    let raw = raw_envelope(&tenant, &project, raw_ref.clone());
    let span = canonical_span(&tenant, &project, raw_ref.clone(), Some(output_ref.clone()));
    trace_store
        .write_batch(Arc::new(CanonicalTraceBatch::one(raw, span)))
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    // Sanity: all three artifacts are readable before the sweep.
    for artifact in [&raw_ref, &output_ref, &orphan_ref] {
        artifacts
            .get_bytes(artifact)
            .await
            .unwrap_or_else(|err| panic!("artifact should exist before sweep: {err}"));
    }

    let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());
    let candidates = vec![raw_ref.clone(), output_ref.clone(), orphan_ref.clone()];
    let report = sweeper
        .sweep_slice(
            &trace_store,
            tenant.clone(),
            Some(project.clone()),
            &candidates,
            SweepConfig::default(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(
        report.metrics.retained, 2,
        "raw + output refs are still referenced"
    );
    assert_eq!(report.metrics.deleted, 1, "only the orphan is deleted");
    assert_eq!(report.deleted_uris, vec![orphan_ref.uri.clone()]);

    // Referenced artifacts survive.
    artifacts
        .get_bytes(&raw_ref)
        .await
        .unwrap_or_else(|err| panic!("referenced raw artifact must survive: {err}"));
    artifacts
        .get_bytes(&output_ref)
        .await
        .unwrap_or_else(|err| panic!("referenced output artifact must survive: {err}"));

    // The orphan's bytes are gone.
    match artifacts.get_bytes(&orphan_ref).await {
        Err(_) => {}
        Ok(_) => panic!("orphaned artifact should have been deleted"),
    }

    // Re-running the sweep is idempotent: the orphan is already gone, so nothing
    // new is deleted and the referenced artifacts are still retained.
    let second = sweeper
        .sweep_slice(
            &trace_store,
            tenant,
            Some(project),
            &candidates,
            SweepConfig::default(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(second.metrics.retained, 2);
    assert_eq!(
        second.metrics.deleted, 1,
        "delete of a missing object is a no-op success"
    );
}

#[tokio::test]
async fn project_scoped_sweep_ignores_out_of_scope_candidates() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts =
        Arc::new(FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}")));
    let trace_store = SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}"));

    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let other_project = ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}"));
    let in_scope_orphan = put(&artifacts, &tenant, &project, b"in-scope-orphan").await;
    let out_of_scope = put(&artifacts, &tenant, &other_project, b"other-project").await;

    let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());
    let candidates = vec![in_scope_orphan.clone(), out_of_scope.clone()];
    let report = sweeper
        .sweep_slice(
            &trace_store,
            tenant.clone(),
            Some(project.clone()),
            &candidates,
            SweepConfig::default(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(report.metrics.retained, 0);
    assert_eq!(report.metrics.deleted, 1);
    assert_eq!(report.deleted_uris, vec![in_scope_orphan.uri.clone()]);

    match artifacts.get_bytes(&in_scope_orphan).await {
        Err(_) => {}
        Ok(_) => panic!("in-scope orphan should have been deleted"),
    }
    artifacts
        .get_bytes(&out_of_scope)
        .await
        .unwrap_or_else(|err| panic!("out-of-scope candidate must survive: {err}"));
}

async fn put(
    store: &FsArtifactStore,
    tenant: &TenantId,
    project: &ProjectId,
    bytes: &[u8],
) -> ArtifactRef {
    store
        .put_bytes(
            tenant,
            project,
            "application/json",
            RedactionClass::Internal,
            bytes,
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"))
}

fn raw_envelope(tenant: &TenantId, project: &ProjectId, body_ref: ArtifactRef) -> RawEnvelope {
    RawEnvelope {
        schema_version: RAW_SCHEMA_VERSION,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        source: SourceDialect::Native,
        source_schema_url: None,
        source_schema_version: None,
        received_at: Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp")),
        idempotency_key: IdempotencyKey::new("tenant:project:trace:raw")
            .unwrap_or_else(|err| panic!("{err}")),
        payload_hash: body_ref.sha256.clone(),
        body_ref,
        auth_context: AuthContext {
            api_key_id: None,
            scopes: BTreeSet::new(),
        },
    }
}

fn canonical_span(
    tenant: &TenantId,
    project: &ProjectId,
    raw_ref: ArtifactRef,
    output_ref: Option<ArtifactRef>,
) -> CanonicalSpan {
    CanonicalSpan {
        schema_version: CANONICAL_SCHEMA_VERSION,
        normalizer_version: "beater-native-v1".to_string(),
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
        span_id: SpanId::new("root").unwrap_or_else(|err| panic!("{err}")),
        parent_span_id: None,
        seq: 1,
        kind: AgentSpanKind::AgentRun,
        name: "run".to_string(),
        status: SpanStatus::Ok,
        start_time: Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 1)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp")),
        end_time: None,
        model: None,
        cost: None,
        tokens: None,
        input_ref: None,
        output_ref,
        attributes: BTreeMap::new(),
        unmapped_attrs: json!({}),
        raw_ref,
        sampling_weight: None,
    }
}
