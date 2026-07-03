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
    // Conversation-level scorers (§20.10 #7.8 / R18.8). These judge a whole
    // session/thread group rather than a single turn; they route through the
    // judge broker (§10.1 judge lane) like the rubric LLM judge. §10.4
    // assumption: the §10.1.1 debiasing protocol (calibration, position-swap,
    // small panel) holds. Conversations are the cluster unit, so scores
    // aggregate with trajectory/conversation-clustered SE (§10.3 #1), never as
    // a mean of independent per-turn scores.
    EvaluatorCatalogEntry {
        id: "conversation_coherence",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "Conversation coherence",
        description: "Judges whether turns across a session/thread stay mutually consistent and on-topic. §10.4 assumption: judge debiasing (§10.1.1) holds; aggregates with conversation-clustered SE, not per-turn means.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: false,
    },
    EvaluatorCatalogEntry {
        id: "session_completeness",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "Session completeness",
        description: "Judges whether a session/thread resolved the user's overall goal. §10.4 assumption: judge debiasing (§10.1.1) holds; aggregates with conversation-clustered SE, not per-turn means.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: false,
    },
    EvaluatorCatalogEntry {
        id: "user_frustration",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "User frustration",
        description: "Judges signs of user frustration (repetition, escalation, abandonment) across a session/thread. §10.4 assumption: judge debiasing (§10.1.1) holds; aggregates with conversation-clustered SE, not per-turn means.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: false,
    },
    // Agent-trajectory scorers (§20.10 #7.8 / R18.8). These judge an ordered
    // span sequence (plan→step→tool→…) and route through the judge broker for
    // quality scoring (§10.4 trajectory / process-reward row, judge lane).
    // §10.4 assumption: trajectory quality is jointly modeled (AgentPRM-style
    // promise+progress), NOT a mean of independent per-step scores; per-step
    // scores aggregate with trajectory-clustered SE (§10.3 #1, cluster =
    // trajectory) [arXiv:2511.08325; arXiv:2507.21504].
    EvaluatorCatalogEntry {
        id: "tool_selection_quality",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "Tool selection quality",
        description: "Judges whether the agent chose appropriate tools for each step of the trajectory. §10.4 assumption: trajectory quality is jointly modeled; aggregates with trajectory-clustered SE, not per-step means.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: false,
    },
    EvaluatorCatalogEntry {
        id: "tool_error_rate",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "Tool error rate",
        description: "Judges the rate and severity of tool-call failures across the trajectory. §10.4 assumption: trajectory quality is jointly modeled; aggregates with trajectory-clustered SE, not per-step means.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: false,
    },
    EvaluatorCatalogEntry {
        id: "action_completion",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "Action completion",
        description: "Judges whether the agent completed the actions its trajectory set out to perform. §10.4 assumption: trajectory quality is jointly modeled; aggregates with trajectory-clustered SE, not per-step means.",
        requires_reference: false,
        consumes_trace: true,
        wasm_safe: false,
    },
    EvaluatorCatalogEntry {
        id: "agent_flow",
        lane: EvaluatorLane::JudgeBroker,
        display_name: "Agent flow",
        description: "Judges the overall coherence and progress of the agent's step sequence (promise+progress). §10.4 assumption: trajectory quality is jointly modeled; aggregates with trajectory-clustered SE, not per-step means.",
        requires_reference: false,
        consumes_trace: true,
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
            let cost = required_trace_u64(spec, case, "cost_micros")?;
            Ok(binary_score(
                *max_micros >= 0 && cost <= *max_micros as u64,
                "cost_budget",
            ))
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

/// The statistical test that produced an [`ExperimentComparison`]. The gate
/// records which method was **actually executed** so a reader can tell a
/// t-test result from an exact McNemar, Wilcoxon, bootstrap, cluster-robust, or
/// anytime-valid sequential one. The old single `PairedNormalApproximation`
/// (a hard-coded-z normal approximation with no p-value) is gone — see
/// `beater-stats`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalTest {
    /// Student's paired t-test (continuous paired metric).
    PairedT,
    /// Exact McNemar test (paired binary outcome).
    McnemarExact,
    /// Wilcoxon signed-rank (continuous, non-normal paired differences); the
    /// reported delta/CI are on the Hodges-Lehmann pseudo-median scale.
    WilcoxonSignedRank,
    /// Paired percentile bootstrap (small N / unclear assumptions), with a
    /// fixed, documented seed so gate decisions are reproducible.
    PairedBootstrap,
    /// Cluster-robust paired t-test (CR1 sandwich SE, `G − 1` df) for a design
    /// whose `cluster_key` marks the observations as non-independent.
    ClusteredPairedT,
    /// Anytime-valid normal-mixture confidence sequence + e-value p, for
    /// pre-registered `Online` + `Sequential` designs (§10.3 #6); valid under
    /// continuous peeking and any data-dependent stopping time.
    SequentialEValue,
}

