use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    AnnotationId, DatasetCaseId, DatasetId, ProjectId, ReviewQueueId, ReviewTaskId, SpanId,
    TenantId, Timestamp, TraceId,
};
use beater_datasets::{promote_trace_span_to_case, DatasetCase};
use beater_schema::TraceView;
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReviewQueue {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub queue_id: ReviewQueueId,
    pub name: String,
    #[schema(value_type = serde_json::Value)]
    pub annotation_schema: Value,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewTaskState {
    Open,
    Submitted,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Pass,
    Fail,
    NeedsFix,
    Unsure,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReviewTask {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub queue_id: ReviewQueueId,
    pub task_id: ReviewTaskId,
    pub trace_id: TraceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<SpanId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<DatasetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_case_id: Option<DatasetCaseId>,
    pub priority: i64,
    pub state: ReviewTaskState,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReviewAnnotation {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub queue_id: ReviewQueueId,
    pub task_id: ReviewTaskId,
    pub annotation_id: AnnotationId,
    pub reviewer_id: String,
    pub verdict: ReviewVerdict,
    #[schema(value_type = serde_json::Value)]
    pub payload: Value,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateReviewQueueRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub queue_id: Option<ReviewQueueId>,
    pub name: String,
    pub annotation_schema: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnqueueReviewTaskRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub queue_id: ReviewQueueId,
    pub task_id: Option<ReviewTaskId>,
    pub trace_id: TraceId,
    pub span_id: Option<SpanId>,
    pub dataset_id: Option<DatasetId>,
    pub dataset_case_id: Option<DatasetCaseId>,
    pub priority: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubmitAnnotationRequest {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub queue_id: ReviewQueueId,
    pub task_id: ReviewTaskId,
    pub annotation_id: Option<AnnotationId>,
    pub reviewer_id: String,
    pub verdict: ReviewVerdict,
    pub payload: Value,
}

#[async_trait]
pub trait HumanReviewStore: Send + Sync {
    async fn create_queue(&self, request: CreateReviewQueueRequest) -> StoreResult<ReviewQueue>;

    async fn get_queue(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
    ) -> StoreResult<ReviewQueue>;

    async fn enqueue_task(&self, request: EnqueueReviewTaskRequest) -> StoreResult<ReviewTask>;

    async fn get_task(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        task_id: ReviewTaskId,
    ) -> StoreResult<ReviewTask>;

    async fn list_tasks(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        state: Option<ReviewTaskState>,
    ) -> StoreResult<Vec<ReviewTask>>;

    async fn submit_annotation(
        &self,
        request: SubmitAnnotationRequest,
    ) -> StoreResult<ReviewAnnotation>;

    async fn get_annotation(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        task_id: ReviewTaskId,
        annotation_id: AnnotationId,
    ) -> StoreResult<ReviewAnnotation>;

    async fn list_annotations(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        task_id: ReviewTaskId,
    ) -> StoreResult<Vec<ReviewAnnotation>>;
}

#[derive(Clone)]
pub struct SqliteHumanReviewStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteHumanReviewStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory review sqlite")?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create review sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite review store {}", path.display()))?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
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

                CREATE TABLE IF NOT EXISTS review_queues (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    queue_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    queue_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, queue_id)
                );

                CREATE TABLE IF NOT EXISTS review_tasks (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    queue_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    trace_id TEXT NOT NULL,
                    span_id TEXT,
                    dataset_id TEXT,
                    dataset_case_id TEXT,
                    priority INTEGER NOT NULL,
                    state TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    task_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, queue_id, task_id)
                );

                CREATE INDEX IF NOT EXISTS idx_review_tasks_queue_state
                  ON review_tasks (tenant_id, project_id, queue_id, state, priority DESC, created_at ASC);

                CREATE TABLE IF NOT EXISTS review_annotations (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    queue_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    annotation_id TEXT NOT NULL,
                    reviewer_id TEXT NOT NULL,
                    verdict TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    annotation_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, queue_id, task_id, annotation_id)
                );

                CREATE INDEX IF NOT EXISTS idx_review_annotations_task
                  ON review_annotations (tenant_id, project_id, queue_id, task_id, created_at ASC);
                "#,
            )
            .context("initialize sqlite review store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("sqlite review connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl HumanReviewStore for SqliteHumanReviewStore {
    async fn create_queue(&self, request: CreateReviewQueueRequest) -> StoreResult<ReviewQueue> {
        let queue_id = match request.queue_id {
            Some(queue_id) => queue_id,
            None => ReviewQueueId::new(Uuid::new_v4().to_string()).map_err(StoreError::backend)?,
        };
        let queue = ReviewQueue {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            queue_id,
            name: request.name,
            annotation_schema: request.annotation_schema,
            created_at: Utc::now(),
        };
        let queue_json = serde_json::to_string(&queue)
            .context("serialize review queue")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO review_queues
                  (tenant_id, project_id, queue_id, name, created_at, queue_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    queue.tenant_id.as_str(),
                    queue.project_id.as_str(),
                    queue.queue_id.as_str(),
                    queue.name.as_str(),
                    queue.created_at.to_rfc3339(),
                    queue_json
                ],
            )
            .context("insert review queue")
            .into_store()?;
        Ok(queue)
    }

    async fn get_queue(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
    ) -> StoreResult<ReviewQueue> {
        let connection = self.lock().into_store()?;
        let queue_json = connection
            .query_row(
                r#"
                SELECT queue_json
                FROM review_queues
                WHERE tenant_id = ?1 AND project_id = ?2 AND queue_id = ?3
                "#,
                params![tenant_id.as_str(), project_id.as_str(), queue_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query review queue")
            .into_store()?
            .ok_or_else(|| {
                StoreError::NotFound(format!("review queue {} not found", queue_id.as_str()))
            })?;
        serde_json::from_str(&queue_json)
            .context("decode review queue")
            .into_store()
    }

    async fn enqueue_task(&self, request: EnqueueReviewTaskRequest) -> StoreResult<ReviewTask> {
        let queue = self
            .get_queue(
                request.tenant_id.clone(),
                request.project_id.clone(),
                request.queue_id.clone(),
            )
            .await?;
        let task_id = match request.task_id {
            Some(task_id) => task_id,
            None => ReviewTaskId::new(Uuid::new_v4().to_string()).map_err(StoreError::backend)?,
        };
        let now = Utc::now();
        let task = ReviewTask {
            tenant_id: queue.tenant_id,
            project_id: queue.project_id,
            queue_id: queue.queue_id,
            task_id,
            trace_id: request.trace_id,
            span_id: request.span_id,
            dataset_id: request.dataset_id,
            dataset_case_id: request.dataset_case_id,
            priority: request.priority,
            state: ReviewTaskState::Open,
            created_at: now,
            updated_at: now,
        };
        let task_json = serde_json::to_string(&task)
            .context("serialize review task")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO review_tasks
                  (tenant_id, project_id, queue_id, task_id, trace_id, span_id, dataset_id,
                   dataset_case_id, priority, state, created_at, updated_at, task_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
                params![
                    task.tenant_id.as_str(),
                    task.project_id.as_str(),
                    task.queue_id.as_str(),
                    task.task_id.as_str(),
                    task.trace_id.as_str(),
                    task.span_id.as_ref().map(|id| id.as_str()),
                    task.dataset_id.as_ref().map(|id| id.as_str()),
                    task.dataset_case_id.as_ref().map(|id| id.as_str()),
                    task.priority,
                    task_state_name(&task.state),
                    task.created_at.to_rfc3339(),
                    task.updated_at.to_rfc3339(),
                    task_json
                ],
            )
            .context("insert review task")
            .into_store()?;
        Ok(task)
    }

    async fn get_task(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        task_id: ReviewTaskId,
    ) -> StoreResult<ReviewTask> {
        let connection = self.lock().into_store()?;
        get_task_locked(&connection, &tenant_id, &project_id, &queue_id, &task_id).into_store()
    }

    async fn list_tasks(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        state: Option<ReviewTaskState>,
    ) -> StoreResult<Vec<ReviewTask>> {
        let connection = self.lock().into_store()?;
        let state_filter = state.as_ref().map(task_state_name);
        let mut statement = connection
            .prepare(
                r#"
                SELECT task_json
                FROM review_tasks
                WHERE tenant_id = ?1 AND project_id = ?2 AND queue_id = ?3
                  AND (?4 IS NULL OR state = ?4)
                ORDER BY priority DESC, created_at ASC, task_id ASC
                "#,
            )
            .context("prepare review task list")
            .into_store()?;
        let rows = statement
            .query_map(
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    queue_id.as_str(),
                    state_filter
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query review tasks")
            .into_store()?;
        let mut tasks = Vec::new();
        for row in rows {
            let task_json = row.context("read review task row").into_store()?;
            tasks.push(
                serde_json::from_str(&task_json)
                    .context("decode review task")
                    .into_store()?,
            );
        }
        Ok(tasks)
    }

    async fn submit_annotation(
        &self,
        request: SubmitAnnotationRequest,
    ) -> StoreResult<ReviewAnnotation> {
        let now = Utc::now();
        let mut task = self
            .get_task(
                request.tenant_id.clone(),
                request.project_id.clone(),
                request.queue_id.clone(),
                request.task_id.clone(),
            )
            .await?;
        let annotation_id = match request.annotation_id {
            Some(annotation_id) => annotation_id,
            None => AnnotationId::new(Uuid::new_v4().to_string()).map_err(StoreError::backend)?,
        };
        let annotation = ReviewAnnotation {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            queue_id: request.queue_id,
            task_id: request.task_id,
            annotation_id,
            reviewer_id: request.reviewer_id,
            verdict: request.verdict,
            payload: request.payload,
            created_at: now,
        };
        task.state = ReviewTaskState::Submitted;
        task.updated_at = now;
        let task_json = serde_json::to_string(&task)
            .context("serialize submitted review task")
            .into_store()?;
        let annotation_json = serde_json::to_string(&annotation)
            .context("serialize review annotation")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO review_annotations
                  (tenant_id, project_id, queue_id, task_id, annotation_id, reviewer_id,
                   verdict, created_at, annotation_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    annotation.tenant_id.as_str(),
                    annotation.project_id.as_str(),
                    annotation.queue_id.as_str(),
                    annotation.task_id.as_str(),
                    annotation.annotation_id.as_str(),
                    annotation.reviewer_id.as_str(),
                    verdict_name(&annotation.verdict),
                    annotation.created_at.to_rfc3339(),
                    annotation_json
                ],
            )
            .context("insert review annotation")
            .into_store()?;
        connection
            .execute(
                r#"
                UPDATE review_tasks
                SET state = ?1, updated_at = ?2, task_json = ?3
                WHERE tenant_id = ?4 AND project_id = ?5 AND queue_id = ?6 AND task_id = ?7
                "#,
                params![
                    task_state_name(&task.state),
                    task.updated_at.to_rfc3339(),
                    task_json,
                    task.tenant_id.as_str(),
                    task.project_id.as_str(),
                    task.queue_id.as_str(),
                    task.task_id.as_str()
                ],
            )
            .context("mark review task submitted")
            .into_store()?;
        Ok(annotation)
    }

    async fn get_annotation(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        task_id: ReviewTaskId,
        annotation_id: AnnotationId,
    ) -> StoreResult<ReviewAnnotation> {
        let connection = self.lock().into_store()?;
        let annotation_json = connection
            .query_row(
                r#"
                SELECT annotation_json
                FROM review_annotations
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND queue_id = ?3
                  AND task_id = ?4
                  AND annotation_id = ?5
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    queue_id.as_str(),
                    task_id.as_str(),
                    annotation_id.as_str()
                ],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query review annotation")
            .into_store()?
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "review annotation {} not found",
                    annotation_id.as_str()
                ))
            })?;
        serde_json::from_str(&annotation_json)
            .context("decode review annotation")
            .into_store()
    }

    async fn list_annotations(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        queue_id: ReviewQueueId,
        task_id: ReviewTaskId,
    ) -> StoreResult<Vec<ReviewAnnotation>> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT annotation_json
                FROM review_annotations
                WHERE tenant_id = ?1 AND project_id = ?2 AND queue_id = ?3 AND task_id = ?4
                ORDER BY created_at ASC, annotation_id ASC
                "#,
            )
            .context("prepare review annotation list")
            .into_store()?;
        let rows = statement
            .query_map(
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    queue_id.as_str(),
                    task_id.as_str()
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query review annotations")
            .into_store()?;
        let mut annotations = Vec::new();
        for row in rows {
            let annotation_json = row.context("read review annotation row").into_store()?;
            annotations.push(
                serde_json::from_str(&annotation_json)
                    .context("decode review annotation")
                    .into_store()?,
            );
        }
        Ok(annotations)
    }
}

