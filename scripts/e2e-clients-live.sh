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
server_log="$(mktemp)"
log_dir="${BEATER_E2E_LOG_DIR:-$(mktemp -d)}"
keep_logs=1
if [ -z "${BEATER_E2E_LOG_DIR:-}" ]; then
  keep_logs=0
fi
mkdir -p "$log_dir"
export BEATER_BASE_URL="http://127.0.0.1:$PORT"
export BEATER_TENANT="demo"
export BEATER_PROJECT="demo"

cleanup() {
  if [ -n "${pid:-}" ]; then
    kill "$pid" 2>/dev/null || true
  fi
  rm -rf "$data_dir"
  [ "$keep_logs" -eq 1 ] || rm -rf "$log_dir"
}
trap cleanup EXIT

echo "==> Building + launching beaterd"
cargo build -q -p beaterd
./target/debug/beaterd --addr "127.0.0.1:$PORT" --otlp-grpc-addr "127.0.0.1:$GRPC_PORT" \
  --data-dir "$data_dir" --auth-mode local >"$server_log" 2>&1 &
pid=$!
for _ in $(seq 1 60); do curl -fsS "$BEATER_BASE_URL/health" >/dev/null 2>&1 && { ready=1; break; }; sleep 0.5; done
[ -n "${ready:-}" ] || { echo "beaterd not healthy"; cat "$server_log"; exit 1; }
echo "    beaterd live on $BEATER_BASE_URL"

declare -a PASS=() SKIP=() FAIL=()
declare -a SUPPORTED_LANGS=()
declare -a REQUIRED_LANGS=()
declare -A REQUIRED_LANGS_SEEN=()

discover_supported_langs() {
  for lang_dir in sdks/conformance/*/; do
    [ -d "$lang_dir" ] || continue
    lang="$(basename "$lang_dir")"
    if [ -f "sdks/conformance/$lang/run.sh" ]; then
      SUPPORTED_LANGS+=("$lang")
    fi
  done
}

normalize_lang_token() {
  local token="$1"
  token="${token,,}"
  token="${token#"${token%%[![:space:]]*}"}"
  token="${token%"${token##*[![:space:]]}"}"
  case "$token" in
    c++|cxx)
      token="cpp"
      ;;
  esac
  printf '%s' "$token"
}

is_supported_lang() {
  local value="$1"
  local lang
  for lang in "${SUPPORTED_LANGS[@]}"; do
    [ "$lang" = "$value" ] && return 0
  done
  return 1
}

validate_requirements() {
  local raw="$1"
  local need normalized
  local IFS=','
  local -a tokens
  read -r -a tokens <<< "$raw"

  if [ ${#tokens[@]} -eq 0 ]; then
    return 0
  fi

  for need in "${tokens[@]}"; do
    normalized="$(normalize_lang_token "$need")"
    [ -z "$normalized" ] && continue
    if [ "${REQUIRED_LANGS_SEEN[$normalized]+x}" = x ]; then
      continue
    fi

    if ! is_supported_lang "$normalized"; then
      echo "ERROR: unknown required language '$normalized' in BEATER_CONFORMANCE_REQUIRE" >&2
      echo "Supported languages: ${SUPPORTED_LANGS[*]:-none}" >&2
      return 1
    fi

    REQUIRED_LANGS_SEEN["$normalized"]=1
    REQUIRED_LANGS+=("$normalized")
  done
}

discover_supported_langs

IFS=$'\n' SUPPORTED_LANGS=($(printf '%s\n' "${SUPPORTED_LANGS[@]}" | sort))
unset IFS

if [ -n "${BEATER_CONFORMANCE_REQUIRE:-}" ]; then
  validate_requirements "$BEATER_CONFORMANCE_REQUIRE"
fi

run_lang() {
  local lang="$1" toolcheck="$2"
  local lang_log="${log_dir}/${lang}-conformance.log"
  if [ ! -f "sdks/conformance/$lang/run.sh" ]; then
    echo "==> SKIP $lang (no conformance program yet)"; SKIP+=("$lang"); return
  fi
  if ! eval "$toolcheck" >/dev/null 2>&1; then
    echo "==> SKIP $lang (toolchain missing)"; SKIP+=("$lang"); return
  fi
  echo "==> $lang conformance"
  if bash "sdks/conformance/$lang/run.sh" 2>&1 | tee "$lang_log"; then
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
run_lang python "python3 --version"
run_lang typescript "node --version"
run_lang rust "cargo --version"
run_lang go "go version"
run_lang c "cmake --version"
run_lang java "command -v mvn"
run_lang cpp "cpprest_present"

echo
echo "================ live conformance summary ================"
echo "PASS: ${PASS[*]:-none}"
echo "SKIP: ${SKIP[*]:-none}"
echo "FAIL: ${FAIL[*]:-none}"
if [ -n "${BEATER_E2E_LOG_DIR:-}" ]; then
  echo "Conformance logs: $log_dir"
fi
# Never report success if nothing actually ran (all skipped == tested nothing).
if [ ${#PASS[@]} -eq 0 ]; then
  echo "ERROR: no language conformance ran (all skipped) -- nothing was tested" >&2
  exit 1
fi
# Optionally require a specific set (comma-separated) to have PASSed.
if [ -n "${BEATER_CONFORMANCE_REQUIRE:-}" ]; then
  for need in "${REQUIRED_LANGS[@]}"; do
    printf '%s\n' "${PASS[@]}" | grep -qx "$need" || { echo "ERROR: required language '$need' did not PASS" >&2; exit 1; }
  done
fi
[ ${#FAIL[@]} -eq 0 ] || exit 1
