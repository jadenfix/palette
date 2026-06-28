//! beater-incidents — agent incident response rooms.
//!
//! In-memory, deterministic store for agent incidents with quarantine
//! (containment), rollback tasks, and a postmortem closure gate. Models GitHub
//! issue #272: incident response rooms with quarantine, rollback, and
//! postmortem evidence.

use beater_core::{
    sha256_hex, AgentId, SessionId, SpanId, TenantScope, Timestamp, TraceId, UserId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle state of an incident.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncidentState {
    Open,
    Contained,
    Mitigated,
    Closed,
}

/// Severity ranking of an incident.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Sev1,
    Sev2,
    Sev3,
}

/// What triggered the incident to be opened.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSource {
    Trace(TraceId),
    Alert(String),
    Receipt(String),
    Feedback(String),
    Manual,
}

/// Evidence gathered into an incident room.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IncidentEvidence {
    pub trace_ids: Vec<TraceId>,
    pub span_ids: Vec<SpanId>,
    pub receipt_ids: Vec<String>,
    pub audit_event_ids: Vec<String>,
    pub alert_ids: Vec<String>,
}

/// Kind of quarantine / containment action engaged on an incident.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContainmentKind {
    DisableAgent,
    RevokeCapability,
    BlockConnector,
    BlockExternalWrites,
}

/// A containment action recorded on an incident.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ContainmentAction {
    pub kind: ContainmentKind,
    pub target: String,
    #[schema(value_type = String, format = DateTime)]
    pub engaged_at: Timestamp,
    pub engaged_by: UserId,
    pub audited: bool,
}

/// Kind of rollback work item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RollbackKind {
    Cancel,
    Refund,
    Delete,
    Revert,
    Retry,
}

/// Status of a rollback task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStatus {
    Pending,
    Done,
    Failed,
}

/// A rollback task tracking remediation of an external side effect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RollbackTask {
    pub kind: RollbackKind,
    pub external_ref: String,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub deadline: Option<Timestamp>,
    pub owner: UserId,
    pub status: RollbackStatus,
}

/// Kind of postmortem follow-up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FollowUpKind {
    DatasetCase,
    Eval,
    PolicyChange,
    ConnectorFix,
    InstructionUpdate,
}

/// A postmortem follow-up item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FollowUp {
    pub kind: FollowUpKind,
    pub reference: String,
}

/// A postmortem write-up that gates incident closure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Postmortem {
    pub summary: String,
    pub follow_ups: Vec<FollowUp>,
}

/// An agent incident response room.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AgentIncident {
    pub incident_id: String,
    pub scope: TenantScope,
    pub title: String,
    pub severity: Severity,
    pub state: IncidentState,
    pub opened_from: IncidentSource,
    pub agent_id: Option<AgentId>,
    pub session_id: Option<SessionId>,
    pub evidence: IncidentEvidence,
    pub containment_actions: Vec<ContainmentAction>,
    pub rollback_tasks: Vec<RollbackTask>,
    pub postmortem: Option<Postmortem>,
    #[schema(value_type = String, format = DateTime)]
    pub opened_at: Timestamp,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub closed_at: Option<Timestamp>,
}

/// An audit marker emitted when a containment action is engaged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IncidentAuditEntry {
    pub incident_id: String,
    pub kind: ContainmentKind,
    pub target: String,
    pub engaged_by: UserId,
    #[schema(value_type = String, format = DateTime)]
    pub engaged_at: Timestamp,
}

/// A single entry in the merged incident timeline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TimelineEntry {
    #[schema(value_type = String, format = DateTime)]
    pub at: Timestamp,
    pub kind: TimelineKind,
    pub detail: String,
}

/// Category of a timeline entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimelineKind {
    Opened,
    Containment,
    Rollback,
    Postmortem,
    Closed,
}

