use anyhow::Context;
use beater_alerts::{
    decide_trace_sampling, AlertEngine, AlertInput, AlertLinks, AlertPolicy, AlertSeverity,
    OnlineSamplingPolicy,
};
use beater_audit::{
    pii_unmask_event, AuditOutcome, AuditStore, PiiUnmaskAuditInput, SqliteAuditStore,
};
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
use beater_bus::{BusMessage, DurableBus, SqliteDurableBus};
use beater_calibration::{
    calibrate_eval_report, CalibrationPolicy, CalibrationStore, SqliteCalibrationStore,
};
use beater_core::{
    AgentReleaseId, AnnotationId, ApiKeyId, DatasetId, DatasetVersionId, EnvironmentId,
    EvaluatorVersionId, ExperimentRunId, GateId, IdempotencyKey, Money, Page, PageRequest,
    ProjectId, ProviderSecretId, ReviewQueueId, ReviewTaskId, SpanId, TenantId, TenantScope,
    TraceId,
};
use beater_datasets::{
    evaluate_dataset_version, evaluate_dataset_version_with_judge, promote_trace_span_to_case,
    DatasetCase, DatasetEvalSpec, DatasetJudgeEvalSpec, DatasetStore, SqliteDatasetStore,
};
use beater_eval::{
    compare_paired_scores, EvaluationCase, EvaluatorKind, EvaluatorSpec, ExperimentComparison,
    GateDecision, GatePolicy, StatisticalTest,
};
use beater_experiments::{
    run_agent_experiment, run_deterministic_experiment, run_judge_experiment, AgentExperimentSpec,
    CaseOutputOverride, ExperimentRunReport, ExperimentRunSpec, ExperimentStore,
    JudgeExperimentRunSpec, ReferenceAgentAdapter, SqliteExperimentStore, StaticAgentAdapter,
};
use beater_gates::{run_gate, GateDefinition, GateStore, InconclusivePolicy, SqliteGateStore};
use beater_human::{
    promote_review_annotation_to_dataset_case, CreateReviewQueueRequest, EnqueueReviewTaskRequest,
    HumanReviewStore, ReviewVerdict, SqliteHumanReviewStore, SubmitAnnotationRequest,
};
use beater_ingest::{
    anonymous_auth_context, smoke_trace, IngestPolicy, IngestService, NativeIngestRequest,
};
use beater_judge::{
    JudgeBrokerRequest, JudgeBrokerService, JudgeLedgerStore, KeywordJudgeProvider,
    SqliteJudgeLedger,
};
use beater_otlp::{encode_export_trace_request, export_to_raw_trace_ingest_request};
use beater_replay::{
    execute_replay, ReplayEvent, ReplayEventKind, ReplayScenario, ReplayStep, ReplayStore,
    SqliteReplayStore,
};
use beater_schema::{
    AgentSpanKind, CanonicalTraceBatch, EvaluatorLane, RawEnvelope, RedactionClass, RunFilter,
    RunSummary, SpanFilter, SpanStatus, SpanSummary, TraceView, WriteAck,
};
use beater_secrets::{
    EncryptedSqliteProviderSecretStore, ProviderSecretStore, PutProviderSecretRequest,
    SecretKeyring,
};
use beater_security::ApiScope;
use beater_store::{StoreError, TraceStore};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use beater_usage::{
    judge_usage_from_outcome, record_usage_batch, SqliteUsageLedger, UsageLedgerStore, UsageMeter,
};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, InstrumentationScope, KeyValue};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{
    span, status, ResourceSpans, ScopeSpans, Span, Status,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tonic::metadata::MetadataValue;
use tonic::Request as TonicRequest;

#[derive(Debug, Parser)]
#[command(name = "beaterctl", about = "Beater local development and CI helper")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Smoke {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        #[arg(long)]
        http_url: Option<String>,
        #[arg(long)]
        otlp_grpc_url: Option<String>,
        #[arg(long, default_value = "demo")]
        tenant_id: String,
        #[arg(long, default_value = "demo")]
        project_id: String,
        #[arg(long, default_value = "local")]
        environment_id: String,
        #[arg(long, default_value_t = 5000)]
        timeout_ms: u64,
    },
    Gate {
        #[arg(long, value_delimiter = ',')]
        baseline: Vec<f64>,
        #[arg(long, value_delimiter = ',')]
        candidate: Vec<f64>,
        #[arg(long, default_value_t = 10)]
        min_sample_size: usize,
        #[arg(long, default_value_t = 0.0)]
        max_regression: f64,
    },
    JudgeBudget {
        #[arg(long)]
        remaining_micros: i64,
    },
    JudgeFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    BusFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    IngestOutageFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    ReplayFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    EvalFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    JudgeDatasetFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    ExperimentFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    JudgeExperimentFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    GatePolicyCreate {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        project_id: String,
        #[arg(long)]
        gate_id: String,
        #[arg(long, default_value = "main")]
        name: String,
        #[arg(long)]
        dataset_id: Option<String>,
        #[arg(long)]
        evaluator_version_id: Option<String>,
        #[arg(long, value_enum, default_value_t = InconclusivePolicyArg::Fail)]
        inconclusive_policy: InconclusivePolicyArg,
    },
    GateRun {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        project_id: String,
        #[arg(long)]
        gate_id: String,
        #[arg(long)]
        experiment_run_id: Option<String>,
    },
    GateRunFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    ReviewFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    CalibrationFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    UsageFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    AuditFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    AgentFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    AlertFixture {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    ApiKeyCreate {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        #[arg(long)]
        tenant_id: String,
        #[arg(long)]
        project_id: String,
        #[arg(long)]
        environment_id: String,
        #[arg(long, value_delimiter = ',', default_value = "admin")]
        scopes: Vec<ApiScopeArg>,
    },
    ApiKeyRevoke {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        #[arg(long)]
        api_key_id: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ApiScopeArg {
    #[value(name = "trace-write")]
    TraceWrite,
    #[value(name = "trace-read")]
    TraceRead,
    #[value(name = "dataset-write")]
    DatasetWrite,
    #[value(name = "eval-run")]
    EvalRun,
    #[value(name = "pii-unmask")]
    PiiUnmask,
    Admin,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InconclusivePolicyArg {
    Pass,
    Fail,
}

impl From<ApiScopeArg> for ApiScope {
    fn from(value: ApiScopeArg) -> Self {
        match value {
            ApiScopeArg::TraceWrite => Self::TraceWrite,
            ApiScopeArg::TraceRead => Self::TraceRead,
            ApiScopeArg::DatasetWrite => Self::DatasetWrite,
            ApiScopeArg::EvalRun => Self::EvalRun,
            ApiScopeArg::PiiUnmask => Self::PiiUnmask,
            ApiScopeArg::Admin => Self::Admin,
        }
    }
}

impl From<InconclusivePolicyArg> for InconclusivePolicy {
    fn from(value: InconclusivePolicyArg) -> Self {
        match value {
            InconclusivePolicyArg::Pass => Self::Pass,
            InconclusivePolicyArg::Fail => Self::Fail,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Smoke {
            data_dir,
            http_url,
            otlp_grpc_url,
            tenant_id,
            project_id,
            environment_id,
            timeout_ms,
        } => {
            let output = if let Some(http_url) = http_url {
                run_remote_smoke(
                    http_url,
                    otlp_grpc_url,
                    tenant_id,
                    project_id,
                    environment_id,
                    timeout_ms,
                )
                .await?
            } else {
                run_local_smoke(data_dir).await?
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Gate {
            baseline,
            candidate,
            min_sample_size,
            max_regression,
        } => {
            let comparison = compare_paired_scores(
                &baseline,
                &candidate,
                &GatePolicy {
                    min_sample_size,
                    max_regression,
                    ..GatePolicy::default()
                },
            )
            .context("compare paired scores")?;
            println!("{}", serde_json::to_string_pretty(&comparison)?);
        }
        Command::JudgeBudget { remaining_micros } => {
            let budget = Money::usd_micros(remaining_micros);
            println!("{}", serde_json::to_string_pretty(&budget)?);
        }
        Command::JudgeFixture { data_dir } => {
            let secret_keyring = SecretKeyring::load_or_create_local_file(
                data_dir.join("provider-secrets.key"),
                "local-v1",
            )?;
            let secrets = EncryptedSqliteProviderSecretStore::open(
                data_dir.join("provider-secrets.sqlite"),
                secret_keyring,
            )?;
            let ledger = SqliteJudgeLedger::open(data_dir.join("judge.sqlite"))?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let fixture_secret = "sk-local-fixture-secret";
            let metadata = secrets
                .put_secret(PutProviderSecretRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    provider: "openai".to_string(),
                    display_name: "local judge fixture".to_string(),
                    secret_value: fixture_secret.to_string(),
                })
                .await
                .context("create provider secret")?;
            let broker = JudgeBrokerService::new(
                secrets,
                ledger.clone(),
                KeywordJudgeProvider::new(Money::usd_micros(25)),
                Money::usd_micros(100),
            );
            let first = broker
                .evaluate(judge_fixture_request(
                    &tenant,
                    &project,
                    metadata.provider_secret_id.clone(),
                ))
                .await
                .context("run first judge evaluation")?;
            let second = broker
                .evaluate(judge_fixture_request(
                    &tenant,
                    &project,
                    metadata.provider_secret_id.clone(),
                ))
                .await
                .context("run cached judge evaluation")?;
            let ledger = ledger
                .list_records(tenant.clone(), project.clone())
                .await
                .context("list judge ledger records")?;
            let output = serde_json::json!({
                "provider_secret": metadata,
                "first": first,
                "second": second,
                "ledger": ledger
            });
            let output = serde_json::to_string_pretty(&output)?;
            if output.contains(fixture_secret) {
                anyhow::bail!("judge fixture output leaked provider secret material");
            }
            assert_secret_not_in_judge_fixture_files(&data_dir, fixture_secret)?;
            println!("{output}");
        }
        Command::BusFixture { data_dir } => {
            let path = data_dir.join("bus.sqlite");
            let bus = SqliteDurableBus::open(&path, 128)?;
            let mut poison = BusMessage::new(
                TenantId::new("demo")?,
                ProjectId::new("demo")?,
                IdempotencyKey::new("bus-fixture-poison")?,
                "fixture.poison",
                b"poison".to_vec(),
            );
            poison.max_attempts = 2;
            bus.publish(poison)
                .await
                .context("publish poison message")?;
            drop(bus);

            let bus = SqliteDurableBus::open(&path, 128)?;
            let mut batch = bus
                .consume_batch(1)
                .await
                .context("consume first attempt")?;
            let first = batch
                .pop()
                .ok_or_else(|| anyhow::anyhow!("expected persisted bus message"))?;
            bus.retry_or_dlq(first, "fixture failure".to_string())
                .await
                .context("retry poison message")?;
            drop(bus);

            let bus = SqliteDurableBus::open(&path, 128)?;
            let mut batch = bus
                .consume_batch(1)
                .await
                .context("consume retry attempt")?;
            let second = batch
                .pop()
                .ok_or_else(|| anyhow::anyhow!("expected retried bus message"))?;
            bus.retry_or_dlq(second, "fixture failure".to_string())
                .await
                .context("move poison message to dlq")?;
            let dlq = bus.dlq().await.context("read bus dlq")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "depth": bus.depth().await?,
                    "dlq_len": dlq.len(),
                    "dlq": dlq
                }))?
            );
        }
        Command::IngestOutageFixture { data_dir } => {
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let environment = EnvironmentId::new("prod")?;
            let trace_id = TraceId::new("ingest-outage-trace")?;
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let bus_path = data_dir.join("bus.sqlite");
            let request = NativeIngestRequest {
                scope: TenantScope::new(tenant.clone(), project.clone(), environment),
                trace_id: trace_id.clone(),
                span_id: SpanId::new("root")?,
                parent_span_id: None,
                seq: 1,
                kind: AgentSpanKind::AgentRun,
                name: "ingest outage fixture".to_string(),
                status: SpanStatus::Ok,
                start_time: Some(Utc::now()),
                end_time: Some(Utc::now()),
                model: None,
                cost: None,
                tokens: None,
                input: Some(json!("accepted during trace store outage")),
                output: Some(json!("recovered from durable queue")),
                attributes: BTreeMap::new(),
                redaction_class: RedactionClass::Internal,
                idempotency_key: Some(IdempotencyKey::new("ingest-outage-fixture")?),
                auth_context: None,
            };

            let outage_bus = Arc::new(SqliteDurableBus::open(&bus_path, 128)?);
            let outage_service = IngestService::new(
                artifacts.clone(),
                Arc::new(UnavailableTraceStore),
                outage_bus.clone(),
                IngestPolicy::default(),
            );
            let buffered = outage_service
                .buffer_native(request)
                .await
                .context("buffer native trace while trace store is unavailable")?;
            let retry_report = outage_service
                .drain_trace_writes_for(&tenant, &project, 10)
                .await
                .context("drain trace writes during simulated outage")?;
            if retry_report.retried != 1 {
                anyhow::bail!("expected one retried trace write during outage");
            }
            drop(outage_service);
            drop(outage_bus);

            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let before_recovery = traces
                .get_trace(tenant.clone(), trace_id.clone())
                .await
                .context("read trace before recovery")?;
            if !before_recovery.spans.is_empty() {
                anyhow::bail!("trace store should be empty before recovery drain");
            }

            let recovery_bus = Arc::new(SqliteDurableBus::open(&bus_path, 128)?);
            let recovered = IngestService::new(
                artifacts,
                traces.clone(),
                recovery_bus,
                IngestPolicy::default(),
            );
            let recovery_report = recovered
                .drain_trace_writes_for(&tenant, &project, 10)
                .await
                .context("drain trace writes after store recovery")?;
            let trace = traces
                .get_trace(tenant.clone(), trace_id.clone())
                .await
                .context("read recovered trace")?;
            if trace.spans.len() != 1 {
                anyhow::bail!("expected one recovered span, got {}", trace.spans.len());
            }
            let queue_status = recovered
                .queue_status(tenant.clone(), project.clone())
                .await
                .context("read ingest queue status")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "buffered": buffered,
                    "retry_report": retry_report,
                    "recovery_report": recovery_report,
                    "trace_span_count": trace.spans.len(),
                    "trace_name": trace.spans[0].name,
                    "queue_status": queue_status
                }))?
            );
        }
        Command::ReplayFixture { data_dir } => {
            let path = data_dir.join("replay.sqlite");
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace = TraceId::new("smoke-trace")?;
            let store = SqliteReplayStore::open(&path)?;
            let events = replay_fixture_events(&tenant, &project, &trace)?;
            for event in events.clone() {
                store
                    .put_event(event)
                    .await
                    .context("record replay event")?;
            }
            drop(store);

            let store = SqliteReplayStore::open(&path)?;
            let events = store
                .list_events(tenant.clone(), project.clone(), trace.clone())
                .await
                .context("load replay events")?;
            let cassette = store
                .cassette(tenant.clone(), project.clone(), trace.clone())
                .await
                .context("build replay cassette")?;
            let report = execute_replay(
                &cassette,
                &events,
                ReplayScenario {
                    tenant_id: tenant,
                    project_id: project,
                    trace_id: trace,
                    steps: events
                        .iter()
                        .map(|event| ReplayStep {
                            seq: event.seq,
                            kind: event.kind.clone(),
                            request: event.request.clone(),
                        })
                        .collect(),
                    fork_after_seq: None,
                },
            )
            .context("execute deterministic replay")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "cassette": cassette,
                    "report": report
                }))?
            );
        }
        Command::EvalFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace_id = TraceId::new("smoke-trace")?;
            let span_id = SpanId::new("smoke-root")?;
            let trace = traces
                .get_trace(tenant.clone(), trace_id)
                .await
                .context("read smoke trace")?;
            let dataset = datasets
                .create_dataset(tenant.clone(), project.clone(), "smoke-fixture".to_string())
                .await
                .context("create smoke dataset")?;
            let case = promote_trace_span_to_case(
                tenant.clone(),
                project.clone(),
                dataset.dataset_id.clone(),
                &trace,
                Some(span_id),
                Some(json!({ "answer": "world" })),
            )
            .context("promote smoke trace to dataset")?;
            datasets
                .put_case(case)
                .await
                .context("store smoke dataset case")?;
            let version = datasets
                .create_version(tenant.clone(), project.clone(), dataset.dataset_id, None)
                .await
                .context("create smoke dataset version")?;
            let report = evaluate_dataset_version(
                &version,
                DatasetEvalSpec {
                    evaluator: EvaluatorSpec {
                        id: "exact".to_string(),
                        lane: EvaluatorLane::DeterministicWasi,
                        kind: EvaluatorKind::ExactMatch,
                    },
                    evaluator_version_id: EvaluatorVersionId::new("exact-v1")?,
                    agent_release_id: AgentReleaseId::new("smoke-release")?,
                    prompt_version_id: None,
                    code_hash: None,
                    wasm_hash: None,
                },
            )
            .context("run smoke dataset eval")?;
            let report = datasets
                .write_eval_report(report)
                .await
                .context("store smoke dataset eval report")?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::JudgeDatasetFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let secret_keyring = SecretKeyring::load_or_create_local_file(
                data_dir.join("provider-secrets.key"),
                "local-v1",
            )?;
            let secrets = EncryptedSqliteProviderSecretStore::open(
                data_dir.join("provider-secrets.sqlite"),
                secret_keyring,
            )?;
            let ledger = SqliteJudgeLedger::open(data_dir.join("judge.sqlite"))?;
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace = traces
                .get_trace(tenant.clone(), TraceId::new("smoke-trace")?)
                .await
                .context("read smoke trace")?;
            let dataset = datasets
                .create_dataset(tenant.clone(), project.clone(), "judge-fixture".to_string())
                .await
                .context("create judge dataset")?;
            let case = promote_trace_span_to_case(
                tenant.clone(),
                project.clone(),
                dataset.dataset_id.clone(),
                &trace,
                Some(SpanId::new("smoke-root")?),
                Some(json!({ "answer": "world" })),
            )
            .context("promote smoke trace to judge dataset")?;
            datasets
                .put_case(case)
                .await
                .context("store judge dataset case")?;
            let version = datasets
                .create_version(tenant.clone(), project.clone(), dataset.dataset_id, None)
                .await
                .context("create judge dataset version")?;
            let fixture_secret = "sk-local-dataset-judge-secret";
            let secret_metadata = secrets
                .put_secret(PutProviderSecretRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    provider: "openai".to_string(),
                    display_name: "local dataset judge fixture".to_string(),
                    secret_value: fixture_secret.to_string(),
                })
                .await
                .context("create judge provider secret")?;
            let broker = JudgeBrokerService::new(
                secrets,
                ledger.clone(),
                KeywordJudgeProvider::new(Money::usd_micros(25)),
                Money::usd_micros(100),
            );
            let report = evaluate_dataset_version_with_judge(
                &version,
                DatasetJudgeEvalSpec {
                    eval: DatasetEvalSpec {
                        evaluator: EvaluatorSpec {
                            id: "judge-correctness".to_string(),
                            lane: EvaluatorLane::JudgeBroker,
                            kind: EvaluatorKind::LlmJudge {
                                rubric: "correctness".to_string(),
                                model: "judge-model".to_string(),
                            },
                        },
                        evaluator_version_id: EvaluatorVersionId::new("judge-v1")?,
                        agent_release_id: AgentReleaseId::new("smoke-release")?,
                        prompt_version_id: None,
                        code_hash: None,
                        wasm_hash: None,
                    },
                    provider_secret_id: secret_metadata.provider_secret_id.clone(),
                },
                &broker,
            )
            .await
            .context("run judge dataset eval")?;
            let report = datasets
                .write_eval_report(report)
                .await
                .context("store judge dataset eval report")?;
            let ledger_records = ledger
                .list_records(tenant, project)
                .await
                .context("list judge dataset ledger")?;
            let output = serde_json::to_string_pretty(&serde_json::json!({
                "provider_secret": secret_metadata,
                "report": report,
                "ledger": ledger_records
            }))?;
            if output.contains(fixture_secret) {
                anyhow::bail!("judge dataset fixture output leaked provider secret material");
            }
            assert_secret_not_in_judge_fixture_files(&data_dir, fixture_secret)?;
            println!("{output}");
        }
        Command::ExperimentFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let experiments = SqliteExperimentStore::open(data_dir.join("experiments.sqlite"))?;
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace = traces
                .get_trace(tenant.clone(), TraceId::new("smoke-trace")?)
                .await
                .context("read smoke trace")?;
            let dataset = datasets
                .create_dataset(
                    tenant.clone(),
                    project.clone(),
                    "experiment-fixture".to_string(),
                )
                .await
                .context("create experiment dataset")?;
            let case = promote_trace_span_to_case(
                tenant.clone(),
                project.clone(),
                dataset.dataset_id.clone(),
                &trace,
                Some(SpanId::new("smoke-root")?),
                Some(json!({ "answer": "world" })),
            )
            .context("promote smoke trace to experiment dataset")?;
            let case = datasets
                .put_case(case)
                .await
                .context("store experiment dataset case")?;
            let version = datasets
                .create_version(tenant, project, dataset.dataset_id, None)
                .await
                .context("create experiment dataset version")?;
            let report = run_deterministic_experiment(
                &version,
                ExperimentRunSpec {
                    baseline_release_id: AgentReleaseId::new("baseline-release")?,
                    candidate_release_id: AgentReleaseId::new("candidate-release")?,
                    evaluator: EvaluatorSpec {
                        id: "exact".to_string(),
                        lane: EvaluatorLane::DeterministicWasi,
                        kind: EvaluatorKind::ExactMatch,
                    },
                    evaluator_version_id: EvaluatorVersionId::new("exact-v1")?,
                    gate_policy: GatePolicy {
                        min_sample_size: 1,
                        max_regression: 0.05,
                        ..GatePolicy::default()
                    },
                    baseline_outputs: vec![CaseOutputOverride {
                        case_id: case.case_id.clone(),
                        output: json!({ "answer": "nope" }),
                        trace: None,
                    }],
                    candidate_outputs: vec![CaseOutputOverride {
                        case_id: case.case_id,
                        output: json!({ "answer": "world" }),
                        trace: None,
                    }],
                },
            )
            .context("run deterministic experiment")?;
            let report = experiments
                .write_run(report)
                .await
                .context("store deterministic experiment")?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::JudgeExperimentFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let experiments = SqliteExperimentStore::open(data_dir.join("experiments.sqlite"))?;
            let secret_keyring = SecretKeyring::load_or_create_local_file(
                data_dir.join("provider-secrets.key"),
                "local-v1",
            )?;
            let secrets = EncryptedSqliteProviderSecretStore::open(
                data_dir.join("provider-secrets.sqlite"),
                secret_keyring,
            )?;
            let ledger = SqliteJudgeLedger::open(data_dir.join("judge.sqlite"))?;
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace = traces
                .get_trace(tenant.clone(), TraceId::new("smoke-trace")?)
                .await
                .context("read smoke trace")?;
            let dataset = datasets
                .create_dataset(
                    tenant.clone(),
                    project.clone(),
                    "judge-experiment-fixture".to_string(),
                )
                .await
                .context("create judge experiment dataset")?;
            let case = promote_trace_span_to_case(
                tenant.clone(),
                project.clone(),
                dataset.dataset_id.clone(),
                &trace,
                Some(SpanId::new("smoke-root")?),
                Some(json!({ "answer": "world" })),
            )
            .context("promote smoke trace to judge experiment dataset")?;
            let case = datasets
                .put_case(case)
                .await
                .context("store judge experiment dataset case")?;
            let version = datasets
                .create_version(tenant.clone(), project.clone(), dataset.dataset_id, None)
                .await
                .context("create judge experiment dataset version")?;
            let fixture_secret = "sk-local-experiment-judge-secret";
            let secret_metadata = secrets
                .put_secret(PutProviderSecretRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    provider: "openai".to_string(),
                    display_name: "local experiment judge fixture".to_string(),
                    secret_value: fixture_secret.to_string(),
                })
                .await
                .context("create experiment judge provider secret")?;
            let broker = JudgeBrokerService::new(
                secrets,
                ledger.clone(),
                KeywordJudgeProvider::new(Money::usd_micros(25)),
                Money::usd_micros(200),
            );
            let report = run_judge_experiment(
                &version,
                JudgeExperimentRunSpec {
                    experiment: ExperimentRunSpec {
                        baseline_release_id: AgentReleaseId::new("baseline-judge")?,
                        candidate_release_id: AgentReleaseId::new("candidate-judge")?,
                        evaluator: EvaluatorSpec {
                            id: "judge-correctness".to_string(),
                            lane: EvaluatorLane::JudgeBroker,
                            kind: EvaluatorKind::LlmJudge {
                                rubric: "correctness".to_string(),
                                model: "judge-model".to_string(),
                            },
                        },
                        evaluator_version_id: EvaluatorVersionId::new("judge-v1")?,
                        gate_policy: GatePolicy {
                            min_sample_size: 1,
                            max_regression: 0.05,
                            ..GatePolicy::default()
                        },
                        baseline_outputs: vec![CaseOutputOverride {
                            case_id: case.case_id.clone(),
                            output: json!({ "answer": "nope" }),
                            trace: None,
                        }],
                        candidate_outputs: vec![CaseOutputOverride {
                            case_id: case.case_id,
                            output: json!({ "answer": "world" }),
                            trace: None,
                        }],
                    },
                    provider_secret_id: secret_metadata.provider_secret_id.clone(),
                },
                &broker,
            )
            .await
            .context("run judge experiment")?;
            let report = experiments
                .write_run(report)
                .await
                .context("store judge experiment")?;
            let ledger_records = ledger
                .list_records(tenant, project)
                .await
                .context("list judge experiment ledger")?;
            let output = serde_json::to_string_pretty(&serde_json::json!({
                "provider_secret": secret_metadata,
                "report": report,
                "ledger": ledger_records
            }))?;
            if output.contains(fixture_secret) {
                anyhow::bail!("judge experiment fixture output leaked provider secret material");
            }
            assert_secret_not_in_judge_fixture_files(&data_dir, fixture_secret)?;
            println!("{output}");
        }
        Command::GatePolicyCreate {
            data_dir,
            tenant_id,
            project_id,
            gate_id,
            name,
            dataset_id,
            evaluator_version_id,
            inconclusive_policy,
        } => {
            let gates = SqliteGateStore::open(data_dir.join("gates.sqlite"))?;
            let gate = gates
                .put_gate(GateDefinition {
                    tenant_id: TenantId::new(tenant_id)?,
                    project_id: ProjectId::new(project_id)?,
                    gate_id: GateId::new(gate_id)?,
                    name,
                    dataset_id: dataset_id.map(DatasetId::new).transpose()?,
                    evaluator_version_id: evaluator_version_id
                        .map(EvaluatorVersionId::new)
                        .transpose()?,
                    inconclusive_policy: inconclusive_policy.into(),
                    created_at: Utc::now(),
                })
                .await
                .context("create gate policy")?;
            println!("{}", serde_json::to_string_pretty(&gate)?);
        }
        Command::GateRun {
            data_dir,
            tenant_id,
            project_id,
            gate_id,
            experiment_run_id,
        } => {
            let gates = SqliteGateStore::open(data_dir.join("gates.sqlite"))?;
            let experiments = SqliteExperimentStore::open(data_dir.join("experiments.sqlite"))?;
            let report = run_gate(
                &gates,
                &experiments,
                TenantId::new(tenant_id)?,
                ProjectId::new(project_id)?,
                GateId::new(gate_id)?,
                experiment_run_id.map(ExperimentRunId::new).transpose()?,
            )
            .await
            .context("run gate")?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                std::process::exit(1);
            }
        }
        Command::GateRunFixture { data_dir } => {
            let gates = SqliteGateStore::open(data_dir.join("gates.sqlite"))?;
            let experiments = SqliteExperimentStore::open(data_dir.join("experiments.sqlite"))?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let dataset = DatasetId::new("gate-fixture-dataset")?;
            let evaluator = EvaluatorVersionId::new("exact-v1")?;
            let gate = gates
                .put_gate(GateDefinition {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    gate_id: GateId::new("main")?,
                    name: "main".to_string(),
                    dataset_id: Some(dataset.clone()),
                    evaluator_version_id: Some(evaluator.clone()),
                    inconclusive_policy: InconclusivePolicy::Fail,
                    created_at: Utc::now(),
                })
                .await
                .context("create gate fixture policy")?;
            let older_pass = experiments
                .write_run(gate_fixture_experiment(
                    &tenant,
                    &project,
                    GateFixtureExperimentSpec {
                        experiment_run_id: "gate-older-pass",
                        dataset: &dataset,
                        evaluator: &evaluator,
                        decision: GateDecision::Pass,
                        delta: 0.1,
                        created_at: "2026-06-19T10:00:00Z",
                    },
                )?)
                .await
                .context("store older passing gate fixture experiment")?;
            let latest_fail = experiments
                .write_run(gate_fixture_experiment(
                    &tenant,
                    &project,
                    GateFixtureExperimentSpec {
                        experiment_run_id: "gate-latest-fail",
                        dataset: &dataset,
                        evaluator: &evaluator,
                        decision: GateDecision::FailRegression,
                        delta: -0.25,
                        created_at: "2026-06-19T11:00:00Z",
                    },
                )?)
                .await
                .context("store latest failing gate fixture experiment")?;
            let report = run_gate(
                &gates,
                &experiments,
                tenant,
                project,
                gate.gate_id.clone(),
                None,
            )
            .await
            .context("run gate fixture")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "gate": gate,
                    "older_experiment_run_id": older_pass.experiment_run_id,
                    "latest_experiment_run_id": latest_fail.experiment_run_id,
                    "gate_run": report
                }))?
            );
        }
        Command::ReviewFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let reviews = SqliteHumanReviewStore::open(data_dir.join("reviews.sqlite"))?;
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace_id = TraceId::new("smoke-trace")?;
            let span_id = SpanId::new("smoke-root")?;
            let trace = traces
                .get_trace(tenant.clone(), trace_id.clone())
                .await
                .context("read smoke trace")?;
            let dataset = datasets
                .create_dataset(
                    tenant.clone(),
                    project.clone(),
                    "review-fixture".to_string(),
                )
                .await
                .context("create review dataset")?;
            let queue = reviews
                .create_queue(CreateReviewQueueRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    queue_id: Some(ReviewQueueId::new("quality")?),
                    name: "quality".to_string(),
                    annotation_schema: json!({
                        "type": "object",
                        "properties": {
                            "reference": {"type": "object"},
                            "notes": {"type": "string"}
                        },
                        "required": ["reference"]
                    }),
                })
                .await
                .context("create review queue")?;
            let task = reviews
                .enqueue_task(EnqueueReviewTaskRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    queue_id: queue.queue_id.clone(),
                    task_id: Some(ReviewTaskId::new("smoke-review")?),
                    trace_id,
                    span_id: Some(span_id),
                    dataset_id: Some(dataset.dataset_id.clone()),
                    dataset_case_id: None,
                    priority: 10,
                })
                .await
                .context("enqueue review task")?;
            let annotation = reviews
                .submit_annotation(SubmitAnnotationRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    queue_id: queue.queue_id.clone(),
                    task_id: task.task_id.clone(),
                    annotation_id: Some(AnnotationId::new("smoke-annotation")?),
                    reviewer_id: "local-reviewer".to_string(),
                    verdict: ReviewVerdict::Pass,
                    payload: json!({
                        "reference": {"answer": "world"},
                        "notes": "fixture human label"
                    }),
                })
                .await
                .context("submit review annotation")?;
            let submitted_task = reviews
                .get_task(
                    tenant.clone(),
                    project.clone(),
                    queue.queue_id.clone(),
                    task.task_id.clone(),
                )
                .await
                .context("read submitted review task")?;
            let case = promote_review_annotation_to_dataset_case(
                tenant.clone(),
                project.clone(),
                dataset.dataset_id.clone(),
                &trace,
                &task,
                &annotation,
                None,
            )
            .context("promote review annotation to dataset case")?;
            let case = datasets
                .put_case(case)
                .await
                .context("store review dataset case")?;
            let version = datasets
                .create_version(
                    tenant.clone(),
                    project.clone(),
                    dataset.dataset_id.clone(),
                    None,
                )
                .await
                .context("create review dataset version")?;
            let report = evaluate_dataset_version(
                &version,
                DatasetEvalSpec {
                    evaluator: EvaluatorSpec {
                        id: "exact".to_string(),
                        lane: EvaluatorLane::DeterministicWasi,
                        kind: EvaluatorKind::ExactMatch,
                    },
                    evaluator_version_id: EvaluatorVersionId::new("review-exact-v1")?,
                    agent_release_id: AgentReleaseId::new("review-agent")?,
                    prompt_version_id: None,
                    code_hash: None,
                    wasm_hash: None,
                },
            )
            .context("evaluate human-reviewed dataset")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "queue": queue,
                    "task": submitted_task,
                    "annotation": annotation,
                    "dataset": dataset,
                    "case": case,
                    "version": version,
                    "eval_report": report
                }))?
            );
        }
        Command::CalibrationFixture { data_dir } => {
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let calibrations = SqliteCalibrationStore::open(data_dir.join("calibrations.sqlite"))?;
            let secret_keyring = SecretKeyring::load_or_create_local_file(
                data_dir.join("provider-secrets.key"),
                "local-v1",
            )?;
            let secrets = EncryptedSqliteProviderSecretStore::open(
                data_dir.join("provider-secrets.sqlite"),
                secret_keyring,
            )?;
            let ledger = SqliteJudgeLedger::open(data_dir.join("judge.sqlite"))?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let dataset = datasets
                .create_dataset(
                    tenant.clone(),
                    project.clone(),
                    "calibration-fixture".to_string(),
                )
                .await
                .context("create calibration dataset")?;
            for case in calibration_fixture_cases(&tenant, &project, &dataset.dataset_id)? {
                datasets
                    .put_case(case)
                    .await
                    .context("store calibration case")?;
            }
            let version = datasets
                .create_version(
                    tenant.clone(),
                    project.clone(),
                    dataset.dataset_id.clone(),
                    None,
                )
                .await
                .context("create calibration dataset version")?;
            let fixture_secret = "sk-local-calibration-judge-secret";
            let secret_metadata = secrets
                .put_secret(PutProviderSecretRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    provider: "openai".to_string(),
                    display_name: "local calibration judge fixture".to_string(),
                    secret_value: fixture_secret.to_string(),
                })
                .await
                .context("create calibration judge provider secret")?;
            let broker = JudgeBrokerService::new(
                secrets,
                ledger.clone(),
                KeywordJudgeProvider::new(Money::usd_micros(25)),
                Money::usd_micros(200),
            );
            let eval_report = evaluate_dataset_version_with_judge(
                &version,
                DatasetJudgeEvalSpec {
                    eval: DatasetEvalSpec {
                        evaluator: EvaluatorSpec {
                            id: "judge-correctness".to_string(),
                            lane: EvaluatorLane::JudgeBroker,
                            kind: EvaluatorKind::LlmJudge {
                                rubric: "correctness".to_string(),
                                model: "judge-model".to_string(),
                            },
                        },
                        evaluator_version_id: EvaluatorVersionId::new("judge-calibration-v1")?,
                        agent_release_id: AgentReleaseId::new("calibration-agent")?,
                        prompt_version_id: None,
                        code_hash: None,
                        wasm_hash: None,
                    },
                    provider_secret_id: secret_metadata.provider_secret_id.clone(),
                },
                &broker,
            )
            .await
            .context("run calibration judge eval")?;
            let eval_report = datasets
                .write_eval_report(eval_report)
                .await
                .context("store calibration eval report")?;
            let calibration = calibrate_eval_report(
                &version,
                &eval_report,
                CalibrationPolicy {
                    pass_threshold: 0.5,
                },
            )
            .context("calibrate judge report")?;
            let calibration = calibrations
                .write_report(calibration)
                .await
                .context("store calibration report")?;
            let output = serde_json::to_string_pretty(&serde_json::json!({
                "provider_secret": secret_metadata,
                "dataset": dataset,
                "version": version,
                "eval_report": eval_report,
                "calibration": calibration
            }))?;
            if output.contains(fixture_secret) {
                anyhow::bail!("calibration fixture output leaked provider secret material");
            }
            assert_secret_not_in_judge_fixture_files(&data_dir, fixture_secret)?;
            println!("{output}");
        }
        Command::UsageFixture { data_dir } => {
            let secret_keyring = SecretKeyring::load_or_create_local_file(
                data_dir.join("provider-secrets.key"),
                "local-v1",
            )?;
            let secrets = EncryptedSqliteProviderSecretStore::open(
                data_dir.join("provider-secrets.sqlite"),
                secret_keyring,
            )?;
            let judge_ledger = SqliteJudgeLedger::open(data_dir.join("judge.sqlite"))?;
            let usage = SqliteUsageLedger::open(data_dir.join("usage.sqlite"))?;
            let run_id = Utc::now().timestamp_micros();
            let tenant = TenantId::new(format!("demo-{run_id}"))?;
            let project = ProjectId::new("demo")?;
            let fixture_secret = "sk-local-usage-judge-secret";
            let secret_metadata = secrets
                .put_secret(PutProviderSecretRequest {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    provider: "openai".to_string(),
                    display_name: "local usage judge fixture".to_string(),
                    secret_value: fixture_secret.to_string(),
                })
                .await
                .context("create usage judge provider secret")?;
            let broker = JudgeBrokerService::new(
                secrets,
                judge_ledger.clone(),
                KeywordJudgeProvider::new(Money::usd_micros(25)),
                Money::usd_micros(100),
            );
            let first = broker
                .evaluate(judge_fixture_request(
                    &tenant,
                    &project,
                    secret_metadata.provider_secret_id.clone(),
                ))
                .await
                .context("run usage fixture first judge evaluation")?;
            let second = broker
                .evaluate(judge_fixture_request(
                    &tenant,
                    &project,
                    secret_metadata.provider_secret_id.clone(),
                ))
                .await
                .context("run usage fixture cached judge evaluation")?;
            let inserts = vec![
                judge_usage_from_outcome(&first),
                judge_usage_from_outcome(&second),
            ];
            let first_write = record_usage_batch(&usage, inserts.clone())
                .await
                .context("write usage fixture records")?;
            let second_write = record_usage_batch(&usage, inserts)
                .await
                .context("rewrite usage fixture records idempotently")?;
            let usage_records = usage
                .list_usage(tenant.clone(), project.clone())
                .await
                .context("list usage fixture records")?;
            let usage_summary = usage
                .summarize_usage(tenant.clone(), project.clone())
                .await
                .context("summarize usage fixture records")?;
            let judge_total = usage_summary
                .totals
                .get(UsageMeter::JudgeCostMicros.as_str())
                .map(|total| total.quantity)
                .unwrap_or_default();
            if usage_records.len() != 2 {
                anyhow::bail!("expected two usage records, got {}", usage_records.len());
            }
            if judge_total != 25 {
                anyhow::bail!("expected 25 judge cost micros, got {judge_total}");
            }
            let output = serde_json::to_string_pretty(&serde_json::json!({
                "provider_secret": secret_metadata,
                "first_judge": first,
                "second_judge": second,
                "first_usage_write": first_write,
                "second_usage_write": second_write,
                "usage_records": usage_records,
                "usage_summary": usage_summary
            }))?;
            if output.contains(fixture_secret) {
                anyhow::bail!("usage fixture output leaked provider secret material");
            }
            assert_secret_not_in_judge_fixture_files(&data_dir, fixture_secret)?;
            println!("{output}");
        }
        Command::AuditFixture { data_dir } => {
            let audit = SqliteAuditStore::open(data_dir.join("audit.sqlite"))?;
            let run_id = Utc::now().timestamp_micros();
            let tenant = TenantId::new(format!("demo-{run_id}"))?;
            let project = ProjectId::new("demo")?;
            let environment = EnvironmentId::new("prod")?;
            let trace = TraceId::new("audit-fixture-trace")?;
            let denied = audit
                .append_event(pii_unmask_event(PiiUnmaskAuditInput {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    environment_id: Some(environment.clone()),
                    actor_api_key_id: Some(ApiKeyId::new("trace-read-key")?),
                    trace_id: trace.clone(),
                    outcome: AuditOutcome::Denied,
                    reason: Some("incident-123".to_string()),
                    attributes: json!({
                        "sensitive_ref_count": 1,
                        "error": "api key scope pii:unmask is missing"
                    }),
                }))
                .await
                .context("write denied unmask audit event")?;
            let allowed = audit
                .append_event(pii_unmask_event(PiiUnmaskAuditInput {
                    tenant_id: tenant.clone(),
                    project_id: project.clone(),
                    environment_id: Some(environment),
                    actor_api_key_id: Some(ApiKeyId::new("pii-unmask-key")?),
                    trace_id: trace,
                    outcome: AuditOutcome::Allowed,
                    reason: Some("incident-123".to_string()),
                    attributes: json!({
                        "sensitive_ref_count": 1,
                        "unmasked": true
                    }),
                }))
                .await
                .context("write allowed unmask audit event")?;
            let events = audit
                .list_events(tenant.clone(), project.clone())
                .await
                .context("list audit fixture events")?;
            if events.len() != 2 {
                anyhow::bail!("expected two audit events, got {}", events.len());
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "denied": denied,
                    "allowed": allowed,
                    "events": events
                }))?
            );
        }
        Command::AgentFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
            let experiments = SqliteExperimentStore::open(data_dir.join("experiments.sqlite"))?;
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace = traces
                .get_trace(tenant.clone(), TraceId::new("smoke-trace")?)
                .await
                .context("read smoke trace")?;
            let dataset = datasets
                .create_dataset(tenant.clone(), project.clone(), "agent-fixture".to_string())
                .await
                .context("create agent dataset")?;
            let case = promote_trace_span_to_case(
                tenant.clone(),
                project.clone(),
                dataset.dataset_id.clone(),
                &trace,
                Some(SpanId::new("smoke-root")?),
                Some(json!({ "answer": "world" })),
            )
            .context("promote smoke trace to agent dataset")?;
            datasets
                .put_case(case)
                .await
                .context("store agent dataset case")?;
            let version = datasets
                .create_version(tenant, project, dataset.dataset_id, None)
                .await
                .context("create agent dataset version")?;
            let report = run_agent_experiment(
                &version,
                AgentExperimentSpec {
                    baseline_release_id: AgentReleaseId::new("baseline-agent")?,
                    candidate_release_id: AgentReleaseId::new("candidate-agent")?,
                    evaluator: EvaluatorSpec {
                        id: "exact".to_string(),
                        lane: EvaluatorLane::DeterministicWasi,
                        kind: EvaluatorKind::ExactMatch,
                    },
                    evaluator_version_id: EvaluatorVersionId::new("exact-v1")?,
                    gate_policy: GatePolicy {
                        min_sample_size: 1,
                        max_regression: 0.05,
                        ..GatePolicy::default()
                    },
                },
                &StaticAgentAdapter::new(json!({ "answer": "nope" }), "static-baseline"),
                &ReferenceAgentAdapter::new("reference-candidate"),
            )
            .await
            .context("run agent harness experiment")?;
            let report = experiments
                .write_run(report)
                .await
                .context("store agent harness experiment")?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AlertFixture { data_dir } => {
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
            let bus = local_bus(&data_dir)?;
            let service =
                IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
            let _outcome = smoke_trace(&service).await.context("run smoke trace")?;
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let trace_id = TraceId::new("smoke-trace")?;
            let trace = traces
                .get_trace(tenant.clone(), trace_id.clone())
                .await
                .context("read smoke trace")?;
            let sampling = decide_trace_sampling(
                &trace,
                &OnlineSamplingPolicy {
                    sample_rate_per_mille: 1000,
                    keep_errors: true,
                    slow_ms_threshold: Some(1),
                    high_cost_micros_threshold: Some(1),
                },
            );
            let now = Utc::now();
            let alert = AlertEngine::new().evaluate(
                &AlertPolicy {
                    policy_id: "smoke-alert".to_string(),
                    endpoint_url: "https://example.test/beater-webhook".to_string(),
                    signing_secret: "local-secret".to_string(),
                    severity: AlertSeverity::Warning,
                    fire_when_score_at_or_below: 0.5,
                    dedupe_window_seconds: 60,
                    maintenance_windows: Vec::new(),
                },
                AlertInput {
                    tenant_id: tenant,
                    project_id: project,
                    trace_id,
                    group_key: "smoke-eval-low-score".to_string(),
                    title: "Smoke eval score below threshold".to_string(),
                    score: 0.25,
                    baseline_score: Some(1.0),
                    links: AlertLinks {
                        trace_url: "http://localhost:8080/traces/smoke-trace".to_string(),
                        cluster_url: Some("http://localhost:8080/clusters/smoke".to_string()),
                        dataset_url: Some("http://localhost:8080/datasets/smoke".to_string()),
                        gate_url: Some("http://localhost:8080/gates/smoke".to_string()),
                    },
                    now,
                },
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "sampling": sampling,
                    "alert": alert
                }))?
            );
        }
        Command::ApiKeyCreate {
            data_dir,
            tenant_id,
            project_id,
            environment_id,
            scopes,
        } => {
            let store = SqliteApiKeyStore::open(data_dir.join("security.sqlite"))?;
            let created = store
                .create_key(CreateApiKeyRequest {
                    tenant_id: TenantId::new(tenant_id)?,
                    project_id: ProjectId::new(project_id)?,
                    environment_id: EnvironmentId::new(environment_id)?,
                    scopes: scopes
                        .into_iter()
                        .map(ApiScope::from)
                        .collect::<BTreeSet<_>>(),
                })
                .await
                .context("create api key")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "api_key_id": created.record.api_key_id,
                    "tenant_id": created.record.tenant_id,
                    "project_id": created.record.project_id,
                    "environment_id": created.record.environment_id,
                    "scopes": created.record.scopes,
                    "active": created.record.active,
                    "created_at": created.record.created_at,
                    "secret": created.secret
                }))?
            );
        }
        Command::ApiKeyRevoke {
            data_dir,
            api_key_id,
        } => {
            let store = SqliteApiKeyStore::open(data_dir.join("security.sqlite"))?;
            let revoked = store
                .revoke_key(ApiKeyId::new(api_key_id)?, Utc::now())
                .await
                .context("revoke api key")?;
            println!("{}", serde_json::to_string_pretty(&revoked)?);
        }
    }
    Ok(())
}

