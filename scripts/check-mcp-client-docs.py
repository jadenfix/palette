#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DOC = ROOT / "docs/mcp-client-setup.md"
BEATERD_MAIN = ROOT / "bins/beaterd/src/main.rs"
OAUTH_SERVER = ROOT / "crates/beater-oauth-server/src/lib.rs"
MCP_LIB = ROOT / "crates/beater-mcp/src/lib.rs"


DOC_REQUIRED = [
    "Claude Code",
    "Codex",
    "beaterd mcp --stdio",
    "claude mcp add",
    "[mcp_servers.beater]",
    "command = \"beaterd\"",
    "POST /mcp",
    "GET  /mcp",
    "/.well-known/oauth-authorization-server",
    "/oauth/authorize",
    "/oauth/token",
    "tools/list",
    "tools/call",
    "\"name\":\"help\"",
    "BEATER_MCP_TOKEN",
]

SOURCE_REQUIRED = {
    BEATERD_MAIN: ["mcp", "stdio", "beater_mcp::serve_stdio"],
    OAUTH_SERVER: [
        "/.well-known/oauth-authorization-server",
        "/oauth/authorize",
        "/oauth/token",
    ],
    MCP_LIB: ["route(\"/mcp\"", "tools/list", "tools/call", "serve_stdio"],
}


def fail(message: str) -> None:
    print(f"MCP client docs contract failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> str:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")
    text = path.read_text()
    if not text.strip():
        fail(f"empty file: {path.relative_to(ROOT)}")
    return text


def main() -> None:
    doc = require_file(DOC)
    for needle in DOC_REQUIRED:
        if needle not in doc:
            fail(f"{DOC.relative_to(ROOT)} must mention {needle!r}")
    if re.search(r"\bstdio\b.{0,32}\bplanned\b", doc, flags=re.IGNORECASE | re.DOTALL):
        fail("docs must not describe stdio as planned")

    for path, needles in SOURCE_REQUIRED.items():
        text = require_file(path)
        for needle in needles:
            if needle not in text:
                fail(f"{path.relative_to(ROOT)} must still expose {needle!r}")

    print("MCP client setup docs match the implemented transports.")


if __name__ == "__main__":
    main()
