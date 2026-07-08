//! beater-receipts — agent action receipts for transactions, external writes,
//! and user-visible changes (GitHub issue #269).
//!
//! Receipts are stored in an append-only SHA-256 hash chain, mirroring the
//! pattern in `beater-audit`: each record carries the `prev_hash` of the record
//! before it and a `hash` computed over the canonical JSON of the record
//! *without* its own `hash` field. Tampering with any stored record (including
//! reordering or editing a middle record) breaks the chain and is detected by
//! [`ReceiptLedger::verify_chain`].
//!
//! `before_summary` / `after_summary` are REDACTED, human-readable summaries —
//! never raw secrets. The `risk_class` ([`RedactionClass`]) records how
//! sensitive the underlying action is.

use beater_core::{
    AgentId, Clock, Sha256Hash, SpanId, TenantScope, Timestamp, TraceId, UserId, sha256_json_hash,
};
use beater_schema::RedactionClass;
use serde::{Deserialize, Serialize};

/// The all-zero hash used as the genesis `prev_hash` for the first receipt.
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Returns the genesis (all-zero) chain hash.
fn genesis_hash() -> Sha256Hash {
    // The constant is a valid 64-char hash identifier (no whitespace, non-empty).
    Sha256Hash::new(GENESIS_HASH).unwrap_or_else(|err| panic!("genesis hash invalid: {err}"))
}

/// Errors that can occur while verifying or mutating a receipt chain.
#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    /// A record's stored `prev_hash` does not match the previous record's `hash`
    /// (or the genesis hash for the first record).
    #[error(
        "receipt {receipt_id} at seq {seq}: prev_hash mismatch (expected {expected}, found {found})"
    )]
    PrevHashMismatch {
        receipt_id: String,
        seq: u64,
        expected: String,
        found: String,
    },
    /// A record's stored `hash` does not match the hash recomputed over its
    /// contents — the record (or its ordering) was tampered with.
    #[error(
        "receipt {receipt_id} at seq {seq}: hash mismatch (expected {expected}, found {found})"
    )]
    HashMismatch {
        receipt_id: String,
        seq: u64,
        expected: String,
        found: String,
    },
    /// A record's `seq` is not the expected monotonically increasing value.
    #[error("receipt {receipt_id}: seq mismatch (expected {expected}, found {found})")]
    SeqMismatch {
        receipt_id: String,
        expected: u64,
        found: u64,
    },
    /// Failed to compute a record hash.
    #[error("failed to hash receipt: {0}")]
    Hash(#[from] beater_core::JsonHashError),
    /// A referenced original receipt could not be found in the ledger.
    #[error("original receipt {0} not found")]
    OriginalNotFound(String),
}

/// The kind of action a receipt records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Payment,
    Refund,
    Message,
    Ticket,
    PullRequest,
    DbWrite,
    Deploy,
    Other,
}

/// How the action was authorized.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Authorization {
    pub approval_id: Option<String>,
    pub lease_id: Option<String>,
    pub authorizer: Option<UserId>,
    pub policy_id: Option<String>,
}

/// A reference to an object in an external system the action touched.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExternalObjectRef {
    pub system: String,
    pub object_id: String,
    pub url: Option<String>,
}

/// The outcome of the action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Succeeded,
    Failed,
    Pending,
}

/// Whether and how an action can be undone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Reversibility {
    /// The action can be reversed, optionally before a deadline, via `method`.
    Reversible {
        #[schema(value_type = Option<String>, format = DateTime)]
        deadline: Option<Timestamp>,
        method: String,
    },
    /// The action cannot be reversed.
    Irreversible,
    /// The action has already been reversed: `reversal_receipt_id` is the id of
    /// the later receipt that reversed it. Set on the ORIGINAL receipt and
    /// points *forward* to the receipt that undid it.
    Reversed { reversal_receipt_id: String },
    /// The action itself reverses an earlier action: `original_receipt_id` is the
    /// id of the receipt being undone. Set on the REVERSAL receipt and points
    /// *back* to the original it reverses.
    Reverses { original_receipt_id: String },
}

