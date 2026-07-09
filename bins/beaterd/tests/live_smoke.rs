use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use beater_core::{ProjectId, TenantId, TraceId, lower_hex};
use beater_otlp::encode_export_trace_request;
use beater_schema::{CanonicalTraceBatch, TraceView, WriteAck};
use beater_store::{StoreError, TraceStore};
use beater_store_sql::SqliteTraceStore;
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, trace_service_client::TraceServiceClient,
};
use opentelemetry_proto::tonic::common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{
    ResourceSpans, ScopeSpans, Span, Status, span, status,
};
use serde::Deserialize;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tonic::Request as TonicRequest;
use tonic::metadata::MetadataValue;

static LIVE_SMOKE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[tokio::test]
async fn beaterd_accepts_otlp_http_and_grpc_and_makes_traces_queryable() -> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn(tempdir.path(), http_addr, grpc_addr)?;
    let http_url = format!("http://{http_addr}");
    let grpc_url = format!("http://{grpc_addr}");

    wait_for_health(&http_url).await?;

    let (http_trace, http_span) = smoke_ids();
    let http_export = otlp_smoke_export(http_trace, http_span, "beaterd http smoke");
    reqwest::Client::new()
        .post(format!(
            "{http_url}/v1/otlp/demo/demo/local/v1/traces?durability=buffered"
        ))
        .header("content-type", "application/x-protobuf")
        .body(encode_export_trace_request(&http_export))
        .send()
        .await?
        .error_for_status()?;
    let http_trace_id = lower_hex(&http_trace);
    let http_trace = wait_for_trace(&http_url, &http_trace_id).await?;
    assert_eq!(span_count(&http_trace), 1);
    let http_search = wait_for_search_hit(&http_url, &http_trace_id).await?;
    assert_eq!(hit_count(&http_search), 1);

    let (grpc_trace, grpc_span) = smoke_ids();
    let grpc_export = otlp_smoke_export(grpc_trace, grpc_span, "beaterd grpc smoke");
    let mut client = TraceServiceClient::connect(grpc_url).await?;
    let mut request = TonicRequest::new(grpc_export);
    request
        .metadata_mut()
        .insert("x-beater-tenant-id", metadata_value("demo")?);
    request
        .metadata_mut()
        .insert("x-beater-project-id", metadata_value("demo")?);
    request
        .metadata_mut()
        .insert("x-beater-environment-id", metadata_value("local")?);
    client.export(request).await?;
    let grpc_trace_id = lower_hex(&grpc_trace);
    let grpc_trace = wait_for_trace(&http_url, &grpc_trace_id).await?;
    assert_eq!(span_count(&grpc_trace), 1);
    let grpc_search = wait_for_search_hit(&http_url, &grpc_trace_id).await?;
    assert_eq!(hit_count(&grpc_search), 1);

    Ok(())
}

const TEMPORAL_HISTORY_FIXTURE: &str =
    include_str!("../../../crates/beater-temporal/tests/fixtures/order_workflow_history.json");

#[tokio::test]
async fn beaterd_imports_temporal_history_into_queryable_trace() -> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn(tempdir.path(), http_addr, grpc_addr)?;
    let http_url = format!("http://{http_addr}");
    wait_for_health(&http_url).await?;

    // Language-agnostic import: POST a raw Temporal GetWorkflowExecutionHistory document
    // to the unified import endpoint with the `temporal_history` source selector.
    let history: serde_json::Value = serde_json::from_str(TEMPORAL_HISTORY_FIXTURE)?;
    let body = serde_json::json!({ "source": "temporal_history", "payload": history });
    reqwest::Client::new()
        .post(format!("{http_url}/v1/import/demo/demo/local"))
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    // The workflow run id becomes the trace id; the history reconstructs a span tree.
    let trace = wait_for_trace(&http_url, "11111111-1111-1111-1111-111111111111").await?;
    // workflow root + activity + timer + child workflow + signal.
    assert_eq!(span_count(&trace), 5);

    let spans = trace
        .get("spans")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("trace has no spans array"))?;
    let find = |name: &str| {
        spans
            .iter()
            .find(|span| span.get("name").and_then(serde_json::Value::as_str) == Some(name))
    };
    let kind_of = |span: &serde_json::Value| {
        span.get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string()
    };

    let root =
        find("OrderWorkflow").ok_or_else(|| anyhow::anyhow!("missing workflow root span"))?;
    assert_eq!(kind_of(root), "agent.run");
    assert_eq!(
        root.get("parent_span_id")
            .and_then(serde_json::Value::as_str),
        None
    );

    let activity = find("ValidateOrder").ok_or_else(|| anyhow::anyhow!("missing activity span"))?;
    assert_eq!(kind_of(activity), "tool.call");
    assert_eq!(
        activity
            .get("parent_span_id")
            .and_then(serde_json::Value::as_str),
        root.get("span_id").and_then(serde_json::Value::as_str)
    );

    let child = find("FulfillmentWorkflow")
        .ok_or_else(|| anyhow::anyhow!("missing child workflow span"))?;
    assert_eq!(kind_of(child), "agent.run");

    Ok(())
}

