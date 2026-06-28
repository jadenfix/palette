#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

sys.dont_write_bytecode = True


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts/gate2_proof_contract.py"


def load_module():
    spec = importlib.util.spec_from_file_location("gate2_proof_contract", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_confirmation_code_uses_stable_contract_vector() -> None:
    module = load_module()
    vector = module.GATE2_CONFIRMATION_TEST_VECTOR

    assert (
        module.gate2_confirmation_code(
            vector["salt"],
            vector["trace_id"],
            vector["span_id"],
        )
        == vector["code"]
    )
    assert (
        module.gate2_confirmation_code(
            vector["salt"],
            vector["trace_id"],
            "fedcba9876543210",
        )
        != vector["code"]
    )


def test_gate2_image_helpers_keep_public_ghcr_contract() -> None:
    module = load_module()

    for image_name in module.GATE2_IMAGE_NAMES:
        image = module.gate2_image(image_name)
        assert image.env_var.startswith("BEATER_") or image.env_var == "BEATERD_IMAGE"
        assert module.gate2_image_repo(image_name) == f"ghcr.io/jadenfix/beater/{image_name}"
        assert (
            module.gate2_registry_repository(image_name)
            == f"jadenfix/beater/{image_name}"
        )
        assert (
            module.gate2_image_ref(image_name, "abc123")
            == f"ghcr.io/jadenfix/beater/{image_name}:abc123"
        )
        assert (
            module.gate2_image_digest_prefix(image_name)
            == f"ghcr.io/jadenfix/beater/{image_name}@sha256:"
        )
        assert image.proof_ref_field
        assert image.proof_digest_field


def test_raw_public_preflight_command_pins_expected_commit() -> None:
    module = load_module()
    command = module.raw_public_preflight_command_for_sha("0123456789abcdef")

    assert (
        "https://raw.githubusercontent.com/jadenfix/beater/0123456789abcdef/"
        "scripts/gate2-outside-local-preflight.sh"
    ) in command
    assert 'BEATER_GATE2_EXPECTED_COMMIT="0123456789abcdef"' in command
    assert "/$sha/" not in command


def test_immutable_log_url_requires_actions_run_or_job() -> None:
    module = load_module()

    assert module.is_immutable_log_url("https://github.com/jadenfix/beater/actions/runs/123")
    assert module.is_immutable_log_url(
        "https://github.com/jadenfix/beater/actions/runs/123/job/456"
    )
    assert not module.is_immutable_log_url(
        "https://github.com/jadenfix/beater/actions/workflows/gate2-proof-contract.yml"
    )
    assert not module.is_immutable_log_url("https://github.com/jadenfix/beater/pull/123")


def test_unresolved_argument_rejects_placeholders() -> None:
    module = load_module()

    assert module.is_unresolved_argument("...")
    assert module.is_unresolved_argument("todo")
    assert module.is_unresolved_argument("observed tbd in terminal")
    assert module.is_unresolved_argument("none")
    assert not module.is_unresolved_argument("none", allow_none=True)
    assert not module.is_unresolved_argument("confirmed browser recording hash")


def test_observation_errors_require_positive_visible_fragments() -> None:
    module = load_module()

    assert module.observation_errors(
        "LLM observation",
        (
            "I saw the llm.call prompt, completion, model, token breakdown, cost, "
            "latency, and confirmation code in the dashboard."
        ),
        module.LLM_OBSERVATION_FRAGMENTS,
    ) == []
    negated = module.observation_errors(
        "Waterfall observation",
        "The run was missing the turn, step, tool, and MCP waterfall.",
        module.WATERFALL_OBSERVATION_FRAGMENTS,
    )
    assert "Waterfall observation must be a positive observation, not negated evidence" in negated
    missing = module.observation_errors(
        "Waterfall observation",
        "I saw the run and turn in the dashboard.",
        module.WATERFALL_OBSERVATION_FRAGMENTS,
    )
    assert "Waterfall observation must mention: step, tool, MCP" in missing


def main() -> None:
    for test in (
        test_confirmation_code_uses_stable_contract_vector,
        test_gate2_image_helpers_keep_public_ghcr_contract,
        test_raw_public_preflight_command_pins_expected_commit,
        test_immutable_log_url_requires_actions_run_or_job,
        test_unresolved_argument_rejects_placeholders,
        test_observation_errors_require_positive_visible_fragments,
    ):
        test()
    print("gate2_proof_contract tests passed.")


if __name__ == "__main__":
    main()
