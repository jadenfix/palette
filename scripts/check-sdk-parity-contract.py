#!/usr/bin/env python3
"""Static contract check for the SDK parity gate.

The live conformance runner proves generated clients can talk to beaterd. This
check covers the static half of ARCHITECTURE.md Section 20.8 #6.5: every SDK
participant is declared, every conformance program is wired into the runner, and
the hand-written ergonomic SDKs expose the same logical surface from the shared
manifest.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "sdks/conformance/surface-manifest.json"
EXPECTED_GENERATED = ["c", "cpp", "go", "java", "python", "rust", "typescript"]
EXPECTED_ERGONOMIC = ["go", "java", "python", "rust", "typescript"]


class Check:
    def __init__(self, root: Path, manifest_path: Path) -> None:
        self.root = root
        self.manifest_path = manifest_path
        self.failures: list[str] = []

    def fail(self, message: str) -> None:
        self.failures.append(message)

    def path(self, rel: str) -> Path:
        return self.root / rel

    def require_path(self, rel: str, *, executable: bool = False) -> None:
        path = self.path(rel)
        if not path.exists():
            self.fail(f"missing required path: {rel}")
            return
        if executable and not path.is_file():
            self.fail(f"required executable is not a file: {rel}")

    def read(self, rel: str) -> str:
        path = self.path(rel)
        try:
            return path.read_text(encoding="utf-8")
        except FileNotFoundError:
            self.fail(f"missing required file: {rel}")
        except UnicodeDecodeError as exc:
            self.fail(f"file is not valid UTF-8: {rel}: {exc}")
        return ""

    def load_manifest(self) -> dict[str, Any]:
        try:
            with self.manifest_path.open(encoding="utf-8") as fh:
                data = json.load(fh)
        except FileNotFoundError:
            self.fail(f"missing manifest: {self.manifest_path.relative_to(self.root)}")
            return {}
        except json.JSONDecodeError as exc:
            self.fail(f"manifest is not valid JSON: {exc}")
            return {}
        if not isinstance(data, dict):
            self.fail("manifest root must be an object")
            return {}
        return data

    def check(self) -> int:
        data = self.load_manifest()
        if not data:
            return self.report()

        if data.get("schema_version") != 1:
            self.fail("surface-manifest.json schema_version must be 1")

        self.check_generated(data.get("generated_client_participants"))
        self.check_ergonomic(data)
        self.check_ci_wiring()
        return self.report()

    def check_generated(self, participants: Any) -> None:
        if not isinstance(participants, list):
            self.fail("generated_client_participants must be a list")
            return

        languages = [entry.get("language") for entry in participants if isinstance(entry, dict)]
        if sorted(languages) != EXPECTED_GENERATED:
            self.fail(
                "generated_client_participants must list exactly "
                f"{', '.join(EXPECTED_GENERATED)}; found {', '.join(sorted(map(str, languages)))}"
            )

        e2e_runner = self.read("scripts/e2e-clients-live.sh")
        actual_dirs = sorted(
            path.name
            for path in self.path("sdks/conformance").iterdir()
            if path.is_dir()
        )
        if actual_dirs != EXPECTED_GENERATED:
            self.fail(
                "sdks/conformance directories must match the generated SDK set; "
                f"found {', '.join(actual_dirs)}"
            )

        for entry in participants:
            if not isinstance(entry, dict):
                self.fail("generated_client_participants entries must be objects")
                continue
            language = str(entry.get("language", ""))
            for key in ("client_path", "conformance_script"):
                value = entry.get(key)
                if not isinstance(value, str) or not value:
                    self.fail(f"{language}: missing {key}")
                    continue
                self.require_path(value, executable=key == "conformance_script")
            for rel in entry.get("required_files", []):
                if not isinstance(rel, str):
                    self.fail(f"{language}: required_files entries must be strings")
                    continue
                self.require_path(rel)
            if f"run_lang {language} " not in e2e_runner:
                self.fail(f"scripts/e2e-clients-live.sh must invoke run_lang {language}")

    def check_ergonomic(self, data: dict[str, Any]) -> None:
        participants = data.get("ergonomic_sdk_participants")
        if not isinstance(participants, list):
            self.fail("ergonomic_sdk_participants must be a list")
            return

        languages = [entry.get("language") for entry in participants if isinstance(entry, dict)]
        if sorted(languages) != EXPECTED_ERGONOMIC:
            self.fail(
                "ergonomic_sdk_participants must list exactly "
                f"{', '.join(EXPECTED_ERGONOMIC)}; found {', '.join(sorted(map(str, languages)))}"
            )

        span_values = require_string_list(data, "required_span_kind_values", self.fail)
        attr_values = require_string_list(data, "required_attribute_values", self.fail)
        header_values = require_string_list(data, "required_ingest_header_values", self.fail)
        env_vars = require_string_list(data, "required_env_vars", self.fail)

        for entry in participants:
            if not isinstance(entry, dict):
                self.fail("ergonomic_sdk_participants entries must be objects")
                continue
            language = str(entry.get("language", ""))
            package_path = entry.get("package_path")
            semconv_file = entry.get("semconv_file")
            source_files = entry.get("source_files")
            capability_tokens = entry.get("capability_tokens")

            if not isinstance(package_path, str):
                self.fail(f"{language}: package_path must be set")
                continue
            self.require_path(package_path)
            if not isinstance(semconv_file, str):
                self.fail(f"{language}: semconv_file must be set")
                continue
            self.require_path(semconv_file)
            if not isinstance(source_files, list) or not source_files:
                self.fail(f"{language}: source_files must be a non-empty list")
                continue

            source = ""
            for rel in source_files:
                if not isinstance(rel, str):
                    self.fail(f"{language}: source_files entries must be strings")
                    continue
                self.require_path(rel)
                source += "\n" + self.read(rel)
            semconv = self.read(semconv_file)

            if not isinstance(capability_tokens, dict):
                self.fail(f"{language}: capability_tokens must be an object")
            else:
                for capability, tokens in sorted(capability_tokens.items()):
                    if not isinstance(tokens, list) or not tokens:
                        self.fail(f"{language}: {capability} must declare one or more tokens")
                        continue
                    for token in tokens:
                        if not isinstance(token, str):
                            self.fail(f"{language}: {capability} token must be a string")
                        elif token not in source:
                            self.fail(f"{language}: missing {capability} token {token!r}")

            for env_var in env_vars:
                if env_var not in source:
                    self.fail(f"{language}: missing env var {env_var}")
            for span_kind in span_values:
                if span_kind not in semconv:
                    self.fail(f"{language}: missing span kind {span_kind}")
            for attr in attr_values:
                if attr not in semconv:
                    self.fail(f"{language}: missing attribute key {attr}")
            for header in header_values:
                if header not in semconv:
                    self.fail(f"{language}: missing ingest header {header}")

    def check_ci_wiring(self) -> None:
        self.require_path("scripts/check-sdk-parity.sh")
        self.require_path(".github/workflows/sdk-parity.yml")

        wrapper = self.read("scripts/check-sdk-parity.sh")
        if "scripts/check-sdk-parity-contract.py" not in wrapper:
            self.fail("scripts/check-sdk-parity.sh must run check-sdk-parity-contract.py")
        if "scripts/e2e-clients-live.sh" not in wrapper:
            self.fail("scripts/check-sdk-parity.sh must keep a live conformance path")

        workflow = self.read(".github/workflows/sdk-parity.yml")
        for token in ("name: sdk-parity", "bash scripts/check-sdk-parity.sh"):
            if token not in workflow:
                self.fail(f".github/workflows/sdk-parity.yml missing {token!r}")

    def report(self) -> int:
        if self.failures:
            print("SDK parity contract check failed:", file=sys.stderr)
            for failure in self.failures:
                print(f"  - {failure}", file=sys.stderr)
            return 1
        print("SDK parity contract is wired for all generated and ergonomic SDK participants.")
        return 0


def require_string_list(
    data: dict[str, Any], key: str, fail: Callable[[str], None]
) -> list[str]:
    value = data.get(key)
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        fail(f"{key} must be a list of strings")
        return []
    return value


def main() -> int:
    return Check(ROOT, MANIFEST).check()


if __name__ == "__main__":
    raise SystemExit(main())
