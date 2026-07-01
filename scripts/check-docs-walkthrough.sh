#!/usr/bin/env bash
# Docs-walkthrough drift verifier.
#
# Parses the key walkthrough/quickstart docs (README.md, docs/local-dev.md,
# CONTRIBUTING.md) and asserts that every artifact they reference — scripts,
# example files, doc pages, workflow files, cargo xtask subcommands — still
# exists in the repository.  This is a **static verifier only**: it never
# executes network, build, or deploy commands.
#
# Usage:
#   scripts/check-docs-walkthrough.sh [--dry-run]
#
# --dry-run (or default): inspect only, print every check, exit non-zero on
#   drift.  No side-effectful commands are ever run regardless of this flag.
#
# Exit non-zero on ANY drift.  Compatible with bash 3.2+ (macOS default).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Accept --dry-run (default behaviour is already non-destructive; the flag is
# documented for explicitness and consumed here so unknown-flag errors are
# avoided when callers pass it).
for arg in "$@"; do
  case "$arg" in
    --dry-run) ;;   # default — nothing changes
    *) echo "Unknown flag: $arg" >&2; exit 2 ;;
  esac
done

fail=0
warnings=0

step()   { echo; echo "==> $*"; }
ok()     { echo "    [OK]  $*"; }
err()    { echo "    [ERR] $*" >&2; fail=1; }
notice() { echo "    [SKIP] $*" >&2; warnings=$((warnings+1)); }

# Docs to scan (the walkthrough / quickstart surface).
WALKTHROUGH_DOCS="README.md docs/local-dev.md CONTRIBUTING.md"

# ---------------------------------------------------------------------------
# Step 1: all referenced scripts/*.sh and scripts/*.py exist.
# ---------------------------------------------------------------------------
step "1/6 scripts referenced in walkthrough docs exist"

# shellcheck disable=SC2086
while IFS= read -r ref; do
  [ -z "$ref" ] && continue
  path="$root/$ref"
  if [ ! -f "$path" ]; then
    err "Missing script: $ref"
  elif [ ! -s "$path" ]; then
    err "Empty script file: $ref"
  else
    ok "$ref"
  fi
done < <(grep -h -oE 'scripts/[A-Za-z0-9_.-]+\.(sh|py)' $WALKTHROUGH_DOCS 2>/dev/null | sort -u)

# ---------------------------------------------------------------------------
# Step 2: referenced examples/ paths exist.
# ---------------------------------------------------------------------------
step "2/6 examples/ paths referenced in walkthrough docs exist"

# shellcheck disable=SC2086
while IFS= read -r ref; do
  [ -z "$ref" ] && continue
  # Strip trailing punctuation that the regex may capture.
  ref="${ref%,}"
  ref="${ref%.}"
  ref="${ref%)}"
  # Skip bare 'examples/' without a concrete subpath.
  [ "$ref" = "examples/" ] && continue
  path="$root/$ref"
  if [ ! -e "$path" ]; then
    err "Missing example path: $ref"
  else
    ok "$ref"
  fi
done < <(grep -h -oE 'examples/[A-Za-z0-9_./-]+' $WALKTHROUGH_DOCS 2>/dev/null | sort -u)

# ---------------------------------------------------------------------------
# Step 3: static docs/ paths referenced in walkthrough docs exist.
#
# Evidence / generated artefacts that are written at proof-run time are
# explicitly excluded because they may not be committed on every branch.
# ---------------------------------------------------------------------------
step "3/6 static docs/ paths referenced in walkthrough docs exist"

is_generated_evidence() {
  case "$1" in
    docs/demos/gate2-outside-compose.log) return 0 ;;
    docs/demos/gate2-outside-terminal.log) return 0 ;;
    *) return 1 ;;
  esac
}

# shellcheck disable=SC2086
while IFS= read -r ref; do
  [ -z "$ref" ] && continue
  ref="${ref%,}"
  ref="${ref%.}"
  ref="${ref%)}"

  if is_generated_evidence "$ref"; then
    notice "Generated evidence artefact (absent until proof runs): $ref"
    continue
  fi

  path="$root/$ref"
  if [ ! -f "$path" ]; then
    err "Missing doc: $ref"
  else
    ok "$ref"
  fi
