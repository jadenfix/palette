//! §21.4 anti-overfitting guardrail: generalization-gap detection and a
//! Thresholdout reusable-holdout mechanism.
//!
//! The RSI optimizer searches over candidates using the Train/Val splits and is
//! accepted only on the held-out Test split. Two distinct overfitting risks
//! remain, and this module addresses both:
//!
//! 1. **Generalization gap.** A candidate may beat baseline on the optimization
//!    split yet fail to reproduce that lift on held-out data — the signature of
//!    fitting noise. [`assess_generalization_gap`] bootstraps a confidence
//!    interval for `(optimize_lift − holdout_lift)` and flags a *significant*
//!    gap.
//! 2. **Adaptive reuse of the holdout.** When the optimizer queries the Val set
//!    many times across rounds, naive reuse leaks information and overfits the
//!    holdout. [`Thresholdout`] (Dwork et al., "The reusable holdout", Science
//!    2015) bounds that leakage: it answers with the training estimate unless
//!    the holdout *significantly* disagrees, and only then spends budget to
//!    return a noised holdout value.

use crate::{bootstrap_diff_ci, mean, StatsError, Xorshift64};

/// Outcome of a generalization-gap assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct GapAssessment {
    /// Mean paired lift `(candidate − baseline)` on the optimization split.
    pub optimize_lift: f64,
    /// Mean paired lift `(candidate − baseline)` on the held-out split.
    pub holdout_lift: f64,
    /// `optimize_lift − holdout_lift`. Positive ⇒ the candidate looks better on
    /// data the optimizer could see than on held-out data.
    pub gap: f64,
    /// Lower bound of the bootstrap CI for `gap`.
    pub gap_ci_low: f64,
    /// Upper bound of the bootstrap CI for `gap`.
    pub gap_ci_high: f64,
    /// `true` when the CI lower bound exceeds `tolerance` — i.e. the
    /// optimization-set advantage is *significantly* not reproduced on held-out
    /// data. Such a candidate must be rejected even if it marginally passes the
    /// held-out gate on its own.
    pub overfit: bool,
}

/// Assess the generalization gap between an optimization split and a held-out
/// split for a baseline→candidate change.
///
/// Inputs are paired per split: `optimize_baseline[i]` and
/// `optimize_candidate[i]` are the two scores for the same optimization-split
/// case, and likewise for the holdout split. The per-case lift is
/// `candidate − baseline`; the gap is the difference of mean lifts, with a
/// percentile-bootstrap CI (reusing [`bootstrap_diff_ci`]).
///
/// `tolerance` is the largest gap considered benign (e.g. `0.0` for "held-out
/// lift must not be significantly below optimize lift"). `overfit` is true when
/// `gap_ci_low > tolerance` (a one-sided test at level `(1 − confidence)/2`).
///
/// This reads the holdout split *raw*. To stay valid under many adaptive RSI
/// rounds it must be treated as single-use per holdout, or its holdout statistic
/// routed through a shared [`Thresholdout`]/[`Ladder`] so the budget bounds the
/// whole adaptive sequence — otherwise repeated calls leak the holdout.
#[allow(clippy::too_many_arguments)]
pub fn assess_generalization_gap(
    optimize_baseline: &[f64],
    optimize_candidate: &[f64],
    holdout_baseline: &[f64],
    holdout_candidate: &[f64],
    tolerance: f64,
    confidence: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<GapAssessment, StatsError> {
    if optimize_baseline.len() != optimize_candidate.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: optimize_baseline.len(),
            candidate: optimize_candidate.len(),
        });
    }
    if holdout_baseline.len() != holdout_candidate.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: holdout_baseline.len(),
            candidate: holdout_candidate.len(),
        });
    }
    if !tolerance.is_finite() {
        return Err(StatsError::NonFinite);
    }

    // Paired per-case lifts.
    let optimize_lifts: Vec<f64> = optimize_candidate
        .iter()
        .zip(optimize_baseline)
        .map(|(c, b)| c - b)
        .collect();
    let holdout_lifts: Vec<f64> = holdout_candidate
        .iter()
        .zip(holdout_baseline)
        .map(|(c, b)| c - b)
        .collect();

    // gap = mean(optimize_lifts) − mean(holdout_lifts), with a bootstrap CI.
    // bootstrap_diff_ci validates emptiness, finiteness, confidence, n_resamples.
    let interval = bootstrap_diff_ci(
        &optimize_lifts,
        &holdout_lifts,
        confidence,
        n_resamples,
        seed,
    )?;

    let optimize_lift = mean(&optimize_lifts);
    let holdout_lift = mean(&holdout_lifts);

    Ok(GapAssessment {
        optimize_lift,
        holdout_lift,
        gap: interval.estimate,
        gap_ci_low: interval.lower,
        gap_ci_high: interval.upper,
        overfit: interval.lower > tolerance,
    })
}

