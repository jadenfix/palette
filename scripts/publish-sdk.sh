#!/usr/bin/env bash
# Publish one SDK at a synchronized version. Called per-target by release.yml.
# Each target no-ops with a clear message when its registry secret is absent, so
# the pipeline is wired now and "just works" once tokens are added as repo
# secrets. C/C++ have no central registry -- they ship as source + release tarballs.
#
# Version handling: generated clients are versioned at generation time
# (BEATER_SDK_VERSION -> openapi-generator, see regen-sdks.sh); the published
# ergonomic packages are stamped here so a `v0.2.0` tag actually publishes 0.2.0.
set -euo pipefail

usage() {
  echo "usage: publish-sdk.sh <target> <version>"
  echo "       publish-sdk.sh --validate-version <version>"
}

validate_version() {
  local candidate="$1"
  local semver_re='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-((0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z]*)(\.(0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z]*))*))?(\+([0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*))?$'
  if [[ ! "$candidate" =~ $semver_re ]]; then
    echo "ERROR: release version must be SemVer without a leading v: $candidate" >&2
    exit 1
  fi
}

if [[ "${1:-}" == "--validate-version" ]]; then
  [ "$#" -eq 2 ] || { usage >&2; exit 2; }
  validate_version "$2"
  echo "Valid release version: $2"
  exit 0
fi

target="${1:?usage: publish-sdk.sh <target> <version>}"
version="${2:?usage: publish-sdk.sh <target> <version>}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
validate_version "$version"

skip() { echo "SKIP $target: $1 not set (wire the secret to enable publishing)"; exit 0; }

# Stamp the single [project]/[package] version line of an ergonomic manifest.
stamp_pyproject() { perl -0pi -e 's/(\n\s*version\s*=\s*")[^"]*(")/${1}'"$version"'${2}/' "$1"; }

case "$target" in
  rust)
    [ -n "${CARGO_REGISTRY_TOKEN:-}" ] || skip CARGO_REGISTRY_TOKEN
    # --allow-dirty: the client is (re)generated + version-stamped at release time,
    # so the worktree is intentionally not a committed state.
    (cd sdks/clients/rust && cargo publish --allow-dirty --token "$CARGO_REGISTRY_TOKEN")
    ;;
  python)
    [ -n "${PYPI_TOKEN:-}" ] || skip PYPI_TOKEN
    pip install --quiet build twine
    stamp_pyproject sdks/python/pyproject.toml
    for pkg in sdks/clients/python sdks/python; do
      (cd "$pkg" && python -m build && TWINE_USERNAME=__token__ TWINE_PASSWORD="$PYPI_TOKEN" twine upload dist/*)
    done
    ;;
  typescript)
    [ -n "${NPM_TOKEN:-}" ] || skip NPM_TOKEN
    # Write the token narrowly and remove it on exit.
    ( umask 077 && echo "//registry.npmjs.org/:_authToken=${NPM_TOKEN}" > ~/.npmrc )
    trap 'rm -f ~/.npmrc' EXIT
    (cd sdks/typescript && npm version "$version" --no-git-tag-version --allow-same-version)
    for pkg in sdks/clients/typescript sdks/typescript; do
      (cd "$pkg" && npm install && npm run build --if-present && npm publish --access public)
    done
    ;;
  go)
    # Go modules publish by pushing the tag; the module proxy serves sdks/clients/go.
    echo "go: tag v${version} pushed; module proxy will serve sdks/clients/go at that version"
    ;;
  java)
    [ -n "${OSSRH_USERNAME:-}" ] || skip OSSRH_USERNAME
    [ -n "${OSSRH_PASSWORD:-}" ] || skip OSSRH_PASSWORD
    (cd sdks/clients/java && mvn --batch-mode versions:set -DnewVersion="$version" -DgenerateBackupPoms=false && mvn --batch-mode deploy -DskipTests)
    ;;
  ruby)
    [ -n "${RUBYGEMS_API_KEY:-}" ] || skip RUBYGEMS_API_KEY
    # The gem version is stamped at generation time (gemVersion -> version.rb,
    # see regen-sdks.sh). Write the API key narrowly and remove it on exit.
    ( umask 077 && mkdir -p ~/.gem && printf -- '---\n:rubygems_api_key: %s\n' "$RUBYGEMS_API_KEY" > ~/.gem/credentials )
    trap 'rm -f ~/.gem/credentials' EXIT
    (cd sdks/clients/ruby && gem build beater_client.gemspec -o beater_client.gem && gem push beater_client.gem)
    ;;
  php)
    # Packagist serves from the git tag (like Go): pushing v${version} makes it
    # available. If a Packagist API user/token is wired, ping the update hook so
    # the new tag is indexed immediately; otherwise no-op with a clear message.
    if [ -n "${PACKAGIST_USERNAME:-}" ] && [ -n "${PACKAGIST_API_TOKEN:-}" ]; then
      curl -fsS -XPOST -H'content-type:application/json' \
        "https://packagist.org/api/update-package?username=${PACKAGIST_USERNAME}&apiToken=${PACKAGIST_API_TOKEN}" \
        -d'{"repository":{"url":"https://github.com/jadenfix/beater"}}' >/dev/null
      echo "php: pinged Packagist update hook for tag v${version}"
    else
      echo "php: tag v${version} pushed; Packagist serves sdks/clients/php from the tag (set PACKAGIST_USERNAME/PACKAGIST_API_TOKEN to force reindex)"
    fi
    ;;
  csharp)
    [ -n "${NUGET_API_KEY:-}" ] || skip NUGET_API_KEY
    # The package version is stamped at generation time (packageVersion, see
    # regen-sdks.sh). Pack the generated client and push the produced .nupkg.
    (cd sdks/clients/csharp && dotnet pack -c Release src/Beater.Client/Beater.Client.csproj -o ./nupkg)
    dotnet nuget push "sdks/clients/csharp/nupkg/"*.nupkg \
      --source https://api.nuget.org/v3/index.json --api-key "$NUGET_API_KEY" --skip-duplicate
    ;;
  kotlin)
    # Kotlin publishes to Maven Central via the same OSSRH credentials as Java,
    # plus a GPG signing key (Central requires signed artifacts). The generated
    # client's build.gradle only applies `maven-publish` with no publication or
    # repository, so publishing is supplied by an init script (kept out of the
    # regenerated tree). The artifact version is stamped at generation time
    # (artifactVersion). Skip clearly if any required secret is missing rather
    # than uploading nothing and reporting a false success.
    [ -n "${OSSRH_USERNAME:-}" ] || skip OSSRH_USERNAME
    [ -n "${OSSRH_PASSWORD:-}" ] || skip OSSRH_PASSWORD
    [ -n "${OSSRH_URL:-}" ] || skip OSSRH_URL
    [ -n "${SIGNING_KEY:-}" ] || skip SIGNING_KEY
    init_script="$root/sdks/config/maven-central.init.gradle"
    # Use the client's pinned wrapper via `sh` so a missing exec bit doesn't matter.
    (cd sdks/clients/kotlin && sh ./gradlew --no-daemon --console=plain \
      --init-script "$init_script" publish)
    ;;
  *)
    echo "Unknown target: $target" >&2; exit 1 ;;
esac

echo "Published $target @ $version"
