#!/usr/bin/env bash
# One-command drift test: proves the API contract, the 7 SDK clients, the MCP
# tools, the docs, and the semantic conventions are all in sync with the Rust
# handlers. Run before pushing; CI (sdk-contract.yml) runs the same gates.
#
# Exit non-zero on ANY drift. See CONTRIBUTING.md.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

fail=0
sdk_regen_skipped=0
step() { echo; echo "==> $1"; }

step "1/6 spec == served routes (openapi_coverage)"
cargo test -q -p beater-api --test openapi_coverage || fail=1

step "2/6 generated OpenAPI snapshots are current"
tmp_spec="$(mktemp "${TMPDIR:-/tmp}/beater-openapi-check.XXXXXX")"
trap 'rm -f "$tmp_spec"' EXIT
cargo run -q -p beater-api --example dump_openapi > "$tmp_spec" || fail=1
if [ "$fail" -eq 0 ]; then
  if ! cmp -s "$tmp_spec" sdks/openapi/beater-api.json; then
    echo "sdks/openapi/beater-api.json is stale; run scripts/regen-sdks.sh" >&2
    fail=1
  fi
  if ! cmp -s "$tmp_spec" web/dashboard/openapi/beater-read-api.json; then
    echo "web/dashboard/openapi/beater-read-api.json is stale; run scripts/regen-sdks.sh" >&2
    fail=1
  fi
fi

step "3/6 all 7 SDK clients are current (regen --check)"
# Client generation needs Docker; skip only the client check with a loud message
# if unavailable. The committed OpenAPI snapshots are still checked above.
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  scripts/regen-sdks.sh --check || fail=1
else
  echo "WARN: docker unavailable -- skipping client regen check (CI enforces it)" >&2
  sdk_regen_skipped=1
fi

step "4/6 API shape consistency audit"
python3 scripts/audit-api-shapes.py || fail=1

step "5/6 semantic conventions == all 5 SDKs"
cargo xtask regen-semconv >/dev/null
git diff --exit-code -- sdks/semconv/conventions.json || { echo "conventions.json is stale" >&2; fail=1; }
python3 scripts/check-semconv-drift.py || fail=1

step "6/6 docs walkthrough references are current"
scripts/check-docs-walkthrough.sh --dry-run || fail=1

echo
if [ "$fail" -ne 0 ]; then
  echo "CONTRACT DRIFT DETECTED -- regenerate (see CONTRIBUTING.md) and commit." >&2
  exit 1
fi
if [ "$sdk_regen_skipped" -eq 1 ]; then
  echo "Partial local contract check passed: API, OpenAPI snapshots, MCP tools, docs, and conventions are in sync."
  echo "SDK client regeneration was not verified because Docker is unavailable; CI enforces it."
else
  echo "No drift: API, 7 SDKs, MCP tools, docs, and conventions are all in sync."
fi
