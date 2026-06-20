use std::fs;
use std::path::PathBuf;

#[test]
fn self_host_files_define_gate_two_compose_surface() {
    let root = repo_root();
    let compose = read(root.join("docker-compose.yml"));
    for service in ["postgres:", "nats:", "minio:", "beaterd:", "dashboard:"] {
        assert!(
            compose.contains(service),
            "docker-compose.yml must define {service}"
        );
    }
    assert!(compose.contains("profiles: [\"clickhouse\"]"));
    assert!(compose.contains("http://beaterd:8080"));
    assert!(compose.contains("http://beaterd:4317"));
    assert!(compose.contains("${BEATER_HTTP_PORT:-8080}:8080"));
    assert!(compose.contains("${BEATER_DASHBOARD_PORT:-3000}:3000"));
    assert!(compose.contains("target: runtime"));
    assert!(compose.contains("target: tools"));

    let dockerfile = read(root.join("Dockerfile"));
    assert!(dockerfile.contains("cargo install cargo-chef --locked"));
    assert!(dockerfile.contains("FROM chef AS planner"));
    assert!(dockerfile.contains("cargo chef prepare --recipe-path recipe.json"));
    assert!(dockerfile.contains("FROM chef AS rust-deps"));
    assert!(dockerfile.contains("COPY --from=planner /app/recipe.json recipe.json"));
    assert!(dockerfile.contains("cargo chef cook --release --recipe-path recipe.json"));
    assert!(dockerfile.contains("FROM rust-deps AS beaterd-builder"));
    assert!(dockerfile.contains("FROM rust-deps AS beaterctl-builder"));
    assert!(dockerfile.contains("FROM runtime AS tools"));

    let dashboard_dockerfile = read(root.join("web/dashboard/Dockerfile"));
    assert!(dashboard_dockerfile.contains("npm run build"));
    assert!(dashboard_dockerfile.contains("server.js"));

    let prebuilt_compose = read(root.join("docker-compose.prebuilt.yml"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/beaterd:main"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/dashboard:main"));
    assert!(!prebuilt_compose.contains("build:"));

    let image_workflow = read(root.join(".github/workflows/container-images.yml"));
    assert!(image_workflow.contains("packages: write"));
    assert!(image_workflow.contains("ubuntu-24.04-arm"));
    assert!(image_workflow.contains("platform: linux/arm64"));
    assert!(image_workflow.contains("docker buildx imagetools create"));
    assert!(image_workflow.contains("context: ."));
    assert!(image_workflow.contains("target: runtime"));
    assert!(image_workflow.contains("Build beaterctl tools target"));
    assert!(image_workflow.contains("target: tools"));
    assert!(image_workflow.contains("push: false"));
    assert!(image_workflow.contains("cache-from: type=gha,scope=beaterd-${{ matrix.suffix }}"));
    assert!(
        image_workflow.contains("cache-to: type=gha,mode=max,scope=beaterd-${{ matrix.suffix }}")
    );
    assert!(image_workflow
        .contains("cache-to: type=gha,mode=max,scope=beaterctl-tools-${{ matrix.suffix }}"));
    assert!(image_workflow.contains("ghcr.io/${{ github.repository }}/beaterd:main"));
    assert!(image_workflow.contains("ghcr.io/${{ github.repository }}/dashboard:main"));
}

#[test]
fn migrations_cover_trace_store_and_queue_contracts() {
    let root = repo_root();
    let sqlite = read(root.join("migrations/sqlite/0001_local_beaterd.sql"));
    let postgres = read(root.join("migrations/postgres/0001_initial.sql"));
    for table in [
        "raw_envelopes",
        "spans",
        "quota_counters",
        "queue_messages",
        "inflight_messages",
        "dead_letters",
        "api_keys",
        "datasets",
        "gate_runs",
        "audit_events",
        "replay_events",
    ] {
        assert!(
            sqlite.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "sqlite migration must create {table}"
        );
        assert!(
            postgres.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "postgres migration must create {table}"
        );
    }
    assert!(sqlite.contains("api_key_id TEXT PRIMARY KEY"));
    assert!(sqlite.contains("provider_secret_id TEXT PRIMARY KEY"));
    assert!(sqlite.contains("PRIMARY KEY (tenant_id, project_id, trace_id, span_id, seq)"));
    assert!(postgres.contains("PRIMARY KEY (tenant_id, project_id, trace_id, span_id, seq)"));
    assert!(postgres.contains("idx_spans_tenant_kind_status"));

    let clickhouse = read(root.join("migrations/clickhouse/0001_trace_store.sql"));
    assert!(clickhouse.contains("CREATE TABLE IF NOT EXISTS beater.raw_envelopes"));
    assert!(clickhouse.contains("CREATE TABLE IF NOT EXISTS beater.spans"));
    assert!(clickhouse.contains("ORDER BY (tenant_id, project_id, environment_id, trace_id"));
    assert!(clickhouse.contains("CREATE MATERIALIZED VIEW IF NOT EXISTS beater.trace_runs_mv"));
}

