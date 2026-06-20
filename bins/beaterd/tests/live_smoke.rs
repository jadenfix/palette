use beater_otlp::encode_export_trace_request;
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, InstrumentationScope, KeyValue};
use opentelemetry_proto::tonic::resource::v1::Resource;
use opentelemetry_proto::tonic::trace::v1::{
    span, status, ResourceSpans, ScopeSpans, Span, Status,
};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tonic::metadata::MetadataValue;
use tonic::Request as TonicRequest;

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
    let http_trace_id = hex(&http_trace);
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
    let grpc_trace_id = hex(&grpc_trace);
    let grpc_trace = wait_for_trace(&http_url, &grpc_trace_id).await?;
    assert_eq!(span_count(&grpc_trace), 1);
    let grpc_search = wait_for_search_hit(&http_url, &grpc_trace_id).await?;
    assert_eq!(hit_count(&grpc_search), 1);

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
    assert_eq!(error["status"], 429);
    assert!(error["error"]
        .as_str()
        .unwrap_or_default()
        .contains("quota exceeded"));

    sleep_until_unix_second(reset_at + 1).await?;
    let third = post_otlp_http(&http_url_b, "quota replica b after reset").await?;
    assert_eq!(third.status(), reqwest::StatusCode::OK);

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
        let child = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Self { child })
    }
}

impl Drop for BeaterdChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
        if let Ok(response) = client.get(format!("{http_url}/health")).send().await {
            if response.status().is_success() {
                return Ok(());
            }
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
    let url = format!(
        "{http_url}/v1/search/demo/spans?project_id=demo&environment_id=local&trace_id={trace_id}&kind=llm.call&status=ok"
    );
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
    let (trace, span) = smoke_ids();
    let export = otlp_smoke_export(trace, span, name);
    let response = reqwest::Client::new()
        .post(format!("{http_url}/v1/otlp/demo/demo/local/v1/traces"))
        .header("content-type", "application/x-protobuf")
        .body(encode_export_trace_request(&export))
        .send()
        .await?;
    Ok(response)
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
