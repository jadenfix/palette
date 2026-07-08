//! Judge-calibration agreement statistics (§10.5): Cohen's kappa for a 2×2
//! human-vs-judge confusion table (with a bootstrap CI), the Brier proper
//! scoring rule, and the binned expected calibration error.
//!
//! These are the primitives `beater-calibration` reports. They live here so the
//! math is testable in isolation and so the report can carry *uncertainty* —
//! a point-estimate kappa over a small human-labelled sample is high-variance,
//! and reporting it bare invites over-reading.

use crate::{ConfidenceInterval, StatsError, Xorshift64, validate_alpha};

/// A 2×2 agreement (confusion) table between two binary raters — for judge
/// calibration, rater A is the human label and rater B the judge label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgreementCounts {
    /// Both raters said pass.
    pub both_pass: u64,
    /// Rater A (human) pass, rater B (judge) fail.
    pub a_pass_b_fail: u64,
    /// Rater A (human) fail, rater B (judge) pass.
    pub a_fail_b_pass: u64,
    /// Both raters said fail.
    pub both_fail: u64,
}

impl AgreementCounts {
    /// Total number of rated items.
    pub fn total(&self) -> u64 {
        self.both_pass + self.a_pass_b_fail + self.a_fail_b_pass + self.both_fail
    }

    /// Observed agreement `p_o` — the fraction of items the raters agree on.
    pub fn observed_agreement(&self) -> f64 {
        let n = self.total();
        if n == 0 {
            return 0.0;
        }
        (self.both_pass + self.both_fail) as f64 / n as f64
    }

    /// Chance-expected agreement `p_e` under independent raters with the
    /// observed marginals.
    pub fn expected_agreement(&self) -> f64 {
        let n = self.total() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let a_pass = (self.both_pass + self.a_pass_b_fail) as f64 / n;
        let a_fail = (self.a_fail_b_pass + self.both_fail) as f64 / n;
        let b_pass = (self.both_pass + self.a_fail_b_pass) as f64 / n;
        let b_fail = (self.a_pass_b_fail + self.both_fail) as f64 / n;
        a_pass * b_pass + a_fail * b_fail
    }
}

/// Cohen's kappa `(p_o − p_e) / (1 − p_e)` for a 2×2 agreement table.
///
/// Degenerate marginals (`p_e = 1`, both raters constant): kappa is `1.0` when
/// the raters also agree perfectly, else `0.0` — chance-corrected agreement is
/// undefined there, and this is the conservative convention `beater-calibration`
/// has always used.
///
/// # Errors
///
/// [`StatsError::EmptySample`] when the table is empty.
pub fn cohen_kappa_binary(counts: AgreementCounts) -> Result<f64, StatsError> {
    if counts.total() == 0 {
        return Err(StatsError::EmptySample);
    }
    let p_o = counts.observed_agreement();
    let p_e = counts.expected_agreement();
    let denominator = 1.0 - p_e;
    if denominator.abs() < f64::EPSILON {
        if (p_o - 1.0).abs() < f64::EPSILON {
            return Ok(1.0);
        }
        return Ok(0.0);
    }
    Ok((p_o - p_e) / denominator)
}

