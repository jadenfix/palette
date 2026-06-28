use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::Router;
use beater_alerts::{AlertDecision, SamplingDecision, SamplingReason};
use beater_api::{router, ApiState};
use beater_archive::{ArchiveManifest, ParquetTraceArchive};
use beater_audit::SqliteAuditStore;
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
use beater_bus::{BusMessage, DurableBus, InMemoryBus};
use beater_calibration::{CalibrationReport, SqliteCalibrationStore};
use beater_core::{
    ApiKeyId, Clock, EnvironmentId, IdempotencyKey, Money, OrganizationId, Page, ProjectId, SpanId,
    TenantId, TenantScope, Timestamp, TraceId,
};
use beater_datasets::{
    Dataset, DatasetCase, DatasetEvalReport, DatasetVersionSnapshot, SqliteDatasetStore,
};
use beater_eval::{
    compare_paired_scores, evaluate_deterministic, EvaluationCase, EvaluatorKind, EvaluatorSpec,
    GateDecision, GatePolicy,
};
use beater_experiments::{ExperimentRunReport, SqliteExperimentStore};
use beater_gates::{GateDefinition, GateRunReport, SqliteGateStore};
use beater_human::{
    ReviewAnnotation, ReviewQueue, ReviewTask, ReviewTaskState, SqliteHumanReviewStore,
};
use beater_ingest::{
    DeadLetterReplayReport, IngestOutcome, IngestPolicy, IngestQueueStatus, IngestService,
    NativeIngestRequest, TraceIngestedDrainReport, TraceIngestedReconcileReport,
    TraceWriteDrainReport, TRACE_INGESTED_KIND,
};
use beater_judge::{JudgeBrokerService, KeywordJudgeProvider, SqliteJudgeLedger};
use beater_replay::{plan_replay, ReplayMode};
use beater_sandbox::WasmEvaluatorRuntime;
use beater_schema::{
    AgentSpanKind, AuthContext, CanonicalSpan, EvaluatorLane, ModelRef, RedactionClass,
    ReplayCassette, RunSummary, SpanStatus, TraceView,
};
use beater_search::{SearchIndex, SearchRequest, SearchResponse, TantivySearchIndex};
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_security::{api_key_id_from_secret, verify_webhook, ApiScope};
use beater_store::{
    ArtifactStore, EnvironmentMetadata, MetadataStore, OrganizationMetadata, ProjectMetadata,
    StoreError, StoreResult, TraceStore,
};
use beater_store_memory::InMemoryMetadataStore;
use beater_store_obj::FsArtifactStore;
use beater_store_sql::{SqliteQuotaLimiter, SqliteTraceStore};
use beater_usage::{SqliteUsageLedger, UsageMeter, UsageSummary};
use chrono::{Duration, TimeZone, Utc};
use http::header::RETRY_AFTER;
use http::{Request, StatusCode};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

struct FailNTimesSearchIndex {
    inner: TantivySearchIndex,
    remaining_failures: AtomicUsize,
}

impl FailNTimesSearchIndex {
    fn new(inner: TantivySearchIndex, failures: usize) -> Self {
        Self {
            inner,
            remaining_failures: AtomicUsize::new(failures),
        }
    }
}

struct MutableClock {
    now: Mutex<Timestamp>,
}

impl MutableClock {
    fn new(now: Timestamp) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    fn set(&self, now: Timestamp) {
        *self.now.lock().unwrap_or_else(|err| panic!("{err}")) = now;
    }
}

impl Clock for MutableClock {
    fn now(&self) -> Timestamp {
        self.now
            .lock()
            .unwrap_or_else(|err| panic!("{err}"))
            .to_owned()
    }
}

#[async_trait]
impl SearchIndex for FailNTimesSearchIndex {
    async fn index_spans(&self, spans: &[CanonicalSpan]) -> StoreResult<()> {
        if self.remaining_failures.load(Ordering::SeqCst) > 0 {
            let _ = self.remaining_failures.fetch_update(
                Ordering::SeqCst,
                Ordering::SeqCst,
                |remaining| remaining.checked_sub(1),
            );
            return Err(StoreError::backend("simulated search outage"));
        }
        self.inner.index_spans(spans).await
    }

    async fn search(&self, query: SearchRequest) -> StoreResult<SearchResponse> {
        self.inner.search(query).await
    }
}

