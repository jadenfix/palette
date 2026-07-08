//! Anytime-valid confidence sequence for a mean (§10.3 #6 / roadmap PR-A4) —
//! the *interval* companion to the one-sided e-value test in
//! [`crate::SequentialMeanTest`].
//!
//! A continuously-peeked online stream cannot use a fixed-horizon interval: the
//! more often you look, the more often a nominal-95% interval excludes the truth
//! somewhere along the way. A **confidence sequence** fixes this: a sequence of
//! intervals `CS_n` with the *time-uniform* guarantee
//!
//! ```text
//! P( ∀n: μ ∈ CS_n ) ≥ 1 − α,
//! ```
//!
//! so a gate may recompute the interval after every observation and act on it at
//! any data-dependent stopping time without inflating its error rate.
//!
//! ## Construction (Robbins' normal mixture)
//!
//! For observations whose deviations from the true mean are `σ`-sub-Gaussian
//! (bounded scores in `[a, b]` are `((b−a)/2)`-sub-Gaussian), the mixture of the
//! sub-Gaussian e-process `exp(λS_n − nλ²σ²/2)` over `λ ~ N(0, τ²)` has the
//! closed form
//!
//! ```text
//! E_n(μ₀) = (1 + nσ²τ²)^{−1/2} · exp( S_n(μ₀)² τ² / (2(1 + nσ²τ²)) )
//! ```
//!
//! with `S_n(μ₀) = Σ(x_i − μ₀)`; it is a non-negative supermartingale under
//! `μ = μ₀` with `E_0 = 1`, so Ville's inequality bounds the whole trajectory at
//! once. Inverting `E_n(μ₀) < 1/α` in `μ₀` gives the interval
//!
//! ```text
//! CS_n = mean_n ± √( 2σ²(1 + nσ²τ²)/(n²τ²) · ln( √(1 + nσ²τ²) / α ) )
//! ```
//!
//! (Robbins 1970; Howard et al. 2021 §3.2). Any fixed `τ > 0` chosen *before*
//! the data is valid; `τ` only tunes at which sample size the sequence is
//! tightest. A natural pre-registered choice is `τ = e/σ²` where `e` is the
//! planned minimum detectable effect — the same scale the likelihood-ratio-
//! optimal fixed bet [`crate::recommended_lambda`] uses.

use crate::{ConfidenceInterval, StatsError, mean};

/// Outcome of an anytime-valid mean estimate over a peeked stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfidenceSequenceOutcome {
    /// The running sample mean at the current `n`.
    pub estimate: f64,
    /// The confidence-sequence interval at the current `n` — valid
    /// *simultaneously* over all peek times, so acting on it at any stopping
    /// time keeps the nominal error rate.
    pub ci: ConfidenceInterval,
    /// Anytime-valid two-sided p-value for `H₀: μ = mu0`, from the running
    /// supremum of the mixture e-process (`min(1, 1/sup_{s≤n} E_s)`).
    pub p_value: f64,
    /// Number of observations folded in.
    pub sample_size: usize,
}

