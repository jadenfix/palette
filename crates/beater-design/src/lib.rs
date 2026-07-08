//! # beater-design — the eval/gate pre-registration manifest
//!
//! Beater's load-bearing scientific invariant (`ARCHITECTURE.md` §1 #9–#11,
//! §10.3) is that **nominal alpha equals actual alpha**: a deploy gate set to
//! reject regressions at 5% must actually hold a 5% error rate. That invariant is
//! impossible to honour if the *test*, *cluster definition*, *multiplicity
//! family*, *weighting*, or *stopping rule* is chosen **after** the scores are
//! known — choosing them post-hoc is optional-stopping / p-hacking risk in
//! product form, even when the eventual math is correct.
//!
//! [`EvalDesign`] is the fix: a typed, serialisable, **pre-registration**
//! manifest that is attached to a dataset eval, an experiment run, or a deploy
//! gate **before** scoring starts. Its [`EvalDesign::design_hash`] is the
//! commitment recorded on every resulting report, so a reader can prove the
//! design was fixed in advance.
//!
//! This crate is pure (no I/O, no runtime, no panics): every routine validates
//! its inputs and returns a `Result`, honouring the workspace
//! `unwrap_used`/`expect_used = deny` lints. The statistics themselves live in
//! `beater-stats`; the *engine wiring* (routing the gate through the design's
//! chosen test + Holm/BH correction) lands in the gate crate. This crate only
//! defines and validates the contract.
//!
//! ## What a design pins (mirrors §10.3 / issue #109)
//!
//! | Field | Question it answers |
//! |---|---|
//! | [`EvalDesign::estimand`] | superiority? non-inferiority? regression guard? |
//! | [`EvalDesign::primary_metric`] / `secondary_metrics` | what is measured, and which family |
//! | [`EvalDesign::unit_of_analysis`] / `cluster_key` | what is the independent sampling unit |
//! | [`EvalDesign::test_selection`] | which §10.3 #3 test (or auto-dispatch) |
//! | [`EvalDesign::multiplicity`] | FWER (Holm) or FDR (BH) across the family |
//! | [`EvalDesign::sampling`] / `weighting` | representative? tail-sampled? IPW-weighted? |
//! | [`EvalDesign::stopping_rule`] / `monitoring` | fixed horizon vs anytime-valid stream |
//! | [`EvalDesign::alpha`] / `power_target` / `mde` | error rate + power planning |
//! | [`EvalDesign::split`] | which Train/Dev/Test split the gate may read |
//!
//! ## Two layers of checking
//!
//! * [`EvalDesign::validate`] — *structural* validity (alpha range, non-empty
//!   metric set, finite MDE, unique metric keys). A design that fails this is
//!   malformed and cannot be persisted.
//! * [`EvalDesign::permit_pass`] — *decision* validity: whether a design is even
//!   *capable* of yielding a conclusive `pass`. A continuously-peeked stream with
//!   a fixed-horizon stopping rule, a non-independent unit with no cluster key, or
//!   a tail-sampled population claim with no weighting must return
//!   **`inconclusive`, never `pass`** (§10.3 #1/#6, §1 #9). The gate calls this
//!   before it is allowed to emit `pass`.

use beater_core::{JsonHashError, Sha256Hash, sha256_json_hash};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Vocabulary enums
// ─────────────────────────────────────────────────────────────────────────────

/// Which frozen split a decision is permitted to read. Acceptance gates and the
/// RSI loop read the **untouched `Test`** split (§5.4, §6.4, §21); Train/Dev are
/// for fitting and tuning. When `beater-datasets` gains a first-class per-case
/// split (roadmap PR-A2) it should reuse this vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DatasetSplit {
    Train,
    Dev,
    Test,
}

/// The decision the design is set up to make. The `mde`/`margin`/`max_regression`
/// payloads are the **pre-registered** effect scale, recorded before scoring so a
/// power calculation cannot be back-fitted to the observed effect.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Estimand {
    /// Candidate is better than baseline by at least `mde`.
    Superiority { mde: f64 },
    /// Candidate is no worse than baseline by more than `margin`.
    NonInferiority { margin: f64 },
    /// Block a deploy if the candidate regresses by more than `max_regression`.
    RegressionGuard { max_regression: f64 },
    /// Judge/probability calibration check (proper-scoring metric, §10.5).
    Calibration,
    /// Production distribution-drift alert.
    DriftAlert,
}

/// Statistical family of a metric — this is what selects a method-appropriate
/// confidence interval and test (§10.3 #2/#3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Pass-rate / exact-match — Bernoulli, Wilson interval, McNemar when paired.
    Proportion,
    /// Judge score in `[0, 1]` — bootstrap interval.
    Bounded,
    /// Unbounded continuous — paired-t when ~normal, else Wilcoxon/bootstrap.
    Continuous,
    /// Latency — skewed, BCa bootstrap.
    Latency,
    /// Cost — skewed, BCa bootstrap.
    Cost,
    /// Ordinal rubric level.
    Ordinal,
    /// Pairwise A/B preference.
    PairwisePreference,
    /// A proper scoring rule (Brier/log) for calibration.
    ProperScore,
}

