#!/usr/bin/env python3
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / "scripts" / "check-semconv-drift.py"


def load_module():
    spec = importlib.util.spec_from_file_location("check_semconv_drift", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError(f"could not load {SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    stdout = io.StringIO()
    stderr = io.StringIO()
    with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
        spec.loader.exec_module(module)
    assert stdout.getvalue() == ""
    assert stderr.getvalue() == ""
    return module


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def write_fixture(root: Path, *, include_env: bool = True, extra_semconv: str = "") -> None:
    write(
        root / "sdks/semconv/conventions.json",
        json.dumps(
            {
                "span_kinds": ["agent.run"],
                "attributes": {"model": "llm.model"},
                "defaults": {"base_url": "http://127.0.0.1:8080"},
                "env": {"api_key": "BEATER_API_KEY"},
            }
        ),
    )
    write(
        root / "sdk/semconv.txt",
        f'''
const RUN = "agent.run";
const MODEL = "llm.model"; // "tool.call" only in a comment
const DOC = "http://127.0.0.1:8080/path"; // URL with // inside a string
{extra_semconv}
''',
    )
    env_line = 'const KEY = "BEATER_API_KEY";' if include_env else ""
    write(
        root / "sdk/config.txt",
        f'''
const BASE_URL = "http://127.0.0.1:8080";
{env_line}
''',
    )


def test_import_is_side_effect_free() -> None:
    load_module()


def test_assigned_values_ignore_comments_but_keep_comment_markers_inside_strings() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as temp:
        path = Path(temp) / "sample.ts"
        write(
            path,
            '''
const span = "agent.run"; // "comment.only"
const endpoint = "http://127.0.0.1:8080/v1";
''',
        )
        values = module.assigned_values(path, "//")
    assert "agent.run" in values
    assert "http://127.0.0.1:8080/v1" in values
    assert "comment.only" not in values


def test_semconv_wire_values_filter_config_and_urls() -> None:
    module = load_module()
    values = module.semconv_wire_values(
        {"agent.run", "llm.model", "x-beater-project-id", "http://127.0.0.1:8080", "BEATER_API_KEY"}
    )
    assert values == {"agent.run", "llm.model", "x-beater-project-id"}


def test_fixture_passes_when_semconv_and_config_match() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        write_fixture(root)
        failed, lines = module.check_all(root, {"toy": ("sdk/semconv.txt", "sdk/config.txt", "//")})
    assert not failed
    assert any(line.startswith("PASS toy:") for line in lines)


def test_missing_config_value_is_reported() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        write_fixture(root, include_env=False)
        failed, lines = module.check_all(root, {"toy": ("sdk/semconv.txt", "sdk/config.txt", "//")})
    assert failed
    output = "\n".join(lines)
    assert "missing defaults/env values" in output
    assert "BEATER_API_KEY" in output


def test_extra_semconv_wire_value_is_reported() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        write_fixture(root, extra_semconv='const EXTRA = "tool.call";')
        failed, lines = module.check_all(root, {"toy": ("sdk/semconv.txt", "sdk/config.txt", "//")})
    assert failed
    output = "\n".join(lines)
    assert "has extra assigned values" in output
    assert "tool.call" in output


def main() -> None:
    for test in (
        test_import_is_side_effect_free,
        test_assigned_values_ignore_comments_but_keep_comment_markers_inside_strings,
        test_semconv_wire_values_filter_config_and_urls,
        test_fixture_passes_when_semconv_and_config_match,
        test_missing_config_value_is_reported,
        test_extra_semconv_wire_value_is_reported,
    ):
        test()
    print("check-semconv-drift tests passed")


if __name__ == "__main__":
    main()