/// Compute the Robbins normal-mixture confidence sequence for the mean of
/// `samples` at the current sample size, plus the anytime-valid p-value against
/// `H₀: μ = mu0`.
///
/// * `sigma` — the sub-Gaussian scale of a single observation (for scores
///   bounded in `[a, b]`, use `(b − a) / 2`). Must be finite and `> 0`.
/// * `tau` — the mixture scale, chosen **before** the data (pre-registered);
///   must be finite and `> 0`. Larger `τ` tightens the sequence at small `n`,
///   smaller `τ` at large `n`; validity does not depend on the choice.
/// * `alpha` — the time-uniform error budget in `(0, 1)`.
///
/// The p-value scans every prefix (the e-process' running supremum), so it is
/// exactly the p-value a peek-every-step analyst is entitled to.
///
/// # Errors
///
/// [`StatsError::EmptySample`], [`StatsError::NonFinite`] (samples or `mu0`),
/// [`StatsError::InvalidParameter`] for a bad `sigma`/`tau`, or
/// [`StatsError::InvalidAlpha`].
pub fn confidence_sequence_mean(
    samples: &[f64],
    mu0: f64,
    sigma: f64,
    tau: f64,
    alpha: f64,
) -> Result<ConfidenceSequenceOutcome, StatsError> {
    crate::validate_alpha(alpha)?;
    if samples.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if !mu0.is_finite() {
        return Err(StatsError::NonFinite);
    }
    if samples.iter().any(|v| !v.is_finite()) {
        return Err(StatsError::NonFinite);
    }
    if !sigma.is_finite() || sigma <= 0.0 {
        return Err(StatsError::InvalidParameter {
            name: "sigma",
            value: sigma,
        });
    }
    if !tau.is_finite() || tau <= 0.0 {
        return Err(StatsError::InvalidParameter {
            name: "tau",
            value: tau,
        });
    }

    let sig2 = sigma * sigma;
    let tau2 = tau * tau;

    // Running supremum of ln E_s(mu0) over every prefix s ≤ n: the evidence a
    // peek-every-step analyst has actually accumulated against H0.
    let mut running_sum = 0.0_f64;
    let mut sup_log_e = 0.0_f64; // ln E_0 = 0
    for (i, x) in samples.iter().enumerate() {
        running_sum += x - mu0;
        let s = (i + 1) as f64;
        let log_e = log_mixture_evalue(running_sum, s, sig2, tau2);
        sup_log_e = sup_log_e.max(log_e);
    }
    let p_value = (-sup_log_e).exp().min(1.0);

    let n = samples.len() as f64;
    let estimate = mean(samples);
    let radius = cs_radius(n, sig2, tau2, alpha);

    Ok(ConfidenceSequenceOutcome {
        estimate,
        ci: ConfidenceInterval {
            low: estimate - radius,
            high: estimate + radius,
            confidence: 1.0 - alpha,
        },
        p_value,
        sample_size: samples.len(),
    })
}

/// `ln E_n(μ₀)` for the normal-mixture e-process with centered sum `s_n`.
fn log_mixture_evalue(s_n: f64, n: f64, sig2: f64, tau2: f64) -> f64 {
    let denom = 1.0 + n * sig2 * tau2;
    s_n * s_n * tau2 / (2.0 * denom) - 0.5 * denom.ln()
}

