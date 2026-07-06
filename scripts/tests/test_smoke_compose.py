#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "smoke-compose.sh"
ALL_KINDS = (
    "agent.run",
    "agent.turn",
    "agent.plan",
    "agent.step",
    "retrieval.query",
    "memory.read",
    "guardrail.check",
    "llm.call",
    "tool.call",
    "mcp.request",
    "memory.write",
    "evaluator.run",
    "human.review",
    "replay.run",
)


def write_executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body)
    path.chmod(0o755)


def seed_fake_bin(bin_dir: Path) -> None:
    write_executable(
        bin_dir / "docker",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'docker:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
exit 0
""",
    )
    write_executable(
        bin_dir / "curl",
        """#!/usr/bin/env bash
if [ -n "${BEATER_TEST_LOG:-}" ]; then
  printf 'curl:%s\\n' "$*" >> "$BEATER_TEST_LOG"
fi
url="${@: -1}"
case "$url" in
  */health)
    printf 'ok'
    ;;
  */openapi.json)
    if [ "${BEATER_TEST_STALE_OPENAPI:-0}" = "1" ]; then
      printf 'missing contract marker'
    else
      printf 'started_after'
    fi
    ;;
  *'/v1/traces/demo?'*)
    printf '{"items":[{"trace_id":"trace-compose-1"}],"model":"gpt-demo"}'
    ;;
  *'trace=trace-compose-1'*)
    printf 'Agent Trace Debugger call-policy-model %s' "${BEATER_TEST_DASHBOARD_KINDS:-}"
    ;;
  *)
    printf 'Agent Trace Debugger'
    ;;
esac
""",
    )


def run_smoke(
    env: dict[str, str] | None = None,
) -> tuple[subprocess.CompletedProcess[str], str]:
    with tempfile.TemporaryDirectory() as temp:
        temp_dir = Path(temp)
        script = temp_dir / "smoke-compose.sh"
        shutil.copy2(SCRIPT, script)

        bin_dir = temp_dir / "bin"
        log = temp_dir / "calls.log"
        seed_fake_bin(bin_dir)

        merged_env = os.environ.copy()
        merged_env.update(
            {
                "PATH": f"{bin_dir}{os.pathsep}{merged_env['PATH']}",
                "BEATER_TEST_LOG": str(log),
                "BEATER_TEST_DASHBOARD_KINDS": " ".join(ALL_KINDS),
            }
        )
        if env:
            merged_env.update(env)

        result = subprocess.run(
            ["bash", str(script)],
            cwd=temp_dir,
            env=merged_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        return result, log.read_text() if log.exists() else ""


def docker_calls(log: str) -> list[str]:
    return [
        line.removeprefix("docker:")
        for line in log.splitlines()
        if line.startswith("docker:")
    ]


def curl_urls(log: str) -> list[str]:
    urls = []
    for line in log.splitlines():
        if line.startswith("curl:"):
            urls.append(line.rsplit(" ", 1)[-1])
    return urls


def test_compose_smoke_runs_expected_services_and_cleans_up() -> None:
    result, log = run_smoke()

    assert result.returncode == 0, result.stderr
    assert "Beater compose smoke passed." in result.stdout
    assert docker_calls(log) == [
        "compose -p beater-smoke up -d --build beaterd dashboard",
        "compose -p beater-smoke run --rm beaterctl",
        "compose -p beater-smoke run --rm otel-python-smoke",
        "compose -p beater-smoke down -v --remove-orphans",
    ]
    assert "http://127.0.0.1:8080/health" in curl_urls(log)
    assert (
        "http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local"
        in curl_urls(log)
    )


def test_keep_compose_skips_cleanup_and_customizes_ports_and_project() -> None:
    result, log = run_smoke(
        {
            "COMPOSE_PROJECT_NAME": "custom-smoke",
            "KEEP_BEATER_COMPOSE": "1",
            "BEATER_HTTP_PORT": "18080",
            "BEATER_DASHBOARD_PORT": "13000",
        }
    )

    assert result.returncode == 0, result.stderr
    calls = docker_calls(log)
    assert calls[:3] == [
        "compose -p custom-smoke up -d --build beaterd dashboard",
        "compose -p custom-smoke run --rm beaterctl",
        "compose -p custom-smoke run --rm otel-python-smoke",
    ]
    assert not any(" down " in call for call in calls)
    urls = curl_urls(log)
    assert "http://127.0.0.1:18080/health" in urls
    assert "http://127.0.0.1:13000/?tenant=demo&project=demo&environment=local" in urls


def test_compose_smoke_failure_still_cleans_up() -> None:
    result, log = run_smoke({"BEATER_TEST_STALE_OPENAPI": "1"})

    assert result.returncode == 1
    assert "Expected 'started_after' in http://127.0.0.1:8080/openapi.json" in result.stderr
    assert (
        "Beater compose smoke failed; capturing compose status and logs before cleanup."
        in result.stderr
    )
    assert docker_calls(log)[-3:] == [
        "compose -p beater-smoke ps",
        "compose -p beater-smoke logs --no-color --timestamps",
        "compose -p beater-smoke down -v --remove-orphans",
    ]


def main() -> None:
    for test in (
        test_compose_smoke_runs_expected_services_and_cleans_up,
        test_keep_compose_skips_cleanup_and_customizes_ports_and_project,
        test_compose_smoke_failure_still_cleans_up,
    ):
        test()
    print("smoke-compose tests passed")


if __name__ == "__main__":
    main()
