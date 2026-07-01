#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import io
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "check-gate0-foundations.py"
SPEC = importlib.util.spec_from_file_location("check_gate0_foundations", SCRIPT)
assert SPEC is not None
gate0 = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(gate0)


VALID_SECURITY = """# Security Policy

## Reporting a vulnerability (coordinated disclosure)

Do not open a public issue. Report privately through GitHub Security Advisories
using the repository's Security -> Report a vulnerability private advisory
workflow.
"""


class Gate0GovernanceSecurityPresenceTests(unittest.TestCase):
    def make_repo(self, overrides: dict[str, str | None] | None = None) -> Path:
        temp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(temp_dir.cleanup)
        repo = Path(temp_dir.name)
        files = {
            "LICENSE": "Apache-2.0\n",
            "GOVERNANCE.md": "# Governance\nMaintainers are listed here.\n",
            "SECURITY.md": VALID_SECURITY,
            "CONTRIBUTING.md": "# Contributing\nRun tests before opening a PR.\n",
        }
        if overrides:
            files.update(overrides)

        for name, content in files.items():
            if content is None:
                continue
            (repo / name).write_text(content, encoding="utf-8")
        return repo

    def assert_gate0_fails(self, repo: Path, message: str) -> None:
        stderr = io.StringIO()
        with self.assertRaises(SystemExit), redirect_stderr(stderr):
            gate0.check_governance_security_presence(repo)
        self.assertIn(message, stderr.getvalue())

    def test_required_docs_pass_when_present_nonempty_and_security_has_private_disclosure(self) -> None:
        repo = self.make_repo()

        gate0.check_governance_security_presence(repo)

    def test_required_docs_fail_when_missing(self) -> None:
        repo = self.make_repo({"GOVERNANCE.md": None})

        self.assert_gate0_fails(repo, "GOVERNANCE.md is missing")

    def test_required_docs_fail_when_empty(self) -> None:
        repo = self.make_repo({"CONTRIBUTING.md": " \n\t\n"})

        self.assert_gate0_fails(repo, "CONTRIBUTING.md must not be empty")

    def test_security_must_include_private_or_coordinated_disclosure_path(self) -> None:
        repo = self.make_repo(
            {
                "SECURITY.md": "# Security Policy\nOpen a public issue with vulnerability details.\n",
            }
        )

        self.assert_gate0_fails(repo, "SECURITY.md must describe a private/coordinated disclosure path")


if __name__ == "__main__":
    unittest.main()
