//! Multiple-comparison corrections used by gates that evaluate more than one
//! metric or segment at a time.

use crate::StatsError;

/// Per-hypothesis outcome after a multiplicity correction.
///
/// Results are returned in the same order as the input p-values so callers can
/// zip them back onto their metric/segment identifiers without a side table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultiplicityDecision {
    pub index: usize,
    pub raw_p_value: f64,
    pub adjusted_p_value: f64,
    pub reject: bool,
}

#[derive(Debug, Clone, Copy)]
struct RankedPValue {
    index: usize,
    p_value: f64,
}

/// Holm-Bonferroni family-wise error-rate correction.
///
/// This is a step-down correction: p-values are sorted ascending, each p-value
/// is multiplied by the number of remaining hypotheses, and the adjusted values
/// are made monotonic. A hypothesis rejects when its adjusted p-value is <=
/// `alpha`.
pub fn holm_bonferroni(
    p_values: &[f64],
    alpha: f64,
) -> Result<Vec<MultiplicityDecision>, StatsError> {
    validate_alpha(alpha)?;
    let ranked = ranked_p_values(p_values)?;
    let m = ranked.len();
    let mut decisions = Vec::with_capacity(m);
    let mut running_max = 0.0_f64;

    for (rank, item) in ranked.iter().enumerate() {
        let remaining = (m - rank) as f64;
        running_max = running_max.max((item.p_value * remaining).min(1.0));
        decisions.push(MultiplicityDecision {
            index: item.index,
            raw_p_value: item.p_value,
            adjusted_p_value: running_max,
            reject: running_max <= alpha,
        });
    }

    decisions.sort_by_key(|decision| decision.index);
    Ok(decisions)
}

/// Benjamini-Hochberg false-discovery-rate correction.
///
/// This is a step-up correction: p-values are sorted ascending, each p-value is
/// scaled by `m / rank`, and the adjusted values are made monotonic by scanning
/// from the largest p-value down. A hypothesis rejects when its adjusted p-value
/// is <= `alpha`.
pub fn benjamini_hochberg(
    p_values: &[f64],
    alpha: f64,
) -> Result<Vec<MultiplicityDecision>, StatsError> {
    validate_alpha(alpha)?;
    let ranked = ranked_p_values(p_values)?;
    let m = ranked.len();
    let m_f = m as f64;
    let mut adjusted_by_rank = vec![0.0; m];
    let mut running_min = 1.0_f64;

    for rank_index in (0..m).rev() {
        let rank = (rank_index + 1) as f64;
        running_min = running_min.min((ranked[rank_index].p_value * m_f / rank).min(1.0));
        adjusted_by_rank[rank_index] = running_min;
    }

    let mut decisions = ranked
        .iter()
        .zip(adjusted_by_rank)
        .map(|(item, adjusted_p_value)| MultiplicityDecision {
            index: item.index,
            raw_p_value: item.p_value,
            adjusted_p_value,
            reject: adjusted_p_value <= alpha,
        })
        .collect::<Vec<_>>();

    decisions.sort_by_key(|decision| decision.index);
    Ok(decisions)
}

fn validate_alpha(alpha: f64) -> Result<(), StatsError> {
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return Err(StatsError::InvalidAlpha(alpha));
    }
    Ok(())
}

fn ranked_p_values(p_values: &[f64]) -> Result<Vec<RankedPValue>, StatsError> {
    let mut ranked = Vec::with_capacity(p_values.len());
    for (index, p_value) in p_values.iter().copied().enumerate() {
        if !p_value.is_finite() || !(0.0..=1.0).contains(&p_value) {
            return Err(StatsError::InvalidParameter {
                name: "p_value",
                value: p_value,
            });
        }
        ranked.push(RankedPValue { index, p_value });
    }
    ranked.sort_by(|left, right| {
        left.p_value
            .total_cmp(&right.p_value)
            .then_with(|| left.index.cmp(&right.index))
    });
    Ok(ranked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holm_bonferroni_adjusts_and_preserves_input_order() -> Result<(), StatsError> {
        let decisions = holm_bonferroni(&[0.03, 0.01, 0.04], 0.05)?;

        assert_eq!(decisions.len(), 3);
        assert_eq!(decisions[0].index, 0);
        assert_eq!(decisions[1].index, 1);
        assert_eq!(decisions[2].index, 2);
        assert!((decisions[0].adjusted_p_value - 0.06).abs() < 1e-12);
        assert!((decisions[1].adjusted_p_value - 0.03).abs() < 1e-12);
        assert!((decisions[2].adjusted_p_value - 0.06).abs() < 1e-12);
        assert!(!decisions[0].reject);
        assert!(decisions[1].reject);
        assert!(!decisions[2].reject);
        Ok(())
    }

    #[test]
    fn holm_bonferroni_rejects_step_down_prefix() -> Result<(), StatsError> {
        let decisions = holm_bonferroni(&[0.01, 0.02, 0.50], 0.05)?;

        assert!(decisions[0].reject);
        assert!(decisions[1].reject);
        assert!(!decisions[2].reject);
        assert!((decisions[0].adjusted_p_value - 0.03).abs() < 1e-12);
        assert!((decisions[1].adjusted_p_value - 0.04).abs() < 1e-12);
        assert!((decisions[2].adjusted_p_value - 0.50).abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn benjamini_hochberg_adjusts_and_preserves_input_order() -> Result<(), StatsError> {
        let decisions = benjamini_hochberg(&[0.04, 0.01, 0.03], 0.05)?;

        assert_eq!(decisions.len(), 3);
        assert_eq!(decisions[0].index, 0);
        assert_eq!(decisions[1].index, 1);
        assert_eq!(decisions[2].index, 2);
        assert!((decisions[0].adjusted_p_value - 0.04).abs() < 1e-12);
        assert!((decisions[1].adjusted_p_value - 0.03).abs() < 1e-12);
        assert!((decisions[2].adjusted_p_value - 0.04).abs() < 1e-12);
        assert!(decisions.iter().all(|decision| decision.reject));
        Ok(())
    }

    #[test]
    fn benjamini_hochberg_can_reject_non_holm_case() -> Result<(), StatsError> {
        let p_values = [0.01, 0.03, 0.04];
        let holm = holm_bonferroni(&p_values, 0.05)?;
        let bh = benjamini_hochberg(&p_values, 0.05)?;

        assert_eq!(holm.iter().filter(|decision| decision.reject).count(), 1);
        assert_eq!(bh.iter().filter(|decision| decision.reject).count(), 3);
        Ok(())
    }

    #[test]
    fn multiplicity_allows_empty_input() -> Result<(), StatsError> {
        assert!(holm_bonferroni(&[], 0.05)?.is_empty());
        assert!(benjamini_hochberg(&[], 0.05)?.is_empty());
        Ok(())
    }

    #[test]
    fn multiplicity_rejects_bad_alpha_and_p_values() {
        assert!(matches!(
            holm_bonferroni(&[0.01], 0.0),
            Err(StatsError::InvalidAlpha(0.0))
        ));
        assert!(matches!(
            benjamini_hochberg(&[f64::NAN], 0.05),
            Err(StatsError::InvalidParameter {
                name: "p_value",
                ..
            })
        ));
        assert!(matches!(
            benjamini_hochberg(&[1.01], 0.05),
            Err(StatsError::InvalidParameter {
                name: "p_value",
                ..
            })
        ));
    }
}
