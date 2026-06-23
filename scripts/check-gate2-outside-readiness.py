#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

sys.dont_write_bytecode = True

from gate2_proof_contract import REMOTE_URL


IMAGE_NAMES = ["beaterd", "dashboard", "dashboard-e2e", "otel-python"]
EXPECTED_PLATFORMS = ["linux/amd64", "linux/arm64"]
REMOTE_URL_NO_SUFFIX = REMOTE_URL.removesuffix(".git")
DEFAULT_COMPOSE_SERVICES = {"beaterd", "dashboard"}
PROFILED_THIRD_PARTY_SERVICES = {
    "docker-compose.yml": {
        "postgres": ["deps"],
        "nats": ["deps"],
        "minio": ["deps"],
        "clickhouse": ["clickhouse"],
    },
    "docker-compose.prebuilt.yml": {
        "postgres": ["deps"],
        "nats": ["deps"],
        "minio": ["deps"],
    },
}
TIMED_COMPOSE_SERVICES = {
    "docker-compose.yml": {
        "beaterd",
        "dashboard",
        "dashboard-e2e",
        "beaterctl",
        "otel-python-smoke",
        "otel-python-quickstart",
    },
    "docker-compose.prebuilt.yml": {
        "beaterd",
        "dashboard",
        "dashboard-e2e",
        "otel-python-smoke",
        "otel-python-quickstart",
    },
}
THIRD_PARTY_IMAGE_PREFIXES = (
    "postgres:",
    "nats:",
    "minio/",
    "clickhouse/",
)


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run_git(args: list[str]) -> str:
    command = ["git", *args]
    try:
        return subprocess.check_output(
            command, cwd=repo_root(), text=True, stderr=subprocess.DEVNULL
        ).strip()
    except FileNotFoundError as err:
        raise SystemExit("git is required to check Gate 2 outside-run readiness") from err
    except subprocess.CalledProcessError as err:
        raise SystemExit(f"{' '.join(command)} failed in {repo_root()}") from err


def current_commit() -> str:
    commit = run_git(["rev-parse", "HEAD"])
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise SystemExit(f"git HEAD is not a lowercase 40-character SHA: {commit!r}")
    return commit


def normalized_github_remote(url: str) -> str:
    normalized = url.rstrip("/")
    if normalized.endswith(".git"):
        normalized = normalized[:-4]
    return normalized


def require_repo_shape(args: argparse.Namespace) -> None:
    if args.skip_repo_shape:
        return

    branch = run_git(["branch", "--show-current"])
    if branch != "main" and not args.allow_non_main:
        raise SystemExit(f"Gate 2 outside-run readiness must be checked on main, got {branch!r}")

    origin = run_git(["remote", "get-url", "origin"])
    if normalized_github_remote(origin) != REMOTE_URL_NO_SUFFIX:
        raise SystemExit(
            f"origin must be {REMOTE_URL} or {REMOTE_URL_NO_SUFFIX}, got {origin!r}"
        )

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
    for compose_name in PROFILED_THIRD_PARTY_SERVICES:
        services = compose_service_blocks(compose_name)
        for service, body in services.items():
            image = service_image(body)
            if not uses_third_party_image(image):
                continue
            if "@sha256:" not in image:
                raise SystemExit(
                    f"{compose_name} service {service} must pin third-party image {image} "
                    "to a sha256 digest"
                )
            tag, digest = image.split("@", 1)
            if not re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
                raise SystemExit(
                    f"{compose_name} service {service} has invalid image digest {digest!r}"
                )
            floating = f"image: {tag}"
            if any(line.strip() == floating for line in body):
                raise SystemExit(
                    f"{compose_name} service {service} must not use floating image tag {tag}"
                )
            print(f"ok pinned {compose_name} {tag}@{digest}")


def line_indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def compose_service_blocks(compose_name: str) -> dict[str, list[str]]:
    services: dict[str, list[str]] = {}
    current_service: str | None = None
    in_services = False
    for line in (repo_root() / compose_name).read_text().splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        indent = line_indent(line)
        if indent == 0:
            if stripped == "services:":
                in_services = True
                current_service = None
                continue
            if in_services:
                break
        if not in_services:
            continue
        service_match = re.fullmatch(r"  ([A-Za-z0-9_.-]+):", line)
        if service_match:
            current_service = service_match.group(1)
            services[current_service] = []
            continue
        if current_service is not None:
            services[current_service].append(line)
    return services