#[tokio::test]
async fn beaterd_quota_is_shared_across_two_replicas_and_resets_on_window() -> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let quota_path = tempdir.path().join("shared").join("quotas.sqlite");
    let addrs = free_addrs(4)?;
    let http_a = addrs[0];
    let grpc_a = addrs[1];
    let http_b = addrs[2];
    let grpc_b = addrs[3];
    let quota_window_seconds = 3;
    let options = BeaterdSpawnOptions {
        per_project_event_quota: Some(1),
        quota_window_seconds: Some(quota_window_seconds),
        quota_db_path: Some(quota_path),
        ..BeaterdSpawnOptions::default()
    };
    let _replica_a = BeaterdChild::spawn_with_options(
        &tempdir.path().join("replica-a"),
        http_a,
        grpc_a,
        options.clone(),
    )?;
    let http_url_a = format!("http://{http_a}");
    wait_for_health(&http_url_a).await?;

    let _replica_b = BeaterdChild::spawn_with_options(
        &tempdir.path().join("replica-b"),
        http_b,
        grpc_b,
        options,
    )?;
    let http_url_b = format!("http://{http_b}");
    wait_for_health(&http_url_b).await?;
    wait_for_quota_window_margin(quota_window_seconds as u64, Duration::from_millis(2_000)).await?;

    let first = post_otlp_http(&http_url_a, "quota replica a").await?;
    assert_eq!(first.status(), reqwest::StatusCode::OK);

    let second = post_otlp_http(&http_url_b, "quota replica b throttled").await?;
    assert_eq!(second.status(), reqwest::StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        second
            .headers()
            .get("x-ratelimit-limit")
            .and_then(|value| value.to_str().ok()),
        Some("1")
    );
    assert_eq!(
        second
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok()),
        Some("0")
    );
    let reset_at = second
        .headers()
        .get("x-ratelimit-reset")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("missing x-ratelimit-reset"))?
        .parse::<i64>()?;
    let error = second.json::<serde_json::Value>().await?;
    assert_eq!(error["error"], "too_many_requests");
    assert_eq!(error["status"], 429);
    assert!(
        error["message"]
            .as_str()
            .unwrap_or_default()
            .contains("quota exceeded")
    );

    sleep_until_unix_second(reset_at + 1).await?;
    let third = post_otlp_http(&http_url_b, "quota replica b after reset").await?;
    assert_eq!(third.status(), reqwest::StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn beaterd_consumer_kill_restart_dlq_replay_recovers_trace_ingested_work()
-> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let data_dir = tempdir.path().join("beaterd");
    let marker_path = tempdir.path().join("trace-ingested-leased");
    let hold_path = tempdir.path().join("trace-ingested-hold");
    let fail_path = tempdir.path().join("trace-ingested-fail");
    std::fs::write(&hold_path, b"hold")?;
    let addrs = free_addrs(4)?;
    let first_http = addrs[0];
    let first_grpc = addrs[1];
    let second_http = addrs[2];
    let second_grpc = addrs[3];

    let mut first = BeaterdChild::spawn_with_options(
        &data_dir,
        first_http,
        first_grpc,
        BeaterdSpawnOptions {
            trace_ingested_lease_marker: Some(marker_path.clone()),
            trace_ingested_hold_path: Some(hold_path.clone()),
            ..BeaterdSpawnOptions::default()
        },
    )?;
    let first_url = format!("http://{first_http}");
    wait_for_health(&first_url).await?;
    let trace_id = post_buffered_otlp_http(&first_url, "consumer kill restart replay").await?;
    wait_for_file(&marker_path, Duration::from_secs(5)).await?;
    assert_queue_depths(&first_url, 0, 1).await?;
    first.kill_and_wait();
    let _ = std::fs::remove_file(&hold_path);

    std::fs::write(&fail_path, b"fail")?;
    let _second = BeaterdChild::spawn_with_options(
        &data_dir,
        second_http,
        second_grpc,
        BeaterdSpawnOptions {
            trace_ingested_fail_while_path: Some(fail_path.clone()),
            ..BeaterdSpawnOptions::default()
        },
    )?;
    let second_url = format!("http://{second_http}");
    wait_for_health(&second_url).await?;
    let dead_letter = wait_for_dead_letter(
        &second_url,
        "trace.ingested",
        "test trace.ingested failure",
        Duration::from_secs(5),
    )
    .await?;
    assert_search_hit_count(&second_url, &trace_id, 0).await?;

    std::fs::remove_file(&fail_path)?;
    replay_dead_letter(&second_url, &dead_letter).await?;
    let trace = wait_for_trace(&second_url, &trace_id).await?;
    assert_eq!(span_count(&trace), 1);
    let search = wait_for_search_hit(&second_url, &trace_id).await?;
    assert_eq!(hit_count(&search), 1);
    wait_for_queue_empty(&second_url, Duration::from_secs(5)).await?;

    Ok(())
}

