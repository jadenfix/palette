use beater_core::Money;
use beater_schema::{EvalReproducibility, EvaluatorLane};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

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
    #[error("invalid numeric tolerance for evaluator {evaluator_id}: {reason}")]
    InvalidNumericTolerance {
        evaluator_id: String,
        reason: String,
    },
    #[error("evaluator {evaluator_id} requires a reference value")]
    MissingReference { evaluator_id: String },
    #[error("evaluator {evaluator_id} requires trace metric {metric}")]
    MissingTraceMetric {
        evaluator_id: String,
        metric: &'static str,
    },
    #[error(
        "underpowered comparison: sample_size={sample_size}, min_sample_size={min_sample_size}"
    )]
    Underpowered {
        sample_size: usize,
        min_sample_size: usize,
    },
    #[error("statistics error: {0}")]
    Statistics(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EvaluationCase {
    #[schema(value_type = serde_json::Value)]
    pub input: Value,
    #[schema(value_type = serde_json::Value)]
    pub output: Value,
    #[schema(value_type = Option<serde_json::Value>)]
    pub reference: Option<Value>,
    #[schema(value_type = Option<serde_json::Value>)]
    pub trace: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ScoreResult {
    pub score: f64,
    pub label: Option<String>,
    #[schema(value_type = serde_json::Value)]
    pub evidence: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EvaluatorSpec {
    pub id: String,
    pub lane: EvaluatorLane,
    pub kind: EvaluatorKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvaluatorKind {
    ExactMatch,
    RegexMatch {
        pattern: String,
    },
    NumericTolerance {
        abs: f64,
        rel: f64,
    },
    JsonObject,
    CostBudget {
        max_micros: i64,
    },
    LatencyBudgetMs {
        max_ms: u64,
    },
    LlmJudge {
        rubric: String,
        model: String,
    },
    /// Browser world-state success: asserts the final step's observed page
    /// (url and/or DOM) matches the configured target — NOT the agent's
    /// self-reported "done". Reads `trace.browser_steps`.
    BrowserTaskSuccess {
        url_contains: Option<String>,
        dom_contains: Option<String>,
    },
    /// Browser step efficiency: passes when the run used at most `max_steps`
    /// browser steps (catches looping/backtracking). Reads `trace.browser_steps`.
    BrowserStepEfficiency {
        max_steps: u64,
    },
    /// Browser grounding: fraction of element-targeted steps that resolved to
    /// their intended element; score is the ratio, passes at `min_ratio`.
    BrowserGrounding {
        min_ratio: f64,
    },
    /// Browser recovery: passes when the run either hit no errors or recovered
    /// to a successful final step (catches death spirals after a failed action).
    BrowserRecovery,
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
        id: "numeric_tolerance",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Numeric tolerance",
        description:
            "Scores numeric output against a numeric reference within absolute/relative tolerance.",
        requires_reference: true,
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
    EvaluatorCatalogEntry {
        id: "browser_task_success",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Browser task success",
        description: "Scores whether the final browser observation matches a world-state target.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "browser_step_efficiency",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Browser step efficiency",
        description: "Scores whether the browser run stayed within a step budget.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "browser_grounding",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Browser grounding",
        description: "Scores the fraction of element-targeted steps that resolved their element.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: true,
    },
    EvaluatorCatalogEntry {
        id: "browser_recovery",
        lane: EvaluatorLane::DeterministicWasi,
        display_name: "Browser recovery",
        description: "Scores whether the browser run avoided or recovered from action failures.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: true,
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
            Self::NumericTolerance { .. } => "numeric_tolerance",
            Self::JsonObject => "json_object",
            Self::CostBudget { .. } => "cost_budget",
            Self::LatencyBudgetMs { .. } => "latency_budget_ms",
            Self::LlmJudge { .. } => "llm_judge",
            Self::BrowserTaskSuccess { .. } => "browser_task_success",
            Self::BrowserStepEfficiency { .. } => "browser_step_efficiency",
            Self::BrowserGrounding { .. } => "browser_grounding",
            Self::BrowserRecovery => "browser_recovery",
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
            let reference = required_reference(spec, case)?;
            let pass = reference == &case.output;
            Ok(binary_score(pass, "exact_match"))
        }
        EvaluatorKind::RegexMatch { pattern } => {
            let output = case.output.as_str().unwrap_or_default();
            let regex =
                Regex::new(pattern).map_err(|err| EvalError::InvalidRegex(err.to_string()))?;
            Ok(binary_score(regex.is_match(output), "regex_match"))
        }
        EvaluatorKind::NumericTolerance { abs, rel } => {
            numeric_tolerance_score(spec, case, *abs, *rel)
        }
        EvaluatorKind::JsonObject => Ok(binary_score(case.output.is_object(), "json_object")),
        EvaluatorKind::CostBudget { max_micros } => {
            let cost = required_trace_i64(spec, case, "cost_micros")?;
            Ok(binary_score(cost <= *max_micros, "cost_budget"))
        }
        EvaluatorKind::LatencyBudgetMs { max_ms } => {
            let latency = required_trace_u64(spec, case, "latency_ms")?;
            Ok(binary_score(latency <= *max_ms, "latency_budget"))
        }
        EvaluatorKind::LlmJudge { .. } => Err(EvalError::RequiresJudgeBroker(spec.id.clone())),
        EvaluatorKind::BrowserTaskSuccess {
            url_contains,
            dom_contains,
        } => {
            let steps = browser_steps(case);
            let observation = steps
                .last()
                .and_then(|step| step.get("outcome"))
                .and_then(|outcome| outcome.get("observation"));
            let url = observation
                .and_then(|obs| obs.get("url"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let url_ok = url_contains
                .as_ref()
                .map(|needle| url.contains(needle.as_str()))
                .unwrap_or(true);
            // The DOM check is only evaluated when the DOM is present in the
            // trace. An ingested run stores DOM out of line (artifacts), so its
            // browser_steps carry no dom_html — a `dom_contains` check there is
            // unevaluable and must NOT fail (which would manufacture a spurious
            // regression); native runs always inline dom_html, so they evaluate
            // normally including genuine "does not contain" failures.
            let dom_value = observation
                .and_then(|obs| obs.get("dom_html"))
                .and_then(Value::as_str);
            let dom_ok = match (dom_contains.as_ref(), dom_value) {
                (Some(needle), Some(dom)) => dom.contains(needle.as_str()),
                (Some(_), None) => true,
                (None, _) => true,
            };
            let pass = !steps.is_empty() && url_ok && dom_ok;
            Ok(binary_score(pass, "browser_task_success"))
        }
        EvaluatorKind::BrowserStepEfficiency { max_steps } => {
            let count = browser_steps(case).len() as u64;
            Ok(binary_score(
                count > 0 && count <= *max_steps,
                "browser_step_efficiency",
            ))
        }
        EvaluatorKind::BrowserGrounding { min_ratio } => {
            let steps = browser_steps(case);
            let mut targeted = 0u64;
            let mut grounded = 0u64;
            for step in &steps {
                let grounding = step
                    .get("outcome")
                    .and_then(|outcome| outcome.get("grounding"));
                let has_selector = grounding
                    .and_then(|grounding| grounding.get("selector"))
                    .map(|selector| !selector.is_null())
                    .unwrap_or(false);
                if has_selector {
                    // Grounding is only measurable when the producer reported
                    // whether the element matched. An absent/unknown
                    // `matched_element` (e.g. browser-use, which does not expose
                    // grounding; or a Stagehand step with no resolved selector)
                    // is EXCLUDED from the ratio rather than counted as a miss —
                    // otherwise a successful ingested run would score 0.
                    if let Some(matched) = grounding
                        .and_then(|grounding| grounding.get("matched_element"))
                        .and_then(Value::as_bool)
                    {
                        targeted += 1;
                        if matched {
                            grounded += 1;
                        }
                    }
                }
            }
            let ratio = if targeted == 0 {
                1.0
            } else {
                grounded as f64 / targeted as f64
            };
            let pass = ratio >= *min_ratio;
            Ok(ScoreResult {
                score: ratio,
                label: Some(if pass { "pass" } else { "fail" }.to_string()),
                evidence: serde_json::json!({
                    "metric": "browser_grounding",
                    "targeted": targeted,
                    "grounded": grounded,
                    "ratio": ratio,
                    "min_ratio": min_ratio,
                    "pass": pass,
                }),
            })
        }
        EvaluatorKind::BrowserRecovery => {
            let steps = browser_steps(case);
            let status_of = |step: &Value| -> String {
                step.get("outcome")
                    .and_then(|outcome| outcome.get("status"))
                    .and_then(Value::as_str)
                    .unwrap_or("ok")
                    .to_string()
            };
            let any_error = steps.iter().any(|step| status_of(step) == "error");
            let final_ok = steps
                .last()
                .map(|step| status_of(step) == "ok")
                .unwrap_or(false);
            let pass = !steps.is_empty() && (!any_error || final_ok);
            Ok(binary_score(pass, "browser_recovery"))
        }
    }
}

fn required_reference<'a>(
    spec: &EvaluatorSpec,
    case: &'a EvaluationCase,
) -> Result<&'a Value, EvalError> {
    case.reference
        .as_ref()
        .ok_or_else(|| EvalError::MissingReference {
            evaluator_id: spec.id.clone(),
        })
}

fn required_trace_i64(
    spec: &EvaluatorSpec,
    case: &EvaluationCase,
    metric: &'static str,
) -> Result<i64, EvalError> {
    case.trace
        .as_ref()
        .and_then(|trace| trace.get(metric))
        .and_then(Value::as_i64)
        .ok_or_else(|| EvalError::MissingTraceMetric {
            evaluator_id: spec.id.clone(),
            metric,
        })
}

fn required_trace_u64(
    spec: &EvaluatorSpec,
    case: &EvaluationCase,
    metric: &'static str,
) -> Result<u64, EvalError> {
    case.trace
        .as_ref()
        .and_then(|trace| trace.get(metric))
        .and_then(Value::as_u64)
        .ok_or_else(|| EvalError::MissingTraceMetric {
            evaluator_id: spec.id.clone(),
            metric,
        })
}

fn numeric_tolerance_score(
    spec: &EvaluatorSpec,
    case: &EvaluationCase,
    abs: f64,
    rel: f64,
) -> Result<ScoreResult, EvalError> {
    if !abs.is_finite() || !rel.is_finite() || abs < 0.0 || rel < 0.0 {
        return Err(EvalError::InvalidNumericTolerance {
            evaluator_id: spec.id.clone(),
            reason: "abs and rel must be finite non-negative numbers".to_string(),
        });
    }

    let output = case.output.as_f64();
    let reference = required_reference(spec, case)?.as_f64();
    let (difference, allowed, pass) = match (output, reference) {
        (Some(output), Some(reference)) => {
            let difference = (output - reference).abs();
            let allowed = abs.max(rel * reference.abs());
            (Some(difference), Some(allowed), difference <= allowed)
        }
        _ => (None, None, false),
    };

    Ok(ScoreResult {
        score: if pass { 1.0 } else { 0.0 },
        label: Some(if pass { "pass" } else { "fail" }.to_string()),
        evidence: serde_json::json!({
            "metric": "numeric_tolerance",
            "output": output,
            "reference": reference,
            "difference": difference,
            "allowed": allowed,
            "abs": abs,
            "rel": rel,
            "pass": pass,
        }),
    })
}

/// Extract the `browser_steps` array (serialized `StepTriple`s) from a case
/// trace. Returns an empty vec when absent — every browser evaluator degrades to
/// a deterministic "fail/neutral" rather than erroring on a non-browser trace.
fn browser_steps(case: &EvaluationCase) -> Vec<Value> {
    case.trace
        .as_ref()
        .and_then(|trace| trace.get("browser_steps"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn binary_score(pass: bool, metric: &str) -> ScoreResult {
    ScoreResult {
        score: if pass { 1.0 } else { 0.0 },
        label: Some(if pass { "pass" } else { "fail" }.to_string()),
        evidence: serde_json::json!({ "metric": metric, "pass": pass }),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JudgeRequest {
    pub rubric: String,
    pub model: String,
    #[schema(value_type = serde_json::Value)]
    pub input: Value,
    #[schema(value_type = serde_json::Value)]
    pub output: Value,
    #[schema(value_type = Option<serde_json::Value>)]
    pub reference: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JudgeResponse {
    pub score: f64,
    pub rationale: String,
    pub cost: Money,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExperimentComparison {
    pub sample_size: usize,
    pub baseline_mean: f64,
    pub candidate_mean: f64,
    pub delta: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    /// Real two-sided p-value from `test`. The previous normal-approximation path
    /// reported no p-value at all.
    pub p_value: f64,
    pub decision: GateDecision,
    pub test: StatisticalTest,
    pub adjusted_alpha: f64,
    /// Minimum detectable effect at the current sample size, in the metric's own
    /// units, at the gate's (adjusted) alpha and the standard power of 0.8
    /// (§10.3 #5). Populated only when `decision` is `Inconclusive` — the
    /// comparison lacked the power to resolve the regression bound, and
    /// regressions smaller than this are invisible at this N. `None` on a
    /// conclusive decision (or when the paired differences have zero spread, so
    /// no effect-scale is defined). This replaces a bare "underpowered" flag with
    /// the actionable "how small an effect could we even have seen" number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mde: Option<f64>,
    /// Number of paired observations that would be required to detect the
    /// *observed* effect at the gate's (adjusted) alpha and power 0.8 (§10.3 #5).
    /// Populated only when `decision` is `Inconclusive` and the observed effect is
    /// non-degenerate (non-zero delta over non-zero difference spread). `None`
    /// otherwise. This answers "how many more cases would have made this
    /// conclusive?".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_n: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
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

/// The statistical test that produced an [`ExperimentComparison`]. These mirror
/// `beater_stats::TestKind`; the gate records which method was actually used so a
/// reader can tell a t-test result from an exact McNemar one. The old single
/// `PairedNormalApproximation` (a hard-coded-z normal approximation with no
/// p-value) is gone — see `beater-stats`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalTest {
    /// Student's paired t-test (continuous paired metric).
    PairedT,
    /// Exact McNemar test (paired binary outcome).
    McnemarExact,
}

impl From<beater_stats::TestKind> for StatisticalTest {
    fn from(kind: beater_stats::TestKind) -> Self {
        match kind {
            beater_stats::TestKind::PairedT => StatisticalTest::PairedT,
            beater_stats::TestKind::McnemarExact => StatisticalTest::McnemarExact,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
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
    // Pairs must align one-to-one; a length mismatch is a caller bug, not
    // something to silently paper over by truncating to the shorter prefix.
    if baseline.len() != candidate.len() {
        return Err(EvalError::Statistics(format!(
            "baseline and candidate must be the same length, got {} and {}",
            baseline.len(),
            candidate.len()
        )));
    }
    let n = baseline.len();
    if n < policy.min_sample_size {
        return Err(EvalError::Underpowered {
            sample_size: n,
            min_sample_size: policy.min_sample_size,
        });
    }
    // Reject non-finite scores up front so they cannot slip past the degenerate
    // single-case branch below and silently produce a Pass on NaN.
    if baseline
        .iter()
        .chain(candidate.iter())
        .any(|score| !score.is_finite())
    {
        return Err(EvalError::Statistics(
            "scores must be finite (no NaN or infinity)".to_string(),
        ));
    }
    if !(policy.alpha.is_finite() && policy.alpha > 0.0 && policy.alpha < 1.0) {
        return Err(EvalError::Statistics(format!(
            "alpha must be in (0, 1), got {}",
            policy.alpha
        )));
    }

    // Single-step Bonferroni correction across the comparison family: the
    // per-comparison level the CI and decision are computed at. No lower clamp — a
    // large `comparison_count` must genuinely shrink alpha; clamping it up would let
    // the family-wise error rate exceed the requested level. `compare_paired`
    // validates the result is a usable alpha in (0, 1).
    let adjusted_alpha = policy.alpha / policy.comparison_count.max(1) as f64;

    if n == 0 {
        return Err(EvalError::Statistics("no scores to compare".to_string()));
    }

    // A single paired observation has no sampling variability, so a real
    // variance-based test is undefined — `beater-stats` correctly refuses n < 2.
    // This is the deterministic single-case smoke-gate regime (a caller opts in by
    // setting `min_sample_size = 1`): the interval collapses to the point estimate,
    // and the p-value is 1.0 because one sample carries no power to reject the null.
    // The gate still decides from that degenerate interval against the regression
    // bound, preserving deterministic single-case behavior.
    if n < 2 {
        let delta =
            candidate.first().copied().unwrap_or(0.0) - baseline.first().copied().unwrap_or(0.0);
        let decision = if delta < -policy.max_regression {
            GateDecision::FailRegression
        } else {
            GateDecision::Pass
        };
        return Ok(ExperimentComparison {
            sample_size: n,
            baseline_mean: mean(baseline),
            candidate_mean: mean(candidate),
            delta,
            ci_low: delta,
            ci_high: delta,
            p_value: 1.0,
            decision,
            test: StatisticalTest::PairedT,
            adjusted_alpha,
            // The single-case smoke-gate regime never returns `Inconclusive`, so
            // there is no underpowered verdict to annotate.
            mde: None,
            required_n: None,
        });
    }

    // Real statistics: a method-appropriate test (exact McNemar for paired binary
    // outcomes, Student's paired t otherwise) with a real p-value and a CI whose
    // nominal level equals its actual level — not the old hard-coded-z normal
    // approximation. See `beater-stats`.
    let outcome = beater_stats::compare_paired(baseline, candidate, adjusted_alpha)
        .map_err(|err| EvalError::Statistics(err.to_string()))?;

    // Every test reports the mean difference as its estimate; the McNemar path
    // also carries a difference CI, so a CI is always present here.
    let ci = outcome.ci.unwrap_or(beater_stats::ConfidenceInterval {
        low: outcome.estimate,
        high: outcome.estimate,
        confidence: 1.0 - adjusted_alpha,
    });
    let delta = outcome.estimate;
    let decision = if ci.high < -policy.max_regression {
        GateDecision::FailRegression
    } else if ci.low >= -policy.max_regression {
        GateDecision::Pass
    } else {
        GateDecision::Inconclusive
    };

    // §10.3 #5: an inconclusive verdict means the comparison lacked the power to
    // resolve the regression bound. Rather than a bare "underpowered", report the
    // minimum detectable effect at this N and the sample size that would have made
    // the *observed* effect detectable — both at the gate's alpha and power 0.8.
    // The planning math is standardized (Cohen's d), so we scale by the SD of the
    // paired differences to express the MDE in the metric's own units and to
    // standardize the observed delta. This covers both the paired-t and (as a
    // normal approximation) the McNemar path, since both reduce to a mean
    // difference of the paired observations.
    let (mde, required_n) = if decision == GateDecision::Inconclusive {
        power_annotations(baseline, candidate, n, delta, adjusted_alpha)
    } else {
        (None, None)
    };

    Ok(ExperimentComparison {
        sample_size: n,
        baseline_mean: mean(baseline),
        candidate_mean: mean(candidate),
        delta,
        ci_low: ci.low,
        ci_high: ci.high,
        p_value: outcome.p_value,
        decision,
        test: outcome.test.into(),
        adjusted_alpha,
        mde,
        required_n,
    })
}

/// Compute the §10.3 #5 power annotations for an inconclusive comparison: the
/// minimum detectable effect at the current sample size (in the metric's own
/// units) and the sample size required to detect the observed effect, both at the
/// gate's `alpha` and the standard power of 0.8.
///
/// Returns `(None, None)` when the paired differences have no spread (no
/// effect-scale is defined), and a `None` `required_n` when the observed effect is
/// exactly zero (no finite N detects a null effect) — the MDE is still reported in
/// that case.
fn power_annotations(
    baseline: &[f64],
    candidate: &[f64],
    n: usize,
    delta: f64,
    alpha: f64,
) -> (Option<f64>, Option<usize>) {
    let differences: Vec<f64> = candidate
        .iter()
        .zip(baseline.iter())
        .map(|(c, b)| c - b)
        .collect();
    let sd = std_dev(&differences);
    if !sd.is_finite() || sd <= 0.0 {
        // Zero (or non-finite) spread: a standardized effect is undefined.
        return (None, None);
    }

    // MDE in the metric's own units = standardized MDE × SD of the differences.
    let mde = beater_stats::minimum_detectable_effect(n, alpha, beater_stats::DEFAULT_POWER)
        .ok()
        .map(|d| d * sd);

    // Required N to detect the observed standardized effect d = |delta| / SD.
    let required_n =
        beater_stats::required_sample_size(delta / sd, alpha, beater_stats::DEFAULT_POWER).ok();

    (mde, required_n)
}

/// Unbiased (n − 1) sample standard deviation; 0.0 for fewer than two values.
fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let sum_sq: f64 = values.iter().map(|v| (v - m).powi(2)).sum();
    (sum_sq / (values.len() as f64 - 1.0)).sqrt()
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// What the platform can still provide when an [`EvalResult`] is rerun. The
/// reproducibility manifest recorded at scoring time pins the inputs that made a
/// score reproducible; on rerun those inputs may have disappeared (a judge model
/// retired, a deterministic evaluator's wasm artifact garbage-collected, an input
/// artifact expired). This describes the *current* availability so
/// [`detect_non_reproducible_reason`] can compare it against the pinned manifest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RerunEnvironment {
    /// Judge model ids still served (e.g. not deprecated/retired).
    pub available_judge_model_ids: BTreeSet<String>,
    /// Deterministic-evaluator wasm hashes still resolvable in the registry.
    pub available_wasm_hashes: BTreeSet<String>,
    /// Input-artifact content hashes still present in object storage.
    pub available_input_artifact_hashes: BTreeSet<String>,
}

impl RerunEnvironment {
    /// Construct from owned strings; convenience for callers that hold `Vec`s.
    pub fn new(
        judge_model_ids: impl IntoIterator<Item = String>,
        wasm_hashes: impl IntoIterator<Item = String>,
        input_artifact_hashes: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            available_judge_model_ids: judge_model_ids.into_iter().collect(),
            available_wasm_hashes: wasm_hashes.into_iter().collect(),
            available_input_artifact_hashes: input_artifact_hashes.into_iter().collect(),
        }
    }
}

/// Determine whether an evaluation is still reproducible given the inputs its
/// [`EvalReproducibility`] manifest pinned and what the [`RerunEnvironment`] can
/// still supply. Returns `Some(reason)` — the human-readable value to store in
/// [`beater_schema::EvalResult::non_reproducible_reason`] — when one or more
/// pinned inputs are unavailable, or `None` when the rerun is fully reproducible.
///
/// Detection covers the three inputs that most often go missing:
/// * `judge_model_id` — the judge model was retired/deprecated;
/// * `wasm_hash` — the deterministic evaluator's wasm artifact is gone;
/// * `input_artifact_hashes` — one or more input artifacts expired.
///
/// A manifest that pinned no judge model and no wasm hash (e.g. a pure
/// deterministic evaluator with inline inputs) is reproducible as long as its
/// input artifacts are available.
pub fn detect_non_reproducible_reason(
    manifest: &EvalReproducibility,
    environment: &RerunEnvironment,
) -> Option<String> {
    let mut reasons = Vec::new();

    if let Some(judge_model_id) = &manifest.judge_model_id {
        if !environment
            .available_judge_model_ids
            .contains(judge_model_id)
        {
            reasons.push(format!(
                "judge model '{judge_model_id}' is no longer available (deprecated or retired)"
            ));
        }
    }

    if let Some(wasm_hash) = &manifest.wasm_hash {
        if !environment
            .available_wasm_hashes
            .contains(wasm_hash.as_str())
        {
            reasons.push(format!(
                "evaluator wasm artifact '{}' is unavailable",
                wasm_hash.as_str()
            ));
        }
    }

    let missing_inputs: Vec<&str> = manifest
        .input_artifact_hashes
        .iter()
        .filter(|hash| {
            !environment
                .available_input_artifact_hashes
                .contains(hash.as_str())
        })
        .map(|hash| hash.as_str())
        .collect();
    if !missing_inputs.is_empty() {
        reasons.push(format!(
            "input artifact(s) unavailable: {}",
            missing_inputs.join(", ")
        ));
    }

    if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluator_catalog_classifies_execution_lanes() {
        let catalog = evaluator_catalog();
        assert_eq!(catalog.len(), 11);

        let exact = evaluator_catalog_entry("exact_match")
            .unwrap_or_else(|| panic!("exact_match catalog entry should exist"));
        assert_eq!(exact.lane, EvaluatorLane::DeterministicWasi);
        assert!(exact.requires_reference);
        assert!(exact.wasm_safe);

        let cost = EvaluatorKind::CostBudget { max_micros: 10 };
        assert_eq!(cost.catalog_id(), "cost_budget");
        assert_eq!(cost.expected_lane(), EvaluatorLane::DeterministicWasi);
        assert!(cost.catalog_entry().consumes_trace);

        let numeric = EvaluatorKind::NumericTolerance { abs: 0.1, rel: 0.0 };
        assert_eq!(numeric.catalog_id(), "numeric_tolerance");
        assert!(numeric.catalog_entry().requires_reference);
        assert!(!numeric.catalog_entry().consumes_trace);

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
    fn numeric_tolerance_scores_against_numeric_reference() {
        let spec = deterministic_spec(EvaluatorKind::NumericTolerance {
            abs: 0.05,
            rel: 0.01,
        });
        let passing = EvaluationCase {
            input: serde_json::json!("estimate"),
            output: serde_json::json!(100.9),
            reference: Some(serde_json::json!(100.0)),
            trace: None,
        };
        let failing = EvaluationCase {
            output: serde_json::json!(102.0),
            ..passing.clone()
        };
        let non_numeric = EvaluationCase {
            output: serde_json::json!("100.0"),
            ..passing.clone()
        };

        let pass = evaluate_deterministic(&spec, &passing).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(pass.score, 1.0);
        let difference = pass.evidence["difference"]
            .as_f64()
            .unwrap_or_else(|| panic!("difference should be numeric"));
        let allowed = pass.evidence["allowed"]
            .as_f64()
            .unwrap_or_else(|| panic!("allowed should be numeric"));
        assert!((difference - 0.9).abs() < 1e-12);
        assert_eq!(allowed, 1.0);

        let fail = evaluate_deterministic(&spec, &failing).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(fail.score, 0.0);
        assert_eq!(fail.evidence["pass"], serde_json::json!(false));

        let rejected =
            evaluate_deterministic(&spec, &non_numeric).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(rejected.score, 0.0);
        assert_eq!(rejected.evidence["difference"], serde_json::json!(null));
    }

    #[test]
    fn numeric_tolerance_rejects_invalid_thresholds() {
        let spec = deterministic_spec(EvaluatorKind::NumericTolerance {
            abs: -0.1,
            rel: 0.0,
        });
        let case = EvaluationCase {
            input: serde_json::json!(null),
            output: serde_json::json!(1.0),
            reference: Some(serde_json::json!(1.0)),
            trace: None,
        };

        assert!(matches!(
            evaluate_deterministic(&spec, &case),
            Err(EvalError::InvalidNumericTolerance { .. })
        ));
    }

    #[test]
    fn reference_scorers_reject_missing_reference() {
        let case = EvaluationCase {
            input: serde_json::json!("question"),
            output: serde_json::json!("answer"),
            reference: None,
            trace: None,
        };

        assert!(matches!(
            evaluate_deterministic(&deterministic_spec(EvaluatorKind::ExactMatch), &case),
            Err(EvalError::MissingReference { evaluator_id }) if evaluator_id == "exact_match"
        ));

        assert!(matches!(
            evaluate_deterministic(
                &deterministic_spec(EvaluatorKind::NumericTolerance { abs: 0.0, rel: 0.0 }),
                &case,
            ),
            Err(EvalError::MissingReference { evaluator_id }) if evaluator_id == "numeric_tolerance"
        ));
    }

    #[test]
    fn budget_scorers_reject_missing_trace_metrics() {
        let case = EvaluationCase {
            input: serde_json::json!(null),
            output: serde_json::json!(null),
            reference: None,
            trace: Some(serde_json::json!({})),
        };

        assert!(matches!(
            evaluate_deterministic(
                &deterministic_spec(EvaluatorKind::CostBudget { max_micros: 100 }),
                &case,
            ),
            Err(EvalError::MissingTraceMetric { evaluator_id, metric })
                if evaluator_id == "cost_budget" && metric == "cost_micros"
        ));

        assert!(matches!(
            evaluate_deterministic(
                &deterministic_spec(EvaluatorKind::LatencyBudgetMs { max_ms: 100 }),
                &case,
            ),
            Err(EvalError::MissingTraceMetric { evaluator_id, metric })
                if evaluator_id == "latency_budget_ms" && metric == "latency_ms"
        ));
    }

    #[test]
    fn budget_scorers_score_present_trace_metrics() {
        let case = EvaluationCase {
            input: serde_json::json!(null),
            output: serde_json::json!(null),
            reference: None,
            trace: Some(serde_json::json!({
                "cost_micros": 42,
                "latency_ms": 99,
            })),
        };

        let cost = evaluate_deterministic(
            &deterministic_spec(EvaluatorKind::CostBudget { max_micros: 100 }),
            &case,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(cost.score, 1.0);

        let latency = evaluate_deterministic(
            &deterministic_spec(EvaluatorKind::LatencyBudgetMs { max_ms: 50 }),
            &case,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(latency.score, 0.0);
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

        // Ten paired cases that all regress pass->fail. The exact McNemar p
        // (2 * 0.5^10 ~ 2e-3) clears the 4x-Bonferroni-corrected alpha (0.0125),
        // so the score-interval upper bound excludes the regression threshold and
        // the gate fails the candidate.
        let comparison = compare_paired_scores(
            &[1.0; 10],
            &[0.0; 10],
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
        // A conclusive verdict carries no underpowered annotation.
        assert!(comparison.mde.is_none());
        assert!(comparison.required_n.is_none());
    }

    #[test]
    fn inconclusive_gate_reports_mde_and_required_n() {
        // A tiny, noisy paired effect: the difference distribution straddles zero,
        // so the CI straddles the regression bound and the gate is *inconclusive*
        // — exactly the underpowered regime §10.3 #5 must annotate.
        let baseline = vec![0.0; 8];
        let candidate = vec![0.5, -0.5, 0.5, -0.5, 0.5, -0.5, 0.5, -0.4];
        let comparison = compare_paired_scores(
            &baseline,
            &candidate,
            &GatePolicy {
                min_sample_size: 4,
                ..GatePolicy::default()
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            comparison.decision,
            GateDecision::Inconclusive,
            "expected an inconclusive (underpowered) verdict, got {:?}",
            comparison.decision
        );

        // Instead of a bare "underpowered", the comparison now reports actionable
        // power numbers: a finite, positive MDE at this N and a finite required-N
        // to detect the observed effect.
        let mde = comparison
            .mde
            .unwrap_or_else(|| panic!("inconclusive comparison must report an MDE"));
        assert!(mde.is_finite() && mde > 0.0, "mde = {mde}");
        let required_n = comparison
            .required_n
            .unwrap_or_else(|| panic!("inconclusive comparison must report required_n"));
        // The observed effect is far smaller than the spread, so detecting it
        // would take many more than the 8 cases we ran.
        assert!(
            required_n > comparison.sample_size,
            "required_n = {required_n}"
        );
    }

    #[test]
    fn gate_is_inconclusive_when_too_few_discordant_for_exact_significance() {
        // Five paired cases all regress, but with a 4x correction the exact
        // McNemar p (2 * 0.5^5 = 0.0625) does NOT clear alpha = 0.0125. The honest
        // verdict is Inconclusive, not FailRegression: the score interval used for
        // the decision stays consistent with the exact test (the old Wald-CI path
        // wrongly reported a regression here).
        let comparison = compare_paired_scores(
            &[1.0; 5],
            &[0.0; 5],
            &GatePolicy {
                min_sample_size: 5,
                max_regression: 0.05,
                comparison_count: 4,
                ..GatePolicy::default()
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(comparison.decision, GateDecision::Inconclusive);
        assert!(
            (comparison.p_value - 0.0625).abs() < 1e-9,
            "p={}",
            comparison.p_value
        );
    }

    #[test]
    fn mismatched_score_lengths_error() {
        let result = compare_paired_scores(&[1.0, 1.0, 1.0], &[1.0, 1.0], &GatePolicy::default());
        assert!(matches!(result, Err(EvalError::Statistics(_))));
    }

    #[test]
    fn non_finite_scores_error() {
        let result = compare_paired_scores(
            &[1.0, 1.0, 1.0, 1.0, 1.0],
            &[1.0, f64::NAN, 1.0, 1.0, 1.0],
            &GatePolicy {
                min_sample_size: 1,
                ..GatePolicy::default()
            },
        );
        assert!(matches!(result, Err(EvalError::Statistics(_))));
    }

    fn browser_step(action: &str, selector: Option<&str>, matched: bool, status: &str) -> Value {
        serde_json::json!({
            "action": { "action": action },
            "outcome": {
                "status": status,
                "grounding": {
                    "selector": selector,
                    "selector_existed": matched,
                    "matched_element": matched,
                },
                "observation": {
                    "url": "https://shop.example/checkout/confirmation",
                    "dom_html": "<div id=\"order-confirmed\">Thank you</div>",
                },
            },
        })
    }

    fn browser_case(steps: Vec<Value>) -> EvaluationCase {
        EvaluationCase {
            input: serde_json::json!("book a flight"),
            output: serde_json::json!(null),
            reference: None,
            trace: Some(serde_json::json!({ "browser_steps": steps })),
        }
    }

    fn deterministic_spec(kind: EvaluatorKind) -> EvaluatorSpec {
        EvaluatorSpec {
            id: kind.catalog_id().to_string(),
            lane: EvaluatorLane::DeterministicWasi,
            kind,
        }
    }

    fn judge_manifest() -> EvalReproducibility {
        use beater_core::{
            AgentReleaseId, DatasetCaseId, DatasetVersionId, EvaluatorVersionId, Sha256Hash,
        };
        EvalReproducibility {
            dataset_version_id: DatasetVersionId::new("dsv-1")
                .unwrap_or_else(|err| panic!("{err}")),
            dataset_case_id: DatasetCaseId::new("case-1").unwrap_or_else(|err| panic!("{err}")),
            agent_release_id: AgentReleaseId::new("rel-1").unwrap_or_else(|err| panic!("{err}")),
            prompt_version_id: None,
            evaluator_version_id: EvaluatorVersionId::new("ev-1")
                .unwrap_or_else(|err| panic!("{err}")),
            code_hash: None,
            wasm_hash: None,
            wasi_abi_version: None,
            judge_model_id: Some("gpt-judge-2024-01".to_string()),
            judge_provider: Some("openai".to_string()),
            judge_parameters: serde_json::json!({ "temperature": 0.0 }),
            judge_seed: Some(7),
            judge_rubric_version: Some("v1".to_string()),
            normalizer_version: "beater-otlp-v1".to_string(),
            trace_schema_version: 1,
            input_artifact_hashes: vec![
                Sha256Hash::new("input-hash-a").unwrap_or_else(|err| panic!("{err}"))
            ],
        }
    }

    #[test]
    fn browser_evaluator_catalog_ids_match_entries() {
        for kind in [
            EvaluatorKind::BrowserTaskSuccess {
                url_contains: None,
                dom_contains: None,
            },
            EvaluatorKind::BrowserStepEfficiency { max_steps: 1 },
            EvaluatorKind::BrowserGrounding { min_ratio: 1.0 },
            EvaluatorKind::BrowserRecovery,
        ] {
            assert_eq!(kind.catalog_entry().id, kind.catalog_id());
            assert_eq!(kind.expected_lane(), EvaluatorLane::DeterministicWasi);
            assert!(kind.catalog_entry().consumes_trace);
        }
    }

    #[test]
    fn browser_task_success_checks_world_state_not_self_report() {
        let case = browser_case(vec![browser_step("click", Some("#pay"), true, "ok")]);
        let pass = deterministic_spec(EvaluatorKind::BrowserTaskSuccess {
            url_contains: Some("confirmation".to_string()),
            dom_contains: Some("order-confirmed".to_string()),
        });
        assert_eq!(
            evaluate_deterministic(&pass, &case)
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            1.0
        );

        let fail = deterministic_spec(EvaluatorKind::BrowserTaskSuccess {
            url_contains: Some("error".to_string()),
            dom_contains: None,
        });
        assert_eq!(
            evaluate_deterministic(&fail, &case)
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            0.0
        );

        // No browser steps => cannot have succeeded.
        let empty = evaluate_deterministic(
            &deterministic_spec(EvaluatorKind::BrowserTaskSuccess {
                url_contains: None,
                dom_contains: None,
            }),
            &browser_case(vec![]),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(empty.score, 0.0);
    }

    #[test]
    fn browser_task_success_skips_dom_check_when_dom_absent() {
        // An ingested step carries url but no dom_html (DOM lives in artifacts).
        // A dom_contains check is unevaluable and must not fail the run; the
        // url check still applies.
        let ingested = serde_json::json!({
            "action": { "action": "click", "args": {} },
            "outcome": {
                "status": "ok",
                "grounding": { "selector": "#pay", "selector_existed": true, "matched_element": true },
                "observation": { "url": "https://shop/checkout/confirmation" },
            },
        });
        let spec = deterministic_spec(EvaluatorKind::BrowserTaskSuccess {
            url_contains: Some("confirmation".to_string()),
            dom_contains: Some("order-confirmed".to_string()),
        });
        let result = evaluate_deterministic(&spec, &browser_case(vec![ingested.clone()]))
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(result.score, 1.0, "url matches and dom check is skipped");

        // But a wrong url still fails even when the dom check is skipped.
        let url_fail = deterministic_spec(EvaluatorKind::BrowserTaskSuccess {
            url_contains: Some("error".to_string()),
            dom_contains: Some("order-confirmed".to_string()),
        });
        assert_eq!(
            evaluate_deterministic(&url_fail, &browser_case(vec![ingested]))
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            0.0
        );
    }

    #[test]
    fn deprecated_judge_model_marks_rerun_non_reproducible() {
        // R5.5 deprecated-model test: the judge model that produced the original
        // score has been retired, so it is absent from the rerun environment.
        let manifest = judge_manifest();
        let environment = RerunEnvironment::new(
            // The deprecated model id is NOT present; a newer model is.
            ["gpt-judge-2025-06".to_string()],
            std::iter::empty(),
            ["input-hash-a".to_string()],
        );
        let reason = detect_non_reproducible_reason(&manifest, &environment)
            .unwrap_or_else(|| panic!("deprecated judge model must be flagged"));
        assert!(
            reason.contains("gpt-judge-2024-01") && reason.contains("no longer available"),
            "reason should name the deprecated model: {reason}"
        );
    }

    #[test]
    fn browser_step_efficiency_enforces_budget() {
        let steps = vec![
            browser_step("click", Some("#a"), true, "ok"),
            browser_step("click", Some("#b"), true, "ok"),
        ];
        let within = deterministic_spec(EvaluatorKind::BrowserStepEfficiency { max_steps: 3 });
        assert_eq!(
            evaluate_deterministic(&within, &browser_case(steps.clone()))
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            1.0
        );
        let over = deterministic_spec(EvaluatorKind::BrowserStepEfficiency { max_steps: 1 });
        assert_eq!(
            evaluate_deterministic(&over, &browser_case(steps))
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            0.0
        );
    }

    #[test]
    fn browser_grounding_reports_resolution_ratio() {
        let steps = vec![
            browser_step("click", Some("#a"), true, "ok"),
            browser_step("click", Some("#b"), false, "error"),
            browser_step("scroll", None, true, "ok"), // not element-targeted, ignored
        ];
        let spec = deterministic_spec(EvaluatorKind::BrowserGrounding { min_ratio: 0.75 });
        let result = evaluate_deterministic(&spec, &browser_case(steps))
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(result.score, 0.5); // 1 of 2 targeted steps grounded
        assert_eq!(result.label.as_deref(), Some("fail"));
    }

    #[test]
    fn browser_grounding_excludes_unknown_matched_element() {
        // An ingested span can carry a selector but OMIT matched_element when the
        // SDK doesn't expose grounding (browser-use). Such steps must be excluded
        // from the ratio, not scored as misses — else a successful run scores 0.
        let selector_no_grounding = serde_json::json!({
            "action": { "action": "click" },
            "outcome": {
                "status": "ok",
                "grounding": { "selector": "#pay", "selector_existed": true },
                "observation": { "url": "https://shop/confirm" },
            },
        });
        // Only this step has a known matched_element (true).
        let known = browser_step("click", Some("#cart"), true, "ok");
        let spec = deterministic_spec(EvaluatorKind::BrowserGrounding { min_ratio: 1.0 });
        let result =
            evaluate_deterministic(&spec, &browser_case(vec![selector_no_grounding, known]))
                .unwrap_or_else(|err| panic!("{err}"));
        // 1 of 1 *known* targeted steps grounded => 1.0, not 0.5.
        assert_eq!(result.score, 1.0);
        assert_eq!(result.evidence.get("targeted"), Some(&serde_json::json!(1)));
    }

    #[test]
    fn browser_recovery_distinguishes_spiral_from_recovery() {
        let recovered = vec![
            browser_step("click", Some("#a"), false, "error"),
            browser_step("click", Some("#a"), true, "ok"),
        ];
        let spec = deterministic_spec(EvaluatorKind::BrowserRecovery);
        assert_eq!(
            evaluate_deterministic(&spec, &browser_case(recovered))
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            1.0
        );

        let spiraled = vec![
            browser_step("click", Some("#a"), false, "error"),
            browser_step("click", Some("#b"), false, "error"),
        ];
        assert_eq!(
            evaluate_deterministic(&spec, &browser_case(spiraled))
                .unwrap_or_else(|err| panic!("{err}"))
                .score,
            0.0
        );
    }

    #[test]
    fn reproducible_rerun_returns_no_reason() {
        let manifest = judge_manifest();
        let environment = RerunEnvironment::new(
            ["gpt-judge-2024-01".to_string()],
            std::iter::empty(),
            ["input-hash-a".to_string()],
        );
        assert_eq!(
            detect_non_reproducible_reason(&manifest, &environment),
            None
        );
    }

    #[test]
    fn missing_wasm_and_input_artifacts_are_reported_together() {
        use beater_core::Sha256Hash;
        let mut manifest = judge_manifest();
        manifest.judge_model_id = None;
        manifest.judge_provider = None;
        manifest.wasm_hash =
            Some(Sha256Hash::new("wasm-hash-1").unwrap_or_else(|err| panic!("{err}")));
        manifest.input_artifact_hashes = vec![
            Sha256Hash::new("input-hash-a").unwrap_or_else(|err| panic!("{err}")),
            Sha256Hash::new("input-hash-b").unwrap_or_else(|err| panic!("{err}")),
        ];
        // Neither the wasm artifact nor input-hash-b is still available.
        let environment = RerunEnvironment::new(
            std::iter::empty(),
            std::iter::empty(),
            ["input-hash-a".to_string()],
        );
        let reason = detect_non_reproducible_reason(&manifest, &environment)
            .unwrap_or_else(|| panic!("missing artifacts must be flagged"));
        assert!(reason.contains("wasm-hash-1"), "{reason}");
        assert!(reason.contains("input-hash-b"), "{reason}");
        assert!(!reason.contains("input-hash-a"), "{reason}");
    }
}
