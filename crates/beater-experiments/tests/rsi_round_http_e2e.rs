//! End-to-end RSI optimization-round tests that drive `run_optimization_round`
//! through the **real** `beater-judge` HTTP provider seam
//! (`AnthropicJudgeProvider` / `OpenAiJudgeProvider`) against a programmable
//! in-process mock model server.
//!
//! These are the "real RSI testing with mocked API calls" suite for issues #438
//! / #434 / #435: the proposer's reflective rewrite and (in the dual-HTTP tests)
//! every per-case baseline-vs-candidate evaluation make genuine `reqwest` HTTP
//! round-trips — JSON request shaping, provider auth headers, response decoding,
//! retry policy — against an axum mock that emulates the Anthropic Messages API
//! (and an OpenAI variant). Nothing about the proposal → evaluate → held-out
//! gate + anti-overfit loop is stubbed in-process except the mock model itself.
//!
//! The matrix deliberately spans the loop's whole behavior surface:
//!   * acceptance of a candidate that genuinely generalizes (dual-HTTP),
//!   * rejection of a candidate that overfits the optimization split (dual-HTTP),
//!   * rejection of a candidate that regresses (dual-HTTP),
//!   * the full proposer error taxonomy (5xx, non-retryable 4xx, retry-then-OK,
//!     malformed body, empty completion),
//!   * the gating-invariant / honest-seam checks (reflective brief is a plain
//!     completion, never a judge rubric; BYOK key reaches the provider; no secret
//!     leaks into the proposed candidate),
//!   * and the deterministic gate edge cases (underpowered Test split, missing
//!     Test/optimization split, evaluator failure, deferred strategies, empty
//!     goal, ParamSearch grid) driven with a real HTTP proposer + scripted scores.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::err_expect)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use beater_eval::{GateDecision, GatePolicy, VarianceReduction};
use beater_experiments::{
    run_optimization_round, CandidateChange, CandidateEvaluator, CaseScore, FailureExample,
    OptimizationRoundConfig, OptimizerError, OptimizerStrategy, Split,
};
use beater_judge::{
    AnthropicJudgeProvider, GenerationRequest, GenerationResponse, HttpJudgeProviderConfig,
    JudgeProviderResult, OpenAiJudgeProvider, ProviderCredentials, RetryPolicy, TextGenerator,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;

/// A marker injected into the proposer's canned rewrite so the mock model can
/// tell a *candidate* evaluation call (system prompt = the rewrite) from a
/// *baseline* one (system prompt = the original prompt) without the test leaking
/// split information into the HTTP layer.
const CANDIDATE_MARKER: &str = "BEATER-CANDIDATE";

/// The canned "improved" system prompt the mock proposer returns. Carries the
/// candidate marker so downstream evaluation calls are attributable.
const REWRITE: &str = "You are BEATER-CANDIDATE, a meticulous factual assistant. \
                       Answer only when confident and cite a source for every claim.";

/// One factual case the dual-HTTP evaluator scores: the model is re-prompted with
/// the baseline and candidate system prompts and graded against `expected`.
#[derive(Clone, Debug)]
struct E2ECase {
    question: String,
    expected: String,
    split: Split,
}

impl E2ECase {
    fn new(question: &str, expected: &str, split: Split) -> Self {
        Self {
            question: question.to_string(),
            expected: expected.to_string(),
            split,
        }
    }
    fn is_holdout(&self) -> bool {
        self.split == Split::Test
    }
}

/// 6 optimization (Train/Val) + 6 held-out (Test) factual cases. The Test
/// questions are disjoint from the optimization questions, so acceptance is a
/// genuine generalization check rather than memorization.
fn e2e_cases() -> Vec<E2ECase> {
    vec![
        E2ECase::new("Who wrote the novel '1984'?", "George Orwell", Split::Train),
        E2ECase::new("What is the capital of Australia?", "Canberra", Split::Val),
        E2ECase::new("How many moons does Mars have?", "Two", Split::Train),
        E2ECase::new("What is the chemical symbol for gold?", "Au", Split::Val),
        E2ECase::new("What is the largest planet?", "Jupiter", Split::Train),
        E2ECase::new("In what year did Apollo 11 land?", "1969", Split::Val),
        E2ECase::new(
            "Who painted the Mona Lisa?",
            "Leonardo da Vinci",
            Split::Test,
        ),
        E2ECase::new("What is the capital of Canada?", "Ottawa", Split::Test),
        E2ECase::new(
            "What is the tallest mountain on Earth?",
            "Everest",
            Split::Test,
        ),
        E2ECase::new("What is the chemical symbol for sodium?", "Na", Split::Test),
        E2ECase::new("How many continents are there?", "Seven", Split::Test),
        E2ECase::new(
            "What gas do plants primarily absorb?",
            "Carbon dioxide",
            Split::Test,
        ),
    ]
}

/// The seeded failing examples that motivate every round (build the reflective
/// brief handed to the proposer).
fn failures() -> Vec<FailureExample> {
    vec![
        FailureExample::from_parts(
            "Who wrote the novel '1984'?",
            Some("George Orwell".to_string()),
            "Possibly Aldous Huxley.",
            0.0,
            None,
        ),
        FailureExample::from_parts(
            "What is the capital of Australia?",
            Some("Canberra".to_string()),
            "Sydney.",
            0.0,
            None,
        ),
        FailureExample::from_parts(
            "How many moons does Mars have?",
            Some("Two".to_string()),
            "One, I think.",
            0.0,
            Some("low confidence".to_string()),
        ),
    ]
}

/// Case-insensitive, whitespace-normalized substring grading — the same simple,
/// deterministic scorer the live CLI evaluator uses.
fn substring_match_score(answer: &str, expected: &str) -> f64 {
    fn normalize(s: &str) -> String {
        s.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    let answer = normalize(answer);
    let expected = normalize(expected);
    if !expected.is_empty() && answer.contains(&expected) {
        1.0
    } else {
        0.0
    }
}

/// How the mock model answers *evaluation* calls, as a function of whether the
/// caller used the candidate prompt and whether the question is held out.
#[derive(Clone, Copy, Debug)]
enum EvalPolicy {
    /// Candidate answers correctly everywhere, baseline never → uniform lift.
    Generalize,
    /// Candidate answers correctly only on the optimization split and reverts to
    /// wrong on the held-out split; baseline never correct → the overfit signature.
    Overfit,
    /// Baseline answers correctly everywhere, candidate never → a regression.
    Regress,
    /// Evaluation calls are never expected (proposer-only / error tests).
    Unused,
}

/// How the mock answers *proposer* (reflective-rewrite) calls.
#[derive(Clone, Debug)]
enum ProposerBehavior {
    /// 200 OK with the canned rewrite text.
    Rewrite,
    /// 200 OK but an empty completion (proposer must reject it).
    Empty,
    /// 200 OK with a body the Anthropic decoder cannot parse.
    Malformed,
    /// Always fail with this HTTP status.
    Status(u16),
    /// Fail with `status` for the first `fails` attempts, then return the rewrite —
    /// exercises the provider retry policy.
    FailThenRewrite { fails: usize, status: u16 },
}

struct MockState {
    proposer: ProposerBehavior,
    eval: EvalPolicy,
    cases: Vec<E2ECase>,
    /// Every request body the mock received, in arrival order.
    captured_bodies: Mutex<Vec<Value>>,
    /// Every `x-api-key` (Anthropic) / bearer token the mock received.
    captured_keys: Mutex<Vec<String>>,
    /// Proposer attempt counter, for the retry behavior.
    proposer_attempts: AtomicUsize,
}

impl MockState {
    fn new(proposer: ProposerBehavior, eval: EvalPolicy) -> Arc<Self> {
        Arc::new(Self {
            proposer,
            eval,
            cases: e2e_cases(),
            captured_bodies: Mutex::new(Vec::new()),
            captured_keys: Mutex::new(Vec::new()),
            proposer_attempts: AtomicUsize::new(0),
        })
    }
}

fn anthropic_text(text: &str) -> Response {
    Json(json!({ "content": [{ "type": "text", "text": text }] })).into_response()
}

fn openai_text(text: &str) -> Response {
    Json(json!({ "choices": [{ "message": { "content": text } }] })).into_response()
}

/// Shared request classification: a proposer call carries the reflective system
/// prompt ("prompt engineer") and a user body containing "GOAL:"; everything else
/// is an evaluation call whose system prompt is either the baseline or the
/// candidate rewrite.
fn classify(body: &Value) -> (bool, String, String) {
    // Anthropic shape: top-level "system"; OpenAI shape: a system message.
    let system = body
        .get("system")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            body.get("messages")?
                .as_array()?
                .iter()
                .find(|m| m.get("role").and_then(Value::as_str) == Some("system"))?
                .get("content")?
                .as_str()
                .map(str::to_string)
        })
        .unwrap_or_default();
    let user = body
        .get("messages")
        .and_then(Value::as_array)
        .and_then(|messages| {
            messages
                .iter()
                .rev()
                .find(|m| m.get("role").and_then(Value::as_str) == Some("user"))
        })
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    // Structural proposer signal, not a fixture coincidence: `LlmRewrite`'s
    // reflective brief unconditionally emits a "FAILURE STATS:" line (even with
    // zero failures), and that exact token never appears in a factual question or
    // an answer. So an evaluation call can never be misclassified as a proposer
    // call (or vice-versa) regardless of how a question/prompt is worded.
    let is_proposer = user.contains("FAILURE STATS:");
    (is_proposer, system, user)
}