impl From<beater_stats::TestKind> for StatisticalTest {
    fn from(kind: beater_stats::TestKind) -> Self {
        match kind {
            beater_stats::TestKind::PairedT => StatisticalTest::PairedT,
            beater_stats::TestKind::McnemarExact => StatisticalTest::McnemarExact,
            beater_stats::TestKind::ClusteredPairedT => StatisticalTest::ClusteredPairedT,
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

/// Validate the shared paired-comparison preconditions (aligned lengths, the
/// policy sample floor, finite scores, a usable alpha, a non-zero comparison
/// family) and return the family-adjusted per-comparison alpha.
///
/// Single-step Bonferroni across the comparison family: no lower clamp — a
/// large `comparison_count` must genuinely shrink alpha; clamping it up would
/// let the family-wise error rate exceed the requested level (`beater-stats`
/// re-validates the result is a usable alpha in `(0, 1)`).
fn validated_adjusted_alpha(
    baseline: &[f64],
    candidate: &[f64],
    policy: &GatePolicy,
) -> Result<f64, EvalError> {
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
    // single-case branch and silently produce a Pass on NaN.
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
    if policy.comparison_count == 0 {
        return Err(EvalError::Statistics(
            "comparison_count must be greater than zero".to_string(),
        ));
    }
    if n == 0 {
        return Err(EvalError::Statistics("no scores to compare".to_string()));
    }
    Ok(policy.alpha / policy.comparison_count as f64)
}

/// The deterministic single-case smoke-gate regime (`n < 2`, a caller opts in
/// by setting `min_sample_size = 1`): a single paired observation has no
/// sampling variability, so a real variance-based test is undefined —
/// `beater-stats` correctly refuses `n < 2`. The interval collapses to the
/// point estimate and the p-value is 1.0 (one sample carries no power to
/// reject the null); the gate still decides from that degenerate interval
/// against the regression bound, preserving deterministic single-case behavior.
fn single_case_smoke_comparison(
    baseline: &[f64],
    candidate: &[f64],
    policy: &GatePolicy,
    adjusted_alpha: f64,
) -> ExperimentComparison {
    let delta =
        candidate.first().copied().unwrap_or(0.0) - baseline.first().copied().unwrap_or(0.0);
    let decision = if delta < -policy.max_regression {
        GateDecision::FailRegression
    } else {
        GateDecision::Pass
    };
    ExperimentComparison {
        sample_size: baseline.len(),
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
    }
}

pub fn compare_paired_scores(
    baseline: &[f64],
    candidate: &[f64],
    policy: &GatePolicy,
) -> Result<ExperimentComparison, EvalError> {
    let adjusted_alpha = validated_adjusted_alpha(baseline, candidate, policy)?;
    let n = baseline.len();
    if n < 2 {
        return Ok(single_case_smoke_comparison(
            baseline,
            candidate,
            policy,
            adjusted_alpha,
        ));
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

/// The pre-registration manifest type, re-exported so gate callers can build and
/// pass a design without taking a direct `beater-design` dependency.
pub use beater_design::EvalDesign;

/// The multiple-comparison policy (§10.3 #4), re-exported so gate/round callers
/// can select a family-wise (Holm) or false-discovery (Benjamini-Hochberg)
/// correction across a candidate family without a direct `beater-design`
/// dependency.
pub use beater_design::MultiplicityPolicy;

/// The variance-reduction policy (§10.3 #4), re-exported so gate/round callers can
/// pre-register CUPED (with its covariate name) without a direct `beater-design`
/// dependency.
pub use beater_design::VarianceReduction;

/// Build a conservative [`EvalDesign`] for a deploy gate from its [`GatePolicy`]
/// and the realised sample size. This is the design the experiment/gate path
/// pre-registers when the caller has not supplied one of its own: a single-metric,
/// fixed-horizon, offline regression-guard on the untouched `Test` split, with no
/// multiplicity family and case-level (independent) units. It is intentionally the
/// *safe* default — it always satisfies [`EvalDesign::permit_pass`], so wiring it
/// in changes no existing gate decision, while making the design contract
/// load-bearing for any caller that later supplies a richer (and possibly
/// refusing) design.
pub fn conservative_gate_design(policy: &GatePolicy, sample_size: usize) -> EvalDesign {
    use beater_design::{
        DatasetSplit, Estimand, MetricDirection, MetricSpec, MetricType, Monitoring,
        MultiplicityPolicy, RepetitionPlan, SamplingDesign, StoppingRule, TestSelectionPolicy,
        UnitOfAnalysis, WeightingPolicy, CURRENT_ANALYSIS_VERSION, DEFAULT_POWER,
    };
    EvalDesign {
        name: "experiment-gate".to_string(),
        hypothesis: "candidate does not regress beyond the policy bound".to_string(),
        estimand: Estimand::RegressionGuard {
            max_regression: policy.max_regression.max(0.0),
        },
        primary_metric: MetricSpec::new(
            "primary",
            MetricType::Bounded,
            MetricDirection::HigherIsBetter,
        ),
        secondary_metrics: Vec::new(),
        unit_of_analysis: UnitOfAnalysis::Case,
        cluster_key: None,
        test_selection: TestSelectionPolicy::Auto,
        multiplicity: MultiplicityPolicy::None,
        sampling: SamplingDesign::Random,
        weighting: WeightingPolicy::Unweighted,
        monitoring: Monitoring::Offline,
        stopping_rule: StoppingRule::FixedHorizon {
            planned_n: sample_size.max(1),
        },
        repetition: RepetitionPlan::default(),
        alpha: policy.alpha,
        power_target: DEFAULT_POWER,
        mde: None,
        split: DatasetSplit::Test,
        gate_may_read_split: true,
        analysis_version: CURRENT_ANALYSIS_VERSION,
        variance_reduction: beater_design::VarianceReduction::None,
    }
}

/// Design-aware deploy gate: execute the pre-registered [`EvalDesign`] and
/// enforce it before a `Pass` can be certified (§1 #9, §10.3).
///
/// The executed test is **dispatched from the design** (see
/// [`compare_paired_scores_designed`]): `test_selection` picks the §10.3 #3
/// method (with `Auto` resolving by metric family + satisfied assumptions via
/// `beater_stats::recommend_paired_test`), an `Online + Sequential` design runs
/// the anytime-valid confidence-sequence path, and a design that pre-registers
/// a `cluster_key` can only certify a `Pass` when per-case cluster labels were
/// actually supplied (this entry point supplies none, so such designs are
/// downgraded — use [`compare_paired_scores_designed`] with cluster ids).
///
/// A design that is structurally malformed ([`EvalDesign::validate`]) or
/// *incapable of a valid decision* ([`EvalDesign::permit_pass`]) can **never**
/// yield `Pass`: the decision is forced to `Inconclusive`. A regression failure
/// is preserved (an invalid design is no reason to ship).
pub fn compare_paired_scores_with_design(
    baseline: &[f64],
    candidate: &[f64],
    policy: &GatePolicy,
    design: &EvalDesign,
) -> Result<ExperimentComparison, EvalError> {
    compare_paired_scores_designed(
        &DesignedComparisonInputs {
            baseline,
            candidate,
            covariate: None,
            cluster_ids: None,
        },
        policy,
        design,
    )
}

/// Design-aware deploy gate with optional CUPED variance reduction (§10.3 #4 /
/// #436 item 4).
///
/// When the pre-registered [`EvalDesign`] declares
/// [`VarianceReduction::Cuped`], a per-case `covariate` is supplied for **every**
/// Test case, and there are at least 3 paired observations, the paired difference
/// is CUPED-adjusted (the covariate is regressed out, centered on the design's
/// **known** population mean μ_x) before the t-test. The one-sample regression
/// estimator μ̂ = mean(d) − θ(x̄ − μ_x) both **moves** the point estimate off the
/// raw delta (centering on the sample mean instead would be degenerate — see
/// [`beater_stats::cuped_paired_t_test`]) and tightens the confidence interval;
/// together that is what lets a borderline round clear the regression bound on the
/// same data. The recorded `test` stays [`StatisticalTest::PairedT`] — a paired
/// t-test on CUPED-adjusted differences is still a paired t-test — and that CUPED
/// was applied is recorded in the pre-registered [`EvalDesign`] (the source of
/// truth, part of the hashed commitment), never the wire result, so the `/v1`
/// contract is unchanged.
///
/// In every other case — no CUPED policy, an absent/partial covariate, or `n < 3`
/// — this is byte-for-byte [`compare_paired_scores_with_design`], so declaring a
/// covariate can only ever *add* power, never change a decision it wasn't wired
/// for. The pre-registration manifest is enforced exactly as the plain design
/// path does: a structurally-invalid or non-permitting design can never certify a
/// `Pass`.
///
/// The covariate MUST be a genuine pre-experiment quantity independent of the
/// candidate under test (never an arm's own score) — see
/// [`beater_stats::cuped_adjust`]. That provenance is the caller's responsibility.
pub fn compare_paired_scores_cuped(
    baseline: &[f64],
    candidate: &[f64],
    covariate: Option<&[f64]>,
    policy: &GatePolicy,
    design: &EvalDesign,
) -> Result<ExperimentComparison, EvalError> {
    compare_paired_scores_designed(
        &DesignedComparisonInputs {
            baseline,
            candidate,
            covariate,
            cluster_ids: None,
        },
        policy,
        design,
    )
}

/// Per-case inputs for the fully design-honoring comparison
/// ([`compare_paired_scores_designed`]).
#[derive(Clone, Copy, Debug)]
pub struct DesignedComparisonInputs<'a> {
    /// Baseline per-case scores.
    pub baseline: &'a [f64],
    /// Candidate per-case scores, paired one-to-one with `baseline`.
    pub candidate: &'a [f64],
    /// The pre-registered CUPED covariate, aligned per case (see
    /// [`compare_paired_scores_cuped`] for its validity requirements).
    pub covariate: Option<&'a [f64]>,
    /// Per-case cluster labels resolving the design's `cluster_key`. Required
    /// (aligned one-to-one) for a design that pre-registers clustering to be
    /// able to certify a `Pass`. Build this with [`resolve_cluster_ids`] rather
    /// than by hand so an unresolvable key is surfaced instead of silently
    /// dropped.
    pub cluster_ids: Option<&'a [String]>,
}

/// The per-case identifiers a caller can offer so a pre-registered
/// [`beater_design::ClusterKey`] can be resolved into concrete cluster labels
/// for [`compare_paired_scores_designed`].
///
/// A caller fills in whichever identifiers it has for a case; a `DatasetCase`,
/// for example, supplies `trace_id` from its `source_trace_id`. Keeping this a
/// caller-provided bag of strings (rather than a concrete case type) is what
/// lets `beater-eval` resolve clustering without depending on `beater-datasets`
/// or the trace store — the resolution is pure and the layering stays clean.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CaseClusterIds<'a> {
    /// The case's originating trace, for [`beater_design::ClusterKey::Trace`].
    pub trace_id: Option<&'a str>,
    /// The multi-turn conversation the case belongs to, for
    /// [`beater_design::ClusterKey::Conversation`].
    pub conversation_id: Option<&'a str>,
    /// The prompt template the case was generated from, for
    /// [`beater_design::ClusterKey::PromptTemplate`].
    pub prompt_template_id: Option<&'a str>,
    /// The shared-seed group, for [`beater_design::ClusterKey::SeedGroup`].
    pub seed_group_id: Option<&'a str>,
    /// The value of the named custom attribute, for
    /// [`beater_design::ClusterKey::Custom`] — the caller looks the value up by
    /// the key's `name` and supplies it here.
    pub custom: Option<&'a str>,
}

/// The outcome of resolving a design's `cluster_key` against the per-case
/// identifiers a caller could offer (see [`resolve_cluster_ids`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClusterResolution {
    /// The design does not pre-register clustering (`cluster_key == None`); no
    /// cluster labels are needed and the offline i.i.d. path is correct.
    NotClustered,
    /// The `cluster_key` resolved to a per-case label for every case.
    Resolved(Vec<String>),
    /// The `cluster_key` is set but the per-case identifier it names was absent
    /// for at least one case, so clustered execution cannot be honored — the
    /// executor will refuse to certify a `Pass` rather than fall back to
    /// too-small i.i.d. standard errors (§10.3 #1).
    Unresolvable(beater_design::ClusterKey),
}

impl ClusterResolution {
    /// The per-case labels to hand [`DesignedComparisonInputs::cluster_ids`]:
    /// `Some` only when fully [`Resolved`](ClusterResolution::Resolved).
    /// `NotClustered` and `Unresolvable` both map to `None`, which the executor
    /// already handles correctly (run offline when unclustered; refuse a `Pass`
    /// when a set key could not be honored).
    pub fn labels(&self) -> Option<&[String]> {
        match self {
            ClusterResolution::Resolved(labels) => Some(labels),
            ClusterResolution::NotClustered | ClusterResolution::Unresolvable(_) => None,
        }
    }
}

/// Resolve a pre-registered [`EvalDesign::cluster_key`] into the per-case
/// cluster labels [`compare_paired_scores_designed`] needs, from the
/// identifiers a caller can offer.
///
/// This is the bridge between the declarative design manifest and the executor:
/// `beater-design` records *which* key clusters the observations, and this
/// function turns that key plus a caller's per-case identifiers into the actual
/// labels — or reports that the key cannot be honored, so a clustered design is
/// safely refused instead of analyzed with i.i.d. standard errors.
///
/// * [`ClusterKey::Case`](beater_design::ClusterKey::Case) always resolves:
///   each case is its own cluster (labelled by index), which the CR1 SE reduces
///   to the i.i.d. SE up to the finite-cluster correction.
/// * `Trace`/`Conversation`/`PromptTemplate`/`SeedGroup`/`Custom` resolve only
///   when the corresponding [`CaseClusterIds`] field is present for **every**
///   case; a single gap makes the whole design unresolvable (all-or-nothing, so
///   clusters are never silently split).
///
/// Pass one [`CaseClusterIds`] per score pair. The per-case-vs-score length is
/// re-checked by [`compare_paired_scores_designed`] (a mismatch drops the
/// labels and refuses a `Pass`), so this function is a pure, total mapping.
pub fn resolve_cluster_ids(
    design: &EvalDesign,
    per_case: &[CaseClusterIds<'_>],
) -> ClusterResolution {
    use beater_design::ClusterKey;

    let Some(key) = design.cluster_key.clone() else {
        return ClusterResolution::NotClustered;
    };

    // `ClusterKey::Case` needs no external identifier — each case is its own
    // cluster, labelled by position.
    if let ClusterKey::Case = key {
        let labels = (0..per_case.len()).map(|i| i.to_string()).collect();
        return ClusterResolution::Resolved(labels);
    }

    // Every other key selects one per-case identifier; it must be present for
    // all cases or the design cannot be honored.
    let mut labels = Vec::with_capacity(per_case.len());
    for ids in per_case {
        let value = match &key {
            ClusterKey::Case => unreachable!("handled above"),
            ClusterKey::Trace => ids.trace_id,
            ClusterKey::Conversation => ids.conversation_id,
            ClusterKey::PromptTemplate => ids.prompt_template_id,
            ClusterKey::SeedGroup => ids.seed_group_id,
            ClusterKey::Custom { .. } => ids.custom,
        };
        match value {
            Some(label) => labels.push(label.to_owned()),
            None => return ClusterResolution::Unresolvable(key),
        }
    }
    ClusterResolution::Resolved(labels)
}

/// Deterministic seed for the gate's paired-bootstrap path: gate decisions must
/// be reproducible from the raw scores alone, so the resampling stream is fixed
/// rather than drawn from ambient entropy.
const GATE_BOOTSTRAP_SEED: u64 = 0xBEA7_E12A_11D5_EED5;

/// Bootstrap resamples for the gate's paired-bootstrap path (§10.3 default).
const GATE_BOOTSTRAP_RESAMPLES: usize = 10_000;

/// The fully design-honoring paired comparison (roadmap PR-A4): executes the
/// §10.3 method the pre-registered [`EvalDesign`] committed to, instead of
/// unconditionally running a paired-t/McNemar.
///
/// Dispatch, in priority order:
///
/// 1. **Sequential** — an `Online` design with a `Sequential` stopping rule
///    runs the anytime-valid normal-mixture confidence sequence
///    (`beater_stats::confidence_sequence_mean`), valid under continuous
///    peeking; requires a `[0, 1]`-bounded metric family
///    (`Proportion`/`Bounded`/`ProperScore`, so the paired difference is
///    1-sub-Gaussian) and no cluster key. Otherwise the comparison falls back
///    to the offline method and a `Pass` is downgraded to `Inconclusive` —
///    fixed-horizon inference on a peeked stream cannot certify a win.
/// 2. **Clustered** — a design with a `cluster_key` runs the cluster-robust
///    paired t (`beater_stats::clustered_paired_t_test`, CR1 SE, `G − 1` df)
///    over the supplied `cluster_ids`. If the labels are missing or misaligned
///    the i.i.d. result is computed but a `Pass` is downgraded: i.i.d. standard
///    errors on clustered data are exactly the inflated-false-win hazard the
///    design pre-registered against (§10.3 #1). CUPED is not combined with
///    clustering (the θ fit assumes independent pairs); the covariate is
///    ignored on this path.
/// 3. **Offline test selection** — `test_selection` picks the method;
///    [`TestSelectionPolicy::Auto`] resolves it from the data via
///    `beater_stats::recommend_paired_test` (binary → exact McNemar;
///    small N → paired bootstrap; ~normal differences → paired t; otherwise
///    Wilcoxon signed-rank). A pre-registered CUPED policy applies on the
///    t-family path only (the regression estimator is a t-family method); the
///    Wilcoxon path reports the Hodges-Lehmann pseudo-median and its
///    distribution-free CI (falling back to the paired bootstrap in the rare
///    small-N/tight-alpha corner where that CI is undefined). An explicitly
///    pinned `McnemarExact` on non-binary scores is a design/executor mismatch
///    and errors loudly rather than silently running a different test.
///
/// The decision rule is identical on every path: the (method-appropriate) CI is
/// tested against the negated regression bound, `Inconclusive` verdicts carry
/// the §10.3 #5 power annotations, and a design that fails
/// [`EvalDesign::validate`]/[`EvalDesign::permit_pass`] — or whose pre-registered
/// requirements this call could not honor — can never certify a `Pass`, while
/// regression failures are always preserved.
pub fn compare_paired_scores_designed(
    inputs: &DesignedComparisonInputs<'_>,
    policy: &GatePolicy,
    design: &EvalDesign,
) -> Result<ExperimentComparison, EvalError> {
    use beater_design::{Monitoring, StoppingRule};

    let (baseline, candidate) = (inputs.baseline, inputs.candidate);
    // Shared validation + the degenerate single-case smoke regime, exactly as
    // the plain path, so every dispatch below can assume clean, aligned,
    // finite inputs.
    let adjusted_alpha = validated_adjusted_alpha(baseline, candidate, policy)?;
    let n = baseline.len();
    if n < 2 {
        let smoke = single_case_smoke_comparison(baseline, candidate, policy, adjusted_alpha);
        return Ok(enforce_design(smoke, design, true));
    }

    let differences: Vec<f64> = candidate
        .iter()
        .zip(baseline.iter())
        .map(|(c, b)| c - b)
        .collect();

    let is_sequential_design = design.monitoring == Monitoring::Online
        && matches!(design.stopping_rule, StoppingRule::Sequential { .. });
    let wants_cluster = design.cluster_key.is_some();
    let cluster_ids = inputs
        .cluster_ids
        .filter(|ids| ids.len() == n)
        .filter(|_| wants_cluster);

    // ── 1. Anytime-valid sequential path (§10.3 #6) ──────────────────────────
    if is_sequential_design {
        let bounded_metric = matches!(
            design.primary_metric.metric_type,
            beater_design::MetricType::Proportion
                | beater_design::MetricType::Bounded
                | beater_design::MetricType::ProperScore
        );
        if bounded_metric && !wants_cluster {
            // Scores in [0, 1] ⇒ paired differences in [−1, 1] ⇒ 1-sub-Gaussian.
            let sigma = 1.0;
            // The mixture scale comes from the pre-registered effect size (the
            // same scale the LR-optimal fixed bet uses); any fixed choice is
            // valid, this one makes the sequence tightest near the planned
            // effect. Default to 1.0 when the design pins no usable scale.
            let effect = design.mde.or(pre_registered_effect_scale(design));
            let tau = effect.filter(|e| e.is_finite() && *e > 0.0).unwrap_or(1.0);
            let outcome = beater_stats::confidence_sequence_mean(
                &differences,
                0.0,
                sigma,
                tau,
                adjusted_alpha,
            )
            .map_err(|err| EvalError::Statistics(err.to_string()))?;
            let comparison = assemble_comparison(
                baseline,
                candidate,
                outcome.estimate,
                outcome.ci,
                outcome.p_value,
                StatisticalTest::SequentialEValue,
                policy,
                adjusted_alpha,
            );
            return Ok(enforce_design(comparison, design, true));
        }
        // Sequential was pre-registered but cannot be honored (unbounded metric
        // family, or clustering — the e-process assumes independent bounded
        // increments): compute the best offline answer and refuse to certify it.
        let comparison = offline_dispatch(inputs, &differences, policy, design, adjusted_alpha)?;
        return Ok(enforce_design(comparison, design, false));
    }

    // ── 2. Cluster-robust path (§10.3 #1) ────────────────────────────────────
    if wants_cluster {
        if let Some(ids) = cluster_ids {
            let outcome = beater_stats::clustered_paired_t_test(&differences, ids, adjusted_alpha)
                .map_err(|err| EvalError::Statistics(err.to_string()))?;
            let ci = outcome.ci.unwrap_or(beater_stats::ConfidenceInterval {
                low: outcome.estimate,
                high: outcome.estimate,
                confidence: 1.0 - adjusted_alpha,
            });
            let comparison = assemble_comparison(
                baseline,
                candidate,
                outcome.estimate,
                ci,
                outcome.p_value,
                StatisticalTest::ClusteredPairedT,
                policy,
                adjusted_alpha,
            );
            return Ok(enforce_design(comparison, design, true));
        }
        // The design pre-registered clustering but no aligned labels were
        // supplied: i.i.d. SEs would be exactly the too-small-SE hazard the
        // manifest exists to prevent, so the offline result cannot certify Pass.
        let comparison = offline_dispatch(inputs, &differences, policy, design, adjusted_alpha)?;
        return Ok(enforce_design(comparison, design, false));
    }

    // ── 3. Offline dispatch by pre-registered test selection ─────────────────
    let comparison = offline_dispatch(inputs, &differences, policy, design, adjusted_alpha)?;
    Ok(enforce_design(comparison, design, true))
}

/// The effect scale the design pre-registered on its estimand, if positive.
fn pre_registered_effect_scale(design: &EvalDesign) -> Option<f64> {
    use beater_design::Estimand;
    match design.estimand {
        Estimand::Superiority { mde } => Some(mde),
        Estimand::NonInferiority { margin } => Some(margin),
        Estimand::RegressionGuard { max_regression } if max_regression > 0.0 => {
            Some(max_regression)
        }
        _ => None,
    }
}

/// Offline fixed-horizon dispatch on the design's pre-registered
/// `test_selection` (with `Auto` resolved from the data), including the CUPED
/// policy on the t-family path.
fn offline_dispatch(
    inputs: &DesignedComparisonInputs<'_>,
    differences: &[f64],
    policy: &GatePolicy,
    design: &EvalDesign,
    adjusted_alpha: f64,
) -> Result<ExperimentComparison, EvalError> {
    use beater_design::TestSelectionPolicy;
    use beater_stats::PairedTestChoice;

    let (baseline, candidate) = (inputs.baseline, inputs.candidate);
    let choice = match design.test_selection {
        TestSelectionPolicy::Auto => beater_stats::recommend_paired_test(baseline, candidate)
            .map_err(|err| EvalError::Statistics(err.to_string()))?,
        TestSelectionPolicy::PairedT => PairedTestChoice::PairedT,
        TestSelectionPolicy::McnemarExact => {
            let binary = baseline
                .iter()
                .chain(candidate.iter())
                .all(|v| *v == 0.0 || *v == 1.0);
            if !binary {
                return Err(EvalError::Statistics(
                    "design pre-registered exact McNemar but the scores are not a paired \
                     binary outcome"
                        .to_string(),
                ));
            }
            PairedTestChoice::McnemarExact
        }
        TestSelectionPolicy::WilcoxonSignedRank => PairedTestChoice::WilcoxonSignedRank,
        TestSelectionPolicy::PairedBootstrap => PairedTestChoice::PairedBootstrap,
    };

    match choice {
        PairedTestChoice::McnemarExact => {
            // `compare_paired` selects exact McNemar for binary data and carries
            // the score-interval CI for the paired difference in proportions.
            let outcome = beater_stats::compare_paired(baseline, candidate, adjusted_alpha)
                .map_err(|err| EvalError::Statistics(err.to_string()))?;
            let ci = outcome.ci.unwrap_or(beater_stats::ConfidenceInterval {
                low: outcome.estimate,
                high: outcome.estimate,
                confidence: 1.0 - adjusted_alpha,
            });
            Ok(assemble_comparison(
                baseline,
                candidate,
                outcome.estimate,
                ci,
                outcome.p_value,
                outcome.test.into(),
                policy,
                adjusted_alpha,
            ))
        }
        PairedTestChoice::PairedT => {
            // The CUPED regression estimator is a t-family method, so the
            // pre-registered policy applies exactly on this branch (n ≥ 3, a
            // fully aligned covariate, and a declared population mean).
            if let VarianceReduction::Cuped {
                population_mean, ..
            } = &design.variance_reduction
            {
                if let Some(covariate) = inputs
                    .covariate
                    .filter(|c| c.len() == baseline.len() && baseline.len() >= 3)
                {
                    let outcome = beater_stats::cuped_paired_t_test(
                        differences,
                        covariate,
                        *population_mean,
                        adjusted_alpha,
                    )
                    .map_err(|err| EvalError::Statistics(err.to_string()))?;
                    let ci = outcome.ci.unwrap_or(beater_stats::ConfidenceInterval {
                        low: outcome.estimate,
                        high: outcome.estimate,
                        confidence: 1.0 - adjusted_alpha,
                    });
                    return Ok(assemble_comparison(
                        baseline,
                        candidate,
                        outcome.estimate,
                        ci,
                        outcome.p_value,
                        // A paired t on CUPED-adjusted differences is still a
                        // paired t; the CUPED provenance lives in the hashed
                        // design commitment.
                        StatisticalTest::PairedT,
                        policy,
                        adjusted_alpha,
                    ));
                }
            }
            let outcome = beater_stats::paired_t_test(differences, adjusted_alpha)
                .map_err(|err| EvalError::Statistics(err.to_string()))?;
            let ci = outcome.ci.unwrap_or(beater_stats::ConfidenceInterval {
                low: outcome.estimate,
                high: outcome.estimate,
                confidence: 1.0 - adjusted_alpha,
            });
            Ok(assemble_comparison(
                baseline,
                candidate,
                outcome.estimate,
                ci,
                outcome.p_value,
                StatisticalTest::PairedT,
                policy,
                adjusted_alpha,
            ))
        }
        PairedTestChoice::WilcoxonSignedRank => {
            let outcome = beater_stats::wilcoxon_signed_rank(differences, adjusted_alpha)
                .map_err(|err| EvalError::Statistics(err.to_string()))?;
            match outcome.ci {
                Some(ci) => Ok(assemble_comparison(
                    baseline,
                    candidate,
                    outcome.estimate,
                    ci,
                    outcome.p_value,
                    StatisticalTest::WilcoxonSignedRank,
                    policy,
                    adjusted_alpha,
                )),
                // Too few pairs for a distribution-free CI at this alpha: the
                // assumption-light paired bootstrap still yields a decidable
                // interval.
                None => paired_bootstrap_comparison(
                    baseline,
                    candidate,
                    differences,
                    policy,
                    adjusted_alpha,
                ),
            }
        }
        PairedTestChoice::PairedBootstrap => {
            paired_bootstrap_comparison(baseline, candidate, differences, policy, adjusted_alpha)
        }
    }
}

/// The paired-bootstrap comparison with the gate's fixed, documented seed.
fn paired_bootstrap_comparison(
    baseline: &[f64],
    candidate: &[f64],
    differences: &[f64],
    policy: &GatePolicy,
    adjusted_alpha: f64,
) -> Result<ExperimentComparison, EvalError> {
    let outcome = beater_stats::paired_bootstrap_test(
        differences,
        adjusted_alpha,
        GATE_BOOTSTRAP_RESAMPLES,
        GATE_BOOTSTRAP_SEED,
    )
    .map_err(|err| EvalError::Statistics(err.to_string()))?;
    Ok(assemble_comparison(
        baseline,
        candidate,
        outcome.estimate,
        outcome.ci,
        outcome.p_value,
        StatisticalTest::PairedBootstrap,
        policy,
        adjusted_alpha,
    ))
}

/// Assemble an [`ExperimentComparison`] from a method outcome: apply the shared
/// CI-vs-regression-bound decision rule and the §10.3 #5 power annotations.
#[allow(clippy::too_many_arguments)]
fn assemble_comparison(
    baseline: &[f64],
    candidate: &[f64],
    estimate: f64,
    ci: beater_stats::ConfidenceInterval,
    p_value: f64,
    test: StatisticalTest,
    policy: &GatePolicy,
    adjusted_alpha: f64,
) -> ExperimentComparison {
    let decision = if ci.high < -policy.max_regression {
        GateDecision::FailRegression
    } else if ci.low >= -policy.max_regression {
        GateDecision::Pass
    } else {
        GateDecision::Inconclusive
    };
    let (mde, required_n) = if decision == GateDecision::Inconclusive {
        power_annotations(
            baseline,
            candidate,
            baseline.len(),
            estimate,
            adjusted_alpha,
        )
    } else {
        (None, None)
    };
    ExperimentComparison {
        sample_size: baseline.len(),
        baseline_mean: mean(baseline),
        candidate_mean: mean(candidate),
        delta: estimate,
        ci_low: ci.low,
        ci_high: ci.high,
        p_value,
        decision,
        test,
        adjusted_alpha,
        mde,
        required_n,
    }
}

/// Enforce the pre-registration manifest on a computed comparison: a design
/// that fails validation/permit-pass — or whose pre-registered requirements the
/// executor could not honor (`requirements_honored == false`) — can never
/// certify a `Pass`. Regression failures are preserved.
fn enforce_design(
    mut comparison: ExperimentComparison,
    design: &EvalDesign,
    requirements_honored: bool,
) -> ExperimentComparison {
    let design_ok =
        design.validate().is_ok() && design.permit_pass().is_ok() && requirements_honored;
    if !design_ok && comparison.decision == GateDecision::Pass {
        comparison.decision = GateDecision::Inconclusive;
        comparison.mde = None;
        comparison.required_n = None;
    }
    comparison
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
/// Delegates to `beater-stats` so the SD used to standardize effects here is
/// the same SD the power/MDE formulas there assume.
fn std_dev(values: &[f64]) -> f64 {
    beater_stats::sample_std_dev(values)
}

fn mean(values: &[f64]) -> f64 {
    beater_stats::sample_mean(values)
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
        assert_eq!(catalog.len(), 18);

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
    fn cost_budget_rejects_negative_trace_metric() {
        let case = EvaluationCase {
            input: serde_json::json!(null),
            output: serde_json::json!(null),
            reference: None,
            trace: Some(serde_json::json!({ "cost_micros": -1 })),
        };

        assert!(matches!(
            evaluate_deterministic(
                &deterministic_spec(EvaluatorKind::CostBudget { max_micros: 100 }),
                &case,
            ),
            Err(EvalError::MissingTraceMetric { evaluator_id, metric })
                if evaluator_id == "cost_budget" && metric == "cost_micros"
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

        let negative_budget = evaluate_deterministic(
            &deterministic_spec(EvaluatorKind::CostBudget { max_micros: -1 }),
            &case,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(negative_budget.score, 0.0);

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

    // ── design-aware gate: EvalDesign::permit_pass is load-bearing ────────────

    #[test]
    fn conservative_gate_design_always_permits_pass() {
        // The derived default design must never itself block a Pass, so wiring it
        // into the gate path changes no existing decision.
        let design = conservative_gate_design(&GatePolicy::default(), 20);
        assert_eq!(design.validate(), Ok(()));
        assert_eq!(design.permit_pass(), Ok(()));
    }

    #[test]
    fn valid_design_preserves_pass() {
        let baseline = [0.50, 0.55, 0.48, 0.52, 0.51, 0.49, 0.53, 0.47, 0.50, 0.52];
        let candidate = [0.60, 0.64, 0.59, 0.62, 0.61, 0.58, 0.63, 0.57, 0.60, 0.62];
        let policy = GatePolicy::default();
        let plain = compare_paired_scores(&baseline, &candidate, &policy)
            .unwrap_or_else(|err| panic!("{err}"));
        let design = conservative_gate_design(&policy, baseline.len());
        let gated = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(plain.decision, GateDecision::Pass);
        assert_eq!(
            gated.decision, plain.decision,
            "valid design preserves Pass"
        );
    }

    #[test]
    fn refusing_design_forces_inconclusive_despite_clear_win() {
        // A clearly-passing comparison, but the design is incapable of a valid
        // decision (online stream with a fixed-horizon test). The gate must refuse
        // to certify a Pass — inconclusive, never pass (§10.3 #6).
        use beater_design::{Monitoring, StoppingRule};
        let baseline = [0.50, 0.55, 0.48, 0.52, 0.51, 0.49, 0.53, 0.47, 0.50, 0.52];
        let candidate = [0.60, 0.64, 0.59, 0.62, 0.61, 0.58, 0.63, 0.57, 0.60, 0.62];
        let policy = GatePolicy::default();
        let mut design = conservative_gate_design(&policy, baseline.len());
        design.monitoring = Monitoring::Online; // peeked stream + FixedHorizon → refusal
        assert!(matches!(
            design.stopping_rule,
            StoppingRule::FixedHorizon { .. }
        ));
        assert!(design.permit_pass().is_err());
        let gated = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(gated.decision, GateDecision::Inconclusive);
    }

    #[test]
    fn refusing_design_preserves_regression_failure() {
        // An invalid design does not whitewash a regression into a pass — a clear
        // regression still fails.
        use beater_design::Monitoring;
        let baseline = [0.90, 0.92, 0.88, 0.91, 0.89, 0.93, 0.90, 0.92, 0.91, 0.89];
        let candidate = [0.50, 0.52, 0.48, 0.51, 0.49, 0.53, 0.50, 0.52, 0.51, 0.49];
        let policy = GatePolicy::default();
        let mut design = conservative_gate_design(&policy, baseline.len());
        design.monitoring = Monitoring::Online;
        let gated = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(gated.decision, GateDecision::FailRegression);
    }

    // A fixture where the paired difference is a fixed +0.05 lift buried under
    // noise that a pre-experiment covariate almost fully explains. `baseline` is
    // constant, so the covariate is independent of it (a valid difficulty proxy).
    // Plain gate: the noise widens the CI past 0 → Inconclusive. CUPED regresses
    // the covariate out → the CI tightens around +0.05 → Pass, same delta.
    fn cuped_fixture() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let covariate = vec![0.0, 1.0, 0.1, 0.9, 0.2, 0.8, 0.3, 0.7, 0.4, 0.6, 0.45, 0.55];
        let baseline = vec![0.5; 12];
        let candidate = vec![
            0.405, 0.695, 0.435, 0.665, 0.465, 0.635, 0.495, 0.605, 0.525, 0.575, 0.54, 0.56,
        ];
        (baseline, candidate, covariate)
    }

    fn cuped_design(policy: &GatePolicy, n: usize, population_mean: f64) -> EvalDesign {
        let mut design = conservative_gate_design(policy, n);
        design.variance_reduction = VarianceReduction::Cuped {
            covariate: "prior_difficulty".to_string(),
            population_mean,
        };
        design
    }

    #[test]
    fn cuped_resolves_an_inconclusive_gate_via_the_regression_estimator() {
        let (baseline, candidate, covariate) = cuped_fixture();
        let policy = GatePolicy::default();

        // Without the covariate the noise leaves the CI straddling the bound.
        let plain = compare_paired_scores(&baseline, &candidate, &policy)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            plain.decision,
            GateDecision::Inconclusive,
            "plain gate should be underpowered against the noise"
        );

        // The covariate's known population mean (0.7) is above this Test sample's
        // mean (0.5); since the difference rises with the covariate, the regression
        // estimator corrects the lift UPWARD, and the variance-reduced CI clears the
        // bound → Pass.
        let mu_x = 0.7;
        let design = cuped_design(&policy, baseline.len(), mu_x);
        let gated =
            compare_paired_scores_cuped(&baseline, &candidate, Some(&covariate), &policy, &design)
                .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            gated.decision,
            GateDecision::Pass,
            "CUPED should resolve it"
        );
        // Still reported as a paired t-test (CUPED is recorded in the design, not
        // the wire result, keeping the /v1 contract unchanged).
        assert_eq!(gated.test, StatisticalTest::PairedT);

        // The reported delta is the regression estimator μ̂ = mean(d) − θ(x̄ − μ_x),
        // computed independently here — it MOVES off the plain mean (that movement
        // is exactly what makes the variance-reduced CI valid).
        let differences: Vec<f64> = candidate
            .iter()
            .zip(baseline.iter())
            .map(|(c, b)| c - b)
            .collect();
        let adj = beater_stats::cuped_adjust(&differences, &covariate)
            .unwrap_or_else(|err| panic!("{err}"));
        let expected = plain.delta - adj.theta * (adj.covariate_mean - mu_x);
        assert!(
            (gated.delta - expected).abs() < 1e-9,
            "delta must be the regression estimator: {} vs {expected}",
            gated.delta
        );
        assert!(gated.delta > plain.delta, "estimate corrected upward");
        assert!(
            gated.ci_high - gated.ci_low < plain.ci_high - plain.ci_low,
            "CUPED CI must be narrower"
        );
    }

    #[test]
    fn cuped_gate_is_identical_to_plain_when_policy_is_none() {
        let (baseline, candidate, covariate) = cuped_fixture();
        let policy = GatePolicy::default();
        // conservative_gate_design carries VarianceReduction::None.
        let design = conservative_gate_design(&policy, baseline.len());
        let plain = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        // Even with a covariate on hand, a None policy takes the unadjusted path.
        let via_cuped =
            compare_paired_scores_cuped(&baseline, &candidate, Some(&covariate), &policy, &design)
                .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(plain, via_cuped, "None policy must be byte-identical");
    }

    #[test]
    fn cuped_gate_falls_back_when_covariate_absent_or_too_short() {
        let (baseline, candidate, covariate) = cuped_fixture();
        let policy = GatePolicy::default();
        let design = cuped_design(&policy, baseline.len(), 0.5);
        let plain = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        // No covariate supplied → unadjusted path.
        let none = compare_paired_scores_cuped(&baseline, &candidate, None, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(none, plain);
        // Misaligned covariate → unadjusted path (never silently truncates pairs).
        let short = compare_paired_scores_cuped(
            &baseline,
            &candidate,
            Some(&covariate[..3]),
            &policy,
            &design,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(short, plain);
    }

    #[test]
    fn cuped_gate_still_refuses_a_pass_under_an_invalid_design() {
        use beater_design::Monitoring;
        let (baseline, candidate, covariate) = cuped_fixture();
        let policy = GatePolicy::default();
        let mut design = cuped_design(&policy, baseline.len(), 0.7);
        design.monitoring = Monitoring::Online; // online + fixed-horizon → refusal
        let gated =
            compare_paired_scores_cuped(&baseline, &candidate, Some(&covariate), &policy, &design)
                .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            gated.decision,
            GateDecision::Inconclusive,
            "an invalid design cannot certify a Pass even with CUPED"
        );
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

    #[test]
    fn zero_comparison_count_errors() {
        let result = compare_paired_scores(
            &[1.0; 10],
            &[1.0; 10],
            &GatePolicy {
                comparison_count: 0,
                ..GatePolicy::default()
            },
        );
        assert!(matches!(
            result,
            Err(EvalError::Statistics(message)) if message.contains("comparison_count")
        ));
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
    fn conversation_and_trajectory_scorers_are_judge_lane_and_resolvable() {
        // §20.10 #7.8 / R18.8: conversation-level and agent-trajectory named
        // scorers are catalogued as judge-lane metadata entries (reusing the
        // judge-broker mechanism), each resolvable by id via the lookup helper.
        let conversation_scorers = [
            "conversation_coherence",
            "session_completeness",
            "user_frustration",
        ];
        let trajectory_scorers = [
            "tool_selection_quality",
            "tool_error_rate",
            "action_completion",
            "agent_flow",
        ];
        for id in conversation_scorers.iter().chain(trajectory_scorers.iter()) {
            let entry = evaluator_catalog_entry(id)
                .unwrap_or_else(|| panic!("catalog entry {id} should exist"));
            assert_eq!(entry.id, *id);
            assert_eq!(
                entry.lane,
                EvaluatorLane::JudgeBroker,
                "{id} must be judge-lane"
            );
            assert!(!entry.wasm_safe, "{id} is a judge scorer, not wasm-safe");
            assert!(!entry.requires_reference, "{id} scores groups, not refs");
            assert!(entry.consumes_trace, "{id} reads the session/trajectory");
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

    // ── compare_paired_scores_designed: PR-A4 design-honoring dispatch ───────

    #[test]
    fn auto_dispatch_runs_wilcoxon_on_skewed_differences() {
        // One huge outlier difference fails the normality screen → Wilcoxon.
        let baseline = vec![0.0; 12];
        let candidate = vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 9.0];
        let policy = GatePolicy::default();
        let design = conservative_gate_design(&policy, baseline.len());
        let out = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::WilcoxonSignedRank);
        // The delta is the Hodges-Lehmann pseudo-median, robust to the outlier.
        assert!(
            out.delta < 1.0,
            "HL estimate must not be dragged to the outlier: {}",
            out.delta
        );
        assert!(out.p_value < 0.05, "consistent lift: p = {}", out.p_value);
    }

    #[test]
    fn auto_dispatch_runs_bootstrap_below_the_small_n_threshold() {
        let baseline = vec![0.50, 0.55, 0.48, 0.52, 0.51];
        let candidate = vec![0.60, 0.66, 0.57, 0.63, 0.61];
        let policy = GatePolicy {
            min_sample_size: 3,
            ..GatePolicy::default()
        };
        let design = conservative_gate_design(&policy, baseline.len());
        let out = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::PairedBootstrap);
        // Deterministic: the gate seed is fixed, so a rerun is identical.
        let again = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out, again);
    }

    #[test]
    fn pinned_wilcoxon_overrides_the_auto_recommendation() {
        // Normal-ish differences would auto-select the paired t; the design
        // pinned Wilcoxon, and the executor must honor the pre-registration.
        let baseline = vec![0.50, 0.55, 0.48, 0.52, 0.51, 0.49, 0.53, 0.47, 0.50, 0.52];
        let candidate = vec![0.60, 0.64, 0.59, 0.62, 0.61, 0.58, 0.63, 0.57, 0.60, 0.62];
        let policy = GatePolicy::default();
        let mut design = conservative_gate_design(&policy, baseline.len());
        design.test_selection = beater_design::TestSelectionPolicy::WilcoxonSignedRank;
        let out = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::WilcoxonSignedRank);
        assert_eq!(out.decision, GateDecision::Pass, "consistent lift passes");
    }

    #[test]
    fn pinned_mcnemar_on_non_binary_scores_errors_loudly() {
        let baseline = vec![0.5; 10];
        let candidate = vec![0.7; 10];
        let policy = GatePolicy::default();
        let mut design = conservative_gate_design(&policy, baseline.len());
        design.test_selection = beater_design::TestSelectionPolicy::McnemarExact;
        let result = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design);
        assert!(
            matches!(result, Err(EvalError::Statistics(_))),
            "a McNemar pin on continuous scores is a design/executor mismatch: {result:?}"
        );
    }

    #[test]
    fn cuped_on_binary_scores_keeps_the_exact_mcnemar_test() {
        // Previously a CUPED covariate silently switched a paired-binary
        // comparison to a t-test on {−1,0,1} differences. CUPED is a t-family
        // estimator: on binary data the exact McNemar test must still run and
        // the covariate is ignored.
        let baseline = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let candidate = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0];
        let covariate = vec![0.5; 10];
        let policy = GatePolicy::default();
        let design = cuped_design(&policy, baseline.len(), 0.5);
        let out =
            compare_paired_scores_cuped(&baseline, &candidate, Some(&covariate), &policy, &design)
                .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::McnemarExact);
    }

    fn clustered_design(policy: &GatePolicy, n: usize) -> EvalDesign {
        let mut design = conservative_gate_design(policy, n);
        design.unit_of_analysis = beater_design::UnitOfAnalysis::Conversation;
        design.cluster_key = Some(beater_design::ClusterKey::Conversation);
        design
    }

    #[test]
    fn clustered_design_with_ids_runs_the_cluster_robust_test() {
        let policy = GatePolicy {
            min_sample_size: 2,
            ..GatePolicy::default()
        };

        // All-positive lifts, but only TWO independent conversations: the i.i.d.
        // paired t would certify a Pass; the cluster-robust test (df = 1) knows
        // there are really just two observations.
        let baseline = vec![0.5; 6];
        let candidate = vec![0.80, 0.81, 0.79, 0.55, 0.54, 0.56];
        let clusters: Vec<String> = ["a", "a", "a", "b", "b", "b"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let design = clustered_design(&policy, baseline.len());

        let iid = compare_paired_scores(&baseline, &candidate, &policy)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            iid.decision,
            GateDecision::Pass,
            "the naive i.i.d. test would have certified this"
        );

        let out = compare_paired_scores_designed(
            &DesignedComparisonInputs {
                baseline: &baseline,
                candidate: &candidate,
                covariate: None,
                cluster_ids: Some(&clusters),
            },
            &policy,
            &design,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::ClusteredPairedT);
        assert_ne!(
            out.decision,
            GateDecision::Pass,
            "two clusters cannot certify a win: {out:?}"
        );

        // A consistent lift across many independent clusters DOES pass.
        let baseline: Vec<f64> = vec![0.5; 24];
        let candidate: Vec<f64> = (0..24)
            .map(|i| 0.7 + 0.01 * ((i % 3) as f64 - 1.0))
            .collect();
        let clusters: Vec<String> = (0..24).map(|i| format!("c{}", i / 2)).collect();
        let design = clustered_design(&policy, baseline.len());
        let out = compare_paired_scores_designed(
            &DesignedComparisonInputs {
                baseline: &baseline,
                candidate: &candidate,
                covariate: None,
                cluster_ids: Some(&clusters),
            },
            &policy,
            &design,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::ClusteredPairedT);
        assert_eq!(out.decision, GateDecision::Pass);
    }

    #[test]
    fn clustered_design_without_ids_cannot_certify_a_pass() {
        // Data that passes under i.i.d. assumptions; the design pre-registered
        // clustering, but no labels were supplied → the executor must refuse to
        // certify, not silently fall back to too-small SEs.
        let baseline = vec![0.5; 12];
        let candidate = vec![0.7; 12];
        let policy = GatePolicy::default();
        let design = clustered_design(&policy, baseline.len());
        let out = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            out.decision,
            GateDecision::Inconclusive,
            "unhonored cluster pre-registration must block Pass: {out:?}"
        );
        // A genuine regression is still reported (an unhonored design is no
        // reason to ship).
        let regressed = vec![0.2; 12];
        let out = compare_paired_scores_with_design(&baseline, &regressed, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.decision, GateDecision::FailRegression);
    }

    fn sequential_design(policy: &GatePolicy, n: usize) -> EvalDesign {
        let mut design = conservative_gate_design(policy, n);
        design.monitoring = beater_design::Monitoring::Online;
        design.stopping_rule = beater_design::StoppingRule::Sequential {
            method: beater_design::SequentialMethod::ConfidenceSequence,
        };
        design
    }

    #[test]
    fn sequential_design_runs_the_anytime_valid_path() {
        // A strong, consistent lift on a bounded metric over a peeked stream.
        let n = 400;
        let baseline = vec![0.5; n];
        let candidate = vec![0.8; n];
        let policy = GatePolicy::default();
        let design = sequential_design(&policy, n);
        let out = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::SequentialEValue);
        assert_eq!(out.decision, GateDecision::Pass, "{out:?}");
        assert!(out.p_value < 0.05, "p = {}", out.p_value);
        assert!((out.delta - 0.3).abs() < 1e-12);
        // The anytime CI is wider than a fixed-horizon one would be — that is
        // the honest price of continuous peeking.
        assert!(out.ci_low > 0.0 && out.ci_low < 0.3);
    }

