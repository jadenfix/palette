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
compose_logs_path="${BEATER_GATE2_COMPOSE_LOGS:-}"
outside_wrapper="${BEATER_GATE2_OUTSIDE_WRAPPER:-0}"
prebuilt_pull_policy="${BEATER_GATE2_PULL_POLICY:-always}"
post_slo_timeout_seconds="${BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS:-300}"
min_recording_seconds="8.0"
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
compose_logs_artifact="not requested"
time_to_quickstart_click_seconds=""
script_to_quickstart_click_seconds=""
quickstart_click_source="not requested"
manual_quickstart_confirmation="not requested"
manual_quickstart_confirmation_code="not requested"
manual_quickstart_confirmation_salt="not requested"
quickstart_span_id=""
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
gate2_run_id="${BEATER_GATE2_RUN_ID:-gate2-${git_sha:0:12}-${start_epoch}-$$}"
quickstart_confirmation_salt="${BEATER_GATE2_CONFIRMATION_SALT:-gate2-${start_epoch}-$$-${RANDOM:-0}-${RANDOM:-0}-${RANDOM:-0}}"
export BEATER_GATE2_CONFIRMATION_SALT="$quickstart_confirmation_salt"
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
  startup_args=(up -d --build beaterd dashboard)
else
  compose_files=(-f docker-compose.prebuilt.yml)
  startup_mode="prebuilt-image"
  startup_args=(up -d --pull "$prebuilt_pull_policy" beaterd dashboard)
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

