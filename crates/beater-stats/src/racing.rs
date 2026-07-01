//! Best-arm identification by a Hoeffding race (§10.3 / #436 item 2).
//!
//! When an optimizer emits *many* candidate arms and scores each on a common set
//! of held-out cases, we do not need a full fixed-horizon test against every arm
//! to know which ones are hopeless. A **Hoeffding race** (Maron & Moore 1994)
//! puts a distribution-free confidence interval around each arm's mean and
//! eliminates any arm whose interval lies entirely below another arm's interval —
//! it is *confidently dominated*. The survivors are the arms that could still be
//! the best at the chosen error level.
//!
//! The bound is Hoeffding's inequality for means of samples in a range `R`:
//! after `n` observations, `|mean − μ| ≤ R·√(ln(2/α') / (2n))` with probability
//! `1 − α'`. We union-bound the per-arm error over the `k` arms (`α' = α/k`) so
//! the *simultaneous* coverage across all arms is `1 − α`. Because it is
//! distribution-free it needs no normality assumption — appropriate for the
//! bounded, skewed per-case scores an eval gate sees.

use crate::{mean, StatsError};

/// Per-arm confidence summary from a [`hoeffding_race`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmSummary {
    /// The arm's index in the input slice.
    pub index: usize,
    /// Number of samples observed for the arm.
    pub n: usize,
    /// Sample mean of the arm.
    pub mean: f64,
    /// Lower Hoeffding confidence bound `mean − radius`.
    pub lower: f64,
    /// Upper Hoeffding confidence bound `mean + radius`.
    pub upper: f64,
    /// `true` iff some other arm's lower bound exceeds this arm's upper bound —
    /// i.e. this arm is *confidently dominated* and can be dropped.
    pub eliminated: bool,
}

/// The outcome of a Hoeffding race across arms.
#[derive(Debug, Clone, PartialEq)]
pub struct RaceOutcome {
    /// Per-arm summaries, in input order.
    pub arms: Vec<ArmSummary>,
    /// Index of the empirically best arm (largest mean; lowest index breaks ties).
    pub best_index: usize,
    /// Indices of arms that survive (not confidently dominated), in input order.
    pub survivors: Vec<usize>,
}

