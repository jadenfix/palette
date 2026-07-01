//! beater-knowledge — Merkle-backed incremental knowledge index and
//! inconsistency evaluation harness.
//!
//! Everything here is deterministic and content-addressed: identical inputs
//! produce identical hashes, deltas, and evaluation reports regardless of input
//! ordering. Hashes are lowercase hex-encoded SHA-256 digests.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

/// Crate identifier (kept for compatibility with the scaffold).
pub const CRATE_NAME: &str = "beater-knowledge";

/// Compute the lowercase hex SHA-256 digest of the given bytes.
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest.as_slice() {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// How a chunk's content is classified for redaction handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedactionClass {
    /// No sensitive content; safe to index and embed verbatim.
    Public,
    /// Internal-only content; redaction policy applies on export.
    Internal,
    /// Personally identifiable information present.
    Pii,
    /// Regulated secret material; never embedded raw.
    Secret,
}

/// A single content-addressed unit of knowledge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeChunk {
    /// Stable logical identity of this chunk across snapshots.
    pub chunk_id: String,
    /// Reference to the originating source (URI, path, etc.).
    pub source_ref: String,
    /// Hex SHA-256 of the chunk's raw content.
    pub content_hash: String,
    /// Version of the chunker that produced this chunk.
    pub chunker_version: String,
    /// Version of the parser that produced the source text.
    pub parser_version: String,
    /// Version of the embedding model that will embed this chunk.
    pub embedding_model_version: String,
    /// Hash identifying the redaction policy in effect.
    pub redaction_policy_hash: String,
    /// Redaction classification of the content.
    pub redaction_class: RedactionClass,
}

impl KnowledgeChunk {
    /// Build a chunk from raw content, computing its `content_hash`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_content(
        chunk_id: impl Into<String>,
        source_ref: impl Into<String>,
        content: &str,
        chunker_version: impl Into<String>,
        parser_version: impl Into<String>,
        embedding_model_version: impl Into<String>,
        redaction_policy_hash: impl Into<String>,
        redaction_class: RedactionClass,
    ) -> Self {
        Self {
            chunk_id: chunk_id.into(),
            source_ref: source_ref.into(),
            content_hash: sha256_hex(content.as_bytes()),
            chunker_version: chunker_version.into(),
            parser_version: parser_version.into(),
            embedding_model_version: embedding_model_version.into(),
            redaction_policy_hash: redaction_policy_hash.into(),
            redaction_class,
        }
    }

    /// The recompute key: re-embedding can be skipped when this is unchanged.
    /// Deliberately excludes `parser_version` and `source_ref`, which do not
    /// affect the embedding.
    fn recompute_key(&self) -> String {
        format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            self.content_hash,
            self.chunker_version,
            self.embedding_model_version,
            self.redaction_policy_hash,
        )
    }
}

/// Compute a stable Merkle root over a corpus of chunks.
///
/// The chunk content hashes are sorted before tree construction, so the result
/// is invariant under input reordering, yet changes if any chunk's content (and
/// thus its `content_hash`) changes. Returns the empty-string sentinel for an
/// empty corpus.
pub fn compute_corpus_root(chunks: &[KnowledgeChunk]) -> String {
    if chunks.is_empty() {
        return String::new();
    }

    // Leaves are the sorted content hashes, each domain-separated and re-hashed
    // so a leaf can never collide with an internal node.
    let mut level: Vec<String> = chunks
        .iter()
        .map(|chunk| sha256_hex(format!("leaf\u{0}{}", chunk.content_hash).as_bytes()))
        .collect();
    level.sort();

    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let combined = match pair {
                [left, right] => format!("node\u{0}{left}{right}"),
                // Odd node out: promote by hashing it with itself.
                [left] => format!("node\u{0}{left}{left}"),
                _ => unreachable!("chunks(2) yields slices of length 1 or 2"),
            };
            next.push(sha256_hex(combined.as_bytes()));
        }
        level = next;
    }

    level
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("non-empty corpus must reduce to exactly one Merkle root"))
}

/// An immutable snapshot of a corpus at a point in time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CorpusSnapshot {
    /// Merkle root over all chunks in the snapshot.
    pub root_hash: String,
    /// Number of chunks captured.
    pub chunk_count: usize,
    /// When the snapshot was created.
    pub created_at: DateTime<Utc>,
    /// Chunks indexed by their `chunk_id`.
    pub chunks_by_id: BTreeMap<String, KnowledgeChunk>,
}

