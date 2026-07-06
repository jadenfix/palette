#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bare-metal-run-matrix.sh [--list] [--dry-run] [--json] [--target <name>] [--target-index <index>]

Options:
  --list                List available target names from the lane policy.
  --dry-run             Print workflow invocation payloads without running them.
  --json                Emit JSON payloads (instead of text output).
  --target-index <n>    Dispatch a single profile by policy index (0-based).
  --target <name>       Run only the named target profile.
  -h, --help            Show this message.
USAGE
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
POLICY_PATH="${REPO_ROOT}/docs/engineering/bare-metal-lane-policy.json"
DISPATCH_SCRIPT="${SCRIPT_DIR}/bare-metal-dispatch.sh"

list_only=0
dry_run=0
json_output=0
target_filter=""
target_index=""

if [[ "$#" -eq 0 ]]; then
  set -- --help
fi

while (($#)); do
  case "$1" in
    --list)
      list_only=1
      ;;
    --dry-run)
      dry_run=1
      ;;
    --json)
      json_output=1
      ;;
    --target)
      if (($# < 2)); then
        echo "--target requires a value" >&2
        usage
        exit 1
      fi
      target_filter="$2"
      shift
      ;;
    --target-index)
      if (($# < 2)); then
        echo "--target-index requires a value" >&2
        usage
        exit 1
      fi
      target_index="$2"
      shift
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

dispatch_args=(--policy "$POLICY_PATH")
if (( list_only == 1 )); then
  dispatch_args+=(--list)
fi
if (( dry_run == 1 )); then
  dispatch_args+=(--dry-run)
fi
if (( json_output == 1 )); then
  dispatch_args+=(--json)
fi
if [[ -n "$target_filter" ]]; then
  dispatch_args+=(--target "$target_filter")
fi
if [[ -n "$target_index" ]]; then
  dispatch_args+=(--target-index "$target_index")
fi

python3 "$DISPATCH_SCRIPT" "${dispatch_args[@]}" | while IFS='' read -r line; do
  echo "$line"
done
