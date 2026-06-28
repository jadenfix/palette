//! Student's paired t-test.

use crate::numerics::{students_t_quantile, students_t_two_sided_p};
use crate::{
    mean, sample_variance, validate_alpha, ConfidenceInterval, StatsError, TestKind, TestOutcome,
};

/// Two-sided paired t-test over the per-pair `differences`.
///
/// Returns the mean difference, a real Student-t confidence interval at
/// `1 - alpha`, and a real two-sided p-value. Degrees of freedom are `n - 1`.
///
/// When the differences have zero variance the test is degenerate: the p-value
/// is 1.0 for an all-zero difference (no evidence of any effect) and 0.0 for a
/// constant non-zero difference (a perfectly consistent shift), and the CI
/// collapses to the point estimate.
pub fn paired_t_test(differences: &[f64], alpha: f64) -> Result<TestOutcome, StatsError> {
    validate_alpha(alpha)?;
    let n = differences.len();
    if n < 2 {
        return Err(StatsError::TooFewSamples { got: n, need: 2 });
    }
    for d in differences {
        if !d.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }

    let estimate = mean(differences);
    let variance = sample_variance(differences);
    let standard_error = (variance / n as f64).sqrt();
    let df = n as f64 - 1.0;

    let (p_value, ci) = if standard_error == 0.0 {
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
        test: TestKind::PairedT,
        df: Some(df),
        sample_size: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_samples() {
        assert!(matches!(
            paired_t_test(&[0.1], 0.05),
            Err(StatsError::TooFewSamples { got: 1, need: 2 })
        ));
    }

    #[test]
    fn known_one_sample_t_value() {
        // Differences 1..=10: mean=5.5, sd≈3.0277, se≈0.9574, t≈5.745, df=9.
        // Two-sided p ≈ 2.78e-4 (textbook). Check t-derived p in a tight band.
        let diffs: Vec<f64> = (1..=10).map(|v| v as f64).collect();
        let out = paired_t_test(&diffs, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!((out.estimate - 5.5).abs() < 1e-9);
        assert_eq!(out.df, Some(9.0));
        assert!(
            out.p_value < 1e-3 && out.p_value > 1e-4,
            "p={}",
            out.p_value
        );
        // 95% CI for n=10, df=9: t*=2.262 -> 5.5 ± 2.262*0.9574 ≈ [3.33, 7.67].
        let ci = out.ci.unwrap_or_else(|| panic!("expected ci"));
        assert!((ci.low - 3.334).abs() < 0.02, "low={}", ci.low);
        assert!((ci.high - 7.666).abs() < 0.02, "high={}", ci.high);
    }

    #[test]
    fn constant_nonzero_difference_is_degenerate() {
        let out = paired_t_test(&[2.0, 2.0, 2.0, 2.0], 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.p_value, 0.0);
        let ci = out.ci.unwrap_or_else(|| panic!("expected ci"));
        assert_eq!(ci.low, 2.0);
        assert_eq!(ci.high, 2.0);
    }

    #[test]
    fn zero_difference_is_degenerate() {
        let out = paired_t_test(&[0.0, 0.0, 0.0], 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.p_value, 1.0);
    }

    #[test]
    fn wider_alpha_gives_narrower_interval() {
        let diffs = [0.1, 0.2, -0.05, 0.15, 0.0, 0.3];
        let tight = paired_t_test(&diffs, 0.01)
            .unwrap_or_else(|err| panic!("{err}"))
            .ci
            .unwrap_or_else(|| panic!("expected ci"));
        let loose = paired_t_test(&diffs, 0.10)
            .unwrap_or_else(|err| panic!("{err}"))
            .ci
            .unwrap_or_else(|| panic!("expected ci"));
        assert!((tight.high - tight.low) > (loose.high - loose.low));
    }
}
