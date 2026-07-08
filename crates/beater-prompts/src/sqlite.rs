//! SQLite-backed [`PromptRegistry`] wired into the running beaterd service.
//!
//! Mirrors the per-crate rusqlite stores used elsewhere in the workspace
//! (e.g. `beater-datasets`): a single connection behind a mutex, schema created
//! on open, and template payloads persisted as JSON. Prompt records and their
//! immutable versions are scoped by `(tenant_id, project_id, prompt_id)`.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::{Context, anyhow};
use beater_core::{Clock, ProjectId, PromptId, PromptVersionId, SystemClock, TenantId, Timestamp};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{
    AddPromptVersion, CreatePrompt, CreatedPrompt, Prompt, PromptRegistry, PromptRegistryError,
    PromptRegistryResult, PromptTemplate, PromptVersion, PromptVersionDiff, PromptVersionMetadata,
    diff_lines, new_prompt_id, new_prompt_version_id,
};

/// SQLite-backed prompt registry. Cloneable; clones share one connection.
#[derive(Clone)]
pub struct SqlitePromptRegistry {
    connection: Arc<Mutex<Connection>>,
    clock: Arc<dyn Clock>,
}

impl SqlitePromptRegistry {
    /// Open an in-memory store (tests and ephemeral use).
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory prompt sqlite")?;
        Self::from_connection(connection, Arc::new(SystemClock))
    }

    /// Open (creating if needed) a file-backed store at `path`.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create prompt sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite prompt store {}", path.display()))?;
        Self::from_connection(connection, Arc::new(SystemClock))
    }

    /// Override the clock (deterministic timestamps in tests).
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    fn from_connection(connection: Connection, clock: Arc<dyn Clock>) -> anyhow::Result<Self> {
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
            clock,
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> anyhow::Result<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|err| anyhow!("sqlite prompt connection mutex poisoned: {err}"))?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS prompts (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    prompt_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, prompt_id)
                );

                CREATE TABLE IF NOT EXISTS prompt_versions (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    prompt_id TEXT NOT NULL,
                    version_id TEXT NOT NULL,
                    version_number INTEGER NOT NULL,
                    template_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    created_by TEXT,
                    message TEXT,
                    PRIMARY KEY (tenant_id, project_id, prompt_id, version_id)
                );
                "#,
            )
            .context("initialize sqlite prompt store")?;
        Ok(())
    }

    fn lock(&self) -> PromptRegistryResult<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_err| PromptRegistryError::LockPoisoned)
    }
}

fn backend(err: impl std::fmt::Display) -> PromptRegistryError {
    PromptRegistryError::Backend(err.to_string())
}

fn encode_timestamp(timestamp: Timestamp) -> String {
    timestamp.to_rfc3339()
}

fn decode_timestamp(value: &str) -> PromptRegistryResult<Timestamp> {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|err| backend(format!("invalid stored timestamp {value:?}: {err}")))
}

fn prompt_from_row(row: &Row<'_>) -> PromptRegistryResult<Prompt> {
    let tenant_id: String = row.get(0).map_err(backend)?;
    let project_id: String = row.get(1).map_err(backend)?;
    let prompt_id: String = row.get(2).map_err(backend)?;
    let name: String = row.get(3).map_err(backend)?;
    let description: Option<String> = row.get(4).map_err(backend)?;
    let created_at: String = row.get(5).map_err(backend)?;
    let updated_at: String = row.get(6).map_err(backend)?;
    Ok(Prompt {
        tenant_id: TenantId::new(tenant_id).map_err(backend)?,
        project_id: ProjectId::new(project_id).map_err(backend)?,
        prompt_id: PromptId::new(prompt_id).map_err(backend)?,
        name,
        description,
        created_at: decode_timestamp(&created_at)?,
        updated_at: decode_timestamp(&updated_at)?,
    })
}