pub fn promote_review_annotation_to_dataset_case(
    tenant_id: TenantId,
    project_id: ProjectId,
    dataset_id: DatasetId,
    trace: &TraceView,
    task: &ReviewTask,
    annotation: &ReviewAnnotation,
    reference_override: Option<Value>,
) -> anyhow::Result<DatasetCase> {
    if task.tenant_id != tenant_id || task.project_id != project_id {
        return Err(anyhow!("review task crosses tenant/project boundary"));
    }
    if annotation.tenant_id != tenant_id
        || annotation.project_id != project_id
        || annotation.queue_id != task.queue_id
        || annotation.task_id != task.task_id
    {
        return Err(anyhow!(
            "review annotation {} does not belong to task {}",
            annotation.annotation_id.as_str(),
            task.task_id.as_str()
        ));
    }
    if trace.trace_id != task.trace_id {
        return Err(anyhow!(
            "review task {} references trace {}, but loaded trace is {}",
            task.task_id.as_str(),
            task.trace_id.as_str(),
            trace.trace_id.as_str()
        ));
    }
    let reference = reference_override
        .and_then(usable_reference)
        .or_else(|| annotation_reference(annotation))
        .ok_or_else(|| {
            anyhow!(
                "review annotation {} requires a non-null reference or expected value before dataset promotion",
                annotation.annotation_id.as_str()
            )
        })?;
    promote_trace_span_to_case(
        tenant_id,
        project_id,
        dataset_id,
        trace,
        task.span_id.clone(),
        Some(reference),
    )
}

