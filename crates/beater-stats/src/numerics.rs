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
//! incomplete-beta continued fraction; Acklam's algorithm for the normal quantile.

use std::f64::consts::PI;

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
    let (mut lo, mut hi) = (-1.0e7_f64, 1.0e7_f64);
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
pub fn binomial_lower_tail_half(k: u64, n: u64) -> f64 {
    let ln_half_pow_n = n as f64 * 0.5_f64.ln();
    let ln_n_fact = ln_gamma(n as f64 + 1.0);
    let mut sum = 0.0;
    let upper = k.min(n);
    for i in 0..=upper {
        let ln_choose = ln_n_fact - ln_gamma(i as f64 + 1.0) - ln_gamma((n - i) as f64 + 1.0);
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

    #[test]
    fn binomial_tail_known_values() {
        assert!(close(binomial_lower_tail_half(0, 3), 0.125, 1e-12)); // 0.5^3
        assert!(close(binomial_lower_tail_half(0, 5), 0.031_25, 1e-12)); // 0.5^5
        assert!(close(binomial_lower_tail_half(5, 10), 0.623_046_875, 1e-9));
        assert!(close(binomial_lower_tail_half(10, 10), 1.0, 1e-12));
    }
}
