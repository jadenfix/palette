#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bare-metal-pr-helper.sh --title <title> [options]

Create a bare-metal workflow PR with the mandatory reviewer handoff metadata in-body.

Options:
  --title <title>            PR title.
  --base <branch>            PR base branch (default: main).
  --head <branch>            Source branch/remote-ref for gh create.
  --repo <owner/repo>        Optional gh repository override.
  --reviewer <handle>        Required reviewed-by github handle.
  --plan-file <path>         Optional file describing the implemented plan.
  --dry-run                  Render body only, do not create PR.
  --draft                    Create PR as draft.
  --help
EOF
}

TITLE=""
BASE="main"
HEAD=""
REPO=""
REVIEWER=""
PLAN_FILE=""
DRY_RUN=0
DRAFT=0

while (($#)); do
  case "$1" in
    --title)
      if (($# < 2)); then
        echo "--title requires a value" >&2
        usage
        exit 1
      fi
      TITLE="$2"
      shift
      ;;
    --base)
      if (($# < 2)); then
        echo "--base requires a value" >&2
        usage
        exit 1
      fi
      BASE="$2"
      shift
      ;;
    --head)
      if (($# < 2)); then
        echo "--head requires a value" >&2
        usage
        exit 1
      fi
      HEAD="$2"
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
    --reviewer)
      if (($# < 2)); then
        echo "--reviewer requires a value" >&2
        usage
        exit 1
      fi
      REVIEWER="$2"
      shift
      ;;
    --plan-file)
      if (($# < 2)); then
        echo "--plan-file requires a value" >&2
        usage
        exit 1
      fi
      PLAN_FILE="$2"
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      ;;
    --draft)
      DRAFT=1
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

if [[ -z "$TITLE" ]]; then
  echo "--title is required" >&2
  usage
  exit 1
fi

if [[ -z "$REVIEWER" ]]; then
  echo "--reviewer is required (review ownership)" >&2
  usage
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required" >&2
  exit 1
fi

tmp_file="$(mktemp)"
trap 'rm -f "$tmp_file"' EXIT

plan_contents=""
if [[ -n "$PLAN_FILE" ]]; then
  if [[ ! -f "$PLAN_FILE" ]]; then
    echo "missing plan file: $PLAN_FILE" >&2
    exit 1
  fi
  plan_contents="\n\n## Plan reference\n$(cat "$PLAN_FILE")"
fi

changed="$(git status --short | awk '{print "- " $2}' || true)"

cat >"$tmp_file" <<EOF
## 1.) agent
- bare-metal implementation changes (automation + validation lanes)

## 2.) purpose
- implement and harden bare-metal lane orchestration, readiness checks, optimization hints, and e2e evidence pipelines.

## 3.) reviewed-by
- @${REVIEWER}

## Changes
${changed:-"- no tracked working tree changes detected"}

## Evidence
- bare-metal scripts and workflows updated for concurrency, policy validation, and e2e log assertions.

## Merge requirements
- Independent reviewer approval from @$REVIEWER is required.
- PR should include per-run artifacts and readiness summary checks.
${plan_contents}
EOF

if [[ "$DRY_RUN" -eq 1 ]]; then
  cat "$tmp_file"
  exit 0
fi

create_args=(pr create --title "$TITLE" --body-file "$tmp_file" --base "$BASE")
if [[ -n "$HEAD" ]]; then
  create_args+=(--head "$HEAD")
fi
if [[ -n "$REPO" ]]; then
  create_args+=(--repo "$REPO")
fi
create_args+=(--reviewer "$REVIEWER")
if [[ "$DRAFT" -eq 1 ]]; then
  create_args+=(--draft)
fi

pr_url=$(gh "${create_args[@]}" --json url -q .url)
echo "created pr: $pr_url"
