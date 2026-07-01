//! `beater-prompts` - Mixdown prompt registry (ARCHITECTURE.md §20.6 #4.7).
//!
//! Tenant/project-scoped prompt records, immutable prompt versions, variables,
//! tags, and body diffs. The [`PromptRegistry`] trait has two implementations:
//! [`InMemoryPromptRegistry`] (tests/embedding) and [`SqlitePromptRegistry`]
//! (the persistent store wired into the running beaterd service). The `/v1`
//! prompt endpoints in `beater-api` are the live consumer of this crate.

mod sqlite;

pub use sqlite::SqlitePromptRegistry;

use beater_core::{Clock, ProjectId, PromptId, PromptVersionId, SystemClock, TenantId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PromptTemplate {
    pub body: String,
    pub variables: Vec<PromptVariable>,
    pub tags: Vec<String>,
}

impl PromptTemplate {
    #[must_use]
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            variables: Vec::new(),
            tags: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PromptVariable {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
}

impl PromptVariable {
    #[must_use]
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: true,
            default: None,
        }
    }

    #[must_use]
    pub fn optional(name: impl Into<String>, default: Option<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: false,
            default,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Prompt {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub prompt_id: PromptId,
    pub name: String,
    pub description: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PromptVersion {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub prompt_id: PromptId,
    pub version_id: PromptVersionId,
    pub version_number: u32,
    pub template: PromptTemplate,
    pub metadata: PromptVersionMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PromptVersionMetadata {
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
    pub created_by: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatePrompt {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub template: PromptTemplate,
    pub created_by: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddPromptVersion {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub prompt_id: PromptId,
    pub template: PromptTemplate,
    pub created_by: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatedPrompt {
    pub prompt: Prompt,
    pub version: PromptVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PromptVersionDiff {
    pub from_version_id: PromptVersionId,
    pub to_version_id: PromptVersionId,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    Unchanged,
    Added,
    Removed,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PromptRegistryError {
    #[error("prompt not found: {prompt_id}")]
    PromptNotFound { prompt_id: PromptId },
    #[error("prompt version not found: {version_id}")]
    VersionNotFound { version_id: PromptVersionId },
    #[error("prompt version {version_id} does not belong to prompt {prompt_id}")]
    VersionPromptMismatch {
        prompt_id: PromptId,
        version_id: PromptVersionId,
    },
    #[error("prompt registry lock poisoned")]
    LockPoisoned,
    #[error("prompt registry backend error: {0}")]
    Backend(String),
}

pub type PromptRegistryResult<T> = Result<T, PromptRegistryError>;

pub trait PromptRegistry: Send + Sync {
    fn create_prompt(&self, request: CreatePrompt) -> PromptRegistryResult<CreatedPrompt>;
    fn add_version(&self, request: AddPromptVersion) -> PromptRegistryResult<PromptVersion>;
    fn list_prompts(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
    ) -> PromptRegistryResult<Vec<Prompt>>;
    fn get_prompt(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<Prompt>;
    fn list_versions(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<Vec<PromptVersion>>;
    fn get_latest_version(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<PromptVersion>;
    fn get_version(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
        version_id: &PromptVersionId,
    ) -> PromptRegistryResult<PromptVersion>;
    fn diff_versions(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
        from_version_id: &PromptVersionId,
        to_version_id: &PromptVersionId,
    ) -> PromptRegistryResult<PromptVersionDiff>;
}

#[derive(Clone)]
pub struct InMemoryPromptRegistry {
    state: Arc<Mutex<RegistryState>>,
    clock: Arc<dyn Clock>,
}

impl Default for InMemoryPromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPromptRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(Arc::new(SystemClock))
    }

    #[must_use]
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            state: Arc::new(Mutex::new(RegistryState::default())),
            clock,
        }
    }

    fn lock(&self) -> PromptRegistryResult<MutexGuard<'_, RegistryState>> {
        self.state
            .lock()
            .map_err(|_err| PromptRegistryError::LockPoisoned)
    }
}

impl PromptRegistry for InMemoryPromptRegistry {
    fn create_prompt(&self, request: CreatePrompt) -> PromptRegistryResult<CreatedPrompt> {
        let now = self.clock.now();
        let prompt_id = new_prompt_id();
        let version_id = new_prompt_version_id();
        let key = PromptKey::new(
            request.tenant_id.clone(),
            request.project_id.clone(),
            prompt_id.clone(),
        );
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

        let mut state = self.lock()?;
        state.prompts.insert(key.clone(), prompt.clone());
        state.versions.insert(key, vec![version.clone()]);
        Ok(CreatedPrompt { prompt, version })
    }

    fn add_version(&self, request: AddPromptVersion) -> PromptRegistryResult<PromptVersion> {
        let key = PromptKey::new(
            request.tenant_id.clone(),
            request.project_id.clone(),
            request.prompt_id.clone(),
        );
        let now = self.clock.now();
        let mut state = self.lock()?;
        let versions =
            state
                .versions
                .get_mut(&key)
                .ok_or_else(|| PromptRegistryError::PromptNotFound {
                    prompt_id: request.prompt_id.clone(),
                })?;
        let version = PromptVersion {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            prompt_id: request.prompt_id.clone(),
            version_id: new_prompt_version_id(),
            version_number: versions.len() as u32 + 1,
            template: request.template,
            metadata: PromptVersionMetadata {
                created_at: now,
                created_by: request.created_by,
                message: request.message,
            },
        };
        versions.push(version.clone());
        if let Some(prompt) = state.prompts.get_mut(&key) {
            prompt.updated_at = now;
        }
        Ok(version)
    }

    fn list_prompts(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
    ) -> PromptRegistryResult<Vec<Prompt>> {
        let state = self.lock()?;
        Ok(state
            .prompts
            .values()
            .filter(|prompt| &prompt.tenant_id == tenant_id && &prompt.project_id == project_id)
            .cloned()
            .collect())
    }

    fn get_prompt(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<Prompt> {
        let key = PromptKey::new(tenant_id.clone(), project_id.clone(), prompt_id.clone());
        let state = self.lock()?;
        state
            .prompts
            .get(&key)
            .cloned()
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
        let key = PromptKey::new(tenant_id.clone(), project_id.clone(), prompt_id.clone());
        let state = self.lock()?;
        let versions =
            state
                .versions
                .get(&key)
                .ok_or_else(|| PromptRegistryError::PromptNotFound {
                    prompt_id: prompt_id.clone(),
                })?;
        Ok(versions.clone())
    }

    fn get_latest_version(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        prompt_id: &PromptId,
    ) -> PromptRegistryResult<PromptVersion> {
        let versions = self.list_versions(tenant_id, project_id, prompt_id)?;
        versions
            .last()
            .cloned()
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
        let versions = self.list_versions(tenant_id, project_id, prompt_id)?;
        versions
            .into_iter()
            .find(|version| &version.version_id == version_id)
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
        if from.prompt_id != *prompt_id {
            return Err(PromptRegistryError::VersionPromptMismatch {
                prompt_id: prompt_id.clone(),
                version_id: from_version_id.clone(),
            });
        }
        if to.prompt_id != *prompt_id {
            return Err(PromptRegistryError::VersionPromptMismatch {
                prompt_id: prompt_id.clone(),
                version_id: to_version_id.clone(),
            });
        }

        Ok(PromptVersionDiff {
            from_version_id: from.version_id,
            to_version_id: to.version_id,
            lines: diff_lines(&from.template.body, &to.template.body),
        })
    }
}

#[derive(Clone, Debug, Default)]
struct RegistryState {
    prompts: BTreeMap<PromptKey, Prompt>,
    versions: BTreeMap<PromptKey, Vec<PromptVersion>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PromptKey {
    tenant_id: TenantId,
    project_id: ProjectId,
    prompt_id: PromptId,
}

impl PromptKey {
    fn new(tenant_id: TenantId, project_id: ProjectId, prompt_id: PromptId) -> Self {
        Self {
            tenant_id,
            project_id,
            prompt_id,
        }
    }
}

fn new_prompt_id() -> PromptId {
    PromptId::new(format!("prompt_{}", Uuid::new_v4())).unwrap_or_else(|err| {
        panic!("generated prompt id must be valid: {err}");
    })
}

fn new_prompt_version_id() -> PromptVersionId {
    PromptVersionId::new(format!("promptver_{}", Uuid::new_v4())).unwrap_or_else(|err| {
        panic!("generated prompt version id must be valid: {err}");
    })
}

fn diff_lines(old: &str, new: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let mut lengths = vec![vec![0usize; new_lines.len() + 1]; old_lines.len() + 1];
    for old_idx in (0..old_lines.len()).rev() {
        for new_idx in (0..new_lines.len()).rev() {
            lengths[old_idx][new_idx] = if old_lines[old_idx] == new_lines[new_idx] {
                lengths[old_idx + 1][new_idx + 1] + 1
            } else {
                lengths[old_idx + 1][new_idx].max(lengths[old_idx][new_idx + 1])
            };
        }
    }

    let mut old_idx = 0;
    let mut new_idx = 0;
    let mut result = Vec::new();
    while old_idx < old_lines.len() && new_idx < new_lines.len() {
        if old_lines[old_idx] == new_lines[new_idx] {
            result.push(DiffLine {
                kind: DiffLineKind::Unchanged,
                old_line: Some(old_idx + 1),
                new_line: Some(new_idx + 1),
                text: old_lines[old_idx].to_string(),
            });
            old_idx += 1;
            new_idx += 1;
        } else if lengths[old_idx + 1][new_idx] >= lengths[old_idx][new_idx + 1] {
            result.push(DiffLine {
                kind: DiffLineKind::Removed,
                old_line: Some(old_idx + 1),
                new_line: None,
                text: old_lines[old_idx].to_string(),
            });
            old_idx += 1;
        } else {
            result.push(DiffLine {
                kind: DiffLineKind::Added,
                old_line: None,
                new_line: Some(new_idx + 1),
                text: new_lines[new_idx].to_string(),
            });
            new_idx += 1;
        }
    }
    while old_idx < old_lines.len() {
        result.push(DiffLine {
            kind: DiffLineKind::Removed,
            old_line: Some(old_idx + 1),
            new_line: None,
            text: old_lines[old_idx].to_string(),
        });
        old_idx += 1;
    }
    while new_idx < new_lines.len() {
        result.push(DiffLine {
            kind: DiffLineKind::Added,
            old_line: None,
            new_line: Some(new_idx + 1),
            text: new_lines[new_idx].to_string(),
        });
        new_idx += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::FixedClock;
    use chrono::{TimeZone, Utc};

    fn tenant(value: &str) -> TenantId {
        TenantId::new(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn project(value: &str) -> ProjectId {
        ProjectId::new(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn registry() -> InMemoryPromptRegistry {
        let now = Utc
            .with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("valid fixed timestamp"));
        InMemoryPromptRegistry::with_clock(Arc::new(FixedClock::new(now)))
    }

    fn template(body: &str) -> PromptTemplate {
        PromptTemplate {
            body: body.to_string(),
            variables: vec![PromptVariable::required("question")],
            tags: vec!["support".to_string()],
        }
    }

    fn create_request(tenant_id: TenantId, project_id: ProjectId, body: &str) -> CreatePrompt {
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
    fn create_prompt_creates_initial_version() {
        let store = registry();
        let created = store
            .create_prompt(create_request(
                tenant("tenant-a"),
                project("project-a"),
                "Answer {{question}}",
            ))
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(created.prompt.name, "answer-support-question");
        assert_eq!(created.version.version_number, 1);
        assert_eq!(created.version.template.variables[0].name, "question");
        assert_eq!(created.version.template.tags, vec!["support"]);
        assert_eq!(
            created.version.metadata.created_by.as_deref(),
            Some("agent")
        );
    }

    #[test]
    fn add_list_and_latest_versions() {
        let store = registry();
        let tenant_id = tenant("tenant-a");
        let project_id = project("project-a");
        let created = store
            .create_prompt(create_request(
                tenant_id.clone(),
                project_id.clone(),
                "Answer {{question}}",
            ))
            .unwrap_or_else(|err| panic!("{err}"));

        let second = store
            .add_version(AddPromptVersion {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                prompt_id: created.prompt.prompt_id.clone(),
                template: template("Answer {{question}} with citations"),
                created_by: Some("agent".to_string()),
                message: Some("add citations".to_string()),
            })
            .unwrap_or_else(|err| panic!("{err}"));

        let versions = store
            .list_versions(&tenant_id, &project_id, &created.prompt.prompt_id)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version_number, 1);
        assert_eq!(versions[1].version_number, 2);

        let latest = store
            .get_latest_version(&tenant_id, &project_id, &created.prompt.prompt_id)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(latest.version_id, second.version_id);
    }

    #[test]
    fn get_version_by_id() {
        let store = registry();
        let tenant_id = tenant("tenant-a");
        let project_id = project("project-a");
        let created = store
            .create_prompt(create_request(
                tenant_id.clone(),
                project_id.clone(),
                "Answer {{question}}",
            ))
            .unwrap_or_else(|err| panic!("{err}"));

        let fetched = store
            .get_version(
                &tenant_id,
                &project_id,
                &created.prompt.prompt_id,
                &created.version.version_id,
            )
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(fetched, created.version);
    }

    #[test]
    fn diff_versions_reports_body_changes() {
        let store = registry();
        let tenant_id = tenant("tenant-a");
        let project_id = project("project-a");
        let created = store
            .create_prompt(create_request(
                tenant_id.clone(),
                project_id.clone(),
                "system\nanswer briefly\ncite sources",
            ))
            .unwrap_or_else(|err| panic!("{err}"));
        let second = store
            .add_version(AddPromptVersion {
                tenant_id: tenant_id.clone(),
                project_id: project_id.clone(),
                prompt_id: created.prompt.prompt_id.clone(),
                template: template("system\nanswer with detail\ncite sources\nend"),
                created_by: None,
                message: None,
            })
            .unwrap_or_else(|err| panic!("{err}"));

        let diff = store
            .diff_versions(
                &tenant_id,
                &project_id,
                &created.prompt.prompt_id,
                &created.version.version_id,
                &second.version_id,
            )
            .unwrap_or_else(|err| panic!("{err}"));
        let kinds: Vec<DiffLineKind> = diff.lines.iter().map(|line| line.kind).collect();
        assert_eq!(
            kinds,
            vec![
                DiffLineKind::Unchanged,
                DiffLineKind::Removed,
                DiffLineKind::Added,
                DiffLineKind::Unchanged,
                DiffLineKind::Added
            ]
        );
        assert_eq!(diff.lines[1].text, "answer briefly");
        assert_eq!(diff.lines[2].text, "answer with detail");
    }

    #[test]
    fn tenant_and_project_scope_is_isolated() {
        let store = registry();
        let tenant_a = tenant("tenant-a");
        let tenant_b = tenant("tenant-b");
        let project_a = project("project-a");
        let project_b = project("project-b");
        let created = store
            .create_prompt(create_request(
                tenant_a.clone(),
                project_a.clone(),
                "Answer {{question}}",
            ))
            .unwrap_or_else(|err| panic!("{err}"));

        let Err(wrong_tenant) =
            store.list_versions(&tenant_b, &project_a, &created.prompt.prompt_id)
        else {
            panic!("wrong tenant should not see prompt versions");
        };
        assert_eq!(
            wrong_tenant,
            PromptRegistryError::PromptNotFound {
                prompt_id: created.prompt.prompt_id.clone()
            }
        );

        let Err(wrong_project) =
            store.get_latest_version(&tenant_a, &project_b, &created.prompt.prompt_id)
        else {
            panic!("wrong project should not see latest prompt version");
        };
        assert_eq!(
            wrong_project,
            PromptRegistryError::PromptNotFound {
                prompt_id: created.prompt.prompt_id.clone()
            }
        );
    }
}