#[tokio::test]
async fn beaterd_trace_write_kill_replay_preserves_buffered_trace() -> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let data_dir = tempdir.path().join("beaterd");
    let marker_path = tempdir.path().join("trace-write-leased");
    let hold_path = tempdir.path().join("trace-write-hold");
    let fail_path = tempdir.path().join("trace-store-fail");
    std::fs::write(&hold_path, b"hold")?;
    let addrs = free_addrs(4)?;
    let first_http = addrs[0];
    let first_grpc = addrs[1];
    let second_http = addrs[2];
    let second_grpc = addrs[3];

    let mut first = BeaterdChild::spawn_with_options(
        &data_dir,
        first_http,
        first_grpc,
        BeaterdSpawnOptions {
            trace_write_lease_marker: Some(marker_path.clone()),
            trace_write_hold_path: Some(hold_path.clone()),
            ..BeaterdSpawnOptions::default()
        },
    )?;
    let first_url = format!("http://{first_http}");
    wait_for_health(&first_url).await?;
    let trace_id = post_buffered_otlp_http(&first_url, "trace write kill restart replay").await?;
    wait_for_file(&marker_path, Duration::from_secs(5)).await?;
    assert_queue_depths(&first_url, 1, 0).await?;
    assert_trace_span_count(&first_url, &trace_id, 0).await?;
    first.kill_and_wait();
    let _ = std::fs::remove_file(&hold_path);

    std::fs::write(&fail_path, b"fail")?;
    let _second = BeaterdChild::spawn_with_options(
        &data_dir,
        second_http,
        second_grpc,
        BeaterdSpawnOptions {
            trace_write_max_attempts: Some(1),
            trace_store_fail_write_while_path: Some(fail_path.clone()),
            ..BeaterdSpawnOptions::default()
        },
    )?;
    let second_url = format!("http://{second_http}");
    wait_for_health(&second_url).await?;
    let dead_letter = wait_for_dead_letter(
        &second_url,
        "trace.write_batch",
        "test trace store write failure",
        Duration::from_secs(5),
    )
    .await?;
    assert_trace_span_count(&second_url, &trace_id, 0).await?;
    assert_search_hit_count(&second_url, &trace_id, 0).await?;

    std::fs::remove_file(&fail_path)?;
    replay_dead_letter(&second_url, &dead_letter).await?;
    let trace = wait_for_trace(&second_url, &trace_id).await?;
    assert_eq!(span_count(&trace), 1);
    let search = wait_for_search_hit(&second_url, &trace_id).await?;
    assert_eq!(hit_count(&search), 1);
    wait_for_queue_empty(&second_url, Duration::from_secs(5)).await?;

    Ok(())
}

