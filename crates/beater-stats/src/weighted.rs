//! Weighted aggregate primitives for tail-sampled roll-ups.

use crate::StatsError;

#[derive(Debug, Clone, Copy)]
struct WeightedSums {
    mean: f64,
    weight_sum: f64,
    weight_sq_sum: f64,
}

/// Compute the inverse-probability weighted mean.
///
/// This is the Hajek ratio estimate used for tail-sampled aggregates in §9:
///
/// ```text
/// mean = sum(w_i * y_i) / sum(w_i)
/// ```
///
/// For Beater production traces, `w_i` is `sampling_weight = 1 / keep_probability`.
/// Values must be finite, weights must be finite and strictly positive, and both
/// slices must have the same non-zero length.
pub fn weighted_mean(values: &[f64], weights: &[f64]) -> Result<f64, StatsError> {
    Ok(weighted_sums(values, weights)?.mean)
}

/// Compute a weighted-sample standard error for [`weighted_mean`].
///
/// This is a Kish effective-sample-size SE for independent weighted observations:
///
/// ```text
/// mean  = sum(w_i * y_i) / sum(w_i)
/// n_eff = sum(w_i)^2 / sum(w_i^2)
/// s_w^2 = (n_eff / (n_eff - 1)) * sum(w_i * (y_i - mean)^2) / sum(w_i)
/// SE    = sqrt(s_w^2 / n_eff)
/// ```
///
/// With equal weights this reduces to the usual sample-mean standard error
/// `sqrt(sample_variance / n)`. It is a weighted-sample uncertainty estimate for
/// aggregate roll-ups; it is not a clustered SE or an anytime-valid confidence
/// sequence. A single observation, or observations with zero weighted dispersion,
/// return `0.0`.
pub fn weighted_standard_error(values: &[f64], weights: &[f64]) -> Result<f64, StatsError> {
    let sums = weighted_sums(values, weights)?;
    if values.len() < 2 {
        return Ok(0.0);
    }

    let mut weighted_ss = 0.0;
    for (&value, &weight) in values.iter().zip(weights.iter()) {
        let delta = value - sums.mean;
        let term = weight * delta * delta;
        if !term.is_finite() {
            return Err(StatsError::NonFinite);
        }
        weighted_ss += term;
        if !weighted_ss.is_finite() {
            return Err(StatsError::NonFinite);
        }
    }

    if weighted_ss == 0.0 {
        return Ok(0.0);
    }

    let n_eff = sums.weight_sum * sums.weight_sum / sums.weight_sq_sum;
    if !n_eff.is_finite() {
        return Err(StatsError::NonFinite);
    }
    if n_eff <= 1.0 {
        return Ok(0.0);
    }

    let weighted_population_variance = weighted_ss / sums.weight_sum;
    let corrected_variance = weighted_population_variance * n_eff / (n_eff - 1.0);
    let standard_error = (corrected_variance / n_eff).sqrt();
    if standard_error.is_finite() {
        Ok(standard_error)
    } else {
        Err(StatsError::NonFinite)
    }
}

fn weighted_sums(values: &[f64], weights: &[f64]) -> Result<WeightedSums, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptySample);
    }
    if values.len() != weights.len() {
        return Err(StatsError::MismatchedLengths {
            baseline: values.len(),
            candidate: weights.len(),
        });
    }

    let mut weight_sum = 0.0;
    let mut weighted_value_sum = 0.0;
    let mut weight_sq_sum = 0.0;

    for (&value, &weight) in values.iter().zip(weights.iter()) {
        if !value.is_finite() {
            return Err(StatsError::NonFinite);
        }
        if !weight.is_finite() || weight <= 0.0 {
            return Err(StatsError::InvalidParameter {
                name: "weight",
                value: weight,
            });
        }

        let weighted_value = weight * value;
        let weight_sq = weight * weight;
        if !weighted_value.is_finite() || !weight_sq.is_finite() {
            return Err(StatsError::NonFinite);
        }

        weight_sum += weight;
        weighted_value_sum += weighted_value;
        weight_sq_sum += weight_sq;
        if !weight_sum.is_finite() || !weighted_value_sum.is_finite() || !weight_sq_sum.is_finite()
        {
            return Err(StatsError::NonFinite);
        }
    }

    let mean = weighted_value_sum / weight_sum;
    if !mean.is_finite() {
        return Err(StatsError::NonFinite);
    }

    Ok(WeightedSums {
        mean,
        weight_sum,
        weight_sq_sum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mean;

    #[test]
    fn weighted_mean_differs_from_unweighted_mean() {
        let values = [0.0, 10.0];
        let weights = [9.0, 1.0];

        let weighted = weighted_mean(&values, &weights).unwrap_or_else(|err| panic!("{err}"));
        let unweighted = mean(&values);

        assert!((weighted - 1.0).abs() < 1e-12);
        assert!((unweighted - 5.0).abs() < 1e-12);
        assert!((weighted - unweighted).abs() > 1.0);
    }

    #[test]
    fn rejects_invalid_weights_and_values() {
        assert!(matches!(
            weighted_mean(&[1.0], &[0.0]),
            Err(StatsError::InvalidParameter { name: "weight", .. })
        ));
        assert!(matches!(
            weighted_mean(&[1.0], &[-1.0]),
            Err(StatsError::InvalidParameter { name: "weight", .. })
        ));
        assert!(matches!(
            weighted_mean(&[1.0], &[f64::INFINITY]),
            Err(StatsError::InvalidParameter { name: "weight", .. })
        ));
        assert!(matches!(
            weighted_standard_error(&[f64::NAN], &[1.0]),
            Err(StatsError::NonFinite)
        ));
    }

    #[test]
    fn rejects_empty_and_mismatched_inputs() {
        assert_eq!(weighted_mean(&[], &[]), Err(StatsError::EmptySample));
        assert!(matches!(
            weighted_mean(&[1.0, 2.0], &[1.0]),
            Err(StatsError::MismatchedLengths { .. })
        ));
    }

    #[test]
    fn single_observation_has_zero_standard_error() {
        let se = weighted_standard_error(&[42.0], &[3.0]).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(se, 0.0);
    }

    #[test]
    fn all_equal_values_have_zero_standard_error() {
        let values = [7.0, 7.0, 7.0, 7.0];
        let weights = [1.0, 2.0, 3.0, 4.0];

        let mean = weighted_mean(&values, &weights).unwrap_or_else(|err| panic!("{err}"));
        let se = weighted_standard_error(&values, &weights).unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(mean, 7.0);
        assert_eq!(se, 0.0);
    }

    #[test]
    fn weighted_standard_error_matches_hand_computed_value() {
        // values=[1,3,5], weights=[1,2,1]:
        // mean = 3
        // n_eff = 4^2 / (1^2 + 2^2 + 1^2) = 8/3
        // weighted dispersion = (1*4 + 2*0 + 1*4) / 4 = 2
        // corrected variance = 2 * (8/3) / (5/3) = 3.2
        // SE = sqrt(3.2 / (8/3)) = sqrt(1.2)
        let values = [1.0, 3.0, 5.0];
        let weights = [1.0, 2.0, 1.0];

        let se = weighted_standard_error(&values, &weights).unwrap_or_else(|err| panic!("{err}"));

        assert!((se - 1.2_f64.sqrt()).abs() < 1e-12, "se={se}");
    }
}
