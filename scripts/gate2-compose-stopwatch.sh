#!/usr/bin/env bash
set -euo pipefail

project="${COMPOSE_PROJECT_NAME:-beater-stopwatch}"
keep="${KEEP_BEATER_COMPOSE:-1}"
reuse="${BEATER_GATE2_REUSE:-0}"
write_proof="${BEATER_GATE2_WRITE_PROOF:-0}"
proof_path="${BEATER_GATE2_STOPWATCH_PROOF:-docs/demos/gate2-compose-stopwatch.md}"
local_build="${BEATER_GATE2_LOCAL_BUILD:-0}"
browser_proof="${BEATER_GATE2_BROWSER_PROOF:-0}"
record_demo="${BEATER_GATE2_RECORD_DEMO:-0}"
record_demo_video="${BEATER_GATE2_RECORD_VIDEO:-docs/demos/gate2-compose-browser-demo.webm}"
record_demo_notes="${BEATER_GATE2_RECORD_NOTES:-docs/demos/gate2-compose-browser-demo.md}"
outside_wrapper="${BEATER_GATE2_OUTSIDE_WRAPPER:-0}"
prebuilt_pull_policy="${BEATER_GATE2_PULL_POLICY:-always}"
post_slo_timeout_seconds="${BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS:-300}"
host_http_port="${BEATER_HTTP_PORT:-8080}"
host_otlp_grpc_port="${BEATER_OTLP_GRPC_PORT:-4317}"
host_dashboard_port="${BEATER_DASHBOARD_PORT:-3000}"
api_url="http://127.0.0.1:$host_http_port"
otlp_url="http://127.0.0.1:$host_otlp_grpc_port"
dashboard_base_url="http://127.0.0.1:$host_dashboard_port"
start_epoch="$(date +%s)"
clone_started_epoch="${BEATER_GATE2_CLONE_STARTED_EPOCH:-}"
timing_start_epoch="$start_epoch"
timing_start_source="script"
clone_started_at="not provided"
if [[ -n "$clone_started_epoch" ]]; then
  if [[ ! "$clone_started_epoch" =~ ^[0-9]+$ ]]; then
    echo "BEATER_GATE2_CLONE_STARTED_EPOCH must be a Unix epoch second value" >&2
    exit 1
  fi
  if (( clone_started_epoch > start_epoch )); then
    echo "BEATER_GATE2_CLONE_STARTED_EPOCH must not be in the future" >&2
    exit 1
  fi
  timing_start_epoch="$clone_started_epoch"
  timing_start_source="external-clone"
fi
deadline_epoch=$((timing_start_epoch + 300))
started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
quickstart_browser_proof_status="not requested"
waterfall_browser_proof_status="not requested"
record_demo_status="not requested"
record_demo_sha256="not requested"
time_to_quickstart_click_seconds=""
all_kind_trace_id=""
all_kind_dashboard_url=""
outside_runner_next_steps=""
e2e_base_url="http://dashboard:3000"
git_sha="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
git_branch="$(git branch --show-current 2>/dev/null || echo unknown)"
git_branch="${git_branch:-detached}"
git_origin="$(git remote get-url origin 2>/dev/null || echo unknown)"
git_worktree_status="$(git status --porcelain 2>/dev/null || true)"
if [[ -n "$git_worktree_status" ]]; then
  git_worktree_clean="no"
else
  git_worktree_clean="yes"
fi
os_arch="$(uname -sm 2>/dev/null || echo unknown)"
docker_version="$(docker --version 2>/dev/null || echo unknown)"
compose_version="$(docker compose version 2>/dev/null || echo unknown)"
utc_from_epoch() {
  local epoch="$1"
  if date -u -r "$epoch" +"%Y-%m-%dT%H:%M:%SZ" >/dev/null 2>&1; then
    date -u -r "$epoch" +"%Y-%m-%dT%H:%M:%SZ"
  else
    date -u -d "@$epoch" +"%Y-%m-%dT%H:%M:%SZ"
  fi
}
if [[ "$timing_start_source" == "external-clone" ]]; then
  clone_started_at="$(utc_from_epoch "$clone_started_epoch")"
