//! Minimal special-function numerics, hand-rolled so this crate needs no
//! heavyweight stats/linear-algebra dependency. Each routine is a textbook
//! algorithm pinned to known values in the unit tests below.
//!
//! We need exactly four things for the paired gate:
//! - Student's t two-sided tail probability (the p-value) and its quantile (the
//!   CI critical value) — both built on the regularized incomplete beta;
//! - the standard-normal quantile (the McNemar difference-CI critical value);
//! - the exact Binomial(n, ½) lower tail (the exact McNemar p-value).
//!
//! References: Lanczos (1964) for `ln_gamma`; Numerical Recipes §6.4 for the
//! incomplete-beta continued fraction; Acklam's algorithm for the normal quantile;
//! Cody (1969) for `erfc`.

use std::f64::consts::PI;

/// Complementary error function `erfc(x)` via W. J. Cody's rational Chebyshev
/// approximations (Math. Comp. 23, 1969; the `CALERF` reference implementation),
/// accurate to ~1 ulp in *relative* terms over the whole real line.
///
/// Relative accuracy is the load-bearing property: the previous
/// Abramowitz & Stegun §7.1.26 polynomial had only *absolute* error ≤ 1.5×10⁻⁷,
/// so any tail probability below ~10⁻⁷ (a z-statistic beyond ≈ 5.2) came back
/// with O(1) relative error — a reported p-value of 10⁻¹² could be wrong by
/// orders of magnitude. Cody's three-regime form keeps ~15 significant digits
/// all the way down to the underflow horizon (`x ≈ 26.5`, p ≈ 10⁻³⁰⁰).
// Coefficients are kept digit-for-digit as published (CALERF), so they can be
// audited against the reference; the extra digits round to the same f64.
#[allow(clippy::excessive_precision)]
pub fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let y = x.abs();
    let result = if y <= 0.46875 {
        // erf(x) = x · P(y²)/Q(y²); erfc = 1 − erf.
        const A: [f64; 5] = [
            3.161_123_743_870_565_6,
            1.138_641_541_510_501_56e2,
            3.774_852_376_853_020_2e2,
            3.209_377_589_138_469_47e3,
            1.857_777_061_846_031_53e-1,
        ];
        const B: [f64; 4] = [
            2.360_129_095_234_412_09e1,
            2.440_246_379_344_441_73e2,
            1.282_616_526_077_372_28e3,
            2.844_236_833_439_170_62e3,
        ];
        let z = if y > 1.11e-16 { y * y } else { 0.0 };
        let mut num = A[4] * z;
        let mut den = z;
        for i in 0..3 {
            num = (num + A[i]) * z;
            den = (den + B[i]) * z;
        }
        // erf on this range; the sign of x is folded in at the end.
        return 1.0 - x * (num + A[3]) / (den + B[3]);
    } else if y <= 4.0 {
        // erfc(y) = e^{−y²} · P(y)/Q(y).
        const C: [f64; 9] = [
            5.641_884_969_886_700_9e-1,
            8.883_149_794_388_375_9,
            6.611_919_063_714_162_95e1,
            2.986_351_381_974_001_31e2,
            8.819_522_212_417_690_9e2,
            1.712_047_612_634_070_58e3,
            2.051_078_377_826_071_47e3,
            1.230_339_354_797_997_25e3,
            2.153_115_354_744_038_46e-8,
        ];
        const D: [f64; 8] = [
            1.574_492_611_070_983_47e1,
            1.176_939_508_913_124_99e2,
            5.371_811_018_620_098_58e2,
            1.621_389_574_566_690_19e3,
            3.290_799_235_733_459_63e3,
            4.362_619_090_143_247_16e3,
            3.439_367_674_143_721_64e3,
            1.230_339_354_803_749_42e3,
        ];
        let mut num = C[8] * y;
        let mut den = y;
        for i in 0..7 {
            num = (num + C[i]) * y;
            den = (den + D[i]) * y;
        }
        exp_neg_sq(y) * (num + C[7]) / (den + D[7])
    } else if y < 26.543 {
        // erfc(y) = e^{−y²}/y · (1/√π − r(1/y²)/y² · …), Cody region 3.
        const P: [f64; 6] = [
            3.053_266_349_612_323_44e-1,
            3.603_448_999_498_044_39e-1,
            1.257_817_261_112_292_46e-1,
            1.608_378_514_874_227_66e-2,
            6.587_491_615_298_378_03e-4,
            1.631_538_713_730_209_78e-2,
        ];
        const Q: [f64; 5] = [
            2.568_520_192_289_822_42,
            1.872_952_849_923_460_47,
            5.279_051_029_514_284_12e-1,
            6.051_834_131_244_131_91e-2,
            2.335_204_976_268_691_85e-3,
        ];
        const ONE_OVER_SQRT_PI: f64 = 5.641_895_835_477_562_9e-1;
        let z = 1.0 / (y * y);
        let mut num = P[5] * z;
        let mut den = z;
        for i in 0..4 {
            num = (num + P[i]) * z;
            den = (den + Q[i]) * z;
        }
        let r = z * (num + P[4]) / (den + Q[4]);
        exp_neg_sq(y) * (ONE_OVER_SQRT_PI - r) / y
    } else {
        // Beyond the double-precision underflow horizon.
        0.0
    };
    if x < 0.0 { 2.0 - result } else { result }
}

