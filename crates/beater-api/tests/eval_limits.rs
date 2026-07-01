//! Resource bounding for inline eval execution: the process-wide eval
//! concurrency limiter must QUEUE (never reject) concurrent evals, and the
//! per-request timeout must surface on the already-documented 500 path with
//! an actionable message. Limits are constructed directly on the test's
//! `ApiState` (not via env vars) so parallel tests cannot race on process env.

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::Router;
use beater_api::{router, ApiState, EvalLimits};
use beater_bus::InMemoryBus;
use beater_core::{EnvironmentId, Money, ProjectId, SpanId, TenantId, TenantScope, TraceId};
use beater_datasets::{Dataset, DatasetEvalReport, DatasetVersionSnapshot, SqliteDatasetStore};
use beater_ingest::{IngestPolicy, IngestService, NativeIngestRequest};
use beater_judge::{
    JudgeBroker, JudgeBrokerError, JudgeBrokerOutcome, JudgeBrokerRequest, JudgeBrokerService,
    KeywordJudgeProvider, SqliteJudgeLedger,
};
use beater_schema::{AgentSpanKind, RedactionClass, SpanStatus};
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use http::{Request, StatusCode};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

/// Wraps the real keyword-judge broker with a per-call async delay and a
/// concurrency probe, so tests can hold judge dataset evals in flight and
/// observe whether two evals ever overlapped inside the broker.
struct SlowJudgeBroker {
    inner: Arc<dyn JudgeBroker>,
    delay: Duration,
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
}