/// Fields supplied by the caller when appending a receipt. The ledger derives
/// `receipt_id` (if absent), `created_at`, `seq`, `prev_hash`, and `hash`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReceiptInput {
    /// Optional caller-supplied receipt id; a UUID is generated when absent.
    pub receipt_id: Option<String>,
    pub scope: TenantScope,
    pub agent_id: AgentId,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub action_kind: ActionKind,
    pub authorization: Authorization,
    pub external_object_ref: Option<ExternalObjectRef>,
    /// REDACTED summary of state before the action — never raw secrets.
    pub before_summary: Option<String>,
    /// REDACTED summary of state after the action — never raw secrets.
    pub after_summary: Option<String>,
    pub outcome: Outcome,
    pub reversibility: Reversibility,
    pub risk_class: RedactionClass,
}

/// An immutable, hash-chained record of an agent action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AgentActionReceipt {
    pub receipt_id: String,
    pub scope: TenantScope,
    pub agent_id: AgentId,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub action_kind: ActionKind,
    pub authorization: Authorization,
    pub external_object_ref: Option<ExternalObjectRef>,
    /// REDACTED summary of state before the action — never raw secrets.
    pub before_summary: Option<String>,
    /// REDACTED summary of state after the action — never raw secrets.
    pub after_summary: Option<String>,
    pub outcome: Outcome,
    pub reversibility: Reversibility,
    pub risk_class: RedactionClass,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
    pub seq: u64,
    pub prev_hash: Sha256Hash,
    pub hash: Sha256Hash,
}

/// The set of fields a receipt's `hash` is computed over: every field of
/// [`AgentActionReceipt`] *except* `hash` itself. Borrowed for cheap hashing.
#[derive(Serialize)]
struct ReceiptHashView<'a> {
    receipt_id: &'a str,
    scope: &'a TenantScope,
    agent_id: &'a AgentId,
    trace_id: &'a Option<TraceId>,
    span_id: &'a Option<SpanId>,
    action_kind: &'a ActionKind,
    authorization: &'a Authorization,
    external_object_ref: &'a Option<ExternalObjectRef>,
    before_summary: &'a Option<String>,
    after_summary: &'a Option<String>,
    outcome: &'a Outcome,
    reversibility: &'a Reversibility,
    risk_class: &'a RedactionClass,
    created_at: &'a Timestamp,
    seq: u64,
    prev_hash: &'a Sha256Hash,
}

impl<'a> From<&'a AgentActionReceipt> for ReceiptHashView<'a> {
    fn from(r: &'a AgentActionReceipt) -> Self {
        Self {
            receipt_id: &r.receipt_id,
            scope: &r.scope,
            agent_id: &r.agent_id,
            trace_id: &r.trace_id,
            span_id: &r.span_id,
            action_kind: &r.action_kind,
            authorization: &r.authorization,
            external_object_ref: &r.external_object_ref,
            before_summary: &r.before_summary,
            after_summary: &r.after_summary,
            outcome: &r.outcome,
            reversibility: &r.reversibility,
            risk_class: &r.risk_class,
            created_at: &r.created_at,
            seq: r.seq,
            prev_hash: &r.prev_hash,
        }
    }
}

/// Computes the chain hash for a receipt over all of its fields except `hash`.
fn compute_hash(receipt: &AgentActionReceipt) -> Result<Sha256Hash, ChainError> {
    Ok(sha256_json_hash(&ReceiptHashView::from(receipt))?)
}

/// An in-memory, append-only ledger of agent action receipts forming a single
/// SHA-256 hash chain.
pub struct ReceiptLedger {
    clock: Box<dyn Clock>,
    receipts: Vec<AgentActionReceipt>,
}

impl ReceiptLedger {
    /// Creates a ledger that timestamps receipts using `clock`.
    pub fn new(clock: impl Clock + 'static) -> Self {
        Self {
            clock: Box::new(clock),
            receipts: Vec::new(),
        }
    }

    /// Appends a receipt to the chain, deriving `seq`, `prev_hash`, `created_at`,
    /// `hash`, and a `receipt_id` if none was supplied. Returns the new receipt.
    pub fn append(&mut self, input: ReceiptInput) -> Result<&AgentActionReceipt, ChainError> {
        let seq = self.receipts.len() as u64;
        let prev_hash = self
            .receipts
            .last()
            .map(|r| r.hash.clone())
            .unwrap_or_else(genesis_hash);
        let receipt_id = input
            .receipt_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let mut receipt = AgentActionReceipt {
            receipt_id,
            scope: input.scope,
            agent_id: input.agent_id,
            trace_id: input.trace_id,
            span_id: input.span_id,
            action_kind: input.action_kind,
            authorization: input.authorization,
            external_object_ref: input.external_object_ref,
            before_summary: input.before_summary,
            after_summary: input.after_summary,
            outcome: input.outcome,
            reversibility: input.reversibility,
            risk_class: input.risk_class,
            created_at: self.clock.now(),
            seq,
            prev_hash,
            // Placeholder; replaced immediately below before the record is stored.
            hash: genesis_hash(),
        };
        receipt.hash = compute_hash(&receipt)?;
        self.receipts.push(receipt);
        // `push` cannot leave the vec empty, so `last` is always `Some` here.
        self.receipts
            .last()
            .ok_or_else(|| ChainError::OriginalNotFound("just-appended receipt".to_string()))
    }

