use async_trait::async_trait;
use beater_core::{sha256_hex, ArtifactId, ProjectId, Sha256Hash, TenantId};
use beater_schema::{ArtifactRef, RedactionClass};
use beater_store::{ArtifactStore, StoreError, StoreResult};
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

const TEMP_FILE_MARKER: &str = ".tmp.";

#[derive(Clone, Debug)]
pub struct FsArtifactStore {
    root: Arc<PathBuf>,
    max_bytes: Option<u64>,
}

impl FsArtifactStore {
    pub fn new(root: impl Into<PathBuf>) -> StoreResult<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(StoreError::backend)?;
        Ok(Self {
            root: Arc::new(root),
            max_bytes: None,
        })
    }

    /// Set a hard byte ceiling for future artifact writes.
    ///
    /// `FsArtifactStore::new` remains uncapped for compatibility. Callers that
    /// need resource governance can opt in with this builder without changing
    /// the [`ArtifactStore`] trait.
    pub fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }

    fn validate_size(
        &self,
        size_bytes: usize,
        redaction_class: &RedactionClass,
    ) -> StoreResult<u64> {
        let size_bytes = u64::try_from(size_bytes).map_err(|_| {
            StoreError::LimitExceeded("artifact too large to represent as u64".to_string())
        })?;
        if let Some(max_bytes) = self.max_bytes {
            if size_bytes > max_bytes {
                return Err(StoreError::LimitExceeded(format!(
                    "artifact too large: {size_bytes} > {max_bytes} bytes \
                     (redaction_class={redaction_class:?})"
                )));
            }
        }
        Ok(size_bytes)
    }

    fn artifact_id_for_bytes(bytes: &[u8]) -> StoreResult<ArtifactId> {
        ArtifactId::new(blake3::hash(bytes).to_hex().to_string())
            .map_err(|err| StoreError::Integrity(err.to_string()))
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

    fn atomic_temp_path(path: &std::path::Path) -> StoreResult<PathBuf> {
        let file_name = path.file_name().ok_or_else(|| {
            StoreError::Integrity(format!(
                "artifact path has no file name: {}",
                path.display()
            ))
        })?;
        let mut temp_name = OsString::from(".");
        temp_name.push(file_name);
        temp_name.push(TEMP_FILE_MARKER);
        temp_name.push(Uuid::new_v4().to_string());
        Ok(path.with_file_name(temp_name))
    }

    fn write_file_atomically(path: &std::path::Path, bytes: &[u8]) -> StoreResult<()> {
        let temp_path = Self::atomic_temp_path(path)?;
        let result = (|| -> std::io::Result<()> {
            let mut temp_file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)?;
            temp_file.write_all(bytes)?;
            temp_file.flush()?;
            temp_file.sync_all()?;
            drop(temp_file);

            fs::rename(&temp_path, path)?;
            sync_parent_dir_best_effort(path);
            Ok(())
        })();

        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                Err(StoreError::backend(error))
            }
        }
    }

    fn write_file_once(path: &std::path::Path, bytes: &[u8]) -> StoreResult<()> {
        match fs::read(path) {
            Ok(existing) => {
                if existing == bytes {
                    return Ok(());
                }
                return Err(StoreError::Integrity(format!(
                    "content-addressed artifact collision at {}",
                    path.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(StoreError::backend(error)),
        }

        Self::write_file_atomically(path, bytes)
    }
}

fn sync_parent_dir_best_effort(path: &std::path::Path) {
    #[cfg(unix)]
    {
        if let Some(parent) = path.parent() {
            if let Ok(parent_dir) = fs::File::open(parent) {
                let _ = parent_dir.sync_all();
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = path;
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
        let size_bytes = self.validate_size(bytes.len(), &redaction_class)?;
        let artifact_id = Self::artifact_id_for_bytes(bytes)?;
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
        Self::write_file_once(&path, bytes)?;

        Ok(ArtifactRef {
            artifact_id,
            uri: format!("artifact://{relative}"),
            sha256,
            size_bytes,
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

    async fn delete_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<()> {
        let path = self.path_for_uri(&artifact_ref.uri)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            // Already gone: treat as success so the sweeper is idempotent.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(StoreError::backend(error)),
        }
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
        assert_eq!(temp_artifact_file_count(tempdir.path()), 0);
    }

    #[tokio::test]
    async fn put_bytes_deduplicates_same_tenant_project_content() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let payload = b"repeatable prompt/tool body";

        let first = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Internal,
                payload,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let second = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Internal,
                payload,
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        let expected_id = blake3::hash(payload).to_hex().to_string();
        assert_eq!(first.artifact_id.as_str(), expected_id);
        assert_eq!(second.artifact_id.as_str(), expected_id);
        assert_eq!(first.uri, second.uri);
        assert_eq!(artifact_file_count(tempdir.path()), 1);
        assert_eq!(temp_artifact_file_count(tempdir.path()), 0);
        assert_eq!(
            store
                .get_bytes(&second)
                .await
                .unwrap_or_else(|err| panic!("{err}")),
            payload
        );
    }

    #[tokio::test]
    async fn put_bytes_commits_final_file_without_temp_artifacts() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let artifact = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Internal,
                b"atomic bytes",
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let path = store
            .path_for_uri(&artifact.uri)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            std::fs::read(&path).unwrap_or_else(|err| panic!("{err}")),
            b"atomic bytes"
        );
        assert_eq!(temp_artifact_file_count(tempdir.path()), 0);
    }

    #[tokio::test]
    async fn atomic_put_preserves_read_and_hash_validation() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let artifact = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Internal,
                b"hash checked bytes",
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            store
                .get_bytes(&artifact)
                .await
                .unwrap_or_else(|err| panic!("{err}")),
            b"hash checked bytes"
        );

        let path = store
            .path_for_uri(&artifact.uri)
            .unwrap_or_else(|err| panic!("{err}"));
        std::fs::write(path, b"tampered").unwrap_or_else(|err| panic!("{err}"));
        match store.get_bytes(&artifact).await {
            Err(StoreError::Integrity(message)) => {
                assert!(message.contains("artifact hash mismatch"));
            }
            other => panic!("expected StoreError::Integrity for corrupt artifact, got {other:?}"),
        }
        assert_eq!(temp_artifact_file_count(tempdir.path()), 0);
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

    /// `get_bytes` and `delete_bytes` both route through `path_for_uri` →
    /// `resolve_within_root`, so a caller who forges or replays an `ArtifactRef`
    /// with a malicious `uri` field (e.g. deserialised from untrusted JSON or a
    /// tampered HTTP response) must be rejected by both operations with
    /// `StoreError::Integrity`, not a silent escape to the filesystem.
    #[tokio::test]
    async fn get_and_delete_bytes_reject_forged_malicious_uris() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path()).unwrap_or_else(|err| panic!("{err}"));

        let make_ref = |uri: &str| ArtifactRef {
            artifact_id: ArtifactId::new("abc").unwrap_or_else(|err| panic!("{err}")),
            uri: uri.to_string(),
            // SHA-256 of empty input — irrelevant since the guard fires before
            // any read is attempted.
            sha256: Sha256Hash::new(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            )
            .unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 0,
            mime_type: "application/octet-stream".to_string(),
            redaction_class: RedactionClass::Internal,
        };

        let cases: &[&str] = &[
            // Classic double-dot traversal.
            "artifact://../../etc/passwd",
            // Traversal embedded after a valid-looking prefix segment.
            "artifact://tenant/../../../etc/passwd",
            // Absolute path — PathBuf::join discards the root when the joined
            // component is absolute; the guard must catch this before the join.
            "artifact:///etc/passwd",
            // Dot segment.
            "artifact://./secret",
            // Empty relative path.
            "artifact://",
        ];

        for uri in cases {
            match store.get_bytes(&make_ref(uri)).await {
                Err(StoreError::Integrity(_)) => {}
                other => {
                    panic!("get_bytes: expected StoreError::Integrity for {uri:?}, got {other:?}")
                }
            }
            match store.delete_bytes(&make_ref(uri)).await {
                Err(StoreError::Integrity(_)) => {}
                other => panic!(
                    "delete_bytes: expected StoreError::Integrity for {uri:?}, got {other:?}"
                ),
            }
        }

        // A well-formed URI must pass the guard (even though the file doesn't
        // exist — the guard passes and the OS returns a backend error).
        let valid = make_ref("artifact://tenant/project/abc123");
        assert!(
            matches!(store.get_bytes(&valid).await, Err(StoreError::Backend(_))),
            "expected a backend (not-found) error for a valid URI, not an integrity error"
        );
    }

    #[tokio::test]
    async fn put_bytes_accepts_artifact_at_configured_size_cap() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path())
            .unwrap_or_else(|err| panic!("{err}"))
            .with_max_bytes(4);
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let artifact = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Internal,
                b"1234",
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(artifact.size_bytes, 4);
        let bytes = store
            .get_bytes(&artifact)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bytes, b"1234");
    }

    #[tokio::test]
    async fn put_bytes_rejects_artifact_over_configured_size_cap_before_writing() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = FsArtifactStore::new(tempdir.path())
            .unwrap_or_else(|err| panic!("{err}"))
            .with_max_bytes(4);
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let result = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Sensitive,
                b"12345",
            )
            .await;

        match result {
            Err(StoreError::LimitExceeded(message)) => {
                assert!(message.contains("artifact too large: 5 > 4 bytes"));
                assert!(message.contains("redaction_class=Sensitive"));
            }
            other => {
                panic!("expected StoreError::LimitExceeded for oversized artifact, got {other:?}")
            }
        }
        assert_eq!(artifact_file_count(tempdir.path()), 0);
    }

    #[tokio::test]
    async fn fs_artifact_store_deletes_and_is_idempotent() {
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

        // The object is readable before deletion.
        store
            .get_bytes(&artifact)
            .await
            .unwrap_or_else(|err| panic!("{err}"));

        // Delete removes the backing bytes.
        store
            .delete_bytes(&artifact)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        match store.get_bytes(&artifact).await {
            Err(StoreError::Backend(_)) => {}
            other => panic!("expected backend error reading a deleted artifact, got {other:?}"),
        }

        // Deleting again is a no-op success (idempotent sweeper safety).
        store
            .delete_bytes(&artifact)
            .await
            .unwrap_or_else(|err| panic!("deleting a missing artifact must succeed: {err}"));
    }

    fn artifact_file_count(root: &std::path::Path) -> usize {
        std::fs::read_dir(root)
            .unwrap_or_else(|err| panic!("{err}"))
            .map(|entry| {
                let entry = entry.unwrap_or_else(|err| panic!("{err}"));
                let path = entry.path();
                let file_type = entry.file_type().unwrap_or_else(|err| panic!("{err}"));
                if file_type.is_dir() {
                    artifact_file_count(&path)
                } else if file_type.is_file() {
                    1
                } else {
                    0
                }
            })
            .sum()
    }

    fn temp_artifact_file_count(root: &std::path::Path) -> usize {
        std::fs::read_dir(root)
            .unwrap_or_else(|err| panic!("{err}"))
            .map(|entry| {
                let entry = entry.unwrap_or_else(|err| panic!("{err}"));
                let path = entry.path();
                let file_type = entry.file_type().unwrap_or_else(|err| panic!("{err}"));
                if file_type.is_dir() {
                    temp_artifact_file_count(&path)
                } else if file_type.is_file()
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().contains(TEMP_FILE_MARKER))
                {
                    1
                } else {
                    0
                }
            })
            .sum()
    }
}
