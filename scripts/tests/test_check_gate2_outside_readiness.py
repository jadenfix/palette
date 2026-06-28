#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import shutil
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/check-gate2-outside-readiness.py"
SCRIPTS_DIR = ROOT / "scripts"


def load_module():
    if str(SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(SCRIPTS_DIR))
    spec = importlib.util.spec_from_file_location("check_gate2_outside_readiness", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def copy_compose_files(temp_root: Path) -> None:
    for name in ("docker-compose.yml", "docker-compose.prebuilt.yml"):
        shutil.copy(ROOT / name, temp_root / name)


def expect_failure(label: str, action, expected: str) -> None:
    try:
        action()
    except SystemExit as err:
        if expected not in str(err):
            raise AssertionError(f"{label}: expected {expected!r} in {err}") from err
        return
    raise AssertionError(f"{label}: expected failure")


def with_temp_contract(action) -> None:
    with tempfile.TemporaryDirectory() as temp:
        temp_root = Path(temp)
        copy_compose_files(temp_root)
        module = load_module()
        module.repo_root = lambda: temp_root
        action(module, temp_root)


def test_current_compose_files_pass() -> None:
    with_temp_contract(lambda module, _: module.require_compose_default_path_contract())


def test_dashboard_api_must_stay_on_beaterd() -> None:
    def run(module, temp_root: Path) -> None:
        compose = temp_root / "docker-compose.yml"
        compose.write_text(
            compose.read_text().replace(
                "BEATER_API_BASE_URL: http://beaterd:8080",
                "BEATER_API_BASE_URL: https://api.beater.cloud",
            )
        )
        expect_failure(
            "dashboard backend URL",
            module.require_compose_default_path_contract,
            "BEATER_API_BASE_URL=http://beaterd:8080",
        )

    with_temp_contract(run)


def test_browser_api_must_stay_localhost() -> None:
    def run(module, temp_root: Path) -> None:
        compose = temp_root / "docker-compose.prebuilt.yml"
        compose.write_text(
            compose.read_text().replace(
                "NEXT_PUBLIC_BEATER_API_BASE_URL: http://localhost:${BEATER_HTTP_PORT:-8080}",
                "NEXT_PUBLIC_BEATER_API_BASE_URL: https://beater.vercel.app",
            )
        )
        expect_failure(
            "browser URL",
            module.require_compose_default_path_contract,
            "NEXT_PUBLIC_BEATER_API_BASE_URL=http://localhost",
        )

    with_temp_contract(run)


def test_beaterctl_must_use_local_http_and_otlp() -> None:
    def run(module, temp_root: Path) -> None:
        compose = temp_root / "docker-compose.yml"
        compose.write_text(
            compose.read_text().replace(
                '"http://beaterd:4317"',
                '"https://otel.beater.cloud"',
            )
        )
        expect_failure(
            "beaterctl OTLP URL",
            module.require_compose_default_path_contract,
            "http://beaterd:4317",
        )

    with_temp_contract(run)


def main() -> None:
    for test in (
        test_current_compose_files_pass,
        test_dashboard_api_must_stay_on_beaterd,
        test_browser_api_must_stay_localhost,
        test_beaterctl_must_use_local_http_and_otlp,
    ):
        test()
    print("check-gate2-outside-readiness compose endpoint tests passed.")


if __name__ == "__main__":
    main()
