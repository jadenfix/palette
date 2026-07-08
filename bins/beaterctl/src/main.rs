use anyhow::Context;
use beater_alerts::{
    AlertEngine, AlertInput, AlertLinks, AlertPolicy, AlertSeverity, OnlineSamplingPolicy,
    decide_trace_sampling,
};
use beater_api::openapi::urlencode;
use beater_audit::{
    AuditOutcome, AuditStore, PiiUnmaskAuditInput, SqliteAuditStore, pii_unmask_event,
};
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
use beater_bus::{BusMessage, DurableBus, SqliteDurableBus};
use beater_calibration::{
    CalibrationPolicy, CalibrationStore, SqliteCalibrationStore, calibrate_eval_report,
};
use beater_core::{
    AgentReleaseId, AnnotationId, ApiKeyId, DatasetId, DatasetVersionId, EnvironmentId,
    EvaluatorVersionId, ExperimentRunId, GateId, IdempotencyKey, Money, Page, PageRequest,
    ProjectId, ProviderSecretId, ReviewQueueId, ReviewTaskId, SpanId, TenantId, TenantScope,
    TraceId, lower_hex,
};
use beater_datasets::{
    DatasetCase, DatasetEvalSpec, DatasetJudgeEvalSpec, DatasetStore, SqliteDatasetStore,
    evaluate_dataset_version, evaluate_dataset_version_with_judge, promote_trace_span_to_case,
};
use beater_eval::{
    EvaluationCase, EvaluatorKind, EvaluatorSpec, ExperimentComparison, GateDecision, GatePolicy,
    StatisticalTest, compare_paired_scores,
};
use beater_experiments::{
    AgentExperimentSpec, CandidateChange, CandidateEvaluator, CaseOutputOverride, CaseScore,
    ExperimentRunReport, ExperimentRunSpec, ExperimentStore, FailureExample,
    JudgeExperimentRunSpec, OptimizationRoundConfig, OptimizerStrategy, ReferenceAgentAdapter,
    Split, SqliteExperimentStore, StaticAgentAdapter, run_agent_experiment,
    run_deterministic_experiment, run_judge_experiment, run_optimization_round,
};
use beater_gates::{GateDefinition, GateStore, InconclusivePolicy, SqliteGateStore, run_gate};
use beater_human::{
    CreateReviewQueueRequest, EnqueueReviewTaskRequest, HumanReviewStore, ReviewVerdict,
    SqliteHumanReviewStore, SubmitAnnotationRequest, promote_review_annotation_to_dataset_case,
};
use beater_ingest::{
    IngestPolicy, IngestService, NativeIngestRequest, TRACE_WRITE_BATCH_KIND,
    anonymous_auth_context, smoke_trace,
};
use beater_judge::{
    AnthropicJudgeProvider, GenerationRequest, GenerationResponse, HttpJudgeProviderConfig,
    JudgeBrokerRequest, JudgeBrokerService, JudgeLedgerStore, JudgeProviderResult,
    KeywordJudgeProvider, ProviderCredentials, SqliteJudgeLedger, TextGenerator,
};
use beater_otlp::{encode_export_trace_request, export_to_raw_trace_ingest_request};
use beater_replay::{
    ReplayEvent, ReplayEventKind, ReplayScenario, ReplayStep, SqliteReplayStore, execute_replay,
};
use beater_schema::{
    AgentSpanKind, CanonicalAttrs, CanonicalTraceBatch, EvaluatorLane, RawEnvelope, RedactionClass,
    RunFilter, RunSummary, SpanFilter, SpanStatus, SpanSummary, TraceView, WriteAck,
};
use beater_secrets::{
    EncryptedSqliteProviderSecretStore, ProviderSecretStore, PutProviderSecretRequest,
    SecretEncryptionKey, SecretKeyring,
};
use beater_security::ApiScope;
use beater_store::{StoreError, TraceStore};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use beater_usage::{
    SqliteUsageLedger, UsageLedgerStore, UsageMeter, judge_usage_from_outcome, record_usage_batch,
};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, trace_service_client::TraceServiceClient,
};
use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, span, status,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tonic::Request as TonicRequest;
use tonic::metadata::MetadataValue;