fi
if [[ "$local_build" != "1" && "$git_sha" =~ ^[0-9a-f]{40}$ ]]; then
  export BEATERD_IMAGE="${BEATERD_IMAGE:-ghcr.io/jadenfix/beater/beaterd:$git_sha}"
  export BEATER_DASHBOARD_IMAGE="${BEATER_DASHBOARD_IMAGE:-ghcr.io/jadenfix/beater/dashboard:$git_sha}"
  export BEATER_DASHBOARD_E2E_IMAGE="${BEATER_DASHBOARD_E2E_IMAGE:-ghcr.io/jadenfix/beater/dashboard-e2e:$git_sha}"
  export BEATER_OTEL_PYTHON_IMAGE="${BEATER_OTEL_PYTHON_IMAGE:-ghcr.io/jadenfix/beater/otel-python:$git_sha}"
fi
beater_image_ref="${BEATERD_IMAGE:-local-build}"
dashboard_image_ref="${BEATER_DASHBOARD_IMAGE:-local-build}"
dashboard_e2e_image_ref="${BEATER_DASHBOARD_E2E_IMAGE:-local-build}"
otel_python_image_ref="${BEATER_OTEL_PYTHON_IMAGE:-local-build}"
if [[ "$record_demo" == "1" ]]; then
  browser_proof="1"
fi
all_kinds=(
  agent.run
  agent.turn
  agent.plan
  agent.step
  retrieval.query
  memory.read
  guardrail.check
  llm.call
  tool.call
  mcp.request
  memory.write
  evaluator.run
  human.review
  replay.run
)
if [[ "$local_build" == "1" ]]; then
  compose_files=(-f docker-compose.yml)
  startup_mode="local-build"
  startup_args=(up -d --build postgres nats minio beaterd dashboard)
else
  compose_files=(-f docker-compose.prebuilt.yml)
  startup_mode="prebuilt-image"
  startup_args=(up -d --pull "$prebuilt_pull_policy" postgres nats minio beaterd dashboard)
fi

compose() {
  docker compose "${compose_files[@]}" -p "$project" "$@"
}

compose_run_e2e() {
  local run_args=(run --rm)
  if [[ "$local_build" == "1" ]]; then
    run_args+=(--build)
  else
    run_args+=(--pull "$prebuilt_pull_policy")
  fi
  compose "${run_args[@]}" "$@"
}

compose_run_tool() {
  local run_args=(run --rm)
  if [[ "$local_build" == "1" ]]; then
    run_args+=(--build)
  else
    run_args+=(--pull "$prebuilt_pull_policy")
  fi
  compose "${run_args[@]}" "$@"
}

cleanup() {
  if [[ "$keep" != "1" ]]; then
    compose down -v --remove-orphans >/dev/null 2>&1 || true
  fi
}

clean_start() {
  if [[ "$reuse" == "1" ]]; then
    return 0
  fi
  compose down -v --remove-orphans >/dev/null 2>&1 || true
}

terminate_tree() {
  local pid="$1"
  local child
  for child in $(pgrep -P "$pid" 2>/dev/null || true); do
    terminate_tree "$child"
  done
  kill "$pid" 2>/dev/null || true
}

kill_tree() {
  local pid="$1"
  local child
  for child in $(pgrep -P "$pid" 2>/dev/null || true); do
    kill_tree "$child"
  done
  kill -9 "$pid" 2>/dev/null || true
}

run_before_deadline() {
  local label="$1"
  shift
  "$@" &
  local pid=$!
  while kill -0 "$pid" 2>/dev/null; do
    if (( $(date +%s) >= deadline_epoch )); then
      echo "Timed out during $label" >&2
      terminate_tree "$pid"
      sleep 2
      kill_tree "$pid"
      wait "$pid" 2>/dev/null || true
      return 1
    fi
    sleep 1
  done
  wait "$pid"
}

run_with_step_timeout() {
  local label="$1"
  shift
  local saved_deadline="$deadline_epoch"
  deadline_epoch=$(($(date +%s) + post_slo_timeout_seconds))
  run_before_deadline "$label" "$@"
  local status=$?
  deadline_epoch="$saved_deadline"
  return "$status"
}

wait_url() {
  local url="$1"
  local label="$2"
  until curl -fsS "$url" >/tmp/beater-stopwatch-response 2>/tmp/beater-stopwatch-error; do
    if (( $(date +%s) >= deadline_epoch )); then
      echo "Timed out waiting for $label at $url" >&2
      cat /tmp/beater-stopwatch-error >&2 || true
      return 1
    fi
    sleep 2
  done
}

require_text() {
  local url="$1"
  local needle="$2"
  local body
  body="$(curl -fsS "$url")"
  if [[ "$body" != *"$needle"* ]]; then
    echo "Expected '$needle' in $url" >&2
    return 1
  fi
}