/// `e^{−y²}` computed as `e^{−q²}·e^{−(y−q)(y+q)}` with `q = ⌊16y⌋/16`, Cody's
/// argument-splitting trick: `q²` is exact in binary (q has ≤ 4 fractional
/// bits), so the rounding error of forming `y²` directly — which `exp` would
/// amplify by `y²` ulps — is avoided.
fn exp_neg_sq(y: f64) -> f64 {
    let q = (y * 16.0).trunc() / 16.0;
    let del = (y - q) * (y + q);
    (-q * q).exp() * (-del).exp()
}

/// Natural log of the gamma function via the Lanczos approximation (g = 7).
pub fn ln_gamma(x: f64) -> f64 {
    // Reflection for the left half-plane keeps the series in its accurate range.
    if x < 0.5 {
        return (PI / (PI * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    const G: f64 = 7.0;
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let x = x - 1.0;
    let t = x + G + 0.5;
    let mut series = COEFFS[0];
    for (i, &c) in COEFFS.iter().enumerate().skip(1) {
        series += c / (x + i as f64);
    }
    0.5 * (2.0 * PI).ln() + (x + 0.5) * t.ln() - t + series.ln()
}

/// Continued-fraction kernel for the regularized incomplete beta (NR `betacf`).
fn beta_continued_fraction(x: f64, a: f64, b: f64) -> f64 {
    const MAX_ITER: usize = 300;
    const EPS: f64 = 3.0e-14;
    const TINY: f64 = 1.0e-300;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=MAX_ITER {
        let m_f = m as f64;
        let two_m = 2.0 * m_f;
        // Even step.
        let aa = m_f * (b - m_f) * x / ((qam + two_m) * (a + two_m));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        h *= d * c;
        // Odd step.
        let aa = -(a + m_f) * (qab + m_f) * x / ((a + two_m) * (qap + two_m));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < EPS {
            break;
        }
    }
    h
}

/// Regularized incomplete beta `I_x(a, b)`.
pub fn reg_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let ln_beta = ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b);
    let front = (ln_beta + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_continued_fraction(x, a, b) / a
    } else {
        1.0 - front * beta_continued_fraction(1.0 - x, b, a) / b
    }
}

/// Two-sided tail probability `P(|T| ≥ |t|)` for a Student's t with `df` degrees
/// of freedom — i.e. the two-sided p-value. Equals `I_x(df/2, 1/2)` with
/// `x = df / (df + t²)`.
pub fn students_t_two_sided_p(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return f64::NAN;
    }
    let x = df / (df + t * t);
    reg_incomplete_beta(x, df / 2.0, 0.5).clamp(0.0, 1.0)
}

