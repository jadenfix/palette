//! # Power / MDE / minimum-sample planning (ARCHITECTURE.md §10.3 #5)
//!
//! Before a deploy-gate may return *pass*, the comparison must have been
//! adequately powered to detect a meaningful regression; an underpowered
//! comparison returns *inconclusive*, never *pass* (§10.3 #5, §1 invariant).
//! When that happens the gate should not report a bare "underpowered" — it must
//! say *how* underpowered. This module supplies the two actionable numbers
//! [`beater-eval`](../../beater_eval/index.html) attaches to an inconclusive
//! verdict:
//!
//! * [`minimum_detectable_effect`] — the smallest effect the comparison *could*
//!   have resolved at its current sample size (regressions smaller than this are
//!   invisible at this N); and
//! * [`required_sample_size`] — how many paired observations would be needed to
//!   detect the *observed* (or any target) effect.
//!
//! [`achieved_power`] is the complementary query named in §10.3 #5: the power the
//! comparison actually had against a given effect.
//!
//! ## Model
//!
//! These are the standard **paired / one-sample normal-approximation** planning
//! formulas, expressed in **standardized** units (Cohen's *d* = mean difference
//! ÷ SD of the paired differences). With `z_{1-α/2}` and `z_{1-β}` the normal
//! quantiles (`β = 1 − power`):
//!
//! ```text
//! n   = ⌈ ( (z_{1-α/2} + z_{1-β}) / d )² ⌉            // required_sample_size
//! d   =   (z_{1-α/2} + z_{1-β}) / √n                  // minimum_detectable_effect (standardized)
//! 1-β = Φ( |d|·√n − z_{1-α/2} )                       // achieved_power
//! ```
//!
//! A caller working in a metric's own units multiplies the standardized MDE by
//! the SD of the paired differences to obtain the MDE in those units, and divides
//! an absolute effect by that SD to obtain the standardized effect for
//! [`required_sample_size`] — which is exactly what the gate does.
//!
//! ## Why the normal approximation (and not the exact noncentral-*t*)
//!
//! The normal-approximation `n` is the textbook closed form and is what makes the
//! inverse [`minimum_detectable_effect`] a clean reciprocal of
//! [`required_sample_size`]. It is mildly **anti-conservative** versus the exact
//! noncentral-*t* sample size (which adds the *t* degrees-of-freedom correction):
//! for *d* = 0.5 at α = 0.05, power = 0.8 the normal approximation yields **32**
//! pairs where the exact one-sample *t* calculation (e.g. G\*Power) yields **34**.
//! That gap shrinks to nothing as *n* grows. The gate uses these numbers as
//! *guidance attached to an inconclusive verdict*, not as a hard accept rule, so
//! the closed form is the right precision/complexity trade.
//!
//! ## Proportion / McNemar case
//!
//! For a paired binary outcome the same standardized machinery applies once the
//! effect is expressed as a standardized mean difference of the paired ±1/0
//! differences (the gate computes exactly this SD), which is the normal
//! approximation to the McNemar/paired-proportion power. In addition,
//! [`mcnemar_achieved_power`] and [`mcnemar_required_discordant`] compute the
//! **exact** discordant-pair (McNemar) power over the conditional binomial of the
//! discordant pairs — the sharper calculation §10.3 #5 calls for when the
//! discordant rate is known.

use crate::numerics::normal_quantile;
use crate::{normal_cdf, validate_alpha, StatsError};

/// The conventional power target for sample planning (§10.3 #5). A gate that does
/// not specify otherwise plans for an 80 % chance of detecting the effect.
pub const DEFAULT_POWER: f64 = 0.8;

/// Validate `power ∈ (0, 1)`, mirroring [`validate_alpha`].
fn validate_power(power: f64) -> Result<(), StatsError> {
    if power.is_finite() && power > 0.0 && power < 1.0 {
        Ok(())
    } else {
        Err(StatsError::InvalidParameter {
            name: "power",
            value: power,
        })
    }
}