fn eval_answer(state: &MockState, system: &str, user: &str) -> String {
    let is_candidate = system.contains(CANDIDATE_MARKER);
    let case = state
        .cases
        .iter()
        .find(|c| c.question.trim() == user.trim());
    let (expected, holdout) = match case {
        Some(c) => (c.expected.as_str(), c.is_holdout()),
        None => ("", false),
    };
    let correct = match state.eval {
        EvalPolicy::Generalize => is_candidate,
        EvalPolicy::Overfit => is_candidate && !holdout,
        EvalPolicy::Regress => !is_candidate,
        EvalPolicy::Unused => false,
    };
    if correct {
        expected.to_string()
    } else {
        "I am not sure about that.".to_string()
    }
}

/// Anthropic-shaped mock handler.
async fn anthropic_handler(
    State(state): State<Arc<MockState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if let Some(key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
        state.captured_keys.lock().unwrap().push(key.to_string());
    }
    state.captured_bodies.lock().unwrap().push(body.clone());
    let (is_proposer, system, user) = classify(&body);
    if is_proposer {
        proposer_response(&state, anthropic_text)
    } else {
        anthropic_text(&eval_answer(&state, &system, &user))
    }
}

/// OpenAI-shaped mock handler (proves the provider seam is provider-agnostic).
async fn openai_handler(
    State(state): State<Arc<MockState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        state
            .captured_keys
            .lock()
            .unwrap()
            .push(auth.trim_start_matches("Bearer ").to_string());
    }
    state.captured_bodies.lock().unwrap().push(body.clone());
    let (is_proposer, system, user) = classify(&body);
    if is_proposer {
        proposer_response(&state, openai_text)
    } else {
        openai_text(&eval_answer(&state, &system, &user))
    }
}

