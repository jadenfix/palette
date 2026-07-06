#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable

sys.dont_write_bytecode = True

REPO_ROOT = Path(__file__).resolve().parent.parent
CANONICAL_BEATER_REMOTES = (
    "https://github.com/jadenfix/beater",
    "https://github.com/jadenfix/beater.git",
    "git@github.com:jadenfix/beater.git",
    "git@github.com:jadenfix/beater",
)
E2E_COMMAND_BY_LANG: dict[str, str] = {
    "python": "python3",
    "typescript": "node",
    "rust": "cargo",
    "go": "go",
    "c": "cmake",
    "cpp": "cmake",
    "java": "mvn",
}
E2E_LANG_ALIASES: dict[str, str] = {
    "c++": "cpp",
    "cxx": "cpp",
}


def supported_conformance_languages() -> list[str]:
    conformance_root = REPO_ROOT / "sdks" / "conformance"
    if not conformance_root.exists():
        return []

    languages: list[str] = []
    for entry in sorted(conformance_root.iterdir()):
        if not entry.is_dir():
            continue
        if (entry / "run.sh").is_file():
            languages.append(entry.name.lower())
    return languages


def parse_conformance_requirements(value: str) -> list[str]:
    tokens: list[str] = []
    for token in value.split(","):
        normalized = token.strip().lower()
        if normalized:
            tokens.append(E2E_LANG_ALIASES.get(normalized, normalized))
    return tokens


def validate_conformance_requirements(raw: str) -> list[str]:
    if not raw:
        return []

    required = parse_conformance_requirements(raw)
    if not required:
        return []

    supported = supported_conformance_languages()
    if not supported:
        fail("no e2e conformance clients found under sdks/conformance")

    required_set = set(required)
    supported_set = set(supported)
    missing = sorted(required_set - supported_set)
    if missing:
        fail(
            "unsupported values in --e2e-clients-live-require: "
            f"{', '.join(missing)}; supported: {', '.join(sorted(supported))}"
        )
    return sorted(set(required))


def required_commands_for_requirements(requirements: list[str]) -> set[str]:
    commands = set()
    for token in requirements:
        command = E2E_COMMAND_BY_LANG.get(token)
        if command is not None:
            commands.add(command)
    return commands


def require_e2e_runtime_commands(requirements: list[str]) -> None:
    required_commands = required_commands_for_requirements(requirements)
    for command in sorted(required_commands):
        require_commands([command])


@dataclass(frozen=True)
class BareMetalProfile:
    hostname: str
    os_name: str
    kernel: str
    architecture: str
    cpu_vendor: str
    cpu_model: str
    cpu_logical_cores: int
    cpu_physical_cores: int
    has_avx2: bool
    has_avx512: bool
    memory_gib: float
    virtualization: str
    gpu_present: bool
    gpu_count: int
    gpu_names: list[str]
    cuda_present: bool
    cuda_driver: str | None
    cuda_runtime_version: str | None
    timestamp_utc: str


def fail(message: str) -> None:
    print(f"bare-metal readiness failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def run_output(
    command: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    timeout: int = 20,
) -> str:
    try:
        result = subprocess.run(
            command,
            cwd=str(cwd) if cwd else None,
            env=os.environ.copy() if env is None else env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            check=True,
            timeout=timeout,
        )
    except FileNotFoundError as err:
        fail(f"required command missing: {command[0]!r}")
        raise SystemExit from err
    except subprocess.CalledProcessError as err:
        output = (err.stdout or "").strip()
        if len(output) > 1600:
            output = output[:1600] + "..."
        fail(f"{command[0]} failed: {output!r}")
    return result.stdout.strip()


def require_commands(commands: Iterable[str]) -> None:
    for command in commands:
        if shutil.which(command) is None:
            fail(f"required command {command!r} not found in PATH")


def normalize_remote(url: str) -> str:
    normalized = url.strip().rstrip("/")
    if normalized.endswith(".git"):
        normalized = normalized[:-4]
    return normalized


def git_output(*args: str) -> str:
    return run_output(["git", *args], cwd=REPO_ROOT)


