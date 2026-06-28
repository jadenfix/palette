//! Exact McNemar test for a paired binary outcome.

use crate::numerics::binomial_lower_tail_half;
use crate::StatsError;

/// Exact two-sided McNemar p-value for a paired binary outcome, where `b` and
/// `c` are the two discordant-pair counts (e.g. 0→1 and 1→0). Under H0 the
/// discordant pairs split Binomial(b + c, ½); the two-sided p-value is
/// `min(1, 2 · P(X ≤ min(b, c)))`. With no discordant pairs the p-value is 1.0.
pub fn mcnemar_exact_p(b: u64, c: u64) -> Result<f64, StatsError> {
    let n = b + c;
    if n == 0 {
        return Ok(1.0);
    }
    let k = b.min(c);
    Ok((2.0 * binomial_lower_tail_half(k, n)).min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_discordant_pairs_is_one() {
        assert_eq!(
            mcnemar_exact_p(0, 0).unwrap_or_else(|err| panic!("{err}")),
            1.0
        );
    }

    #[test]
    fn known_small_values() {
        // b=3, c=0 -> 2 * P(X<=0 | n=3) = 2 * 0.125 = 0.25.
        assert!((mcnemar_exact_p(3, 0).unwrap_or_else(|err| panic!("{err}")) - 0.25).abs() < 1e-9);
        // b=5, c=0 -> 2 * 0.5^5 = 0.0625.
        assert!(
            (mcnemar_exact_p(5, 0).unwrap_or_else(|err| panic!("{err}")) - 0.0625).abs() < 1e-9
        );
        // Symmetric: order of b,c does not matter.
        assert_eq!(
            mcnemar_exact_p(2, 7).unwrap_or_else(|err| panic!("{err}")),
            mcnemar_exact_p(7, 2).unwrap_or_else(|err| panic!("{err}"))
        );
    }
}