/// One answer from a [`Thresholdout`] query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThresholdoutAnswer {
    /// The holdout agreed with training (within the noisy threshold); the
    /// training estimate is returned and **no budget is spent**.
    Train(f64),
    /// The holdout *significantly* disagreed; a noised holdout estimate is
    /// returned and one unit of budget was spent.
    Holdout(f64),
    /// The holdout budget is exhausted; no further holdout information may be
    /// released. Callers must stop trusting holdout queries.
    BudgetExhausted,
}

/// Dwork et al. Thresholdout (Science 2015): a reusable-holdout mechanism in the
/// Sparse-Vector style.
///
/// Maintains a query budget and a **persistent** threshold-noise offset `gamma`.
/// Each [`query`](Thresholdout::query) compares a training statistic against a
/// holdout statistic; it returns the training value (free) unless they differ by
/// more than `threshold + gamma + eta` (fresh per-query noise `eta`), in which
/// case it spends budget, returns a *noised* holdout value, and
/// **re-randomizes `gamma`**. Keeping `gamma` fixed between releases is the
/// load-bearing detail: it stops an adversary from averaging the noise away over
/// repeated free queries, which is what actually bounds holdout overfitting
/// across rounds.
///
/// Note: `tolerance` is a heuristic noise scale, *not* calibrated to a formal
/// differential-privacy budget ε. This gives the Thresholdout *structure* (and
/// its averaging-resistance) without claiming a numeric DP guarantee — pick
/// `tolerance` small relative to `threshold`, and keep the holdout large.
#[derive(Debug, Clone)]
pub struct Thresholdout {
    threshold: f64,
    tolerance: f64,
    budget: u32,
    spent: u32,
    /// Persistent threshold-noise offset, re-drawn only after a release.
    gamma: f64,
    rng: Xorshift64,
}

impl Thresholdout {
    /// Construct a Thresholdout.
    ///
    /// * `threshold` — the base gap (on the statistic's scale) beyond which the
    ///   holdout is considered to disagree. Must be finite and > 0.
    /// * `tolerance` — the Laplace noise scale; larger = more averaging-resistance,
    ///   less accuracy. Must be finite and > 0.
    /// * `budget` — maximum number of holdout releases. Must be ≥ 1.
    pub fn new(threshold: f64, tolerance: f64, budget: u32, seed: u64) -> Result<Self, StatsError> {
        if !threshold.is_finite() || threshold <= 0.0 {
            return Err(StatsError::InvalidParameter {
                name: "threshold",
                value: threshold,
            });
        }
        if !tolerance.is_finite() || tolerance <= 0.0 {
            return Err(StatsError::InvalidParameter {
                name: "tolerance",
                value: tolerance,
            });
        }
        if budget == 0 {
            return Err(StatsError::InvalidParameter {
                name: "budget",
                value: 0.0,
            });
        }
        let mut out = Self {
            threshold,
            tolerance,
            budget,
            spent: 0,
            gamma: 0.0,
            rng: Xorshift64::new(seed),
        };
        // Draw the initial persistent threshold-noise offset γ ~ Lap(2·tolerance).
        out.gamma = out.laplace(2.0 * tolerance);
        Ok(out)
    }

    /// Number of holdout releases still available.
    pub fn remaining(&self) -> u32 {
        self.budget.saturating_sub(self.spent)
    }

