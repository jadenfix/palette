use async_trait::async_trait;
use beater_core::{sha256_hex, ArtifactId, ProjectId, Sha256Hash, TenantId};
use beater_schema::{ArtifactRef, RedactionClass};
use beater_store::{ArtifactStore, StoreError, StoreResult};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct FsArtifactStore {
    root: Arc<PathBuf>,
}

impl FsArtifactStore {
    pub fn new(root: impl Into<PathBuf>) -> StoreResult<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(StoreError::backend)?;
        Ok(Self {
            root: Arc::new(root),
        })
    }

    fn path_for_uri(&self, uri: &str) -> StoreResult<PathBuf> {
        let prefix = "artifact://";
        let relative = uri
            .strip_prefix(prefix)
            .ok_or_else(|| StoreError::Integrity(format!("unsupported artifact uri: {uri}")))?;
        self.resolve_within_root(relative)
    }

    /// Join `relative` onto the store root and confine the result to that root.
    ///
    /// Without this, a `..` segment, an absolute path (`/etc/passwd` — note that
    /// `PathBuf::join` *discards* the root when the joined component is
    /// absolute), or a tenant/project id containing path separators could read
    /// or write outside the artifact store. We reject `.`/`..`/empty segments
    /// and then verify, component-wise, that the joined path still starts with
    /// the root.
    fn resolve_within_root(&self, relative: &str) -> StoreResult<PathBuf> {
        if relative.is_empty() {
            return Err(StoreError::Integrity("empty artifact path".to_string()));
        }
        for segment in relative.split('/') {
            if segment == ".." || segment == "." || segment.is_empty() {
                return Err(StoreError::Integrity(format!(
                    "artifact path cannot contain '.', '..' or empty segments: {relative}"
                )));
            }
        }
        let path = self.root.join(relative);
        // `starts_with` is component-based (not a lexical string prefix), so it
        // also catches absolute `relative` values that made `join` drop the root.
        if !path.starts_with(self.root.as_ref()) {
            return Err(StoreError::Integrity(format!(
                "artifact path escapes the store root: {relative}"
            )));
        }
        Ok(path)
    }
}

#[async_trait]
impl ArtifactStore for FsArtifactStore {
    async fn put_bytes(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        mime_type: &str,
        redaction_class: RedactionClass,
        bytes: &[u8],
    ) -> StoreResult<ArtifactRef> {
        let artifact_id = ArtifactId::new(Uuid::new_v4().to_string())
            .map_err(|err| StoreError::Integrity(err.to_string()))?;
        let sha256 = Sha256Hash::new(sha256_hex(bytes))
            .map_err(|err| StoreError::Integrity(err.to_string()))?;
        let relative = format!(
            "{}/{}/{}",
            tenant_id.as_str(),
            project_id.as_str(),
            artifact_id.as_str()
        );
        // Confine the write to the store root even if a tenant/project id slips
        // through with path separators or traversal segments.
        let path = self.resolve_within_root(&relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(StoreError::backend)?;
        }
        fs::write(&path, bytes).map_err(StoreError::backend)?;

        Ok(ArtifactRef {
            artifact_id,
            uri: format!("artifact://{relative}"),
            sha256,
            size_bytes: bytes.len() as u64,
            mime_type: mime_type.to_string(),
            redaction_class,
        })
    }

    async fn get_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<Vec<u8>> {
        let path = self.path_for_uri(&artifact_ref.uri)?;
        let bytes = fs::read(&path).map_err(StoreError::backend)?;
        let actual = sha256_hex(&bytes);
        if actual != artifact_ref.sha256.as_str() {
            return Err(StoreError::Integrity(format!(
                "artifact hash mismatch for {}: expected {}, got {}",
                artifact_ref.uri,
                artifact_ref.sha256.as_str(),
                actual
            )));
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fs_artifact_store_round_trips_and_checks_hash() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let artifact = store
            .put_bytes(
                &tenant,
                &project,
                "application/json",
                RedactionClass::Sensitive,
                br#"{"ok":true}"#,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let bytes = store
            .get_bytes(&artifact)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bytes, br#"{"ok":true}"#);
    }

    #[tokio::test]
    async fn fs_artifact_store_rejects_corrupt_bytes() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let artifact = store
            .put_bytes(
                &tenant,
                &project,
                "application/json",
                RedactionClass::Sensitive,
                br#"{"ok":true}"#,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let path = store
            .path_for_uri(&artifact.uri)
            .unwrap_or_else(|err| panic!("{err}"));
        std::fs::write(path, br#"{"ok":false}"#).unwrap_or_else(|err| panic!("{err}"));

        match store.get_bytes(&artifact).await {
            Err(StoreError::Integrity(message)) => {
                assert!(message.contains("artifact hash mismatch"));
            }
            other => panic!("expected StoreError::Integrity for corrupt artifact, got {other:?}"),
        }
    }

    #[test]
    fn path_for_uri_rejects_traversal_and_absolute_paths() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));

        // Absolute path: `join` would otherwise discard the root and read /etc/passwd.
        assert!(store.path_for_uri("artifact:///etc/passwd").is_err());
        // Classic `..` traversal.
        assert!(store.path_for_uri("artifact://../../etc/passwd").is_err());
        assert!(store
            .path_for_uri("artifact://tenant/../../../etc/passwd")
            .is_err());
        // `.` segments and empty paths.
        assert!(store.path_for_uri("artifact://./secret").is_err());
        assert!(store.path_for_uri("artifact://").is_err());
        // A normal id triple still resolves inside the root.
        let ok = store
            .path_for_uri("artifact://tenant/project/abc")
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(ok.starts_with(tempdir.path()));
    }

    #[tokio::test]
    async fn put_bytes_rejects_tenant_id_with_path_separators() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        // Ids are not guaranteed to be path-safe (the JSON ingest path can carry
        // unvalidated values), so the store must reject a traversal attempt.
        let tenant = TenantId::new("../../tmp").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let result = store
            .put_bytes(
                &tenant,
                &project,
                "application/json",
                RedactionClass::Sensitive,
                br#"{"ok":true}"#,
            )
            .await;
        assert!(
            matches!(result, Err(StoreError::Integrity(_))),
            "expected traversal tenant id to be rejected, got {result:?}"
        );
    }
}
