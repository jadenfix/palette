//! beater-identity — agent identity registry, capability leases, and emergency
//! kill switches (issue #267).
//!
//! Everything here is in-memory and deterministic: the [`IdentityRegistry`]
//! holds identities, leases, kill switches, and an append-only audit log in
//! `BTreeMap`/`Vec` containers and never reaches for wall-clock time on its own
//! — callers pass `now` explicitly so behavior is reproducible in tests.

use std::collections::{BTreeMap, BTreeSet};

use beater_core::{AgentId, EnvironmentId, Money, ProjectId, Timestamp, UserId};
use beater_schema::ModelRef;
use serde::{Deserialize, Serialize};

/// Identifier for a [`CapabilityLease`].
pub type LeaseId = String;

/// The set of things an [`AgentIdentity`] is permitted to do.
///
/// All collections are `BTreeSet`s so serialization and iteration are
/// deterministic.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CapabilitySet {
    /// Tool names the agent may invoke.
    pub allowed_tools: BTreeSet<String>,
    /// Connector names the agent may reach.
    pub allowed_connectors: BTreeSet<String>,
    /// Command classes the agent may run.
    pub command_classes: BTreeSet<String>,
    /// External-write scopes the agent may target. A literal `"*"` is treated
    /// as an overly broad scope by [`IdentityRegistry::report_findings`].
    pub external_write_scopes: BTreeSet<String>,
    /// Data classes the agent may touch.
    pub data_classes: BTreeSet<String>,
    /// When true, otherwise-allowed actions still require an explicit approval
    /// lease at authorization time.
    pub requires_approval: bool,
    /// Optional spend ceiling for the agent.
    pub max_cost: Option<Money>,
    /// Optional wall-clock runtime ceiling, in seconds.
    pub max_runtime_secs: Option<u64>,
}

impl CapabilitySet {
    /// Whether `scope` is allowed by this set: it must appear in one of the
    /// scope-bearing collections (tools, connectors, or external-write scopes).
    fn allows_scope(&self, scope: &str) -> bool {
        self.allowed_tools.contains(scope)
            || self.allowed_connectors.contains(scope)
            || self.external_write_scopes.contains(scope)
    }
}

/// A registered agent identity and the capabilities it currently holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AgentIdentity {
    /// Stable identifier for the agent.
    pub agent_id: AgentId,
    /// Human-readable name.
    pub display_name: String,
    /// The owning user. An empty owner is flagged by `report_findings`.
    pub owner: UserId,
    /// Host the agent runs from.
    pub source_host: String,
    /// Model the agent is bound to, if known.
    pub model: Option<ModelRef>,
    /// Project the identity belongs to.
    pub project_id: ProjectId,
    /// Environment the identity belongs to.
    pub environment_id: EnvironmentId,
    /// When the identity was registered.
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
    /// Last time the agent was observed active.
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seen_at: Option<Timestamp>,
    /// When the identity was disabled, if it has been.
    #[schema(value_type = Option<String>, format = DateTime)]
    pub disabled_at: Option<Timestamp>,
    /// Capabilities granted to the agent.
    pub capabilities: CapabilitySet,
}

impl AgentIdentity {
    /// Whether the identity is currently disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled_at.is_some()
    }
}

/// A time-boxed, reviewer-approved grant of additional scopes to an agent.
///
/// Leases gate *elevated* actions: an [`AuthorizationDecision`] for an elevated
/// [`RequestedAction`] requires an active lease covering the requested scope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CapabilityLease {
    /// Identifier for this lease.
    pub lease_id: LeaseId,
    /// Agent the lease applies to.
    pub agent_id: AgentId,
    /// Why the lease was granted.
    pub reason: String,
    /// Reviewer who approved the lease.
    pub reviewer: UserId,
    /// Scopes this lease unlocks.
    pub granted_scopes: BTreeSet<String>,
    /// When the lease became valid.
    #[schema(value_type = String, format = DateTime)]
    pub issued_at: Timestamp,
    /// When the lease expires.
    #[schema(value_type = String, format = DateTime)]
    pub expires_at: Timestamp,
    /// When the lease was revoked, if it has been.
    #[schema(value_type = Option<String>, format = DateTime)]
    pub revoked_at: Option<Timestamp>,
}

