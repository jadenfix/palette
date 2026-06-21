#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
dry_run="${BEATER_GATE2_OUTSIDE_RUN_DRY_RUN:-0}"
expected_origin="https://github.com/jadenfix/beater.git"
if [[ "$dry_run" == "1" && -n "${BEATER_GATE2_EXPECTED_ORIGIN:-}" ]]; then
  expected_origin="$BEATER_GATE2_EXPECTED_ORIGIN"
fi

fail() {
  echo "Gate 2 outside-run preflight failed: $*" >&2
  exit 1
}

require_unset_or_value() {
  local name="$1"
  local expected="$2"
  local reason="$3"
  local value="${!name:-}"
  if [[ -n "$value" && "$value" != "$expected" ]]; then
    fail "$name must be unset or '$expected' for outside-person evidence ($reason); got '$value'"
  fi
}

require_unset() {
  local name="$1"
  local reason="$2"
  local value="${!name:-}"
  if [[ -n "$value" ]]; then
    fail "$name must be unset for outside-person evidence ($reason); got '$value'"
  fi
}

require_clone_timer() {
  local value="${BEATER_GATE2_CLONE_STARTED_EPOCH:-}"
  if [[ "$dry_run" == "1" ]]; then
    return 0
  fi
  if [[ -z "$value" ]]; then
    fail "BEATER_GATE2_CLONE_STARTED_EPOCH must be set before git clone; use the documented clone-to-browser command"
  fi
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "BEATER_GATE2_CLONE_STARTED_EPOCH must be a Unix epoch second value"
  fi
}

require_command() {
  local name="$1"
  local reason="$2"
  if ! command -v "$name" >/dev/null 2>&1; then
    fail "missing required command '$name' ($reason)"
  fi
}

require_git_provenance() {
  local branch
  local origin
  local dirty
  branch="$(git -C "$repo_root" branch --show-current 2>/dev/null || true)"
  if [[ "$branch" != "main" ]]; then
    fail "outside-person evidence must run from the main branch; got '${branch:-detached}'"
  fi
  origin="$(git -C "$repo_root" remote get-url origin 2>/dev/null || true)"
  if [[ "$origin" != "$expected_origin" ]]; then
    fail "outside-person evidence must run from origin '$expected_origin'; got '${origin:-missing}'"
  fi
  dirty="$(git -C "$repo_root" status --porcelain 2>/dev/null || true)"
  if [[ -n "$dirty" ]]; then
    fail "outside-person evidence must run from a clean worktree"
  fi
}

if [[ $# -ne 0 ]]; then
  fail "this wrapper takes no arguments"
fi

require_git_provenance
require_clone_timer
require_unset_or_value BEATER_GATE2_REUSE 0 "warm-loop reuse is not valid evidence"
require_unset_or_value BEATER_GATE2_LOCAL_BUILD 0 "the outside run must use prebuilt SHA-pinned images"
require_unset_or_value BEATER_GATE2_PULL_POLICY always "the outside run must pull current images"
require_unset_or_value BEATER_HTTP_PORT 8080 "the default API port is required"
require_unset_or_value BEATER_OTLP_GRPC_PORT 4317 "the default OTLP gRPC port is required"
require_unset_or_value BEATER_DASHBOARD_PORT 3000 "the default dashboard port is required"
require_unset_or_value BEATER_GATE2_WRITE_PROOF 1 "the outside run must write a stopwatch proof"
require_unset_or_value BEATER_GATE2_BROWSER_PROOF 1 "the outside run must prove the browser flow"
require_unset_or_value BEATER_GATE2_RECORD_DEMO 1 "the outside run must record the browser flow"
require_unset_or_value BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS 300 "the outside run must use the documented post-SLO timeout"
require_unset_or_value KEEP_BEATER_COMPOSE 1 "the dashboard must remain running for outside-person click-through"
require_unset BEATERD_IMAGE "the wrapper pins beaterd to the checked-out commit SHA"
require_unset BEATER_DASHBOARD_IMAGE "the wrapper pins dashboard to the checked-out commit SHA"
require_unset BEATER_DASHBOARD_E2E_IMAGE "the wrapper pins dashboard-e2e to the checked-out commit SHA"
require_unset BEATER_OTEL_PYTHON_IMAGE "the wrapper pins otel-python to the checked-out commit SHA"
require_unset BEATER_GATE2_STOPWATCH_PROOF "the outside run must write docs/demos/gate2-compose-stopwatch.md"
require_unset BEATER_GATE2_RECORD_VIDEO "the outside run must write docs/demos/gate2-compose-browser-demo.webm"
require_unset BEATER_GATE2_RECORD_NOTES "the outside run must write docs/demos/gate2-compose-browser-demo.md"
require_unset COMPOSE_PROJECT_NAME "the outside run must use the default beater-stopwatch Compose project"

export BEATER_GATE2_WRITE_PROOF=1
export BEATER_GATE2_BROWSER_PROOF=1
export BEATER_GATE2_RECORD_DEMO=1
export BEATER_GATE2_REUSE="${BEATER_GATE2_REUSE:-0}"
export BEATER_GATE2_LOCAL_BUILD="${BEATER_GATE2_LOCAL_BUILD:-0}"
export BEATER_GATE2_PULL_POLICY="${BEATER_GATE2_PULL_POLICY:-always}"
export BEATER_GATE2_OUTSIDE_WRAPPER=1
export KEEP_BEATER_COMPOSE=1

cd "$repo_root"

if [[ "$dry_run" == "1" ]]; then
  cat <<EOF
Gate 2 outside-run wrapper preflight passed.
Would execute:
  BEATER_GATE2_CLONE_STARTED_EPOCH="\$BEATER_GATE2_CLONE_STARTED_EPOCH" BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
EOF
  exit 0
fi

require_command python3 "post-run proof generation and validation require python3"

exec scripts/gate2-compose-stopwatch.sh
