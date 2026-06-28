//! # beater-stats — comparative-statistics primitives for agent evaluation
//!
//! This crate provides the pure-math layer that **`beater-eval`**, **`beater-gates`**,
//! the online-eval worker, and the RSI loop (§21) all call into for statistically
//! valid comparisons.  Nothing in this crate does I/O, allocates a runtime, or
//! panics on bad input — every routine validates its inputs and returns a
//! `Result`, so the crate honors the workspace `unwrap_used`/`expect_used = deny`
//! lints.
//!
//! ## What is implemented
//!
//! | Function | ARCHITECTURE.md § | Use |
//! |---|---|---|
//! | [`wilson_interval`] | §6.3, §10.3 #2, §10.3 #7 | binomial-proportion CI |
//! | [`two_proportion_z_test`] | §6.3, §10.3 #3 | unpaired candidate-vs-baseline |
//! | [`bootstrap_diff_ci`] | §6.3, §10.3 #2, §21.4 | bounded/continuous diff CI |
//! | [`compare_paired`] | §10.3 | paired deploy-gate selector |
//! | [`paired_t_test`] | §10.3 | continuous paired metric |
//! | [`mcnemar_exact_p`] | §10.3 | paired binary outcome |
//! | [`required_sample_size`] / [`minimum_detectable_effect`] / [`achieved_power`] | §10.3 #5 | power / MDE / minimum-sample planning |
//!
//! The paired layer ([`compare_paired`]) is what the **experiment gate** calls
//! today: it picks **Student's paired t** for continuous metrics and the **exact
//! McNemar test** for paired binary outcomes, returning a *real* two-sided
//! p-value computed with a method-appropriate test — replacing the previous
//! hand-rolled paired normal-approximation in `beater-eval` that hard-coded its
//! critical value (`z = 1.96 / 2.576`) and reported no p-value at all, so its
//! *nominal* alpha did not equal its *actual* alpha.
//!
//! Anytime-valid / sequential inference (mSPRT, §10.3 #6) is the **required
//! follow-on phase** and is not included here; see §10.3 phasing note.
//!
//! ## Design invariants
//!
//! * **No panics** — inputs that violate preconditions return `Err(StatsError::…)`.
//! * **Reproducible** — all randomised code (bootstrap) accepts an explicit `seed`
//!   so results are deterministic and can be committed to a test oracle.
//! * **No heavyweight deps** — the normal/Student-t quantiles, incomplete beta,
//!   exact binomial tail, and normal CDF are hand-rolled (see [`numerics`] and the
//!   helpers below) so the crate pulls in only `thiserror`.

mod mcnemar;
mod numerics;
mod paired;
mod power;

pub use mcnemar::mcnemar_exact_p;
pub use paired::paired_t_test;
pub use power::{achieved_power, minimum_detectable_effect, required_sample_size, DEFAULT_POWER};

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

/// Errors returned by `beater-stats` functions.  They are total: every routine
/// validates its inputs and returns one of these rather than panicking.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StatsError {
    // ── Fixed-horizon proportion / bootstrap core (§10.3 #2/#3, §6.3) ──
    /// `trials` was zero — there is no proportion to estimate.
    #[error("trials must be > 0")]
    ZeroTrials,
    /// `successes` exceeded `trials`.
    #[error("successes ({successes}) > trials ({trials})")]
    SuccessesExceedTrials { successes: u64, trials: u64 },
    /// A sample was empty where at least one observation is required.
    #[error("sample must not be empty")]
    EmptySample,
    /// A parameter was outside its valid range.
    #[error("parameter `{name}` = {value} is out of range")]
    InvalidParameter { name: &'static str, value: f64 },
    /// The requested number of bootstrap resamples must be ≥ 1.
    #[error("n_resamples must be ≥ 1, got {0}")]
    InvalidResampleCount(usize),

    // ── Paired deploy-gate layer (§10.3) ──
    /// Fewer observations than the method requires.
    #[error("too few samples: got {got}, need at least {need}")]
    TooFewSamples { got: usize, need: usize },
    /// Two paired inputs had different lengths.
    #[error("mismatched sample lengths: {baseline} vs {candidate}")]
    MismatchedLengths { baseline: usize, candidate: usize },
    /// `alpha` outside the open interval (0, 1).
    #[error("alpha must be in (0, 1), got {0}")]
    InvalidAlpha(f64),
    /// A non-finite (NaN/inf) value appeared in the input.
    #[error("non-finite value in input")]
    NonFinite,
}

// ─────────────────────────────────────────────────────────────────────────────
// Result types
// ─────────────────────────────────────────────────────────────────────────────

/// A confidence interval with its central estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    /// Lower bound of the confidence interval.
    pub lower: f64,
    /// Upper bound of the confidence interval.
    pub upper: f64,
    /// The point estimate around which the interval is centred.
    /// For Wilson this is the *Wilson centre*, not the raw `k/n`.
    pub center: f64,
}

