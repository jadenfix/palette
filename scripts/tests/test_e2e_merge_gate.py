#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "e2e-merge-gate.sh"


def write_executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")
    path.chmod(0o755)


def seed_fake_repo(root: Path, *, docker_available: bool) -> None:
    scripts = root / "scripts"
    scripts.mkdir(parents=True, exist_ok=True)
    shutil.copy2(SCRIPT, scripts / "e2e-merge-gate.sh")

    for name in (
        "check-contract-sync.sh",
        "e2e-sdk-live.sh",
        "e2e-sdk-live-ts.sh",
        "e2e-clients-live.sh",
        "smoke-compose.sh",
    ):
        write_executable(
            scripts / name,
            f"""#!/usr/bin/env bash
echo script:{name}
""",
        )

    bin_dir = root / "bin"
    write_executable(
        bin_dir / "cargo",
        """#!/usr/bin/env bash
echo cargo:$*
""",
    )
    docker_body = """#!/usr/bin/env bash
if [ "$1" = "info" ]; then
"""
    docker_body += "  exit 0\n" if docker_available else "  exit 1\n"
    docker_body += """fi
echo docker:$*
"""
    write_executable(bin_dir / "docker", docker_body)


def run_gate(
    *,
    env: dict[str, str] | None = None,
    docker_available: bool = False,
) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        seed_fake_repo(root, docker_available=docker_available)
        merged_env = os.environ.copy()
        merged_env.update(
            {
                "PATH": f"{root / 'bin'}{os.pathsep}{merged_env['PATH']}",
                "CI": "0",
            }
        )
        if env:
            merged_env.update(env)
        return subprocess.run(
            ["bash", str(root / "scripts" / "e2e-merge-gate.sh")],
            cwd=root,
            env=merged_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )


def test_skip_flags_are_reported_without_full_proof_claim() -> None:
    result = run_gate(
        env={
            "BEATER_DEEP_E2E_SKIP_NATIVE_SDKS": "1",
            "BEATER_DEEP_E2E_SKIP_CLIENT_CONFORMANCE": "1",
            "BEATER_DEEP_E2E_SKIP_COMPOSE": "1",
        }
    )

    assert result.returncode == 0, result.stderr
    assert "Deep merge E2E passed." in result.stdout
    assert "Verified stages:" in result.stdout
    assert "Skipped stages (not verified in this run):" in result.stdout
    assert "native Python/TypeScript SDK live OTLP round trips" in result.stdout
    assert "generated-client live conformance" in result.stdout
    assert "compose/dashboard self-host smoke" in result.stdout
    assert "Full product-loop proof completed" not in result.stdout


def test_full_fake_run_reports_full_product_loop_proof() -> None:
    result = run_gate(docker_available=True)

    assert result.returncode == 0, result.stderr
    assert "Skipped stages" not in result.stdout
    assert (
        "Full product-loop proof completed: contract, MCP, CLI, SDK, and self-host smoke are coherent."
        in result.stdout
    )
    assert "script:smoke-compose.sh" in result.stdout


def test_docker_unavailable_is_reported_as_unverified_when_not_required() -> None:
    result = run_gate(docker_available=False)

    assert result.returncode == 0, result.stderr
    assert "WARN: docker unavailable -- skipping compose/dashboard smoke." in result.stderr
    assert "SDK client regeneration check (docker unavailable)" in result.stdout
    assert "compose/dashboard self-host smoke (docker unavailable)" in result.stdout
    assert "Full product-loop proof completed" not in result.stdout


if __name__ == "__main__":
    for test in (
        test_skip_flags_are_reported_without_full_proof_claim,
        test_full_fake_run_reports_full_product_loop_proof,
        test_docker_unavailable_is_reported_as_unverified_when_not_required,
    ):
        test()
    print("e2e-merge-gate tests passed")
