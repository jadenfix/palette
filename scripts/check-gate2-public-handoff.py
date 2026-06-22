#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import selectors
import shutil
import socket
import subprocess
import tempfile
import time
from pathlib import Path


REMOTE_URL = "https://github.com/jadenfix/beater.git"
RAW_PREFLIGHT_URL = (
    "https://raw.githubusercontent.com/jadenfix/beater/main/"
    "scripts/gate2-outside-local-preflight.sh"
)
RAW_PUBLIC_PREFLIGHT_COMMAND = f"curl -fsSL {RAW_PREFLIGHT_URL} | bash"
FULL_RUN_PORTS = [
    (8080, "beaterd HTTP", "BEATER_HTTP_PORT"),
    (4317, "OTLP gRPC", "BEATER_OTLP_GRPC_PORT"),
    (3000, "dashboard", "BEATER_DASHBOARD_PORT"),
]
STOPWATCH_COMPOSE_DOWN = [
    "docker",
    "compose",
    "-f",
    "docker-compose.prebuilt.yml",
    "-p",
    "beater-stopwatch",
    "down",
    "-v",
    "--remove-orphans",
]
GATE2_SHELL_SCRIPTS = [
    "scripts/check-openapi-drift.sh",
    "scripts/gate2-compose-stopwatch.sh",
    "scripts/gate2-outside-local-preflight.sh",
    "scripts/gate2-outside-run.sh",
    "scripts/gate2-proof.sh",
    "scripts/smoke-compose.sh",
    "scripts/validate-gate2-outside-proof.sh",
]
MANUAL_CHECKPOINT_MARKER = "Manual outside-run checkpoint:"


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def field_value_from(path: Path, name: str) -> str:
    text = path.read_text()
    matches = re.findall(rf"^- {re.escape(name)}:[ \t]*(.*)$", text, re.MULTILINE)
    if len(matches) != 1:
        raise SystemExit(f"{path} must contain exactly one field: {name}")
    return matches[0].strip().strip("`").strip()


def diagnostic_terminal_excerpt(clone_dir: Path) -> str:
    stopwatch_path = clone_dir / "docs/demos/gate2-compose-stopwatch.md"
    quickstart_dashboard = field_value_from(stopwatch_path, "Quickstart dashboard")
    all_kind_dashboard = field_value_from(stopwatch_path, "All-kind dashboard")
    return (
        "Gate 2 compose stopwatch passed; Browser recording: passed; "
        f"Quickstart dashboard: {quickstart_dashboard}; "
        f"All-kind dashboard: {all_kind_dashboard}"
    )


def run(
    args: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
    quiet: bool = False,
    input_text: str | None = None,
) -> str:
    try:
        output = subprocess.check_output(
            args,
            cwd=cwd,
            env=env,
            input=input_text,
            stderr=subprocess.STDOUT,
            text=True,
        )
    except subprocess.CalledProcessError as err:
        raise SystemExit(
            f"command failed in {cwd}: {' '.join(args)}\n{err.output}"
        ) from err
    if output and not quiet:
        print(output, end="" if output.endswith("\n") else "\n")
    return output.strip()


