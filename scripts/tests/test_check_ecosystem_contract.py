#!/usr/bin/env python3
from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check-ecosystem-contract.py"


def copy_contract_inputs(root: Path) -> None:
    for rel in (
        "docs/ecosystem-integration-contract.md",
        "docs/offline-self-host.md",
        "GOVERNANCE.md",
        "crates/beater-api/src/lib.rs",
        "bins/beaterd/src/main.rs",
    ):
        source = ROOT / rel
        dest = root / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, dest)


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(SCRIPT), str(root)],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def test_current_ecosystem_contract_passes() -> None:
    result = run(ROOT)

    assert result.returncode == 0, result.stderr
    assert "Static ecosystem integration markers are aligned" in result.stdout


def test_rejects_missing_neighbor_repo_boundary() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_contract_inputs(root)
        doc = root / "docs" / "ecosystem-integration-contract.md"
        doc.write_text(
            doc.read_text(encoding="utf-8").replace("beaterOS", "otherOS"),
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert "missing marker 'beaterOS'" in result.stderr


def test_rejects_missing_aether_payment_boundary() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_contract_inputs(root)
        doc = root / "docs" / "ecosystem-integration-contract.md"
        doc.write_text(
            doc.read_text(encoding="utf-8").replace("PaymentEnvelope", "SettlementEnvelope"),
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert "missing marker 'PaymentEnvelope'" in result.stderr


def test_rejects_missing_payment_trace_metadata_fixture() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_contract_inputs(root)
        api = root / "crates" / "beater-api" / "src" / "lib.rs"
        api.write_text(
            api.read_text(encoding="utf-8").replace(
                "aether.payment_envelope_id",
                "aether.payment_envelope_removed",
            ),
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert "missing marker 'aether.payment_envelope_id'" in result.stderr


def test_rejects_product_billing_route_contract() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_contract_inputs(root)
        api = root / "crates" / "beater-api" / "src" / "lib.rs"
        api.write_text(
            api.read_text(encoding="utf-8") + '\n.route("/v1/billing/webhooks/stripe")\n',
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert "crates/beater-api/src/lib.rs must not contain marker" in result.stderr


def test_rejects_daemon_billing_store_contract() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_contract_inputs(root)
        daemon = root / "bins" / "beaterd" / "src" / "main.rs"
        daemon.write_text(
            daemon.read_text(encoding="utf-8")
            + '\nlet billing_db_path = data_dir.join("billing.sqlite");\n',
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert "bins/beaterd/src/main.rs must not contain marker" in result.stderr


def test_rejects_missing_import_ingress_route() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_contract_inputs(root)
        api = root / "crates" / "beater-api" / "src" / "lib.rs"
        api.write_text(
            api.read_text(encoding="utf-8").replace(
                '"/v1/import/:tenant_id/:project_id/:environment_id"',
                '"/v1/import-removed/:tenant_id/:project_id/:environment_id"',
            ),
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert (
            "missing marker '\"/v1/import/:tenant_id/:project_id/:environment_id\"'"
            in result.stderr
        )


if __name__ == "__main__":
    for test in (
        test_current_ecosystem_contract_passes,
        test_rejects_missing_neighbor_repo_boundary,
        test_rejects_missing_aether_payment_boundary,
        test_rejects_missing_payment_trace_metadata_fixture,
        test_rejects_product_billing_route_contract,
        test_rejects_daemon_billing_store_contract,
        test_rejects_missing_import_ingress_route,
    ):
        test()
    print("check-ecosystem-contract tests passed")