    #[test]
    fn sequential_design_on_an_unbounded_metric_cannot_certify_a_pass() {
        // Latency is not [0,1]-bounded, so the sub-Gaussian scale is unknown and
        // the sequential path cannot honestly run: the offline result is
        // computed but a Pass is refused.
        let n = 400;
        let baseline = vec![0.5; n];
        let candidate = vec![0.8; n];
        let policy = GatePolicy::default();
        let mut design = sequential_design(&policy, n);
        design.primary_metric = beater_design::MetricSpec::new(
            "p95_latency_ms",
            beater_design::MetricType::Latency,
            beater_design::MetricDirection::LowerIsBetter,
        );
        let out = compare_paired_scores_with_design(&baseline, &candidate, &policy, &design)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_ne!(out.test, StatisticalTest::SequentialEValue);
        assert_eq!(
            out.decision,
            GateDecision::Inconclusive,
            "an unhonorable sequential pre-registration must block Pass: {out:?}"
        );
    }

    // ── resolve_cluster_ids: the design→executor cluster-label bridge ───────

    #[test]
    fn resolve_cluster_ids_maps_no_key_to_not_clustered() {
        let design = conservative_gate_design(&GatePolicy::default(), 4);
        let per_case = vec![CaseClusterIds::default(); 4];
        let resolution = resolve_cluster_ids(&design, &per_case);
        assert_eq!(resolution, ClusterResolution::NotClustered);
        assert_eq!(resolution.labels(), None);
    }