save_compose_logs() {
  local path="$1"
  mkdir -p "$(dirname "$path")"
  {
    echo "# Gate 2 Compose Logs"
    echo "Saved at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "Compose project: $project"
    echo "Startup mode: $startup_mode"
    echo "Command: docker compose ${compose_files[*]} -p $project logs --no-color --timestamps"
    echo
    compose logs --no-color --timestamps
  } >"$path"
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
  compose down -v --remove-orphans >/dev/null
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

confirm_manual_quickstart_click() {
  if [[ "$outside_wrapper" != "1" ]]; then
    return 0
  fi
  local now
  local remaining
  now="$(date +%s)"
  remaining=$((deadline_epoch - now))
  if (( remaining <= 0 )); then
    echo "Timed out before manual quickstart click confirmation." >&2
    return 1
  fi
  if [[ -z "${quickstart_confirmation_code:-}" ]]; then
    echo "Missing quickstart confirmation code before manual checkpoint." >&2
    return 1
  fi
  local entered_code
  cat <<EOF

Manual outside-run checkpoint:
  ${remaining}s remain in the 5-minute clone-to-click SLO.
  In a normal browser, open the quickstart trace-list URL above first, click the
  quickstart trace, click the llm.call span, and confirm prompt, completion,
  model, token breakdown, cost, latency, and the confirmation code are visible.
  Type the confirmation code shown in the selected llm.call detail, then press Enter.
EOF
  if ! IFS= read -r -t "$remaining" entered_code; then
    echo "Timed out waiting for manual quickstart click confirmation." >&2
    return 1
  fi
  entered_code="$(printf '%s' "$entered_code" | tr '[:lower:]' '[:upper:]' | tr -d '[:space:]')"
  if [[ "$entered_code" != "$quickstart_confirmation_code" ]]; then
    echo "Manual quickstart confirmation code did not match the selected llm.call detail." >&2
    return 1
  fi
  script_to_quickstart_click_seconds=$(($(date +%s) - start_epoch))
  time_to_quickstart_click_seconds=$(($(date +%s) - timing_start_epoch))
  quickstart_click_source="manual-outside-runner"
  manual_quickstart_confirmation="yes"
  manual_quickstart_confirmation_code="$quickstart_confirmation_code"
  manual_quickstart_confirmation_salt="$quickstart_confirmation_salt"
  if (( time_to_quickstart_click_seconds > 300 )); then
    echo "Time-to-quickstart-click exceeded 300s: ${time_to_quickstart_click_seconds}s" >&2
    return 1
  fi
}

first_trace_id() {
  sed -n 's/.*"trace_id"[[:space:]]*:[[:space:]]*"\([0-9a-f]\{32\}\)".*/\1/p' | head -n 1
}

first_llm_span_id() {
  tr '{' '\n' \
    | sed -n '/"kind"[[:space:]]*:[[:space:]]*"llm.call"/s/.*"span_id"[[:space:]]*:[[:space:]]*"\([0-9a-f]\{16\}\)".*/\1/p' \
    | head -n 1
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

sha256_text() {
  local text="$1"
  if command -v shasum >/dev/null 2>&1; then
    printf '%s' "$text" | shasum -a 256 | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$text" | sha256sum | awk '{print $1}'
  else
    echo "unknown"
  fi
}

quickstart_confirmation_code_for_span() {
  local trace="$1"
  local span="$2"
  local digest
  digest="$(sha256_text "gate2:$quickstart_confirmation_salt:$trace:$span")"
  if [[ ! "$digest" =~ ^[0-9a-f]{64}$ ]]; then
    echo "Could not compute quickstart confirmation code for span $span in trace $trace" >&2
    return 1
  fi
  printf '%s\n' "${digest:0:8}" | tr '[:lower:]' '[:upper:]'
}

recording_probe() {
  local file="$1"
  ffprobe -v error \
    -select_streams v:0 \
    -show_entries stream=codec_type \
    -show_entries format=duration \
    -of default=noprint_wrappers=1:nokey=0 \
    "$file" 2>/dev/null
}

recording_duration_seconds() {
  awk -F= '/^duration=/ { print $2; exit }'
}

require_reviewable_recording() {
  local file="$1"
  local probe
  local duration
  probe="$(recording_probe "$file" || true)"
  if ! printf '%s\n' "$probe" | grep -qx "codec_type=video"; then
    echo "Gate 2 browser recording must contain a video stream: $file" >&2
    return 1
  fi
  duration="$(printf '%s\n' "$probe" | recording_duration_seconds)"
  if ! awk -v duration="$duration" -v minimum="$min_recording_seconds" 'BEGIN {
    if (duration !~ /^[0-9]+([.][0-9]+)?$/) exit 1
    exit ((duration + 0) >= (minimum + 0)) ? 0 : 1
  }'; then
    echo "Gate 2 browser recording must be a reviewable video of at least ${min_recording_seconds}s; got ${duration:-missing} for $file" >&2
    return 1
  fi
}

service_image_digest() {
  local service="$1"
  local image_id
  local repo_digest
  local full_id
  local expected_repo=""
  local image_ref=""
  case "$service" in
    beaterd)
      expected_repo="ghcr.io/jadenfix/beater/beaterd"
      ;;
    dashboard)
      expected_repo="ghcr.io/jadenfix/beater/dashboard"
      ;;
    dashboard-e2e)
      expected_repo="ghcr.io/jadenfix/beater/dashboard-e2e"
      ;;
    otel-python | otel-python-quickstart | otel-python-smoke)
      expected_repo="ghcr.io/jadenfix/beater/otel-python"
      ;;
  esac
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
  repo_digest="$(
    docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' "$image_id" 2>/dev/null \
      | awk -v prefix="${expected_repo}@sha256:" 'index($0, prefix) == 1 { print; exit }' \
      || true
  )"
  if [[ -n "$repo_digest" ]]; then
    echo "$repo_digest"
    return
  fi
  if [[ -n "$expected_repo" ]]; then
    echo "unknown"
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
    print_port_owner_details "$port"
  elif command -v ss >/dev/null 2>&1; then
    echo "Listening process on TCP $port:" >&2
    ss -ltnp "sport = :$port" >&2 || true
  else
    echo "Install lsof or ss to identify the process holding TCP $port." >&2
  fi
}

print_port_owner_details() {
  local port="$1"
  local pid
  local owner_command
  local owner_cwd
  if ! command -v lsof >/dev/null 2>&1; then
    return 0
  fi
  while IFS= read -r pid; do
    if [[ ! "$pid" =~ ^[0-9]+$ ]]; then
      continue
    fi
    owner_command=""
    if command -v ps >/dev/null 2>&1; then
      owner_command="$(ps -p "$pid" -o command= 2>/dev/null | sed -n '1p' || true)"
    fi
    if [[ -n "$owner_command" ]]; then
      echo "process $pid command: $owner_command" >&2
    fi
    owner_cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p' | head -n 1 || true)"
    if [[ -n "$owner_cwd" ]]; then
      echo "process $pid cwd: $owner_cwd" >&2
    fi
  done < <(lsof -nP -t -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk '!seen[$0]++' || true)
}

