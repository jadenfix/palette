#!/usr/bin/env bash
# Per-language LIVE conformance: launch beaterd, then for each language whose
# toolchain is present, run a program written IN THAT LANGUAGE that drives the
# generated control-plane client against the live API and verifies the typed
# request/response shapes match. Proves API-shape == SDK-shape, per language.
#
# Languages without a local toolchain are SKIPPED with a clear message (no
# silent gaps): Java needs a JDK+Maven, C++ needs cpprestsdk.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

PORT="${BEATER_E2E_PORT:-18100}"
GRPC_PORT="${BEATER_E2E_GRPC_PORT:-14337}"
data_dir="$(mktemp -d)"
log="$(mktemp)"
export BEATER_BASE_URL="http://127.0.0.1:$PORT"
export BEATER_TENANT="demo"
export BEATER_PROJECT="demo"

cleanup() { [ -n "${pid:-}" ] && kill "$pid" 2>/dev/null || true; rm -rf "$data_dir"; }
trap cleanup EXIT

echo "==> Building + launching beaterd"
cargo build -q -p beaterd
./target/debug/beaterd --addr "127.0.0.1:$PORT" --otlp-grpc-addr "127.0.0.1:$GRPC_PORT" \
  --data-dir "$data_dir" --auth-mode local >"$log" 2>&1 &
pid=$!
for _ in $(seq 1 60); do curl -fsS "$BEATER_BASE_URL/health" >/dev/null 2>&1 && { ready=1; break; }; sleep 0.5; done
[ -n "${ready:-}" ] || { echo "beaterd not healthy"; cat "$log"; exit 1; }
echo "    beaterd live on $BEATER_BASE_URL"

declare -a PASS=() SKIP=() FAIL=()

run_lang() {
  local lang="$1" toolcheck="$2"
  if [ ! -f "sdks/conformance/$lang/run.sh" ]; then
    echo "==> SKIP $lang (no conformance program yet)"; SKIP+=("$lang"); return
  fi
  if ! eval "$toolcheck" >/dev/null 2>&1; then
    echo "==> SKIP $lang (toolchain missing)"; SKIP+=("$lang"); return
  fi
  echo "==> $lang conformance"
  if bash "sdks/conformance/$lang/run.sh"; then
    echo "    $lang PASS"; PASS+=("$lang")
  else
    echo "    $lang FAIL"; FAIL+=("$lang")
  fi
}

# Portable toolchecks (work on Linux CI and macOS; run.sh sets JAVA_HOME for mvn).
cpprest_present() {
  command -v cmake >/dev/null 2>&1 || return 1
  test -d "$(brew --prefix cpprestsdk 2>/dev/null)" 2>/dev/null && return 0
  pkg-config --exists cpprest 2>/dev/null && return 0
  ls /usr/lib/libcpprest.* /usr/local/lib/libcpprest.* /opt/homebrew/opt/cpprestsdk 2>/dev/null | grep -q . && return 0
  return 1
}
# The generated C client links libcurl, so a bare cmake is not enough: require the
# curl development headers too, otherwise C "attempts and fails" instead of SKIP.
c_present() {
  command -v cmake >/dev/null 2>&1 || return 1
  { command -v cc >/dev/null 2>&1 || command -v gcc >/dev/null 2>&1; } || return 1
  pkg-config --exists libcurl 2>/dev/null && return 0
  ls /usr/include/curl/curl.h /usr/local/include/curl/curl.h /opt/homebrew/include/curl/curl.h 2>/dev/null | grep -q . && return 0
  return 1
}
run_lang python "python3 --version"
run_lang typescript "node --version"
run_lang rust "cargo --version"
run_lang go "go version"
run_lang c "c_present"
run_lang java "command -v mvn"
run_lang cpp "cpprest_present"

echo
echo "================ live conformance summary ================"
echo "PASS: ${PASS[*]:-none}"
echo "SKIP: ${SKIP[*]:-none}"
echo "FAIL: ${FAIL[*]:-none}"
# Never report success if nothing actually ran (all skipped == tested nothing).
if [ ${#PASS[@]} -eq 0 ]; then
  echo "ERROR: no language conformance ran (all skipped) -- nothing was tested" >&2
  exit 1
fi
# When a required set is given, it is the contract: every listed language must
# have PASSed (a required language that FAILed is not in PASS, so this catches
# it). Optional languages whose full toolchain happened to be present but failed
# are reported above but are best-effort, not gating -- this mirrors the SKIP
# policy and is why BEATER_CONFORMANCE_REQUIRE exists as an allowlist.
if [ -n "${BEATER_CONFORMANCE_REQUIRE:-}" ]; then
  for need in ${BEATER_CONFORMANCE_REQUIRE//,/ }; do
    printf '%s\n' "${PASS[@]}" | grep -qx "$need" || { echo "ERROR: required language '$need' did not PASS" >&2; exit 1; }
  done
  exit 0
fi
# No explicit allowlist (ad-hoc local run): stay strict -- any failure gates.
[ ${#FAIL[@]} -eq 0 ] || exit 1