#[test]
fn clean_clone_smoke_uses_stock_otel_and_browser_visible_trace() {
    let root = repo_root();
    let compose_script = read(root.join("scripts/smoke-compose.sh"));
    assert!(compose_script.contains("docker compose"));
    assert!(compose_script.contains("compose run --rm beaterctl"));
    assert!(compose_script.contains("compose run --rm otel-python-smoke"));
    assert!(compose_script.contains("call-policy-model"));
    assert!(compose_script.contains("Agent Trace Debugger"));

    let proof_script = read(root.join("scripts/gate2-proof.sh"));
    assert!(proof_script.contains("beaterctl\" smoke"));
    assert!(proof_script.contains("examples/python/otel_smoke.py"));
    assert!(proof_script.contains("examples/python/five_line_otel.py"));
    assert!(proof_script.contains("quickstart_trace_id"));
    assert!(proof_script.contains("npx playwright test tests/e2e/dashboard.spec.ts"));
    assert!(proof_script.contains("npm run test:e2e:quickstart"));
    assert!(proof_script.contains("npm run record:gate2"));
    assert!(proof_script.contains("scripts/check-openapi-drift.sh"));

    let stopwatch_script = read(root.join("scripts/gate2-compose-stopwatch.sh"));
    assert!(stopwatch_script.contains("docker compose"));
    assert!(stopwatch_script.contains("python3 -m venv"));
    assert!(stopwatch_script.contains("examples/python/five_line_otel.py"));
    assert!(stopwatch_script.contains("examples/python/otel_smoke.py"));
    assert!(stopwatch_script.contains("OTEL_EXPORTER_OTLP_ENDPOINT"));
    assert!(stopwatch_script.contains("duration_seconds > 300"));
    assert!(stopwatch_script.contains("time_to_first_trace_seconds > 300"));
    assert!(stopwatch_script.contains("time_to_quickstart_click_seconds > 300"));
    assert!(stopwatch_script.contains("git rev-parse HEAD"));
    assert!(stopwatch_script.contains("docker compose version"));
    assert!(stopwatch_script.contains("compose images"));
    assert!(stopwatch_script.contains("BEATER_GATE2_WRITE_PROOF"));
    assert!(stopwatch_script.contains("BEATER_GATE2_BROWSER_PROOF"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RECORD_DEMO"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RECORD_MODE=compose"));
    assert!(stopwatch_script.contains("BEATER_E2E_QUICKSTART_TRACE_ID"));
    assert!(stopwatch_script.contains("gate2-compose-browser-demo.webm"));
    assert!(stopwatch_script.contains("Browser recording SHA256"));
    assert!(stopwatch_script.contains("sha256_file"));
    assert!(stopwatch_script.contains("npm run test:e2e:quickstart"));
    assert!(stopwatch_script.contains("npx playwright test tests/e2e/dashboard.spec.ts"));
    assert!(stopwatch_script.contains("all_kind_trace_id"));
    assert!(stopwatch_script.contains("all-kind nested agent waterfall"));
    assert!(stopwatch_script.contains("PLAYWRIGHT_BASE_URL"));
    assert!(stopwatch_script.contains("BEATER_GATE2_REUSE"));
    assert!(stopwatch_script.contains("clean_start"));
    assert!(stopwatch_script.contains("compose down -v --remove-orphans"));
    assert!(stopwatch_script.contains("rm -rf \"$venv_dir\""));
    assert!(stopwatch_script.contains("docker-compose.prebuilt.yml"));
    assert!(stopwatch_script.contains("BEATER_GATE2_PULL_POLICY"));
    assert!(stopwatch_script.contains("--pull \"$prebuilt_pull_policy\""));
    assert!(stopwatch_script.contains("BEATER_GATE2_LOCAL_BUILD"));

    let outside_proof = read(root.join("docs/demos/gate2-outside-person-proof.md"));
    assert!(outside_proof.contains("Status: not yet completed"));
    assert!(outside_proof.contains("BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1"));
    assert!(outside_proof.contains("Docker Compose version"));
    assert!(outside_proof.contains("Beater image digest"));
    assert!(outside_proof.contains("Screen recording SHA256"));
    assert!(outside_proof.contains("gate2-compose-browser-demo.webm"));
    assert!(outside_proof.contains("gate2-compose-browser-demo.md"));
    assert!(outside_proof.contains("Terminal output excerpt"));
    assert!(outside_proof.contains("http://127.0.0.1:3000/"));
    assert!(outside_proof.contains("Time-to-first-trace was 300 seconds or less"));
    assert!(outside_proof.contains("Time-to-quickstart-click was 300 seconds or less"));
    assert!(outside_proof.contains("run -> turn -> step -> tool -> MCP"));
    assert!(outside_proof.contains("using only public repository instructions"));

    let readme = read(root.join("README.md"));
    assert!(readme.contains("docs/demos/gate2-outside-person-proof.md"));
    assert!(readme.contains("BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh"));
    assert!(readme.contains("gate2-compose-browser-demo.webm"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert!(requirements.contains("docs/demos/gate2-outside-person-proof.md"));

    let compose = read(root.join("docker-compose.yml"));
    assert!(compose.contains("otel-python-quickstart"));
    assert!(compose.contains("five_line_otel.py"));

    let quickstart = read(root.join("examples/python/five_line_otel.py"));
    let code_lines = quickstart
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .count();
    assert_eq!(
        code_lines, 5,
        "quickstart OTEL snippet must remain five executable lines"
    );
    assert!(quickstart.contains("opentelemetry"));
    assert!(quickstart.contains("OTLPSpanExporter"));
    assert!(quickstart.contains("five-line-llm-call"));
    assert!(quickstart.contains("gpt-quickstart"));
    assert!(!quickstart.contains("beaterctl"));

    let python = read(root.join("examples/python/otel_smoke.py"));
    assert!(python.contains("opentelemetry.exporter.otlp"));
    assert!(python.contains("openinference.span.kind"));
    for kind in [
        "agent.run",
        "agent.turn",
        "agent.plan",
        "agent.step",
        "llm.call",
        "tool.call",
        "mcp.request",
        "retrieval.query",
        "memory.read",
        "memory.write",
        "guardrail.check",
        "human.review",
        "evaluator.run",
        "replay.run",
    ] {
        assert!(python.contains(kind), "python OTLP smoke must emit {kind}");
    }
    assert!(python.contains("llm.model_name"));
    assert!(python.contains("llm.cost.amount_micros"));
    assert!(!python.contains("beaterctl"));

    let record_script = read(root.join("web/dashboard/tests/e2e/record-gate2-demo.mjs"));
    assert!(record_script.contains("recordVideo"));
    assert!(record_script.contains("BEATER_GATE2_RECORD_MODE"));
    assert!(record_script.contains("recordQuickstartFlow"));
    assert!(record_script.contains("recordAllKindFlow"));
    assert!(record_script.contains("gate2-compose-browser-demo.webm"));
    assert!(record_script.contains("createHash(\"sha256\")"));
    assert!(record_script.contains("docs/demos"));
    assert!(record_script.contains("gate2-browser-demo.webm"));

    let compose_recording_notes = read(root.join("docs/demos/gate2-compose-browser-demo.md"));
    assert!(compose_recording_notes.contains("# Gate 2 Compose Browser Demo"));
    assert!(compose_recording_notes.contains("alternate host ports"));
    assert!(compose_recording_notes.contains("http://127.0.0.1:3000"));
    assert!(compose_recording_notes.contains("gate2-compose-browser-demo.webm"));
    assert!(compose_recording_notes.contains("SHA256"));
    assert!(compose_recording_notes.contains("Quickstart trace"));
    assert!(compose_recording_notes.contains("All-kind trace"));
    assert!(compose_recording_notes.contains("click five-line trace"));
    assert!(compose_recording_notes.contains("docs/demos/gate2-outside-person-proof.md"));

    let compose_recording = root.join("docs/demos/gate2-compose-browser-demo.webm");
    let metadata = fs::metadata(&compose_recording)
        .unwrap_or_else(|err| panic!("stat {}: {err}", compose_recording.display()));
    assert!(
        metadata.len() > 64 * 1024,
        "Gate 2 compose recording must be a committed non-empty browser video"
    );

    let quickstart_e2e = read(root.join("web/dashboard/tests/e2e/quickstart.spec.ts"));
    assert!(quickstart_e2e.contains("five-line-llm-call"));
    assert!(quickstart_e2e.contains("gpt-quickstart"));
    assert!(quickstart_e2e.contains("kind=llm.call&model=gpt-quickstart"));
    assert!(quickstart_e2e.contains("traceRow.click()"));
    assert!(quickstart_e2e.contains("toHaveURL"));
    assert!(quickstart_e2e.contains("hello from stock OpenTelemetry"));
    assert!(quickstart_e2e.contains("hello from Beater"));
    assert!(quickstart_e2e.contains("data-icon"));
}

fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = manifest.parent().and_then(|path| path.parent()) else {
        panic!("beaterd manifest must live under bins/beaterd");
    };
    root.to_path_buf()
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
