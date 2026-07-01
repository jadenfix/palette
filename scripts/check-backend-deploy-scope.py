#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEPLOY_WORKFLOW = ROOT / ".github/workflows/deploy-backend.yml"
ALLOWED_PUSH_PATHS = {
    "crates/**",
    "bins/**",
    "Dockerfile",
    ".dockerignore",
    "fly.toml",
    ".github/workflows/deploy-backend.yml",
}
FORBIDDEN_PATTERNS = (
    (re.compile(r"\bpull_request\s*:"), "backend deploy must not run on pull_request"),
    (re.compile(r"\bworkflow_run\s*:"), "backend deploy must not run from another workflow"),
    (re.compile(r"\bschedule\s*:"), "backend deploy must not run on a timer"),
    (re.compile(r"\bweb/dashboard\b"), "dashboard deploy work belongs to deploy-dashboard"),
    (re.compile(r"\bvercel\b", re.IGNORECASE), "Vercel deploy work belongs to deploy-dashboard"),
    (re.compile(r"\bnpm\b"), "frontend package work does not belong in backend deploy"),
    (re.compile(r"\bnode\b"), "frontend Node setup does not belong in backend deploy"),
    (re.compile(r"\bdocker\s+compose\b"), "compose smoke work does not belong in backend deploy"),
    (re.compile(r"\bdocker-compose\b"), "compose smoke work does not belong in backend deploy"),
)
SECRET_REF_RE = re.compile(r"\$\{\{\s*secrets\.([A-Za-z0-9_]+)\s*\}\}")


def code_line(line: str) -> str:
    return line.split("#", 1)[0].rstrip()


def yaml_list_after(lines: list[str], key: str) -> list[str]:
    for index, line in enumerate(lines):
        stripped = code_line(line)
        if stripped.strip() != f"{key}:":
            continue

        indent = len(line) - len(line.lstrip(" "))
        values: list[str] = []
        for child in lines[index + 1 :]:
            child_code = code_line(child)
            if not child_code.strip():
                continue
            child_indent = len(child) - len(child.lstrip(" "))
            if child_indent <= indent:
                break
            match = re.match(r"\s*-\s*[\"']?([^\"'\s]+)[\"']?\s*$", child_code)
            if match:
                values.append(match.group(1))
        return values
    return []


def main() -> int:
    if not DEPLOY_WORKFLOW.exists():
        print(f"missing deploy workflow: {DEPLOY_WORKFLOW.relative_to(ROOT)}", file=sys.stderr)
        return 1

    lines = DEPLOY_WORKFLOW.read_text().splitlines()
    workflow = "\n".join(code_line(line) for line in lines)
    paths = set(yaml_list_after(lines, "paths"))
    errors: list[str] = []

    if "branches: [main]" not in workflow:
        errors.append("deploy-backend must deploy only from push.branches: [main]")
    if "workflow_dispatch:" not in workflow:
        errors.append("deploy-backend should retain manual dispatch for operational redeploys")
    missing_paths = ALLOWED_PUSH_PATHS - paths
    if missing_paths:
        errors.append(f"deploy-backend push.paths is missing {sorted(missing_paths)}")
    extra_paths = paths - ALLOWED_PUSH_PATHS
    if extra_paths:
        errors.append(f"deploy-backend push.paths must stay backend-only, got extra paths {sorted(extra_paths)}")
    if "FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}" not in workflow:
        errors.append("deploy-backend must source only the Fly deploy token from GitHub secrets")
    if "flyctl deploy --remote-only" not in workflow:
        errors.append("deploy-backend must keep Fly remote deploy as the deploy command")
    if 'curl -fsS "https://${url}/health"' not in workflow:
        errors.append("deploy-backend must keep the post-deploy /health smoke check")

    for lineno, line in enumerate(lines, start=1):
        stripped = code_line(line).strip()
        if not stripped:
            continue
        for pattern, reason in FORBIDDEN_PATTERNS:
            if pattern.search(stripped):
                errors.append(f"{DEPLOY_WORKFLOW.relative_to(ROOT)}:{lineno}: {reason}: {stripped}")
        for secret in SECRET_REF_RE.findall(stripped):
            if secret != "FLY_API_TOKEN":
                errors.append(
                    f"{DEPLOY_WORKFLOW.relative_to(ROOT)}:{lineno}: "
                    f"runtime secret {secret} must live in Fly secrets, not GitHub Actions"
                )

    if errors:
        print("Backend deploy scope drift detected:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print("Backend deploy scope is limited to Fly beaterd deploys from green main.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