done < <(grep -h -oE 'docs/[A-Za-z0-9_./-]+\.(md|webm|json|log)' $WALKTHROUGH_DOCS 2>/dev/null | sort -u)

# ---------------------------------------------------------------------------
# Step 4: .github/ paths referenced in walkthrough docs exist.
# ---------------------------------------------------------------------------
step "4/6 .github/ paths referenced in walkthrough docs exist"

# shellcheck disable=SC2086
while IFS= read -r ref; do
  [ -z "$ref" ] && continue
  ref="${ref%,}"
  ref="${ref%.}"
  ref="${ref%)}"
  # Skip bare directory references like '.github/workflows/'.
  case "$ref" in */) continue ;; esac
  path="$root/$ref"
  if [ ! -f "$path" ]; then
    err "Missing .github file: $ref"
  else
    ok "$ref"
  fi
done < <(grep -h -oE '\.github/[A-Za-z0-9_./-]+' $WALKTHROUGH_DOCS 2>/dev/null | sort -u)

# ---------------------------------------------------------------------------
# Step 5: cargo xtask subcommands referenced in walkthrough docs are defined
#         in crates/xtask/src/main.rs.
# ---------------------------------------------------------------------------
step "5/6 cargo xtask subcommands referenced in walkthrough docs are defined"

XTASK_SOURCE="$root/crates/xtask/src/main.rs"
if [ ! -f "$XTASK_SOURCE" ]; then
  err "xtask source not found: crates/xtask/src/main.rs"
else
  # shellcheck disable=SC2086
  while IFS= read -r subcmd; do
    [ -z "$subcmd" ] && continue
    # Convert kebab-case to PascalCase using python3 (portable; avoids
    # GNU-only \u in sed that macOS BSD sed does not support).
    pascal="$(python3 -c "
import sys
s = sys.argv[1]
print(''.join(w.capitalize() for w in s.split('-')))
" "$subcmd")"
    if grep -qF "Cmd::$pascal" "$XTASK_SOURCE"; then
      ok "cargo xtask $subcmd  (Cmd::$pascal found in xtask source)"
    else
      err "cargo xtask $subcmd not found in xtask source (expected Cmd::$pascal in crates/xtask/src/main.rs)"
    fi
  done < <(grep -h -oE 'cargo xtask [a-z-]+' $WALKTHROUGH_DOCS 2>/dev/null \
             | grep -oE '[a-z-]+$' \
             | sort -u)
fi

# ---------------------------------------------------------------------------
# Step 6: governance docs required by ARCHITECTURE.md §20.7 #5.11 / §24.3
#         are present and non-empty.
# ---------------------------------------------------------------------------
step "6/6 governance docs required by the architecture exist"

require_doc_contains() {
  path="$1"
  needle="$2"
  label="$3"
  if [ ! -f "$path" ]; then
    err "Missing governance doc: $path"
  elif [ ! -s "$path" ]; then
    err "Empty governance doc: $path"
  elif ! grep -qF "$needle" "$path"; then
    err "$path must mention $label"
  else
    ok "$path  ($label)"
  fi
}

require_doc_contains "LICENSE" "Apache License" "Apache license"
require_doc_contains "GOVERNANCE.md" "no-rug-pull promise" "open-core governance"
require_doc_contains "SECURITY.md" "vulnerability" "coordinated vulnerability reporting"
require_doc_contains "CONTRIBUTING.md" "contract" "contract regeneration workflow"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo
if [ "$warnings" -gt 0 ]; then
  echo "$warnings skipped check(s) (generated evidence artefacts — see SKIP lines above)." >&2
fi
if [ "$fail" -ne 0 ]; then
  echo "DOCS WALKTHROUGH DRIFT DETECTED — update the walkthrough docs or restore the" >&2
  echo "referenced artefacts.  See docs/local-dev.md and CONTRIBUTING.md." >&2
  exit 1
fi
echo "All walkthrough references are valid — no docs drift detected."