/// Required number of paired observations to detect a **standardized** effect
/// (Cohen's *d*) at the given two-sided `alpha` and `power`, using the
/// paired/one-sample normal approximation.
///
/// ```text
/// n = ⌈ ( (z_{1-α/2} + z_{1-β}) / d )² ⌉ ,  β = 1 − power
/// ```
///
/// The result is at least `1`. `effect_size` is taken in absolute value, so a
/// regression (negative effect) and an equal-magnitude improvement need the same
/// N.
///
/// # Errors
///
/// * [`StatsError::InvalidAlpha`] when `alpha ∉ (0, 1)`.
/// * [`StatsError::InvalidParameter`] when `power ∉ (0, 1)`, or when
///   `effect_size` is zero or non-finite (a zero effect needs infinitely many
///   samples — there is no finite N to return).
///
/// # Example
///
/// ```
/// use beater_stats::required_sample_size;
///
/// // Detecting a half-SD effect at α = 0.05, power = 0.8 (paired/one-sample
/// // normal approximation) needs 32 pairs.
/// let n = required_sample_size(0.5, 0.05, 0.8).unwrap();
/// assert_eq!(n, 32);
/// ```
pub fn required_sample_size(effect_size: f64, alpha: f64, power: f64) -> Result<usize, StatsError> {
    validate_alpha(alpha)?;
    validate_power(power)?;
    if !effect_size.is_finite() || effect_size == 0.0 {
        return Err(StatsError::InvalidParameter {
            name: "effect_size",
            value: effect_size,
        });
    }

    let z_alpha = normal_quantile(1.0 - alpha / 2.0);
    let z_power = normal_quantile(power);
    let n = ((z_alpha + z_power) / effect_size.abs()).powi(2);

    // Round the continuous requirement up to whole samples. Snap values that are
    // an integer to within floating-point slop first, so the exact inverse of
    // `minimum_detectable_effect` round-trips (e.g. √n² = 32.0000…1 must not ceil
    // to 33).
    let rounded = if (n - n.round()).abs() < 1e-9 {
        n.round()
    } else {
        n.ceil()
    };
    Ok((rounded as usize).max(1))
}

/// Smallest **standardized** effect (Cohen's *d*) detectable with `n` paired
/// observations at the given two-sided `alpha` and `power` — the inverse of
/// [`required_sample_size`].
///
/// ```text
/// d = (z_{1-α/2} + z_{1-β}) / √n ,  β = 1 − power
/// ```
///
/// The value is in **SD units**; multiply by the SD of the paired differences to
/// express the MDE in the metric's own units.
///
/// # Errors
///
/// * [`StatsError::InvalidAlpha`] when `alpha ∉ (0, 1)`.
/// * [`StatsError::InvalidParameter`] when `power ∉ (0, 1)` or `n == 0`.
///
/// # Example
///
/// ```
/// use beater_stats::minimum_detectable_effect;
///
/// // At n = 32, α = 0.05, power = 0.8 the MDE is ≈ 0.5 SD — the inverse of the
/// // required-N example.
/// let d = minimum_detectable_effect(32, 0.05, 0.8).unwrap();
/// assert!((d - 0.495).abs() < 1e-2, "d = {d}");
/// ```
pub fn minimum_detectable_effect(n: usize, alpha: f64, power: f64) -> Result<f64, StatsError> {
    validate_alpha(alpha)?;
    validate_power(power)?;
    if n == 0 {
        return Err(StatsError::InvalidParameter {
            name: "n",
            value: 0.0,
        });
    }

    let z_alpha = normal_quantile(1.0 - alpha / 2.0);
    let z_power = normal_quantile(power);
    Ok((z_alpha + z_power) / (n as f64).sqrt())
}

/// Statistical power actually achieved against a **standardized** effect
/// `effect_size` with `n` paired observations at two-sided `alpha`
/// (the §10.3 #5 `achieved_power` query):
///
/// ```text
/// power = Φ( |d|·√n − z_{1-α/2} )
/// ```
///
/// Returns a value in `[0, 1)` (a one-sided lower bound on the two-sided power,
/// which is the standard planning approximation). A zero effect yields exactly
/// `alpha / 2`, the size of the test.
///
/// # Errors
///
/// * [`StatsError::InvalidAlpha`] when `alpha ∉ (0, 1)`.
/// * [`StatsError::InvalidParameter`] when `n == 0` or `effect_size` is
///   non-finite.
pub fn achieved_power(n: usize, effect_size: f64, alpha: f64) -> Result<f64, StatsError> {
    validate_alpha(alpha)?;
    if n == 0 {
        return Err(StatsError::InvalidParameter {
            name: "n",
            value: 0.0,
        });
    }
    if !effect_size.is_finite() {
        return Err(StatsError::InvalidParameter {
            name: "effect_size",
            value: effect_size,
        });
    }

    let z_alpha = normal_quantile(1.0 - alpha / 2.0);
    let lambda = effect_size.abs() * (n as f64).sqrt();
    Ok(normal_cdf(lambda - z_alpha))
}