    /// Records the reversal of `original_receipt_id` by appending a NEW receipt.
    ///
    /// Linkage is bidirectional and points in the natural direction of each
    /// field's name:
    /// * the NEW reversal receipt's `reversibility` becomes
    ///   [`Reversibility::Reverses`] carrying the original's id (it records which
    ///   receipt it undoes);
    /// * the ORIGINAL receipt's `reversibility` becomes
    ///   [`Reversibility::Reversed`] carrying the new reversal receipt's id (it
    ///   points forward to the receipt that undid it).
    ///
    /// Because the original receipt is updated in place, every receipt from the
    /// original onward is re-hashed so the SHA-256 chain stays intact
    /// ([`ReceiptLedger::verify_chain`] still passes). Returns the
    /// [`ReversalLink`] connecting the two ids.
    pub fn record_reversal(
        &mut self,
        original_receipt_id: &str,
        mut reversal: ReceiptInput,
    ) -> Result<ReversalLink, ChainError> {
        let original_index = self
            .receipts
            .iter()
            .position(|r| r.receipt_id == original_receipt_id)
            .ok_or_else(|| ChainError::OriginalNotFound(original_receipt_id.to_string()))?;

        // The new receipt reverses the original: it points *back* at it.
        reversal.reversibility = Reversibility::Reverses {
            original_receipt_id: original_receipt_id.to_string(),
        };
        let reversal_receipt_id = self.append(reversal)?.receipt_id.clone();

        // The original has now been reversed: mark it pointing *forward* at the
        // new reversal receipt, then re-chain from the original onward so the
        // hash chain stays valid after the in-place edit.
        self.receipts[original_index].reversibility = Reversibility::Reversed {
            reversal_receipt_id: reversal_receipt_id.clone(),
        };
        self.rechain_from(original_index)?;

        Ok(ReversalLink {
            original_receipt_id: original_receipt_id.to_string(),
            reversal_receipt_id,
        })
    }

    /// Recomputes `prev_hash` and `hash` for every receipt from `start` to the
    /// end of the chain, preserving append-only ordering and `seq`. Used after an
    /// in-place edit to an existing receipt so [`ReceiptLedger::verify_chain`]
    /// keeps passing.
    fn rechain_from(&mut self, start: usize) -> Result<(), ChainError> {
        for index in start..self.receipts.len() {
            let prev_hash = if index == 0 {
                genesis_hash()
            } else {
                self.receipts[index - 1].hash.clone()
            };
            self.receipts[index].prev_hash = prev_hash;
            self.receipts[index].hash = compute_hash(&self.receipts[index])?;
        }
        Ok(())
    }

    /// Verifies the full chain: sequencing, `prev_hash` linkage, and per-record
    /// hashes. Returns `Ok(())` when intact, or the first detected [`ChainError`].
    pub fn verify_chain(&self) -> Result<(), ChainError> {
        let mut expected_prev = genesis_hash();
        for (index, receipt) in self.receipts.iter().enumerate() {
            let expected_seq = index as u64;
            if receipt.seq != expected_seq {
                return Err(ChainError::SeqMismatch {
                    receipt_id: receipt.receipt_id.clone(),
                    expected: expected_seq,
                    found: receipt.seq,
                });
            }
            if receipt.prev_hash != expected_prev {
                return Err(ChainError::PrevHashMismatch {
                    receipt_id: receipt.receipt_id.clone(),
                    seq: receipt.seq,
                    expected: expected_prev.as_str().to_string(),
                    found: receipt.prev_hash.as_str().to_string(),
                });
            }
            let recomputed = compute_hash(receipt)?;
            if recomputed != receipt.hash {
                return Err(ChainError::HashMismatch {
                    receipt_id: receipt.receipt_id.clone(),
                    seq: receipt.seq,
                    expected: recomputed.as_str().to_string(),
                    found: receipt.hash.as_str().to_string(),
                });
            }
            expected_prev = receipt.hash.clone();
        }
        Ok(())
    }

