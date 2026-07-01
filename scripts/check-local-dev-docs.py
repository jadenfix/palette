#!/usr/bin/env python3
"""Verify the local development guide preserves the documented bootstrap path."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/local-dev.md"
WORKFLOW = ROOT / ".github/workflows/local-dev-docs.yml"

DOC_TOKENS = {
    "requirement and adjacent docs": [
        "R12.4",
        "CONTRIBUTING.md",
        "../.github/",
    ],
    "prerequisites": [
        "Rust",
        "rustfmt",
        "clippy",
        "Docker Compose v2",
        "Node 24+",
        "Python 3.12+",
    ],
    "bootstrap commands": [
        "git clone https://github.com/jadenfix/beater.git",
        "cargo build --workspace",
        "cargo test --workspace",
        "cd web/dashboard && npm ci",
    ],
    "local runtime": [
        "single `beaterd` binary",
        "cargo run -p beaterd -- --auth-mode local",
        "docker compose up beaterd dashboard",
        ":8080",
        ":4317",
        ":4318",
        "SQLite + filesystem",
        "default `beaterd` mode requires API-key auth",
    ],
    "first trace": [
        "zero SDK",
        "pip install opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc",
        "python examples/python/five_line_otel.py",
        "http://localhost:3000",
    ],
    "contract regeneration": [
        "sdks/openapi/beater-api.json",
        "cargo xtask regen-spec",
        "scripts/regen-sdks.sh",
        "cargo xtask regen-semconv",
        "scripts/check-contract-sync.sh",
    ],
    "lint and telemetry": [
        "cargo fmt --all",
        "cargo clippy --workspace --all-targets",
        "Self-host telemetry is **opt-out**",
        "BEATER_SELF_HOST_TELEMETRY=1",
        "docs/offline-self-host.md",
    ],
    "pr gate": [
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/workflows/sdk-contract.yml",
        "cannot merge",
    ],
}

WORKFLOW_TOKENS = [
    "name: local-dev-docs",
    "python3 scripts/check-local-dev-docs.py",
    "docs/local-dev.md",
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
        require_tokens(f"docs/local-dev.md {label}", doc, tokens, failures)
    require_tokens(".github/workflows/local-dev-docs.yml", workflow, WORKFLOW_TOKENS, failures)

    if failures:
        print("Local development docs check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Local development docs preserve the bootstrap and contract workflow.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
