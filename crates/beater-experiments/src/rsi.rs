//! The gated RSI acceptance loop: tie the split substrate (PR1) and the
//! anti-overfitting guardrail (PR2) into a single accept/reject decision.
//!
//! A candidate is accepted only if it **both**:
//! 1. passes the held-out **Test** split gate (`compare_paired_scores` →
//!    [`GateDecision::Pass`]), and
//! 2. clears the **generalization-gap** guardrail — its lift on the held-out
//!    split is not significantly below its lift on the optimization (Train+Val)
//!    split (`assess_generalization_gap`).
//!
//! Either check alone is insufficient: the test gate misses a candidate that
//! merely *doesn't regress* on Test while massively overfitting the optimization
//! split; the gap guardrail catches exactly that. This is the mechanism that
//! makes "non-overfitting RSI" a property the system enforces rather than hopes
//! for.

use beater_core::DatasetCaseId;
use beater_datasets::{
    split_for_input, DatasetCase, DatasetVersionSnapshot, SplitConfig, SplitLabel,
};
use beater_eval::{
    compare_paired_scores, evaluate_deterministic, EvaluationCase, EvaluatorSpec,
    ExperimentComparison, GateDecision, GatePolicy,
};
use beater_stats::{assess_generalization_gap, GapAssessment};
use serde_json::Value;
use std::collections::BTreeMap;

/// The outcome of a gated candidate evaluation.
#[derive(Clone, Debug)]
pub struct GatedDecision {
    /// Whether the candidate is accepted: held-out gate Pass **and** no
    /// significant generalization gap.
    pub accepted: bool,
    /// Human-readable explanation, carried to the audit trail.
    pub reason: String,
    /// The held-out Test split gate decision.
    pub test_decision: GateDecision,
    /// The full held-out Test split comparison.
    pub test_comparison: ExperimentComparison,
    /// The generalization-gap assessment between optimize and held-out splits.
    pub gap: GapAssessment,
    /// Number of cases in each split (Train, Val, Test).
    pub n_train: usize,
    /// Cases in the Val split.
    pub n_val: usize,
    /// Cases in the Test (held-out) split.
    pub n_test: usize,
}

