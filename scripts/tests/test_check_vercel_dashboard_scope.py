#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check-vercel-dashboard-scope.py"


ROOT_VERCEL_JSON = """{
  "$schema": "https://openapi.vercel.sh/vercel.json",
  "git": {
    "deploymentEnabled": {
      "codex/**": false
    }
  }
}
"""


def seed_repo(repo: Path) -> Path:
    script = repo / "scripts" / "check-vercel-dashboard-scope.py"
    script.parent.mkdir(parents=True)
    shutil.copy2(SCRIPT, script)
    script.chmod(0o755)

    (repo / "vercel.json").write_text(ROOT_VERCEL_JSON)
    dashboard = repo / "web" / "dashboard"
    dashboard.mkdir(parents=True)
    (dashboard / "vercel.json").write_text(
        '{"framework":"nextjs","buildCommand":"npm run build"}\n'
    )

    workflow = repo / ".github" / "workflows" / "deploy-dashboard.yml"
    workflow.parent.mkdir(parents=True)
    workflow.write_text(
        """name: deploy-dashboard
on:
  push:
    branches: [main]
    paths:
      - "web/dashboard/**"
      - ".github/workflows/deploy-dashboard.yml"
  workflow_dispatch:
jobs:
  deploy:
    defaults:
      run:
        working-directory: web/dashboard
    steps:
      - run: vercel build --prod
      - run: vercel deploy --prebuilt --prod
"""
    )
    return script


def run_check(mutator=None) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as temp:
        repo = Path(temp) / "repo"
        script = seed_repo(repo)
        if mutator:
            mutator(repo)
        env = os.environ.copy()
        return subprocess.run(
            ["python3", str(script)],
            cwd=repo,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )


def test_vercel_scope_accepts_dashboard_only_deploy_and_codex_auto_deploy_off() -> None:
    result = run_check()

    assert result.returncode == 0, result.stderr
    assert "Vercel dashboard deploy scope is limited" in result.stdout


def test_vercel_scope_rejects_missing_root_codex_deploy_guard() -> None:
    def remove_root_config(repo: Path) -> None:
        (repo / "vercel.json").unlink()

    result = run_check(remove_root_config)

    assert result.returncode == 1
    assert "missing root Vercel config" in result.stderr


def test_vercel_scope_rejects_broad_root_vercel_deploy_config() -> None:
    def broaden_root_config(repo: Path) -> None:
        (repo / "vercel.json").write_text(
            '{"git":{"deploymentEnabled":{"codex/**":false,"main":true}}}\n'
        )

    result = run_check(broaden_root_config)

    assert result.returncode == 1
    assert "root vercel.json must only disable" in result.stderr


def test_vercel_scope_rejects_backend_work_in_dashboard_deploy() -> None:
    def add_backend_deploy_path(repo: Path) -> None:
        workflow = repo / ".github" / "workflows" / "deploy-dashboard.yml"
        workflow.write_text(workflow.read_text() + "      - run: cargo build -p beaterd\n")

    result = run_check(add_backend_deploy_path)

    assert result.returncode == 1
    assert "Rust backend commands do not belong" in result.stderr


def main() -> None:
    for test in (
        test_vercel_scope_accepts_dashboard_only_deploy_and_codex_auto_deploy_off,
        test_vercel_scope_rejects_missing_root_codex_deploy_guard,
        test_vercel_scope_rejects_broad_root_vercel_deploy_config,
        test_vercel_scope_rejects_backend_work_in_dashboard_deploy,
    ):
        test()
    print("vercel-dashboard-scope tests passed")


if __name__ == "__main__":
    main()
