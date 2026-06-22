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
    for service in ["postgres", "nats", "minio"] {
        assert!(
            compose_service_block(&compose, service).contains("profiles: [\"deps\"]"),
            "docker-compose.yml {service} must be opt-in until beaterd uses it at runtime"
        );
    }
    assert!(
        !compose_service_block(&compose, "beaterd").contains("depends_on:"),
        "beaterd must not depend on unused external services in the default local compose path"
    );
    assert!(compose.contains("dashboard-e2e:"));
    assert!(compose.contains("target: e2e"));
    assert!(compose.contains("profiles: [\"clickhouse\"]"));
    assert!(compose.contains("http://beaterd:8080"));
    assert!(compose.contains("http://beaterd:4317"));
    assert!(compose.contains("${BEATER_HTTP_PORT:-8080}:8080"));
    assert!(compose.contains("${BEATER_DASHBOARD_PORT:-3000}:3000"));
    assert!(compose.contains("target: runtime"));
    assert!(compose.contains("target: tools"));
    assert_pinned_image(
        &compose,
        "docker-compose.yml postgres",
        "postgres:17-alpine",
        "sha256:dc17045ccfd343b49600570ea734b9c4991cf1c3f3302e67df51e3b402dd55c4",
    );
    assert_pinned_image(
        &compose,
        "docker-compose.yml nats",
        "nats:2.11-alpine",
        "sha256:e4bf19f15fd3218814a4e3c9e0064e1334bd8aa20d5984b9f1a0afd084f8cc00",
    );
    assert_pinned_image(
        &compose,
        "docker-compose.yml minio",
        "minio/minio:latest",
        "sha256:14cea493d9a34af32f524e538b8346cf79f3321eff8e708c1e2960462bd8936e",
    );
    assert_pinned_image(
        &compose,
        "docker-compose.yml clickhouse",
        "clickhouse/clickhouse-server:latest",
        "sha256:07afc18d8a9706eb9d85c5c5d2752e5270f91bbc2894caeaecb73e4d0f603bf5",
    );

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
    let root_dockerignore = read(root.join(".dockerignore"));
    for ignored in [
        "web",
        "examples",
        "docs",
        "scripts",
        ".github",
        ".vercel",
        "README.md",
        "REQUIREMENTS.md",
        "docker-compose.*.yml",
    ] {
        assert!(
            dockerignore_ignores(&root_dockerignore, ignored),
            "root Rust image context should ignore {ignored}"
        );
    }

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
    let otel_python_dockerfile = read(root.join("examples/python/Dockerfile"));
    assert!(otel_python_dockerfile.contains("FROM python:3.12-slim"));
    assert!(otel_python_dockerfile.contains("opentelemetry-sdk"));
    assert!(otel_python_dockerfile.contains("opentelemetry-exporter-otlp-proto-grpc"));
    assert!(otel_python_dockerfile.contains("COPY five_line_otel.py otel_smoke.py"));

    let prebuilt_compose = read(root.join("docker-compose.prebuilt.yml"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/beaterd:main"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/dashboard:main"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/dashboard-e2e:main"));
    assert!(prebuilt_compose.contains("ghcr.io/jadenfix/beater/otel-python:main"));
    assert!(prebuilt_compose.contains("dashboard-e2e:"));
    assert!(prebuilt_compose.contains("otel-python-quickstart:"));
    assert!(prebuilt_compose.contains("otel-python-smoke:"));
    assert!(prebuilt_compose.contains("profiles: [\"proof\"]"));
    for service in ["postgres", "nats", "minio"] {
        assert!(
            compose_service_block(&prebuilt_compose, service).contains("profiles: [\"deps\"]"),
            "docker-compose.prebuilt.yml {service} must be opt-in until beaterd uses it at runtime"
        );
    }
    assert!(
        !compose_service_block(&prebuilt_compose, "beaterd").contains("depends_on:"),
        "beaterd must not depend on unused external services in the default prebuilt compose path"
    );
    assert!(prebuilt_compose.contains("PLAYWRIGHT_BASE_URL: http://dashboard:3000"));
    assert!(prebuilt_compose.contains("./docs/demos:/workspace/docs/demos"));
    assert!(!prebuilt_compose.contains("build:"));
    assert!(!prebuilt_compose.contains("BEATER_POSTGRES_PORT"));
    assert!(!prebuilt_compose.contains("BEATER_NATS_PORT"));
    assert!(!prebuilt_compose.contains("BEATER_MINIO_PORT"));
    assert_pinned_image(
        &prebuilt_compose,
        "docker-compose.prebuilt.yml postgres",
        "postgres:17-alpine",
        "sha256:dc17045ccfd343b49600570ea734b9c4991cf1c3f3302e67df51e3b402dd55c4",
    );
    assert_pinned_image(
        &prebuilt_compose,
        "docker-compose.prebuilt.yml nats",
        "nats:2.11-alpine",
        "sha256:e4bf19f15fd3218814a4e3c9e0064e1334bd8aa20d5984b9f1a0afd084f8cc00",
    );
    assert_pinned_image(
        &prebuilt_compose,
        "docker-compose.prebuilt.yml minio",
        "minio/minio:latest",
        "sha256:14cea493d9a34af32f524e538b8346cf79f3321eff8e708c1e2960462bd8936e",
    );

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
    assert!(image_workflow.contains("Build and push OTEL Python runner"));
    assert!(image_workflow.contains("context: ./examples/python"));
    assert!(image_workflow.contains("ghcr.io/${{ github.repository }}/otel-python:main"));
    assert!(image_workflow.contains("Publish OTEL Python runner manifest"));

    let gate2_workflow = read(root.join(".github/workflows/gate2-proof-contract.yml"));
    assert!(gate2_workflow.contains("pull_request:"));
    assert!(gate2_workflow.contains("permissions:"));
    assert!(gate2_workflow.contains("contents: read"));
    assert!(gate2_workflow.contains("cargo fmt --all -- --check"));
    assert!(gate2_workflow.contains("bash -n \"$script\""));
    for script in [
        "scripts/check-openapi-drift.sh",
        "scripts/gate2-compose-stopwatch.sh",
        "scripts/gate2-outside-local-preflight.sh",
        "scripts/gate2-outside-run.sh",
        "scripts/gate2-proof.sh",
        "scripts/smoke-compose.sh",
        "scripts/validate-gate2-outside-proof.sh",
    ] {
        assert!(
            gate2_workflow.contains(script),
            "gate2-proof-contract must syntax-check {script}"
        );
    }
    assert!(
        gate2_workflow.contains("python3 -m py_compile scripts/check-gate2-outside-readiness.py")
    );
    assert!(gate2_workflow.contains("python3 -m py_compile scripts/check-gate2-public-handoff.py"));
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
    assert!(compose_script.contains("compose up -d --build beaterd dashboard"));
    assert!(!compose_script.contains("compose up -d --build postgres nats minio"));
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
    assert!(stopwatch_script.contains("examples/python/five_line_otel.py"));
    assert!(stopwatch_script.contains("otel-python-quickstart"));
    assert!(stopwatch_script.contains("otel-python-smoke"));
    assert!(stopwatch_script.contains("compose_run_tool"));
    assert!(!stopwatch_script.contains("duration_seconds > 300"));
    assert!(stopwatch_script.contains("time_to_first_trace_seconds > 300"));
    assert!(stopwatch_script.contains("time_to_quickstart_click_seconds > 300"));
    assert!(stopwatch_script.contains("confirm_manual_quickstart_click"));
    assert!(stopwatch_script.contains("manual-outside-runner"));
    assert!(stopwatch_script.contains("Manual quickstart confirmation"));
    assert!(stopwatch_script.contains("gate2_run_id"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RUN_ID"));
    assert!(stopwatch_script.contains("release=$gate2_run_id"));
    assert!(stopwatch_script.contains("quickstart_list_url="));
    assert!(stopwatch_script.contains("kind=llm.call&model=gpt-quickstart&release=$gate2_run_id"));
    assert!(stopwatch_script.contains("Direct quickstart trace URL:"));
    assert!(stopwatch_script
        .contains("open $quickstart_list_url in a normal browser for the quickstart trace list"));
    assert!(stopwatch_script.contains("Quickstart release ID"));
    assert!(stopwatch_script.contains("git rev-parse HEAD"));
    assert!(stopwatch_script.contains("git branch --show-current"));
    assert!(stopwatch_script.contains("git remote get-url origin"));
    assert!(stopwatch_script.contains("git status --porcelain"));
    assert!(stopwatch_script.contains("Git branch"));
    assert!(stopwatch_script.contains("Git origin"));
    assert!(stopwatch_script.contains("Git worktree clean"));
    assert!(stopwatch_script.contains("docker compose version"));
    assert!(stopwatch_script.contains("compose images"));
    assert!(stopwatch_script.contains("proof-image beaterd"));
    assert!(stopwatch_script.contains("proof-image dashboard-e2e"));
    assert!(stopwatch_script.contains("startup_args=(up -d --build beaterd dashboard)"));
    assert!(stopwatch_script
        .contains("startup_args=(up -d --pull \"$prebuilt_pull_policy\" beaterd dashboard)"));
    assert!(!stopwatch_script.contains("startup_args=(up -d --build postgres nats minio"));
    assert!(!stopwatch_script
        .contains("startup_args=(up -d --pull \"$prebuilt_pull_policy\" postgres nats minio"));
    assert!(stopwatch_script
        .contains("run_before_deadline \"Gate 2 prerequisite preflight\" preflight_prerequisites"));
    assert!(
        stopwatch_script.contains("run_before_deadline \"Gate 2 port preflight\" preflight_ports")
    );
    assert!(stopwatch_script.contains("require_command docker"));
    assert!(stopwatch_script.contains("require_command curl"));
    assert!(stopwatch_script.contains("shasum or sha256sum"));
    assert!(stopwatch_script.contains("DOCKER_HOST"));
    assert!(stopwatch_script.contains("docker_endpoint_is_local"));
    assert!(stopwatch_script.contains("tcp://localhost:"));
    assert!(stopwatch_script.contains("tcp://127."));
    assert!(stopwatch_script.contains("tcp://[::1]:"));
    assert!(stopwatch_script.contains("docker context inspect"));
    assert!(stopwatch_script.contains("requires a local Docker context"));
    assert!(!stopwatch_script.contains("require_command python3"));
    assert!(!stopwatch_script.contains("python3 -m venv"));
    assert!(!stopwatch_script.contains("pip --version"));
    assert!(!stopwatch_script.contains("require_command npm"));
    assert!(stopwatch_script.contains("docker info"));
    assert!(stopwatch_script.contains("require_free_port \"$host_http_port\""));
    assert!(stopwatch_script.contains("require_free_port \"$host_otlp_grpc_port\""));
    assert!(stopwatch_script.contains("require_free_port \"$host_dashboard_port\""));
    assert!(stopwatch_script.contains("If another app is listed below, stop that app"));
    assert!(stopwatch_script.contains("print_port_owner_details"));
    assert!(stopwatch_script.contains("process $pid command:"));
    assert!(stopwatch_script.contains("process $pid cwd:"));
    let prerequisite_preflight = find_required(
        &stopwatch_script,
        "run_before_deadline \"Gate 2 prerequisite preflight\" preflight_prerequisites",
    );
    let clean_start = find_required(
        &stopwatch_script,
        "run_before_deadline \"clean previous Gate 2 state\" clean_start",
    );
    let port_preflight = find_required(
        &stopwatch_script,
        "run_before_deadline \"Gate 2 port preflight\" preflight_ports",
    );
    let compose_startup = find_required(
        &stopwatch_script,
        "run_before_deadline \"compose startup ($startup_mode)\"",
    );
    assert!(prerequisite_preflight < clean_start);
    assert!(clean_start < port_preflight);
    assert!(port_preflight < compose_startup);
    assert!(stopwatch_script.contains("BEATER_GATE2_WRITE_PROOF"));
    assert!(stopwatch_script.contains("BEATER_GATE2_BROWSER_PROOF"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RECORD_DEMO"));
    assert!(stopwatch_script.contains("BEATER_GATE2_RECORD_MODE=compose"));
    assert!(stopwatch_script.contains("BEATER_GATE2_OUTSIDE_WRAPPER=\"$outside_wrapper\""));
    assert!(stopwatch_script.contains("BEATER_E2E_QUICKSTART_TRACE_ID"));
    assert!(stopwatch_script.contains("BEATER_E2E_QUICKSTART_RELEASE"));
    assert!(stopwatch_script.contains("compose_run_e2e"));
    assert!(stopwatch_script.contains("run_args+=(--build)"));
    assert!(stopwatch_script.contains("run_args+=(--pull \"$prebuilt_pull_policy\")"));
    assert!(stopwatch_script.contains("dashboard-e2e"));
    assert!(stopwatch_script.contains("e2e_base_url=\"http://dashboard:3000\""));
    assert!(stopwatch_script.contains("BEATER_GATE2_PUBLIC_DASHBOARD_BASE"));
    assert!(stopwatch_script.contains("gate2-compose-browser-demo.webm"));
    assert!(stopwatch_script.contains("Browser recording SHA256"));
    assert!(stopwatch_script.contains("sha256_file"));
    assert!(stopwatch_script.contains("min_recording_seconds=\"8.0\""));
    assert!(stopwatch_script.contains("recording_probe()"));
    assert!(stopwatch_script.contains("recording_duration_seconds()"));
    assert!(stopwatch_script.contains("codec_type=video"));
    assert!(stopwatch_script.contains("require_reviewable_recording()"));
    let reviewable_recording_check = find_required(
        &stopwatch_script,
        "require_reviewable_recording \"$record_demo_video\"",
    );
    let recording_pass_marker = stopwatch_script
        .rfind("record_demo_status=\"passed\"")
        .expect("expected recording pass marker");
    assert!(
        reviewable_recording_check < recording_pass_marker,
        "recording duration must be validated before marking browser recording passed"
    );
    assert!(stopwatch_script.contains("proof_followup_block"));
    assert!(stopwatch_script.contains("outside-run stopwatch source artifact"));
    assert!(stopwatch_script.contains("This is an automated local stopwatch proof"));
    assert!(stopwatch_script.contains("npx playwright test tests/e2e/quickstart.spec.ts"));
    assert!(stopwatch_script.contains("npx playwright test tests/e2e/dashboard.spec.ts"));
    assert!(stopwatch_script.contains("all_kind_trace_id"));
    assert!(stopwatch_script.contains("all-kind nested agent waterfall"));
    assert!(stopwatch_script.contains("PLAYWRIGHT_BASE_URL"));
    assert!(stopwatch_script.contains("BEATER_GATE2_REUSE"));
    assert!(stopwatch_script.contains("clean_start"));
    let clean_start_body = &stopwatch_script[find_required(&stopwatch_script, "clean_start() {")
        ..find_required(&stopwatch_script, "terminate_tree() {")];
    assert!(clean_start_body.contains("compose down -v --remove-orphans >/dev/null"));
    assert!(!clean_start_body.contains("|| true"));
    assert!(!stopwatch_script.contains("venv_dir"));
    assert!(stopwatch_script.contains("docker-compose.prebuilt.yml"));
    assert!(stopwatch_script.contains("BEATER_GATE2_PULL_POLICY"));
    assert!(stopwatch_script.contains("--pull \"$prebuilt_pull_policy\""));
    assert!(stopwatch_script.contains("BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS"));
    assert!(stopwatch_script.contains("BEATER_GATE2_LOCAL_BUILD"));
    assert!(stopwatch_script.contains("BEATER_GATE2_OUTSIDE_WRAPPER"));
    assert!(stopwatch_script.contains("Outside-run wrapper"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/beaterd:$git_sha"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/dashboard:$git_sha"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/dashboard-e2e:$git_sha"));
    assert!(stopwatch_script.contains("ghcr.io/jadenfix/beater/otel-python:$git_sha"));
    assert!(stopwatch_script.contains("run_with_step_timeout"));
    assert!(stopwatch_script.contains("service_image_digest"));
    assert!(stopwatch_script.contains("docker image inspect"));
    assert!(stopwatch_script.contains("expected_repo"));
    assert!(stopwatch_script.contains("awk -v prefix=\"${expected_repo}@sha256:\""));
    assert!(stopwatch_script.contains("Beater image reference"));
    assert!(stopwatch_script.contains("Dashboard image reference"));
    assert!(stopwatch_script.contains("Dashboard e2e image reference"));
    assert!(stopwatch_script.contains("OTEL Python image reference"));
    assert!(stopwatch_script.contains("Beater image digest"));
    assert!(stopwatch_script.contains("Dashboard image digest"));
    assert!(stopwatch_script.contains("Dashboard e2e image digest"));
    assert!(stopwatch_script.contains("OTEL Python image digest"));
    assert!(stopwatch_script.contains("API endpoint"));
    assert!(stopwatch_script.contains("Dashboard base"));

    let outside_run = read(root.join("scripts/gate2-outside-run.sh"));
    assert!(outside_run.contains("require_git_provenance"));
    assert!(outside_run.contains("https://github.com/jadenfix/beater.git"));
    assert!(outside_run.contains("outside-person evidence must run from the main branch"));
    assert!(outside_run.contains("outside-person evidence must run from origin"));
    assert!(outside_run.contains("outside-person evidence must run from a clean worktree"));
    assert!(outside_run.contains("BEATER_GATE2_EXPECTED_ORIGIN"));
    assert!(outside_run.contains("BEATER_GATE2_WRITE_PROOF=1"));
    assert!(outside_run.contains("BEATER_GATE2_BROWSER_PROOF=1"));
    assert!(outside_run.contains("BEATER_GATE2_RECORD_DEMO=1"));
    assert!(outside_run.contains("BEATER_GATE2_REUSE"));
    assert!(outside_run.contains("BEATER_GATE2_LOCAL_BUILD"));
    assert!(outside_run.contains("BEATER_DASHBOARD_PORT"));
    assert!(outside_run.contains("BEATER_GATE2_OUTSIDE_WRAPPER=1"));
    assert!(outside_run.contains("require_unset BEATERD_IMAGE"));
    assert!(outside_run.contains("require_unset BEATER_DASHBOARD_IMAGE"));
    assert!(outside_run.contains("require_unset BEATER_DASHBOARD_E2E_IMAGE"));
    assert!(outside_run.contains("require_unset BEATER_OTEL_PYTHON_IMAGE"));
    assert!(outside_run.contains("require_unset BEATER_GATE2_RUN_ID"));
    assert!(outside_run.contains("fresh per-run quickstart release ID"));
    assert!(outside_run.contains("BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS"));
    assert!(outside_run.contains("outside evidence must validate against public GHCR"));
    assert!(outside_run.contains("the wrapper pins beaterd to the checked-out commit SHA"));
    assert!(outside_run.contains("require_unset BEATER_GATE2_STOPWATCH_PROOF"));
    assert!(outside_run.contains("require_unset BEATER_GATE2_RECORD_VIDEO"));
    assert!(outside_run.contains("require_unset BEATER_GATE2_RECORD_NOTES"));
    assert!(outside_run.contains("docs/demos/gate2-compose-stopwatch.md"));
    assert!(outside_run.contains("docs/demos/gate2-compose-browser-demo.webm"));
    assert!(outside_run.contains("docs/demos/gate2-compose-browser-demo.md"));
    assert!(outside_run.contains("require_unset_or_value KEEP_BEATER_COMPOSE 1"));
    assert!(outside_run.contains("require_unset COMPOSE_PROJECT_NAME"));
    assert!(outside_run.contains("default beater-stopwatch Compose project"));
    assert!(outside_run.contains("export KEEP_BEATER_COMPOSE=1"));
    assert!(outside_run.contains("scripts/gate2-compose-stopwatch.sh"));
    assert!(outside_run.contains("Gate 2 outside-run wrapper preflight passed"));

    let outside_local_preflight = read(root.join("scripts/gate2-outside-local-preflight.sh"));
    assert!(outside_local_preflight.contains("require_command git"));
    assert!(outside_local_preflight.contains("require_command docker"));
    assert!(outside_local_preflight.contains("require_command curl"));
    assert!(outside_local_preflight.contains("require_command ffprobe"));
    assert!(outside_local_preflight.contains("require_python3"));
    assert!(outside_local_preflight.contains("version 3.9 or newer"));
    assert!(outside_local_preflight.contains("shasum"));
    assert!(outside_local_preflight.contains("sha256sum"));
    assert!(outside_local_preflight.contains("DOCKER_HOST"));
    assert!(outside_local_preflight.contains("docker_endpoint_is_local"));
    assert!(outside_local_preflight.contains("docker context inspect"));
    assert!(outside_local_preflight.contains("Docker Compose v2"));
    assert!(outside_local_preflight.contains("browser proof uses 127.0.0.1"));
    assert!(outside_local_preflight.contains("free TCP $port before starting the stopwatch"));
    assert!(outside_local_preflight.contains("print_port_owner_details"));
    assert!(outside_local_preflight.contains("process $pid command:"));
    assert!(outside_local_preflight.contains("process $pid cwd:"));
    for port in ["8080", "4317", "3000"] {
        assert!(
            outside_local_preflight.contains(port),
            "outside local preflight must check TCP {port}"
        );
    }

    let outside_validator = read(root.join("scripts/validate-gate2-outside-proof.sh"));
    assert!(outside_validator.contains("--allow-pending"));
    assert!(outside_validator.contains("--diagnostic"));
    assert!(outside_validator.contains("Status must be 'completed.'"));
    assert!(outside_validator.contains("diagnostic."));
    assert!(outside_validator.contains("Diagnostic validation requires Status: diagnostic."));
    assert!(outside_validator.contains("Gate 2 diagnostic proof is valid"));
    assert!(outside_validator.contains("scripts/gate2-outside-run.sh"));
    assert!(outside_validator.contains("\"Outside-run wrapper\""));
    assert!(outside_validator.contains("Outside-run wrapper must be yes"));
    assert!(outside_validator.contains("\"Quickstart click source\""));
    assert!(outside_validator.contains("Quickstart click source must be manual-outside-runner"));
    assert!(outside_validator.contains("\"Manual quickstart confirmation\""));
    assert!(outside_validator.contains("Manual quickstart confirmation must be yes"));
    assert!(outside_validator.contains(":[ \\t]*(.*)$"));
    assert!(outside_validator.contains("BEATER_GATE2_REUSE=1"));
    assert!(outside_validator.contains("BEATER_DASHBOARD_PORT="));
    assert!(outside_validator.contains("DEFAULT_API_ENDPOINT = \"http://127.0.0.1:8080\""));
    assert!(outside_validator.contains("DEFAULT_DASHBOARD_BASE = \"http://127.0.0.1:3000\""));
    assert!(outside_validator.contains("all pass-checklist boxes must be checked"));
    assert!(outside_validator.contains("hashlib.sha256"));
    assert!(outside_validator.contains("MIN_RECORDING_BYTES"));
    assert!(outside_validator.contains("MIN_RECORDING_SECONDS = 8.0"));
    assert!(outside_validator.contains("require_webm_recording"));
    assert!(outside_validator.contains("require_tracked_artifact"));
    assert!(outside_validator.contains("require_committed_clean_path"));
    assert!(outside_validator.contains("must be committed and clean before Gate 2 closure"));
    assert!(outside_validator.contains("must be tracked by git before Gate 2 closure"));
    assert!(outside_validator.contains("screen recording must start with a WebM/EBML header"));
    assert!(outside_validator.contains("screen recording must declare WebM DocType"));
    assert!(outside_validator.contains("screen recording WebM must contain a Segment element"));
    assert!(outside_validator.contains("screen recording WebM must contain an Info element"));
    assert!(outside_validator.contains("screen recording WebM must contain a video track"));
    assert!(outside_validator.contains("reviewable full-flow capture of at least"));
    assert!(outside_validator.contains("must not be a symlink"));
    assert!(outside_validator.contains("subprocess.check_output"));
    assert!(outside_validator.contains("parse_qs"));
    assert!(outside_validator.contains("screen recording sha mismatch"));
    assert!(outside_validator.contains("require_ghcr_image_digest"));
    assert!(outside_validator.contains("repo_artifact_path"));
    assert!(outside_validator.contains("must be a repo-relative path under docs/demos"));
    assert!(outside_validator.contains("must live under docs/demos"));
    assert!(outside_validator.contains("stopwatch proof file does not exist"));
    assert!(outside_validator.contains("outside-run source evidence"));
    assert!(outside_validator.contains("not automated local proof"));
    assert!(outside_validator.contains("outside-run stopwatch source artifact"));
    assert!(outside_validator.contains("DEFAULT_OTLP_ENDPOINT = \"http://127.0.0.1:4317\""));
    assert!(outside_validator.contains("API endpoint must be"));
    assert!(outside_validator.contains("(\"Startup mode\", \"prebuilt-image\")"));
    assert!(outside_validator.contains("(\"Prebuilt pull policy\", \"always\")"));
    assert!(outside_validator.contains("(\"Compose project\", \"beater-stopwatch\")"));
    assert!(outside_validator
        .contains("Commit SHA must match current HEAD or be an evidence-only ancestor"));
    assert!(outside_validator.contains("docs/demos evidence changes"));
    assert!(outside_validator.contains("Branch must be main"));
    assert!(outside_validator.contains("Clone URL must be"));
    assert!(outside_validator.contains("Worktree clean must be yes"));
    assert!(outside_validator.contains("(\"Git branch\", \"main\")"));
    assert!(outside_validator.contains("(\"Git origin\", EXPECTED_CLONE_URL)"));
    assert!(outside_validator.contains("(\"Git worktree clean\", \"yes\")"));
    assert!(outside_validator.contains("require_equal(\"branch\""));
    assert!(outside_validator.contains("require_equal(\"clone URL\""));
    assert!(outside_validator.contains("require_equal(\"worktree clean\""));
    assert!(outside_validator.contains("runner relationship/prior exposure"));
    assert!(outside_validator.contains("forbid_alternate_evidence(stopwatch_text"));
    assert!(outside_validator.contains("forbid_alternate_evidence(notes_text"));
    assert!(outside_validator.contains("require_equal(\"quickstart trace id\""));
    assert!(outside_validator.contains("require_quickstart_release_id"));
    assert!(outside_validator.contains("\"Quickstart release ID\""));
    assert!(outside_validator.contains("quickstart release ID"));
    assert!(outside_validator.contains("proof-image"));
    assert!(outside_validator.contains("service_segments"));
    assert!(
        outside_validator.contains("Gate 2 outside-person proof draft is internally consistent")
    );
    assert!(outside_validator.contains("require_equal(\"quickstart dashboard URL\""));
    assert!(outside_validator.contains("require_equal(\"screen recording sha256\""));
    assert!(outside_validator.contains("require_equal(\"screen recording notes sha256\""));
    assert!(outside_validator.contains("\"beater image digest\""));
    assert!(outside_validator.contains("\"dashboard image digest\""));
    assert!(outside_validator.contains("\"dashboard e2e image digest\""));
    assert!(outside_validator.contains("\"otel python image digest\""));
    assert!(outside_validator.contains("registry_manifest_from_ghcr"));
    assert!(outside_validator.contains("Docker-Content-Digest"));
    assert!(outside_validator.contains("must match public GHCR manifest digest"));
    assert!(outside_validator.contains("require_ghcr_sha_image_ref"));
    assert!(outside_validator.contains("\"Beater image reference\""));
    assert!(outside_validator.contains("\"Dashboard image reference\""));
    assert!(outside_validator.contains("\"Dashboard e2e image reference\""));
    assert!(outside_validator.contains("\"OTEL Python image reference\""));
    assert!(outside_validator.contains("require_equal(\"commit SHA\""));
    assert!(outside_validator.contains("tenant"));
    assert!(outside_validator.contains("screen recording notes dashboard base"));
    assert!(outside_validator.contains("Recording mode must be compose"));
    assert!(outside_validator.contains("require_recording_shows_full_flow"));
    assert!(outside_validator.contains("screen recording notes Shows must describe"));
    assert!(outside_validator.contains("must be the concrete dashboard URL"));
    assert!(outside_validator.contains("OUTSIDE_RUN_ATTESTATION"));
    assert!(outside_validator.contains("DIAGNOSTIC_ATTESTATION"));
    assert!(outside_validator.contains("\"Outside-run attestation\""));
    assert!(outside_validator
        .contains("Outside-run attestation must match the required unaided outside-run statement"));

    let outside_generator = read(root.join("scripts/generate-gate2-outside-proof.py"));
    assert!(outside_generator.contains("CANONICAL_COMMAND"));
    assert!(outside_generator.contains("scripts/gate2-outside-run.sh"));
    assert!(outside_generator.contains("Outside-run wrapper"));
    assert!(outside_generator.contains("Quickstart click source"));
    assert!(outside_generator.contains("Manual quickstart confirmation"));
    assert!(outside_generator.contains("Quickstart release ID"));
    assert!(outside_generator.contains("Git branch"));
    assert!(outside_generator.contains("Git origin"));
    assert!(outside_generator.contains("Git worktree clean"));
    assert!(outside_generator.contains("OUTSIDE_RUN_ATTESTATION"));
    assert!(outside_generator.contains("DIAGNOSTIC_ATTESTATION"));
    assert!(outside_generator.contains("--runner-name"));
    assert!(outside_generator.contains("--prior-exposure"));
    assert!(outside_generator.contains("--attest-outside-run"));
    assert!(outside_generator.contains("--diagnostic-report"));
    assert!(outside_generator.contains("diagnostic."));
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
    assert!(outside_generator.contains("OTEL Python image reference"));
    assert!(outside_generator.contains("Browser recording SHA256"));
    assert!(outside_generator.contains("Beater image digest"));
    assert!(outside_generator.contains("Dashboard image digest"));
    assert!(outside_generator.contains("Dashboard e2e image digest"));
    assert!(outside_generator.contains("OTEL Python image digest"));

    let outside_readiness = read(root.join("scripts/check-gate2-outside-readiness.py"));
    assert!(outside_readiness.contains("IMAGE_NAMES"));
    assert!(outside_readiness.contains("EXPECTED_PLATFORMS"));
    assert!(outside_readiness.contains("COMMON_PINNED_THIRD_PARTY_IMAGES"));
    assert!(outside_readiness.contains("CLICKHOUSE_PINNED_THIRD_PARTY_IMAGE"));
    assert!(outside_readiness.contains("*COMMON_PINNED_THIRD_PARTY_IMAGES"));
    assert!(outside_readiness.contains("linux/amd64"));
    assert!(outside_readiness.contains("linux/arm64"));
    assert!(outside_readiness.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(outside_readiness.contains("https://github.com/jadenfix/beater.git"));
    assert!(outside_readiness.contains("worktree must be clean"));
    assert!(outside_readiness.contains("missing public GHCR manifest"));
    assert!(outside_readiness.contains("--registry-fixture"));

    let public_handoff = read(root.join("scripts/check-gate2-public-handoff.py"));
    assert!(public_handoff.contains("https://github.com/jadenfix/beater.git"));
    assert!(public_handoff.contains("RAW_PUBLIC_PREFLIGHT_COMMAND"));
    assert!(public_handoff.contains("RAW_PREFLIGHT_URL"));
    assert!(public_handoff.contains("https://raw.githubusercontent.com/jadenfix/beater/main/"));
    assert!(public_handoff.contains("scripts/gate2-outside-local-preflight.sh"));
    assert!(public_handoff.contains("\"bash\", \"-o\", \"pipefail\", \"-lc\""));
    assert!(public_handoff.contains("run_raw_public_preflight(args)"));
    assert!(public_handoff.contains("git"));
    assert!(public_handoff.contains("clone"));
    assert!(public_handoff.contains("clone_command = [\"git\", \"clone\""));
    assert!(!public_handoff.contains("\"--depth\""));
    assert!(!public_handoff.contains("\"--branch\""));
    assert!(public_handoff.contains("compile(path.read_text(), str(path), 'exec')"));
    assert!(public_handoff.contains("GATE2_SHELL_SCRIPTS"));
    assert!(public_handoff.contains("bash\", \"-n\", script"));
    for script in [
        "scripts/check-openapi-drift.sh",
        "scripts/gate2-compose-stopwatch.sh",
        "scripts/gate2-outside-local-preflight.sh",
        "scripts/gate2-outside-run.sh",
        "scripts/gate2-proof.sh",
        "scripts/smoke-compose.sh",
        "scripts/validate-gate2-outside-proof.sh",
    ] {
        assert!(
            public_handoff.contains(script),
            "public handoff verifier must syntax-check {script}"
        );
    }
    assert!(public_handoff.contains("main"));
    assert!(public_handoff.contains("public handoff clone is not the expected commit"));
    assert!(public_handoff.contains("scripts/check-gate2-outside-readiness.py"));
    assert!(public_handoff.contains("scripts/gate2-outside-run.sh"));
    assert!(public_handoff.contains("BEATER_GATE2_OUTSIDE_RUN_DRY_RUN"));
    assert!(public_handoff.contains("BEATERD_IMAGE"));
    assert!(public_handoff.contains("BEATER_DASHBOARD_IMAGE"));
    assert!(public_handoff.contains("BEATER_DASHBOARD_E2E_IMAGE"));
    assert!(public_handoff.contains("BEATER_OTEL_PYTHON_IMAGE"));
    assert!(public_handoff.contains("BEATER_GATE2_STOPWATCH_PROOF"));
    assert!(public_handoff.contains("BEATER_GATE2_RECORD_VIDEO"));
    assert!(public_handoff.contains("BEATER_GATE2_RECORD_NOTES"));
    assert!(public_handoff.contains("BEATER_GATE2_RUN_ID"));
    assert!(public_handoff.contains("BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS"));
    assert!(public_handoff.contains("KEEP_BEATER_COMPOSE"));
    let raw_preflight_idx = public_handoff
        .find("run_raw_public_preflight(args)")
        .expect("raw public preflight call in public handoff verifier");
    let clone_idx = public_handoff
        .find("clone_dir, temp_owner, clone_started_epoch = clone_repo")
        .expect("first clone call in public handoff verifier");
    assert!(
        raw_preflight_idx < clone_idx,
        "public handoff verifier must run the raw public preflight before cloning"
    );
    assert!(public_handoff.contains("COMPOSE_PROJECT_NAME"));
    assert!(public_handoff.contains("BEATER_GATE2_EXPECTED_ORIGIN"));
    assert!(public_handoff.contains("--registry-fixture"));
    assert!(public_handoff.contains("--skip-local-readiness"));
    assert!(public_handoff.contains("import time"));
    assert!(public_handoff.contains("import selectors"));
    assert!(public_handoff.contains("import shutil"));
    assert!(public_handoff.contains("import socket"));
    assert!(public_handoff.contains("MANUAL_CHECKPOINT_MARKER"));
    assert!(public_handoff.contains("\"Manual outside-run checkpoint:\""));
    assert!(public_handoff.contains("run_with_manual_checkpoint_confirmation"));
    assert!(public_handoff.contains("diagnostic full-run did not observe"));
    assert!(public_handoff.contains("FULL_RUN_PORTS"));
    assert!(public_handoff.contains("(8080, \"beaterd HTTP\", \"BEATER_HTTP_PORT\")"));
    assert!(public_handoff.contains("(4317, \"OTLP gRPC\", \"BEATER_OTLP_GRPC_PORT\")"));
    assert!(public_handoff.contains("(3000, \"dashboard\", \"BEATER_DASHBOARD_PORT\")"));
    assert!(public_handoff.contains("preflight_full_run_runtime"));
    assert!(public_handoff.contains("require_full_run_source(args)"));
    assert!(public_handoff.contains("shutil.which"));
    assert!(public_handoff.contains("socket.create_connection"));
    assert!(public_handoff.contains("def port_resolution_hint"));
    assert!(public_handoff.contains("Stop the process or app listening on TCP"));
    assert!(public_handoff.contains("do not set"));
    assert!(public_handoff.contains("--registry-fixture"));
    assert!(public_handoff.contains("does not support"));
    assert!(public_handoff.contains("run([\"docker\", \"info\"]"));
    assert!(public_handoff.contains("run([\"docker\", \"compose\", \"version\"]"));
    assert!(public_handoff.contains("shasum or sha256sum"));
    assert!(public_handoff.contains("DOCKER_HOST"));
    assert!(public_handoff.contains("docker_endpoint_is_local"));
    assert!(public_handoff.contains("tcp://localhost:"));
    assert!(public_handoff.contains("tcp://127."));
    assert!(public_handoff.contains("tcp://[::1]:"));
    assert!(public_handoff.contains("require_local_docker_host_env"));
    assert!(public_handoff.contains("require_local_docker_context"));
    assert!(public_handoff.contains("[\"docker\", \"context\", \"inspect\""));
    assert!(public_handoff.contains("requires a local Docker daemon"));
    assert!(public_handoff.contains("requires a local Docker context"));
    assert!(public_handoff.contains("STOPWATCH_COMPOSE_DOWN"));
    assert!(public_handoff.contains("def cleanup_stopwatch_compose"));
    assert!(public_handoff.contains("cleanup_stopwatch_compose(repo_root(), fatal=True)"));
    assert!(public_handoff.contains("cleanup_stopwatch_compose(clone_dir, fatal=False)"));
    assert!(public_handoff.contains("cleanup_local_stopwatch_compose"));
    assert!(public_handoff.contains("free it rather than setting"));
    assert!(public_handoff.contains("clone_started_epoch = int(time.time())"));
    assert!(public_handoff.contains("env[\"BEATER_GATE2_CLONE_STARTED_EPOCH\"]"));
    assert!(public_handoff.contains("run_with_manual_checkpoint_confirmation("));
    assert!(public_handoff.contains("cleanup_cloned_compose"));
    assert!(public_handoff.contains("docker-compose.prebuilt.yml"));
    assert!(public_handoff.contains("Auto-confirming the manual quickstart checkpoint"));
    assert!(public_handoff.contains("diagnostic auto-confirmed the manual checkpoint"));
    assert!(public_handoff.contains("def public_clone_env"));
    assert!(public_handoff.contains("GIT_CONFIG_GLOBAL"));
    assert!(public_handoff.contains("--diagnostic-report"));
    assert!(
        public_handoff.contains("[\"scripts/validate-gate2-outside-proof.sh\", \"--diagnostic\"]")
    );
    assert!(
        public_handoff.contains("--full-run executes the exact scripts/gate2-outside-run.sh path")
    );

    let outside_proof = read(root.join("docs/demos/gate2-outside-person-proof.md"));
    assert!(outside_proof.contains("Status: not yet completed"));
    assert!(outside_proof.contains("gate2-outside-local-preflight.sh"));
    assert!(outside_proof.contains("before the stopwatch starts"));
    assert!(outside_proof.contains("scripts/gate2-outside-run.sh"));
    assert!(outside_proof.contains("sets the required proof/browser/recording flags"));
    assert!(outside_proof.contains("Outside-run wrapper:"));
    assert!(outside_proof.contains("Outside-run wrapper: yes"));
    assert!(outside_proof.contains("outside-run stopwatch source artifact"));
    assert!(outside_proof.contains("Git branch: main"));
    assert!(outside_proof.contains("Git worktree clean: yes"));
    assert!(outside_proof.contains("Worktree clean"));
    assert!(outside_proof.contains("prebuilt image"));
    assert!(outside_proof.contains("evidence artifact path"));
    assert!(outside_proof.contains("Compose project"));
    assert!(outside_proof.contains("teardown overrides"));
    assert!(outside_proof.contains("rejects"));
    assert!(outside_proof.contains("warm-loop reuse"));
    assert!(outside_proof.contains("Preflight status"));
    assert!(outside_proof.contains("Docker was running before the stopwatch started"));
    assert!(outside_proof.contains("curl was available before the stopwatch started"));
    assert!(outside_proof.contains("prebuilt"));
    assert!(outside_proof.contains("`otel-python` container"));
    assert!(outside_proof.contains("`dashboard-e2e`"));
    assert!(outside_proof.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(outside_proof.contains("scripts/generate-gate2-outside-proof.py"));
    assert!(outside_proof.contains("--attest-outside-run"));
    assert!(outside_proof.contains("Docker Compose version"));
    assert!(outside_proof.contains("scripts/check-gate2-public-handoff.py"));
    assert!(outside_proof.contains("uses one\nfresh clone"));
    assert!(outside_proof.contains("uses a second fresh clone"));
    assert!(outside_proof
        .contains("executes the second clone's\n`scripts/gate2-outside-run.sh` wrapper"));
    assert!(
        outside_proof.contains("waits until the wrapper prints the\nmanual quickstart checkpoint")
    );
    assert!(outside_proof.contains("auto-confirms that checkpoint for diagnostic\nautomation only"));
    assert!(outside_proof.contains("preflights the local runtime"));
    assert!(outside_proof.contains("local Docker daemon"));
    assert!(outside_proof.contains("SHA tooling"));
    assert!(outside_proof.contains("free those default ports"));
    assert!(outside_proof.contains("stop that app and rerun"));
    assert!(outside_proof.contains("fixture or fork URLs"));
    assert!(outside_proof.contains("scripts/check-gate2-outside-readiness.py"));
    assert!(outside_proof.contains("fresh clone from"));
    assert!(outside_proof.contains("cloned readiness"));
    assert!(outside_proof.contains("wrapper dry-run checks"));
    assert!(outside_proof.contains("immediately before that second `git clone`"));
    assert!(outside_proof.contains("public"));
    assert!(outside_proof.contains("public multi-arch GHCR images"));
    assert!(outside_proof.contains("for the exact commit"));
    assert!(outside_proof.contains("Beater image reference"));
    assert!(outside_proof.contains("Dashboard image reference"));
    assert!(outside_proof.contains("Dashboard e2e image reference"));
    assert!(outside_proof.contains("OTEL Python image reference"));
    assert!(outside_proof.contains("Beater image digest"));
    assert!(outside_proof.contains("Dashboard e2e image digest"));
    assert!(outside_proof.contains("OTEL Python image digest"));
    assert!(outside_proof.contains("API endpoint"));
    assert!(outside_proof.contains("Dashboard base"));
    assert!(outside_proof.contains("Quickstart release ID"));
    assert!(outside_proof.contains("per-run quickstart release ID"));
    assert!(outside_proof.contains("`proof-image` digest rows"));
    assert!(outside_proof.contains("Timing start source"));
    assert!(outside_proof.contains("Clone started at"));
    assert!(outside_proof.contains("Script-to-first-trace"));
    assert!(outside_proof.contains("Screen recording SHA256"));
    assert!(outside_proof.contains("gate2-compose-browser-demo.webm"));
    assert!(outside_proof.contains("gate2-compose-browser-demo.md"));
    assert!(outside_proof.contains("Terminal output excerpt"));
    assert!(outside_proof.contains("Dashboard base: `http://127.0.0.1:3000`"));
    assert!(!outside_proof.contains("http://127.0.0.1:3000/..."));
    assert!(!outside_proof.contains("none / describe"));
    assert!(!outside_proof.contains("`python3` is required after the timed run"));
    assert!(outside_proof.contains("Time-to-first-trace was 300 seconds or less"));
    assert!(outside_proof.contains("Time-to-first-trace includes clone time"));
    assert!(outside_proof
        .contains("Manual quickstart click confirmation was recorded before 300 seconds"));
    assert!(outside_proof.contains("Quickstart click source"));
    assert!(outside_proof.contains("Manual quickstart confirmation"));
    assert!(outside_proof.contains("run -> turn -> step -> tool -> MCP"));
    assert!(outside_proof.contains("using only public repository instructions"));
    assert!(outside_proof.contains("Outside-run attestation"));
    assert!(outside_proof.contains("Prior Beater repo exposure"));
    assert!(outside_proof.contains("default API/OTLP/dashboard endpoints"));
    assert!(outside_proof.contains("tested public GitHub origin"));
    assert!(outside_proof.contains("cross-checks default"));
    assert!(outside_proof.contains("image digests"));
    assert!(outside_proof.contains("screen-recording notes"));
    assert!(outside_proof.contains("`ffprobe` playable-video metadata"));
    assert!(
        outside_proof.contains("playable WebM capture of at least 64 KiB and at least 8 seconds")
    );
    assert!(outside_proof.contains("Recording mode: compose"));
    assert!(outside_proof.contains("EBML/WebM, Segment, Info,"));
    assert!(outside_proof.contains("must not resolve through symlinks"));
    for fragment in [
        "prompt",
        "completion",
        "model",
        "token breakdown",
        "cost",
        "latency",
    ] {
        assert!(outside_proof.contains(fragment));
    }
    assert!(outside_proof.contains("SHA256 against the committed artifact"));

    let readme = read(root.join("README.md"));
    assert!(readme.contains("docs/demos/gate2-outside-person-proof.md"));
    assert!(readme.contains("gate2-outside-local-preflight.sh"));
    assert!(readme.contains("before `t=\"$(date +%s)\"`"));
    assert!(readme.contains("not started in the timed default path"));
    assert!(readme.contains("available with the `deps` profile"));
    assert!(readme.contains("default compose proof starts `beaterd` and the dashboard"));
    assert!(readme.contains("scripts/gate2-outside-run.sh"));
    assert!(readme.contains("scripts/check-gate2-public-handoff.py"));
    assert!(readme.contains("uses one\nfresh clone"));
    assert!(readme.contains("uses a second fresh clone"));
    assert!(readme.contains("executes the second clone's\n`scripts/gate2-outside-run.sh` wrapper"));
    assert!(readme.contains("waits until the wrapper prints the\nmanual quickstart checkpoint"));
    assert!(readme.contains("auto-confirms that checkpoint for diagnostic\nautomation only"));
    assert!(readme.contains("preflights the local runtime"));
    assert!(readme.contains("local Docker daemon"));
    assert!(readme.contains("`ffprobe`"));
    assert!(readme.contains("SHA tooling"));
    assert!(readme.contains("free the default"));
    assert!(readme.contains("stop that app and rerun"));
    assert!(readme.contains("fixture or fork URLs"));
    assert!(readme.contains("scripts/check-gate2-outside-readiness.py"));
    assert!(readme.contains("Outside-run wrapper: yes"));
    assert!(readme.contains("outside-run stopwatch source artifact"));
    assert!(readme.contains("prebuilt image overrides"));
    assert!(readme.contains("evidence"));
    assert!(readme.contains("artifact path overrides"));
    assert!(readme.contains("alternate Compose project names"));
    assert!(readme.contains("teardown"));
    assert!(readme.contains("scripts/generate-gate2-outside-proof.py"));
    assert!(readme.contains("--attest-outside-run"));
    assert!(readme.contains("proof writing"));
    assert!(readme.contains("browser proof"));
    assert!(readme.contains("browser recording"));
    assert!(readme.contains("BEATER_GATE2_CLONE_STARTED_EPOCH"));
    assert!(readme.contains("clone time"));
    assert!(readme.contains("It also sets"));
    assert!(readme.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(readme.contains("removes any previous Beater stopwatch project"));
    assert!(readme.contains("gate2-compose-browser-demo.webm"));
    assert!(readme.contains("prebuilt `dashboard-e2e` Playwright browser proof"));
    assert!(readme.contains("prebuilt stock OpenTelemetry Python runner container"));
    assert!(readme.contains("pins `beaterd`, `dashboard`, `dashboard-e2e`, and `otel-python`"));
    assert!(readme.contains("current-SHA"));
    assert!(readme.contains("`beaterd`, `dashboard`, `dashboard-e2e`, and `otel-python` GHCR"));
    assert!(readme.contains("mismatched SHA-pinned image references"));
    assert!(readme.contains("Time-to-quickstart-click"));
    assert!(readme.contains("manual\nquickstart click confirmation"));
    assert!(readme.contains("checks local Docker, Docker Compose, curl, `ffprobe`,"));
    assert!(readme.contains("mismatched trace IDs"));
    assert!(readme.contains("mismatched API/dashboard endpoints"));
    assert!(readme.contains("repo-relative `docs/demos/` artifacts"));
    assert!(readme.contains("prebuilt GHCR image digests"));
    assert!(readme.contains("public GHCR manifest digest"));
    assert!(readme.contains("mismatched image digests"));
    assert!(readme.contains("BEATER_GATE2_RUN_ID"));
    assert!(readme.contains("fresh per-run quickstart release ID"));
    assert!(readme.contains("Beater image service rows"));
    assert!(readme.contains("structured `proof-image` rows"));
    assert!(readme.contains("recording notes from a different dashboard session"));
    assert!(readme.contains("playable WebM metadata"));
    assert!(readme.contains("playable WebM capture of\nat least 64 KiB and at least 8 seconds"));
    assert!(readme.contains("Recording mode: compose"));
    assert!(readme.contains("EBML/WebM, Segment,"));
    assert!(readme.contains("artifact paths must not traverse symlinks"));
    assert!(readme.contains("The notes"));
    assert!(readme.contains("must declare\n`Recording mode: compose`"));
    assert!(readme.contains("describe the full recorded flow"));
    assert!(readme.contains("hash that does not match the committed file"));
    assert!(readme.contains("fresh clone from `https://github.com/jadenfix/beater.git`"));
    assert!(readme.contains("cloned readiness"));
    assert!(readme.contains("wrapper dry-run checks"));
    assert!(readme.contains("immediately before that second `git clone`"));
    assert!(readme.contains("gate2-proof-contract"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert!(requirements.contains("docs/demos/gate2-outside-person-proof.md"));
    assert!(requirements.contains("scripts/gate2-outside-run.sh"));
    assert!(requirements.contains("scripts/check-gate2-public-handoff.py"));
    assert!(requirements.contains("scripts/check-gate2-outside-readiness.py"));
    assert!(requirements.contains("public-clone handoff verifier"));
    assert!(requirements.contains("requires the clone to match the current commit"));
    assert!(requirements.contains(
        "alternate-port/image-override/artifact-path/compose-project/teardown/run-id/registry-fixture evidence"
    ));
    assert!(requirements.contains("wrapper marker"));
    assert!(requirements.contains("scripts/generate-gate2-outside-proof.py"));
    assert!(requirements.contains("scripts/validate-gate2-outside-proof.sh"));
    assert!(requirements.contains("image-digest"));
    assert!(requirements.contains("quickstart release-ID"));
    assert!(requirements
        .contains("structured Beater image service rows plus `proof-image` digest rows"));
    assert!(requirements.contains("SHA-pinned prebuilt GHCR image references"));
    assert!(requirements.contains("dashboard-e2e"));
    assert!(requirements.contains("otel-python"));
    assert!(requirements.contains("recording-notes"));
    assert!(requirements.contains("outside-run attestation"));
    assert!(requirements.contains("outside-run stopwatch source artifact marker"));
    assert!(requirements.contains("repo-relative non-symlink `docs/demos/` artifacts"));
    assert!(requirements.contains("prebuilt GHCR image digests"));
    assert!(requirements.contains("public GHCR manifest digests"));
    assert!(requirements.contains("default API/OTLP/dashboard endpoints"));
    assert!(requirements.contains("recording-notes full-flow check"));
    assert!(requirements.contains("recording-file WebM/min-size/structure guard"));
    assert!(requirements.contains("recording-hash cross-checks"));
    assert!(requirements.contains("public multi-arch GHCR images"));
    assert!(requirements.contains("Gate 2 `--full-run` public handoff verification"));
    assert!(requirements.contains("canonical public source, Docker, Docker Compose v2"));
    assert!(requirements.contains("free default `8080`/`4317`/`3000` ports"));
    assert!(requirements.contains("maintainer-only runtime evidence"));
    assert!(requirements.contains("CI-enforced"));

    let compose = read(root.join("docker-compose.yml"));
    assert!(compose.contains("otel-python-quickstart"));
    assert!(compose.contains("five_line_otel.py"));
    assert!(compose.contains("context: ./examples/python"));
    assert!(!compose.contains("pip install --no-cache-dir opentelemetry-sdk"));

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
    assert!(quickstart.contains("beater.release_id"));
    assert!(quickstart.contains("BEATER_GATE2_RUN_ID"));
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
    assert!(record_script.contains("BEATER_GATE2_OUTSIDE_WRAPPER"));
    assert!(record_script.contains("BEATER_GATE2_PUBLIC_DASHBOARD_BASE"));
    assert!(record_script.contains("recordQuickstartFlow"));
    assert!(record_script.contains("recordAllKindFlow"));
    assert!(record_script.contains("gate2-compose-browser-demo.webm"));
    assert!(record_script.contains("createHash(\"sha256\")"));
    assert!(record_script.contains("docs/demos"));
    assert!(record_script.contains("gate2-browser-demo.webm"));
    assert!(record_script.contains("token breakdown"));
    assert!(record_script.contains("minimumRecordingMs = 9000"));
    assert!(record_script.contains("llmReviewDwellMs = 4500"));
    assert!(record_script.contains("toolReviewDwellMs = 2500"));
    assert!(record_script.contains("waitForReviewableRecording"));
    assert!(record_script.contains("Recording mode: compose"));
    assert!(record_script.contains("12 total, 5 prompt, 7 completion"));
    assert!(record_script.contains("33 total, 18 prompt, 11 completion, 4 reasoning"));

    let local_recording_notes = read(root.join("docs/demos/gate2-browser-demo.md"));
    assert!(local_recording_notes.contains("gate2-browser-demo.webm"));
    assert!(local_recording_notes.contains("Recording mode: all-kind"));
    assert!(local_recording_notes.contains("token breakdown"));
    assert!(!local_recording_notes.contains("model/tokens/cost"));

    let compose_recording_notes = read(root.join("docs/demos/gate2-compose-browser-demo.md"));
    assert!(compose_recording_notes.contains("# Gate 2 Compose Browser Demo"));
    assert!(compose_recording_notes.contains("Status: pending regeneration"));
    assert!(compose_recording_notes.contains("8-second reviewability floor"));
    assert!(compose_recording_notes.contains("default dashboard URL"));
    assert!(compose_recording_notes.contains("http://127.0.0.1:3000"));
    assert!(compose_recording_notes.contains("gate2-compose-browser-demo.webm"));
    assert!(compose_recording_notes.contains("SHA256"));
    assert!(compose_recording_notes.contains("Recording mode: compose"));
    assert!(compose_recording_notes.contains("Quickstart trace"));
    assert!(compose_recording_notes.contains("All-kind trace"));
    assert!(compose_recording_notes.contains("click five-line trace"));
    assert!(compose_recording_notes.contains("token breakdown"));
    assert!(!compose_recording_notes.contains("model, tokens, cost"));
    assert!(compose_recording_notes.contains("docs/demos/gate2-outside-person-proof.md"));

    let compose_recording = root.join("docs/demos/gate2-compose-browser-demo.webm");
    assert!(
        !compose_recording.exists(),
        "stale Gate 2 compose recordings must not occupy the canonical evidence path"
    );
    let compose_recording_fixture =
        root.join("bins/beaterd/tests/fixtures/gate2-compose-browser-demo.webm");
    let metadata = fs::metadata(&compose_recording_fixture)
        .unwrap_or_else(|err| panic!("stat {}: {err}", compose_recording_fixture.display()));
    assert!(
        metadata.len() > 64 * 1024,
        "Gate 2 compose recording test fixture must be a committed non-empty browser video"
    );

    let quickstart_e2e = read(root.join("web/dashboard/tests/e2e/quickstart.spec.ts"));
    assert!(quickstart_e2e.contains("five-line-llm-call"));
    assert!(quickstart_e2e.contains("gpt-quickstart"));
    assert!(quickstart_e2e.contains("kind=llm.call&model=gpt-quickstart"));
    assert!(quickstart_e2e.contains("BEATER_E2E_QUICKSTART_RELEASE"));
    assert!(quickstart_e2e.contains("releaseParam"));
    assert!(quickstart_e2e.contains("traceRow.click()"));
    assert!(quickstart_e2e.contains("toHaveURL"));
    assert!(quickstart_e2e.contains("hello from stock OpenTelemetry"));
    assert!(quickstart_e2e.contains("hello from Beater"));
    assert!(quickstart_e2e.contains("data-icon"));
    assert!(quickstart_e2e.contains("12 total, 5 prompt, 7 completion"));
    assert!(quickstart_e2e.contains("Span metrics"));
    assert!(quickstart_e2e.contains("Latency"));

    let readme = read(root.join("README.md"));
    assert!(readme.contains("As soon as the first `Open the dashboard:` quickstart URL appears"));
    assert!(readme.contains("not wait for the script to finish"));
}

fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = manifest.parent().and_then(|path| path.parent()) else {
        panic!("beaterd manifest must live under bins/beaterd");
    };
    root.to_path_buf()
}

fn find_required(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .unwrap_or_else(|| panic!("expected text not found: {needle:?}"))
}

fn assert_pinned_image(compose: &str, label: &str, image: &str, digest: &str) {
    let pinned = format!("image: {image}@{digest}");
    assert!(
        compose.lines().any(|line| line.trim() == pinned),
        "{label} must pin {image} to {digest}"
    );
    let floating = format!("image: {image}");
    assert!(
        !compose.lines().any(|line| line.trim() == floating),
        "{label} must not use floating image tag {image}"
    );
}

fn compose_service_block(compose: &str, service: &str) -> String {
    let marker = format!("  {service}:");
    let mut block = String::new();
    let mut in_block = false;
    for line in compose.lines() {
        if line == marker {
            in_block = true;
        } else if in_block
            && line.starts_with("  ")
            && !line.starts_with("    ")
            && line.ends_with(':')
        {
            break;
        }
        if in_block {
            block.push_str(line);
            block.push('\n');
        }
    }
    if block.is_empty() {
        panic!("compose service not found: {service}");
    }
    block
}

fn dockerignore_ignores(dockerignore: &str, pattern: &str) -> bool {
    dockerignore.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#') && (trimmed == pattern || trimmed == format!("{pattern}/"))
    })
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
