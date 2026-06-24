use beater_core::Money;
use beater_schema::EvaluatorLane;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("evaluator {0} requires judge broker lane")]
    RequiresJudgeBroker(String),
    #[error("evaluator {0} requires deterministic lane")]
    RequiresDeterministicLane(String),
    #[error(
        "evaluator {evaluator_id} kind {catalog_id} requires {expected:?} lane, got {actual:?}"
    )]
    EvaluatorLaneMismatch {
        evaluator_id: String,
        catalog_id: String,
        expected: EvaluatorLane,
        actual: EvaluatorLane,
    },
    #[error("invalid regex: {0}")]
    InvalidRegex(String),
    #[error(
        "underpowered comparison: sample_size={sample_size}, min_sample_size={min_sample_size}"
    )]
    Underpowered {
        sample_size: usize,
        min_sample_size: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluationCase {
    pub input: Value,
    pub output: Value,
    pub reference: Option<Value>,
    pub trace: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreResult {
    pub score: f64,
    pub label: Option<String>,
    pub evidence: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluatorSpec {
    pub id: String,
    pub lane: EvaluatorLane,
    pub kind: EvaluatorKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluatorKind {
    ExactMatch,
    RegexMatch { pattern: String },
    JsonObject,
    CostBudget { max_micros: i64 },
    LatencyBudgetMs { max_ms: u64 },
    LlmJudge { rubric: String, model: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EvaluatorCatalogEntry {
    pub id: &'static str,
    pub lane: EvaluatorLane,
    pub display_name: &'static str,
    pub description: &'static str,
    pub requires_reference: bool,
    pub consumes_trace: bool,
    pub wasm_safe: bool,
}

pub const EVALUATOR_CATALOG: &[EvaluatorCatalogEntry] = &[
    EvaluatorCatalogEntry {
        id: "exact_match",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Exact match",
        description: "Scores output equality against a reference value.",
        requires_reference: true,
        consumes_trace: false,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "regex_match",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Regex match",
        description: "Scores a string output against a configured regular expression.",
        requires_reference: false,
        consumes_trace: false,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "json_object",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "JSON object",
        description: "Scores whether the output is a JSON object.",
        requires_reference: false,
        consumes_trace: false,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "cost_budget",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Cost budget",
        description: "Scores whether trace cost stays within a configured micro-unit budget.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "latency_budget_ms",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Latency budget",
        description: "Scores whether trace latency stays within a configured millisecond budget.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "llm_judge",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "LLM judge",
        description: "Routes model-dependent scoring through the judge broker.",
        requires_reference: false,
        consumes_trace: false,
        wasm_safe: false,
    },
];

pub fn evaluator_catalog() -> &'static [EvaluatorCatalogEntry] {
    EVALUATOR_CATALOG
}

pub fn evaluator_catalog_entry(id: &str) -> Option<&'static EvaluatorCatalogEntry> {
    EVALUATOR_CATALOG.iter().find(|entry| entry.id == id)
}

impl EvaluatorKind {
    pub fn catalog_id(&self) -> &'static str {
        match self {
            Self::ExactMatch => "exact_match",
            Self::RegexMatch { .. } => "regex_match",
            Self::JsonObject => "json_object",
            Self::CostBudget { .. } => "cost_budget",
            Self::LatencyBudgetMs { .. } => "latency_budget_ms",
            Self::LlmJudge { .. } => "llm_judge",
        }
    }

    pub fn catalog_entry(&self) -> &'static EvaluatorCatalogEntry {
        let id = self.catalog_id();
        evaluator_catalog_entry(id).unwrap_or_else(|| unreachable!("missing catalog entry: {id}"))
    }

    pub fn expected_lane(&self) -> EvaluatorLane {
        self.catalog_entry().lane
    }
}

impl EvaluatorSpec {
    pub fn validate_catalog_lane(&self) -> Result<(), EvalError> {
        let entry = self.kind.catalog_entry();
        if self.lane == entry.lane {
            return Ok(());
        }

        Err(EvalError::EvaluatorLaneMismatch {
            evaluator_id: self.id.clone(),
            catalog_id: entry.id.to_string(),
            expected: entry.lane,
            actual: self.lane,
        })
    }
}

pub fn evaluate_deterministic(
    spec: &EvaluatorSpec,
    case: &EvaluationCase,
) -> Result<ScoreResult, EvalError> {
    spec.validate_catalog_lane()?;
    if spec.lane != EvaluatorLane::DeterministicWasi {
        return Err(EvalError::RequiresDeterministicLane(spec.id.clone()));
    }

    match &spec.kind {
        EvaluatorKind::ExactMatch => {
            let pass = case.reference.as_ref() == Some(&case.output);
            Ok(binary_score(pass, "exact_match"))
        }
        EvaluatorKind::RegexMatch { pattern } => {
            let output = case.output.as_str().unwrap_or_default();
            let regex =
                Regex::new(pattern).map_err(|err| EvalError::InvalidRegex(err.to_string()))?;
            Ok(binary_score(regex.is_match(output), "regex_match"))
        }
        EvaluatorKind::JsonObject => Ok(binary_score(case.output.is_object(), "json_object")),
        EvaluatorKind::CostBudget { max_micros } => {
            let cost = case
                .trace
                .as_ref()
                .and_then(|trace| trace.get("cost_micros"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            Ok(binary_score(cost <= *max_micros, "cost_budget"))
        }
        EvaluatorKind::LatencyBudgetMs { max_ms } => {
            let latency = case
                .trace
                .as_ref()
                .and_then(|trace| trace.get("latency_ms"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            Ok(binary_score(latency <= *max_ms, "latency_budget"))
        }
        EvaluatorKind::LlmJudge { .. } => Err(EvalError::RequiresJudgeBroker(spec.id.clone())),
    }
}

fn binary_score(pass: bool, metric: &str) -> ScoreResult {
    ScoreResult {
        score: if pass { 1.0 } else { 0.0 },
        label: Some(if pass { "pass" } else { "fail" }.to_string()),
        evidence: serde_json::json!({ "metric": metric, "pass": pass }),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeRequest {
    pub rubric: String,
    pub model: String,
    pub input: Value,
    pub output: Value,
    pub reference: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgeResponse {
    pub score: f64,
    pub rationale: String,
    pub cost: Money,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentComparison {
    pub sample_size: usize,
    pub baseline_mean: f64,
    pub candidate_mean: f64,
    pub delta: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub decision: GateDecision,
    pub test: StatisticalTest,
    pub adjusted_alpha: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    Pass,
    FailRegression,
    Inconclusive,
}

impl GateDecision {
    /// Stable snake_case name used in persisted reports.
    pub fn name(&self) -> &'static str {
        match self {
            GateDecision::Pass => "pass",
            GateDecision::FailRegression => "fail_regression",
            GateDecision::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalTest {
    PairedNormalApproximation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GatePolicy {
    pub min_sample_size: usize,
    pub max_regression: f64,
    pub alpha: f64,
    pub comparison_count: usize,
}

impl Default for GatePolicy {
    fn default() -> Self {
        Self {
            min_sample_size: 10,
            max_regression: 0.0,
            alpha: 0.05,
            comparison_count: 1,
        }
    }
}

pub fn compare_paired_scores(
    baseline: &[f64],
    candidate: &[f64],
    policy: &GatePolicy,
) -> Result<ExperimentComparison, EvalError> {
    let n = baseline.len().min(candidate.len());
    if n < policy.min_sample_size {
        return Err(EvalError::Underpowered {
            sample_size: n,
            min_sample_size: policy.min_sample_size,
        });
    }

    let mut differences = Vec::with_capacity(n);
    for index in 0..n {
        differences.push(candidate[index] - baseline[index]);
    }
    let delta = mean(&differences);
    let baseline_mean = mean(&baseline[..n]);
    let candidate_mean = mean(&candidate[..n]);
    let variance = sample_variance(&differences);
    let standard_error = (variance / n as f64).sqrt();
    let adjusted_alpha = (policy.alpha / policy.comparison_count.max(1) as f64).clamp(0.0001, 0.5);
    let z = if adjusted_alpha <= 0.01 { 2.576 } else { 1.96 };
    let ci_low = delta - z * standard_error;
    let ci_high = delta + z * standard_error;
    let decision = if ci_high < -policy.max_regression {
        GateDecision::FailRegression
    } else if ci_low >= -policy.max_regression {
        GateDecision::Pass
    } else {
        GateDecision::Inconclusive
    };

    Ok(ExperimentComparison {
        sample_size: n,
        baseline_mean,
        candidate_mean,
        delta,
        ci_low,
        ci_high,
        decision,
        test: StatisticalTest::PairedNormalApproximation,
        adjusted_alpha,
    })
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn sample_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let value_mean = mean(values);
    values
        .iter()
        .map(|value| {
            let diff = value - value_mean;
            diff * diff
        })
        .sum::<f64>()
        / (values.len() - 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluator_catalog_classifies_execution_lanes() {
        let catalog = evaluator_catalog();
        assert_eq!(catalog.len(), 6);

        let exact = evaluator_catalog_entry("exact_match")
            .unwrap_or_else(|| panic!("exact_match catalog entry should exist"));
        assert_eq!(exact.lane, EvaluatorLane::DeterministicWasi);
        assert!(exact.requires_reference);
        assert!(exact.wasm_safe);

        let cost = EvaluatorKind::CostBudget { max_micros: 10 };
        assert_eq!(cost.catalog_id(), "cost_budget");
        assert_eq!(cost.expected_lane(), EvaluatorLane::DeterministicWasi);
        assert!(cost.catalog_entry().consumes_trace);

        let judge = EvaluatorKind::LlmJudge {
            rubric: "correctness".to_string(),
            model: "judge-model".to_string(),
        };
        assert_eq!(judge.catalog_id(), "llm_judge");
        assert_eq!(judge.expected_lane(), EvaluatorLane::JudgeBroker);
        assert!(!judge.catalog_entry().wasm_safe);

        let mismatch = EvaluatorSpec {
            id: "bad-exact".to_string(),
            lane: EvaluatorLane::JudgeBroker,
            kind: EvaluatorKind::ExactMatch,
        }
        .validate_catalog_lane();
        assert!(matches!(
            mismatch,
            Err(EvalError::EvaluatorLaneMismatch {
                expected: EvaluatorLane::DeterministicWasi,
                actual: EvaluatorLane::JudgeBroker,
                ..
            })
        ));
    }

    #[test]
    fn deterministic_and_judge_lanes_are_separate() {
        let case = EvaluationCase {
            input: serde_json::json!("question"),
            output: serde_json::json!("answer"),
            reference: Some(serde_json::json!("answer")),
            trace: None,
        };
        let exact = EvaluatorSpec {
            id: "exact".to_string(),
            lane: EvaluatorLane::DeterministicWasi,
            kind: EvaluatorKind::ExactMatch,
        };
        let judge = EvaluatorSpec {
            id: "judge".to_string(),
            lane: EvaluatorLane::JudgeBroker,
            kind: EvaluatorKind::LlmJudge {
                rubric: "correctness".to_string(),
                model: "judge-model".to_string(),
            },
        };

        assert_eq!(
            evaluate_deterministic(&exact, &case)
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            1.0
        );
        assert!(matches!(
            evaluate_deterministic(&judge, &case),
            Err(EvalError::RequiresDeterministicLane(_)) | Err(EvalError::RequiresJudgeBroker(_))
        ));
    }

    #[test]
    fn gates_fail_underpowered_and_use_confidence_bounds() {
        let underpowered = compare_paired_scores(
            &[0.9, 0.9],
            &[0.8, 0.8],
            &GatePolicy {
                min_sample_size: 3,
                ..GatePolicy::default()
            },
        );
        assert!(matches!(underpowered, Err(EvalError::Underpowered { .. })));

        let comparison = compare_paired_scores(
            &[1.0, 1.0, 1.0, 1.0, 1.0],
            &[0.0, 0.0, 0.0, 0.0, 0.0],
            &GatePolicy {
                min_sample_size: 5,
                max_regression: 0.05,
                comparison_count: 4,
                ..GatePolicy::default()
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(comparison.decision, GateDecision::FailRegression);
        assert!(comparison.adjusted_alpha < 0.05);
    }
}
