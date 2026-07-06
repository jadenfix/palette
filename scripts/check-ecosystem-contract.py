#!/usr/bin/env python3
"""Static drift checks for Beater ecosystem integration boundaries.

The active neighbor repositories are not available in CI, so this check only
guards Beater's side of the contract: stable ingress paths, hosted-only billing,
and the standalone/offline promise.
"""

from __future__ import annotations

import sys
from pathlib import Path


REQUIRED_DOC_MARKERS = (
    "Tempo",
    "beater.js",
    "beaterOS",
    "Aether",
    "BEATER_TRACE_EXPORT_URL",
    "BEATER_OTLP_EXPORT_URL",
    "POST /v1/traces",
    "POST /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces",
    "POST /v1/traces/native",
    "POST /v1/import/{tenant_id}/{project_id}/{environment_id}",
    "non-default Cargo `billing` feature",
    "must not authorize or block local beaterOS actions",
    "PaymentEnvelope",
    "AIC/SWR escrow",
    "observed metadata",
    "release funds",
    "refunds are compensating entries",
)

REQUIRED_API_MARKERS = (
    '.route("/v1/traces", post(ingest_otlp_json_collector))',
    '.route("/v1/traces/native", post(ingest_native))',
    '"/v1/otlp/:tenant_id/:project_id/:environment_id/v1/traces"',
    '"/v1/import/:tenant_id/:project_id/:environment_id"',
    '"/v1/billing/webhooks/stripe"',
    "#[cfg(feature = \"billing\")]",
    "beateros.payment_mandate_id",
    "aether.payment_envelope_id",
)

REQUIRED_BEATERD_MARKERS = (
    "#[cfg(feature = \"billing\")]",
    "billing.sqlite",
    "with_billing",
)

REQUIRED_OFFLINE_MARKERS = (
    "fully self-hosted",
    "no dependency on",
)

REQUIRED_GOVERNANCE_MARKERS = (
    "mandatory phone-home",
    "license-key check",
)


def repo_root() -> Path:
    if len(sys.argv) > 2:
        raise SystemExit("usage: scripts/check-ecosystem-contract.py [repo-root]")
    if len(sys.argv) == 2:
        return Path(sys.argv[1]).resolve()
    return Path(__file__).resolve().parents[1]


def read(root: Path, rel: str, failures: list[str]) -> str:
    path = root / rel
    if not path.is_file():
        failures.append(f"{rel} is missing")
        return ""
    return path.read_text(encoding="utf-8")


def require_markers(text: str, rel: str, markers: tuple[str, ...], failures: list[str]) -> None:
    for marker in markers:
        if marker not in text:
            failures.append(f"{rel} missing marker {marker!r}")


def main() -> None:
    root = repo_root()
    failures: list[str] = []

    contract = read(root, "docs/ecosystem-integration-contract.md", failures)
    api = read(root, "crates/beater-api/src/lib.rs", failures)
    beaterd = read(root, "bins/beaterd/src/main.rs", failures)
    offline = read(root, "docs/offline-self-host.md", failures)
    governance = read(root, "GOVERNANCE.md", failures)

    require_markers(
        contract,
        "docs/ecosystem-integration-contract.md",
        REQUIRED_DOC_MARKERS,
        failures,
    )
    require_markers(api, "crates/beater-api/src/lib.rs", REQUIRED_API_MARKERS, failures)
    require_markers(beaterd, "bins/beaterd/src/main.rs", REQUIRED_BEATERD_MARKERS, failures)
    require_markers(offline, "docs/offline-self-host.md", REQUIRED_OFFLINE_MARKERS, failures)
    require_markers(governance, "GOVERNANCE.md", REQUIRED_GOVERNANCE_MARKERS, failures)

    if failures:
        print("ECOSYSTEM CONTRACT DRIFT DETECTED", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        raise SystemExit(1)

    print("Static ecosystem integration markers are aligned with Beater-side code and docs.")


if __name__ == "__main__":
    main()