fn version_from_row(row: &Row<'_>) -> PromptRegistryResult<PromptVersion> {
    let tenant_id: String = row.get(0).map_err(backend)?;
    let project_id: String = row.get(1).map_err(backend)?;
    let prompt_id: String = row.get(2).map_err(backend)?;
    let version_id: String = row.get(3).map_err(backend)?;
    let version_number: i64 = row.get(4).map_err(backend)?;
    let template_json: String = row.get(5).map_err(backend)?;
    let created_at: String = row.get(6).map_err(backend)?;
    let created_by: Option<String> = row.get(7).map_err(backend)?;
    let message: Option<String> = row.get(8).map_err(backend)?;
    let template: PromptTemplate = serde_json::from_str(&template_json)
        .map_err(|err| backend(format!("invalid stored template json: {err}")))?;
    let version_number = u32::try_from(version_number)
        .map_err(|err| backend(format!("invalid stored version number: {err}")))?;
    Ok(PromptVersion {
        tenant_id: TenantId::new(tenant_id).map_err(backend)?,
        project_id: ProjectId::new(project_id).map_err(backend)?,
        prompt_id: PromptId::new(prompt_id).map_err(backend)?,
        version_id: PromptVersionId::new(version_id).map_err(backend)?,
        version_number,
        template,
        metadata: PromptVersionMetadata {
            created_at: decode_timestamp(&created_at)?,
            created_by,
            message,
        },
    })
}

const PROMPT_COLUMNS: &str =
    "tenant_id, project_id, prompt_id, name, description, created_at, updated_at";
const VERSION_COLUMNS: &str = "tenant_id, project_id, prompt_id, version_id, version_number, \
     template_json, created_at, created_by, message";

fn prompt_exists(
    connection: &Connection,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    prompt_id: &PromptId,
) -> PromptRegistryResult<bool> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM prompts WHERE tenant_id = ?1 AND project_id = ?2 AND prompt_id = ?3",
            params![tenant_id.as_str(), project_id.as_str(), prompt_id.as_str()],
            |_| Ok(()),
        )
        .optional()
        .map_err(backend)?
        .is_some();
    Ok(exists)
}

