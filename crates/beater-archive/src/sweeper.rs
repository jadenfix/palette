//! Orphaned-artifact sweeper.
//!
//! Reconciles a set of candidate artifact references against the artifacts still
//! referenced by live spans in a [`TraceStore`], deleting the bytes for any
//! candidate that is no longer reachable. This reclaims object-store space after
//! traces age out / are deleted from the hot store while their artifacts (raw
//! payloads, span inputs/outputs) linger.
//!
//! The sweep is conservative (delete only what is provably unreferenced) and
//! idempotent (delete of a missing object is a no-op), so it is safe to re-run
//! and safe to race with concurrent deletes.

use beater_core::{PageRequest, ProjectId, TenantId};
use beater_schema::{ArtifactRef, CanonicalSpan, SpanFilter};
use beater_store::{ArtifactStore, StoreResult, TraceStore};
use std::collections::BTreeSet;
use std::sync::Arc;

/// Maximum number of spans materialized per reconciliation pass. Run summaries
/// in this codebase already materialize spans through `query_spans`, so this
/// bounds memory for the sweep the same way.
const SWEEP_SCAN_LIMIT: u32 = u32::MAX;

/// Outcome of a sweep pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SweepReport {
    /// Candidate artifacts whose URI was still referenced by a live span and so
    /// were left in place.
    pub retained: usize,
    /// Candidate artifacts that were unreferenced and deleted from the store.
    pub deleted: usize,
    /// URIs of the artifacts that were deleted, in sorted order.
    pub deleted_uris: Vec<String>,
}

/// Deletes object-store artifacts that are no longer referenced by any live span.
pub struct OrphanedArtifactSweeper {
    artifacts: Arc<dyn ArtifactStore>,
}

impl OrphanedArtifactSweeper {
    pub fn new(artifacts: Arc<dyn ArtifactStore>) -> Self {
        Self { artifacts }
    }

    /// Reconciles `candidates` against the artifacts referenced by the live spans
    /// of `tenant` (optionally scoped to `project`) in `trace_store`, deleting any
    /// candidate whose URI is not referenced.
    ///
    /// `candidates` is the set of artifacts known to exist in the object store
    /// (e.g. an object-store listing). Candidates outside the requested
    /// tenant/project scope are ignored; in-scope candidates not referenced by a
    /// live span are treated as orphaned and deleted.
    pub async fn sweep(
        &self,
        trace_store: &dyn TraceStore,
        tenant: TenantId,
        project: Option<ProjectId>,
        candidates: &[ArtifactRef],
    ) -> StoreResult<SweepReport> {
        let sweep_tenant = tenant.clone();
        let sweep_project = project.clone();
        let live = self
            .live_referenced_uris(trace_store, tenant, project)
            .await?;
        self.delete_orphans(candidates, &live, &sweep_tenant, sweep_project.as_ref())
            .await
    }

    async fn live_referenced_uris(
        &self,
        trace_store: &dyn TraceStore,
        tenant: TenantId,
        project: Option<ProjectId>,
    ) -> StoreResult<BTreeSet<String>> {
        // `query_spans` returns span summaries, which intentionally drop the
        // artifact refs, so reconcile against full traces instead. We discover
        // the live trace ids via the span summaries, then load each trace's full
        // spans to collect every referenced artifact uri.
        let summaries = trace_store
            .query_spans(
                tenant.clone(),
                SpanFilter {
                    project_id: project.clone(),
                    ..SpanFilter::default()
                },
                PageRequest {
                    limit: SWEEP_SCAN_LIMIT,
                    cursor: None,
                },
            )
            .await?
            .items;

        let mut trace_ids = BTreeSet::new();
        for summary in summaries {
            trace_ids.insert(summary.trace_id);
        }

        let mut referenced = BTreeSet::new();
        for trace_id in trace_ids {
            let view = match &project {
                Some(project) => {
                    trace_store
                        .get_project_trace(tenant.clone(), project.clone(), trace_id)
                        .await?
                }
                None => trace_store.get_trace(tenant.clone(), trace_id).await?,
            };
            referenced.extend(referenced_artifact_uris(&view.spans));
        }
        Ok(referenced)
    }

    async fn delete_orphans(
        &self,
        candidates: &[ArtifactRef],
        live: &BTreeSet<String>,
        tenant: &TenantId,
        project: Option<&ProjectId>,
    ) -> StoreResult<SweepReport> {
        let mut report = SweepReport::default();
        for candidate in candidates {
            if !artifact_uri_in_scope(&candidate.uri, tenant, project) {
                continue;
            }
            if live.contains(&candidate.uri) {
                report.retained += 1;
                continue;
            }
            self.artifacts.delete_bytes(candidate).await?;
            report.deleted += 1;
            report.deleted_uris.push(candidate.uri.clone());
        }
        report.deleted_uris.sort();
        Ok(report)
    }
}