#[tokio::test]
async fn beaterd_external_trace_store_kill_replays_buffered_trace() -> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let data_dir = tempdir.path().join("beaterd");
    let trace_store_path = tempdir.path().join("external-trace-store.sqlite");
    let trace_store_addr = free_addr()?;
    let trace_store = ExternalTraceStoreSidecar::spawn(trace_store_addr, &trace_store_path).await?;
    let marker_path = tempdir
        .path()
        .join("trace-write-leased-before-storage-kill");
    let hold_path = tempdir.path().join("trace-write-hold-before-storage-kill");
    std::fs::write(&hold_path, b"hold")?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn_with_options(
        &data_dir,
        http_addr,
        grpc_addr,
        BeaterdSpawnOptions {
            trace_write_max_attempts: Some(1),
            trace_write_lease_marker: Some(marker_path.clone()),
            trace_write_hold_path: Some(hold_path.clone()),
            http_trace_store_url: Some(trace_store.url()),
            ..BeaterdSpawnOptions::default()
        },
    )?;
    let http_url = format!("http://{http_addr}");
    wait_for_health(&http_url).await?;

    let (trace_id, buffered_response) =
        post_otlp_http_with_durability(&http_url, "external trace store killed", Some("buffered"))
            .await?;
    assert_eq!(buffered_response.status(), reqwest::StatusCode::OK);
    let buffered_ack = buffered_response.json::<serde_json::Value>().await?;
    assert_eq!(buffered_ack["accepted_raw"], 1);
    assert_eq!(buffered_ack["accepted_spans"], 1);
    assert_eq!(buffered_ack["downstream_queued"], true);
    wait_for_file(&marker_path, Duration::from_secs(5)).await?;
    trace_store.stop().await?;
    std::fs::remove_file(&hold_path)?;
    let dead_letter = wait_for_dead_letter(
        &http_url,
        "trace.write_batch",
        "http trace store write-batch request failed",
        Duration::from_secs(5),
    )
    .await?;
    assert_only_dead_letter(&http_url, &dead_letter).await?;

    let recovered_trace_store =
        ExternalTraceStoreSidecar::spawn(trace_store_addr, &trace_store_path).await?;
    assert_trace_span_count(&http_url, &trace_id, 0).await?;
    assert_search_hit_count(&http_url, &trace_id, 0).await?;

    replay_dead_letter(&http_url, &dead_letter).await?;
    let trace = wait_for_trace(&http_url, &trace_id).await?;
    assert_eq!(span_count(&trace), 1);
    let search = wait_for_search_hit(&http_url, &trace_id).await?;
    assert_eq!(hit_count(&search), 1);
    wait_for_queue_empty(&http_url, Duration::from_secs(5)).await?;
    recovered_trace_store.stop().await?;

    Ok(())
}