wait_text() {
  local url="$1"
  local needle="$2"
  local label="$3"
  until require_text "$url" "$needle" >/dev/null 2>&1; do
    if (( $(date +%s) >= deadline_epoch )); then
      echo "Timed out waiting for $label to contain '$needle' at $url" >&2
      require_text "$url" "$needle"
      return 1
    fi
    sleep 2
  done
}

first_trace_id() {
  sed -n 's/.*"trace_id"[[:space:]]*:[[:space:]]*"\([0-9a-f]\{32\}\)".*/\1/p' | head -n 1
}

sha256_file() {
  local file="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  else
    echo "unknown"
  fi
}

service_image_digest() {
  local service="$1"
  local image_id
  local repo_digest
  local full_id
  local image_ref=""
  image_id="$(compose images -q "$service" 2>/dev/null | head -n 1 || true)"
  if [[ -z "$image_id" ]]; then
    case "$service" in
      beaterd)
        image_ref="${BEATERD_IMAGE:-}"
        ;;
      dashboard)
        image_ref="${BEATER_DASHBOARD_IMAGE:-}"
        ;;
      dashboard-e2e)
        image_ref="${BEATER_DASHBOARD_E2E_IMAGE:-}"
        ;;
      otel-python | otel-python-quickstart | otel-python-smoke)
        image_ref="${BEATER_OTEL_PYTHON_IMAGE:-}"
        ;;
    esac
    if [[ -n "$image_ref" ]]; then
      image_id="$(docker image inspect --format '{{.Id}}' "$image_ref" 2>/dev/null || true)"
    fi
  fi
  if [[ -z "$image_id" ]]; then
    echo "unknown"
    return
  fi
  repo_digest="$(docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' "$image_id" 2>/dev/null | head -n 1 || true)"
  if [[ -n "$repo_digest" ]]; then
    echo "$repo_digest"
    return
  fi
  full_id="$(docker image inspect --format '{{.Id}}' "$image_id" 2>/dev/null || true)"
  if [[ -n "$full_id" ]]; then
    echo "$full_id"
    return
  fi
  echo "$image_id"
}

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    return 1
  fi
}

print_port_owner() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    echo "Listening process on TCP $port:" >&2
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >&2 || true
  elif command -v ss >/dev/null 2>&1; then
    echo "Listening process on TCP $port:" >&2
    ss -ltnp "sport = :$port" >&2 || true
  else
    echo "Install lsof or ss to identify the process holding TCP $port." >&2
  fi
}