fn artifact_uri_in_scope(uri: &str, tenant: &TenantId, project: Option<&ProjectId>) -> bool {
    let Some(path) = uri.strip_prefix("artifact://") else {
        return false;
    };
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() != 3
        || segments
            .iter()
            .any(|segment| segment.is_empty() || *segment == "." || *segment == "..")
    {
        return false;
    }
    if segments[0] != tenant.as_str() {
        return false;
    }
    match project {
        Some(project) => segments[1] == project.as_str(),
        None => true,
    }
}

/// Collects every artifact uri referenced by the given spans (raw payloads plus
/// any span input/output artifacts).
fn referenced_artifact_uris(spans: &[CanonicalSpan]) -> BTreeSet<String> {
    let mut uris = BTreeSet::new();
    for span in spans {
        uris.insert(span.raw_ref.uri.clone());
        if let Some(input) = &span.input_ref {
            uris.insert(input.uri.clone());
        }
        if let Some(output) = &span.output_ref {
            uris.insert(output.uri.clone());
        }
    }
    uris
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use beater_core::{ArtifactId, Sha256Hash};
    use beater_schema::RedactionClass;
    use std::sync::Mutex;

    #[test]
    fn artifact_uri_scope_uses_exact_path_segments() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let other_tenant = TenantId::new("tenant-other").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        assert!(artifact_uri_in_scope(
            "artifact://tenant/project/artifact",
            &tenant,
            Some(&project)
        ));
        assert!(artifact_uri_in_scope(
            "artifact://tenant/other-project/artifact",
            &tenant,
            None
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/projectile/artifact",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant-other/project/artifact",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/project/artifact/extra",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/project",
            &tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/../project",
            &tenant,
            None
        ));
        assert!(!artifact_uri_in_scope(
            "https://tenant/project/artifact",
            &tenant,
            Some(&project)
        ));

        let slash_tenant = TenantId::new("tenant/slash").unwrap_or_else(|err| panic!("{err}"));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/slash/project/artifact",
            &slash_tenant,
            Some(&project)
        ));
        assert!(!artifact_uri_in_scope(
            "artifact://tenant/project/artifact",
            &other_tenant,
            None
        ));
    }

    #[tokio::test]
    async fn delete_orphans_skips_candidates_outside_sweep_scope() {
        let artifacts = Arc::new(RecordingArtifactStore::default());
        let sweeper = OrphanedArtifactSweeper::new(artifacts.clone());
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let live = BTreeSet::from([String::from("artifact://tenant/project/live")]);

        let report = sweeper
            .delete_orphans(
                &[
                    artifact("artifact://tenant/project/live"),
                    artifact("artifact://tenant/project/orphan"),
                    artifact("artifact://tenant/projectile/orphan"),
                    artifact("artifact://tenant/other-project/orphan"),
                    artifact("artifact://tenant-other/project/orphan"),
                    artifact("artifact://tenant/project/orphan/extra"),
                    artifact("artifact://tenant/project"),
                    artifact("https://tenant/project/orphan"),
                ],
                &live,
                &tenant,
                Some(&project),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.retained, 1);
        assert_eq!(report.deleted, 1);
        assert_eq!(
            report.deleted_uris,
            vec![String::from("artifact://tenant/project/orphan")]
        );
        assert_eq!(
            artifacts.deleted(),
            vec![String::from("artifact://tenant/project/orphan")]
        );
    }

    #[derive(Default)]
    struct RecordingArtifactStore {
        deleted: Mutex<Vec<String>>,
    }

    impl RecordingArtifactStore {
        fn deleted(&self) -> Vec<String> {
            self.deleted
                .lock()
                .unwrap_or_else(|err| panic!("{err}"))
                .clone()
        }
    }

    #[async_trait]
    impl ArtifactStore for RecordingArtifactStore {
        async fn put_bytes(
            &self,
            _tenant_id: &TenantId,
            _project_id: &ProjectId,
            _mime_type: &str,
            _redaction_class: RedactionClass,
            _bytes: &[u8],
        ) -> StoreResult<ArtifactRef> {
            panic!("not used")
        }

        async fn get_bytes(&self, _artifact_ref: &ArtifactRef) -> StoreResult<Vec<u8>> {
            panic!("not used")
        }

        async fn delete_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<()> {
            self.deleted
                .lock()
                .unwrap_or_else(|err| panic!("{err}"))
                .push(artifact_ref.uri.clone());
            Ok(())
        }
    }

    fn artifact(uri: &str) -> ArtifactRef {
        ArtifactRef {
            artifact_id: ArtifactId::new(uri.replace("://", "-").replace('/', "-"))
                .unwrap_or_else(|err| panic!("{err}")),
            uri: uri.to_string(),
            sha256: Sha256Hash::new("test-sha").unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 1,
            mime_type: "application/octet-stream".to_string(),
            redaction_class: RedactionClass::Internal,
        }
    }
}