#[derive(Debug, Parser)]
#[command(name = "beaterctl", about = "Beater local development and CI helper")]
struct Args {
    /// Base URL of the Beater API for remote commands.
    #[arg(
        long,
        global = true,
        env = "BEATER_BASE_URL",
        default_value = "http://127.0.0.1:8080"
    )]
    base_url: String,
    /// API key sent as `Authorization: Bearer <key>` on remote requests.
    #[arg(long, global = true, env = "BEATER_API_KEY")]
    api_key: Option<String>,
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
    /// Prove the local quickstart loop reaches a scored failing case.
    Quickstart {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        #[arg(long, default_value = "http://127.0.0.1:3000")]
        dashboard_url: String,
    },
    /// Validate live OTLP ingest and print a zero-code exporter env block.
    Ingest {
        #[command(subcommand)]
        command: IngestCommand,
    },
    /// Call any Beater API endpoint by its OpenAPI operationId.
    ///
    /// The operation's HTTP method and path template are resolved from the
    /// in-process OpenAPI spec (the same contract the SDKs are generated from),
    /// so the CLI never drifts from the server surface.
    Api {
        /// OpenAPI operationId, e.g. `listTraces`.
        operation_id: String,
        /// Path/query parameters as `key=value`. Path params (matching
        /// `{name}` in the template) are substituted; the rest become query
        /// string parameters.
        #[arg(long = "param", value_name = "KEY=VALUE")]
        params: Vec<String>,
        /// Request body for non-GET operations: literal JSON or `@path/to/file`.
        #[arg(long)]
        body: Option<String>,
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
    /// Drive one recursive-self-improvement optimization round end-to-end with a
    /// deterministic, network-free seeded fixture.
    ///
    /// This is the first production caller of
    /// `beater_experiments::run_optimization_round`: it builds an
    /// `OptimizationRoundConfig` from seeded failing examples + cases, proposes a
    /// candidate via the `LlmRewrite` strategy backed by an in-process canned
    /// `TextGenerator` (no LLM key, no network), scores it with a seeded
    /// `FixtureCandidateEvaluator`, and routes the candidate through the existing
    /// held-out **Test** gate + §21.4 anti-overfit guardrail. The seeded scenario
    /// is a *generalizing* candidate, so the happy path shows acceptance with a
    /// `Pass` gate decision; the overfit-rejection path is unit-tested in
    /// beater-experiments. A live run against real providers + a real candidate
    /// evaluator (executing the agent per case) is the next layer up — this
    /// fixture only proves the proposal → gate loop end-to-end, deterministically.
    ///
    /// With `--record-trace`, the completed round is ALSO emitted as one
    /// canonical Beater trace (via the same `IngestService::ingest_native` path
    /// `smoke` uses) into the `--data-dir` SQLite store and read back, proving the
    /// optimization loop is observable end-to-end with no network.
    RsiRoundFixture {
        /// Emit the round as a canonical trace into `--data-dir` and read it back,
        /// adding `trace_id`/`trace_span_count` to the JSON report. Off by default
        /// so the pure fixture path needs no store.
        #[arg(long)]
        record_trace: bool,
        /// SQLite store + artifact dir used only when `--record-trace` is set.
        /// Defaults to a process-temp dir so the command needs no arguments.
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
    },
    /// Drive one recursive-self-improvement optimization round end-to-end against
    /// a REAL Anthropic model — the live counterpart to `rsi-round-fixture`.
    ///
    /// This closes the "synthetic generation + synthetic scores" gap: a real
    /// Anthropic model BOTH proposes the prompt rewrite (via `LlmRewrite` over the
    /// beater-judge text-generation seam) AND scores every candidate by being
    /// re-prompted per seeded factual Q/A case under the baseline vs. candidate
    /// system prompt. Each answer is graded deterministically (case-insensitive
    /// normalized substring match against the expected answer → 1.0 else 0.0),
    /// then routed through the SAME held-out **Test** gate + §21.4 anti-overfit
    /// guardrail the fixture uses.
    ///
    /// This performs REAL Anthropic API calls and costs tokens. It needs a BYOK
    /// `ANTHROPIC_API_KEY` in the environment; if it is unset/empty the command
    /// returns a clean error (NOT a panic) pointing at `rsi-round-fixture` for a
    /// no-network demo. The default `--model` (`claude-haiku-4-5-20251001`) is a
    /// current, cheap Anthropic model and is overridable.
    RsiRound {
        /// Anthropic model id used for BOTH the proposer rewrite and per-case
        /// evaluation. Overridable; defaults to a current, cheap model.
        #[arg(long, default_value = "claude-haiku-4-5-20251001")]
        model: String,
        /// Optimization goal. Defaults to the seeded factual-hallucination
        /// scenario so the command is a self-contained smoke.
        #[arg(long, default_value = "reduce hallucinations on factual lookups")]
        goal: String,
        /// Baseline system prompt the candidate must beat. Defaults to the same
        /// minimal prompt the fixture seeds.
        #[arg(long, default_value = "You are a helpful assistant.")]
        current_prompt: String,
        /// Emit the round as a canonical trace into `--data-dir` and read it back,
        /// adding `trace_id`/`trace_span_count` to the JSON report. Independent of
        /// the `ANTHROPIC_API_KEY` requirement for the live round itself.
        #[arg(long)]
        record_trace: bool,
        /// SQLite store + artifact dir used only when `--record-trace` is set.
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
    /// Re-encrypt every stored provider secret under the keyring's active key
    /// (R9.3 key rotation). Rows are decrypted under whichever key they were
    /// written with and re-wrapped under `--active-key-id` in a single
    /// transaction; the call is idempotent (a second run re-wraps nothing).
    ///
    /// SAFETY: rotation scans for stale rows and then re-wraps them under a fresh
    /// transaction, so it MUST run with no concurrent writers to the provider
    /// secret store (no API process taking writes) — a secret inserted between
    /// the scan and the rotation transaction would be missed by this pass. Stop
    /// writers, run this, then resume. Provide `--retiring-key-base64` /
    /// `--retiring-key-id` when the active key file is the new key so the old key
    /// is still available to decrypt rows written under it.
    SecretRotate {
        #[arg(long, default_value = ".beater")]
        data_dir: PathBuf,
        /// Key id new ciphertext is wrapped under (the active key in the local
        /// key file at `<data_dir>/provider-secrets.key`).
        #[arg(long, default_value = "local-v1")]
        active_key_id: String,
        /// Base64 of a retiring key that some rows are still encrypted under, so
        /// they can be decrypted and re-wrapped under the active key.
        #[arg(long)]
        retiring_key_base64: Option<String>,
        /// Key id of the retiring key supplied via `--retiring-key-base64`.
        #[arg(long)]
        retiring_key_id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum IngestCommand {
    Test {
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
    let base_url = args.base_url;
    let api_key = args.api_key;
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
                    api_key.as_deref(),
                )
                .await?
            } else {
                run_local_smoke(data_dir).await?
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Quickstart {
            data_dir,
            dashboard_url,
        } => {
            let output = run_local_quickstart(data_dir, &base_url, &dashboard_url).await?;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Ingest {
            command:
                IngestCommand::Test {
                    http_url,
                    otlp_grpc_url,
                    tenant_id,
                    project_id,
                    environment_id,
                    timeout_ms,
                },
        } => {
            let http_url = http_url.unwrap_or_else(|| base_url.clone());
            let output = run_ingest_test(
                http_url,
                otlp_grpc_url,
                tenant_id,
                project_id,
                environment_id,
                timeout_ms,
                api_key.as_deref(),
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::Api {
            operation_id,
            params,
            body,
        } => {
            run_api_call(&base_url, api_key.as_deref(), &operation_id, &params, body).await?;
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
            if comparison.decision != GateDecision::Pass {
                std::process::exit(1);
            }
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
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let bus = SqliteDurableBus::open(&path, 128)?;
            let mut poison = BusMessage::new(
                tenant.clone(),
                project.clone(),
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
            let dead_letter = dlq
                .first()
                .ok_or_else(|| anyhow::anyhow!("expected poison message in dlq"))?;
            let replay_ack = bus
                .replay_dead_letter(&tenant, &project, &dead_letter.message.message_id, true)
                .await
                .context("replay poison message from dlq")?;
            let mut replayed = bus
                .consume_batch(1)
                .await
                .context("consume replayed poison message")?;
            let replayed_message = replayed
                .pop()
                .ok_or_else(|| anyhow::anyhow!("expected replayed poison message"))?;
            let replayed_attempts = replayed_message.attempts;
            let replayed_message_id = replayed_message.message_id.clone();
            bus.ack(replayed_message)
                .await
                .context("ack replayed poison message")?;
            let dlq_after_replay = bus.dlq().await.context("read bus dlq after replay")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "depth": bus.depth().await?,
                    "dlq_len_before_replay": dlq.len(),
                    "dlq_len": dlq_after_replay.len(),
                    "replay_ack": replay_ack,
                    "replayed_attempts": replayed_attempts,
                    "replayed_message_id": replayed_message_id,
                    "dlq": dlq_after_replay
                }))?
            );
        }
        Command::IngestOutageFixture { data_dir } => {
            let tenant = TenantId::new("demo")?;
            let project = ProjectId::new("demo")?;
            let environment = EnvironmentId::new("prod")?;
            let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
            let bus_path = data_dir.join("bus.sqlite");

            let outage_bus = Arc::new(SqliteDurableBus::open(&bus_path, 128)?);
            let explicit_error_request = ingest_outage_request(
                &tenant,
                &project,
                &environment,
                "ingest-outage-explicit-error",
                "explicit-error-root",
                "ingest-outage-explicit-error",
                "explicit error during trace store outage",
            )?;
            let explicit_error_service = IngestService::new(
                artifacts.clone(),
                Arc::new(UnavailableTraceStore),
                outage_bus.clone(),
                IngestPolicy::default(),
            );
            let explicit_error = explicit_error_service
                .ingest_native(explicit_error_request)
                .await
                .err()
                .ok_or_else(|| anyhow::anyhow!("direct ingest should explicitly error"))?
                .to_string();
            if !explicit_error.contains("trace store unavailable") {
                anyhow::bail!("unexpected explicit error: {explicit_error}");
            }

            let dlq_request = ingest_outage_request(
                &tenant,
                &project,
                &environment,
                "ingest-outage-dlq",
                "dlq-root",
                "ingest-outage-dlq",
                "accepted then dlq during trace store outage",
            )?;
            let dlq_service = IngestService::new(
                artifacts.clone(),
                Arc::new(UnavailableTraceStore),
                outage_bus.clone(),
                IngestPolicy {
                    trace_write_max_attempts: 1,
                    ..IngestPolicy::default()
                },
            );
            let dlq_buffered = dlq_service
                .buffer_native(dlq_request)
                .await
                .context("buffer native trace that should later DLQ")?;
            let dlq_report = dlq_service
                .drain_trace_writes_for(&tenant, &project, 10)
                .await
                .context("drain trace write that should DLQ during outage")?;
            if dlq_report.dead_lettered != 1 {
                anyhow::bail!(
                    "expected one dead-lettered trace write during outage, got {}",
                    dlq_report.dead_lettered
                );
            }
            let dlq_status = dlq_service
                .queue_status(tenant.clone(), project.clone())
                .await
                .context("read DLQ status after outage")?;
            let dead_letter = dlq_status
                .dead_letters
                .iter()
                .find(|dead_letter| dead_letter.message.kind == TRACE_WRITE_BATCH_KIND)
                .ok_or_else(|| anyhow::anyhow!("expected trace.write_batch dead letter"))?;
            if !dead_letter.reason.contains("trace store unavailable") {
                anyhow::bail!("unexpected DLQ reason: {}", dead_letter.reason);
            }

            let trace_id = TraceId::new("ingest-outage-recovered")?;
            let recovery_request = ingest_outage_request(
                &tenant,
                &project,
                &environment,
                trace_id.as_str(),
                "recovered-root",
                "ingest-outage-recovered",
                "accepted during trace store outage and recovered",
            )?;
            let outage_service = IngestService::new(
                artifacts.clone(),
                Arc::new(UnavailableTraceStore),
                outage_bus.clone(),
                IngestPolicy::default(),
            );
            let buffered = outage_service
                .buffer_native(recovery_request)
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
            let submitted_events = 3usize;
            let explicit_errors = 1usize;
            let dead_lettered = dlq_report.dead_lettered;
            let recovered_events = usize::from(trace.spans.len() == 1);
            let resolved_events = explicit_errors + dead_lettered + recovered_events;
            let lost = submitted_events.saturating_sub(resolved_events);
            if lost != 0 {
                anyhow::bail!("chaos accounting lost {lost} event(s)");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "no_silent_drop": {
                        "submitted_events": submitted_events,
                        "accepted_buffered": dlq_buffered.ack.accepted_spans + buffered.ack.accepted_spans,
                        "explicit_errors": explicit_errors,
                        "dead_lettered": dead_lettered,
                        "recovered": recovered_events,
                        "lost": lost
                    },
                    "explicit_error": explicit_error,
                    "dlq_buffered": dlq_buffered,
                    "dlq_report": dlq_report,
                    "dlq_reason": dead_letter.reason,
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
            let explicit_inconclusive = experiments
                .write_run(gate_fixture_experiment(
                    &tenant,
                    &project,
                    GateFixtureExperimentSpec {
                        experiment_run_id: "gate-explicit-inconclusive",
                        dataset: &dataset,
                        evaluator: &evaluator,
                        decision: GateDecision::Inconclusive,
                        delta: 0.0,
                        created_at: "2026-06-19T10:30:00Z",
                    },
                )?)
                .await
                .context("store explicit inconclusive gate fixture experiment")?;
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
                    "inconclusive_experiment_run_id": explicit_inconclusive.experiment_run_id,
                    "gate_run": report
                }))?
            );
        }
        Command::RsiRoundFixture {
            record_trace,
            data_dir,
        } => {
            let mut output = run_rsi_round_fixture().await?;
            if record_trace {
                record_rsi_round_trace(
                    &data_dir,
                    "rsi-round-fixture",
                    "fixture",
                    "reduce hallucinations on factual lookups",
                    &mut output,
                )
                .await?;
            }
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Command::RsiRound {
            model,
            goal,
            current_prompt,
            record_trace,
            data_dir,
        } => {
            let goal_for_trace = goal.clone();
            let model_for_trace = model.clone();
            let mut output = run_rsi_round_live(model, goal, current_prompt).await?;
            if record_trace {
                record_rsi_round_trace(
                    &data_dir,
                    "rsi-round",
                    &model_for_trace,
                    &goal_for_trace,
                    &mut output,
                )
                .await?;
            }
            println!("{}", serde_json::to_string_pretty(&output)?);
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
        Command::SecretRotate {
            data_dir,
            active_key_id,
            retiring_key_base64,
            retiring_key_id,
        } => {
            // Load the active key (id + material) from the local key file.
            let active_key = SecretEncryptionKey::from_base64(
                active_key_id.clone(),
                &active_key_file_base64(&data_dir)?,
            )?;
            // Build a rotation keyring: active key + (optionally) a retiring key
            // so rows still encrypted under the old key remain decryptable.
            let mut keys = vec![active_key];
            match (retiring_key_base64, retiring_key_id) {
                (Some(encoded), Some(key_id)) => {
                    keys.push(SecretEncryptionKey::from_base64(key_id, &encoded)?);
                }
                (None, None) => {}
                _ => anyhow::bail!(
                    "--retiring-key-base64 and --retiring-key-id must be supplied together"
                ),
            }
            let keyring = SecretKeyring::with_keys(active_key_id.clone(), keys)?;
            let store = EncryptedSqliteProviderSecretStore::open(
                data_dir.join("provider-secrets.sqlite"),
                keyring,
            )?;
            // NOTE: must run with no concurrent writers (see SecretRotate docs).
            let rotated = store
                .rotate_to_active_key()
                .context("rotate provider secrets to active key")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "active_key_id": active_key_id,
                    "rotated_rows": rotated,
                    "warning": "run with no concurrent writers to the provider secret store"
                }))?
            );
        }
    }
    Ok(())
}