impl CapabilityLease {
    /// Whether the lease is active at `now`: issued, not yet expired, and not
    /// revoked.
    pub fn is_active(&self, now: Timestamp) -> bool {
        if let Some(revoked_at) = self.revoked_at
            && revoked_at <= now
        {
            return false;
        }
        self.issued_at <= now && now < self.expires_at
    }

    /// Whether the lease is active at `now` *and* covers `scope`.
    fn covers(&self, scope: &str, now: Timestamp) -> bool {
        self.is_active(now) && self.granted_scopes.contains(scope)
    }
}

/// What a [`KillSwitch`] blocks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KillScope {
    /// Block all actions by one agent.
    Agent(AgentId),
    /// Block one capability for one agent.
    Capability {
        /// Agent whose capability is blocked.
        agent: AgentId,
        /// The blocked capability/scope name.
        capability: String,
    },
    /// Block a connector for every agent.
    Connector(String),
    /// Block a routine for every agent.
    Routine(String),
    /// Block all actions within an environment.
    Environment(EnvironmentId),
    /// Block every external write across the board.
    AllExternalWrites,
}

/// An engaged emergency kill switch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KillSwitch {
    /// What this switch blocks.
    pub scope: KillScope,
    /// Why it was engaged.
    pub reason: String,
    /// When it was engaged.
    #[schema(value_type = String, format = DateTime)]
    pub engaged_at: Timestamp,
    /// Who engaged it.
    pub engaged_by: UserId,
}

impl KillSwitch {
    /// Whether this switch blocks `action` for `identity`. The `identity` is
    /// supplied so environment-scoped switches can match on the agent's home
    /// environment.
    fn blocks(&self, action: &RequestedAction, identity: &AgentIdentity) -> bool {
        match &self.scope {
            KillScope::Agent(agent) => *agent == action.agent_id,
            KillScope::Capability { agent, capability } => {
                *agent == action.agent_id
                    && (action.tool.as_deref() == Some(capability.as_str())
                        || action.connector.as_deref() == Some(capability.as_str())
                        || action.external_write_scope.as_deref() == Some(capability.as_str()))
            }
            KillScope::Connector(connector) => {
                action.connector.as_deref() == Some(connector.as_str())
            }
            KillScope::Routine(routine) => action.tool.as_deref() == Some(routine.as_str()),
            KillScope::Environment(environment) => *environment == identity.environment_id,
            KillScope::AllExternalWrites => action.external_write_scope.is_some(),
        }
    }
}

/// A single action an agent wants to perform, evaluated by
/// [`IdentityRegistry::authorize`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RequestedAction {
    /// Agent requesting the action.
    pub agent_id: AgentId,
    /// Tool the action would invoke, if any.
    pub tool: Option<String>,
    /// Connector the action would use, if any.
    pub connector: Option<String>,
    /// External-write scope the action targets, if any.
    pub external_write_scope: Option<String>,
    /// Whether the action is elevated and therefore requires an active lease.
    pub elevated: bool,
}

impl RequestedAction {
    /// The scope strings this action needs the capability set / a lease to
    /// cover, in deterministic order.
    fn required_scopes(&self) -> Vec<String> {
        let mut scopes = Vec::new();
        if let Some(tool) = &self.tool {
            scopes.push(tool.clone());
        }
        if let Some(connector) = &self.connector {
            scopes.push(connector.clone());
        }
        if let Some(scope) = &self.external_write_scope {
            scopes.push(scope.clone());
        }
        scopes
    }
}

/// The outcome of authorizing a [`RequestedAction`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationDecision {
    /// The action is permitted.
    Allow,
    /// The action is denied, with a human-readable reason.
    Deny {
        /// Why the action was denied.
        reason: String,
    },
}

impl AuthorizationDecision {
    /// Whether this decision is [`AuthorizationDecision::Allow`].
    pub fn is_allowed(&self) -> bool {
        matches!(self, AuthorizationDecision::Allow)
    }

    fn deny(reason: impl Into<String>) -> Self {
        AuthorizationDecision::Deny {
            reason: reason.into(),
        }
    }
}

/// The kind of action recorded in an [`IdentityAuditEntry`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// An authorization decision was made.
    Authorize,
    /// A kill switch was engaged.
    EngageKillSwitch,
    /// A kill switch was released.
    ReleaseKillSwitch,
    /// A capability lease was granted.
    GrantLease,
    /// A capability lease was revoked.
    RevokeLease,
}