port_is_free() {
  if (: >/dev/tcp/127.0.0.1/"$1") >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

docker_endpoint_is_local() {
  local endpoint="$1"
  [[
    -z "$endpoint" ||
    "$endpoint" == "<no value>" ||
    "$endpoint" == unix://* ||
    "$endpoint" == npipe://* ||
    "$endpoint" == tcp://localhost:* ||
    "$endpoint" == tcp://127.* ||
    "$endpoint" == tcp://[::1]:*
  ]]
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
If another app is listed below, stop that app before rerunning; do not set
$env_name for outside-person evidence.
EOF
    print_port_owner "$port"
    return 1
  fi
}

preflight_prerequisites() {
  require_command docker
  require_command curl
  if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
    echo "Gate 2 stopwatch requires shasum or sha256sum before the stopwatch starts." >&2
    return 1
  fi
  if [[ "$record_demo" == "1" ]] && ! command -v ffprobe >/dev/null 2>&1; then
    echo "Gate 2 recording proof requires ffprobe before the stopwatch starts." >&2
    return 1
  fi
  if ! docker_endpoint_is_local "${DOCKER_HOST:-}"; then
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
  if ! docker_endpoint_is_local "$docker_context_host"; then
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

run_before_deadline "five-line OTEL snippet" compose_run_tool \
  -e BEATER_GATE2_RUN_ID="$gate2_run_id" \
  otel-python-quickstart

quickstart_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-quickstart&release=$gate2_run_id"
wait_text "$quickstart_query" "$gate2_run_id" "fresh five-line OTEL trace"
trace_id="$(curl -fsS "$quickstart_query" | first_trace_id)"
if [[ -z "$trace_id" ]]; then
  echo "Could not parse quickstart trace_id from $quickstart_query" >&2
  exit 1
fi
quickstart_span_id="$(curl -fsS "$api_url/v1/traces/demo/$trace_id" | first_llm_span_id)"
if [[ -z "$quickstart_span_id" ]]; then
  echo "Could not parse quickstart span_id from $api_url/v1/traces/demo/$trace_id" >&2
  exit 1
fi
quickstart_confirmation_code="$(quickstart_confirmation_code_for_span "$trace_id" "$quickstart_span_id")"
dashboard_url="$dashboard_base_url/?tenant=demo&project=demo&environment=local&trace=$trace_id"
quickstart_list_url="$dashboard_base_url/?tenant=demo&project=demo&environment=local&kind=llm.call&model=gpt-quickstart&release=$gate2_run_id"

wait_text "$quickstart_list_url" "Agent Trace Debugger" "dashboard"
wait_text "$quickstart_list_url" "five-line-llm-call" "dashboard quickstart trace"
wait_text "$quickstart_list_url" "gpt-quickstart" "dashboard model detail"
wait_text "$dashboard_url" "hello from stock OpenTelemetry" "dashboard prompt detail"
script_to_first_trace_seconds=$(($(date +%s) - start_epoch))
time_to_first_trace_seconds=$(($(date +%s) - timing_start_epoch))
if (( time_to_first_trace_seconds > 300 )); then
  echo "Time-to-first-trace exceeded 300s: ${time_to_first_trace_seconds}s" >&2
  exit 1
fi
cat <<EOF
Gate 2 first trace visible in ${time_to_first_trace_seconds}s.

Open this quickstart trace-list URL first:
  $quickstart_list_url

Direct quickstart trace URL:
  $dashboard_url
EOF

if [[ "$outside_wrapper" == "1" ]]; then
  cat <<EOF

Outside-run timing-critical browser step:
  Open the quickstart trace-list URL above in a normal browser now; do not wait for the script to finish.
  Click the quickstart trace, then click the llm.call span.
  Confirm prompt, completion, model, token breakdown, cost, latency, and the confirmation code are visible before 5 minutes.
  Keep this command running for the post-SLO all-kind and recording evidence.
EOF
fi

confirm_manual_quickstart_click

if [[ "$browser_proof" == "1" ]]; then
  quickstart_browser_proof_args=(
    -e BEATER_E2E_QUICKSTART_TRACE_ID="$trace_id" \
    -e BEATER_E2E_QUICKSTART_RELEASE="$gate2_run_id" \
    -e PLAYWRIGHT_BASE_URL="$e2e_base_url" \
    dashboard-e2e \
    npx playwright test tests/e2e/quickstart.spec.ts
  )
  if [[ "$outside_wrapper" == "1" ]]; then
    run_with_step_timeout "five-line dashboard browser proof" compose_run_e2e "${quickstart_browser_proof_args[@]}"
  else
    run_before_deadline "five-line dashboard browser proof" compose_run_e2e "${quickstart_browser_proof_args[@]}"
  fi
  quickstart_browser_proof_status="passed"
  if [[ -z "$time_to_quickstart_click_seconds" ]]; then
    script_to_quickstart_click_seconds=$(($(date +%s) - start_epoch))
    time_to_quickstart_click_seconds=$(($(date +%s) - timing_start_epoch))
    quickstart_click_source="automated-browser-proof"
    if (( time_to_quickstart_click_seconds > 300 )); then
      echo "Time-to-quickstart-click exceeded 300s: ${time_to_quickstart_click_seconds}s" >&2
      exit 1
    fi
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
      -e BEATER_GATE2_OUTSIDE_WRAPPER="$outside_wrapper" \
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
    require_reviewable_recording "$record_demo_video"
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
proof_image_summary="$(cat <<EOF
$image_summary
proof-image beaterd $beater_image_ref $beater_image_digest
proof-image dashboard $dashboard_image_ref $dashboard_image_digest
proof-image dashboard-e2e $dashboard_e2e_image_ref $dashboard_e2e_image_digest
proof-image otel-python $otel_python_image_ref $otel_python_image_digest
EOF
)"
if [[ -n "$compose_logs_path" ]]; then
  save_compose_logs "$compose_logs_path"
  compose_logs_artifact="$compose_logs_path"
fi
if [[ "$outside_wrapper" == "1" ]]; then
  proof_followup_block="$(cat <<'EOF'
This is an outside-run stopwatch source artifact generated through
`scripts/gate2-outside-run.sh`. Gate 2 closes only after the completed
outside-person proof, this stopwatch proof, the recording, and the recording
notes are committed and `scripts/validate-gate2-outside-proof.sh` passes.

Regenerate:

Use the documented clone-to-browser command in
`docs/demos/gate2-outside-person-proof.md` from a fresh parent directory.
EOF
)"
else
  proof_followup_block="$(cat <<'EOF'
