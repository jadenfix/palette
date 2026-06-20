#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
dry_run="${BEATER_GATE2_OUTSIDE_RUN_DRY_RUN:-0}"

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

if [[ $# -ne 0 ]]; then
  fail "this wrapper takes no arguments"
fi

require_unset_or_value BEATER_GATE2_REUSE 0 "warm-loop reuse is not valid evidence"
require_unset_or_value BEATER_GATE2_LOCAL_BUILD 0 "the outside run must use prebuilt SHA-pinned images"
require_unset_or_value BEATER_GATE2_PULL_POLICY always "the outside run must pull current images"
require_unset_or_value BEATER_HTTP_PORT 8080 "the default API port is required"
require_unset_or_value BEATER_OTLP_GRPC_PORT 4317 "the default OTLP gRPC port is required"
require_unset_or_value BEATER_DASHBOARD_PORT 3000 "the default dashboard port is required"
require_unset_or_value BEATER_GATE2_WRITE_PROOF 1 "the outside run must write a stopwatch proof"
require_unset_or_value BEATER_GATE2_BROWSER_PROOF 1 "the outside run must prove the browser flow"
require_unset_or_value BEATER_GATE2_RECORD_DEMO 1 "the outside run must record the browser flow"

export BEATER_GATE2_WRITE_PROOF=1
export BEATER_GATE2_BROWSER_PROOF=1
export BEATER_GATE2_RECORD_DEMO=1
export BEATER_GATE2_REUSE="${BEATER_GATE2_REUSE:-0}"
export BEATER_GATE2_LOCAL_BUILD="${BEATER_GATE2_LOCAL_BUILD:-0}"
export BEATER_GATE2_PULL_POLICY="${BEATER_GATE2_PULL_POLICY:-always}"

cd "$repo_root"

if [[ "$dry_run" == "1" ]]; then
  cat <<EOF
Gate 2 outside-run wrapper preflight passed.
Would execute:
  BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
EOF
  exit 0
fi

exec scripts/gate2-compose-stopwatch.sh