/// An append-only audit record of an authorization or kill-switch operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IdentityAuditEntry {
    /// What happened.
    pub action: AuditAction,
    /// The target the action applied to (agent id, lease id, scope, ...).
    pub target: String,
    /// Human-readable outcome of the action.
    pub outcome: String,
    /// When the action happened.
    #[schema(value_type = String, format = DateTime)]
    pub at: Timestamp,
}

/// A governance finding produced by [`IdentityRegistry::report_findings`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Finding {
    /// The identity has no owner set.
    NoOwner {
        /// Affected agent.
        agent_id: AgentId,
    },
    /// The identity has not been seen within the configured threshold (or has
    /// never been seen at all).
    Unused {
        /// Affected agent.
        agent_id: AgentId,
        /// Last time the agent was seen, if ever.
        #[schema(value_type = Option<String>, format = DateTime)]
        last_seen_at: Option<Timestamp>,
    },
    /// A lease referenced by the agent has expired but is still on record.
    ExpiredLeaseStillReferenced {
        /// Affected agent.
        agent_id: AgentId,
        /// The expired lease.
        lease_id: LeaseId,
    },
    /// The identity holds an overly broad scope (e.g. `"*"`).
    BroadScope {
        /// Affected agent.
        agent_id: AgentId,
        /// The offending scope.
        scope: String,
    },
}

/// Errors returned by [`IdentityRegistry`] operations.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// No identity is registered for the given agent.
    #[error("unknown agent identity: {0}")]
    UnknownAgent(AgentId),
    /// An identity already exists for the given agent.
    #[error("agent identity already registered: {0}")]
    AlreadyRegistered(AgentId),
    /// No lease exists for the given id.
    #[error("unknown lease: {0}")]
    UnknownLease(LeaseId),
}

/// In-memory registry of identities, leases, kill switches, and an audit log.
#[derive(Clone, Debug, Default)]
pub struct IdentityRegistry {
    identities: BTreeMap<AgentId, AgentIdentity>,
    leases: BTreeMap<LeaseId, CapabilityLease>,
    kill_switches: Vec<KillSwitch>,
    audit_log: Vec<IdentityAuditEntry>,
}

