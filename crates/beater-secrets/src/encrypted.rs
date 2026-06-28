use super::{
    IntoStoreResult, ProviderSecret, ProviderSecretMetadata, ProviderSecretStore,
    PutProviderSecretRequest, RevokedProviderSecret,
};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use beater_core::{ProjectId, ProviderSecretId, TenantId, Timestamp};
use beater_store::{StoreError, StoreResult};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chrono::{DateTime, Utc};
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 12;

#[derive(Clone, PartialEq, Eq)]
pub struct SecretEncryptionKey {
    key_id: String,
    bytes: [u8; KEY_BYTES],
}

impl SecretEncryptionKey {
    pub fn new(key_id: impl Into<String>, bytes: [u8; KEY_BYTES]) -> anyhow::Result<Self> {
        let key_id = key_id.into();
        if key_id.trim().is_empty() || key_id.chars().any(char::is_whitespace) {
            return Err(anyhow!(
                "secret encryption key id must be non-empty and whitespace-free"
            ));
        }
        Ok(Self { key_id, bytes })
    }

    pub fn generate(key_id: impl Into<String>) -> anyhow::Result<Self> {
        let mut bytes = [0_u8; KEY_BYTES];
        OsRng.fill_bytes(&mut bytes);
        Self::new(key_id, bytes)
    }

    pub fn from_base64(key_id: impl Into<String>, encoded: &str) -> anyhow::Result<Self> {
        let decoded = STANDARD_NO_PAD
            .decode(encoded.trim())
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(encoded.trim()))
            .context("decode provider secret encryption key")?;
        let bytes: [u8; KEY_BYTES] = decoded
            .try_into()
            .map_err(|_| anyhow!("provider secret encryption key must be 32 bytes"))?;
        Self::new(key_id, bytes)
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn to_base64(&self) -> String {
        STANDARD_NO_PAD.encode(self.bytes)
    }
}

impl Debug for SecretEncryptionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretEncryptionKey")
            .field("key_id", &self.key_id)
            .field("bytes", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct SecretKeyring {
    active_key_id: String,
    keys: Arc<BTreeMap<String, SecretEncryptionKey>>,
}

impl SecretKeyring {
    pub fn single(key: SecretEncryptionKey) -> Self {
        let active_key_id = key.key_id.clone();
        let mut keys = BTreeMap::new();
        keys.insert(active_key_id.clone(), key);
        Self {
            active_key_id,
            keys: Arc::new(keys),
        }
    }

    /// Construct a keyring from multiple keys, naming one as the active (write)
    /// key. Retired keys remain available for decrypting ciphertext written under
    /// them, which is what makes an online key rotation possible: callers add the
    /// new key as `active_key_id` while keeping the old keys so existing rows can
    /// still be read (and re-wrapped) until they are migrated.
    pub fn with_keys(
        active_key_id: impl Into<String>,
        keys: impl IntoIterator<Item = SecretEncryptionKey>,
    ) -> anyhow::Result<Self> {
        let active_key_id = active_key_id.into();
        let mut map = BTreeMap::new();
        for key in keys {
            let key_id = key.key_id.clone();
            if map.insert(key_id.clone(), key).is_some() {
                return Err(anyhow!(
                    "secret keyring contains a duplicate key id {key_id}"
                ));
            }
        }
        if !map.contains_key(&active_key_id) {
            return Err(anyhow!(
                "secret keyring active key id {active_key_id} is not present in the supplied keys"
            ));
        }
        Ok(Self {
            active_key_id,
            keys: Arc::new(map),
        })
    }

    /// The id of the key new ciphertext is wrapped under.
    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }

    pub fn generated_for_tests() -> anyhow::Result<Self> {
        Ok(Self::single(SecretEncryptionKey::generate("test-v1")?))
    }

    pub fn from_base64(key_id: impl Into<String>, encoded: &str) -> anyhow::Result<Self> {
        Ok(Self::single(SecretEncryptionKey::from_base64(
            key_id, encoded,
        )?))
    }

    pub fn load_or_create_local_file(
        path: impl AsRef<Path>,
        key_id: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let key_id = key_id.into();
        if path.exists() {
            let encoded = fs::read_to_string(path)
                .with_context(|| format!("read provider secret key file {}", path.display()))?;
            return Self::from_base64(key_id, &encoded);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create provider secret key dir {}", parent.display()))?;
        }
        let key = SecretEncryptionKey::generate(key_id)?;
        write_new_key_file(path, &key.to_base64())?;
        Ok(Self::single(key))
    }

    fn active_key(&self) -> anyhow::Result<&SecretEncryptionKey> {
        self.keys
            .get(&self.active_key_id)
            .ok_or_else(|| anyhow!("active provider secret encryption key is missing"))
    }

    fn key(&self, key_id: &str) -> anyhow::Result<&SecretEncryptionKey> {
        self.keys
            .get(key_id)
            .ok_or_else(|| anyhow!("provider secret encryption key {key_id} is unavailable"))
    }
}

