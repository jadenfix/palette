#!/usr/bin/env bash
# Live conformance for the generated Java control-plane client.
#   1. Install the generated client (ai.beater:beater-client) into the local repo.
#   2. Build a tiny module that depends on it and round-trips health +
#      createDataset + listTraces against a live beaterd, asserting typed shapes.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
client="$(cd "$here/../../clients/java" && pwd)"

# Prefer a keg-only JDK + Maven from Homebrew when present (macOS dev boxes);
# otherwise fall back to the ambient JDK/JAVA_HOME (Linux CI, containers).
if [ -d "/opt/homebrew/opt/openjdk" ]; then
  export PATH="/opt/homebrew/opt/openjdk/bin:$PATH"
  export JAVA_HOME="/opt/homebrew/opt/openjdk"
elif [ -z "${JAVA_HOME:-}" ] && command -v java >/dev/null 2>&1; then
  # Derive JAVA_HOME from the java on PATH so mvn's launcher is satisfied.
  export JAVA_HOME="$(dirname "$(dirname "$(readlink -f "$(command -v java)")")")"
fi

: "${BEATER_BASE_URL:?BEATER_BASE_URL must be set (live beaterd)}"

echo "  installing generated client into local maven repo"
mvn -q -f "$client/pom.xml" -DskipTests install

echo "  building conformance module"
mvn -q -f "$here/pom.xml" -DskipTests package

echo "  running conformance"
java -jar "$here/target/beater-conformance.jar"