    /// Returns all receipts in chain order.
    pub fn list(&self) -> &[AgentActionReceipt] {
        &self.receipts
    }

    /// Returns receipts matching every supplied filter criterion.
    pub fn filter(&self, filter: &ReceiptFilter) -> Vec<&AgentActionReceipt> {
        self.receipts.iter().filter(|r| filter.matches(r)).collect()
    }

    /// Returns the receipt with the given id, if present.
    pub fn get(&self, receipt_id: &str) -> Option<&AgentActionReceipt> {
        self.receipts.iter().find(|r| r.receipt_id == receipt_id)
    }
}

/// Linkage returned by [`ReceiptLedger::record_reversal`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReversalLink {
    pub original_receipt_id: String,
    pub reversal_receipt_id: String,
}

/// Criteria for [`ReceiptLedger::filter`]. A `None` field matches everything.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReceiptFilter {
    pub agent_id: Option<AgentId>,
    pub outcome: Option<Outcome>,
    pub action_kind: Option<ActionKind>,
}

impl ReceiptFilter {
    fn matches(&self, receipt: &AgentActionReceipt) -> bool {
        if let Some(agent_id) = &self.agent_id
            && &receipt.agent_id != agent_id
        {
            return false;
        }
        if let Some(outcome) = self.outcome
            && receipt.outcome != outcome
        {
            return false;
        }
        if let Some(action_kind) = self.action_kind
            && receipt.action_kind != action_kind
        {
            return false;
        }
        true
    }

    /// Builder helper: filter by agent.
    pub fn by_agent(agent_id: AgentId) -> Self {
        Self {
            agent_id: Some(agent_id),
            ..Self::default()
        }
    }

    /// Builder helper: filter by outcome.
    pub fn by_outcome(outcome: Outcome) -> Self {
        Self {
            outcome: Some(outcome),
            ..Self::default()
        }
    }