fn proposer_response(state: &MockState, ok: fn(&str) -> Response) -> Response {
    let attempt = state.proposer_attempts.fetch_add(1, Ordering::SeqCst);
    match &state.proposer {
        ProposerBehavior::Rewrite => ok(REWRITE),
        ProposerBehavior::Empty => ok("   "),
        ProposerBehavior::Malformed => {
            (StatusCode::OK, Json(json!({ "unexpected": "shape" }))).into_response()
        }
        ProposerBehavior::Status(code) => {
            (StatusCode::from_u16(*code).unwrap(), "provider error").into_response()
        }
        ProposerBehavior::FailThenRewrite { fails, status } => {
            if attempt < *fails {
                (StatusCode::from_u16(*status).unwrap(), "transient").into_response()
            } else {
                ok(REWRITE)
            }
        }
    }
}

enum Wire {
    Anthropic,
    OpenAi,
}

/// Spawn the mock server and return its base endpoint URL.
async fn spawn(state: Arc<MockState>, wire: Wire) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = match wire {
        Wire::Anthropic => Router::new()
            .route("/v1/messages", post(anthropic_handler))
            .with_state(state),
        Wire::OpenAi => Router::new()
            .route("/v1/chat/completions", post(openai_handler))
            .with_state(state),
    };
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let path = match wire {
        Wire::Anthropic => "/v1/messages",
        Wire::OpenAi => "/v1/chat/completions",
    };
    format!("http://{addr}{path}")
}

fn provider_config(endpoint: String) -> HttpJudgeProviderConfig {
    HttpJudgeProviderConfig {
        endpoint_url: endpoint,
        max_cost: beater_core::Money::usd_micros(0),
        // Single attempt by default; retry tests override this.
        retry_policy: RetryPolicy {
            max_attempts: 1,
            base_backoff_ms: 0,
        },
    }
}

/// A real-HTTP `CandidateEvaluator`: for each seeded case it asks the model once
/// under the baseline prompt and once under the candidate's rewritten prompt, then
/// grades both answers deterministically — the same shape as the live CLI
/// evaluator, but pointed at the mock.
struct HttpModelEvaluator<G: TextGenerator> {
    generator: G,
    credentials: ProviderCredentials,
    model: String,
    baseline_prompt: String,
    cases: Vec<E2ECase>,
}

impl<G: TextGenerator> HttpModelEvaluator<G> {
    async fn answer(&self, system: &str, question: &str) -> Result<String, String> {
        let req = GenerationRequest::new(self.model.clone(), question)
            .with_system(system)
            .with_temperature(0.0)
            .with_max_tokens(64);
        self.generator
            .generate(req, self.credentials.clone())
            .await
            .map(|r| r.text)
            .map_err(|e| e.to_string())
    }
}

#[async_trait]
impl<G: TextGenerator> CandidateEvaluator for HttpModelEvaluator<G> {
    async fn evaluate(
        &self,
        candidate: &CandidateChange,
        _cases: &[Value],
    ) -> Result<Vec<CaseScore>, String> {
        let mut scores = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            let baseline = self.answer(&self.baseline_prompt, &case.question).await?;
            let candidate_answer = self.answer(&candidate.target, &case.question).await?;
            scores.push(CaseScore {
                split: case.split,
                baseline_score: substring_match_score(&baseline, &case.expected),
                candidate_score: substring_match_score(&candidate_answer, &case.expected),
                covariate: None,
            });
        }
        Ok(scores)
    }
}

fn round_config(strategy: OptimizerStrategy) -> OptimizationRoundConfig {
    OptimizationRoundConfig::new(
        "reduce hallucinations on factual lookups",
        "You are a helpful assistant.",
        failures(),
        (0..12).map(|i| json!({ "case": i })).collect(),
        strategy,
        GatePolicy {
            min_sample_size: 6,
            max_regression: 0.0,
            alpha: 0.05,
            comparison_count: 1,
        },
    )
}

fn creds() -> ProviderCredentials {
    ProviderCredentials::new("anthropic", "sk-mock-byok-key")
}

// ---------------------------------------------------------------------------
// Dual-HTTP full E2E: proposer AND every per-case evaluation hit the mock.
// ---------------------------------------------------------------------------

/// A candidate that genuinely generalizes (correct on held-out questions too) is
/// accepted: real HTTP proposal, real HTTP scoring, real held-out gate + real
/// anti-overfit guardrail.
#[tokio::test]
async fn dual_http_generalizing_candidate_is_accepted() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Generalize);
    let endpoint = spawn(state.clone(), Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let evaluator = HttpModelEvaluator {
        generator: provider.clone(),
        credentials: creds(),
        model: "mock-model".to_string(),
        baseline_prompt: "You are a helpful assistant.".to_string(),
        cases: e2e_cases(),
    };

    let outcome = run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        creds(),
        &evaluator,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(outcome.evaluated.len(), 1);
    let eval = &outcome.evaluated[0];
    assert_eq!(eval.gate.decision, GateDecision::Pass);
    assert!(
        !eval.overfit.overfit,
        "uniform lift must not be flagged: {:?}",
        eval.overfit
    );
    assert!(eval.accepted);
    let accepted = outcome.accepted.expect("a candidate should be accepted");
    assert_eq!(accepted.proposed_by, OptimizerStrategy::LlmRewrite);
    assert!(accepted.target.contains(CANDIDATE_MARKER));
    // The proposer's rewrite did not echo the BYOK secret.
    assert!(!accepted.target.contains("sk-mock"));
}