def parse_cpu_info() -> tuple[str, str, int, int, bool, bool]:
    path = Path("/proc/cpuinfo")
    vendor = "unknown"
    model = "unknown"
    flags: set[str] = set()
    logical_cores = os.cpu_count() or 1
    physical_pairs: set[tuple[str, str]] = set()
    logical_cores_seen: int | None = 0
    cpu_cores_hint: int | None = None

    if path.exists():
        blocks = [block for block in path.read_text(errors="ignore").split("\n\n") if block.strip()]
        for block in blocks:
            fields = {}
            for line in block.splitlines():
                if ":" not in line:
                    continue
                key, value = line.split(":", 1)
                key = key.strip().lower()
                value = value.strip()
                fields[key] = value

            if "vendor_id" in fields and vendor == "unknown":
                vendor = fields["vendor_id"]
            if model == "unknown":
                model = (
                    fields.get("model name")
                    or fields.get("model")
                    or fields.get("cpu model")
                    or model
                )

            if "flags" in fields:
                flags.update(fields["flags"].split())
            if "features" in fields:
                flags.update(fields["features"].split())

            if "processor" in fields:
                logical_cores_seen = (logical_cores_seen or 0) + 1

            if fields.get("cpu cores", "").isdigit():
                cpu_cores_hint = max(cpu_cores_hint or 0, int(fields["cpu cores"]))

            if "physical id" in fields and "core id" in fields:
                physical_pairs.add((fields["physical id"], fields["core id"]))

        if logical_cores_seen:
            logical_cores = logical_cores_seen

    if physical_pairs:
        physical_cores = len(physical_pairs)
    elif cpu_cores_hint and cpu_cores_hint > 0:
        physical_cores = min(cpu_cores_hint, logical_cores)
    else:
        physical_cores = max(1, logical_cores // 2)

    has_avx2 = "avx2" in flags
    has_avx512 = any(flag.startswith("avx512") for flag in flags)
    return (
        vendor,
        model,
        max(1, logical_cores),
        max(1, physical_cores),
        has_avx2,
        has_avx512,
    )


def parse_mem_gib() -> float:
    meminfo = Path("/proc/meminfo")
    if meminfo.exists():
        for line in meminfo.read_text(errors="ignore").splitlines():
            if not line.startswith("MemTotal:"):
                continue
            parts = line.split()
            if len(parts) >= 2 and parts[1].isdigit():
                return round(int(parts[1]) / (1024.0 * 1024.0), 2)

    free_command = shutil.which("free")
    if free_command is None:
        return 0.0

    output = run_output([free_command, "-b"], timeout=10)
    for line in output.splitlines():
        if not line.startswith("Mem:"):
            continue
        parts = line.split()
        if len(parts) >= 2 and parts[1].isdigit():
            return round(int(parts[1]) / (1024.0**3), 2)
    return 0.0


def detect_virtualization() -> str:
    detector = shutil.which("systemd-detect-virt")
    if detector is not None:
        output = run_output([detector, "-c"], timeout=10).lower().strip()
        if output in {"", "none"}:
            return "bare-metal"
        return output

    cgroup = Path("/proc/self/cgroup")
    if cgroup.exists():
        text = cgroup.read_text(errors="ignore").lower()
        if re.search(r"/(docker|lxc|podman|kubepods|libpod)", text):
            return "virtualized"

    return "unknown"


def detect_gpu_profiles() -> tuple[bool, int, list[str], bool, str | None, str | None]:
    nvidia_smi = shutil.which("nvidia-smi")
    if nvidia_smi is None:
        return False, 0, [], False, None, None

    try:
        output = run_output(
            [
                nvidia_smi,
                "--query-gpu=name,driver_version",
                "--format=csv,noheader,nounits",
            ],
            timeout=20,
        )
    except SystemExit:
        return False, 0, [], False, None, None

    if not output:
        return False, 0, [], False, None, None

    names: list[str] = []
    drivers: set[str] = set()
    for line in output.splitlines():
        parts = [part.strip() for part in line.split(",")]
        if not parts:
            continue
        names.append(parts[0])
        if len(parts) > 1 and parts[1]:
            drivers.add(parts[1])

    runtime_version: str | None = None
    if shutil.which("nvcc") is not None:
        try:
            runtime_line = run_output(["nvcc", "--version"], timeout=20)
            match = re.search(r"release\s+([0-9]+\.[0-9]+)", runtime_line)
            if match:
                runtime_version = match.group(1)
        except SystemExit:
            runtime_version = None

    return True, len(names), names, True, sorted(drivers)[0] if drivers else None, runtime_version


def bare_metal_profile() -> BareMetalProfile:
    import platform

    uname = os.uname()
    cpu_vendor, cpu_model, logical_cores, physical_cores, has_avx2, has_avx512 = parse_cpu_info()
    return BareMetalProfile(
        hostname=uname.nodename,
        os_name=f"{uname.sysname} {uname.release}",
        kernel=uname.version,
        architecture=platform.machine(),
        cpu_vendor=cpu_vendor,
        cpu_model=cpu_model,
        cpu_logical_cores=max(1, logical_cores),
        cpu_physical_cores=max(1, physical_cores),
        has_avx2=has_avx2,
        has_avx512=has_avx512,
        memory_gib=parse_mem_gib(),
        virtualization=detect_virtualization(),
        gpu_present=False,
        gpu_count=0,
        gpu_names=[],
        cuda_present=False,
        cuda_driver=None,
        cuda_runtime_version=None,
        timestamp_utc=datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    )


def _augment_cuda(profile: BareMetalProfile) -> BareMetalProfile:
    gpu_present, gpu_count, gpu_names, cuda_present, cuda_driver, runtime_version = (
        detect_gpu_profiles()
    )
    return BareMetalProfile(
        hostname=profile.hostname,
        os_name=profile.os_name,
        kernel=profile.kernel,
        architecture=profile.architecture,
        cpu_vendor=profile.cpu_vendor,
        cpu_model=profile.cpu_model,
        cpu_logical_cores=profile.cpu_logical_cores,
        cpu_physical_cores=profile.cpu_physical_cores,
        has_avx2=profile.has_avx2,
        has_avx512=profile.has_avx512,
        memory_gib=profile.memory_gib,
        virtualization=profile.virtualization,
        gpu_present=gpu_present,
        gpu_count=gpu_count,
        gpu_names=gpu_names,
        cuda_present=cuda_present,
        cuda_driver=cuda_driver,
        cuda_runtime_version=runtime_version,
        timestamp_utc=profile.timestamp_utc,
    )


def classify_hardware(profile: BareMetalProfile) -> str:
    labels: list[str] = []
    if profile.architecture in {"x86_64", "amd64"}:
        labels.append("x86_64-linux")
    elif profile.architecture in {"aarch64", "arm64"}:
        labels.append("arm64-linux")

    if profile.gpu_present:
        labels.append("gpu")
    if profile.cuda_present:
        labels.append("cuda")
    if profile.memory_gib >= 64.0:
        labels.append("memory-pro")
    if profile.cpu_logical_cores >= 16:
        labels.append("high-core")
    if profile.cpu_physical_cores >= 16:
        labels.append("high-physical-core")
    if profile.has_avx2:
        labels.append("avx2")
    if profile.virtualization == "bare-metal":
        labels.append("bare-metal")
    elif profile.virtualization not in {"none", "unknown"}:
        labels.append("virtualized")

    if not labels:
        return "generic-linux"
    return ",".join(sorted(labels))


def check_repo_state(args: argparse.Namespace) -> None:
    if args.skip_repo_shape:
        return

    branch = git_output("branch", "--show-current")
    if not branch and not args.allow_non_main:
        fail("refusing to run with detached HEAD unless --allow-non-main is set")
    if branch != "main" and not args.allow_non_main:
        fail(f"readiness checks must run on main; current branch is {branch!r}")

    origin = normalize_remote(git_output("remote", "get-url", "origin"))
    if origin not in {normalize_remote(candidate) for candidate in CANONICAL_BEATER_REMOTES}:
        fail(
            f"origin must be one of {list(CANONICAL_BEATER_REMOTES)!r}; got {origin!r}"
        )

    if not args.allow_dirty and git_output("status", "--porcelain"):
        fail("repository must be clean; commit or stash local changes before readiness")


def check_requirements(profile: BareMetalProfile, args: argparse.Namespace) -> None:
    if args.min_memory_gib > 0 and profile.memory_gib < args.min_memory_gib:
        fail(f"insufficient memory: {profile.memory_gib} GiB < minimum {args.min_memory_gib} GiB")

    if args.min_cpu_threads > 0 and profile.cpu_logical_cores < args.min_cpu_threads:
        fail(
            f"insufficient logical CPU cores: {profile.cpu_logical_cores} < minimum {args.min_cpu_threads}"
        )

    if args.min_physical_cores > 0 and profile.cpu_physical_cores < args.min_physical_cores:
        fail(
            f"insufficient physical cores: {profile.cpu_physical_cores} < minimum {args.min_physical_cores}"
        )

    if args.require_bare_metal and profile.virtualization != "bare-metal":
        fail(
            f"runner appears virtualized ({profile.virtualization}); this lane is bare-metal only"
        )

    if args.require_cuda and not profile.cuda_present:
        fail("CUDA is required but no CUDA runtime was detected")

    if args.require_gpu and not profile.gpu_present:
        fail("GPU-required profile was requested, but no GPU was detected")

    if args.require_avx2 and not profile.has_avx2:
        fail("profile must support AVX2")


def check_expected_classes(profile: BareMetalProfile, args: argparse.Namespace) -> None:
    classes = {name.strip() for name in args.expected_classes.split(",") if name.strip()}
    if not classes:
        return

    actual = {name for name in classify_hardware(profile).split(",") if name}
    for item in classes:
        if item.startswith("~"):
            if item[1:] in actual:
                fail(f"class {item!r} is excluded by policy, but profile is {sorted(actual)}")
            continue
        if item not in actual:
            fail(f"profile does not match required class {item!r}; actual {sorted(actual)}")


def run_gate_readiness_checks(args: argparse.Namespace) -> None:
    if args.check_gate2_readiness:
        print("running Gate 2 outside-readiness check")
        command = [
            sys.executable,
            "scripts/check-gate2-outside-readiness.py",
            "--skip-repo-shape",
        ]
        if args.allow_non_main:
            command.append("--allow-non-main")
        if args.allow_dirty:
            command.append("--allow-dirty")
        if args.gate2_registry_fixture:
            command.extend(["--registry-fixture", args.gate2_registry_fixture])
        run_output(command, cwd=REPO_ROOT, timeout=120)

    if args.run_gate2_compose_stopwatch:
        print("running Gate 2 compose stopwatch proof")
        stopwatch_env = os.environ.copy()
        stopwatch_env["BEATER_GATE2_WRITE_PROOF"] = "1"
        stopwatch_env["BEATER_GATE2_BROWSER_PROOF"] = "1"
        stopwatch_env["BEATER_GATE2_RECORD_DEMO"] = "1"
        if args.gate2_compose_stopwatch_proof:
            stopwatch_env["BEATER_GATE2_STOPWATCH_PROOF"] = args.gate2_compose_stopwatch_proof
        if args.gate2_compose_stopwatch_compose_log:
            stopwatch_env["BEATER_GATE2_COMPOSE_LOGS"] = args.gate2_compose_stopwatch_compose_log
        if args.gate2_compose_stopwatch_terminal_log:
            stopwatch_env["BEATER_GATE2_TERMINAL_LOG"] = args.gate2_compose_stopwatch_terminal_log
        run_output(
            ["bash", "scripts/gate2-compose-stopwatch.sh"],
            cwd=REPO_ROOT,
            env=stopwatch_env,
            timeout=7200,
        )

    if args.run_e2e_clients_live:
        print("running SDK language conformance clients live checks")
        client_env = os.environ.copy()
        if args.e2e_clients_live_require:
            client_env["BEATER_CONFORMANCE_REQUIRE"] = args.e2e_clients_live_require
        if args.e2e_clients_live_log_dir:
            client_env["BEATER_E2E_LOG_DIR"] = args.e2e_clients_live_log_dir
        run_output(
            ["bash", "scripts/e2e-clients-live.sh"],
            cwd=REPO_ROOT,
            env=client_env,
            timeout=7200,
        )

    if args.run_live_smoke:
        print("running beaterd live smoke check")
        command = [
            "cargo",
            "test",
            "-p",
            "beaterd",
            "--test",
            "live_smoke",
            "--",
            "--test-threads=1",
        ]
        if args.live_smoke_extra:
            command.extend(shlex.split(args.live_smoke_extra))
        run_output(command, cwd=REPO_ROOT, timeout=7200)


def write_summary(
    profile: BareMetalProfile,
    args: argparse.Namespace,
    path: Path | None = None,
) -> dict[str, object]:
    payload = asdict(profile)
    if args.lane_target_name:
        payload["lane_target_name"] = args.lane_target_name
    payload["class"] = classify_hardware(profile)
    payload["checks"] = {
        "lane_target_name": args.lane_target_name,
        "expected_classes": [token.strip() for token in args.expected_classes.split(",") if token.strip()],
        "require_bare_metal": args.require_bare_metal,
        "require_cuda": args.require_cuda,
        "require_gpu": args.require_gpu,
        "require_avx2": args.require_avx2,
        "min_memory_gib": args.min_memory_gib,
        "min_cpu_threads": args.min_cpu_threads,
        "min_physical_cores": args.min_physical_cores,
        "allow_non_main": args.allow_non_main,
        "allow_dirty": args.allow_dirty,
        "check_gate2_readiness": args.check_gate2_readiness,
        "run_gate2_compose_stopwatch": args.run_gate2_compose_stopwatch,
        "gate2_compose_stopwatch_proof": args.gate2_compose_stopwatch_proof,
        "gate2_compose_stopwatch_compose_log": args.gate2_compose_stopwatch_compose_log,
        "gate2_compose_stopwatch_terminal_log": args.gate2_compose_stopwatch_terminal_log,
        "run_live_smoke": args.run_live_smoke,
        "run_e2e_clients_live": args.run_e2e_clients_live,
        "e2e_clients_live_require": args.e2e_clients_live_require,
        "e2e_clients_live_log_dir": args.e2e_clients_live_log_dir,
        "skip_repo_shape": args.skip_repo_shape,
        "head": git_output("rev-parse", "HEAD"),
        "script": str(Path(__file__).name),
    }

        if path is not None:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
            print(f"wrote readiness report: {path}")

            if os.environ.get("GITHUB_STEP_SUMMARY"):
                with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a") as handle:
                    handle.write("# Bare-metal readiness summary\n")
                    if args.lane_target_name:
                        handle.write(f"- lane_target_name: `{args.lane_target_name}`\n")
                    handle.write(f"- class: {payload['class']}\n")
                    handle.write(f"- host: `{payload['hostname']}`\n")
                    handle.write(f"- os: `{payload['os_name']}`\n")
                    handle.write(f"- architecture: `{payload['architecture']}`\n")
                    handle.write(f"- cpu: `{payload['cpu_vendor']} {payload['cpu_model']}`\n")
                    handle.write(
                        f"- cores: `logical {payload['cpu_logical_cores']} / physical {payload['cpu_physical_cores']}`\n"
                    )
                    handle.write(f"- virtualization: `{payload['virtualization']}`\n")
                    handle.write(f"- memory_gib: `{payload['memory_gib']}`\n")
                    handle.write(f"- gpu_present: `{payload['gpu_present']}`\n")
                    handle.write(f"- cuda_present: `{payload['cuda_present']}`\n")

    return payload


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Bare-metal readiness gate for beater")
    parser.add_argument("--lane-target-name", default="", help="Lane target label")
    parser.add_argument(
        "--expected-classes",
        default="bare-metal",
        help="Comma-separated required hardware classes (e.g. bare-metal,cuda,x86_64-linux). Prefix with ~ to reject.",
    )
    parser.add_argument("--require-bare-metal", action="store_true", help="require bare-metal host")
    parser.add_argument("--require-cuda", action="store_true", help="require CUDA runtime")
    parser.add_argument("--require-gpu", action="store_true", help="require at least one GPU")
    parser.add_argument("--require-avx2", action="store_true", help="require AVX2 CPU support")
    parser.add_argument("--min-memory-gib", type=float, default=8.0, help="minimum host memory in GiB")
    parser.add_argument("--min-cpu-threads", type=int, default=2, help="minimum logical CPU count")
    parser.add_argument(
        "--min-physical-cores",
        type=int,
        default=0,
        help="minimum physical CPU core count (0 disables this check)",
    )
    parser.add_argument("--allow-non-main", action="store_true", help="allow non-main branch")
    parser.add_argument("--allow-dirty", action="store_true", help="allow dirty working tree")
    parser.add_argument("--skip-repo-shape", action="store_true", help="skip git/remote/branch checks")
    parser.add_argument("--check-gate2-readiness", action="store_true", help="run check-gate2-outside-readiness.py")
    parser.add_argument(
        "--gate2-registry-fixture",
        help="pass-through registry fixture to check-gate2-outside-readiness.py",
    )
    parser.add_argument(
        "--run-gate2-compose-stopwatch",
        action="store_true",
        help="run Gate 2 compose stopwatch proof",
    )
    parser.add_argument(
        "--gate2-compose-stopwatch-proof",
        default="",
        help="path to Gate 2 compose stopwatch proof output",
    )
    parser.add_argument(
        "--gate2-compose-stopwatch-compose-log",
        default="",
        help="path to Gate 2 compose stopwatch compose logs",
    )
    parser.add_argument(
        "--gate2-compose-stopwatch-terminal-log",
        default="",
        help="path to Gate 2 compose stopwatch terminal logs",
    )
    parser.add_argument("--run-live-smoke", action="store_true", help="run beaterd live smoke tests")
    parser.add_argument(
        "--live-smoke-extra",
        default="",
        help="extra args appended to the live-smoke cargo test command",
    )
    parser.add_argument(
        "--run-e2e-clients-live",
        action="store_true",
        help="run scripts/e2e-clients-live.sh",
    )
    parser.add_argument(
        "--e2e-clients-live-require",
        default="",
        help="comma-separated languages required to pass in scripts/e2e-clients-live.sh",
    )
    parser.add_argument(
        "--e2e-clients-live-log-dir",
        default="",
        help="directory where scripts/e2e-clients-live.sh writes per-language logs",
    )
    parser.add_argument("--summary-json", help="write JSON readiness summary to file")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.min_memory_gib < 0:
        fail("--min-memory-gib must be >= 0")
    if args.min_cpu_threads < 0:
        fail("--min-cpu-threads must be >= 0")
    if args.min_physical_cores < 0:
        fail("--min-physical-cores must be >= 0")
    if args.run_e2e_clients_live:
        normalized_requirements = validate_conformance_requirements(args.e2e_clients_live_require)
        args.e2e_clients_live_require = ",".join(sorted(set(normalized_requirements)))
        if args.e2e_clients_live_log_dir:
            if not Path(args.e2e_clients_live_log_dir).is_absolute():
                fail("--e2e-clients-live-log-dir must be absolute")
        require_e2e_runtime_commands(normalized_requirements)

    require_commands(["git", "python3"])
    if args.check_gate2_readiness:
        require_commands(["docker"])
    if args.run_gate2_compose_stopwatch:
        require_commands(["docker"])
    if args.run_live_smoke:
        require_commands(["cargo"])

    profile = _augment_cuda(bare_metal_profile())
    if not args.skip_repo_shape:
        check_repo_state(args)

    check_requirements(profile, args)
    check_expected_classes(profile, args)
    run_gate_readiness_checks(args)

    output = write_summary(
        profile,
        args,
        Path(args.summary_json) if args.summary_json else None,
    )
    print(json.dumps(output, sort_keys=True, indent=2))
    print("bare-metal readiness gate passed.")


if __name__ == "__main__":
    main()