/// Percentile-bootstrap confidence interval for Cohen's kappa: resample the
/// `n` rated items with replacement from the observed 2×2 cell proportions
/// (a multinomial bootstrap) and read the `α/2` / `1 − α/2` percentiles of the
/// resampled kappas. Deterministic under `seed`.
///
/// The interval quantifies what a bare kappa hides: with the small
/// human-labelled samples typical of judge calibration, kappa's sampling
/// variability is large, and decisions keyed on a point estimate over-read it.
///
/// # Errors
///
/// [`StatsError::EmptySample`], [`StatsError::InvalidAlpha`], or
/// [`StatsError::InvalidResampleCount`] for `n_resamples == 0`.
pub fn cohen_kappa_ci(
    counts: AgreementCounts,
    alpha: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<ConfidenceInterval, StatsError> {
    validate_alpha(alpha)?;
    if counts.total() == 0 {
        return Err(StatsError::EmptySample);
    }
    if n_resamples == 0 {
        return Err(StatsError::InvalidResampleCount(n_resamples));
    }

    let n = counts.total();
    // Cell boundaries for inverse-CDF sampling of the multinomial.
    let c1 = counts.both_pass;
    let c2 = c1 + counts.a_pass_b_fail;
    let c3 = c2 + counts.a_fail_b_pass;

    let mut kappas: Vec<f64> = Vec::with_capacity(n_resamples);
    let mut rng = Xorshift64::new(seed);
    for _ in 0..n_resamples {
        let mut resampled = AgreementCounts {
            both_pass: 0,
            a_pass_b_fail: 0,
            a_fail_b_pass: 0,
            both_fail: 0,
        };
        for _ in 0..n {
            let draw = rng.next_index(n as usize) as u64;
            if draw < c1 {
                resampled.both_pass += 1;
            } else if draw < c2 {
                resampled.a_pass_b_fail += 1;
            } else if draw < c3 {
                resampled.a_fail_b_pass += 1;
            } else {
                resampled.both_fail += 1;
            }
        }
        kappas.push(cohen_kappa_binary(resampled)?);
    }
    kappas.sort_by(|a, b| a.total_cmp(b));
    let lo_idx = (((alpha / 2.0) * n_resamples as f64).floor() as usize).min(n_resamples - 1);
    let hi_idx = (((1.0 - alpha / 2.0) * n_resamples as f64).floor() as usize).min(n_resamples - 1);
    Ok(ConfidenceInterval {
        low: kappas[lo_idx],
        high: kappas[hi_idx],
        confidence: 1.0 - alpha,
    })
}

/// Brier score `mean((p_i − y_i)²)` — the proper scoring rule for probabilistic
/// judge scores against binary human outcomes (`y ∈ {0, 1}`). Lower is better;
/// a perfectly calibrated, perfectly sharp judge scores 0.
///
/// # Errors
///
/// [`StatsError::EmptySample`], [`StatsError::MismatchedLengths`],
/// [`StatsError::NonFinite`], or [`StatsError::InvalidParameter`] when a
/// prediction is outside `[0, 1]` or an outcome is not exactly 0 or 1.
pub fn brier_score(predictions: &[f64], outcomes: &[f64]) -> Result<f64, StatsError> {
    validate_probability_pairs(predictions, outcomes)?;
    let total: f64 = predictions
        .iter()
        .zip(outcomes.iter())
        .map(|(p, y)| (p - y) * (p - y))
        .sum();
    Ok(total / predictions.len() as f64)
}

/// Expected calibration error over `bin_count` equal-width probability bins:
/// `ECE = Σ_b (n_b / N) · |accuracy_b − mean_confidence_b|`. Predictions of
/// exactly `1.0` land in the top bin. Empty bins contribute nothing.
///
/// Reported as a point estimate only: binned ECE is a biased, bin-sensitive
/// functional, and a resampled interval around it inherits that bias — so the
/// honest presentation is the estimate plus its binning convention.
///
/// # Errors
///
/// Same validation as [`brier_score`], plus
/// [`StatsError::InvalidParameter`] for `bin_count == 0`.
pub fn expected_calibration_error(
    predictions: &[f64],
    outcomes: &[f64],
    bin_count: usize,
) -> Result<f64, StatsError> {
    validate_probability_pairs(predictions, outcomes)?;
    if bin_count == 0 {
        return Err(StatsError::InvalidParameter {
            name: "bin_count",
            value: 0.0,
        });
    }
    let mut confidence_sums = vec![0.0_f64; bin_count];
    let mut outcome_sums = vec![0.0_f64; bin_count];
    let mut counts = vec![0usize; bin_count];
    for (p, y) in predictions.iter().zip(outcomes.iter()) {
        let index = if *p >= 1.0 {
            bin_count - 1
        } else {
            (p * bin_count as f64).floor() as usize
        };
        confidence_sums[index] += p;
        outcome_sums[index] += y;
        counts[index] += 1;
    }
    let n = predictions.len() as f64;
    let mut ece = 0.0;
    for b in 0..bin_count {
        if counts[b] == 0 {
            continue;
        }
        let weight = counts[b] as f64 / n;
        let mean_confidence = confidence_sums[b] / counts[b] as f64;
        let accuracy = outcome_sums[b] / counts[b] as f64;
        ece += weight * (accuracy - mean_confidence).abs();
    }
    Ok(ece)
}

fn validate_probability_pairs(predictions: &[f64], outcomes: &[f64]) -> Result<(), StatsError> {
    if predictions.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if predictions.len() != outcomes.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: predictions.len(),
            candidate: outcomes.len(),
        });
    }
    for p in predictions {
        if !p.is_finite() {
            return Err(StatsError::NonFinite);
        }
        if !(0.0..=1.0).contains(p) {
            return Err(StatsError::InvalidParameter {
                name: "prediction",
                value: *p,
            });
        }
    }
    for y in outcomes {
        if *y != 0.0 && *y != 1.0 {
            return Err(StatsError::InvalidParameter {
                name: "outcome",
                value: *y,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(
        both_pass: u64,
        a_pass_b_fail: u64,
        a_fail_b_pass: u64,
        both_fail: u64,
    ) -> AgreementCounts {
        AgreementCounts {
            both_pass,
            a_pass_b_fail,
            a_fail_b_pass,
            both_fail,
        }
    }

    /// Textbook 2×2 kappa: the classic Cohen (1960)-style example
    /// [[20, 5], [10, 15]]: p_o = 0.7, p_e = 0.5·0.6 + 0.5·0.4 = 0.5, κ = 0.4.
    #[test]
    fn kappa_matches_textbook() {
        let k = cohen_kappa_binary(counts(20, 5, 10, 15)).unwrap_or_else(|err| panic!("{err}"));
        assert!((k - 0.4).abs() < 1e-12, "kappa = {k}");
    }

    #[test]
    fn kappa_degenerate_marginals() {
        // Both raters always pass: perfect but chance-degenerate agreement.
        let k = cohen_kappa_binary(counts(10, 0, 0, 0)).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(k, 1.0);
        // Rater A always passes, rater B always fails: p_e = 0... marginals are
        // constant-per-rater but opposite, p_e = 1·0 + 0·1 = 0, p_o = 0 → κ = 0.
        let k = cohen_kappa_binary(counts(0, 10, 0, 0)).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(k, 0.0);
        assert!(matches!(
            cohen_kappa_binary(counts(0, 0, 0, 0)),
            Err(StatsError::EmptySample)
        ));
    }

    /// Chance-level agreement gives κ ≈ 0; perfect agreement gives κ = 1.
    #[test]
    fn kappa_anchors() {
        // Independent raters, balanced marginals: [[25,25],[25,25]] → κ = 0.
        let k = cohen_kappa_binary(counts(25, 25, 25, 25)).unwrap_or_else(|err| panic!("{err}"));
        assert!(k.abs() < 1e-12);
        let k = cohen_kappa_binary(counts(30, 0, 0, 70)).unwrap_or_else(|err| panic!("{err}"));
        assert!((k - 1.0).abs() < 1e-12);
    }

    #[test]
    fn kappa_ci_brackets_the_estimate_and_is_deterministic() {
        let c = counts(20, 5, 10, 15);
        let k = cohen_kappa_binary(c).unwrap_or_else(|err| panic!("{err}"));
        let ci1 = cohen_kappa_ci(c, 0.05, 4000, 42).unwrap_or_else(|err| panic!("{err}"));
        let ci2 = cohen_kappa_ci(c, 0.05, 4000, 42).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(ci1, ci2, "same seed must reproduce the interval");
        assert!(ci1.low <= k && k <= ci1.high, "CI {ci1:?} must bracket {k}");
        assert!(ci1.low > -1.0 - 1e-9 && ci1.high <= 1.0 + 1e-9);
        // n = 50 is small: the interval must be honestly wide.
        assert!(ci1.high - ci1.low > 0.2, "suspiciously tight: {ci1:?}");
    }

    /// Monte-Carlo coverage of the kappa bootstrap CI: draw tables from a known
    /// population, check the nominal-90% interval covers the true kappa at
    /// roughly the nominal rate (percentile bootstrap on a bounded, discrete
    /// statistic — accept a generous band).
    #[test]
    fn kappa_ci_has_reasonable_coverage() {
        // Population: p(both_pass)=0.35, p(a_only)=0.10, p(b_only)=0.10, p(both_fail)=0.45.
        // p_o = 0.8; marginals a_pass=0.45, b_pass=0.45 → p_e = 0.45·0.45+0.55·0.55 = 0.505.
        // κ_true = (0.8 − 0.505)/0.495 ≈ 0.59596.
        let kappa_true = (0.8 - 0.505) / 0.495;
        let mut rng = crate::Xorshift64::new(7);
        let (trials, n) = (400usize, 80u64);
        let mut covered = 0usize;
        for t in 0..trials {
            let mut c = counts(0, 0, 0, 0);
            for _ in 0..n {
                let u = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
                if u < 0.35 {
                    c.both_pass += 1;
                } else if u < 0.45 {
                    c.a_pass_b_fail += 1;
                } else if u < 0.55 {
                    c.a_fail_b_pass += 1;
                } else {
                    c.both_fail += 1;
                }
            }
            let ci = cohen_kappa_ci(c, 0.10, 1000, t as u64).unwrap_or_else(|err| panic!("{err}"));
            if ci.low <= kappa_true && kappa_true <= ci.high {
                covered += 1;
            }
        }
        let coverage = covered as f64 / trials as f64;
        assert!(
            (0.82..=0.97).contains(&coverage),
            "coverage {coverage} out of band for nominal 0.90"
        );
    }

    #[test]
    fn brier_known_values() {
        // Perfect confident predictions → 0; maximally wrong → 1; hedged 0.5 → 0.25.
        assert_eq!(
            brier_score(&[1.0, 0.0], &[1.0, 0.0]).unwrap_or_else(|err| panic!("{err}")),
            0.0
        );
        assert_eq!(
            brier_score(&[0.0, 1.0], &[1.0, 0.0]).unwrap_or_else(|err| panic!("{err}")),
            1.0
        );
        assert!(
            (brier_score(&[0.5, 0.5], &[1.0, 0.0]).unwrap_or_else(|err| panic!("{err}")) - 0.25)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn brier_and_ece_validate_inputs() {
        assert!(matches!(
            brier_score(&[], &[]),
            Err(StatsError::EmptySample)
        ));
        assert!(matches!(
            brier_score(&[0.5], &[1.0, 0.0]),
            Err(StatsError::MismatchedLengths { .. })
        ));
        assert!(matches!(
            brier_score(&[1.5], &[1.0]),
            Err(StatsError::InvalidParameter {
                name: "prediction",
                ..
            })
        ));
        assert!(matches!(
            brier_score(&[0.5], &[0.7]),
            Err(StatsError::InvalidParameter {
                name: "outcome",
                ..
            })
        ));
        assert!(matches!(
            expected_calibration_error(&[0.5], &[1.0], 0),
            Err(StatsError::InvalidParameter {
                name: "bin_count",
                ..
            })
        ));
    }

    /// Hand-computed ECE with 2 bins: predictions {0.2, 0.3} (bin 0) with
    /// outcomes {0, 1} → conf 0.25, acc 0.5, gap 0.25; predictions {0.9, 1.0}
    /// (bin 1) with outcomes {1, 1} → conf 0.95, acc 1.0, gap 0.05.
    /// ECE = 0.5·0.25 + 0.5·0.05 = 0.15.
    #[test]
    fn ece_hand_computed() {
        let ece = expected_calibration_error(&[0.2, 0.3, 0.9, 1.0], &[0.0, 1.0, 1.0, 1.0], 2)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!((ece - 0.15).abs() < 1e-12, "ece = {ece}");
    }

    /// A perfectly calibrated predictor has ECE ≈ 0 in the large-sample limit.
    #[test]
    fn ece_near_zero_for_calibrated_predictions() {
        let mut rng = crate::Xorshift64::new(99);
        let mut predictions = Vec::new();
        let mut outcomes = Vec::new();
        for i in 0..20_000 {
            let p = ((i % 10) as f64 + 0.5) / 10.0; // 0.05, 0.15, ..., 0.95
            let u = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
            predictions.push(p);
            outcomes.push(if u < p { 1.0 } else { 0.0 });
        }
        let ece = expected_calibration_error(&predictions, &outcomes, 10)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(ece < 0.02, "calibrated ECE should be near 0, got {ece}");
    }
}
