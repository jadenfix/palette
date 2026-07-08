use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const OSS_QUERY_LAG_SLO_MS: u64 = 15_000;
static REMOTE_SMOKE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn remote_smoke_http_reports_query_lag_under_oss_slo() -> anyhow::Result<()> {
    let _guard = REMOTE_SMOKE_LOCK.lock().await;
    let tempdir = tempfile::tempdir()?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn(tempdir.path().to_path_buf(), http_addr, grpc_addr)?;
    let http_url = format!("http://{http_addr}");
    wait_for_health(&http_url).await?;

    let smoke = run_beaterctl_smoke(&http_url, None, None)?;
    assert_eq!(smoke["mode"], "remote");
    assert_eq!(smoke["protocol"], "http");
    assert_eq!(smoke["source"], "otlp");
    assert_eq!(smoke["trace_span_count"], 1);
    assert_lag_under_slo(&smoke);

    Ok(())
}

#[tokio::test]
async fn ingest_test_reports_trace_and_zero_code_env() -> anyhow::Result<()> {
    let _guard = REMOTE_SMOKE_LOCK.lock().await;
    let tempdir = tempfile::tempdir()?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn(tempdir.path().to_path_buf(), http_addr, grpc_addr)?;
    let http_url = format!("http://{http_addr}");
    wait_for_health(&http_url).await?;

    let output = run_beaterctl_ingest_test(&http_url)?;
    assert_eq!(output["command"], "ingest test");
    assert_eq!(output["mode"], "remote");
    assert_eq!(output["protocol"], "http");
    assert_eq!(output["source"], "otlp");
    assert_eq!(output["trace_span_count"], 1);
    assert_lag_under_slo(&output);

    let env = &output["zero_code_env"];
    assert_eq!(env["BEATER_TENANT_ID"], "demo");
    assert_eq!(env["BEATER_PROJECT_ID"], "demo");
    assert_eq!(env["BEATER_ENVIRONMENT_ID"], "local");
    assert_eq!(env["OTEL_EXPORTER_OTLP_PROTOCOL"], "http/protobuf");
    assert_eq!(env["OTEL_EXPORTER_OTLP_HEADERS"], "");
    assert_eq!(
        env["OTEL_EXPORTER_OTLP_TRACES_ENDPOINT"],
        format!("{http_url}/v1/otlp/demo/demo/local/v1/traces")
    );

    Ok(())
}

#[tokio::test]
async fn remote_smoke_grpc_reports_query_lag_under_oss_slo() -> anyhow::Result<()> {
    let _guard = REMOTE_SMOKE_LOCK.lock().await;
    let tempdir = tempfile::tempdir()?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let _server = BeaterdChild::spawn(tempdir.path().to_path_buf(), http_addr, grpc_addr)?;
    let http_url = format!("http://{http_addr}");
    let grpc_url = format!("http://{grpc_addr}");
    wait_for_health(&http_url).await?;

    let smoke = run_beaterctl_smoke(&http_url, Some(&grpc_url), None)?;
    assert_eq!(smoke["mode"], "remote");
    assert_eq!(smoke["protocol"], "grpc");
    assert_eq!(smoke["source"], "otlp");
    assert_eq!(smoke["trace_span_count"], 1);
    assert_lag_under_slo(&smoke);

    Ok(())
}

#[tokio::test]
async fn remote_smoke_http_strict_auth_reads_back_trace() -> anyhow::Result<()> {
    let _guard = REMOTE_SMOKE_LOCK.lock().await;
    let tempdir = tempfile::tempdir()?;
    let addrs = free_addrs(2)?;
    let http_addr = addrs[0];
    let grpc_addr = addrs[1];
    let api_key = create_trace_smoke_key(tempdir.path())?;
    let _server = BeaterdChild::spawn_with_auth_mode(
        tempdir.path().to_path_buf(),
        http_addr,
        grpc_addr,
        "required",
    )?;
    let http_url = format!("http://{http_addr}");
    wait_for_health(&http_url).await?;

    let smoke = run_beaterctl_smoke(&http_url, None, Some(&api_key))?;
    assert_eq!(smoke["mode"], "remote");
    assert_eq!(smoke["protocol"], "http");
    assert_eq!(smoke["source"], "otlp");
    assert_eq!(smoke["trace_span_count"], 1);
    assert_lag_under_slo(&smoke);

    Ok(())
}