#[tokio::test]
async fn api_ingest_store_eval_gate_and_replay_are_integrated() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let search = Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let archive = ParquetTraceArchive::new(tempdir.path().join("archive"))
        .unwrap_or_else(|err| panic!("{err}"));
    let datasets = Arc::new(SqliteDatasetStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let experiments =
        Arc::new(SqliteExperimentStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let gates = Arc::new(SqliteGateStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let human_reviews =
        Arc::new(SqliteHumanReviewStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let calibrations =
        Arc::new(SqliteCalibrationStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let usage = Arc::new(SqliteUsageLedger::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let provider_secrets = Arc::new(
        EncryptedSqliteProviderSecretStore::in_memory(
            SecretKeyring::generated_for_tests().unwrap_or_else(|err| panic!("{err}")),
        )
        .unwrap_or_else(|err| panic!("{err}")),
    );
    let judge_ledger =
        Arc::new(SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let judge_broker = Arc::new(JudgeBrokerService::new(
        provider_secrets.clone(),
        judge_ledger.clone(),
        KeywordJudgeProvider::new(Money::usd_micros(25)),
        Money::usd_micros(100),
    ));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(
        ApiState::with_integrations(ingest, traces, search, archive, datasets, experiments)
            .with_gates(gates)
            .with_human_reviews(human_reviews)
            .with_calibrations(calibrations)
            .with_usage(usage)
            .with_judge(provider_secrets, judge_broker, judge_ledger),
    );

    let request = native_request();
    let body = serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/trace")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let trace: TraceView = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(trace.spans.len(), 1);
    assert_eq!(trace.spans[0].kind, AgentSpanKind::AgentRun);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let downstream: TraceIngestedDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(downstream.completed, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/search/tenant/spans?q=answer&kind=agent.run&status=ok")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let search: SearchResponse =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(search.hits.len(), 1);
    assert_eq!(search.hits[0].span_id, "span");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/online/tenant/project/traces/trace/sampling")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"sample_rate_per_mille":1000,"keep_errors":true,"slow_ms_threshold":null,"high_cost_micros_threshold":null}"#,
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let sampling: SamplingDecision =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(sampling.selected);
    assert_eq!(sampling.reason, SamplingReason::RoutineSampled);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/archive/tenant/project/trace")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let manifest: ArchiveManifest =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(manifest.span_count, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/archive/tenant/project/spans?trace_id=trace&kind=agent.run&status=ok")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let archived: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    let rows = archived["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("archive rows must be an array"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["span_id"], "span");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/datasets/tenant/project")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"failures"}"#))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let dataset: Dataset = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/datasets/tenant/project/{}/cases/from-trace",
                    dataset.dataset_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"trace_id":"trace","span_id":"span","reference":"answer"}"#,
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let dataset_case: DatasetCase =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(dataset_case.input, json!("question"));
    assert_eq!(dataset_case.output, json!("answer"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/datasets/tenant/project/{}/versions",
                    dataset.dataset_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let version: DatasetVersionSnapshot =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(version.cases.len(), 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/datasets/tenant/project/{}/versions/{}/evals/deterministic",
                    dataset.dataset_id.as_str(),
                    version.version_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"evaluator_id":"exact","evaluator_version_id":"exact-v1","agent_release_id":"release-a","kind":{"type":"exact_match"}}"#,
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let report: DatasetEvalReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.result_count, 1);
    assert_eq!(report.aggregate_score, 1.0);
    assert_eq!(
        report.results[0].reproducibility.dataset_version_id,
        version.version_id
    );
    assert_eq!(
        report.results[0].reproducibility.dataset_case_id,
        dataset_case.case_id
    );
    assert!(report.results[0].reproducibility.code_hash.is_some());

    let fixture_secret = "sk-full-stack-dataset-judge";
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/provider-secrets/tenant/project")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "provider": "openai",
                        "display_name": "dataset judge",
                        "secret_value": fixture_secret
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let body_text = String::from_utf8(body.to_vec()).unwrap_or_else(|err| panic!("{err}"));
    assert!(!body_text.contains(fixture_secret));
    let provider_secret: serde_json::Value =
        serde_json::from_str(&body_text).unwrap_or_else(|err| panic!("{err}"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/datasets/tenant/project/{}/versions/{}/evals/judge",
                    dataset.dataset_id.as_str(),
                    version.version_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "evaluator_id": "judge-correctness",
                        "evaluator_version_id": "judge-v1",
                        "agent_release_id": "release-a",
                        "kind": {
                            "type": "llm_judge",
                            "rubric": "correctness",
                            "model": "judge-model"
                        },
                        "provider_secret_id": provider_secret["provider_secret_id"]
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let body_text = String::from_utf8(body.to_vec()).unwrap_or_else(|err| panic!("{err}"));
    assert!(!body_text.contains(fixture_secret));
    let judge_report: DatasetEvalReport =
        serde_json::from_str(&body_text).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(judge_report.result_count, 1);
    assert_eq!(judge_report.aggregate_score, 1.0);
    let judge_result = &judge_report.results[0];
    assert_eq!(judge_result.cost, Some(Money::usd_micros(25)));
    assert_eq!(
        judge_result.reproducibility.judge_model_id.as_deref(),
        Some("judge-model")
    );
    assert_eq!(
        judge_result.reproducibility.judge_provider.as_deref(),
        Some("openai")
    );
    assert!(judge_result.reproducibility.wasi_abi_version.is_none());
    assert_eq!(
        judge_result.reproducibility.judge_parameters["cached"],
        json!(false)
    );
    let usage_summary = get_usage_summary(&app, None).await;
    assert_eq!(
        usage_summary
            .totals
            .get(UsageMeter::JudgeCostMicros.as_str())
            .map(|total| total.quantity),
        Some(25)
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/calibrations/tenant/project/{}/versions/{}",
                    dataset.dataset_id.as_str(),
                    version.version_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "eval_report_id": judge_report.report_id.clone(),
                        "pass_threshold": 0.5
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let calibration: CalibrationReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(calibration.eval_report_id, judge_report.report_id);
    assert_eq!(calibration.sample_count, 1);
    assert_eq!(calibration.observed_agreement, 1.0);
    assert_eq!(calibration.cohen_kappa, 1.0);

    let experiment_body = serde_json::json!({
        "baseline_release_id": "release-baseline",
        "candidate_release_id": "release-candidate",
        "evaluator_id": "exact",
        "evaluator_version_id": "exact-v1",
        "kind": {"type": "exact_match"},
        "gate_policy": {
            "min_sample_size": 1,
            "max_regression": 0.05,
            "alpha": 0.05,
            "comparison_count": 1
        },
        "baseline_outputs": [
            {
                "case_id": dataset_case.case_id.as_str(),
                "output": "wrong"
            }
        ],
        "candidate_outputs": [
            {
                "case_id": dataset_case.case_id.as_str(),
                "output": "answer"
            }
        ]
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/experiments/tenant/project/{}/versions/{}/deterministic",
                    dataset.dataset_id.as_str(),
                    version.version_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&experiment_body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let experiment: ExperimentRunReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(experiment.case_scores.len(), 1);
    assert_eq!(experiment.case_scores[0].baseline_score, 0.0);
    assert_eq!(experiment.case_scores[0].candidate_score, 1.0);
    assert_eq!(experiment.decision, GateDecision::Pass);

    let judge_experiment_body = serde_json::json!({
        "baseline_release_id": "judge-baseline",
        "candidate_release_id": "judge-candidate",
        "evaluator_id": "judge-correctness",
        "evaluator_version_id": "judge-v1",
        "kind": {
            "type": "llm_judge",
            "rubric": "correctness",
            "model": "judge-model"
        },
        "gate_policy": {
            "min_sample_size": 1,
            "max_regression": 0.05,
            "alpha": 0.05,
            "comparison_count": 1
        },
        "baseline_outputs": [
            {
                "case_id": dataset_case.case_id.as_str(),
                "output": "wrong"
            }
        ],
        "candidate_outputs": [
            {
                "case_id": dataset_case.case_id.as_str(),
                "output": "answer"
            }
        ],
        "provider_secret_id": provider_secret["provider_secret_id"]
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/experiments/tenant/project/{}/versions/{}/judge",
                    dataset.dataset_id.as_str(),
                    version.version_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&judge_experiment_body)
                        .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let body_text = String::from_utf8(body.to_vec()).unwrap_or_else(|err| panic!("{err}"));
    assert!(!body_text.contains(fixture_secret));
    let judge_experiment: ExperimentRunReport =
        serde_json::from_str(&body_text).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(judge_experiment.case_scores.len(), 1);
    assert_eq!(judge_experiment.case_scores[0].baseline_score, 0.0);
    assert_eq!(judge_experiment.case_scores[0].candidate_score, 1.0);
    assert_eq!(
        judge_experiment.case_scores[0].baseline_cost,
        Some(Money::usd_micros(25))
    );
    assert_eq!(
        judge_experiment.case_scores[0].candidate_cost,
        Some(Money::usd_micros(0))
    );
    assert_eq!(judge_experiment.case_scores[0].baseline_cached, Some(false));
    assert_eq!(judge_experiment.case_scores[0].candidate_cached, Some(true));
    assert_eq!(judge_experiment.decision, GateDecision::Pass);
    let usage_summary = get_usage_summary(&app, None).await;
    assert_eq!(
        usage_summary
            .totals
            .get(UsageMeter::JudgeCostMicros.as_str())
            .map(|total| total.quantity),
        Some(50)
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/gates/tenant/project")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "gate_id": "release-main",
                        "name": "release main",
                        "dataset_id": dataset.dataset_id.as_str(),
                        "evaluator_version_id": "judge-v1",
                        "inconclusive_policy": "fail"
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let gate: GateDefinition = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(gate.gate_id.as_str(), "release-main");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/gates/tenant/project/release-main/run")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let gate_run: GateRunReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(gate_run.passed);
    assert_eq!(
        gate_run.experiment_run_id,
        judge_experiment.experiment_run_id
    );
    assert_eq!(gate_run.experiment_decision, GateDecision::Pass);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/datasets/tenant/project")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"human-reviewed"}"#))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let review_dataset: Dataset =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/review-queues/tenant/project")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "queue_id": "quality",
                        "name": "quality",
                        "annotation_schema": {
                            "type": "object",
                            "required": ["reference"]
                        }
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let review_queue: ReviewQueue =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(review_queue.queue_id.as_str(), "quality");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/review-queues/tenant/project/quality/tasks/from-trace")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "task_id": "review-task",
                        "trace_id": "trace",
                        "span_id": "span",
                        "dataset_id": review_dataset.dataset_id.as_str(),
                        "priority": 10
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let review_task: ReviewTask =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(review_task.state, ReviewTaskState::Open);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/review-queues/tenant/project/quality/tasks/review-task/annotations")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "annotation_id": "review-annotation",
                        "reviewer_id": "human-a",
                        "verdict": "pass",
                        "payload": {
                            "reference": "answer",
                            "notes": "matches expected output"
                        }
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let review_annotation: ReviewAnnotation =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        review_annotation.annotation_id.as_str(),
        "review-annotation"
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/review-queues/tenant/project/quality/tasks?state=submitted")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let submitted_tasks: Vec<ReviewTask> =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(submitted_tasks.len(), 1);
    assert_eq!(submitted_tasks[0].state, ReviewTaskState::Submitted);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(
                    "/v1/review-queues/tenant/project/quality/tasks/review-task/annotations/review-annotation/promote",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "dataset_id": review_dataset.dataset_id.as_str()
                    }))
                    .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let reviewed_case: DatasetCase =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(reviewed_case.reference, Some(json!("answer")));
    assert_eq!(reviewed_case.source_span_id.as_str(), "span");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/datasets/tenant/project/{}/versions",
                    review_dataset.dataset_id.as_str()
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let reviewed_version: DatasetVersionSnapshot =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(reviewed_version.cases.len(), 1);
    assert_eq!(reviewed_version.cases[0].case_id, reviewed_case.case_id);

    let now = Utc::now();
    let alert_body = serde_json::json!({
        "policy": {
            "policy_id": "low-score",
            "endpoint_url": "https://example.test/beater",
            "signing_secret": "secret",
            "severity": "critical",
            "fire_when_score_at_or_below": 0.5,
            "dedupe_window_seconds": 300,
            "maintenance_windows": []
        },
        "input": {
            "tenant_id": "tenant",
            "project_id": "project",
            "trace_id": "trace",
            "group_key": "eval:exact:trace",
            "title": "Exact eval score dropped",
            "score": 0.1,
            "baseline_score": 1.0,
            "links": {
                "trace_url": "http://localhost/traces/trace",
                "cluster_url": "http://localhost/clusters/cluster",
                "dataset_url": format!("http://localhost/datasets/{}", dataset.dataset_id.as_str()),
                "gate_url": format!("http://localhost/experiments/{}", experiment.experiment_run_id.as_str())
            },
            "now": now
        }
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/alerts/tenant/project/traces/trace/webhook")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&alert_body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let alert: AlertDecision = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(alert.emitted);
    let delivery = alert
        .delivery
        .as_ref()
        .unwrap_or_else(|| panic!("expected alert delivery"));
    assert_eq!(
        delivery.body["links"]["trace_url"],
        "http://localhost/traces/trace"
    );
    assert_eq!(
        delivery.body["links"]["dataset_url"],
        format!("http://localhost/datasets/{}", dataset.dataset_id.as_str())
    );
    let signed_body = serde_json::to_vec(&delivery.body).unwrap_or_else(|err| panic!("{err}"));
    verify_webhook(
        b"secret",
        &signed_body,
        delivery
            .headers
            .get("beater-signature")
            .unwrap_or_else(|| panic!("missing beater-signature")),
        now,
        Duration::seconds(300),
    )
    .unwrap_or_else(|err| panic!("{err}"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/alerts/tenant/project/traces/trace/webhook")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&alert_body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let deduped: AlertDecision =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(!deduped.emitted);
    assert_eq!(deduped.suppressed_reason.as_deref(), Some("dedupe_window"));

    let score = evaluate_deterministic(
        &EvaluatorSpec {
            id: "exact".to_string(),
            lane: EvaluatorLane::DeterministicWasi,
            kind: EvaluatorKind::ExactMatch,
        },
        &EvaluationCase {
            input: json!("question"),
            output: json!("answer"),
            reference: Some(json!("answer")),
            trace: None,
        },
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(score.score, 1.0);

    // Deterministic-scorer authored as a WASI Component Model component (the
    // evaluator runtime no longer accepts bare core modules): exports
    // `score(case-json: string) -> s32`, returning 10000 bp on non-empty input.
    let wasm = wat::parse_str(
        r#"
        (component
          (core module $m
            (memory (export "memory") 1)
            (func (export "score") (param $ptr i32) (param $len i32) (result i32)
              local.get $len
              i32.const 0
              i32.gt_s
              if (result i32)
                i32.const 10000
              else
                i32.const 0
              end)
            (func (export "cabi_realloc")
              (param i32 i32 i32 i32) (result i32)
              i32.const 0))
          (core instance $i (instantiate $m))
          (func $score (param "case-json" string) (result s32)
            (canon lift
              (core func $i "score")
              (memory $i "memory")
              (realloc (func $i "cabi_realloc"))))
          (export "score" (func $score)))
        "#,
    )
    .unwrap_or_else(|err| panic!("{err}"));
    let wasm_score = WasmEvaluatorRuntime::default()
        .evaluate_case_json(
            &wasm,
            &EvaluationCase {
                input: json!("question"),
                output: json!("answer"),
                reference: Some(json!("answer")),
                trace: None,
            },
        )
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(wasm_score.score, 1.0);

    let comparison = compare_paired_scores(
        &[0.8, 0.8, 0.8, 0.8, 0.8],
        &[0.9, 0.9, 0.9, 0.9, 0.9],
        &GatePolicy {
            min_sample_size: 5,
            max_regression: 0.05,
            ..GatePolicy::default()
        },
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(comparison.decision, GateDecision::Pass);

    let replay = plan_replay(
        &ReplayCassette {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            trace_id: TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
            provider_events: 1,
            tool_events: 1,
            memory_events: 1,
            retrieval_events: 1,
            clock_events: 1,
            random_events: 1,
            missing_required_kinds: Vec::new(),
        },
        None,
    );
    assert_eq!(replay.mode, ReplayMode::DeterministicReplay);
}

#[tokio::test]
async fn buffered_ingest_drains_scoped_trace_writes_through_api() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let search = Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(ApiState::with_search(ingest, traces.clone(), search));

    let request = native_request();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native?durability=buffered")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let outcome: beater_ingest::IngestOutcome =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(outcome.ack.accepted_raw, 1);
    assert_eq!(outcome.ack.accepted_spans, 1);

    let cold_trace = traces
        .get_trace(request.scope.tenant_id.clone(), request.trace_id.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(cold_trace.spans.is_empty());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.total_depth, 1);
    assert_eq!(status.trace_write_depth, 1);
    assert_eq!(status.trace_ingested_depth, 0);
    assert!(status.dead_letters.is_empty());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-writes/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let report: TraceWriteDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.consumed, 1);
    assert_eq!(report.written_raw, 1);
    assert_eq!(report.written_spans, 1);
    assert_eq!(report.downstream_published, 1);
    assert_eq!(
        report.trace_ids,
        vec![TraceId::new("trace").unwrap_or_else(|err| panic!("{err}"))]
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let downstream: TraceIngestedDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(downstream.completed, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/trace")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let trace: TraceView = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(trace.spans.len(), 1);
    assert_eq!(trace.spans[0].name, "full-stack agent run");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/search/tenant/spans?q=answer&kind=agent.run&status=ok")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let search: SearchResponse =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(search.hits.len(), 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_write_depth, 0);
    assert_eq!(status.trace_ingested_depth, 0);
}

#[tokio::test]
async fn duplicate_native_ingest_is_reconciled_through_api() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(
        artifacts,
        traces.clone(),
        bus.clone(),
        IngestPolicy::default(),
    );
    let app = router(ApiState::new(ingest, traces.clone()));
    let request = native_request();
    let body = serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}"));

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(body.clone()))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = to_bytes(first.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let first_outcome: IngestOutcome =
        serde_json::from_slice(&first_body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first_outcome.ack.accepted_raw, 1);
    assert_eq!(first_outcome.ack.accepted_spans, 1);
    assert_eq!(first_outcome.ack.duplicate_raw, 0);
    assert_eq!(first_outcome.ack.duplicate_spans, 0);

    let second = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = to_bytes(second.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let second_outcome: IngestOutcome =
        serde_json::from_slice(&second_body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(second_outcome.ack.accepted_raw, 0);
    assert_eq!(second_outcome.ack.accepted_spans, 0);
    assert_eq!(second_outcome.ack.duplicate_raw, 1);
    assert_eq!(second_outcome.ack.duplicate_spans, 1);
    assert_eq!(bus.depth_for_kind(TRACE_INGESTED_KIND).await, Ok(1));

    let trace = traces
        .get_trace(request.scope.tenant_id, request.trace_id)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(trace.spans.len(), 1);
}

#[tokio::test]
async fn project_scoped_archive_routes_do_not_merge_same_trace_id_across_projects() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let search = Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let archive = ParquetTraceArchive::new(tempdir.path().join("archive"))
        .unwrap_or_else(|err| panic!("{err}"));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(ApiState::with_search_and_archive(
        ingest, traces, search, archive,
    ));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/archive/tenant/project/spans?trace_id=trace")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let archive_response: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(
        archive_response["rows"]
            .as_array()
            .unwrap_or_else(|| panic!("archive rows should be an array"))
            .len(),
        0
    );

    let project_request = native_request();
    let mut other_project_request = native_request();
    other_project_request.scope = TenantScope::new(
        TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
        ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}")),
        EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
    );
    other_project_request.span_id = SpanId::new("other-span").unwrap_or_else(|err| panic!("{err}"));
    other_project_request.idempotency_key =
        Some(IdempotencyKey::new("other-project-same-trace").unwrap_or_else(|err| panic!("{err}")));

    for request in [project_request, other_project_request] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/traces/native")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                    ))
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
    }

    for project_id in ["project", "other-project"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/archive/tenant/{project_id}/trace"))
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/v1/archive/tenant/{project_id}/spans?trace_id=trace"
                    ))
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let archive_response: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        let rows = archive_response["rows"]
            .as_array()
            .unwrap_or_else(|| panic!("archive rows should be an array"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["project_id"], project_id);
    }
}

#[tokio::test]
async fn trace_list_span_and_io_endpoints_back_dashboard_reads() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(ApiState::new(ingest, traces));
    let started_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let mut request = native_request();
    request.start_time = Some(started_at);
    request.end_time = Some(started_at + Duration::milliseconds(1000));
    request.model = Some(ModelRef {
        provider: "openai".to_string(),
        name: "gpt-dashboard".to_string(),
    });
    request.cost = Some(Money::usd_micros(200));
    request
        .attributes
        .insert("agent.release_id".to_string(), json!("release-a"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant?project_id=project&environment_id=prod&limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let runs: Page<RunSummary> =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(runs.items.len(), 1);
    assert_eq!(runs.items[0].trace_id, request.trace_id);
    assert_eq!(runs.items[0].span_count, 1);
    assert_eq!(runs.items[0].duration_ms, Some(1000));
    assert_eq!(runs.items[0].total_cost, Some(Money::usd_micros(200)));
    assert_eq!(runs.items[0].models.len(), 1);
    assert_eq!(runs.items[0].models[0].name, "gpt-dashboard");
    assert_eq!(runs.items[0].release_ids, vec!["release-a".to_string()]);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(
                    "/v1/traces/tenant?project_id=project&environment_id=prod&status=ok&kind=agent.run&started_after=2025-12-31T23:59:59Z&started_before=2026-01-01T00:00:01Z&model=gpt-dashboard&release=release-a&min_cost_micros=100&max_cost_micros=300&min_latency_ms=900&max_latency_ms=1100",
                )
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let filtered_runs: Page<RunSummary> =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(filtered_runs.items.len(), 1);
    assert_eq!(filtered_runs.items[0].trace_id, request.trace_id);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant?project_id=other-project&environment_id=prod&limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let wrong_project_runs: Page<RunSummary> =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(wrong_project_runs.items.is_empty());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/trace/span")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let span_json: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(span_json["kind"], json!("agent.run"));
    let span: CanonicalSpan = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(span.name, "full-stack agent run");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/trace/span/io")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let io: serde_json::Value = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(io["input"]["kind"], json!("inline"));
    assert_eq!(io["input"]["value"], json!("question"));
    assert_eq!(io["output"]["kind"], json!("inline"));
    assert_eq!(io["output"]["value"], json!("answer"));

    let mut artifact_request = native_request();
    artifact_request.trace_id =
        TraceId::new("artifact-trace").unwrap_or_else(|err| panic!("{err}"));
    artifact_request.span_id = SpanId::new("artifact-span").unwrap_or_else(|err| panic!("{err}"));
    artifact_request.input = Some(json!("x".repeat(20_000)));
    artifact_request.output = None;
    artifact_request
        .attributes
        .insert("input.value".to_string(), json!("stale attribute"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&artifact_request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/artifact-trace/artifact-span/io")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let artifact_io: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(artifact_io["input"]["kind"], json!("artifact"));
    assert_ne!(artifact_io["input"], json!("stale attribute"));
    assert_eq!(artifact_io["output"]["kind"], json!("missing"));

    let mut sensitive_artifact_request = artifact_request;
    sensitive_artifact_request.trace_id =
        TraceId::new("sensitive-artifact-trace").unwrap_or_else(|err| panic!("{err}"));
    sensitive_artifact_request.span_id =
        SpanId::new("sensitive-artifact-span").unwrap_or_else(|err| panic!("{err}"));
    sensitive_artifact_request.redaction_class = RedactionClass::Sensitive;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&sensitive_artifact_request)
                        .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/sensitive-artifact-trace/sensitive-artifact-span/io")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let redacted_io: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(redacted_io["input"]["kind"], json!("redacted"));
}

#[tokio::test]
async fn dlq_replay_restores_trace_ingested_work_through_api() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let search = Arc::new(FailNTimesSearchIndex::new(
        TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")),
        3,
    ));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(ApiState::with_search(ingest, traces, search));

    let request = native_request();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native?durability=buffered")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-writes/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let report: TraceWriteDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.downstream_published, 1);

    for attempt in 1..=3 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                    .body(Body::empty())
                    .unwrap_or_else(|err| panic!("{err}")),
            )
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        if attempt < 3 {
            assert_eq!(response.status(), StatusCode::OK);
        } else {
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        }
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("{err}"));
        let report: TraceIngestedDrainReport =
            serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(report.consumed, 1);
        assert_eq!(report.failed_work, 1);
        if attempt < 3 {
            assert_eq!(report.retried, 1);
            assert_eq!(report.dead_lettered, 0);
        } else {
            assert_eq!(report.retried, 0);
            assert_eq!(report.dead_lettered, 1);
        }
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_ingested_depth, 0);
    assert_eq!(status.dead_letters.len(), 1);
    let dead_letter = &status.dead_letters[0];
    assert_eq!(dead_letter.message.kind, TRACE_INGESTED_KIND);
    assert_eq!(dead_letter.message.attempts, 3);
    assert!(dead_letter.reason.contains("simulated search outage"));
    let message_id = dead_letter.message.message_id.clone();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/ingest/tenant/project/dead-letters/{message_id}/replay"
                ))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let replay: DeadLetterReplayReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(replay.message_id, message_id);
    assert!(replay.reset_attempts);
    assert!(replay.ack.accepted);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_ingested_depth, 1);
    assert!(status.dead_letters.is_empty());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let report: TraceIngestedDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.completed, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/search/tenant/spans?q=answer&kind=agent.run&status=ok")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let search: SearchResponse =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(search.hits.len(), 1);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_ingested_depth, 0);
    assert!(status.dead_letters.is_empty());
}

#[tokio::test]
async fn malformed_trace_ingested_event_returns_error_and_lands_in_dlq() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(
        artifacts,
        traces.clone(),
        bus.clone(),
        IngestPolicy::default(),
    );
    let app = router(ApiState::new(ingest, traces));
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let mut poison = BusMessage::new(
        tenant.clone(),
        project.clone(),
        IdempotencyKey::new("poison-trace-ingested").unwrap_or_else(|err| panic!("{err}")),
        TRACE_INGESTED_KIND,
        b"not-json".to_vec(),
    );
    poison.max_attempts = 1;
    bus.publish(poison)
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let report: TraceIngestedDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.consumed, 1);
    assert_eq!(report.invalid_messages, 1);
    assert_eq!(report.dead_lettered, 1);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.dead_letters.len(), 1);
    assert_eq!(status.dead_letters[0].message.kind, TRACE_INGESTED_KIND);
    assert!(status.dead_letters[0]
        .reason
        .contains("invalid trace.ingested payload"));
}

#[tokio::test]
async fn reconcile_trace_ingested_recovers_direct_write_after_publish_outage_through_api() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let outage_bus = Arc::new(InMemoryBus::new(0));
    let outage_ingest = IngestService::new(
        artifacts.clone(),
        traces.clone(),
        outage_bus,
        IngestPolicy::default(),
    );
    let outage_app = router(ApiState::new(outage_ingest, traces.clone()));
    let request = native_request();

    let response = outage_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let error: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(error["error"]
        .as_str()
        .unwrap_or_default()
        .contains("capacity 0"));

    let stored_trace = traces
        .get_project_trace(
            request.scope.tenant_id.clone(),
            request.scope.project_id.clone(),
            request.trace_id.clone(),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(stored_trace.spans.len(), 1);

    let healthy_bus = Arc::new(InMemoryBus::new(16));
    let healthy_ingest = IngestService::new(
        artifacts,
        traces.clone(),
        healthy_bus,
        IngestPolicy::default(),
    );
    let search = Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let app = router(ApiState::with_search(healthy_ingest, traces, search));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/traces/trace/reconcile")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let reconcile: TraceIngestedReconcileReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(reconcile.trace_id, request.trace_id);
    assert_eq!(reconcile.span_count, 1);
    assert_eq!(reconcile.downstream_accepted, 1);
    assert_eq!(reconcile.downstream_duplicate, 0);
    assert!(reconcile.downstream_queued);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_ingested_depth, 1);
    assert!(status.dead_letters.is_empty());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/traces/trace/reconcile")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let duplicate_reconcile: TraceIngestedReconcileReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(duplicate_reconcile.downstream_accepted, 0);
    assert_eq!(duplicate_reconcile.downstream_duplicate, 1);
    assert!(duplicate_reconcile.downstream_queued);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_ingested_depth, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tenant/project/trace-ingested/drain?limit=10")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let report: TraceIngestedDrainReport =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(report.consumed, 1);
    assert_eq!(report.completed, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/search/tenant/spans?q=answer&kind=agent.run&status=ok")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let search: SearchResponse =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(search.hits.len(), 1);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/ingest/tenant/project/queue")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status: IngestQueueStatus =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(status.trace_ingested_depth, 0);
    assert!(status.dead_letters.is_empty());
}

#[tokio::test]
async fn buffered_ingest_backpressure_returns_429() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(0));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(ApiState::new(ingest, traces));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native?durability=buffered")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&native_request()).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let error: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(error["status"], json!(429));
    assert!(error["error"]
        .as_str()
        .unwrap_or_default()
        .contains("capacity 0"));
}

