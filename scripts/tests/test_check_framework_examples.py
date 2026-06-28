#!/usr/bin/env python3
from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check-framework-examples.py"


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def copy_framework_examples(root: Path) -> None:
    shutil.copytree(ROOT / "examples", root / "examples")


def run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(SCRIPT), str(root)],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def test_current_framework_examples_pass() -> None:
    result = run(ROOT)

    assert result.returncode == 0, result.stderr
    assert "Framework examples are registered" in result.stdout


def test_rejects_unregistered_framework_example() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_framework_examples(root)
        write(
            root / "examples" / "typescript" / "frameworks" / "unlisted.mjs",
            "console.log('unregistered framework example');\n",
        )

        result = run(root)

        assert result.returncode == 1
        assert "unlisted.mjs is not registered" in result.stderr


def test_rejects_provider_wrapper_in_vercel_example() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        copy_framework_examples(root)
        path = root / "examples" / "typescript" / "frameworks" / "vercel-ai-sdk-otlp.mjs"
        path.write_text(
            path.read_text(encoding="utf-8") + "\nimport { openai } from \"@ai-sdk/openai\";\n",
            encoding="utf-8",
        )

        result = run(root)

        assert result.returncode == 1
        assert "must not import provider wrapper '@ai-sdk/openai'" in result.stderr


if __name__ == "__main__":
    for test in (
        test_current_framework_examples_pass,
        test_rejects_unregistered_framework_example,
        test_rejects_provider_wrapper_in_vercel_example,
    ):
        test()
    print("check-framework-examples tests passed")
