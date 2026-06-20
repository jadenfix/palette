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
    assert!(compose.contains("dashboard-e2e:"));
    assert!(compose.contains("target: e2e"));
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
    assert!(dashboard_dockerfile.contains("FROM mcr.microsoft.com/playwright:v1.57.0-noble AS e2e"));
    assert!(dashboard_dockerfile.contains("CMD [\"npx\", \"playwright\", \"test\"]"));
    assert!(dashboard_dockerfile.contains("server.js"));
    let dashboard_dockerignore = read(root.join("web/dashboard/.dockerignore"));
    assert!(dashboard_dockerignore.contains("node_modules"));
    assert!(dashboard_dockerignore.contains(".next"));
    assert!(dashboard_dockerignore.contains("test-results"));
    assert!(dashboard_dockerignore.contains("playwright-report"));

    let prebuilt_compose = read(root.join("docker-compose.prebuilt.yml"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/beaterd:main"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/dashboard:main"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/dashboard-e2e:main"));
    assert!(prebuilt_compose.contains("dashboard-e2e:"));
    assert!(prebuilt_compose.contains("profiles: [\"proof\"]"));
    assert!(prebuilt_compose.contains("PLAYWRIGHT_BASE_URL: http://dashboard:3000"));
    assert!(prebuilt_compose.contains("./docs/demos:/workspace/docs/demos"));
    assert!(!prebuilt_compose.contains("build:"));
    assert!(!prebuilt_compose.contains("BEATER_POSTGRES_PORT"));
    assert!(!prebuilt_compose.contains("BEATER_NATS_PORT"));
    assert!(!prebuilt_compose.contains("BEATER_MINIO_PORT"));

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
    assert!(image_workflow.contains("Build and push dashboard e2e runner"));
    assert!(image_workflow.contains("target: e2e"));
    assert!(image_workflow.contains("ghcr.io/${{ github.repository }}/dashboard-e2e:main"));
    assert!(image_workflow.contains("Publish dashboard e2e manifest"));

    let gate2_workflow = read(root.join(".github/workflows/gate2-proof-contract.yml"));
    assert!(gate2_workflow.contains("pull_request:"));
    assert!(gate2_workflow.contains("permissions:"));
    assert!(gate2_workflow.contains("contents: read"));
    assert!(gate2_workflow.contains("cargo fmt --all -- --check"));
    assert!(gate2_workflow.contains("bash -n scripts/validate-gate2-outside-proof.sh"));
    assert!(
        gate2_workflow.contains("python3 -m py_compile scripts/generate-gate2-outside-proof.py")
    );
    assert!(gate2_workflow.contains("scripts/validate-gate2-outside-proof.sh --allow-pending"));
    assert!(gate2_workflow.contains("cargo test -p beaterd --test self_host_contract"));
    assert!(gate2_workflow.contains("cargo test -p beaterd --test gate2_outside_validator"));
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
    assert!(!stopwatch_script.contains("duration_seconds > 300"));
    assert!(stopwatch_script.contains("time_to_first_trace_seconds > 300"));
    assert!(stopwatch_script.contains("time_to_quickstart_click_seconds > 300"));
    assert!(stopwatch_script.contains("git rev-parse HEAD"));
    assert!(stopwatch_script.contains("docker compose version"));
    assert!(stopwatch_script.contains("compose images"));
    assert!(stopwatch_script
        .contains("run_before_deadline \"Gate 2 prerequisite preflight\" preflight_prerequisites"));
    assert!(
        stopwatch_script.contains("run_before_deadline \"Gate 2 port preflight\" preflight_ports")
    );
    assert!(stopwatch_script.contains("require_command docker"));
    assert!(stopwatch_script.contains("require_command curl"));
    assert!(stopwatch_script.contains("require_command python3"));
    assert!(!stopwatch_script.contains("require_command npm"));
    assert!(stopwatch_script.contains("docker info"));
    assert!(stopwatch_script.contains("require_free_port \"$host_http_port\""));
    assert!(stopwatch_script.contains("require_free_port \"$host_otlp_grpc_port\""));
    assert!(stopwatch_script.contains("require_free_port \"$host_dashboard_port\""));
    let prerequisite_preflight = stopwatch_script
        .find("run_before_deadline \"Gate 2 prerequisite preflight\" preflight_prerequisites")
        .expect("stopwatch script should run prerequisite preflight");
    let clean_start = stopwatch_script
        .find("run_before_deadline \"clean previous Gate 2 state\" clean_start")
        .expect("stopwatch script should clean previous state");
    let port_preflight = stopwatch_script
        .find("run_before_deadline \"Gate 2 port preflight\" preflight_ports")
        .expect("stopwatch script should run port preflight");
    let compose_startup = stopwatch_script
        .find("run_before_deadline \"compose startup ($startup_mode)\"")
        .expect("stopwatch script should start compose");
    assert!(prerequisite_preflight < clean_start);
    assert!(clean_start < port_preflight);
    assert!(port_preflight < compose_startup);
    assert!(stopwatch_script.contains("BEATER_GATE2_WRITE_PROOF"));
    assert!(stopwatch_script.contains("BEATER_GATE2_BROWSER_PROOF"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RECORD_DEMO"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RECORD_MODE=compose"));
    assert!(stopwatch_script.contains("BEATER_E2E_QUICKSTART_TRACE_ID"));
    assert!(stopwatch_script.contains("compose_run_e2e"));
    assert!(stopwatch_script.contains("run_args+=(--build)"));
    assert!(stopwatch_script.contains("run_args+=(--pull \"$prebuilt_pull_policy\")"));
    assert!(stopwatch_script.contains("dashboard-e2e"));
    assert!(stopwatch_script.contains("e2e_base_url=\"http://dashboard:3000\""));
    assert!(stopwatch_script.contains("BEATER_GATE2_PUBLIC_DASHBOARD_BASE"));
    assert!(stopwatch_script.contains("gate2-compose-browser-demo.webm"));
    assert!(stopwatch_script.contains("Browser recording SHA256"));
    assert!(stopwatch_script.contains("sha256_file"));
    assert!(stopwatch_script.contains("npx playwright test tests/e2e/quickstart.spec.ts"));
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
    assert!(stopwatch_script.contains("BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS"));
    assert!(stopwatch_script.contains("BEATER_GATE2_LOCAL_BUILD"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/beaterd:$git_sha"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/dashboard:$git_sha"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/dashboard-e2e:$git_sha"));
    assert!(stopwatch_script.contains("python3 -m venv \"$preflight_venv\""));
    assert!(stopwatch_script.contains("pip --version"));
    assert!(stopwatch_script.contains("run_with_step_timeout"));
    assert!(stopwatch_script.contains("service_image_digest"));
    assert!(stopwatch_script.contains("docker image inspect"));
    assert!(stopwatch_script.contains("Beater image reference"));
    assert!(stopwatch_script.contains("Dashboard image reference"));
    assert!(stopwatch_script.contains("Dashboard e2e image reference"));
    assert!(stopwatch_script.contains("Beater image digest"));
    assert!(stopwatch_script.contains("Dashboard image digest"));
    assert!(stopwatch_script.contains("Dashboard e2e image digest"));
    assert!(stopwatch_script.contains("API endpoint"));
    assert!(stopwatch_script.contains("Dashboard base"));

    let outside_validator = read(root.join("scripts/validate-gate2-outside-proof.sh"));
    assert!(outside_validator.contains("--allow-pending"));
    assert!(outside_validator.contains("Status must be 'completed.'"));
    assert!(outside_validator.contains(":[ \\t]*(.*)$"));
    assert!(outside_validator.contains("BEATER_GATE2_REUSE=1"));
    assert!(outside_validator.contains("BEATER_DASHBOARD_PORT="));
    assert!(outside_validator.contains("DEFAULT_API_ENDPOINT = \"http://127.0.0.1:8080\""));
    assert!(outside_validator.contains("DEFAULT_DASHBOARD_BASE = \"http://127.0.0.1:3000\""));
    assert!(outside_validator.contains("all pass-checklist boxes must be checked"));
    assert!(outside_validator.contains("hashlib.sha256"));
    assert!(outside_validator.contains("subprocess.check_output"));
    assert!(outside_validator.contains("parse_qs"));
    assert!(outside_validator.contains("screen recording sha mismatch"));
    assert!(outside_validator.contains("require_image_digest"));
    assert!(outside_validator.contains("require_ghcr_image_digest"));
    assert!(outside_validator.contains("repo_artifact_path"));
    assert!(outside_validator.contains("must be a repo-relative path under docs/demos"));
    assert!(outside_validator.contains("must live under docs/demos"));
    assert!(outside_validator.contains("stopwatch proof file does not exist"));
    assert!(outside_validator.contains("DEFAULT_OTLP_ENDPOINT = \"http://127.0.0.1:4317\""));
    assert!(outside_validator.contains("API endpoint must be"));
    assert!(outside_validator.contains("(\"Startup mode\", \"prebuilt-image\")"));
    assert!(outside_validator.contains("(\"Prebuilt pull policy\", \"always\")"));
    assert!(outside_validator
        .contains("Commit SHA must match current HEAD or be an evidence-only ancestor"));
    assert!(outside_validator.contains("docs/demos evidence changes"));
    assert!(outside_validator.contains("Branch must be main"));
    assert!(outside_validator.contains("runner relationship/prior exposure"));
    assert!(outside_validator.contains("forbid_alternate_evidence(stopwatch_text"));
    assert!(outside_validator.contains("forbid_alternate_evidence(notes_text"));
    assert!(outside_validator.contains("require_equal(\"quickstart trace id\""));
    assert!(outside_validator.contains("require_equal(\"quickstart dashboard URL\""));
    assert!(outside_validator.contains("require_equal(\"screen recording sha256\""));
    assert!(outside_validator.contains("require_equal(\"screen recording notes sha256\""));
    assert!(outside_validator.contains("\"beater image digest\""));
    assert!(outside_validator.contains("\"dashboard image digest\""));
    assert!(outside_validator.contains("\"dashboard e2e image digest\""));
    assert!(outside_validator.contains("require_ghcr_sha_image_ref"));
    assert!(outside_validator.contains("\"Beater image reference\""));
    assert!(outside_validator.contains("\"Dashboard image reference\""));
    assert!(outside_validator.contains("\"Dashboard e2e image reference\""));
    assert!(outside_validator.contains("require_equal(\"commit SHA\""));
    assert!(outside_validator.contains("tenant"));
    assert!(outside_validator.contains("screen recording notes dashboard base"));
    assert!(outside_validator.contains("must be the concrete dashboard URL"));
    assert!(outside_validator.contains("OUTSIDE_RUN_ATTESTATION"));
    assert!(outside_validator.contains("\"Outside-run attestation\""));
    assert!(outside_validator
        .contains("Outside-run attestation must match the required unaided outside-run statement"));

    let outside_generator = read(root.join("scripts/generate-gate2-outside-proof.py"));
    assert!(outside_generator.contains("CANONICAL_COMMAND"));
    assert!(outside_generator.contains("OUTSIDE_RUN_ATTESTATION"));
    assert!(outside_generator.contains("--runner-name"));
    assert!(outside_generator.contains("--prior-exposure"));
    assert!(outside_generator.contains("--attest-outside-run"));
    assert!(outside_generator
        .contains("--attest-outside-run is required for completed Gate 2 proof generation"));
    assert!(outside_generator.contains("valid only when the named runner is outside"));
    assert!(outside_generator.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(outside_generator.contains("require_pending_or_force"));
    assert!(outside_generator.contains("API endpoint"));
    assert!(outside_generator.contains("Dashboard base"));
    assert!(outside_generator.contains("Beater image reference"));
    assert!(outside_generator.contains("Dashboard image reference"));
    assert!(outside_generator.contains("Dashboard e2e image reference"));
    assert!(outside_generator.contains("Browser recording SHA256"));
    assert!(outside_generator.contains("Beater image digest"));
    assert!(outside_generator.contains("Dashboard image digest"));
    assert!(outside_generator.contains("Dashboard e2e image digest"));

    let outside_proof = read(root.join("docs/demos/gate2-outside-person-proof.md"));
    assert!(outside_proof.contains("Status: not yet completed"));
    assert!(outside_proof.contains("BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1"));
    assert!(outside_proof.contains("Preflight status"));
    assert!(outside_proof.contains("Docker was running before the stopwatch started"));
    assert!(outside_proof.contains("Python and curl were available"));
    assert!(outside_proof.contains("prebuilt `dashboard-e2e` container"));
    assert!(outside_proof.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(outside_proof.contains("scripts/generate-gate2-outside-proof.py"));
    assert!(outside_proof.contains("--attest-outside-run"));
    assert!(outside_proof.contains("Docker Compose version"));
    assert!(outside_proof.contains("Beater image reference"));
    assert!(outside_proof.contains("Dashboard image reference"));
    assert!(outside_proof.contains("Dashboard e2e image reference"));
    assert!(outside_proof.contains("Beater image digest"));
    assert!(outside_proof.contains("Dashboard e2e image digest"));
    assert!(outside_proof.contains("API endpoint"));
    assert!(outside_proof.contains("Dashboard base"));
    assert!(outside_proof.contains("Screen recording SHA256"));
    assert!(outside_proof.contains("gate2-compose-browser-demo.webm"));
    assert!(outside_proof.contains("gate2-compose-browser-demo.md"));
    assert!(outside_proof.contains("Terminal output excerpt"));
    assert!(outside_proof.contains("http://127.0.0.1:3000/"));
    assert!(outside_proof.contains("Time-to-first-trace was 300 seconds or less"));
    assert!(outside_proof.contains("Time-to-quickstart-click was 300 seconds or less"));
    assert!(outside_proof.contains("run -> turn -> step -> tool -> MCP"));
    assert!(outside_proof.contains("using only public repository instructions"));
    assert!(outside_proof.contains("Outside-run attestation"));
    assert!(outside_proof.contains("Prior Beater repo exposure"));
    assert!(outside_proof.contains("default API/OTLP/dashboard endpoints"));
    assert!(outside_proof.contains("cross-checks default"));
    assert!(outside_proof.contains("image digests"));
    assert!(outside_proof.contains("screen-recording notes"));
    assert!(outside_proof.contains("SHA256 against the committed artifact"));

    let readme = read(root.join("README.md"));
    assert!(readme.contains("docs/demos/gate2-outside-person-proof.md"));
    assert!(readme.contains("scripts/generate-gate2-outside-proof.py"));
    assert!(readme.contains("--attest-outside-run"));
    assert!(readme.contains("BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh"));
    assert!(readme.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(readme.contains("removes any previous Beater stopwatch project"));
    assert!(readme.contains("gate2-compose-browser-demo.webm"));
    assert!(readme.contains("prebuilt `dashboard-e2e` Playwright browser proof"));
    assert!(readme.contains("pins `beaterd`, `dashboard`, and `dashboard-e2e`"));
    assert!(readme.contains("mismatched SHA-pinned image references"));
    assert!(readme.contains("time-to-quickstart-click"));
    assert!(readme.contains("checks Docker, Python, and curl"));
    assert!(readme.contains("mismatched trace IDs"));
    assert!(readme.contains("mismatched API/dashboard endpoints"));
    assert!(readme.contains("repo-relative `docs/demos/` artifacts"));
    assert!(readme.contains("prebuilt GHCR image digests"));
    assert!(readme.contains("mismatched image digests"));
    assert!(readme.contains("recording notes from a different dashboard session"));
    assert!(readme.contains("hash that does not match the committed file"));
    assert!(readme.contains("gate2-proof-contract"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert!(requirements.contains("docs/demos/gate2-outside-person-proof.md"));
    assert!(requirements.contains("scripts/generate-gate2-outside-proof.py"));
    assert!(requirements.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(requirements.contains("image-digest"));
    assert!(requirements.contains("SHA-pinned prebuilt GHCR image references"));
    assert!(requirements.contains("dashboard-e2e"));
    assert!(requirements.contains("recording-notes"));
    assert!(requirements.contains("outside-run attestation"));
    assert!(requirements.contains("repo-relative `docs/demos/` artifacts"));
    assert!(requirements.contains("prebuilt GHCR image digests"));
    assert!(requirements.contains("default API/OTLP/dashboard endpoints"));
    assert!(requirements.contains("recording-hash cross-checks"));
    assert!(requirements.contains("CI-enforced"));

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
    assert!(record_script.contains("BEATER_GATE2_PUBLIC_DASHBOARD_BASE"));
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