impl PromptRegistry for SqlitePromptRegistry {
    fn create_prompt(&self, request: CreatePrompt) -> PromptRegistryResult<CreatedPrompt> {
        let now = self.clock.now();
        let prompt_id = new_prompt_id();
        let version_id = new_prompt_version_id();
        let prompt = Prompt {
            tenant_id: request.tenant_id.clone(),
            project_id: request.project_id.clone(),
            prompt_id: prompt_id.clone(),
            name: request.name,
            description: request.description,
            created_at: now,
            updated_at: now,
        };
        let version = PromptVersion {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            prompt_id,
            version_id,
            version_number: 1,
            template: request.template,
            metadata: PromptVersionMetadata {
                created_at: now,
                created_by: request.created_by,
                message: request.message,
            },
        };
        let template_json = serde_json::to_string(&version.template).map_err(backend)?;

        let connection = self.lock()?;
        connection
            .execute(
                "INSERT INTO prompts (tenant_id, project_id, prompt_id, name, description, \
                 created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    prompt.tenant_id.as_str(),
                    prompt.project_id.as_str(),
                    prompt.prompt_id.as_str(),
                    prompt.name,
                    prompt.description,
                    encode_timestamp(prompt.created_at),
                    encode_timestamp(prompt.updated_at),
                ],
            )
            .map_err(backend)?;
        connection
            .execute(
                "INSERT INTO prompt_versions (tenant_id, project_id, prompt_id, version_id, \
                 version_number, template_json, created_at, created_by, message) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    version.tenant_id.as_str(),
                    version.project_id.as_str(),
                    version.prompt_id.as_str(),
                    version.version_id.as_str(),
                    i64::from(version.version_number),
                    template_json,
                    encode_timestamp(version.metadata.created_at),
                    version.metadata.created_by,
                    version.metadata.message,
                ],
            )
            .map_err(backend)?;
        Ok(CreatedPrompt { prompt, version })
    }

    fn add_version(&self, request: AddPromptVersion) -> PromptRegistryResult<PromptVersion> {
        let now = self.clock.now();
        let template_json = serde_json::to_string(&request.template).map_err(backend)?;
        let connection = self.lock()?;
        if !prompt_exists(
            &connection,
            &request.tenant_id,
            &request.project_id,
            &request.prompt_id,
        )? {
            return Err(PromptRegistryError::PromptNotFound {
                prompt_id: request.prompt_id,
            });
        }
        let next_number: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(version_number), 0) + 1 FROM prompt_versions \
                 WHERE tenant_id = ?1 AND project_id = ?2 AND prompt_id = ?3",
                params![
                    request.tenant_id.as_str(),
                    request.project_id.as_str(),
                    request.prompt_id.as_str()
                ],
                |row| row.get(0),
            )
            .map_err(backend)?;
        let version_number = u32::try_from(next_number)
            .map_err(|err| backend(format!("version number overflow: {err}")))?;
        let version = PromptVersion {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            prompt_id: request.prompt_id,
            version_id: new_prompt_version_id(),
            version_number,
            template: request.template,
            metadata: PromptVersionMetadata {
                created_at: now,
                created_by: request.created_by,
                message: request.message,
            },
        };
        connection
            .execute(
                "INSERT INTO prompt_versions (tenant_id, project_id, prompt_id, version_id, \
                 version_number, template_json, created_at, created_by, message) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    version.tenant_id.as_str(),
                    version.project_id.as_str(),
                    version.prompt_id.as_str(),
                    version.version_id.as_str(),
                    i64::from(version.version_number),
                    template_json,
                    encode_timestamp(version.metadata.created_at),
                    version.metadata.created_by,
                    version.metadata.message,
                ],
            )
            .map_err(backend)?;
        connection
            .execute(
                "UPDATE prompts SET updated_at = ?4 \
                 WHERE tenant_id = ?1 AND project_id = ?2 AND prompt_id = ?3",
                params![
                    version.tenant_id.as_str(),
                    version.project_id.as_str(),
                    version.prompt_id.as_str(),
                    encode_timestamp(now),
                ],
            )
            .map_err(backend)?;
        Ok(version)
    }

    fn list_prompts(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
    ) -> PromptRegistryResult<Vec<Prompt>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(&format!(
                "SELECT {PROMPT_COLUMNS} FROM prompts \
                 WHERE tenant_id = ?1 AND project_id = ?2 ORDER BY created_at, prompt_id"
            ))
            .map_err(backend)?;
        let rows = statement
            .query_map(params![tenant_id.as_str(), project_id.as_str()], |row| {
                Ok(prompt_from_row(row))
            })
            .map_err(backend)?;
        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row.map_err(backend)??);
        }
        Ok(prompts)
    }

    fn get_prompt(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<Prompt> {
        let connection = self.lock()?;
        connection
            .query_row(
                &format!(
                    "SELECT {PROMPT_COLUMNS} FROM prompts \
                     WHERE tenant_id = ?1 AND project_id = ?2 AND prompt_id = ?3"
                ),
                params![tenant_id.as_str(), project_id.as_str(), prompt_id.as_str()],
                |row| Ok(prompt_from_row(row)),
            )
            .optional()
            .map_err(backend)?
            .transpose()?
            .ok_or_else(|| PromptRegistryError::PromptNotFound {
                prompt_id: prompt_id.clone(),
            })
    }

    fn list_versions(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<Vec<PromptVersion>> {
        let connection = self.lock()?;
        if !prompt_exists(&connection, tenant_id, project_id, prompt_id)? {
            return Err(PromptRegistryError::PromptNotFound {
                prompt_id: prompt_id.clone(),
            });
        }
        let mut statement = connection
            .prepare(&format!(
                "SELECT {VERSION_COLUMNS} FROM prompt_versions \
                 WHERE tenant_id = ?1 AND project_id = ?2 AND prompt_id = ?3 \
                 ORDER BY version_number"
            ))
            .map_err(backend)?;
        let rows = statement
            .query_map(
                params![tenant_id.as_str(), project_id.as_str(), prompt_id.as_str()],
                |row| Ok(version_from_row(row)),
            )
            .map_err(backend)?;
        let mut versions = Vec::new();
        for row in rows {
            versions.push(row.map_err(backend)??);
        }
        Ok(versions)
    }

    fn get_latest_version(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<PromptVersion> {
        let versions = self.list_versions(tenant_id, project_id, prompt_id)?;
        versions
            .into_iter()
            .last()
            .ok_or_else(|| PromptRegistryError::PromptNotFound {
                prompt_id: prompt_id.clone(),
            })
    }

    fn get_version(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
        version_id: &PromptVersionId,
    ) -> PromptRegistryResult<PromptVersion> {
        let connection = self.lock()?;
        connection
            .query_row(
                &format!(
                    "SELECT {VERSION_COLUMNS} FROM prompt_versions \
                     WHERE tenant_id = ?1 AND project_id = ?2 AND prompt_id = ?3 \
                     AND version_id = ?4"
                ),
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    prompt_id.as_str(),
                    version_id.as_str()
                ],
                |row| Ok(version_from_row(row)),
            )
            .optional()
            .map_err(backend)?
            .transpose()?
            .ok_or_else(|| PromptRegistryError::VersionNotFound {
                version_id: version_id.clone(),
            })
    }

    fn diff_versions(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
        from_version_id: &PromptVersionId,
        to_version_id: &PromptVersionId,
    ) -> PromptRegistryResult<PromptVersionDiff> {
        let from = self.get_version(tenant_id, project_id, prompt_id, from_version_id)?;
        let to = self.get_version(tenant_id, project_id, prompt_id, to_version_id)?;
        Ok(PromptVersionDiff {
            from_version_id: from.version_id,
            to_version_id: to.version_id,
            lines: diff_lines(&from.template.body, &to.template.body),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PromptVariable;
    use beater_core::FixedClock;
    use chrono::TimeZone;

    fn tenant(value: &str) -> TenantId {
        TenantId::new(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn project(value: &str) -> ProjectId {
        ProjectId::new(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn store() -> SqlitePromptRegistry {
        let now = Utc
            .with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid fixed timestamp"));
        SqlitePromptRegistry::in_memory()
            .unwrap_or_else(|err| panic!("{err}"))
            .with_clock(Arc::new(FixedClock::new(now)))
    }

    fn template(body: &str) -> PromptTemplate {
        PromptTemplate {
            body: body.to_string(),
            variables: vec![PromptVariable::required("question")],
            tags: vec!["support".to_string()],
        }
    }

    fn create(tenant_id: TenantId, project_id: ProjectId, body: &str) -> CreatePrompt {
        CreatePrompt {
            tenant_id,
            project_id,
            name: "answer-support-question".to_string(),
            description: Some("Support prompt".to_string()),
            template: template(body),
            created_by: Some("agent".to_string()),
            message: Some("initial".to_string()),
        }
    }

    #[test]
    fn persists_prompt_versions_and_diffs() {
        let store = store();
        let tenant_id = tenant("tenant-a");
        let project_id = project("project-a");
        let created = store
            .create_prompt(create(
                tenant_id.clone(),
                project_id.clone(),
                "system\nanswer briefly",
            ))
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(created.version.version_number, 1);

        let second = store
            .add_version(AddPromptVersion {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                prompt_id: created.prompt.prompt_id.clone(),
                template: template("system\nanswer with detail"),
                created_by: None,
                message: Some("expand".to_string()),
            })
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(second.version_number, 2);

        let prompts = store
            .list_prompts(&tenant_id, &project_id)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].prompt_id, created.prompt.prompt_id);

        let fetched = store
            .get_prompt(&tenant_id, &project_id, &created.prompt.prompt_id)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(fetched.name, "answer-support-question");

        let versions = store
            .list_versions(&tenant_id, &project_id, &created.prompt.prompt_id)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].template.variables[0].name, "question");

        let diff: PromptVersionDiff = store
            .diff_versions(
                &tenant_id,
                &project_id,
                &created.prompt.prompt_id,
                &created.version.version_id,
                &second.version_id,
            )
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(diff.lines.iter().any(|line| line.text == "answer briefly"));
        assert!(
            diff.lines
                .iter()
                .any(|line| line.text == "answer with detail")
        );
    }

    #[test]
    fn missing_prompt_is_not_found_and_scopes_isolate() {
        let store = store();
        let tenant_a = tenant("tenant-a");
        let project_a = project("project-a");
        let created = store
            .create_prompt(create(tenant_a.clone(), project_a.clone(), "body"))
            .unwrap_or_else(|err| panic!("{err}"));

        let Err(err) = store.get_prompt(&tenant("tenant-b"), &project_a, &created.prompt.prompt_id)
        else {
            panic!("cross-tenant read must not see the prompt");
        };
        assert_eq!(
            err,
            PromptRegistryError::PromptNotFound {
                prompt_id: created.prompt.prompt_id.clone()
            }
        );

        let empty = store
            .list_prompts(&tenant("tenant-b"), &project_a)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(empty.is_empty());
    }
}
