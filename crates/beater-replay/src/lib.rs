use anyhow::{anyhow, Context};
use beater_core::{sha256_json_hash, ProjectId, Sha256Hash, SpanId, TenantId, Timestamp, TraceId};
use beater_schema::{CanonicalSpan, ReplayCassette, SpanStatus};
use beater_store::{IntoStoreResult, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayMode {
    DeterministicReplay,
    ForkedReplay,
    Simulation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPlan {
    pub trace_id: TraceId,
    pub mode: ReplayMode,
    pub guarantee: String,
    pub missing_required_kinds: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayEventKind {
    Provider,
    Tool,
    Memory,
    Retrieval,
    Clock,
    Random,
}

impl ReplayEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::Tool => "tool",
            Self::Memory => "memory",
            Self::Retrieval => "retrieval",
            Self::Clock => "clock",
            Self::Random => "random",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub seq: u64,
    pub kind: ReplayEventKind,
    pub request: Value,
    pub response: Value,
    pub request_hash: Sha256Hash,
    pub response_hash: Sha256Hash,
    pub recorded_at: Timestamp,
}

impl ReplayEvent {
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        trace_id: TraceId,
        seq: u64,
        kind: ReplayEventKind,
        request: Value,
        response: Value,
    ) -> anyhow::Result<Self> {
        let request_hash = json_hash(&request)?;
        let response_hash = json_hash(&response)?;
        Ok(Self {
            tenant_id,
            project_id,
            trace_id,
            seq,
            kind,
            request,
            response,
            request_hash,
            response_hash,
            recorded_at: Utc::now(),
        })
    }
}

#[derive(Clone)]
pub struct SqliteReplayStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteReplayStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory().context("open in-memory replay sqlite")?;
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
                .with_context(|| format!("create replay sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open replay sqlite {}", path.display()))?;
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

                CREATE TABLE IF NOT EXISTS replay_events (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    trace_id TEXT NOT NULL,
                    seq INTEGER NOT NULL,
                    kind TEXT NOT NULL,
                    request_hash TEXT NOT NULL,
                    response_hash TEXT NOT NULL,
                    recorded_at TEXT NOT NULL,
                    event_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, trace_id, seq, kind, request_hash)
                );

                CREATE INDEX IF NOT EXISTS idx_replay_events_trace_order
                ON replay_events (tenant_id, project_id, trace_id, seq);
                "#,
            )
            .context("initialize replay sqlite")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("replay sqlite connection mutex poisoned: {err}"))
    }

    pub async fn cassette(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        trace_id: TraceId,
    ) -> StoreResult<ReplayCassette> {
        let events = self
            .list_events(tenant_id.clone(), project_id, trace_id.clone())
            .await?;
        Ok(cassette_from_events(tenant_id, trace_id, &events))
    }

    pub async fn put_event(&self, event: ReplayEvent) -> StoreResult<ReplayEvent> {
        let event_json = serde_json::to_string(&event)
            .context("serialize replay event")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT OR IGNORE INTO replay_events
                  (tenant_id, project_id, trace_id, seq, kind, request_hash, response_hash,
                   recorded_at, event_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    event.tenant_id.as_str(),
                    event.project_id.as_str(),
                    event.trace_id.as_str(),
                    event.seq as i64,
                    event.kind.as_str(),
                    event.request_hash.as_str(),
                    event.response_hash.as_str(),
                    event.recorded_at.to_rfc3339(),
                    event_json,
                ],
            )
            .context("insert replay event")
            .into_store()?;
        Ok(event)
    }

    pub async fn list_events(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        trace_id: TraceId,
    ) -> StoreResult<Vec<ReplayEvent>> {
        let connection = self.lock().into_store()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT event_json
                FROM replay_events
                WHERE tenant_id = ?1 AND project_id = ?2 AND trace_id = ?3
                ORDER BY seq ASC, kind ASC, request_hash ASC
                "#,
            )
            .context("prepare replay event list")
            .into_store()?;
        let rows = statement
            .query_map(
                params![tenant_id.as_str(), project_id.as_str(), trace_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .context("query replay events")
            .into_store()?;
        let mut events = Vec::new();
        for row in rows {
            let json = row.context("read replay event row").into_store()?;
            events.push(
                serde_json::from_str::<ReplayEvent>(&json)
                    .context("decode replay event")
                    .into_store()?,
            );
        }
        Ok(events)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayStep {
    pub seq: u64,
    pub kind: ReplayEventKind,
    pub request: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayScenario {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub steps: Vec<ReplayStep>,
    pub fork_after_seq: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayedStep {
    pub seq: u64,
    pub kind: ReplayEventKind,
    pub request_hash: Sha256Hash,
    pub response_hash: Sha256Hash,
    pub response: Value,
    pub replayed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayRunReport {
    pub replay_run_id: String,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub trace_id: TraceId,
    pub plan: ReplayPlan,
    pub replayed_steps: Vec<ReplayedStep>,
    pub live_steps_required: Vec<ReplayStep>,
    pub created_at: Timestamp,
}

pub fn execute_replay(
    cassette: &ReplayCassette,
    events: &[ReplayEvent],
    scenario: ReplayScenario,
) -> anyhow::Result<ReplayRunReport> {
    if cassette.tenant_id.as_str() != scenario.tenant_id.as_str()
        || cassette.trace_id.as_str() != scenario.trace_id.as_str()
    {
        return Err(anyhow!(
            "replay scenario crosses cassette tenant/trace boundary"
        ));
    }
    if events.iter().any(|event| {
        event.tenant_id.as_str() != scenario.tenant_id.as_str()
            || event.project_id.as_str() != scenario.project_id.as_str()
            || event.trace_id.as_str() != scenario.trace_id.as_str()
    }) {
        return Err(anyhow!("replay event crosses scenario boundary"));
    }

    let fork_after = scenario
        .fork_after_seq
        .map(|seq| SpanId::new(format!("fork-after-seq-{seq}")))
        .transpose()?;
    let plan = plan_replay(cassette, fork_after);
    let by_key = event_index(events);
    let mut replayed_steps = Vec::new();
    let mut live_steps_required = Vec::new();

    for step in scenario.steps {
        if scenario
            .fork_after_seq
            .is_some_and(|fork_after_seq| step.seq > fork_after_seq)
        {
            live_steps_required.push(step);
            continue;
        }
        let request_hash = json_hash(&step.request)?;
        let key = event_key(step.seq, &step.kind, &request_hash);
        let Some(event) = by_key.get(&key) else {
            if plan.mode == ReplayMode::DeterministicReplay {
                return Err(anyhow!(
                    "deterministic replay missing event seq={} kind={} request_hash={}",
                    step.seq,
                    step.kind.as_str(),
                    request_hash.as_str()
                ));
            }
            live_steps_required.push(step);
            continue;
        };
        replayed_steps.push(ReplayedStep {
            seq: step.seq,
            kind: step.kind,
            request_hash,
            response_hash: event.response_hash.clone(),
            response: event.response.clone(),
            replayed: true,
        });
    }

    Ok(ReplayRunReport {
        replay_run_id: Uuid::new_v4().to_string(),
        tenant_id: scenario.tenant_id,
        project_id: scenario.project_id,
        trace_id: scenario.trace_id,
        plan,
        replayed_steps,
        live_steps_required,
        created_at: Utc::now(),
    })
}

pub fn cassette_from_events(
    tenant_id: TenantId,
    trace_id: TraceId,
    events: &[ReplayEvent],
) -> ReplayCassette {
    let mut counts = BTreeMap::<ReplayEventKind, usize>::new();
    for event in events {
        *counts.entry(event.kind.clone()).or_default() += 1;
    }
    let required = BTreeSet::from([
        ReplayEventKind::Provider,
        ReplayEventKind::Tool,
        ReplayEventKind::Memory,
        ReplayEventKind::Retrieval,
        ReplayEventKind::Clock,
        ReplayEventKind::Random,
    ]);
    let observed = counts.keys().cloned().collect::<BTreeSet<_>>();
    let missing_required_kinds = required
        .difference(&observed)
        .map(|kind| kind.as_str().to_string())
        .collect();
    ReplayCassette {
        tenant_id,
        trace_id,
        provider_events: *counts.get(&ReplayEventKind::Provider).unwrap_or(&0),
        tool_events: *counts.get(&ReplayEventKind::Tool).unwrap_or(&0),
        memory_events: *counts.get(&ReplayEventKind::Memory).unwrap_or(&0),
        retrieval_events: *counts.get(&ReplayEventKind::Retrieval).unwrap_or(&0),
        clock_events: *counts.get(&ReplayEventKind::Clock).unwrap_or(&0),
        random_events: *counts.get(&ReplayEventKind::Random).unwrap_or(&0),
        missing_required_kinds,
    }
}

fn event_index(events: &[ReplayEvent]) -> BTreeMap<String, ReplayEvent> {
    events
        .iter()
        .map(|event| {
            (
                event_key(event.seq, &event.kind, &event.request_hash),
                event.clone(),
            )
        })
        .collect()
}

fn event_key(seq: u64, kind: &ReplayEventKind, request_hash: &Sha256Hash) -> String {
    format!("{seq}:{}:{}", kind.as_str(), request_hash.as_str())
}

fn json_hash(value: &Value) -> anyhow::Result<Sha256Hash> {
    sha256_json_hash(value).context("serialize replay json for hash")
}

pub fn plan_replay(cassette: &ReplayCassette, fork_after: Option<SpanId>) -> ReplayPlan {
    let mode = if cassette.missing_required_kinds.is_empty() && fork_after.is_none() {
        ReplayMode::DeterministicReplay
    } else if fork_after.is_some() {
        ReplayMode::ForkedReplay
    } else {
        ReplayMode::Simulation
    };
    let guarantee = match mode {
        ReplayMode::DeterministicReplay => {
            "all required cassettes are present; replay should not call live providers".to_string()
        }
        ReplayMode::ForkedReplay => {
            "captured trace is reused until fork point; live providers/tools may run after fork"
                .to_string()
        }
        ReplayMode::Simulation => {
            "one or more cassette kinds are missing; substitutes or live calls are required"
                .to_string()
        }
    };

    ReplayPlan {
        trace_id: cassette.trace_id.clone(),
        mode,
        guarantee,
        missing_required_kinds: cassette.missing_required_kinds.clone(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpanEvidence {
    pub span_id: SpanId,
    pub score: f64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FailureAttribution {
    pub trace_id: TraceId,
    pub root_cause_span_id: Option<SpanId>,
    pub confidence: f64,
    pub evidence: Vec<SpanEvidence>,
}

pub fn attribute_failure(
    trace_id: TraceId,
    spans: &[CanonicalSpan],
    evidence: &[SpanEvidence],
) -> FailureAttribution {
    let mut sorted_spans = spans.to_vec();
    sorted_spans.sort_by_key(|span| span.seq);

    for span in &sorted_spans {
        if span.status == SpanStatus::Error {
            return FailureAttribution {
                trace_id,
                root_cause_span_id: Some(span.span_id.clone()),
                confidence: 0.8,
                evidence: evidence.to_vec(),
            };
        }
        if let Some(score) = evidence.iter().find(|item| item.span_id == span.span_id) {
            if score.score < 0.5 {
                return FailureAttribution {
                    trace_id,
                    root_cause_span_id: Some(span.span_id.clone()),
                    confidence: 0.65,
                    evidence: evidence.to_vec(),
                };
            }
        }
    }

    FailureAttribution {
        trace_id,
        root_cause_span_id: None,
        confidence: 0.0,
        evidence: evidence.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{EnvironmentId, ProjectId, TenantId};
    use beater_schema::{AgentSpanKind, CANONICAL_SCHEMA_VERSION};
    use chrono::Utc;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn replay_mode_is_honest_about_cassette_completeness() {
        let complete = ReplayCassette {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            provider_events: 1,
            tool_events: 1,
            memory_events: 1,
            retrieval_events: 1,
            clock_events: 1,
            random_events: 1,
            missing_required_kinds: Vec::new(),
        };
        assert_eq!(
            plan_replay(&complete, None).mode,
            ReplayMode::DeterministicReplay
        );
        assert_eq!(
            plan_replay(
                &complete,
                Some(SpanId::new("fork").unwrap_or_else(|err| panic!("{err}")))
            )
            .mode,
            ReplayMode::ForkedReplay
        );

        let missing = ReplayCassette {
            missing_required_kinds: vec!["tool".to_string()],
            ..complete
        };
        assert_eq!(plan_replay(&missing, None).mode, ReplayMode::Simulation);
    }

    #[tokio::test]
    async fn sqlite_replay_store_persists_events_and_builds_complete_cassette() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let path = tempdir.path().join("replay.sqlite");
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
        let store = SqliteReplayStore::open(&path).unwrap_or_else(|err| panic!("{err}"));
        let events = complete_events(&tenant, &project, &trace);
        for event in events.clone() {
            store
                .put_event(event)
                .await
                .unwrap_or_else(|err| panic!("{err}"));
        }
        drop(store);

        let reopened = SqliteReplayStore::open(&path).unwrap_or_else(|err| panic!("{err}"));
        let loaded = reopened
            .list_events(tenant.clone(), project.clone(), trace.clone())
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(loaded.len(), events.len());
        let cassette = reopened
            .cassette(tenant, project, trace)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(cassette.provider_events, 1);
        assert_eq!(cassette.tool_events, 1);
        assert_eq!(cassette.memory_events, 1);
        assert_eq!(cassette.retrieval_events, 1);
        assert_eq!(cassette.clock_events, 1);
        assert_eq!(cassette.random_events, 1);
        assert!(cassette.missing_required_kinds.is_empty());
    }

    #[test]
    fn deterministic_replay_uses_cassette_responses_and_rejects_misses() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
        let events = complete_events(&tenant, &project, &trace);
        let cassette = cassette_from_events(tenant.clone(), trace.clone(), &events);
        let report = execute_replay(
            &cassette,
            &events,
            ReplayScenario {
                tenant_id: tenant.clone(),
                project_id: project.clone(),
                trace_id: trace.clone(),
                steps: events
                    .iter()
                    .map(|event| ReplayStep {
                        seq: event.seq,
                        kind: event.kind.clone(),
                        request: event.request.clone(),
                    })
                    .collect(),
                fork_after_seq: None,
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.plan.mode, ReplayMode::DeterministicReplay);
        assert_eq!(report.replayed_steps.len(), 6);
        assert!(report.live_steps_required.is_empty());
        assert_eq!(report.replayed_steps[0].response, json!({"provider": "ok"}));

        let error = execute_replay(
            &cassette,
            &events,
            ReplayScenario {
                tenant_id: tenant,
                project_id: project,
                trace_id: trace,
                steps: vec![ReplayStep {
                    seq: 1,
                    kind: ReplayEventKind::Provider,
                    request: json!({"prompt": "changed"}),
                }],
                fork_after_seq: None,
            },
        )
        .err()
        .unwrap_or_else(|| panic!("deterministic replay should reject request hash miss"));
        assert!(error
            .to_string()
            .contains("deterministic replay missing event"));
    }

    #[test]
    fn forked_replay_replays_prefix_and_marks_later_steps_live() {
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
        let trace = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
        let events = complete_events(&tenant, &project, &trace);
        let cassette = cassette_from_events(tenant.clone(), trace.clone(), &events);
        let report = execute_replay(
            &cassette,
            &events,
            ReplayScenario {
                tenant_id: tenant,
                project_id: project,
                trace_id: trace,
                steps: events
                    .iter()
                    .take(3)
                    .map(|event| ReplayStep {
                        seq: event.seq,
                        kind: event.kind.clone(),
                        request: event.request.clone(),
                    })
                    .collect(),
                fork_after_seq: Some(2),
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(report.plan.mode, ReplayMode::ForkedReplay);
        assert_eq!(report.replayed_steps.len(), 2);
        assert_eq!(report.live_steps_required.len(), 1);
        assert_eq!(report.live_steps_required[0].seq, 3);
    }

    #[test]
    fn failure_attribution_selects_earliest_bad_span() {
        let trace_id = TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"));
        let first = fixture_span("first", 1, SpanStatus::Ok);
        let second = fixture_span("second", 2, SpanStatus::Ok);
        let third = fixture_span("third", 3, SpanStatus::Error);
        let attribution = attribute_failure(
            trace_id.clone(),
            &[third.clone(), first.clone(), second.clone()],
            &[SpanEvidence {
                span_id: second.span_id.clone(),
                score: 0.1,
                reason: "tool call wrong".to_string(),
            }],
        );

        assert_eq!(attribution.trace_id, trace_id);
        assert_eq!(attribution.root_cause_span_id, Some(second.span_id));
    }

    fn fixture_span(span_id: &str, seq: u64, status: SpanStatus) -> CanonicalSpan {
        CanonicalSpan {
            schema_version: CANONICAL_SCHEMA_VERSION,
            normalizer_version: "test".to_string(),
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            span_id: SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}")),
            parent_span_id: None,
            seq,
            kind: AgentSpanKind::AgentStep,
            name: span_id.to_string(),
            status,
            start_time: Utc::now(),
            end_time: None,
            model: None,
            cost: None,
            tokens: None,
            input_ref: None,
            output_ref: None,
            attributes: BTreeMap::new(),
            unmapped_attrs: json!({}),
            raw_ref: fixture_artifact(),
        }
    }

    fn complete_events(
        tenant: &TenantId,
        project: &ProjectId,
        trace: &TraceId,
    ) -> Vec<ReplayEvent> {
        vec![
            fixture_event(
                tenant,
                project,
                trace,
                1,
                ReplayEventKind::Provider,
                "provider",
            ),
            fixture_event(tenant, project, trace, 2, ReplayEventKind::Tool, "tool"),
            fixture_event(tenant, project, trace, 3, ReplayEventKind::Memory, "memory"),
            fixture_event(
                tenant,
                project,
                trace,
                4,
                ReplayEventKind::Retrieval,
                "retrieval",
            ),
            fixture_event(tenant, project, trace, 5, ReplayEventKind::Clock, "clock"),
            fixture_event(tenant, project, trace, 6, ReplayEventKind::Random, "random"),
        ]
    }

    fn fixture_event(
        tenant: &TenantId,
        project: &ProjectId,
        trace: &TraceId,
        seq: u64,
        kind: ReplayEventKind,
        label: &str,
    ) -> ReplayEvent {
        ReplayEvent::new(
            tenant.clone(),
            project.clone(),
            trace.clone(),
            seq,
            kind,
            json!({ "request": label }),
            json!({ label: "ok" }),
        )
        .unwrap_or_else(|err| panic!("{err}"))
    }

    fn fixture_artifact() -> beater_schema::ArtifactRef {
        beater_schema::ArtifactRef {
            artifact_id: beater_core::ArtifactId::new("artifact")
                .unwrap_or_else(|err| panic!("{err}")),
            uri: "artifact://tenant/project/artifact".to_string(),
            sha256: beater_core::Sha256Hash::new("hash").unwrap_or_else(|err| panic!("{err}")),
            size_bytes: 0,
            mime_type: "application/json".to_string(),
            redaction_class: beater_schema::RedactionClass::Internal,
        }
    }
}