/// Errors returned by the incident store.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IncidentError {
    #[error("incident not found: {0}")]
    NotFound(String),
    #[error("rollback task index out of range: {0}")]
    RollbackIndexOutOfRange(usize),
    #[error("cannot close incident without at least one postmortem follow-up")]
    ClosureGateMissingFollowUp,
}

/// Request to open a new incident.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct OpenIncidentRequest {
    pub scope: TenantScope,
    pub title: String,
    pub severity: Severity,
    pub opened_from: IncidentSource,
    pub agent_id: Option<AgentId>,
    pub session_id: Option<SessionId>,
}

/// In-memory, deterministic incident store.
#[derive(Debug, Default)]
pub struct IncidentStore {
    incidents: HashMap<String, AgentIncident>,
    audit_log: Vec<IncidentAuditEntry>,
}

impl IncidentStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a new incident. The id is derived deterministically from the
    /// opening timestamp and title so opens are reproducible in tests.
    pub fn open_incident(&mut self, req: OpenIncidentRequest, now: Timestamp) -> AgentIncident {
        let incident_id = Self::derive_id(&req.title, now);
        let incident = AgentIncident {
            incident_id: incident_id.clone(),
            scope: req.scope,
            title: req.title,
            severity: req.severity,
            state: IncidentState::Open,
            opened_from: req.opened_from,
            agent_id: req.agent_id,
            session_id: req.session_id,
            evidence: IncidentEvidence::default(),
            containment_actions: Vec::new(),
            rollback_tasks: Vec::new(),
            postmortem: None,
            opened_at: now,
            closed_at: None,
        };
        self.incidents.insert(incident_id, incident.clone());
        incident
    }

    fn derive_id(title: &str, now: Timestamp) -> String {
        let seed = format!("{}|{}", now.timestamp_nanos_opt().unwrap_or(0), title);
        let digest = sha256_hex(seed.as_bytes());
        format!("inc_{}", &digest[..32])
    }

    /// Fetch an incident by id.
    pub fn get(&self, id: &str) -> Option<&AgentIncident> {
        self.incidents.get(id)
    }

    /// List incidents, optionally filtered by state and/or agent. Results are
    /// sorted by `opened_at` then `incident_id` for deterministic ordering.
    pub fn list(
        &self,
        state: Option<IncidentState>,
        agent_id: Option<&AgentId>,
    ) -> Vec<&AgentIncident> {
        let mut out: Vec<&AgentIncident> = self
            .incidents
            .values()
            .filter(|inc| state.is_none_or(|s| inc.state == s))
            .filter(|inc| agent_id.is_none_or(|a| inc.agent_id.as_ref() == Some(a)))
            .collect();
        out.sort_by(|a, b| {
            a.opened_at
                .cmp(&b.opened_at)
                .then_with(|| a.incident_id.cmp(&b.incident_id))
        });
        out
    }

    /// Read access to the in-memory audit log.
    pub fn audit_log(&self) -> &[IncidentAuditEntry] {
        &self.audit_log
    }

    fn incident_mut(&mut self, id: &str) -> Result<&mut AgentIncident, IncidentError> {
        self.incidents
            .get_mut(id)
            .ok_or_else(|| IncidentError::NotFound(id.to_string()))
    }

    /// Merge additional evidence into an incident.
    pub fn add_evidence(
        &mut self,
        id: &str,
        evidence: IncidentEvidence,
    ) -> Result<&AgentIncident, IncidentError> {
        let incident = self.incident_mut(id)?;
        incident.evidence.trace_ids.extend(evidence.trace_ids);
        incident.evidence.span_ids.extend(evidence.span_ids);
        incident.evidence.receipt_ids.extend(evidence.receipt_ids);
        incident
            .evidence
            .audit_event_ids
            .extend(evidence.audit_event_ids);
        incident.evidence.alert_ids.extend(evidence.alert_ids);
        Ok(&*incident)
    }

    /// Append a containment action and emit an audit marker. Marks the stored
    /// action as `audited` and (if the incident is still Open) advances state
    /// to Contained.
    pub fn add_containment(
        &mut self,
        id: &str,
        action: ContainmentAction,
    ) -> Result<&AgentIncident, IncidentError> {
        let entry = IncidentAuditEntry {
            incident_id: id.to_string(),
            kind: action.kind,
            target: action.target.clone(),
            engaged_by: action.engaged_by.clone(),
            engaged_at: action.engaged_at,
        };
        let incident = self.incident_mut(id)?;
        let mut action = action;
        action.audited = true;
        incident.containment_actions.push(action);
        if incident.state == IncidentState::Open {
            incident.state = IncidentState::Contained;
        }
        self.audit_log.push(entry);
        self.get(id)
            .ok_or_else(|| IncidentError::NotFound(id.to_string()))
    }

    /// Append a rollback task.
    pub fn add_rollback_task(
        &mut self,
        id: &str,
        task: RollbackTask,
    ) -> Result<&AgentIncident, IncidentError> {
        let incident = self.incident_mut(id)?;
        incident.rollback_tasks.push(task);
        Ok(&*incident)
    }

    /// Update the status of the rollback task at `index`.
    pub fn update_rollback_status(
        &mut self,
        id: &str,
        index: usize,
        status: RollbackStatus,
    ) -> Result<&AgentIncident, IncidentError> {
        let incident = self.incident_mut(id)?;
        let task = incident
            .rollback_tasks
            .get_mut(index)
            .ok_or(IncidentError::RollbackIndexOutOfRange(index))?;
        task.status = status;
        Ok(&*incident)
    }

    /// Set the lifecycle state directly (does not gate; use `close_incident`
    /// for the postmortem-gated Closed transition).
    pub fn set_state(
        &mut self,
        id: &str,
        state: IncidentState,
    ) -> Result<&AgentIncident, IncidentError> {
        let incident = self.incident_mut(id)?;
        incident.state = state;
        Ok(&*incident)
    }

    /// Close an incident. ENFORCES the closure gate: the postmortem must carry
    /// at least one follow-up. On success the incident is set to Closed and
    /// `closed_at` is stamped.
    pub fn close_incident(
        &mut self,
        id: &str,
        postmortem: Postmortem,
        now: Timestamp,
    ) -> Result<&AgentIncident, IncidentError> {
        if postmortem.follow_ups.is_empty() {
            return Err(IncidentError::ClosureGateMissingFollowUp);
        }
        let incident = self.incident_mut(id)?;
        incident.postmortem = Some(postmortem);
        incident.state = IncidentState::Closed;
        incident.closed_at = Some(now);
        Ok(&*incident)
    }
}

