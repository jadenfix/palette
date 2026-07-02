//! Shared bootstrap-resampling engine.
//!
//! Four routines in this crate resample the same way — [`crate::bootstrap_diff_ci`],
//! [`crate::bootstrap_bca_ci`], [`crate::paired_bootstrap_test`], and
//! [`crate::clustered_bootstrap_ci`]. Before this module each carried its own
//! copy of the `#[cfg(feature = "parallel")]` resample loop and its own percentile
//! index arithmetic. [`Bootstrap`] is the single place that owns those two
//! concerns, so every bootstrap in the crate inherits the same guarantees:
//!
//! * **Reproducible & order-independent.** Replicate `i` is driven by an
//!   independent [`Xorshift64`] substream seeded from `(seed, i)` via SplitMix64,
//!   so it is a pure function of its index — the distribution never depends on the
//!   order the replicates are evaluated in.
//! * **Parallel when it pays.** Under the default-on `parallel` feature the
//!   replicates fan out across cores with `rayon`; `into_par_iter().map().collect()`
//!   preserves index order, so the sequential and parallel builds are
//!   bit-identical (locked by golden tests in each caller).
//! * **No wasted sorting.** [`percentile_endpoints`] quickselects only the two
//!   order statistics a two-sided interval needs, in expected `O(n)`, rather than
//!   fully sorting the replicate distribution. BCa, which reads several
//!   data-dependent quantiles, sorts once itself.

use crate::Xorshift64;

/// A bootstrap resampling plan: how many replicates to draw and the RNG seed that
/// makes them reproducible. Construct one, then call [`Bootstrap::replicates`]
/// with the statistic to resample.
pub(crate) struct Bootstrap {
    n_resamples: usize,
    seed: u64,
}

impl Bootstrap {
    pub(crate) fn new(n_resamples: usize, seed: u64) -> Self {
        Self { n_resamples, seed }
    }

    /// Draw `n_resamples` replicates, evaluating `statistic` once per replicate on
    /// that replicate's own RNG substream.
    ///
    /// `statistic` must derive **all** of its randomness from the `&mut Xorshift64`
    /// it is handed — then each replicate is a pure function of its index, which is
    /// what makes the result order-independent and safe to evaluate across cores.
    /// The returned vector is in index order in both the sequential and parallel
    /// builds.
    pub(crate) fn replicates<F>(&self, statistic: F) -> Vec<f64>
    where
        F: Fn(&mut Xorshift64) -> f64 + Sync + Send,
    {
        let seed = self.seed;
        let draw = move |i: usize| {
            let mut rng = Xorshift64::for_resample(seed, i);
            statistic(&mut rng)
        };
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            (0..self.n_resamples).into_par_iter().map(draw).collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            (0..self.n_resamples).map(draw).collect()
        }
    }
}

/// The two-sided percentile endpoints of a replicate distribution at significance
/// `alpha` (i.e. the `alpha/2` and `1 - alpha/2` quantiles under the standard
/// floored-index convention), extracted by quickselect.
///
/// `replicates` is reordered in place and must be non-empty with all entries
/// finite (so `total_cmp` is a total order). After `select_nth_unstable_by(hi)`
/// the prefix `replicates[..hi]` holds exactly the `hi` smallest elements, so the
/// lower endpoint is a second quickselect within that prefix — no full sort.
pub(crate) fn percentile_endpoints(replicates: &mut [f64], alpha: f64) -> (f64, f64) {
    let n = replicates.len();
    debug_assert!(
        n >= 1,
        "percentile_endpoints requires at least one replicate"
    );
    let last = n - 1;
    let lo_idx = (((alpha / 2.0) * n as f64).floor() as usize).min(last);
    let hi_idx = (((1.0 - alpha / 2.0) * n as f64).floor() as usize).min(last);

    replicates.select_nth_unstable_by(hi_idx, |x, y| x.total_cmp(y));
    let upper = replicates[hi_idx];
    let lower = if lo_idx < hi_idx {
        replicates[..hi_idx].select_nth_unstable_by(lo_idx, |x, y| x.total_cmp(y));
        replicates[lo_idx]
    } else {
        replicates[lo_idx]
    };
    (lower, upper)
}
