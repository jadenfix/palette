//! `beater-stats` — the real statistics behind Beater's experiment gates.
//!
//! This crate exists to make Principle 9/11 (§1, §10.3 of `ARCHITECTURE.md`) true
//! in code: a deploy gate may return *pass* only on a **real p-value** computed
//! with a **method-appropriate test** for the metric type, never the previous
//! hand-rolled paired normal-approximation in `beater-eval` that hard-coded its
//! critical value (`z = 1.96 / 2.576`) and reported no p-value at all — so its
//! *nominal* alpha did not equal its *actual* alpha.
//!
//! ## Scope is deliberately narrow
//! It implements only what the gate **calls today**: Student's paired t for
//! continuous metrics and the exact McNemar test for paired binary outcomes,
//! behind a [`compare_paired`] selector. Other methods named in §10.3 (Wilcoxon,
//! bootstrap CIs, Wilson intervals, Holm/Benjamini-Hochberg multiplicity, power
//! planning) are intentionally **not** here: each lands with the consumer that
//! needs it (power planning with the richer "underpowered" gate report, #61;
//! multiplicity with multi-metric experiments, §20.5 #3.5) rather than shipping
//! as unreachable code. Anytime-valid sequences (mSPRT, §10.3 #6) are the
//! Phase-4 online follow-on.
//!
//! The special functions it needs (incomplete beta, Student-t quantile, normal
//! quantile, exact binomial tail) are hand-rolled in [`numerics`] so the crate
//! pulls in no heavyweight stats/linear-algebra dependency — only `thiserror`.

mod mcnemar;
mod numerics;
mod paired;

pub use mcnemar::mcnemar_exact_p;
pub use paired::paired_t_test;

use numerics::normal_quantile;

/// Errors returned by the statistics routines. They are total: every routine
/// validates its inputs and returns one of these rather than panicking, so the
/// crate honors the workspace `unwrap_used`/`expect_used = deny` lints.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StatsError {
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
        let z = normal_quantile(1.0 - alpha / 2.0);
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

pub(crate) fn validate_alpha(alpha: f64) -> Result<(), StatsError> {
    if alpha.is_finite() && alpha > 0.0 && alpha < 1.0 {
        Ok(())
    } else {
        Err(StatsError::InvalidAlpha(alpha))
    }
}

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

fn is_binary(values: &[f64]) -> bool {
    values.iter().all(|v| *v == 0.0 || *v == 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