fn local_bus(data_dir: &Path) -> anyhow::Result<Arc<dyn DurableBus>> {
    Ok(Arc::new(SqliteDurableBus::open(
        data_dir.join("bus.sqlite"),
        128,
    )?))
}

async fn run_local_smoke(data_dir: PathBuf) -> anyhow::Result<serde_json::Value> {
    let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
    let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
    let bus = local_bus(&data_dir)?;
    let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let scope = TenantScope::new(
        TenantId::new("demo")?,
        ProjectId::new("demo")?,
        EnvironmentId::new("local")?,
    );
    let (trace_bytes, span_bytes) = smoke_ids();
    let export = otlp_smoke_export(trace_bytes, span_bytes);
    let raw_bytes = encode_export_trace_request(&export);
    let raw_request = export_to_raw_trace_ingest_request(
        scope.clone(),
        raw_bytes,
        export,
        anonymous_auth_context(),
    )
    .context("build OTLP smoke ingest request")?;
    let trace_id = raw_request
        .spans
        .first()
        .map(|span| span.trace_id.clone())
        .ok_or_else(|| anyhow::anyhow!("OTLP smoke export produced no spans"))?;
    let outcome = service
        .buffer_raw_trace_batch(raw_request)
        .await
        .context("buffer OTLP smoke trace")?;
    let write_report = service
        .drain_trace_writes_for(&scope.tenant_id, &scope.project_id, 10)
        .await
        .context("drain OTLP smoke trace writes")?;
    let trace = traces
        .get_trace(scope.tenant_id.clone(), trace_id.clone())
        .await
        .context("read OTLP smoke trace")?;
    let downstream_report = service
        .drain_trace_ingested_for(&scope.tenant_id, &scope.project_id, 10, {
            let traces = traces.clone();
            move |trace_ref| {
                let traces = traces.clone();
                async move {
                    traces
                        .get_trace(trace_ref.tenant_id, trace_ref.trace_id)
                        .await
                        .map(|_| ())
                        .map_err(|err| err.to_string())
                }
            }
        })
        .await
        .context("drain OTLP smoke downstream work")?;
    Ok(json!({
        "mode": "local",
        "source": "otlp",
        "trace_id": trace_id,
        "outcome": outcome,
        "write_report": write_report,
        "downstream_report": downstream_report,
        "trace_span_count": trace.spans.len(),
        "normalizer_version": trace.spans.first().map(|span| span.normalizer_version.clone())
    }))
}

