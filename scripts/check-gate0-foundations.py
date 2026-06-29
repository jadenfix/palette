#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path


REPO = Path(__file__).resolve().parent.parent
TRAIT_SCAN_CRATES = [
    "crates/beater-alerts",
    "crates/beater-auth",
    "crates/beater-audit",
    "crates/beater-bus",
    "crates/beater-calibration",
    "crates/beater-datasets",
    "crates/beater-eval",
    "crates/beater-experiments",
    "crates/beater-gates",
    "crates/beater-human",
    "crates/beater-judge",
    "crates/beater-replay",
    "crates/beater-search",
    "crates/beater-secrets",
    "crates/beater-store",
    "crates/beater-temporal",
    "crates/beater-usage",
]


def fail(message: str) -> None:
    print(f"Gate 0 foundation contract failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(command: list[str]) -> str:
    result = subprocess.run(
        command,
        cwd=REPO,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if result.stdout:
        print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
    if result.returncode != 0:
        fail(f"command failed: {' '.join(command)}")
    return result.stdout


def run_json(command: list[str]) -> object:
    result = subprocess.run(
        command,
        cwd=REPO,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if result.returncode != 0:
        if result.stdout:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        fail(f"command failed: {' '.join(command)}")
    return json.loads(result.stdout)


def read(path: str) -> str:
    return (REPO / path).read_text()


def rust_block(text: str, start_pattern: str) -> str | None:
    start = re.compile(start_pattern)
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if not start.search(line):
            continue

        block: list[str] = []
        depth = 0
        saw_open = False
        for current in lines[index:]:
            block.append(current)
            if "{" in current:
                saw_open = True
            depth += current.count("{") - current.count("}")
            if saw_open and depth <= 0:
                return "\n".join(block)
    return None


def anyhow_result_aliases(text: str) -> set[str]:
    aliases: set[str] = set()
    for match in re.finditer(r"use\s+anyhow::(?P<body>[^;]+);", text, re.MULTILINE):
        body = match.group("body").strip()
        imports = [body]
        if body.startswith("{") and body.endswith("}"):
            imports = [part.strip() for part in body[1:-1].split(",")]

        for item in imports:
            alias = re.fullmatch(r"Result(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?", item)
            if alias:
                aliases.add(alias.group(1) or "Result")
            if item == "*":
                aliases.add("Result")
    return aliases


def anyhow_error_aliases(text: str) -> set[str]:
    aliases: set[str] = set()
    for match in re.finditer(r"use\s+anyhow::(?P<body>[^;]+);", text, re.MULTILINE):
        body = match.group("body").strip()
        imports = [body]
        if body.startswith("{") and body.endswith("}"):
            imports = [part.strip() for part in body[1:-1].split(",")]

        for item in imports:
            alias = re.fullmatch(r"Error(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?", item)
            if alias:
                aliases.add(alias.group(1) or "Error")
            if item == "*":
                aliases.add("Error")
    return aliases


def anyhow_type_aliases(text: str, result_aliases: set[str], error_aliases: set[str]) -> set[str]:
    aliases: set[str] = set()
    for match in re.finditer(
        r"^\s*(?:pub\s+)?type\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^=]+>)?\s*=\s*([^;]+);",
        text,
        re.MULTILINE,
    ):
        name, target = match.groups()
        if "anyhow::Result" in target or "anyhow::Error" in target:
            aliases.add(name)
            continue
        if any(re.search(rf"\b{re.escape(alias)}\b", target) for alias in result_aliases | error_aliases):
            aliases.add(name)
    return aliases


def cargo_package_names(tree: str) -> set[str]:
    package_names: set[str] = set()
    for line in tree.splitlines():
        match = re.search(r"([A-Za-z0-9_-]+)\s+v\d", line)
        if match:
            package_names.add(match.group(1).lower())
    return package_names


def direct_dependencies(package_name: str) -> list[dict[str, object]]:
    metadata = run_json(["cargo", "metadata", "--no-deps", "--format-version", "1"])
    packages = metadata.get("packages", []) if isinstance(metadata, dict) else []
    for package in packages:
        if isinstance(package, dict) and package.get("name") == package_name:
            dependencies = package.get("dependencies", [])
            return [dep for dep in dependencies if isinstance(dep, dict)]
    fail(f"cargo metadata did not include package {package_name}")


def check_store_crate_has_no_sqlite_dependency() -> None:
    tree = run(["cargo", "tree", "-p", "beater-store", "--format", "{p}"])
    package_hits = sorted(
        name
        for name in cargo_package_names(tree)
        if name == "rusqlite"
        or name == "sqlite"
        or name == "libsqlite3-sys"
        or name == "beater-store-sql"
        or "sqlite" in name
    )

    dep_hits: list[str] = []
    for dependency in direct_dependencies("beater-store"):
        name = str(dependency.get("name", "")).lower()
        features = [str(feature).lower() for feature in dependency.get("features", [])]
        if (
            name in {"rusqlite", "sqlite", "libsqlite3-sys", "sqlx", "beater-store-sql"}
            or "sqlite" in name
            or any("sqlite" in feature for feature in features)
        ):
            dep_hits.append(f"{name} features={features}")

    hits = package_hits + dep_hits
    if hits:
        fail("beater-store must stay trait/types-only; found SQLite/backend dependency: " + ", ".join(hits))


def check_trace_store_conformance_runs_on_two_backends() -> None:
    conformance = read("crates/beater-store-conformance/src/lib.rs")
    for symbol in [
        "assert_trace_store_conformance",
        "assert_metadata_store_conformance",
        "assert_quota_limiter_conformance",
    ]:
        if symbol not in conformance:
            fail(f"beater-store-conformance must define {symbol}")

    sql = read("crates/beater-store-sql/src/lib.rs")
    memory = read("crates/beater-store-memory/src/lib.rs")
    artifact = read("crates/beater-store-obj/src/lib.rs")
    for symbol in [
        "assert_trace_store_conformance",
        "assert_metadata_store_conformance",
        "assert_quota_limiter_conformance",
    ]:
        if symbol not in sql:
            fail(f"beater-store-sql tests must call {symbol}")
        if symbol not in memory:
            fail(f"beater-store-memory tests must call {symbol}")

    for symbol in [
        "impl ArtifactStore for FsArtifactStore",
        "fs_artifact_store_round_trips_and_checks_hash",
        "fs_artifact_store_rejects_corrupt_bytes",
        "sha256_hex(&bytes)",
        "artifact hash mismatch",
        "StoreError::Integrity",
    ]:
        if symbol not in artifact:
            fail(f"beater-store-obj must preserve artifact store proof: {symbol}")

    run(
        [
            "cargo",
            "test",
            "-p",
            "beater-store-conformance",
            "-p",
            "beater-store-memory",
            "-p",
            "beater-store-sql",
            "-p",
            "beater-store-obj",
        ]
    )


def check_metadata_store_boundary() -> None:
    store = read("crates/beater-store/src/lib.rs")
    if "pub trait MetadataStore" not in store:
        fail("beater-store must define MetadataStore")

    api = read("crates/beater-api/src/lib.rs")
    if "metadata: Arc<dyn MetadataStore>" not in api:
        fail("beater-api must store metadata behind Arc<dyn MetadataStore>")
    if "with_metadata(mut self, metadata: Arc<dyn MetadataStore>)" not in api:
        fail("beater-api must expose MetadataStore injection for consumers")


def check_no_anyhow_in_public_traits() -> None:
    trait_start = re.compile(r"^\s*pub\s+trait\s+[A-Za-z0-9_]+")
    failures: list[str] = []
    for crate in TRAIT_SCAN_CRATES:
        for path in sorted((REPO / crate).glob("src/**/*.rs")):
            text = path.read_text()
            result_aliases = anyhow_result_aliases(text)
            error_aliases = anyhow_error_aliases(text)
            type_aliases = anyhow_type_aliases(text, result_aliases, error_aliases)
            in_trait = False
            brace_depth = 0
            trait_line = 0
            for line_no, line in enumerate(text.splitlines(), start=1):
                if not in_trait and trait_start.search(line):
                    in_trait = True
                    trait_line = line_no
                    brace_depth = line.count("{") - line.count("}")
                elif in_trait:
                    brace_depth += line.count("{") - line.count("}")

                result_alias_hit = any(
                    re.search(rf"\b{re.escape(alias)}\s*<", line) for alias in result_aliases
                )
                error_alias_hit = any(
                    re.search(rf"\b{re.escape(alias)}\b", line) for alias in error_aliases
                )
                type_alias_hit = any(
                    re.search(rf"\b{re.escape(alias)}\s*<", line) for alias in type_aliases
                )
                if in_trait and (
                    "anyhow::Result" in line
                    or "anyhow::Error" in line
                    or result_alias_hit
                    or error_alias_hit
                    or type_alias_hit
                    or re.search(r"type\s+Error\s*=\s*.*anyhow", line)
                ):
                    failures.append(
                        f"{path.relative_to(REPO)}:{line_no} inside trait starting at line {trait_line}"
                    )

                if in_trait and brace_depth <= 0 and (line_no != trait_line or "}" in line):
                    in_trait = False
                    trait_line = 0

    if failures:
        fail("public storage/eval trait methods must use typed errors, not anyhow:\n  " + "\n  ".join(failures))


def check_core_schema_clock_boundary() -> None:
    core = read("crates/beater-core/src/lib.rs")
    for symbol in [
        "pub trait Clock",
        "pub struct SystemClock",
        "impl Clock for SystemClock",
        "DateTime::<Utc>::from(SystemTime::now())",
        "pub struct FixedClock",
        "impl Clock for FixedClock",
        "pub enum Currency",
        "pub struct Money",
        "pub fn try_add(&self, other: &Self) -> Result<Self, MoneyError>",
        "pub fn try_sub(&self, other: &Self) -> Result<Self, MoneyError>",
        "CurrencyMismatch",
        "Overflow",
    ]:
        if symbol not in core:
            fail(f"beater-core must preserve foundational primitive: {symbol}")

    failures = []
    for crate in ["crates/beater-core", "crates/beater-schema"]:
        for path in sorted((REPO / crate).glob("src/**/*.rs")):
            system_clock_block = rust_block(path.read_text(), r"^\s*impl\s+Clock\s+for\s+SystemClock\s*\{")
            for line_no, line in enumerate(path.read_text().splitlines(), start=1):
                if "Utc::now()" in line:
                    failures.append(f"{path.relative_to(REPO)}:{line_no}")
                if "SystemTime::now()" in line and (
                    system_clock_block is None or line not in system_clock_block
                ):
                    failures.append(f"{path.relative_to(REPO)}:{line_no}")
    if failures:
        fail(
            "found direct wall-clock access in core/schema outside SystemClock; use Clock injection:\n  "
            + "\n  ".join(failures)
        )

    run(["cargo", "test", "-p", "beater-core", "-p", "beater-schema"])


def check_schema_owns_rollups_and_mappings() -> None:
    schema = read("crates/beater-schema/src/lib.rs")
    for symbol in [
        "pub fn span_matches",
        "pub fn span_summary",
        "pub fn roll_up_runs",
        "pub fn filter_run_summaries",
    ]:
        if symbol not in schema:
            fail(f"beater-schema must own {symbol}")

    for type_name in ["AgentSpanKind", "SpanStatus"]:
        block = rust_block(schema, rf"^\s*impl\s+{type_name}\s*\{{")
        if block is None:
            fail(f"beater-schema must own impl {type_name}")
        for symbol in ["pub fn as_str(&self)", "pub fn parse(value: &str)"]:
            if symbol not in block:
                fail(f"beater-schema impl {type_name} must own {symbol}")

    store = read("crates/beater-store/src/lib.rs")
    for symbol in [
        "pub async fn query_runs_by_materializing_spans",
        "roll_up_runs",
        "filter_run_summaries",
    ]:
        if symbol not in store:
            fail(f"beater-store must own materialized run-query fallback via {symbol}")

    for path in ["crates/beater-store-memory/src/lib.rs", "crates/beater-store-sql/src/lib.rs"]:
        text = read(path)
        # Span mapping must always go through the shared schema helper.
        if "span_summary" not in text:
            fail(f"{path} must use shared span mapping via span_summary")
        # Run rollups must go through a shared run-query helper — either the
        # materializing fallback (query_runs_by_materializing_spans) or the
        # backend-pushdown finalizer (finalize_run_aggregates, used by the SQL
        # backends that aggregate in the database) — never a backend-local
        # reimplementation of the rollup.
        shared_run_helpers = ["query_runs_by_materializing_spans", "finalize_run_aggregates"]
        if not any(symbol in text for symbol in shared_run_helpers):
            fail(
                f"{path} must use a shared run-query helper "
                f"(one of {shared_run_helpers})"
            )
        for forbidden in ["fn roll_up_runs", "fn filter_run_summaries", "fn aggregate_run_status"]:
            if forbidden in text:
                fail(f"{path} must not define backend-local {forbidden}")

    search = read("crates/beater-search/src/lib.rs")
    for symbol in ["span.kind.as_str()", "span.status.as_str()"]:
        if symbol not in search:
            fail(f"beater-search must use schema span mapping via {symbol}")

    archive = read("crates/beater-archive/src/lib.rs")
    for symbol in ["span.kind.as_str()", "span.status.as_str()", "kind.as_str()", "status.as_str()"]:
        if symbol not in archive:
            fail(f"beater-archive must use schema span mapping via {symbol}")

    api = read("crates/beater-api/src/lib.rs")
    for symbol in ["AgentSpanKind::parse(&value)", "SpanStatus::parse(&value)"]:
        if symbol not in api:
            fail(f"beater-api must delegate span filter parsing to schema via {symbol}")


def check_temporal_contract() -> None:
    """Anti-drift guard for the Temporal integration.

    The Temporal history converter must stay pinned to a declared schema, classify
    every pinned event type explicitly (the only wildcard maps to the counted
    ``Unknown`` branch), keep its exhaustiveness test wired, and ship golden fixtures.
    Any divergence here is a hard build failure rather than silent data loss.
    """
    crate = REPO / "crates/beater-temporal"
    lib = crate / "src/lib.rs"
    if not lib.exists():
        fail("beater-temporal crate is missing src/lib.rs")
    text = lib.read_text(encoding="utf-8")

    required_symbols = [
        "TEMPORAL_HISTORY_CONTRACT",
        "KNOWN_EVENT_TYPES",
        "fn classify(",
        "every_known_event_type_is_classified",
    ]
    missing = [symbol for symbol in required_symbols if symbol not in text]
    if missing:
        fail(
            "beater-temporal must keep its anti-drift contract; missing: "
            + ", ".join(missing)
        )

    # The classify() fallback must route unknown events to the counted Unknown branch,
    # never silently absorb them into a mapped class.
    if "_ => Unknown" not in text:
        fail(
            "beater-temporal classify() must end with `_ => Unknown` so new Temporal "
            "event types are counted as unmapped, not silently dropped"
        )

    fixtures = crate / "tests/fixtures"
    if not fixtures.is_dir() or not any(fixtures.iterdir()):
        fail("beater-temporal must ship golden Temporal history fixtures under tests/fixtures")


def main() -> None:
    check_store_crate_has_no_sqlite_dependency()
    check_trace_store_conformance_runs_on_two_backends()
    check_metadata_store_boundary()
    check_no_anyhow_in_public_traits()
    check_core_schema_clock_boundary()
    check_schema_owns_rollups_and_mappings()
    check_temporal_contract()
    print("Gate 0 foundation contract passed.")


if __name__ == "__main__":
    main()
