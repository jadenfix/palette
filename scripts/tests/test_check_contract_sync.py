#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check-contract-sync.sh"


def write_executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body)
    path.chmod(0o755)


def seed_temp_repo(repo: Path) -> Path:
    scripts = repo / "scripts"
    scripts.mkdir(parents=True)
    shutil.copy2(SCRIPT, scripts / "check-contract-sync.sh")
    write_executable(
        scripts / "regen-sdks.sh",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'regen-sdks:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
exit "${BEATER_TEST_REGEN_SDKS_RC:-0}"
""",
    )
    write_executable(
        scripts / "check-docs-walkthrough.sh",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'check-docs-walkthrough:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
exit "${BEATER_TEST_DOCS_WALKTHROUGH_RC:-0}"
""",
    )
    return scripts / "check-contract-sync.sh"


def seed_fake_bin(bin_dir: Path) -> None:
    write_executable(
        bin_dir / "cargo",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'cargo:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
case "$*" in
  "test -q -p beater-api --test openapi_coverage")
    exit "${BEATER_TEST_CARGO_TEST_RC:-0}"
    ;;
  "xtask regen-semconv")
    exit "${BEATER_TEST_REGEN_SEMCONV_RC:-0}"
    ;;
esac
exit 0
""",
    )
    write_executable(
        bin_dir / "docker",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'docker:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
if [ "${1:-}" = "info" ]; then
  exit "${BEATER_TEST_DOCKER_INFO_RC:-0}"
fi
exit 0
""",
    )
    write_executable(
        bin_dir / "git",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'git:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
if [ "${1:-}" = "diff" ]; then
  exit "${BEATER_TEST_GIT_DIFF_RC:-0}"
fi
exit 0
""",
    )
    write_executable(
        bin_dir / "python3",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'python3:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
exit "${BEATER_TEST_PYTHON_RC:-0}"
""",
    )


def run_contract_check(
    env: dict[str, str] | None = None,
) -> tuple[subprocess.CompletedProcess[str], str]:
    with tempfile.TemporaryDirectory() as temp:
        temp_dir = Path(temp)
        script = seed_temp_repo(temp_dir / "repo")
        bin_dir = temp_dir / "bin"
        log = temp_dir / "calls.log"
        seed_fake_bin(bin_dir)

        merged_env = os.environ.copy()
        merged_env.update(
            {
                "PATH": f"{bin_dir}{os.pathsep}{merged_env['PATH']}",
                "BEATER_TEST_LOG": str(log),
            }
        )
        if env:
            merged_env.update(env)

        result = subprocess.run(
            ["bash", str(script)],
            cwd=temp_dir / "repo",
            env=merged_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        return result, log.read_text() if log.exists() else ""


def test_contract_check_warns_but_passes_when_local_docker_is_unavailable() -> None:
    result, calls = run_contract_check({"BEATER_TEST_DOCKER_INFO_RC": "1"})

    assert result.returncode == 0, result.stderr
    assert "WARN: docker unavailable -- skipping client regen check" in result.stderr
    assert "No drift: API, 7 SDKs, MCP tools, docs, and conventions are all in sync." in result.stdout
    assert "cargo:test -q -p beater-api --test openapi_coverage" in calls
    assert "cargo:xtask regen-semconv" in calls
    assert "check-docs-walkthrough:--dry-run" in calls
    assert "regen-sdks:" not in calls


def test_contract_check_fails_when_sdk_regen_check_reports_drift() -> None:
    result, calls = run_contract_check(
        {
            "BEATER_TEST_DOCKER_INFO_RC": "0",
            "BEATER_TEST_REGEN_SDKS_RC": "1",
        }
    )

    assert result.returncode == 1
    assert "CONTRACT DRIFT DETECTED -- regenerate" in result.stderr
    assert "==> 5/5 docs walkthrough references are current" in result.stdout
    assert "regen-sdks:--check" in calls


def test_contract_check_fails_when_semconv_snapshot_is_stale() -> None:
    result, calls = run_contract_check(
        {
            "BEATER_TEST_DOCKER_INFO_RC": "1",
            "BEATER_TEST_GIT_DIFF_RC": "1",
        }
    )

    assert result.returncode == 1
    assert "conventions.json is stale" in result.stderr
    assert "CONTRACT DRIFT DETECTED -- regenerate" in result.stderr
    assert "git:diff --exit-code -- sdks/semconv/conventions.json" in calls


def main() -> None:
    for test in (
        test_contract_check_warns_but_passes_when_local_docker_is_unavailable,
        test_contract_check_fails_when_sdk_regen_check_reports_drift,
        test_contract_check_fails_when_semconv_snapshot_is_stale,
    ):
        test()
    print("contract-sync tests passed")


if __name__ == "__main__":
    main()
