#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bare-metal-lane-fleet.sh [options]

Parallel lane orchestration wrapper around bare-metal-dispatch.sh.

Options:
  --target <name>        Dispatch a specific target by name (repeatable).
  --all                  Dispatch all policy targets (default).
  --target-index <n>     Dispatch a target by policy index (repeatable).
  --policy <path>        Path to lane policy JSON.
  --ref <ref>            Pass workflow ref override to dispatch.
  --repo <owner/repo>    Pass gh repo override to dispatch.
  --workers <n>          Maximum concurrent dispatch jobs (default: 2).
  --dry-run              Print commands, do not dispatch.
  --require-git-auth      Enforce gh auth checks before dispatch.
  --validate-only         Validate policy only and exit.
  --help

Examples:
  bash scripts/bare-metal-lane-fleet.sh --all --workers 3 --dry-run
  bash scripts/bare-metal-lane-fleet.sh --target cuda --target cuda-pro --workers 1
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
POLICY_PATH="${REPO_ROOT}/docs/engineering/bare-metal-lane-policy.json"
DISPATCH_SCRIPT="${SCRIPT_DIR}/bare-metal-dispatch.sh"
WORKERS=2

declare -a TARGET_FILTERS
declare -a INDEX_FILTERS
REF=""
REPO=""
DRY_RUN=0
VALIDATE_ONLY=0
REQUIRE_GIT_AUTH=0

