use anyhow::Context;
use beater_api::{router, ApiState};
use beater_archive::ParquetTraceArchive;
use beater_audit::SqliteAuditStore;
use beater_auth::SqliteApiKeyStore;
use beater_bus::{DurableBus, InMemoryBus, SqliteDurableBus};
use beater_calibration::SqliteCalibrationStore;
use beater_core::{IdempotencyKey, Money, Page, PageRequest, ProjectId, TenantId, TraceId};
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_ingest::{IngestPolicy, IngestService};
use beater_judge::{
    HttpRoutingJudgeProvider, JudgeBrokerService, JudgeProvider, KeywordJudgeProvider,
    SqliteJudgeLedger,
};
use beater_otlp::{OtlpGrpcTraceService, TraceServiceServer};
use beater_schema::{
    CanonicalTraceBatch, RawEnvelope, RunFilter, RunSummary, SpanFilter, SpanSummary, TraceView,
    WriteAck,
};
use beater_search::{SearchIndex, TantivySearchIndex, TraceIngestedSearchProcessor};
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store::{StoreError, StoreResult, TraceStore};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::{
    migrate_local_beaterd_sqlite, SqliteMetadataStore, SqliteQuotaLimiter, SqliteTraceStore,
};
use beater_usage::SqliteUsageLedger;
use clap::{Parser, ValueEnum};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
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
    #[arg(long, env = "BEATER_PER_PROJECT_EVENT_QUOTA")]
    per_project_event_quota: Option<u64>,
    #[arg(long, env = "BEATER_QUOTA_WINDOW_SECONDS", default_value_t = 60)]
    quota_window_seconds: i64,
    #[arg(long, env = "BEATER_QUOTA_DB_PATH")]
    quota_db_path: Option<PathBuf>,
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
    #[arg(long, env = "BEATER_TRACE_WRITE_MAX_ATTEMPTS", default_value_t = 3)]
    trace_write_max_attempts: u32,
    #[arg(
        long,
        env = "BEATER_TRACE_INGESTED_DRAIN_INTERVAL_MS",
        default_value_t = 1000
    )]
    trace_ingested_drain_interval_ms: u64,
    #[arg(long, hide = true, env = "BEATER_TEST_TRACE_WRITE_LEASE_MARKER")]
    test_trace_write_lease_marker: Option<PathBuf>,
    #[arg(long, hide = true, env = "BEATER_TEST_TRACE_WRITE_HOLD_PATH")]
    test_trace_write_hold_path: Option<PathBuf>,
    #[arg(long, hide = true, env = "BEATER_TEST_TRACE_INGESTED_LEASE_MARKER")]
    test_trace_ingested_lease_marker: Option<PathBuf>,
    #[arg(long, hide = true, env = "BEATER_TEST_TRACE_INGESTED_HOLD_PATH")]
    test_trace_ingested_hold_path: Option<PathBuf>,
    #[arg(long, hide = true, env = "BEATER_TEST_TRACE_INGESTED_FAIL_WHILE_PATH")]
    test_trace_ingested_fail_while_path: Option<PathBuf>,
    #[arg(
        long,
        hide = true,
        env = "BEATER_TEST_TRACE_STORE_FAIL_WRITE_WHILE_PATH"
    )]
    test_trace_store_fail_write_while_path: Option<PathBuf>,
    #[arg(long, hide = true, env = "BEATER_TEST_HTTP_TRACE_STORE_URL")]
    test_http_trace_store_url: Option<String>,
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
    if args.test_http_trace_store_url.is_some() && !cfg!(debug_assertions) {
        anyhow::bail!("--test-http-trace-store-url is only supported in debug/test builds");
    }
    let trace_db_path = args.data_dir.join("traces.sqlite");
    let quota_path = args
        .quota_db_path
        .clone()
        .unwrap_or_else(|| args.data_dir.join("quotas.sqlite"));
    let metadata_db_path = args.data_dir.join("metadata.sqlite");
    let dataset_db_path = args.data_dir.join("datasets.sqlite");
    let experiment_db_path = args.data_dir.join("experiments.sqlite");
    let gate_db_path = args.data_dir.join("gates.sqlite");
    let review_db_path = args.data_dir.join("reviews.sqlite");
    let calibration_db_path = args.data_dir.join("calibrations.sqlite");
    let usage_db_path = args.data_dir.join("usage.sqlite");
    let audit_db_path = args.data_dir.join("audit.sqlite");
    let provider_secret_db_path = args.data_dir.join("provider-secrets.sqlite");
    let judge_db_path = args.data_dir.join("judge.sqlite");
    let bus_db_path = args.data_dir.join("bus.sqlite");
    let security_db_path = args.data_dir.join("security.sqlite");
    let mut sqlite_store_paths = vec![
        trace_db_path.clone(),
        quota_path.clone(),
        metadata_db_path.clone(),
        dataset_db_path.clone(),
        experiment_db_path.clone(),
        gate_db_path.clone(),
        review_db_path.clone(),
        calibration_db_path.clone(),
        usage_db_path.clone(),
        audit_db_path.clone(),
        provider_secret_db_path.clone(),
        judge_db_path.clone(),
    ];
    if matches!(args.bus_backend, BusBackendArg::Sqlite) {
        sqlite_store_paths.push(bus_db_path.clone());
    }
    if matches!(args.auth_mode, AuthModeArg::Required) {
        sqlite_store_paths.push(security_db_path.clone());
    }
    migrate_local_sqlite_stores(&sqlite_store_paths)?;

    let artifacts = Arc::new(FsArtifactStore::new(args.data_dir.join("artifacts"))?);
    let sqlite_traces = Arc::new(SqliteTraceStore::open(trace_db_path)?);
    let traces: Arc<dyn TraceStore> = if let Some(url) = args.test_http_trace_store_url.clone() {
        Arc::new(HttpTraceStore::new(url))
    } else {
        match args.test_trace_store_fail_write_while_path.clone() {
            Some(path) => Arc::new(FailSwitchTraceStore::new(sqlite_traces.clone(), path)),
            None => sqlite_traces.clone(),
        }
    };
    let quota_limiter = Arc::new(SqliteQuotaLimiter::open(quota_path)?);
    let metadata = Arc::new(SqliteMetadataStore::open(metadata_db_path)?);
    let search = Arc::new(TantivySearchIndex::open_or_create(
        args.data_dir.join("search"),
    )?);
    let archive = ParquetTraceArchive::new(args.data_dir.join("archive"))?;
    let datasets = Arc::new(SqliteDatasetStore::open(dataset_db_path)?);
    let experiments = Arc::new(SqliteExperimentStore::open(experiment_db_path)?);
    let gates = Arc::new(SqliteGateStore::open(gate_db_path)?);
    let human_reviews = Arc::new(SqliteHumanReviewStore::open(review_db_path)?);
    let calibrations = Arc::new(SqliteCalibrationStore::open(calibration_db_path)?);
    let usage = Arc::new(SqliteUsageLedger::open(usage_db_path)?);
    let audit = Arc::new(SqliteAuditStore::open(audit_db_path)?);
    let provider_secret_keyring = match args.provider_secret_key.as_deref() {
        Some(encoded) => SecretKeyring::from_base64("env-v1", encoded)?,
        None => SecretKeyring::load_or_create_local_file(
            args.data_dir.join("provider-secrets.key"),
            "local-v1",
        )?,
    };
    let provider_secrets = Arc::new(EncryptedSqliteProviderSecretStore::open(
        provider_secret_db_path,
        provider_secret_keyring,
    )?);
    let judge_ledger = Arc::new(SqliteJudgeLedger::open(judge_db_path)?);
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
            SqliteDurableBus::open(bus_db_path, args.bus_capacity).map_err(anyhow::Error::from)?,
        ),
        BusBackendArg::Memory => Arc::new(InMemoryBus::new(args.bus_capacity)),
    };
    let ingest_policy = IngestPolicy {
        per_project_event_quota: args.per_project_event_quota,
        quota_window_seconds: args.quota_window_seconds,
        trace_write_max_attempts: args.trace_write_max_attempts,
        ..IngestPolicy::default()
    };
    let ingest = IngestService::new(artifacts, traces.clone(), bus, ingest_policy)
        .with_quota_limiter(quota_limiter)
        .with_importer(std::sync::Arc::new(
            beater_temporal::TemporalHistoryImporter,
        ));
    if args.trace_write_drain_interval_ms > 0 {
        let trace_write_hooks = TraceWriteWorkerHooks {
            lease_marker_path: args.test_trace_write_lease_marker.clone(),
            hold_path: args.test_trace_write_hold_path.clone(),
        };
        spawn_trace_write_worker(
            ingest.clone(),
            Duration::from_millis(args.trace_write_drain_interval_ms),
            trace_write_hooks,
        );
    }
    if args.trace_ingested_drain_interval_ms > 0 {
        let trace_ingested_hooks = TraceIngestedWorkerHooks {
            lease_marker_path: args.test_trace_ingested_lease_marker.clone(),
            hold_path: args.test_trace_ingested_hold_path.clone(),
            fail_while_path: args.test_trace_ingested_fail_while_path.clone(),
        };
        spawn_trace_ingested_worker(
            ingest.clone(),
            traces.clone(),
            search.clone(),
            Duration::from_millis(args.trace_ingested_drain_interval_ms),
            trace_ingested_hooks,
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
        let api_keys = Arc::new(SqliteApiKeyStore::open(security_db_path)?);
        state = state.require_auth(api_keys);
    }
    // Serve the MCP endpoint (`/mcp`) alongside the HTTP API, sharing the same
    // `ApiState` and auth. The MCP tool catalog is derived from the OpenAPI spec
    // and dispatches through the real router, so it cannot drift from the API.
    let app = router(state.clone()).merge(beater_mcp::router(state));
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

fn migrate_local_sqlite_stores(paths: &[PathBuf]) -> anyhow::Result<()> {
    let mut unique_paths = paths.to_vec();
    unique_paths.sort();
    unique_paths.dedup();
    for path in unique_paths {
        migrate_local_beaterd_sqlite(&path)
            .with_context(|| format!("migrate local sqlite schema {}", path.display()))?;
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct TraceWriteWorkerHooks {
    lease_marker_path: Option<PathBuf>,
    hold_path: Option<PathBuf>,
}

fn spawn_trace_write_worker(
    ingest: IngestService,
    interval: Duration,
    hooks: TraceWriteWorkerHooks,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let hooks = hooks.clone();
            let report = match ingest
                .drain_trace_writes_with_hook(100, move |_queued| {
                    let hooks = hooks.clone();
                    async move { apply_trace_write_test_hooks(&hooks).await }
                })
                .await
            {
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

async fn apply_trace_write_test_hooks(hooks: &TraceWriteWorkerHooks) -> Result<(), String> {
    if let Some(marker_path) = &hooks.lease_marker_path {
        write_hook_marker(marker_path, "trace.write")?;
    }
    if let Some(hold_path) = &hooks.hold_path {
        while hold_path.exists() {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct TraceIngestedWorkerHooks {
    lease_marker_path: Option<PathBuf>,
    hold_path: Option<PathBuf>,
    fail_while_path: Option<PathBuf>,
}

fn spawn_trace_ingested_worker(
    ingest: IngestService,
    traces: Arc<dyn TraceStore>,
    search: Arc<dyn SearchIndex>,
    interval: Duration,
    hooks: TraceIngestedWorkerHooks,
) {
    tokio::spawn(async move {
        let search_processor = TraceIngestedSearchProcessor::new(traces, search);
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let search_processor = search_processor.clone();
            let hooks = hooks.clone();
            let report = match ingest
                .drain_trace_ingested(100, move |trace_ref| {
                    let search_processor = search_processor.clone();
                    let hooks = hooks.clone();
                    async move {
                        apply_trace_ingested_test_hooks(&hooks).await?;
                        search_processor
                            .process_trace(
                                trace_ref.tenant_id,
                                trace_ref.project_id,
                                trace_ref.trace_id,
                            )
                            .await
                    }
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

async fn apply_trace_ingested_test_hooks(hooks: &TraceIngestedWorkerHooks) -> Result<(), String> {
    if let Some(marker_path) = &hooks.lease_marker_path {
        write_hook_marker(marker_path, "trace.ingested")?;
    }
    if let Some(hold_path) = &hooks.hold_path {
        while hold_path.exists() {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    if let Some(fail_while_path) = &hooks.fail_while_path {
        if fail_while_path.exists() {
            return Err(format!(
                "test trace.ingested failure while {} exists",
                fail_while_path.display()
            ));
        }
    }
    Ok(())
}

fn write_hook_marker(path: &Path, lane: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {lane} marker dir failed: {err}"))?;
    }
    fs::write(path, b"leased").map_err(|err| format!("write {lane} marker failed: {err}"))
}

// Hidden live-test adapter for proving external TraceStore outages over TCP.
#[derive(Clone)]
struct HttpTraceStore {
    client: reqwest::Client,
    base_url: String,
}

impl HttpTraceStore {
    fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap_or_else(|err| panic!("http trace store test client must build: {err}")),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    async fn post_json<B, T>(&self, path: &str, body: B) -> StoreResult<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}/trace-store/{path}", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|err| {
                StoreError::backend(format!("http trace store {path} request failed: {err}"))
            })?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|err| format!("failed to read error body: {err}"));
            return Err(StoreError::backend(format!(
                "http trace store {path} returned {status}: {body}"
            )));
        }
        response.json::<T>().await.map_err(|err| {
            StoreError::backend(format!("http trace store {path} decode failed: {err}"))
        })
    }
}

#[async_trait::async_trait]
impl TraceStore for HttpTraceStore {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
        self.post_json("write-batch", batch).await
    }

    async fn get_trace(&self, tenant: TenantId, trace: TraceId) -> StoreResult<TraceView> {
        self.post_json(
            "get-trace",
            json!({
                "tenant_id": tenant,
                "trace_id": trace,
            }),
        )
        .await
    }

    async fn get_project_trace(
        &self,
        tenant: TenantId,
        project: ProjectId,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        self.post_json(
            "get-project-trace",
            json!({
                "tenant_id": tenant,
                "project_id": project,
                "trace_id": trace,
            }),
        )
        .await
    }

    async fn get_raw_envelope(
        &self,
        _tenant: TenantId,
        _project: ProjectId,
        _idempotency_key: IdempotencyKey,
    ) -> StoreResult<Option<RawEnvelope>> {
        Err(StoreError::backend(
            "http trace store test adapter does not support get_raw_envelope",
        ))
    }

    async fn query_runs(
        &self,
        _tenant: TenantId,
        _filter: RunFilter,
        _page: PageRequest,
    ) -> StoreResult<Page<RunSummary>> {
        Err(StoreError::backend(
            "http trace store test adapter does not support query_runs",
        ))
    }

    async fn query_spans(
        &self,
        _tenant: TenantId,
        _filter: SpanFilter,
        _page: PageRequest,
    ) -> StoreResult<Page<SpanSummary>> {
        Err(StoreError::backend(
            "http trace store test adapter does not support query_spans",
        ))
    }
}

#[derive(Clone)]
struct FailSwitchTraceStore {
    inner: Arc<SqliteTraceStore>,
    fail_write_while_path: PathBuf,
}

impl FailSwitchTraceStore {
    fn new(inner: Arc<SqliteTraceStore>, fail_write_while_path: PathBuf) -> Self {
        Self {
            inner,
            fail_write_while_path,
        }
    }
}

#[async_trait::async_trait]
impl TraceStore for FailSwitchTraceStore {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> StoreResult<WriteAck> {
        if self.fail_write_while_path.exists() {
            return Err(StoreError::backend(format!(
                "test trace store write failure while {} exists",
                self.fail_write_while_path.display()
            )));
        }
        self.inner.write_batch(batch).await
    }

    async fn get_trace(&self, tenant: TenantId, trace: TraceId) -> StoreResult<TraceView> {
        self.inner.get_trace(tenant, trace).await
    }

    async fn get_project_trace(
        &self,
        tenant: TenantId,
        project: ProjectId,
        trace: TraceId,
    ) -> StoreResult<TraceView> {
        self.inner.get_project_trace(tenant, project, trace).await
    }

    async fn get_raw_envelope(
        &self,
        tenant: TenantId,
        project: ProjectId,
        idempotency_key: IdempotencyKey,
    ) -> StoreResult<Option<RawEnvelope>> {
        self.inner
            .get_raw_envelope(tenant, project, idempotency_key)
            .await
    }

    async fn query_runs(
        &self,
        tenant: TenantId,
        filter: RunFilter,
        page: PageRequest,
    ) -> StoreResult<Page<RunSummary>> {
        self.inner.query_runs(tenant, filter, page).await
    }

    async fn query_spans(
        &self,
        tenant: TenantId,
        filter: SpanFilter,
        page: PageRequest,
    ) -> StoreResult<Page<SpanSummary>> {
        self.inner.query_spans(tenant, filter, page).await
    }
}
