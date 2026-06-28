#!/usr/bin/env python3
import os
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "publish-sdk.sh"


def run_publish_sdk(*args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    for key in (
        "CARGO_REGISTRY_TOKEN",
        "PYPI_TOKEN",
        "NPM_TOKEN",
        "OSSRH_USERNAME",
        "OSSRH_PASSWORD",
    ):
        env.pop(key, None)
    return subprocess.run(
        [str(SCRIPT), *args],
        cwd=ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


class PublishSdkVersionValidationTests(unittest.TestCase):
    def test_validate_version_accepts_semver(self) -> None:
        for version in ("0.2.0", "1.2.3-alpha.1", "1.2.3+build.7"):
            with self.subTest(version=version):
                result = run_publish_sdk("--validate-version", version)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn(f"Valid release version: {version}", result.stdout)

    def test_validate_version_rejects_non_semver(self) -> None:
        for version in ("v0.2.0", "0.2", "01.2.3", "1.2.3-"):
            with self.subTest(version=version):
                result = run_publish_sdk("--validate-version", version)
                self.assertEqual(result.returncode, 1)
                self.assertIn("must be SemVer without a leading v", result.stderr)

    def test_publish_validates_before_secret_skip(self) -> None:
        result = run_publish_sdk("python", "v0.2.0")
        self.assertEqual(result.returncode, 1)
        self.assertIn("must be SemVer without a leading v", result.stderr)
        self.assertNotIn("SKIP python", result.stdout)

    def test_go_publish_accepts_valid_version_without_registry_secret(self) -> None:
        result = run_publish_sdk("go", "0.2.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("go: tag v0.2.0 pushed", result.stdout)
        self.assertIn("Published go @ 0.2.0", result.stdout)


if __name__ == "__main__":
    unittest.main()