fn write_new_key_file(path: &Path, encoded_key: &str) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("create provider secret key file {}", path.display()))?;
        file.write_all(encoded_key.as_bytes())
            .with_context(|| format!("write provider secret key file {}", path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("finish provider secret key file {}", path.display()))?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("create provider secret key file {}", path.display()))?;
        file.write_all(encoded_key.as_bytes())
            .with_context(|| format!("write provider secret key file {}", path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("finish provider secret key file {}", path.display()))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct EncryptedSqliteProviderSecretStore {
    connection: Arc<Mutex<Connection>>,
    keyring: SecretKeyring,
}

impl EncryptedSqliteProviderSecretStore {
    pub fn in_memory(keyring: SecretKeyring) -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory()
            .context("open in-memory encrypted provider secret sqlite")?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
            keyring,
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>, keyring: SecretKeyring) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "create encrypted provider secret sqlite dir {}",
                    parent.display()
                )
            })?;
        }
        let connection = Connection::open(path).with_context(|| {
            format!(
                "open encrypted provider secret sqlite store {}",
                path.display()
            )
        })?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
            keyring,
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> anyhow::Result<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS encrypted_provider_secrets (
                    provider_secret_id TEXT PRIMARY KEY,
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    provider TEXT NOT NULL,
                    display_name TEXT NOT NULL,
                    key_id TEXT NOT NULL,
                    nonce BLOB NOT NULL,
                    ciphertext BLOB NOT NULL,
                    active INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    rotated_at TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_encrypted_provider_secrets_scope
                ON encrypted_provider_secrets (tenant_id, project_id, provider, active);
                "#,
            )
            .context("initialize encrypted provider secret sqlite store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|err| {
            anyhow!("encrypted provider secret sqlite connection mutex poisoned: {err}")
        })
    }

    fn encrypt_secret(
        &self,
        metadata: &ProviderSecretMetadata,
        secret_value: &str,
    ) -> anyhow::Result<EncryptedSecretValue> {
        let key = self.keyring.active_key()?;
        let mut nonce = [0_u8; NONCE_BYTES];
        OsRng.fill_bytes(&mut nonce);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.bytes));
        let aad = secret_aad(metadata);
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: secret_value.as_bytes(),
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|err| anyhow!("encrypt provider secret: {err}"))?;
        Ok(EncryptedSecretValue {
            key_id: key.key_id.clone(),
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt_secret(
        &self,
        metadata: &ProviderSecretMetadata,
        encrypted: EncryptedSecretValue,
    ) -> anyhow::Result<String> {
        let key = self.keyring.key(&encrypted.key_id)?;
        let nonce: [u8; NONCE_BYTES] = encrypted
            .nonce
            .try_into()
            .map_err(|_| anyhow!("provider secret nonce must be 12 bytes"))?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key.bytes));
        let aad = secret_aad(metadata);
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: encrypted.ciphertext.as_slice(),
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|err| anyhow!("decrypt provider secret: {err}"))?;
        String::from_utf8(plaintext).context("provider secret is not valid utf-8")
    }

    /// Re-wrap every stored ciphertext that is not already wrapped under the
    /// keyring's active key. Each row is decrypted with the key it was written
    /// under (which must still be present in the keyring) and re-encrypted with a
    /// fresh nonce under the active key, then written back in a single
    /// transaction. This is the online half of a key rotation: load a keyring via
    /// [`SecretKeyring::with_keys`] containing both the retiring and the new
    /// active key, call this, and once it returns the old key material can be
    /// dropped. Idempotent — a second call re-wraps nothing.
    ///
    /// # Concurrency (TOCTOU)
    ///
    /// The stale-row scan ([`load_rows_not_under_key`]) and the re-wrap
    /// transaction acquire the store lock separately, so a row inserted by a
    /// concurrent writer *between* the scan and the transaction would be missed
    /// by this pass (it would stay under its original key). Rotation must
    /// therefore run with **no concurrent writers** to the provider secret store
    /// — quiesce the API/write path, run rotation, then resume. (The operator
    /// entrypoint is `beaterctl secret-rotate`, which surfaces this requirement.)
    ///
    /// Returns the number of rows re-encrypted.
    pub fn rotate_to_active_key(&self) -> anyhow::Result<usize> {
        let active_key_id = self.keyring.active_key_id().to_string();
        let stale = self.load_rows_not_under_key(&active_key_id)?;
        if stale.is_empty() {
            return Ok(0);
        }
        let mut connection = self.lock()?;
        let tx = connection
            .transaction()
            .context("begin provider secret rotation transaction")?;
        for row in &stale {
            let plaintext = self.decrypt_secret(&row.metadata, row.encrypted.clone())?;
            let rewrapped = self.encrypt_secret(&row.metadata, &plaintext)?;
            tx.execute(
                r#"
                UPDATE encrypted_provider_secrets
                SET key_id = ?1, nonce = ?2, ciphertext = ?3
                WHERE provider_secret_id = ?4
                "#,
                params![
                    rewrapped.key_id,
                    rewrapped.nonce,
                    rewrapped.ciphertext,
                    row.metadata.provider_secret_id.as_str(),
                ],
            )
            .context("re-encrypt provider secret under active key")?;
        }
        tx.commit()
            .context("commit provider secret rotation transaction")?;
        Ok(stale.len())
    }

    fn load_rows_not_under_key(
        &self,
        active_key_id: &str,
    ) -> anyhow::Result<Vec<EncryptedSecretRow>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT provider_secret_id, tenant_id, project_id, provider, display_name,
                       active, created_at, rotated_at, key_id, nonce, ciphertext
                FROM encrypted_provider_secrets
                WHERE key_id <> ?1
                ORDER BY provider_secret_id ASC
                "#,
            )
            .context("prepare provider secret rotation scan")?;
        let rows = statement
            .query_map(params![active_key_id], decode_encrypted_secret_row)
            .context("scan provider secrets for rotation")?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row.context("decode provider secret row for rotation")?);
        }
        Ok(records)
    }
}

