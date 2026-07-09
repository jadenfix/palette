#!/usr/bin/env python3
"""Filter intentional pre-1.0 OpenAPI breaking changes from oasdiff output."""

from __future__ import annotations

import re
import sys
from pathlib import Path


LEGACY_SCOPE_VALUES = {
    "dataset_write",
    "eval_run",
    "pii_unmask",
    "scenario_read",
    "scenario_write",
    "trace_read",
    "trace_write",
}


def error_blocks(text: str) -> list[str]:
    blocks: list[str] = []
    current: list[str] = []
    for line in text.splitlines():
        if line.startswith("error\t["):
            if current:
                blocks.append("\n".join(current))
            current = [line]
        elif current:
            if not line.strip():
                blocks.append("\n".join(current))
                current = []
            else:
                current.append(line)
    if current:
        blocks.append("\n".join(current))
    return blocks


def is_allowed_alignment_break(block: str) -> bool:
    if "[response-required-property-removed]" in block:
        return "removed the required property `status` from the response" in block
    if "[request-property-enum-value-removed]" in block:
        match = re.search(r"removed the enum value `([^`]+)` of the request property `scopes/items/`", block)
        return match is not None and match.group(1) in LEGACY_SCOPE_VALUES
    return False


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: filter-oasdiff-breaking.py OASDIFF_LOG", file=sys.stderr)
        return 2
    text = Path(sys.argv[1]).read_text(encoding="utf-8")
    blocks = error_blocks(text)
    unexpected = [block for block in blocks if not is_allowed_alignment_break(block)]
    if unexpected:
        print("Unexpected OpenAPI breaking changes:", file=sys.stderr)
        for block in unexpected:
            print(block, file=sys.stderr)
            print(file=sys.stderr)
        return 1
    if blocks:
        print(f"Allowed {len(blocks)} intentional pre-1.0 auth contract alignment breaks.")
    else:
        print("No OpenAPI breaking changes detected.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