impl SlowJudgeBroker {
    fn new(inner: Arc<dyn JudgeBroker>, delay: Duration) -> Self {
        Self {
            inner,
            delay,
            in_flight: Arc::new(AtomicUsize::new(0)),
            max_in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn max_observed_in_flight(&self) -> usize {
        self.max_in_flight.load(Ordering::SeqCst)
    }
}

/// Decrements the in-flight gauge even when the surrounding future is
/// cancelled (e.g. by the eval timeout).
struct InFlightGuard(Arc<AtomicUsize>);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl JudgeBroker for SlowJudgeBroker {
    async fn evaluate(
        &self,
        request: JudgeBrokerRequest,
    ) -> Result<JudgeBrokerOutcome, JudgeBrokerError> {
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(now, Ordering::SeqCst);
        let _guard = InFlightGuard(self.in_flight.clone());
        tokio::time::sleep(self.delay).await;
        self.inner.evaluate(request).await
    }

    fn remaining_budget(&self) -> Result<Money, JudgeBrokerError> {
        self.inner.remaining_budget()
    }
}

struct EvalTestApp {
    app: Router,
    judge: Arc<SlowJudgeBroker>,
    dataset_id: String,
    version_id: String,
    provider_secret_id: serde_json::Value,
    _tempdir: tempfile::TempDir,
}

/// Build a router with datasets + judge wired, the given eval limits, and one
/// dataset version (one case promoted from an ingested trace) plus a provider
/// secret, ready for judge/deterministic eval requests.
async fn eval_test_app(limits: EvalLimits, judge_delay: Duration) -> EvalTestApp {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let datasets = Arc::new(SqliteDatasetStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let provider_secrets = Arc::new(
        EncryptedSqliteProviderSecretStore::in_memory(
            SecretKeyring::generated_for_tests().unwrap_or_else(|err| panic!("{err}")),
        )
        .unwrap_or_else(|err| panic!("{err}")),
    );
    let judge_ledger =
        Arc::new(SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let inner_broker: Arc<dyn JudgeBroker> = Arc::new(JudgeBrokerService::new(
        provider_secrets.clone(),
        judge_ledger.clone(),
        KeywordJudgeProvider::new(Money::usd_micros(25)),
        Money::usd_micros(10_000),
    ));
    let judge = Arc::new(SlowJudgeBroker::new(inner_broker, judge_delay));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(
        ApiState::new(ingest, traces)
            .with_datasets(datasets)
            .with_judge(provider_secrets, judge.clone(), judge_ledger)
            .with_eval_limits(limits),
    );

    let (status, _) = post_json(
        &app,
        "/v1/traces/native",
        serde_json::to_value(native_request()).unwrap_or_else(|err| panic!("{err}")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, dataset) = post_json(
        &app,
        "/v1/datasets/tenant/project",
        json!({"name": "bounded-evals"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let dataset: Dataset = serde_json::from_value(dataset).unwrap_or_else(|err| panic!("{err}"));
    let dataset_id = dataset.dataset_id.as_str().to_string();

    let (status, _) = post_json(
        &app,
        &format!("/v1/datasets/tenant/project/{dataset_id}/cases/from-trace"),
        json!({"trace_id": "trace", "span_id": "span", "reference": "answer"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, version) = post_json(
        &app,
        &format!("/v1/datasets/tenant/project/{dataset_id}/versions"),
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let version: DatasetVersionSnapshot =
        serde_json::from_value(version).unwrap_or_else(|err| panic!("{err}"));
    let version_id = version.version_id.as_str().to_string();

    let (status, provider_secret) = post_json(
        &app,
        "/v1/provider-secrets/tenant/project",
        json!({
            "provider": "openai",
            "display_name": "eval limits judge",
            "secret_value": "sk-eval-limits-judge"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let provider_secret_id = provider_secret["provider_secret_id"].clone();

    EvalTestApp {
        app,
        judge,
        dataset_id,
        version_id,
        provider_secret_id,
        _tempdir: tempdir,
    }
}

async fn post_json(
    app: &Router,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|err| panic!("non-JSON body ({err}): {}", String::from_utf8_lossy(&bytes)));
    (status, value)
}

fn judge_eval_uri(app: &EvalTestApp) -> String {
    format!(
        "/v1/datasets/tenant/project/{}/versions/{}/evals/judge",
        app.dataset_id, app.version_id
    )
}

fn judge_eval_body(app: &EvalTestApp, evaluator_version_id: &str) -> serde_json::Value {
    json!({
        "evaluator_id": "judge-correctness",
        "evaluator_version_id": evaluator_version_id,
        "agent_release_id": "release-a",
        "kind": {
            "type": "llm_judge",
            "rubric": "correctness",
            "model": "judge-model"
        },
        "provider_secret_id": app.provider_secret_id
    })
}

#[tokio::test]
async fn concurrent_judge_evals_queue_on_the_limiter_and_both_succeed() {
    // One permit: two concurrent slow judge dataset evals must serialize on
    // the semaphore (queue, not reject) and both complete with 200.
    let app = eval_test_app(
        EvalLimits::new(1, Duration::from_secs(30)),
        Duration::from_millis(200),
    )
    .await;

    let uri = judge_eval_uri(&app);
    // Distinct evaluator versions so the second eval cannot be served from
    // the broker cache namespace of the first (both still traverse the slow
    // wrapper either way; this keeps the probe honest).
    let first = post_json(&app.app, &uri, judge_eval_body(&app, "judge-v1"));
    let second = post_json(&app.app, &uri, judge_eval_body(&app, "judge-v2"));
    let ((status_a, report_a), (status_b, report_b)) = tokio::join!(first, second);

    assert_eq!(status_a, StatusCode::OK, "first eval failed: {report_a}");
    assert_eq!(status_b, StatusCode::OK, "second eval failed: {report_b}");
    let report_a: DatasetEvalReport =
        serde_json::from_value(report_a).unwrap_or_else(|err| panic!("{err}"));
    let report_b: DatasetEvalReport =
        serde_json::from_value(report_b).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report_a.result_count, 1);
    assert_eq!(report_b.result_count, 1);

    // The limiter held the second eval back while the first slept inside the
    // judge broker: at no point did two evals overlap.
    assert_eq!(
        app.judge.max_observed_in_flight(),
        1,
        "evals overlapped despite a single-permit limiter"
    );
}

#[tokio::test]
async fn hung_judge_eval_times_out_on_the_documented_500_path() {
    // A judge that sleeps far past the eval timeout: the request must return
    // the existing internal-error (500) shape with an actionable message, not
    // hang the handler forever.
    let app = eval_test_app(
        EvalLimits::new(4, Duration::from_millis(100)),
        Duration::from_secs(60),
    )
    .await;

    let uri = judge_eval_uri(&app);
    let (status, body) = post_json(&app.app, &uri, judge_eval_body(&app, "judge-v1")).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "body: {body}");
    assert_eq!(body["status"], json!(500));
    let message = body["error"].as_str().unwrap_or_default();
    assert!(
        message.contains("eval timed out after"),
        "unexpected error message: {message}"
    );
    assert!(
        message.contains("BEATER_EVAL_TIMEOUT_SECS"),
        "timeout message must name the env knob: {message}"
    );
}

#[tokio::test]
async fn deterministic_eval_runs_on_the_blocking_pool_under_the_limiter() {
    // The sync CPU-bound deterministic eval path now goes through
    // spawn_blocking under the same limiter; a single-permit state must still
    // serve it successfully end to end.
    let app = eval_test_app(
        EvalLimits::new(1, Duration::from_secs(30)),
        Duration::from_millis(1),
    )
    .await;

    let (status, report) = post_json(
        &app.app,
        &format!(
            "/v1/datasets/tenant/project/{}/versions/{}/evals/deterministic",
            app.dataset_id, app.version_id
        ),
        json!({
            "evaluator_id": "exact",
            "evaluator_version_id": "exact-v1",
            "agent_release_id": "release-a",
            "kind": {"type": "exact_match"}
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {report}");
    let report: DatasetEvalReport =
        serde_json::from_value(report).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.result_count, 1);
    assert_eq!(report.aggregate_score, 1.0);
}

fn native_request() -> NativeIngestRequest {
    NativeIngestRequest {
        scope: TenantScope::new(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
        ),
        trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
        span_id: SpanId::new("span").unwrap_or_else(|err| panic!("{err}")),
        parent_span_id: None,
        seq: 1,
        kind: AgentSpanKind::AgentRun,
        name: "eval limits agent run".to_string(),
        status: SpanStatus::Ok,
        start_time: None,
        end_time: None,
        model: None,
        cost: None,
        tokens: None,
        input: Some(json!("question")),
        output: Some(json!("answer")),
        attributes: BTreeMap::new(),
        redaction_class: RedactionClass::Internal,
        idempotency_key: None,
        auth_context: None,
    }
}
