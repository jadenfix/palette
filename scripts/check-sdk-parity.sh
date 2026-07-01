#!/usr/bin/env bash
# Local equivalent for the sdk-parity gate. The static manifest check is fast
# and always-on; pass --live to also run the existing language-toolchain live
# conformance suite against a local beaterd.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

python3 scripts/check-sdk-parity-contract.py

case "${1:-}" in
  "")
    ;;
  --live)
    scripts/e2e-clients-live.sh
    ;;
  *)
    echo "usage: scripts/check-sdk-parity.sh [--live]" >&2
    exit 2
    ;;
esac
