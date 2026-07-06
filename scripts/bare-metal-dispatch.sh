#!/usr/bin/env python3
"""Dispatch bare-metal lane targets via workflow_dispatch."""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
POLICY_INPUT_KEYS = {
    "name",
    "expected_classes",
    "min_memory_gib",
    "min_cpu_threads",
    "min_physical_cores",
    "summary_path",
    "require_bare_metal",
    "require_cuda",
    "require_gpu",
    "require_avx2",
    "run_live_smoke",
    "run_gate2_compose_stopwatch",
    "run_e2e_clients_live",
    "e2e_clients_live_require",
    "check_gate2_readiness",
    "gate2_registry_fixture",
    "emit_bare_metal_recommendations",
    "apply_bare_metal_optimizations",
    "allow_non_main_readiness",
    "allow_dirty_worktree",
}
KNOWN_CLASS_TOKENS = {
    "bare-metal",
    "virtualized",
    "x86_64-linux",
    "amd64-linux",
    "arm64-linux",
    "aarch64-linux",
    "generic-linux",
    "gpu",
    "cuda",
    "memory-pro",
    "high-core",
    "high-physical-core",
    "avx2",
}


def supported_conformance_languages() -> list[str]:
    conformance_root = SCRIPT_DIR.parent / "sdks" / "conformance"
    if not conformance_root.exists():
        return []
    languages: list[str] = []
    for entry in sorted(conformance_root.iterdir()):
        if entry.is_dir() and (entry / "run.sh").is_file():
            languages.append(entry.name.lower())
    return languages


def parse_e2e_requirements(value: str) -> list[str]:
    aliases = {
        "c++": "cpp",
        "cxx": "cpp",
    }
    normalized = []
    for token in value.split(","):
        normalized_token = token.strip().lower()
        if not normalized_token:
            continue
        normalized.append(aliases.get(normalized_token, normalized_token))
    return sorted(set(normalized))


def validate_e2e_requirements(name: str, raw: str, supported: list[str]) -> list[str]:
    if not raw:
        return []
    required = parse_e2e_requirements(raw)
    if not required:
        return []

    supported_set = set(supported)
    unsupported = sorted(set(required) - supported_set)
    if not supported:
        raise SystemExit("no e2e conformance clients found under sdks/conformance")
    if unsupported:
        raise SystemExit(
            f"target {name!r} declares unsupported e2e_clients_live_require values {unsupported}; "
            f"supported values are {supported}"
        )
    return required


def validate_expected_classes(name: str, expected_classes: list[object]) -> None:
    for cls in expected_classes:
        if not isinstance(cls, str):
            raise SystemExit(f"target {name!r} expected_classes must be strings")

        normalized = cls.strip()
        if not normalized:
            raise SystemExit(f"target {name!r} expected_classes contains an empty class token")

        token = normalized[1:] if normalized.startswith("~") else normalized
        if token not in KNOWN_CLASS_TOKENS:
            raise SystemExit(
                f"target {name!r} uses unknown expected class {normalized!r}; "
                f"supported classes are {sorted(KNOWN_CLASS_TOKENS)}"
            )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="dispatch bare-metal policy targets")
    parser.add_argument(
        "--policy",
        default=str(SCRIPT_DIR.parent / "docs/engineering/bare-metal-lane-policy.json"),
        help="path to lane policy JSON",
    )
    parser.add_argument("--target", default="", help="dispatch only this target by name")
    parser.add_argument("--target-index", type=int, help="dispatch a single target by policy index")
    parser.add_argument("--list", action="store_true", help="print targets instead of dispatching")
    parser.add_argument("--json", action="store_true", help="emit JSON payload summary to stdout")
    parser.add_argument("--dry-run", action="store_true", help="print gh commands only")
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="validate policy and print a short summary",
    )
    parser.add_argument("--ref", default="", help="run workflow at target ref")
    parser.add_argument(
        "--workflow",
        default=".github/workflows/bare-metal-e2e-readiness.yml",
        help="workflow path",
    )
    parser.add_argument(
        "--repo-root",
        default=str(SCRIPT_DIR.parent),
        help="directory containing .github/workflows",
    )
    parser.add_argument("--repo", default="", help="optional gh -R owner/repo")
    parser.add_argument("--require-git-auth", action="store_true", help="require git + GH auth checks")
    return parser.parse_args()


def shell_quote(value: str) -> str:
    return shlex.quote(value)


def bool_inputs(value: bool) -> str:
    return "true" if value else "false"


