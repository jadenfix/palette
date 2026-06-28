#!/usr/bin/env bash
# Report every #[ignore] / #[ignore = "..."] test across Rust crates.
#
# Columns: FILE:LINE | TEST NAME | REASON | OWNING AREA
# Exit 0 by default (report only).  Pass --require-reason to exit non-zero when
# any #[ignore] attribute lacks an attached reason string (hygiene gate).
#
# Usage:
#   scripts/report-ignored-tests.sh                  # print table
#   scripts/report-ignored-tests.sh --require-reason # enforce reason annotations

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

require_reason=0
self_test=0
for arg in "$@"; do
  case "$arg" in
    --require-reason) require_reason=1 ;;
    --self-test)      self_test=1 ;;
    *) echo "Unknown flag: $arg" >&2; exit 2 ;;
  esac
done

# ---------------------------------------------------------------------------
# --self-test: verify the parser handles both bare and annotated #[ignore].
# ---------------------------------------------------------------------------
if [[ "$self_test" -eq 1 ]]; then
  echo "==> self-test: parser handles both #[ignore] forms"
  _st_dir="$(mktemp -d)"
  trap 'rm -rf "$_st_dir"' EXIT

  mkdir -p "$_st_dir/crates/test-self/src"
  cat >"$_st_dir/crates/test-self/src/lib.rs" <<'RSEOF'
#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn bare_ignore_test() { todo!() }

    #[test]
    #[ignore = "needs special env"]
    fn annotated_ignore_test() { todo!() }
}
RSEOF

  # Scan the scratch crate (simulating what the main loop does).
  _st_hits="$(grep -rn --include="*.rs" '#\[ignore' "$_st_dir/crates/" 2>/dev/null \
    | grep -v '^\([^:]*\):[0-9]*:[[:space:]]*//' || true)"
  _st_count="$(printf '%s\n' "$_st_hits" | grep -c '#\[ignore' || true)"
  if [[ "$_st_count" -ne 2 ]]; then
    echo "FAIL: expected 2 hits, got $_st_count" >&2; exit 1
  fi
  # Check bare #[ignore] line is present (no "=" sign follows the attribute).
  if printf '%s\n' "$_st_hits" | grep -qE '#\[ignore\]'; then
    echo "  ok: bare #[ignore] line detected"
  else
    echo "FAIL: bare #[ignore] line not found" >&2; exit 1
  fi
  # Check annotated form is present.
  if printf '%s\n' "$_st_hits" | grep -q 'needs special env'; then
    echo "  ok: annotated #[ignore = \"...\"] line detected with reason text"
  else
    echo "FAIL: annotated reason not found" >&2; exit 1
  fi
  rm -rf "$_st_dir"
  trap - EXIT
  echo "self-test passed"
  exit 0
fi

# ---------------------------------------------------------------------------
# Collect every #[ignore...] line from Rust sources in crates/ and bins/.
# ---------------------------------------------------------------------------

tmpfile="$(mktemp)"
trap 'rm -f "$tmpfile"' EXIT

# Match only lines where #[ignore starts as an actual attribute (not inside //, //!, or ///).
grep -rn --include="*.rs" '#\[ignore' crates/ bins/ 2>/dev/null \
  | grep -v '^\([^:]*\):[0-9]*:[[:space:]]*//' \
  >"$tmpfile" || true

if [[ ! -s "$tmpfile" ]]; then
  echo "No ignored tests found."
  exit 0
fi

# ---------------------------------------------------------------------------
# Parse hits and extract: location, test name, reason, owning area.
# ---------------------------------------------------------------------------

print_header() {
  printf "%-55s  %-40s  %-50s  %s\n" "LOCATION" "TEST NAME" "REASON" "OWNING AREA"
  printf "%s\n" "$(printf '%0.s-' {1..185})"
}

rows=()
missing_reason=()

while IFS=: read -r relfile lineno rest; do
  # Extract reason string from #[ignore = "..."]
  reason=""
  if [[ "$rest" =~ \#\[ignore[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
    reason="${BASH_REMATCH[1]}"
  fi

  # Look ahead from lineno+1 for the fn declaration (within the next 5 lines).
  testname="<unknown>"
  absfile="$root/$relfile"
  if [[ -f "$absfile" ]]; then
    while IFS= read -r lookahead; do
      if [[ "$lookahead" =~ (async[[:space:]]+)?fn[[:space:]]+([A-Za-z_][A-Za-z0-9_]*) ]]; then
        testname="${BASH_REMATCH[2]}"
        break
      fi
    done < <(tail -n +"$((lineno + 1))" "$absfile" | head -5)
  fi

  # Owning area: crate name from crates/<name> or bins/<name>.
  area=""
  if [[ "$relfile" =~ ^crates/([^/]+) ]]; then
    area="${BASH_REMATCH[1]}"
  elif [[ "$relfile" =~ ^bins/([^/]+) ]]; then
    area="${BASH_REMATCH[1]}"
  fi
  # Append §section hint if present in the reason.
  if [[ "$reason" =~ §([0-9]+(\.[0-9]+)*) ]]; then
    area="$area (§${BASH_REMATCH[1]})"
  fi

  loc="$relfile:$lineno"
  rows+=("$loc|$testname|$reason|$area")

  if [[ -z "$reason" ]]; then
    missing_reason+=("$loc :: $testname")
  fi
done < "$tmpfile"

# ---------------------------------------------------------------------------
# Print table.
# ---------------------------------------------------------------------------
echo
echo "Ignored-test ledger — $(date -u +%Y-%m-%dT%H:%MZ)"
echo
print_header
for row in "${rows[@]}"; do
  IFS='|' read -r loc name rsn ar <<<"$row"
  printf "%-55s  %-40s  %-50s  %s\n" "$loc" "$name" "${rsn:-(no reason)}" "$ar"
done

echo
echo "Total ignored tests: ${#rows[@]}"

# ---------------------------------------------------------------------------
# --require-reason gate.
# ---------------------------------------------------------------------------
if [[ "$require_reason" -eq 1 && "${#missing_reason[@]}" -gt 0 ]]; then
  echo
  echo "FAIL: the following ignored tests have no reason string:" >&2
  for m in "${missing_reason[@]}"; do
    echo "  $m" >&2
  done
  echo 'Add #[ignore = "<reason>"] to each test to pass this gate.' >&2
  exit 1
fi

exit 0