    #[test]
    fn resolve_cluster_ids_case_key_is_always_resolvable_by_index() {
        let mut design = conservative_gate_design(&GatePolicy::default(), 3);
        design.cluster_key = Some(beater_design::ClusterKey::Case);
        let per_case = vec![CaseClusterIds::default(); 3];
        let resolution = resolve_cluster_ids(&design, &per_case);
        assert_eq!(
            resolution,
            ClusterResolution::Resolved(vec!["0".into(), "1".into(), "2".into()])
        );
    }

    #[test]
    fn resolve_cluster_ids_trace_key_reads_the_trace_id() {
        let mut design = conservative_gate_design(&GatePolicy::default(), 4);
        design.cluster_key = Some(beater_design::ClusterKey::Trace);
        // Two conversations, two turns each — the multi-turn case §10.3 #1 is about.
        let per_case = vec![
            CaseClusterIds {
                trace_id: Some("conv-a"),
                ..Default::default()
            },
            CaseClusterIds {
                trace_id: Some("conv-a"),
                ..Default::default()
            },
            CaseClusterIds {
                trace_id: Some("conv-b"),
                ..Default::default()
            },
            CaseClusterIds {
                trace_id: Some("conv-b"),
                ..Default::default()
            },
        ];
        assert_eq!(
            resolve_cluster_ids(&design, &per_case),
            ClusterResolution::Resolved(vec![
                "conv-a".into(),
                "conv-a".into(),
                "conv-b".into(),
                "conv-b".into(),
            ])
        );
    }

