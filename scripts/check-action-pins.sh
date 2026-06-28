#!/usr/bin/env bash
# Fail if any third-party GitHub Action is referenced by a mutable ref instead of
# an immutable 40-char commit SHA. A moving tag/branch (@master, @main, @stable,
# @v4, ...) lets an upstream tag move or a repo compromise change code that runs
# inside CI with deploy/publish credentials. See issue #135.
#
# Policy:
#   - Third-party actions (owner != "actions", the GitHub-owned org) MUST pin to
#     a full 40-hex commit SHA. Keep a trailing "# <version>" comment for humans.
#   - First-party `actions/*` and local `./path` actions are allowed by tag.
#   - Bump pinned SHAs intentionally via .github/dependabot.yml.
set -euo pipefail

default_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="${1:-${BEATER_ACTION_PINS_ROOT:-$default_root}}"
violations=0

check_uses_ref() {
  local file="$1"
  local lineno="$2"
  local text="$3"
  local ref pin

  # The action reference: strip everything up to and including "uses:", trim,
  # and drop any trailing "# comment".
  ref="$(printf '%s' "$text" | sed -E 's/.*uses:[[:space:]]*//; s/[[:space:]]*#.*$//; s/[[:space:]]*$//')"
  ref="${ref//$'\r'/}"
  ref="${ref//\"/}"
  ref="${ref//\'/}"

  case "$ref" in
    ./*|../*|.github/*) return 0 ;;      # local action / reusable workflow
    actions/*) return 0 ;;              # GitHub first-party org (trusted)
    docker://*) return 0 ;;             # container action image refs are out of scope
    *@*) : ;;                           # owner/repo[@path]@ref — check below
    *) return 0 ;;                      # no @ref — out of scope
  esac

  pin="${ref##*@}"
  if [[ "$pin" =~ ^[0-9a-fA-F]{40}$ ]]; then
    return 0                            # immutable SHA — OK
  fi

  printf 'MUTABLE ACTION REF  %s:%s\n    %s\n' "$file" "$lineno" "$ref" >&2
  violations=$((violations + 1))
}

scan_file() {
  local file="$1"
  local match lineno text
  while IFS= read -r match; do
    lineno="${match%%:*}"
    text="${match#*:}"
    check_uses_ref "$file" "$lineno" "$text"
  done < <(grep -nE '^[[:space:]]*-?[[:space:]]*uses:' "$file" || true)
}

scan_dir() {
  local dir="$1"
  [[ -d "$dir" ]] || return 0
  while IFS= read -r -d '' file; do
    scan_file "$file"
  done < <(
    find "$dir" -type f \( -name '*.yml' -o -name '*.yaml' \) -print0
  )
}

scan_dir "$root/.github/workflows"
scan_dir "$root/.github/actions"

if [[ "$violations" -gt 0 ]]; then
  cat >&2 <<EOF

Found $violations third-party action(s) using a mutable ref.
Pin each to a full commit SHA, e.g.:
    uses: docker/build-push-action@f9f3042f7e2789586610d6e8b85c8f03e5195baf # v7
Resolve a tag's SHA with:
    gh api repos/<owner>/<repo>/commits/<tag> --jq .sha
EOF
  exit 1
fi

echo "All third-party GitHub Actions are pinned to immutable commit SHAs."