impl IdentityRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    // --- identities ---------------------------------------------------------

    /// Register a new identity. Errors if one already exists for the agent.
    pub fn register(&mut self, identity: AgentIdentity) -> Result<(), RegistryError> {
        if self.identities.contains_key(&identity.agent_id) {
            return Err(RegistryError::AlreadyRegistered(identity.agent_id));
        }
        self.identities.insert(identity.agent_id.clone(), identity);
        Ok(())
    }

    /// Get an identity by agent id.
    pub fn get(&self, agent_id: &AgentId) -> Option<&AgentIdentity> {
        self.identities.get(agent_id)
    }

    /// List all identities in deterministic (agent-id) order.
    pub fn list(&self) -> Vec<&AgentIdentity> {
        self.identities.values().collect()
    }

    /// Disable an identity at `now`. Idempotent: disabling an already-disabled
    /// identity keeps the original `disabled_at`.
    pub fn disable(&mut self, agent_id: &AgentId, now: Timestamp) -> Result<(), RegistryError> {
        let identity = self
            .identities
            .get_mut(agent_id)
            .ok_or_else(|| RegistryError::UnknownAgent(agent_id.clone()))?;
        if identity.disabled_at.is_none() {
            identity.disabled_at = Some(now);
        }
        Ok(())
    }

    /// Record that an agent was seen at `now`.
    pub fn mark_seen(&mut self, agent_id: &AgentId, now: Timestamp) -> Result<(), RegistryError> {
        let identity = self
            .identities
            .get_mut(agent_id)
            .ok_or_else(|| RegistryError::UnknownAgent(agent_id.clone()))?;
        identity.last_seen_at = Some(now);
        Ok(())
    }

    // --- leases -------------------------------------------------------------

    /// Grant a capability lease. The agent must be registered.
    pub fn grant_lease(&mut self, lease: CapabilityLease) -> Result<(), RegistryError> {
        if !self.identities.contains_key(&lease.agent_id) {
            return Err(RegistryError::UnknownAgent(lease.agent_id));
        }
        self.audit(IdentityAuditEntry {
            action: AuditAction::GrantLease,
            target: lease.lease_id.clone(),
            outcome: format!("granted to {}", lease.agent_id),
            at: lease.issued_at,
        });
        self.leases.insert(lease.lease_id.clone(), lease);
        Ok(())
    }

    /// Revoke a lease at `now`. Idempotent on the `revoked_at` field.
    pub fn revoke_lease(&mut self, lease_id: &str, now: Timestamp) -> Result<(), RegistryError> {
        let lease = self
            .leases
            .get_mut(lease_id)
            .ok_or_else(|| RegistryError::UnknownLease(lease_id.to_string()))?;
        if lease.revoked_at.is_none() {
            lease.revoked_at = Some(now);
        }
        self.audit(IdentityAuditEntry {
            action: AuditAction::RevokeLease,
            target: lease_id.to_string(),
            outcome: "revoked".to_string(),
            at: now,
        });
        Ok(())
    }

    /// Leases on record for an agent, in deterministic (lease-id) order.
    pub fn leases_for(&self, agent_id: &AgentId) -> Vec<&CapabilityLease> {
        self.leases
            .values()
            .filter(|lease| lease.agent_id == *agent_id)
            .collect()
    }

    // --- kill switches ------------------------------------------------------

    /// Engage a kill switch.
    pub fn engage_kill_switch(&mut self, switch: KillSwitch) {
        self.audit(IdentityAuditEntry {
            action: AuditAction::EngageKillSwitch,
            target: kill_scope_target(&switch.scope),
            outcome: format!("engaged by {}", switch.engaged_by),
            at: switch.engaged_at,
        });
        self.kill_switches.push(switch);
    }

    /// Release every kill switch whose scope equals `scope`. Returns the number
    /// released. `now`/`released_by` are recorded in the audit log.
    pub fn release_kill_switch(
        &mut self,
        scope: &KillScope,
        now: Timestamp,
        released_by: &UserId,
    ) -> usize {
        let before = self.kill_switches.len();
        self.kill_switches.retain(|switch| switch.scope != *scope);
        let released = before - self.kill_switches.len();
        self.audit(IdentityAuditEntry {
            action: AuditAction::ReleaseKillSwitch,
            target: kill_scope_target(scope),
            outcome: format!("released {released} switch(es) by {released_by}"),
            at: now,
        });
        released
    }

    /// Currently engaged kill switches.
    pub fn kill_switches(&self) -> &[KillSwitch] {
        &self.kill_switches
    }

    // --- authorization ------------------------------------------------------

    /// Authorize a [`RequestedAction`] at `now`, appending the decision to the
    /// audit log.
    ///
    /// Checks run in order:
    /// 1. the identity exists and is not disabled;
    /// 2. no engaged kill switch blocks the action's scope;
    /// 3. the agent's capability set covers every required scope;
    /// 4. if the action is elevated (or the capability set requires approval),
    ///    an active lease must cover every required scope.
    pub fn authorize(&mut self, action: &RequestedAction, now: Timestamp) -> AuthorizationDecision {
        let decision = self.evaluate(action, now);
        self.audit(IdentityAuditEntry {
            action: AuditAction::Authorize,
            target: action.agent_id.to_string(),
            outcome: match &decision {
                AuthorizationDecision::Allow => "allow".to_string(),
                AuthorizationDecision::Deny { reason } => format!("deny: {reason}"),
            },
            at: now,
        });
        decision
    }

    fn evaluate(&self, action: &RequestedAction, now: Timestamp) -> AuthorizationDecision {
        // 1. identity exists and is enabled.
        let Some(identity) = self.identities.get(&action.agent_id) else {
            return AuthorizationDecision::deny(format!(
                "unknown agent identity: {}",
                action.agent_id
            ));
        };
        if identity.is_disabled() {
            return AuthorizationDecision::deny(format!(
                "agent identity disabled: {}",
                action.agent_id
            ));
        }

        // 2. no kill switch blocks the action.
        for switch in &self.kill_switches {
            if switch.blocks(action, identity) {
                return AuthorizationDecision::deny(format!(
                    "blocked by kill switch: {}",
                    kill_scope_target(&switch.scope)
                ));
            }
        }

        let required = action.required_scopes();

        // 3. capability set covers every required scope.
        for scope in &required {
            if !identity.capabilities.allows_scope(scope) {
                return AuthorizationDecision::deny(format!("missing capability: {scope}"));
            }
        }

        // 4. elevated (or approval-gated) actions need an active covering lease.
        if action.elevated || identity.capabilities.requires_approval {
            for scope in &required {
                let covered = self
                    .leases
                    .values()
                    .any(|lease| lease.agent_id == action.agent_id && lease.covers(scope, now));
                if !covered {
                    return AuthorizationDecision::deny(format!(
                        "elevated action requires an active lease for scope: {scope}"
                    ));
                }
            }
        }

        AuthorizationDecision::Allow
    }

    // --- audit & findings ---------------------------------------------------

    /// The append-only audit log.
    pub fn audit_log(&self) -> &[IdentityAuditEntry] {
        &self.audit_log
    }

    fn audit(&mut self, entry: IdentityAuditEntry) {
        self.audit_log.push(entry);
    }

    /// Report governance findings as of `now`. `unused_after_secs` is the age
    /// threshold (in seconds) past which an identity counts as unused.
    ///
    /// Findings are returned in deterministic order: identities are visited in
    /// agent-id order, and per identity in finding-kind order (no owner,
    /// unused, expired lease, broad scope).
    pub fn report_findings(&self, now: Timestamp, unused_after_secs: u64) -> Vec<Finding> {
        let mut findings = Vec::new();
        let threshold = chrono::Duration::seconds(unused_after_secs as i64);

        for identity in self.identities.values() {
            // No owner.
            if identity.owner.as_str().is_empty() {
                findings.push(Finding::NoOwner {
                    agent_id: identity.agent_id.clone(),
                });
            }

            // Unused: never seen, or last seen older than the threshold.
            let stale = match identity.last_seen_at {
                None => true,
                Some(last_seen) => now.signed_duration_since(last_seen) > threshold,
            };
            if stale {
                findings.push(Finding::Unused {
                    agent_id: identity.agent_id.clone(),
                    last_seen_at: identity.last_seen_at,
                });
            }

            // Expired lease still on record for this agent.
            for lease in self.leases.values() {
                if lease.agent_id == identity.agent_id
                    && lease.revoked_at.is_none()
                    && lease.expires_at <= now
                {
                    findings.push(Finding::ExpiredLeaseStillReferenced {
                        agent_id: identity.agent_id.clone(),
                        lease_id: lease.lease_id.clone(),
                    });
                }
            }

            // Broad scope.
            if identity.capabilities.external_write_scopes.contains("*") {
                findings.push(Finding::BroadScope {
                    agent_id: identity.agent_id.clone(),
                    scope: "*".to_string(),
                });
            }
        }

        findings
    }
}