async fn run_remote_smoke(
    http_url: String,
    otlp_grpc_url: Option<String>,
    tenant_id: String,
    project_id: String,
    environment_id: String,
    timeout_ms: u64,
) -> anyhow::Result<serde_json::Value> {
    let (trace_bytes, span_bytes) = smoke_ids();
    let trace_id = hex(&trace_bytes);
    let export = otlp_smoke_export(trace_bytes, span_bytes);
    let protocol = if let Some(otlp_grpc_url) = otlp_grpc_url {
        emit_remote_grpc(
            otlp_grpc_url,
            export,
            &tenant_id,
            &project_id,
            &environment_id,
        )
        .await?;
        "grpc"
    } else {
        emit_remote_http(
            &http_url,
            &tenant_id,
            &project_id,
            &environment_id,
            &encode_export_trace_request(&export),
        )
        .await?;
        "http"
    };
    let trace = wait_for_remote_trace(
        &http_url,
        &tenant_id,
        &trace_id,
        StdDuration::from_millis(timeout_ms),
    )
    .await?;
    let spans = trace
        .get("spans")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(json!({
        "mode": "remote",
        "protocol": protocol,
        "source": "otlp",
        "trace_id": trace_id,
        "trace_read_url": format!("{}/v1/traces/{}/{}", trim_url(&http_url), tenant_id, trace_id),
        "trace_span_count": spans.len(),
        "normalizer_version": spans.first().and_then(|span| span.get("normalizer_version")).cloned(),
    }))
}

