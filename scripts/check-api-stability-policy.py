#!/usr/bin/env python3
"""Verify the API stability policy stays tied to the real contract gates."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "docs/api-stability-policy.md"
WORKFLOW = ROOT / ".github/workflows/sdk-contract.yml"

POLICY_TOKENS = {
    "stable /v1 contract": [
        "/v1",
        "OpenAPI",
        "sdks/openapi/beater-api.json",
        "crates/beater-api",
        "scripts/check-contract-sync.sh",
        ".github/workflows/sdk-contract.yml",
        "sdk-contract",
        "oasdiff",
    ],
    "pre-1.0 caveat": [
        "Pre-1.0 caveat",
        "no wire/SDK backward-compatibility is promised",
        "before 1.0",
        "does not relax the contract discipline",
    ],
    "deprecation mechanics": [
        "deprecated: true",
        "Deprecation",
        "Sunset",
        "RFC 8594",
        "6 months",
        "/v2",
    ],
    "self-host parity": [
        "Self-hosted `beaterd` and Beater Cloud serve the identical contract",
        "OSS runs without Beater Cloud",
    ],
}

WORKFLOW_TOKENS = [
    "python3 scripts/check-api-stability-policy.py",
    "scripts/regen-sdks.sh --check",
    "python3 scripts/audit-api-shapes.py",
    "tufin/oasdiff breaking",
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
    policy = read(POLICY, failures)
    workflow = read(WORKFLOW, failures)

    for label, tokens in POLICY_TOKENS.items():
        require_tokens(f"docs/api-stability-policy.md {label}", policy, tokens, failures)
    require_tokens(".github/workflows/sdk-contract.yml", workflow, WORKFLOW_TOKENS, failures)

    if failures:
        print("API stability policy check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("API stability policy is tied to the /v1 contract and sdk-contract gate.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
