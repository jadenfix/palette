use anyhow::Context;
use beater_api::{router, ApiState};
use beater_archive::ParquetTraceArchive;
use beater_audit::SqliteAuditStore;
use beater_auth::SqliteApiKeyStore;
use beater_bus::{DurableBus, InMemoryBus, SqliteDurableBus};
use beater_calibration::SqliteCalibrationStore;
use beater_core::Money;
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_ingest::{IngestPolicy, IngestService, QueuedTraceWork};
use beater_judge::{
    HttpRoutingJudgeProvider, JudgeBrokerService, JudgeProvider, KeywordJudgeProvider,
    SqliteJudgeLedger,
};
use beater_otlp::{OtlpGrpcTraceService, TraceServiceServer};
use beater_search::{SearchIndex, TantivySearchIndex};
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store::TraceStore;
use beater_store_obj::FsArtifactStore;
use beater_store_sql::{SqliteMetadataStore, SqliteQuotaLimiter, SqliteTraceStore};
use beater_usage::SqliteUsageLedger;
use clap::{Parser, ValueEnum};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;

#[derive(Debug, Parser)]
#[command(
    name = "beaterd",
    about = "All-in-one Beater agent observability server"
)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: SocketAddr,
    #[arg(long, default_value = "127.0.0.1:4317")]
    otlp_grpc_addr: SocketAddr,
    #[arg(long, default_value = "demo")]
    default_tenant_id: String,
    #[arg(long, default_value = "demo")]
    default_project_id: String,
    #[arg(long, default_value = "local")]
    default_environment_id: String,
    #[arg(long, default_value = ".beater")]
    data_dir: PathBuf,
    #[arg(long, default_value_t = 1024)]
    bus_capacity: usize,
    #[arg(long, value_enum, default_value_t = BusBackendArg::Sqlite)]
    bus_backend: BusBackendArg,
    #[arg(long, value_enum, default_value_t = AuthModeArg::Local)]
    auth_mode: AuthModeArg,
    #[arg(long, env = "BEATER_PROVIDER_SECRET_KEY")]
    provider_secret_key: Option<String>,
    #[arg(long, value_enum, default_value_t = JudgeProviderArg::Keyword)]
    judge_provider: JudgeProviderArg,
    #[arg(long, env = "BEATER_JUDGE_BUDGET_MICROS", default_value_t = 1_000_000)]
    judge_budget_micros: i64,
    #[arg(
        long,
        env = "BEATER_TRACE_WRITE_DRAIN_INTERVAL_MS",
        default_value_t = 1000
    )]
    trace_write_drain_interval_ms: u64,
    #[arg(
        long,
        env = "BEATER_TRACE_INGESTED_DRAIN_INTERVAL_MS",
        default_value_t = 1000
    )]
    trace_ingested_drain_interval_ms: u64,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum AuthModeArg {
    Local,
    Required,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BusBackendArg {
    Sqlite,
    Memory,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum JudgeProviderArg {
    Keyword,
    HttpRouting,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let artifacts = Arc::new(FsArtifactStore::new(args.data_dir.join("artifacts"))?);
    let traces = Arc::new(SqliteTraceStore::open(args.data_dir.join("traces.sqlite"))?);
    let quota_limiter = Arc::new(SqliteQuotaLimiter::open(
        args.data_dir.join("quotas.sqlite"),
    )?);
    let metadata = Arc::new(SqliteMetadataStore::open(
        args.data_dir.join("metadata.sqlite"),
    )?);
    let search = Arc::new(TantivySearchIndex::open_or_create(
        args.data_dir.join("search"),
    )?);
    let archive = ParquetTraceArchive::new(args.data_dir.join("archive"))?;
    let datasets = Arc::new(SqliteDatasetStore::open(
        args.data_dir.join("datasets.sqlite"),
    )?);
    let experiments = Arc::new(SqliteExperimentStore::open(
        args.data_dir.join("experiments.sqlite"),
    )?);
    let gates = Arc::new(SqliteGateStore::open(args.data_dir.join("gates.sqlite"))?);
    let human_reviews = Arc::new(SqliteHumanReviewStore::open(
        args.data_dir.join("reviews.sqlite"),
    )?);
    let calibrations = Arc::new(SqliteCalibrationStore::open(
        args.data_dir.join("calibrations.sqlite"),
    )?);
    let usage = Arc::new(SqliteUsageLedger::open(args.data_dir.join("usage.sqlite"))?);
    let audit = Arc::new(SqliteAuditStore::open(args.data_dir.join("audit.sqlite"))?);
    let provider_secret_keyring = match args.provider_secret_key.as_deref() {
        Some(encoded) => SecretKeyring::from_base64("env-v1", encoded)?,
        None => SecretKeyring::load_or_create_local_file(
            args.data_dir.join("provider-secrets.key"),
            "local-v1",
        )?,
    };
    let provider_secrets = Arc::new(EncryptedSqliteProviderSecretStore::open(
        args.data_dir.join("provider-secrets.sqlite"),
        provider_secret_keyring,
    )?);
    let judge_ledger = Arc::new(SqliteJudgeLedger::open(args.data_dir.join("judge.sqlite"))?);
    let judge_provider: Arc<dyn JudgeProvider> = match args.judge_provider {
        JudgeProviderArg::Keyword => Arc::new(KeywordJudgeProvider::default()),
        JudgeProviderArg::HttpRouting => Arc::new(HttpRoutingJudgeProvider::default()),
    };
    let judge_broker = Arc::new(JudgeBrokerService::new(
        provider_secrets.clone(),
        judge_ledger.clone(),
        judge_provider,
        Money::usd_micros(args.judge_budget_micros),
    ));
    let bus: Arc<dyn DurableBus> = match args.bus_backend {
        BusBackendArg::Sqlite => Arc::new(
            SqliteDurableBus::open(args.data_dir.join("bus.sqlite"), args.bus_capacity)
                .map_err(anyhow::Error::from)?,
        ),
        BusBackendArg::Memory => Arc::new(InMemoryBus::new(args.bus_capacity)),
    };
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default())
        .with_quota_limiter(quota_limiter);
    if args.trace_write_drain_interval_ms > 0 {
        spawn_trace_write_worker(
            ingest.clone(),
            Duration::from_millis(args.trace_write_drain_interval_ms),
        );
    }
    if args.trace_ingested_drain_interval_ms > 0 {
        spawn_trace_ingested_worker(
            ingest.clone(),
            traces.clone(),
            search.clone(),
            Duration::from_millis(args.trace_ingested_drain_interval_ms),
        );
    }
    let otlp_default_scope = beater_core::TenantScope::new(
        beater_core::TenantId::new(args.default_tenant_id.clone())?,
        beater_core::ProjectId::new(args.default_project_id.clone())?,
        beater_core::EnvironmentId::new(args.default_environment_id.clone())?,
    );
    let otlp_grpc = OtlpGrpcTraceService::new(ingest.clone(), otlp_default_scope);
    let mut state =
        ApiState::with_integrations(ingest, traces, search, archive, datasets, experiments)
            .with_metadata(metadata)
            .with_gates(gates)
            .with_human_reviews(human_reviews)
            .with_calibrations(calibrations)
            .with_usage(usage)
            .with_audit(audit)
            .with_judge(provider_secrets, judge_broker, judge_ledger);
    if matches!(args.auth_mode, AuthModeArg::Required) {
        let api_keys = Arc::new(SqliteApiKeyStore::open(
            args.data_dir.join("security.sqlite"),
        )?);
        state = state.require_auth(api_keys);
    }
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(args.addr)
        .await
        .with_context(|| format!("bind {}", args.addr))?;
    let http_server = async move {
        axum::serve(listener, app)
            .await
            .context("serve beaterd http")
    };
    let grpc_server = async move {
        Server::builder()
            .add_service(TraceServiceServer::new(otlp_grpc))
            .serve(args.otlp_grpc_addr)
            .await
            .context("serve beaterd otlp grpc")
    };
    tokio::try_join!(http_server, grpc_server)?;
    Ok(())
}

fn spawn_trace_write_worker(ingest: IngestService, interval: Duration) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let report = match ingest.drain_trace_writes(100).await {
                Ok(report) => report,
                Err(error) => {
                    eprintln!("trace write drain failed: {error}");
                    continue;
                }
            };
            if report.consumed > 0
                && (report.failed_writes > 0 || report.failed_downstream_publishes > 0)
            {
                eprintln!(
                    "trace write drain completed with failures: consumed={} failed_writes={} failed_downstream_publishes={} retried={} dlq={}",
                    report.consumed,
                    report.failed_writes,
                    report.failed_downstream_publishes,
                    report.retried,
                    report.dead_lettered
                );
            }
        }
    });
}

