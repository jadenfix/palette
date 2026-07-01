//! beater-autonomy — autonomy-level policy simulation.
//!
//! Deterministic historical-replay simulation that estimates the impact of
//! promoting an agent to a higher autonomy level, *before* changing any live
//! behavior. Nothing in this crate alters runtime autonomy; it only replays
//! recorded [`SessionActionEvent`]s against a candidate [`AutonomyPolicy`] and
//! reports what *would* have happened.

use std::collections::BTreeMap;

use beater_core::{AgentId, SessionId, Timestamp, TraceId, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Crate identifier, retained for parity with the original scaffold.
pub const CRATE_NAME: &str = "beater-autonomy";

/// Autonomy levels, ordered low → high by operational capability.
///
/// `Disabled` is a special terminal state: an agent that is switched off. It is
/// *not* "more autonomous" than [`AutonomyLevel::AutoExternalWrite`]; its
/// [`AutonomyLevel::rank`] is therefore handled specially (see that method).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    /// Watches only; never acts or suggests.
    Observe,
    /// Proposes actions for a human to perform manually.
    Suggest,
    /// May act, but must ask before every action.
    AskBeforeAction,
    /// Auto-executes low-risk actions only.
    AutoLowRisk,
    /// Auto-executes with receipts/audit trail.
    AutoWithReceipts,
    /// Auto-executes including external writes.
    AutoExternalWrite,
    /// Agent is switched off entirely.
    Disabled,
}

impl AutonomyLevel {
    /// Explicit capability rank, low → high.
    ///
    /// `Disabled` is special-cased to `0` because a disabled agent has *no*
    /// operational autonomy at all — it ranks below every active level rather
    /// than above them, even though it is the last variant declared.
    pub fn rank(self) -> u8 {
        match self {
            AutonomyLevel::Disabled => 0,
            AutonomyLevel::Observe => 1,
            AutonomyLevel::Suggest => 2,
            AutonomyLevel::AskBeforeAction => 3,
            AutonomyLevel::AutoLowRisk => 4,
            AutonomyLevel::AutoWithReceipts => 5,
            AutonomyLevel::AutoExternalWrite => 6,
        }
    }

    /// Whether this level auto-executes any actions at all.
    pub fn is_autonomous(self) -> bool {
        matches!(
            self,
            AutonomyLevel::AutoLowRisk
                | AutonomyLevel::AutoWithReceipts
                | AutonomyLevel::AutoExternalWrite
        )
    }
}

/// Risk classification for a single action, ordered low → high.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RiskTag {
    Low,
    Medium,
    High,
    Destructive,
}

impl RiskTag {
    /// Stable string key used in [`PolicySimulationReport::risk_deltas`].
    pub fn as_key(self) -> &'static str {
        match self {
            RiskTag::Low => "low",
            RiskTag::Medium => "medium",
            RiskTag::High => "high",
            RiskTag::Destructive => "destructive",
        }
    }
}

/// A candidate autonomy policy to be simulated before promotion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AutonomyPolicy {
    /// Stable policy identifier.
    pub policy_id: String,
    /// Agent the policy applies to, or `None` for an org-wide default.
    pub agent_id: Option<AgentId>,
    /// Autonomy level granted by this policy.
    pub level: AutonomyLevel,
    /// Require a passing eval before this policy may take effect.
    pub requires_eval_pass: bool,
    /// Require no recent incidents before this policy may take effect.
    pub requires_no_recent_incidents: bool,
    /// Number of reviewers required to approve the rollout.
    pub reviewer_threshold: u32,
    /// Require a capability lease for actions under this policy.
    pub requires_capability_lease: bool,
    /// Maximum blast radius (impacted resources) allowed per action.
    pub max_blast_radius: u32,
    /// Maximum risk tag that is auto-allowed at this level.
    pub auto_allowed_risk: RiskTag,
}

/// One recorded action taken within a session, used as replay input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SessionActionEvent {
    pub session_id: SessionId,
    pub trace_id: TraceId,
    /// Human-readable action kind (e.g. `"send_email"`).
    pub action_kind: String,
    pub risk_tag: RiskTag,
    /// A human explicitly approved this action.
    pub was_approved: bool,
    /// A human explicitly denied this action.
    pub was_denied: bool,
    /// The action was destructive.
    pub was_destructive: bool,
    /// The action wrote to an external system.
    pub was_external_write: bool,
    /// Latency contributed by the human-in-the-loop step, in milliseconds.
    pub latency_ms: u64,
    /// The action ultimately succeeded.
    pub outcome_ok: bool,
    #[schema(value_type = String, format = DateTime)]
    pub at: Timestamp,
}