    #[test]
    fn resolve_cluster_ids_is_all_or_nothing_on_a_gap() {
        let mut design = conservative_gate_design(&GatePolicy::default(), 3);
        design.cluster_key = Some(beater_design::ClusterKey::Trace);
        let per_case = vec![
            CaseClusterIds {
                trace_id: Some("t1"),
                ..Default::default()
            },
            CaseClusterIds::default(), // gap: no trace id for this case
            CaseClusterIds {
                trace_id: Some("t3"),
                ..Default::default()
            },
        ];
        assert_eq!(
            resolve_cluster_ids(&design, &per_case),
            ClusterResolution::Unresolvable(beater_design::ClusterKey::Trace)
        );
    }

    #[test]
    fn resolve_cluster_ids_custom_key_reads_the_custom_field() {
        let mut design = conservative_gate_design(&GatePolicy::default(), 2);
        design.cluster_key = Some(beater_design::ClusterKey::Custom {
            name: "tenant".into(),
        });
        let per_case = vec![
            CaseClusterIds {
                custom: Some("acme"),
                ..Default::default()
            },
            CaseClusterIds {
                custom: Some("acme"),
                ..Default::default()
            },
        ];
        assert_eq!(
            resolve_cluster_ids(&design, &per_case),
            ClusterResolution::Resolved(vec!["acme".into(), "acme".into()])
        );
    }