/// Build the merged, time-ordered timeline for an incident.
///
/// Merges the open event, every containment action, every rollback task, the
/// postmortem (if any), and the closure, sorted by timestamp. Rollback tasks
/// without a deadline anchor to `opened_at`, and the postmortem (which carries
/// no timestamp of its own) anchors to `closed_at` if present else `opened_at`.
pub fn incident_timeline(incident: &AgentIncident) -> Vec<TimelineEntry> {
    let mut entries: Vec<TimelineEntry> = Vec::new();

    entries.push(TimelineEntry {
        at: incident.opened_at,
        kind: TimelineKind::Opened,
        detail: format!("opened: {}", incident.title),
    });

    for action in &incident.containment_actions {
        entries.push(TimelineEntry {
            at: action.engaged_at,
            kind: TimelineKind::Containment,
            detail: format!("{:?} on {}", action.kind, action.target),
        });
    }

    for task in &incident.rollback_tasks {
        entries.push(TimelineEntry {
            at: task.deadline.unwrap_or(incident.opened_at),
            kind: TimelineKind::Rollback,
            detail: format!("{:?} {} ({:?})", task.kind, task.external_ref, task.status),
        });
    }

    if let Some(postmortem) = &incident.postmortem {
        entries.push(TimelineEntry {
            at: incident.closed_at.unwrap_or(incident.opened_at),
            kind: TimelineKind::Postmortem,
            detail: format!(
                "postmortem: {} ({} follow-ups)",
                postmortem.summary,
                postmortem.follow_ups.len()
            ),
        });
    }

    if let Some(closed_at) = incident.closed_at {
        entries.push(TimelineEntry {
            at: closed_at,
            kind: TimelineKind::Closed,
            detail: "closed".to_string(),
        });
    }

    entries.sort_by(|a, b| a.at.cmp(&b.at));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{
        AgentId, EnvironmentId, ProjectId, SessionId, SpanId, TenantId, TraceId, UserId,
    };
    use chrono::{Duration, TimeZone, Utc};

    fn scope() -> TenantScope {
        TenantScope::new(
            TenantId::new("tenant-1").unwrap_or_else(|e| panic!("{e}")),
            ProjectId::new("project-1").unwrap_or_else(|e| panic!("{e}")),
            EnvironmentId::new("env-1").unwrap_or_else(|e| panic!("{e}")),
        )
    }

    fn user(s: &str) -> UserId {
        UserId::new(s).unwrap_or_else(|e| panic!("{e}"))
    }

    fn agent(s: &str) -> AgentId {
        AgentId::new(s).unwrap_or_else(|e| panic!("{e}"))
    }

    fn ts(secs: i64) -> Timestamp {
        Utc.timestamp_opt(secs, 0)
            .single()
            .unwrap_or_else(|| panic!("invalid timestamp"))
    }

    fn expect_err<T: std::fmt::Debug>(
        result: Result<T, IncidentError>,
        msg: &str,
    ) -> IncidentError {
        match result {
            Ok(value) => panic!("{msg}: expected Err, got Ok({value:?})"),
            Err(err) => err,
        }
    }

    fn open(store: &mut IncidentStore, title: &str, now: Timestamp) -> AgentIncident {
        store.open_incident(
            OpenIncidentRequest {
                scope: scope(),
                title: title.to_string(),
                severity: Severity::Sev1,
                opened_from: IncidentSource::Manual,
                agent_id: Some(agent("agent-1")),
                session_id: Some(SessionId::new("sess-1").unwrap_or_else(|e| panic!("{e}"))),
            },
            now,
        )
    }

    #[test]
    fn open_incident_starts_open() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "billing loop", ts(100));
        assert_eq!(inc.state, IncidentState::Open);
        assert!(inc.closed_at.is_none());
        assert!(inc.incident_id.starts_with("inc_"));
        assert!(store.get(&inc.incident_id).is_some());
    }

    #[test]
    fn open_incident_id_is_deterministic() {
        let mut a = IncidentStore::new();
        let mut b = IncidentStore::new();
        let x = open(&mut a, "same", ts(7));
        let y = open(&mut b, "same", ts(7));
        assert_eq!(x.incident_id, y.incident_id);
    }

    #[test]
    fn cannot_close_without_follow_up() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "no followups", ts(100));
        let err = expect_err(
            store.close_incident(
                &inc.incident_id,
                Postmortem {
                    summary: "we learned things".to_string(),
                    follow_ups: vec![],
                },
                ts(200),
            ),
            "closure gate must reject empty follow-ups",
        );
        assert_eq!(err, IncidentError::ClosureGateMissingFollowUp);
        let stored = store
            .get(&inc.incident_id)
            .unwrap_or_else(|| panic!("missing"));
        assert_ne!(stored.state, IncidentState::Closed);
        assert!(stored.closed_at.is_none());
    }

    #[test]
    fn closes_with_one_follow_up() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "closable", ts(100));
        let closed = store
            .close_incident(
                &inc.incident_id,
                Postmortem {
                    summary: "root caused".to_string(),
                    follow_ups: vec![FollowUp {
                        kind: FollowUpKind::Eval,
                        reference: "eval-42".to_string(),
                    }],
                },
                ts(500),
            )
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(closed.state, IncidentState::Closed);
        assert_eq!(closed.closed_at, Some(ts(500)));
        assert!(closed.postmortem.is_some());
    }

    #[test]
    fn close_unknown_incident_is_not_found() {
        let mut store = IncidentStore::new();
        let err = expect_err(
            store.close_incident(
                "inc_missing",
                Postmortem {
                    summary: "x".to_string(),
                    follow_ups: vec![FollowUp {
                        kind: FollowUpKind::Eval,
                        reference: "e".to_string(),
                    }],
                },
                ts(1),
            ),
            "should be not found",
        );
        assert_eq!(err, IncidentError::NotFound("inc_missing".to_string()));
    }

    #[test]
    fn containment_recorded_and_audited() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "compromise", ts(100));
        let updated = store
            .add_containment(
                &inc.incident_id,
                ContainmentAction {
                    kind: ContainmentKind::DisableAgent,
                    target: "agent-1".to_string(),
                    engaged_at: ts(120),
                    engaged_by: user("responder"),
                    audited: false,
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(updated.containment_actions.len(), 1);
        assert!(updated.containment_actions[0].audited);
        assert_eq!(updated.state, IncidentState::Contained);

        let log = store.audit_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].incident_id, inc.incident_id);
        assert_eq!(log[0].kind, ContainmentKind::DisableAgent);
        assert_eq!(log[0].target, "agent-1");
    }

    #[test]
    fn evidence_accumulates() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "evidence", ts(100));
        store
            .add_evidence(
                &inc.incident_id,
                IncidentEvidence {
                    trace_ids: vec![TraceId::new("trace-a").unwrap_or_else(|e| panic!("{e}"))],
                    span_ids: vec![SpanId::new("span-a").unwrap_or_else(|e| panic!("{e}"))],
                    receipt_ids: vec!["r1".to_string()],
                    audit_event_ids: vec![],
                    alert_ids: vec!["al-1".to_string()],
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        let updated = store
            .add_evidence(
                &inc.incident_id,
                IncidentEvidence {
                    trace_ids: vec![TraceId::new("trace-b").unwrap_or_else(|e| panic!("{e}"))],
                    ..Default::default()
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(updated.evidence.trace_ids.len(), 2);
        assert_eq!(updated.evidence.receipt_ids, vec!["r1".to_string()]);
        assert_eq!(updated.evidence.alert_ids, vec!["al-1".to_string()]);
    }

    #[test]
    fn rollback_status_transitions() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "rollback", ts(100));
        store
            .add_rollback_task(
                &inc.incident_id,
                RollbackTask {
                    kind: RollbackKind::Refund,
                    external_ref: "charge_123".to_string(),
                    deadline: Some(ts(300)),
                    owner: user("owner"),
                    status: RollbackStatus::Pending,
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        let updated = store
            .update_rollback_status(&inc.incident_id, 0, RollbackStatus::Done)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(updated.rollback_tasks[0].status, RollbackStatus::Done);
    }

    #[test]
    fn rollback_index_out_of_range() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "rollback-oob", ts(100));
        let err = expect_err(
            store.update_rollback_status(&inc.incident_id, 5, RollbackStatus::Failed),
            "out of range",
        );
        assert_eq!(err, IncidentError::RollbackIndexOutOfRange(5));
    }

    #[test]
    fn set_state_transitions_to_mitigated() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "mitigate", ts(100));
        let updated = store
            .set_state(&inc.incident_id, IncidentState::Mitigated)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(updated.state, IncidentState::Mitigated);
    }

    #[test]
    fn list_filters_by_state_and_agent() {
        let mut store = IncidentStore::new();
        let a = open(&mut store, "a", ts(100));
        let _b = open(&mut store, "b", ts(200));
        // Move a to Contained via containment.
        store
            .add_containment(
                &a.incident_id,
                ContainmentAction {
                    kind: ContainmentKind::BlockConnector,
                    target: "stripe".to_string(),
                    engaged_at: ts(110),
                    engaged_by: user("u"),
                    audited: false,
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));

        let open_only = store.list(Some(IncidentState::Open), None);
        assert_eq!(open_only.len(), 1);
        assert_eq!(open_only[0].incident_id, _b.incident_id);

        let contained = store.list(Some(IncidentState::Contained), None);
        assert_eq!(contained.len(), 1);
        assert_eq!(contained[0].incident_id, a.incident_id);

        let by_agent = store.list(None, Some(&agent("agent-1")));
        assert_eq!(by_agent.len(), 2);

        let other_agent = store.list(None, Some(&agent("agent-other")));
        assert!(other_agent.is_empty());
    }

    #[test]
    fn list_sorted_by_opened_at() {
        let mut store = IncidentStore::new();
        let later = open(&mut store, "later", ts(900));
        let earlier = open(&mut store, "earlier", ts(100));
        let all = store.list(None, None);
        assert_eq!(all[0].incident_id, earlier.incident_id);
        assert_eq!(all[1].incident_id, later.incident_id);
    }

    #[test]
    fn timeline_ordering_correct() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "timeline", ts(100));
        store
            .add_containment(
                &inc.incident_id,
                ContainmentAction {
                    kind: ContainmentKind::BlockExternalWrites,
                    target: "db".to_string(),
                    engaged_at: ts(150),
                    engaged_by: user("u"),
                    audited: false,
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        store
            .add_rollback_task(
                &inc.incident_id,
                RollbackTask {
                    kind: RollbackKind::Revert,
                    external_ref: "deploy-9".to_string(),
                    deadline: Some(ts(300)),
                    owner: user("u"),
                    status: RollbackStatus::Pending,
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        store
            .close_incident(
                &inc.incident_id,
                Postmortem {
                    summary: "done".to_string(),
                    follow_ups: vec![FollowUp {
                        kind: FollowUpKind::PolicyChange,
                        reference: "policy-1".to_string(),
                    }],
                },
                ts(500),
            )
            .unwrap_or_else(|e| panic!("{e}"));

        let inc = store
            .get(&inc.incident_id)
            .unwrap_or_else(|| panic!("missing"));
        let timeline = incident_timeline(inc);
        let kinds: Vec<TimelineKind> = timeline.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TimelineKind::Opened,
                TimelineKind::Containment,
                TimelineKind::Rollback,
                TimelineKind::Postmortem,
                TimelineKind::Closed,
            ]
        );
        // monotonic non-decreasing timestamps
        for pair in timeline.windows(2) {
            assert!(pair[0].at <= pair[1].at);
        }
    }

    #[test]
    fn timeline_anchors_unscheduled_rollback_to_open() {
        let mut store = IncidentStore::new();
        let inc = open(&mut store, "anchor", ts(1000));
        store
            .add_rollback_task(
                &inc.incident_id,
                RollbackTask {
                    kind: RollbackKind::Cancel,
                    external_ref: "job-1".to_string(),
                    deadline: None,
                    owner: user("u"),
                    status: RollbackStatus::Pending,
                },
            )
            .unwrap_or_else(|e| panic!("{e}"));
        let inc = store
            .get(&inc.incident_id)
            .unwrap_or_else(|| panic!("missing"));
        let timeline = incident_timeline(inc);
        let rollback = timeline
            .iter()
            .find(|e| e.kind == TimelineKind::Rollback)
            .unwrap_or_else(|| panic!("no rollback entry"));
        assert_eq!(rollback.at, ts(1000));
    }

    #[test]
    fn enum_serializes_snake_case() {
        let v = serde_json::to_value(Severity::Sev1).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(v, serde_json::json!("sev1"));
        let v = serde_json::to_value(RollbackStatus::Pending).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(v, serde_json::json!("pending"));
        let v = serde_json::to_value(IncidentState::Contained).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(v, serde_json::json!("contained"));
        // sanity: Duration import unused guard
        let _ = Duration::seconds(1);
    }
}
