//! Content-addressed corpus identity — a deterministic Merkle root over a set
//! of content-addressed leaves.
//!
//! A corpus (a dataset version, a knowledge pack, a set of trace spans) is a
//! *set* of items that each already have a stable content hash. This module
//! folds that set into a single [`CorpusRoot`]: a short hash that names the
//! exact contents, so a trace, eval, replay, or customer-facing report can cite
//! the precise snapshot it ran against and two parties holding the same items
//! can confirm it with one comparison.
//!
//! Three properties make the root trustworthy:
//!
//! * **History-independent.** The root is a function of the leaf *set*, not of
//!   insertion or storage order — leaves are sorted by key before hashing, so
//!   two replicas that built the same corpus by different paths agree. This is
//!   the defining property of the prolly-tree / Merkle-search-tree family
//!   (issue #322); we get it here from the sort rather than from
//!   content-defined chunking.
//! * **Domain-separated.** Leaf and interior nodes hash under distinct prefixes
//!   and keys are length-prefixed, so a leaf digest can never be confused with
//!   an interior node and `(key, content_hash)` boundaries are unambiguous
//!   (second-preimage hardening).
//! * **Avalanche.** Changing any single leaf's key or content changes the root.
//!
//! ## Why a sorted binary Merkle tree and not a full prolly/CDC tree
//!
//! The cited SOTA (issues #317, #318, #322) layers content-defined chunking
//! (FastCDC) and persistent prolly-tree nodes to get *sublinear* diff and
//! structural sharing across many historical versions over a network. Those pay
//! off only with a consumer that diffs huge roots without holding both leaf
//! sets, or that stores thousands of versions with shared subtrees, or that
//! reconciles set ranges peer-to-peer. Beater has none of those consumers
//! today: its corpora are bounded sets of records that already carry content
//! hashes, and the first consumer simply needs to *name* a snapshot. So we
//! build the honest minimal structure — a real, history-independent Merkle root
//! — and leave the CDC chunker and persistent prolly nodes to land with the
//! sync/large-N consumer that actually needs them. The leaf abstraction here is
//! exactly what that future tree would consume, so this is a foundation, not a
//! throwaway.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::lower_hex;

/// Domain tag mixed into leaf digests so a leaf can never collide with a node.
const LEAF_DOMAIN: &[u8] = b"beater.merkle.leaf.v1";
/// Domain tag mixed into interior-node digests.
const NODE_DOMAIN: &[u8] = b"beater.merkle.node.v1";
/// Digest preimage for the empty corpus (a corpus with no leaves).
const EMPTY_DOMAIN: &[u8] = b"beater.merkle.empty.v1";

/// A content-addressed Merkle root naming the exact contents of a corpus.
///
/// Serialized as its lowercase-hex SHA-256 string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CorpusRoot(String);

impl CorpusRoot {
    /// The hex digest as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume into the owned hex string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for CorpusRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One entry in a corpus: a stable `key` (its identity within the corpus) and
/// the `content_hash` of its contents (any stable hex digest the caller trusts
/// to change whenever the contents change).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleLeaf {
    /// Identity of the item within the corpus; must be unique.
    pub key: String,
    /// Stable content digest of the item.
    pub content_hash: String,
}

impl MerkleLeaf {
    /// Convenience constructor.
    pub fn new(key: impl Into<String>, content_hash: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            content_hash: content_hash.into(),
        }
    }
}

/// Errors that make a corpus root undefined.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MerkleError {
    /// Two leaves shared a key, so the corpus is not a well-formed set.
    #[error("duplicate corpus leaf key: {0}")]
    DuplicateKey(String),
}

/// Fold a set of content-addressed leaves into a single [`CorpusRoot`].
///
/// The result is independent of the iteration order of `leaves` and changes
/// whenever any leaf's key or content hash changes. Returns
/// [`MerkleError::DuplicateKey`] if two leaves share a key (the input is then
/// not a set and the root would be ambiguous).
pub fn corpus_root<I>(leaves: I) -> Result<CorpusRoot, MerkleError>
where
    I: IntoIterator<Item = MerkleLeaf>,
{
    let mut leaves: Vec<MerkleLeaf> = leaves.into_iter().collect();
    leaves.sort_by(|a, b| a.key.cmp(&b.key));
    for adjacent in leaves.windows(2) {
        if adjacent[0].key == adjacent[1].key {
            return Err(MerkleError::DuplicateKey(adjacent[0].key.clone()));
        }
    }

    if leaves.is_empty() {
        return Ok(CorpusRoot(lower_hex(&Sha256::digest(EMPTY_DOMAIN))));
    }

    let mut level: Vec<[u8; 32]> = leaves.iter().map(leaf_digest).collect();
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| {
                if pair.len() == 2 {
                    node_digest(&pair[0], &pair[1])
                } else {
                    // Odd node out: promote it unchanged (as Certificate
                    // Transparency does). Promotion never duplicates a node, so
                    // it avoids the duplicate-leaf forgery of naive schemes.
                    pair[0]
                }
            })
            .collect();
    }

    Ok(CorpusRoot(lower_hex(&level[0])))
}