def parse_inline_list(value: str) -> list[str]:
    value = value.strip()
    if not value:
        return []
    if value.startswith("[") and value.endswith("]"):
        items = value[1:-1].split(",")
    else:
        items = [value]
    return [item.strip().strip("'\"") for item in items if item.strip()]


def service_field_lines(body: list[str], field: str) -> tuple[int, str] | None:
    prefix = f"    {field}:"
    for index, line in enumerate(body):
        if line.startswith(prefix):
            return index, line[len(prefix) :].strip()
    return None


def service_profiles(body: list[str]) -> list[str]:
    field = service_field_lines(body, "profiles")
    if field is None:
        return []
    index, value = field
    profiles = parse_inline_list(value)
    if profiles:
        return profiles
    for line in body[index + 1 :]:
        if line_indent(line) <= 4:
            break
        item = re.fullmatch(r"\s*-\s*([^#]+?)\s*(?:#.*)?", line)
        if item:
            profiles.append(item.group(1).strip().strip("'\""))
    return profiles


def service_image(body: list[str]) -> str | None:
    field = service_field_lines(body, "image")
    if field is None:
        return None
    _, value = field
    return value


def service_depends_on(body: list[str]) -> set[str]:
    field = service_field_lines(body, "depends_on")
    if field is None:
        return set()
    index, value = field
    dependencies = set(parse_inline_list(value))
    for line in body[index + 1 :]:
        if line_indent(line) <= 4:
            break
        list_item = re.fullmatch(r"\s*-\s*([A-Za-z0-9_.-]+)\s*(?:#.*)?", line)
        mapping_item = re.fullmatch(r"      ([A-Za-z0-9_.-]+):.*", line)
        if list_item:
            dependencies.add(list_item.group(1))
        elif mapping_item:
            dependencies.add(mapping_item.group(1))
    return dependencies


def uses_third_party_image(image: str | None) -> bool:
    if image is None:
        return False
    return image.startswith(THIRD_PARTY_IMAGE_PREFIXES)


def require_compose_default_path_contract() -> None:
    third_party_services = {
        service
        for services in PROFILED_THIRD_PARTY_SERVICES.values()
        for service in services
    }
    for compose_name, expected_profiles in PROFILED_THIRD_PARTY_SERVICES.items():
        services = compose_service_blocks(compose_name)
        for service, profiles in expected_profiles.items():
            actual = service_profiles(services.get(service, []))
            if actual != profiles:
                raise SystemExit(
                    f"{compose_name} service {service} must set profiles {profiles}, got {actual}"
                )
        default_services = {
            service for service, body in services.items() if not service_profiles(body)
        }
        if default_services != DEFAULT_COMPOSE_SERVICES:
            raise SystemExit(
                f"{compose_name} default services must be exactly "
                f"{sorted(DEFAULT_COMPOSE_SERVICES)}, got {sorted(default_services)}"
            )
        for service in default_services:
            image = service_image(services[service])
            if uses_third_party_image(image):
                raise SystemExit(
                    f"{compose_name} default service {service} must not use third-party image {image}"
                )
        for service in TIMED_COMPOSE_SERVICES[compose_name] & services.keys():
            dependencies = service_depends_on(services[service])
            blocked = sorted(dependencies & third_party_services)
            if blocked:
                raise SystemExit(
                    f"{compose_name} default/timed service {service} must not depend on "
                    f"profiled third-party service(s): {', '.join(blocked)}"
                )
        print(
            f"ok default compose path {compose_name} services {sorted(DEFAULT_COMPOSE_SERVICES)}"
        )


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
    require_compose_default_path_contract()
    validate_outside_proof_template()
    require_registry_images(args, commit)
    print(f"Gate 2 outside-run readiness passed for {commit}")


if __name__ == "__main__":
    main()
