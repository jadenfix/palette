#!/usr/bin/env python3
"""Check that migration docs describe the durable backend contracts."""

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parent.parent
MIGRATIONS = ROOT / "migrations"
ROOT_README = MIGRATIONS / "README.md"
BACKENDS = {
    "sqlite": {
        "root": ["local OSS runtime", "--data-dir", "schema-migration metadata"],
        "readme": ["SQLite", "`beaterd`", "_beater_schema_migrations"],
    },
    "postgres": {
        "root": ["transactional metadata", "trace hot-store schema"],
        "readme": ["Postgres", "TraceStore", "conformance"],
    },
    "clickhouse": {
        "root": ["scale-oriented trace/raw-event schema", "tenant-leading sort"],
        "readme": ["ClickHouse", "tenant", "project"],
    },
}


def contains_all(label: str, text: str, needles: list[str]) -> list[str]:
    return [f"{label}: missing {needle!r}" for needle in needles if needle not in text]


def sql_files(backend: str) -> list[Path]:
    return sorted((MIGRATIONS / backend).glob("*.sql"))


def check_backend_dirs() -> list[str]:
    failures = []
    root_text = ROOT_README.read_text(encoding="utf-8")

    failures.extend(
        contains_all(
            "root migrations README",
            root_text,
            [
                "durable schema contracts",
                "SQLite files under\n`--data-dir`",
                "Postgres and ClickHouse migrations",
                "`TraceStore`",
            ],
        )
    )

    for backend, needles in BACKENDS.items():
        backend_dir = MIGRATIONS / backend
        readme = backend_dir / "README.md"
        if not backend_dir.is_dir():
            failures.append(f"{backend}/: directory is missing")
            continue
        if not readme.is_file():
            failures.append(f"{backend}/README.md is missing")
        else:
            failures.extend(
                contains_all(
                    f"{backend}/README.md",
                    readme.read_text(encoding="utf-8"),
                    needles["readme"],
                )
            )
        if f"`{backend}/`" not in root_text:
            failures.append(f"root migrations README does not list `{backend}/`")
        failures.extend(contains_all(f"root README `{backend}/` entry", root_text, needles["root"]))
        if not sql_files(backend):
            failures.append(f"{backend}/ has no SQL migration files")

    return failures


def check_clickhouse_sort_keys() -> list[str]:
    failures = []
    readme = (MIGRATIONS / "clickhouse" / "README.md").read_text(encoding="utf-8")
    sql = "\n".join(path.read_text(encoding="utf-8") for path in sql_files("clickhouse"))

    failures.extend(
        contains_all(
            "clickhouse docs/sql",
            readme + "\n" + sql,
            ["tenant", "project", "ORDER BY", "TTL"],
        )
    )

    order_by_keys = re.findall(r"ORDER BY\s*\(([^)]*)\)", sql, flags=re.IGNORECASE | re.MULTILINE)
    if not order_by_keys:
        failures.append("clickhouse SQL has no ORDER BY keys")
        return failures

    for keys in order_by_keys:
        columns = [column.strip().split()[0] for column in keys.split(",")]
        if columns[:2] != ["tenant_id", "project_id"]:
            failures.append(
                "clickhouse ORDER BY key must start with tenant_id, project_id; "
                f"found {', '.join(columns[:2]) or '<empty>'}"
            )

    return failures


def main() -> int:
    failures = check_backend_dirs()
    failures.extend(check_clickhouse_sort_keys())

    if failures:
        print("Migration docs contract drift:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Migration docs cover SQLite, Postgres, and ClickHouse contracts.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