/// Read the raw base64 of the local provider-secret key file.
fn active_key_file_base64(data_dir: &Path) -> anyhow::Result<String> {
    let path = data_dir.join("provider-secrets.key");
    let encoded = std::fs::read_to_string(&path)
        .with_context(|| format!("read provider secret key file {}", path.display()))?;
    Ok(encoded.trim().to_string())
}

fn local_bus(data_dir: &Path) -> anyhow::Result<Arc<dyn DurableBus>> {
    Ok(Arc::new(SqliteDurableBus::open(
        data_dir.join("bus.sqlite"),
        128,
    )?))
}

fn ingest_outage_request(
    tenant: &TenantId,
    project: &ProjectId,
    environment: &EnvironmentId,
    trace_id: &str,
    span_id: &str,
    idempotency_key: &str,
    label: &str,
) -> anyhow::Result<NativeIngestRequest> {
    Ok(NativeIngestRequest {
        scope: TenantScope::new(tenant.clone(), project.clone(), environment.clone()),
        trace_id: TraceId::new(trace_id)?,
        span_id: SpanId::new(span_id)?,
        parent_span_id: None,
        seq: 1,
        kind: AgentSpanKind::AgentRun,
        name: label.to_string(),
        status: SpanStatus::Ok,
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        model: None,
        cost: None,
        tokens: None,
        input: Some(json!({ "input": label })),
        output: Some(json!({ "output": label })),
        attributes: BTreeMap::new(),
        redaction_class: RedactionClass::Internal,
        idempotency_key: Some(IdempotencyKey::new(idempotency_key)?),
        auth_context: None,
    })
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
                        .get_project_trace(
                            trace_ref.tenant_id,
                            trace_ref.project_id,
                            trace_ref.trace_id,
                        )
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
    api_key: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let (trace_bytes, span_bytes) = smoke_ids();
    let trace_id = lower_hex(&trace_bytes);
    let export = otlp_smoke_export(trace_bytes, span_bytes);
    let started = std::time::Instant::now();
    let protocol = if let Some(otlp_grpc_url) = otlp_grpc_url {
        emit_remote_grpc(
            otlp_grpc_url,
            export,
            &tenant_id,
            &project_id,
            &environment_id,
            api_key,
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
            api_key,
        )
        .await?;
        "http"
    };
    let trace = wait_for_remote_trace(
        &http_url,
        &tenant_id,
        &project_id,
        &environment_id,
        &trace_id,
        StdDuration::from_millis(timeout_ms),
        api_key,
    )
    .await?;
    let trace_query_lag_ms = started.elapsed().as_millis() as u64;
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
        "trace_query_lag_ms": trace_query_lag_ms,
        "trace_span_count": spans.len(),
        "normalizer_version": spans.first().and_then(|span| span.get("normalizer_version")).cloned(),
    }))
}

async fn run_ingest_test(
    http_url: String,
    otlp_grpc_url: Option<String>,
    tenant_id: String,
    project_id: String,
    environment_id: String,
    timeout_ms: u64,
    api_key: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let zero_code_env = zero_code_otlp_env(
        &http_url,
        otlp_grpc_url.as_deref(),
        &tenant_id,
        &project_id,
        &environment_id,
        api_key.is_some(),
    );
    let mut smoke = run_remote_smoke(
        http_url,
        otlp_grpc_url,
        tenant_id,
        project_id,
        environment_id,
        timeout_ms,
        api_key,
    )
    .await?;
    let object = smoke
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("remote ingest smoke did not return an object"))?;
    object.insert("command".to_string(), json!("ingest test"));
    object.insert("zero_code_env".to_string(), zero_code_env);
    Ok(smoke)
}

async fn run_local_quickstart(
    data_dir: PathBuf,
    api_url: &str,
    dashboard_url: &str,
) -> anyhow::Result<serde_json::Value> {
    let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
    let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
    let datasets = SqliteDatasetStore::open(data_dir.join("datasets.sqlite"))?;
    let api_keys = SqliteApiKeyStore::open(data_dir.join("security.sqlite"))?;
    let bus = local_bus(&data_dir)?;
    let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let tenant = TenantId::new("demo")?;
    let project = ProjectId::new("demo")?;
    let environment = EnvironmentId::new("local")?;
    let span_id = SpanId::new("smoke-root")?;

    let api_key = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            environment_id: environment.clone(),
            scopes: [
                ApiScope::TraceWrite,
                ApiScope::TraceRead,
                ApiScope::DatasetWrite,
                ApiScope::EvalRun,
            ]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        })
        .await
        .context("create quickstart api key")?;
    let mut zero_code_env = zero_code_otlp_env(
        api_url,
        None,
        tenant.as_str(),
        project.as_str(),
        environment.as_str(),
        true,
    );
    zero_code_env
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("zero-code env was not an object"))?
        .insert("BEATER_API_KEY".to_string(), json!(api_key.secret.clone()));

    let ingest_outcome = smoke_trace(&service)
        .await
        .context("ingest quickstart smoke trace")?;
    let trace = traces
        .get_trace(tenant.clone(), TraceId::new("smoke-trace")?)
        .await
        .context("read quickstart smoke trace")?;
    let dataset = datasets
        .create_dataset(
            tenant.clone(),
            project.clone(),
            "quickstart-scored-failure".to_string(),
        )
        .await
        .context("create quickstart dataset")?;
    let case = promote_trace_span_to_case(
        tenant.clone(),
        project.clone(),
        dataset.dataset_id.clone(),
        &trace,
        Some(span_id.clone()),
        Some(json!({ "answer": "expected quickstart failure" })),
    )
    .context("promote quickstart trace to failing dataset case")?;
    let case = datasets
        .put_case(case)
        .await
        .context("store quickstart dataset case")?;
    let version = datasets
        .create_version(
            tenant.clone(),
            project.clone(),
            dataset.dataset_id.clone(),
            Some(vec![case.case_id.clone()]),
        )
        .await
        .context("create quickstart dataset version")?;
    let report = evaluate_dataset_version(
        &version,
        DatasetEvalSpec {
            evaluator: EvaluatorSpec {
                id: "exact".to_string(),
                lane: EvaluatorLane::DeterministicWasi,
                kind: EvaluatorKind::ExactMatch,
            },
            evaluator_version_id: EvaluatorVersionId::new("exact-v1")?,
            agent_release_id: AgentReleaseId::new("quickstart-smoke-release")?,
            prompt_version_id: None,
            code_hash: None,
            wasm_hash: None,
        },
    )
    .context("run quickstart exact-match eval")?;
    let report = datasets
        .write_eval_report(report)
        .await
        .context("store quickstart eval report")?;
    let result = report
        .results
        .first()
        .ok_or_else(|| anyhow::anyhow!("quickstart eval produced no results"))?;
    let scored_failure = result.score == 0.0 && result.label.as_deref() == Some("fail");
    if !scored_failure {
        anyhow::bail!(
            "quickstart did not reach a scored failure: score={} label={:?}",
            result.score,
            result.label
        );
    }

    let dashboard_trace_url = format!(
        "{}/?tenant={}&project={}&environment={}&trace={}&span={}",
        trim_url(dashboard_url),
        urlencode(tenant.as_str()),
        urlencode(project.as_str()),
        urlencode(environment.as_str()),
        urlencode(trace.trace_id.as_str()),
        urlencode(span_id.as_str())
    );
    let api_trace_url = format!(
        "{}/v1/traces/{}/{}?project_id={}&environment_id={}",
        trim_url(api_url),
        urlencode(tenant.as_str()),
        urlencode(trace.trace_id.as_str()),
        urlencode(project.as_str()),
        urlencode(environment.as_str())
    );

    Ok(json!({
        "command": "quickstart",
        "mode": "local",
        "source": "native-smoke",
        "tenant_id": tenant,
        "project_id": project,
        "environment_id": environment,
        "api_key": {
            "api_key_id": api_key.record.api_key_id,
            "scopes": api_key.record.scopes,
            "secret": api_key.secret,
        },
        "zero_code_env": zero_code_env,
        "trace": {
            "trace_id": trace.trace_id,
            "span_id": span_id,
            "span_count": trace.spans.len(),
            "ingest_outcome": ingest_outcome,
        },
        "dataset": {
            "dataset_id": dataset.dataset_id,
            "dataset_version_id": version.version_id,
            "case_id": case.case_id,
        },
        "eval": {
            "report_id": report.report_id,
            "evaluator_version_id": report.evaluator_version_id,
            "score": result.score,
            "label": result.label.clone(),
            "evidence": result.evidence.clone(),
            "aggregate_score": report.aggregate_score,
            "result_count": report.result_count,
        },
        "scored_failure": scored_failure,
        "dashboard_url": dashboard_trace_url,
        "api_trace_url": api_trace_url,
    }))
}