async fn emit_remote_http(
    http_url: &str,
    tenant_id: &str,
    project_id: &str,
    environment_id: &str,
    body: &[u8],
) -> anyhow::Result<()> {
    let url = format!(
        "{}/v1/otlp/{}/{}/{}/v1/traces?durability=buffered",
        trim_url(http_url),
        tenant_id,
        project_id,
        environment_id
    );
    reqwest::Client::new()
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body.to_vec())
        .send()
        .await
        .context("send OTLP HTTP smoke trace")?
        .error_for_status()
        .context("OTLP HTTP smoke trace was rejected")?;
    Ok(())
}

async fn emit_remote_grpc(
    otlp_grpc_url: String,
    export: ExportTraceServiceRequest,
    tenant_id: &str,
    project_id: &str,
    environment_id: &str,
) -> anyhow::Result<()> {
    let mut client = TraceServiceClient::connect(otlp_grpc_url)
        .await
        .context("connect to OTLP gRPC endpoint")?;
    let mut request = TonicRequest::new(export);
    request
        .metadata_mut()
        .insert("x-beater-tenant-id", metadata_value(tenant_id)?);
    request
        .metadata_mut()
        .insert("x-beater-project-id", metadata_value(project_id)?);
    request
        .metadata_mut()
        .insert("x-beater-environment-id", metadata_value(environment_id)?);
    client
        .export(request)
        .await
        .context("send OTLP gRPC smoke trace")?;
    Ok(())
}