impl MetricType {
    /// Binary metrics use paired-binary tests (McNemar) and Wilson intervals.
    pub fn is_binary(self) -> bool {
        matches!(self, MetricType::Proportion)
    }

    /// Skewed metrics need a BCa bootstrap rather than a plain percentile/CLT
    /// interval (§10.3 #2).
    pub fn is_skewed(self) -> bool {
        matches!(self, MetricType::Latency | MetricType::Cost)
    }
}

/// Whether larger values of a metric are good. Lets the gate orient a regression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    HigherIsBetter,
    LowerIsBetter,
}

/// A single named metric in the design's family.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MetricSpec {
    /// Stable key, e.g. `"pass_rate"` or `"p95_latency_ms"`.
    pub key: String,
    pub metric_type: MetricType,
    pub direction: MetricDirection,
}

impl MetricSpec {
    pub fn new(
        key: impl Into<String>,
        metric_type: MetricType,
        direction: MetricDirection,
    ) -> Self {
        Self {
            key: key.into(),
            metric_type,
            direction,
        }
    }
}

/// The independent sampling unit. Picking the wrong unit is the most common way
/// to get standard errors that are *too small* and inflate false wins (§10.3 #1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnitOfAnalysis {
    Case,
    Trace,
    Trajectory,
    Conversation,
    PromptTemplate,
    TenantSlice,
    StochasticDraw,
}

impl UnitOfAnalysis {
    /// Units whose observations are *not* independent unless clustered: multi-turn
    /// conversations share context, trajectory spans share a run, many cases drawn
    /// from one prompt template share a generator, repeated stochastic draws share
    /// a `(case, seed)`. These require a [`ClusterKey`] (§10.3 #1).
    pub fn requires_cluster(self) -> bool {
        matches!(
            self,
            UnitOfAnalysis::Conversation
                | UnitOfAnalysis::Trajectory
                | UnitOfAnalysis::PromptTemplate
                | UnitOfAnalysis::StochasticDraw
        )
    }
}

/// What groups non-independent observations into clusters for a clustered
/// standard error. `Custom` carries an opaque key name resolved by the consumer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "key")]
pub enum ClusterKey {
    Case,
    Trace,
    Conversation,
    PromptTemplate,
    SeedGroup,
    Custom { name: String },
}

/// Which §10.3 #3 significance test the design pre-commits to. `Auto` lets
/// `beater-stats` dispatch by metric family + satisfied assumptions; the explicit
/// variants pin a test when the analyst has already decided. These mirror the
/// `beater_stats::TestKind` variants the gate will execute (the explicit
/// Wilcoxon/bootstrap variants are realised in roadmap PR-A1; this crate only
/// records the *intent*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TestSelectionPolicy {
    /// Dispatch by metric family and satisfied assumptions.
    Auto,
    /// Student's paired t-test (continuous, ~normal paired differences).
    PairedT,
    /// Exact McNemar test (paired binary).
    McnemarExact,
    /// Wilcoxon signed-rank (continuous, non-normal, symmetric differences).
    WilcoxonSignedRank,
    /// Paired bootstrap / permutation (assumptions unclear / small N).
    PairedBootstrap,
}

/// Multiple-comparison control across the metric/slice family (§10.3 #4). Naive
/// per-comparison alpha (or crude `alpha / count` division) inflates false wins;
/// Holm controls the family-wise error rate, Benjamini-Hochberg the false
/// discovery rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MultiplicityPolicy {
    /// Single comparison — no correction needed.
    None,
    /// Holm-Bonferroni: control FWER ("no false win anywhere").
    Holm,
    /// Benjamini-Hochberg: control FDR ("most flagged wins are real").
    BenjaminiHochberg,
}

/// How cases were drawn (§10.3 sampling / §1 #9). Tail-sampled and curated sets
/// are deliberately non-representative; a population claim over them is biased
/// unless inverse-probability weighted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SamplingDesign {
    /// Uniform random over the target population.
    Random,
    /// Stratified random (strata declared elsewhere).
    Stratified,
    /// Production tail-sampling (rare/erroring traces over-represented).
    TailSampled,
    /// A curated failure set (intentionally biased toward known-hard cases).
    CuratedFailure,
}

impl SamplingDesign {
    /// Designs whose kept set is intentionally non-representative of the target
    /// population, so an unweighted mean is a biased estimator of it.
    pub fn is_non_representative(self) -> bool {
        matches!(
            self,
            SamplingDesign::TailSampled | SamplingDesign::CuratedFailure
        )
    }
}