/// A candidate that beats baseline only on the optimization split and reverts on
/// the held-out split is rejected by the anti-overfit guardrail — even though its
/// held-out delta alone (0 → 0) would not trip the regression gate. This is the
/// load-bearing §21.4 invariant, proven over real HTTP.
#[tokio::test]
async fn dual_http_overfit_candidate_is_rejected() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Overfit);
    let endpoint = spawn(state.clone(), Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let evaluator = HttpModelEvaluator {
        generator: provider.clone(),
        credentials: creds(),
        model: "mock-model".to_string(),
        baseline_prompt: "You are a helpful assistant.".to_string(),
        cases: e2e_cases(),
    };

    let outcome = run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        creds(),
        &evaluator,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));

    let eval = &outcome.evaluated[0];
    assert!(
        eval.overfit.overfit,
        "expected overfit flag: {:?}",
        eval.overfit
    );
    assert!(
        eval.overfit.gap_ci_low > 0.0,
        "gap CI must exclude 0: {:?}",
        eval.overfit
    );
    assert!(!eval.accepted, "overfit candidate must not be accepted");
    assert!(outcome.accepted.is_none());
}

/// A candidate that is strictly worse than baseline trips the held-out regression
/// gate and is rejected.
#[tokio::test]
async fn dual_http_regressing_candidate_is_rejected() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Regress);
    let endpoint = spawn(state.clone(), Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let evaluator = HttpModelEvaluator {
        generator: provider.clone(),
        credentials: creds(),
        model: "mock-model".to_string(),
        baseline_prompt: "You are a helpful assistant.".to_string(),
        cases: e2e_cases(),
    };

    let outcome = run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        creds(),
        &evaluator,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));

    let eval = &outcome.evaluated[0];
    assert_eq!(eval.gate.decision, GateDecision::FailRegression);
    assert!(!eval.accepted);
    assert!(outcome.accepted.is_none());
}

/// The same generalizing acceptance, but routed through the **OpenAI** provider
/// wire — proves the loop is provider-agnostic across the `TextGenerator` seam.
#[tokio::test]
async fn dual_http_generalizing_candidate_accepted_via_openai_wire() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Generalize);
    let endpoint = spawn(state.clone(), Wire::OpenAi).await;
    let provider = OpenAiJudgeProvider::new(provider_config(endpoint));
    let evaluator = HttpModelEvaluator {
        generator: provider.clone(),
        credentials: ProviderCredentials::new("openai", "sk-openai-mock"),
        model: "mock-model".to_string(),
        baseline_prompt: "You are a helpful assistant.".to_string(),
        cases: e2e_cases(),
    };

    let outcome = run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        ProviderCredentials::new("openai", "sk-openai-mock"),
        &evaluator,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));

    assert!(outcome.accepted.is_some());
    assert!(outcome.evaluated[0].accepted);
}

// ---------------------------------------------------------------------------
// Honest-seam / gating-invariant checks over real HTTP.
// ---------------------------------------------------------------------------

/// The proposer's HTTP call must be a *plain completion* carrying the reflective
/// brief — NOT a judge/scoring rubric — and must carry the BYOK key in the
/// provider's auth header. Proves the "proposal, not scoring" seam end-to-end.
#[tokio::test]
async fn proposer_sends_reflective_brief_not_a_judge_rubric_and_carries_the_key() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Generalize);
    let endpoint = spawn(state.clone(), Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let evaluator = HttpModelEvaluator {
        generator: provider.clone(),
        credentials: creds(),
        model: "mock-model".to_string(),
        baseline_prompt: "You are a helpful assistant.".to_string(),
        cases: e2e_cases(),
    };

    run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        creds(),
        &evaluator,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));

    let bodies = state.captured_bodies.lock().unwrap().clone();
    let proposer_body = bodies
        .iter()
        .find(|b| {
            b.get("system")
                .and_then(Value::as_str)
                .map(|s| s.contains("prompt engineer"))
                .unwrap_or(false)
        })
        .expect("a proposer request must have been sent");
    let serialized = proposer_body.to_string();
    assert!(
        serialized.contains("GOAL:"),
        "reflective brief missing GOAL: {serialized}"
    );
    assert!(
        serialized.contains("FAILURE STATS:"),
        "brief missing failure stats"
    );
    // It is a generation call, never the strict-judge scoring contract.
    assert!(
        !serialized.contains("strict evaluation judge"),
        "{serialized}"
    );
    assert!(proposer_body.get("response_format").is_none());

    // The BYOK key reached the provider on every call.
    let keys = state.captured_keys.lock().unwrap().clone();
    assert!(!keys.is_empty());
    assert!(
        keys.iter().all(|k| k == "sk-mock-byok-key"),
        "keys: {keys:?}"
    );
}

// ---------------------------------------------------------------------------
// Proposer error taxonomy — real HTTP failures map to typed OptimizerErrors.
// ---------------------------------------------------------------------------

/// A scripted evaluator that returns a fixed, generalizing score sheet without any
/// network — used by the proposer-error and gate-edge tests where the model call
/// is not the thing under test.
#[derive(Clone)]
struct ScriptedEvaluator {
    optimize: Vec<(f64, f64)>,
    test: Vec<(f64, f64)>,
}

#[async_trait]
impl CandidateEvaluator for ScriptedEvaluator {
    async fn evaluate(
        &self,
        _c: &CandidateChange,
        _cases: &[Value],
    ) -> Result<Vec<CaseScore>, String> {
        let mut out = Vec::new();
        for (i, (b, c)) in self.optimize.iter().enumerate() {
            let split = if i % 2 == 0 { Split::Train } else { Split::Val };
            out.push(CaseScore {
                split,
                baseline_score: *b,
                candidate_score: *c,
                covariate: None,
            });
        }
        for (b, c) in &self.test {
            out.push(CaseScore {
                split: Split::Test,
                baseline_score: *b,
                candidate_score: *c,
                covariate: None,
            });
        }
        Ok(out)
    }
}

fn generalizing_scores() -> ScriptedEvaluator {
    ScriptedEvaluator {
        optimize: vec![(0.5, 0.9); 6],
        test: vec![(0.5, 0.9); 6],
    }
}