#[async_trait]
impl ProviderSecretStore for EncryptedSqliteProviderSecretStore {
    async fn put_secret(
        &self,
        request: PutProviderSecretRequest,
    ) -> StoreResult<ProviderSecretMetadata> {
        let metadata = ProviderSecretMetadata {
            provider_secret_id: ProviderSecretId::new(Uuid::new_v4().to_string())
                .map_err(StoreError::backend)?,
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            provider: request.provider,
            display_name: request.display_name,
            active: true,
            created_at: Utc::now(),
            rotated_at: None,
        };
        let encrypted = self
            .encrypt_secret(&metadata, &request.secret_value)
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO encrypted_provider_secrets
                  (provider_secret_id, tenant_id, project_id, provider, display_name,
                   key_id, nonce, ciphertext, active, created_at, rotated_at)
                VALUES
                  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    metadata.provider_secret_id.as_str(),
                    metadata.tenant_id.as_str(),
                    metadata.project_id.as_str(),
                    metadata.provider.as_str(),
                    metadata.display_name.as_str(),
                    encrypted.key_id,
                    encrypted.nonce,
                    encrypted.ciphertext,
                    if metadata.active { 1_i64 } else { 0_i64 },
                    metadata.created_at.to_rfc3339(),
                    metadata.rotated_at.as_ref().map(|time| time.to_rfc3339()),
                ],
            )
            .context("insert encrypted provider secret")
            .into_store()?;
        Ok(metadata)
    }

    async fn get_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
    ) -> StoreResult<Option<ProviderSecret>> {
        let row = {
            let connection = self.lock().into_store()?;
            connection
                .query_row(
                    r#"
                    SELECT provider_secret_id, tenant_id, project_id, provider, display_name,
                           active, created_at, rotated_at, key_id, nonce, ciphertext
                    FROM encrypted_provider_secrets
                    WHERE tenant_id = ?1
                      AND project_id = ?2
                      AND provider_secret_id = ?3
                      AND active = 1
                    "#,
                    params![
                        tenant_id.as_str(),
                        project_id.as_str(),
                        provider_secret_id.as_str()
                    ],
                    decode_encrypted_secret_row,
                )
                .optional()
                .context("get encrypted provider secret")
                .into_store()?
        };
        let Some(row) = row else {
            return Ok(None);
        };
        let secret_value = self
            .decrypt_secret(&row.metadata, row.encrypted)
            .into_store()?;
        Ok(Some(ProviderSecret::from_decrypted(
            row.metadata,
            secret_value,
        )))
    }

    async fn revoke_secret(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        provider_secret_id: ProviderSecretId,
        rotated_at: Timestamp,
    ) -> StoreResult<Option<RevokedProviderSecret>> {
        let connection = self.lock().into_store()?;
        let changed = connection
            .execute(
                r#"
                UPDATE encrypted_provider_secrets
                SET active = 0, rotated_at = ?4
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND provider_secret_id = ?3
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    provider_secret_id.as_str(),
                    rotated_at.to_rfc3339()
                ],
            )
            .context("revoke encrypted provider secret")
            .into_store()?;
        if changed == 0 {
            return Ok(None);
        }
        Ok(Some(RevokedProviderSecret {
            provider_secret_id,
            active: false,
            rotated_at,
        }))
    }

    async fn list_secret_metadata(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
    ) -> StoreResult<Vec<ProviderSecretMetadata>> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT provider_secret_id, tenant_id, project_id, provider, display_name,
                       active, created_at, rotated_at
                FROM encrypted_provider_secrets
                WHERE tenant_id = ?1 AND project_id = ?2
                ORDER BY created_at DESC, provider_secret_id ASC
                "#,
            )
            .context("prepare list encrypted provider secrets")
            .into_store()?;
        let rows = statement
            .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
                decode_metadata(row)
            })
            .context("query encrypted provider secret metadata")
            .into_store()?;
        let mut records = Vec::new();
        for row in rows {
            records.push(
                row.context("decode encrypted provider secret metadata")
                    .into_store()?,
            );
        }
        Ok(records)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct EncryptedSecretValue {
    key_id: String,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EncryptedSecretRow {
    metadata: ProviderSecretMetadata,
    encrypted: EncryptedSecretValue,
}

fn decode_encrypted_secret_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EncryptedSecretRow> {
    Ok(EncryptedSecretRow {
        metadata: decode_metadata(row)?,
        encrypted: EncryptedSecretValue {
            key_id: row.get(8)?,
            nonce: row.get(9)?,
            ciphertext: row.get(10)?,
        },
    })
}

fn decode_metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderSecretMetadata> {
    let created_at = parse_time(row.get::<_, String>(6)?)?;
    let rotated_at = row
        .get::<_, Option<String>>(7)?
        .map(parse_time)
        .transpose()?;
    Ok(ProviderSecretMetadata {
        provider_secret_id: ProviderSecretId::new(row.get::<_, String>(0)?)
            .map_err(sql_decode_error)?,
        tenant_id: TenantId::new(row.get::<_, String>(1)?).map_err(sql_decode_error)?,
        project_id: ProjectId::new(row.get::<_, String>(2)?).map_err(sql_decode_error)?,
        provider: row.get(3)?,
        display_name: row.get(4)?,
        active: row.get::<_, i64>(5)? != 0,
        created_at,
        rotated_at,
    })
}

fn parse_time(value: String) -> rusqlite::Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                value.len(),
                rusqlite::types::Type::Text,
                Box::new(err),
            )
        })
}