def run_with_manual_checkpoint_confirmation(
    args: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout_seconds: int = 1800,
) -> str:
    process = subprocess.Popen(
        args,
        cwd=cwd,
        env=env,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    assert process.stdout is not None
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    output = bytearray()
    marker = MANUAL_CHECKPOINT_MARKER.encode()
    confirmed = False
    deadline = time.monotonic() + timeout_seconds

    try:
        while True:
            if time.monotonic() > deadline:
                process.kill()
                decoded = output.decode(errors="replace")
                raise SystemExit(
                    f"timed out waiting for {' '.join(args)} in {cwd}\n{decoded}"
                )

            if process.poll() is not None:
                while True:
                    try:
                        chunk = os.read(process.stdout.fileno(), 4096)
                    except OSError:
                        break
                    if not chunk:
                        break
                    output.extend(chunk)
                break

            for key, _ in selector.select(timeout=0.25):
                chunk = os.read(key.fileobj.fileno(), 4096)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                output.extend(chunk)
                if not confirmed and marker in output:
                    assert process.stdin is not None
                    process.stdin.write(b"\n")
                    process.stdin.flush()
                    process.stdin.close()
                    confirmed = True
    finally:
        selector.close()

    returncode = process.wait()
    decoded = output.decode(errors="replace")
    if decoded:
        print(decoded, end="" if decoded.endswith("\n") else "\n")
    if returncode != 0:
        raise SystemExit(f"command failed in {cwd}: {' '.join(args)}\n{decoded}")
    if not confirmed:
        raise SystemExit(
            "diagnostic full-run did not observe the manual quickstart checkpoint; "
            "the wrapper path did not prove the timing-critical browser handoff"
        )
    return decoded.strip()


def current_commit() -> str:
    commit = run(["git", "rev-parse", "HEAD"], cwd=repo_root(), quiet=True)
    if len(commit) != 40 or any(ch not in "0123456789abcdef" for ch in commit):
        raise SystemExit(f"current HEAD is not a lowercase 40-character SHA: {commit!r}")
    return commit


def run_local_readiness(args: argparse.Namespace) -> None:
    if args.skip_local_readiness:
        return
    command = [str(repo_root() / "scripts/check-gate2-outside-readiness.py")]
    if args.registry_fixture:
        command.extend(["--registry-fixture", str(Path(args.registry_fixture).resolve())])
    run(command, cwd=repo_root())


def port_is_free(port: int) -> bool:
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.25):
            return False
    except OSError:
        return True


