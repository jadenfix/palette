#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
data_dir="${BEATER_GATE2_DATA_DIR:-/tmp/beater-gate2-proof}"
http_addr="${BEATER_GATE2_HTTP_ADDR:-127.0.0.1:8080}"
grpc_addr="${BEATER_GATE2_GRPC_ADDR:-127.0.0.1:4317}"
dashboard_host="${BEATER_GATE2_DASHBOARD_HOST:-127.0.0.1}"
dashboard_port="${BEATER_GATE2_DASHBOARD_PORT:-3000}"
api_url="http://$http_addr"
grpc_url="http://$grpc_addr"
dashboard_url="http://$dashboard_host:$dashboard_port"
venv_dir="${BEATER_GATE2_VENV:-/tmp/beater-gate2-otel-venv}"
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

server_pid=""
dashboard_pid=""

cleanup() {
  if [[ -n "$dashboard_pid" ]]; then kill "$dashboard_pid" >/dev/null 2>&1 || true; fi
  if [[ -n "$server_pid" ]]; then kill "$server_pid" >/dev/null 2>&1 || true; fi
}

wait_url() {
  local url="$1"
  local label="$2"
  local deadline=$((SECONDS + 120))
  until curl -fsS "$url" >/dev/null 2>&1; do
    if (( SECONDS >= deadline )); then
      echo "Timed out waiting for $label at $url" >&2
      return 1
    fi
    sleep 1
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
    sleep 1
  done
}

json_field() {
  python3 -c 'import json,sys; print(json.load(sys.stdin)[sys.argv[1]])' "$1"
}

first_trace_id() {
  python3 -c 'import json,sys; print(json.load(sys.stdin)["items"][0]["trace_id"])'
}

trap cleanup EXIT

rm -rf "$data_dir"

(
  cd "$root/web/dashboard"
  npm ci
)

"$root/scripts/check-openapi-drift.sh"
cargo build -q -p beaterd -p beaterctl

"$root/target/debug/beaterd" \
  --data-dir "$data_dir" \
  --addr "$http_addr" \
  --otlp-grpc-addr "$grpc_addr" \
  --trace-write-drain-interval-ms 25 \
  --trace-ingested-drain-interval-ms 25 &
server_pid="$!"
wait_url "$api_url/health" "beaterd"

http_smoke="$("$root/target/debug/beaterctl" smoke \
  --http-url "$api_url" \
  --tenant-id demo \
  --project-id demo \
  --environment-id local \
  --timeout-ms 10000)"

grpc_smoke="$("$root/target/debug/beaterctl" smoke \
  --http-url "$api_url" \
  --otlp-grpc-url "$grpc_url" \
  --tenant-id demo \
  --project-id demo \
  --environment-id local \
  --timeout-ms 10000)"

trace_id="$(printf '%s' "$grpc_smoke" | json_field trace_id)"

python3 -m venv "$venv_dir"
"$venv_dir/bin/pip" install --quiet --upgrade pip
"$venv_dir/bin/pip" install --quiet opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
OTEL_EXPORTER_OTLP_ENDPOINT="$grpc_url" \
BEATER_TENANT_ID=demo \
BEATER_PROJECT_ID=demo \
BEATER_ENVIRONMENT_ID=local \
  "$venv_dir/bin/python" "$root/examples/python/otel_smoke.py"

python_trace_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-demo&release=compose-demo"
wait_text "$python_trace_query" "gpt-demo" "stock Python OTLP trace"
python_trace_id="$(curl -fsS "$python_trace_query" | first_trace_id)"

OTEL_EXPORTER_OTLP_ENDPOINT="$grpc_url" \
  "$venv_dir/bin/python" "$root/examples/python/five_line_otel.py"

quickstart_trace_query="$api_url/v1/traces/demo?project_id=demo&environment_id=local&kind=llm.call&model=gpt-quickstart"
wait_text "$quickstart_trace_query" "gpt-quickstart" "five-line stock Python OTLP trace"
quickstart_trace_id="$(curl -fsS "$quickstart_trace_query" | first_trace_id)"

(
  cd "$root/web/dashboard"
  npm run build
  rm -rf .next/standalone/.next/static
  mkdir -p .next/standalone/.next
  cp -R .next/static .next/standalone/.next/static
)

(
  cd "$root/web/dashboard"
  BEATER_API_BASE_URL="$api_url" \
    NEXT_PUBLIC_BEATER_API_BASE_URL="$api_url" \
    HOSTNAME="$dashboard_host" \
    PORT="$dashboard_port" \
    node .next/standalone/server.js
) &
dashboard_pid="$!"

wait_url "$dashboard_url/?tenant=demo&project=demo&environment=local" "dashboard"

require_text "$api_url/v1/traces/demo/$trace_id" "beaterctl otlp smoke"
require_text "$api_url/openapi.json" "min_cost_micros"
python_trace_api="$api_url/v1/traces/demo/$python_trace_id"
python_trace_dashboard="$dashboard_url/?tenant=demo&project=demo&environment=local&trace=$python_trace_id"
for kind in "${all_kinds[@]}"; do
  wait_text "$python_trace_api" "$kind" "Python all-kind API trace"
done
wait_text "$python_trace_dashboard" "Agent Trace Debugger" "dashboard"
wait_text "$python_trace_dashboard" "call-policy-model" "dashboard llm.call row"
for kind in "${all_kinds[@]}"; do
  wait_text "$python_trace_dashboard" "$kind" "dashboard all-kind waterfall"
done

if [[ "${BEATER_GATE2_SKIP_BROWSER:-0}" != "1" ]]; then
  (
    cd "$root/web/dashboard"
    if [[ "${BEATER_GATE2_SKIP_PLAYWRIGHT_INSTALL:-0}" != "1" ]]; then
      npx playwright install chromium
    fi
    BEATER_E2E_TRACE_ID="$python_trace_id" PLAYWRIGHT_BASE_URL="$dashboard_url" \
      npx playwright test tests/e2e/dashboard.spec.ts
    BEATER_E2E_QUICKSTART_TRACE_ID="$quickstart_trace_id" PLAYWRIGHT_BASE_URL="$dashboard_url" \
      npm run test:e2e:quickstart
    if [[ "${BEATER_GATE2_RECORD_DEMO:-0}" == "1" ]]; then
      BEATER_E2E_TRACE_ID="$python_trace_id" PLAYWRIGHT_BASE_URL="$dashboard_url" npm run record:gate2
    fi
  )
fi

cat <<EOF
Gate 2 local proof passed.

HTTP smoke:
$http_smoke

gRPC smoke:
$grpc_smoke

Open the dashboard:
  $dashboard_url/?tenant=demo&project=demo&environment=local

Specific smoke trace:
  $dashboard_url/?tenant=demo&project=demo&environment=local&trace=$trace_id

Python all-kind trace:
  $python_trace_dashboard

Five-line quickstart trace:
  $dashboard_url/?tenant=demo&project=demo&environment=local&trace=$quickstart_trace_id
EOF
