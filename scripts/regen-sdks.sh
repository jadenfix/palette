#!/usr/bin/env bash
# Regenerate the OpenAPI spec and every control-plane SDK from it.
#
# This is the heart of the zero-drift guarantee: ONE spec
# (sdks/openapi/beater-api.json) is generated from the Rust handlers, and every
# Layer-1 client is generated from that spec. Run after any API change, then
# commit the result. CI runs `--check` to fail on drift.
#
# Requires Docker (openapi-generator runs in a pinned container -- no local Java).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cleanup_paths=()
cleanup() {
  local path
  for path in "${cleanup_paths[@]}"; do
    rm -rf "$path"
  done
}
trap cleanup EXIT

# Pin the generator for reproducible output.
GENERATOR_IMAGE="openapitools/openapi-generator-cli:v7.11.0"
SPEC="sdks/openapi/beater-api.json"
LANGS=(rust python typescript go java c cpp)

CHECK_MODE=0
if [[ "${1:-}" == "--check" ]]; then
  CHECK_MODE=1
fi

snapshot_check_target() {
  local target="$1"
  mkdir -p "$check_snapshot/$(dirname "$target")"
  if [[ -e "$target" ]]; then
    cp -a "$target" "$check_snapshot/$target"
  fi
}

check_target_drifted() {
  local target="$1"
  local before="$check_snapshot/$target"
  if [[ ! -e "$before" || ! -e "$target" ]]; then
    [[ -e "$before" || -e "$target" ]]
    return
  fi
  ! diff -qr "$before" "$target" >/dev/null
}

check_snapshot=""
if [[ "$CHECK_MODE" == "1" ]]; then
  check_snapshot="$(mktemp -d "${TMPDIR:-/tmp}/beater-sdk-check-before.XXXXXX")"
  cleanup_paths+=("$check_snapshot")
  snapshot_check_target "$SPEC"
  snapshot_check_target "web/dashboard/openapi/beater-read-api.json"
  snapshot_check_target "sdks/clients"
fi

normalize_generated_text_files() {
  local out="$1"
  shift

  local file
  for file in "$@"; do
    if [[ -f "$out/$file" ]]; then
      perl -0pi -e 's/[ \t]+$//mg; s/\n+\z/\n/' "$out/$file"
    fi
  done
}

# Optional release version for the generated clients (default keeps configs' 0.1.0).
VERSION="${BEATER_SDK_VERSION:-}"
version_props=()
if [[ -n "$VERSION" ]]; then
  version_props=(--additional-properties "packageVersion=$VERSION,artifactVersion=$VERSION,npmVersion=$VERSION")
fi

echo "==> Regenerating OpenAPI spec from beater-api handlers"
tmp_spec="$(mktemp "${TMPDIR:-/tmp}/beater-openapi.XXXXXX")"
cleanup_paths+=("$tmp_spec")
cargo run -q -p beater-api --example dump_openapi > "$tmp_spec"
mv "$tmp_spec" "$SPEC"
# Keep the dashboard snapshot identical to the canonical spec.
cp "$SPEC" web/dashboard/openapi/beater-read-api.json

echo "==> Pulling generator image ($GENERATOR_IMAGE)"
docker pull -q "$GENERATOR_IMAGE" >/dev/null

for lang in "${LANGS[@]}"; do
  out="sdks/clients/$lang"
  echo "==> Generating $lang -> $out"
  rm -rf "$out"
  mkdir -p "$out"
  # Run the generator as the host user so output isn't root-owned/read-only on Linux
  # CI runners (where the daemon runs as root); otherwise the patch step below cannot
  # write its temp files. No-op on Docker Desktop, which already maps to the host user.
  docker run --rm \
    --user "$(id -u):$(id -g)" \
    -v "$root:/local" \
    "$GENERATOR_IMAGE" generate \
    -i "/local/$SPEC" \
    -c "/local/sdks/config/$lang.yaml" \
    ${version_props[@]+"${version_props[@]}"} \
    -o "/local/$out" \
    >/dev/null

  # Reproducibly re-apply committed fixes for known openapi-generator output bugs
  # (C/C++ only). This keeps the generated clients buildable WITHOUT hand-editing
  # after each regen -- the patch is the single source of those fixes. Fail loudly
  # (no fuzz, no backups) if the patch no longer applies cleanly to fresh output.
  if [[ -f "sdks/patches/$lang.patch" ]]; then
    echo "    applying sdks/patches/$lang.patch"
    patch -p1 --fuzz=0 --no-backup-if-mismatch -d "$out" < "sdks/patches/$lang.patch"
    if find "$out" -name '*.rej' -o -name '*.orig' | grep -q .; then
      echo "ERROR: patch left .rej/.orig in $out -- patch is stale vs generated output" >&2
      exit 1
    fi
  fi

  # Several generator templates emit trailing spaces in markdown tables,
  # method stubs, and comments for the touched ingest operation. Normalize
  # those files so `regen --check` and `git diff --check` agree without
  # hand-editing generated clients or churning unrelated generated output.
  case "$lang" in
    c)
      normalize_generated_text_files "$out" \
        README.md \
        api/IngestAPI.c \
        api/IngestAPI.h \
        docs/IngestAPI.md
      ;;
    cpp)
      normalize_generated_text_files "$out" \
        include/beater-client/api/IngestApi.h
      ;;
    go)
      normalize_generated_text_files "$out" \
        README.md \
        docs/IngestAPI.md
      ;;
    java)
      normalize_generated_text_files "$out" \
        README.md \
        docs/IngestApi.md \
        src/main/java/ai/beater/client/api/IngestApi.java \
        src/main/java/ai/beater/client/model/AuditAction.java \
        src/test/java/ai/beater/client/api/IngestApiTest.java
      ;;
    python)
      normalize_generated_text_files "$out" \
        README.md \
        beater_client/api/ingest_api.py \
        docs/IngestApi.md
      ;;
    rust)
      normalize_generated_text_files "$out" \
        README.md \
        docs/IngestApi.md
      ;;
  esac
done

if [[ "$CHECK_MODE" == "1" ]]; then
  echo "==> Checking for drift"
  drifted=0
  for target in "$SPEC" "web/dashboard/openapi/beater-read-api.json" "sdks/clients"; do
    if check_target_drifted "$target"; then
      drifted=1
    fi
  done
  if [[ "$drifted" == "1" ]]; then
    echo "ERROR: generated artifacts are stale. Run scripts/regen-sdks.sh and commit." >&2
    echo "Differences from the pre-check generated tree:" >&2
    diff -qr "$check_snapshot/$SPEC" "$SPEC" >&2 || true
    diff -qr \
      "$check_snapshot/web/dashboard/openapi/beater-read-api.json" \
      "web/dashboard/openapi/beater-read-api.json" >&2 || true
    diff -qr "$check_snapshot/sdks/clients" "sdks/clients" >&2 || true
    exit 1
  fi
  echo "No drift: spec and all SDK clients are current."
fi

echo "Done. Layer-1 clients in sdks/clients/{${LANGS[*]// /,}}."