/// Disclaimer attached to every report, kept as a constant so callers can match
/// on / display it consistently.
pub const EVIDENCE_DISCLAIMER: &str = "Historical-replay evidence: counts reflect what the candidate policy would have done against recorded events only. This is NOT a statistically valid prediction of future behavior. Sampling caveat: replayed events are a non-random, partial sample of past sessions and may omit rare or unobserved action patterns.";

/// Result of replaying events against a candidate policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PolicySimulationReport {
    /// Human approvals that would no longer be needed (auto-approved instead).
    pub approvals_avoided: u32,
    /// Previously-denied / destructive / external-write actions that would now
    /// be auto-approved without a human in the loop.
    pub denials_bypassed: u32,
    /// Actions whose human-escalation behavior would change.
    pub escalations_changed: u32,
    /// Estimated human-in-the-loop latency saved, in milliseconds.
    pub predicted_latency_saved_ms: u64,
    /// Net change in auto-allowed action count, keyed by risk tag.
    pub risk_deltas: BTreeMap<String, i64>,
    /// Sample of high-risk actions that would be auto-allowed under the policy.
    pub high_risk_examples: Vec<SessionActionEvent>,
    /// Distinct sessions touched by the replay.
    pub impacted_sessions: u32,
    /// Whether guardrails block this policy from being rolled out.
    pub blocked: bool,
    /// Human-readable warnings.
    pub warnings: Vec<String>,
    /// Evidence-quality disclaimer (see [`EVIDENCE_DISCLAIMER`]).
    pub evidence_disclaimer: String,
}

/// Replay `events` against `policy` and report the predicted impact.
///
/// `eval_passing` / `recent_incident` describe the *current* world state and
/// are checked against the policy's guardrails. This function is pure and
/// deterministic: identical inputs always yield an identical report.
pub fn simulate_policy(
    policy: &AutonomyPolicy,
    events: &[SessionActionEvent],
    eval_passing: bool,
    recent_incident: bool,
) -> PolicySimulationReport {
    let mut approvals_avoided: u32 = 0;
    let mut denials_bypassed: u32 = 0;
    let mut escalations_changed: u32 = 0;
    let mut predicted_latency_saved_ms: u64 = 0;
    let mut high_risk_examples: Vec<SessionActionEvent> = Vec::new();
    let mut risk_deltas: BTreeMap<String, i64> = BTreeMap::new();
    let mut sessions: std::collections::BTreeSet<&SessionId> = std::collections::BTreeSet::new();

    // Only an autonomous level auto-executes anything; otherwise the policy
    // changes nothing about who acts.
    let level_autonomous = policy.level.is_autonomous();

    for event in events {
        sessions.insert(&event.session_id);

        // Would this action be auto-approved under the candidate policy?
        let auto_allowed = level_autonomous && event.risk_tag <= policy.auto_allowed_risk;

        if auto_allowed {
            *risk_deltas
                .entry(event.risk_tag.as_key().to_string())
                .or_insert(0) += 1;

            // A human previously approved this -> approval now avoided.
            if event.was_approved {
                approvals_avoided = approvals_avoided.saturating_add(1);
                predicted_latency_saved_ms =
                    predicted_latency_saved_ms.saturating_add(event.latency_ms);
                escalations_changed = escalations_changed.saturating_add(1);
            }

            // Previously denied / destructive / external-write that would now
            // sail through without a human is a bypass and a high-risk example.
            if event.was_denied || event.was_destructive || event.was_external_write {
                denials_bypassed = denials_bypassed.saturating_add(1);
                escalations_changed = escalations_changed.saturating_add(1);
                high_risk_examples.push(event.clone());
            }
        }
    }

    let impacted_sessions = u32::try_from(sessions.len()).unwrap_or(u32::MAX);

    let mut warnings: Vec<String> = Vec::new();
    let has_high_risk_autos = denials_bypassed > 0;

    // Guardrail violations only *block* when the policy would actually let a
    // risky action through automatically.
    let eval_guard_fails = policy.requires_eval_pass && !eval_passing;
    let incident_guard_fails = policy.requires_no_recent_incidents && recent_incident;

    if eval_guard_fails {
        warnings
            .push("Policy requires a passing eval, but the current eval is failing.".to_string());
    }
    if incident_guard_fails {
        warnings.push(
            "Policy requires no recent incidents, but a recent incident is present.".to_string(),
        );
    }
    if has_high_risk_autos {
        warnings.push(format!(
            "{denials_bypassed} previously-gated high-risk action(s) would be auto-approved under this policy.",
        ));
    }

    let blocked = (eval_guard_fails || incident_guard_fails) && has_high_risk_autos;
    if blocked {
        warnings.push(
            "Rollout blocked: guardrails are not satisfied while high-risk actions would be auto-approved.".to_string(),
        );
    }

    PolicySimulationReport {
        approvals_avoided,
        denials_bypassed,
        escalations_changed,
        predicted_latency_saved_ms,
        risk_deltas,
        high_risk_examples,
        impacted_sessions,
        blocked,
        warnings,
        evidence_disclaimer: EVIDENCE_DISCLAIMER.to_string(),
    }
}