async fn wait_for_remote_trace(
    http_url: &str,
    tenant_id: &str,
    trace_id: &str,
    timeout: StdDuration,
) -> anyhow::Result<serde_json::Value> {
    let url = format!(
        "{}/v1/traces/{}/{}",
        trim_url(http_url),
        tenant_id,
        trace_id
    );
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let trace = client
            .get(&url)
            .send()
            .await
            .context("read smoke trace")?
            .error_for_status()
            .context("smoke trace read failed")?
            .json::<serde_json::Value>()
            .await
            .context("decode smoke trace response")?;
        let span_count = trace
            .get("spans")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
        if span_count > 0 {
            return Ok(trace);
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!(
                "trace {} was not queryable at {} before timeout; last response: {}",
                trace_id,
                url,
                trace
            );
        }
        tokio::time::sleep(StdDuration::from_millis(50)).await;
    }
}

fn trim_url(url: &str) -> &str {
    url.trim_end_matches('/')
}

fn metadata_value(value: &str) -> anyhow::Result<MetadataValue<tonic::metadata::Ascii>> {
    value
        .parse()
        .map_err(|err| anyhow::anyhow!("invalid gRPC metadata value {value:?}: {err}"))
}

fn smoke_ids() -> ([u8; 16], [u8; 8]) {
    let now = Utc::now().timestamp_nanos_opt().unwrap_or_default() as u128;
    let trace = now.to_be_bytes();
    let span = (now as u64).to_be_bytes();
    (trace, span)
}