/// Render a [`KillScope`] as a stable, human-readable audit target string.
fn kill_scope_target(scope: &KillScope) -> String {
    match scope {
        KillScope::Agent(agent) => format!("agent:{agent}"),
        KillScope::Capability { agent, capability } => {
            format!("capability:{agent}/{capability}")
        }
        KillScope::Connector(connector) => format!("connector:{connector}"),
        KillScope::Routine(routine) => format!("routine:{routine}"),
        KillScope::Environment(environment) => format!("environment:{environment}"),
        KillScope::AllExternalWrites => "all_external_writes".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{Clock, FixedClock};
    use chrono::{TimeZone, Utc};

    fn ts(secs: i64) -> Timestamp {
        Utc.timestamp_opt(secs, 0)
            .single()
            .unwrap_or_else(|| panic!("valid timestamp for {secs}"))
    }

    fn agent(id: &str) -> AgentId {
        AgentId::new(id).unwrap_or_else(|e| panic!("{e}"))
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).unwrap_or_else(|e| panic!("{e}"))
    }

    fn project() -> ProjectId {
        ProjectId::new("proj").unwrap_or_else(|e| panic!("{e}"))
    }

    fn environment(id: &str) -> EnvironmentId {
        EnvironmentId::new(id).unwrap_or_else(|e| panic!("{e}"))
    }

    fn base_identity(id: &str) -> AgentIdentity {
        let mut caps = CapabilitySet::default();
        caps.allowed_tools.insert("search".to_string());
        caps.external_write_scopes
            .insert("github:write".to_string());
        AgentIdentity {
            agent_id: agent(id),
            display_name: format!("agent {id}"),
            owner: user("owner-1"),
            source_host: "host-1".to_string(),
            model: None,
            project_id: project(),
            environment_id: environment("prod"),
            created_at: ts(0),
            last_seen_at: Some(ts(0)),
            disabled_at: None,
            capabilities: caps,
        }
    }

    fn lease(id: &str, agent_id: &str, scope: &str, issued: i64, expires: i64) -> CapabilityLease {
        let mut scopes = BTreeSet::new();
        scopes.insert(scope.to_string());
        CapabilityLease {
            lease_id: id.to_string(),
            agent_id: agent(agent_id),
            reason: "needed".to_string(),
            reviewer: user("reviewer-1"),
            granted_scopes: scopes,
            issued_at: ts(issued),
            expires_at: ts(expires),
            revoked_at: None,
        }
    }

    fn registered() -> IdentityRegistry {
        let mut registry = IdentityRegistry::new();
        registry
            .register(base_identity("a1"))
            .unwrap_or_else(|e| panic!("{e}"));
        registry
    }

    #[test]
    fn register_rejects_duplicates_and_get_list_work() {
        let mut registry = registered();
        assert!(registry.get(&agent("a1")).is_some());
        assert_eq!(registry.list().len(), 1);
        let err = registry.register(base_identity("a1"));
        assert_eq!(err, Err(RegistryError::AlreadyRegistered(agent("a1"))));
    }

    #[test]
    fn lease_is_active_window() {
        let mut l = lease("l1", "a1", "github:write", 10, 20);
        assert!(!l.is_active(ts(5)));
        assert!(l.is_active(ts(10)));
        assert!(l.is_active(ts(19)));
        assert!(!l.is_active(ts(20)));
        l.revoked_at = Some(ts(15));
        assert!(!l.is_active(ts(16)));
        assert!(l.is_active(ts(12)));
    }

    #[test]
    fn non_elevated_action_with_capability_allows() {
        let mut registry = registered();
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert_eq!(
            registry.authorize(&action, ts(100)),
            AuthorizationDecision::Allow
        );
    }

    #[test]
    fn missing_capability_denies() {
        let mut registry = registered();
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("delete".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        let decision = registry.authorize(&action, ts(100));
        assert_eq!(
            decision,
            AuthorizationDecision::deny("missing capability: delete")
        );
    }

    #[test]
    fn disabled_identity_denies() {
        let mut registry = registered();
        registry
            .disable(&agent("a1"), ts(50))
            .unwrap_or_else(|e| panic!("{e}"));
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn unknown_agent_denies() {
        let mut registry = registered();
        let action = RequestedAction {
            agent_id: agent("nope"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn expired_lease_blocks_elevated_action() {
        let mut registry = registered();
        registry
            .grant_lease(lease("l1", "a1", "github:write", 0, 50))
            .unwrap_or_else(|e| panic!("{e}"));
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: None,
            connector: None,
            external_write_scope: Some("github:write".to_string()),
            elevated: true,
        };
        // At t=100 the lease (expires at 50) is gone -> denied.
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn active_lease_allows_elevated_action() {
        let mut registry = registered();
        registry
            .grant_lease(lease("l1", "a1", "github:write", 0, 200))
            .unwrap_or_else(|e| panic!("{e}"));
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: None,
            connector: None,
            external_write_scope: Some("github:write".to_string()),
            elevated: true,
        };
        assert_eq!(
            registry.authorize(&action, ts(100)),
            AuthorizationDecision::Allow
        );
    }

    #[test]
    fn elevated_without_any_lease_denies() {
        let mut registry = registered();
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: None,
            connector: None,
            external_write_scope: Some("github:write".to_string()),
            elevated: true,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn revoked_lease_blocks_elevated_action() {
        let mut registry = registered();
        registry
            .grant_lease(lease("l1", "a1", "github:write", 0, 200))
            .unwrap_or_else(|e| panic!("{e}"));
        registry
            .revoke_lease("l1", ts(50))
            .unwrap_or_else(|e| panic!("{e}"));
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: None,
            connector: None,
            external_write_scope: Some("github:write".to_string()),
            elevated: true,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn agent_kill_switch_blocks() {
        let mut registry = registered();
        registry.engage_kill_switch(KillSwitch {
            scope: KillScope::Agent(agent("a1")),
            reason: "incident".to_string(),
            engaged_at: ts(10),
            engaged_by: user("admin"),
        });
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn all_external_writes_kill_switch_blocks_and_releases() {
        let mut registry = registered();
        registry.engage_kill_switch(KillSwitch {
            scope: KillScope::AllExternalWrites,
            reason: "freeze".to_string(),
            engaged_at: ts(10),
            engaged_by: user("admin"),
        });
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: None,
            connector: None,
            external_write_scope: Some("github:write".to_string()),
            elevated: false,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());

        let released =
            registry.release_kill_switch(&KillScope::AllExternalWrites, ts(110), &user("admin"));
        assert_eq!(released, 1);
        assert!(registry.authorize(&action, ts(120)).is_allowed());
    }

    #[test]
    fn environment_kill_switch_blocks_by_home_environment() {
        let mut registry = registered();
        registry.engage_kill_switch(KillSwitch {
            scope: KillScope::Environment(environment("prod")),
            reason: "env freeze".to_string(),
            engaged_at: ts(10),
            engaged_by: user("admin"),
        });
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn requires_approval_forces_lease_even_when_not_elevated() {
        let mut registry = IdentityRegistry::new();
        let mut identity = base_identity("a2");
        identity.capabilities.requires_approval = true;
        registry
            .register(identity)
            .unwrap_or_else(|e| panic!("{e}"));
        let action = RequestedAction {
            agent_id: agent("a2"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert!(!registry.authorize(&action, ts(100)).is_allowed());
    }

    #[test]
    fn audit_log_records_decisions_and_kill_switches() {
        let mut registry = registered();
        registry.engage_kill_switch(KillSwitch {
            scope: KillScope::Agent(agent("a1")),
            reason: "incident".to_string(),
            engaged_at: ts(10),
            engaged_by: user("admin"),
        });
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        let _ = registry.authorize(&action, ts(100));
        let log = registry.audit_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].action, AuditAction::EngageKillSwitch);
        assert_eq!(log[1].action, AuditAction::Authorize);
        assert!(log[1].outcome.starts_with("deny"));
    }

    #[test]
    fn report_findings_flags_stale_and_broad_scope() {
        let mut registry = IdentityRegistry::new();

        // Stale (last seen long ago) + broad external-write scope.
        let mut bad = base_identity("a1");
        bad.last_seen_at = Some(ts(0));
        bad.capabilities
            .external_write_scopes
            .insert("*".to_string());
        registry.register(bad).unwrap_or_else(|e| panic!("{e}"));

        let findings = registry.report_findings(ts(10_000), 3_600);
        // Unused (last seen at 0, now 10000, threshold 3600) and broad scope.
        assert!(findings.iter().any(|f| matches!(f, Finding::Unused { .. })));
        assert!(
            findings
                .iter()
                .any(|f| matches!(f, Finding::BroadScope { scope, .. } if scope == "*"))
        );
    }

    #[test]
    fn report_findings_flags_no_owner_when_owner_empty() {
        // Build an identity then force an empty owner by constructing through
        // serde to bypass the non-empty newtype invariant deterministically.
        let mut identity = base_identity("a3");
        identity.last_seen_at = Some(ts(9_999)); // fresh, so only no-owner fires
        let json = serde_json::to_value(&identity).unwrap_or_else(|e| panic!("{e}"));
        let mut json = json;
        json["owner"] = serde_json::Value::String(String::new());
        let identity: AgentIdentity =
            serde_json::from_value(json).unwrap_or_else(|e| panic!("{e}"));
        assert!(identity.owner.as_str().is_empty());

        let mut registry = IdentityRegistry::new();
        registry
            .register(identity)
            .unwrap_or_else(|e| panic!("{e}"));
        let findings = registry.report_findings(ts(10_000), 3_600);
        assert!(
            findings
                .iter()
                .any(|f| matches!(f, Finding::NoOwner { .. }))
        );
    }

    #[test]
    fn report_findings_flags_expired_lease_still_referenced() {
        let mut registry = registered();
        registry
            .grant_lease(lease("l1", "a1", "github:write", 0, 50))
            .unwrap_or_else(|e| panic!("{e}"));
        let findings = registry.report_findings(ts(100), 1_000_000);
        assert!(findings.iter().any(
            |f| matches!(f, Finding::ExpiredLeaseStillReferenced { lease_id, .. } if lease_id == "l1")
        ));
    }

    #[test]
    fn fixed_clock_drives_now_deterministically() {
        let clock = FixedClock::new(ts(100));
        let mut registry = registered();
        let action = RequestedAction {
            agent_id: agent("a1"),
            tool: Some("search".to_string()),
            connector: None,
            external_write_scope: None,
            elevated: false,
        };
        assert!(registry.authorize(&action, clock.now()).is_allowed());
    }
}
