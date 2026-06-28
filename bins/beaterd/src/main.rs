mod metrics;
mod metrics_http;

use anyhow::Context;
use beater_accounts::SqliteAccountStore;
use beater_api::{router, ApiState};
use beater_archive::ParquetTraceArchive;
use beater_audit::SqliteAuditStore;
use beater_auth::SqliteApiKeyStore;
use beater_bus::{DeadLetter, DurableBus, InMemoryBus, SqliteDurableBus};
use beater_calibration::SqliteCalibrationStore;
use beater_core::{IdempotencyKey, Money, Page, PageRequest, ProjectId, TenantId, TraceId};
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_ingest::{
    ImportError, IngestPolicy, IngestService, RawTraceIngestRequest, SourceImporter,
    TraceCompletionConfig, TRACE_INGESTED_KIND, TRACE_WRITE_BATCH_KIND,
};
use beater_judge::{
    HttpRoutingJudgeProvider, JudgeBrokerService, JudgeProvider, KeywordJudgeProvider,
    SqliteJudgeLedger,
};
use beater_oauth::SqliteOAuthStore;
use beater_oauth_server::OAuthServerState;
use beater_otlp::{OtlpGrpcTraceService, TraceServiceServer};
use beater_schema::{
    ArtifactRef, AuthContext, CanonicalTraceBatch, RawEnvelope, RedactionClass, RunFilter,
    RunSummary, SpanFilter, SpanSummary, TraceView, WriteAck,
};
use beater_search::{SearchIndex, TantivySearchIndex, TraceIngestedSearchProcessor};
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store::{ArtifactStore, StoreError, StoreResult, TraceStore};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::{
    migrate_local_beaterd_sqlite, SqliteMetadataStore, SqliteQuotaLimiter, SqliteTraceStore,
};
use beater_usage::SqliteUsageLedger;
use clap::{Parser, Subcommand, ValueEnum};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;