    /// End-to-end: resolving trace ids and feeding them through the executor
    /// makes a trace-clustered design actually usable — two conversations can't
    /// certify a win, many independent conversations can, and a design whose
    /// key can't be resolved is refused. This is what "the executor is ready
    /// for cluster labels" means in practice.
    #[test]
    fn resolved_trace_cluster_flows_through_the_executor() {
        let policy = GatePolicy {
            min_sample_size: 2,
            ..GatePolicy::default()
        };
        let mut design = conservative_gate_design(&policy, 6);
        design.unit_of_analysis = beater_design::UnitOfAnalysis::Conversation;
        design.cluster_key = Some(beater_design::ClusterKey::Trace);

        // Six turns across two conversations, all improving: i.i.d. would pass.
        let baseline = vec![0.5; 6];
        let candidate = vec![0.80, 0.81, 0.79, 0.55, 0.54, 0.56];
        let traces = ["c1", "c1", "c1", "c2", "c2", "c2"];
        let per_case: Vec<CaseClusterIds> = traces
            .iter()
            .map(|t| CaseClusterIds {
                trace_id: Some(*t),
                ..Default::default()
            })
            .collect();
        let resolution = resolve_cluster_ids(&design, &per_case);
        let out = compare_paired_scores_designed(
            &DesignedComparisonInputs {
                baseline: &baseline,
                candidate: &candidate,
                covariate: None,
                cluster_ids: resolution.labels(),
            },
            &policy,
            &design,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::ClusteredPairedT);
        assert_ne!(
            out.decision,
            GateDecision::Pass,
            "two conversations cannot certify a win: {out:?}"
        );

        // Twelve independent single-turn conversations with a consistent lift.
        let baseline = vec![0.5; 12];
        let candidate = vec![0.7; 12];
        let traces: Vec<String> = (0..12).map(|i| format!("conv-{i}")).collect();
        let per_case: Vec<CaseClusterIds> = traces
            .iter()
            .map(|t| CaseClusterIds {
                trace_id: Some(t.as_str()),
                ..Default::default()
            })
            .collect();
        let resolution = resolve_cluster_ids(&design, &per_case);
        let out = compare_paired_scores_designed(
            &DesignedComparisonInputs {
                baseline: &baseline,
                candidate: &candidate,
                covariate: None,
                cluster_ids: resolution.labels(),
            },
            &policy,
            &design,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(out.test, StatisticalTest::ClusteredPairedT);
        assert_eq!(out.decision, GateDecision::Pass, "{out:?}");

        // Unresolved: no trace ids at all → executor refuses to certify a Pass.
        let per_case = vec![CaseClusterIds::default(); 12];
        let resolution = resolve_cluster_ids(&design, &per_case);
        assert!(matches!(resolution, ClusterResolution::Unresolvable(_)));
        let out = compare_paired_scores_designed(
            &DesignedComparisonInputs {
                baseline: &baseline,
                candidate: &candidate,
                covariate: None,
                cluster_ids: resolution.labels(),
            },
            &policy,
            &design,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            out.decision,
            GateDecision::Inconclusive,
            "an unresolvable cluster key must block Pass: {out:?}"
        );
    }
}