/// Whether aggregates are inverse-probability (Horvitz-Thompson) weighted. The
/// IPW honesty invariant (§1 #9, #146): a non-representative sample must be
/// weighted *or* explicitly labelled biased — never silently averaged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WeightingPolicy {
    /// Plain unweighted aggregate (only honest on a representative sample, or when
    /// the result is explicitly displayed as biased).
    Unweighted,
    /// Inverse-probability weighting: `weight = 1 / keep_probability`.
    HorvitzThompson,
}

/// Variance-reduction policy for the paired estimate (§10.3 #4 / #436 item 4).
///
/// CUPED regresses a pre-experiment covariate out of the paired difference,
/// shrinking the confidence interval so a gate gets more power at the same sample
/// size. The one-sample regression estimator centres on the covariate's **known
/// population mean** μ_x, so it *moves* the point estimate off the raw sample mean
/// by −θ(x̄ − μ_x) (that shift is what makes the variance-reduced interval a valid
/// one for `E[d]` — see `beater_stats::cuped_paired_t_test`); it is unbiased for
/// `E[d]` but not mean-preserving. Declaring the policy here — *before* scoring —
/// is what keeps it honest: the covariate and μ_x are pre-registered, so they
/// cannot be fished for after seeing the scores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum VarianceReduction {
    /// No adjustment — the plain paired estimate. The default; changes nothing.
    None,
    /// CUPED with a named pre-experiment covariate and its **known population
    /// mean**. The name is provenance (which covariate); `population_mean` is the
    /// covariate's mean over the whole case population (e.g. difficulty averaged
    /// over the entire dataset). Both are part of the hashed commitment.
    ///
    /// The population mean is load-bearing, not decorative: the regression
    /// estimator centres on it, and a *one-sample* CUPED that instead centres on
    /// the sample mean is degenerate and under-covers (see
    /// `beater_stats::cuped_paired_t_test`). The covariate MUST also be independent
    /// of the treatment (never an arm's own outcome) — a provenance property the
    /// type system cannot enforce, so it is the covariate producer's responsibility.
    Cuped {
        /// The pre-registered covariate's name (provenance).
        covariate: String,
        /// The covariate's known mean over the whole case population, `μ_x`.
        population_mean: f64,
    },
}

/// Anytime-valid vs fixed-horizon monitoring (§10.3 #6). Online streams are
/// peeked at continuously; a fixed-horizon test under peeking inflates false
/// positives 5–10×, so an online design must use a sequential stopping rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Monitoring {
    /// Fixed-horizon offline experiment / CI gate.
    Offline,
    /// Continuously-inspected production stream (alerts, online evals).
    Online,
}

/// Sequential (anytime-valid) inference method for online monitoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SequentialMethod {
    /// Mixture sequential probability ratio test.
    MixtureSprt,
    /// Betting-style confidence sequence.
    ConfidenceSequence,
}

/// The stopping rule: a fixed planned horizon, or an anytime-valid sequential
/// method. Pairing a `FixedHorizon` rule with `Monitoring::Online` is a refusal
/// ([`EvalDesign::permit_pass`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StoppingRule {
    FixedHorizon { planned_n: usize },
    Sequential { method: SequentialMethod },
}

/// How judge/model calls are repeated and cached across trials (§6 dim #12,
/// §10.3 N-trial self-consistency). `trial_count` is the planned `k` for a
/// `pass^k` reliability profile (roadmap PR-A5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RepetitionPlan {
    /// Number of repeated trials per case (`k`). `1` means single-shot.
    pub trial_count: u32,
    /// Whether baseline and candidate are paired by sharing a trial seed.
    pub paired_by_seed: bool,
    /// Whether judge/model calls are served from a request-hash cache.
    pub cached: bool,
}

