#!/usr/bin/env python3
"""Validate bare-metal readiness summary artifacts."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REQUIRED_FIELDS = {
    "lane_target_name",
    "hostname",
    "architecture",
    "cpu_logical_cores",
    "cpu_physical_cores",
    "memory_gib",
    "virtualization",
    "gpu_present",
    "gpu_count",
    "cuda_present",
    "class",
    "checks",
}
REQUIRED_CHECK_FIELDS = {
    "expected_classes",
    "require_bare_metal",
    "require_cuda",
    "require_gpu",
    "require_avx2",
    "min_memory_gib",
    "min_cpu_threads",
    "min_physical_cores",
    "run_gate2_compose_stopwatch",
    "run_e2e_clients_live",
    "run_live_smoke",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate bare-metal readiness summary JSON")
    parser.add_argument("--summary", required=True, help="Path to bare-metal-readiness-summary.json")
    parser.add_argument("--lane-target-name", default="", help="Expected lane target name")
    parser.add_argument("--expect-class", default="", help="Expected class substring to contain")
    parser.add_argument("--expect-memory-gib", type=float, default=None, help="Assert minimum memory_gib")
    parser.add_argument("--expect-cpu-threads", type=int, default=None, help="Assert minimum logical cores")
    parser.add_argument("--expect-physical-cores", type=int, default=None, help="Assert minimum physical cores")
    parser.add_argument("--require-cuda", action="store_true", help="Require report checks.require_cuda true")
    parser.add_argument("--require-gpu", action="store_true", help="Require report checks.require_gpu true")
    parser.add_argument("--require-bare-metal", action="store_true", help="Require report checks.require_bare_metal true")
    parser.add_argument("--require-avx2", action="store_true", help="Require report checks.require_avx2 true")
    parser.add_argument("--require-gate2", action="store_true", help="Require report checks.run_gate2_compose_stopwatch true")
    parser.add_argument("--require-e2e", action="store_true", help="Require report checks.run_e2e_clients_live true")
    parser.add_argument("--require-live", action="store_true", help="Require report checks.run_live_smoke true")
    parser.add_argument(
        "--expect-e2e-langs",
        default="",
        help="Require these comma-separated languages in checks.e2e_clients_live_require",
    )
    parser.add_argument(
        "--require-e2e-log-dir",
        default="",
        help="Require per-language e2e conformance logs under this directory",
    )
    parser.add_argument("--require-gate2-proof", help="Path to proof artifact for gate2 stopwatch")
    parser.add_argument("--require-gate2-compose-log", help="Path to gate2 compose log artifact")
    parser.add_argument("--require-gate2-terminal-log", help="Path to gate2 terminal log artifact")
    return parser.parse_args()


def fail(message: str) -> None:
    print(f"bare-metal readiness report validation failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_file_exists(path: str, what: str) -> None:
    if not path:
        return
    if not Path(path).exists():
        fail(f"{what} missing: {path}")
    if Path(path).stat().st_size == 0:
        fail(f"{what} is empty: {path}")


def validate(summary: dict[str, object], args: argparse.Namespace) -> None:
    missing = REQUIRED_FIELDS - set(summary)
    if missing:
        fail(f"summary missing keys: {sorted(missing)}")

    checks = summary.get("checks")
    if not isinstance(checks, dict):
        fail("summary.checks missing or invalid")

    missing_checks = REQUIRED_CHECK_FIELDS - set(checks)
    if missing_checks:
        fail(f"summary.checks missing keys: {sorted(missing_checks)}")

    if args.lane_target_name:
        reported_lane = summary.get("lane_target_name", "")
        if str(reported_lane) != args.lane_target_name:
            fail(f"lane_target_name mismatch: {reported_lane!r} != {args.lane_target_name!r}")

    if args.expect_class:
        actual_class = str(summary.get("class", ""))
        if args.expect_class not in actual_class:
            fail(f"class {actual_class!r} does not contain expected {args.expect_class!r}")

    mem = float(summary.get("memory_gib", 0.0))
    if args.expect_memory_gib is not None and mem < args.expect_memory_gib:
        fail(f"memory_gib {mem} < expected {args.expect_memory_gib}")

    logical = int(summary.get("cpu_logical_cores", 0))
    if args.expect_cpu_threads is not None and logical < args.expect_cpu_threads:
        fail(f"cpu_logical_cores {logical} < expected {args.expect_cpu_threads}")

    physical = int(summary.get("cpu_physical_cores", 0))
    if args.expect_physical_cores is not None and physical < args.expect_physical_cores:
        fail(f"cpu_physical_cores {physical} < expected {args.expect_physical_cores}")

    bool_checks = {
        "require_bare_metal": args.require_bare_metal,
        "require_cuda": args.require_cuda,
        "require_gpu": args.require_gpu,
        "require_avx2": args.require_avx2,
        "run_gate2_compose_stopwatch": args.require_gate2,
        "run_e2e_clients_live": args.require_e2e,
        "run_live_smoke": args.require_live,
    }
    for key, required in bool_checks.items():
        if not required:
            continue
        if bool(checks.get(key)) is not True:
            fail(f"checks.{key} expected true")

    validate_file_exists(args.require_gate2_proof, "gate2 proof")
    validate_file_exists(args.require_gate2_compose_log, "gate2 compose log")
    validate_file_exists(args.require_gate2_terminal_log, "gate2 terminal log")

    if args.expect_e2e_langs:
        expected = [token.strip().lower() for token in args.expect_e2e_langs.split(",") if token.strip()]
        reported = [token.strip().lower() for token in str(checks.get("e2e_clients_live_require", "")).split(",") if token.strip()]
        missing_langs = [lang for lang in expected if lang not in reported]
        if missing_langs:
            fail(f"reported e2e require list {reported} does not include expected {missing_langs}")

    if args.require_e2e_log_dir:
        logs_dir = Path(args.require_e2e_log_dir)
        if not logs_dir.is_dir():
            fail(f"e2e log directory not found: {args.require_e2e_log_dir}")
        for lang in [token.strip().lower() for token in str(checks.get("e2e_clients_live_require", "")).split(",") if token.strip()]:
            validate_file_exists(str(logs_dir / f"{lang}-conformance.log"), f"e2e {lang} conformance log")


def main() -> None:
    args = parse_args()
    path = Path(args.summary)
    if not path.exists():
        fail(f"summary file missing: {path}")

    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        fail("summary content is not an object")

    validate(payload, args)

    print(f"validated summary: {path}")


if __name__ == "__main__":
    main()