#[tokio::test]
async fn beaterd_storage_failure_accounts_every_event_without_silent_drop() -> anyhow::Result<()> {
    let _guard = live_smoke_guard().await;
    let tempdir = tempfile::tempdir()?;
    let data_dir = tempdir.path().join("beaterd");
    let fail_path = tempdir.path().join("trace-store-fail");
    std::fs::write(&fail_path, b"fail")?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn_with_options(
        &data_dir,
        http_addr,
        grpc_addr,
        BeaterdSpawnOptions {
            trace_write_max_attempts: Some(1),
            trace_store_fail_write_while_path: Some(fail_path.clone()),
            ..BeaterdSpawnOptions::default()
        },
    )?;
    let http_url = format!("http://{http_addr}");
    wait_for_health(&http_url).await?;

    let (explicit_trace_id, explicit_response) =
        post_otlp_http_with_durability(&http_url, "storage chaos explicit error", None).await?;
    assert_eq!(
        explicit_response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    let explicit_error = explicit_response.json::<serde_json::Value>().await?;
    assert_eq!(explicit_error["error"], "internal_server_error");
    assert_eq!(explicit_error["status"], 500);
    assert!(
        explicit_error["message"]
            .as_str()
            .unwrap_or_default()
            .contains("test trace store write failure")
    );

    let (dlq_trace_id, dlq_response) =
        post_otlp_http_with_durability(&http_url, "storage chaos dlq", Some("buffered")).await?;
    assert_eq!(dlq_response.status(), reqwest::StatusCode::OK);
    let dead_letter = wait_for_dead_letter(
        &http_url,
        "trace.write_batch",
        "test trace store write failure",
        Duration::from_secs(5),
    )
    .await?;

    std::fs::remove_file(&fail_path)?;
    let recovered_trace_id = post_buffered_otlp_http(&http_url, "storage chaos recovered").await?;
    let recovered_trace = wait_for_trace(&http_url, &recovered_trace_id).await?;
    assert_eq!(span_count(&recovered_trace), 1);
    let recovered_search = wait_for_search_hit(&http_url, &recovered_trace_id).await?;
    assert_eq!(hit_count(&recovered_search), 1);

    assert_trace_span_count(&http_url, &explicit_trace_id, 0).await?;
    assert_trace_span_count(&http_url, &dlq_trace_id, 0).await?;
    assert_search_hit_count(&http_url, &explicit_trace_id, 0).await?;
    assert_search_hit_count(&http_url, &dlq_trace_id, 0).await?;
    assert_only_dead_letter(&http_url, &dead_letter).await?;

    let submitted_events = 3usize;
    let explicit_errors = 1usize;
    let dead_lettered = 1usize;
    let recovered = usize::from(span_count(&recovered_trace) == 1);
    let lost = submitted_events.saturating_sub(explicit_errors + dead_lettered + recovered);
    assert_eq!(lost, 0);

    Ok(())
}

struct BeaterdChild {
    child: Child,
}

#[derive(Clone, Default)]
struct BeaterdSpawnOptions {
    per_project_event_quota: Option<u64>,
    quota_window_seconds: Option<i64>,
    quota_db_path: Option<PathBuf>,
    trace_write_max_attempts: Option<u32>,
    trace_write_lease_marker: Option<PathBuf>,
    trace_write_hold_path: Option<PathBuf>,
    http_trace_store_url: Option<String>,
    trace_ingested_lease_marker: Option<PathBuf>,
    trace_ingested_hold_path: Option<PathBuf>,
    trace_ingested_fail_while_path: Option<PathBuf>,
    trace_store_fail_write_while_path: Option<PathBuf>,
}

impl BeaterdChild {
    fn spawn(
        data_dir: &Path,
        http_addr: SocketAddr,
        grpc_addr: SocketAddr,
    ) -> anyhow::Result<Self> {
        Self::spawn_with_options(
            data_dir,
            http_addr,
            grpc_addr,
            BeaterdSpawnOptions::default(),
        )
    }

    fn spawn_with_options(
        data_dir: &Path,
        http_addr: SocketAddr,
        grpc_addr: SocketAddr,
        options: BeaterdSpawnOptions,
    ) -> anyhow::Result<Self> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_beaterd"));
        command
            .arg("--addr")
            .arg(http_addr.to_string())
            .arg("--otlp-grpc-addr")
            .arg(grpc_addr.to_string())
            .arg("--data-dir")
            .arg(data_dir)
            // #127: beaterd now requires auth by default. This smoke harness
            // exercises trace/dataset routes anonymously, so opt into local.
            .arg("--auth-mode")
            .arg("local")
            .arg("--trace-write-drain-interval-ms")
            .arg("25")
            .arg("--trace-ingested-drain-interval-ms")
            .arg("25");
        if let Some(limit) = options.per_project_event_quota {
            command
                .arg("--per-project-event-quota")
                .arg(limit.to_string());
        }
        if let Some(window_seconds) = options.quota_window_seconds {
            command
                .arg("--quota-window-seconds")
                .arg(window_seconds.to_string());
        }
        if let Some(path) = options.quota_db_path {
            command.arg("--quota-db-path").arg(path);
        }
        if let Some(max_attempts) = options.trace_write_max_attempts {
            command
                .arg("--trace-write-max-attempts")
                .arg(max_attempts.to_string());
        }
        if let Some(path) = options.trace_write_lease_marker {
            command.arg("--test-trace-write-lease-marker").arg(path);
        }
        if let Some(path) = options.trace_write_hold_path {
            command.arg("--test-trace-write-hold-path").arg(path);
        }
        if let Some(url) = options.http_trace_store_url {
            command.arg("--test-http-trace-store-url").arg(url);
        }
        if let Some(path) = options.trace_ingested_lease_marker {
            command.arg("--test-trace-ingested-lease-marker").arg(path);
        }
        if let Some(path) = options.trace_ingested_hold_path {
            command.arg("--test-trace-ingested-hold-path").arg(path);
        }
        if let Some(path) = options.trace_ingested_fail_while_path {
            command
                .arg("--test-trace-ingested-fail-while-path")
                .arg(path);
        }
        if let Some(path) = options.trace_store_fail_write_while_path {
            command
                .arg("--test-trace-store-fail-write-while-path")
                .arg(path);
        }
        let child = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Self { child })
    }

    fn kill_and_wait(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for BeaterdChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct ExternalTraceStoreSidecar {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<std::io::Result<()>>>,
}

#[derive(Clone)]
struct ExternalTraceStoreState {
    traces: Arc<SqliteTraceStore>,
}

#[derive(Deserialize)]
struct TraceStoreGetTraceRequest {
    tenant_id: TenantId,
    trace_id: TraceId,
}

#[derive(Deserialize)]
struct TraceStoreGetProjectTraceRequest {
    tenant_id: TenantId,
    project_id: ProjectId,
    trace_id: TraceId,
}

impl ExternalTraceStoreSidecar {
    async fn spawn(addr: SocketAddr, trace_store_path: &Path) -> anyhow::Result<Self> {
        let traces = Arc::new(SqliteTraceStore::open(trace_store_path)?);
        let state = ExternalTraceStoreState { traces };
        let app = Router::new()
            .route("/trace-store/write-batch", post(sidecar_write_batch))
            .route("/trace-store/get-trace", post(sidecar_get_trace))
            .route(
                "/trace-store/get-project-trace",
                post(sidecar_get_project_trace),
            )
            .with_state(state);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        let (shutdown, shutdown_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
        });
        Ok(Self {
            addr,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    async fn stop(mut self) -> anyhow::Result<()> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.task.take() {
            task.await
                .map_err(|err| anyhow::anyhow!("trace store sidecar join failed: {err}"))??;
        }
        Ok(())
    }
}

impl Drop for ExternalTraceStoreSidecar {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

async fn sidecar_write_batch(
    State(state): State<ExternalTraceStoreState>,
    Json(batch): Json<CanonicalTraceBatch>,
) -> Result<Json<WriteAck>, (StatusCode, String)> {
    state
        .traces
        .write_batch(Arc::new(batch))
        .await
        .map(Json)
        .map_err(sidecar_store_error)
}

async fn sidecar_get_trace(
    State(state): State<ExternalTraceStoreState>,
    Json(request): Json<TraceStoreGetTraceRequest>,
) -> Result<Json<TraceView>, (StatusCode, String)> {
    state
        .traces
        .get_trace(request.tenant_id, request.trace_id)
        .await
        .map(Json)
        .map_err(sidecar_store_error)
}

async fn sidecar_get_project_trace(
    State(state): State<ExternalTraceStoreState>,
    Json(request): Json<TraceStoreGetProjectTraceRequest>,
) -> Result<Json<TraceView>, (StatusCode, String)> {
    state
        .traces
        .get_project_trace(request.tenant_id, request.project_id, request.trace_id)
        .await
        .map(Json)
        .map_err(sidecar_store_error)
}

fn sidecar_store_error(error: StoreError) -> (StatusCode, String) {
    let status = match error {
        StoreError::NotFound(_) => StatusCode::NOT_FOUND,
        StoreError::Conflict(_) => StatusCode::CONFLICT,
        StoreError::Backpressure(_) => StatusCode::SERVICE_UNAVAILABLE,
        StoreError::LimitExceeded(_) => StatusCode::PAYLOAD_TOO_LARGE,
        StoreError::Integrity(_) => StatusCode::UNPROCESSABLE_ENTITY,
        StoreError::Backend(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}

async fn live_smoke_guard() -> tokio::sync::MutexGuard<'static, ()> {
    LIVE_SMOKE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn wait_for_health(http_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(response) = client.get(format!("{http_url}/health")).send().await
            && response.status().is_success()
        {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("beaterd did not become healthy at {http_url}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_trace(http_url: &str, trace_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let url = format!("{http_url}/v1/traces/demo/{trace_id}");
    loop {
        let trace = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        if span_count(&trace) > 0 {
            return Ok(trace);
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("trace {trace_id} was not queryable at {url}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_search_hit(http_url: &str, trace_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let url = search_url(http_url, trace_id);
    loop {
        let response = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        if hit_count(&response) > 0 {
            return Ok(response);
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("trace {trace_id} was not searchable at {url}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn search_url(http_url: &str, trace_id: &str) -> String {
    format!(
        "{http_url}/v1/search/demo/spans?project_id=demo&environment_id=local&trace_id={trace_id}&kind=llm.call&status=ok"
    )
}

async fn wait_for_quota_window_margin(
    window_seconds: u64,
    min_remaining: Duration,
) -> anyhow::Result<()> {
    let window_millis = u128::from(window_seconds.max(1)) * 1_000;
    let min_remaining_millis = min_remaining.as_millis();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let now_millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let elapsed = now_millis % window_millis;
        let remaining = window_millis.saturating_sub(elapsed);
        if remaining >= min_remaining_millis {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("quota window did not expose enough remaining time");
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn sleep_until_unix_second(target: i64) -> anyhow::Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    if target > now {
        tokio::time::sleep(Duration::from_secs((target - now) as u64)).await;
    }
    Ok(())
}

async fn post_otlp_http(http_url: &str, name: &str) -> anyhow::Result<reqwest::Response> {
    let (_, response) = post_otlp_http_with_durability(http_url, name, None).await?;
    Ok(response)
}

async fn post_otlp_http_with_durability(
    http_url: &str,
    name: &str,
    durability: Option<&str>,
) -> anyhow::Result<(String, reqwest::Response)> {
    let (trace, span) = smoke_ids();
    let export = otlp_smoke_export(trace, span, name);
    let mut url = format!("{http_url}/v1/otlp/demo/demo/local/v1/traces");
    if let Some(durability) = durability {
        url.push_str("?durability=");
        url.push_str(durability);
    }
    let response = reqwest::Client::new()
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(encode_export_trace_request(&export))
        .send()
        .await?;
    Ok((lower_hex(&trace), response))
}

async fn post_buffered_otlp_http(http_url: &str, name: &str) -> anyhow::Result<String> {
    let (trace, span) = smoke_ids();
    let export = otlp_smoke_export(trace, span, name);
    reqwest::Client::new()
        .post(format!(
            "{http_url}/v1/otlp/demo/demo/local/v1/traces?durability=buffered"
        ))
        .header("content-type", "application/x-protobuf")
        .body(encode_export_trace_request(&export))
        .send()
        .await?
        .error_for_status()?;
    Ok(lower_hex(&trace))
}

async fn wait_for_file(path: &Path, timeout: Duration) -> anyhow::Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if path.exists() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("file did not appear: {}", path.display());
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn queue_status(http_url: &str) -> anyhow::Result<serde_json::Value> {
    Ok(reqwest::Client::new()
        .get(format!("{http_url}/v1/ingest/demo/demo/queue"))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?)
}

async fn assert_queue_depths(
    http_url: &str,
    expected_trace_write: u64,
    expected_trace_ingested: u64,
) -> anyhow::Result<()> {
    let status = queue_status(http_url).await?;
    let trace_write_depth = status
        .get("trace_write_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let trace_ingested_depth = status
        .get("trace_ingested_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    if trace_write_depth != expected_trace_write || trace_ingested_depth != expected_trace_ingested
    {
        anyhow::bail!(
            "expected queue depths trace.write={expected_trace_write} trace.ingested={expected_trace_ingested}, got {status}"
        );
    }
    Ok(())
}

async fn assert_only_dead_letter(http_url: &str, message_id: &str) -> anyhow::Result<()> {
    let status = queue_status(http_url).await?;
    let total_depth = status
        .get("total_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let trace_write_depth = status
        .get("trace_write_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let trace_ingested_depth = status
        .get("trace_ingested_depth")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    let dead_letters = status
        .get("dead_letters")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(total_depth, 0, "unexpected live queue depth");
    assert_eq!(trace_write_depth, 0, "unexpected trace.write queue depth");
    assert_eq!(
        trace_ingested_depth, 0,
        "unexpected trace.ingested queue depth"
    );
    assert_eq!(dead_letters.len(), 1);
    let actual_message_id = dead_letters[0]
        .get("message")
        .and_then(|message| message.get("message_id"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("dead letter missing message id"))?;
    assert_eq!(actual_message_id, message_id);
    Ok(())
}

async fn wait_for_dead_letter(
    http_url: &str,
    kind: &str,
    reason_contains: &str,
    timeout: Duration,
) -> anyhow::Result<String> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let status = queue_status(http_url).await?;
        if let Some(dead_letters) = status
            .get("dead_letters")
            .and_then(serde_json::Value::as_array)
        {
            for dead_letter in dead_letters {
                let message = dead_letter
                    .get("message")
                    .ok_or_else(|| anyhow::anyhow!("dead letter missing message"))?;
                let message_kind = message
                    .get("kind")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let reason = dead_letter
                    .get("reason")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if message_kind == kind && reason.contains(reason_contains) {
                    return message
                        .get("message_id")
                        .and_then(serde_json::Value::as_str)
                        .map(ToString::to_string)
                        .ok_or_else(|| anyhow::anyhow!("dead letter missing message_id"));
                }
            }
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("dead letter {kind} with reason {reason_contains:?} did not appear");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn replay_dead_letter(http_url: &str, message_id: &str) -> anyhow::Result<()> {
    let report = reqwest::Client::new()
        .post(format!(
            "{http_url}/v1/ingest/demo/demo/dead-letters/{message_id}/replay"
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    assert_eq!(
        report.get("message_id").and_then(serde_json::Value::as_str),
        Some(message_id)
    );
    assert_eq!(
        report
            .get("reset_attempts")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("ack")
            .and_then(|ack| ack.get("accepted"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    Ok(())
}

async fn assert_search_hit_count(
    http_url: &str,
    trace_id: &str,
    expected: usize,
) -> anyhow::Result<()> {
    let response = reqwest::Client::new()
        .get(search_url(http_url, trace_id))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let actual = hit_count(&response);
    if actual != expected {
        anyhow::bail!("expected {expected} search hits for {trace_id}, got {actual}");
    }
    Ok(())
}

async fn assert_trace_span_count(
    http_url: &str,
    trace_id: &str,
    expected: usize,
) -> anyhow::Result<()> {
    let trace = reqwest::Client::new()
        .get(format!("{http_url}/v1/traces/demo/{trace_id}"))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let actual = span_count(&trace);
    if actual != expected {
        anyhow::bail!("expected {expected} spans for {trace_id}, got {actual}");
    }
    Ok(())
}

async fn wait_for_queue_empty(http_url: &str, timeout: Duration) -> anyhow::Result<()> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let status = queue_status(http_url).await?;
        let trace_write_depth = status
            .get("trace_write_depth")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        let trace_ingested_depth = status
            .get("trace_ingested_depth")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        let dead_letters = status
            .get("dead_letters")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
        if trace_write_depth == 0 && trace_ingested_depth == 0 && dead_letters == 0 {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("queue did not become empty: {status}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn span_count(trace: &serde_json::Value) -> usize {
    trace
        .get("spans")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or_default()
}

fn hit_count(response: &serde_json::Value) -> usize {
    response
        .get("hits")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or_default()
}

fn free_addr() -> anyhow::Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?)
}

fn free_addrs(count: usize) -> anyhow::Result<Vec<SocketAddr>> {
    let mut addrs = Vec::with_capacity(count);
    while addrs.len() < count {
        let addr = free_addr()?;
        if !addrs.contains(&addr) {
            addrs.push(addr);
        }
    }
    Ok(addrs)
}

fn metadata_value(value: &str) -> anyhow::Result<MetadataValue<tonic::metadata::Ascii>> {
    value
        .parse()
        .map_err(|err| anyhow::anyhow!("invalid gRPC metadata value {value:?}: {err}"))
}

fn smoke_ids() -> ([u8; 16], [u8; 8]) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let trace = now.to_be_bytes();
    let span = (now as u64).to_be_bytes();
    (trace, span)
}

fn otlp_smoke_export(
    trace_id: [u8; 16],
    span_id: [u8; 8],
    name: &str,
) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![otel_kv("service.name", otel_string("beaterd-live-smoke"))],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope {
                    name: "beaterd-live-smoke".to_string(),
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
                    name: name.to_string(),
                    kind: span::SpanKind::Client as i32,
                    start_time_unix_nano: 1_700_000_000_000_000_000,
                    end_time_unix_nano: 1_700_000_001_000_000_000,
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