    /// Query the holdout with a training statistic and a holdout statistic.
    ///
    /// Non-finite inputs yield [`ThresholdoutAnswer::Train`] of the training
    /// value (the conservative, budget-free answer).
    pub fn query(&mut self, train_stat: f64, holdout_stat: f64) -> ThresholdoutAnswer {
        if self.remaining() == 0 {
            return ThresholdoutAnswer::BudgetExhausted;
        }
        if !train_stat.is_finite() || !holdout_stat.is_finite() {
            return ThresholdoutAnswer::Train(train_stat);
        }
        // Canonical Thresholdout comparison: |train − holdout| > threshold + γ + η,
        // with persistent γ and fresh per-query η ~ Lap(4·tolerance).
        let eta = self.laplace(4.0 * self.tolerance);
        if (train_stat - holdout_stat).abs() > self.threshold + self.gamma + eta {
            self.spent += 1;
            let released = holdout_stat + self.laplace(self.tolerance);
            // Re-randomize the persistent offset after each release.
            self.gamma = self.laplace(2.0 * self.tolerance);
            ThresholdoutAnswer::Holdout(released)
        } else {
            ThresholdoutAnswer::Train(train_stat)
        }
    }

    /// Laplace(0, scale) via inverse-CDF on a uniform draw in the open interval
    /// (0, 1), which keeps the distribution symmetric and mean-zero.
    fn laplace(&mut self, scale: f64) -> f64 {
        // (bits + 0.5) / 2^53 ∈ (0, 1) — open on both ends, symmetric about 0.5.
        let bits = self.rng.next_u64() >> 11;
        let u = (bits as f64 + 0.5) / (1u64 << 53) as f64;
        let centered = u - 0.5; // (-0.5, 0.5)
        -scale * centered.signum() * (1.0 - 2.0 * centered.abs()).ln()
    }
}

/// Blum–Hardt "Ladder" (ICML 2015): a leaderboard-style reusable holdout for the
/// multi-round candidate-selection pattern.
///
/// Across RSI rounds the gate repeatedly asks "is this candidate the new best on
/// held-out data?". The Ladder only reveals a new best score when a submission
/// beats the running best by a margin `eta`; otherwise it re-reveals the prior
/// best. This bounds the adaptive leaderboard error to roughly
/// `O((log(kn)/n)^(1/3))` in the number of submissions `k`, an exponential
/// improvement over the `sqrt(k)` of a naive leaderboard. Complements
/// [`Thresholdout`]: use the Ladder for "track the best candidate", Thresholdout
/// for "answer this holdout query".
#[derive(Debug, Clone, Default)]
pub struct Ladder {
    margin: f64,
    best: Option<f64>,
}

impl Ladder {
    /// Construct a Ladder for a *maximized* held-out score. `margin` (η) is the
    /// improvement a submission must clear to be revealed; must be finite and > 0.
    pub fn new(margin: f64) -> Result<Self, StatsError> {
        if !margin.is_finite() || margin <= 0.0 {
            return Err(StatsError::InvalidParameter {
                name: "margin",
                value: margin,
            });
        }
        Ok(Self { margin, best: None })
    }

    /// Submit a candidate's held-out score; returns the currently-revealed best.
    /// The best advances only when `holdout_score` beats it by more than
    /// `margin`, so noise below the margin can never move the leaderboard.
    pub fn submit(&mut self, holdout_score: f64) -> Result<f64, StatsError> {
        if !holdout_score.is_finite() {
            return Err(StatsError::NonFinite);
        }
        let advance = match self.best {
            None => true,
            Some(b) => holdout_score > b + self.margin,
        };
        if advance {
            self.best = Some(holdout_score);
        }
        Ok(self.best.unwrap_or(holdout_score))
    }