async fn run_with_proposer(
    behavior: ProposerBehavior,
    retry: RetryPolicy,
) -> Result<beater_experiments::OptimizationOutcome, OptimizerError> {
    let state = MockState::new(behavior, EvalPolicy::Unused);
    let endpoint = spawn(state, Wire::Anthropic).await;
    let mut config = provider_config(endpoint);
    config.retry_policy = retry;
    let provider = AnthropicJudgeProvider::new(config);
    run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        creds(),
        &generalizing_scores(),
    )
    .await
}

#[tokio::test]
async fn proposer_server_error_surfaces_as_proposer_failed() {
    let err = run_with_proposer(
        ProposerBehavior::Status(500),
        RetryPolicy {
            max_attempts: 1,
            base_backoff_ms: 0,
        },
    )
    .await
    .err()
    .expect("a 500 must fail the proposer");
    assert!(matches!(err, OptimizerError::ProposerFailed(_)), "{err:?}");
}

#[tokio::test]
async fn proposer_non_retryable_4xx_surfaces_as_proposer_failed() {
    let err = run_with_proposer(
        ProposerBehavior::Status(400),
        RetryPolicy {
            max_attempts: 3,
            base_backoff_ms: 0,
        },
    )
    .await
    .err()
    .expect("a 400 must fail the proposer");
    assert!(matches!(err, OptimizerError::ProposerFailed(_)), "{err:?}");
}

#[tokio::test]
async fn proposer_empty_completion_surfaces_as_proposer_failed() {
    let err = run_with_proposer(
        ProposerBehavior::Empty,
        RetryPolicy {
            max_attempts: 1,
            base_backoff_ms: 0,
        },
    )
    .await
    .err()
    .expect("an empty completion must fail the proposer");
    assert!(matches!(err, OptimizerError::ProposerFailed(_)), "{err:?}");
}

#[tokio::test]
async fn proposer_malformed_body_surfaces_as_proposer_failed() {
    let err = run_with_proposer(
        ProposerBehavior::Malformed,
        RetryPolicy {
            max_attempts: 1,
            base_backoff_ms: 0,
        },
    )
    .await
    .err()
    .expect("a malformed body must fail the proposer");
    assert!(matches!(err, OptimizerError::ProposerFailed(_)), "{err:?}");
}

/// A transient 503 on the first attempt is retried by the provider; the second
/// attempt returns the rewrite and the round completes and accepts. Proves the
/// retry policy is wired through the live proposer path.
#[tokio::test]
async fn proposer_retries_transient_failure_then_succeeds() {
    let outcome = run_with_proposer(
        ProposerBehavior::FailThenRewrite {
            fails: 1,
            status: 503,
        },
        RetryPolicy {
            max_attempts: 3,
            base_backoff_ms: 0,
        },
    )
    .await
    .unwrap_or_else(|e| panic!("retry should have recovered: {e}"));
    assert!(
        outcome.accepted.is_some(),
        "candidate should be accepted after retry"
    );
}

// ---------------------------------------------------------------------------
// Gate edge cases — real HTTP proposer + scripted scores for exact control.
// ---------------------------------------------------------------------------

async fn run_real_proposer_with<E: CandidateEvaluator>(
    eval: E,
    strategy: OptimizerStrategy,
) -> Result<beater_experiments::OptimizationOutcome, OptimizerError> {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Unused);
    let endpoint = spawn(state, Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    run_optimization_round(round_config(strategy), &provider, creds(), &eval).await
}

/// A held-out Test split smaller than `min_sample_size` is a candidate the gate
/// cannot judge — a typed `GateFailed`, never a silent accept.
#[tokio::test]
async fn underpowered_test_split_is_a_typed_gate_error() {
    let eval = ScriptedEvaluator {
        optimize: vec![(0.5, 0.9); 6],
        test: vec![(0.5, 0.9); 2],
    };
    let err = run_real_proposer_with(eval, OptimizerStrategy::LlmRewrite)
        .await
        .err()
        .expect("an underpowered Test split must error");
    assert!(matches!(err, OptimizerError::GateFailed(_)), "{err:?}");
}

/// No held-out Test cases at all → the gate cannot grant acceptance → typed error.
#[tokio::test]
async fn missing_test_split_is_a_typed_gate_error() {
    let eval = ScriptedEvaluator {
        optimize: vec![(0.5, 0.9); 6],
        test: vec![],
    };
    let err = run_real_proposer_with(eval, OptimizerStrategy::LlmRewrite)
        .await
        .err()
        .expect("a missing Test split must error");
    assert!(matches!(err, OptimizerError::GateFailed(_)), "{err:?}");
}

/// No optimization (Train/Val) cases → the generalization gap cannot be assessed
/// → typed error (the anti-overfit guardrail refuses to wave the candidate
/// through unscrutinized).
#[tokio::test]
async fn missing_optimization_split_is_a_typed_gate_error() {
    let eval = ScriptedEvaluator {
        optimize: vec![],
        test: vec![(0.5, 0.9); 6],
    };
    let err = run_real_proposer_with(eval, OptimizerStrategy::LlmRewrite)
        .await
        .err()
        .expect("a missing optimization split must error");
    assert!(matches!(err, OptimizerError::GateFailed(_)), "{err:?}");
}

/// An evaluator that fails (e.g. an agent-runtime error) surfaces as a typed
/// `EvaluationFailed`, not a panic or a silent skip.
#[tokio::test]
async fn evaluator_failure_surfaces_as_evaluation_failed() {
    struct FailingEvaluator;
    #[async_trait]
    impl CandidateEvaluator for FailingEvaluator {
        async fn evaluate(
            &self,
            _c: &CandidateChange,
            _cases: &[Value],
        ) -> Result<Vec<CaseScore>, String> {
            Err("agent runtime exploded".to_string())
        }
    }
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Unused);
    let endpoint = spawn(state, Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let err = run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        creds(),
        &FailingEvaluator,
    )
    .await
    .err()
    .expect("evaluator failure must error");
    assert!(
        matches!(err, OptimizerError::EvaluationFailed(_)),
        "{err:?}"
    );
}

