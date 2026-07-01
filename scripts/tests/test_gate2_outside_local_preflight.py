#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import socket
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "gate2-outside-local-preflight.sh"
DEFAULT_PORTS = (8080, 4317, 3000)


def write_executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body)
    path.chmod(0o755)


def seed_fake_bin(bin_dir: Path) -> None:
    simple_commands = ("git", "curl", "ffprobe", "shasum", "tee")
    for name in simple_commands:
        write_executable(
            bin_dir / name,
            f"""#!/usr/bin/env bash
if [ -n "${{BEATER_TEST_LOG:-}}" ]; then
  printf '{name}:%s\\n' "$*" >> "$BEATER_TEST_LOG"
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
cat >/dev/null || true
exit "${BEATER_TEST_PYTHON_RC:-0}"
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
if [ "${1:-}" = "compose" ] && [ "${2:-}" = "version" ]; then
  exit "${BEATER_TEST_DOCKER_COMPOSE_RC:-0}"
fi
if [ "${1:-}" = "context" ] && [ "${2:-}" = "inspect" ]; then
  printf '%s\\n' "${BEATER_TEST_DOCKER_CONTEXT_HOST:-unix:///var/run/docker.sock}"
  exit 0
fi
exit 0
""",
    )


def default_ports_are_free() -> bool:
    for port in DEFAULT_PORTS:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.settimeout(0.2)
            if sock.connect_ex(("127.0.0.1", port)) == 0:
                return False
    return True


def run_preflight(
    env: dict[str, str] | None = None,
    *,
    create_existing_clone: bool = False,
) -> tuple[subprocess.CompletedProcess[str], str]:
    with tempfile.TemporaryDirectory() as temp:
        temp_dir = Path(temp)
        run_dir = temp_dir / "outside-run-parent"
        run_dir.mkdir()
        if create_existing_clone:
            (run_dir / "beater").mkdir()
        script = temp_dir / "gate2-outside-local-preflight.sh"
        shutil.copy2(SCRIPT, script)

        bin_dir = temp_dir / "bin"
        log = temp_dir / "calls.log"
        seed_fake_bin(bin_dir)

        merged_env = os.environ.copy()
        merged_env.update(
            {
                "PATH": f"{bin_dir}{os.pathsep}{merged_env['PATH']}",
                "BEATER_TEST_LOG": str(log),
                "BEATER_GATE2_EXPECTED_COMMIT": "",
                "DOCKER_HOST": "",
            }
        )
        if env:
            merged_env.update(env)

        result = subprocess.run(
            ["bash", str(script)],
            cwd=run_dir,
            env=merged_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        return result, log.read_text() if log.exists() else ""


def test_preflight_passes_with_local_runtime_and_clean_environment() -> None:
    if not default_ports_are_free():
        if os.environ.get("PYTEST_CURRENT_TEST"):
            import pytest

            pytest.skip("default Gate 2 ports are already in use")
        return

    result, calls = run_preflight()

    assert result.returncode == 0, result.stderr
    assert "Gate 2 outside-run local preflight passed." in result.stdout
    assert "docker:info" in calls
    assert "docker:compose version" in calls
    assert "docker:context inspect --format {{.Endpoints.docker.Host}}" in calls


def test_preflight_rejects_remote_docker_host_before_runtime_checks() -> None:
    result, calls = run_preflight({"DOCKER_HOST": "tcp://192.0.2.10:2375"})

    assert result.returncode == 1
    assert "DOCKER_HOST must point at a local Docker daemon" in result.stderr
    assert "docker:info" not in calls


def test_preflight_rejects_existing_beater_directory_before_clone() -> None:
    result, calls = run_preflight(create_existing_clone=True)

    assert result.returncode == 1
    assert "current directory already contains ./beater" in result.stderr
    assert "docker:info" not in calls


def test_preflight_rejects_preconfigured_proof_artifact_paths() -> None:
    result, calls = run_preflight({"BEATER_GATE2_STOPWATCH_PROOF": "custom.md"})

    assert result.returncode == 1
    assert "BEATER_GATE2_STOPWATCH_PROOF must be unset" in result.stderr
    assert "docker:info" not in calls


def main() -> None:
    for test in (
        test_preflight_passes_with_local_runtime_and_clean_environment,
        test_preflight_rejects_remote_docker_host_before_runtime_checks,
        test_preflight_rejects_existing_beater_directory_before_clone,
        test_preflight_rejects_preconfigured_proof_artifact_paths,
    ):
        test()
    print("gate2 outside local preflight tests passed")


if __name__ == "__main__":
    main()
