#!/usr/bin/env python3
"""Verify the hosted deployment guide preserves the architecture boundary."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs/hosting.md"
WORKFLOW = ROOT / ".github/workflows/hosting-docs.yml"

DOC_TOKENS = {
    "hosted split": [
        "dashboard on **Vercel**",
        "Rust backend (`beaterd`) on **Fly.io**",
        "Vercel can host the dashboard but **not** `beaterd`",
        "persistent process and a writable\nvolume",
        "ARCHITECTURE.md` §3.2",
    ],
    "secret-clean posture": [
        "secret-clean",
        "no secrets are committed",
        "Fly secrets",
        "Vercel encrypted env",
        "GitHub\nActions secrets",
        "Only secret *names* appear in this repo",
    ],
    "strict backend auth and storage": [
        "fly volumes create beater_data",
        "BEATER_PROVIDER_SECRET_KEY",
        "--auth-mode required",
        "Bootstrap the first Admin API key",
        "Do not\nuse the bootstrap Admin key as a standing Vercel credential",
        "minimum scopes the web tier needs",
    ],
    "dashboard env safety": [
        "Root Directory = `web/dashboard`",
        "encrypted, server-side",
        "Do **not** use `NEXT_PUBLIC_*` for any secret",
        "BEATER_API_BASE_URL",
        "BEATER_API_TOKEN",
        "BEATER_API_KEY",
        "BEATER_GATE2_CONFIRMATION_SALT",
    ],
    "deploy workflows": [
        ".github/workflows/deploy-backend.yml",
        ".github/workflows/deploy-dashboard.yml",
        "FLY_API_TOKEN",
        "VERCEL_TOKEN",
        "VERCEL_ORG_ID",
        "VERCEL_PROJECT_ID",
    ],
    "post-deploy smoke": [
        "curl -fsS https://beater-api.fly.dev/health",
        "cargo run -q -p beaterctl",
        "PLAYWRIGHT_BASE_URL",
        "npm run test:e2e",
    ],
}

WORKFLOW_TOKENS = [
    "name: hosting-docs",
    "python3 scripts/check-hosting-docs.py",
    "docs/hosting.md",
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
        require_tokens(f"docs/hosting.md {label}", doc, tokens, failures)
    require_tokens(".github/workflows/hosting-docs.yml", workflow, WORKFLOW_TOKENS, failures)

    if failures:
        print("Hosting docs check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Hosting docs preserve the Vercel/Fly/beaterd deployment boundary.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
