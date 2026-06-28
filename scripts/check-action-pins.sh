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

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
violations=0

while IFS= read -r line; do
  file="${line%%:*}"
  rest="${line#*:}"
  lineno="${rest%%:*}"
  # The action reference: strip everything up to and including "uses:", trim, and
  # drop any trailing "# comment".
  ref="$(printf '%s' "$line" | sed -E 's/.*uses:[[:space:]]*//; s/[[:space:]]*#.*$//; s/[[:space:]]*$//')"

  # Skip quoting and local/reusable-workflow references.
  ref="${ref//\"/}"
  ref="${ref//\'/}"
  case "$ref" in
    ./*|.github/*) continue ;;          # local action / reusable workflow
    actions/*) continue ;;              # GitHub first-party org (trusted)
    *@*) : ;;                            # owner/repo@ref — check below
    *) continue ;;                      # no @ref (e.g. docker://) — out of scope
  esac

  pin="${ref##*@}"
  if [[ "$pin" =~ ^[0-9a-f]{40}$ ]]; then
    continue                            # immutable SHA — OK
  fi

  printf 'MUTABLE ACTION REF  %s:%s\n    %s\n' "$file" "$lineno" "$ref" >&2
  violations=$((violations + 1))
done < <(grep -rnE '^[[:space:]]*-?[[:space:]]*uses:' "$root/.github/workflows/" || true)

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