fn get_task_locked(
    connection: &Connection,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    queue_id: &ReviewQueueId,
    task_id: &ReviewTaskId,
) -> anyhow::Result<ReviewTask> {
    let task_json = connection
        .query_row(
            r#"
            SELECT task_json
            FROM review_tasks
            WHERE tenant_id = ?1 AND project_id = ?2 AND queue_id = ?3 AND task_id = ?4
            "#,
            params![
                tenant_id.as_str(),
                project_id.as_str(),
                queue_id.as_str(),
                task_id.as_str()
            ],
            |row| row.get::<_, String>(0),
        )
        .with_context(|| format!("review task {} not found", task_id.as_str()))?;
    serde_json::from_str(&task_json).context("decode review task")
}

fn annotation_reference(annotation: &ReviewAnnotation) -> Option<Value> {
    annotation
        .payload
        .get("reference")
        .cloned()
        .and_then(usable_reference)
        .or_else(|| annotation.payload.get("expected").cloned())
        .and_then(usable_reference)
}

fn usable_reference(reference: Value) -> Option<Value> {
    if reference.is_null() {
        None
    } else {
        Some(reference)
    }
}

fn task_state_name(state: &ReviewTaskState) -> &'static str {
    match state {
        ReviewTaskState::Open => "open",
        ReviewTaskState::Submitted => "submitted",
        ReviewTaskState::Cancelled => "cancelled",
    }
}