/// A deterministic (LLM-free) strategy drives the whole grid through the gate
/// without ever calling the generator: ParamSearch emits 6 grid points, each
/// gated against the scripted generalizing scores.
#[tokio::test]
async fn param_search_grid_is_gated_without_calling_the_generator() {
    // A generator that panics if touched — proves ParamSearch never calls it.
    struct PanicGenerator;
    #[async_trait]
    impl TextGenerator for PanicGenerator {
        async fn generate(
            &self,
            _r: GenerationRequest,
            _c: ProviderCredentials,
        ) -> JudgeProviderResult<GenerationResponse> {
            panic!("deterministic strategy must not call the generator");
        }
    }
    let outcome = run_optimization_round(
        round_config(OptimizerStrategy::ParamSearch),
        &PanicGenerator,
        creds(),
        &generalizing_scores(),
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(outcome.evaluated.len(), 6);
    assert!(outcome.evaluated.iter().all(|e| e.accepted));
    assert_eq!(
        outcome.accepted.unwrap().proposed_by,
        OptimizerStrategy::ParamSearch
    );
}

/// A genuinely deferred strategy returns a typed `NotYetImplemented` rather than
/// proposing junk — and never reaches the gate.
#[tokio::test]
async fn deferred_strategy_is_not_yet_implemented() {
    let err = run_real_proposer_with(generalizing_scores(), OptimizerStrategy::Gepa)
        .await
        .err()
        .expect("Gepa is deferred");
    assert!(
        matches!(
            err,
            OptimizerError::NotYetImplemented(OptimizerStrategy::Gepa)
        ),
        "{err:?}"
    );
}

/// An empty optimization goal is rejected up front as a config error — the round
/// never proposes or calls the model.
#[tokio::test]
async fn empty_goal_is_a_config_error() {
    struct PanicGenerator;
    #[async_trait]
    impl TextGenerator for PanicGenerator {
        async fn generate(
            &self,
            _r: GenerationRequest,
            _c: ProviderCredentials,
        ) -> JudgeProviderResult<GenerationResponse> {
            panic!("empty goal must short-circuit before any model call");
        }
    }
    let mut cfg = round_config(OptimizerStrategy::LlmRewrite);
    cfg.goal = "   ".to_string();
    let err = run_optimization_round(cfg, &PanicGenerator, creds(), &generalizing_scores())
        .await
        .err()
        .expect("empty goal must error");
    assert!(matches!(err, OptimizerError::InvalidConfig(_)), "{err:?}");
}

// ---------------------------------------------------------------------------
// Additional gate-behaviour coverage surfaced by review: the third gate verdict,
// multiple-comparison correction, the OpenAI failure path, accept-ordering with
// mixed candidate outcomes, and an empty failure set.
// ---------------------------------------------------------------------------

/// A held-out Test split that is powered (≥ `min_sample_size`) but statistically
/// ambiguous — a tiny, noisy effect whose CI straddles the regression bound —
/// resolves to `Inconclusive`, NOT `Pass`, and is therefore not accepted. This
/// exercises the third `GateDecision` arm (and its power annotations) that the
/// Pass/FailRegression tests never reach.
#[tokio::test]
async fn inconclusive_gate_decision_is_not_accepted() {
    // Small mean lift with high variance → paired-t CI brackets 0 at α=0.05.
    let noisy = vec![
        (0.50, 0.55),
        (0.50, 0.45),
        (0.50, 0.62),
        (0.50, 0.40),
        (0.50, 0.58),
        (0.50, 0.48),
    ];
    let eval = ScriptedEvaluator {
        optimize: noisy.clone(),
        test: noisy,
    };
    let outcome = run_real_proposer_with(eval, OptimizerStrategy::LlmRewrite)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    let evaluation = &outcome.evaluated[0];
    assert_eq!(
        evaluation.gate.decision,
        GateDecision::Inconclusive,
        "expected an ambiguous CI to be Inconclusive, got {:?} (ci [{}, {}])",
        evaluation.gate.decision,
        evaluation.gate.ci_low,
        evaluation.gate.ci_high
    );
    assert!(!evaluation.accepted, "an inconclusive gate must not accept");
    assert!(outcome.accepted.is_none());
}

/// Multiple-comparison correction flows through the RSI gate: the *same*
/// candidate that clears the gate at `comparison_count = 1` is held to a
/// Bonferroni-tightened bound at `comparison_count = 12` and no longer clears it.
/// Proves screening many candidates against one baseline cannot manufacture a
/// "win" by chance — the §21.4 / #436 multiplicity guard is real in the loop.
#[tokio::test]
async fn multiple_comparison_correction_tightens_the_rsi_gate() {
    // A modest, genuine lift (mean ≈ +0.15, sd ≈ 0.13) chosen so the paired-t CI
    // lower bound clears 0 at α=0.05 but dips below 0 once α is divided by 12 —
    // i.e. significant alone, withdrawn under a 12-way Bonferroni correction.
    let pairs = vec![
        (0.5, 0.5),
        (0.5, 0.8),
        (0.5, 0.5),
        (0.5, 0.8),
        (0.5, 0.55),
        (0.5, 0.75),
        (0.5, 0.6),
        (0.5, 0.7),
    ];
    let scores = ScriptedEvaluator {
        optimize: pairs.clone(),
        test: pairs,
    };

    let decision_at = |comparison_count: usize, scores: ScriptedEvaluator| async move {
        let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Unused);
        let endpoint = spawn(state, Wire::Anthropic).await;
        let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
        let mut cfg = round_config(OptimizerStrategy::LlmRewrite);
        cfg.gate_policy.comparison_count = comparison_count;
        let outcome = run_optimization_round(cfg, &provider, creds(), &scores)
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        outcome.evaluated[0].gate.decision.clone()
    };

    let at_one = decision_at(1, scores.clone()).await;
    let at_twelve = decision_at(12, scores).await;
    assert_eq!(at_one, GateDecision::Pass, "should pass uncorrected");
    assert_ne!(
        at_twelve,
        GateDecision::Pass,
        "a 12x Bonferroni correction must withdraw the uncorrected pass"
    );
}