fn zero_code_otlp_env(
    http_url: &str,
    otlp_grpc_url: Option<&str>,
    tenant_id: &str,
    project_id: &str,
    environment_id: &str,
    auth_configured: bool,
) -> serde_json::Value {
    let mut headers = Vec::new();
    if otlp_grpc_url.is_some() {
        headers.push(format!("x-beater-tenant-id={tenant_id}"));
        headers.push(format!("x-beater-project-id={project_id}"));
        headers.push(format!("x-beater-environment-id={environment_id}"));
    }
    if auth_configured {
        headers.push("x-beater-api-key=${BEATER_API_KEY}".to_string());
    }

    let mut env = serde_json::Map::new();
    env.insert("BEATER_TENANT_ID".to_string(), json!(tenant_id));
    env.insert("BEATER_PROJECT_ID".to_string(), json!(project_id));
    env.insert("BEATER_ENVIRONMENT_ID".to_string(), json!(environment_id));
    env.insert(
        "OTEL_EXPORTER_OTLP_HEADERS".to_string(),
        json!(headers.join(",")),
    );

    if let Some(grpc_url) = otlp_grpc_url {
        env.insert("OTEL_EXPORTER_OTLP_ENDPOINT".to_string(), json!(grpc_url));
        env.insert("OTEL_EXPORTER_OTLP_PROTOCOL".to_string(), json!("grpc"));
    } else {
        env.insert(
            "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT".to_string(),
            json!(format!(
                "{}/v1/otlp/{}/{}/{}/v1/traces",
                trim_url(http_url),
                urlencode(tenant_id),
                urlencode(project_id),
                urlencode(environment_id)
            )),
        );
        env.insert(
            "OTEL_EXPORTER_OTLP_PROTOCOL".to_string(),
            json!("http/protobuf"),
        );
    }

    serde_json::Value::Object(env)
}

/// Attach `Authorization: Bearer <key>` when an API key is configured.
///
/// Backward compatible: when `api_key` is `None` the request is unchanged.
fn with_bearer(builder: reqwest::RequestBuilder, api_key: Option<&str>) -> reqwest::RequestBuilder {
    match api_key {
        Some(key) => builder.bearer_auth(key),
        None => builder,
    }
}

async fn emit_remote_http(
    http_url: &str,
    tenant_id: &str,
    project_id: &str,
    environment_id: &str,
    body: &[u8],
    api_key: Option<&str>,
) -> anyhow::Result<()> {
    let url = format!(
        "{}/v1/otlp/{}/{}/{}/v1/traces?durability=buffered",
        trim_url(http_url),
        tenant_id,
        project_id,
        environment_id
    );
    let request = reqwest::Client::new()
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body.to_vec());
    with_bearer(request, api_key)
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
    api_key: Option<&str>,
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
    if let Some(key) = api_key {
        request
            .metadata_mut()
            .insert("authorization", metadata_value(&format!("Bearer {key}"))?);
    }
    client
        .export(request)
        .await
        .context("send OTLP gRPC smoke trace")?;
    Ok(())
}

async fn wait_for_remote_trace(
    http_url: &str,
    tenant_id: &str,
    project_id: &str,
    environment_id: &str,
    trace_id: &str,
    timeout: StdDuration,
    api_key: Option<&str>,
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
        let trace = with_bearer(client.get(&url), api_key)
            .header("x-beater-project-id", project_id)
            .header("x-beater-environment-id", environment_id)
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

/// A resolved OpenAPI operation: its HTTP method and path template.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedOperation {
    method: String,
    path_template: String,
}

/// Find the operation with the given `operationId` in the OpenAPI document.
///
/// `spec` is the OpenAPI document serialized to JSON (e.g. from
/// `beater_api::openapi::openapi()`). Returns the HTTP method (uppercased) and
/// the path template (e.g. `/v1/traces/{tenant_id}`).
fn resolve_operation(
    spec: &serde_json::Value,
    operation_id: &str,
) -> anyhow::Result<ResolvedOperation> {
    beater_api::openapi::operations(spec)
        .into_iter()
        .find(|op| op.operation_id == operation_id)
        .map(|op| ResolvedOperation {
            method: op.method.to_ascii_uppercase(),
            path_template: op.path.to_string(),
        })
        .with_context(|| {
            format!("no operation with operationId `{operation_id}` found in the OpenAPI spec")
        })
}

/// Parse `key=value` params into an ordered list of pairs.
fn parse_params(params: &[String]) -> anyhow::Result<Vec<(String, String)>> {
    params
        .iter()
        .map(|raw| {
            raw.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .with_context(|| format!("invalid --param `{raw}`; expected key=value"))
        })
        .collect()
}

/// Substitute `{name}` placeholders in the template with matching params.
///
/// Returns the filled path and the params that were NOT consumed as path
/// segments (those become query-string parameters). Errors if a template
/// placeholder has no matching param.
fn fill_path_template(
    path_template: &str,
    params: &[(String, String)],
) -> anyhow::Result<(String, Vec<(String, String)>)> {
    let mut path = path_template.to_string();
    let mut leftover = Vec::new();
    let mut used = std::collections::BTreeSet::new();

    // Determine which param keys correspond to placeholders.
    for (key, value) in params {
        let placeholder = format!("{{{key}}}");
        if path.contains(&placeholder) {
            path = path.replace(&placeholder, &urlencode(value));
            used.insert(key.clone());
        }
    }
    for (key, value) in params {
        if !used.contains(key) {
            leftover.push((key.clone(), value.clone()));
        }
    }

    // Any remaining `{...}` placeholder means a required path param was missing.
    if let Some(start) = path.find('{')
        && let Some(end) = path[start..].find('}')
    {
        let missing = &path[start + 1..start + end];
        anyhow::bail!("missing --param `{missing}` for path parameter in `{path_template}`");
    }

    Ok((path, leftover))
}

/// Build the full request URL from base, filled path, and query params.
fn build_request_url(base_url: &str, path: &str, query: &[(String, String)]) -> String {
    let mut url = format!("{}{path}", trim_url(base_url));
    if !query.is_empty() {
        let qs = query
            .iter()
            .map(|(k, v)| format!("{}={}", urlencode(k), urlencode(v)))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&qs);
    }
    url
}

/// Resolve `--body` (literal JSON or `@file`) into a JSON value.
fn load_body(body: &str) -> anyhow::Result<serde_json::Value> {
    let raw = if let Some(path) = body.strip_prefix('@') {
        std::fs::read_to_string(path).with_context(|| format!("read --body file `{path}`"))?
    } else {
        body.to_string()
    };
    serde_json::from_str(&raw).context("parse --body as JSON")
}