/// The confidence-sequence half-width at sample size `n`.
fn cs_radius(n: f64, sig2: f64, tau2: f64, alpha: f64) -> f64 {
    let denom = 1.0 + n * sig2 * tau2;
    ((2.0 * denom / (n * n * tau2)) * (denom.sqrt() / alpha).ln()).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Xorshift64;
    use crate::numerics::normal_quantile;

    #[test]
    fn validates_inputs() {
        assert!(matches!(
            confidence_sequence_mean(&[], 0.0, 0.5, 1.0, 0.05),
            Err(StatsError::EmptySample)
        ));
        assert!(matches!(
            confidence_sequence_mean(&[0.1, f64::NAN], 0.0, 0.5, 1.0, 0.05),
            Err(StatsError::NonFinite)
        ));
        assert!(matches!(
            confidence_sequence_mean(&[0.1], 0.0, 0.0, 1.0, 0.05),
            Err(StatsError::InvalidParameter { name: "sigma", .. })
        ));
        assert!(matches!(
            confidence_sequence_mean(&[0.1], 0.0, 0.5, -1.0, 0.05),
            Err(StatsError::InvalidParameter { name: "tau", .. })
        ));
        assert!(matches!(
            confidence_sequence_mean(&[0.1], 0.0, 0.5, 1.0, 1.0),
            Err(StatsError::InvalidAlpha(_))
        ));
    }

    /// Pin the closed-form radius and e-value against a hand-computed case:
    /// n = 4, σ = 0.5, τ = 2, α = 0.05, samples summing to S = 1.2 above μ₀ = 0.
    #[test]
    fn closed_form_matches_hand_computation() {
        let samples = [0.3, 0.3, 0.3, 0.3];
        let out = confidence_sequence_mean(&samples, 0.0, 0.5, 2.0, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        // denom = 1 + n σ² τ² = 1 + 4·0.25·4 = 5.
        // radius = sqrt( (2·5 / (16·4)) · ln(√5 / 0.05) ) = sqrt(0.15625·ln(44.72...)).
        let expected_radius = (0.156_25_f64 * (5.0_f64.sqrt() / 0.05).ln()).sqrt();
        assert!(
            ((out.ci.high - out.estimate) - expected_radius).abs() < 1e-12,
            "radius {} vs {expected_radius}",
            out.ci.high - out.estimate
        );
        // Final-step ln E = S²τ²/(2·denom) − ln(√denom) = 1.44·4/10 − ln(√5).
        let expected_log_e = 1.44 * 4.0 / 10.0 - 5.0_f64.sqrt().ln();
        // All prefixes have smaller |S| here (monotone), so sup is the last step
        // unless an earlier prefix beats it — verify against a direct scan.
        let mut sup: f64 = 0.0;
        let mut s = 0.0;
        for (i, x) in samples.iter().enumerate() {
            s += x;
            let n = (i + 1) as f64;
            let denom = 1.0 + n * 0.25 * 4.0;
            sup = sup.max(s * s * 4.0 / (2.0 * denom) - 0.5 * denom.ln());
        }
        assert!((sup - expected_log_e).abs() < 1e-12 || sup > expected_log_e);
        assert!((out.p_value - (-sup).exp().min(1.0)).abs() < 1e-12);
    }

    /// The load-bearing guarantee: TIME-UNIFORM coverage. Simulate a null
    /// stream and check that the fraction of runs where the sequence EVER
    /// excludes the true mean (peeking at every step) stays ≤ α — the property
    /// a fixed-horizon interval catastrophically fails under peeking.
    #[test]
    fn time_uniform_coverage_under_continuous_peeking() {
        let (trials, steps) = (2000usize, 400usize);
        let (sigma, tau, alpha, mu) = (0.5_f64, 1.0_f64, 0.05_f64, 0.5_f64);
        let mut rng = Xorshift64::new(0xC0FF_EE00);
        let mut ever_missed = 0usize;
        let mut fixed_ever_missed = 0usize;
        let z = normal_quantile(1.0 - alpha / 2.0);
        for _ in 0..trials {
            let mut sum = 0.0_f64;
            let mut missed = false;
            let mut fixed_missed = false;
            for step in 1..=steps {
                // Bernoulli(0.5) in {0,1}: mean 0.5, 0.5-sub-Gaussian.
                let x = if rng.next_u64() & 1 == 1 { 1.0 } else { 0.0 };
                sum += x;
                let n = step as f64;
                let m = sum / n;
                let r = cs_radius(n, sigma * sigma, tau * tau, alpha);
                if (m - mu).abs() > r {
                    missed = true;
                }
                // The naive fixed-horizon interval recomputed at every peek.
                let fixed_r = z * sigma / n.sqrt();
                if (m - mu).abs() > fixed_r {
                    fixed_missed = true;
                }
            }
            if missed {
                ever_missed += 1;
            }
            if fixed_missed {
                fixed_ever_missed += 1;
            }
        }
        let miss_rate = ever_missed as f64 / trials as f64;
        let fixed_miss_rate = fixed_ever_missed as f64 / trials as f64;
        assert!(
            miss_rate <= alpha + 0.01,
            "time-uniform miss rate {miss_rate} must be ≤ α = {alpha}"
        );
        // And demonstrate the hazard the CS exists to fix: the naive interval
        // peeked at every step misses far more often than α.
        assert!(
            fixed_miss_rate > 3.0 * alpha,
            "fixed-horizon peeking should badly over-miss, got {fixed_miss_rate}"
        );
    }

    /// A genuine lift is eventually detected: the p-value crosses α and the
    /// interval excludes the null mean.
    #[test]
    fn detects_a_real_lift() {
        // Constant lift of 0.2 over μ₀ = 0 (σ = 0.5 scale).
        let samples: Vec<f64> = vec![0.2; 200];
        let out = confidence_sequence_mean(&samples, 0.0, 0.5, 0.8, 0.05)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(out.p_value < 0.05, "p = {}", out.p_value);
        assert!(out.ci.low > 0.0, "CI must exclude 0: {:?}", out.ci);
        assert!((out.estimate - 0.2).abs() < 1e-12);
    }

    /// The interval width shrinks roughly like √(ln n / n).
    #[test]
    fn radius_shrinks_with_n() {
        let r10 = cs_radius(10.0, 0.25, 1.0, 0.05);
        let r1000 = cs_radius(1000.0, 0.25, 1.0, 0.05);
        assert!(r1000 < r10 / 5.0, "r10={r10}, r1000={r1000}");
    }
}