/// Errors raised when proposing a rollout.
#[derive(Debug, thiserror::Error)]
pub enum RolloutError {
    #[error("policy rollout refused: simulation report is blocked by guardrails")]
    Blocked,
}

/// Record produced when a (non-blocked) policy is proposed for rollout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RolloutRecord {
    pub policy_id: String,
    pub policy_version: u32,
    pub approver: UserId,
    /// Short, human-readable summary of the simulation that justified rollout.
    pub simulation_summary: String,
    /// Whether post-rollout monitoring is required.
    pub monitoring_required: bool,
    #[schema(value_type = String, format = DateTime)]
    pub at: Timestamp,
}

/// Propose rolling out `policy` given its simulation `report`.
///
/// Refuses with [`RolloutError::Blocked`] if the report is blocked. Monitoring
/// is required whenever the policy would auto-approve any high-risk action.
pub fn propose_rollout(
    policy: &AutonomyPolicy,
    report: &PolicySimulationReport,
    approver: UserId,
    now: Timestamp,
) -> Result<RolloutRecord, RolloutError> {
    if report.blocked {
        return Err(RolloutError::Blocked);
    }

    let monitoring_required = report.denials_bypassed > 0 || policy.level.is_autonomous();

    let simulation_summary = format!(
        "approvals_avoided={}, denials_bypassed={}, escalations_changed={}, latency_saved_ms={}, impacted_sessions={}",
        report.approvals_avoided,
        report.denials_bypassed,
        report.escalations_changed,
        report.predicted_latency_saved_ms,
        report.impacted_sessions,
    );

    Ok(RolloutRecord {
        policy_id: policy.policy_id.clone(),
        policy_version: 1,
        approver,
        simulation_summary,
        monitoring_required,
        at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(secs: i64) -> Timestamp {
        chrono::Utc
            .timestamp_opt(secs, 0)
            .single()
            .unwrap_or_else(|| panic!("invalid timestamp"))
    }

    fn sid(n: u32) -> SessionId {
        SessionId::new(format!("sess-{n}")).unwrap_or_else(|e| panic!("{e}"))
    }

    fn tid(n: u32) -> TraceId {
        TraceId::new(format!("trace-{n}")).unwrap_or_else(|e| panic!("{e}"))
    }

    fn event(
        session: u32,
        risk: RiskTag,
        approved: bool,
        denied: bool,
        destructive: bool,
        external: bool,
        latency: u64,
    ) -> SessionActionEvent {
        SessionActionEvent {
            session_id: sid(session),
            trace_id: tid(session),
            action_kind: "act".to_string(),
            risk_tag: risk,
            was_approved: approved,
            was_denied: denied,
            was_destructive: destructive,
            was_external_write: external,
            latency_ms: latency,
            outcome_ok: true,
            at: ts(1_000 + i64::from(session)),
        }
    }

    fn policy(
        level: AutonomyLevel,
        auto_allowed: RiskTag,
        eval: bool,
        incidents: bool,
    ) -> AutonomyPolicy {
        AutonomyPolicy {
            policy_id: "pol-1".to_string(),
            agent_id: None,
            level,
            requires_eval_pass: eval,
            requires_no_recent_incidents: incidents,
            reviewer_threshold: 1,
            requires_capability_lease: false,
            max_blast_radius: 10,
            auto_allowed_risk: auto_allowed,
        }
    }

    #[test]
    fn rank_orders_levels_low_to_high() {
        assert!(AutonomyLevel::Observe.rank() < AutonomyLevel::Suggest.rank());
        assert!(AutonomyLevel::Suggest.rank() < AutonomyLevel::AskBeforeAction.rank());
        assert!(AutonomyLevel::AskBeforeAction.rank() < AutonomyLevel::AutoLowRisk.rank());
        assert!(AutonomyLevel::AutoLowRisk.rank() < AutonomyLevel::AutoWithReceipts.rank());
        assert!(AutonomyLevel::AutoWithReceipts.rank() < AutonomyLevel::AutoExternalWrite.rank());
    }

    #[test]
    fn disabled_ranks_below_every_active_level() {
        assert_eq!(AutonomyLevel::Disabled.rank(), 0);
        assert!(AutonomyLevel::Disabled.rank() < AutonomyLevel::Observe.rank());
        assert!(AutonomyLevel::Disabled.rank() < AutonomyLevel::AutoExternalWrite.rank());
    }

    #[test]
    fn risk_tag_ordering() {
        assert!(RiskTag::Low < RiskTag::Medium);
        assert!(RiskTag::Medium < RiskTag::High);
        assert!(RiskTag::High < RiskTag::Destructive);
    }

    #[test]
    fn non_autonomous_level_changes_nothing() {
        let p = policy(AutonomyLevel::Suggest, RiskTag::Destructive, false, false);
        let events = vec![event(1, RiskTag::Low, true, false, false, false, 500)];
        let r = simulate_policy(&p, &events, true, false);
        assert_eq!(r.approvals_avoided, 0);
        assert_eq!(r.denials_bypassed, 0);
        assert_eq!(r.predicted_latency_saved_ms, 0);
        assert!(r.risk_deltas.is_empty());
        assert!(!r.blocked);
    }

    #[test]
    fn approvals_avoided_counts_and_latency_sums() {
        let p = policy(AutonomyLevel::AutoLowRisk, RiskTag::Low, false, false);
        let events = vec![
            event(1, RiskTag::Low, true, false, false, false, 300),
            event(2, RiskTag::Low, true, false, false, false, 200),
            // High risk: above auto_allowed, so not auto-approved.
            event(3, RiskTag::High, true, false, false, false, 999),
        ];
        let r = simulate_policy(&p, &events, true, false);
        assert_eq!(r.approvals_avoided, 2);
        assert_eq!(r.predicted_latency_saved_ms, 500);
        assert_eq!(r.denials_bypassed, 0);
        assert!(r.high_risk_examples.is_empty());
        assert_eq!(r.impacted_sessions, 3);
        assert!(!r.blocked);
    }

    #[test]
    fn destructive_auto_allow_flags_high_risk_example() {
        let p = policy(
            AutonomyLevel::AutoExternalWrite,
            RiskTag::Destructive,
            false,
            false,
        );
        let events = vec![event(1, RiskTag::Destructive, false, false, true, false, 0)];
        let r = simulate_policy(&p, &events, true, false);
        assert_eq!(r.denials_bypassed, 1);
        assert_eq!(r.high_risk_examples.len(), 1);
        // No guardrails configured -> not blocked even with high-risk autos.
        assert!(!r.blocked);
        assert!(r.warnings.iter().any(|w| w.contains("high-risk")));
    }

    #[test]
    fn previously_denied_auto_allow_is_bypass() {
        let p = policy(AutonomyLevel::AutoWithReceipts, RiskTag::High, false, false);
        let events = vec![event(1, RiskTag::High, false, true, false, false, 100)];
        let r = simulate_policy(&p, &events, true, false);
        assert_eq!(r.denials_bypassed, 1);
        assert_eq!(r.high_risk_examples.len(), 1);
    }

    #[test]
    fn guardrail_blocks_when_eval_failing_and_high_risk_auto() {
        let p = policy(
            AutonomyLevel::AutoExternalWrite,
            RiskTag::Destructive,
            true,
            false,
        );
        let events = vec![event(1, RiskTag::Destructive, false, false, true, true, 0)];
        let r = simulate_policy(&p, &events, false, false);
        assert!(r.blocked);
        assert!(r.high_risk_examples.len() == 1);
        assert!(r.warnings.iter().any(|w| w.contains("blocked")));
    }

    #[test]
    fn guardrail_blocks_on_recent_incident() {
        let p = policy(
            AutonomyLevel::AutoExternalWrite,
            RiskTag::Destructive,
            false,
            true,
        );
        let events = vec![event(1, RiskTag::Destructive, false, false, true, false, 0)];
        let r = simulate_policy(&p, &events, true, true);
        assert!(r.blocked);
    }

    #[test]
    fn guardrail_does_not_block_without_high_risk_autos() {
        // Eval failing, but only low-risk approvals would change -> no block.
        let p = policy(AutonomyLevel::AutoLowRisk, RiskTag::Low, true, false);
        let events = vec![event(1, RiskTag::Low, true, false, false, false, 100)];
        let r = simulate_policy(&p, &events, false, false);
        assert!(!r.blocked);
        assert_eq!(r.approvals_avoided, 1);
    }

    #[test]
    fn risk_deltas_keyed_by_tag() {
        let p = policy(
            AutonomyLevel::AutoExternalWrite,
            RiskTag::Destructive,
            false,
            false,
        );
        let events = vec![
            event(1, RiskTag::Low, true, false, false, false, 10),
            event(2, RiskTag::Low, true, false, false, false, 10),
            event(3, RiskTag::Destructive, false, false, true, false, 10),
        ];
        let r = simulate_policy(&p, &events, true, false);
        assert_eq!(r.risk_deltas.get("low"), Some(&2));
        assert_eq!(r.risk_deltas.get("destructive"), Some(&1));
    }

    #[test]
    fn report_always_carries_disclaimer_with_sampling_caveat() {
        let p = policy(AutonomyLevel::Observe, RiskTag::Low, false, false);
        let r = simulate_policy(&p, &[], true, false);
        assert_eq!(r.evidence_disclaimer, EVIDENCE_DISCLAIMER);
        assert!(r.evidence_disclaimer.contains("Historical-replay"));
        assert!(r
            .evidence_disclaimer
            .to_lowercase()
            .contains("sampling caveat"));
    }

    #[test]
    fn simulation_is_deterministic() {
        let p = policy(
            AutonomyLevel::AutoExternalWrite,
            RiskTag::Destructive,
            true,
            false,
        );
        let events = vec![
            event(1, RiskTag::Low, true, false, false, false, 50),
            event(2, RiskTag::Destructive, false, true, true, true, 70),
            event(2, RiskTag::High, true, false, false, false, 30),
        ];
        let a = simulate_policy(&p, &events, false, false);
        let b = simulate_policy(&p, &events, false, false);
        assert_eq!(a, b);
    }

    #[test]
    fn rollout_refused_when_blocked() {
        let p = policy(
            AutonomyLevel::AutoExternalWrite,
            RiskTag::Destructive,
            true,
            false,
        );
        let events = vec![event(1, RiskTag::Destructive, false, false, true, false, 0)];
        let r = simulate_policy(&p, &events, false, false);
        assert!(r.blocked);
        let approver = UserId::new("u-1").unwrap_or_else(|e| panic!("{e}"));
        let res = propose_rollout(&p, &r, approver, ts(2_000));
        assert!(matches!(res, Err(RolloutError::Blocked)));
    }

    #[test]
    fn rollout_succeeds_when_not_blocked() {
        let p = policy(AutonomyLevel::AutoLowRisk, RiskTag::Low, false, false);
        let events = vec![event(1, RiskTag::Low, true, false, false, false, 120)];
        let r = simulate_policy(&p, &events, true, false);
        assert!(!r.blocked);
        let approver = UserId::new("u-1").unwrap_or_else(|e| panic!("{e}"));
        let rec = propose_rollout(&p, &r, approver, ts(2_000)).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(rec.policy_id, "pol-1");
        assert_eq!(rec.policy_version, 1);
        assert!(rec.monitoring_required);
        assert!(rec.simulation_summary.contains("approvals_avoided=1"));
    }
}
