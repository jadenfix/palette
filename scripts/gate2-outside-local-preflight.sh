#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "Gate 2 outside-run local preflight failed: $*" >&2
  exit 1
}

require_command() {
  local name="$1"
  local reason="$2"
  if ! command -v "$name" >/dev/null 2>&1; then
    fail "missing required command '$name' ($reason)"
  fi
}

require_python3() {
  require_command python3 "proof generation and validation require python3 3.9+"
  if ! python3 - <<'PY' >/dev/null 2>&1
import sys
raise SystemExit(0 if sys.version_info >= (3, 9) else 1)
PY
  then
    local version
    version="$(python3 -c 'import sys; print(".".join(str(part) for part in sys.version_info[:3]))' 2>/dev/null || true)"
    fail "python3 must be version 3.9 or newer; got '${version:-unknown}'"
  fi
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

port_is_free() {
  if (: >/dev/tcp/127.0.0.1/"$1") >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

print_port_owner() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >&2 || true
    print_port_owner_details "$port"
  elif command -v ss >/dev/null 2>&1; then
    ss -ltnp "sport = :$port" >&2 || true
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

require_command git "the timed run clones the public Beater repo"
require_command docker "the timed run starts the public Docker Compose topology"
require_command curl "the timed run checks the local Beater API"
require_command ffprobe "completed proof validation verifies the WebM recording"
require_python3

if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
  fail "missing required command 'shasum' or 'sha256sum' (recording hash proof)"
fi

if ! docker_endpoint_is_local "${DOCKER_HOST:-}"; then
  fail "DOCKER_HOST must point at a local Docker daemon because browser proof uses 127.0.0.1"
fi

docker info >/dev/null 2>&1 || fail "Docker daemon is not reachable; start Docker and rerun"
docker compose version >/dev/null 2>&1 || fail "Docker Compose v2 is required"

docker_context_host="$(docker context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null | head -n 1 || true)"
if ! docker_endpoint_is_local "$docker_context_host"; then
  fail "Docker context must be local because browser proof uses 127.0.0.1; current endpoint is $docker_context_host"
fi

for port in 8080 4317 3000; do
  if ! port_is_free "$port"; then
    echo "TCP $port is already in use before the timed Gate 2 run." >&2
    print_port_owner "$port"
    fail "free TCP $port before starting the stopwatch; do not use alternate ports for outside-person evidence"
  fi
done

echo "Gate 2 outside-run local preflight passed."
