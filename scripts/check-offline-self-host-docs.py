#!/usr/bin/env python3
"""Verify the offline self-host doc keeps the OSS no-cloud contract explicit."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/offline-self-host.md"
WORKFLOW = ROOT / ".github/workflows/offline-self-host-docs.yml"

DOC_TOKENS = {
    "R1.3 no-cloud posture": [
        "R1.3",
        "without Beater Cloud",
        "no dependency on\nBeater Cloud",
        "no mandatory Beater Cloud account",
        "No call to any `*.beater.dev` / `beater.cloud` host is required",
    ],
    "default local runtime": [
        "single `beaterd` process plus the\ndashboard",
        "embedded SQLite + the local\n  filesystem",
        "docker compose up beaterd dashboard",
        "beaterd --auth-mode local",
        "--auth-mode required",
    ],
    "egress and telemetry": [
        "no outbound calls except to configured providers",
        "BEATER_SELF_HOST_TELEMETRY",
        "https://telemetry.beater.dev/v1/usage",
        "fails closed",
    ],
    "dashboard and Vercel boundary": [
        "Vercel is not required for self-host",
        "local compose\n  `dashboard` service",
        "hosted Vercel deploys\n  are dashboard/control-plane surfaces only",
        "not the `beaterd` runtime",
    ],
    "proof hook": [
        "bins/beaterd/tests/offline_compose.rs",
        "keeps external backends\nopt-in",
    ],
}

WORKFLOW_TOKENS = [
    "name: offline-self-host-docs",
    "python3 scripts/check-offline-self-host-docs.py",
    "docs/offline-self-host.md",
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
        require_tokens(f"docs/offline-self-host.md {label}", doc, tokens, failures)
    require_tokens(".github/workflows/offline-self-host-docs.yml", workflow, WORKFLOW_TOKENS, failures)

    if failures:
        print("Offline self-host docs check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Offline self-host docs preserve the OSS no-cloud runtime contract.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
