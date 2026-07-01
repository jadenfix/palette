//! Wilcoxon signed-rank test (ARCHITECTURE.md §10.3 #3) — the paired test for
//! continuous, **non-normal** difference distributions, where a paired *t*-test's
//! normality assumption is not met and an exact McNemar (binary) test does not
//! apply.
//!
//! The location estimate paired with the test is the **Hodges-Lehmann** pseudo-
//! median (the median of the Walsh averages `(dᵢ + dⱼ)/2`), and the interval is a
//! distribution-free confidence interval derived from the signed-rank null. This
//! keeps the reported estimate and CI on the same footing as the test, exactly as
//! the paired-*t* path reports a mean difference with a Student-*t* interval.

use crate::numerics::normal_quantile;
use crate::{normal_cdf, validate_alpha, ConfidenceInterval, StatsError};

/// Outcome of a Wilcoxon signed-rank test.
///
/// `estimate`/`ci` are on the Hodges-Lehmann (pseudo-median) scale; `p_value` is
/// the two-sided significance. The estimator is deliberately *not* a
/// [`crate::TestKind`]-tagged [`crate::TestOutcome`]: surfacing the new test
/// through the gate's `StatisticalTest` contract enum is the follow-on wiring
/// step (roadmap PR-A4), so this crate returns a self-contained result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WilcoxonOutcome {
    /// Hodges-Lehmann pseudo-median of the paired differences.
    pub estimate: f64,
    /// Distribution-free CI for the pseudo-median, or `None` when `n` is too small
    /// for the rank index to be positive at this `alpha`.
    pub ci: Option<ConfidenceInterval>,
    /// Two-sided p-value.
    pub p_value: f64,
    /// Number of pairs supplied.
    pub sample_size: usize,
    /// Number of non-zero differences (zeros are dropped from the rank statistic).
    pub nonzero: usize,
    /// Whether the exact signed-rank null distribution was used (`true`) or the
    /// tie-corrected normal approximation (`false`).
    pub used_exact: bool,
}

/// Largest number of non-zero differences for which the exact signed-rank null is
/// enumerated by dynamic programming. Above this, or when `|d|` has ties, the
/// tie-corrected normal approximation with a continuity correction is used. `50`
/// keeps the subset-count DP within exact `f64` integer range (`2⁵⁰ < 2⁵³`).
const EXACT_MAX_NONZERO: usize = 50;

/// Two-sided Wilcoxon signed-rank test over the per-pair `differences`
/// (`candidate − baseline`).
///
/// Zeros are dropped from the rank statistic (the standard Pratt-vs-Wilcoxon
/// choice here is plain Wilcoxon: discard zeros). The exact null is used when the
/// non-zero count is small and `|d|` has no ties; otherwise the normal
/// approximation with tie correction `Σ(t³ − t)/48` and a ½ continuity correction
/// is used.
///
/// # Errors
///
/// * [`StatsError::InvalidAlpha`] when `alpha ∉ (0, 1)`.
/// * [`StatsError::TooFewSamples`] when fewer than two pairs are supplied.
/// * [`StatsError::NonFinite`] when any difference is NaN/inf.
///
/// # Example
///
/// ```
/// use beater_stats::wilcoxon_signed_rank;
///
/// // Six improvements, two small regressions; no ties in |d|.
/// let diffs = [0.5, 0.7, -0.2, 0.8, 0.3, 0.6, -0.1, 0.9];
/// let out = wilcoxon_signed_rank(&diffs, 0.05)?;
/// // Exact two-sided p = 2 · (#subsets of {1..8} summing ≤ 3) / 2⁸ = 10/256.
/// assert!((out.p_value - 0.0390625).abs() < 1e-9);
/// assert!(out.estimate > 0.0);
/// # Ok::<(), beater_stats::StatsError>(())
/// ```
pub fn wilcoxon_signed_rank(
    differences: &[f64],
    alpha: f64,
) -> Result<WilcoxonOutcome, StatsError> {
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

    // Non-zero differences carry the rank statistic.
    let nonzero: Vec<f64> = differences.iter().copied().filter(|d| *d != 0.0).collect();
    let m = nonzero.len();

    // Hodges-Lehmann estimate + distribution-free CI use ALL Walsh averages
    // (including zero differences), which is the standard pseudo-median.
    let walsh = walsh_averages(differences);
    let estimate = median_sorted(&sorted(&walsh));
    let ci = signed_rank_ci(&walsh, n, alpha);

    if m == 0 {
        // Every difference is zero: no evidence of any effect.
        return Ok(WilcoxonOutcome {
            estimate,
            ci,
            p_value: 1.0,
            sample_size: n,
            nonzero: 0,
            used_exact: true,
        });
    }

    // Average ranks of |d|, then the positive-rank sum W⁺.
    let abs: Vec<f64> = nonzero.iter().map(|d| d.abs()).collect();
    let ranks = average_ranks(&abs);
    let mut w_plus = 0.0;
    for (d, r) in nonzero.iter().zip(ranks.iter()) {
        if *d > 0.0 {
            w_plus += r;
        }
    }

    let has_ties = has_tied_abs(&abs);
    let (p_value, used_exact) = if m <= EXACT_MAX_NONZERO && !has_ties {
        (exact_two_sided_p(w_plus, m), true)
    } else {
        (normal_approx_two_sided_p(w_plus, &abs), false)
    };

    Ok(WilcoxonOutcome {
        estimate,
        ci,
        p_value,
        sample_size: n,
        nonzero: m,
        used_exact,
    })
}

