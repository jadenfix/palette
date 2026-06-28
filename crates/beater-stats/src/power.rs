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
//! approximation to the McNemar/paired-proportion power. A dedicated
//! exact-McNemar power routine (over the discordant-pair binomial) is a possible
//! follow-on; see the TODO below. // TODO(#111): exact discordant-pair (McNemar)
//! power, in addition to the normal approximation used here.

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
pub fn required_sample_size(
    effect_size: f64,
    alpha: f64,
    power: f64,
) -> Result<usize, StatsError> {
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
pub fn minimum_detectable_effect(
    n: usize,
    alpha: f64,
    power: f64,
) -> Result<f64, StatsError> {
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
    }
}
