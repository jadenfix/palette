//! Deterministic train/val/test splitting — the data substrate for
//! non-overfitting recursive self-improvement (RSI).
//!
//! The RSI loop (`beater-experiments`) only *proposes* changes; a candidate is
//! accepted only if it beats baseline on a **held-out Test set** with
//! statistical confidence. That guarantee is meaningless unless the held-out set
//! is genuinely held out and cannot be contaminated by the data the optimizer
//! trained/selected on.
//!
//! ## The split is keyed on input *content*, not on `case_id`
//!
//! A case's split is a pure function of `(sha256(input), seed)`. Two consequences
//! that make the held-out gate trustworthy:
//!
//! 1. **Stable under re-versioning.** Adding/removing cases, or rebuilding a
//!    dataset version, never reshuffles an existing case's split.
//! 2. **Leak-proof by construction for identical inputs.** Any two cases with the
//!    same input land in the *same* split — even if they carry different
//!    `case_id`s or are re-promoted in a later version. The classic
//!    "same example in train and test" contamination is therefore *impossible*
//!    here, not merely detected after the fact.
//!
//! The residual risk is *near-duplicate* inputs (byte-different but semantically
//! equal). That requires n-gram/embedding overlap and is handled at
//! dataset-generation time (the RSI simulation harness); [`duplicate_inputs`]
//! reports exact duplicates as a data-quality signal in the meantime.

use crate::DatasetCase;
use beater_core::{sha256_json_hash, DatasetCaseId, JsonHashError, Sha256Hash};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Which partition of a dataset a case belongs to.
///
/// `Train` is what an optimizer may rewrite/learn from, `Val` is what it may
/// query to *select* among candidates (under a reusable-holdout budget, see
/// `beater-stats`), and `Test` is the held-out gate set — ideally queried once,
/// at acceptance time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitLabel {
    /// Optimization set — the optimizer may read and fit to these.
    Train,
    /// Selection set — used to pick among candidates; query budget applies.
    Val,
    /// Held-out gate set — the acceptance decision is made here.
    Test,
}

impl SplitLabel {
    /// Snake-case name, matching the serde representation.
    pub fn name(self) -> &'static str {
        match self {
            SplitLabel::Train => "train",
            SplitLabel::Val => "val",
            SplitLabel::Test => "test",
        }
    }
}

/// Errors constructing a [`SplitConfig`].
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum SplitError {
    /// A fraction was negative or non-finite.
    #[error("split fraction must be finite and non-negative, got {0}")]
    InvalidFraction(String),
    /// The three fractions did not sum to ~1.0.
    #[error("split fractions must sum to 1.0 (±1e-9), got {0}")]
    FractionsDoNotSumToOne(f64),
}

/// Target proportions for a deterministic split, plus the seed that anchors the
/// hash bucketing.
///
/// Fractions are *targets*: actual counts converge to these as the number of
/// distinct inputs grows (hash bucketing is unbiased but not exact on small N).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitConfig {
    /// Fraction assigned to [`SplitLabel::Train`].
    pub train: f64,
    /// Fraction assigned to [`SplitLabel::Val`].
    pub val: f64,
    /// Fraction assigned to [`SplitLabel::Test`].
    pub test: f64,
    /// Seed mixed into the hash so the same dataset can be re-split independently.
    pub seed: u64,
}

impl SplitConfig {
    /// Validate and construct. Fractions must be finite, non-negative, and sum to
    /// 1.0 within `1e-9`.
    pub fn new(train: f64, val: f64, test: f64, seed: u64) -> Result<Self, SplitError> {
        for f in [train, val, test] {
            if !f.is_finite() || f < 0.0 {
                return Err(SplitError::InvalidFraction(f.to_string()));
            }
        }
        let sum = train + val + test;
        if (sum - 1.0).abs() > 1e-9 {
            return Err(SplitError::FractionsDoNotSumToOne(sum));
        }
        Ok(Self {
            train,
            val,
            test,
            seed,
        })
    }

