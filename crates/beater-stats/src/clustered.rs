//! Clustered standard errors (ARCHITECTURE.md §10.3 #1) — when observations are
//! **not independent** (multi-turn conversations sharing context, many cases drawn
//! from one prompt template, repeated stochastic draws sharing a seed), naive
//! i.i.d. standard errors are *too small* and inflate false wins. The cluster-
//! robust ("sandwich") SE treats whole clusters as the independent unit.
//!
//! The cluster definition comes from the pre-registered design's
//! `cluster_key` (`beater-design`); this module only consumes the resolved
//! per-observation cluster labels.

use crate::{mean, StatsError, Xorshift64};

/// A cluster-robust standard error of a sample mean, with the cluster and
/// observation counts that produced it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClusteredStandardError {
    /// Cluster-robust standard error of the mean.
    pub standard_error: f64,
    /// Number of distinct clusters `G` (the effective independent sample size).
    pub n_clusters: usize,
    /// Number of observations `N`.
    pub n: usize,
}

/// Cluster-robust standard error of the mean of `values`, with `cluster_ids[i]`
/// the cluster label of `values[i]`.
///
/// # Formula (CR1 sandwich for a mean)
///
/// With residuals `eᵢ = vᵢ − v̄`, cluster sums `S_g = Σ_{i∈g} eᵢ`, `G` clusters,
/// and `N` observations:
///
/// ```text
/// Var(v̄) = ( G / (G−1) ) · ( 1 / N² ) · Σ_g S_g²
/// SE      = √Var(v̄)
/// ```
///
/// The `G/(G−1)` factor is the standard finite-cluster correction. When every
/// observation is its own cluster this reduces (up to that factor) to the i.i.d.
/// standard error; when observations within a cluster are positively correlated
/// it is **larger** than the i.i.d. SE, as it should be.
///
/// # Errors
///
/// * [`StatsError::EmptySample`] when `values` is empty.
/// * [`StatsError::MismatchedLengths`] when `values` and `cluster_ids` differ.
/// * [`StatsError::NonFinite`] when any value is NaN/inf.
/// * [`StatsError::TooFewSamples`] when there are fewer than two clusters (the
///   between-cluster variance is then unidentified).
///
/// # Example
///
/// ```
/// use beater_stats::clustered_standard_error;
///
/// // Two tight clusters that disagree strongly: the i.i.d. SE understates the
/// // uncertainty, the clustered SE captures it.
/// let values = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
/// let clusters = ["a", "a", "a", "b", "b", "b"];
/// let cr = clustered_standard_error(&values, &clusters)?;
/// assert_eq!(cr.n_clusters, 2);
/// # Ok::<(), beater_stats::StatsError>(())
/// ```
pub fn clustered_standard_error<T: PartialEq>(
    values: &[f64],
    cluster_ids: &[T],
) -> Result<ClusteredStandardError, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if values.len() != cluster_ids.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: values.len(),
            candidate: cluster_ids.len(),
        });
    }
    for v in values {
        if !v.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }

    let n = values.len();
    let avg = mean(values);

    // Group residual sums by cluster, preserving first-seen order. Linear scan is
    // fine for eval-scale N and keeps the crate dependency-free.
    let mut labels: Vec<&T> = Vec::new();
    let mut cluster_sums: Vec<f64> = Vec::new();
    for (value, id) in values.iter().zip(cluster_ids.iter()) {
        let residual = value - avg;
        if let Some(pos) = labels.iter().position(|l| **l == *id) {
            cluster_sums[pos] += residual;
        } else {
            labels.push(id);
            cluster_sums.push(residual);
        }
    }

    let g = cluster_sums.len();
    if g < 2 {
        return Err(StatsError::TooFewSamples { got: g, need: 2 });
    }

    let sum_sq: f64 = cluster_sums.iter().map(|s| s * s).sum();
    let correction = g as f64 / (g as f64 - 1.0);
    let variance = correction * sum_sq / (n as f64 * n as f64);

    Ok(ClusteredStandardError {
        standard_error: variance.sqrt(),
        n_clusters: g,
        n,
    })
}

/// The naive i.i.d. standard error of the mean, `√(s² / n)`, exposed so callers
/// (and tests) can compare it against the clustered SE.
pub fn iid_standard_error(values: &[f64]) -> Result<f64, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    for v in values {
        if !v.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }
    let var = crate::sample_variance(values);
    Ok((var / values.len() as f64).sqrt())
}