def sanitize_target_name(value: str) -> str:
    safe = re.sub(r"[^a-zA-Z0-9._-]+", "-", value.strip().lower())
    safe = safe.strip("-.")
    return safe or "target"


def to_bool_list(value: object) -> list[bool]:
    return [bool(value)] if isinstance(value, bool) else [False]


def expect_bool(value: object) -> bool:
    if isinstance(value, bool):
        return value
    raise SystemExit(f"policy value must be boolean, got {value!r}")


def expect_str(value: object) -> str:
    if isinstance(value, str):
        return value
    raise SystemExit(f"policy value must be string, got {value!r}")


def expect_int(value: object) -> int:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise SystemExit(f"policy value must be integer, got {value!r}")
    return int(value)


def validate_target(name: str, target: dict[str, object], seen: set[str], supported_languages: list[str]) -> None:
    if not isinstance(target, dict):
        raise SystemExit(f"target {name!r} must be a mapping")
    if name in seen:
        raise SystemExit(f"duplicate target name: {name!r}")
    seen.add(name)

    expected_classes = target.get("expected_classes", [])
    if not isinstance(expected_classes, list) or not expected_classes:
        raise SystemExit(f"target {name!r} requires expected_classes list")
    validate_expected_classes(name, expected_classes)

    for key in (
        "min_memory_gib",
        "min_cpu_threads",
        "min_physical_cores",
    ):
        if key in target:
            if expect_int(target[key]) < 0:
                raise SystemExit(f"target {name!r} key {key!r} must be >= 0")

    for key in (
        "require_bare_metal",
        "require_cuda",
        "require_gpu",
        "require_avx2",
        "run_live_smoke",
        "run_gate2_compose_stopwatch",
        "run_e2e_clients_live",
        "check_gate2_readiness",
        "emit_bare_metal_recommendations",
        "apply_bare_metal_optimizations",
        "allow_non_main_readiness",
        "allow_dirty_worktree",
    ):
        if key in target:
            expect_bool(target[key])

    if "e2e_clients_live_require" in target and target["e2e_clients_live_require"] is not None:
        expect_str(target["e2e_clients_live_require"])
        validate_e2e_requirements(name, str(target["e2e_clients_live_require"]), supported_languages)
    if "summary_path" in target and target["summary_path"] is not None:
        expect_str(target["summary_path"])
    if "gate2_registry_fixture" in target and target["gate2_registry_fixture"] is not None:
        expect_str(target["gate2_registry_fixture"])

    unknown = set(target) - POLICY_INPUT_KEYS
    if unknown:
        raise SystemExit(f"target {name!r} has unknown keys: {sorted(unknown)}")


def build_payloads(policy_path: Path, target_filter: str, target_index: int | None) -> list[dict[str, object]]:
    data = json.loads(policy_path.read_text(encoding="utf-8"))
    targets = data.get("targets", [])
    if not isinstance(targets, list):
        raise SystemExit(f"policy file does not contain targets[]: {policy_path}")

    if target_filter:
        targets = [t for t in targets if str(t.get("name")) == target_filter]
    elif target_index is not None:
        if target_index < 0 or target_index >= len(targets):
            raise SystemExit(f"target-index {target_index} out of range")
        targets = [targets[target_index]]

    payloads: list[dict[str, object]] = []
    seen_targets: set[str] = set()
    supported_languages = supported_conformance_languages()
    for target in targets:
        name = str(target.get("name", "")).strip()
        if not name:
            raise SystemExit(f"target missing required name: {target!r}")
        if not isinstance(target, dict):
            raise SystemExit(f"target entry is not an object: {target!r}")

        validate_target(name, target, seen_targets, supported_languages)
        normalized_requirements = parse_e2e_requirements(str(target.get("e2e_clients_live_require", "")))
        if normalized_requirements:
            normalized_requirements = validate_e2e_requirements(
                name, str(target.get("e2e_clients_live_require", "")), supported_languages
            )

        expected = target["expected_classes"]

        sanitized_name = sanitize_target_name(name)
        summary_override = (
            Path(str(target.get("summary_path", "bare-metal-readiness-summary.json")).strip() or "bare-metal-readiness-summary.json")
            .name
        )
        payloads.append(
            {
                "lane_target_name": name,
                "inputs": {
                    "lane_target_name": sanitized_name,
                    "expected_classes": ",".join(map(str, expected)),
                    "require_bare_metal": bool(target.get("require_bare_metal", True)),
                    "cuda_matrix": to_bool_list(target.get("require_cuda", False)),
                    "gpu_matrix": to_bool_list(target.get("require_gpu", False)),
                    "liveness_matrix": to_bool_list(target.get("run_live_smoke", False)),
                    "min_memory_gib": float(target.get("min_memory_gib", 8)),
                    "min_cpu_threads": int(target.get("min_cpu_threads", 2)),
                    "min_physical_cores": int(target.get("min_physical_cores", 0)),
                    "require_avx2": bool(target.get("require_avx2", False)),
                    "check_gate2_readiness": bool(target.get("check_gate2_readiness", False)),
                    "gate2_registry_fixture": str(target.get("gate2_registry_fixture", "")),
                    "run_gate2_compose_stopwatch": bool(
                        target.get("run_gate2_compose_stopwatch", False)
                    ),
                    "run_e2e_clients_live": bool(target.get("run_e2e_clients_live", False)),
                    "e2e_clients_live_require": ",".join(normalized_requirements),
                    "emit_bare_metal_recommendations": bool(
                        target.get("emit_bare_metal_recommendations", False)
                    ),
                    "apply_bare_metal_optimizations": bool(
                        target.get("apply_bare_metal_optimizations", False)
                    ),
                    "allow_non_main_readiness": bool(target.get("allow_non_main_readiness", True)),
                    "allow_dirty_worktree": bool(target.get("allow_dirty_worktree", False)),
                    "summary_path": summary_override,
                },
            }
        )
    return payloads