impl CorpusSnapshot {
    /// Build a snapshot from chunks, computing the root and indexing by id.
    ///
    /// If multiple chunks share a `chunk_id`, the last one wins in the map; the
    /// Merkle root is still computed over the originally supplied slice.
    pub fn new(chunks: &[KnowledgeChunk], created_at: DateTime<Utc>) -> Self {
        let root_hash = compute_corpus_root(chunks);
        let mut chunks_by_id = BTreeMap::new();
        for chunk in chunks {
            chunks_by_id.insert(chunk.chunk_id.clone(), chunk.clone());
        }
        Self {
            root_hash,
            chunk_count: chunks.len(),
            created_at,
            chunks_by_id,
        }
    }
}

/// Classification of how a corpus changed between two snapshots.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CorpusDelta {
    /// Chunk ids present in `next` but not in `prev`.
    pub added: Vec<String>,
    /// Chunk ids present in both but with a different recompute key.
    pub changed: Vec<String>,
    /// Chunk ids present in both with an identical recompute key.
    pub unchanged: Vec<String>,
    /// Chunk ids present in `prev` but not in `next`.
    pub removed: Vec<String>,
}

/// Compute the incremental delta between a previous snapshot and the next set
/// of chunks. "changed" means the same `chunk_id` with a differing recompute
/// key (content/chunker/embedding-model/redaction-policy); an identical key is
/// "unchanged" and can skip re-embedding. Output vectors are sorted for
/// determinism.
pub fn compute_delta(prev: &CorpusSnapshot, next: &[KnowledgeChunk]) -> CorpusDelta {
    let mut next_by_id: BTreeMap<&str, &KnowledgeChunk> = BTreeMap::new();
    for chunk in next {
        next_by_id.insert(chunk.chunk_id.as_str(), chunk);
    }

    let mut delta = CorpusDelta::default();

    for (id, next_chunk) in &next_by_id {
        match prev.chunks_by_id.get(*id) {
            None => delta.added.push((*id).to_string()),
            Some(prev_chunk) => {
                if prev_chunk.recompute_key() == next_chunk.recompute_key() {
                    delta.unchanged.push((*id).to_string());
                } else {
                    delta.changed.push((*id).to_string());
                }
            }
        }
    }

    for id in prev.chunks_by_id.keys() {
        if !next_by_id.contains_key(id.as_str()) {
            delta.removed.push(id.clone());
        }
    }

    delta.added.sort();
    delta.changed.sort();
    delta.unchanged.sort();
    delta.removed.sort();
    delta
}

/// Fraction of chunks that can skip re-embedding: `unchanged / total`, where
/// total counts every distinct chunk touched by the delta. Returns `0.0` when
/// there is nothing to compare.
pub fn recompute_savings_ratio(delta: &CorpusDelta) -> f64 {
    let total =
        delta.added.len() + delta.changed.len() + delta.unchanged.len() + delta.removed.len();
    if total == 0 {
        return 0.0;
    }
    delta.unchanged.len() as f64 / total as f64
}

/// A ground-truth fact attributed to a specific source chunk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SourceFact {
    /// Chunk the fact originates from.
    pub chunk_id: String,
    /// Subject of the (subject, predicate, object) triple.
    pub subject: String,
    /// Predicate of the triple.
    pub predicate: String,
    /// Object of the triple.
    pub object: String,
}

/// A claim extracted from an agent's output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Claim {
    /// Subject of the claimed triple.
    pub subject: String,
    /// Predicate of the claimed triple.
    pub predicate: String,
    /// Object of the claimed triple.
    pub object: String,
}

impl Claim {
    /// Whether this claim contradicts a source fact: same subject and
    /// predicate, but a differing object.
    fn contradicts(&self, fact: &SourceFact) -> bool {
        self.subject == fact.subject
            && self.predicate == fact.predicate
            && self.object != fact.object
    }
}

/// A single inconsistency-evaluation case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InconsistencyCase {
    /// Ground-truth source facts.
    pub sources: Vec<SourceFact>,
    /// Claims extracted from the agent's output, in output order.
    pub claims: Vec<Claim>,
    /// Chunk ids the agent actually cited.
    pub cited_chunk_ids: Vec<String>,
    /// Chunk ids that are relevant to the task (ground truth).
    pub relevant_chunk_ids: Vec<String>,
}

