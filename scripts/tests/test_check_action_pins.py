#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import tempfile
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / "scripts" / "check-action-pins.sh"
PIN = "0123456789abcdef0123456789abcdef01234567"


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(SCRIPT), str(root)],
        cwd=REPO,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )


def test_allows_local_first_party_and_sha_pinned_refs() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        write(
            root / ".github" / "workflows" / "ok.yml",
            f"""
name: ok
jobs:
  ok:
    steps:
      - uses: actions/checkout@v7
      - uses: ./.github/actions/local
      - uses: vendor/tool@{PIN} # v1
      - uses: vendor/repo/.github/workflows/reuse.yml@{PIN}
""",
        )

        result = run(root)

        assert result.returncode == 0, result.stderr
        assert "pinned to immutable commit SHAs" in result.stdout


def test_rejects_mutable_third_party_workflow_refs() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        write(
            root / ".github" / "workflows" / "bad.yml",
            """
name: bad
jobs:
  bad:
    steps:
      - uses: docker/build-push-action@v7
      - uses: org/repo/.github/workflows/reuse.yml@main
""",
        )

        result = run(root)

        assert result.returncode == 1
        assert "docker/build-push-action@v7" in result.stderr
        assert "org/repo/.github/workflows/reuse.yml@main" in result.stderr


def test_scans_composite_action_metadata() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        write(
            root / ".github" / "actions" / "composite" / "action.yml",
            """
name: composite
runs:
  using: composite
  steps:
    - uses: third-party/setup@v1
""",
        )

        result = run(root)

        assert result.returncode == 1
        assert "third-party/setup@v1" in result.stderr


if __name__ == "__main__":
    for test in [
        test_allows_local_first_party_and_sha_pinned_refs,
        test_rejects_mutable_third_party_workflow_refs,
        test_scans_composite_action_metadata,
    ]:
        test()
    print("check-action-pins tests passed")
