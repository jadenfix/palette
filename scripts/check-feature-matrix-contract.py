#!/usr/bin/env python3
"""Verify the public feature matrix preserves the open-core boundary."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/feature-matrix.md"
WORKFLOW = ROOT / ".github/workflows/feature-matrix-contract.yml"

DOC_TOKENS = {
    "license and public boundary": [
        "R12.1",
        "Apache-2.0",
        "GOVERNANCE.md",
        "no-rug-pull promise",
        "There is no\n\"open core that quietly relicenses.\"",
    ],
    "contract and engine stay open": [
        "The **contract and the engine are open.**",
        "sdks/openapi/beater-api.json",
        "crates/beater-schema",
        "sdks/semconv",
        "self-hostable `beaterd` server",
        "never a fork of the data model or a paywall on the SDK protocol",
    ],
    "required OSS capabilities": [
        "| All-in-one `beaterd` server | Yes | Yes (managed) |",
        "| OpenAPI `/v1` contract + 7 SDK clients | Yes | Yes (identical contract) |",
        "| Zero-SDK OTLP ingest (HTTP + gRPC) | Yes | Yes |",
        "| Eval / judge / gates / calibration | Yes | Yes |",
        "| Security primitives (scoped keys, audit events, redaction) | Yes | Yes (managed governance/export UX) |",
    ],
    "commercial boundary": [
        "Commercial offerings are *operational\nconvenience*",
        "the open-source server can do everything the protocol allows",
        "hosting, scale, and support",
        "Managed multi-region / autoscaling",
        "SLA, dedicated support, hosted scale",
    ],
    "enforcement hooks": [
        "CONTRIBUTING.md",
        "BEATER_SELF_HOST_TELEMETRY",
        "beater_core::SelfHostTelemetryConfig",
        "`thin-bins` cargo feature",
    ],
}

WORKFLOW_TOKENS = [
    "name: feature-matrix-contract",
    "python3 scripts/check-feature-matrix-contract.py",
    "docs/feature-matrix.md",
]


def read(path: Path, failures: list[str]) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        failures.append(f"missing required file: {path.relative_to(ROOT)}")
    except UnicodeDecodeError as exc:
        failures.append(f"file is not valid UTF-8: {path.relative_to(ROOT)}: {exc}")
    return ""


def require_tokens(label: str, text: str, tokens: list[str], failures: list[str]) -> None:
    for token in tokens:
        if token not in text:
            failures.append(f"{label} missing required token: {token!r}")


def main() -> int:
    failures: list[str] = []
    doc = read(DOC, failures)
    workflow = read(WORKFLOW, failures)

    for label, tokens in DOC_TOKENS.items():
        require_tokens(f"docs/feature-matrix.md {label}", doc, tokens, failures)
    require_tokens(".github/workflows/feature-matrix-contract.yml", workflow, WORKFLOW_TOKENS, failures)

    if failures:
        print("Feature matrix contract check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Feature matrix preserves the Apache-2.0 open-core boundary.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