fn spawn_trace_ingested_worker(
    ingest: IngestService,
    traces: Arc<SqliteTraceStore>,
    search: Arc<TantivySearchIndex>,
    interval: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let traces = traces.clone();
            let search = search.clone();
            let report = match ingest
                .drain_trace_ingested(100, move |trace_ref| {
                    let traces = traces.clone();
                    let search = search.clone();
                    async move { index_trace_ref(traces, search, trace_ref).await }
                })
                .await
            {
                Ok(report) => report,
                Err(error) => {
                    eprintln!("trace.ingested drain failed: {error}");
                    continue;
                }
            };
            if report.consumed > 0 && report.failed_work > 0 {
                eprintln!(
                    "trace.ingested drain completed with failed work: consumed={} failed={} retried={} dlq={}",
                    report.consumed, report.failed_work, report.retried, report.dead_lettered
                );
            }
        }
    });
}

async fn index_trace_ref(
    traces: Arc<SqliteTraceStore>,
    search: Arc<TantivySearchIndex>,
    trace_ref: QueuedTraceWork,
) -> Result<(), String> {
    let trace = traces
        .get_trace(trace_ref.tenant_id.clone(), trace_ref.trace_id.clone())
        .await
        .map_err(|err| {
            format!(
                "trace.ingested readback failed for tenant={} trace={}: {err}",
                trace_ref.tenant_id, trace_ref.trace_id
            )
        })?;
    search.index_spans(&trace.spans).await.map_err(|err| {
        format!(
            "trace.ingested indexing failed for tenant={} trace={}: {err}",
            trace_ref.tenant_id, trace_ref.trace_id
        )
    })
}