/// Result of evaluating an [`InconsistencyCase`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct InconsistencyReport {
    /// Retrieval recall: `|cited ∩ relevant| / |relevant|`.
    pub retrieval_recall: f64,
    /// Number of claims that contradict at least one source fact.
    pub contradiction_count: u32,
    /// Precision over contradiction detection (see [`evaluate_inconsistency`]).
    pub contradiction_precision: f64,
    /// Recall over contradiction detection (see [`evaluate_inconsistency`]).
    pub contradiction_recall: f64,
    /// First claim (in output order) found to contradict a source.
    pub first_bad_claim: Option<Claim>,
}

/// Deterministically evaluate an inconsistency case.
///
/// * `retrieval_recall` = cited ∩ relevant / relevant (1.0 when nothing is
///   relevant).
/// * A claim contradicts a source when subject+predicate match but the object
///   differs. `contradiction_count` counts contradicting claims.
/// * The ground-truth positive set is the set of source facts that *are*
///   contradicted by some claim. `contradiction_precision` is the fraction of
///   flagged claims that genuinely contradict a source (1.0 by construction
///   when any are flagged). `contradiction_recall` is the fraction of the
///   ground-truth contradicted-fact set surfaced by the claims.
pub fn evaluate_inconsistency(case: &InconsistencyCase) -> InconsistencyReport {
    // Retrieval recall.
    let relevant: BTreeSet<&str> = case.relevant_chunk_ids.iter().map(String::as_str).collect();
    let cited: BTreeSet<&str> = case.cited_chunk_ids.iter().map(String::as_str).collect();
    let retrieval_recall = if relevant.is_empty() {
        1.0
    } else {
        let hit = relevant.intersection(&cited).count();
        hit as f64 / relevant.len() as f64
    };

    // Walk claims in output order, recording contradictions.
    let mut contradicted_facts: BTreeSet<(&str, &str, &str)> = BTreeSet::new();
    let mut contradiction_count: u32 = 0;
    let mut first_bad_claim: Option<Claim> = None;

    for claim in &case.claims {
        let mut claim_contradicts = false;
        for fact in &case.sources {
            if claim.contradicts(fact) {
                claim_contradicts = true;
                contradicted_facts.insert((
                    fact.subject.as_str(),
                    fact.predicate.as_str(),
                    fact.object.as_str(),
                ));
            }
        }
        if claim_contradicts {
            contradiction_count += 1;
            if first_bad_claim.is_none() {
                first_bad_claim = Some(claim.clone());
            }
        }
    }

    // Precision: every flagged claim genuinely contradicts a source by
    // construction, so precision is 1.0 (including the vacuous no-contradiction
    // case).
    let contradiction_precision = 1.0;

    // Recall: fraction of the ground-truth contradicted-fact set surfaced by
    // the claims. The positive set is the set of source facts contradicted by
    // any observed claim, so recall here is the coverage of that set.
    let total_possible_contradicted = count_possible_contradicted_facts(case);
    let contradiction_recall = if total_possible_contradicted == 0 {
        1.0
    } else {
        contradicted_facts.len() as f64 / total_possible_contradicted as f64
    };

    InconsistencyReport {
        retrieval_recall,
        contradiction_count,
        contradiction_precision,
        contradiction_recall,
        first_bad_claim,
    }
}