while (($#)); do
  case "$1" in
    --target)
      if (($# < 2)); then
        echo "--target requires a value" >&2
        usage
        exit 1
      fi
      TARGET_FILTERS+=("$2")
      shift
      ;;
    --target-index)
      if (($# < 2)); then
        echo "--target-index requires a value" >&2
        usage
        exit 1
      fi
      INDEX_FILTERS+=("$2")
      shift
      ;;
    --all)
      TARGET_FILTERS=()
      INDEX_FILTERS=()
      ;;
    --policy)
      if (($# < 2)); then
        echo "--policy requires a value" >&2
        usage
        exit 1
      fi
      POLICY_PATH="$2"
      shift
      ;;
    --ref)
      if (($# < 2)); then
        echo "--ref requires a value" >&2
        usage
        exit 1
      fi
      REF="$2"
      shift
      ;;
    --repo)
      if (($# < 2)); then
        echo "--repo requires a value" >&2
        usage
        exit 1
      fi
      REPO="$2"
      shift
      ;;
    --workers)
      if (($# < 2)); then
        echo "--workers requires a value" >&2
        usage
        exit 1
      fi
      WORKERS="$2"
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    --validate-only)
      VALIDATE_ONLY=1
      ;;
    --require-git-auth)
      REQUIRE_GIT_AUTH=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
  shift
done

if [[ "$WORKERS" -lt 1 ]]; then
  echo "--workers must be >= 1" >&2
  exit 1
fi
if [[ ! -f "$POLICY_PATH" ]]; then
  echo "missing policy file: $POLICY_PATH" >&2
  exit 1
fi
if [[ ! -f "$DISPATCH_SCRIPT" ]]; then
  echo "missing dispatch script: $DISPATCH_SCRIPT" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required" >&2
  exit 1
fi

if ! command -v bash >/dev/null 2>&1; then
  echo "bash required" >&2
  exit 1
fi

discover_targets() {
  local -a all_names=()
  local -a selected_targets=()
  local -A seen

  while IFS= read -r target; do
    all_names+=("$target")
  done < <(
    python3 - "$POLICY_PATH" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    data = json.load(handle)

for target in data.get("targets", []):
    name = str(target.get("name", "")).strip()
    if name:
        print(name)
PY
  )

  if [[ ${#INDEX_FILTERS[@]} -gt 0 ]]; then
    local index
    for index in "${INDEX_FILTERS[@]}"; do
      if ! [[ "$index" =~ ^[0-9]+$ ]]; then
        echo "invalid target-index: $index" >&2
        exit 1
      fi
      if (( index < 0 || index >= ${#all_names[@]} )); then
        echo "target-index $index out of range (0..$(( ${#all_names[@]} - 1 )))" >&2
        exit 1
      fi
      selected_targets+=("${all_names[index]}")
    done
  fi

  if [[ ${#TARGET_FILTERS[@]} -eq 0 && ${#selected_targets[@]} -eq 0 ]]; then
    selected_targets=("${all_names[@]}")
  fi

  if [[ ${#TARGET_FILTERS[@]} -gt 0 ]]; then
    selected_targets+=("${TARGET_FILTERS[@]}")
  fi

  for target in "${selected_targets[@]}"; do
    if [[ -z "${seen[$target]+x}" ]]; then
      seen[$target]=1
      TARGET_FILTERS+=("$target")
    fi
  done
}

discover_targets
if [[ ${#TARGET_FILTERS[@]} -eq 0 && ${#INDEX_FILTERS[@]} -eq 0 ]]; then
  echo "no lane targets selected" >&2
  exit 1
fi

if [[ "$VALIDATE_ONLY" -eq 1 ]]; then
  python3 "$DISPATCH_SCRIPT" --policy "$POLICY_PATH" --validate-only
  exit 0
fi

dispatch_one() {
  local target="$1"
  local log_dir="$2"
  local idx="$3"
  local safe_target
  local log_file

  safe_target="$(printf '%s' "$target" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9._-' '-')"
  log_file="${log_dir}/dispatch-${idx}-${safe_target}.log"

  local args=(
    "$DISPATCH_SCRIPT"
    --policy "$POLICY_PATH"
    --target "$target"
  )
  if [[ -n "$REF" ]]; then
    args+=(--ref "$REF")
  fi
  if [[ -n "$REPO" ]]; then
    args+=(--repo "$REPO")
  fi
  if [[ "$REQUIRE_GIT_AUTH" -eq 1 ]]; then
    args+=(--require-git-auth)
  fi
  if [[ "$DRY_RUN" -eq 1 ]]; then
    args+=(--dry-run)
  fi

  (
    set -o pipefail
    {
      if [[ "$DRY_RUN" -eq 1 ]]; then
        echo "[dry-run] ${args[*]}"
      else
        echo "[dispatch] ${args[*]}"
      fi
      python3 "${args[@]}"
    } >"$log_file" 2>&1
    status=$?
    echo "${status}" >"${log_file}.status"
  )
}

tmp_root="$(mktemp -d)"
log_dir="${tmp_root}/dispatch-logs"
mkdir -p "$log_dir"
declare -a pids
idx=0

for target in "${TARGET_FILTERS[@]}"; do
  while (( ${#pids[@]} >= WORKERS )); do
    running_pid="${pids[0]}"
    if wait "${running_pid}"; then
      true
    fi
    pids=("${pids[@]:1}")
  done

  dispatch_one "$target" "$log_dir" "$idx" &
  pid=$!
  pids+=("$pid")
  idx=$((idx + 1))
done

# Wait for all in-flight workers.
for pid in "${pids[@]}"; do
  if ! wait "$pid"; then
    failed=1
  fi
done

failed=0
for status_file in "${log_dir}"/*.status; do
  if [[ ! -f "$status_file" ]]; then
    continue
  fi
  status="$(cat "$status_file")"
  if [[ "$status" != "0" ]]; then
    failed=1
  fi
done

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "dry-run complete; command traces logged under: $log_dir"
else
  echo "dispatch complete; status logs in: $log_dir"
fi

if [[ "$failed" -ne 0 ]]; then
  echo "one or more dispatch jobs failed; inspect dispatch logs for details." >&2
  for status_file in "${log_dir}"/*.status; do
    [[ -f "$status_file" ]] || continue
    if [[ "$(cat "$status_file")" != "0" ]]; then
      echo "FAILED: $status_file" >&2
      tail -n 30 "${status_file%.status}" >&2
    fi
  done
  exit 1
fi
