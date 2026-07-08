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

use crate::numerics::{students_t_quantile, students_t_two_sided_p};
use crate::{ConfidenceInterval, StatsError, TestKind, TestOutcome, mean, sample_variance};

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

    // No usable covariate spread ⇒ no information to regress out ⇒ identity
    // adjustment. Guard at the floating-point noise floor *relative to the
    // covariate's own scale* (not exact `== 0.0`): a covariate that varies only at
    // the ULP level would otherwise divide through a near-zero `cov_ss` and blow θ
    // up into a huge, meaningless slope that overfits the metric. Genuine (if
    // small) spread still passes through and is handled honestly by the n−2 df.
    let covariate_scale: f64 = covariate.iter().map(|x| x * x).sum();
    if cov_ss <= f64::EPSILON * covariate_scale {
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

/// The two-sided **regression-estimator** t-test for the mean paired difference,
/// using a pre-experiment `covariate` with a *known* population mean to reduce
/// variance (CUPED — §10.3 #4 / #436 item 4).
///
/// `differences[i] = candidate_i − baseline_i`, `covariate[i]` is the
/// pre-experiment covariate `X_i` for the same case, and
/// `population_covariate_mean` (`μ_x`) is the covariate's **known** mean over the
/// whole case population (e.g. the difficulty averaged over the entire dataset).
/// The estimator is
///
/// ```text
/// μ̂_reg = mean(d) − θ · (x̄ − μ_x),   θ = Cov(x, d) / Var(x)
/// ```
///
/// which is an unbiased estimator of `E[d]` with variance reduced by `1 − ρ²`.
///
/// # Why `μ_x` must be *known*, not the sample mean
///
/// For a **one-sample** mean, centering on the sample covariate mean `x̄` is
/// degenerate: `μ̂_reg` collapses back to `mean(d)`, yet the residual-variance CI
/// is narrower than the true sampling variance of `mean(d)`, so it **under-covers**
/// the population mean (a Monte-Carlo check in this module measures ~74% coverage
/// for a nominal 95% interval — an inflated false-pass rate). Centering on a
/// *known* `μ_x` instead lets the estimate move by `−θ(x̄ − μ_x)`, which is exactly
/// the term that makes the residual-variance CI valid again (~95% coverage). So a
/// caller MUST pass a genuinely known `μ_x`; passing `x̄` reproduces the
/// unadjusted mean with an invalid (too-narrow) interval.
///
/// # Degrees of freedom
///
/// `θ` is estimated from the data, so the test uses `df = n − 2` and the residual
/// mean square `SS_resid/(n − 2)`; `n ≥ 3` is required. A degenerate/uninformative
/// covariate (`θ ≈ 0`) gracefully reduces to the unadjusted mean.
pub fn cuped_paired_t_test(
    differences: &[f64],
    covariate: &[f64],
    population_covariate_mean: f64,
    alpha: f64,
) -> Result<TestOutcome, StatsError> {
    if !(alpha.is_finite() && alpha > 0.0 && alpha < 1.0) {
        return Err(StatsError::InvalidAlpha(alpha));
    }
    if !population_covariate_mean.is_finite() {
        return Err(StatsError::NonFinite);
    }
    let n = differences.len();
    if n < 3 {
        return Err(StatsError::TooFewSamples { got: n, need: 3 });
    }
    // `cuped_adjust` validates equal lengths, finiteness, and `n ≥ 2`, returns the
    // fitted slope `θ`, the sample covariate mean `x̄`, and the sample-centered
    // adjusted series (whose deviations from their mean are the OLS residuals —
    // invariant to the centering constant).
    let outcome = cuped_adjust(differences, covariate)?;

    // Regression estimator: re-centre on the KNOWN population covariate mean so the
    // point estimate is an unbiased, lower-variance estimator of E[d]. When the
    // covariate is uninformative `θ = 0` and this is just `mean(differences)`.
    let estimate =
        mean(differences) - outcome.theta * (outcome.covariate_mean - population_covariate_mean);

    let df = n as f64 - 2.0; // one df for the mean, one for the estimated θ.
    // Residual mean square with the n−2 denominator (not the n−1 sample variance),
    // so the SE and the `df = n−2` t-quantile are consistent and the interval has
    // exact nominal coverage under normality (nominal α = actual α). SS_resid is
    // invariant to the centering constant, so the sample-centered `adjusted` series
    // carries it.
    let residual_ss = sample_variance(&outcome.adjusted) * (n as f64 - 1.0);
    let residual_mean_square = residual_ss / df;
    // SE of the OLS mean-response evaluated at the KNOWN covariate value μ_x — NOT
    // simply σ̂²/n. Because μ̂ extrapolates from the sample covariate mean x̄ out to
    // μ_x, its variance carries the textbook leverage term σ̂²·(x̄−μ_x)²/S_xx on top
    // of σ̂²/n. Dropping it under-covers E[d] whenever μ_x differs from x̄ by more
    // than sampling noise — the exact regime the gate permits (a known population
    // mean need not equal this sample's mean). `cuped_adjust` computes the same
    // S_xx internally; recompute here (O(n)) to keep its public API unchanged. When
    // the covariate has no spread θ = 0, so μ̂ = mean(d) and the leverage term is 0.
    let covariate_ss: f64 = covariate
        .iter()
        .map(|x| (x - outcome.covariate_mean).powi(2))
        .sum();
    let leverage = if covariate_ss > 0.0 {
        (outcome.covariate_mean - population_covariate_mean).powi(2) / covariate_ss
    } else {
        0.0
    };
    let standard_error = (residual_mean_square * (1.0 / n as f64 + leverage)).sqrt();

    let (p_value, ci) = if standard_error == 0.0 {
        // No residual spread: degenerate but well-defined (a perfectly consistent
        // adjusted shift), mirroring `paired_t_test`.
        let p = if estimate == 0.0 { 1.0 } else { 0.0 };
        (
            p,
            ConfidenceInterval {
                low: estimate,
                high: estimate,
                confidence: 1.0 - alpha,
            },
        )
    } else {
        let t = estimate / standard_error;
        let p = students_t_two_sided_p(t, df);
        let crit = students_t_quantile(1.0 - alpha / 2.0, df);
        let half = crit * standard_error;
        (
            p,
            ConfidenceInterval {
                low: estimate - half,
                high: estimate + half,
                confidence: 1.0 - alpha,
            },
        )
    };

    Ok(TestOutcome {
        estimate,
        ci: Some(ci),
        p_value,
        // The adjusted metric is continuous, so this is a paired-t result; the
        // caller records that CUPED was applied via the pre-registered design.
        test: TestKind::PairedT,
        df: Some(df),
        sample_size: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Xorshift64;
    use crate::paired_t_test;

    /// Standard normal via Box-Muller on a deterministic xorshift stream.
    fn gaussian(rng: &mut Xorshift64) -> f64 {
        let u1 = (rng.next_u64() as f64 / u64::MAX as f64).max(1e-12);
        let u2 = rng.next_u64() as f64 / u64::MAX as f64;
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// The load-bearing statistical guarantee: the regression-estimator CI has
    /// **valid nominal coverage of the population mean** `E[d]` — the invariant
    /// that makes the gate's α honest. A seeded Monte-Carlo draws cases from a
    /// population where the covariate genuinely explains most of the variance, and
    /// checks that the nominal-95% interval covers the true mean ~95% of the time,
    /// while being materially narrower than the unadjusted paired-t (the power win).
    ///
    /// This is what caught the original bug: centering on the *sample* mean instead
    /// of the known population mean under-covers (~74%). It is checked across a
    /// range of `n` because the `θ`-estimation cost is an `n`-dependent effect.
    #[test]
    fn regression_estimator_has_valid_population_coverage_and_narrows_the_ci() {
        // Model: x_i ~ N(0,1) (sample mean x̄ ≈ 0); ε_i ~ N(0,σ²);
        // d_i = μ + β·x_i + ε_i. The regression estimator targets the mean response
        // a + b·μ_x = μ + β·μ_x at the KNOWN covariate value μ_x, and its interval
        // must cover that target at the nominal rate.
        //
        // Two regimes per n:
        //   A. μ_x = 0 — μ_x coincides with x̄, so the target is the marginal E[d]=μ
        //      and CUPED is directly comparable to the plain paired-t (coverage AND
        //      width). This is the "on-center" case the original test covered.
        //   B. μ_x = 0.7 — μ_x is offset from x̄≈0, so the estimator EXTRAPOLATES.
        //      Coverage here is only valid if the SE carries the leverage term
        //      σ̂²·(x̄−μ_x)²/S_xx; the old σ̂²/n SE under-covers (~0.88) and this
        //      assertion is what guards that fix. (Regime B is not comparable to the
        //      plain paired-t, which estimates the marginal mean μ, not μ+β·μ_x.)
        let (mu, beta, sigma, trials) = (0.3_f64, 1.5_f64, 1.0_f64, 6000usize);
        for &n in &[10usize, 25, 60] {
            let mut rng =
                Xorshift64::new(0x00C0_FFEE_1234_5678 ^ (n as u64).wrapping_mul(0x9E37_79B1));
            for (regime, &mu_x) in [0.0_f64, 0.7].iter().enumerate() {
                let on_center = regime == 0;
                let target = mu + beta * mu_x; // the mean response the CI must cover.
                let (mut cuped_cover, mut plain_cover) = (0usize, 0usize);
                let (mut cuped_w, mut plain_w) = (0.0_f64, 0.0_f64);
                for _ in 0..trials {
                    let (mut d, mut x) = (Vec::with_capacity(n), Vec::with_capacity(n));
                    for _ in 0..n {
                        let xi = gaussian(&mut rng);
                        x.push(xi);
                        d.push(mu + beta * xi + gaussian(&mut rng) * sigma);
                    }
                    let cuped =
                        cuped_paired_t_test(&d, &x, mu_x, 0.05).unwrap_or_else(|e| panic!("{e}"));
                    let cc = cuped.ci.unwrap_or_else(|| panic!("ci"));
                    if cc.low <= target && target <= cc.high {
                        cuped_cover += 1;
                    }
                    cuped_w += cc.high - cc.low;

                    // The plain paired-t estimates the marginal mean μ; only compare
                    // it on-center, where the target IS μ.
                    if on_center {
                        let plain = paired_t_test(&d, 0.05).unwrap_or_else(|e| panic!("{e}"));
                        let pc = plain.ci.unwrap_or_else(|| panic!("ci"));
                        if pc.low <= target && target <= pc.high {
                            plain_cover += 1;
                        }
                        plain_w += pc.high - pc.low;
                    }
                }
                let cuped_rate = cuped_cover as f64 / trials as f64;
                let cuped_mw = cuped_w / trials as f64;
                println!("n={n} μ_x={mu_x}: CUPED coverage={cuped_rate:.3} width={cuped_mw:.3}");
                // Nominal α = actual α in BOTH regimes: coverage within a few
                // Monte-Carlo SEs of 0.95, and NOT anti-conservative. Regime B is the
                // one the old (leverage-free) SE failed (~0.88).
                assert!(
                    (0.93..=0.965).contains(&cuped_rate),
                    "n={n} μ_x={mu_x}: CUPED must have ~95% coverage, got {cuped_rate}"
                );
                if on_center {
                    let plain_rate = plain_cover as f64 / trials as f64;
                    let plain_mw = plain_w / trials as f64;
                    println!(
                        "n={n} μ_x={mu_x}: plain coverage={plain_rate:.3} width={plain_mw:.3}"
                    );
                    // The whole point of CUPED: a materially narrower interval (β·x
                    // explains ~69% of Var(d) here, so ~40%+ width reduction).
                    assert!(
                        cuped_mw < 0.8 * plain_mw,
                        "n={n}: CUPED must narrow the CI, got {cuped_mw} vs {plain_mw}"
                    );
                }
            }
        }
    }

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

    #[test]
    fn paired_t_needs_at_least_three_samples() {
        assert!(matches!(
            cuped_paired_t_test(&[0.1, 0.2], &[0.3, 0.4], 0.35, 0.05),
            Err(StatsError::TooFewSamples { got: 2, need: 3 })
        ));
    }

    #[test]
    fn paired_t_rejects_bad_alpha_and_nonfinite_mean() {
        assert!(matches!(
            cuped_paired_t_test(&[0.1, 0.2, 0.3], &[0.1, 0.2, 0.3], 0.2, 0.0),
            Err(StatsError::InvalidAlpha(_))
        ));
        assert!(matches!(
            cuped_paired_t_test(&[0.1, 0.2, 0.3], &[0.1, 0.2, 0.3], f64::NAN, 0.05),
            Err(StatsError::NonFinite)
        ));
    }

    #[test]
    fn estimate_is_the_regression_estimator_closed_form() {
        let differences = [0.10, 0.30, -0.05, 0.22, 0.14, 0.08, 0.19, 0.01];
        let covariate = [0.9, 0.1, 0.7, 0.2, 0.5, 0.6, 0.3, 0.8];
        let adj = cuped_adjust(&differences, &covariate).unwrap_or_else(|err| panic!("{err}"));
        let mu_x = 0.4_f64; // a KNOWN population mean, deliberately ≠ sample mean.

        let out = cuped_paired_t_test(&differences, &covariate, mu_x, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        // μ̂_reg = mean(d) − θ(x̄ − μ_x): the estimate MOVES off the plain mean.
        let expected = mean(&differences) - adj.theta * (adj.covariate_mean - mu_x);
        assert!(
            (out.estimate - expected).abs() < 1e-12,
            "regression-estimator formula: {} vs {expected}",
            out.estimate
        );
        assert_eq!(out.df, Some(differences.len() as f64 - 2.0));

        // Centering on the SAMPLE mean is the degenerate case: the estimate
        // collapses back to the plain mean (documented as the invalid choice).
        let degenerate = cuped_paired_t_test(&differences, &covariate, adj.covariate_mean, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!((degenerate.estimate - mean(&differences)).abs() < 1e-12);
    }

    #[test]
    fn correlated_covariate_tightens_the_interval() {
        // Differences strongly driven by the covariate: CUPED shrinks the CI versus
        // the unadjusted paired-t, even paying the extra degree of freedom. (The CI
        // width does not depend on μ_x, so any known value exercises the width.)
        let covariate = [0.1, 0.9, 0.3, 0.7, 0.5, 0.2, 0.8, 0.4, 0.6, 0.0];
        let differences: Vec<f64> = covariate
            .iter()
            .enumerate()
            .map(|(i, x)| 0.1 + 0.4 * x + if i % 2 == 0 { 0.01 } else { -0.01 })
            .collect();
        let cuped = cuped_paired_t_test(&differences, &covariate, 0.5, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        let plain = paired_t_test(&differences, 0.05).unwrap_or_else(|err| panic!("{err}"));
        let cuped_ci = cuped.ci.unwrap_or_else(|| panic!("expected ci"));
        let plain_ci = plain.ci.unwrap_or_else(|| panic!("expected ci"));
        assert!(
            (cuped_ci.high - cuped_ci.low) < (plain_ci.high - plain_ci.low),
            "CUPED CI width {} should be < plain CI width {}",
            cuped_ci.high - cuped_ci.low,
            plain_ci.high - plain_ci.low
        );
    }

    #[test]
    fn uninformative_covariate_barely_changes_the_interval() {
        // A near-constant covariate carries almost no information, so the CUPED
        // interval is essentially the unadjusted one (widened only by the n−2 df).
        let differences = [0.10, 0.30, -0.05, 0.22, 0.14, 0.08, 0.19, 0.01];
        let covariate = [0.4, 0.4, 0.4, 0.41, 0.4, 0.4, 0.4, 0.4];
        let cuped = cuped_paired_t_test(&differences, &covariate, 0.4, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        let plain = paired_t_test(&differences, 0.05).unwrap_or_else(|err| panic!("{err}"));
        let cuped_ci = cuped.ci.unwrap_or_else(|| panic!("expected ci"));
        let plain_ci = plain.ci.unwrap_or_else(|| panic!("expected ci"));
        let cuped_w = cuped_ci.high - cuped_ci.low;
        let plain_w = plain_ci.high - plain_ci.low;
        assert!(
            (cuped_w - plain_w).abs() < 0.2 * plain_w,
            "uninformative covariate should barely move the CI: {cuped_w} vs {plain_w}"
        );
    }
}