async fn run_api_call(
    base_url: &str,
    api_key: Option<&str>,
    operation_id: &str,
    params: &[String],
    body: Option<String>,
) -> anyhow::Result<()> {
    let spec =
        serde_json::to_value(beater_api::openapi::openapi()).context("serialize OpenAPI spec")?;
    let op = resolve_operation(&spec, operation_id)?;
    let parsed = parse_params(params)?;
    let (path, query) = fill_path_template(&op.path_template, &parsed)?;
    let url = build_request_url(base_url, &path, &query);

    let method = reqwest::Method::from_bytes(op.method.as_bytes())
        .with_context(|| format!("invalid HTTP method `{}`", op.method))?;
    let is_get = method == reqwest::Method::GET;

    let mut request = reqwest::Client::new().request(method, &url);
    if let Some(body) = body {
        if is_get {
            anyhow::bail!("operation `{operation_id}` is GET; --body is not allowed");
        }
        request = request.json(&load_body(&body)?);
    }
    request = with_bearer(request, api_key);

    let response = request
        .send()
        .await
        .with_context(|| format!("send {} {url}", op.method))?;
    let status = response.status();
    let text = response.text().await.context("read response body")?;

    println!(
        "{} {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or("")
    );
    if !text.is_empty() {
        println!("{text}");
    }
    if !status.is_success() {
        anyhow::bail!("request failed with status {}", status.as_u16());
    }
    Ok(())
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
            p_value: 1.0,
            decision: spec.decision.clone(),
            test: StatisticalTest::PairedT,
            adjusted_alpha: 0.05,
            mde: None,
            required_n: None,
        },
        decision: spec.decision,
        gate_policy: GatePolicy {
            min_sample_size: 1,
            ..GatePolicy::default()
        },
        created_at,
    })
}

/// A deterministic, network-free [`TextGenerator`] for the RSI fixture: it never
/// calls a model and returns a single canned "improved" prompt regardless of the
/// reflective brief it is handed. This keeps `rsi-round-fixture` reproducible and
/// key-free while still exercising the real `LlmRewrite::propose_async` path,
/// which turns the generated text into the candidate's rewritten prompt.
struct CannedRewriteGenerator {
    completion: String,
}

#[async_trait::async_trait]
impl TextGenerator for CannedRewriteGenerator {
    async fn generate(
        &self,
        _req: GenerationRequest,
        _credentials: ProviderCredentials,
    ) -> JudgeProviderResult<GenerationResponse> {
        Ok(GenerationResponse {
            text: self.completion.clone(),
            model: Some("fixture-canned-generator".to_string()),
        })
    }
}

/// A deterministic [`CandidateEvaluator`] for the RSI fixture. It ignores the
/// opaque case payloads and emits seeded, reproducible paired baseline-vs-candidate
/// [`CaseScore`]s: a uniform lift (`baseline_score` → `candidate_score`) on every
/// split, so the candidate generalizes — the same lift on the held-out **Test**
/// split as on the optimization (Train/Val) split. That clears both the held-out
/// gate (a real paired improvement) and the anti-overfit guardrail (no gap),
/// demonstrating the loop accepting a generalizing candidate. The overfit-rejection
/// path (good only on the optimization split) is unit-tested in beater-experiments.
struct FixtureCandidateEvaluator {
    /// Number of optimization-split (Train/Val) cases to score.
    optimize_cases: usize,
    /// Number of held-out Test cases to score.
    test_cases: usize,
    /// Baseline score applied to every case (the current policy).
    baseline_score: f64,
    /// Candidate score applied to every case (the proposed policy).
    candidate_score: f64,
}

#[async_trait::async_trait]
impl CandidateEvaluator for FixtureCandidateEvaluator {
    async fn evaluate(
        &self,
        _candidate: &CandidateChange,
        _cases: &[serde_json::Value],
    ) -> Result<Vec<CaseScore>, String> {
        let mut scores = Vec::with_capacity(self.optimize_cases + self.test_cases);
        for i in 0..self.optimize_cases {
            // Alternate Train/Val; both count as the optimization split for the
            // generalization-gap assessment.
            let split = if i % 2 == 0 { Split::Train } else { Split::Val };
            scores.push(CaseScore {
                split,
                baseline_score: self.baseline_score,
                candidate_score: self.candidate_score,
                covariate: None,
            });
        }
        for _ in 0..self.test_cases {
            scores.push(CaseScore {
                split: Split::Test,
                baseline_score: self.baseline_score,
                candidate_score: self.candidate_score,
                covariate: None,
            });
        }
        Ok(scores)
    }
}

/// Drive one deterministic, seeded RSI optimization round and build the JSON
/// report. Proves the propose → evaluate → held-out gate + anti-overfit loop
/// end-to-end with no network and no LLM key.
async fn run_rsi_round_fixture() -> anyhow::Result<serde_json::Value> {
    // Seeded failing examples motivating the round (deterministic excerpts).
    let failures = vec![
        FailureExample::from_parts(
            "Who wrote the novel '1984'?",
            Some("George Orwell".to_string()),
            "I am not certain, possibly Aldous Huxley.",
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
            "I think it has one moon.",
            0.0,
            None,
        ),
    ];

    // The injected, network-free proposer: a canned improved system prompt that
    // the real `LlmRewrite::propose_async` path turns into the candidate.
    let generator = CannedRewriteGenerator {
        completion: "You are a meticulous factual assistant. Answer only when confident, cite \
                     the specific source for each factual claim, and say 'I am not sure' when \
                     you cannot verify an answer."
            .to_string(),
    };

    // 6 optimization cases + 6 held-out Test cases.
    let optimize_cases = 6usize;
    let test_cases = 6usize;
    let evaluator = FixtureCandidateEvaluator {
        optimize_cases,
        test_cases,
        // Uniform 0.5 → 0.9 lift everywhere: a generalizing candidate.
        baseline_score: 0.5,
        candidate_score: 0.9,
    };

    let cfg = OptimizationRoundConfig::new(
        "reduce hallucinations on factual lookups",
        "You are a helpful assistant.",
        failures,
        // Opaque case payloads; the fixture evaluator ignores their contents.
        (0..(optimize_cases + test_cases))
            .map(|i| json!({ "case": i }))
            .collect(),
        OptimizerStrategy::LlmRewrite,
        GatePolicy {
            min_sample_size: 6,
            max_regression: 0.0,
            alpha: 0.05,
            comparison_count: 1,
        },
    );

    let outcome = run_optimization_round(
        cfg,
        &generator,
        // No real key needed: the canned generator never reads credentials.
        ProviderCredentials::new("fixture", "sk-fixture-no-network"),
        &evaluator,
    )
    .await
    .context("run rsi optimization round fixture")?;

    let evaluated: Vec<serde_json::Value> = outcome
        .evaluated
        .iter()
        .map(|evaluation| {
            json!({
                "candidate": {
                    "kind": evaluation.candidate.kind,
                    "target": evaluation.candidate.target,
                    "description": evaluation.candidate.description,
                    "proposed_by": evaluation.candidate.proposed_by,
                },
                "gate_decision": evaluation.gate.decision.name(),
                "gate": {
                    "sample_size": evaluation.gate.sample_size,
                    "baseline_mean": evaluation.gate.baseline_mean,
                    "candidate_mean": evaluation.gate.candidate_mean,
                    "delta": evaluation.gate.delta,
                    "ci_low": evaluation.gate.ci_low,
                    "ci_high": evaluation.gate.ci_high,
                    "p_value": evaluation.gate.p_value,
                },
                "overfit_flag": evaluation.overfit.overfit,
                "overfit": {
                    "optimize_lift": evaluation.overfit.optimize_lift,
                    "holdout_lift": evaluation.overfit.holdout_lift,
                    "gap": evaluation.overfit.gap,
                    "gap_ci_low": evaluation.overfit.gap_ci_low,
                    "gap_ci_high": evaluation.overfit.gap_ci_high,
                },
                "accepted": evaluation.accepted,
            })
        })
        .collect();

    Ok(json!({
        "accepted_candidate": outcome.accepted.map(|candidate| json!({
            "kind": candidate.kind,
            "target": candidate.target,
            "description": candidate.description,
            "proposed_by": candidate.proposed_by,
        })),
        "evaluated": evaluated,
    }))
}

/// A seeded factual Q/A case for the live RSI round: a `question`, its
/// canonical `expected_answer`, and the split it belongs to. The model is
/// re-prompted with the baseline and candidate system prompts on each question
/// and graded deterministically against `expected_answer`.
#[derive(Clone, Debug)]
struct FactualCase {
    question: String,
    expected_answer: String,
    split: Split,
}

/// Wraps a [`TextGenerator`] and forces every outgoing [`GenerationRequest`] to
/// use a fixed model id, regardless of what the caller put in `req.model`.
///
/// `LlmRewrite::propose_async` hardcodes its rewrite model (`LLM_REWRITE_MODEL`
/// = an OpenAI id) in the `GenerationRequest` it builds, so to route the live
/// proposer through Anthropic with the operator-chosen `--model` we override the
/// model on the way through. The honest seam (`generate`) is otherwise untouched.
struct ModelForcingGenerator<G: TextGenerator> {
    inner: G,
    model: String,
}

