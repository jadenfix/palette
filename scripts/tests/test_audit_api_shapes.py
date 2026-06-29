#!/usr/bin/env python3
"""Focused tests for the API shape audit used by the contract drift gate.

The cases mirror ARCHITECTURE.md's contract-source expectations: one OpenAPI
contract feeds SDKs, MCP, CLI, dashboard/docs, and check-contract-sync blocks
shape drift before it can merge.
"""

from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "audit-api-shapes.py"


def load_module():
    spec = importlib.util.spec_from_file_location("audit_api_shapes", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    stdout = io.StringIO()
    stderr = io.StringIO()
    with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
    assert stdout.getvalue() == ""
    assert stderr.getvalue() == ""
    return module


def json_response(schema: str) -> dict:
    return {
        "description": schema,
        "content": {
            "application/json": {
                "schema": {"$ref": f"#/components/schemas/{schema}"}
            }
        },
    }


def valid_contract_spec() -> dict:
    return {
        "openapi": "3.0.3",
        "paths": {
            "/health": {
                "get": {
                    "tags": ["health"],
                    "operationId": "health",
                    "responses": {"200": json_response("HealthResponse")},
                }
            },
            "/v1/traces": {
                "get": {
                    "tags": ["traces"],
                    "operationId": "listTraces",
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "schema": {"type": "integer"},
                        }
                    ],
                    "responses": {
                        "200": json_response("TraceListResponse"),
                        "400": json_response("ApiErrorBody"),
                    },
                },
                "post": {
                    "tags": ["traces"],
                    "operationId": "createTrace",
                    "responses": {
                        "201": json_response("CreateTraceResponse"),
                        "400": json_response("ErrorResponse"),
                        "422": json_response("DrainPartialSuccess"),
                    },
                },
            },
        },
        "components": {
            "schemas": {
                "ApiErrorBody": {},
                "CreateTraceResponse": {},
                "DrainPartialSuccess": {},
                "ErrorResponse": {},
                "HealthResponse": {},
                "TraceListResponse": {},
            }
        },
    }


def violations_for(spec: dict) -> list[str]:
    module = load_module()
    return module.audit_spec(spec).violations


def test_import_has_no_cli_side_effects() -> None:
    module = load_module()

    assert module.DEFAULT_SPEC == "sdks/openapi/beater-api.json"
    assert hasattr(module, "audit_spec")
    assert hasattr(module, "main")


def test_accepts_contract_shapes_required_for_drift_gate() -> None:
    module = load_module()

    result = module.audit_spec(valid_contract_spec())

    assert result.violations == []
    assert result.operation_count == 3
    assert result.unique_operation_id_count == 3
    assert result.schema_count == 6


def test_rejects_operation_identity_and_tag_drift() -> None:
    spec = valid_contract_spec()
    spec["paths"]["/v1/traces"]["get"]["operationId"] = "ListTraces"
    spec["paths"]["/v1/traces"]["get"]["tags"] = ["traces", "runs"]

    violations = violations_for(spec)

    assert "GET /v1/traces: operationId 'ListTraces' is not camelCase" in violations
    assert (
        "GET /v1/traces: expected exactly 1 tag, got ['traces', 'runs']"
    ) in violations


def test_rejects_duplicate_operation_ids() -> None:
    spec = valid_contract_spec()
    spec["paths"]["/v1/traces"]["post"]["operationId"] = "listTraces"
    spec["paths"]["/v1/traces"]["post"]["parameters"] = [{"name": "cursor"}]

    violations = violations_for(spec)

    assert (
        "operationId 'listTraces' is duplicated: "
        "['GET /v1/traces', 'POST /v1/traces']"
    ) in violations


def test_rejects_non_shared_error_and_inline_success_shapes() -> None:
    spec = valid_contract_spec()
    responses = spec["paths"]["/v1/traces"]["get"]["responses"]
    responses["200"] = {
        "description": "anonymous list",
        "content": {
            "application/json": {
                "schema": {
                    "type": "object",
                    "properties": {"items": {"type": "array"}},
                }
            }
        },
    }
    responses["400"] = json_response("ProblemDetails")

    violations = violations_for(spec)

    assert (
        "GET /v1/traces: error 400 body is not the shared error schema "
        "(got #/components/schemas/ProblemDetails)"
    ) in violations
    assert (
        "GET /v1/traces: success 200 uses an inline anonymous object (name it)"
    ) in violations


def test_requires_pagination_for_list_contracts() -> None:
    spec = valid_contract_spec()
    spec["paths"]["/v1/traces"]["get"].pop("parameters")

    violations = violations_for(spec)

    assert (
        "GET /v1/traces: list op 'listTraces' lacks pagination params"
    ) in violations


def test_keeps_documented_list_pagination_exemptions() -> None:
    spec = valid_contract_spec()
    list_op = spec["paths"]["/v1/traces"]["get"]
    list_op["operationId"] = "listAuditEvents"
    list_op.pop("parameters")

    assert violations_for(spec) == []


def test_main_reports_failure_for_cli_gate() -> None:
    module = load_module()
    spec = copy.deepcopy(valid_contract_spec())
    spec["paths"]["/v1/traces"]["get"].pop("parameters")

    with tempfile.TemporaryDirectory() as temp:
        spec_path = Path(temp) / "beater-api.json"
        spec_path.write_text(json.dumps(spec), encoding="utf-8")
        stdout = io.StringIO()

        code = module.main([str(spec_path)], stream=stdout)

    assert code == 1
    assert "audited 3 operations, 3 unique operationIds, 6 schemas" in stdout.getvalue()
    assert "CONSISTENCY VIOLATIONS" in stdout.getvalue()
    assert "list op 'listTraces' lacks pagination params" in stdout.getvalue()


def main() -> None:
    for test in (
        test_import_has_no_cli_side_effects,
        test_accepts_contract_shapes_required_for_drift_gate,
        test_rejects_operation_identity_and_tag_drift,
        test_rejects_duplicate_operation_ids,
        test_rejects_non_shared_error_and_inline_success_shapes,
        test_requires_pagination_for_list_contracts,
        test_keeps_documented_list_pagination_exemptions,
        test_main_reports_failure_for_cli_gate,
    ):
        test()
    print("audit-api-shapes tests passed")


if __name__ == "__main__":
    main()
