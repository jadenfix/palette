//! Always-valid sequential inference via test martingales / e-values
//! (§10.3 #6 / #436 item 1).
//!
//! A fixed-horizon test is only valid at its pre-registered sample size: peeking
//! at the data and stopping when it looks significant inflates the false-positive
//! rate. A **test martingale** (equivalently, an *e-process*) fixes this. For a
//! one-sided mean hypothesis `H0: μ ≤ μ0` against `H1: μ > μ0`, with observations
//! that are `σ`-sub-Gaussian (bounded scores in `[a, b]` are `((b−a)/2)`-sub-
//! Gaussian), the process
//!
//! ```text
//! E_n = exp( λ · Σ_{i=1}^n (x_i − μ0)  −  n · λ² σ² / 2 ),   λ > 0
//! ```
//!
//! is a non-negative supermartingale under `H0` with `E_0 = 1` (each increment's
//! moment generating function is bounded by `exp(λ²σ²/2)` by sub-Gaussianity).
//! **Ville's inequality** then gives, for the entire (unbounded) sequence at once,
//!
//! ```text
//! P_{H0}( ∃ n :  E_n ≥ 1/α )  ≤  α.
//! ```
//!
//! So the rule "reject `H0` the moment `E_n ≥ 1/α`" controls the type-I error at
//! `α` **no matter how often you peek or when you stop** — you may run the eval
//! case-by-case and stop as soon as the evidence crosses the bound, spending only
//! the samples you need. `E_n` is also an *e-value*: its reciprocal is an
//! anytime-valid p-value, and e-values combine across experiments by multiplication.
//!
//! `λ` must be chosen in advance (not from the data) to keep the martingale valid;
//! [`recommended_lambda`] returns the mixture-free choice that is
//! likelihood-ratio-optimal for a target lift.

use crate::StatsError;

/// The state of a running one-sided sequential mean test.
///
/// Feed observations with [`observe`](SequentialMeanTest::observe); query
/// [`e_value`](SequentialMeanTest::e_value) or
/// [`reject`](SequentialMeanTest::reject) after any number of them. Cheap and
/// streaming — it keeps only a running log-e-value and a count, so peeking is free.
#[derive(Debug, Clone)]
pub struct SequentialMeanTest {
    mu0: f64,
    lambda: f64,
    /// The per-observation variance penalty `λ² σ² / 2`.
    penalty: f64,
    /// `ln E_n`, accumulated for numerical stability.
    log_e: f64,
    n: usize,
}

impl SequentialMeanTest {
    /// Start a test of `H0: μ ≤ mu0` vs `H1: μ > mu0` for `sigma`-sub-Gaussian
    /// observations, using betting parameter `lambda`.
    ///
    /// `lambda` must be finite and `> 0` and chosen *before* seeing data;
    /// `sigma` (the sub-Gaussian scale) must be finite and `> 0`; `mu0` must be
    /// finite.
    pub fn new(mu0: f64, lambda: f64, sigma: f64) -> Result<Self, StatsError> {
        if !mu0.is_finite() {
            return Err(StatsError::NonFinite);
        }
        if !lambda.is_finite() || lambda <= 0.0 {
            return Err(StatsError::InvalidParameter {
                name: "lambda",
                value: lambda,
            });
        }
        if !sigma.is_finite() || sigma <= 0.0 {
            return Err(StatsError::InvalidParameter {
                name: "sigma",
                value: sigma,
            });
        }
        Ok(Self {
            mu0,
            lambda,
            penalty: lambda * lambda * sigma * sigma / 2.0,
            log_e: 0.0,
            n: 0,
        })
    }

    /// Fold one observation into the running e-value. Non-finite inputs are
    /// rejected as an error rather than silently corrupting the martingale.
    pub fn observe(&mut self, x: f64) -> Result<(), StatsError> {
        if !x.is_finite() {
            return Err(StatsError::NonFinite);
        }
        // ln E_n += λ (x − μ0) − λ²σ²/2.
        self.log_e += self.lambda * (x - self.mu0) - self.penalty;
        self.n += 1;
        Ok(())
    }

    /// Number of observations folded in so far.
    pub fn n(&self) -> usize {
        self.n
    }

    /// The current e-value `E_n = exp(ln E_n)`. Starts at `1.0` (no evidence).
    pub fn e_value(&self) -> f64 {
        self.log_e.exp()
    }

    /// Whether `H0` can be rejected at level `alpha` right now, i.e. `E_n ≥ 1/α`.
    /// Valid to call after every observation (that is the whole point).
    pub fn reject(&self, alpha: f64) -> Result<bool, StatsError> {
        if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return Err(StatsError::InvalidAlpha(alpha));
        }
        // Compare in log space to avoid overflow: ln E_n ≥ ln(1/α) = −ln α.
        Ok(self.log_e >= -alpha.ln())
    }

    /// The anytime-valid p-value `min(1, 1/E_n)`: valid to report at any stopping
    /// time, unlike a fixed-horizon p-value.
    pub fn anytime_p_value(&self) -> f64 {
        (-self.log_e).exp().min(1.0)
    }
}

/// Batch convenience: fold a whole slice through a [`SequentialMeanTest`] and
/// return the final e-value. Equivalent to `observe`-ing each element in order.
pub fn evalue_one_sided_mean(
    samples: &[f64],
    mu0: f64,
    lambda: f64,
    sigma: f64,
) -> Result<f64, StatsError> {
    let mut test = SequentialMeanTest::new(mu0, lambda, sigma)?;
    for &x in samples {
        test.observe(x)?;
    }
    Ok(test.e_value())
}