def maybe_require_auth() -> None:
    if subprocess.run(["git", "rev-parse", "--is-inside-work-tree"], check=False).returncode != 0:
        raise SystemExit("not a git repository; run here or pass --require-git-auth false")
    if subprocess.run(["gh", "auth", "status"], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode != 0:
        raise SystemExit("gh authentication required; run gh auth login or pass --require-git-auth false")


def dispatch_with_gh(payloads: list[dict[str, object]], args: argparse.Namespace) -> None:
    workflow = args.workflow
    refs = [args.ref] if args.ref else [""]
    repo_root = Path(args.repo_root)
    if not repo_root.exists():
        raise SystemExit(f"repository root not found: {repo_root}")

    for payload in payloads:
        target_name = str(payload["lane_target_name"])
        inputs = payload["inputs"]  # type: ignore[assignment]
        target_slug = str(inputs["lane_target_name"])

        for ref in refs:
            cmd = ["gh", "workflow", "run", workflow]
            if args.repo:
                cmd.extend(["-R", args.repo])
            if ref:
                cmd.extend(["--ref", ref])
            for key, value in inputs.items():
                if isinstance(value, bool):
                    payload_value = bool_inputs(value)
                elif isinstance(value, int):
                    payload_value = str(value)
                elif isinstance(value, float):
                    payload_value = str(value)
                elif isinstance(value, list):
                    payload_value = json.dumps(value)
                else:
                    payload_value = str(value)
                cmd.extend(["-f", f"{key}={payload_value}"])

            cmdline = " ".join(shell_quote(part) for part in cmd)
            if args.dry_run:
                print(f"# dispatch: {target_name}")
                print(cmdline)
                continue

            result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(repo_root))
            if result.returncode != 0:
                print(result.stdout)
                print(result.stderr, file=sys.stderr)
                raise SystemExit(result.returncode)

            print(f"{target_name} ({target_slug}) => {result.stdout.strip() or 'dispatched'}")


def main() -> None:
    args = parse_args()
    policy_path = Path(args.policy)
    if not policy_path.exists():
        raise SystemExit(f"policy file not found: {policy_path}")

    if args.require_git_auth:
        maybe_require_auth()

    payloads = build_payloads(policy_path, args.target, args.target_index)

    if args.validate_only:
        print(json.dumps({"validated": len(payloads), "targets": [p["lane_target_name"] for p in payloads]}, sort_keys=True))
        return

    if not payloads:
        raise SystemExit("no targets matched")

    if args.list:
        if args.json:
            print(json.dumps(payloads, sort_keys=True, indent=2))
            return
        for payload in payloads:
            print(f"target={payload['lane_target_name']}")
            print(json.dumps(payload["inputs"], sort_keys=True))
        return

    if args.json:
        print(json.dumps(payloads, sort_keys=True, indent=2))

    dispatch_with_gh(payloads, args)


if __name__ == "__main__":
    main()