fn otlp_smoke_export(trace_id: [u8; 16], span_id: [u8; 8]) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![otel_kv("service.name", otel_string("beaterctl-smoke"))],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope {
                    name: "beaterctl".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    attributes: Vec::new(),
                    dropped_attributes_count: 0,
                }),
                spans: vec![Span {
                    trace_id: trace_id.to_vec(),
                    span_id: span_id.to_vec(),
                    trace_state: String::new(),
                    parent_span_id: Vec::new(),
                    flags: 0,
                    name: "beaterctl otlp smoke".to_string(),
                    kind: span::SpanKind::Client as i32,
                    start_time_unix_nano: Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    end_time_unix_nano: Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                    attributes: vec![
                        otel_kv("openinference.span.kind", otel_string("llm")),
                        otel_kv("input.value", otel_string("hello")),
                        otel_kv("output.value", otel_string("world")),
                    ],
                    dropped_attributes_count: 0,
                    events: Vec::new(),
                    dropped_events_count: 0,
                    links: Vec::new(),
                    dropped_links_count: 0,
                    status: Some(Status {
                        message: String::new(),
                        code: status::StatusCode::Ok as i32,
                    }),
                }],
                schema_url: "https://opentelemetry.io/schemas/1.37.0".to_string(),
            }],
            schema_url: "https://opentelemetry.io/schemas/1.37.0".to_string(),
        }],
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn otel_kv(key: &str, value: AnyValue) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        key_strindex: 0,
        value: Some(value),
    }
}