/// Validate a probability in the open interval `(0, 1)`.
fn validate_unit_prob(name: &'static str, value: f64) -> Result<(), StatsError> {
    if value.is_finite() && value > 0.0 && value < 1.0 {
        Ok(())
    } else {
        Err(StatsError::InvalidParameter { name, value })
    }
}

/// The exact-McNemar critical count for `d` discordant pairs at two-sided
/// `alpha`: the largest `k` with `2·P(X ≤ k | X ~ Binomial(d, ½)) ≤ alpha`, so
/// the rejection region is `{k ≤ k_lo} ∪ {k ≥ d − k_lo}`. `None` when even
/// `k = 0` cannot reject. The lower CDF is accumulated with the *same* log-space
/// recurrence as [`crate::numerics::binomial_lower_tail_half`] (same terms, same
/// summation order), so the region agrees exactly with the p-value
/// [`crate::mcnemar_exact_p`] reports. The scan stops by `k = ⌈d/2⌉`, where the
/// doubled tail is ≥ 1, so it costs `O(k_lo)` — never more than `O(d/2)`.
fn mcnemar_critical_count(d: u64, alpha: f64) -> Option<u64> {
    let ln_half_pow_n = d as f64 * 0.5_f64.ln();
    let mut ln_choose = 0.0_f64;
    let mut cdf0 = ln_half_pow_n.exp(); // P(X ≤ 0) = 2^{−d}
    let mut k_lo = None;
    let mut k = 0_u64;
    while 2.0 * cdf0 <= alpha && k <= d / 2 {
        k_lo = Some(k);
        k += 1;
        ln_choose += ((d - k + 1) as f64).ln() - (k as f64).ln();
        cdf0 += (ln_choose + ln_half_pow_n).exp();
    }
    k_lo
}

/// **Exact** power of the two-sided exact-McNemar (sign) test, *conditional on*
/// `n_discordant` discordant pairs, when the true probability that a discordant
/// pair favours the candidate is `pi` (§10.3 #5).
///
/// The rejection region is `{k : exact two-sided p(k, D−k) ≤ alpha}` over the `D`
/// discordant pairs (the same exact p-value the gate reports), and the power is
/// the alternative-hypothesis mass on that region under `Binomial(D, pi)`.
/// Because the region is the symmetric pair of tails `{k ≤ k_lo} ∪ {k ≥ D − k_lo}`,
/// the power is two `O(k_lo)` log-space tail sums — `O(D)` in the worst case,
/// not the `O(D²)` of testing every `k`'s p-value separately.
///
/// Returns `0.0` when `n_discordant == 0` (no discordant pair can ever reject).
///
/// # Errors
///
/// [`StatsError::InvalidAlpha`] for `alpha ∉ (0, 1)`, or
/// [`StatsError::InvalidParameter`] for `pi ∉ (0, 1)`.
///
/// # Example
///
/// ```
/// use beater_stats::mcnemar_achieved_power;
///
/// // 40 discordant pairs strongly favouring the candidate (pi = 0.8): the exact
/// // sign test is very well powered.
/// let power = mcnemar_achieved_power(40, 0.8, 0.05).unwrap();
/// assert!(power > 0.95, "power = {power}");
/// ```
pub fn mcnemar_achieved_power(n_discordant: u64, pi: f64, alpha: f64) -> Result<f64, StatsError> {
    validate_alpha(alpha)?;
    validate_unit_prob("pi", pi)?;
    if n_discordant == 0 {
        return Ok(0.0);
    }
    let d = n_discordant;
    let Some(k_lo) = mcnemar_critical_count(d, alpha) else {
        return Ok(0.0);
    };
    // Alternative-hypothesis mass on both rejection tails, sharing one binomial-
    // coefficient recurrence: for j = 0..=k_lo the lower-tail term is
    // C(d,j)·πʲ(1−π)^{d−j} and the upper-tail term (k = d − j) is
    // C(d,j)·π^{d−j}(1−π)ʲ. k_lo < d/2 always (the doubled H0 tail at d/2 is
    // ≥ 1 > α), so the two tails never overlap.
    let ln_pi = pi.ln();
    let ln_q = (1.0 - pi).ln();
    let d_f = d as f64;
    let mut ln_choose = 0.0_f64;
    let mut power = 0.0_f64;
    for j in 0..=k_lo {
        if j > 0 {
            ln_choose += ((d - j + 1) as f64).ln() - (j as f64).ln();
        }
        let j_f = j as f64;
        power += (ln_choose + j_f * ln_pi + (d_f - j_f) * ln_q).exp();
        power += (ln_choose + (d_f - j_f) * ln_pi + j_f * ln_q).exp();
    }
    Ok(power.clamp(0.0, 1.0))
}