/// The likelihood-ratio-optimal betting parameter `λ = target_lift / σ²` for
/// detecting a mean lift of `target_lift` above `mu0` in `sigma`-sub-Gaussian
/// data. This is the (mixture-free) `λ` that maximizes the expected log growth of
/// the e-value under the alternative, so the test stops fastest when the true
/// lift is near `target_lift`. Must be chosen before seeing data.
///
/// `target_lift` must be finite and `> 0`; `sigma` must be finite and `> 0`.
pub fn recommended_lambda(target_lift: f64, sigma: f64) -> Result<f64, StatsError> {
    if !target_lift.is_finite() || target_lift <= 0.0 {
        return Err(StatsError::InvalidParameter {
            name: "target_lift",
            value: target_lift,
        });
    }
    if !sigma.is_finite() || sigma <= 0.0 {
        return Err(StatsError::InvalidParameter {
            name: "sigma",
            value: sigma,
        });
    }
    Ok(target_lift / (sigma * sigma))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_parameters() {
        assert!(matches!(
            SequentialMeanTest::new(0.0, 0.0, 0.5),
            Err(StatsError::InvalidParameter { name: "lambda", .. })
        ));
        assert!(matches!(
            SequentialMeanTest::new(0.0, 0.5, 0.0),
            Err(StatsError::InvalidParameter { name: "sigma", .. })
        ));
        assert!(matches!(
            recommended_lambda(-1.0, 0.5),
            Err(StatsError::InvalidParameter {
                name: "target_lift",
                ..
            })
        ));
    }

    #[test]
    fn starts_at_unit_e_value() {
        let test = SequentialMeanTest::new(0.5, 1.0, 0.5).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(test.e_value(), 1.0);
        assert_eq!(test.n(), 0);
        assert_eq!(test.anytime_p_value(), 1.0);
    }

    #[test]
    fn benign_data_at_the_null_never_rejects() {
        // Every observation exactly at μ0: each increment is −λ²σ²/2 < 0, so the
        // e-value only shrinks. A peeking analyst can never be tricked into a
        // false reject on truly-null data.
        let mut test = SequentialMeanTest::new(0.5, 1.0, 0.5).unwrap_or_else(|err| panic!("{err}"));
        for _ in 0..500 {
            test.observe(0.5).unwrap_or_else(|err| panic!("{err}"));
            assert!(
                !test.reject(0.05).unwrap_or_else(|err| panic!("{err}")),
                "null data must never cross the boundary"
            );
        }
        assert!(test.e_value() < 1.0);
        assert!(test.anytime_p_value() >= 1.0 - 1e-9);
    }

    #[test]
    fn strong_alternative_rejects_and_stops_early() {
        // A real lift (μ ≈ 0.9 vs μ0 = 0.5): the e-value grows and crosses 1/α.
        let lambda = recommended_lambda(0.4, 0.5).unwrap_or_else(|err| panic!("{err}"));
        let mut test =
            SequentialMeanTest::new(0.5, lambda, 0.5).unwrap_or_else(|err| panic!("{err}"));
        let mut stopped_at = None;
        for i in 0..200 {
            test.observe(0.9).unwrap_or_else(|err| panic!("{err}"));
            if test.reject(0.05).unwrap_or_else(|err| panic!("{err}")) {
                stopped_at = Some(i + 1);
                break;
            }
        }
        let n = stopped_at.unwrap_or_else(|| panic!("a strong lift must eventually reject"));
        assert!(
            n < 100,
            "should stop early under a strong lift, stopped at {n}"
        );
        assert!(test.e_value() >= 1.0 / 0.05);
        assert!(test.anytime_p_value() <= 0.05);
    }

    #[test]
    fn one_sided_ignores_a_negative_effect() {
        // Data well BELOW μ0: this one-sided test for μ > μ0 must not reject.
        let mut test = SequentialMeanTest::new(0.5, 1.0, 0.5).unwrap_or_else(|err| panic!("{err}"));
        for _ in 0..300 {
            test.observe(0.1).unwrap_or_else(|err| panic!("{err}"));
        }
        assert!(!test.reject(0.05).unwrap_or_else(|err| panic!("{err}")));
        assert!(test.e_value() < 1.0);
    }

    #[test]
    fn batch_helper_matches_streaming() {
        let samples = [0.7, 0.8, 0.6, 0.9, 0.75];
        let batch =
            evalue_one_sided_mean(&samples, 0.5, 1.0, 0.5).unwrap_or_else(|err| panic!("{err}"));
        let mut test = SequentialMeanTest::new(0.5, 1.0, 0.5).unwrap_or_else(|err| panic!("{err}"));
        for &x in &samples {
            test.observe(x).unwrap_or_else(|err| panic!("{err}"));
        }
        assert!((batch - test.e_value()).abs() < 1e-12);
    }

    #[test]
    fn more_positive_evidence_grows_the_e_value() {
        let weak = evalue_one_sided_mean(&[0.55, 0.55, 0.55], 0.5, 1.0, 0.5)
            .unwrap_or_else(|err| panic!("{err}"));
        let strong = evalue_one_sided_mean(&[0.95, 0.95, 0.95], 0.5, 1.0, 0.5)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(strong > weak);
    }
}