/// Cluster bootstrap: resample whole **clusters** (not individual observations)
/// with replacement and return a percentile CI for the mean. This is the
/// resampling analogue of [`clustered_standard_error`] and is what §10.3 #1 means
/// by "resample whole clusters for clustered data".
///
/// # Errors
///
/// Same input validation as [`clustered_standard_error`], plus
/// [`StatsError::InvalidParameter`] for a `confidence` outside `(0, 1)` and
/// [`StatsError::InvalidResampleCount`] for `n_resamples == 0`.
pub fn clustered_bootstrap_ci<T: PartialEq + Clone>(
    values: &[f64],
    cluster_ids: &[T],
    confidence: f64,
    n_resamples: usize,
    seed: u64,
) -> Result<crate::BootstrapInterval, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if values.len() != cluster_ids.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: values.len(),
            candidate: cluster_ids.len(),
        });
    }
    for v in values {
        if !v.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }
    if !(0.0 < confidence && confidence < 1.0) {
        return Err(StatsError::InvalidParameter {
            name: "confidence",
            value: confidence,
        });
    }
    if n_resamples == 0 {
        return Err(StatsError::InvalidResampleCount(n_resamples));
    }

    // Bucket observations by cluster (first-seen order).
    let mut labels: Vec<T> = Vec::new();
    let mut buckets: Vec<Vec<f64>> = Vec::new();
    for (value, id) in values.iter().zip(cluster_ids.iter()) {
        if let Some(pos) = labels.iter().position(|l| *l == *id) {
            buckets[pos].push(*value);
        } else {
            labels.push(id.clone());
            buckets.push(vec![*value]);
        }
    }
    let g = buckets.len();

    let observed = mean(values);
    let mut rng = Xorshift64::new(seed);
    let mut means: Vec<f64> = Vec::with_capacity(n_resamples);
    for _ in 0..n_resamples {
        let mut sum = 0.0;
        let mut count = 0usize;
        for _ in 0..g {
            let bucket = &buckets[rng.next_index(g)];
            sum += bucket.iter().sum::<f64>();
            count += bucket.len();
        }
        means.push(sum / count as f64);
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let lo_idx = ((alpha / 2.0) * n_resamples as f64).floor() as usize;
    let hi_idx = ((1.0 - alpha / 2.0) * n_resamples as f64).floor() as usize;
    Ok(crate::BootstrapInterval {
        lower: means[lo_idx.min(n_resamples - 1)],
        upper: means[hi_idx.min(n_resamples - 1)],
        estimate: observed,
        n_resamples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_inputs() {
        assert!(matches!(
            clustered_standard_error::<u8>(&[], &[]),
            Err(StatsError::EmptySample)
        ));
        assert!(matches!(
            clustered_standard_error(&[1.0, 2.0], &[0u8]),
            Err(StatsError::MismatchedLengths { .. })
        ));
        assert!(matches!(
            clustered_standard_error(&[1.0, f64::NAN], &[0u8, 1u8]),
            Err(StatsError::NonFinite)
        ));
    }

    #[test]
    fn single_cluster_is_unidentified() {
        assert!(matches!(
            clustered_standard_error(&[1.0, 2.0, 3.0], &["a", "a", "a"]),
            Err(StatsError::TooFewSamples { .. })
        ));
    }

    #[test]
    fn clustered_se_exceeds_iid_when_within_cluster_correlated() {
        // Strong within-cluster correlation: each cluster is near-constant, the two
        // clusters disagree. i.i.d. SE sees 6 "independent" points; the clustered
        // SE correctly sees 2.
        let values = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let clusters = ["a", "a", "a", "b", "b", "b"];
        let cr = clustered_standard_error(&values, &clusters).unwrap_or_else(|err| panic!("{err}"));
        let iid = iid_standard_error(&values).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(cr.n_clusters, 2);
        assert!(
            cr.standard_error > iid,
            "clustered {} should exceed iid {}",
            cr.standard_error,
            iid
        );
    }

    #[test]
    fn singleton_clusters_are_close_to_iid() {
        // Every observation its own cluster: clustered SE ≈ iid SE (up to the
        // G/(G-1) finite-cluster correction).
        let values = [0.1, 0.4, 0.2, 0.7, 0.5, 0.3, 0.6, 0.8];
        let clusters: Vec<usize> = (0..values.len()).collect();
        let cr = clustered_standard_error(&values, &clusters).unwrap_or_else(|err| panic!("{err}"));
        let iid = iid_standard_error(&values).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(cr.n_clusters, values.len());
        // Within ~15% (the correction inflates a little at G=8).
        assert!(
            (cr.standard_error / iid - 1.0).abs() < 0.15,
            "clustered {} vs iid {}",
            cr.standard_error,
            iid
        );
    }

    #[test]
    fn cluster_bootstrap_is_deterministic() {
        let values = [0.0, 0.1, 1.0, 1.1, 2.0, 2.1];
        let clusters = ["a", "a", "b", "b", "c", "c"];
        let first = clustered_bootstrap_ci(&values, &clusters, 0.95, 2_000, 7)
            .unwrap_or_else(|err| panic!("{err}"));
        let second = clustered_bootstrap_ci(&values, &clusters, 0.95, 2_000, 7)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(first, second);
        assert!(first.lower <= first.estimate && first.estimate <= first.upper);
    }
}