/// CDF of a Student's t with `df` degrees of freedom.
fn students_t_cdf(t: f64, df: f64) -> f64 {
    let half_two_sided = 0.5 * students_t_two_sided_p(t, df);
    if t >= 0.0 {
        1.0 - half_two_sided
    } else {
        half_two_sided
    }
}

/// Quantile (inverse CDF) of a Student's t with `df` degrees of freedom, by
/// bisection on the monotone CDF. `p` is clamped to the open interval.
pub fn students_t_quantile(p: f64, df: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    // Expand a symmetric bracket until it straddles the quantile, so extreme
    // tails (tiny alpha at low df) are not silently clipped, then bisect.
    let (mut lo, mut hi) = (-1.0_f64, 1.0_f64);
    while lo > -f64::MAX / 2.0 && students_t_cdf(lo, df) > p {
        lo *= 2.0;
    }
    while hi < f64::MAX / 2.0 && students_t_cdf(hi, df) < p {
        hi *= 2.0;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if students_t_cdf(mid, df) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Standard-normal quantile via Acklam's rational approximation
/// (|relative error| < 1.2e-9 across the open interval).
pub fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.024_25;
    const P_HIGH: f64 = 1.0 - P_LOW;
    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// Exact Binomial(n, ½) lower tail `P(X ≤ k)`, summed in log-space for stability.
///
/// The log binomial coefficient is carried forward by the recurrence
/// `ln C(n,i) = ln C(n,i−1) + ln(n−i+1) − ln(i)`, so each term costs two cheap
/// `ln` calls instead of two `ln_gamma` evaluations — an O(k) gamma-free sum that
/// matters when the discordant-pair count `k` is large.
pub fn binomial_lower_tail_half(k: u64, n: u64) -> f64 {
    let ln_half_pow_n = n as f64 * 0.5_f64.ln();
    let upper = k.min(n);
    // ln C(n,0) = 0, so the i = 0 term is exp(n·ln½).
    let mut ln_choose = 0.0_f64;
    let mut sum = ln_half_pow_n.exp();
    for i in 1..=upper {
        ln_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        sum += (ln_choose + ln_half_pow_n).exp();
    }
    sum.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn ln_gamma_known_values() {
        assert!(close(ln_gamma(0.5), 0.572_364_942_9, 1e-9)); // ln(√π)
        assert!(close(ln_gamma(1.0), 0.0, 1e-12));
        assert!(close(ln_gamma(5.0), 24.0_f64.ln(), 1e-10)); // ln(4!)
        assert!(close(ln_gamma(10.0), 362_880.0_f64.ln(), 1e-9)); // ln(9!)
    }

    #[test]
    fn incomplete_beta_symmetry_and_known() {
        // I_x(a,b) + I_{1-x}(b,a) = 1.
        let v = reg_incomplete_beta(0.3, 2.0, 5.0);
        let w = reg_incomplete_beta(0.7, 5.0, 2.0);
        assert!(close(v + w, 1.0, 1e-10));
        // I_0.5(1,1) = 0.5 (uniform).
        assert!(close(reg_incomplete_beta(0.5, 1.0, 1.0), 0.5, 1e-12));
    }

    #[test]
    fn students_t_two_sided_p_matches_textbook() {
        // t = 5.745, df = 9 (the 1..=10 differences example): two-sided p ≈ 2.78e-4.
        let p = students_t_two_sided_p(5.745, 9.0);
        assert!(close(p, 2.78e-4, 2e-5), "p={p}");
        // t = 0 -> p = 1.
        assert!(close(students_t_two_sided_p(0.0, 9.0), 1.0, 1e-12));
    }

    #[test]
    fn students_t_quantile_matches_tables() {
        // 97.5th percentile, df=9 -> 2.262; df=1 -> 12.706; large df -> ~1.96.
        assert!(close(students_t_quantile(0.975, 9.0), 2.262, 1e-3));
        assert!(close(students_t_quantile(0.975, 1.0), 12.706, 2e-3));
        assert!(close(students_t_quantile(0.975, 1.0e6), 1.960, 1e-3));
    }

    #[test]
    fn normal_quantile_matches_tables() {
        assert!(close(normal_quantile(0.975), 1.959_963_98, 1e-6));
        assert!(close(normal_quantile(0.995), 2.575_829_3, 1e-6));
        assert!(close(normal_quantile(0.5), 0.0, 1e-9));
    }

    /// Cody erfc must hold ~13+ significant digits across all three regimes and
    /// down to the underflow horizon. References are correctly-rounded values
    /// (IEEE `erfc`, cross-checked against mpmath), kept digit-for-digit.
    #[test]
    #[allow(clippy::excessive_precision)]
    fn erfc_relative_accuracy_across_regimes() {
        let cases: [(f64, f64); 10] = [
            (0.1, 8.875_370_839_817_151_58e-1),     // region 1 (series)
            (0.46875, 5.073_865_267_820_619_75e-1), // region boundary
            (0.5, 4.795_001_221_869_534_81e-1),     // region 2
            (1.0, 1.572_992_070_502_851_34e-1),
            (2.0, 4.677_734_981_047_265_36e-3),
            (3.0, 2.209_049_699_858_543_78e-5),
            (4.0, 1.541_725_790_028_002_0e-8), // region 2/3 boundary
            (5.0, 1.537_459_794_428_035_14e-12), // region 3
            (10.0, 2.088_487_583_762_544_88e-45),
            (20.0, 5.395_865_611_607_900_48e-176),
        ];
        for (x, want) in cases {
            let got = erfc(x);
            let rel = ((got - want) / want).abs();
            assert!(
                rel < 1e-12,
                "erfc({x}) = {got:e}, want {want:e}, rel {rel:e}"
            );
            // Mirror identity erfc(−x) = 2 − erfc(x).
            let neg = erfc(-x);
            assert!(
                ((neg - (2.0 - want)) / 2.0).abs() < 1e-14,
                "erfc({}) = {neg}",
                -x
            );
        }
        // Beyond the underflow horizon and at the extremes.
        assert_eq!(erfc(27.0), 0.0);
        assert_eq!(erfc(f64::INFINITY), 0.0);
        assert_eq!(erfc(f64::NEG_INFINITY), 2.0);
        assert!(erfc(f64::NAN).is_nan());
        assert!((erfc(0.0) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn binomial_tail_known_values() {
        assert!(close(binomial_lower_tail_half(0, 3), 0.125, 1e-12)); // 0.5^3
        assert!(close(binomial_lower_tail_half(0, 5), 0.031_25, 1e-12)); // 0.5^5
        assert!(close(binomial_lower_tail_half(5, 10), 0.623_046_875, 1e-9));
        assert!(close(binomial_lower_tail_half(10, 10), 1.0, 1e-12));
    }

    /// The incremental log-binomial recurrence must stay accurate at large `n`,
    /// where it carries `ln C(n,i)` across many iterations. Reference values from
    /// exact `sum(comb(n,i))/2^n`. The full tail `P(X ≤ n) = 1` also pins that the
    /// per-term coefficients still sum to one after thousands of steps.
    #[test]
    fn binomial_tail_large_n_recurrence_is_accurate() {
        // P(X ≤ 500 | n = 1000): 0.5·(1 + P(X = 500)).
        assert!(close(
            binomial_lower_tail_half(500, 1000),
            0.512_612_509_089_180_4,
            1e-9
        ));
        // A deep lower tail far below the median.
        assert!(close(
            binomial_lower_tail_half(50, 200),
            4.196_510_437_802_38e-13,
            1e-15
        ));
        // The whole distribution sums to 1 even after n steps of the recurrence.
        assert!(close(binomial_lower_tail_half(1000, 1000), 1.0, 1e-9));
        assert!(close(binomial_lower_tail_half(5000, 5000), 1.0, 1e-9));
    }
}
