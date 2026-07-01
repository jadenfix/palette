#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DEPLOY_WORKFLOW = ROOT / ".github/workflows/deploy-dashboard.yml"
VERCEL_CONFIG = ROOT / "web/dashboard/vercel.json"
ALLOWED_PUSH_PATHS = {
    "web/dashboard/**",
    ".github/workflows/deploy-dashboard.yml",
}
FORBIDDEN_DEPLOY_PATTERNS = (
    (re.compile(r"\bbeaterd\b", re.IGNORECASE), "beaterd must not run on Vercel"),
    (re.compile(r"\bbeaterctl\b", re.IGNORECASE), "beaterctl smoke work is not deploy work"),
    (re.compile(r"\bcargo\s+(run|test|build|xtask)\b", re.IGNORECASE), "Rust backend commands do not belong in the Vercel deploy"),
    (re.compile(r"\bdocker(?:-compose|\s+compose|\s|$)", re.IGNORECASE), "Docker backend work does not belong in the Vercel deploy"),
    (re.compile(r"\bcompose\b", re.IGNORECASE), "compose smoke work does not belong in the Vercel deploy"),
    (re.compile(r"\bclickhouse\b", re.IGNORECASE), "ClickHouse writers run outside Vercel"),
    (re.compile(r"\bingest\b", re.IGNORECASE), "long-running ingest runs outside Vercel"),
    (re.compile(r"\bevals?\b", re.IGNORECASE), "eval pools run outside Vercel"),
    (re.compile(r"\breplay\b", re.IGNORECASE), "replay pools run outside Vercel"),
    (re.compile(r"\barchive\b", re.IGNORECASE), "stateful archive work runs outside Vercel"),
    (re.compile(r"\bworkers?\b", re.IGNORECASE), "stateful workers run outside Vercel"),
)


def code_line(line: str) -> str:
    return line.split("#", 1)[0].rstrip()


def yaml_list_after(lines: list[str], key: str) -> list[str]:
    for index, line in enumerate(lines):
        stripped = code_line(line)
        if not stripped.strip().startswith(f"{key}:"):
            continue
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


def check_deploy_workflow(errors: list[str]) -> None:
    if not DEPLOY_WORKFLOW.exists():
        errors.append(f"missing deploy workflow: {DEPLOY_WORKFLOW.relative_to(ROOT)}")
        return

    lines = DEPLOY_WORKFLOW.read_text().splitlines()
    workflow = "\n".join(code_line(line) for line in lines)
    paths = set(yaml_list_after(lines, "paths"))

    if "branches: [main]" not in workflow:
        errors.append("deploy-dashboard must deploy production only from push.branches: [main]")
    if "workflow_dispatch:" not in workflow:
        errors.append("deploy-dashboard should retain manual dispatch for operational redeploys")
    missing_paths = ALLOWED_PUSH_PATHS - paths
    if missing_paths:
        errors.append(f"deploy-dashboard push.paths is missing {sorted(missing_paths)}")
    extra_paths = paths - ALLOWED_PUSH_PATHS
    if extra_paths:
        errors.append(f"deploy-dashboard push.paths must stay dashboard-only, got extra paths {sorted(extra_paths)}")
    if "working-directory: web/dashboard" not in workflow:
        errors.append("deploy-dashboard must default run steps to working-directory: web/dashboard")
    if "vercel build --prod" not in workflow or "vercel deploy --prebuilt --prod" not in workflow:
        errors.append("deploy-dashboard must keep the build/prebuilt production deploy flow")

    for lineno, line in enumerate(lines, start=1):
        stripped = code_line(line).strip()
        if not stripped:
            continue
        for pattern, reason in FORBIDDEN_DEPLOY_PATTERNS:
            if pattern.search(stripped):
                errors.append(f"{DEPLOY_WORKFLOW.relative_to(ROOT)}:{lineno}: {reason}: {stripped}")


def check_vercel_config(errors: list[str]) -> None:
    if not VERCEL_CONFIG.exists():
        errors.append(f"missing Vercel config: {VERCEL_CONFIG.relative_to(ROOT)}")
        return

    try:
        config = json.loads(VERCEL_CONFIG.read_text())
    except json.JSONDecodeError as err:
        errors.append(f"{VERCEL_CONFIG.relative_to(ROOT)} is not valid JSON: {err}")
        return

    if not isinstance(config, dict):
        errors.append(f"{VERCEL_CONFIG.relative_to(ROOT)} must be a JSON object")
        return

    if config.get("framework") != "nextjs":
        errors.append("web/dashboard/vercel.json must keep framework: nextjs")
    if config.get("buildCommand") != "npm run build":
        errors.append("web/dashboard/vercel.json must keep buildCommand: npm run build")

    config_text = json.dumps(config, sort_keys=True)
    for pattern, reason in FORBIDDEN_DEPLOY_PATTERNS:
        if pattern.search(config_text):
            errors.append(f"{VERCEL_CONFIG.relative_to(ROOT)}: {reason}")


def main() -> int:
    errors: list[str] = []
    check_deploy_workflow(errors)
    check_vercel_config(errors)

    if errors:
        print("Vercel dashboard deploy scope drift detected:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print("Vercel dashboard deploy scope is limited to the dashboard build.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
