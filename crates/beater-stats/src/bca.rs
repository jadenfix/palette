//! Bias-corrected and accelerated (BCa) bootstrap (ARCHITECTURE.md §10.3 #2) and
//! the small-N **paired bootstrap** test.
//!
//! The plain percentile bootstrap ([`crate::bootstrap_diff_ci`]) under-covers for
//! **skewed** statistics (cost, latency): its endpoints are shifted because it
//! ignores the bias and skew of the bootstrap distribution. BCa adjusts the
//! percentile endpoints by a **bias correction** `z₀` (how far the median of the
//! replicates sits from the observed estimate) and an **acceleration** `a`
//! (the jackknife skewness of the statistic), restoring ~nominal coverage.

use crate::numerics::normal_quantile;
use crate::{mean, normal_cdf, BootstrapInterval, ConfidenceInterval, StatsError, Xorshift64};

/// Outcome of a paired bootstrap test over per-pair differences.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PairedBootstrapOutcome {
    /// Observed mean difference.
    pub estimate: f64,
    /// Percentile bootstrap CI for the mean difference.
    pub ci: ConfidenceInterval,
    /// Two-sided bootstrap achieved significance level for `H₀: mean diff = 0`.
    pub p_value: f64,
    /// Number of bootstrap resamples used.
    pub n_resamples: usize,
}