/// Smallest number of **discordant pairs** needed for the exact-McNemar test to
/// reach `power_target` against a discordant-favouring probability `pi`, at
/// two-sided `alpha` (§10.3 #5).
///
/// Note this is the count of *discordant* pairs, not total cases: the total `N`
/// also depends on the discordant rate `b + c` over all pairs, which the caller
/// scales separately.
///
/// # Sawtooth non-monotonicity
///
/// The exact test's power is **not** monotone in `D`: each time `D` grows past a
/// point where the discrete critical count cannot yet advance, the power *dips*
/// before recovering (e.g. at `pi = 0.8, α = 0.05` the power first crosses 80 %
/// at `D = 20`, but `D = 21` has power 0.769 and `D = 22` has 0.733). Returning
/// the first crossing would therefore hand back a `D` at which collecting *one
/// more pair* makes the study underpowered. This function instead returns the
/// smallest `D` that starts a run of 64 consecutive sizes all meeting the
/// target, so the recommendation is robust to the caller landing a few pairs
/// above the plan. (The window is a pragmatic guard: observed dips
/// extend a handful of sizes past a crossing, far less than 64.)
///
/// # Cost
///
/// Each power evaluation is `O(D)` (see [`mcnemar_achieved_power`]), so the scan
/// is `O(D_req²)` overall — comfortable for any realistic planning target. A
/// normal-approximation precheck rejects effectively-null `pi` (whose exact
/// requirement would exceed the internal cap) before scanning.
///
/// # Errors
///
/// [`StatsError::InvalidAlpha`], [`StatsError::InvalidParameter`] for `pi ∉ (0, 1)`
/// or `pi == 0.5` (no effect — unbounded), or `power_target ∉ (0, 1)`, or when the
/// target is not reachable within the internal cap (a degenerate request).
pub fn mcnemar_required_discordant(
    pi: f64,
    alpha: f64,
    power_target: f64,
) -> Result<usize, StatsError> {
    validate_alpha(alpha)?;
    validate_power(power_target)?;
    validate_unit_prob("pi", pi)?;
    if (pi - 0.5).abs() < 1e-12 {
        return Err(StatsError::InvalidParameter {
            name: "pi",
            value: pi,
        });
    }
    const CAP: u64 = 100_000;
    /// Consecutive sizes that must all meet the target before the run's start is
    /// returned; see "Sawtooth non-monotonicity" above.
    const STABLE_WINDOW: u64 = 64;

    // Normal-approximation estimate of the required D (sign-test planning form:
    // H0 SD is ½, H1 SD is √(π(1−π))). If even the approximation is far beyond
    // the cap, refuse immediately instead of burning an O(CAP²) scan.
    let z_alpha = normal_quantile(1.0 - alpha / 2.0);
    let z_power = normal_quantile(power_target);
    let delta = (pi - 0.5).abs();
    let d_norm = ((z_alpha * 0.5 + z_power * (pi * (1.0 - pi)).sqrt()) / delta).powi(2);
    if !d_norm.is_finite() || d_norm > 1.5 * CAP as f64 {
        return Err(StatsError::InvalidParameter {
            name: "n_discordant",
            value: CAP as f64,
        });
    }

    let mut run_start: Option<u64> = None;
    let mut run_len = 0_u64;
    for d in 1..=CAP + STABLE_WINDOW {
        if mcnemar_achieved_power(d, pi, alpha)? >= power_target {
            if run_start.is_none() {
                run_start = Some(d);
                run_len = 0;
            }
            run_len += 1;
            if run_len >= STABLE_WINDOW {
                // run_start is Some by construction inside this branch.
                if let Some(start) = run_start {
                    if start <= CAP {
                        return Ok(start as usize);
                    }
                    break;
                }
            }
        } else {
            run_start = None;
            run_len = 0;
        }
    }
    Err(StatsError::InvalidParameter {
        name: "n_discordant",
        value: CAP as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── required_sample_size — pinned to textbook values ──────────────────────

    /// Canonical planning result: detecting a 0.5 SD effect at α = 0.05,
    /// power = 0.8 with the paired/one-sample normal approximation.
    /// (z_{0.975} + z_{0.8})² / 0.5² = (1.95996 + 0.84162)² / 0.25 = 31.40 → 32.
    /// The exact one-sample noncentral-t value (G*Power) is 34; the normal
    /// approximation is mildly anti-conservative, as documented.
    #[test]
    fn required_n_half_sd_textbook() {
        let n = required_sample_size(0.5, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert!((31..=33).contains(&n), "n = {n}, expected ~32");
    }

    /// A smaller effect needs many more samples; a larger one needs fewer.
    #[test]
    fn required_n_monotone_in_effect() {
        let small = required_sample_size(0.2, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        let large = required_sample_size(1.0, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        // 0.2 SD ≈ 197 pairs; 1.0 SD ≈ 8 pairs.
        assert!((196..=199).contains(&small), "small = {small}");
        assert!((7..=9).contains(&large), "large = {large}");
        assert!(small > large);
    }

    /// Sign of the effect does not change the required N.
    #[test]
    fn required_n_uses_magnitude() {
        let pos = required_sample_size(0.5, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        let neg = required_sample_size(-0.5, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(pos, neg);
    }

    /// Higher power and tighter alpha both raise the required N.
    #[test]
    fn required_n_grows_with_power_and_alpha() {
        let base = required_sample_size(0.5, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        let more_power = required_sample_size(0.5, 0.05, 0.9).unwrap_or_else(|err| panic!("{err}"));
        let tighter = required_sample_size(0.5, 0.01, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert!(more_power > base, "{more_power} !> {base}");
        assert!(tighter > base, "{tighter} !> {base}");
    }

    // ── minimum_detectable_effect — inverse round-trip ────────────────────────

    /// MDE at n = 32 should be ≈ 0.5 SD — the inverse of the required-N pin.
    #[test]
    fn mde_round_trips_required_n() {
        let d = minimum_detectable_effect(32, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert!((d - 0.4953).abs() < 1e-3, "d = {d}");
        // And re-planning for that exact d recovers n = 32.
        let n = required_sample_size(d, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(n, 32);
    }

    /// MDE shrinks like 1/√n.
    #[test]
    fn mde_shrinks_with_n() {
        let d32 = minimum_detectable_effect(32, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        let d128 = minimum_detectable_effect(128, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        // Quadrupling n halves the MDE.
        assert!((d32 / d128 - 2.0).abs() < 1e-9, "ratio = {}", d32 / d128);
    }

    // ── achieved_power ────────────────────────────────────────────────────────

    /// At the planned n the achieved power should be ≈ the target power.
    #[test]
    fn achieved_power_matches_target_at_planned_n() {
        let p = achieved_power(32, 0.5, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!((p - 0.8).abs() < 0.02, "power = {p}");
    }

    /// The rounded required sample size must not come back underpowered for the
    /// target it was planned against. This is the A6 gate invariant in miniature:
    /// a caller can ask for the minimum N, then verify `achieved_power >= target`
    /// before allowing a pass verdict.
    #[test]
    fn required_n_meets_requested_power() {
        for (effect, alpha, target_power) in [
            (0.2, 0.05, 0.8),
            (0.5, 0.05, 0.8),
            (0.5, 0.01, 0.8),
            (0.5, 0.05, 0.9),
            (1.0, 0.05, 0.8),
        ] {
            let n = required_sample_size(effect, alpha, target_power)
                .unwrap_or_else(|err| panic!("{err}"));
            let planned = achieved_power(n, effect, alpha).unwrap_or_else(|err| panic!("{err}"));
            assert!(
                planned + 1e-12 >= target_power,
                "n={n}, effect={effect}, alpha={alpha}, target={target_power}, achieved={planned}"
            );
        }
    }

    #[test]
    fn achieved_power_grows_with_n_and_effect_size() {
        let base = achieved_power(16, 0.4, 0.05).unwrap_or_else(|err| panic!("{err}"));
        let more_n = achieved_power(64, 0.4, 0.05).unwrap_or_else(|err| panic!("{err}"));
        let bigger_effect = achieved_power(16, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));

        assert!(more_n > base, "{more_n} !> {base}");
        assert!(bigger_effect > base, "{bigger_effect} !> {base}");
    }

    /// `minimum_detectable_effect` is the closed-form inverse of
    /// `achieved_power`: at that MDE and N, the achieved power should be the
    /// target power.
    #[test]
    fn mde_achieves_requested_power() {
        for (n, alpha, target_power) in [(16, 0.05, 0.8), (64, 0.01, 0.8), (128, 0.05, 0.9)] {
            let mde = minimum_detectable_effect(n, alpha, target_power)
                .unwrap_or_else(|err| panic!("{err}"));
            let achieved = achieved_power(n, mde, alpha).unwrap_or_else(|err| panic!("{err}"));
            assert!(
                (achieved - target_power).abs() < 1e-6,
                "n={n}, alpha={alpha}, target={target_power}, mde={mde}, achieved={achieved}"
            );
        }
    }

    /// A zero effect can only be "detected" at the test's size (α/2 one-sided).
    #[test]
    fn achieved_power_zero_effect_is_test_size() {
        let p = achieved_power(50, 0.0, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!((p - 0.025).abs() < 1e-3, "power = {p}");
    }

    // ── validation ────────────────────────────────────────────────────────────

    #[test]
    fn rejects_bad_alpha() {
        assert!(matches!(
            required_sample_size(0.5, 0.0, 0.8),
            Err(StatsError::InvalidAlpha(_))
        ));
        assert!(matches!(
            minimum_detectable_effect(10, 1.0, 0.8),
            Err(StatsError::InvalidAlpha(_))
        ));
        assert!(matches!(
            achieved_power(10, 0.5, 1.5),
            Err(StatsError::InvalidAlpha(_))
        ));
    }

    #[test]
    fn rejects_bad_power() {
        assert!(matches!(
            required_sample_size(0.5, 0.05, 0.0),
            Err(StatsError::InvalidParameter { name: "power", .. })
        ));
        assert!(matches!(
            minimum_detectable_effect(10, 0.05, 1.0),
            Err(StatsError::InvalidParameter { name: "power", .. })
        ));
    }

    #[test]
    fn rejects_bad_effect_and_n() {
        assert!(matches!(
            required_sample_size(0.0, 0.05, 0.8),
            Err(StatsError::InvalidParameter {
                name: "effect_size",
                ..
            })
        ));
        assert!(matches!(
            required_sample_size(f64::NAN, 0.05, 0.8),
            Err(StatsError::InvalidParameter {
                name: "effect_size",
                ..
            })
        ));
        assert!(matches!(
            minimum_detectable_effect(0, 0.05, 0.8),
            Err(StatsError::InvalidParameter { name: "n", .. })
        ));
        assert!(matches!(
            achieved_power(0, 0.5, 0.05),
            Err(StatsError::InvalidParameter { name: "n", .. })
        ));
        assert!(matches!(
            required_sample_size(f64::INFINITY, 0.05, 0.8),
            Err(StatsError::InvalidParameter {
                name: "effect_size",
                ..
            })
        ));
        assert!(matches!(
            achieved_power(10, f64::NEG_INFINITY, 0.05),
            Err(StatsError::InvalidParameter {
                name: "effect_size",
                ..
            })
        ));
    }

    // ── exact McNemar power (resolves the #111 TODO) ──────────────────────────

    #[test]
    fn mcnemar_power_zero_discordant_is_zero() {
        let p = mcnemar_achieved_power(0, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(p, 0.0);
    }

    #[test]
    fn mcnemar_power_increases_with_discordant_count() {
        let small = mcnemar_achieved_power(10, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));
        let large = mcnemar_achieved_power(60, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(large > small, "{large} !> {small}");
        assert!(large > 0.95, "well-powered at 60 discordant: {large}");
        assert!((0.0..=1.0).contains(&small));
    }

    /// Pin the exact conditional power against reference values computed with
    /// exact rational arithmetic (Python `fractions` over the full binomial),
    /// so the `O(k_lo)` two-tail evaluation provably matches the brute-force
    /// "test every k's exact p-value" definition.
    #[test]
    fn mcnemar_power_matches_exact_rational_reference() {
        let cases: [(u64, f64, f64); 6] = [
            (10, 0.8, 0.375_813_836_8),
            (25, 0.8, 0.890_877_233_059_705),
            (40, 0.8, 0.980_592_631_171_619),
            (60, 0.8, 0.997_956_461_188_856),
            (50, 0.65, 0.505_979_935_486_655),
            (120, 0.65, 0.892_428_618_525_588),
        ];
        for (d, pi, want) in cases {
            let got = mcnemar_achieved_power(d, pi, 0.05).unwrap_or_else(|err| panic!("{err}"));
            assert!(
                (got - want).abs() < 1e-12,
                "power(D={d}, pi={pi}) = {got}, want {want}"
            );
        }
    }

    /// The exact power is NOT monotone in D (discrete critical-value sawtooth):
    /// at pi = 0.8 it first crosses 80 % at D = 20 but dips back below at 21–22.
    /// `mcnemar_required_discordant` must not return a first-crossing D whose
    /// immediate successors are underpowered.
    #[test]
    fn mcnemar_required_discordant_is_sawtooth_safe() {
        // Exhibit the sawtooth itself.
        let at_20 = mcnemar_achieved_power(20, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));
        let at_21 = mcnemar_achieved_power(21, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(at_20 >= 0.8, "first crossing at D=20: {at_20}");
        assert!(at_21 < 0.8, "sawtooth dip at D=21: {at_21}");

        // The recommendation must start a stable run, not the first crossing.
        let d = mcnemar_required_discordant(0.8, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(d, 23, "stable-run answer (reference: exact rational scan)");
        for extra in 0..64u64 {
            let p = mcnemar_achieved_power(d as u64 + extra, 0.8, 0.05)
                .unwrap_or_else(|err| panic!("{err}"));
            assert!(
                p >= 0.8,
                "D={} must stay powered, got {p}",
                d as u64 + extra
            );
        }

        // Independent second effect size: stable answer 97, not the crossing 90.
        let d = mcnemar_required_discordant(0.65, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(d, 97);
    }

    /// An effectively-null pi whose exact requirement exceeds the cap must be
    /// refused quickly by the normal-approximation precheck.
    #[test]
    fn mcnemar_required_discordant_refuses_untenable_effect() {
        assert!(matches!(
            mcnemar_required_discordant(0.500_5, 0.05, 0.8),
            Err(StatsError::InvalidParameter {
                name: "n_discordant",
                ..
            })
        ));
    }

    #[test]
    fn mcnemar_power_at_null_is_near_alpha() {
        // pi = 0.5 is the null: the test should reject at roughly its size.
        let p = mcnemar_achieved_power(40, 0.5, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(p <= 0.05 + 1e-9, "size should not exceed alpha: {p}");
    }

    #[test]
    fn mcnemar_required_discordant_round_trips_power() {
        let d = mcnemar_required_discordant(0.8, 0.05, 0.8).unwrap_or_else(|err| panic!("{err}"));
        // The returned D clears the target; D-1 does not (it is the smallest).
        let at = mcnemar_achieved_power(d as u64, 0.8, 0.05).unwrap_or_else(|err| panic!("{err}"));
        assert!(at >= 0.8, "achieved {at} at D={d}");
        if d > 1 {
            let below = mcnemar_achieved_power(d as u64 - 1, 0.8, 0.05)
                .unwrap_or_else(|err| panic!("{err}"));
            assert!(below < 0.8, "D-1 should be underpowered: {below}");
        }
    }

    #[test]
    fn mcnemar_required_discordant_rejects_null_effect() {
        assert!(matches!(
            mcnemar_required_discordant(0.5, 0.05, 0.8),
            Err(StatsError::InvalidParameter { name: "pi", .. })
        ));
    }
}