/// The OpenAI provider wire surfaces a regression rejection too — proving the
/// loop is provider-agnostic on the *failure* path, not just the accept path.
#[tokio::test]
async fn dual_http_regressing_candidate_is_rejected_via_openai_wire() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Regress);
    let endpoint = spawn(state.clone(), Wire::OpenAi).await;
    let provider = OpenAiJudgeProvider::new(provider_config(endpoint));
    let evaluator = HttpModelEvaluator {
        generator: provider.clone(),
        credentials: ProviderCredentials::new("openai", "sk-openai-mock"),
        model: "mock-model".to_string(),
        baseline_prompt: "You are a helpful assistant.".to_string(),
        cases: e2e_cases(),
    };
    let outcome = run_optimization_round(
        round_config(OptimizerStrategy::LlmRewrite),
        &provider,
        ProviderCredentials::new("openai", "sk-openai-mock"),
        &evaluator,
    )
    .await
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        outcome.evaluated[0].gate.decision,
        GateDecision::FailRegression
    );
    assert!(outcome.accepted.is_none());
}

/// A scripted evaluator that scores a candidate by its `target`: candidates whose
/// target contains `accept_marker` generalize (accepted); all others regress
/// (rejected). Lets a ParamSearch round produce a *mix* of accepted and rejected
/// grid points.
struct TargetedEvaluator {
    accept_marker: String,
}

#[async_trait]
impl CandidateEvaluator for TargetedEvaluator {
    async fn evaluate(
        &self,
        candidate: &CandidateChange,
        _cases: &[Value],
    ) -> Result<Vec<CaseScore>, String> {
        let good = candidate.target.contains(&self.accept_marker);
        let (b, c) = if good { (0.5, 0.9) } else { (0.9, 0.5) };
        let mut out = Vec::new();
        for i in 0..6 {
            let split = if i % 2 == 0 { Split::Train } else { Split::Val };
            out.push(CaseScore {
                split,
                baseline_score: b,
                candidate_score: c,
                covariate: None,
            });
        }
        for _ in 0..6 {
            out.push(CaseScore {
                split: Split::Test,
                baseline_score: b,
                candidate_score: c,
                covariate: None,
            });
        }
        Ok(out)
    }
}

/// When several proposed candidates clear both gates, the FIRST in proposal order
/// is the accepted one — and every candidate is still evaluated for the audit
/// trail. Driven by a ParamSearch grid where only the `temperature=0.3` points
/// generalize: the earlier `temperature=0` points are evaluated and rejected, the
/// first accepting point wins, and later accepting points do not override it.
#[tokio::test]
async fn first_accepted_candidate_in_proposal_order_wins() {
    let evaluator = TargetedEvaluator {
        accept_marker: "temperature=0.3".to_string(),
    };
    let outcome = run_real_proposer_with(evaluator, OptimizerStrategy::ParamSearch)
        .await
        .unwrap_or_else(|e| panic!("{e}"));

    // All six grid points were evaluated (full audit trail).
    assert_eq!(outcome.evaluated.len(), 6);
    let accepted_count = outcome.evaluated.iter().filter(|e| e.accepted).count();
    assert_eq!(
        accepted_count, 2,
        "both temperature=0.3 points should clear"
    );
    // The earliest accepting grid point wins: temperature=0.3, top_p=0.9.
    let accepted = outcome.accepted.expect("a candidate should be accepted");
    assert!(
        accepted.target.contains("temperature=0.3,top_p=0.9"),
        "first-in-order accept should win, got {}",
        accepted.target
    );
    // The earlier temperature=0 points were evaluated but rejected (regression).
    assert!(outcome.evaluated[0]
        .candidate
        .target
        .contains("temperature=0,"));
    assert!(!outcome.evaluated[0].accepted);
}

/// An optimization round with NO failing examples still proposes and gates: the
/// reflective brief is built with zero examples (its "FAILURE STATS: 0" line) and
/// the round runs to a normal verdict. Pins the current behaviour — empty
/// failures are allowed, not a hard config error.
#[tokio::test]
async fn empty_failures_round_still_proposes_and_gates() {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Unused);
    let endpoint = spawn(state, Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let mut cfg = round_config(OptimizerStrategy::LlmRewrite);
    cfg.failures = Vec::new();
    let outcome = run_optimization_round(cfg, &provider, creds(), &generalizing_scores())
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(outcome.evaluated.len(), 1, "a candidate is still proposed");
    assert!(
        outcome.accepted.is_some(),
        "a generalizing candidate is accepted"
    );
}

// ---------------------------------------------------------------------------
// CUPED variance reduction (§10.3 #4 / #436 item 4) end-to-end.
//
// A worked sample of the whole path: a real HTTP proposer, the public
// `run_optimization_round` entrypoint, and a per-case pre-experiment difficulty
// covariate carried on `CaseScore::covariate`. The held-out lift is a genuine but
// small +0.05 buried under noise that the difficulty proxy almost fully explains,
// so the raw paired-t is underpowered — until a *pre-registered* CUPED covariate
// regresses the noise out.
// ---------------------------------------------------------------------------