#[async_trait::async_trait]
impl<G: TextGenerator> TextGenerator for ModelForcingGenerator<G> {
    async fn generate(
        &self,
        mut req: GenerationRequest,
        credentials: ProviderCredentials,
    ) -> JudgeProviderResult<GenerationResponse> {
        req.model = self.model.clone();
        self.inner.generate(req, credentials).await
    }
}

/// Tokenize a string for deterministic grading.
///
/// Normalization rules (single-sourced so the answer and expected strings are
/// treated identically):
/// 1. Lowercase the whole string (case-insensitive matching).
/// 2. Strip commas that sit *between* two ASCII digits, so a thousands-separated
///    number like `299,792` collapses to the bare token `299792` and matches a
///    plain `299792` answer.
/// 3. Split on any character that is not ASCII-alphanumeric (whitespace and all
///    other punctuation become separators), so `Canberra.` yields `canberra`.
/// 4. Drop empty tokens produced by runs of separators.
///
/// Digits and letters survive; everything else is a boundary. The result is the
/// ordered list of word/number tokens in the string.
fn grading_tokens(s: &str) -> Vec<String> {
    // Lowercase once, then drop digit-grouping commas (e.g. "299,792" -> "299792")
    // before tokenizing so a comma between digits never splits a number.
    let lowered = s.to_lowercase();
    let chars: Vec<char> = lowered.chars().collect();
    let mut cleaned = String::with_capacity(lowered.len());
    for (i, &c) in chars.iter().enumerate() {
        let comma_between_digits = c == ','
            && i > 0
            && chars[i - 1].is_ascii_digit()
            && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        if !comma_between_digits {
            cleaned.push(c);
        }
    }
    cleaned
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|tok| !tok.is_empty())
        .map(str::to_string)
        .collect()
}

/// Token-boundary grading: `1.0` iff the `expected` answer's token sequence
/// appears as a CONTIGUOUS sub-slice of the `answer`'s token sequence, else
/// `0.0`. An empty `expected` always scores `0.0`.
///
/// Both strings are normalized via [`grading_tokens`] (lowercased, digit-comma
/// stripped, split on non-alphanumerics). Matching on whole tokens — not raw
/// substrings — kills the substring false-positives that plagued the old grader:
/// expected `"Au"` no longer matches inside `"because"`/`"Australia"`, `"Na"` no
/// longer matches inside `"national"`, and `"Two"` no longer matches inside
/// `"between"`/`"network"`. It also fixes the false-negative where `"299792"`
/// failed to match an answer that wrote `"299,792"`. Multi-word expecteds
/// (`"Mount Everest"`, `"Leonardo da Vinci"`) match only as consecutive tokens.
///
/// Deliberately simple and deterministic so the score reflects the model's
/// answer, not a second LLM's opinion of it.
fn token_match_score(answer: &str, expected: &str) -> f64 {
    let answer_tokens = grading_tokens(answer);
    let expected_tokens = grading_tokens(expected);
    if expected_tokens.is_empty() {
        return 0.0;
    }
    let matched = answer_tokens
        .windows(expected_tokens.len())
        .any(|window| window == expected_tokens.as_slice());
    if matched { 1.0 } else { 0.0 }
}

/// A live [`CandidateEvaluator`] that closes the "synthetic scores" gap: for each
/// seeded factual case it calls the SAME Anthropic model once with the baseline
/// system prompt + the question and once with the candidate's system prompt
/// (`candidate.target`) + the question, then grades each answer deterministically
/// against the expected answer. Calls are sequential (modest concurrency) and use
/// a small `max_tokens`.
struct ModelCandidateEvaluator<G: TextGenerator> {
    generator: G,
    credentials: ProviderCredentials,
    model: String,
    baseline_prompt: String,
    cases: Vec<FactualCase>,
}

impl<G: TextGenerator> ModelCandidateEvaluator<G> {
    /// Ask the model `question` under `system_prompt` and return its raw answer.
    async fn answer(&self, system_prompt: &str, question: &str) -> Result<String, String> {
        let req = GenerationRequest::new(self.model.clone(), question)
            .with_system(system_prompt)
            .with_temperature(0.0)
            .with_max_tokens(256);
        let resp = self
            .generator
            .generate(req, self.credentials.clone())
            .await
            .map_err(|err| err.to_string())?;
        Ok(resp.text)
    }
}

#[async_trait::async_trait]
impl<G: TextGenerator> CandidateEvaluator for ModelCandidateEvaluator<G> {
    async fn evaluate(
        &self,
        candidate: &CandidateChange,
        // The opaque case payloads passed to the round are ignored; the seeded
        // `FactualCase`s (with expected answers) are the source of truth here.
        _cases: &[serde_json::Value],
    ) -> Result<Vec<CaseScore>, String> {
        // The candidate's rewritten system prompt lives in `target` for the
        // SystemPrompt change kind produced by LlmRewrite.
        let candidate_prompt = candidate.target.as_str();
        let mut scores = Vec::with_capacity(self.cases.len());
        // Sequential to keep it simple and gentle on rate limits.
        for case in &self.cases {
            let baseline_answer = self.answer(&self.baseline_prompt, &case.question).await?;
            let candidate_answer = self.answer(candidate_prompt, &case.question).await?;
            scores.push(CaseScore {
                split: case.split,
                baseline_score: token_match_score(&baseline_answer, &case.expected_answer),
                candidate_score: token_match_score(&candidate_answer, &case.expected_answer),
                covariate: None,
            });
        }
        Ok(scores)
    }
}

/// Seeded factual Q/A cases for the live RSI round: ~6 Train/Val optimization
/// cases + ~6 held-out Test cases. The Test set is genuinely held out (different
/// questions than the optimization split) so acceptance is a real generalization
/// check, not memorization of the optimization questions.
fn live_factual_cases() -> Vec<FactualCase> {
    let optimize = [
        ("Who wrote the novel '1984'?", "George Orwell"),
        ("What is the capital of Australia?", "Canberra"),
        ("How many moons does Mars have?", "Two"),
        ("What is the chemical symbol for gold?", "Au"),
        ("In what year did the first human land on the Moon?", "1969"),
        ("What is the largest planet in our solar system?", "Jupiter"),
    ];
    let test = [
        ("Who painted the Mona Lisa?", "Leonardo da Vinci"),
        ("What is the capital of Canada?", "Ottawa"),
        ("What is the tallest mountain on Earth?", "Mount Everest"),
        ("What is the chemical symbol for sodium?", "Na"),
        ("How many continents are there on Earth?", "Seven"),
        (
            "What is the speed of light in a vacuum (approx, km/s)?",
            "299792",
        ),
    ];
    let mut cases = Vec::with_capacity(optimize.len() + test.len());
    for (i, (question, expected)) in optimize.iter().enumerate() {
        // Alternate Train/Val; both count as the optimization split.
        let split = if i % 2 == 0 { Split::Train } else { Split::Val };
        cases.push(FactualCase {
            question: (*question).to_string(),
            expected_answer: (*expected).to_string(),
            split,
        });
    }
    for (question, expected) in test {
        cases.push(FactualCase {
            question: question.to_string(),
            expected_answer: expected.to_string(),
            split: Split::Test,
        });
    }
    cases
}

/// Build the Anthropic provider config for the live RSI round, honoring an
/// optional `BEATER_ANTHROPIC_BASE_URL` override.
///
/// Default: the real `https://api.anthropic.com/v1/messages` endpoint. When the
/// env var is set (and non-empty) its value replaces `endpoint_url`, so the live
/// `rsi-round` command can be driven against a mock Anthropic server in tests/CI
/// without real network — every other field (cost cap, retry policy) and the BYOK
/// key gate are unchanged. This is a testing seam, not a feature surface.
fn anthropic_provider_config() -> HttpJudgeProviderConfig {
    let mut config = HttpJudgeProviderConfig::anthropic_default();
    if let Some(base_url) = std::env::var("BEATER_ANTHROPIC_BASE_URL")
        .ok()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
    {
        config.endpoint_url = base_url;
    }
    config
}