fn verdict_name(verdict: &ReviewVerdict) -> &'static str {
    match verdict {
        ReviewVerdict::Pass => "pass",
        ReviewVerdict::Fail => "fail",
        ReviewVerdict::NeedsFix => "needs_fix",
        ReviewVerdict::Unsure => "unsure",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{ArtifactId, EnvironmentId, Money, Sha256Hash, TokenCounts};
    use beater_schema::{
        AgentSpanKind, ArtifactRef, CanonicalSpan, ModelRef, RedactionClass, SpanStatus,
        CANONICAL_SCHEMA_VERSION,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn review_store_round_trips_queue_task_and_annotation() -> anyhow::Result<()> {
        let store = SqliteHumanReviewStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let queue = store
            .create_queue(CreateReviewQueueRequest {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                queue_id: Some(ReviewQueueId::new("quality")?),
                name: "quality".to_string(),
                annotation_schema: json!({"type":"object"}),
            })
            .await?;
        let task = store
            .enqueue_task(EnqueueReviewTaskRequest {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                queue_id: queue.queue_id.clone(),
                task_id: Some(ReviewTaskId::new("task-1")?),
                trace_id: TraceId::new("trace-1")?,
                span_id: Some(SpanId::new("span-1")?),
                dataset_id: None,
                dataset_case_id: None,
                priority: 10,
            })
            .await?;

        let open = store
            .list_tasks(
                tenant.clone(),
                project.clone(),
                queue.queue_id.clone(),
                Some(ReviewTaskState::Open),
            )
            .await?;
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].task_id, task.task_id);

        let annotation = store
            .submit_annotation(SubmitAnnotationRequest {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                queue_id: queue.queue_id.clone(),
                task_id: task.task_id.clone(),
                annotation_id: Some(AnnotationId::new("annotation-1")?),
                reviewer_id: "reviewer-a".to_string(),
                verdict: ReviewVerdict::Fail,
                payload: json!({"reference": "expected answer", "notes": "wrong answer"}),
            })
            .await?;

        let submitted = store
            .get_task(tenant, project, queue.queue_id, task.task_id)
            .await?;
        assert_eq!(submitted.state, ReviewTaskState::Submitted);
        assert_eq!(annotation.verdict, ReviewVerdict::Fail);
        Ok(())
    }

    #[tokio::test]
    async fn review_task_listing_isolates_scope() -> anyhow::Result<()> {
        let store = SqliteHumanReviewStore::in_memory()?;
        let tenant = TenantId::new("tenant")?;
        let other_tenant = TenantId::new("other-tenant")?;
        let project = ProjectId::new("project")?;
        let other_project = ProjectId::new("other-project")?;

        let target_queue = create_queue(&store, &tenant, &project, "target").await?;
        let sibling_queue = create_queue(&store, &tenant, &project, "sibling").await?;
        let other_project_queue =
            create_queue(&store, &tenant, &other_project, "other-project-queue").await?;
        let other_tenant_queue =
            create_queue(&store, &other_tenant, &project, "other-tenant-queue").await?;

        enqueue_task(&store, &tenant, &project, &target_queue, "target-task", 10).await?;
        enqueue_task(&store, &tenant, &project, &sibling_queue, "queue-leak", 20).await?;
        enqueue_task(
            &store,
            &tenant,
            &other_project,
            &other_project_queue,
            "project-leak",
            30,
        )
        .await?;
        enqueue_task(
            &store,
            &other_tenant,
            &project,
            &other_tenant_queue,
            "tenant-leak",
            40,
        )
        .await?;

        let tasks = store
            .list_tasks(
                tenant,
                project,
                target_queue.queue_id,
                Some(ReviewTaskState::Open),
            )
            .await?;

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_id.as_str(), "target-task");
        Ok(())
    }

    #[tokio::test]
    async fn review_annotation_promotes_to_dataset_case_with_human_reference() -> anyhow::Result<()>
    {
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let queue = ReviewQueueId::new("quality")?;
        let task = fixture_review_task(&tenant, &project, &queue)?;
        let annotation = fixture_annotation(
            &tenant,
            &project,
            &queue,
            &task,
            "annotation-1",
            json!({"reference": {"answer": "world"}}),
        )?;
        let trace = fixture_trace(&tenant, &project)?;
        let case = promote_review_annotation_to_dataset_case(
            tenant,
            project,
            DatasetId::new("dataset")?,
            &trace,
            &task,
            &annotation,
            None,
        )?;

        assert_eq!(case.reference, Some(json!({"answer": "world"})));
        assert_eq!(case.source_span_id.as_str(), "span-1");
        Ok(())
    }

    #[tokio::test]
    async fn review_annotation_promotes_to_dataset_case_with_expected_value() -> anyhow::Result<()>
    {
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let queue = ReviewQueueId::new("quality")?;
        let task = fixture_review_task(&tenant, &project, &queue)?;
        let annotation = fixture_annotation(
            &tenant,
            &project,
            &queue,
            &task,
            "annotation-expected",
            json!({"expected": {"answer": "world"}}),
        )?;
        let trace = fixture_trace(&tenant, &project)?;
        let case = promote_review_annotation_to_dataset_case(
            tenant,
            project,
            DatasetId::new("dataset")?,
            &trace,
            &task,
            &annotation,
            None,
        )?;

        assert_eq!(case.reference, Some(json!({"answer": "world"})));
        Ok(())
    }

    #[tokio::test]
    async fn review_annotation_promotion_allows_reference_override() -> anyhow::Result<()> {
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let queue = ReviewQueueId::new("quality")?;
        let task = fixture_review_task(&tenant, &project, &queue)?;
        let annotation = fixture_annotation(
            &tenant,
            &project,
            &queue,
            &task,
            "annotation-override",
            json!({"notes": "use supplied reference"}),
        )?;
        let trace = fixture_trace(&tenant, &project)?;
        let case = promote_review_annotation_to_dataset_case(
            tenant,
            project,
            DatasetId::new("dataset")?,
            &trace,
            &task,
            &annotation,
            Some(json!({"answer": "override"})),
        )?;

        assert_eq!(case.reference, Some(json!({"answer": "override"})));
        Ok(())
    }

    #[tokio::test]
    async fn review_annotation_promotion_rejects_missing_reference() -> anyhow::Result<()> {
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let queue = ReviewQueueId::new("quality")?;
        let task = fixture_review_task(&tenant, &project, &queue)?;
        let annotation = fixture_annotation(
            &tenant,
            &project,
            &queue,
            &task,
            "annotation-missing-reference",
            json!({"reference": null, "expected": null}),
        )?;
        let trace = fixture_trace(&tenant, &project)?;

        let error = match promote_review_annotation_to_dataset_case(
            tenant,
            project,
            DatasetId::new("dataset")?,
            &trace,
            &task,
            &annotation,
            None,
        ) {
            Ok(_) => panic!("promotion should reject an annotation without a usable reference"),
            Err(error) => error,
        };

        assert!(error
            .to_string()
            .contains("requires a non-null reference or expected value"));
        Ok(())
    }

    fn fixture_review_task(
        tenant: &TenantId,
        project: &ProjectId,
        queue: &ReviewQueueId,
    ) -> anyhow::Result<ReviewTask> {
        Ok(ReviewTask {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            queue_id: queue.clone(),
            task_id: ReviewTaskId::new("task-1")?,
            trace_id: TraceId::new("trace-1")?,
            span_id: Some(SpanId::new("span-1")?),
            dataset_id: None,
            dataset_case_id: None,
            priority: 1,
            state: ReviewTaskState::Submitted,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn create_queue(
        store: &SqliteHumanReviewStore,
        tenant: &TenantId,
        project: &ProjectId,
        queue_id: &str,
    ) -> anyhow::Result<ReviewQueue> {
        Ok(store
            .create_queue(CreateReviewQueueRequest {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                queue_id: Some(ReviewQueueId::new(queue_id)?),
                name: queue_id.to_string(),
                annotation_schema: json!({"type": "object"}),
            })
            .await?)
    }

    async fn enqueue_task(
        store: &SqliteHumanReviewStore,
        tenant: &TenantId,
        project: &ProjectId,
        queue: &ReviewQueue,
        task_id: &str,
        priority: i64,
    ) -> anyhow::Result<ReviewTask> {
        Ok(store
            .enqueue_task(EnqueueReviewTaskRequest {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                queue_id: queue.queue_id.clone(),
                task_id: Some(ReviewTaskId::new(task_id)?),
                trace_id: TraceId::new(format!("trace-{task_id}"))?,
                span_id: Some(SpanId::new(format!("span-{task_id}"))?),
                dataset_id: None,
                dataset_case_id: None,
                priority,
            })
            .await?)
    }

    fn fixture_annotation(
        tenant: &TenantId,
        project: &ProjectId,
        queue: &ReviewQueueId,
        task: &ReviewTask,
        annotation_id: &str,
        payload: Value,
    ) -> anyhow::Result<ReviewAnnotation> {
        Ok(ReviewAnnotation {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            queue_id: queue.clone(),
            task_id: task.task_id.clone(),
            annotation_id: AnnotationId::new(annotation_id)?,
            reviewer_id: "reviewer-a".to_string(),
            verdict: ReviewVerdict::NeedsFix,
            payload,
            created_at: Utc::now(),
        })
    }

    fn fixture_trace(tenant: &TenantId, project: &ProjectId) -> anyhow::Result<TraceView> {
        let mut attributes = BTreeMap::new();
        attributes.insert("input.value".to_string(), json!("question"));
        attributes.insert("output.value".to_string(), json!({"answer": "wrong"}));
        Ok(TraceView {
            tenant_id: tenant.clone(),
            trace_id: TraceId::new("trace-1")?,
            spans: vec![CanonicalSpan {
                schema_version: CANONICAL_SCHEMA_VERSION,
                normalizer_version: "test".to_string(),
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                environment_id: EnvironmentId::new("prod")?,
                trace_id: TraceId::new("trace-1")?,
                span_id: SpanId::new("span-1")?,
                parent_span_id: None,
                seq: 1,
                kind: AgentSpanKind::AgentRun,
                name: "agent".to_string(),
                status: SpanStatus::Ok,
                start_time: Utc::now(),
                end_time: Some(Utc::now()),
                model: Some(ModelRef {
                    provider: "openai".to_string(),
                    name: "model".to_string(),
                }),
                cost: Some(Money::usd_micros(12)),
                tokens: Some(TokenCounts {
                    input: 1,
                    output: 2,
                    reasoning: 0,
                    cache_read: 0,
                }),
                input_ref: None,
                output_ref: None,
                attributes,
                unmapped_attrs: json!({}),
                raw_ref: ArtifactRef {
                    artifact_id: ArtifactId::new("raw")?,
                    uri: "artifact://tenant/project/raw".to_string(),
                    sha256: Sha256Hash::new("11".repeat(32))?,
                    size_bytes: 2,
                    mime_type: "application/json".to_string(),
                    redaction_class: RedactionClass::Internal,
                },
            }],
        })
    }
}
