#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

from gate2_proof_contract import GATE2_EXPECTED_PLATFORMS, GATE2_IMAGES


ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = ROOT / ".github/workflows/container-images.yml"
PREBUILT_COMPOSE = ROOT / "docker-compose.prebuilt.yml"
GITHUB_REPOSITORY = "${{ github.repository }}"
GITHUB_SHA = "${{ github.sha }}"
MATRIX_SUFFIX = "${{ matrix.suffix }}"


def fail(message: str) -> None:
    print(f"container image contract failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_contains(source: str, needle: str, description: str) -> None:
    if needle not in source:
        fail(f"missing {description}: {needle}")


def require_workflow_shape(workflow: str) -> None:
    require_contains(workflow, "push:\n    branches: [main]", "main-only push trigger")
    require_contains(workflow, "workflow_dispatch:", "manual workflow dispatch")
    require_contains(workflow, "packages: write", "GHCR package publish permission")
    require_contains(workflow, "fail-fast: false", "independent per-platform image builds")
    for platform in GATE2_EXPECTED_PLATFORMS:
        require_contains(workflow, f"platform: {platform}", f"{platform} build matrix entry")


def workflow_arch_suffix(platform: str) -> str:
    arch = platform.split("/", 1)[1]
    if arch not in {"amd64", "arm64"}:
        fail(f"unsupported Gate 2 image platform {platform!r}")
    return arch


def require_image_build_and_publish(workflow: str) -> None:
    expected_suffixes = [workflow_arch_suffix(platform) for platform in GATE2_EXPECTED_PLATFORMS]
    for image in GATE2_IMAGES:
        matrix_sha_tag = f"ghcr.io/{GITHUB_REPOSITORY}/{image.image_name}:{GITHUB_SHA}-{MATRIX_SUFFIX}"
        matrix_main_tag = f"ghcr.io/{GITHUB_REPOSITORY}/{image.image_name}:main-{MATRIX_SUFFIX}"
        manifest_sha_tag = f"-t ghcr.io/{GITHUB_REPOSITORY}/{image.image_name}:{GITHUB_SHA}"
        manifest_main_tag = f"-t ghcr.io/{GITHUB_REPOSITORY}/{image.image_name}:main"

        require_contains(workflow, matrix_sha_tag, f"{image.image_name} SHA matrix tag")
        require_contains(workflow, matrix_main_tag, f"{image.image_name} main matrix tag")
        require_contains(workflow, manifest_sha_tag, f"{image.image_name} SHA manifest tag")
        require_contains(workflow, manifest_main_tag, f"{image.image_name} main manifest tag")
        for suffix in expected_suffixes:
            require_contains(
                workflow,
                f"ghcr.io/{GITHUB_REPOSITORY}/{image.image_name}:{GITHUB_SHA}-{suffix}",
                f"{image.image_name} {suffix} manifest source",
            )


def require_prebuilt_compose_defaults(compose: str) -> None:
    for image in GATE2_IMAGES:
        expected = "image: ${" + image.env_var + ":-" + image.repo + ":main}"
        require_contains(
            compose,
            expected,
            f"docker-compose.prebuilt.yml default for {image.env_var}",
        )


def main() -> None:
    workflow = WORKFLOW.read_text()
    compose = PREBUILT_COMPOSE.read_text()
    require_workflow_shape(workflow)
    require_image_build_and_publish(workflow)
    require_prebuilt_compose_defaults(compose)
    print("Container image contract matches the Gate 2 image catalog.")


if __name__ == "__main__":
    main()