/// Result of a two-proportion z-test.
#[derive(Debug, Clone, PartialEq)]
pub struct TwoProportionResult {
    /// The pooled-variance z-statistic (signed: positive means `p1 > p2`).
    pub z_stat: f64,
    /// Two-sided p-value: `2 · Φ(−|z|)`.
    pub p_value: f64,
}

/// A bootstrap confidence interval for a scalar statistic.
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapInterval {
    /// Lower percentile endpoint.
    pub lower: f64,
    /// Upper percentile endpoint.
    pub upper: f64,
    /// The observed statistic on the original (un-resampled) samples.
    pub estimate: f64,
    /// Number of bootstrap resamples actually used.
    pub n_resamples: usize,
}

/// A confidence interval for a point estimate at a stated confidence level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfidenceInterval {
    pub low: f64,
    pub high: f64,
    /// e.g. 0.95 for a 95% interval (== 1 - alpha).
    pub confidence: f64,
}

/// Which test produced an outcome — recorded so a reader can tell a t-test
/// result from an exact McNemar one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKind {
    /// Student's paired t-test (continuous paired metric).
    PairedT,
    /// Exact McNemar test (paired binary outcome).
    McnemarExact,
}

/// The result of a hypothesis test: the point estimate (always the mean
/// difference), its confidence interval, a real two-sided p-value, the test
/// used, the degrees of freedom where defined, and the effective sample size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestOutcome {
    pub estimate: f64,
    pub ci: Option<ConfidenceInterval>,
    pub p_value: f64,
    pub test: TestKind,
    pub df: Option<f64>,
    pub sample_size: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Common z-scores (Φ⁻¹(1 − α/2))
// ─────────────────────────────────────────────────────────────────────────────

/// z for 90 % confidence (Φ⁻¹(0.95)).
pub const Z_90: f64 = 1.644_853_626_951_472_7;
/// z for 95 % confidence (Φ⁻¹(0.975)).  Used in §6.3 and §10.3.
pub const Z_95: f64 = 1.959_963_984_540_054;
/// z for 99 % confidence (Φ⁻¹(0.995)).
pub const Z_99: f64 = 2.575_829_303_548_901;

