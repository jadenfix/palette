#!/usr/bin/env python3
import argparse
import json
import os
import re
import subprocess
import urllib.request
from pathlib import Path


IMAGE_NAMES = ["beaterd", "dashboard", "dashboard-e2e", "otel-python"]
EXPECTED_PLATFORMS = ["linux/amd64", "linux/arm64"]
REMOTE_URL = "https://github.com/jadenfix/beater.git"
COMMON_PINNED_THIRD_PARTY_IMAGES = [
    (
        "postgres:17-alpine",
        "sha256:dc17045ccfd343b49600570ea734b9c4991cf1c3f3302e67df51e3b402dd55c4",
    ),
    (
        "nats:2.11-alpine",
        "sha256:e4bf19f15fd3218814a4e3c9e0064e1334bd8aa20d5984b9f1a0afd084f8cc00",
    ),
    (
        "minio/minio:latest",
        "sha256:14cea493d9a34af32f524e538b8346cf79f3321eff8e708c1e2960462bd8936e",
    ),
]
CLICKHOUSE_PINNED_THIRD_PARTY_IMAGE = (
    "clickhouse/clickhouse-server:latest",
    "sha256:07afc18d8a9706eb9d85c5c5d2752e5270f91bbc2894caeaecb73e4d0f603bf5",
)
PINNED_THIRD_PARTY_IMAGES = {
    "docker-compose.yml": [
        *COMMON_PINNED_THIRD_PARTY_IMAGES,
        CLICKHOUSE_PINNED_THIRD_PARTY_IMAGE,
    ],
    "docker-compose.prebuilt.yml": COMMON_PINNED_THIRD_PARTY_IMAGES,
}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run_git(args: list[str]) -> str:
    return subprocess.check_output(
        ["git", *args], cwd=repo_root(), text=True, stderr=subprocess.DEVNULL
    ).strip()


def current_commit() -> str:
    try:
        commit = run_git(["rev-parse", "HEAD"])
    except (OSError, subprocess.CalledProcessError) as err:
        raise SystemExit(f"could not read git HEAD: {err}") from err
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise SystemExit(f"git HEAD is not a lowercase 40-character SHA: {commit!r}")
    return commit


def require_repo_shape(args: argparse.Namespace) -> None:
    if args.skip_repo_shape:
        return

    branch = run_git(["branch", "--show-current"])
    if branch != "main" and not args.allow_non_main:
        raise SystemExit(f"Gate 2 outside-run readiness must be checked on main, got {branch!r}")

    origin = run_git(["remote", "get-url", "origin"])
    if origin != REMOTE_URL:
        raise SystemExit(f"origin must be {REMOTE_URL}, got {origin!r}")

    dirty = run_git(["status", "--porcelain"])
    if dirty and not args.allow_dirty:
        raise SystemExit(
            "worktree must be clean before handing Gate 2 to an outside runner; "
            "rerun with --allow-dirty only for local diagnostics"
        )


def validate_outside_proof_template() -> None:
    validator = repo_root() / "scripts/validate-gate2-outside-proof.sh"
    env = os.environ.copy()
    env.pop("BEATER_GATE2_OUTSIDE_PROOF", None)
    subprocess.check_call([str(validator), "--allow-pending"], cwd=repo_root(), env=env)


def registry_manifest_from_fixture(image_name: str, fixture_dir: Path) -> dict:
    path = fixture_dir / f"{image_name}.json"
    if not path.exists():
        raise SystemExit(f"missing registry fixture for {image_name}: {path}")
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as err:
        raise SystemExit(f"invalid registry fixture JSON for {image_name}: {err}") from err


def registry_manifest_from_ghcr(image_name: str, commit: str) -> dict:
    image = f"jadenfix/beater/{image_name}"
    token_url = f"https://ghcr.io/token?service=ghcr.io&scope=repository:{image}:pull"
    try:
        with urllib.request.urlopen(token_url, timeout=20) as response:
            token = json.load(response)["token"]
        request = urllib.request.Request(
            f"https://ghcr.io/v2/{image}/manifests/{commit}",
            headers={
                "Authorization": f"Bearer {token}",
                "Accept": (
                    "application/vnd.oci.image.index.v1+json, "
                    "application/vnd.docker.distribution.manifest.list.v2+json"
                ),
            },
        )
        with urllib.request.urlopen(request, timeout=20) as response:
            return json.load(response)
    except Exception as err:
        raise SystemExit(
            f"missing public GHCR manifest for ghcr.io/jadenfix/beater/{image_name}:{commit}: {err}"
        ) from err


def manifest_platforms(manifest: dict) -> list[str]:
    platforms = []
    for item in manifest.get("manifests", []):
        platform = item.get("platform", {})
        os_name = platform.get("os")
        arch = platform.get("architecture")
        if os_name and arch and os_name != "unknown":
            platforms.append(f"{os_name}/{arch}")
    return sorted(set(platforms))


def require_registry_images(args: argparse.Namespace, commit: str) -> None:
    fixture_dir = Path(args.registry_fixture) if args.registry_fixture else None
    for image_name in IMAGE_NAMES:
        manifest = (
            registry_manifest_from_fixture(image_name, fixture_dir)
            if fixture_dir
            else registry_manifest_from_ghcr(image_name, commit)
        )
        platforms = manifest_platforms(manifest)
        if platforms != EXPECTED_PLATFORMS:
            raise SystemExit(
                f"platforms mismatch for {image_name}:{commit}: "
                f"expected {EXPECTED_PLATFORMS}, got {platforms}"
        )
        print(f"ok image ghcr.io/jadenfix/beater/{image_name}:{commit} {platforms}")


def require_pinned_third_party_images() -> None:
    for compose_name, images in PINNED_THIRD_PARTY_IMAGES.items():
        compose = (repo_root() / compose_name).read_text()
        compose_lines = {line.strip() for line in compose.splitlines()}
        for image, digest in images:
            pinned = f"image: {image}@{digest}"
            floating = f"image: {image}"
            if pinned not in compose_lines:
                raise SystemExit(f"{compose_name} must pin {image} to {digest}")
            if floating in compose_lines:
                raise SystemExit(f"{compose_name} must not use floating image tag {image}")
            print(f"ok pinned {compose_name} {image}@{digest}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Check that the repo is ready to hand to an unaided Gate 2 outside runner."
        )
    )
    parser.add_argument(
        "--registry-fixture",
        help="Directory containing beaterd/dashboard/dashboard-e2e/otel-python JSON manifests.",
    )
    parser.add_argument(
        "--skip-repo-shape",
        action="store_true",
        help="Skip branch/origin/clean-worktree checks for fixture tests.",
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow local dirty worktree diagnostics; do not use for outside-run handoff.",
    )
    parser.add_argument(
        "--allow-non-main",
        action="store_true",
        help="Allow local diagnostics on a non-main branch; do not use for outside-run handoff.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    commit = current_commit()
    require_repo_shape(args)
    require_pinned_third_party_images()
    validate_outside_proof_template()
    require_registry_images(args, commit)
    print(f"Gate 2 outside-run readiness passed for {commit}")


if __name__ == "__main__":
    main()