#[tokio::test]
async fn api_quota_429_includes_reset_headers() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(
        artifacts,
        traces.clone(),
        bus,
        IngestPolicy {
            per_project_event_quota: Some(1),
            ..IngestPolicy::default()
        },
    );
    let app = router(ApiState::new(ingest, traces));

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&native_request()).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first.status(), StatusCode::OK);

    let mut second_request = native_request();
    second_request.span_id = SpanId::new("quota-span-2").unwrap_or_else(|err| panic!("{err}"));
    second_request.seq = 2;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&second_request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let headers = response.headers().clone();
    assert!(headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .is_some_and(|seconds| seconds >= 0));
    assert_eq!(
        headers
            .get("x-ratelimit-limit")
            .and_then(|value| value.to_str().ok()),
        Some("1")
    );
    assert_eq!(
        headers
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok()),
        Some("0")
    );
    let reset_at = headers
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_else(|| panic!("missing x-ratelimit-reset"));
    assert!(reset_at >= Utc::now().timestamp());

    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let error: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(error["status"], json!(429));
    assert!(error["error"]
        .as_str()
        .unwrap_or_default()
        .contains("quota exceeded"));
}

#[tokio::test]
async fn api_quota_is_shared_across_replicas_and_resets_on_window() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let quota_path = tempdir.path().join("quotas.sqlite");
    let now = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 5)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let reset_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 1, 0)
        .single()
        .unwrap_or_else(|| panic!("valid timestamp"));
    let clock = Arc::new(MutableClock::new(now));
    let policy = IngestPolicy {
        per_project_event_quota: Some(1),
        quota_window_seconds: 60,
        ..IngestPolicy::default()
    };
    let app_a = quota_test_app(
        tempdir.path(),
        "replica-a",
        &quota_path,
        clock.clone(),
        policy.clone(),
    );
    let app_b = quota_test_app(
        tempdir.path(),
        "replica-b",
        &quota_path,
        clock.clone(),
        policy,
    );

    let (status, _, _) = post_native_span(&app_a, "quota-shared-span-1", 1).await;
    assert_eq!(status, StatusCode::OK);

    let (status, headers, error) = post_native_span(&app_b, "quota-shared-span-2", 2).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        headers
            .get("x-ratelimit-limit")
            .and_then(|value| value.to_str().ok()),
        Some("1")
    );
    assert_eq!(
        headers
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok()),
        Some("0")
    );
    let reset_at_header = reset_at.timestamp().to_string();
    assert_eq!(
        headers
            .get("x-ratelimit-reset")
            .and_then(|value| value.to_str().ok()),
        Some(reset_at_header.as_str())
    );
    assert_eq!(error["status"], json!(429));
    assert!(error["error"]
        .as_str()
        .unwrap_or_default()
        .contains("quota exceeded"));

    clock.set(now + Duration::seconds(65));
    let (status, _, _) = post_native_span(&app_b, "quota-shared-span-3", 3).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn hosted_judge_api_uses_byok_refs_cache_and_never_returns_secret() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let search = Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let archive = ParquetTraceArchive::new(tempdir.path().join("judge-archive"))
        .unwrap_or_else(|err| panic!("{err}"));
    let api_keys = Arc::new(SqliteApiKeyStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let provider_secrets = Arc::new(
        EncryptedSqliteProviderSecretStore::in_memory(
            SecretKeyring::generated_for_tests().unwrap_or_else(|err| panic!("{err}")),
        )
        .unwrap_or_else(|err| panic!("{err}")),
    );
    let judge_ledger =
        Arc::new(SqliteJudgeLedger::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let usage = Arc::new(SqliteUsageLedger::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let judge_broker = Arc::new(JudgeBrokerService::new(
        provider_secrets.clone(),
        judge_ledger.clone(),
        KeywordJudgeProvider::new(Money::usd_micros(25)),
        Money::usd_micros(100),
    ));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let app = router(
        ApiState::with_search_and_archive(ingest, traces, search, archive)
            .with_judge(provider_secrets, judge_broker, judge_ledger)
            .with_usage(usage)
            .require_auth(api_keys.clone()),
    );

    let mut admin_scopes = BTreeSet::new();
    admin_scopes.insert(ApiScope::Admin);
    let admin_key = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            scopes: admin_scopes,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let fixture_secret = "sk-hosted-jdg-secret";
    let create_secret_body = json!({
        "provider": "openai",
        "display_name": "hosted judge",
        "secret_value": fixture_secret
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/provider-secrets/tenant/project")
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .header("x-beater-environment-id", "prod")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&create_secret_body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let body_text = String::from_utf8(body.to_vec()).unwrap_or_else(|err| panic!("{err}"));
    assert!(!body_text.contains(fixture_secret));
    let provider_secret: serde_json::Value =
        serde_json::from_str(&body_text).unwrap_or_else(|err| panic!("{err}"));
    assert!(provider_secret.get("secret_value").is_none());
    let provider_secret_id = provider_secret["provider_secret_id"]
        .as_str()
        .unwrap_or_else(|| panic!("provider secret response must include id"))
        .to_string();

    let judge_body = json!({
        "evaluator": {
            "id": "judge-correctness",
            "lane": "judge_broker",
            "kind": {
                "type": "llm_judge",
                "rubric": "correctness",
                "model": "judge-model"
            }
        },
        "case": {
            "input": "question",
            "output": "answer",
            "reference": "answer",
            "trace": null
        },
        "provider_secret_id": provider_secret_id
    });
    let first = post_judge_request(&app, &admin_key.secret, &judge_body).await;
    assert!(!first.to_string().contains(fixture_secret));
    assert_eq!(first["audit"]["cached"], false);
    assert_eq!(first["audit"]["charged_cost"]["amount_micros"], 25);
    assert_eq!(first["remaining_budget"]["amount_micros"], 75);

    let second = post_judge_request(&app, &admin_key.secret, &judge_body).await;
    assert!(!second.to_string().contains(fixture_secret));
    assert_eq!(second["audit"]["cached"], true);
    assert_eq!(second["audit"]["charged_cost"]["amount_micros"], 0);
    assert_eq!(second["remaining_budget"]["amount_micros"], 75);
    assert_eq!(
        first["audit"]["request_hash"],
        second["audit"]["request_hash"]
    );
    assert_eq!(
        first["audit"]["response_hash"],
        second["audit"]["response_hash"]
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/judge/tenant/project/ledger")
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let body_text = String::from_utf8(body.to_vec()).unwrap_or_else(|err| panic!("{err}"));
    assert!(!body_text.contains(fixture_secret));
    let ledger: serde_json::Value =
        serde_json::from_str(&body_text).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(ledger.as_array().map(Vec::len), Some(2));
    let usage_summary = get_usage_summary(&app, Some(&admin_key.secret)).await;
    assert_eq!(
        usage_summary
            .totals
            .get(UsageMeter::JudgeCostMicros.as_str())
            .map(|total| total.quantity),
        Some(25)
    );
}

#[tokio::test]
async fn strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let search = Arc::new(TantivySearchIndex::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let archive = ParquetTraceArchive::new(tempdir.path().join("strict-archive"))
        .unwrap_or_else(|err| panic!("{err}"));
    let api_keys = Arc::new(SqliteApiKeyStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let metadata = seeded_metadata_store().await;
    let audit = Arc::new(SqliteAuditStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(
        artifacts.clone(),
        traces.clone(),
        bus,
        IngestPolicy::default(),
    );
    let app = router(
        ApiState::with_search_and_archive(ingest, traces.clone(), search, archive)
            .with_metadata(metadata)
            .with_audit(audit)
            .require_auth(api_keys.clone()),
    );

    let mut admin_scopes = BTreeSet::new();
    admin_scopes.insert(ApiScope::Admin);
    let admin_key = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            scopes: admin_scopes,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let unauthenticated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&native_request()).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let create_key_body = json!({
        "scopes": ["trace_write", "trace_read"]
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/api-keys/tenant/project/prod")
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&create_key_body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let created_key: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert!(created_key.get("secret_hash").is_none());
    let trace_secret = created_key["secret"]
        .as_str()
        .unwrap_or_else(|| panic!("created key must include one-time secret"))
        .to_string();
    let trace_key_id = api_key_id_from_secret(&trace_secret).unwrap_or_else(|err| panic!("{err}"));
    let mut other_scopes = BTreeSet::new();
    other_scopes.insert(ApiScope::TraceWrite);
    let other_scope_key = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: TenantId::new("other-tenant").unwrap_or_else(|err| panic!("{err}")),
            project_id: ProjectId::new("other-project").unwrap_or_else(|err| panic!("{err}")),
            environment_id: EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}")),
            scopes: other_scopes,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));

    let mut request = native_request();
    request.redaction_class = RedactionClass::Sensitive;
    let mut forged_scopes = BTreeSet::new();
    forged_scopes.insert("admin".to_string());
    request.auth_context = Some(AuthContext {
        api_key_id: Some(ApiKeyId::new("forged").unwrap_or_else(|err| panic!("{err}"))),
        scopes: forged_scopes,
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let missing_scope_headers = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/trace")
                .header("authorization", format!("Bearer {trace_secret}"))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(missing_scope_headers.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant?project_id=project&environment_id=prod&limit=10")
                .header("authorization", format!("Bearer {trace_secret}"))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let runs: Page<RunSummary> =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(runs.items.len(), 1);
    assert_eq!(runs.items[0].trace_id, request.trace_id);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/trace")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-project-id", "project")
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let redacted_trace: TraceView =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(redacted_trace.spans[0].raw_ref.uri, "artifact://redacted");
    assert_eq!(
        redacted_trace.spans[0].attributes["input.value"],
        json!("[redacted]")
    );
    assert_eq!(
        redacted_trace.spans[0].attributes["output.value"],
        json!("[redacted]")
    );

    let missing_span_scope_headers = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/trace/span")
                .header("authorization", format!("Bearer {trace_secret}"))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(missing_span_scope_headers.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/trace/span")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-project-id", "project")
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let span_json: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(span_json["kind"], json!("agent.run"));
    assert_eq!(span_json["attributes"]["input.value"], json!("[redacted]"));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/spans/tenant/trace/span/io")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-project-id", "project")
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let span_io: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(span_io["input"]["kind"], json!("inline"));
    assert_eq!(span_io["input"]["value"], json!("[redacted]"));

    let denied_unmask = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/trace?unmask=true&reason=incident-123")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-project-id", "project")
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(denied_unmask.status(), StatusCode::FORBIDDEN);

    let create_unmask_key_body = json!({
        "scopes": ["trace_read", "pii_unmask"]
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/api-keys/tenant/project/prod")
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&create_unmask_key_body)
                        .unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let unmask_key: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    let unmask_secret = unmask_key["secret"]
        .as_str()
        .unwrap_or_else(|| panic!("unmask key must include one-time secret"))
        .to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/traces/tenant/trace?unmask=true&reason=incident-123")
                .header("authorization", format!("Bearer {unmask_secret}"))
                .header("x-beater-project-id", "project")
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let unmasked_trace: TraceView =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    assert_ne!(unmasked_trace.spans[0].raw_ref.uri, "artifact://redacted");
    assert_eq!(
        unmasked_trace.spans[0].attributes["input.value"],
        json!("question")
    );
    assert_eq!(
        unmasked_trace.spans[0].attributes["output.value"],
        json!("answer")
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/audit/tenant/project")
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let audit_events: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    let audit_events = audit_events
        .as_array()
        .unwrap_or_else(|| panic!("audit events response must be an array"));
    assert_eq!(audit_events.len(), 2);
    assert_eq!(audit_events[0]["action"], "pii_unmask");
    assert_eq!(audit_events[0]["outcome"], "denied");
    assert_eq!(audit_events[1]["outcome"], "allowed");
    assert_eq!(audit_events[1]["reason"], "incident-123");
    assert_eq!(audit_events[1]["attributes"]["sensitive_ref_count"], 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/archive/tenant/project/trace")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/archive/tenant/project/spans?trace_id=trace")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/archive/tenant/project/spans?trace_id=trace&environment_id=dev")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("x-beater-environment-id", "prod")
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let trace = traces
        .get_trace(
            TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}")),
            TraceId::new("trace").unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let raw_bytes = artifacts
        .get_bytes(&trace.spans[0].raw_ref)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let stored_request: NativeIngestRequest =
        serde_json::from_slice(&raw_bytes).unwrap_or_else(|err| panic!("{err}"));
    let stored_auth = stored_request
        .auth_context
        .unwrap_or_else(|| panic!("server should write auth context"));
    assert_eq!(stored_auth.api_key_id, Some(trace_key_id.clone()));
    assert!(stored_auth.scopes.contains("trace:write"));

    let loaded_key = api_keys
        .get_key(trace_key_id.clone())
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("created key should be persisted"));
    assert!(loaded_key.last_used_at.is_some());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/api-keys/tenant/project/prod/{}/revoke",
                    other_scope_key.record.api_key_id.as_str()
                ))
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let other_scope_key = api_keys
        .get_key(other_scope_key.record.api_key_id)
        .await
        .unwrap_or_else(|err| panic!("{err}"))
        .unwrap_or_else(|| panic!("cross-scope key should still exist"));
    assert!(other_scope_key.active);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1/api-keys/tenant/project/prod/{}/revoke",
                    trace_key_id.as_str()
                ))
                .header("authorization", format!("Bearer {}", admin_key.secret))
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("authorization", format!("Bearer {trace_secret}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&native_request()).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn post_judge_request(
    app: &Router,
    admin_secret: &str,
    body: &serde_json::Value,
) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/judge/tenant/project/evaluate")
                .header("authorization", format!("Bearer {admin_secret}"))
                .header("x-beater-environment-id", "prod")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(body).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"))
}

fn quota_test_app(
    root: &std::path::Path,
    name: &str,
    quota_path: &std::path::Path,
    clock: Arc<dyn Clock>,
    policy: IngestPolicy,
) -> Router {
    let artifacts = Arc::new(
        FsArtifactStore::new(root.join(format!("artifacts-{name}")))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(16));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, policy)
        .with_quota_limiter(Arc::new(
            SqliteQuotaLimiter::open(quota_path).unwrap_or_else(|err| panic!("{err}")),
        ))
        .with_clock(clock);
    router(ApiState::new(ingest, traces))
}

async fn post_native_span(
    app: &Router,
    span_id: &str,
    seq: u64,
) -> (StatusCode, http::HeaderMap, serde_json::Value) {
    let mut request = native_request();
    request.span_id = SpanId::new(span_id).unwrap_or_else(|err| panic!("{err}"));
    request.seq = seq;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces/native")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&request).unwrap_or_else(|err| panic!("{err}")),
                ))
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let body = serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"));
    (status, headers, body)
}