fn leaf_digest(leaf: &MerkleLeaf) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(LEAF_DOMAIN);
    // Length-prefix the key so the key/content_hash boundary is unambiguous.
    hasher.update((leaf.key.len() as u64).to_le_bytes());
    hasher.update(leaf.key.as_bytes());
    hasher.update(leaf.content_hash.as_bytes());
    hasher.finalize().into()
}

fn node_digest(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(NODE_DOMAIN);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves(pairs: &[(&str, &str)]) -> Vec<MerkleLeaf> {
        pairs.iter().map(|(k, h)| MerkleLeaf::new(*k, *h)).collect()
    }

    fn root(pairs: &[(&str, &str)]) -> CorpusRoot {
        corpus_root(leaves(pairs)).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn root_is_deterministic() {
        let a = root(&[("k1", "h1"), ("k2", "h2"), ("k3", "h3")]);
        let b = root(&[("k1", "h1"), ("k2", "h2"), ("k3", "h3")]);
        assert_eq!(a, b);
    }

    #[test]
    fn root_is_independent_of_input_order() {
        let forward = root(&[("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]);
        let shuffled = root(&[("c", "3"), ("a", "1"), ("d", "4"), ("b", "2")]);
        assert_eq!(forward, shuffled, "history-independence broken");
    }

    #[test]
    fn changing_content_changes_root() {
        let before = root(&[("a", "1"), ("b", "2")]);
        let after = root(&[("a", "1"), ("b", "2-changed")]);
        assert_ne!(before, after);
    }

    #[test]
    fn changing_key_changes_root() {
        let before = root(&[("a", "1"), ("b", "2")]);
        let after = root(&[("a", "1"), ("b-renamed", "2")]);
        assert_ne!(before, after);
    }

    #[test]
    fn adding_or_removing_a_leaf_changes_root() {
        let two = root(&[("a", "1"), ("b", "2")]);
        let three = root(&[("a", "1"), ("b", "2"), ("c", "3")]);
        assert_ne!(two, three);
    }

    #[test]
    fn empty_corpus_has_a_stable_distinct_root() {
        let empty_a = corpus_root(Vec::new()).unwrap_or_else(|err| panic!("{err}"));
        let empty_b = corpus_root(std::iter::empty()).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(empty_a, empty_b);
        assert_ne!(empty_a, root(&[("a", "1")]));
    }

    #[test]
    fn single_leaf_root_is_domain_separated_from_its_content() {
        // The root of a one-leaf corpus must not equal the raw content hash,
        // otherwise a leaf could be passed off as a root.
        let single = root(&[("a", "deadbeef")]);
        assert_ne!(single.as_str(), "deadbeef");
    }

    #[test]
    fn key_content_boundary_is_unambiguous() {
        // Without length-prefixing, ("ab","c") and ("a","bc") would hash the
        // same leaf preimage and collide.
        let left = root(&[("ab", "c")]);
        let right = root(&[("a", "bc")]);
        assert_ne!(left, right);
    }

    #[test]
    fn odd_leaf_count_is_handled() {
        // Three and five leaves exercise the odd-node promotion path.
        let three = root(&[("a", "1"), ("b", "2"), ("c", "3")]);
        let five = root(&[("a", "1"), ("b", "2"), ("c", "3"), ("d", "4"), ("e", "5")]);
        assert_ne!(three, five);
    }

    #[test]
    fn duplicate_key_is_rejected() {
        let err = corpus_root(leaves(&[("dup", "1"), ("dup", "2")]))
            .err()
            .unwrap_or_else(|| panic!("expected duplicate-key error"));
        assert_eq!(err, MerkleError::DuplicateKey("dup".to_string()));
    }

    #[test]
    fn root_is_lowercase_hex_sha256() {
        let r = root(&[("a", "1")]);
        assert_eq!(r.as_str().len(), 64);
        assert!(r
            .as_str()
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