const DEFAULT_ARTIFACT_MAX_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Parser)]
#[command(
    name = "beaterd",
    about = "All-in-one Beater agent observability server"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
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
    /// Absolute public issuer URL for the OAuth endpoints + metadata documents
    /// (e.g. https://api.example.com). Falls back to http://<addr> when unset.
    #[arg(long, env = "BEATER_ISSUER_URL")]
    issuer_url: Option<String>,
    /// Dashboard login page; an unauthenticated /oauth/authorize redirects here
    /// with ?return_to=<authorize-url>.
    #[arg(long, env = "BEATER_LOGIN_URL")]
    login_url: Option<String>,
    #[arg(long, default_value = ".beater")]
    data_dir: PathBuf,
    /// Maximum bytes accepted for one filesystem artifact write.
    #[arg(
        long,
        env = "BEATER_ARTIFACT_MAX_BYTES",
        default_value_t = DEFAULT_ARTIFACT_MAX_BYTES
    )]
    artifact_max_bytes: u64,
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
    #[arg(long, value_enum, default_value_t = AuthModeArg::Required)]
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
    /// Maximum bytes accepted for a single ingest request payload (native or raw).
    /// Requests larger than this are rejected with 413. Default: 1 MiB.
    #[arg(
        long,
        env = "BEATER_MAX_RAW_PAYLOAD_BYTES",
        default_value_t = 1024 * 1024
    )]
    max_raw_payload_bytes: usize,
    /// Byte threshold below which an artifact is stored inline in the span record
    /// rather than as an external artifact reference. Must not exceed
    /// max_raw_payload_bytes. Default: 16 KiB.
    #[arg(
        long,
        env = "BEATER_INLINE_PAYLOAD_BYTES",
        default_value_t = 16 * 1024
    )]
    inline_payload_bytes: usize,
    /// Maximum number of attributes accepted per span. Spans with more attributes
    /// are rejected with 422. Default: 128.
    #[arg(long, env = "BEATER_MAX_ATTRIBUTES", default_value_t = 128)]
    max_attributes: usize,
    /// Seconds of inactivity after the last span end before an open trace is
    /// classified as idle-complete. Must be positive. Default: 60.
    #[arg(long, env = "BEATER_TRACE_IDLE_TIMEOUT_SECONDS", default_value_t = 60)]
    trace_idle_timeout_seconds: i64,
    /// Seconds after the root span ends during which late-arriving child spans are
    /// still accepted. Must be positive. Default: 10.
    #[arg(long, env = "BEATER_TRACE_LATE_WINDOW_SECONDS", default_value_t = 10)]
    trace_late_window_seconds: i64,
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
    /// Opt in to anonymous self-host usage telemetry. Off by default (R12.5):
    /// a self-hosted beaterd makes no outbound telemetry call unless this is set.
    #[arg(long, env = beater_core::SelfHostTelemetryConfig::ENV_VAR)]
    self_host_telemetry: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Serve the Model Context Protocol over a local transport.
    Mcp {
        /// Read newline-delimited JSON-RPC requests from stdin and write
        /// responses to stdout. Diagnostics stay on stderr.
        #[arg(long)]
        stdio: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum AuthModeArg {
    Local,
    Required,
}

fn validate_auth_mode_bind(args: &Args) -> anyhow::Result<()> {
    if matches!(args.auth_mode, AuthModeArg::Local) && !args.addr.ip().is_loopback() {
        anyhow::bail!(
            "--auth-mode local may only bind HTTP to loopback addresses; got --addr {}. \
             Use --auth-mode required for non-loopback binds.",
            args.addr
        );
    }
    Ok(())
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
    // R13.1 — install the self-observability foundation (metrics registry +
    // structured log writer) before any work happens, so every subsystem can
    // emit into the same process-wide registry.
    let metrics = metrics::init_observability();
    let args = Args::parse();
    if args.test_http_trace_store_url.is_some() && !cfg!(debug_assertions) {
        anyhow::bail!("--test-http-trace-store-url is only supported in debug/test builds");
    }
    validate_auth_mode_bind(&args)?;
    // R12.5: self-host telemetry is opt-out. With the flag unset, this resolves
    // to disabled and beaterd makes no outbound telemetry call.
    let telemetry = beater_core::SelfHostTelemetryConfig::new(args.self_host_telemetry);
    match telemetry.endpoint() {
        Some(endpoint) => {
            eprintln!("self-host telemetry enabled (opt-in); reporting to {endpoint}")
        }
        None => eprintln!("self-host telemetry disabled (opt-out default); no outbound reporting"),
    }
    if matches!(args.auth_mode, AuthModeArg::Local) {
        eprintln!("============================================================");
        eprintln!("WARNING: beaterd is running in INSECURE --auth-mode local.");
        eprintln!("  Listening on {}", args.addr);
        eprintln!("  Mutating/sensitive routes are served ANONYMOUSLY:");
        eprintln!("    tenant/project reads + writes, API-key minting,");
        eprintln!("    provider-secret routes, audit routes, and unmask.");
        eprintln!("  Anyone who can reach this address has full access.");
        eprintln!("  Pass --auth-mode required to enforce API-key authentication.");
        eprintln!("============================================================");
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

    // R13.9 — wrap the object store so every read/write outcome is counted.
    // §23.7 — enforce the artifact write-size cap by default in the runtime,
    // rather than leaving the FsArtifactStore cap as an unwired opt-in.
    let artifacts = Arc::new(build_local_artifact_store(
        &args.data_dir,
        args.artifact_max_bytes,
        metrics.clone(),
    )?);
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
        max_raw_payload_bytes: args.max_raw_payload_bytes,
        inline_payload_bytes: args.inline_payload_bytes,
        max_attributes: args.max_attributes,
        per_project_event_quota: args.per_project_event_quota,
        quota_window_seconds: args.quota_window_seconds,
        trace_write_max_attempts: args.trace_write_max_attempts,
        trace_completion: TraceCompletionConfig {
            idle_timeout: chrono::Duration::seconds(args.trace_idle_timeout_seconds),
            late_window: chrono::Duration::seconds(args.trace_late_window_seconds),
        },
        ..IngestPolicy::default()
    };
    ingest_policy.validate().context(
        "invalid ingest policy; check BEATER_MAX_RAW_PAYLOAD_BYTES, \
                  BEATER_INLINE_PAYLOAD_BYTES, BEATER_MAX_ATTRIBUTES, \
                  BEATER_TRACE_IDLE_TIMEOUT_SECONDS, BEATER_TRACE_LATE_WINDOW_SECONDS",
    )?;
    eprintln!(
        "ingest policy: max_raw_payload_bytes={} inline_payload_bytes={} \
         max_attributes={} trace_idle_timeout_seconds={} trace_late_window_seconds={}",
        ingest_policy.max_raw_payload_bytes,
        ingest_policy.inline_payload_bytes,
        ingest_policy.max_attributes,
        ingest_policy.trace_completion.idle_timeout.num_seconds(),
        ingest_policy.trace_completion.late_window.num_seconds(),
    );
    // Keep a handle to the global, unfiltered bus so the queue-stats sampler can
    // observe DLQ depth/age across ALL tenants (R13.4/R13.6/R13.8), not just the
    // default scope.
    let queue_stats_bus = bus.clone();
    // R13.7 — wrap the source importer so normalization failures are counted by
    // source dialect (and the importer's normalizer version).
    let ingest = IngestService::new(artifacts, traces.clone(), bus, ingest_policy)
        .with_quota_limiter(quota_limiter)
        .with_importer(std::sync::Arc::new(MeteredImporter::new(
            beater_temporal::TemporalHistoryImporter,
            "temporal-history-import-v1",
            metrics.clone(),
        )))
        .with_importer(std::sync::Arc::new(MeteredImporter::new(
            beater_langfuse::LangfuseImporter,
            beater_langfuse::LANGFUSE_CONTRACT,
            metrics.clone(),
        )));
    if args.trace_write_drain_interval_ms > 0 {
        let trace_write_hooks = TraceWriteWorkerHooks {
            lease_marker_path: args.test_trace_write_lease_marker.clone(),
            hold_path: args.test_trace_write_hold_path.clone(),
        };
        spawn_trace_write_worker(
            ingest.clone(),
            Duration::from_millis(args.trace_write_drain_interval_ms),
            trace_write_hooks,
            metrics.clone(),
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
            metrics.clone(),
        );
    }
    let otlp_default_scope = beater_core::TenantScope::new(
        beater_core::TenantId::new(args.default_tenant_id.clone())?,
        beater_core::ProjectId::new(args.default_project_id.clone())?,
        beater_core::EnvironmentId::new(args.default_environment_id.clone())?,
    );
    // R13.4 / R13.6 / R13.8 — periodically sample queue depths, dead-letter
    // backlog, and per-lane lag from the GLOBAL (unfiltered) bus and publish them
    // to the metrics registry. Depth/age therefore reflect the whole deployment
    // across all tenants, not just the default scope. Per-lane DLQ age is
    // attributed by message kind; the per-tenant lag label is the deployment's
    // default tenant (a stable, bounded label) — cardinality stays bounded by the
    // small lane set and the cardinality-safe label helpers.
    spawn_queue_stats_sampler(
        queue_stats_bus,
        beater_core::TenantId::new(args.default_tenant_id.clone())?,
        Duration::from_secs(5),
        metrics.clone(),
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
    // Build the API-key store once (strict auth only) and share it between the
    // `/v1` auth path and the session-authorized `/auth/api-keys` endpoints.
    let api_key_store: Option<Arc<dyn beater_auth::ApiKeyStore>> =
        if matches!(args.auth_mode, AuthModeArg::Required) {
            Some(Arc::new(SqliteApiKeyStore::open(security_db_path)?))
        } else {
            None
        };
    if let Some(api_keys) = api_key_store.clone() {
        state = state.require_auth(api_keys);
    }
    // Serve the MCP endpoint (`/mcp`) alongside the HTTP API, sharing the same
    // `ApiState` and auth. The MCP tool catalog is derived from the OpenAPI spec
    // and dispatches through the real router, so it cannot drift from the API.
    // R13.5 — record per-request query latency via an axum middleware (labelled
    // by matched route template + method for bounded cardinality). The
    // Prometheus `/metrics` route (NOT part of the typed `/v1` contract) is
    // merged in alongside the API and MCP routers.
    // OAuth 2.1 authorization-server HTTP surface (root-level /oauth/* +
    // /.well-known/*), merged in like the MCP router. It owns its own state
    // (OAuth + accounts stores) and is NOT part of the typed /v1 contract. The
    // stores' `open()` runs their own migrations, so they are not added to the
    // beaterd sqlite migration list above.
    let oauth_store = Arc::new(SqliteOAuthStore::open(args.data_dir.join("oauth.sqlite"))?);
    let account_store = Arc::new(SqliteAccountStore::open(
        args.data_dir.join("accounts.sqlite"),
    )?);
    let issuer = args
        .issuer_url
        .clone()
        .unwrap_or_else(|| format!("http://{}", args.addr));
    let oauth_metadata_url = format!("{issuer}/.well-known/oauth-protected-resource");
    // Let the HTTP API + MCP accept OAuth access tokens (not just API keys),
    // sharing the same OAuth store the authorization server writes to.
    state = state.with_oauth(oauth_store.clone(), Some(oauth_metadata_url));
    let oauth_state = OAuthServerState {
        oauth: oauth_store,
        accounts: account_store,
        issuer,
        login_url: args.login_url.clone(),
        scopes_supported: vec![
            "trace:read".to_string(),
            "trace:write".to_string(),
            "mcp:invoke".to_string(),
        ],
        api_keys: api_key_store,
    };

    if let Some(Command::Mcp { stdio }) = args.command {
        if !stdio {
            anyhow::bail!("the mcp subcommand currently requires --stdio");
        }
        beater_mcp::serve_stdio(state)
            .await
            .context("serve mcp stdio")?;
        return Ok(());
    }

    let latency_metrics = metrics.clone();
    let app = router(state.clone())
        .merge(beater_mcp::router(state))
        .layer(axum::middleware::from_fn(move |req, next| {
            let latency_metrics = latency_metrics.clone();
            async move { metrics_http::track_query_latency(latency_metrics, req, next).await }
        }))
        .merge(metrics_http::router(metrics.clone()))
        .merge(beater_oauth_server::router(oauth_state));
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
    metrics: metrics::Metrics,
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
            // R13.3 — write success rate: count successful and failed writes.
            let succeeded = report.written_spans.saturating_add(report.written_raw) as u64;
            metrics.record_write(metrics::OpResult::Success, succeeded);
            metrics.record_write(metrics::OpResult::Failure, report.failed_writes as u64);
            // R13.6 — dead-letter queue depth for the trace.write lane. Workers
            // own DEPTH only; the *_oldest_age_seconds gauge is owned solely by
            // the queue-stats sampler so the two writers never race (see
            // `spawn_queue_stats_sampler`).
            metrics.set_dlq_depth("trace.write", report.dead_lettered);
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
    metrics: metrics::Metrics,
) {
    tokio::spawn(async move {
        let search_processor = TraceIngestedSearchProcessor::new(traces, search);
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            let search_processor = search_processor.clone();
            let hooks = hooks.clone();
            let lag_metrics = metrics.clone();
            let report = match ingest
                .drain_trace_ingested(100, move |trace_ref| {
                    let search_processor = search_processor.clone();
                    let hooks = hooks.clone();
                    let lag_metrics = lag_metrics.clone();
                    async move {
                        apply_trace_ingested_test_hooks(&hooks).await?;
                        let sw = metrics::Stopwatch::start();
                        let result = search_processor
                            .process_trace(
                                trace_ref.tenant_id,
                                trace_ref.project_id,
                                trace_ref.trace_id,
                            )
                            .await;
                        if result.is_ok() {
                            // R13.2 — observe ingest-to-queryable lag (the time to
                            // index a trace so it becomes searchable/queryable).
                            lag_metrics.observe_ingest_lag(sw.elapsed_seconds());
                        }
                        result
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
            // R13.6 — dead-letter queue depth for the trace.ingested lane.
            // Workers own DEPTH only; the *_oldest_age_seconds gauge is owned
            // solely by the queue-stats sampler (see `spawn_queue_stats_sampler`).
            metrics.set_dlq_depth("trace.ingested", report.dead_lettered);
            if report.consumed > 0 && report.failed_work > 0 {
                eprintln!(
                    "trace.ingested drain completed with failed work: consumed={} failed={} retried={} dlq={}",
                    report.consumed, report.failed_work, report.retried, report.dead_lettered
                );
            }
        }
    });
}

/// Seconds between `now` and the oldest `failed_at` of the dead letters whose
/// message `kind` matches `lane` (clamped at 0; 0.0 when none match). Used for
/// the per-lane `*_oldest_age_seconds` gauges.
///
/// ponytail: O(n) scan over the whole DLQ per lane per tick. The DLQ is small
/// and the tick is 5s, so a few linear passes are fine; if the DLQ grows large,
/// group by `kind` once per tick instead.
fn lane_oldest_failure_seconds(
    dead_letters: &[DeadLetter],
    lane_kind: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> f64 {
    oldest_age_seconds(
        dead_letters
            .iter()
            .filter(|dl| dl.message.kind == lane_kind)
            .map(|dl| dl.failed_at),
        now,
    )
}

/// Seconds between `now` and the oldest `enqueued_at` of the dead letters whose
/// message `kind` matches `lane` (clamped at 0; 0.0 when none match). This is the
/// DLQ-DERIVED queue lag for a lane — NOT a live-backlog peek. The durable bus
/// exposes only depths and the DLQ (no API for the enqueue time of the oldest
/// non-failed pending message), so a growing backlog of healthy messages reports
/// lag = 0; the Prometheus HELP text says so. A real fix needs a
/// `DurableBus::oldest_pending_age(kind)` peek outside this binary's owned crate.
fn lane_oldest_enqueue_seconds(
    dead_letters: &[DeadLetter],
    lane_kind: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> f64 {
    oldest_age_seconds(
        dead_letters
            .iter()
            .filter(|dl| dl.message.kind == lane_kind)
            .map(|dl| dl.message.enqueued_at),
        now,
    )
}

/// Seconds between `now` and the minimum of `timestamps` (clamped at 0), or 0.0
/// when the iterator is empty.
fn oldest_age_seconds<I>(timestamps: I, now: chrono::DateTime<chrono::Utc>) -> f64
where
    I: IntoIterator<Item = chrono::DateTime<chrono::Utc>>,
{
    timestamps
        .into_iter()
        .min()
        .map(|ts| (now - ts).num_milliseconds().max(0) as f64 / 1000.0)
        .unwrap_or(0.0)
}

/// R13.4 / R13.6 / R13.8 — periodically sample GLOBAL queue health (across all
/// tenants) and publish it to the metrics registry. The sampler is the SOLE
/// writer of the `*_oldest_age_seconds` gauges (drain workers only set DLQ
/// depth), so the two never race on the age series.
fn spawn_queue_stats_sampler(
    bus: Arc<dyn DurableBus>,
    default_tenant: TenantId,
    interval: Duration,
    metrics: metrics::Metrics,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        // The lag gauge is labelled by tenant; we attribute the deployment-wide
        // DLQ-derived lag to the deployment's default tenant — a stable, bounded
        // label that keeps cardinality flat.
        let tenant_label = default_tenant.as_str().to_string();
        loop {
            ticker.tick().await;
            // Unfiltered, deployment-wide DLQ — every tenant, so `beater_dlq_depth`
            // reflects the whole deployment rather than a single tenant subset.
            let dead_letters = match bus.dlq().await {
                Ok(dead_letters) => dead_letters,
                Err(error) => {
                    eprintln!("queue stats DLQ sample failed: {error}");
                    continue;
                }
            };
            // Live eval-lane depth (count of pending, non-failed messages).
            let eval_queue_depth = match bus.depth_for_kind(TRACE_INGESTED_KIND).await {
                Ok(depth) => depth,
                Err(error) => {
                    eprintln!("queue stats eval-depth sample failed: {error}");
                    continue;
                }
            };
            // Per-lane DLQ depths across all tenants, attributed by message kind.
            let trace_ingested_dlq = dead_letters
                .iter()
                .filter(|dl| dl.message.kind == TRACE_INGESTED_KIND)
                .count();
            let trace_write_dlq = dead_letters
                .iter()
                .filter(|dl| dl.message.kind == TRACE_WRITE_BATCH_KIND)
                .count();
            let now = chrono::Utc::now();
            // DLQ-derived eval-lane lag (oldest ingested enqueue age). The same
            // value feeds the R13.4 eval-queue age gauge and the R13.8 lane lag.
            let eval_lane_lag =
                lane_oldest_enqueue_seconds(&dead_letters, TRACE_INGESTED_KIND, now);

            metrics.set_eval_queue(eval_queue_depth, eval_lane_lag);
            // Sampler owns BOTH the per-lane DLQ depth (global) and the per-lane
            // oldest-age gauge. Workers only ever set depth.
            metrics.set_dlq_depth("trace.ingested", trace_ingested_dlq);
            metrics.set_dlq_depth("trace.write", trace_write_dlq);
            metrics.set_dlq_oldest_age(
                "trace.ingested",
                lane_oldest_failure_seconds(&dead_letters, TRACE_INGESTED_KIND, now),
            );
            metrics.set_dlq_oldest_age(
                "trace.write",
                lane_oldest_failure_seconds(&dead_letters, TRACE_WRITE_BATCH_KIND, now),
            );
            metrics.set_queue_lag("trace.ingested", &tenant_label, eval_lane_lag);
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

/// R13.9 — an [`ArtifactStore`] decorator that records read/write outcomes into
/// the object-store operations counter. Delegates all behaviour to the inner
/// store; only adds a success/failure observation per call.
#[derive(Clone)]
struct MeteredArtifactStore<S> {
    inner: S,
    metrics: metrics::Metrics,
}

impl<S> MeteredArtifactStore<S> {
    fn new(inner: S, metrics: metrics::Metrics) -> Self {
        Self { inner, metrics }
    }

    fn record<T>(&self, op: metrics::ObjectStoreOp, result: &StoreResult<T>) {
        let outcome = if result.is_ok() {
            metrics::OpResult::Success
        } else {
            metrics::OpResult::Failure
        };
        self.metrics.record_object_store_op(op, outcome);
    }
}

fn build_local_artifact_store(
    data_dir: &Path,
    max_bytes: u64,
    metrics: metrics::Metrics,
) -> StoreResult<MeteredArtifactStore<FsArtifactStore>> {
    Ok(MeteredArtifactStore::new(
        FsArtifactStore::new(data_dir.join("artifacts"))?.with_max_bytes(max_bytes),
        metrics,
    ))
}

#[async_trait::async_trait]
impl<S: ArtifactStore> ArtifactStore for MeteredArtifactStore<S> {
    async fn put_bytes(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        mime_type: &str,
        redaction_class: RedactionClass,
        bytes: &[u8],
    ) -> StoreResult<ArtifactRef> {
        let result = self
            .inner
            .put_bytes(tenant_id, project_id, mime_type, redaction_class, bytes)
            .await;
        self.record(metrics::ObjectStoreOp::Write, &result);
        result
    }

    async fn get_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<Vec<u8>> {
        let result = self.inner.get_bytes(artifact_ref).await;
        self.record(metrics::ObjectStoreOp::Read, &result);
        result
    }

    async fn delete_bytes(&self, artifact_ref: &ArtifactRef) -> StoreResult<()> {
        let result = self.inner.delete_bytes(artifact_ref).await;
        self.record(metrics::ObjectStoreOp::Delete, &result);
        result
    }
}

/// R13.7 — a [`SourceImporter`] decorator that counts normalization failures by
/// source dialect and normalizer version. Delegates `source()` and `normalize()`
/// to the inner importer, incrementing the failure counter on `Err`.
struct MeteredImporter<I> {
    inner: I,
    version: &'static str,
    metrics: metrics::Metrics,
}

impl<I> MeteredImporter<I> {
    fn new(inner: I, version: &'static str, metrics: metrics::Metrics) -> Self {
        Self {
            inner,
            version,
            metrics,
        }
    }
}

impl<I: SourceImporter> SourceImporter for MeteredImporter<I> {
    fn source(&self) -> &'static str {
        self.inner.source()
    }

    fn normalize(
        &self,
        scope: &beater_core::TenantScope,
        raw_bytes: &[u8],
        auth: Option<AuthContext>,
    ) -> Result<RawTraceIngestRequest, ImportError> {
        let result = self.inner.normalize(scope, raw_bytes, auth);
        if result.is_err() {
            self.metrics
                .record_normalizer_failure(self.inner.source(), self.version);
        }
        result
    }
}

#[cfg(test)]
mod artifact_cap_tests {
    use super::*;

    #[test]
    fn artifact_max_bytes_has_a_runtime_default_and_cli_override() {
        let default_args = Args::try_parse_from(["beaterd"])
            .unwrap_or_else(|err| panic!("parse default args: {err}"));
        assert_eq!(default_args.artifact_max_bytes, DEFAULT_ARTIFACT_MAX_BYTES);

        let override_args = Args::try_parse_from(["beaterd", "--artifact-max-bytes", "4096"])
            .unwrap_or_else(|err| panic!("parse override args: {err}"));
        assert_eq!(override_args.artifact_max_bytes, 4096);
    }

    #[tokio::test]
    async fn local_artifact_store_builder_enforces_configured_size_cap() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
        let store = build_local_artifact_store(tempdir.path(), 4, metrics::Metrics::default())
            .unwrap_or_else(|err| panic!("{err}"));
        let tenant = TenantId::new("tenant").unwrap_or_else(|err| panic!("{err}"));
        let project = ProjectId::new("project").unwrap_or_else(|err| panic!("{err}"));

        let result = store
            .put_bytes(
                &tenant,
                &project,
                "text/plain",
                RedactionClass::Sensitive,
                b"12345",
            )
            .await;

        match result {
            Err(StoreError::LimitExceeded(message)) => {
                assert!(message.contains("artifact too large: 5 > 4 bytes"));
            }
            other => {
                panic!("expected StoreError::LimitExceeded for oversized artifact, got {other:?}")
            }
        }
    }
}

#[cfg(test)]
mod queue_stats_tests {
    use super::*;
    use beater_bus::BusMessage;
    use beater_core::{IdempotencyKey, ProjectId, TenantId};
    use chrono::{Duration as ChronoDuration, Utc};

    fn dead_letter_for(
        tenant: &str,
        kind: &str,
        enqueued_offset_s: i64,
        failed_offset_s: i64,
        now: chrono::DateTime<Utc>,
    ) -> DeadLetter {
        let mut message = BusMessage::new(
            TenantId::new(tenant).unwrap_or_else(|err| panic!("tenant: {err}")),
            ProjectId::new("demo").unwrap_or_else(|err| panic!("project: {err}")),
            IdempotencyKey::new("k").unwrap_or_else(|err| panic!("key: {err}")),
            kind,
            vec![],
        );
        message.enqueued_at = now - ChronoDuration::seconds(enqueued_offset_s);
        DeadLetter {
            message,
            reason: "boom".to_string(),
            failed_at: now - ChronoDuration::seconds(failed_offset_s),
        }
    }

    fn dead_letter(
        enqueued_offset_s: i64,
        failed_offset_s: i64,
        now: chrono::DateTime<Utc>,
    ) -> DeadLetter {
        dead_letter_for(
            "demo",
            TRACE_INGESTED_KIND,
            enqueued_offset_s,
            failed_offset_s,
            now,
        )
    }

    #[test]
    fn lane_helpers_use_oldest_timestamps() {
        let now = Utc::now();
        let dead_letters = vec![dead_letter(30, 20, now), dead_letter(90, 60, now)];
        // Oldest failed_at is 60s ago; oldest enqueued is 90s ago.
        assert!(
            (lane_oldest_failure_seconds(&dead_letters, TRACE_INGESTED_KIND, now) - 60.0).abs()
                < 1.5
        );
        assert!(
            (lane_oldest_enqueue_seconds(&dead_letters, TRACE_INGESTED_KIND, now) - 90.0).abs()
                < 1.5
        );
    }

    #[test]
    fn lane_helpers_empty_dlq_is_zero_age() {
        let now = Utc::now();
        assert_eq!(
            lane_oldest_failure_seconds(&[], TRACE_INGESTED_KIND, now),
            0.0
        );
        assert_eq!(
            lane_oldest_enqueue_seconds(&[], TRACE_INGESTED_KIND, now),
            0.0
        );
    }

    /// R13.6 regression for the must-fix: the per-lane gauges scan the WHOLE
    /// deployment DLQ (every tenant), not just the default tenant's subset. The
    /// helper has no tenant filter, so dead letters from any tenant are visible.
    #[test]
    fn lane_helpers_span_all_tenants_globally() {
        let now = Utc::now();
        // Dead letters from three different tenants. The old code sampled only
        // one tenant via queue_status's tenant filter and would have seen 5s.
        let dead_letters = vec![
            dead_letter_for("tenant-a", TRACE_INGESTED_KIND, 10, 5, now),
            dead_letter_for("tenant-b", TRACE_INGESTED_KIND, 40, 30, now),
            dead_letter_for("tenant-c", TRACE_INGESTED_KIND, 70, 50, now),
        ];
        // Oldest failure across tenants is 50s ago; oldest enqueue is 70s ago.
        assert!(
            (lane_oldest_failure_seconds(&dead_letters, TRACE_INGESTED_KIND, now) - 50.0).abs()
                < 1.5,
            "all tenants must be visible globally"
        );
        assert!(
            (lane_oldest_enqueue_seconds(&dead_letters, TRACE_INGESTED_KIND, now) - 70.0).abs()
                < 1.5
        );
    }

    /// R13.8: per-lane queue lag is attributed by message kind — write-batch
    /// dead letters must not bleed into the trace.ingested lane lag.
    #[test]
    fn lane_lag_is_per_lane_by_kind() {
        let now = Utc::now();
        let dead_letters = vec![
            // An old write-batch failure that should NOT affect ingested lag.
            dead_letter_for("demo", TRACE_WRITE_BATCH_KIND, 300, 290, now),
            // A newer ingested failure that DOES define the ingested lane lag.
            dead_letter_for("demo", TRACE_INGESTED_KIND, 40, 30, now),
        ];
        // Lane lag is from the ingested message (40s), not the write-batch (300s).
        assert!(
            (lane_oldest_enqueue_seconds(&dead_letters, TRACE_INGESTED_KIND, now) - 40.0).abs()
                < 1.5
        );
        // The write lane still sees its own failure age (290s).
        assert!(
            (lane_oldest_failure_seconds(&dead_letters, TRACE_WRITE_BATCH_KIND, now) - 290.0).abs()
                < 1.5
        );
    }
}

#[cfg(test)]
mod auth_default_tests {
    use super::*;

    // #127: beaterd must require auth out of the box. With no auth flags the
    // parsed mode must be Required so mutating/sensitive routes are not served
    // anonymously by default.
    #[test]
    fn auth_mode_defaults_to_required() {
        let args = Args::parse_from(["beaterd"]);
        assert_eq!(args.auth_mode, AuthModeArg::Required);
    }

    #[test]
    fn auth_mode_local_is_explicit_opt_in() {
        let args = Args::parse_from(["beaterd", "--auth-mode", "local"]);
        assert_eq!(args.auth_mode, AuthModeArg::Local);
    }

    #[test]
    fn auth_mode_required_parses() {
        let args = Args::parse_from(["beaterd", "--auth-mode", "required"]);
        assert_eq!(args.auth_mode, AuthModeArg::Required);
    }

    #[test]
    fn auth_mode_local_allows_loopback_http_bind() {
        let args = Args::parse_from(["beaterd", "--auth-mode", "local"]);
        validate_auth_mode_bind(&args).expect("default loopback bind is allowed");

        let args = Args::parse_from(["beaterd", "--auth-mode", "local", "--addr", "[::1]:8080"]);
        validate_auth_mode_bind(&args).expect("IPv6 loopback bind is allowed");
    }

    #[test]
    fn auth_mode_local_rejects_public_http_bind() {
        for addr in ["0.0.0.0:8080", "[::]:8080", "192.0.2.10:8080"] {
            let args = Args::parse_from(["beaterd", "--auth-mode", "local", "--addr", addr]);
            let err = validate_auth_mode_bind(&args)
                .expect_err("local auth must not bind HTTP publicly")
                .to_string();
            assert!(
                err.contains("--auth-mode local may only bind HTTP to loopback addresses"),
                "{err}"
            );
        }
    }

    #[test]
    fn auth_mode_required_allows_public_http_bind() {
        let args = Args::parse_from([
            "beaterd",
            "--auth-mode",
            "required",
            "--addr",
            "0.0.0.0:8080",
        ]);
        validate_auth_mode_bind(&args).expect("required auth may bind publicly");
    }
}