// ─────────────────────────────────────────────────────────────────────────────
// §6.3 / §10.3 #2 — Wilson score interval
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the **Wilson score confidence interval** for a binomial proportion.
///
/// This is the interval mandated by §10.3 #2 and §6.3 for all binary/proportion
/// metrics (task-success rate, tool-call correctness, guardrail conformance, …).
/// It does **not** degrade at `p̂ = 0` or `p̂ = 1` or at small `n`, unlike the
/// Wald interval the previous code used.
///
/// # Formula (ARCHITECTURE.md §10.3 #7)
///
/// ```text
/// p̂     = successes / trials
/// center = (p̂ + z²/2n) / (1 + z²/n)
/// half   = (z / (1 + z²/n)) · √(p̂(1−p̂)/n + z²/4n²)
/// CI     = [center − half, center + half]
/// ```
///
/// # Arguments
///
/// * `successes` — number of successes `k`.
/// * `trials` — total number of Bernoulli trials `n`; must be `> 0`.
/// * `z` — the normal quantile for the desired confidence level.
///   Use [`Z_95`], [`Z_99`], etc., or compute your own via [`normal_quantile`].
///
/// # Errors
///
/// Returns [`StatsError::ZeroTrials`] when `trials == 0`, or
/// [`StatsError::SuccessesExceedTrials`] when `successes > trials`.
///
/// # Example
///
/// ```
/// use beater_stats::{wilson_interval, Z_95};
///
/// let ci = wilson_interval(8, 10, Z_95).unwrap();
/// // 95 % Wilson CI for 8/10 ≈ [0.490, 0.943]
/// assert!((ci.lower - 0.490).abs() < 1e-3);
/// assert!((ci.upper - 0.943).abs() < 1e-3);
/// ```
pub fn wilson_interval(successes: u64, trials: u64, z: f64) -> Result<Interval, StatsError> {
    if trials == 0 {
        return Err(StatsError::ZeroTrials);
    }
    if successes > trials {
        return Err(StatsError::SuccessesExceedTrials { successes, trials });
    }
    if !z.is_finite() || z <= 0.0 {
        return Err(StatsError::InvalidParameter { name: "z", value: z });
    }

    let n = trials as f64;
    let p_hat = successes as f64 / n;
    let z2 = z * z;

    // Wilson centre — shifted from p̂ towards 0.5 by the z²/n term
    let center = (p_hat + z2 / (2.0 * n)) / (1.0 + z2 / n);

    // Half-width; safe at p̂ = 0 and p̂ = 1 (the second term under the sqrt
    // keeps the interval open when the sample proportion is at a boundary)
    let variance = p_hat * (1.0 - p_hat) / n + z2 / (4.0 * n * n);
    let half = (z / (1.0 + z2 / n)) * variance.sqrt();

    Ok(Interval {
        lower: (center - half).max(0.0),
        upper: (center + half).min(1.0),
        center,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// §6.3 / §10.3 #3 — Two-proportion z-test (candidate vs baseline)
// ─────────────────────────────────────────────────────────────────────────────

/// Test whether two independent binomial proportions differ significantly.
///
/// Used by the acceptance gates (§10.3 #3, §12) to compare a **candidate** agent's
/// pass-rate against the **baseline** pass-rate when the comparison is unpaired
/// (different case assignments) and N is large enough for the normal approximation.
///
/// # Statistic
///
/// ```text
/// p₁ = k₁/n₁,  p₂ = k₂/n₂
/// p_pool = (k₁ + k₂) / (n₁ + n₂)          // pooled under H₀: p₁ = p₂
/// SE     = √(p_pool·(1−p_pool)·(1/n₁ + 1/n₂))
/// z      = (p₁ − p₂) / SE                  // positive ⟹ candidate better
/// p-val  = 2·Φ(−|z|)                        // two-sided
/// ```
///
/// # Errors
///
/// Returns [`StatsError::ZeroTrials`] if either trial count is zero, or
/// [`StatsError::SuccessesExceedTrials`] if either `k > n`.
/// Returns [`StatsError::InvalidParameter`] when the pooled SE is exactly zero
/// (all observations in both arms agree — report with z=0, p=1 in that case;
/// this path is not an error but is noted in the returned struct).
///
/// # Example
///
/// ```
/// use beater_stats::two_proportion_z_test;
///
/// // candidate: 85/100, baseline: 70/100 → should be significant at 95 %
/// let res = two_proportion_z_test(85, 100, 70, 100).unwrap();
/// assert!(res.p_value < 0.05);
/// assert!(res.z_stat > 0.0); // candidate is better
/// ```
pub fn two_proportion_z_test(
    k1: u64,
    n1: u64,
    k2: u64,
    n2: u64,
) -> Result<TwoProportionResult, StatsError> {
    if n1 == 0 || n2 == 0 {
        return Err(StatsError::ZeroTrials);
    }
    if k1 > n1 {
        return Err(StatsError::SuccessesExceedTrials {
            successes: k1,
            trials: n1,
        });
    }
    if k2 > n2 {
        return Err(StatsError::SuccessesExceedTrials {
            successes: k2,
            trials: n2,
        });
    }

    let p1 = k1 as f64 / n1 as f64;
    let p2 = k2 as f64 / n2 as f64;
    let p_pool = (k1 + k2) as f64 / (n1 + n2) as f64;

    let se_sq = p_pool * (1.0 - p_pool) * (1.0 / n1 as f64 + 1.0 / n2 as f64);

    // SE = 0 iff both proportions equal 0 or both equal 1; no information
    if se_sq == 0.0 || !se_sq.is_finite() {
        return Ok(TwoProportionResult {
            z_stat: 0.0,
            p_value: 1.0,
        });
    }

    let z_stat = (p1 - p2) / se_sq.sqrt();
    let p_value = 2.0 * normal_cdf(-z_stat.abs());

    Ok(TwoProportionResult { z_stat, p_value })
}

// ─────────────────────────────────────────────────────────────────────────────
// §6.3 #2/#4/#9 / §10.3 #2 / §21.4 — Bootstrap confidence interval
// ─────────────────────────────────────────────────────────────────────────────

/// Compute a **percentile bootstrap confidence interval** for the difference of
/// means between two independent samples.
///
/// This is the estimator used for bounded/continuous metrics (judge scores,
/// latency, cost, process-quality scores, …) where the CLT is not safe — see
/// §10.3 #2.  The interval is also used by the §21.4 anti-overfitting guardrail
/// to compute the held-out generalization gap (dim #15) with an honest CI.
///
/// # Algorithm
///
/// For each of `n_resamples` bootstrap iterations:
/// 1. Draw `|sample_a|` observations from `sample_a` with replacement.
/// 2. Draw `|sample_b|` observations from `sample_b` with replacement.
/// 3. Compute `mean(resample_a) − mean(resample_b)`.
///
/// Sort the `n_resamples` differences and return the `α/2` and `1−α/2` percentile
/// endpoints as the interval.  The **observed** `mean(a) − mean(b)` is stored in
/// `estimate`.
///
/// # Reproducibility
///
/// A `seed` is required so that a reported CI can be committed to a test oracle
/// and re-derived from the raw samples.  The internal RNG is a 64-bit xorshift
/// (period 2⁶⁴−1), which is sufficient for up to `~10_000` resamples.
///
/// # Arguments
///
/// * `sample_a` — the "candidate" observations.
/// * `sample_b` — the "baseline" observations.
/// * `confidence` — e.g. `0.95` for a 95 % interval; must be in `(0, 1)`.
/// * `n_resamples` — number of bootstrap draws; `10_000` is the §10.3 default.
/// * `seed` — deterministic RNG seed.
///
/// # Errors
///
/// Returns [`StatsError::EmptySample`] if either sample is empty, or
/// [`StatsError::InvalidParameter`] if `confidence` is outside `(0, 1)`, or
/// [`StatsError::InvalidResampleCount`] if `n_resamples == 0`.
///
/// # Example
///
/// ```
/// use beater_stats::bootstrap_diff_ci;
///
/// let a = vec![0.9, 0.8, 0.95, 0.85, 0.88];
/// let b = vec![0.6, 0.7, 0.65, 0.62, 0.68];
/// let ci = bootstrap_diff_ci(&a, &b, 0.95, 10_000, 42).unwrap();
/// // candidate is better by ~0.25; CI should be entirely positive
/// assert!(ci.lower > 0.0);
/// assert!(ci.estimate > 0.0);
/// ```
pub fn bootstrap_diff_ci(
    sample_a: &[f64],
    sample_b: &[f64],
    confidence: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<BootstrapInterval, StatsError> {
    if sample_a.is_empty() || sample_b.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if !(0.0 < confidence && confidence < 1.0) {
        return Err(StatsError::InvalidParameter {
            name: "confidence",
            value: confidence,
        });
    }
    if n_resamples == 0 {
        return Err(StatsError::InvalidResampleCount(n_resamples));
    }

    let observed_est = mean(sample_a) - mean(sample_b);

    let mut rng = Xorshift64::new(seed);
    let mut diffs: Vec<f64> = Vec::with_capacity(n_resamples);

    for _ in 0..n_resamples {
        let ra = resample_mean(sample_a, &mut rng);
        let rb = resample_mean(sample_b, &mut rng);
        diffs.push(ra - rb);
    }

    diffs.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let lo_idx = ((alpha / 2.0) * n_resamples as f64).floor() as usize;
    let hi_idx = ((1.0 - alpha / 2.0) * n_resamples as f64).floor() as usize;

    let lower = diffs[lo_idx.min(n_resamples - 1)];
    let upper = diffs[hi_idx.min(n_resamples - 1)];

    Ok(BootstrapInterval {
        lower,
        upper,
        estimate: observed_est,
        n_resamples,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// §10.3 — Paired deploy-gate selector (paired-t / exact McNemar)
// ─────────────────────────────────────────────────────────────────────────────

/// Compare two paired samples (`candidate` − `baseline`) and return a real test
/// outcome. This is the entry point the experiment gate uses.
///
/// It picks **exact McNemar** when every value is 0 or 1 (a paired binary
/// outcome) and **Student's paired t** otherwise. The reported `estimate` is
/// always the mean difference `mean(candidate) − mean(baseline)`, so the CI is
/// directly comparable against a regression threshold regardless of which test
/// produced the p-value.
pub fn compare_paired(
    baseline: &[f64],
    candidate: &[f64],
    alpha: f64,
) -> Result<TestOutcome, StatsError> {
    validate_alpha(alpha)?;
    if baseline.len() != candidate.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: baseline.len(),
            candidate: candidate.len(),
        });
    }
    let n = baseline.len();
    if n < 2 {
        return Err(StatsError::TooFewSamples { got: n, need: 2 });
    }
    for value in baseline.iter().chain(candidate.iter()) {
        if !value.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }

    if is_binary(baseline) && is_binary(candidate) {
        return mcnemar_outcome(baseline, candidate, alpha);
    }

    let differences: Vec<f64> = candidate
        .iter()
        .zip(baseline.iter())
        .map(|(c, b)| c - b)
        .collect();
    paired_t_test(&differences, alpha)
}

/// Exact-McNemar outcome with a normal-approximation CI on the paired difference
/// in proportions (`(b − c) / N`), where `b`/`c` are the discordant counts.
fn mcnemar_outcome(
    baseline: &[f64],
    candidate: &[f64],
    alpha: f64,
) -> Result<TestOutcome, StatsError> {
    let total = baseline.len();
    let mut b: u64 = 0; // baseline 0 -> candidate 1 (candidate improved)
    let mut c: u64 = 0; // baseline 1 -> candidate 0 (candidate regressed)
    for (base, cand) in baseline.iter().zip(candidate.iter()) {
        match (*base as i64, *cand as i64) {
            (0, 1) => b += 1,
            (1, 0) => c += 1,
            _ => {}
        }
    }
    let p_value = mcnemar_exact_p(b, c)?;
    let n = total as f64;
    let diff = (b as f64 - c as f64) / n;
    // Standard McNemar large-sample SE for the difference of paired proportions.
    let discordant = b as f64 + c as f64;
    let variance = (discordant - (b as f64 - c as f64).powi(2) / n) / (n * n);
    let ci = if variance <= 0.0 {
        ConfidenceInterval {
            low: diff,
            high: diff,
            confidence: 1.0 - alpha,
        }
    } else {
        let z = numerics::normal_quantile(1.0 - alpha / 2.0);
        let half = z * variance.sqrt();
        ConfidenceInterval {
            low: diff - half,
            high: diff + half,
            confidence: 1.0 - alpha,
        }
    };
    Ok(TestOutcome {
        estimate: diff,
        ci: Some(ci),
        p_value,
        test: TestKind::McnemarExact,
        df: None,
        sample_size: total,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Math helpers — normal CDF and quantile (no external deps)
// ─────────────────────────────────────────────────────────────────────────────

/// Standard normal CDF: Φ(x) = P(Z ≤ x).
///
/// Implemented using the complementary error function (erfc) approximation from
/// Abramowitz & Stegun §7.1.26 (max |ε| < 1.5×10⁻⁷ for all finite x).
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * erfc_approx(-x / core::f64::consts::SQRT_2)
}

/// Inverse normal CDF: Φ⁻¹(p) using the rational approximation from
/// Abramowitz & Stegun §26.2.17.  Accurate to about 4.5×10⁻⁴.
///
/// Returns `NaN` for inputs outside `(0, 1)`.
pub fn normal_quantile(p: f64) -> f64 {
    if !(0.0 < p && p < 1.0) {
        return f64::NAN;
    }

    let c0 = 2.515517_f64;
    let c1 = 0.802853_f64;
    let c2 = 0.010328_f64;
    let d1 = 1.432788_f64;
    let d2 = 0.189269_f64;
    let d3 = 0.001308_f64;

    // Halley iteration is not needed for the accuracy we claim.
    let (t, sign) = if p <= 0.5 {
        ((-2.0 * p.ln()).sqrt(), -1.0_f64)
    } else {
        ((-2.0 * (1.0 - p).ln()).sqrt(), 1.0_f64)
    };

    let numerator = c0 + c1 * t + c2 * t * t;
    let denominator = 1.0 + d1 * t + d2 * t * t + d3 * t * t * t;
    sign * (t - numerator / denominator)
}

// Abramowitz & Stegun §7.1.26 — erfc approximation (max |ε| < 1.5×10⁻⁷)
fn erfc_approx(x: f64) -> f64 {
    // Works for x ≥ 0; mirror for negative x
    let (x_abs, flip) = if x < 0.0 { (-x, true) } else { (x, false) };

    let t = 1.0 / (1.0 + 0.3275911 * x_abs);
    let poly = t * (0.254_829_592
        + t * (-0.284_496_736
            + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let erfc = poly * (-x_abs * x_abs).exp();

    if flip {
        2.0 - erfc
    } else {
        erfc
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tiny deterministic RNG — 64-bit xorshift (Marsaglia 2003)
// ─────────────────────────────────────────────────────────────────────────────

struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        // Avoid the forbidden all-zero state
        Self {
            state: if seed == 0 { 0xcafe_babe_dead_beef } else { seed },
        }
    }

    /// Returns the next pseudo-random `u64`.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Returns a `usize` in `[0, n)`.
    fn next_index(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal utilities
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Unbiased (n − 1) sample variance; 0.0 for fewer than two values.
pub(crate) fn sample_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let sum_sq: f64 = values.iter().map(|v| (v - m).powi(2)).sum();
    sum_sq / (values.len() as f64 - 1.0)
}

pub(crate) fn validate_alpha(alpha: f64) -> Result<(), StatsError> {
    if alpha.is_finite() && alpha > 0.0 && alpha < 1.0 {
        Ok(())
    } else {
        Err(StatsError::InvalidAlpha(alpha))
    }
}

fn is_binary(values: &[f64]) -> bool {
    values.iter().all(|v| *v == 0.0 || *v == 1.0)
}

fn resample_mean(xs: &[f64], rng: &mut Xorshift64) -> f64 {
    let n = xs.len();
    let mut sum = 0.0;
    for _ in 0..n {
        sum += xs[rng.next_index(n)];
    }
    sum / n as f64
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── compare_paired selector (§10.3) ───────────────────────────────────────

    #[test]
    fn rejects_invalid_alpha() {
        assert!(matches!(
            compare_paired(&[0.0, 1.0], &[1.0, 1.0], 0.0),
            Err(StatsError::InvalidAlpha(_))
        ));
        assert!(matches!(
            compare_paired(&[0.0, 1.0], &[1.0, 1.0], 1.0),
            Err(StatsError::InvalidAlpha(_))
        ));
    }

    #[test]
    fn rejects_mismatched_lengths() {
        assert!(matches!(
            compare_paired(&[0.0, 1.0, 1.0], &[1.0, 1.0], 0.05),
            Err(StatsError::MismatchedLengths { .. })
        ));
    }

    #[test]
    fn selects_mcnemar_for_binary() {
        // Candidate flips three failures to successes, regresses none.
        let baseline = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let candidate = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let out = compare_paired(&baseline, &candidate, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, TestKind::McnemarExact);
        // delta = (b - c)/N = (3 - 0)/6 = 0.5
        assert!((out.estimate - 0.5).abs() < 1e-9);
        // b=3, c=0 -> exact two-sided p = 2 * 0.5^3 = 0.25
        assert!((out.p_value - 0.25).abs() < 1e-9, "p={}", out.p_value);
    }

    #[test]
    fn selects_paired_t_for_continuous() {
        let baseline = [0.50, 0.55, 0.48, 0.52, 0.51];
        let candidate = [0.60, 0.62, 0.59, 0.61, 0.63];
        let out = compare_paired(&baseline, &candidate, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, TestKind::PairedT);
        assert!(out.estimate > 0.0);
        assert!(out.ci.is_some());
        // A clear, consistent improvement should be significant.
        assert!(out.p_value < 0.05, "p={}", out.p_value);
    }

    #[test]
    fn identical_samples_are_not_significant() {
        let data = [0.3, 0.7, 0.5, 0.9, 0.1];
        let out = compare_paired(&data, &data, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!((out.estimate).abs() < 1e-12);
        assert!((out.p_value - 1.0).abs() < 1e-9);
    }

    // ── normal_cdf / normal_quantile ─────────────────────────────────────────

    #[test]
    fn normal_cdf_known_values() {
        // Φ(0) = 0.5
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        // Φ(1.96) ≈ 0.97500
        assert!((normal_cdf(1.96) - 0.97500).abs() < 1e-4);
        // Φ(−1.96) ≈ 0.02500
        assert!((normal_cdf(-1.96) - 0.02500).abs() < 1e-4);
        // Φ(2.576) ≈ 0.99500
        assert!((normal_cdf(2.576) - 0.99500).abs() < 1e-4);
        // symmetry
        assert!((normal_cdf(1.0) + normal_cdf(-1.0) - 1.0).abs() < 1e-7);
    }

    #[test]
    fn normal_quantile_round_trips() {
        for &p in &[0.025, 0.05, 0.5, 0.95, 0.975, 0.995] {
            let q = normal_quantile(p);
            let back = normal_cdf(q);
            assert!(
                (back - p).abs() < 1e-4,
                "round-trip failed for p={p}: quantile={q}, cdf(quantile)={back}"
            );
        }
    }

    #[test]
    fn z95_constant_matches_quantile() {
        // Our baked-in constant should match the computed quantile
        assert!((Z_95 - normal_quantile(0.975)).abs() < 0.001);
        assert!((Z_99 - normal_quantile(0.995)).abs() < 0.001);
    }

    // ── Wilson score interval ─────────────────────────────────────────────────

    /// §6.3 canonical test: 8 successes out of 10 trials at 95 % confidence.
    /// Reference values from Wilson (1927) / statsmodels proportion_confint.
    #[test]
    fn wilson_8_of_10_95pct() {
        let ci = wilson_interval(8, 10, Z_95).unwrap_or_else(|err| panic!("{err}"));
        // Reference: lower ≈ 0.4904, upper ≈ 0.9432
        assert!((ci.lower - 0.4904).abs() < 1e-3, "lower = {}", ci.lower);
        assert!((ci.upper - 0.9432).abs() < 1e-3, "upper = {}", ci.upper);
        assert!(ci.lower < ci.center && ci.center < ci.upper);
    }

    #[test]
    fn wilson_all_success() {
        // k = n = 10 — p̂ = 1.0; interval should be < 1.0 (shrinkage towards 0.5)
        let ci = wilson_interval(10, 10, Z_95).unwrap_or_else(|err| panic!("{err}"));
        assert!(ci.lower > 0.0, "lower = {}", ci.lower);
        assert!((ci.upper - 1.0).abs() < 1e-9, "upper = {}", ci.upper);
        assert!(ci.center < 1.0);
    }

    #[test]
    fn wilson_zero_successes() {
        // k = 0 — p̂ = 0.0; interval should be > 0.0 (shrinkage towards 0.5)
        let ci = wilson_interval(0, 10, Z_95).unwrap_or_else(|err| panic!("{err}"));
        assert!((ci.lower - 0.0).abs() < 1e-9, "lower = {}", ci.lower);
        assert!(ci.upper > 0.0, "upper = {}", ci.upper);
        assert!(ci.center > 0.0);
    }

    #[test]
    fn wilson_single_trial_success() {
        let ci = wilson_interval(1, 1, Z_95).unwrap_or_else(|err| panic!("{err}"));
        assert!(ci.lower >= 0.0);
        assert!(ci.upper <= 1.0);
    }

    #[test]
    fn wilson_large_n() {
        // At large n Wilson and Wald converge — sanity-check for 900/1000
        let ci = wilson_interval(900, 1000, Z_95).unwrap_or_else(|err| panic!("{err}"));
        let p_hat = 0.9_f64;
        let wald_half = Z_95 * (p_hat * (1.0 - p_hat) / 1000.0_f64).sqrt();
        assert!((ci.center - p_hat).abs() < 0.01);
        assert!((ci.upper - ci.lower - 2.0 * wald_half).abs() < 0.01);
    }

    #[test]
    fn wilson_error_zero_trials() {
        assert_eq!(wilson_interval(0, 0, Z_95), Err(StatsError::ZeroTrials));
    }

    #[test]
    fn wilson_error_successes_exceed_trials() {
        assert_eq!(
            wilson_interval(5, 3, Z_95),
            Err(StatsError::SuccessesExceedTrials {
                successes: 5,
                trials: 3
            })
        );
    }

    #[test]
    fn wilson_error_bad_z() {
        assert_eq!(
            wilson_interval(5, 10, 0.0),
            Err(StatsError::InvalidParameter { name: "z", value: 0.0 })
        );
        assert!(wilson_interval(5, 10, f64::NAN).is_err());
    }

    /// Validate the half-width formula from §10.3 #7 directly.
    #[test]
    fn wilson_formula_spot_check() {
        let k = 3_u64;
        let n = 20_u64;
        let z = Z_95;
        let p_hat = k as f64 / n as f64; // 0.15
        let z2 = z * z;
        let expected_center = (p_hat + z2 / (2.0 * n as f64)) / (1.0 + z2 / n as f64);
        let expected_half = (z / (1.0 + z2 / n as f64))
            * (p_hat * (1.0 - p_hat) / n as f64 + z2 / (4.0 * (n as f64).powi(2))).sqrt();

        let ci = wilson_interval(k, n, z).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            (ci.center - expected_center).abs() < 1e-12,
            "center mismatch: {} vs {}",
            ci.center,
            expected_center
        );
        assert!(
            (ci.lower - (expected_center - expected_half).max(0.0)).abs() < 1e-12,
            "lower mismatch"
        );
        assert!(
            (ci.upper - (expected_center + expected_half).min(1.0)).abs() < 1e-12,
            "upper mismatch"
        );
    }

    // ── Two-proportion z-test ─────────────────────────────────────────────────

    /// Hand-computed reference: 85/100 vs 70/100.
    /// p_pool = 155/200 = 0.775
    /// SE = sqrt(0.775 * 0.225 * (1/100 + 1/100)) = sqrt(0.0034875) ≈ 0.05905
    /// z = (0.85 - 0.70) / 0.05905 ≈ 2.540
    /// p-val = 2 * Φ(-2.540) ≈ 0.0111
    #[test]
    fn two_prop_significant_at_95pct() {
        let res = two_proportion_z_test(85, 100, 70, 100).unwrap_or_else(|err| panic!("{err}"));
        assert!((res.z_stat - 2.540).abs() < 0.01, "z = {}", res.z_stat);
        assert!((res.p_value - 0.0111).abs() < 0.001, "p = {}", res.p_value);
        assert!(res.p_value < 0.05);
    }

    /// Identical proportions → z = 0, p = 1.
    #[test]
    fn two_prop_identical_proportions() {
        let res = two_proportion_z_test(50, 100, 50, 100).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(res.z_stat, 0.0);
        assert!((res.p_value - 1.0).abs() < 1e-9);
    }

    /// Both all-success or all-fail → SE = 0 → z = 0, p = 1.
    #[test]
    fn two_prop_all_success_se_zero() {
        let res = two_proportion_z_test(10, 10, 10, 10).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(res.z_stat, 0.0);
        assert!((res.p_value - 1.0).abs() < 1e-9);
    }

    /// Candidate all-pass, baseline all-fail → maximum discriminability.
    #[test]
    fn two_prop_perfect_separation() {
        // p1 = 1.0, p2 = 0.0
        // p_pool = 5/10 = 0.5
        // SE = sqrt(0.5 * 0.5 * (1/5 + 1/5)) = sqrt(0.25 * 0.4) = sqrt(0.1) ≈ 0.3162
        // z = (1.0 - 0.0) / 0.3162 ≈ 3.162
        let res = two_proportion_z_test(5, 5, 0, 5).unwrap_or_else(|err| panic!("{err}"));
        assert!((res.z_stat - 3.162).abs() < 0.01, "z = {}", res.z_stat);
        assert!(res.p_value < 0.01);
    }

    #[test]
    fn two_prop_direction() {
        // Candidate worse than baseline → negative z
        let res = two_proportion_z_test(40, 100, 60, 100).unwrap_or_else(|err| panic!("{err}"));
        assert!(res.z_stat < 0.0);
    }

    #[test]
    fn two_prop_error_zero_trials() {
        assert_eq!(
            two_proportion_z_test(0, 0, 5, 10),
            Err(StatsError::ZeroTrials)
        );
        assert_eq!(
            two_proportion_z_test(5, 10, 0, 0),
            Err(StatsError::ZeroTrials)
        );
    }

    #[test]
    fn two_prop_error_successes_exceed() {
        assert!(two_proportion_z_test(11, 10, 5, 10).is_err());
        assert!(two_proportion_z_test(5, 10, 11, 10).is_err());
    }

    // ── Bootstrap CI ─────────────────────────────────────────────────────────

    /// Determinism: same seed → identical CI.
    #[test]
    fn bootstrap_deterministic_with_seed() {
        let a = vec![0.9, 0.8, 0.85, 0.88, 0.92];
        let b = vec![0.6, 0.65, 0.7, 0.62, 0.68];
        let ci1 = bootstrap_diff_ci(&a, &b, 0.95, 1000, 42).unwrap_or_else(|err| panic!("{err}"));
        let ci2 = bootstrap_diff_ci(&a, &b, 0.95, 1000, 42).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(ci1.lower, ci2.lower);
        assert_eq!(ci1.upper, ci2.upper);
    }

    /// Different seeds → (almost certainly) different CIs.
    #[test]
    fn bootstrap_different_seeds_differ() {
        let a: Vec<f64> = (0..50).map(|i| 0.5 + (i as f64 * 0.01) % 0.4).collect();
        let b: Vec<f64> = (0..50).map(|i| 0.3 + (i as f64 * 0.01) % 0.3).collect();
        let ci1 = bootstrap_diff_ci(&a, &b, 0.95, 500, 1).unwrap_or_else(|err| panic!("{err}"));
        let ci2 = bootstrap_diff_ci(&a, &b, 0.95, 500, 99999).unwrap_or_else(|err| panic!("{err}"));
        // They should differ (probability of collision is negligible)
        assert!(
            (ci1.lower - ci2.lower).abs() > 1e-12
                || (ci1.upper - ci2.upper).abs() > 1e-12,
            "CIs suspiciously identical across seeds"
        );
    }

    /// Clearly better sample → CI entirely positive.
    #[test]
    fn bootstrap_positive_diff_ci() {
        let a = vec![0.9, 0.85, 0.88, 0.92, 0.95];
        let b = vec![0.5, 0.55, 0.45, 0.52, 0.48];
        let ci = bootstrap_diff_ci(&a, &b, 0.95, 5000, 7).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            ci.lower > 0.0,
            "lower = {} — expected entirely positive diff CI",
            ci.lower
        );
        assert!(ci.estimate > 0.0);
    }

    /// Clearly worse sample → CI entirely negative.
    #[test]
    fn bootstrap_negative_diff_ci() {
        let a = vec![0.5, 0.55, 0.45, 0.52, 0.48];
        let b = vec![0.9, 0.85, 0.88, 0.92, 0.95];
        let ci = bootstrap_diff_ci(&a, &b, 0.95, 5000, 7).unwrap_or_else(|err| panic!("{err}"));
        assert!(ci.upper < 0.0, "upper = {} — expected entirely negative", ci.upper);
        assert!(ci.estimate < 0.0);
    }

    /// Edge case: single-element samples.
    #[test]
    fn bootstrap_single_element_samples() {
        let a = vec![0.8];
        let b = vec![0.6];
        let ci = bootstrap_diff_ci(&a, &b, 0.95, 100, 1).unwrap_or_else(|err| panic!("{err}"));
        // With one-element samples every resample has the same mean → point interval
        assert!((ci.lower - 0.2).abs() < 1e-9);
        assert!((ci.upper - 0.2).abs() < 1e-9);
        assert!((ci.estimate - 0.2).abs() < 1e-9);
    }

    /// Estimate field is correct (observed mean difference).
    #[test]
    fn bootstrap_estimate_is_observed_diff() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![0.5, 1.5, 2.5];
        let ci = bootstrap_diff_ci(&a, &b, 0.95, 1000, 0).unwrap_or_else(|err| panic!("{err}"));
        // mean(a) = 2.0, mean(b) = 1.5 → estimate = 0.5
        assert!((ci.estimate - 0.5).abs() < 1e-12);
    }

    #[test]
    fn bootstrap_error_empty_sample() {
        assert_eq!(
            bootstrap_diff_ci(&[], &[1.0], 0.95, 100, 1),
            Err(StatsError::EmptySample)
        );
        assert_eq!(
            bootstrap_diff_ci(&[1.0], &[], 0.95, 100, 1),
            Err(StatsError::EmptySample)
        );
    }

    #[test]
    fn bootstrap_error_bad_confidence() {
        assert!(bootstrap_diff_ci(&[1.0], &[1.0], 0.0, 100, 1).is_err());
        assert!(bootstrap_diff_ci(&[1.0], &[1.0], 1.0, 100, 1).is_err());
        assert!(bootstrap_diff_ci(&[1.0], &[1.0], -0.5, 100, 1).is_err());
    }

    #[test]
    fn bootstrap_error_zero_resamples() {
        assert_eq!(
            bootstrap_diff_ci(&[1.0], &[1.0], 0.95, 0, 1),
            Err(StatsError::InvalidResampleCount(0))
        );
    }

    // ── §21.4 generalization-gap scenario ────────────────────────────────────

    /// Simulate the §21.4 held-out gap check: train score CI vs test score CI.
    /// A large enough train-test gap with CI-low > 0 should be flagged.
    #[test]
    fn generalization_gap_detected() {
        // "train" scores: inflated by overfitting
        let train: Vec<f64> = vec![0.95; 20];
        // "test" (held-out) scores: lower
        let test: Vec<f64> = vec![0.60; 20];

        let ci = bootstrap_diff_ci(&train, &test, 0.95, 2000, 123).unwrap_or_else(|err| panic!("{err}"));
        // gap CI-low > 0 → overfitting signal should trip the guardrail
        assert!(
            ci.lower > 0.0,
            "CI-low = {} — gap should be clearly positive (overfitting detected)",
            ci.lower
        );
    }
}