/// **BCa** confidence interval for the difference of means `mean(a) − mean(b)`.
///
/// # Algorithm
///
/// 1. Bootstrap replicates `θ*` (resample each sample independently, with
///    replacement), as in the percentile bootstrap.
/// 2. Bias correction `z₀ = Φ⁻¹( #{θ* < θ̂} / B )`.
/// 3. Acceleration `a` from the jackknife over the pooled observations:
///    `a = Σ(θ̄ − θ₍ᵢ₎)³ / ( 6 · (Σ(θ̄ − θ₍ᵢ₎)²)^{3/2} )`.
/// 4. Adjusted percentiles
///    `α₁ = Φ( z₀ + (z₀ + z_{α/2}) / (1 − a(z₀ + z_{α/2})) )` and the symmetric
///    `α₂`, read off the sorted replicates.
///
/// When the bootstrap distribution is degenerate (all replicates equal) or the
/// jackknife variance is zero, the adjustment is undefined; the function falls
/// back to the plain percentile endpoints (and BCa then equals the percentile
/// interval). This keeps the routine total — it never panics or returns `NaN`.
///
/// # Errors
///
/// Same validation as [`crate::bootstrap_diff_ci`], plus
/// [`StatsError::TooFewSamples`] when either sample has fewer than two
/// observations (the jackknife needs a leave-one-out).
pub fn bootstrap_bca_ci(
    sample_a: &[f64],
    sample_b: &[f64],
    confidence: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<BootstrapInterval, StatsError> {
    if sample_a.is_empty() || sample_b.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if sample_a.len() < 2 || sample_b.len() < 2 {
        return Err(StatsError::TooFewSamples {
            got: sample_a.len().min(sample_b.len()),
            need: 2,
        });
    }
    if sample_a
        .iter()
        .chain(sample_b.iter())
        .any(|v| !v.is_finite())
    {
        return Err(StatsError::NonFinite);
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

    let theta_hat = mean(sample_a) - mean(sample_b);

    let mut rng = Xorshift64::new(seed);
    let mut replicates: Vec<f64> = Vec::with_capacity(n_resamples);
    let mut n_below = 0usize;
    for _ in 0..n_resamples {
        let ra = resample_mean(sample_a, &mut rng);
        let rb = resample_mean(sample_b, &mut rng);
        let theta = ra - rb;
        if theta < theta_hat {
            n_below += 1;
        }
        replicates.push(theta);
    }
    replicates.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let percentile = |q: f64| -> f64 {
        let idx = (q * n_resamples as f64).floor() as isize;
        let idx = idx.clamp(0, n_resamples as isize - 1) as usize;
        replicates[idx]
    };

    // Fall back to the percentile interval if the BCa adjustment is undefined.
    let percentile_lo = percentile(alpha / 2.0);
    let percentile_hi = percentile(1.0 - alpha / 2.0);

    let frac_below = n_below as f64 / n_resamples as f64;
    let z0 = if frac_below <= 0.0 || frac_below >= 1.0 {
        // Degenerate bias correction — use the percentile interval.
        return Ok(BootstrapInterval {
            lower: percentile_lo,
            upper: percentile_hi,
            estimate: theta_hat,
            n_resamples,
        });
    } else {
        normal_quantile(frac_below)
    };

    let accel = jackknife_acceleration(sample_a, sample_b);
    let (lower, upper) = match accel {
        Some(a) => {
            let z_lo = normal_quantile(alpha / 2.0);
            let z_hi = normal_quantile(1.0 - alpha / 2.0);
            let adjust = |z: f64| -> f64 {
                let denom = 1.0 - a * (z0 + z);
                if denom == 0.0 || !denom.is_finite() {
                    return f64::NAN;
                }
                normal_cdf(z0 + (z0 + z) / denom)
            };
            let a1 = adjust(z_lo);
            let a2 = adjust(z_hi);
            if a1.is_finite() && a2.is_finite() {
                (percentile(a1), percentile(a2))
            } else {
                (percentile_lo, percentile_hi)
            }
        }
        None => (percentile_lo, percentile_hi),
    };

    Ok(BootstrapInterval {
        lower,
        upper,
        estimate: theta_hat,
        n_resamples,
    })
}

/// Jackknife acceleration `a` for the difference of means, over the pooled
/// observations (leave-one-out from each sample). `None` when the jackknife
/// pseudo-values have zero spread (acceleration undefined).
fn jackknife_acceleration(sample_a: &[f64], sample_b: &[f64]) -> Option<f64> {
    let mean_b = mean(sample_b);
    let mean_a = mean(sample_a);
    let mut theta_jack: Vec<f64> = Vec::with_capacity(sample_a.len() + sample_b.len());

    let sum_a: f64 = sample_a.iter().sum();
    for &x in sample_a {
        let m = (sum_a - x) / (sample_a.len() as f64 - 1.0);
        theta_jack.push(m - mean_b);
    }
    let sum_b: f64 = sample_b.iter().sum();
    for &x in sample_b {
        let m = (sum_b - x) / (sample_b.len() as f64 - 1.0);
        theta_jack.push(mean_a - m);
    }

    let theta_bar = mean(&theta_jack);
    let mut s2 = 0.0;
    let mut s3 = 0.0;
    for t in &theta_jack {
        let d = theta_bar - t;
        s2 += d * d;
        s3 += d * d * d;
    }
    if s2 <= 0.0 {
        return None;
    }
    let denom = 6.0 * s2.powf(1.5);
    if denom == 0.0 || !denom.is_finite() {
        return None;
    }
    Some(s3 / denom)
}

/// Paired bootstrap test over per-pair `differences` — the §10.3 #3 fallback for
/// small N or unclear assumptions. Returns the observed mean difference, a
/// percentile CI, and a two-sided bootstrap achieved significance level (twice the
/// smaller tail mass on the wrong side of zero).
///
/// # Errors
///
/// * [`StatsError::InvalidAlpha`] when `alpha ∉ (0, 1)`.
/// * [`StatsError::TooFewSamples`] when fewer than two pairs are supplied.
/// * [`StatsError::NonFinite`] when any difference is NaN/inf.
/// * [`StatsError::InvalidResampleCount`] when `n_resamples == 0`.
pub fn paired_bootstrap_test(
    differences: &[f64],
    alpha: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<PairedBootstrapOutcome, StatsError> {
    crate::validate_alpha(alpha)?;
    let n = differences.len();
    if n < 2 {
        return Err(StatsError::TooFewSamples { got: n, need: 2 });
    }
    for d in differences {
        if !d.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }
    if n_resamples == 0 {
        return Err(StatsError::InvalidResampleCount(n_resamples));
    }

    let estimate = mean(differences);
    let mut rng = Xorshift64::new(seed);
    let mut means: Vec<f64> = Vec::with_capacity(n_resamples);
    let mut le_zero = 0usize;
    let mut ge_zero = 0usize;
    for _ in 0..n_resamples {
        let m = resample_mean(differences, &mut rng);
        if m <= 0.0 {
            le_zero += 1;
        }
        if m >= 0.0 {
            ge_zero += 1;
        }
        means.push(m);
    }
    means.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

    let lo_idx = ((alpha / 2.0) * n_resamples as f64).floor() as usize;
    let hi_idx = ((1.0 - alpha / 2.0) * n_resamples as f64).floor() as usize;
    let ci = ConfidenceInterval {
        low: means[lo_idx.min(n_resamples - 1)],
        high: means[hi_idx.min(n_resamples - 1)],
        confidence: 1.0 - alpha,
    };

    let tail = le_zero.min(ge_zero) as f64 / n_resamples as f64;
    let p_value = (2.0 * tail).min(1.0);

    Ok(PairedBootstrapOutcome {
        estimate,
        ci,
        p_value,
        n_resamples,
    })
}

/// Mean of a with-replacement resample of `xs`, drawing `xs.len()` draws.
fn resample_mean(xs: &[f64], rng: &mut Xorshift64) -> f64 {
    let n = xs.len();
    let mut sum = 0.0;
    for _ in 0..n {
        sum += xs[rng.next_index(n)];
    }
    sum / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bca_rejects_singleton_samples() {
        assert!(matches!(
            bootstrap_bca_ci(&[1.0], &[2.0, 3.0], 0.95, 1000, 1),
            Err(StatsError::TooFewSamples { .. })
        ));
    }

    #[test]
    fn bca_is_deterministic_and_brackets_estimate() {
        let a = [0.9, 0.8, 0.95, 0.85, 0.88, 0.92];
        let b = [0.6, 0.7, 0.65, 0.62, 0.68, 0.64];
        let first = bootstrap_bca_ci(&a, &b, 0.95, 5_000, 42).unwrap_or_else(|err| panic!("{err}"));
        let second =
            bootstrap_bca_ci(&a, &b, 0.95, 5_000, 42).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(first, second);
        assert!(first.lower <= first.estimate && first.estimate <= first.upper);
    }

    #[test]
    fn bca_differs_from_percentile_on_skewed_data() {
        // A strongly right-skewed "cost" sample vs a symmetric baseline. The
        // acceleration + bias correction should shift the BCa endpoints away from
        // the plain percentile endpoints.
        let a = [1.0, 1.1, 1.0, 1.2, 1.1, 1.0, 1.3, 1.1, 8.0, 9.0];
        let b = [0.5, 0.6, 0.55, 0.5, 0.6, 0.52, 0.58, 0.5, 0.6, 0.55];
        let bca = bootstrap_bca_ci(&a, &b, 0.95, 8_000, 11).unwrap_or_else(|err| panic!("{err}"));
        let pct =
            crate::bootstrap_diff_ci(&a, &b, 0.95, 8_000, 11).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(bca.estimate, pct.estimate, "same point estimate");
        let moved = (bca.lower - pct.lower).abs() > 1e-9 || (bca.upper - pct.upper).abs() > 1e-9;
        assert!(moved, "BCa endpoints should differ from percentile on skew");
    }

    #[test]
    fn bca_matches_percentile_on_symmetric_data() {
        // Near-symmetric, near-zero-skew data: BCa should be close to percentile.
        let a = [0.50, 0.52, 0.48, 0.51, 0.49, 0.53, 0.47, 0.50];
        let b = [0.40, 0.42, 0.38, 0.41, 0.39, 0.43, 0.37, 0.40];
        let bca = bootstrap_bca_ci(&a, &b, 0.95, 8_000, 5).unwrap_or_else(|err| panic!("{err}"));
        let pct =
            crate::bootstrap_diff_ci(&a, &b, 0.95, 8_000, 5).unwrap_or_else(|err| panic!("{err}"));
        assert!((bca.lower - pct.lower).abs() < 0.03);
        assert!((bca.upper - pct.upper).abs() < 0.03);
    }

    #[test]
    fn paired_bootstrap_detects_consistent_shift() {
        let diffs = [0.10, 0.12, 0.09, 0.11, 0.13, 0.08, 0.10, 0.12];
        let out =
            paired_bootstrap_test(&diffs, 0.05, 5_000, 3).unwrap_or_else(|err| panic!("{err}"));
        assert!(out.estimate > 0.0);
        assert!(out.ci.low > 0.0, "CI should exclude zero: {:?}", out.ci);
        assert!(out.p_value < 0.05, "p={}", out.p_value);
    }

    #[test]
    fn paired_bootstrap_null_is_not_significant() {
        let diffs = [0.1, -0.1, 0.05, -0.05, 0.2, -0.2, 0.0, 0.0];
        let out =
            paired_bootstrap_test(&diffs, 0.05, 5_000, 9).unwrap_or_else(|err| panic!("{err}"));
        assert!(out.p_value > 0.05, "p={}", out.p_value);
    }
}