async fn get_usage_summary(app: &Router, admin_secret: Option<&str>) -> UsageSummary {
    let mut builder = Request::builder()
        .method("GET")
        .uri("/v1/usage/tenant/project");
    if let Some(secret) = admin_secret {
        builder = builder
            .header("authorization", format!("Bearer {secret}"))
            .header("x-beater-environment-id", "prod");
    }
    let response = app
        .clone()
        .oneshot(
            builder
                .body(Body::empty())
                .unwrap_or_else(|err| panic!("{err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    serde_json::from_slice(&body).unwrap_or_else(|err| panic!("{err}"))
}

async fn seeded_metadata_store() -> Arc<InMemoryMetadataStore> {
    let metadata = Arc::new(InMemoryMetadataStore::new());
    let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
    let organization = OrganizationId::new("org").unwrap_or_else(|err| panic!("{err}"));
    let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));
    let environment = EnvironmentId::new("prod").unwrap_or_else(|err| panic!("{err}"));
    let created_at = Utc::now();
    metadata
        .put_organization(OrganizationMetadata {
            tenant_id: tenant.clone(),
            organization_id: organization.clone(),
            display_name: "Org".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    metadata
        .put_project(ProjectMetadata {
            tenant_id: tenant.clone(),
            organization_id: organization,
            project_id: project.clone(),
            display_name: "Project".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    metadata
        .put_environment(EnvironmentMetadata {
            tenant_id: tenant,
            project_id: project,
            environment_id: environment,
            display_name: "Production".to_string(),
            created_at,
        })
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    metadata
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
        name: "full-stack agent run".to_string(),
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