fn run_beaterctl_ingest_test(http_url: &str) -> anyhow::Result<serde_json::Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("ingest")
        .arg("test")
        .arg("--http-url")
        .arg(http_url)
        .arg("--tenant-id")
        .arg("demo")
        .arg("--project-id")
        .arg("demo")
        .arg("--environment-id")
        .arg("local")
        .arg("--timeout-ms")
        .arg(OSS_QUERY_LAG_SLO_MS.to_string())
        .output()?;
    assert!(
        output.status.success(),
        "beaterctl ingest test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).map_err(anyhow::Error::from)
}

struct BeaterdChild {
    child: Child,
}

impl BeaterdChild {
    fn spawn(
        data_dir: PathBuf,
        http_addr: SocketAddr,
        grpc_addr: SocketAddr,
    ) -> anyhow::Result<Self> {
        // beaterd defaults to --auth-mode required (b728b9e / #127); most of this
        // smoke harness ingests anonymously, so opt into insecure local explicitly.
        Self::spawn_with_auth_mode(data_dir, http_addr, grpc_addr, "local")
    }

    fn spawn_with_auth_mode(
        data_dir: PathBuf,
        http_addr: SocketAddr,
        grpc_addr: SocketAddr,
        auth_mode: &str,
    ) -> anyhow::Result<Self> {
        let child = Command::new(beaterd_bin()?)
            .arg("--addr")
            .arg(http_addr.to_string())
            .arg("--otlp-grpc-addr")
            .arg(grpc_addr.to_string())
            .arg("--data-dir")
            .arg(data_dir)
            .arg("--auth-mode")
            .arg(auth_mode)
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

fn run_beaterctl_smoke(
    http_url: &str,
    otlp_grpc_url: Option<&str>,
    api_key: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_beaterctl"));
    if let Some(api_key) = api_key {
        command.arg("--api-key").arg(api_key);
    }
    command
        .arg("smoke")
        .arg("--http-url")
        .arg(http_url)
        .arg("--tenant-id")
        .arg("demo")
        .arg("--project-id")
        .arg("demo")
        .arg("--environment-id")
        .arg("local")
        .arg("--timeout-ms")
        .arg(OSS_QUERY_LAG_SLO_MS.to_string());
    if let Some(url) = otlp_grpc_url {
        command.arg("--otlp-grpc-url").arg(url);
    }
    let output = command.output()?;
    assert!(
        output.status.success(),
        "beaterctl smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).map_err(anyhow::Error::from)
}

fn create_trace_smoke_key(data_dir: &Path) -> anyhow::Result<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("api-key-create")
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--tenant-id")
        .arg("demo")
        .arg("--project-id")
        .arg("demo")
        .arg("--environment-id")
        .arg("local")
        .arg("--scopes")
        .arg("trace-write")
        .arg("--scopes")
        .arg("trace-read")
        .output()?;
    assert!(
        output.status.success(),
        "beaterctl api-key-create failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let body: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    body["secret"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("api-key-create did not return secret: {body}"))
}

fn assert_lag_under_slo(smoke: &serde_json::Value) {
    let lag_ms = smoke["trace_query_lag_ms"]
        .as_u64()
        .unwrap_or_else(|| panic!("missing trace_query_lag_ms in {smoke}"));
    assert!(
        lag_ms <= OSS_QUERY_LAG_SLO_MS,
        "trace query lag {lag_ms}ms exceeded {OSS_QUERY_LAG_SLO_MS}ms"
    );
}

async fn wait_for_health(http_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
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

fn beaterd_bin() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_beaterd") {
        return Ok(PathBuf::from(path));
    }
    let current = std::env::current_exe()?;
    let debug_dir = current
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| anyhow::anyhow!("cannot derive target/debug from {}", current.display()))?;
    let candidate = debug_dir.join(format!("beaterd{}", std::env::consts::EXE_SUFFIX));
    if candidate.exists() {
        return Ok(candidate);
    }
    let status = Command::new("cargo")
        .arg("build")
        .arg("-q")
        .arg("-p")
        .arg("beaterd")
        .status()?;
    if !status.success() {
        anyhow::bail!("cargo build -p beaterd failed with {status}");
    }
    Ok(candidate)
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
