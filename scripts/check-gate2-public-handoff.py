#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import socket
import subprocess
import tempfile
import time
from pathlib import Path


REMOTE_URL = "https://github.com/jadenfix/beater.git"
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
    "scripts/gate2-outside-run.sh",
    "scripts/gate2-proof.sh",
    "scripts/smoke-compose.sh",
    "scripts/validate-gate2-outside-proof.sh",
]


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run(
    args: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
    quiet: bool = False,
) -> str:
    try:
        output = subprocess.check_output(
            args,
            cwd=cwd,
            env=env,
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
    return "\n".join(f"    {line}" for line in output.splitlines())


def occupied_port_message(port: int, label: str, env_name: str) -> str:
    return (
        f"{port} ({label}; free it rather than setting {env_name})\n"
        f"{port_owner_hint(port)}"
    )


def require_full_run_source(args: argparse.Namespace) -> None:
    if not args.full_run:
        return
    if args.source_url != REMOTE_URL:
        raise SystemExit(
            "--full-run executes the exact scripts/gate2-outside-run.sh path and "
            "is only supported against the canonical public GitHub repo and GHCR images"
        )
    if args.registry_fixture:
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

    missing = [name for name in ["docker", "curl"] if shutil.which(name) is None]
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
    "BEATERD_IMAGE",
    "BEATER_DASHBOARD_IMAGE",
    "BEATER_DASHBOARD_E2E_IMAGE",
    "BEATER_OTEL_PYTHON_IMAGE",
    "BEATER_GATE2_STOPWATCH_PROOF",
    "BEATER_GATE2_RECORD_VIDEO",
    "BEATER_GATE2_RECORD_NOTES",
    "KEEP_BEATER_COMPOSE",
    "COMPOSE_PROJECT_NAME",
]


def clean_outside_env() -> dict[str, str]:
    env = os.environ.copy()
    for name in OUTSIDE_ENV_NAMES:
        env.pop(name, None)
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
    args: argparse.Namespace, expected_commit: str
) -> tuple[Path, tempfile.TemporaryDirectory | None, int]:
    parent, temp_owner = make_clone_parent(args)
    clone_dir = parent / "beater"
    if clone_dir.exists():
        raise SystemExit(f"clone destination already exists: {clone_dir}")

    clone_started_epoch = int(time.time())
    clone_command = ["git", "clone", args.source_url, str(clone_dir)]
    run(clone_command, cwd=parent)

    clone_commit = run(["git", "rev-parse", "HEAD"], cwd=clone_dir, quiet=True)
    if clone_commit != expected_commit:
        raise SystemExit(
            "public handoff clone is not the expected commit: "
            f"expected {expected_commit}, got {clone_commit}"
        )

    clone_branch = run(["git", "branch", "--show-current"], cwd=clone_dir, quiet=True)
    if clone_branch != "main":
        raise SystemExit(f"public handoff clone must be on main, got {clone_branch!r}")

    clone_origin = run(["git", "remote", "get-url", "origin"], cwd=clone_dir, quiet=True)
    if clone_origin != args.source_url:
        raise SystemExit(
            f"public handoff clone origin must be {args.source_url!r}, got {clone_origin!r}"
        )

    return clone_dir, temp_owner, clone_started_epoch


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

    readiness = ["scripts/check-gate2-outside-readiness.py"]
    if args.source_url != REMOTE_URL:
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


def run_cloned_full_run(
    args: argparse.Namespace, clone_dir: Path, clone_started_epoch: int
) -> None:
    if not args.full_run:
        return

    env = clean_outside_env()
    env["BEATER_GATE2_CLONE_STARTED_EPOCH"] = str(clone_started_epoch)
    try:
        run(["scripts/gate2-outside-run.sh"], cwd=clone_dir, env=env)
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
            "After the clean-clone dry-run checks, run the exact outside-run "
            "wrapper in the clone with clone-start timing, then clean up "
            "Compose. This is a maintainer runtime verifier, not outside-person evidence."
        ),
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    expected_commit = args.expected_commit or current_commit()
    preflight_full_run_runtime(args)
    run_local_readiness(args)
    clone_dir, temp_owner, clone_started_epoch = clone_repo(args, expected_commit)
    try:
        run_cloned_checks(args, clone_dir)
        run_cloned_full_run(args, clone_dir, clone_started_epoch)
        mode = "full run" if args.full_run else "clone"
        print(f"Gate 2 public handoff {mode} passed for {expected_commit}: {clone_dir}")
    finally:
        if temp_owner is not None and not args.keep_clone:
            temp_owner.cleanup()


if __name__ == "__main__":
    main()
