#!/usr/bin/env bash
set -euo pipefail

project="${COMPOSE_PROJECT_NAME:-beater-stopwatch}"
keep="${KEEP_BEATER_COMPOSE:-1}"
write_proof="${BEATER_GATE2_WRITE_PROOF:-0}"
proof_path="${BEATER_GATE2_STOPWATCH_PROOF:-docs/demos/gate2-compose-stopwatch.md}"
venv_dir="${BEATER_GATE2_QUICKSTART_VENV:-/tmp/beater-gate2-compose-otel-venv}"
local_build="${BEATER_GATE2_LOCAL_BUILD:-0}"
host_http_port="${BEATER_HTTP_PORT:-8080}"
host_otlp_grpc_port="${BEATER_OTLP_GRPC_PORT:-4317}"
host_dashboard_port="${BEATER_DASHBOARD_PORT:-3000}"
api_url="http://127.0.0.1:$host_http_port"
otlp_url="http://127.0.0.1:$host_otlp_grpc_port"
dashboard_base_url="http://127.0.0.1:$host_dashboard_port"
start_epoch="$(date +%s)"
deadline_epoch=$((start_epoch + 300))
started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
if [[ "$local_build" == "1" ]]; then
  compose_files=(-f docker-compose.yml)
  startup_mode="local-build"
  startup_args=(up -d --build postgres nats minio beaterd dashboard)
else
  compose_files=(-f docker-compose.prebuilt.yml)
  startup_mode="prebuilt-image"
  startup_args=(up -d --pull missing postgres nats minio beaterd dashboard)
fi

compose() {
  docker compose "${compose_files[@]}" -p "$project" "$@"
}

cleanup() {
  if [[ "$keep" != "1" ]]; then
    compose down -v --remove-orphans >/dev/null 2>&1 || true
  fi
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
      echo "Timed out during $label after 300s" >&2
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
  python3 -c 'import json,sys; print(json.load(sys.stdin)["items"][0]["trace_id"])'
}

trap cleanup EXIT

run_before_deadline "compose startup ($startup_mode)" compose "${startup_args[@]}"
wait_url "$api_url/health" "beaterd"
wait_url "$dashboard_base_url/?tenant=demo&project=demo&environment=local" "dashboard"

run_before_deadline "Python virtualenv creation" python3 -m venv "$venv_dir"
run_before_deadline "pip upgrade" "$venv_dir/bin/pip" install --quiet --upgrade pip
run_before_deadline "OTEL package install" "$venv_dir/bin/pip" install --quiet opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
run_before_deadline "five-line OTEL snippet" env OTEL_EXPORTER_OTLP_ENDPOINT="$otlp_url" "$venv_dir/bin/python" examples/python/five_line_otel.py

quickstart_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-quickstart"
wait_text "$quickstart_query" "gpt-quickstart" "five-line OTEL trace"
trace_id="$(curl -fsS "$quickstart_query" | first_trace_id)"
dashboard_url="$dashboard_base_url/?tenant=demo&project=demo&environment=local&trace=$trace_id"

wait_text "$dashboard_url" "Agent Trace Debugger" "dashboard"
wait_text "$dashboard_url" "five-line-llm-call" "dashboard quickstart trace"
wait_text "$dashboard_url" "gpt-quickstart" "dashboard model detail"
wait_text "$dashboard_url" "hello from stock OpenTelemetry" "dashboard prompt detail"

ended_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
duration_seconds=$(($(date +%s) - start_epoch))
if (( duration_seconds > 300 )); then
  echo "Time-to-first-trace exceeded 300s: ${duration_seconds}s" >&2
  exit 1
fi

if [[ "$write_proof" == "1" ]]; then
  mkdir -p "$(dirname "$proof_path")"
  cat >"$proof_path" <<EOF
# Gate 2 Compose Stopwatch Proof

- Started: $started_at
- Ended: $ended_at
- Duration: ${duration_seconds}s
- Limit: 300s
- Startup mode: $startup_mode
- Compose project: $project
- Snippet: \`examples/python/five_line_otel.py\`
- OTLP endpoint: \`$otlp_url\`
- Trace: \`$trace_id\`
- Dashboard: $dashboard_url

This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

\`\`\`bash
BEATER_GATE2_WRITE_PROOF=1 scripts/gate2-compose-stopwatch.sh
\`\`\`
EOF
fi

cat <<EOF
Gate 2 compose stopwatch passed in ${duration_seconds}s.

Open the dashboard:
  $dashboard_url

Five-line snippet:
  examples/python/five_line_otel.py

OTLP endpoint:
  $otlp_url

Startup mode:
  $startup_mode

Set KEEP_BEATER_COMPOSE=0 to tear containers down automatically.
Set BEATER_GATE2_LOCAL_BUILD=1 to build images from local source instead of
pulling prebuilt GHCR images.
EOF
