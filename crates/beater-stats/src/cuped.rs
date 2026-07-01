//! CUPED variance reduction (Deng et al. 2013, "Improving the Sensitivity of
//! Online Controlled Experiments by Utilizing Pre-Experiment Data") — §10.3 #4 /
//! #436 item 4.
//!
//! CUPED reduces the variance of a metric's mean estimate by regressing out a
//! **pre-experiment covariate** `X` that is correlated with the metric `Y` but
//! independent of the treatment assignment:
//!
//! ```text
//! Y_cuped[i] = Y[i] − θ·(X[i] − mean(X)),   θ = Cov(X, Y) / Var(X)
//! ```
//!
//! Because `mean(X[i] − mean(X)) = 0`, the adjusted metric has the **same sample
//! mean** as `Y` (the point estimate is unbiased), while its variance is reduced
//! by a factor of `1 − ρ²`, where `ρ = corr(X, Y)`. A tighter variance means a
//! narrower confidence interval for the same sample size — i.e. more power per
//! eval dollar, the whole point of #436.
//!
//! ## Choosing the covariate
//!
//! The covariate MUST be a genuine *pre-experiment* quantity that does not
//! mechanically contain the treatment. A per-case difficulty score measured on a
//! prior period is the canonical choice. **Do not** pass the paired baseline
//! score as the covariate for a paired *difference* `Y = candidate − baseline`:
//! `Y` mechanically contains `baseline`, so `θ → −1` un-pairs the design instead
//! of reducing noise. This module computes the adjustment; picking a valid
//! covariate is the caller's responsibility.

use crate::{mean, sample_variance, StatsError};

/// The result of a CUPED adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct CupedOutcome {
    /// The regression coefficient `θ = Cov(X, Y) / Var(X)` (0 when the covariate
    /// has no spread).
    pub theta: f64,
    /// `mean(X)`, the covariate centering constant.
    pub covariate_mean: f64,
    /// The variance-reduced metric values, in input order. Same length and same
    /// sample mean as the input metric.
    pub adjusted: Vec<f64>,
    /// The fraction of variance actually removed: `1 − Var(Y_cuped) / Var(Y)`,
    /// clamped to `[0, 1]`. Asymptotically equals `ρ²`. `0` when the metric has
    /// no spread or the covariate is uninformative.
    pub variance_reduction: f64,
}

/// Compute the CUPED-adjusted `metric` using a pre-experiment `covariate`.
///
/// Both slices must be the same length `n ≥ 2` and all-finite. When the
/// covariate has zero variance the adjustment is a no-op (`θ = 0`,
/// `adjusted = metric`, `variance_reduction = 0`) rather than an error, so a
/// caller can pass a covariate unconditionally.
pub fn cuped_adjust(metric: &[f64], covariate: &[f64]) -> Result<CupedOutcome, StatsError> {
    let n = metric.len();
    if n != covariate.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: n,
            candidate: covariate.len(),
        });
    }
    if n < 2 {
        return Err(StatsError::TooFewSamples { got: n, need: 2 });
    }
    for value in metric.iter().chain(covariate.iter()) {
        if !value.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }

    let metric_mean = mean(metric);
    let covariate_mean = mean(covariate);

    // Cov(X, Y) and Var(X) share the `n − 1` denominator, which cancels in θ, so
    // accumulate the cross-product and covariate sum-of-squares directly.
    let mut cross = 0.0_f64;
    let mut cov_ss = 0.0_f64;
    for (y, x) in metric.iter().zip(covariate.iter()) {
        let dx = x - covariate_mean;
        cross += (y - metric_mean) * dx;
        cov_ss += dx * dx;
    }

    // No covariate spread ⇒ no information to regress out ⇒ identity adjustment.
    if cov_ss == 0.0 {
        return Ok(CupedOutcome {
            theta: 0.0,
            covariate_mean,
            adjusted: metric.to_vec(),
            variance_reduction: 0.0,
        });
    }

    let theta = cross / cov_ss;
    let adjusted: Vec<f64> = metric
        .iter()
        .zip(covariate.iter())
        .map(|(y, x)| y - theta * (x - covariate_mean))
        .collect();

    let metric_var = sample_variance(metric);
    let variance_reduction = if metric_var > 0.0 {
        (1.0 - sample_variance(&adjusted) / metric_var).clamp(0.0, 1.0)
    } else {
        0.0
    };

    Ok(CupedOutcome {
        theta,
        covariate_mean,
        adjusted,
        variance_reduction,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatched_lengths_error() {
        assert!(matches!(
            cuped_adjust(&[1.0, 2.0], &[1.0]),
            Err(StatsError::MismatchedLengths { .. })
        ));
    }

    #[test]
    fn too_few_samples_error() {
        assert!(matches!(
            cuped_adjust(&[1.0], &[1.0]),
            Err(StatsError::TooFewSamples { got: 1, need: 2 })
        ));
    }

    #[test]
    fn non_finite_error() {
        assert!(matches!(
            cuped_adjust(&[1.0, f64::NAN], &[1.0, 2.0]),
            Err(StatsError::NonFinite)
        ));
    }

    #[test]
    fn preserves_the_sample_mean_exactly() {
        let metric = [0.3, 0.7, 0.5, 0.9, 0.1, 0.6];
        let covariate = [0.2, 0.8, 0.4, 0.9, 0.0, 0.7];
        let out = cuped_adjust(&metric, &covariate).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            (mean(&out.adjusted) - mean(&metric)).abs() < 1e-12,
            "CUPED must be unbiased: {} vs {}",
            mean(&out.adjusted),
            mean(&metric)
        );
    }

    #[test]
    fn correlated_covariate_reduces_variance() {
        // Metric strongly correlated with the covariate (Y ≈ X + small noise):
        // CUPED should remove most of the variance.
        let covariate = [0.1, 0.9, 0.3, 0.7, 0.5, 0.2, 0.8, 0.4];
        let metric: Vec<f64> = covariate
            .iter()
            .enumerate()
            .map(|(i, x)| x + if i % 2 == 0 { 0.02 } else { -0.02 })
            .collect();
        let out = cuped_adjust(&metric, &covariate).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            out.variance_reduction > 0.8,
            "expected large variance reduction, got {}",
            out.variance_reduction
        );
        assert!(
            sample_variance(&out.adjusted) < sample_variance(&metric),
            "adjusted variance must be smaller"
        );
        assert!(
            (out.theta - 1.0).abs() < 0.15,
            "theta≈1 for Y≈X, got {}",
            out.theta
        );
    }

    #[test]
    fn uncorrelated_covariate_barely_changes_variance() {
        // Covariate uncorrelated with the metric ⇒ near-zero reduction.
        let metric = [0.5, 0.1, 0.9, 0.2, 0.8, 0.4];
        let covariate = [0.4, 0.4, 0.4, 0.41, 0.4, 0.4];
        let out = cuped_adjust(&metric, &covariate).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            out.variance_reduction < 0.2,
            "uncorrelated covariate should not reduce much, got {}",
            out.variance_reduction
        );
    }

    #[test]
    fn zero_variance_covariate_is_a_noop() {
        let metric = [0.5, 0.1, 0.9, 0.2];
        let covariate = [0.4, 0.4, 0.4, 0.4];
        let out = cuped_adjust(&metric, &covariate).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.theta, 0.0);
        assert_eq!(out.variance_reduction, 0.0);
        assert_eq!(out.adjusted, metric);
    }
}
