#!/usr/bin/env python3
"""Verify committed demo artifact hashes match their docs."""

from pathlib import Path
import hashlib
import re
import sys


ROOT = Path(__file__).resolve().parent.parent
DEMOS = ROOT / "docs" / "demos"

DEMO_DOCS = [
    DEMOS / "gate2-browser-demo.md",
    DEMOS / "gate2-compose-browser-demo.md",
]

REQUIRED_TEXT = {
    "gate2-browser-demo.md": [
        "stock OpenTelemetry Python trace",
        "Recording mode: all-kind",
        "run -> turn -> step -> tool -> MCP",
        "BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh",
    ],
    "gate2-compose-browser-demo.md": [
        "Docker Compose stopwatch path",
        "Recording mode: compose",
        "default dashboard URL `http://127.0.0.1:3000`",
        "docs/demos/gate2-outside-person-proof.md",
        "BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1",
    ],
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    failures: list[str] = []

    for doc_path in DEMO_DOCS:
        text = doc_path.read_text(encoding="utf-8")
        artifact_match = re.search(r"^- Artifact: `([^`]+)`$", text, re.MULTILINE)
        sha_match = re.search(r"^- SHA256: `([0-9a-f]{64})`$", text, re.MULTILINE)
        if not artifact_match:
            failures.append(f"{doc_path.relative_to(ROOT)} is missing an Artifact line")
            continue
        if not sha_match:
            failures.append(f"{doc_path.relative_to(ROOT)} is missing a SHA256 line")
            continue

        artifact = doc_path.parent / artifact_match.group(1)
        if not artifact.is_file():
            failures.append(f"{doc_path.relative_to(ROOT)} points at missing artifact {artifact.name}")
            continue
        if artifact.stat().st_size <= 0:
            failures.append(f"{artifact.relative_to(ROOT)} is empty")

        actual = sha256(artifact)
        expected = sha_match.group(1)
        if actual != expected:
            failures.append(
                f"{artifact.relative_to(ROOT)} sha256 mismatch: documented {expected}, actual {actual}"
            )

        for token in REQUIRED_TEXT.get(doc_path.name, []):
            if token not in text:
                failures.append(f"{doc_path.relative_to(ROOT)} missing {token!r}")

    if failures:
        print("Demo artifact check failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("Demo artifact docs match committed WebM hashes.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