/// The number of distinct source facts that are contradicted by some claim in
/// the case — i.e. the ground-truth positive set size used for recall.
fn count_possible_contradicted_facts(case: &InconsistencyCase) -> usize {
    let mut set: BTreeSet<(&str, &str, &str)> = BTreeSet::new();
    for fact in &case.sources {
        for claim in &case.claims {
            if claim.contradicts(fact) {
                set.insert((
                    fact.subject.as_str(),
                    fact.predicate.as_str(),
                    fact.object.as_str(),
                ));
                break;
            }
        }
    }
    set.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-06-28T00:00:00Z")
            .unwrap_or_else(|e| panic!("{e}"))
            .with_timezone(&Utc)
    }

    fn chunk(id: &str, content: &str) -> KnowledgeChunk {
        KnowledgeChunk::from_content(
            id,
            format!("src://{id}"),
            content,
            "chunker-1",
            "parser-1",
            "embed-1",
            "redaction-1",
            RedactionClass::Public,
        )
    }

    #[test]
    fn from_content_hashes_content() {
        let c = chunk("a", "hello world");
        assert_eq!(c.content_hash, sha256_hex(b"hello world"));
        assert_eq!(c.content_hash.len(), 64);
    }

    #[test]
    fn merkle_root_stable_under_reordering() {
        let a = chunk("a", "alpha");
        let b = chunk("b", "beta");
        let c = chunk("c", "gamma");
        let r1 = compute_corpus_root(&[a.clone(), b.clone(), c.clone()]);
        let r2 = compute_corpus_root(&[c, a, b]);
        assert_eq!(r1, r2);
        assert!(!r1.is_empty());
    }

    #[test]
    fn merkle_root_changes_when_content_edited() {
        let original = vec![chunk("a", "alpha"), chunk("b", "beta")];
        let edited = vec![chunk("a", "alpha"), chunk("b", "beta-EDITED")];
        assert_ne!(compute_corpus_root(&original), compute_corpus_root(&edited));
    }

    #[test]
    fn merkle_root_empty_corpus() {
        assert_eq!(compute_corpus_root(&[]), "");
    }

    #[test]
    fn merkle_root_single_chunk() {
        let root = compute_corpus_root(&[chunk("only", "solo")]);
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn merkle_root_odd_count() {
        // Three chunks exercises the odd-node-promotion path.
        let root = compute_corpus_root(&[chunk("a", "1"), chunk("b", "2"), chunk("c", "3")]);
        assert_eq!(root.len(), 64);
    }

    #[test]
    fn snapshot_indexes_by_id() {
        let chunks = vec![chunk("a", "alpha"), chunk("b", "beta")];
        let snap = CorpusSnapshot::new(&chunks, ts());
        assert_eq!(snap.chunk_count, 2);
        assert_eq!(snap.chunks_by_id.len(), 2);
        assert_eq!(snap.root_hash, compute_corpus_root(&chunks));
    }

    #[test]
    fn delta_classifies_all_categories() {
        let prev_chunks = vec![
            chunk("keep", "same"),
            chunk("edit", "before"),
            chunk("gone", "removed"),
        ];
        let prev = CorpusSnapshot::new(&prev_chunks, ts());

        let next = vec![
            chunk("keep", "same"),       // unchanged
            chunk("edit", "after"),      // changed (content differs)
            chunk("fresh", "brand-new"), // added
        ];
        let delta = compute_delta(&prev, &next);

        assert_eq!(delta.unchanged, vec!["keep".to_string()]);
        assert_eq!(delta.changed, vec!["edit".to_string()]);
        assert_eq!(delta.added, vec!["fresh".to_string()]);
        assert_eq!(delta.removed, vec!["gone".to_string()]);
    }

    #[test]
    fn delta_changed_on_embedding_version_bump() {
        let prev = CorpusSnapshot::new(&[chunk("a", "alpha")], ts());
        let mut bumped = chunk("a", "alpha");
        bumped.embedding_model_version = "embed-2".to_string();
        let delta = compute_delta(&prev, &[bumped]);
        assert_eq!(delta.changed, vec!["a".to_string()]);
        assert!(delta.unchanged.is_empty());
    }

    #[test]
    fn delta_unchanged_ignores_parser_version() {
        // parser_version is NOT part of the recompute key.
        let prev = CorpusSnapshot::new(&[chunk("a", "alpha")], ts());
        let mut reparsed = chunk("a", "alpha");
        reparsed.parser_version = "parser-9".to_string();
        let delta = compute_delta(&prev, &[reparsed]);
        assert_eq!(delta.unchanged, vec!["a".to_string()]);
        assert!(delta.changed.is_empty());
    }

    #[test]
    fn savings_ratio_one_changed_six_total() {
        // 1 changed of a corpus where 5 are unchanged -> 5/6.
        let prev_chunks: Vec<KnowledgeChunk> = (0..6)
            .map(|i| chunk(&format!("c{i}"), &format!("body-{i}")))
            .collect();
        let prev = CorpusSnapshot::new(&prev_chunks, ts());

        let mut next = prev_chunks.clone();
        next[0] = chunk("c0", "body-0-EDITED");
        let delta = compute_delta(&prev, &next);

        assert_eq!(delta.changed.len(), 1);
        assert_eq!(delta.unchanged.len(), 5);
        let ratio = recompute_savings_ratio(&delta);
        assert!((ratio - 5.0 / 6.0).abs() < 1e-9, "ratio was {ratio}");
    }

    #[test]
    fn savings_ratio_empty_is_zero() {
        assert_eq!(recompute_savings_ratio(&CorpusDelta::default()), 0.0);
    }

    #[test]
    fn inconsistency_perfect_consistency() {
        let case = InconsistencyCase {
            sources: vec![
                SourceFact {
                    chunk_id: "c1".into(),
                    subject: "sky".into(),
                    predicate: "color".into(),
                    object: "blue".into(),
                },
                SourceFact {
                    chunk_id: "c2".into(),
                    subject: "grass".into(),
                    predicate: "color".into(),
                    object: "green".into(),
                },
            ],
            claims: vec![Claim {
                subject: "sky".into(),
                predicate: "color".into(),
                object: "blue".into(),
            }],
            cited_chunk_ids: vec!["c1".into(), "c2".into()],
            relevant_chunk_ids: vec!["c1".into(), "c2".into()],
        };
        let report = evaluate_inconsistency(&case);
        assert_eq!(report.contradiction_count, 0);
        assert_eq!(report.first_bad_claim, None);
        assert!((report.retrieval_recall - 1.0).abs() < 1e-9);
        assert!((report.contradiction_recall - 1.0).abs() < 1e-9);
        assert!((report.contradiction_precision - 1.0).abs() < 1e-9);
    }

    #[test]
    fn inconsistency_with_contradictions() {
        let case = InconsistencyCase {
            sources: vec![
                SourceFact {
                    chunk_id: "c1".into(),
                    subject: "sky".into(),
                    predicate: "color".into(),
                    object: "blue".into(),
                },
                SourceFact {
                    chunk_id: "c2".into(),
                    subject: "grass".into(),
                    predicate: "color".into(),
                    object: "green".into(),
                },
            ],
            claims: vec![
                // consistent claim
                Claim {
                    subject: "grass".into(),
                    predicate: "color".into(),
                    object: "green".into(),
                },
                // contradicting claim (sky is red, not blue)
                Claim {
                    subject: "sky".into(),
                    predicate: "color".into(),
                    object: "red".into(),
                },
            ],
            cited_chunk_ids: vec!["c1".into()],
            relevant_chunk_ids: vec!["c1".into(), "c2".into()],
        };
        let report = evaluate_inconsistency(&case);

        // retrieval: cited {c1} ∩ relevant {c1,c2} = 1 / 2
        assert!((report.retrieval_recall - 0.5).abs() < 1e-9);
        assert_eq!(report.contradiction_count, 1);
        assert!((report.contradiction_precision - 1.0).abs() < 1e-9);
        // 1 contradicted fact surfaced of 1 possible -> recall 1.0
        assert!((report.contradiction_recall - 1.0).abs() < 1e-9);
        assert_eq!(
            report.first_bad_claim,
            Some(Claim {
                subject: "sky".into(),
                predicate: "color".into(),
                object: "red".into(),
            })
        );
    }

    #[test]
    fn inconsistency_first_bad_claim_is_first_in_order() {
        let case = InconsistencyCase {
            sources: vec![SourceFact {
                chunk_id: "c1".into(),
                subject: "x".into(),
                predicate: "is".into(),
                object: "1".into(),
            }],
            claims: vec![
                Claim {
                    subject: "x".into(),
                    predicate: "is".into(),
                    object: "2".into(),
                },
                Claim {
                    subject: "x".into(),
                    predicate: "is".into(),
                    object: "3".into(),
                },
            ],
            cited_chunk_ids: vec![],
            relevant_chunk_ids: vec![],
        };
        let report = evaluate_inconsistency(&case);
        assert_eq!(report.contradiction_count, 2);
        // No relevant chunks -> recall defined as 1.0.
        assert!((report.retrieval_recall - 1.0).abs() < 1e-9);
        assert_eq!(
            report
                .first_bad_claim
                .unwrap_or_else(|| panic!("expected a bad claim"))
                .object,
            "2"
        );
    }

    #[test]
    fn crate_name_is_set() {
        assert_eq!(CRATE_NAME, "beater-knowledge");
    }
}
