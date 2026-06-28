#!/usr/bin/env python3
"""Audit the OpenAPI spec for shape consistency across the API surface.

Enforces the conventions that keep the API/MCP/CLI/SDKs nice and in-sync:
  - every operation has a unique camelCase operationId
  - every operation has exactly one resource tag
  - every operation documents a uniform error body (ApiErrorBody) for failures
  - success responses reference a NAMED schema (no anonymous/inline objects)
  - list operations expose pagination (cursor/next_cursor)
Exit 1 on any violation.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import re
import sys
from pathlib import Path
from typing import Any, TextIO

DEFAULT_SPEC = "sdks/openapi/beater-api.json"
CAMEL = re.compile(r"^[a-z][a-zA-Z0-9]*$")
LIST_PAGINATION_EXEMPTIONS = {
    "listAuditEvents",
    "listJudgeLedger",
    "listProviderSecrets",
    "listReviewTasks",
}


@dataclass(frozen=True)
class AuditResult:
    operation_count: int
    unique_operation_id_count: int
    schema_count: int
    violations: list[str]


def load_spec(path: str | Path) -> dict[str, Any]:
    with Path(path).open(encoding="utf-8") as handle:
        return json.load(handle)


def operations(spec: dict[str, Any]) -> list[tuple[str, str, dict[str, Any]]]:
    ops = []
    for path, methods in spec["paths"].items():
        for method, op in methods.items():
            if method in ("parameters",):
                continue
            ops.append((method.upper(), path, op))
    return ops


def audit_spec(spec: dict[str, Any]) -> AuditResult:
    violations = []
    op_ids: dict[str, list[str]] = {}
    ops = operations(spec)

    for method, path, op in ops:
        where = f"{method} {path}"
        oid = op.get("operationId")
        tags = op.get("tags", [])

        if not oid:
            violations.append(f"{where}: missing operationId")
        else:
            if not CAMEL.match(oid):
                violations.append(f"{where}: operationId '{oid}' is not camelCase")
            op_ids.setdefault(oid, []).append(where)

        if len(tags) != 1:
            violations.append(f"{where}: expected exactly 1 tag, got {tags}")

        responses = op.get("responses", {})
        # Health is the only allowed exception to the error-body rule.
        # 422 is used for partial-success (drain-with-dead-letters) and carries a
        # domain payload, not the shared error body, so it's exempt from this rule.
        if oid != "health":
            err_codes = [
                c for c in responses if c.startswith(("4", "5")) and c != "422"
            ]
            if not err_codes:
                violations.append(f"{where}: no documented 4xx/5xx error response")
            for code in err_codes:
                ref = (
                    responses[code]
                    .get("content", {})
                    .get("application/json", {})
                    .get("schema", {})
                    .get("$ref", "")
                )
                if not ref.endswith("/ErrorResponse") and not ref.endswith(
                    "/ApiErrorBody"
                ):
                    violations.append(
                        f"{where}: error {code} body is not the shared error schema (got {ref or 'none'})"
                    )

        # Success response must reference a named schema (no inline/anonymous object).
        for code, body in responses.items():
            if not code.startswith("2"):
                continue
            schema = body.get("content", {}).get("application/json", {}).get(
                "schema", {}
            )
            if not schema:
                continue  # empty 204-style ok
            if (
                "$ref" not in schema
                and schema.get("type") == "object"
                and "properties" in schema
            ):
                violations.append(
                    f"{where}: success {code} uses an inline anonymous object (name it)"
                )

    # operationId uniqueness
    for oid, wheres in op_ids.items():
        if len(wheres) > 1:
            violations.append(f"operationId '{oid}' is duplicated: {wheres}")

    # list operations should paginate
    for method, path, op in ops:
        oid = op.get("operationId", "")
        if oid.startswith("list") and oid not in LIST_PAGINATION_EXEMPTIONS:
            params = {p.get("name") for p in op.get("parameters", [])}
            if "cursor" not in params and "limit" not in params:
                violations.append(
                    f"{method} {path}: list op '{oid}' lacks pagination params"
                )

    return AuditResult(
        operation_count=len(ops),
        unique_operation_id_count=len(op_ids),
        schema_count=len(spec["components"]["schemas"]),
        violations=violations,
    )


def report(result: AuditResult, stream: TextIO = sys.stdout) -> None:
    print(
        f"audited {result.operation_count} operations, "
        f"{result.unique_operation_id_count} unique operationIds, "
        f"{result.schema_count} schemas",
        file=stream,
    )
    if result.violations:
        print(f"\n{len(result.violations)} CONSISTENCY VIOLATIONS:", file=stream)
        for violation in result.violations:
            print(f"  - {violation}", file=stream)
        return
    print("PASS: API shapes are consistent", file=stream)


def main(argv: list[str] | None = None, stream: TextIO = sys.stdout) -> int:
    args = sys.argv[1:] if argv is None else argv
    spec_path = args[0] if args else DEFAULT_SPEC
    result = audit_spec(load_spec(spec_path))
    report(result, stream=stream)
    return 1 if result.violations else 0


if __name__ == "__main__":
    sys.exit(main())