fn otel_string(value: &str) -> AnyValue {
    AnyValue {
        value: Some(any_value::Value::StringValue(value.to_string())),
    }
}

struct GateFixtureExperimentSpec<'a> {
    experiment_run_id: &'a str,
    dataset: &'a DatasetId,
    evaluator: &'a EvaluatorVersionId,
    decision: GateDecision,
    delta: f64,
    created_at: &'a str,
}

fn gate_fixture_experiment(
    tenant: &TenantId,
    project: &ProjectId,
    spec: GateFixtureExperimentSpec<'_>,
) -> anyhow::Result<ExperimentRunReport> {
    let created_at = chrono::DateTime::parse_from_rfc3339(spec.created_at)?.with_timezone(&Utc);
    Ok(ExperimentRunReport {
        experiment_run_id: ExperimentRunId::new(spec.experiment_run_id)?,
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        dataset_id: spec.dataset.clone(),
        dataset_version_id: DatasetVersionId::new("gate-fixture-version")?,
        baseline_release_id: AgentReleaseId::new("gate-baseline")?,
        candidate_release_id: AgentReleaseId::new("gate-candidate")?,
        evaluator_version_id: spec.evaluator.clone(),
        case_scores: Vec::new(),
        comparison: ExperimentComparison {
            sample_size: 1,
            baseline_mean: 1.0,
            candidate_mean: 1.0 + spec.delta,
            delta: spec.delta,
            ci_low: spec.delta,
            ci_high: spec.delta,
            decision: spec.decision.clone(),
            test: StatisticalTest::PairedNormalApproximation,
            adjusted_alpha: 0.05,
        },
        decision: spec.decision,
        gate_policy: GatePolicy {
            min_sample_size: 1,
            ..GatePolicy::default()
        },
        created_at,
    })
}

