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

# Pin the generator for reproducible output.
GENERATOR_VERSION="7.11.0"
GENERATOR_IMAGE="openapitools/openapi-generator-cli:v${GENERATOR_VERSION}"
SPEC="sdks/openapi/beater-api.json"
LANGS=(rust python typescript go java c cpp ruby php csharp kotlin)

# Generator backend: by default the pinned Docker image (CI path). When
# BEATER_OPENAPI_GENERATOR_JAR points at the matching openapi-generator-cli JAR,
# run it with a local JVM instead -- byte-identical output to the Docker image
# (both wrap the same pinned JAR), for environments where Docker Hub is
# unreachable (e.g. a proxy that blocks the registry CDN). The Docker image
# remains the default so CI behavior is unchanged.
GENERATOR_JAR="${BEATER_OPENAPI_GENERATOR_JAR:-}"

CHECK_MODE=0
if [[ "${1:-}" == "--check" ]]; then
  CHECK_MODE=1
fi

# Optional release version for the generated clients (default keeps configs' 0.1.0).
VERSION="${BEATER_SDK_VERSION:-}"
version_props=()
if [[ -n "$VERSION" ]]; then
  version_props=(--additional-properties "packageVersion=$VERSION,artifactVersion=$VERSION,npmVersion=$VERSION,gemVersion=$VERSION")
fi

echo "==> Regenerating OpenAPI spec from beater-api handlers"
tmp_spec="$(mktemp "${TMPDIR:-/tmp}/beater-openapi.XXXXXX")"
trap 'rm -f "$tmp_spec"' EXIT
cargo run -q -p beater-api --example dump_openapi > "$tmp_spec"
mv "$tmp_spec" "$SPEC"
# Keep the dashboard snapshot identical to the canonical spec.
cp "$SPEC" web/dashboard/openapi/beater-read-api.json

# Generate one client. Uses the local JAR when BEATER_OPENAPI_GENERATOR_JAR is
# set, otherwise the pinned Docker image. Both invoke openapi-generator-cli
# v${GENERATOR_VERSION}, so the committed output is identical either way.
run_generator() {
  local lang="$1" out="$2"
  if [[ -n "$GENERATOR_JAR" ]]; then
    java -jar "$GENERATOR_JAR" generate \
      -i "$SPEC" \
      -c "sdks/config/$lang.yaml" \
      ${version_props[@]+"${version_props[@]}"} \
      -o "$out" \
      >/dev/null
  else
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
  fi
}

if [[ -n "$GENERATOR_JAR" ]]; then
  echo "==> Using local generator JAR ($GENERATOR_JAR)"
  # Assert the JAR is the pinned version -- a different generator would emit
  # divergent clients and silently break the zero-drift guarantee.
  jar_version="$(java -jar "$GENERATOR_JAR" version 2>/dev/null | tr -d '[:space:]')"
  if [[ "$jar_version" != "$GENERATOR_VERSION" ]]; then
    echo "ERROR: BEATER_OPENAPI_GENERATOR_JAR is version '$jar_version', expected '$GENERATOR_VERSION'" >&2
    exit 1
  fi
else
  echo "==> Pulling generator image ($GENERATOR_IMAGE)"
  docker pull -q "$GENERATOR_IMAGE" >/dev/null
fi

for lang in "${LANGS[@]}"; do
  out="sdks/clients/$lang"
  echo "==> Generating $lang -> $out"
  rm -rf "$out"
  mkdir -p "$out"
  run_generator "$lang" "$out"

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

  # The Java generator emits a trailing space in this regenerated enum header.
  # Normalize the touched file so `regen --check` and `git diff --check` agree.
  if [[ "$lang" == "java" ]]; then
    java_audit_action="$out/src/main/java/ai/beater/client/model/AuditAction.java"
    if [[ -f "$java_audit_action" ]]; then
      perl -0pi -e 's/[ \t]+$//mg; s/\n+\z/\n/' "$java_audit_action"
    fi
  fi
done

if [[ "$CHECK_MODE" == "1" ]]; then
  echo "==> Checking for drift"
  if ! git diff --quiet -- "$SPEC" web/dashboard/openapi/beater-read-api.json sdks/clients; then
    echo "ERROR: generated artifacts are stale. Run scripts/regen-sdks.sh and commit." >&2
    git --no-pager diff --stat -- "$SPEC" sdks/clients >&2
    exit 1
  fi
  echo "No drift: spec and all SDK clients are current."
fi

echo "Done. Layer-1 clients in sdks/clients/{${LANGS[*]// /,}}."
