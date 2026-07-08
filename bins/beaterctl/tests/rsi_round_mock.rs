//! Binary-level E2E for the LIVE `beaterctl rsi-round` command, driven against an
//! in-process mock Anthropic server.
//!
//! `rsi_round_live.rs` proves the BYOK key-gate fails cleanly with no network.
//! These tests prove the *other* half: with a (fake) key set and the endpoint
//! pointed at a mock via `BEATER_ANTHROPIC_BASE_URL`, the actual shipped binary
//! runs the whole RSI round — proposer rewrite + per-case baseline/candidate
//! evaluation + held-out gate + anti-overfit guardrail — over real HTTP to the
//! mock, and prints a faithful JSON report. No real Anthropic call, no real key.
//!
//! The mock emulates the model so the round's *decision* is determined by the
//! (mocked) answer quality:
//!   * a generalizing candidate (correct everywhere) is accepted;
//!   * an overfitting candidate (correct only on the optimization split) is
//!     rejected by the §21.4 guardrail.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::err_expect)]

use std::sync::Arc;

use axum::{
    Router,
    extract::{Json, State},
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::{Value, json};
use tokio::net::TcpListener;

/// Marker the mock injects into the proposer's rewrite so evaluation calls that
/// use the candidate prompt are distinguishable from baseline ones.
const CANDIDATE_MARKER: &str = "BEATER-CANDIDATE";
const REWRITE: &str =
    "You are BEATER-CANDIDATE, a meticulous factual assistant. Answer correctly and cite sources.";

/// The seeded factual Q→A map the live command grades against
/// (`live_factual_cases()` in the binary). `holdout` marks the held-out Test
/// split (the last six), which the overfit policy answers wrong.
fn cases() -> Vec<(&'static str, &'static str, bool)> {
    vec![
        ("Who wrote the novel '1984'?", "George Orwell", false),
        ("What is the capital of Australia?", "Canberra", false),
        ("How many moons does Mars have?", "Two", false),
        ("What is the chemical symbol for gold?", "Au", false),
        (
            "In what year did the first human land on the Moon?",
            "1969",
            false,
        ),
        (
            "What is the largest planet in our solar system?",
            "Jupiter",
            false,
        ),
        ("Who painted the Mona Lisa?", "Leonardo da Vinci", true),
        ("What is the capital of Canada?", "Ottawa", true),
        (
            "What is the tallest mountain on Earth?",
            "Mount Everest",
            true,
        ),
        ("What is the chemical symbol for sodium?", "Na", true),
        ("How many continents are there on Earth?", "Seven", true),
        (
            "What is the speed of light in a vacuum (approx, km/s)?",
            "299792",
            true,
        ),
    ]
}

#[derive(Clone, Copy)]
enum Policy {
    Generalize,
    Overfit,
}

struct MockState {
    policy: Policy,
}

fn text_response(text: &str) -> Response {
    Json(json!({ "content": [{ "type": "text", "text": text }] })).into_response()
}

async fn handler(State(state): State<Arc<MockState>>, Json(body): Json<Value>) -> Response {
    let system = body
        .get("system")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let user = body
        .get("messages")
        .and_then(Value::as_array)
        .and_then(|m| m.last())
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    // Proposer (reflective rewrite) call → return the canned candidate prompt.
    // Structural signal: the reflective brief always carries a "FAILURE STATS:"
    // line, which never appears in a factual question/answer — so eval calls are
    // never misclassified as proposer calls.
    if user.contains("FAILURE STATS:") {
        return text_response(REWRITE);
    }

    // Evaluation call → grade by which prompt is in play and the policy.
    let is_candidate = system.contains(CANDIDATE_MARKER);
    let case = cases()
        .into_iter()
        .find(|(q, _, _)| user.trim() == q.trim());
    let (expected, holdout) = match case {
        Some((_, a, h)) => (a, h),
        None => ("", false),
    };
    let correct = match state.policy {
        Policy::Generalize => is_candidate,
        Policy::Overfit => is_candidate && !holdout,
    };
    if correct {
        text_response(expected)
    } else {
        text_response("I am not sure about that.")
    }
}

/// Spawn the mock and return the `/v1/messages` endpoint URL.
async fn spawn_mock(policy: Policy) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/v1/messages", post(handler))
        .with_state(Arc::new(MockState { policy }));
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}/v1/messages")
}

async fn run_rsi_round(endpoint: &str) -> Value {
    // The mock server runs on the multi-thread runtime's worker tasks; run the
    // blocking subprocess on a dedicated blocking thread so it can talk to it.
    let endpoint = endpoint.to_string();
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_beaterctl"))
            .arg("rsi-round")
            .env("ANTHROPIC_API_KEY", "sk-mock-fake-key")
            .env("BEATER_ANTHROPIC_BASE_URL", endpoint)
            .output()
            .expect("spawn beaterctl rsi-round")
    })
    .await
    .expect("join subprocess task");
    assert!(
        output.status.success(),
        "rsi-round should succeed against the mock; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("rsi-round must print JSON")
}

/// With a fake key + the endpoint pointed at a mock that makes the candidate
/// generalize, the real binary runs the whole round over HTTP and accepts.
#[tokio::test(flavor = "multi_thread")]
async fn rsi_round_accepts_generalizing_candidate_against_mock() {
    let endpoint = spawn_mock(Policy::Generalize).await;
    let report = run_rsi_round(&endpoint).await;

    assert_eq!(report["model"], "claude-haiku-4-5-20251001");
    let accepted = &report["accepted_candidate"];
    assert!(
        accepted.is_object(),
        "expected an accepted candidate, got {accepted:?}"
    );
    assert_eq!(accepted["proposed_by"], "llm_rewrite");
    assert_eq!(accepted["kind"], "system_prompt");
    assert!(
        accepted["target"]
            .as_str()
            .unwrap_or_default()
            .contains(CANDIDATE_MARKER)
    );

    let evaluated = report["evaluated"].as_array().expect("evaluated array");
    assert_eq!(evaluated.len(), 1);
    assert_eq!(evaluated[0]["gate_decision"], "pass");
    assert_eq!(evaluated[0]["overfit_flag"], false);
    assert_eq!(evaluated[0]["accepted"], true);

    // The report must not leak the (fake) BYOK secret.
    assert!(!report.to_string().contains("sk-mock-fake-key"));
}

/// With the mock making the candidate beat baseline only on the optimization
/// split, the real binary surfaces the §21.4 anti-overfit rejection: the gate may
/// not regress on the held-out split, but the generalization-gap guardrail fires,
/// so no candidate is accepted.
#[tokio::test(flavor = "multi_thread")]
async fn rsi_round_rejects_overfitting_candidate_against_mock() {
    let endpoint = spawn_mock(Policy::Overfit).await;
    let report = run_rsi_round(&endpoint).await;

    assert!(
        report["accepted_candidate"].is_null(),
        "overfitting candidate must not be accepted, got {:?}",
        report["accepted_candidate"]
    );
    let evaluated = report["evaluated"].as_array().expect("evaluated array");
    assert_eq!(evaluated.len(), 1);
    assert_eq!(evaluated[0]["overfit_flag"], true);
    assert_eq!(evaluated[0]["accepted"], false);
}