    /// A conventional 70/15/15 split. Constructed directly since the constants
    /// are statically valid.
    pub fn standard(seed: u64) -> Self {
        Self {
            train: 0.70,
            val: 0.15,
            test: 0.15,
            seed,
        }
    }
}

/// Canonical content fingerprint of a case input — the split key and the
/// duplicate-detection key. Thin wrapper over [`sha256_json_hash`] so callers
/// (e.g. the anti-overfitting gate in PR2) fingerprint inputs the same way.
pub fn fingerprint_input(input: &Value) -> Result<Sha256Hash, JsonHashError> {
    sha256_json_hash(input)
}

/// Map a content fingerprint to a stable fraction in `[0, 1)`.
///
/// The sha256 hex is already uniform; FNV-1a over `seed:fingerprint` followed by
/// a splitmix64 finalizer keeps it uniform while mixing in the seed. Pure and
/// dependency-free so the assignment is reproducible across machines and
/// crate versions.
fn fingerprint_fraction(fingerprint: &Sha256Hash, seed: u64) -> f64 {
    // FNV-1a (64-bit).
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in seed.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash ^= u64::from(b':');
    hash = hash.wrapping_mul(PRIME);
    for byte in fingerprint.as_str().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    // splitmix64 finalizer for full 64-bit avalanche before extracting the fraction.
    let mut z = hash;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    // Top 53 bits → uniform f64 in [0, 1).
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Assign a split given an already-computed content fingerprint.
pub fn split_for_fingerprint(fingerprint: &Sha256Hash, config: &SplitConfig) -> SplitLabel {
    let f = fingerprint_fraction(fingerprint, config.seed);
    if f < config.train {
        SplitLabel::Train
    } else if f < config.train + config.val {
        SplitLabel::Val
    } else {
        SplitLabel::Test
    }
}

/// Assign a case to a split from its input content. Identical inputs always map
/// to the same split — the leak-proof property.
pub fn split_for_input(input: &Value, config: &SplitConfig) -> Result<SplitLabel, JsonHashError> {
    let fingerprint = fingerprint_input(input)?;
    Ok(split_for_fingerprint(&fingerprint, config))
}

/// A dataset partitioned into the three splits, preserving input order within
/// each split.
#[derive(Clone, Debug, Default)]
pub struct SplitPartition {
    /// Cases assigned to [`SplitLabel::Train`].
    pub train: Vec<DatasetCase>,
    /// Cases assigned to [`SplitLabel::Val`].
    pub val: Vec<DatasetCase>,
    /// Cases assigned to [`SplitLabel::Test`].
    pub test: Vec<DatasetCase>,
}

impl SplitPartition {
    /// Borrow the cases for a given split.
    pub fn get(&self, split: SplitLabel) -> &[DatasetCase] {
        match split {
            SplitLabel::Train => &self.train,
            SplitLabel::Val => &self.val,
            SplitLabel::Test => &self.test,
        }
    }

    /// Total number of cases across all splits.
    pub fn len(&self) -> usize {
        self.train.len() + self.val.len() + self.test.len()
    }

    /// Whether the partition holds no cases.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Partition cases into train/val/test by deterministic content-keyed
/// assignment, preserving order within each split. Errors only if a case input
/// cannot be canonically hashed.
pub fn partition(
    cases: &[DatasetCase],
    config: &SplitConfig,
) -> Result<SplitPartition, JsonHashError> {
    let mut p = SplitPartition::default();
    for case in cases {
        match split_for_input(&case.input, config)? {
            SplitLabel::Train => p.train.push(case.clone()),
            SplitLabel::Val => p.val.push(case.clone()),
            SplitLabel::Test => p.test.push(case.clone()),
        }
    }
    Ok(p)
}

/// A set of cases sharing one exact input fingerprint.
///
/// Under content-keyed splitting every member necessarily lands in the same
/// split (so this is never cross-split leakage); it is surfaced as a
/// data-quality signal because duplicate inputs distort per-split weighting and
/// often indicate a generation bug. Near-duplicate (semantically-equal,
/// byte-different) detection is out of scope here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateGroup {
    /// The shared input fingerprint.
    pub fingerprint: Sha256Hash,
    /// The split these duplicates all share.
    pub split: SplitLabel,
    /// The case ids sharing this input (length ≥ 2).
    pub case_ids: Vec<DatasetCaseId>,
}

/// Report exact-duplicate inputs (a data-quality signal). Returns one
/// [`DuplicateGroup`] per fingerprint that appears on more than one case.
pub fn duplicate_inputs(
    cases: &[DatasetCase],
    config: &SplitConfig,
) -> Result<Vec<DuplicateGroup>, JsonHashError> {
    let mut groups: BTreeMap<Sha256Hash, Vec<DatasetCaseId>> = BTreeMap::new();
    for case in cases {
        let fingerprint = fingerprint_input(&case.input)?;
        groups
            .entry(fingerprint)
            .or_default()
            .push(case.case_id.clone());
    }
    let mut out = Vec::new();
    for (fingerprint, case_ids) in groups {
        if case_ids.len() > 1 {
            let split = split_for_fingerprint(&fingerprint, config);
            out.push(DuplicateGroup {
                fingerprint,
                split,
                case_ids,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{DatasetId, EnvironmentId, ProjectId, SpanId, TenantId, Timestamp, TraceId};
    use serde_json::json;

    fn case_with(id: &str, input: serde_json::Value) -> DatasetCase {
        DatasetCase {
            tenant_id: TenantId::new("t").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("p").unwrap_or_else(|err| panic!("{err}")),
            dataset_id: DatasetId::new("d").unwrap_or_else(|err| panic!("{err}")),
            case_id: DatasetCaseId::new(id).unwrap_or_else(|err| panic!("{err}")),
            source_trace_id: TraceId::new("tr").unwrap_or_else(|err| panic!("{err}")),
            source_span_id: SpanId::new("sp").unwrap_or_else(|err| panic!("{err}")),
            source_environment_id: EnvironmentId::new("env").unwrap_or_else(|err| panic!("{err}")),
            input,
            output: json!(null),
            reference: None,
            trace: json!({}),
            normalizer_version: "v1".to_string(),
            trace_schema_version: 1,
            input_artifact_hashes: vec![],
            created_at: Timestamp::default(),
        }
    }

    #[test]
    fn config_validates_fractions() {
        assert!(SplitConfig::new(0.7, 0.15, 0.15, 0).is_ok());
        assert!(matches!(
            SplitConfig::new(0.5, 0.4, 0.4, 0),
            Err(SplitError::FractionsDoNotSumToOne(_))
        ));
        assert!(matches!(
            SplitConfig::new(-0.1, 0.6, 0.5, 0),
            Err(SplitError::InvalidFraction(_))
        ));
        assert!(matches!(
            SplitConfig::new(f64::NAN, 0.5, 0.5, 0),
            Err(SplitError::InvalidFraction(_))
        ));
    }

    #[test]
    fn assignment_is_deterministic() {
        let cfg = SplitConfig::standard(42);
        let input = json!({"prompt": "hello", "n": 7});
        let first = split_for_input(&input, &cfg).unwrap_or_else(|err| panic!("{err}"));
        for _ in 0..100 {
            assert_eq!(
                split_for_input(&input, &cfg).unwrap_or_else(|err| panic!("{err}")),
                first
            );
        }
    }

    #[test]
    fn ratios_converge_within_tolerance() {
        let cfg = SplitConfig::standard(7);
        let n = 20_000usize;
        let (mut tr, mut va, mut te) = (0usize, 0usize, 0usize);
        for i in 0..n {
            match split_for_input(&json!({ "i": i }), &cfg).unwrap_or_else(|err| panic!("{err}")) {
                SplitLabel::Train => tr += 1,
                SplitLabel::Val => va += 1,
                SplitLabel::Test => te += 1,
            }
        }
        let frac = |c: usize| c as f64 / n as f64;
        assert!((frac(tr) - 0.70).abs() < 0.02, "train frac {}", frac(tr));
        assert!((frac(va) - 0.15).abs() < 0.02, "val frac {}", frac(va));
        assert!((frac(te) - 0.15).abs() < 0.02, "test frac {}", frac(te));
    }

    #[test]
    fn identical_input_under_different_ids_shares_a_split() {
        // The leak-proof property: same content => same split, regardless of id,
        // version, or surrounding dataset. This is what makes the held-out gate
        // contamination-proof for exact inputs.
        let cfg = SplitConfig::standard(99);
        let shared = json!({"q": "what is the capital of France?"});
        let a = case_with("case-a", shared.clone());
        let b = case_with("case-b-promoted-later", shared.clone());
        let sa = split_for_input(&a.input, &cfg).unwrap_or_else(|err| panic!("{err}"));
        let sb = split_for_input(&b.input, &cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(sa, sb, "identical inputs must never span splits");
    }

    #[test]
    fn different_seeds_repartition() {
        let a = SplitConfig::standard(1);
        let b = SplitConfig::standard(2);
        let moved = (0..1000)
            .filter(|i| {
                let v = json!({ "i": i });
                split_for_input(&v, &a).unwrap_or_else(|err| panic!("{err}"))
                    != split_for_input(&v, &b).unwrap_or_else(|err| panic!("{err}"))
            })
            .count();
        assert!(
            moved > 100,
            "expected reshuffle across seeds, moved {moved}"
        );
    }

    #[test]
    fn partition_is_disjoint_and_total() {
        let cfg = SplitConfig::standard(3);
        let cases: Vec<_> = (0..500)
            .map(|i| case_with(&format!("case-{i}"), json!({ "x": i })))
            .collect();
        let p = partition(&cases, &cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(p.len(), cases.len());
        let mut seen = std::collections::BTreeSet::new();
        for split in [SplitLabel::Train, SplitLabel::Val, SplitLabel::Test] {
            for c in p.get(split) {
                assert!(seen.insert(c.case_id.clone()), "duplicate across splits");
                // Each case actually belongs to the split it was filed under.
                assert_eq!(
                    split_for_input(&c.input, &cfg).unwrap_or_else(|err| panic!("{err}")),
                    split
                );
            }
        }
    }

    #[test]
    fn all_train_config_assigns_everything_to_train() {
        let cfg = SplitConfig::new(1.0, 0.0, 0.0, 11).unwrap_or_else(|err| panic!("{err}"));
        let cases: Vec<_> = (0..200)
            .map(|i| case_with(&format!("c{i}"), json!({ "i": i })))
            .collect();
        let p = partition(&cases, &cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(p.train.len(), cases.len());
        assert!(p.val.is_empty() && p.test.is_empty());
    }

    #[test]
    fn empty_dataset_partitions_cleanly() {
        let cfg = SplitConfig::standard(0);
        let p = partition(&[], &cfg).unwrap_or_else(|err| panic!("{err}"));
        assert!(p.is_empty());
        assert!(duplicate_inputs(&[], &cfg)
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
    }

    #[test]
    fn duplicate_inputs_detected_and_share_split() {
        let cfg = SplitConfig::standard(5);
        let shared = json!({"prompt": "duplicated input"});
        let cases = vec![
            case_with("c1", shared.clone()),
            case_with("c2", shared.clone()),
            case_with("c3", json!({"prompt": "unique"})),
        ];
        let dups = duplicate_inputs(&cases, &cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dups.len(), 1, "expected exactly one duplicate group");
        assert_eq!(dups[0].case_ids.len(), 2);
        // All duplicates share one split (the leak-proof guarantee).
        let s = split_for_input(&shared, &cfg).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(dups[0].split, s);
    }

    #[test]
    fn near_but_not_identical_inputs_are_not_flagged() {
        // Byte-different inputs must NOT be reported as duplicates, so a future
        // fingerprint change can't silently start over-flagging.
        let cfg = SplitConfig::standard(5);
        let cases = vec![
            case_with("c1", json!({"a": 1})),
            case_with("c2", json!({"a": 2})),
        ];
        assert!(duplicate_inputs(&cases, &cfg)
            .unwrap_or_else(|err| panic!("{err}"))
            .is_empty());
    }
}