fn sql_decode_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn secret_aad(metadata: &ProviderSecretMetadata) -> String {
    format!(
        "beater.provider_secret.v1:{}:{}:{}:{}",
        metadata.tenant_id.as_str(),
        metadata.project_id.as_str(),
        metadata.provider_secret_id.as_str(),
        metadata.provider
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn encrypted_store_round_trips_without_plaintext_in_sqlite_file() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let db_path = tempdir.path().join("provider-secrets.sqlite");
        let keyring = SecretKeyring::generated_for_tests().unwrap_or_else(|err| panic!("{err}"));
        let store = EncryptedSqliteProviderSecretStore::open(&db_path, keyring)
            .unwrap_or_else(|err| panic!("{err}"));
        let metadata = store
            .put_secret(PutProviderSecretRequest {
                tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
                project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
                provider: "openai".to_string(),
                display_name: "encrypted judge".to_string(),
                secret_value: "sk-encrypted-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let loaded = store
            .get_secret(
                metadata.tenant_id.clone(),
                metadata.project_id.clone(),
                metadata.provider_secret_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("encrypted provider secret should exist"));
        assert_eq!(loaded.secret_value(), "sk-encrypted-secret");
        drop(store);

        let db_bytes = fs::read(&db_path).unwrap_or_else(|err| panic!("{err}"));
        assert!(!String::from_utf8_lossy(&db_bytes).contains("sk-encrypted-secret"));
        let wal_path = tempdir.path().join("provider-secrets.sqlite-wal");
        if wal_path.exists() {
            let wal_bytes = fs::read(wal_path).unwrap_or_else(|err| panic!("{err}"));
            assert!(!String::from_utf8_lossy(&wal_bytes).contains("sk-encrypted-secret"));
        }
    }

    #[tokio::test]
    async fn encrypted_store_rejects_wrong_key_material() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let db_path = tempdir.path().join("provider-secrets.sqlite");
        let store = EncryptedSqliteProviderSecretStore::open(
            &db_path,
            SecretKeyring::single(
                SecretEncryptionKey::new("shared-key", [7_u8; KEY_BYTES])
                    .unwrap_or_else(|err| panic!("{err}")),
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let tenant_id = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let metadata = store
            .put_secret(PutProviderSecretRequest {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                provider: "anthropic".to_string(),
                display_name: "encrypted judge".to_string(),
                secret_value: "sk-encrypted-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        drop(store);

        let wrong_key_store = EncryptedSqliteProviderSecretStore::open(
            &db_path,
            SecretKeyring::single(
                SecretEncryptionKey::new("shared-key", [9_u8; KEY_BYTES])
                    .unwrap_or_else(|err| panic!("{err}")),
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let result = wrong_key_store
            .get_secret(tenant_id, project_id, metadata.provider_secret_id)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rotating_re_wraps_ciphertext_under_new_active_key() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let db_path = tempdir.path().join("provider-secrets.sqlite");
        let tenant_id = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project_id = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        // Write a secret under the old (v1) key.
        let old_key = SecretEncryptionKey::new("secrets-v1", [3_u8; KEY_BYTES])
            .unwrap_or_else(|err| panic!("{err}"));
        let store = EncryptedSqliteProviderSecretStore::open(
            &db_path,
            SecretKeyring::single(old_key.clone()),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        let metadata = store
            .put_secret(PutProviderSecretRequest {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                provider: "openai".to_string(),
                display_name: "rotating judge".to_string(),
                secret_value: "sk-rotating-secret".to_string(),
            })
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        drop(store);

        // Reopen with a keyring that has the new active key plus the old key for
        // decrypting existing rows, then rotate.
        let new_key = SecretEncryptionKey::new("secrets-v2", [4_u8; KEY_BYTES])
            .unwrap_or_else(|err| panic!("{err}"));
        let rotating_keyring = SecretKeyring::with_keys("secrets-v2", [new_key.clone(), old_key])
            .unwrap_or_else(|err| panic!("{err}"));
        let store = EncryptedSqliteProviderSecretStore::open(&db_path, rotating_keyring)
            .unwrap_or_else(|err| panic!("{err}"));

        let rewrapped = store
            .rotate_to_active_key()
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(rewrapped, 1, "the single stale row should be re-wrapped");
        // Re-running is a no-op once everything is under the active key.
        assert_eq!(
            store
                .rotate_to_active_key()
                .unwrap_or_else(|err| panic!("{err}")),
            0
        );

        // The secret still decrypts to the original plaintext after rotation.
        let loaded = store
            .get_secret(
                tenant_id.clone(),
                project_id.clone(),
                metadata.provider_secret_id.clone(),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("rotated provider secret should exist"));
        assert_eq!(loaded.secret_value(), "sk-rotating-secret");
        drop(store);

        // The new active key alone (no old key) can now read the row — proving the
        // stored ciphertext is genuinely wrapped under v2, not still under v1.
        let new_only =
            EncryptedSqliteProviderSecretStore::open(&db_path, SecretKeyring::single(new_key))
                .unwrap_or_else(|err| panic!("{err}"));
        let loaded = new_only
            .get_secret(tenant_id, project_id, metadata.provider_secret_id)
            .await
            .unwrap_or_else(|err| panic!("{err}"))
            .unwrap_or_else(|| panic!("rotated provider secret should be readable under v2 alone"));
        assert_eq!(loaded.secret_value(), "sk-rotating-secret");
    }

    #[tokio::test]
    async fn encrypted_provider_secret_store_enforces_tenant_project_scope() {
        let keyring = SecretKeyring::generated_for_tests().unwrap_or_else(|err| panic!("{err}"));
        let store = EncryptedSqliteProviderSecretStore::in_memory(keyring)
            .unwrap_or_else(|err| panic!("{err}"));

        crate::assert_provider_secret_scope_isolation(&store).await;
    }

    #[test]
    fn keyring_requires_active_key_to_be_present() {
        let key = SecretEncryptionKey::new("present", [1_u8; KEY_BYTES])
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(SecretKeyring::with_keys("missing", [key]).is_err());
    }

    #[test]
    fn local_key_file_is_reused_and_redacted() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("provider-secrets.key");
        let first = SecretKeyring::load_or_create_local_file(&path, "local-v1")
            .unwrap_or_else(|err| panic!("{err}"));
        let second = SecretKeyring::load_or_create_local_file(&path, "local-v1")
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            first
                .active_key()
                .unwrap_or_else(|err| panic!("{err}"))
                .to_base64(),
            second
                .active_key()
                .unwrap_or_else(|err| panic!("{err}"))
                .to_base64()
        );
        assert!(!format!("{first:?}").contains(
            first
                .active_key()
                .unwrap_or_else(|err| panic!("{err}"))
                .to_base64()
                .as_str()
        ));
    }
}
