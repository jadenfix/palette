#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import tempfile
from pathlib import Path
from typing import Callable


BACKENDS = ("sqlite", "postgres", "clickhouse")
FILENAME_RE = re.compile(r"^(?P<version>\d{4})_[a-z0-9][a-z0-9_]*\.sql$")


class CheckError(Exception):
    pass


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def migration_checksum(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def collect_checksums(migrations_root: Path) -> dict[str, str]:
    checksums: dict[str, str] = {}
    errors: list[str] = []

    for backend in BACKENDS:
        backend_dir = migrations_root / backend
        if not backend_dir.is_dir():
            errors.append(f"missing migration directory: {backend_dir}")
            continue

        migrations = sorted(backend_dir.glob("*.sql"))
        if not migrations:
            errors.append(f"{backend}: no SQL migrations found")
            continue

        seen_versions: dict[int, str] = {}
        previous_version = -1
        for path in migrations:
            match = FILENAME_RE.fullmatch(path.name)
            if not match:
                errors.append(f"{backend}/{path.name}: expected filename NNNN_name.sql")
                continue

            version = int(match.group("version"))
            if version in seen_versions:
                errors.append(
                    f"{backend}: duplicate migration version {version:04d} in "
                    f"{seen_versions[version]} and {path.name}"
                )
                continue
            if version <= previous_version:
                errors.append(
                    f"{backend}/{path.name}: migration versions must be strictly increasing"
                )
                continue

            seen_versions[version] = path.name
            previous_version = version
            checksums[f"{backend}/{path.name}"] = migration_checksum(path)

    if errors:
        raise CheckError("\n".join(errors))
    return dict(sorted(checksums.items()))


def read_manifest(path: Path) -> dict[str, str]:
    if not path.exists():
        raise CheckError(f"missing checksum manifest: {path}")
    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as err:
        raise CheckError(f"{path} is not valid JSON: {err}") from err
    if not isinstance(data, dict):
        raise CheckError(f"{path} must be a JSON object")
    invalid = {
        key: value
        for key, value in data.items()
        if not isinstance(key, str)
        or not isinstance(value, str)
        or not value.startswith("sha256:")
        or not re.fullmatch(r"sha256:[0-9a-f]{64}", value)
    }
    if invalid:
        raise CheckError(f"{path} contains invalid checksum entries: {sorted(invalid)}")
    return dict(sorted(data.items()))


def write_manifest(path: Path, checksums: dict[str, str]) -> None:
    path.write_text(json.dumps(checksums, indent=2, sort_keys=True) + "\n")


def manifest_errors(expected: dict[str, str], actual: dict[str, str]) -> list[str]:
    errors: list[str] = []

    for key in sorted(actual.keys() - expected.keys()):
        errors.append(f"{key}: migration is not registered in migrations/checksums.json")
    for key in sorted(expected.keys() - actual.keys()):
        errors.append(f"{key}: manifest entry has no matching migration file")
    for key in sorted(actual.keys() & expected.keys()):
        if actual[key] != expected[key]:
            errors.append(
                f"{key}: checksum drift, expected {expected[key]}, got {actual[key]}"
            )

    return errors


def check_manifest(migrations_root: Path, manifest_path: Path, write: bool) -> None:
    checksums = collect_checksums(migrations_root)
    if write:
        write_manifest(manifest_path, checksums)
        print(f"wrote {manifest_path}")
        return

    expected = read_manifest(manifest_path)
    errors = manifest_errors(expected, checksums)
    if errors:
        joined = "\n".join(f"  - {error}" for error in errors)
        raise CheckError(
            "migration checksum drift detected:\n"
            f"{joined}\n"
            "Run `python3 scripts/check-migration-checksums.py --write` "
            "if the migration change is intentional."
        )

    print(f"Migration checksums match {manifest_path} ({len(checksums)} files).")


def write_sql(root: Path, backend: str, filename: str, body: str) -> None:
    path = root / backend / filename
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body)


def expect_failure(label: str, action: Callable[[], None], expected: str) -> None:
    try:
        action()
    except CheckError as err:
        if expected not in str(err):
            raise AssertionError(f"{label}: expected {expected!r} in {err}") from err
        return
    raise AssertionError(f"{label}: expected failure")


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as temp:
        migrations_root = Path(temp) / "migrations"
        manifest_path = migrations_root / "checksums.json"
        write_sql(migrations_root, "sqlite", "0001_local.sql", "CREATE TABLE local(id TEXT);\n")
        write_sql(
            migrations_root,
            "postgres",
            "0001_initial.sql",
            "CREATE TABLE traces(id TEXT PRIMARY KEY);\n",
        )
        write_sql(
            migrations_root,
            "clickhouse",
            "0001_trace_store.sql",
            "CREATE TABLE traces (id String) ENGINE = MergeTree ORDER BY id;\n",
        )

        check_manifest(migrations_root, manifest_path, write=True)
        check_manifest(migrations_root, manifest_path, write=False)

        write_sql(
            migrations_root,
            "postgres",
            "0001_initial.sql",
            "CREATE TABLE traces(id TEXT PRIMARY KEY, changed TEXT);\n",
        )
        expect_failure(
            "modified SQL",
            lambda: check_manifest(migrations_root, manifest_path, write=False),
            "checksum drift",
        )
        write_sql(
            migrations_root,
            "postgres",
            "0001_initial.sql",
            "CREATE TABLE traces(id TEXT PRIMARY KEY);\n",
        )

        write_sql(
            migrations_root,
            "sqlite",
            "0002_extra.sql",
            "CREATE TABLE extra(id TEXT);\n",
        )
        expect_failure(
            "new migration",
            lambda: check_manifest(migrations_root, manifest_path, write=False),
            "not registered",
        )
        (migrations_root / "sqlite/0002_extra.sql").unlink()

        write_sql(
            migrations_root,
            "clickhouse",
            "0001_duplicate.sql",
            "CREATE TABLE duplicate (id String) ENGINE = MergeTree ORDER BY id;\n",
        )
        expect_failure(
            "duplicate version",
            lambda: collect_checksums(migrations_root),
            "duplicate migration version 0001",
        )

    print("Migration checksum self-test passed.")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check checked-in migration SQL against per-backend SHA-256 manifest."
    )
    parser.add_argument("--write", action="store_true", help="refresh migrations/checksums.json")
    parser.add_argument("--self-test", action="store_true", help="run checker self-tests")
    args = parser.parse_args()

    try:
        if args.self_test:
            run_self_test()
        else:
            root = repo_root()
            check_manifest(root / "migrations", root / "migrations/checksums.json", args.write)
    except CheckError as err:
        print(err, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