/// Drive one LIVE, Anthropic-backed RSI optimization round end-to-end and build
/// the same JSON report shape as `run_rsi_round_fixture`, plus the model used.
///
/// REAL Anthropic calls: the proposer rewrite and the per-case baseline/candidate
/// answers all hit `https://api.anthropic.com/v1/messages`. Requires a BYOK key in
/// `ANTHROPIC_API_KEY`; the no-network proof remains `rsi-round-fixture`.
async fn run_rsi_round_live(
    model: String,
    goal: String,
    current_prompt: String,
) -> anyhow::Result<serde_json::Value> {
    // BYOK gate: a clean error, not a panic, and no network call without a key.
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "rsi-round needs a real Anthropic key: set ANTHROPIC_API_KEY (BYOK). For a \
                 no-network demo use `beaterctl rsi-round-fixture`."
            )
        })?;
    let credentials = ProviderCredentials::new("anthropic", api_key);

    // Real Anthropic provider, used as both the proposer's TextGenerator and the
    // evaluator's model. The model-forcing wrapper makes LlmRewrite's hardcoded
    // proposer model resolve to the operator-chosen `--model`.
    //
    // The endpoint defaults to the real Anthropic messages API. A test/dev harness
    // can point it at a mock server with `BEATER_ANTHROPIC_BASE_URL` to exercise
    // this exact live path end-to-end without real network — the URL replaces the
    // default `endpoint_url`, nothing else changes (same BYOK key gate, same
    // request shaping). Unset (the default) keeps the real API.
    let provider = AnthropicJudgeProvider::new(anthropic_provider_config());
    let proposer = ModelForcingGenerator {
        inner: provider.clone(),
        model: model.clone(),
    };

    // The seeded failing examples motivating the round (the same factual scenario
    // the fixture uses), used to build the reflective brief for the proposer.
    let failures = vec![
        FailureExample::from_parts(
            "Who wrote the novel '1984'?",
            Some("George Orwell".to_string()),
            "I am not certain, possibly Aldous Huxley.",
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
            "I think it has one moon.",
            0.0,
            None,
        ),
    ];

    let cases = live_factual_cases();
    let optimize_cases = cases.iter().filter(|c| c.split != Split::Test).count();
    let test_cases = cases.iter().filter(|c| c.split == Split::Test).count();

    let evaluator = ModelCandidateEvaluator {
        generator: provider,
        credentials: credentials.clone(),
        model: model.clone(),
        baseline_prompt: current_prompt.clone(),
        cases,
    };

    let cfg = OptimizationRoundConfig::new(
        goal,
        current_prompt,
        failures,
        // Opaque case payloads (the evaluator uses its own seeded FactualCases);
        // sized to match so the round's bookkeeping lines up with the scores.
        (0..(optimize_cases + test_cases))
            .map(|i| json!({ "case": i }))
            .collect(),
        OptimizerStrategy::LlmRewrite,
        GatePolicy {
            min_sample_size: 6,
            max_regression: 0.0,
            alpha: 0.05,
            comparison_count: 1,
        },
    );

    let outcome = run_optimization_round(cfg, &proposer, credentials, &evaluator)
        .await
        .context("run live anthropic-backed rsi optimization round")?;

    let evaluated: Vec<serde_json::Value> = outcome
        .evaluated
        .iter()
        .map(|evaluation| {
            json!({
                "candidate": {
                    "kind": evaluation.candidate.kind,
                    "target": evaluation.candidate.target,
                    "description": evaluation.candidate.description,
                    "proposed_by": evaluation.candidate.proposed_by,
                },
                "gate_decision": evaluation.gate.decision.name(),
                "gate": {
                    "sample_size": evaluation.gate.sample_size,
                    "baseline_mean": evaluation.gate.baseline_mean,
                    "candidate_mean": evaluation.gate.candidate_mean,
                    "delta": evaluation.gate.delta,
                    "ci_low": evaluation.gate.ci_low,
                    "ci_high": evaluation.gate.ci_high,
                    "p_value": evaluation.gate.p_value,
                },
                "overfit_flag": evaluation.overfit.overfit,
                "overfit": {
                    "optimize_lift": evaluation.overfit.optimize_lift,
                    "holdout_lift": evaluation.overfit.holdout_lift,
                    "gap": evaluation.overfit.gap,
                    "gap_ci_low": evaluation.overfit.gap_ci_low,
                    "gap_ci_high": evaluation.overfit.gap_ci_high,
                },
                "accepted": evaluation.accepted,
            })
        })
        .collect();

    Ok(json!({
        "model": model,
        "accepted_candidate": outcome.accepted.map(|candidate| json!({
            "kind": candidate.kind,
            "target": candidate.target,
            "description": candidate.description,
            "proposed_by": candidate.proposed_by,
        })),
        "evaluated": evaluated,
    }))
}

