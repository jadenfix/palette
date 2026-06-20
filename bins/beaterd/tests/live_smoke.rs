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
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tonic::metadata::MetadataValue;
use tonic::Request as TonicRequest;

#[tokio::test]
async fn beaterd_accepts_otlp_http_and_grpc_and_makes_traces_queryable() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let http_addr = free_addr()?;
    let grpc_addr = free_addr()?;
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

struct BeaterdChild {
    child: Child,
}

impl BeaterdChild {
    fn spawn(
        data_dir: &std::path::Path,
        http_addr: SocketAddr,
        grpc_addr: SocketAddr,
    ) -> anyhow::Result<Self> {
        let child = Command::new(env!("CARGO_BIN_EXE_beaterd"))
            .arg("--addr")
            .arg(http_addr.to_string())
            .arg("--otlp-grpc-addr")
            .arg(grpc_addr.to_string())
            .arg("--data-dir")
            .arg(data_dir)
            .arg("--trace-write-drain-interval-ms")
            .arg("25")
            .arg("--trace-ingested-drain-interval-ms")
            .arg("25")
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

async fn wait_for_health(http_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
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