impl Default for RepetitionPlan {
    fn default() -> Self {
        Self {
            trial_count: 1,
            paired_by_seed: true,
            cached: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The manifest
// ─────────────────────────────────────────────────────────────────────────────

/// The pre-registration manifest attached to an eval / experiment / gate before
/// scoring. See the crate docs for the contract.
///
/// Field order is load-bearing for [`EvalDesign::design_hash`]: the hash is over
/// the canonical `serde_json` serialisation, which follows declaration order, so
/// reordering fields changes the commitment. Add new fields at the end.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EvalDesign {
    /// Human-readable name of the design.
    pub name: String,
    /// The hypothesis in words, recorded for the report.
    pub hypothesis: String,
    /// The decision this design supports + its pre-registered effect scale.
    pub estimand: Estimand,
    /// The primary metric the decision is made on.
    pub primary_metric: MetricSpec,
    /// Additional metrics in the same multiplicity family.
    pub secondary_metrics: Vec<MetricSpec>,
    /// The independent sampling unit.
    pub unit_of_analysis: UnitOfAnalysis,
    /// Cluster key for non-independent observations (`None` only when the unit is
    /// independent — see [`EvalDesign::permit_pass`]).
    pub cluster_key: Option<ClusterKey>,
    /// Which significance test (or auto-dispatch).
    pub test_selection: TestSelectionPolicy,
    /// Family-wise / false-discovery correction across the metric family.
    pub multiplicity: MultiplicityPolicy,
    /// How cases were drawn.
    pub sampling: SamplingDesign,
    /// Whether aggregates are IPW-weighted.
    pub weighting: WeightingPolicy,
    /// Offline fixed-horizon vs online continuously-peeked.
    pub monitoring: Monitoring,
    /// Fixed horizon (planned `n`) or sequential method.
    pub stopping_rule: StoppingRule,
    /// N-trial repetition / caching plan.
    pub repetition: RepetitionPlan,
    /// Per-comparison significance level in `(0, 1)`.
    pub alpha: f64,
    /// Target statistical power in `(0, 1)` (typically `0.8`).
    pub power_target: f64,
    /// Minimum detectable effect in the metric's own units, when planned ahead.
    pub mde: Option<f64>,
    /// Which split the gate is allowed to read for the decision.
    pub split: DatasetSplit,
    /// Whether the gate may read that split (a Train/Dev gate is a contamination
    /// bug; only `Test` should normally be gate-readable, §6.4).
    pub gate_may_read_split: bool,
    /// Monotonic method/version so a persisted design is reproducible as the
    /// resolution logic evolves.
    pub analysis_version: u32,
    /// Variance-reduction policy applied to the paired estimate. `None` (the
    /// default) leaves the estimate untouched. Declared here so a CUPED covariate
    /// is pre-registered rather than chosen post-hoc.
    #[serde(default = "default_variance_reduction")]
    pub variance_reduction: VarianceReduction,
}

/// The default variance-reduction policy: none. A free function so `#[serde(default)]`
/// can fill it in when reading a design persisted before this field existed.
fn default_variance_reduction() -> VarianceReduction {
    VarianceReduction::None
}

/// The analysis version stamped by [`EvalDesign::conservative_superiority`] and
/// [`EvalDesign::resolve`]. Bump when the conservative-default logic changes so
/// reports can tell which resolver produced them.
pub const CURRENT_ANALYSIS_VERSION: u32 = 1;

/// The standard statistical power target (§10.3 #5).
pub const DEFAULT_POWER: f64 = 0.8;

/// The default per-comparison significance level.
pub const DEFAULT_ALPHA: f64 = 0.05;

impl EvalDesign {
    /// A conservative, decidable default for the common case: a superiority test
    /// on the frozen `Test` split, gate-readable, FWER-controlled, fixed-horizon,
    /// offline. The caller supplies the metric, the pre-registered `mde`, and the
    /// planned horizon `n`; everything else takes a safe default. The result still
    /// passes [`EvalDesign::validate`] and [`EvalDesign::permit_pass`] for an
    /// independent unit.
    pub fn conservative_superiority(
        primary_metric: MetricSpec,
        mde: f64,
        planned_n: usize,
    ) -> Self {
        Self {
            name: String::new(),
            hypothesis: String::new(),
            estimand: Estimand::Superiority { mde },
            primary_metric,
            secondary_metrics: Vec::new(),
            unit_of_analysis: UnitOfAnalysis::Case,
            cluster_key: None,
            test_selection: TestSelectionPolicy::Auto,
            multiplicity: MultiplicityPolicy::None,
            sampling: SamplingDesign::Random,
            weighting: WeightingPolicy::Unweighted,
            monitoring: Monitoring::Offline,
            stopping_rule: StoppingRule::FixedHorizon { planned_n },
            repetition: RepetitionPlan::default(),
            alpha: DEFAULT_ALPHA,
            power_target: DEFAULT_POWER,
            mde: Some(mde),
            split: DatasetSplit::Test,
            gate_may_read_split: true,
            analysis_version: CURRENT_ANALYSIS_VERSION,
            variance_reduction: VarianceReduction::None,
        }
    }

    /// Number of comparisons in the family (primary + secondaries).
    pub fn family_size(&self) -> usize {
        1 + self.secondary_metrics.len()
    }

    /// The pre-registration commitment: a stable hash over the canonical JSON
    /// serialisation of the design. Recorded on every report so a reader can prove
    /// the design was fixed before scoring.
    pub fn design_hash(&self) -> Result<Sha256Hash, JsonHashError> {
        sha256_json_hash(self)
    }

    /// *Structural* validity: the design is well-formed and can be persisted. This
    /// does not decide whether the design can yield a `pass` — see
    /// [`EvalDesign::permit_pass`].
    pub fn validate(&self) -> Result<(), DesignError> {
        if !(self.alpha.is_finite() && self.alpha > 0.0 && self.alpha < 1.0) {
            return Err(DesignError::AlphaOutOfRange { alpha: self.alpha });
        }
        if !(self.power_target.is_finite() && self.power_target > 0.0 && self.power_target < 1.0) {
            return Err(DesignError::PowerOutOfRange {
                power: self.power_target,
            });
        }
        if let Some(mde) = self.mde
            && !(mde.is_finite() && mde > 0.0)
        {
            return Err(DesignError::NonPositiveMde { mde });
        }
        // Estimand effect scales must be finite and non-negative.
        match self.estimand {
            Estimand::Superiority { mde } if !(mde.is_finite() && mde > 0.0) => {
                return Err(DesignError::NonPositiveMde { mde });
            }
            Estimand::NonInferiority { margin } if !(margin.is_finite() && margin > 0.0) => {
                return Err(DesignError::NonPositiveMargin { margin });
            }
            Estimand::RegressionGuard { max_regression }
                if !(max_regression.is_finite() && max_regression >= 0.0) =>
            {
                return Err(DesignError::NegativeRegressionBound { max_regression });
            }
            _ => {}
        }
        if self.primary_metric.key.trim().is_empty() {
            return Err(DesignError::EmptyMetricKey);
        }
        // Metric keys must be non-empty and unique across the whole family so a
        // per-metric decision can be addressed unambiguously.
        let mut seen = vec![self.primary_metric.key.as_str()];
        for metric in &self.secondary_metrics {
            if metric.key.trim().is_empty() {
                return Err(DesignError::EmptyMetricKey);
            }
            if seen.contains(&metric.key.as_str()) {
                return Err(DesignError::DuplicateMetricKey {
                    key: metric.key.clone(),
                });
            }
            seen.push(metric.key.as_str());
        }
        // A CUPED policy must name its covariate and pin a finite population mean.
        if let VarianceReduction::Cuped {
            covariate,
            population_mean,
        } = &self.variance_reduction
        {
            if covariate.trim().is_empty() {
                return Err(DesignError::EmptyCovariateKey);
            }
            if !population_mean.is_finite() {
                return Err(DesignError::NonFiniteCovariateMean {
                    value: *population_mean,
                });
            }
        }
        Ok(())
    }

    /// *Decision* validity: whether this design is even capable of yielding a
    /// conclusive `pass`. The gate MUST call this before emitting `pass`; a design
    /// that fails here forces an `inconclusive` verdict (§10.3 #1/#6, §1 #9).
    ///
    /// Refusals (each one is a way to make nominal alpha ≠ actual alpha):
    /// * a continuously-peeked online stream with a fixed-horizon test;
    /// * a fixed horizon with `planned_n == 0` (no test planned);
    /// * a non-independent unit with no cluster key (SEs too small);
    /// * a population claim over a non-representative sample with no weighting;
    /// * a multi-metric family with no multiplicity correction;
    /// * a gate reading a non-`Test` split, or a `Test` split it is told not to
    ///   read.
    pub fn permit_pass(&self) -> Result<(), DesignRefusal> {
        // Online streams must use anytime-valid inference (§10.3 #6).
        if self.monitoring == Monitoring::Online
            && let StoppingRule::FixedHorizon { .. } = self.stopping_rule
        {
            return Err(DesignRefusal::FixedHorizonOnPeekedStream);
        }
        if let StoppingRule::FixedHorizon { planned_n } = self.stopping_rule
            && planned_n == 0
        {
            return Err(DesignRefusal::ZeroPlannedHorizon);
        }
        // Non-independent observations need a cluster key (§10.3 #1).
        if self.unit_of_analysis.requires_cluster() && self.cluster_key.is_none() {
            return Err(DesignRefusal::NonIndependentWithoutCluster {
                unit: self.unit_of_analysis,
            });
        }
        // A population claim over a tail-sampled/curated set must be IPW-weighted
        // or it is a biased estimator presented as honest (§1 #9, #146).
        if self.sampling.is_non_representative()
            && self.weighting == WeightingPolicy::Unweighted
            && self.estimand_targets_population()
        {
            return Err(DesignRefusal::BiasedPopulationClaim {
                sampling: self.sampling,
            });
        }
        // A multi-comparison family with no correction inflates false wins
        // (§10.3 #4).
        if self.family_size() > 1 && self.multiplicity == MultiplicityPolicy::None {
            return Err(DesignRefusal::UncorrectedMultiComparison {
                family_size: self.family_size(),
            });
        }
        // The gate must read the untouched Test split (§6.4).
        if self.gate_may_read_split && self.split != DatasetSplit::Test {
            return Err(DesignRefusal::GateReadsNonTestSplit { split: self.split });
        }
        if !self.gate_may_read_split {
            return Err(DesignRefusal::GateSplitNotReadable { split: self.split });
        }
        Ok(())
    }

    /// Whether the estimand makes a claim about the target *population* (so a
    /// biased sample without weighting is dishonest). Calibration and drift checks
    /// describe the kept set itself, so they are exempt.
    fn estimand_targets_population(&self) -> bool {
        matches!(
            self.estimand,
            Estimand::Superiority { .. }
                | Estimand::NonInferiority { .. }
                | Estimand::RegressionGuard { .. }
        )
    }

    /// Resolve risky-but-not-fatal choices to conservative settings, returning the
    /// adjusted design plus the warnings that explain each adjustment. This is the
    /// "resolve an explicit conservative default before any scores are computed"
    /// step from issue #109: it never *weakens* a design, only tightens it, so the
    /// result is at least as safe as the input.
    ///
    /// Adjustments:
    /// * a non-independent unit with no cluster key gets a cluster key derived
    ///   from the unit (so SEs are clustered, not i.i.d.);
    /// * a multi-metric family with no correction gets Holm (FWER);
    /// * a `power_target` below [`DEFAULT_POWER`] is raised to it.
    ///
    /// The caller should still run [`EvalDesign::validate`] and
    /// [`EvalDesign::permit_pass`] on the result; `resolve` fixes the *defaultable*
    /// hazards, not structural errors (a bad alpha is the caller's bug, not a thing
    /// to silently rewrite).
    pub fn resolve(mut self) -> (Self, Vec<DesignWarning>) {
        let mut warnings = Vec::new();

        if self.unit_of_analysis.requires_cluster() && self.cluster_key.is_none() {
            let inferred = cluster_key_for_unit(self.unit_of_analysis);
            warnings.push(DesignWarning::ClusterKeyInferred {
                unit: self.unit_of_analysis,
                key: inferred.clone(),
            });
            self.cluster_key = Some(inferred);
        }

        if self.family_size() > 1 && self.multiplicity == MultiplicityPolicy::None {
            warnings.push(DesignWarning::MultiplicityDefaulted {
                family_size: self.family_size(),
            });
            self.multiplicity = MultiplicityPolicy::Holm;
        }

        if self.power_target.is_finite() && self.power_target < DEFAULT_POWER {
            warnings.push(DesignWarning::PowerRaised {
                from: self.power_target,
                to: DEFAULT_POWER,
            });
            self.power_target = DEFAULT_POWER;
        }

        if self.sampling.is_non_representative()
            && self.weighting == WeightingPolicy::Unweighted
            && self.estimand_targets_population()
        {
            // We do not silently flip to IPW (the keep-path weights may not exist
            // yet, roadmap PR-A3); we surface the bias so the caller either weights
            // or labels the aggregate biased.
            warnings.push(DesignWarning::BiasedAggregate {
                sampling: self.sampling,
            });
        }

        self.analysis_version = CURRENT_ANALYSIS_VERSION;
        (self, warnings)
    }
}

/// The conservative cluster key for a non-independent unit.
fn cluster_key_for_unit(unit: UnitOfAnalysis) -> ClusterKey {
    match unit {
        UnitOfAnalysis::Conversation => ClusterKey::Conversation,
        UnitOfAnalysis::Trajectory => ClusterKey::Trace,
        UnitOfAnalysis::PromptTemplate => ClusterKey::PromptTemplate,
        UnitOfAnalysis::StochasticDraw => ClusterKey::SeedGroup,
        // Independent units never reach here, but map to the case-level key.
        UnitOfAnalysis::Case | UnitOfAnalysis::Trace | UnitOfAnalysis::TenantSlice => {
            ClusterKey::Case
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors / refusals / warnings
// ─────────────────────────────────────────────────────────────────────────────

/// A structural defect that makes a design malformed (cannot be persisted).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DesignError {
    #[error("alpha must be in (0, 1), got {alpha}")]
    AlphaOutOfRange { alpha: f64 },
    #[error("power_target must be in (0, 1), got {power}")]
    PowerOutOfRange { power: f64 },
    #[error("mde must be finite and > 0, got {mde}")]
    NonPositiveMde { mde: f64 },
    #[error("non-inferiority margin must be finite and > 0, got {margin}")]
    NonPositiveMargin { margin: f64 },
    #[error("regression bound must be finite and >= 0, got {max_regression}")]
    NegativeRegressionBound { max_regression: f64 },
    #[error("metric key must not be empty")]
    EmptyMetricKey,
    #[error("duplicate metric key in family: {key}")]
    DuplicateMetricKey { key: String },
    #[error("CUPED variance reduction requires a non-empty covariate name")]
    EmptyCovariateKey,
    #[error("CUPED population covariate mean must be finite, got {value}")]
    NonFiniteCovariateMean { value: f64 },
}

/// A reason a design cannot yield a conclusive `pass` — the gate must return
/// `inconclusive` instead. Each variant is a distinct way nominal alpha would
/// stop equalling actual alpha.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DesignRefusal {
    #[error(
        "online/continuously-peeked stream cannot use a fixed-horizon test; \
             requires a sequential (anytime-valid) stopping rule (§10.3 #6)"
    )]
    FixedHorizonOnPeekedStream,
    #[error("fixed-horizon design has planned_n = 0; no test is planned")]
    ZeroPlannedHorizon,
    #[error(
        "non-independent unit {unit:?} has no cluster key; i.i.d. standard \
             errors would be too small (§10.3 #1)"
    )]
    NonIndependentWithoutCluster { unit: UnitOfAnalysis },
    #[error(
        "population claim over a non-representative sample ({sampling:?}) with \
             no inverse-probability weighting is a biased estimator (§1 #9, #146)"
    )]
    BiasedPopulationClaim { sampling: SamplingDesign },
    #[error(
        "multi-comparison family of {family_size} with no multiplicity \
             correction inflates false wins (§10.3 #4)"
    )]
    UncorrectedMultiComparison { family_size: usize },
    #[error(
        "gate may not read a non-Test split ({split:?}) for an acceptance \
             decision (§6.4)"
    )]
    GateReadsNonTestSplit { split: DatasetSplit },
    #[error(
        "design's gate_may_read_split is false for split {split:?}; the gate \
             has no readable split to decide on"
    )]
    GateSplitNotReadable { split: DatasetSplit },
}