    /// The currently-revealed best score, if any submission has been made.
    pub fn best(&self) -> Option<f64> {
        self.best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_gap_when_lift_reproduces_on_holdout() {
        // Candidate adds ~+0.2 on both splits → gap ≈ 0, not flagged.
        let opt_b = vec![0.5; 60];
        let opt_c = vec![0.7; 60];
        let hold_b = vec![0.5; 40];
        let hold_c = vec![0.7; 40];
        let a = assess_generalization_gap(&opt_b, &opt_c, &hold_b, &hold_c, 0.05, 0.95, 2000, 1)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!((a.gap).abs() < 0.05, "gap {}", a.gap);
        assert!(!a.overfit, "should not flag a reproduced lift");
    }

    #[test]
    fn detects_overfit_when_holdout_lift_collapses() {
        // Big lift on optimize (+0.4), none on holdout (0) → significant gap.
        let opt_b = vec![0.4; 80];
        let opt_c = vec![0.8; 80];
        let hold_b = vec![0.6; 60];
        let hold_c = vec![0.6; 60];
        let a = assess_generalization_gap(&opt_b, &opt_c, &hold_b, &hold_c, 0.05, 0.95, 2000, 7)
            .unwrap_or_else(|err| panic!("{err}"));
        assert!(a.optimize_lift > 0.3, "optimize_lift {}", a.optimize_lift);
        assert!(
            a.holdout_lift.abs() < 0.05,
            "holdout_lift {}",
            a.holdout_lift
        );
        assert!(a.gap_ci_low > 0.05, "gap_ci_low {}", a.gap_ci_low);
        assert!(a.overfit, "should flag the collapse as overfit");
    }

    #[test]
    fn mismatched_lengths_error() {
        let r = assess_generalization_gap(&[0.1, 0.2], &[0.3], &[0.1], &[0.2], 0.0, 0.95, 100, 1);
        assert!(matches!(r, Err(StatsError::MismatchedLengths { .. })));
    }

    #[test]
    fn thresholdout_validates_params() {
        assert!(Thresholdout::new(0.0, 0.1, 10, 1).is_err());
        assert!(Thresholdout::new(0.1, 0.0, 10, 1).is_err());
        assert!(Thresholdout::new(0.1, 0.1, 0, 1).is_err());
        assert!(Thresholdout::new(0.1, 0.1, 10, 1).is_ok());
    }

    #[test]
    fn thresholdout_returns_train_when_holdout_agrees() {
        // threshold (0.5) dwarfs the noise scale (0.002), so even the heavy Laplace
        // tail of γ+η cannot pull threshold+γ+η below 0: a zero gap is always Train,
        // and no budget is spent.
        let mut t = Thresholdout::new(0.5, 0.002, 100, 42).unwrap_or_else(|err| panic!("{err}"));
        let mut train_answers = 0;
        for _ in 0..50 {
            if let ThresholdoutAnswer::Train(_) = t.query(0.80, 0.80) {
                train_answers += 1;
            }
        }
        assert_eq!(train_answers, 50, "a zero gap must always answer Train");
        assert_eq!(t.remaining(), 100, "no budget should be spent");
    }

    #[test]
    fn ladder_only_advances_past_margin() {
        let mut l = Ladder::new(0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(l.submit(0.70).unwrap_or_else(|err| panic!("{err}")), 0.70);
        // Sub-margin "improvement" (noise) must NOT move the leaderboard.
        assert_eq!(l.submit(0.72).unwrap_or_else(|err| panic!("{err}")), 0.70);
        // A worse score must not move it either.
        assert_eq!(l.submit(0.60).unwrap_or_else(|err| panic!("{err}")), 0.70);
        // A real improvement beyond the margin advances.
        assert_eq!(l.submit(0.80).unwrap_or_else(|err| panic!("{err}")), 0.80);
        assert_eq!(l.best(), Some(0.80));
    }

    #[test]
    fn ladder_validates_and_rejects_non_finite() {
        assert!(Ladder::new(0.0).is_err());
        assert!(Ladder::new(-0.1).is_err());
        let mut l = Ladder::new(0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(matches!(l.submit(f64::NAN), Err(StatsError::NonFinite)));
    }

    #[test]
    fn thresholdout_spends_budget_and_exhausts() {
        let mut t = Thresholdout::new(0.05, 0.001, 3, 9).unwrap_or_else(|err| panic!("{err}"));
        // train and holdout differ far beyond threshold → spends each time.
        let mut holdout_releases = 0;
        let mut exhausted = false;
        for _ in 0..10 {
            match t.query(0.10, 0.90) {
                ThresholdoutAnswer::Holdout(_) => holdout_releases += 1,
                ThresholdoutAnswer::BudgetExhausted => {
                    exhausted = true;
                    break;
                }
                ThresholdoutAnswer::Train(_) => {}
            }
        }
        assert_eq!(holdout_releases, 3, "should release exactly the budget");
        assert!(exhausted, "should report exhaustion after budget spent");
        assert_eq!(t.remaining(), 0);
    }

    #[test]
    fn thresholdout_non_finite_is_safe() {
        let mut t = Thresholdout::new(0.1, 0.1, 5, 1).unwrap_or_else(|err| panic!("{err}"));
        assert!(matches!(
            t.query(f64::NAN, 0.5),
            ThresholdoutAnswer::Train(_)
        ));
        assert_eq!(t.remaining(), 5, "non-finite query must not spend budget");
    }
}