/// All Walsh averages `(dᵢ + dⱼ)/2` for `i ≤ j`.
fn walsh_averages(d: &[f64]) -> Vec<f64> {
    let n = d.len();
    let mut out = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..n {
        for j in i..n {
            out.push((d[i] + d[j]) / 2.0);
        }
    }
    out
}

fn sorted(values: &[f64]) -> Vec<f64> {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

fn median_sorted(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

/// Distribution-free CI for the pseudo-median from the sorted Walsh averages,
/// using the normal approximation to the signed-rank null for the rank index
/// `C = ⌊ n(n+1)/4 − z·√(n(n+1)(2n+1)/24) ⌋`. The interval is
/// `[W₍C₎, W₍M+1−C₎]` (1-indexed). `None` when `C < 1` (too few pairs).
fn signed_rank_ci(walsh: &[f64], n: usize, alpha: f64) -> Option<ConfidenceInterval> {
    let sorted = sorted(walsh);
    let m_total = sorted.len();
    let nn = n as f64;
    let mean_w = nn * (nn + 1.0) / 4.0;
    let sd_w = (nn * (nn + 1.0) * (2.0 * nn + 1.0) / 24.0).sqrt();
    let z = normal_quantile(1.0 - alpha / 2.0);
    let c = (mean_w - z * sd_w).floor();
    if !(c.is_finite()) || c < 1.0 {
        return None;
    }
    let c = c as usize;
    if c == 0 || c > m_total {
        return None;
    }
    let lower = sorted[c - 1];
    let upper = sorted[m_total - c];
    Some(ConfidenceInterval {
        low: lower,
        high: upper,
        confidence: 1.0 - alpha,
    })
}

/// Average (fractional) ranks of `values`, smallest = rank 1, ties share the mean
/// of the ranks they span.
fn average_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| {
        values[a]
            .partial_cmp(&values[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && values[idx[j]] == values[idx[i]] {
            j += 1;
        }
        // positions i..j (0-based) share ranks (i+1)..=j (1-based); average them.
        let avg = ((i + 1 + j) as f64) / 2.0;
        for &k in &idx[i..j] {
            ranks[k] = avg;
        }
        i = j;
    }
    ranks
}

fn has_tied_abs(abs: &[f64]) -> bool {
    let s = sorted(abs);
    s.windows(2).any(|w| w[0] == w[1])
}

/// Exact two-sided p-value for `W⁺` under the signed-rank null over `m` untied
/// ranks, via a subset-sum DP: `counts[s]` = number of subsets of `{1..m}`
/// summing to `s`, so `P(W⁺ = s) = counts[s] / 2ᵐ`.
fn exact_two_sided_p(w_plus: f64, m: usize) -> f64 {
    let total = m * (m + 1) / 2;
    let mut counts = vec![0.0f64; total + 1];
    counts[0] = 1.0;
    for r in 1..=m {
        for s in (r..=total).rev() {
            counts[s] += counts[s - r];
        }
    }
    let denom = 2f64.powi(m as i32);
    let w = w_plus.round() as usize;
    let w = w.min(total);
    // Two-sided: 2 · min(P(W⁺ ≤ w), P(W⁺ ≥ w)), clamped to 1.
    let le: f64 = counts[..=w].iter().sum::<f64>() / denom;
    let ge: f64 = counts[w..].iter().sum::<f64>() / denom;
    (2.0 * le.min(ge)).min(1.0)
}

/// Tie-corrected normal approximation with a ½ continuity correction.
fn normal_approx_two_sided_p(w_plus: f64, abs: &[f64]) -> f64 {
    let m = abs.len() as f64;
    let mean_w = m * (m + 1.0) / 4.0;
    let tie_correction = tie_correction_term(abs);
    let var_w = m * (m + 1.0) * (2.0 * m + 1.0) / 24.0 - tie_correction;
    if var_w <= 0.0 {
        return 1.0;
    }
    let diff = w_plus - mean_w;
    // ½ continuity correction towards the mean.
    let corrected = if diff > 0.0 {
        (diff - 0.5).max(0.0)
    } else {
        (diff + 0.5).min(0.0)
    };
    let z = corrected / var_w.sqrt();
    (2.0 * normal_cdf(-z.abs())).min(1.0)
}

/// `Σ (tᵢ³ − tᵢ) / 48` over tie groups of `|d|`, the signed-rank variance
/// deflation for ties.
fn tie_correction_term(abs: &[f64]) -> f64 {
    let s = sorted(abs);
    let mut total = 0.0;
    let mut i = 0;
    while i < s.len() {
        let mut j = i + 1;
        while j < s.len() && s[j] == s[i] {
            j += 1;
        }
        let t = (j - i) as f64;
        if t > 1.0 {
            total += (t * t * t - t) / 48.0;
        }
        i = j;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_too_few_and_nonfinite() {
        assert!(matches!(
            wilcoxon_signed_rank(&[0.1], 0.05),
            Err(StatsError::TooFewSamples { .. })
        ));
        assert!(matches!(
            wilcoxon_signed_rank(&[0.1, f64::NAN], 0.05),
            Err(StatsError::NonFinite)
        ));
    }

    #[test]
    fn exact_two_sided_p_textbook() {
        // n = 8, no ties in |d|. W⁻ = 3 (ranks 1 and 2 are the two regressions).
        // Two-sided p = 2 · (#subsets of {1..8} with sum ≤ 3)/2⁸ = 2·5/256 = 10/256.
        let diffs = [0.5, 0.7, -0.2, 0.8, 0.3, 0.6, -0.1, 0.9];
        let out = wilcoxon_signed_rank(&diffs, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(out.used_exact);
        assert_eq!(out.nonzero, 8);
        assert!(
            (out.p_value - 10.0 / 256.0).abs() < 1e-12,
            "p={}",
            out.p_value
        );
        assert!(out.estimate > 0.0, "HL estimate should be positive");
    }

    #[test]
    fn drops_zero_differences() {
        let diffs = [0.0, 0.0, 0.5, -0.2, 0.8];
        let out = wilcoxon_signed_rank(&diffs, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.nonzero, 3);
        assert_eq!(out.sample_size, 5);
    }

    #[test]
    fn all_zero_is_not_significant() {
        let out =
            wilcoxon_signed_rank(&[0.0, 0.0, 0.0, 0.0], 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.p_value, 1.0);
        assert!((out.estimate).abs() < 1e-12);
    }

    #[test]
    fn ties_use_normal_approximation() {
        // Repeated |d| magnitudes force the tie-corrected normal path.
        let diffs = [
            0.2, 0.2, 0.2, -0.2, 0.4, 0.4, 0.4, -0.4, 0.6, 0.6, 0.6, -0.6,
        ];
        let out = wilcoxon_signed_rank(&diffs, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(
            !out.used_exact,
            "ties must trigger the normal approximation"
        );
        assert!(out.p_value > 0.0 && out.p_value <= 1.0);
    }

    #[test]
    fn consistent_improvement_is_significant_and_has_positive_ci() {
        let diffs: Vec<f64> = (1..=12).map(|v| 0.1 * v as f64).collect();
        let out = wilcoxon_signed_rank(&diffs, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(out.p_value < 0.05, "p={}", out.p_value);
        let ci = out.ci.unwrap_or_else(|| panic!("expected a CI at n=12"));
        assert!(ci.low > 0.0, "CI should be entirely positive: {ci:?}");
    }
}