/// Run a Hoeffding best-arm race over `arms`, each a slice of samples assumed to
/// lie in a range of width `range` (e.g. `1.0` for scores in `[0, 1]`).
///
/// An arm is eliminated when another arm's lower confidence bound exceeds its
/// upper confidence bound at simultaneous level `1 − alpha`. Higher mean is
/// better. Every arm must have at least one sample; `range` must be finite and
/// `> 0`; `alpha ∈ (0, 1)`.
pub fn hoeffding_race(arms: &[&[f64]], range: f64, alpha: f64) -> Result<RaceOutcome, StatsError> {
    if arms.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if !range.is_finite() || range <= 0.0 {
        return Err(StatsError::InvalidParameter {
            name: "range",
            value: range,
        });
    }
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return Err(StatsError::InvalidAlpha(alpha));
    }

    // Union-bound the per-arm two-sided error across the k arms.
    let k = arms.len();
    let per_arm_alpha = alpha / k as f64;
    let ln_term = (2.0 / per_arm_alpha).ln();

    let mut summaries = Vec::with_capacity(k);
    for (index, samples) in arms.iter().enumerate() {
        if samples.is_empty() {
            return Err(StatsError::TooFewSamples { got: 0, need: 1 });
        }
        for value in samples.iter() {
            if !value.is_finite() {
                return Err(StatsError::NonFinite);
            }
        }
        let n = samples.len();
        let m = mean(samples);
        let radius = range * (ln_term / (2.0 * n as f64)).sqrt();
        summaries.push(ArmSummary {
            index,
            n,
            mean: m,
            lower: m - radius,
            upper: m + radius,
            eliminated: false,
        });
    }

    // The best confidently-established lower bound: an arm is dominated when its
    // upper bound is below this. Using the single largest lower bound is the
    // standard race elimination rule (only a confidently-good arm can knock
    // others out).
    let best_lower = summaries
        .iter()
        .map(|arm| arm.lower)
        .fold(f64::NEG_INFINITY, f64::max);

    for arm in summaries.iter_mut() {
        // Strict domination: another arm is confidently better than this one.
        if arm.upper < best_lower {
            arm.eliminated = true;
        }
    }

    // Empirically best arm: largest mean, lowest index breaks ties.
    let best_index = summaries
        .iter()
        .fold(None::<&ArmSummary>, |acc, arm| match acc {
            Some(best) if best.mean >= arm.mean => Some(best),
            _ => Some(arm),
        })
        .map(|arm| arm.index)
        .unwrap_or(0);

    let survivors = summaries
        .iter()
        .filter(|arm| !arm.eliminated)
        .map(|arm| arm.index)
        .collect();

    Ok(RaceOutcome {
        arms: summaries,
        best_index,
        survivors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_arms_error() {
        assert!(matches!(
            hoeffding_race(&[], 1.0, 0.05),
            Err(StatsError::EmptySample)
        ));
    }

    #[test]
    fn bad_range_and_alpha_error() {
        let a: &[f64] = &[0.5, 0.5];
        assert!(matches!(
            hoeffding_race(&[a], 0.0, 0.05),
            Err(StatsError::InvalidParameter { name: "range", .. })
        ));
        assert!(matches!(
            hoeffding_race(&[a], 1.0, 1.5),
            Err(StatsError::InvalidAlpha(_))
        ));
    }

    #[test]
    fn empty_arm_error() {
        let a: &[f64] = &[0.5];
        let b: &[f64] = &[];
        assert!(matches!(
            hoeffding_race(&[a, b], 1.0, 0.05),
            Err(StatsError::TooFewSamples { got: 0, need: 1 })
        ));
    }

    #[test]
    fn dominant_arm_eliminates_a_clear_loser() {
        // 40 samples each: arm A ≈ 0.95, arm B ≈ 0.05. With n=40 the Hoeffding
        // radius (~0.24 at these params) leaves the intervals disjoint, so B is
        // confidently dominated.
        let a: Vec<f64> = vec![0.95; 40];
        let b: Vec<f64> = vec![0.05; 40];
        let out = hoeffding_race(&[&a, &b], 1.0, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.best_index, 0);
        assert_eq!(out.survivors, vec![0]);
        assert!(out.arms[1].eliminated, "the clear loser must be eliminated");
        assert!(!out.arms[0].eliminated);
    }

    #[test]
    fn tied_arms_all_survive() {
        let a: Vec<f64> = vec![0.6; 30];
        let b: Vec<f64> = vec![0.6; 30];
        let out = hoeffding_race(&[&a, &b], 1.0, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.survivors, vec![0, 1], "equal arms cannot dominate");
        assert!(out.arms.iter().all(|arm| !arm.eliminated));
        // Lowest index breaks the mean tie.
        assert_eq!(out.best_index, 0);
    }

    #[test]
    fn uncertain_arms_with_few_samples_all_survive() {
        // Big mean gap but only 2 samples each ⇒ radius huge ⇒ cannot eliminate.
        let a: Vec<f64> = vec![0.9, 0.9];
        let b: Vec<f64> = vec![0.1, 0.1];
        let out = hoeffding_race(&[&a, &b], 1.0, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.best_index, 0);
        assert_eq!(
            out.survivors,
            vec![0, 1],
            "too few samples to confidently eliminate"
        );
    }

    #[test]
    fn middle_arm_survives_when_not_dominated() {
        // Three arms; only the clearly-worst is eliminated, the close second stays.
        let a: Vec<f64> = vec![0.90; 50];
        let b: Vec<f64> = vec![0.80; 50];
        let c: Vec<f64> = vec![0.02; 50];
        let out = hoeffding_race(&[&a, &b, &c], 1.0, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.best_index, 0);
        assert!(out.survivors.contains(&0));
        assert!(out.survivors.contains(&1), "close runner-up should survive");
        assert!(!out.survivors.contains(&2), "the clear loser is eliminated");
    }
}