    /// Builder helper: filter by action kind.
    pub fn by_action_kind(action_kind: ActionKind) -> Self {
        Self {
            action_kind: Some(action_kind),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{EnvironmentId, FixedClock, ProjectId, TenantId};
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::new(
            chrono::Utc
                .with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
                .single()
                .unwrap_or_else(|| panic!("valid timestamp")),
        )
    }

    fn scope() -> TenantScope {
        TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|e| panic!("{e}")),
            ProjectId::new("project").unwrap_or_else(|e| panic!("{e}")),
            EnvironmentId::new("prod").unwrap_or_else(|e| panic!("{e}")),
        )
    }

    fn agent(name: &str) -> AgentId {
        AgentId::new(name).unwrap_or_else(|e| panic!("{e}"))
    }

    fn input(
        receipt_id: &str,
        agent: AgentId,
        kind: ActionKind,
        outcome: Outcome,
        risk: RedactionClass,
    ) -> ReceiptInput {
        ReceiptInput {
            receipt_id: Some(receipt_id.to_string()),
            scope: scope(),
            agent_id: agent,
            trace_id: None,
            span_id: None,
            action_kind: kind,
            authorization: Authorization::default(),
            external_object_ref: None,
            before_summary: Some("[redacted before]".to_string()),
            after_summary: Some("[redacted after]".to_string()),
            outcome,
            reversibility: Reversibility::Irreversible,
            risk_class: risk,
        }
    }

    fn ledger_with_three() -> ReceiptLedger {
        let mut ledger = ReceiptLedger::new(clock());
        for (id, kind, outcome) in [
            ("r0", ActionKind::Payment, Outcome::Succeeded),
            ("r1", ActionKind::Message, Outcome::Pending),
            ("r2", ActionKind::Deploy, Outcome::Failed),
        ] {
            ledger
                .append(input(
                    id,
                    agent("agent-a"),
                    kind,
                    outcome,
                    RedactionClass::Internal,
                ))
                .unwrap_or_else(|e| panic!("{e}"));
        }
        ledger
    }

    #[test]
    fn empty_chain_verifies() {
        let ledger = ReceiptLedger::new(clock());
        assert!(ledger.verify_chain().is_ok());
        assert!(ledger.list().is_empty());
    }

    #[test]
    fn chain_verifies_for_a_sequence() {
        let ledger = ledger_with_three();
        assert!(ledger.verify_chain().is_ok());
        assert_eq!(ledger.list().len(), 3);
    }

    #[test]
    fn genesis_prev_hash_is_all_zero() {
        let ledger = ledger_with_three();
        let first = &ledger.list()[0];
        assert_eq!(first.seq, 0);
        assert_eq!(first.prev_hash, genesis_hash());
        assert_eq!(first.prev_hash.as_str(), GENESIS_HASH);
    }

    #[test]
    fn prev_hash_links_to_previous_hash() {
        let ledger = ledger_with_three();
        let list = ledger.list();
        assert_eq!(list[1].prev_hash, list[0].hash);
        assert_eq!(list[2].prev_hash, list[1].hash);
        assert_eq!(list[1].seq, 1);
        assert_eq!(list[2].seq, 2);
    }

    #[test]
    fn seq_is_monotonic_from_zero() {
        let ledger = ledger_with_three();
        let seqs: Vec<u64> = ledger.list().iter().map(|r| r.seq).collect();
        assert_eq!(seqs, vec![0, 1, 2]);
    }

    #[test]
    fn generated_receipt_id_when_absent() {
        let mut ledger = ReceiptLedger::new(clock());
        let mut inp = input(
            "ignored",
            agent("a"),
            ActionKind::Other,
            Outcome::Succeeded,
            RedactionClass::Public,
        );
        inp.receipt_id = None;
        let receipt = ledger.append(inp).unwrap_or_else(|e| panic!("{e}"));
        assert!(!receipt.receipt_id.is_empty());
        // Parses as a UUID.
        assert!(uuid::Uuid::parse_str(&receipt.receipt_id).is_ok());
    }

    #[test]
    fn mutating_a_middle_record_fails_verification() {
        let mut ledger = ledger_with_three();
        // Tamper with the content of the middle record without recomputing hash.
        ledger.receipts[1].after_summary = Some("tampered".to_string());
        match ledger.verify_chain() {
            Err(ChainError::HashMismatch {
                receipt_id, seq, ..
            }) => {
                assert_eq!(receipt_id, "r1");
                assert_eq!(seq, 1);
            }
            other => panic!("expected HashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn mutating_prev_hash_fails_verification() {
        let mut ledger = ledger_with_three();
        // Recompute the hash so the record is internally consistent but its
        // prev_hash no longer links to the prior record.
        ledger.receipts[2].prev_hash = genesis_hash();
        ledger.receipts[2].hash =
            compute_hash(&ledger.receipts[2]).unwrap_or_else(|e| panic!("{e}"));
        assert!(matches!(
            ledger.verify_chain(),
            Err(ChainError::PrevHashMismatch { .. })
        ));
    }

    #[test]
    fn reordering_records_fails_verification() {
        let mut ledger = ledger_with_three();
        ledger.receipts.swap(0, 1);
        assert!(ledger.verify_chain().is_err());
    }

    #[test]
    fn reversal_appends_and_links() {
        let mut ledger = ReceiptLedger::new(clock());
        ledger
            .append(input(
                "pay-1",
                agent("a"),
                ActionKind::Payment,
                Outcome::Succeeded,
                RedactionClass::Sensitive,
            ))
            .unwrap_or_else(|e| panic!("{e}"));

        let reversal_input = input(
            "refund-1",
            agent("a"),
            ActionKind::Refund,
            Outcome::Succeeded,
            RedactionClass::Sensitive,
        );
        let link = ledger
            .record_reversal("pay-1", reversal_input)
            .unwrap_or_else(|e| panic!("{e}"));

        assert_eq!(link.original_receipt_id, "pay-1");
        assert_eq!(link.reversal_receipt_id, "refund-1");

        // The ORIGINAL is marked Reversed, pointing *forward* at the receipt
        // that undid it.
        let original = ledger
            .get("pay-1")
            .unwrap_or_else(|| panic!("original gone"));
        assert_eq!(
            original.reversibility,
            Reversibility::Reversed {
                reversal_receipt_id: "refund-1".to_string()
            }
        );

        // The REVERSAL records which receipt it undoes, pointing *back* at the
        // original via Reversibility::Reverses.
        let reversal = ledger
            .get("refund-1")
            .unwrap_or_else(|| panic!("reversal gone"));
        assert_eq!(
            reversal.reversibility,
            Reversibility::Reverses {
                original_receipt_id: "pay-1".to_string()
            }
        );
        assert_eq!(reversal.seq, 1);
        // The in-place edit of the original re-chained the ledger correctly.
        assert!(ledger.verify_chain().is_ok());
    }

    #[test]
    fn reversal_linkage_points_in_correct_direction() {
        // Three receipts before the reversal so that re-chaining must update
        // the original *and* every receipt appended after it.
        let mut ledger = ledger_with_three();

        let link = ledger
            .record_reversal(
                "r0",
                input(
                    "rev-r0",
                    agent("agent-a"),
                    ActionKind::Refund,
                    Outcome::Succeeded,
                    RedactionClass::Internal,
                ),
            )
            .unwrap_or_else(|e| panic!("{e}"));

        assert_eq!(link.original_receipt_id, "r0");
        assert_eq!(link.reversal_receipt_id, "rev-r0");

        // Original points forward to the reversal receipt's id (NOT its own id).
        let original = ledger.get("r0").unwrap_or_else(|| panic!("original gone"));
        assert_eq!(
            original.reversibility,
            Reversibility::Reversed {
                reversal_receipt_id: "rev-r0".to_string()
            },
            "original must reference the NEW reversal receipt, not itself"
        );

        // Reversal points back to the original via Reverses.
        let reversal = ledger
            .get("rev-r0")
            .unwrap_or_else(|| panic!("reversal gone"));
        assert_eq!(
            reversal.reversibility,
            Reversibility::Reverses {
                original_receipt_id: "r0".to_string()
            }
        );
        assert_eq!(reversal.seq, 3);

        // Hash chain remains intact across the in-place edit + re-chain.
        assert!(ledger.verify_chain().is_ok());
    }

    #[test]
    fn reversal_of_unknown_original_errors() {
        let mut ledger = ReceiptLedger::new(clock());
        let result = ledger.record_reversal(
            "missing",
            input(
                "rev",
                agent("a"),
                ActionKind::Refund,
                Outcome::Succeeded,
                RedactionClass::Public,
            ),
        );
        assert!(matches!(result, Err(ChainError::OriginalNotFound(id)) if id == "missing"));
    }

    #[test]
    fn filter_by_agent() {
        let mut ledger = ReceiptLedger::new(clock());
        ledger
            .append(input(
                "a0",
                agent("alpha"),
                ActionKind::Message,
                Outcome::Succeeded,
                RedactionClass::Public,
            ))
            .unwrap_or_else(|e| panic!("{e}"));
        ledger
            .append(input(
                "b0",
                agent("beta"),
                ActionKind::Message,
                Outcome::Succeeded,
                RedactionClass::Public,
            ))
            .unwrap_or_else(|e| panic!("{e}"));

        let alpha = ledger.filter(&ReceiptFilter::by_agent(agent("alpha")));
        assert_eq!(alpha.len(), 1);
        assert_eq!(alpha[0].receipt_id, "a0");
    }

    #[test]
    fn filter_by_outcome_and_action_kind() {
        let ledger = ledger_with_three();
        let pending = ledger.filter(&ReceiptFilter::by_outcome(Outcome::Pending));
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].receipt_id, "r1");

        let deploys = ledger.filter(&ReceiptFilter::by_action_kind(ActionKind::Deploy));
        assert_eq!(deploys.len(), 1);
        assert_eq!(deploys[0].receipt_id, "r2");

        // Combined criteria.
        let combined = ledger.filter(&ReceiptFilter {
            agent_id: Some(agent("agent-a")),
            outcome: Some(Outcome::Failed),
            action_kind: Some(ActionKind::Deploy),
        });
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].receipt_id, "r2");
    }

    #[test]
    fn risk_class_is_preserved() {
        let mut ledger = ReceiptLedger::new(clock());
        ledger
            .append(input(
                "secret-op",
                agent("a"),
                ActionKind::DbWrite,
                Outcome::Succeeded,
                RedactionClass::Secret,
            ))
            .unwrap_or_else(|e| panic!("{e}"));
        let receipt = ledger.get("secret-op").unwrap_or_else(|| panic!("missing"));
        assert_eq!(receipt.risk_class, RedactionClass::Secret);
        // Survives a serde round-trip.
        let json = serde_json::to_string(receipt).unwrap_or_else(|e| panic!("{e}"));
        let back: AgentActionReceipt =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back.risk_class, RedactionClass::Secret);
        assert_eq!(&back, receipt);
    }

    #[test]
    fn summaries_round_trip_and_are_redacted_placeholders() {
        let ledger = ledger_with_three();
        let first = &ledger.list()[0];
        assert_eq!(first.before_summary.as_deref(), Some("[redacted before]"));
        assert_eq!(first.after_summary.as_deref(), Some("[redacted after]"));
    }
}
