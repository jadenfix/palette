#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "check-migration-checksums.py"


def load_module():
    spec = importlib.util.spec_from_file_location("check_migration_checksums", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_sql(root: Path, backend: str, filename: str, body: str) -> None:
    path = root / backend / filename
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body)


def write_minimal_migrations(root: Path) -> None:
    write_sql(root, "sqlite", "0001_local.sql", "CREATE TABLE local(id TEXT);\n")
    write_sql(
        root,
        "postgres",
        "0001_initial.sql",
        "CREATE TABLE traces(id TEXT PRIMARY KEY);\n",
    )
    write_sql(
        root,
        "clickhouse",
        "0001_trace_store.sql",
        "CREATE TABLE traces (id String) ENGINE = MergeTree ORDER BY id;\n",
    )


def expect_check_error(label: str, error_type, action, expected: str) -> None:
    try:
        action()
    except error_type as err:
        if expected not in str(err):
            raise AssertionError(f"{label}: expected {expected!r} in {err}") from err
        return
    raise AssertionError(f"{label}: expected failure")


def test_collects_all_backend_checksums_and_round_trips_manifest() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as temp:
        migrations_root = Path(temp) / "migrations"
        manifest_path = migrations_root / "checksums.json"
        write_minimal_migrations(migrations_root)

        checksums = module.collect_checksums(migrations_root)

        assert list(checksums) == [
            "clickhouse/0001_trace_store.sql",
            "postgres/0001_initial.sql",
            "sqlite/0001_local.sql",
        ]
        assert all(value.startswith("sha256:") for value in checksums.values())

        module.check_manifest(migrations_root, manifest_path, write=True)
        module.check_manifest(migrations_root, manifest_path, write=False)
        assert json.loads(manifest_path.read_text()) == checksums


def test_manifest_errors_report_unregistered_missing_and_drifted_files() -> None:
    module = load_module()
    expected = {
        "postgres/0001_initial.sql": "sha256:" + "0" * 64,
        "sqlite/0001_local.sql": "sha256:" + "1" * 64,
        "sqlite/9999_removed.sql": "sha256:" + "2" * 64,
    }
    actual = {
        "clickhouse/0001_trace_store.sql": "sha256:" + "3" * 64,
        "postgres/0001_initial.sql": "sha256:" + "4" * 64,
        "sqlite/0001_local.sql": "sha256:" + "1" * 64,
    }

    errors = module.manifest_errors(expected, actual)

    assert errors == [
        "clickhouse/0001_trace_store.sql: migration is not registered in migrations/checksums.json",
        "sqlite/9999_removed.sql: manifest entry has no matching migration file",
        "postgres/0001_initial.sql: checksum drift, expected "
        + expected["postgres/0001_initial.sql"]
        + ", got "
        + actual["postgres/0001_initial.sql"],
    ]


def test_rejects_duplicate_migration_versions() -> None:
    with tempfile.TemporaryDirectory() as temp:
        module = load_module()
        migrations_root = Path(temp) / "migrations"
        write_minimal_migrations(migrations_root)
        write_sql(
            migrations_root,
            "sqlite",
            "0001_duplicate.sql",
            "CREATE TABLE duplicate(id TEXT);\n",
        )

        expect_check_error(
            "duplicate migration version",
            module.CheckError,
            lambda: module.collect_checksums(migrations_root),
            "duplicate migration version 0001",
        )


def test_rejects_invalid_manifest_entries() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as temp:
        manifest_path = Path(temp) / "checksums.json"
        manifest_path.write_text(json.dumps({"sqlite/0001_local.sql": "sha1:bad"}))

        expect_check_error(
            "invalid manifest checksum",
            module.CheckError,
            lambda: module.read_manifest(manifest_path),
            "contains invalid checksum entries",
        )


def main() -> None:
    for test in (
        test_collects_all_backend_checksums_and_round_trips_manifest,
        test_manifest_errors_report_unregistered_missing_and_drifted_files,
        test_rejects_duplicate_migration_versions,
        test_rejects_invalid_manifest_entries,
    ):
        test()
    print("check-migration-checksums tests passed.")


if __name__ == "__main__":
    main()