/// Emit a completed RSI optimization `round_report` as ONE canonical Beater
/// trace into the `--data-dir` store, then read it back to prove it landed —
/// dogfooding the exact ingest path `smoke` uses.
///
/// Construction mirrors [`beater_ingest::smoke_trace`]: an [`IngestService`] is
/// built over the same `(FsArtifactStore, SqliteTraceStore, local SqliteDurableBus,
/// IngestPolicy::default())` quartet, and each span is sent through
/// [`IngestService::ingest_native`] as a [`NativeIngestRequest`] (the service
/// builds the `RawEnvelope` + `CanonicalTraceBatch` internally, exactly as
/// `smoke_trace` relies on). The emitted shape is:
///   * root span `AgentRun` ("rsi optimization round") with goal/strategy/model/
///     n_candidates/accepted attributes,
///   * one `AgentPlan` proposal child carrying the candidate target/description,
///   * one `EvaluatorRun` child per evaluated candidate carrying the gate delta,
///     p-value, decision, overfit flag, and acceptance.
///
/// The trace is then read back with `traces.get_trace(...)` (as the smoke
/// fixtures do) and `trace_id` + `trace_span_count` are injected into
/// `round_report`.
async fn record_rsi_round_trace(
    data_dir: &Path,
    command: &str,
    model: &str,
    goal: &str,
    round_report: &mut serde_json::Value,
) -> anyhow::Result<()> {
    // Same store quartet `smoke` builds the IngestService over.
    let artifacts = Arc::new(FsArtifactStore::new(data_dir.join("artifacts"))?);
    let traces = Arc::new(SqliteTraceStore::open(data_dir.join("traces.sqlite"))?);
    let bus = local_bus(data_dir)?;
    let service = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());

    let tenant = TenantId::new("demo")?;
    let project = ProjectId::new("demo")?;
    let environment = EnvironmentId::new("local")?;
    let scope = TenantScope::new(tenant.clone(), project.clone(), environment.clone());

    // A unique, whitespace-free trace id so repeated runs into the same data-dir
    // never collide (the smoke fixture reuses a fixed id; an RSI round is a
    // distinct, repeatable event).
    let nonce = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let trace_id = TraceId::new(format!("rsi-round-{nonce}"))?;
    let root_span_id = SpanId::new(format!("rsi-root-{nonce}"))?;

    let accepted_candidate = round_report.get("accepted_candidate").cloned();
    let accepted = accepted_candidate
        .as_ref()
        .is_some_and(|value| value.is_object());
    let evaluated = round_report
        .get("evaluated")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    // --- root span: AgentRun ------------------------------------------------
    let root_attributes: CanonicalAttrs = [
        ("rsi.command".to_string(), json!(command)),
        ("rsi.goal".to_string(), json!(goal)),
        ("rsi.strategy".to_string(), json!("llm_rewrite")),
        ("rsi.model".to_string(), json!(model)),
        ("rsi.n_candidates".to_string(), json!(evaluated.len())),
        ("rsi.accepted".to_string(), json!(accepted)),
    ]
    .into_iter()
    .collect();
    service
        .ingest_native(NativeIngestRequest {
            scope: scope.clone(),
            trace_id: trace_id.clone(),
            span_id: root_span_id.clone(),
            parent_span_id: None,
            seq: 1,
            kind: AgentSpanKind::AgentRun,
            name: "rsi optimization round".to_string(),
            status: SpanStatus::Ok,
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            model: None,
            cost: None,
            tokens: None,
            input: Some(json!({ "goal": goal, "command": command })),
            output: Some(json!({ "accepted": accepted })),
            attributes: root_attributes,
            redaction_class: RedactionClass::Internal,
            idempotency_key: None,
            auth_context: None,
        })
        .await
        .context("ingest rsi round root span")?;

    // --- proposal child: AgentPlan -----------------------------------------
    // The proposed candidate is the (single) evaluated candidate; fall back to the
    // accepted candidate if present.
    let proposal_candidate = evaluated
        .first()
        .and_then(|evaluation| evaluation.get("candidate").cloned())
        .or(accepted_candidate.clone());
    let mut seq: u64 = 2;
    service
        .ingest_native(NativeIngestRequest {
            scope: scope.clone(),
            trace_id: trace_id.clone(),
            span_id: SpanId::new(format!("rsi-proposal-{nonce}"))?,
            parent_span_id: Some(root_span_id.clone()),
            seq,
            kind: AgentSpanKind::AgentPlan,
            name: "propose candidate".to_string(),
            status: SpanStatus::Ok,
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            model: None,
            cost: None,
            tokens: None,
            input: Some(json!({ "goal": goal })),
            output: proposal_candidate.clone(),
            attributes: [
                (
                    "rsi.candidate.kind".to_string(),
                    proposal_candidate
                        .as_ref()
                        .and_then(|c| c.get("kind").cloned())
                        .unwrap_or(json!(null)),
                ),
                (
                    "rsi.candidate.target".to_string(),
                    proposal_candidate
                        .as_ref()
                        .and_then(|c| c.get("target").cloned())
                        .unwrap_or(json!(null)),
                ),
                (
                    "rsi.candidate.description".to_string(),
                    proposal_candidate
                        .as_ref()
                        .and_then(|c| c.get("description").cloned())
                        .unwrap_or(json!(null)),
                ),
            ]
            .into_iter()
            .collect(),
            redaction_class: RedactionClass::Internal,
            idempotency_key: None,
            auth_context: None,
        })
        .await
        .context("ingest rsi round proposal span")?;

    // --- one EvaluatorRun child per evaluated candidate --------------------
    for (index, evaluation) in evaluated.iter().enumerate() {
        seq += 1;
        let gate = evaluation.get("gate").cloned().unwrap_or(json!({}));
        let attributes: CanonicalAttrs = [
            (
                "rsi.gate.decision".to_string(),
                evaluation
                    .get("gate_decision")
                    .cloned()
                    .unwrap_or(json!(null)),
            ),
            (
                "rsi.gate.delta".to_string(),
                gate.get("delta").cloned().unwrap_or(json!(null)),
            ),
            (
                "rsi.gate.p_value".to_string(),
                gate.get("p_value").cloned().unwrap_or(json!(null)),
            ),
            (
                "rsi.overfit_flag".to_string(),
                evaluation
                    .get("overfit_flag")
                    .cloned()
                    .unwrap_or(json!(null)),
            ),
            (
                "rsi.accepted".to_string(),
                evaluation.get("accepted").cloned().unwrap_or(json!(null)),
            ),
        ]
        .into_iter()
        .collect();
        service
            .ingest_native(NativeIngestRequest {
                scope: scope.clone(),
                trace_id: trace_id.clone(),
                span_id: SpanId::new(format!("rsi-eval-{nonce}-{index}"))?,
                parent_span_id: Some(root_span_id.clone()),
                seq,
                kind: AgentSpanKind::EvaluatorRun,
                name: "evaluate candidate".to_string(),
                status: SpanStatus::Ok,
                start_time: Some(Utc::now()),
                end_time: Some(Utc::now()),
                model: None,
                cost: None,
                tokens: None,
                input: evaluation.get("candidate").cloned(),
                output: Some(json!({
                    "gate_decision": evaluation.get("gate_decision"),
                    "accepted": evaluation.get("accepted"),
                })),
                attributes,
                redaction_class: RedactionClass::Internal,
                idempotency_key: None,
                auth_context: None,
            })
            .await
            .context("ingest rsi round evaluator span")?;
    }

    // Read the trace back from the store, exactly as the smoke fixtures do, to
    // confirm it landed and is queryable.
    let trace = traces
        .get_trace(tenant.clone(), trace_id.clone())
        .await
        .context("read back recorded rsi round trace")?;

    if let Some(object) = round_report.as_object_mut() {
        object.insert("trace_id".to_string(), json!(trace_id.as_str()));
        object.insert("trace_span_count".to_string(), json!(trace.spans.len()));
        object.insert(
            "trace_data_dir".to_string(),
            json!(data_dir.display().to_string()),
        );
    }
    Ok(())
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
        cache_namespace: None,
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
        _batch: Arc<CanonicalTraceBatch>,
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

    async fn get_project_trace(
        &self,
        _tenant: TenantId,
        _project: ProjectId,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> anyhow::Result<serde_json::Value> {
        serde_json::to_value(beater_api::openapi::openapi()).context("serialize spec")
    }

    #[test]
    fn resolves_list_traces_operation() -> anyhow::Result<()> {
        let op = resolve_operation(&spec()?, "listTraces")?;
        assert_eq!(op.method, "GET");
        assert_eq!(op.path_template, "/v1/traces/{tenant_id}");
        Ok(())
    }

    #[test]
    fn substitutes_path_param_and_keeps_query() -> anyhow::Result<()> {
        let params = parse_params(&["tenant_id=acme".to_string(), "limit=50".to_string()])?;
        let (path, query) = fill_path_template("/v1/traces/{tenant_id}", &params)?;
        assert_eq!(path, "/v1/traces/acme");
        assert_eq!(query, vec![("limit".to_string(), "50".to_string())]);
        Ok(())
    }

    #[test]
    fn end_to_end_url_for_list_traces() -> anyhow::Result<()> {
        let op = resolve_operation(&spec()?, "listTraces")?;
        let params = parse_params(&["tenant_id=acme".to_string()])?;
        let (path, query) = fill_path_template(&op.path_template, &params)?;
        let url = build_request_url("http://127.0.0.1:8080/", &path, &query);
        assert_eq!(url, "http://127.0.0.1:8080/v1/traces/acme");
        Ok(())
    }

    #[test]
    fn missing_path_param_errors() -> anyhow::Result<()> {
        let params = parse_params(&[])?;
        let err = match fill_path_template("/v1/traces/{tenant_id}", &params) {
            Ok(filled) => anyhow::bail!("expected missing-param error, got {filled:?}"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("tenant_id"), "got: {err}");
        Ok(())
    }

    #[test]
    fn unknown_operation_id_errors() -> anyhow::Result<()> {
        let err = match resolve_operation(&spec()?, "doesNotExist") {
            Ok(op) => anyhow::bail!("expected resolution failure, got {op:?}"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("doesNotExist"), "got: {err}");
        Ok(())
    }

    // --- token_match_score: token-boundary grading edge cases ---------------
    //
    // Each assertion locks down a brittleness the old `substring_match_score`
    // exhibited: substring false-positives ("Au" in "because"/"Australia",
    // "Na" in "national", "Two" in "between"/"network") and the thousands-comma
    // false-negative ("299792" vs "299,792").

    #[test]
    fn token_match_au_is_a_whole_token_not_a_substring() {
        // True positive: "Au" stands alone as a token after stripping the period.
        assert_eq!(token_match_score("The symbol is Au.", "Au"), 1.0);
        // False positive killed: "au" lives inside "because" but is not its own token.
        assert_eq!(
            token_match_score("Gold is used because it shines", "Au"),
            0.0
        );
        // False positive killed: "Au" inside "Australia" must not match.
        assert_eq!(token_match_score("I visited Australia once", "Au"), 0.0);
    }

    #[test]
    fn token_match_na_is_not_matched_inside_national() {
        // False positive killed: "na" inside "national" is not a standalone token.
        assert_eq!(token_match_score("We sang the national anthem", "Na"), 0.0);
        // True positive: "Na" as its own token (sodium symbol) matches.
        assert_eq!(token_match_score("The symbol is Na.", "Na"), 1.0);
    }

    #[test]
    fn token_match_two_is_not_matched_inside_between_or_network() {
        // False positives killed: "two" inside "between" and "network".
        assert_eq!(token_match_score("It sits between the poles", "Two"), 0.0);
        assert_eq!(token_match_score("routed over the network", "Two"), 0.0);
        // True positive: a standalone "two" token matches.
        assert_eq!(token_match_score("It has two moons", "Two"), 1.0);
    }

    #[test]
    fn token_match_number_ignores_thousands_comma() {
        // False negative fixed: "299,792" normalizes to the bare token "299792".
        assert_eq!(
            token_match_score("light travels about 299,792 km/s", "299792"),
            1.0
        );
        // And a plainly written number still matches.
        assert_eq!(token_match_score("about 299792 km/s", "299792"), 1.0);
        // A genuinely different number does not match.
        assert_eq!(token_match_score("about 300000 km/s", "299792"), 0.0);
    }

    #[test]
    fn token_match_multiword_requires_consecutive_tokens() {
        // True positive: consecutive tokens, even with trailing punctuation/number.
        assert_eq!(
            token_match_score("The tallest is Mount Everest, at 8849m", "Mount Everest"),
            1.0
        );
        // True positive: three consecutive tokens.
        assert_eq!(
            token_match_score("Painted by Leonardo da Vinci in Italy", "Leonardo da Vinci"),
            1.0
        );
        // False positive killed: the same tokens out of order / not contiguous.
        assert_eq!(
            token_match_score("Everest is a mountain; mount it carefully", "Mount Everest"),
            0.0
        );
    }

    #[test]
    fn token_match_canberra_after_punctuation() {
        // Punctuation stripping lets "Canberra" match after a sentence boundary.
        assert_eq!(
            token_match_score("The capital is Canberra.", "Canberra"),
            1.0
        );
    }

    #[test]
    fn token_match_is_case_insensitive() {
        assert_eq!(
            token_match_score("george ORWELL wrote it", "George Orwell"),
            1.0
        );
        assert_eq!(token_match_score("GEORGE orwell", "george ORWELL"), 1.0);
    }

    #[test]
    fn token_match_ignores_leading_and_trailing_punctuation() {
        assert_eq!(token_match_score("...Canberra!!!", "canberra"), 1.0);
        assert_eq!(token_match_score("'Au'", "au"), 1.0);
    }

    #[test]
    fn token_match_clearly_wrong_answer_scores_zero() {
        assert_eq!(token_match_score("The capital is Sydney.", "Canberra"), 0.0);
        assert_eq!(
            token_match_score("Possibly Aldous Huxley wrote it", "George Orwell"),
            0.0
        );
    }

    #[test]
    fn token_match_empty_expected_scores_zero() {
        // Empty expected (or all-punctuation expected) yields no tokens -> 0.0,
        // so a vacuous expected never trivially "matches" every answer.
        assert_eq!(token_match_score("anything at all", ""), 0.0);
        assert_eq!(token_match_score("anything at all", "  ,. "), 0.0);
    }
}