/// A scripted evaluator that emits, per held-out Test case, a `(baseline,
/// candidate, difficulty)` triple — `difficulty` is a pre-experiment covariate
/// (independent of the candidate: baseline is constant, so it is a pure
/// case-hardness proxy, never an arm's own score). The optimization split carries
/// the same clean +0.05 lift so there is no generalization gap for the
/// anti-overfit guardrail to trip on. These are the exact values the
/// `beater-experiments` unit test proves flip the gate, so the outcome is
/// deterministic over the real HTTP seam.
#[derive(Clone)]
struct DifficultyCovariateEvaluator;

impl DifficultyCovariateEvaluator {
    /// 12 held-out cases: `(difficulty, candidate_score)`, baseline fixed at 0.50.
    /// The candidate score tracks difficulty tightly, so almost all of the
    /// case-to-case spread in the paired lift is explained by the covariate.
    const HELD_OUT: [(f64, f64); 12] = [
        (0.0, 0.405),
        (1.0, 0.695),
        (0.1, 0.435),
        (0.9, 0.665),
        (0.2, 0.465),
        (0.8, 0.635),
        (0.3, 0.495),
        (0.7, 0.605),
        (0.4, 0.525),
        (0.6, 0.575),
        (0.45, 0.54),
        (0.55, 0.56),
    ];
}

#[async_trait]
impl CandidateEvaluator for DifficultyCovariateEvaluator {
    async fn evaluate(
        &self,
        _candidate: &CandidateChange,
        _cases: &[Value],
    ) -> Result<Vec<CaseScore>, String> {
        let mut out = Vec::new();
        for (difficulty, candidate_score) in Self::HELD_OUT {
            out.push(CaseScore {
                split: Split::Test,
                baseline_score: 0.50,
                candidate_score,
                covariate: Some(difficulty),
            });
        }
        // Optimization split: the same +0.05 mean lift, cleanly, so the
        // generalization gap is ~0 (no covariate needed off the held-out split).
        for i in 0..8 {
            let split = if i % 2 == 0 { Split::Train } else { Split::Val };
            out.push(CaseScore {
                split,
                baseline_score: 0.50,
                candidate_score: 0.55,
                covariate: None,
            });
        }
        Ok(out)
    }
}

/// Runs one round through the real HTTP proposer with the given variance-reduction
/// policy and returns the outcome. A fresh mock server per call keeps the two
/// rounds independent.
async fn run_round_with_variance_reduction(
    policy: VarianceReduction,
) -> beater_experiments::OptimizationOutcome {
    let state = MockState::new(ProposerBehavior::Rewrite, EvalPolicy::Unused);
    let endpoint = spawn(state, Wire::Anthropic).await;
    let provider = AnthropicJudgeProvider::new(provider_config(endpoint));
    let mut cfg = round_config(OptimizerStrategy::LlmRewrite);
    cfg.variance_reduction = policy;
    run_optimization_round(cfg, &provider, creds(), &DifficultyCovariateEvaluator)
        .await
        .unwrap_or_else(|e| panic!("{e}"))
}

/// The sample, end-to-end: the SAME held-out scores are `Inconclusive` (and not
/// accepted) under the default no-variance-reduction policy, but resolve to a
/// `Pass` (and are accepted) once the round pre-registers a CUPED difficulty
/// covariate — proving the covariate and its known population mean actually reach
/// the gate through the public `run_optimization_round` entrypoint, over real HTTP.
#[tokio::test]
async fn cuped_difficulty_covariate_resolves_the_round_over_real_http() {
    // Baseline policy: no variance reduction → the noise leaves the gate underpowered.
    let plain = run_round_with_variance_reduction(VarianceReduction::None).await;
    assert_eq!(
        plain.evaluated[0].gate.decision,
        GateDecision::Inconclusive,
        "without CUPED the held-out gate is underpowered"
    );
    assert!(
        plain.accepted.is_none(),
        "an inconclusive round accepts nothing"
    );

    // Pre-registered CUPED covariate (known population difficulty mean 0.7): the
    // regression estimator regresses the noise out and the tightened CI clears the
    // bound → Pass, and with no generalization gap the candidate is accepted.
    let cuped = run_round_with_variance_reduction(VarianceReduction::Cuped {
        covariate: "prior_difficulty".to_string(),
        population_mean: 0.7,
    })
    .await;
    let evaluation = &cuped.evaluated[0];
    assert_eq!(
        evaluation.gate.decision,
        GateDecision::Pass,
        "a pre-registered CUPED covariate resolves the same round to a Pass"
    );
    // The wire `test` stays `PairedT` by design — a paired t on CUPED-adjusted
    // differences is still a paired t, and "CUPED was applied" is recorded in the
    // pre-registered design, not the wire result (that is why the /v1 contract is
    // unchanged). So CUPED is observable here through its *effect*, not a new enum:
    // the regression estimator moves the point estimate off the raw delta...
    assert_eq!(evaluation.gate.test, beater_eval::StatisticalTest::PairedT);
    assert!(
        evaluation.gate.delta > plain.evaluated[0].gate.delta,
        "the known-mean regression estimator must move the delta upward \
         (plain {:.4} vs cuped {:.4})",
        plain.evaluated[0].gate.delta,
        evaluation.gate.delta
    );
    // ...and the variance-reduced CI is strictly narrower than the raw paired-t CI
    // — together, the mechanism by which CUPED earns the Pass on the same data.
    let plain_width = plain.evaluated[0].gate.ci_high - plain.evaluated[0].gate.ci_low;
    let cuped_width = evaluation.gate.ci_high - evaluation.gate.ci_low;
    assert!(
        cuped_width < plain_width,
        "CUPED must narrow the gate CI (plain {plain_width:.4} vs cuped {cuped_width:.4})"
    );
    assert!(
        cuped.accepted.is_some(),
        "a covariate-resolved Pass with no overfit gap must be accepted"
    );
}