/// Errors from the gated loop.
#[derive(Debug, thiserror::Error)]
pub enum RsiError {
    /// A case input could not be canonically hashed for split assignment.
    #[error("failed to fingerprint case {case_id} for splitting: {source}")]
    Fingerprint {
        /// The offending case.
        case_id: DatasetCaseId,
        /// Underlying JSON-hash error.
        #[source]
        source: beater_core::JsonHashError,
    },
    /// A split needed to make a decision was empty.
    #[error("{split} split is empty; cannot gate on a held-out set")]
    EmptySplit {
        /// Which split was empty.
        split: &'static str,
    },
    /// Scoring or statistics failed.
    #[error(transparent)]
    Eval(#[from] beater_eval::EvalError),
    /// The generalization-gap statistics failed.
    #[error(transparent)]
    Stats(#[from] beater_stats::StatsError),
}

/// Score one case's output under a deterministic evaluator. The output defaults
/// to the case's stored output when no override is supplied.
fn score_case(
    evaluator: &EvaluatorSpec,
    case: &DatasetCase,
    outputs: &BTreeMap<DatasetCaseId, Value>,
) -> Result<f64, beater_eval::EvalError> {
    let output = outputs.get(&case.case_id).unwrap_or(&case.output).clone();
    let ec = EvaluationCase {
        input: case.input.clone(),
        output,
        reference: case.reference.clone(),
        trace: None,
    };
    Ok(evaluate_deterministic(evaluator, &ec)?.score)
}

/// Per-split paired score vectors (baseline, candidate).
#[derive(Default)]
struct SplitScores {
    baseline: Vec<f64>,
    candidate: Vec<f64>,
}

/// Evaluate a candidate against a held-out Test split, gated by the
/// generalization-gap guardrail.
///
/// `baseline_outputs` / `candidate_outputs` map a case id to its produced output;
/// missing entries fall back to the case's stored output. Splitting is the
/// content-keyed assignment from PR1, so the optimizer can never have touched the
/// Test cases (identical inputs are co-located by construction).
///
/// `gap_tolerance` is the largest benign optimize−holdout lift gap (e.g. `0.0`).
#[allow(clippy::too_many_arguments)]
pub fn gate_candidate_on_holdout(
    snapshot: &DatasetVersionSnapshot,
    evaluator: &EvaluatorSpec,
    baseline_outputs: &BTreeMap<DatasetCaseId, Value>,
    candidate_outputs: &BTreeMap<DatasetCaseId, Value>,
    gate_policy: &GatePolicy,
    split: &SplitConfig,
    gap_tolerance: f64,
    bootstrap_resamples: usize,
    seed: u64,
) -> Result<GatedDecision, RsiError> {
    // Optimize = Train + Val (what the optimizer was allowed to see); Test is held out.
    let mut optimize = SplitScores::default();
    let mut test = SplitScores::default();
    let (mut n_train, mut n_val, mut n_test) = (0usize, 0usize, 0usize);

    for case in &snapshot.cases {
        let label =
            split_for_input(&case.input, split).map_err(|source| RsiError::Fingerprint {
                case_id: case.case_id.clone(),
                source,
            })?;
        let b = score_case(evaluator, case, baseline_outputs)?;
        let c = score_case(evaluator, case, candidate_outputs)?;
        match label {
            SplitLabel::Train => {
                n_train += 1;
                optimize.baseline.push(b);
                optimize.candidate.push(c);
            }
            SplitLabel::Val => {
                n_val += 1;
                optimize.baseline.push(b);
                optimize.candidate.push(c);
            }
            SplitLabel::Test => {
                n_test += 1;
                test.baseline.push(b);
                test.candidate.push(c);
            }
        }
    }

    if test.baseline.is_empty() {
        return Err(RsiError::EmptySplit { split: "test" });
    }
    if optimize.baseline.is_empty() {
        return Err(RsiError::EmptySplit { split: "optimize" });
    }

    // 1) Held-out Test split gate.
    let test_comparison = compare_paired_scores(&test.baseline, &test.candidate, gate_policy)?;
    let test_decision = test_comparison.decision.clone();

    // 2) Generalization-gap guardrail: optimize lift vs held-out lift.
    let confidence = 1.0 - gate_policy.alpha;
    let gap = assess_generalization_gap(
        &optimize.baseline,
        &optimize.candidate,
        &test.baseline,
        &test.candidate,
        gap_tolerance,
        confidence,
        bootstrap_resamples,
        seed,
    )?;

    let passed_gate = matches!(test_decision, GateDecision::Pass);
    let accepted = passed_gate && !gap.overfit;
    let reason = if !passed_gate {
        format!(
            "rejected: held-out gate did not pass (decision={})",
            test_decision.name()
        )
    } else if gap.overfit {
        format!(
            "rejected: generalization gap (optimize lift {:.3} vs held-out lift {:.3}, gap CI low {:.3} > tolerance {:.3})",
            gap.optimize_lift, gap.holdout_lift, gap.gap_ci_low, gap_tolerance
        )
    } else {
        format!(
            "accepted: held-out gate passed and lift generalizes (held-out lift {:.3}, gap {:.3})",
            gap.holdout_lift, gap.gap
        )
    };

    Ok(GatedDecision {
        accepted,
        reason,
        test_decision,
        test_comparison,
        gap,
        n_train,
        n_val,
        n_test,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{DatasetId, EnvironmentId, ProjectId, SpanId, TenantId, Timestamp, TraceId};
    use beater_eval::EvaluatorKind;
    use beater_schema::EvaluatorLane;
    use serde_json::json;

    fn case(i: usize, reference: &str) -> DatasetCase {
        DatasetCase {
            tenant_id: TenantId::new("t").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("p").unwrap_or_else(|err| panic!("{err}")),
            dataset_id: DatasetId::new("d").unwrap_or_else(|err| panic!("{err}")),
            case_id: DatasetCaseId::new(format!("case-{i}")).unwrap_or_else(|err| panic!("{err}")),
            source_trace_id: TraceId::new("tr").unwrap_or_else(|err| panic!("{err}")),
            source_span_id: SpanId::new("sp").unwrap_or_else(|err| panic!("{err}")),
            source_environment_id: EnvironmentId::new("env").unwrap_or_else(|err| panic!("{err}")),
            input: json!({ "i": i }),
            output: json!("wrong"),
            reference: Some(json!(reference)),
            trace: json!({}),
            normalizer_version: "v1".to_string(),
            trace_schema_version: 1,
            input_artifact_hashes: vec![],
            created_at: Timestamp::default(),
        }
    }

    fn snapshot(n: usize) -> DatasetVersionSnapshot {
        let cases: Vec<_> = (0..n).map(|i| case(i, "ok")).collect();
        let corpus_root =
            beater_datasets::corpus_root_for_cases(&cases).unwrap_or_else(|err| panic!("{err}"));
        DatasetVersionSnapshot {
            tenant_id: TenantId::new("t").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("p").unwrap_or_else(|err| panic!("{err}")),
            dataset_id: DatasetId::new("d").unwrap_or_else(|err| panic!("{err}")),
            version_id: beater_core::DatasetVersionId::new("v1")
                .unwrap_or_else(|err| panic!("{err}")),
            cases,
            corpus_root,
            created_at: Timestamp::default(),
        }
    }

    fn exact_evaluator() -> EvaluatorSpec {
        EvaluatorSpec {
            id: "exact".to_string(),
            lane: EvaluatorLane::DeterministicWasi,
            kind: EvaluatorKind::ExactMatch,
        }
    }

    fn policy() -> GatePolicy {
        GatePolicy {
            min_sample_size: 10,
            max_regression: 0.0,
            alpha: 0.05,
            comparison_count: 1,
        }
    }

    #[test]
    fn accepts_a_candidate_that_generalizes() {
        // Baseline is always wrong; the good candidate is correct on EVERY case,
        // so the held-out lift matches the optimization lift → no gap → accept.
        let snap = snapshot(300);
        let split = SplitConfig::standard(123);
        let baseline = BTreeMap::new(); // falls back to stored "wrong"
        let mut candidate = BTreeMap::new();
        for c in &snap.cases {
            candidate.insert(c.case_id.clone(), json!("ok"));
        }
        let d = gate_candidate_on_holdout(
            &snap,
            &exact_evaluator(),
            &baseline,
            &candidate,
            &policy(),
            &split,
            0.0,
            2000,
            7,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert!(
            d.n_test >= 10,
            "need a powered held-out set, got {}",
            d.n_test
        );
        assert_eq!(d.test_decision, GateDecision::Pass, "{}", d.reason);
        assert!(
            !d.gap.overfit,
            "uniform improvement must not look like overfit"
        );
        assert!(d.accepted, "{}", d.reason);
    }

    #[test]
    fn rejects_an_overfit_candidate_the_test_gate_alone_would_pass() {
        // The overfit candidate is correct ONLY on optimization-split cases and
        // stays wrong on held-out cases. On Test it neither improves nor regresses,
        // so the held-out gate alone would PASS it — but the generalization-gap
        // guardrail sees the huge optimize-vs-holdout gap and REJECTS it.
        let snap = snapshot(300);
        let split = SplitConfig::standard(123);
        let baseline = BTreeMap::new();
        let mut candidate = BTreeMap::new();
        for c in &snap.cases {
            let label = split_for_input(&c.input, &split).unwrap_or_else(|err| panic!("{err}"));
            let out = match label {
                SplitLabel::Test => json!("wrong"), // no improvement on held-out
                _ => json!("ok"),                   // big improvement on optimize
            };
            candidate.insert(c.case_id.clone(), out);
        }
        let d = gate_candidate_on_holdout(
            &snap,
            &exact_evaluator(),
            &baseline,
            &candidate,
            &policy(),
            &split,
            0.0,
            2000,
            7,
        )
        .unwrap_or_else(|err| panic!("{err}"));

        // The held-out gate alone is fooled: no regression on Test → Pass.
        assert_eq!(
            d.test_decision,
            GateDecision::Pass,
            "test gate alone should see no regression"
        );
        // But the generalization-gap guardrail catches the overfit and rejects.
        assert!(d.gap.overfit, "gap guardrail should flag the overfit");
        assert!(
            d.gap.optimize_lift > 0.5,
            "optimize lift {}",
            d.gap.optimize_lift
        );
        assert!(
            d.gap.holdout_lift.abs() < 0.1,
            "holdout lift {}",
            d.gap.holdout_lift
        );
        assert!(
            !d.accepted,
            "overfit candidate must be rejected: {}",
            d.reason
        );
    }

    #[test]
    fn errors_when_no_held_out_cases() {
        // All-train split leaves the Test split empty → cannot gate.
        let snap = snapshot(50);
        let split = SplitConfig::new(1.0, 0.0, 0.0, 1).unwrap_or_else(|err| panic!("{err}"));
        let err = gate_candidate_on_holdout(
            &snap,
            &exact_evaluator(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &policy(),
            &split,
            0.0,
            500,
            1,
        )
        .err()
        .unwrap_or_else(|| panic!("expected EmptySplit error"));
        assert!(matches!(err, RsiError::EmptySplit { split: "test" }));
    }
}