fn calibration_fixture_cases(
    tenant: &TenantId,
    project: &ProjectId,
    dataset: &DatasetId,
) -> anyhow::Result<Vec<DatasetCase>> {
    Ok(vec![
        calibration_fixture_case(
            tenant,
            project,
            dataset,
            "case-pass-1",
            json!("alpha"),
            json!("alpha"),
        )?,
        calibration_fixture_case(
            tenant,
            project,
            dataset,
            "case-pass-2",
            json!("bravo"),
            json!("bravo"),
        )?,
        calibration_fixture_case(
            tenant,
            project,
            dataset,
            "case-fail-agree",
            json!("charlie"),
            json!("delta"),
        )?,
        calibration_fixture_case(
            tenant,
            project,
            dataset,
            "case-fail-disagree",
            json!("mentions correctness but is still wrong"),
            json!("expected answer"),
        )?,
    ])
}

fn calibration_fixture_case(
    tenant: &TenantId,
    project: &ProjectId,
    dataset: &DatasetId,
    case_id: &str,
    output: serde_json::Value,
    reference: serde_json::Value,
) -> anyhow::Result<DatasetCase> {
    Ok(DatasetCase {
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        dataset_id: dataset.clone(),
        case_id: beater_core::DatasetCaseId::new(case_id)?,
        source_trace_id: TraceId::new(format!("trace-{case_id}"))?,
        source_span_id: SpanId::new(format!("span-{case_id}"))?,
        source_environment_id: EnvironmentId::new("local")?,
        input: json!({ "prompt": case_id }),
        output,
        reference: Some(reference),
        trace: json!({ "fixture": "calibration" }),
        normalizer_version: "beater-calibration-fixture-v1".to_string(),
        trace_schema_version: 1,
        input_artifact_hashes: Vec::new(),
        created_at: Utc::now(),
    })
}

fn judge_fixture_request(
    tenant: &TenantId,
    project: &ProjectId,
    provider_secret_id: ProviderSecretId,
) -> JudgeBrokerRequest {
    JudgeBrokerRequest {
        tenant_id: tenant.clone(),
        project_id: project.clone(),
        evaluator: EvaluatorSpec {
            id: "judge-correctness".to_string(),
            lane: EvaluatorLane::JudgeBroker,
            kind: EvaluatorKind::LlmJudge {
                rubric: "correctness".to_string(),
                model: "judge-model".to_string(),
            },
        },
        case: EvaluationCase {
            input: json!("question"),
            output: json!("answer"),
            reference: Some(json!("answer")),
            trace: None,
        },
        provider_secret_id,
    }
}

fn assert_secret_not_in_judge_fixture_files(data_dir: &Path, secret: &str) -> anyhow::Result<()> {
    for relative_path in [
        "provider-secrets.sqlite",
        "provider-secrets.sqlite-wal",
        "provider-secrets.sqlite-shm",
        "judge.sqlite",
        "judge.sqlite-wal",
        "judge.sqlite-shm",
        "usage.sqlite",
        "usage.sqlite-wal",
        "usage.sqlite-shm",
    ] {
        let path = data_dir.join(relative_path);
        if !path.exists() {
            continue;
        }
        let bytes = std::fs::read(&path)
            .with_context(|| format!("read fixture persistence file {}", path.display()))?;
        if String::from_utf8_lossy(&bytes).contains(secret) {
            anyhow::bail!(
                "judge fixture persistence file {} leaked provider secret material",
                path.display()
            );
        }
    }
    Ok(())
}

fn replay_fixture_events(
    tenant: &TenantId,
    project: &ProjectId,
    trace: &TraceId,
) -> anyhow::Result<Vec<ReplayEvent>> {
    Ok(vec![
        replay_fixture_event(
            tenant,
            project,
            trace,
            1,
            ReplayEventKind::Provider,
            "provider",
        )?,
        replay_fixture_event(tenant, project, trace, 2, ReplayEventKind::Tool, "tool")?,
        replay_fixture_event(tenant, project, trace, 3, ReplayEventKind::Memory, "memory")?,
        replay_fixture_event(
            tenant,
            project,
            trace,
            4,
            ReplayEventKind::Retrieval,
            "retrieval",
        )?,
        replay_fixture_event(tenant, project, trace, 5, ReplayEventKind::Clock, "clock")?,
        replay_fixture_event(tenant, project, trace, 6, ReplayEventKind::Random, "random")?,
    ])
}

fn replay_fixture_event(
    tenant: &TenantId,
    project: &ProjectId,
    trace: &TraceId,
    seq: u64,
    kind: ReplayEventKind,
    label: &str,
) -> anyhow::Result<ReplayEvent> {
    ReplayEvent::new(
        tenant.clone(),
        project.clone(),
        trace.clone(),
        seq,
        kind,
        json!({ "request": label }),
        json!({ label: "ok" }),
    )
}

struct UnavailableTraceStore;

#[async_trait::async_trait]
impl TraceStore for UnavailableTraceStore {
    async fn write_batch(
        &self,
        _batch: CanonicalTraceBatch,
    ) -> beater_store::StoreResult<WriteAck> {
        Err(StoreError::Backend("trace store unavailable".to_string()))
    }

    async fn get_trace(
        &self,
        _tenant: TenantId,
        _trace: TraceId,
    ) -> beater_store::StoreResult<TraceView> {
        Err(StoreError::Backend("trace store unavailable".to_string()))
    }

    async fn get_raw_envelope(
        &self,
        _tenant: TenantId,
        _project: ProjectId,
        _idempotency_key: IdempotencyKey,
    ) -> beater_store::StoreResult<Option<RawEnvelope>> {
        Err(StoreError::Backend("trace store unavailable".to_string()))
    }

    async fn query_runs(
        &self,
        _tenant: TenantId,
        _filter: RunFilter,
        _page: PageRequest,
    ) -> beater_store::StoreResult<Page<RunSummary>> {
        Err(StoreError::Backend("trace store unavailable".to_string()))
    }

    async fn query_spans(
        &self,
        _tenant: TenantId,
        _filter: SpanFilter,
        _page: PageRequest,
    ) -> beater_store::StoreResult<Page<SpanSummary>> {
        Err(StoreError::Backend("trace store unavailable".to_string()))
    }
}