def port_owner_hint(port: int) -> str:
    if shutil.which("lsof"):
        command = ["lsof", "-nP", f"-iTCP:{port}", "-sTCP:LISTEN"]
    elif shutil.which("ss"):
        command = ["ss", "-ltnp", f"sport = :{port}"]
    else:
        return f"    install lsof or ss to identify the process holding TCP {port}"

    result = subprocess.run(
        command,
        cwd=repo_root(),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    output = result.stdout.strip()
    if not output:
        return f"    no listener details available for TCP {port}"
    details = process_owner_details(port)
    lines = [f"    {line}" for line in output.splitlines()]
    if details:
        lines.extend(f"    {line}" for line in details)
    return "\n".join(lines)


def port_resolution_hint(port: int, label: str, env_name: str) -> str:
    return (
        f"    Stop the process or app listening on TCP {port} before rerunning "
        f"the full handoff. For Gate 2 outside-person evidence, do not set "
        f"{env_name}; the {label} must use the default port."
    )


def process_owner_details(port: int) -> list[str]:
    if not shutil.which("lsof"):
        return []

    result = subprocess.run(
        ["lsof", "-nP", "-t", f"-iTCP:{port}", "-sTCP:LISTEN"],
        cwd=repo_root(),
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    pids = []
    seen = set()
    for line in result.stdout.splitlines():
        pid = line.strip()
        if pid.isdigit() and pid not in seen:
            seen.add(pid)
            pids.append(pid)

    details = []
    for pid in pids:
        command = process_command(pid)
        cwd = process_cwd(pid)
        if command:
            details.append(f"process {pid} command: {command}")
        if cwd:
            details.append(f"process {pid} cwd: {cwd}")
    return details


def process_command(pid: str) -> str | None:
    if not shutil.which("ps"):
        return None
    result = subprocess.run(
        ["ps", "-p", pid, "-o", "command="],
        cwd=repo_root(),
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    command = result.stdout.strip()
    return command or None


def process_cwd(pid: str) -> str | None:
    if not shutil.which("lsof"):
        return None
    result = subprocess.run(
        ["lsof", "-a", "-p", pid, "-d", "cwd", "-Fn"],
        cwd=repo_root(),
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    for line in result.stdout.splitlines():
        if line.startswith("n") and len(line) > 1:
            return line[1:]
    return None


def occupied_port_message(port: int, label: str, env_name: str) -> str:
    return (
        f"{port} ({label}; free it rather than setting {env_name})\n"
        f"{port_owner_hint(port)}\n"
        f"{port_resolution_hint(port, label, env_name)}"
    )


def fixture_full_run_enabled(args: argparse.Namespace) -> bool:
    return (
        args.full_run
        and os.environ.get("BEATER_GATE2_FIXTURE_FULL_RUN") == "1"
        and args.source_url == REMOTE_URL
        and bool(args.registry_fixture)
        and args.skip_local_readiness
    )


def require_full_run_source(args: argparse.Namespace) -> None:
    if not args.full_run:
        return
    if args.source_url != REMOTE_URL:
        raise SystemExit(
            "--full-run executes the exact scripts/gate2-outside-run.sh path and "
            "is only supported against the canonical public GitHub repo and GHCR images"
        )
    if args.registry_fixture and not fixture_full_run_enabled(args):
        raise SystemExit(
            "--full-run verifies canonical public GHCR images and does not support "
            "--registry-fixture"
        )


def docker_endpoint_is_local(endpoint: str) -> bool:
    return (
        not endpoint
        or endpoint == "<no value>"
        or endpoint.startswith("unix://")
        or endpoint.startswith("npipe://")
        or endpoint.startswith("tcp://localhost:")
        or endpoint.startswith("tcp://127.")
        or endpoint.startswith("tcp://[::1]:")
    )


def require_local_docker_host_env() -> None:
    docker_host = os.environ.get("DOCKER_HOST", "")
    if not docker_endpoint_is_local(docker_host):
        raise SystemExit(
            "--full-run requires a local Docker daemon because the browser proof "
            "uses 127.0.0.1; unset DOCKER_HOST or switch to a local Docker context"
        )


def docker_context_endpoint() -> str:
    result = subprocess.run(
        ["docker", "context", "inspect", "--format", "{{.Endpoints.docker.Host}}"],
        cwd=repo_root(),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if result.returncode != 0:
        return ""
    return result.stdout.strip().splitlines()[0] if result.stdout.strip() else ""


def require_local_docker_context() -> None:
    endpoint = docker_context_endpoint()
    if not docker_endpoint_is_local(endpoint):
        raise SystemExit(
            "--full-run requires a local Docker context because the browser proof "
            f"uses 127.0.0.1; current Docker endpoint is {endpoint}"
        )


def cleanup_stopwatch_compose(cwd: Path, *, fatal: bool) -> None:
    result = subprocess.run(
        STOPWATCH_COMPOSE_DOWN,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if result.returncode == 0:
        return
    if fatal:
        raise SystemExit(
            "Gate 2 full-run preflight could not clean the beater-stopwatch "
            f"Compose project before checking ports:\n{result.stdout}"
        )
    print(
        "warning: Gate 2 full-run cleanup failed; clean the "
        "beater-stopwatch Compose project manually if needed"
    )
    if result.stdout:
        print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")


def cleanup_local_stopwatch_compose() -> None:
    cleanup_stopwatch_compose(repo_root(), fatal=True)


def preflight_full_run_runtime(args: argparse.Namespace) -> None:
    if not args.full_run:
        return

    require_full_run_source(args)

    required_commands = ["bash", "curl", "docker", "ffprobe", "git", "python3"]
    missing = [name for name in required_commands if shutil.which(name) is None]
    if shutil.which("shasum") is None and shutil.which("sha256sum") is None:
        missing.append("shasum or sha256sum")
    if missing:
        raise SystemExit(
            "--full-run requires local command(s): " + ", ".join(sorted(missing))
        )

    require_local_docker_host_env()
    run(["docker", "info"], cwd=repo_root(), quiet=True)
    run(["docker", "compose", "version"], cwd=repo_root(), quiet=True)
    require_local_docker_context()
    cleanup_local_stopwatch_compose()

    if fixture_full_run_enabled(args):
        print(
            "fixture full-run mode skips host port socket checks; this is test "
            "coverage only, not outside-person evidence"
        )
        return

    occupied = [
        occupied_port_message(port, label, env_name)
        for port, label, env_name in FULL_RUN_PORTS
        if not port_is_free(port)
    ]
    if occupied:
        raise SystemExit(
            "--full-run requires the outside-person default ports to be free after "
            "cleaning the beater-stopwatch Compose project:\n  "
            + "\n  ".join(occupied)
        )


def run_raw_public_preflight(args: argparse.Namespace) -> None:
    if not args.full_run:
        return
    require_full_run_source(args)
    env = dict(os.environ)
    env["BEATER_GATE2_RAW_PREFLIGHT_PATH"] = env.get("PATH", "")
    shell_command = f'PATH="$BEATER_GATE2_RAW_PREFLIGHT_PATH"; {RAW_PUBLIC_PREFLIGHT_COMMAND}'
    try:
        run(
            ["bash", "-o", "pipefail", "-lc", shell_command],
            cwd=repo_root(),
            env=env,
        )
    except SystemExit as err:
        raise SystemExit(
            "Gate 2 raw public local preflight failed before clone:\n" f"{err}"
        ) from None


OUTSIDE_ENV_NAMES = [
    "BEATER_GATE2_OUTSIDE_RUN_DRY_RUN",
    "BEATER_GATE2_EXPECTED_ORIGIN",
    "BEATER_GATE2_OUTSIDE_WRAPPER",
    "BEATER_GATE2_CLONE_STARTED_EPOCH",
    "BEATER_DASHBOARD_PORT",
    "BEATER_HTTP_PORT",
    "BEATER_OTLP_GRPC_PORT",
    "BEATER_GATE2_REUSE",
    "BEATER_GATE2_LOCAL_BUILD",
    "BEATER_GATE2_PULL_POLICY",
    "BEATER_GATE2_WRITE_PROOF",
    "BEATER_GATE2_BROWSER_PROOF",
    "BEATER_GATE2_RECORD_DEMO",
    "BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS",
    "BEATER_GATE2_RUN_ID",
    "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS",
    "BEATERD_IMAGE",
    "BEATER_DASHBOARD_IMAGE",
    "BEATER_DASHBOARD_E2E_IMAGE",
    "BEATER_OTEL_PYTHON_IMAGE",
    "BEATER_GATE2_STOPWATCH_PROOF",
    "BEATER_GATE2_RECORD_VIDEO",
    "BEATER_GATE2_RECORD_NOTES",
    "BEATER_GATE2_COMPOSE_LOGS",
    "KEEP_BEATER_COMPOSE",
    "COMPOSE_FILE",
    "COMPOSE_PROJECT_NAME",
    "COMPOSE_PROFILES",
    "BEATER_GATE2_FIXTURE_FULL_RUN",
]


def clean_outside_env() -> dict[str, str]:
    env = os.environ.copy()
    for name in OUTSIDE_ENV_NAMES:
        env.pop(name, None)
    for name in list(env):
        if name.startswith("GIT_CONFIG_"):
            env.pop(name, None)
    return env


def public_clone_env(args: argparse.Namespace) -> dict[str, str]:
    env = os.environ.copy()
    if fixture_full_run_enabled(args):
        return env
    for name in list(env):
        if name.startswith("GIT_CONFIG_"):
            env.pop(name, None)
    env["GIT_CONFIG_GLOBAL"] = os.devnull
    env["GIT_CONFIG_SYSTEM"] = os.devnull
    env["GIT_CONFIG_NOSYSTEM"] = "1"
    return env


def make_clone_parent(args: argparse.Namespace) -> tuple[Path, tempfile.TemporaryDirectory | None]:
    if args.clone_parent:
        parent = Path(args.clone_parent).resolve()
        parent.mkdir(parents=True, exist_ok=True)
        return parent, None
    if args.keep_clone:
        return Path(tempfile.mkdtemp(prefix="beater-gate2-public-handoff-")), None
    temp_dir = tempfile.TemporaryDirectory(prefix="beater-gate2-public-handoff-")
    return Path(temp_dir.name), temp_dir


def clone_repo(
    args: argparse.Namespace, expected_commit: str, clone_name: str = "beater"
) -> tuple[Path, tempfile.TemporaryDirectory | None, int]:
    parent, temp_owner = make_clone_parent(args)
    clone_dir = parent / clone_name
    if clone_dir.exists():
        raise SystemExit(f"clone destination already exists: {clone_dir}")

    clone_started_epoch = int(time.time())
    clone_command = ["git", "clone", args.source_url, str(clone_dir)]
    run(clone_command, cwd=parent, env=public_clone_env(args))

    clone_commit = run(["git", "rev-parse", "HEAD"], cwd=clone_dir, quiet=True)
    if clone_commit != expected_commit:
        raise SystemExit(
            "public handoff clone is not the expected commit: "
            f"expected {expected_commit}, got {clone_commit}"
        )

    clone_branch = run(["git", "branch", "--show-current"], cwd=clone_dir, quiet=True)
    if clone_branch != "main":
        raise SystemExit(f"public handoff clone must be on main, got {clone_branch!r}")

    if fixture_full_run_enabled(args):
        run(["git", "config", "remote.origin.url", REMOTE_URL], cwd=clone_dir, quiet=True)
        clone_origin = run(
            ["git", "config", "--get", "remote.origin.url"], cwd=clone_dir, quiet=True
        )
    else:
        clone_origin = run(["git", "remote", "get-url", "origin"], cwd=clone_dir, quiet=True)

    if clone_origin != args.source_url:
        raise SystemExit(
            f"public handoff clone origin must be {args.source_url!r}, got {clone_origin!r}"
        )

    return clone_dir, temp_owner, clone_started_epoch


def require_file_contains(
    clone_dir: Path,
    rel: str,
    snippets: list[str],
    *,
    contract: str,
    normalize_whitespace: bool = False,
) -> None:
    path = clone_dir / rel
    text = path.read_text()
    haystack = re.sub(r"\s+", " ", text) if normalize_whitespace else text
    for snippet in snippets:
        needle = re.sub(r"\s+", " ", snippet) if normalize_whitespace else snippet
        if needle not in haystack:
            raise SystemExit(
                f"{rel} must contain {contract} for outside runners: {snippet!r}"
            )


def require_public_handoff_timing_guard(clone_dir: Path) -> None:
    require_file_contains(
        clone_dir,
        "README.md",
        [
            "As soon as the first `Open this quickstart trace-list URL first:` URL appears",
            "open that filtered trace-list URL",
            "not wait for the script to finish",
            "seconds remaining",
            "5-minute clone-to-click SLO",
            "cleanup hint printed by",
        ],
        contract="quickstart handoff guidance",
        normalize_whitespace=True,
    )
    require_file_contains(
        clone_dir,
        "docs/demos/gate2-outside-person-proof.md",
        [
            "As soon as the first `Open this quickstart trace-list URL first:` URL appears",
            "filtered trace-list URL",
            "do not wait for the script to finish",
            "seconds remaining in the 5-minute clone-to-click SLO",
            "cleanup hint printed",
        ],
        contract="quickstart handoff guidance",
        normalize_whitespace=True,
    )
    require_file_contains(
        clone_dir,
        "scripts/gate2-compose-stopwatch.sh",
        [
            "Manual outside-run checkpoint:\n"
            "  ${remaining}s remain in the 5-minute clone-to-click SLO.\n"
            "  In a normal browser, open the quickstart trace-list URL above first",
            "Outside-run timing-critical browser step:\n"
            "  Open the quickstart trace-list URL above in a normal browser now; do not wait for the script to finish.",
            "Open the quickstart trace-list URL above in a normal browser now",
            "do not wait for the script to finish",
            "${remaining}s remain in the 5-minute clone-to-click SLO",
            'quickstart_list_url="$dashboard_base_url/?tenant=demo&project=demo&environment=local&kind=llm.call&model=gpt-quickstart&release=$gate2_run_id"',
            "open $quickstart_list_url in a normal browser for the quickstart trace list",
            "Direct quickstart trace URL:",
        ],
        contract="quickstart handoff script",
    )
    require_file_contains(
        clone_dir,
        "scripts/gate2-outside-local-preflight.sh",
        [
            "If this is a stale Beater Gate 2 run",
            "docker-compose.prebuilt.yml -p beater-stopwatch down -v --remove-orphans",
            "label=com.docker.compose.project=beater-stopwatch",
        ],
        contract="outside preflight stale-run cleanup guidance",
    )


def run_cloned_checks(args: argparse.Namespace, clone_dir: Path) -> None:
    registry_fixture = str(Path(args.registry_fixture).resolve()) if args.registry_fixture else None

    run(
        [
            "python3",
            "-c",
            (
                "import pathlib, sys\n"
                "for name in sys.argv[1:]:\n"
                "    path = pathlib.Path(name)\n"
                "    compile(path.read_text(), str(path), 'exec')\n"
            ),
            "scripts/check-gate2-outside-readiness.py",
            "scripts/check-gate2-public-handoff.py",
            "scripts/generate-gate2-outside-proof.py",
        ],
        cwd=clone_dir,
    )
    for script in GATE2_SHELL_SCRIPTS:
        run(["bash", "-n", script], cwd=clone_dir)
    require_public_handoff_timing_guard(clone_dir)

    readiness = ["scripts/check-gate2-outside-readiness.py"]
    if args.source_url != REMOTE_URL or fixture_full_run_enabled(args):
        readiness.append("--skip-repo-shape")
    if registry_fixture:
        readiness.extend(["--registry-fixture", registry_fixture])
    run(readiness, cwd=clone_dir)

    env = clean_outside_env()
    env["BEATER_GATE2_OUTSIDE_RUN_DRY_RUN"] = "1"
    env["BEATER_GATE2_EXPECTED_ORIGIN"] = args.source_url
    run(["scripts/gate2-outside-run.sh"], cwd=clone_dir, env=env)


def cleanup_cloned_compose(clone_dir: Path) -> None:
    cleanup_stopwatch_compose(clone_dir, fatal=False)


def run_generated_proof_check(clone_dir: Path, compose_logs_path: Path) -> None:
    proof_path = "docs/demos/gate2-public-handoff-diagnostic-proof.md"
    compose_logs_rel = compose_logs_path.relative_to(clone_dir).as_posix()
    print(
        "Validating generated Gate 2 proof from full-run artifacts "
        "(diagnostic only; not outside-person evidence)."
    )
    run(
        [
            "scripts/generate-gate2-outside-proof.py",
            "--runner-name",
            "Public Handoff Diagnostic",
            "--relationship",
            "maintainer diagnostic verifier; not outside-person evidence",
            "--prior-exposure",
            "none",
            "--machine-os",
            "diagnostic verifier environment",
            "--browser",
            "prebuilt dashboard-e2e browser proof plus normal-browser handoff check",
            "--network-notes",
            "diagnostic verifier path; not outside-person evidence",
            "--llm-observation",
            (
                "diagnostic auto-confirmed the manual checkpoint; not outside-person "
                "evidence; browser proof inspected llm.call prompt, completion, "
                "model, token breakdown, cost, and latency"
            ),
            "--waterfall-observation",
            (
                "diagnostic browser proof opened the all-kind trace; not "
                "outside-person evidence; observed run -> turn -> step -> tool "
                "-> MCP nesting"
            ),
            "--terminal-output-excerpt",
            diagnostic_terminal_excerpt(clone_dir),
            "--compose-logs-saved",
            compose_logs_rel,
            "--preflight-status",
            "passed",
            "--diagnostic-report",
            "--output",
            proof_path,
            "--force",
        ],
        cwd=clone_dir,
    )
    env = clean_outside_env()
    env["BEATER_GATE2_OUTSIDE_PROOF"] = proof_path
    env["BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS"] = "1"
    run(["scripts/validate-gate2-outside-proof.sh", "--diagnostic"], cwd=clone_dir, env=env)
    print("Gate 2 generated proof diagnostic passed.")


def run_cloned_full_run(
    args: argparse.Namespace, clone_dir: Path, clone_started_epoch: int
) -> None:
    if not args.full_run:
        return

    env = clean_outside_env()
    env["BEATER_GATE2_CLONE_STARTED_EPOCH"] = str(clone_started_epoch)
    try:
        print(
            "Auto-confirming the manual quickstart checkpoint for maintainer "
            "diagnostic full-run only; this is not outside-person evidence."
        )
        full_run_output = run_with_manual_checkpoint_confirmation(
            ["scripts/gate2-outside-run.sh"], cwd=clone_dir, env=env
        )
        compose_logs_path = clone_dir / "docs/demos/gate2-public-handoff-diagnostic-compose.log"
        compose_logs_path.write_text(full_run_output)
        run_generated_proof_check(clone_dir, compose_logs_path)
    finally:
        cleanup_cloned_compose(clone_dir)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Verify the Gate 2 outside-run handoff from a clean clone of the public repo."
        )
    )
    parser.add_argument(
        "--source-url",
        default=REMOTE_URL,
        help=f"Git URL to clone for the outside-run handoff. Default: {REMOTE_URL}",
    )
    parser.add_argument(
        "--expected-commit",
        help="Commit SHA the clone must resolve to. Defaults to the current local HEAD.",
    )
    parser.add_argument(
        "--registry-fixture",
        help="Directory containing registry manifest JSON files for offline tests.",
    )
    parser.add_argument(
        "--clone-parent",
        help=(
            "Directory where the verifier should create a beater/ clone. "
            "Defaults to a temporary directory."
        ),
    )
    parser.add_argument(
        "--skip-local-readiness",
        action="store_true",
        help="Skip checking the current checkout first; intended only for fixture tests.",
    )
    parser.add_argument(
        "--keep-clone",
        action="store_true",
        help="Keep the temporary clone after checks pass for manual inspection.",
    )
    parser.add_argument(
        "--full-run",
        action="store_true",
        help=(
            "Preflight the local runtime and raw public preflight pipe before any clone; "
            "after the clean-clone dry-run checks, run the exact "
            "outside-run wrapper in the clone with clone-start timing, then "
            "clean up Compose. This is a maintainer runtime verifier, not "
            "outside-person evidence."
        ),
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    expected_commit = args.expected_commit or current_commit()
    temp_owners: list[tempfile.TemporaryDirectory | None] = []
    preflight_full_run_runtime(args)
    run_raw_public_preflight(args)
    run_local_readiness(args)
    checks_clone_name = "beater-checks" if args.full_run else "beater"
    clone_dir, temp_owner, clone_started_epoch = clone_repo(
        args, expected_commit, checks_clone_name
    )
    temp_owners.append(temp_owner)
    reported_clone_dir = clone_dir
    try:
        run_cloned_checks(args, clone_dir)
        if fixture_full_run_enabled(args):
            mode = "fixture full run"
        elif args.full_run:
            mode = "full run"
        else:
            mode = "clone"
        if args.full_run:
            full_clone_dir, full_temp_owner, full_clone_started_epoch = clone_repo(
                args, expected_commit, "beater"
            )
            temp_owners.append(full_temp_owner)
            reported_clone_dir = full_clone_dir
            run_cloned_full_run(args, full_clone_dir, full_clone_started_epoch)
        else:
            run_cloned_full_run(args, clone_dir, clone_started_epoch)
        print(
            f"Gate 2 public handoff {mode} passed for {expected_commit}: {reported_clone_dir}"
        )
    finally:
        if not args.keep_clone:
            for temp_owner in temp_owners:
                if temp_owner is not None:
                    temp_owner.cleanup()


if __name__ == "__main__":
    main()
