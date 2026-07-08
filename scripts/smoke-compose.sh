#!/usr/bin/env bash
set -euo pipefail

project="${COMPOSE_PROJECT_NAME:-beater-smoke}"
keep="${KEEP_BEATER_COMPOSE:-0}"
host_http_port="${BEATER_HTTP_PORT:-8080}"
host_dashboard_port="${BEATER_DASHBOARD_PORT:-3000}"
api_url="http://127.0.0.1:$host_http_port"
dashboard_url="http://127.0.0.1:$host_dashboard_port"
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

compose() {
  docker compose -p "$project" "$@"
}

cleanup() {
  if [[ "$keep" != "1" ]]; then
    compose down -v --remove-orphans >/dev/null 2>&1 || true
  fi
}

diagnose_failure() {
  echo "Beater compose smoke failed; capturing compose status and logs before cleanup." >&2
  compose ps >&2 || true
  compose logs --no-color --timestamps >&2 || true
}

on_exit() {
  local status=$?
  if (( status != 0 )); then
    diagnose_failure
  fi
  cleanup
  exit "$status"
}

wait_url() {
  local url="$1"
  local label="$2"
  local deadline=$((SECONDS + 120))
  until curl -fsS "$url" >/tmp/beater-smoke-response 2>/tmp/beater-smoke-error; do
    if (( SECONDS >= deadline )); then
      echo "Timed out waiting for $label at $url" >&2
      cat /tmp/beater-smoke-error >&2 || true
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
  local deadline=$((SECONDS + 120))
  until require_text "$url" "$needle" >/dev/null 2>&1; do
    if (( SECONDS >= deadline )); then
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

trap on_exit EXIT

compose up -d --build beaterd dashboard
wait_url "$api_url/health" "beaterd"

compose run --rm beaterctl
compose run --rm otel-python-smoke

wait_url "$dashboard_url/?tenant=demo&project=demo&environment=local" "dashboard"
python_trace_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-demo&release=compose-demo"
wait_text "$python_trace_query" "gpt-demo" "stock Python OTLP trace"
python_trace_id="$(curl -fsS "$python_trace_query" | first_trace_id)"
python_trace_dashboard="$dashboard_url/?tenant=demo&project=demo&environment=local&trace=$python_trace_id"
require_text "$python_trace_dashboard" "Agent Trace Debugger"
wait_text "$python_trace_dashboard" "call-policy-model" "dashboard llm.call row"
for kind in "${all_kinds[@]}"; do
  wait_text "$python_trace_dashboard" "$kind" "dashboard all-kind waterfall"
done
require_text "$api_url/openapi.json" "started_after"

cat <<EOF
Beater compose smoke passed.

Open the dashboard:
  $python_trace_dashboard

Set KEEP_BEATER_COMPOSE=1 to leave containers running after this script exits.
EOF