port_is_free() {
  if (: >/dev/tcp/127.0.0.1/"$1") >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

require_free_port() {
  local port="$1"
  local label="$2"
  local env_name="$3"
  if ! port_is_free "$port"; then
    cat >&2 <<EOF
Port $port for $label is already in use before Gate 2 Compose startup.

For outside-person Gate 2 evidence, free the default port and rerun. For
maintainer diagnostics only, set $env_name to an unused alternate port.
If this is a stale Beater run, clean it up with:
  docker compose -f docker-compose.prebuilt.yml -p $project down -v --remove-orphans
EOF
    print_port_owner "$port"
    return 1
  fi
}

preflight_prerequisites() {
  require_command docker
  require_command curl
  if [[ "$record_demo" == "1" ]] && ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
    echo "Gate 2 recording proof requires shasum or sha256sum before the stopwatch starts." >&2
    return 1
  fi
  if [[ "$record_demo" == "1" ]] && ! command -v ffprobe >/dev/null 2>&1; then
    echo "Gate 2 recording proof requires ffprobe before the stopwatch starts." >&2
    return 1
  fi
  if [[ -n "${DOCKER_HOST:-}" && "${DOCKER_HOST:-}" != unix://* && "${DOCKER_HOST:-}" != npipe://* ]]; then
    echo "Gate 2 outside-person proof requires a local Docker daemon because the browser proof uses 127.0.0.1." >&2
    echo "Unset DOCKER_HOST or switch to a local Docker context and rerun." >&2
    return 1
  fi
  if ! docker info >/dev/null 2>&1; then
    echo "Docker daemon is not reachable; start Docker and rerun Gate 2 proof." >&2
    return 1
  fi
  if ! docker compose version >/dev/null 2>&1; then
    echo "Docker Compose v2 is required for Gate 2 proof." >&2
    return 1
  fi
  local docker_context_host
  docker_context_host="$(docker context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null | head -n 1 || true)"
  if [[ -n "$docker_context_host" && "$docker_context_host" != "<no value>" && "$docker_context_host" != unix://* && "$docker_context_host" != npipe://* ]]; then
    echo "Gate 2 outside-person proof requires a local Docker context because the browser proof uses 127.0.0.1; current Docker endpoint is $docker_context_host." >&2
    return 1
  fi
}

preflight_ports() {
  if [[ "$reuse" != "1" ]]; then
    require_free_port "$host_http_port" "beaterd HTTP" "BEATER_HTTP_PORT"
    require_free_port "$host_otlp_grpc_port" "OTLP gRPC" "BEATER_OTLP_GRPC_PORT"
    require_free_port "$host_dashboard_port" "dashboard" "BEATER_DASHBOARD_PORT"
  fi
}

trap cleanup EXIT

run_before_deadline "Gate 2 prerequisite preflight" preflight_prerequisites
run_before_deadline "clean previous Gate 2 state" clean_start
run_before_deadline "Gate 2 port preflight" preflight_ports
run_before_deadline "compose startup ($startup_mode)" compose "${startup_args[@]}"
wait_url "$api_url/health" "beaterd"
wait_url "$dashboard_base_url/?tenant=demo&project=demo&environment=local" "dashboard"

run_before_deadline "five-line OTEL snippet" compose_run_tool otel-python-quickstart

quickstart_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-quickstart"
wait_text "$quickstart_query" "gpt-quickstart" "five-line OTEL trace"
trace_id="$(curl -fsS "$quickstart_query" | first_trace_id)"
if [[ -z "$trace_id" ]]; then
  echo "Could not parse quickstart trace_id from $quickstart_query" >&2
  exit 1
fi
dashboard_url="$dashboard_base_url/?tenant=demo&project=demo&environment=local&trace=$trace_id"

wait_text "$dashboard_url" "Agent Trace Debugger" "dashboard"
wait_text "$dashboard_url" "five-line-llm-call" "dashboard quickstart trace"
wait_text "$dashboard_url" "gpt-quickstart" "dashboard model detail"
wait_text "$dashboard_url" "hello from stock OpenTelemetry" "dashboard prompt detail"
script_to_first_trace_seconds=$(($(date +%s) - start_epoch))
time_to_first_trace_seconds=$(($(date +%s) - timing_start_epoch))
if (( time_to_first_trace_seconds > 300 )); then
  echo "Time-to-first-trace exceeded 300s: ${time_to_first_trace_seconds}s" >&2
  exit 1
fi
cat <<EOF
Gate 2 first trace visible in ${time_to_first_trace_seconds}s.

Open the dashboard:
  $dashboard_url
EOF

if [[ "$outside_wrapper" == "1" ]]; then
  cat <<EOF

Outside-run timing-critical browser step:
  Open the quickstart URL above in a normal browser now; do not wait for the script to finish.
  Click the quickstart trace, then click the llm.call span.
  Confirm prompt, completion, model, token breakdown, cost, and latency are visible before 5 minutes.
  Keep this command running for the post-SLO all-kind and recording evidence.
EOF
fi

if [[ "$browser_proof" == "1" ]]; then
  run_before_deadline "five-line dashboard browser proof" compose_run_e2e \
    -e BEATER_E2E_QUICKSTART_TRACE_ID="$trace_id" \
    -e PLAYWRIGHT_BASE_URL="$e2e_base_url" \
    dashboard-e2e \
    npx playwright test tests/e2e/quickstart.spec.ts
  quickstart_browser_proof_status="passed"
  script_to_quickstart_click_seconds=$(($(date +%s) - start_epoch))
  time_to_quickstart_click_seconds=$(($(date +%s) - timing_start_epoch))
  if (( time_to_quickstart_click_seconds > 300 )); then
    echo "Time-to-quickstart-click exceeded 300s: ${time_to_quickstart_click_seconds}s" >&2
    exit 1
  fi

  run_with_step_timeout "stock Python all-kind OTEL fixture" compose_run_tool otel-python-smoke

  all_kind_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-demo&release=compose-demo"
  run_with_step_timeout "wait for stock Python all-kind OTEL trace" wait_text "$all_kind_query" "gpt-demo" "stock Python all-kind OTEL trace"
  all_kind_trace_id="$(curl -fsS "$all_kind_query" | first_trace_id)"
  if [[ -z "$all_kind_trace_id" ]]; then
    echo "Could not parse all-kind trace_id from $all_kind_query" >&2
    exit 1
  fi
  all_kind_dashboard_url="$dashboard_base_url/?tenant=demo&project=demo&environment=local&trace=$all_kind_trace_id"
  run_with_step_timeout "wait for dashboard all-kind llm.call" wait_text "$all_kind_dashboard_url" "call-policy-model" "dashboard all-kind llm.call"
  for kind in "${all_kinds[@]}"; do
    run_with_step_timeout "wait for dashboard all-kind waterfall $kind" wait_text "$all_kind_dashboard_url" "$kind" "dashboard all-kind waterfall"
  done

  run_with_step_timeout "all-kind waterfall browser proof" compose_run_e2e \
    -e BEATER_E2E_TRACE_ID="$all_kind_trace_id" \
    -e PLAYWRIGHT_BASE_URL="$e2e_base_url" \
    dashboard-e2e \
    npx playwright test tests/e2e/dashboard.spec.ts
  if [[ "$record_demo" == "1" ]]; then
    run_with_step_timeout "Gate 2 compose browser recording" compose_run_e2e \
      -e BEATER_GATE2_RECORD_MODE=compose \
      -e BEATER_E2E_QUICKSTART_TRACE_ID="$trace_id" \
      -e BEATER_E2E_ALL_KIND_TRACE_ID="$all_kind_trace_id" \
      -e BEATER_GATE2_RECORD_VIDEO="$record_demo_video" \
      -e BEATER_GATE2_RECORD_NOTES="$record_demo_notes" \
      -e PLAYWRIGHT_BASE_URL="$e2e_base_url" \
      -e BEATER_GATE2_PUBLIC_DASHBOARD_BASE="$dashboard_base_url" \
      dashboard-e2e \
      npm run record:gate2
  fi
  waterfall_browser_proof_status="passed"
  if [[ "$record_demo" == "1" ]]; then
    record_demo_status="passed"
    record_demo_sha256="$(sha256_file "$record_demo_video")"
  fi
fi

ended_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
script_duration_seconds=$(($(date +%s) - start_epoch))
duration_seconds=$(($(date +%s) - timing_start_epoch))
time_to_quickstart_click_display="${time_to_quickstart_click_seconds:+${time_to_quickstart_click_seconds}s}"
time_to_quickstart_click_display="${time_to_quickstart_click_display:-not requested}"
script_to_quickstart_click_display="${script_to_quickstart_click_seconds:+${script_to_quickstart_click_seconds}s}"
script_to_quickstart_click_display="${script_to_quickstart_click_display:-not requested}"
image_summary="$(compose images 2>/dev/null || true)"
beater_image_digest="$(service_image_digest beaterd)"
dashboard_image_digest="$(service_image_digest dashboard)"
dashboard_e2e_image_digest="$(service_image_digest dashboard-e2e)"
otel_python_image_digest="$(service_image_digest otel-python-quickstart)"
if [[ "$write_proof" == "1" ]]; then
  mkdir -p "$(dirname "$proof_path")"
  cat >"$proof_path" <<EOF
# Gate 2 Compose Stopwatch Proof

- Timing start source: $timing_start_source
- Clone started at: $clone_started_at
- Script started at: $started_at
- Started: $started_at
- Ended: $ended_at
- Time-to-first-trace: ${time_to_first_trace_seconds}s
- Script-to-first-trace: ${script_to_first_trace_seconds}s
- Time-to-quickstart-click: $time_to_quickstart_click_display
- Script-to-quickstart-click: $script_to_quickstart_click_display
- Total duration: ${duration_seconds}s
- Script duration: ${script_duration_seconds}s
- Limit: 300s
- Git SHA: \`$git_sha\`
- Git branch: \`$git_branch\`
- Git origin: \`$git_origin\`
- Git worktree clean: $git_worktree_clean
- OS/arch: \`$os_arch\`
- Docker: \`$docker_version\`
- Docker Compose: \`$compose_version\`
- Startup mode: $startup_mode
- Clean start: $([[ "$reuse" == "1" ]] && echo "no" || echo "yes")
- Reuse override: \`BEATER_GATE2_REUSE=$reuse\`
- Outside-run wrapper: $([[ "$outside_wrapper" == "1" ]] && echo "yes" || echo "no")
- Prebuilt pull policy: \`$prebuilt_pull_policy\`
- Compose project: $project
- Beater image reference: \`$beater_image_ref\`
- Dashboard image reference: \`$dashboard_image_ref\`
- Dashboard e2e image reference: \`$dashboard_e2e_image_ref\`
- OTEL Python image reference: \`$otel_python_image_ref\`
- Beater image digest: \`$beater_image_digest\`
- Dashboard image digest: \`$dashboard_image_digest\`
- Dashboard e2e image digest: \`$dashboard_e2e_image_digest\`
- OTEL Python image digest: \`$otel_python_image_digest\`
- Quickstart snippet: \`examples/python/five_line_otel.py\`
- API endpoint: \`$api_url\`
- OTLP endpoint: \`$otlp_url\`
- Dashboard base: \`$dashboard_base_url\`
- Quickstart trace: \`$trace_id\`
- Quickstart dashboard: $dashboard_url
- Quickstart browser proof: $quickstart_browser_proof_status
- All-kind nested trace: \`${all_kind_trace_id:-not requested}\`
- All-kind dashboard: ${all_kind_dashboard_url:-not requested}
- All-kind waterfall browser proof: $waterfall_browser_proof_status
- Browser recording: $record_demo_status
- Browser recording artifact: \`$([[ "$record_demo" == "1" ]] && echo "$record_demo_video" || echo "not requested")\`
- Browser recording notes: \`$([[ "$record_demo" == "1" ]] && echo "$record_demo_notes" || echo "not requested")\`
- Browser recording SHA256: \`$record_demo_sha256\`

## Compose Images

\`\`\`text
$image_summary
\`\`\`

This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

\`\`\`bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
\`\`\`
EOF
fi

if [[ "$outside_wrapper" == "1" ]]; then
  outside_runner_next_steps="$(cat <<EOF

Outside-run next steps:
  1. If you have not already done so, open $dashboard_url in a normal browser for the quickstart trace.
  2. Click the quickstart trace, then click the llm.call span.
  3. Confirm prompt, completion, model, token breakdown, cost, and latency are visible.
  4. Open ${all_kind_dashboard_url:-not requested} in a normal browser for the all-kind waterfall.
  5. Confirm run -> turn -> step -> tool -> MCP nesting is visible.
  6. Generate the completed proof with scripts/generate-gate2-outside-proof.py.
  7. Validate it with scripts/validate-gate2-outside-proof.sh.
  8. After evidence is captured, clean up with:
       docker compose -f docker-compose.prebuilt.yml -p $project down -v --remove-orphans
EOF
)"
fi

if [[ "$outside_wrapper" == "1" ]]; then
  maintainer_diagnostic_tips="Maintainer diagnostic overrides are intentionally suppressed for outside-person evidence."
else
  maintainer_diagnostic_tips="$(cat <<'EOF'
Set KEEP_BEATER_COMPOSE=0 to tear containers down automatically.
Set BEATER_GATE2_LOCAL_BUILD=1 to build images from local source instead of
pulling prebuilt GHCR images.
Set BEATER_GATE2_REUSE=1 to skip the pre-run Compose down/volume removal and
state cleanup for local warm-loop debugging only.
Set BEATER_GATE2_BROWSER_PROOF=1 to run the Playwright browser proof for the
five-line quickstart trace and all-kind nested agent waterfall in the same proof
run.
Set BEATER_GATE2_RECORD_DEMO=1 to record the quickstart click-through and
all-kind waterfall browser proof as a committed demo artifact.
Set BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS to control each post-SLO evidence
step timeout after the five-line trace and quickstart click have passed.
EOF
)"
fi

cat <<EOF
Gate 2 compose stopwatch passed in ${time_to_first_trace_seconds}s to first trace (${duration_seconds}s total).

Timing start source:
  $timing_start_source

Script-to-first-trace:
  ${script_to_first_trace_seconds}s

Time to quickstart browser click:
  $time_to_quickstart_click_display

Open the dashboard:
  $dashboard_url

All-kind waterfall dashboard:
  ${all_kind_dashboard_url:-not requested}

Browser recording:
  $record_demo_status

Browser recording artifact:
  $([[ "$record_demo" == "1" ]] && echo "$record_demo_video" || echo "not requested")

Five-line snippet:
  examples/python/five_line_otel.py

OTLP endpoint:
  $otlp_url

Startup mode:
  $startup_mode

Clean start:
  $([[ "$reuse" == "1" ]] && echo "no (BEATER_GATE2_REUSE=1)" || echo "yes")

$maintainer_diagnostic_tips
$outside_runner_next_steps
EOF