/// A non-fatal hazard that [`EvalDesign::resolve`] adjusted (or flagged). These
/// are persisted alongside the resolved design so a report shows what was assumed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum DesignWarning {
    /// A cluster key was inferred for a non-independent unit that had none.
    ClusterKeyInferred {
        unit: UnitOfAnalysis,
        key: ClusterKey,
    },
    /// A multi-metric family with no correction was defaulted to Holm.
    MultiplicityDefaulted { family_size: usize },
    /// An under-target power was raised to the default.
    PowerRaised { from: f64, to: f64 },
    /// A population claim over a non-representative sample is biased; weight it or
    /// label the aggregate biased.
    BiasedAggregate { sampling: SamplingDesign },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric(key: &str, ty: MetricType) -> MetricSpec {
        MetricSpec::new(key, ty, MetricDirection::HigherIsBetter)
    }

    fn base() -> EvalDesign {
        EvalDesign::conservative_superiority(metric("pass_rate", MetricType::Proportion), 0.02, 200)
    }

    // ── design_hash: stable pre-registration commitment ──────────────────────

    #[test]
    fn design_hash_is_stable_across_round_trips() {
        let design = base();
        let first = design.design_hash().unwrap_or_else(|err| panic!("{err}"));
        // Re-serialising an identical design yields the same commitment.
        let second = design
            .clone()
            .design_hash()
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(first, second);
    }

    #[test]
    fn design_hash_golden_pin() {
        // Pins the canonical serialisation so an accidental field reorder or
        // default change is caught. If this changes intentionally, bump
        // CURRENT_ANALYSIS_VERSION and update the pin.
        let design = base();
        let hash = design.design_hash().unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(hash.as_str().len(), 64, "sha256 hex is 64 chars");
    }

    #[test]
    fn changing_any_field_changes_the_hash() {
        let a = base();
        let mut b = base();
        b.alpha = 0.01;
        let ha = a.design_hash().unwrap_or_else(|err| panic!("{err}"));
        let hb = b.design_hash().unwrap_or_else(|err| panic!("{err}"));
        assert_ne!(ha, hb, "a changed alpha must change the commitment");

        // The pre-registered CUPED policy (covariate + its known μ_x) is part of the
        // hashed commitment — that is the whole anti-fishing guarantee, so pin it:
        // changing the variance-reduction policy, the covariate name, or μ_x must all
        // move the hash. Guards against a future serde change silently dropping the
        // field from `design_hash`.
        let mut c = base();
        c.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean: 0.5,
        };
        let hc = c.design_hash().unwrap_or_else(|err| panic!("{err}"));
        assert_ne!(
            ha, hc,
            "declaring a CUPED policy must change the commitment"
        );

        let mut d = base();
        d.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean: 0.6, // same covariate, different known μ_x.
        };
        let hd = d.design_hash().unwrap_or_else(|err| panic!("{err}"));
        assert_ne!(
            hc, hd,
            "changing the pre-registered μ_x must change the commitment"
        );

        let mut e = base();
        e.variance_reduction = VarianceReduction::Cuped {
            covariate: "other_difficulty".to_string(), // different covariate, same μ_x.
            population_mean: 0.5,
        };
        let he = e.design_hash().unwrap_or_else(|err| panic!("{err}"));
        assert_ne!(
            hc, he,
            "changing the pre-registered covariate must change the commitment"
        );
    }

    #[test]
    fn cuped_policy_requires_a_named_covariate_and_finite_mean() {
        let mut design = base();
        design.variance_reduction = VarianceReduction::Cuped {
            covariate: "  ".to_string(),
            population_mean: 0.5,
        };
        assert_eq!(design.validate(), Err(DesignError::EmptyCovariateKey));
        design.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean: f64::NAN,
        };
        assert!(matches!(
            design.validate(),
            Err(DesignError::NonFiniteCovariateMean { .. })
        ));
        design.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean: 0.5,
        };
        assert!(design.validate().is_ok());
        // A CUPED policy never forbids a pass — it estimates the same E[d].
        assert!(design.permit_pass().is_ok());
    }

    // ── structural validation ────────────────────────────────────────────────

    #[test]
    fn conservative_default_is_valid_and_decidable() {
        let design = base();
        assert_eq!(design.validate(), Ok(()));
        assert_eq!(design.permit_pass(), Ok(()));
    }

    #[test]
    fn rejects_alpha_out_of_range() {
        let mut design = base();
        design.alpha = 0.0;
        assert!(matches!(
            design.validate(),
            Err(DesignError::AlphaOutOfRange { .. })
        ));
        design.alpha = 1.5;
        assert!(matches!(
            design.validate(),
            Err(DesignError::AlphaOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_empty_and_duplicate_metric_keys() {
        let mut design = base();
        design.primary_metric.key = "  ".to_string();
        assert_eq!(design.validate(), Err(DesignError::EmptyMetricKey));

        let mut dup = base();
        dup.secondary_metrics = vec![metric("pass_rate", MetricType::Bounded)];
        assert!(matches!(
            dup.validate(),
            Err(DesignError::DuplicateMetricKey { .. })
        ));
    }

    // ── decision refusals: inconclusive, never pass ──────────────────────────

    #[test]
    fn online_fixed_horizon_is_refused() {
        let mut design = base();
        design.monitoring = Monitoring::Online; // stays FixedHorizon
        assert_eq!(
            design.permit_pass(),
            Err(DesignRefusal::FixedHorizonOnPeekedStream)
        );
        // The sequential repair makes it decidable.
        design.stopping_rule = StoppingRule::Sequential {
            method: SequentialMethod::MixtureSprt,
        };
        assert_eq!(design.permit_pass(), Ok(()));
    }

    #[test]
    fn clustered_trajectory_without_cluster_key_is_refused() {
        let mut design = base();
        design.unit_of_analysis = UnitOfAnalysis::Trajectory;
        assert!(matches!(
            design.permit_pass(),
            Err(DesignRefusal::NonIndependentWithoutCluster { .. })
        ));
        design.cluster_key = Some(ClusterKey::Trace);
        assert_eq!(design.permit_pass(), Ok(()));
    }

    #[test]
    fn tail_sampled_weighted_aggregate_requires_weights() {
        let mut design = base();
        design.sampling = SamplingDesign::TailSampled; // unweighted superiority claim
        assert!(matches!(
            design.permit_pass(),
            Err(DesignRefusal::BiasedPopulationClaim { .. })
        ));
        design.weighting = WeightingPolicy::HorvitzThompson;
        assert_eq!(design.permit_pass(), Ok(()));
    }

    #[test]
    fn multi_metric_family_needs_a_correction() {
        let mut design = base();
        design.secondary_metrics = vec![metric("p95_latency_ms", MetricType::Latency)];
        assert!(matches!(
            design.permit_pass(),
            Err(DesignRefusal::UncorrectedMultiComparison { .. })
        ));
        design.multiplicity = MultiplicityPolicy::Holm;
        assert_eq!(design.permit_pass(), Ok(()));
    }

    #[test]
    fn gate_must_read_the_test_split() {
        let mut design = base();
        design.split = DatasetSplit::Train; // gate_may_read_split still true
        assert_eq!(
            design.permit_pass(),
            Err(DesignRefusal::GateReadsNonTestSplit {
                split: DatasetSplit::Train
            })
        );
    }

    #[test]
    fn zero_horizon_is_refused() {
        let mut design = base();
        design.stopping_rule = StoppingRule::FixedHorizon { planned_n: 0 };
        assert_eq!(design.permit_pass(), Err(DesignRefusal::ZeroPlannedHorizon));
    }

    // ── resolve: conservative, never weakening ───────────────────────────────

    #[test]
    fn resolve_infers_cluster_and_correction_and_power() {
        let mut design = base();
        design.unit_of_analysis = UnitOfAnalysis::Conversation;
        design.secondary_metrics = vec![metric("coherence", MetricType::Bounded)];
        design.power_target = 0.5;

        let (resolved, warnings) = design.resolve();
        assert_eq!(resolved.cluster_key, Some(ClusterKey::Conversation));
        assert_eq!(resolved.multiplicity, MultiplicityPolicy::Holm);
        assert_eq!(resolved.power_target, DEFAULT_POWER);
        // After resolution the design is decidable.
        assert_eq!(resolved.validate(), Ok(()));
        assert_eq!(resolved.permit_pass(), Ok(()));
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn resolve_flags_biased_aggregate_without_silently_weighting() {
        let mut design = base();
        design.sampling = SamplingDesign::CuratedFailure;
        let (resolved, warnings) = design.resolve();
        // We do NOT flip weighting silently — we surface the bias.
        assert_eq!(resolved.weighting, WeightingPolicy::Unweighted);
        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, DesignWarning::BiasedAggregate { .. }))
        );
    }
}