This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```
EOF
)"
fi
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
- Quickstart click source: $quickstart_click_source
- Manual quickstart confirmation: $manual_quickstart_confirmation
- Manual confirmation code: $manual_quickstart_confirmation_code
- Manual confirmation salt: \`$manual_quickstart_confirmation_salt\`
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
- Compose logs artifact: \`$compose_logs_artifact\`
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
- Quickstart release ID: \`$gate2_run_id\`
- Quickstart trace: \`$trace_id\`
- Quickstart span: \`$quickstart_span_id\`
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
$proof_image_summary
\`\`\`

$proof_followup_block
EOF
fi

if [[ "$outside_wrapper" == "1" ]]; then
  outside_proof_command="$(python3 scripts/generate-gate2-outside-proof.py --stopwatch-proof "$proof_path" --print-command)"
  outside_runner_next_steps="$(cat <<EOF

Outside-run next steps:
  1. If you have not already done so, open $quickstart_list_url in a normal browser for the quickstart trace list.
  2. Click the quickstart trace, then click the llm.call span.
  3. Confirm prompt, completion, model, token breakdown, cost, latency, and the confirmation code are visible.
  4. Open ${all_kind_dashboard_url:-not requested} in a normal browser for the all-kind waterfall.
  5. Confirm run -> turn -> step -> tool -> MCP nesting is visible.
  6. Use the saved docker compose logs artifact as evidence: $compose_logs_artifact
  7. Generate the completed proof from this prefilled command:
$outside_proof_command
  8. Commit the evidence before closure validation:
       git add docs/demos/gate2-outside-person-proof.md docs/demos/gate2-compose-stopwatch.md docs/demos/gate2-compose-browser-demo.webm docs/demos/gate2-compose-browser-demo.md docs/demos/gate2-outside-compose.log
       git commit -m "add gate2 outside proof"
  9. Validate it with scripts/validate-gate2-outside-proof.sh.
  10. After evidence is captured, clean up with:
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

Open this quickstart trace-list URL first:
  $quickstart_list_url

Direct quickstart trace URL:
  $dashboard_url

All-kind waterfall dashboard:
  ${all_kind_dashboard_url:-not requested}

Browser recording:
  $record_demo_status

Browser recording artifact:
  $([[ "$record_demo" == "1" ]] && echo "$record_demo_video" || echo "not requested")

Compose logs artifact:
  $compose_logs_artifact

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
