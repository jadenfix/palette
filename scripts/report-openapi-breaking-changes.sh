#!/usr/bin/env bash
# Compare sdks/openapi/beater-api.json against the origin/main version and
# report potential breaking changes: removed paths, removed operations (by
# operationId), and removed schema properties.
#
# Usage:
#   scripts/report-openapi-breaking-changes.sh [--fail-on-breaking]
#   scripts/report-openapi-breaking-changes.sh --self-test
#
# By default the script is a reporter only: it prints findings and exits 0.
# Pass --fail-on-breaking to exit 1 when any breakage is found (useful for CI).
# Pass --self-test to validate the reporter against the bundled fixtures.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

step() { printf '\n==> %s\n' "$1"; }

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------
if ! command -v jq >/dev/null 2>&1; then
  printf 'ERROR: jq is required but not found in PATH\n' >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Flags
# ---------------------------------------------------------------------------
FAIL_ON_BREAKING=0
SELF_TEST=0
for arg in "$@"; do
  case "$arg" in
    --fail-on-breaking) FAIL_ON_BREAKING=1 ;;
    --self-test)        SELF_TEST=1 ;;
    *) printf 'ERROR: unknown argument: %s\n' "$arg" >&2; exit 1 ;;
  esac
done

# ---------------------------------------------------------------------------
# Comparison engine
#
# find_breakages BASE HEAD
#   Prints one BREAKING line per finding to stdout.
#   Appends each finding to BREAKING_LINES[] and increments BREAKING_COUNT.
#   Called directly (never in a subshell) so those globals are visible to
#   the caller.
# ---------------------------------------------------------------------------
BREAKING_COUNT=0
BREAKING_LINES=()

_emit() {
  printf '%s\n' "$1"
  BREAKING_LINES+=("$1")
  BREAKING_COUNT=$((BREAKING_COUNT + 1))
}

find_breakages() {
  local base="$1" head="$2"

  # 1. Removed top-level paths
  step "Checking for removed paths"
  while IFS= read -r path; do
    if ! jq -e --arg p "$path" '.paths | has($p)' "$head" >/dev/null 2>&1; then
      _emit "  BREAKING [removed-path]      $path"
    fi
  done < <(jq -r '.paths | keys[]' "$base")

  # 2. Removed operations (any operationId missing from head)
  step "Checking for removed operations (operationId)"
  while IFS= read -r opid; do
    local found
    found=$(jq -r --arg id "$opid" \
      '[.paths[][] | objects | select(.operationId == $id)] | length' \
      "$head" 2>/dev/null) || found=0
    if [[ "$found" -eq 0 ]]; then
      _emit "  BREAKING [removed-operation]  operationId=$opid"
    fi
  done < <(jq -r \
    '[.paths[][] | objects | select(.operationId != null) | .operationId] | .[]' \
    "$base" 2>/dev/null || true)

  # 3. Removed schema properties (components.schemas[*].properties[*])
  step "Checking for removed schema properties"
  while IFS=$'\t' read -r schema field; do
    if ! jq -e --arg s "$schema" --arg f "$field" \
        '(.components.schemas[$s].properties // {}) | has($f)' \
        "$head" >/dev/null 2>&1; then
      _emit "  BREAKING [removed-field]      $schema.$field"
    fi
  done < <(jq -r '
    .components.schemas // {} |
    to_entries[] |
    .key as $schema |
    (.value.properties // {}) |
    keys[] |
    [$schema, .] | @tsv
  ' "$base" 2>/dev/null || true)
}

# ---------------------------------------------------------------------------
# Self-test: run find_breakages on the bundled fixtures and assert it detects
# all three planted breakage types (removed-path, removed-operation,
# removed-field).
# ---------------------------------------------------------------------------
if [[ "$SELF_TEST" -eq 1 ]]; then
  step "Self-test: comparing fixtures"
  base_f="$root/scripts/fixtures/openapi-breaking/base.json"
  head_f="$root/scripts/fixtures/openapi-breaking/head.json"

  for f in "$base_f" "$head_f"; do
    if [[ ! -f "$f" ]]; then
      printf 'ERROR: fixture missing: %s\n' "$f" >&2
      exit 1
    fi
  done

  find_breakages "$base_f" "$head_f"
  printf '\nDetected %d breaking change(s).\n' "$BREAKING_COUNT"

  fail=0
  _assert_finding() {
    local kind="$1"
    local found=0
    for line in "${BREAKING_LINES[@]+"${BREAKING_LINES[@]}"}"; do
      [[ "$line" == *"$kind"* ]] && found=1 && break
    done
    if [[ "$found" -eq 0 ]]; then
      printf 'SELF-TEST FAIL: no %s finding detected\n' "$kind" >&2
      fail=1
    fi
  }

  _assert_finding "removed-path"
  _assert_finding "removed-operation"
  _assert_finding "removed-field"

  if [[ "$BREAKING_COUNT" -lt 3 ]]; then
    printf 'SELF-TEST FAIL: expected >= 3 breakages, got %d\n' "$BREAKING_COUNT" >&2
    fail=1
  fi

  if [[ "$fail" -ne 0 ]]; then
    printf '\nSelf-test FAILED.\n' >&2
    exit 1
  fi
  printf '\nSelf-test PASSED: all expected breakages detected.\n'
  exit 0
fi

# ---------------------------------------------------------------------------
# Normal mode: compare HEAD spec against origin/main.
# ---------------------------------------------------------------------------
current="$root/sdks/openapi/beater-api.json"
if [[ ! -f "$current" ]]; then
  printf 'ERROR: spec not found: %s\n' "$current" >&2
  exit 1
fi

step "Fetching origin/main:sdks/openapi/beater-api.json"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

if ! git show origin/main:sdks/openapi/beater-api.json > "$tmpdir/base.json" 2>/dev/null; then
  printf 'WARN: could not read origin/main spec — is the remote fetched?\n' >&2
  printf 'Run: git fetch origin\n' >&2
  exit 0
fi

step "Comparing HEAD spec against origin/main"
find_breakages "$tmpdir/base.json" "$current"

printf '\n'
if [[ "$BREAKING_COUNT" -eq 0 ]]; then
  printf 'No breaking changes detected.\n'
else
  printf 'Summary: %d potential breaking change(s) detected.\n' "$BREAKING_COUNT"
  if [[ "$FAIL_ON_BREAKING" -eq 1 ]]; then
    printf 'Exiting non-zero (--fail-on-breaking is set).\n' >&2
    exit 1
  fi
fi
