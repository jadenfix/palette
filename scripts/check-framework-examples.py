#!/usr/bin/env python3
"""Static drift checks for Beater framework examples.

This intentionally avoids package installs and live model/network calls. It
only checks syntax, README registration, and import-shape constraints for
examples that document framework coverage from ARCHITECTURE.md §20.8/§20.10.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


MODERN_FRAMEWORK_EXAMPLES = {
    "examples/typescript/frameworks/vercel-ai-sdk-otlp.mjs": {
        "framework": "Vercel AI SDK",
        "required": (
            'await import("ai")',
            '"agent.framework": "vercel-ai-sdk"',
            '"beater.span.kind": "agent.run"',
            '"beater.span.kind": "llm.call"',
            '"beater.span.kind": "tool.call"',
            '"llm.token_count.prompt"',
            '"llm.token_count.completion"',
        ),
        "forbidden": (
            "@ai-sdk/anthropic",
            "@ai-sdk/amazon-bedrock",
            "@ai-sdk/google",
            "@ai-sdk/groq",
            "@ai-sdk/mistral",
            "@ai-sdk/openai",
            "from \"openai\"",
            "from 'openai'",
        ),
    },
}


def repo_root() -> Path:
    if len(sys.argv) > 2:
        raise SystemExit("usage: scripts/check-framework-examples.py [repo-root]")
    if len(sys.argv) == 2:
        return Path(sys.argv[1]).resolve()
    return Path(__file__).resolve().parents[1]


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def fail(failures: list[str], message: str) -> None:
    failures.append(message)


def framework_files(root: Path) -> list[Path]:
    patterns = (
        "examples/python/frameworks/*.py",
        "examples/typescript/frameworks/*.mjs",
    )
    files: list[Path] = []
    for pattern in patterns:
        files.extend(root.glob(pattern))
    return sorted(files)


def check_readme_registration(root: Path, files: list[Path], failures: list[str]) -> None:
    readme = root / "examples" / "README.md"
    if not readme.is_file():
        fail(failures, "examples/README.md is missing")
        return
    text = readme.read_text(encoding="utf-8")
    for path in files:
        path_rel = rel(path, root)
        example_rel = path.relative_to(root / "examples").as_posix()
        if path_rel not in text and example_rel not in text:
            fail(failures, f"{path_rel} is not registered in examples/README.md")


def check_python_syntax(root: Path, files: list[Path], failures: list[str]) -> None:
    for path in files:
        try:
            compile(path.read_text(encoding="utf-8"), str(path), "exec")
        except SyntaxError as error:
            fail(failures, f"{rel(path, root)} failed Python syntax check: {error}")


def check_typescript_syntax(root: Path, files: list[Path], failures: list[str]) -> None:
    node = shutil.which("node")
    if node is None:
        fail(failures, "node is required to syntax-check TypeScript framework examples")
        return
    for path in files:
        result = subprocess.run(
            [node, "--check", str(path)],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            detail = (result.stderr or result.stdout).strip()
            fail(failures, f"{rel(path, root)} failed node --check: {detail}")


def check_modern_framework_examples(root: Path, failures: list[str]) -> None:
    readme_path = root / "examples" / "README.md"
    readme = readme_path.read_text(encoding="utf-8") if readme_path.is_file() else ""
    for path_rel, checks in MODERN_FRAMEWORK_EXAMPLES.items():
        path = root / path_rel
        if not path.is_file():
            fail(failures, f"missing required modern framework example: {path_rel}")
            continue
        source = path.read_text(encoding="utf-8")
        framework = checks["framework"]
        if framework not in readme:
            fail(failures, f"{path_rel} is missing README framework label {framework!r}")
        for needle in checks["required"]:
            if needle not in source:
                fail(failures, f"{path_rel} missing required marker {needle!r}")
        for needle in checks["forbidden"]:
            if needle in source:
                fail(failures, f"{path_rel} must not import provider wrapper {needle!r}")


def main() -> None:
    root = repo_root()
    failures: list[str] = []
    files = framework_files(root)
    if not files:
        fail(failures, "no framework examples found under examples/")

    check_readme_registration(root, files, failures)
    check_python_syntax(root, [path for path in files if path.suffix == ".py"], failures)
    check_typescript_syntax(root, [path for path in files if path.suffix == ".mjs"], failures)
    check_modern_framework_examples(root, failures)

    if failures:
        print("FRAMEWORK EXAMPLE DRIFT DETECTED", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        raise SystemExit(1)

    print(f"Framework examples are registered and syntax-valid ({len(files)} files).")


if __name__ == "__main__":
    main()
