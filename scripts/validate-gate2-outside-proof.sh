#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -- "$script_dir/.." && pwd -P)"
proof_path="${BEATER_GATE2_OUTSIDE_PROOF:-docs/demos/gate2-outside-person-proof.md}"
allow_pending=0
diagnostic=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --allow-pending)
      allow_pending=1
      ;;
    --diagnostic)
      diagnostic=1
      ;;
    *)
      echo "usage: scripts/validate-gate2-outside-proof.sh [--allow-pending] [--diagnostic]" >&2
      exit 2
      ;;
  esac
  shift
done

if [[ $# -ne 0 ]]; then
  echo "usage: scripts/validate-gate2-outside-proof.sh [--allow-pending] [--diagnostic]" >&2
  exit 2
fi

python3 - "$proof_path" "$allow_pending" "$repo_root" "$diagnostic" <<'PY'
import hashlib
import datetime as dt
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path
from typing import Optional
from urllib.parse import parse_qs, urlparse

proof_arg = Path(sys.argv[1])
allow_pending = sys.argv[2] == "1"
repo = Path(sys.argv[3]).resolve()
diagnostic_mode = sys.argv[4] == "1"
proof_path = proof_arg if proof_arg.is_absolute() else repo / proof_arg
errors: list[str] = []

if not proof_path.exists():
    raise SystemExit(f"missing outside-person proof file: {proof_path}")

text = proof_path.read_text()
DEFAULT_API_ENDPOINT = "http://127.0.0.1:8080"
DEFAULT_DASHBOARD_BASE = "http://127.0.0.1:3000"
DEFAULT_OTLP_ENDPOINT = "http://127.0.0.1:4317"
EXPECTED_CLONE_URL = "https://github.com/jadenfix/beater.git"
EXPECTED_QUICKSTART_SNIPPET = "examples/python/five_line_otel.py"
EXPECTED_OUTSIDE_COMMAND = (
    "bash -o pipefail -lc 'sha_line=\"$(git ls-remote --exit-code "
    "https://github.com/jadenfix/beater.git refs/heads/main)\" && "
    "sha=\"${sha_line%%[[:space:]]*}\" && test -n \"$sha\" && "
    "preflight=\"$(mktemp \"${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX\")\" && "
    "curl -fsSL \"https://raw.githubusercontent.com/jadenfix/beater/$sha/"
    "scripts/gate2-outside-local-preflight.sh\" -o \"$preflight\" && bash \"$preflight\" && "
    "t=\"$(date +%s)\" && git clone https://github.com/jadenfix/beater.git && cd beater && "
    "test \"$(git rev-parse HEAD)\" = \"$sha\" && "
    "BEATER_GATE2_CLONE_STARTED_EPOCH=\"$t\" scripts/gate2-outside-run.sh'"
)
MIN_RECORDING_BYTES = 64 * 1024
MIN_RECORDING_SECONDS = 8.0
OUTSIDE_RUN_ATTESTATION = (
    "I attest that I am not a Beater project maintainer, I received no "
    "step-by-step help beyond public repository instructions, I used a fresh "
    "clone, and I completed the Gate 2 flow unaided."
)
OUTSIDE_RECORDING_NOTE = (
    "This recording was generated during the outside-person stopwatch path."
)
DIAGNOSTIC_ATTESTATION = (
    "Diagnostic maintainer full-run used a browser click to read the manual confirmation code; "
    "this is not outside-person evidence and cannot close Gate 2."
)
FORBIDDEN_EVIDENCE = [
    "http://127.0.0.1:13003",
    "http://127.0.0.1:13008",
    "http://127.0.0.1:13080",
    "http://127.0.0.1:14317",
    "BEATER_DASHBOARD_PORT=",
    "BEATER_HTTP_PORT=",
    "BEATER_OTLP_GRPC_PORT=",
    "BEATER_GATE2_REUSE=1",
    "COMPOSE_FILE=",
    "COMPOSE_PROJECT_NAME=",
    "COMPOSE_PROFILES=",
]
proof_abs = proof_path
default_proof_abs = repo / "docs/demos/gate2-outside-person-proof.md"
ALLOW_UNTRACKED_ARTIFACTS = (
    os.environ.get("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS") == "1"
    and proof_abs.resolve() != default_proof_abs.resolve()
)
REGISTRY_FIXTURE_ENV = "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS"
REGISTRY_FIXTURE_TEST_MARKER = ".gate2-registry-fixture-ok-for-tests"
registry_digest_cache: dict[tuple[str, str], set[str]] = {}


def fail(message: str) -> None:
    errors.append(message)


def require_snippet(snippet: str, description: str) -> None:
    if snippet not in text:
        fail(f"missing {description}: {snippet}")


def clean_value(value: str) -> str:
    return value.strip().strip("`").strip()


def field_value_from(source_text: str, name: str, source_name: str) -> str:
    matches = re.findall(rf"^- {re.escape(name)}:[ \t]*(.*)$", source_text, re.MULTILINE)
    if not matches:
        fail(f"missing field in {source_name}: {name}")
        return ""
    if len(matches) > 1:
        fail(f"duplicate field in {source_name}: {name}")
    return clean_value(matches[0])


def field_value(name: str) -> str:
    return field_value_from(text, name, "outside-person proof")


def parse_seconds_from_value(value: str, field_name: str, source_name: str) -> Optional[int]:
    match = re.fullmatch(r"([0-9]+)\s*(?:s|sec|secs|second|seconds)?", value)
    if not match:
        fail(f"{field_name} in {source_name} must be a numeric second count")
        return None
    return int(match.group(1))


def duration_seconds(source_text: str, field_name: str, source_name: str) -> Optional[int]:
    return parse_seconds_from_value(
        field_value_from(source_text, field_name, source_name), field_name, source_name
    )


def timestamp_value(
    source_text: str, field_name: str, source_name: str
) -> Optional[dt.datetime]:
    value = field_value_from(source_text, field_name, source_name)
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", value):
        fail(f"{field_name} in {source_name} must be UTC ISO-8601 like 2026-06-20T12:00:00Z")
        return None
    try:
        return dt.datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=dt.timezone.utc
        )
    except ValueError:
        fail(f"{field_name} in {source_name} must be a valid UTC timestamp")
        return None


def require_duration_matches(
    actual_seconds: Optional[int],
    expected_seconds: Optional[int],
    field_name: str,
    source_name: str,
) -> None:
    if actual_seconds is None or expected_seconds is None:
        return
    if abs(actual_seconds - expected_seconds) > 1:
        fail(
            f"{field_name} in {source_name} must match timestamps "
            f"(expected about {expected_seconds}s, got {actual_seconds}s)"
        )


def require_duration_at_most(
    actual_seconds: Optional[int],
    max_seconds: Optional[int],
    field_name: str,
    max_field_name: str,
    source_name: str,
) -> None:
    if actual_seconds is None or max_seconds is None:
        return
    if actual_seconds > max_seconds + 1:
        fail(f"{field_name} in {source_name} must be within {max_field_name}")


def require_relative_timing(
    source_text: str,
    source_name: str,
    total_relative_field: str,
    script_relative_field: str,
    total_duration: Optional[int],
    script_duration: Optional[int],
    total_duration_field: str,
    clone_to_script_seconds: int,
) -> None:
    total_relative = duration_seconds(source_text, total_relative_field, source_name)
    script_relative = duration_seconds(source_text, script_relative_field, source_name)
    require_duration_at_most(
        total_relative,
        total_duration,
        total_relative_field,
        total_duration_field,
        source_name,
    )
    require_duration_at_most(
        script_relative,
        script_duration,
        script_relative_field,
        "Script duration",
        source_name,
    )
    if total_relative is None or script_relative is None:
        return
    expected_total_relative = script_relative + clone_to_script_seconds
    if abs(total_relative - expected_total_relative) > 1:
        fail(
            f"{total_relative_field} in {source_name} must equal "
            f"{script_relative_field} plus clone-to-script time "
            f"(expected about {expected_total_relative}s, got {total_relative}s)"
        )


def require_timeline(
    source_text: str,
    source_name: str,
    started_field: str,
    ended_field: str,
    total_duration_field: str,
) -> None:
    clone_started = timestamp_value(source_text, "Clone started at", source_name)
    script_started = timestamp_value(source_text, "Script started at", source_name)
    started = timestamp_value(source_text, started_field, source_name)
    ended = timestamp_value(source_text, ended_field, source_name)
    if not all([clone_started, script_started, started, ended]):
        return
    if script_started != started:
        fail(f"Script started at and {started_field} in {source_name} must match")
    if clone_started > script_started:
        fail(f"Clone started at in {source_name} must not be after Script started at")
    if script_started > ended:
        fail(f"Script started at in {source_name} must be at or before {ended_field}")
    if clone_started > script_started or script_started > ended:
        return

    total_duration = duration_seconds(source_text, total_duration_field, source_name)
    script_duration = duration_seconds(source_text, "Script duration", source_name)
    total_expected = int((ended - clone_started).total_seconds())
    script_expected = int((ended - script_started).total_seconds())
    require_duration_matches(
        total_duration, total_expected, total_duration_field, source_name
    )
    require_duration_matches(script_duration, script_expected, "Script duration", source_name)
    clone_to_script_seconds = int((script_started - clone_started).total_seconds())
    require_relative_timing(
        source_text,
        source_name,
        "Time-to-first-trace",
        "Script-to-first-trace",
        total_duration,
        script_duration,
        total_duration_field,
        clone_to_script_seconds,
    )
    require_relative_timing(
        source_text,
        source_name,
        "Time-to-quickstart-click",
        "Script-to-quickstart-click",
        total_duration,
        script_duration,
        total_duration_field,
        clone_to_script_seconds,
    )


def require_max_300(seconds: Optional[int], field_name: str, source_name: str) -> None:
    if seconds is not None and seconds > 300:
        fail(f"{field_name} in {source_name} exceeds 300 seconds")


def require_trace_id(name: str, value: str, source_name: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{32}", value):
        fail(f"{name} in {source_name} must be a lowercase 32-character trace id")


def require_span_id(name: str, value: str, source_name: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{16}", value):
        fail(f"{name} in {source_name} must be a lowercase 16-character span id")


def require_confirmation_salt(name: str, value: str, source_name: str) -> None:
    if not re.fullmatch(r"[A-Za-z0-9._:-]{16,}", value):
        fail(f"{name} in {source_name} must be a concrete per-run salt")


def quickstart_confirmation_code(salt: str, trace_id: str, span_id: str) -> str:
    return hashlib.sha256(f"gate2:{salt}:{trace_id}:{span_id}".encode()).hexdigest()[:8].upper()


def require_confirmation_code(
    name: str, value: str, salt: str, trace_id: str, span_id: str, source_name: str
) -> None:
    expected = quickstart_confirmation_code(salt, trace_id, span_id)
    if value != expected:
        fail(
            f"{name} in {source_name} must be {expected} for quickstart span "
            f"{span_id} in trace {trace_id}"
        )


def require_quickstart_release_id(value: str, commit_sha: str, source_name: str) -> None:
    expected_prefix = f"gate2-{commit_sha[:12]}-"
    if not value.startswith(expected_prefix) or not re.fullmatch(
        re.escape(expected_prefix) + r"[0-9]+-[0-9]+", value
    ):
        fail(
            f"Quickstart release ID in {source_name} must be a fresh gate2 run id "
            f"for commit {commit_sha[:12]}"
        )


def require_ghcr_image_digest(
    name: str, value: str, source_name: str, expected_image: str
) -> None:
    expected_prefix = f"ghcr.io/jadenfix/beater/{expected_image}@sha256:"
    if not re.fullmatch(re.escape(expected_prefix) + r"[0-9a-f]{64}", value):
        fail(f"{name} in {source_name} must be a GHCR repo digest for {expected_image}")


def registry_manifest_from_fixture(image_name: str, fixture_dir: Path) -> tuple[dict, str]:
    path = fixture_dir / f"{image_name}.json"
    if not path.exists():
        fail(f"missing registry fixture for {image_name}: {path}")
        return {}, ""
    try:
        manifest = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as err:
        fail(f"invalid registry fixture JSON for {image_name}: {err}")
        return {}, ""
    digest = str(manifest.get("digest") or manifest.get("Docker-Content-Digest") or "")
    return manifest, digest


def registry_manifest_from_ghcr(image_name: str, commit_sha: str) -> tuple[dict, str]:
    image = f"jadenfix/beater/{image_name}"
    token_url = f"https://ghcr.io/token?service=ghcr.io&scope=repository:{image}:pull"
    try:
        with urllib.request.urlopen(token_url, timeout=20) as response:
            token = json.load(response)["token"]
        request = urllib.request.Request(
            f"https://ghcr.io/v2/{image}/manifests/{commit_sha}",
            headers={
                "Authorization": f"Bearer {token}",
                "Accept": (
                    "application/vnd.oci.image.index.v1+json, "
                    "application/vnd.docker.distribution.manifest.list.v2+json, "
                    "application/vnd.oci.image.manifest.v1+json, "
                    "application/vnd.docker.distribution.manifest.v2+json"
                ),
            },
        )
        with urllib.request.urlopen(request, timeout=20) as response:
            digest = response.headers.get("Docker-Content-Digest", "")
            return json.load(response), digest
    except Exception as err:
        fail(
            f"could not fetch public GHCR manifest for "
            f"ghcr.io/jadenfix/beater/{image_name}:{commit_sha}: {err}"
        )
        return {}, ""


def registry_manifest_for(image_name: str, commit_sha: str) -> tuple[dict, str]:
    fixture = os.environ.get(REGISTRY_FIXTURE_ENV)
    if fixture:
        test_marker = repo / REGISTRY_FIXTURE_TEST_MARKER
        if not (diagnostic_mode or ALLOW_UNTRACKED_ARTIFACTS or test_marker.is_file()):
            fail(
                f"{REGISTRY_FIXTURE_ENV} is only allowed for diagnostic or "
                "temporary generator validation, not normal Gate 2 closure"
            )
            return {}, ""
        return registry_manifest_from_fixture(image_name, Path(fixture))
    return registry_manifest_from_ghcr(image_name, commit_sha)


def registry_digest_refs(image_name: str, commit_sha: str) -> set[str]:
    key = (image_name, commit_sha)
    if key in registry_digest_cache:
        return registry_digest_cache[key]

    manifest, index_digest = registry_manifest_for(image_name, commit_sha)
    repo_ref = f"ghcr.io/jadenfix/beater/{image_name}@"
    digests: set[str] = set()
    for digest in [index_digest, str(manifest.get("digest") or "")]:
        if re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
            digests.add(f"{repo_ref}{digest}")
    for item in manifest.get("manifests", []):
        digest = str(item.get("digest") or "")
        if re.fullmatch(r"sha256:[0-9a-f]{64}", digest):
            digests.add(f"{repo_ref}{digest}")
    if not digests:
        fail(
            f"public GHCR manifest for ghcr.io/jadenfix/beater/{image_name}:{commit_sha} "
            "did not expose any sha256 manifest digests"
        )
    registry_digest_cache[key] = digests
    return digests


def require_digest_bound_to_registry(
    name: str, value: str, expected_image: str, commit_sha: str
) -> None:
    if ALLOW_UNTRACKED_ARTIFACTS:
        return
    allowed = registry_digest_refs(expected_image, commit_sha)
    if value not in allowed:
        fail(
            f"{name} in outside-person proof must match public GHCR manifest digest "
            f"for ghcr.io/jadenfix/beater/{expected_image}:{commit_sha}"
        )


def require_ghcr_sha_image_ref(
    name: str, value: str, source_name: str, expected_image: str, commit_sha: str
) -> None:
    expected = f"ghcr.io/jadenfix/beater/{expected_image}:{commit_sha}"
    if value != expected:
        fail(f"{name} in {source_name} must be {expected!r}, got {value!r}")


def require_terminal_excerpt(value: str, quickstart_url: str, all_kind_url: str) -> None:
    for snippet, description in [
        ("Gate 2 compose stopwatch passed", "compose stopwatch pass line"),
        ("Browser recording: passed", "browser recording pass line"),
        (quickstart_url, "quickstart dashboard URL"),
        (all_kind_url, "all-kind dashboard URL"),
    ]:
        if snippet not in value:
            fail(f"Terminal output excerpt must include {description}: {snippet}")


def require_compose_logs_saved(value: str) -> None:
    normalized = value.lower()
    if (
        not value
        or normalized in {"not saved", "none", "n/a", "na"}
        or normalized.startswith("not saved")
        or "not saved" in normalized
    ):
        fail("`docker compose` logs saved must identify saved logs for Gate 2 evidence")
        return
    if value.startswith("https://"):
        if not re.fullmatch(
            r"https://github\.com/jadenfix/beater/actions/runs/[0-9]+(?:/job/[0-9]+)?",
            value,
        ):
            fail(
                "`docker compose` logs saved must be a repo-relative docs/demos log "
                "file or immutable GitHub Actions run/job URL"
            )
        return
    log_path = validated_artifact_path(
        value,
        "`docker compose` logs saved",
        f"`docker compose` logs file does not exist: {value}",
    )
    if log_path:
        require_committed_clean_path(log_path, "`docker compose` logs saved")
        read_validated_text(log_path, "`docker compose` logs saved")


def require_default_dashboard_url(name: str, value: str, trace_id: str) -> None:
    parsed = urlparse(value)
    if parsed.scheme != "http" or parsed.netloc != "127.0.0.1:3000":
        fail(f"{name} must use {DEFAULT_DASHBOARD_BASE}")
    if parsed.path not in ("", "/"):
        fail(f"{name} must use the dashboard root path")
    if "..." in value:
        fail(f"{name} must be the concrete dashboard URL, not a placeholder")
    params = parse_qs(parsed.query)
    for key, expected in [
        ("tenant", "demo"),
        ("project", "demo"),
        ("environment", "local"),
        ("trace", trace_id),
    ]:
        actual = params.get(key, [])
        if actual != [expected]:
            fail(f"{name} must include {key}={expected}")


def require_compose_images_excerpt(
    value: str, commit_sha: str, expected_digests: dict[str, str]
) -> None:
    segments = [segment.strip() for segment in re.split(r"[|\n]", value) if segment.strip()]
    service_segments = [
        segment for segment in segments if not segment.startswith("proof-image ")
    ]
    for image, service in [
        ("beaterd", "beaterd"),
        ("dashboard", "dashboard"),
        ("dashboard-e2e", "dashboard-e2e"),
        ("otel-python", "otel-python"),
    ]:
        repo = f"ghcr.io/jadenfix/beater/{image}"
        expected_ref = f"{repo}:{commit_sha}"
        expected_digest = expected_digests[image]
        if service in {"beaterd", "dashboard"} and not any(
            repo in segment and commit_sha in segment for segment in service_segments
        ):
            fail(
                f"`docker compose images` excerpt must include {repo} "
                "tagged with the checked-out commit SHA"
            )
        if not any(
            segment.split()[:4] == ["proof-image", service, expected_ref, expected_digest]
            for segment in segments
        ):
            fail(
                f"`docker compose images` excerpt must include proof-image row for "
                f"{repo}:{commit_sha} with digest {expected_digest}"
            )


def require_equal(name: str, outside_value: str, stopwatch_value: str) -> None:
    if outside_value != stopwatch_value:
        fail(f"{name} mismatch between proof artifacts")


def require_recording_shows_full_flow(notes_text: str) -> None:
    shows = field_value_from(notes_text, "Shows", "screen recording notes")
    required_fragments = [
        "click five-line trace",
        "`llm.call` span",
        "prompt",
        "completion",
        "model",
        "token breakdown",
        "cost",
        "latency",
        "confirmation code",
        "run -> turn -> step -> tool -> MCP",
    ]
    missing = [fragment for fragment in required_fragments if fragment not in shows]
    if missing:
        fail(
            "screen recording notes Shows must describe the full Gate 2 flow; "
            "missing: " + ", ".join(missing)
        )


def require_recording_from_outside_wrapper(notes_text: str) -> None:
    if OUTSIDE_RECORDING_NOTE not in notes_text:
        fail(
            "screen recording notes must say the recording was generated "
            "during the outside-person stopwatch path"
        )


def require_runner_observation(
    field_name: str, value: str, required_fragments: list[str]
) -> None:
    normalized = value.lower()
    negated = re.search(
        r"\b(?:did\s+not|didn't|could\s+not|couldn't|cannot|can't|failed\s+to\s+see|"
        r"not\s+visible|not\s+shown|not\s+showing|missing|without)\b",
        normalized,
    )
    if negated:
        fail(f"{field_name} must be a positive observation, not negated evidence")
    if not re.search(
        r"\b(?:saw|seen|visible|read|confirmed|verified|opened|clicked|showed|displayed|inspected)\b",
        normalized,
    ):
        fail(f"{field_name} must describe a positive visible observation")
    missing = [
        fragment for fragment in required_fragments if fragment.lower() not in normalized
    ]
    if missing:
        fail(f"{field_name} must mention: " + ", ".join(missing))


EBML_ID = 0x1A45DFA3
DOCTYPE_ID = 0x4282
SEGMENT_ID = 0x18538067
INFO_ID = 0x1549A966
TRACKS_ID = 0x1654AE6B
TRACK_ENTRY_ID = 0xAE
TRACK_TYPE_ID = 0x83
CLUSTER_ID = 0x1F43B675
SIMPLE_BLOCK_ID = 0xA3
BLOCK_GROUP_ID = 0xA0
BLOCK_ID = 0xA1


def read_vint(
    data: bytes, offset: int, limit: int, max_len: int, strip_marker: bool
) -> Optional[tuple[int, int, bool]]:
    if offset >= limit:
        return None
    first = data[offset]
    if first == 0:
        return None
    marker = 0x80
    length = 1
    while length <= max_len and not (first & marker):
        marker >>= 1
        length += 1
    if length > max_len or offset + length > limit:
        return None
    value = first & (marker - 1) if strip_marker else first
    for index in range(1, length):
        value = (value << 8) | data[offset + index]
    unknown_size = strip_marker and value == (1 << (7 * length)) - 1
    return value, length, unknown_size


def read_ebml_element(
    data: bytes, offset: int, limit: int
) -> Optional[tuple[int, int, int, int]]:
    id_info = read_vint(data, offset, limit, 4, False)
    if id_info is None:
        return None
    element_id, id_len, _ = id_info
    size_info = read_vint(data, offset + id_len, limit, 8, True)
    if size_info is None:
        return None
    size, size_len, unknown_size = size_info
    payload_start = offset + id_len + size_len
    payload_end = limit if unknown_size else payload_start + size
    if payload_end > limit or payload_start > payload_end:
        return None
    return element_id, payload_start, payload_end, payload_end


def ebml_children(data: bytes, start: int, end: int) -> list[tuple[int, int, int, int]]:
    children = []
    offset = start
    while offset < end:
        element = read_ebml_element(data, offset, end)
        if element is None:
            break
        children.append(element)
        next_offset = element[3]
        if next_offset <= offset:
            break
        offset = next_offset
    return children


def ebml_uint(data: bytes, start: int, end: int) -> Optional[int]:
    if start >= end:
        return None
    value = 0
    for byte in data[start:end]:
        value = (value << 8) | byte
    return value


def tracks_have_video(data: bytes, start: int, end: int) -> bool:
    for element_id, payload_start, payload_end, _ in ebml_children(data, start, end):
        if element_id != TRACK_ENTRY_ID:
            continue
        for child_id, child_start, child_end, _ in ebml_children(
            data, payload_start, payload_end
        ):
            if child_id == TRACK_TYPE_ID and ebml_uint(data, child_start, child_end) == 1:
                return True
    return False


def cluster_has_block_payload(data: bytes, start: int, end: int) -> bool:
    for element_id, payload_start, payload_end, _ in ebml_children(data, start, end):
        if element_id == SIMPLE_BLOCK_ID and payload_end - payload_start > 4:
            return True
        if element_id != BLOCK_GROUP_ID:
            continue
        for child_id, child_start, child_end, _ in ebml_children(
            data, payload_start, payload_end
        ):
            if child_id == BLOCK_ID and child_end - child_start > 4:
                return True
    return False


def require_webm_structure(recording_bytes: bytes) -> None:
    ebml = read_ebml_element(recording_bytes, 0, len(recording_bytes))
    if ebml is None or ebml[0] != EBML_ID:
        fail("screen recording must start with a valid EBML header element")
        return

    doc_type = None
    for element_id, payload_start, payload_end, _ in ebml_children(
        recording_bytes, ebml[1], ebml[2]
    ):
        if element_id == DOCTYPE_ID:
            doc_type = recording_bytes[payload_start:payload_end]
            break
    if doc_type != b"webm":
        fail("screen recording must declare WebM DocType in its EBML header")

    segment = None
    for element in ebml_children(recording_bytes, ebml[3], len(recording_bytes)):
        if element[0] == SEGMENT_ID:
            segment = element
            break
    if segment is None:
        fail("screen recording WebM must contain a Segment element")
        return

    has_info = False
    has_tracks = False
    has_video_track = False
    has_cluster = False
    has_block_payload = False
    for element_id, payload_start, payload_end, _ in ebml_children(
        recording_bytes, segment[1], segment[2]
    ):
        if element_id == INFO_ID:
            has_info = True
        elif element_id == TRACKS_ID:
            has_tracks = True
            has_video_track = has_video_track or tracks_have_video(
                recording_bytes, payload_start, payload_end
            )
        elif element_id == CLUSTER_ID:
            has_cluster = True
            has_block_payload = has_block_payload or cluster_has_block_payload(
                recording_bytes, payload_start, payload_end
            )
        if has_info and has_tracks and has_video_track and has_cluster and has_block_payload:
            break

    if not has_info:
        fail("screen recording WebM must contain an Info element")
    if not has_tracks:
        fail("screen recording WebM must contain a Tracks element")
    if not has_video_track:
        fail("screen recording WebM must contain a video track")
    if not has_cluster:
        fail("screen recording WebM must contain a Cluster element")
    if not has_block_payload:
        fail("screen recording WebM must contain a Cluster block payload")


def require_webm_recording(recording_path: Path) -> None:
    recording_bytes = read_validated_bytes(recording_path, "Screen recording")
    if len(recording_bytes) < MIN_RECORDING_BYTES:
        fail(
            "screen recording must be a real WebM capture of at least "
            f"{MIN_RECORDING_BYTES} bytes"
        )
    if not recording_bytes.startswith(bytes.fromhex("1a45dfa3")):
        fail("screen recording must start with a WebM/EBML header")
    if b"webm" not in recording_bytes[:4096]:
        fail("screen recording must declare WebM DocType in its EBML header")
    require_webm_structure(recording_bytes)

    ffprobe = shutil.which("ffprobe")
    if not ffprobe:
        fail("screen recording validation requires ffprobe on PATH")
        return
    try:
        probe = subprocess.run(
            [
                ffprobe,
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=codec_type",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=0",
                str(recording_path),
            ],
            cwd=repo,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as err:
        fail(f"screen recording ffprobe validation failed to start: {err}")
        return
    if probe.returncode != 0:
        fail("screen recording must be a playable WebM video: ffprobe failed")
        return
    if "codec_type=video" not in probe.stdout:
        fail("screen recording must contain a video stream")
    duration_match = re.search(r"^duration=([0-9]+(?:\.[0-9]+)?)$", probe.stdout, re.MULTILINE)
    if not duration_match or float(duration_match.group(1)) <= 0:
        fail("screen recording must have a positive video duration")
        return
    duration = float(duration_match.group(1))
    if duration < MIN_RECORDING_SECONDS:
        fail(
            "screen recording must be a reviewable full-flow capture of at least "
            f"{MIN_RECORDING_SECONDS:g} seconds, got {duration:g}"
        )


def repo_artifact_path(value: str, name: str) -> Optional[Path]:
    path = Path(value)
    valid = True
    if path.is_absolute():
        fail(f"{name} must be a repo-relative path under docs/demos")
        valid = False
    if ".." in path.parts:
        fail(f"{name} must not contain '..'")
        valid = False
    if len(path.parts) < 2 or path.parts[0] != "docs" or path.parts[1] != "demos":
        fail(f"{name} must live under docs/demos")
        valid = False
    if not valid:
        return None
    return repo / path


def require_tracked_artifact(path: Path, name: str) -> bool:
    try:
        rel = path.relative_to(repo)
    except ValueError:
        fail(f"{name} must be inside the repository")
        return False
    probe = repo
    for part in rel.parts:
        probe = probe / part
        try:
            if probe.is_symlink():
                fail(f"{name} must not be a symlink: {rel}")
                return False
        except OSError as err:
            fail(f"{name} could not be inspected for symlinks: {err}")
            return False
    if not path.is_file():
        fail(f"{name} must be a regular file: {rel}")
        return False
    try:
        path.resolve().relative_to(repo.resolve())
    except ValueError:
        fail(f"{name} must resolve inside the repository")
        return False
    if ALLOW_UNTRACKED_ARTIFACTS:
        return True
    try:
        subprocess.check_call(
            ["git", "ls-files", "--error-unmatch", "--", str(rel)],
            cwd=repo,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        fail(f"{name} must be tracked by git before Gate 2 closure: {rel}")
        return False
    return True


def require_committed_clean_path(path: Path, name: str) -> None:
    if ALLOW_UNTRACKED_ARTIFACTS:
        return
    try:
        rel = path.relative_to(repo)
    except ValueError:
        fail(f"{name} must be inside the repository")
        return
    try:
        subprocess.check_call(
            ["git", "ls-files", "--error-unmatch", "--", str(rel)],
            cwd=repo,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        fail(f"{name} must be tracked by git before Gate 2 closure: {rel}")
        return
    try:
        status = subprocess.check_output(
            ["git", "status", "--porcelain=v1", "--", str(rel)],
            cwd=repo,
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        fail(f"could not inspect git status for {name}: {rel}")
        return
    if status:
        fail(f"{name} must be committed and clean before Gate 2 closure: {rel}")


def require_regenerated_after_tested_commit(path: Path, name: str, commit_sha: str) -> None:
    if ALLOW_UNTRACKED_ARTIFACTS or diagnostic_mode:
        return
    try:
        rel = path.relative_to(repo).as_posix()
    except ValueError:
        fail(f"{name} must be inside the repository")
        return
    current_head = git_head()
    if not current_head:
        fail(f"could not inspect HEAD while checking {name} freshness")
        return
    if commit_sha == current_head:
        fail(f"{name} must be committed after the tested Commit SHA: {rel}")
        return
    try:
        tested_blob = git_output(["rev-parse", f"{commit_sha}:{rel}"])
    except (OSError, subprocess.CalledProcessError):
        return
    try:
        current_blob = git_output(["rev-parse", f"HEAD:{rel}"])
    except (OSError, subprocess.CalledProcessError):
        fail(f"could not inspect committed blob for {name}: {rel}")
        return
    if tested_blob == current_blob:
        fail(f"{name} must be regenerated after tested Commit SHA: {rel}")


def validated_artifact_path(value: str, name: str, missing_message: str) -> Optional[Path]:
    path = repo_artifact_path(value, name)
    if path is None:
        return None
    if not path.exists():
        fail(missing_message)
        return None
    if not require_tracked_artifact(path, name):
        return None
    return path


def read_validated_text(path: Path, name: str) -> str:
    try:
        return path.read_text()
    except UnicodeDecodeError:
        fail(f"{name} must be valid UTF-8 text")
    except OSError as err:
        fail(f"{name} could not be read: {err}")
    return ""


def read_validated_bytes(path: Path, name: str) -> bytes:
    try:
        return path.read_bytes()
    except OSError as err:
        fail(f"{name} could not be read: {err}")
    return b""


def git_output(args: list[str]) -> str:
    return subprocess.check_output(
        ["git", *args], cwd=repo, text=True, stderr=subprocess.DEVNULL
    ).strip()


def git_head() -> str:
    try:
        return git_output(["rev-parse", "HEAD"])
    except (OSError, subprocess.CalledProcessError):
        return ""


def require_current_or_evidence_only_commit(commit_sha: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", commit_sha):
        fail("Commit SHA must be a lowercase 40-character git SHA")
        return
    current_head = git_head()
    if not current_head or commit_sha == current_head:
        return
    try:
        subprocess.check_call(
            ["git", "merge-base", "--is-ancestor", commit_sha, "HEAD"],
            cwd=repo,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        changed_paths = git_output(["diff", "--name-only", f"{commit_sha}..HEAD"])
    except (OSError, subprocess.CalledProcessError):
        fail("Commit SHA must match current HEAD or be an evidence-only ancestor")
        return
    non_evidence_paths = [
        path for path in changed_paths.splitlines() if not path.startswith("docs/demos/")
    ]
    if non_evidence_paths:
        fail(
            "Commit SHA must match current HEAD or be followed only by docs/demos evidence changes"
        )


def dirty_worktree_paths() -> list[str]:
    try:
        raw = subprocess.check_output(
            ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
            cwd=repo,
            stderr=subprocess.DEVNULL,
        )
    except (OSError, subprocess.CalledProcessError):
        fail("could not inspect git worktree cleanliness")
        return []

    parts = [
        part
        for part in raw.decode("utf-8", errors="surrogateescape").split("\0")
        if part
    ]
    paths: list[str] = []
    index = 0
    while index < len(parts):
        entry = parts[index]
        status = entry[:2]
        path = entry[3:] if len(entry) > 3 else ""
        if path:
            paths.append(path)
        index += 2 if "R" in status or "C" in status else 1
    return paths


def require_no_dirty_non_evidence_worktree() -> None:
    if ALLOW_UNTRACKED_ARTIFACTS:
        return
    non_evidence_paths = [
        path for path in dirty_worktree_paths() if not path.startswith("docs/demos/")
    ]
    if non_evidence_paths:
        fail(
            "Completed Gate 2 closure proof has uncommitted non-evidence worktree changes"
        )


def forbid_alternate_evidence(source_text: str, source_name: str) -> None:
    for forbidden in FORBIDDEN_EVIDENCE:
        if forbidden in source_text:
            fail(f"{source_name} must not use alternate/warm-loop evidence: {forbidden}")


REQUIRED_PROOF_FIELDS = [
    "Name",
    "Organization or relationship to project",
    "Prior Beater repo exposure",
    "Date",
    "Machine and OS",
    "Docker version",
    "Docker Compose version",
    "Browser",
    "Network notes",
    "Preflight status",
    "Outside-run attestation",
    "Clone URL",
    "Commit SHA",
    "Branch",
    "Worktree clean",
    "OS/arch",
    "Beater image reference",
    "Dashboard image reference",
    "Dashboard e2e image reference",
    "OTEL Python image reference",
    "Beater image digest",
    "Dashboard image digest",
    "Dashboard e2e image digest",
    "OTEL Python image digest",
    "API endpoint",
    "Dashboard base",
    "Quickstart release ID",
    "Timing start source",
    "Clone started at",
    "Script started at",
    "Started at",
    "Ended at",
    "Time-to-first-trace",
    "Script-to-first-trace",
    "Time-to-quickstart-click",
    "Script-to-quickstart-click",
    "Quickstart click source",
    "Manual quickstart confirmation",
    "Manual confirmation code",
    "Manual confirmation salt",
    "Total proof duration",
    "Script duration",
    "Outside-run wrapper",
    "Stopwatch proof file",
    "Screen recording",
    "Screen recording notes",
    "Screen recording SHA256",
    "Terminal output excerpt",
    "Runner llm.call observation",
    "Runner waterfall observation",
    "`docker compose images` excerpt",
    "Quickstart trace ID",
    "Quickstart span ID",
    "Quickstart dashboard URL",
    "All-kind nested trace ID",
    "All-kind dashboard URL",
    "`docker compose` logs saved",
    "Failure notes, if any",
]


def require_pending_template_fields() -> None:
    for field in REQUIRED_PROOF_FIELDS:
        field_value_from(text, field, "pending outside-person proof template")


status_matches = re.findall(r"^Status:\s*(.+)$", text, re.MULTILINE)
status = status_matches[0].strip() if status_matches else ""
if not status_matches:
    fail("missing Status line")
elif len(status_matches) > 1:
    fail("duplicate Status line")
elif status == "not yet completed." and allow_pending:
    pass
elif status == "diagnostic." and diagnostic_mode:
    pass
elif status != "completed.":
    fail("Status must be 'completed.' for Gate 2 closure")
if diagnostic_mode and status != "diagnostic.":
    fail("Diagnostic validation requires Status: diagnostic.")

required_snippets = [
    (EXPECTED_OUTSIDE_COMMAND, "fail-fast clone-to-browser command"),
    ("scripts/gate2-outside-run.sh", "canonical outside-run command"),
    ("BEATER_GATE2_CLONE_STARTED_EPOCH", "clone-to-browser stopwatch command"),
    ("http://127.0.0.1:3000", "default dashboard URL"),
    ("Time-to-first-trace was 300 seconds or less", "first-trace checklist item"),
    ("Time-to-first-trace includes clone time", "clone-inclusive timing checklist item"),
    (
        "Manual quickstart click confirmation code was recorded before 300 seconds",
        "manual browser-click checklist item",
    ),
]
if diagnostic_mode:
    required_snippets.append(("not outside-person evidence", "diagnostic non-evidence marker"))
else:
    required_snippets.append(
        ("using only public repository instructions", "unaided-run requirement")
    )
for snippet, description in required_snippets:
    require_snippet(snippet, description)

if allow_pending and status == "not yet completed.":
    require_pending_template_fields()
    if errors:
        print("Gate 2 outside-person proof template is invalid:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        raise SystemExit(1)
    print(f"Gate 2 outside-person proof is pending but structurally valid: {proof_path}")
    raise SystemExit(0)

if proof_abs.resolve() == default_proof_abs.resolve():
    require_committed_clean_path(proof_abs, "Outside-person proof")

UNRESOLVED_REQUIRED_VALUES = {
    "...",
    "…",
    "unknown",
    "not requested",
    "not reported",
    "tbd",
    "todo",
}
CONCRETE_REQUIRED_FIELDS = {
    "Name",
    "Organization or relationship to project",
    "Prior Beater repo exposure",
    "Machine and OS",
    "Browser",
    "Network notes",
}
EMBEDDED_PLACEHOLDER = re.compile(r"(^|[\s:;,/()_-])(\.\.\.|…|tbd|todo)($|[\s:;,/()_-])", re.I)


def contains_placeholder_fragment(value: str) -> bool:
    return bool(EMBEDDED_PLACEHOLDER.search(value))


def require_date_field(name: str, value: str, source_name: str) -> Optional[dt.date]:
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", value):
        fail(f"{name} in {source_name} must be a valid date like 2026-06-20")
        return None
    try:
        return dt.date.fromisoformat(value)
    except ValueError:
        fail(f"{name} in {source_name} must be a valid date like 2026-06-20")
        return None


def contradiction_text(value: str) -> str:
    cleaned = value.lower()
    negated_patterns = [
        r"\bnot\s+(?:a\s+|an\s+)?(?:beater\s+project\s+)?maintainer\b",
        r"\bno\s+(?:beater\s+)?maintainer\s+role\b",
        r"\bnot\s+(?:an\s+)?internal\b",
        r"\bnot\s+(?:an\s+)?employee\b",
        r"\bnot\s+(?:a\s+)?founder\b",
        r"\bno\s+(?:beater\s+team|project\s+team)\s+role\b",
        r"\bnot\s+(?:on|part\s+of)\s+(?:the\s+)?(?:beater\s+team|project\s+team)\b",
    ]
    for pattern in negated_patterns:
        cleaned = re.sub(pattern, "", cleaned)
    return cleaned

unresolved_fields = []
for field in REQUIRED_PROOF_FIELDS:
    value = field_value(field)
    normalized_value = value.lower()
    if (
        not value
        or value.endswith(":")
        or "none / describe" in value
        or normalized_value in UNRESOLVED_REQUIRED_VALUES
        or contains_placeholder_fragment(value)
    ):
        unresolved_fields.append(field)
    if (
        field in CONCRETE_REQUIRED_FIELDS
        and normalized_value == "none"
        and field != "Prior Beater repo exposure"
    ):
        unresolved_fields.append(field)
if unresolved_fields:
    fail("unresolved required fields: " + ", ".join(dict.fromkeys(unresolved_fields)))

proof_date = require_date_field("Date", field_value("Date"), "outside-person proof")

outside_run_attestation = field_value("Outside-run attestation")
if diagnostic_mode:
    if outside_run_attestation != DIAGNOSTIC_ATTESTATION:
        fail("Diagnostic attestation must mark the proof as non-closure evidence")
elif outside_run_attestation != OUTSIDE_RUN_ATTESTATION:
    fail("Outside-run attestation must match the required unaided outside-run statement")
preflight_status = field_value("Preflight status")
if preflight_status != "passed":
    fail("Preflight status must be passed")

relationship = contradiction_text(field_value("Organization or relationship to project"))
prior_exposure = contradiction_text(field_value("Prior Beater repo exposure"))
if not diagnostic_mode:
    for outside_contradiction in [
        "maintainer",
        "internal",
        "employee",
        "founder",
        "beater team",
        "project team",
    ]:
        if outside_contradiction in relationship or outside_contradiction in prior_exposure:
            fail("runner relationship/prior exposure must not contradict outside-run attestation")

branch = field_value("Branch")
if branch != "main":
    fail(f"Branch must be main, got {branch!r}")
clone_url = field_value("Clone URL")
if clone_url != EXPECTED_CLONE_URL:
    fail(f"Clone URL must be {EXPECTED_CLONE_URL}, got {clone_url!r}")
worktree_clean = field_value("Worktree clean")
if worktree_clean != "yes":
    fail("Worktree clean must be yes")
commit_sha = field_value("Commit SHA")
require_current_or_evidence_only_commit(commit_sha)
require_no_dirty_non_evidence_worktree()
if proof_abs.resolve() == default_proof_abs.resolve():
    require_regenerated_after_tested_commit(
        proof_abs, "Outside-person proof", commit_sha
    )

api_endpoint = field_value("API endpoint")
if api_endpoint != DEFAULT_API_ENDPOINT:
    fail(f"API endpoint must be {DEFAULT_API_ENDPOINT}, got {api_endpoint!r}")
dashboard_base = field_value("Dashboard base")
if dashboard_base != DEFAULT_DASHBOARD_BASE:
    fail(f"Dashboard base must be {DEFAULT_DASHBOARD_BASE}, got {dashboard_base!r}")
outside_wrapper = field_value("Outside-run wrapper")
if outside_wrapper != "yes":
    fail("Outside-run wrapper must be yes; use scripts/gate2-outside-run.sh for evidence")
quickstart_click_source = field_value("Quickstart click source")
if quickstart_click_source != "manual-outside-runner":
    fail("Quickstart click source must be manual-outside-runner for outside-person evidence")
manual_quickstart_confirmation = field_value("Manual quickstart confirmation")
if manual_quickstart_confirmation != "yes":
    fail("Manual quickstart confirmation must be yes for outside-person evidence")
manual_confirmation_code = field_value("Manual confirmation code")
manual_confirmation_salt = field_value("Manual confirmation salt")
require_confirmation_salt(
    "Manual confirmation salt",
    manual_confirmation_salt,
    "outside-person proof",
)
timing_start_source = field_value("Timing start source")
if timing_start_source != "external-clone":
    fail("Timing start source must be external-clone for outside-person evidence")
clone_started_at = field_value("Clone started at")
if clone_started_at == "not provided":
    fail("Clone started at must be captured before git clone")
clone_started_timestamp = timestamp_value(
    text, "Clone started at", "outside-person proof"
)
if (
    proof_date is not None
    and clone_started_timestamp is not None
    and proof_date != clone_started_timestamp.date()
):
    fail(
        "Date in outside-person proof must match Clone started at UTC date "
        f"{clone_started_timestamp.date().isoformat()}"
    )
require_timeline(
    text,
    "outside-person proof",
    "Started at",
    "Ended at",
    "Total proof duration",
)

if "- [ ]" in text:
    fail("all pass-checklist boxes must be checked")

forbid_alternate_evidence(text, "outside-person proof")
compose_images_excerpt = field_value("`docker compose images` excerpt")

quickstart_trace_id = field_value("Quickstart trace ID")
quickstart_span_id = field_value("Quickstart span ID")
all_kind_trace_id = field_value("All-kind nested trace ID")
quickstart_release_id = field_value("Quickstart release ID")
require_trace_id("Quickstart trace ID", quickstart_trace_id, "outside-person proof")
require_span_id("Quickstart span ID", quickstart_span_id, "outside-person proof")
require_trace_id("All-kind nested trace ID", all_kind_trace_id, "outside-person proof")
require_confirmation_code(
    "Manual confirmation code",
    manual_confirmation_code,
    manual_confirmation_salt,
    quickstart_trace_id,
    quickstart_span_id,
    "outside-person proof",
)
require_quickstart_release_id(
    quickstart_release_id, commit_sha, "outside-person proof"
)
if quickstart_trace_id == all_kind_trace_id:
    fail("Quickstart trace ID and All-kind nested trace ID must be distinct")
beater_image_ref = field_value("Beater image reference")
dashboard_image_ref = field_value("Dashboard image reference")
dashboard_e2e_image_ref = field_value("Dashboard e2e image reference")
otel_python_image_ref = field_value("OTEL Python image reference")
beater_image_digest = field_value("Beater image digest")
dashboard_image_digest = field_value("Dashboard image digest")
dashboard_e2e_image_digest = field_value("Dashboard e2e image digest")
otel_python_image_digest = field_value("OTEL Python image digest")
require_ghcr_sha_image_ref(
    "Beater image reference",
    beater_image_ref,
    "outside-person proof",
    "beaterd",
    commit_sha,
)
require_ghcr_sha_image_ref(
    "Dashboard image reference",
    dashboard_image_ref,
    "outside-person proof",
    "dashboard",
    commit_sha,
)
require_ghcr_sha_image_ref(
    "Dashboard e2e image reference",
    dashboard_e2e_image_ref,
    "outside-person proof",
    "dashboard-e2e",
    commit_sha,
)
require_ghcr_sha_image_ref(
    "OTEL Python image reference",
    otel_python_image_ref,
    "outside-person proof",
    "otel-python",
    commit_sha,
)
require_ghcr_image_digest(
    "Beater image digest", beater_image_digest, "outside-person proof", "beaterd"
)
require_ghcr_image_digest(
    "Dashboard image digest",
    dashboard_image_digest,
    "outside-person proof",
    "dashboard",
)
require_ghcr_image_digest(
    "Dashboard e2e image digest",
    dashboard_e2e_image_digest,
    "outside-person proof",
    "dashboard-e2e",
)
require_ghcr_image_digest(
    "OTEL Python image digest",
    otel_python_image_digest,
    "outside-person proof",
    "otel-python",
)
require_digest_bound_to_registry(
    "Beater image digest", beater_image_digest, "beaterd", commit_sha
)
require_digest_bound_to_registry(
    "Dashboard image digest", dashboard_image_digest, "dashboard", commit_sha
)
require_digest_bound_to_registry(
    "Dashboard e2e image digest",
    dashboard_e2e_image_digest,
    "dashboard-e2e",
    commit_sha,
)
require_digest_bound_to_registry(
    "OTEL Python image digest", otel_python_image_digest, "otel-python", commit_sha
)
require_compose_images_excerpt(
    compose_images_excerpt,
    commit_sha,
    {
        "beaterd": beater_image_digest,
        "dashboard": dashboard_image_digest,
        "dashboard-e2e": dashboard_e2e_image_digest,
        "otel-python": otel_python_image_digest,
    },
)
quickstart_url = field_value("Quickstart dashboard URL")
all_kind_url = field_value("All-kind dashboard URL")
require_default_dashboard_url("Quickstart dashboard URL", quickstart_url, quickstart_trace_id)
require_default_dashboard_url("All-kind dashboard URL", all_kind_url, all_kind_trace_id)
require_terminal_excerpt(field_value("Terminal output excerpt"), quickstart_url, all_kind_url)
require_compose_logs_saved(field_value("`docker compose` logs saved"))
require_runner_observation(
    "Runner llm.call observation",
    field_value("Runner llm.call observation"),
    [
        "llm.call",
        "prompt",
        "completion",
        "model",
        "token breakdown",
        "cost",
        "latency",
        "confirmation code",
    ],
)
require_runner_observation(
    "Runner waterfall observation",
    field_value("Runner waterfall observation"),
    ["run", "turn", "step", "tool", "MCP"],
)

recording = field_value("Screen recording")
recording_path = (
    validated_artifact_path(
        recording,
        "Screen recording",
        f"screen recording does not exist: {recording}",
    )
    if recording
    else None
)
notes = field_value("Screen recording notes")
notes_path = (
    validated_artifact_path(
        notes,
        "Screen recording notes",
        f"screen recording notes do not exist: {notes}",
    )
    if notes
    else None
)
if recording_path:
    require_committed_clean_path(recording_path, "Screen recording")
    require_regenerated_after_tested_commit(
        recording_path, "Screen recording", commit_sha
    )
if notes_path:
    require_committed_clean_path(notes_path, "Screen recording notes")
    require_regenerated_after_tested_commit(
        notes_path, "Screen recording notes", commit_sha
    )
notes_text = read_validated_text(notes_path, "Screen recording notes") if notes_path else ""
sha = field_value("Screen recording SHA256")
if sha and not re.fullmatch(r"[0-9a-f]{64}", sha):
    fail("Screen recording SHA256 must be a lowercase 64-character sha256")
if recording_path and re.fullmatch(r"[0-9a-f]{64}", sha):
    require_webm_recording(recording_path)
    actual = hashlib.sha256(read_validated_bytes(recording_path, "Screen recording")).hexdigest()
    if actual != sha:
        fail(f"screen recording sha mismatch: expected {sha}, got {actual}")

if notes_text:
    forbid_alternate_evidence(notes_text, "screen recording notes")
    notes_artifact = field_value_from(notes_text, "Artifact", "screen recording notes")
    notes_sha = field_value_from(notes_text, "SHA256", "screen recording notes")
    notes_dashboard_base = field_value_from(
        notes_text, "Dashboard base", "screen recording notes"
    )
    notes_recording_mode = field_value_from(
        notes_text, "Recording mode", "screen recording notes"
    )
    notes_quickstart_trace = field_value_from(
        notes_text, "Quickstart trace", "screen recording notes"
    )
    notes_all_kind_trace = field_value_from(
        notes_text, "All-kind trace", "screen recording notes"
    )
    if recording and notes_artifact != Path(recording).name:
        fail("screen recording notes artifact must match the screen recording filename")
    require_equal("screen recording notes sha256", sha, notes_sha)
    if notes_dashboard_base != DEFAULT_DASHBOARD_BASE:
        fail(
            "screen recording notes dashboard base must be "
            f"{DEFAULT_DASHBOARD_BASE}, got {notes_dashboard_base!r}"
        )
    if notes_recording_mode != "compose":
        fail("screen recording notes Recording mode must be compose for outside-person proof")
    require_trace_id(
        "Quickstart trace", notes_quickstart_trace, "screen recording notes"
    )
    require_trace_id("All-kind trace", notes_all_kind_trace, "screen recording notes")
    require_equal(
        "screen recording notes quickstart trace",
        quickstart_trace_id,
        notes_quickstart_trace,
    )
    require_equal(
        "screen recording notes all-kind trace", all_kind_trace_id, notes_all_kind_trace
    )
    require_recording_shows_full_flow(notes_text)
    require_recording_from_outside_wrapper(notes_text)

require_max_300(
    duration_seconds(text, "Time-to-first-trace", "outside-person proof"),
    "Time-to-first-trace",
    "outside-person proof",
)
require_max_300(
    duration_seconds(text, "Time-to-quickstart-click", "outside-person proof"),
    "Time-to-quickstart-click",
    "outside-person proof",
)

stopwatch_proof = field_value("Stopwatch proof file")
stopwatch_path = (
    validated_artifact_path(
        stopwatch_proof,
        "Stopwatch proof file",
        f"stopwatch proof file does not exist: {stopwatch_proof}",
    )
    if stopwatch_proof
    else None
)
if stopwatch_path:
    require_committed_clean_path(stopwatch_path, "Stopwatch proof file")
    require_regenerated_after_tested_commit(
        stopwatch_path, "Stopwatch proof file", commit_sha
    )
stopwatch_text = ""
if stopwatch_path:
    stopwatch_text = read_validated_text(stopwatch_path, "Stopwatch proof file")

if stopwatch_text:
    forbid_alternate_evidence(stopwatch_text, "stopwatch proof")
    if "This is an automated local stopwatch proof" in stopwatch_text:
        fail(
            "stopwatch proof for outside-person evidence must identify itself "
            "as outside-run source evidence, not automated local proof"
        )
    if "outside-run stopwatch source artifact" not in stopwatch_text:
        fail("stopwatch proof must identify itself as outside-run source evidence")
    for field, expected in [
        ("Clean start", "yes"),
        ("Startup mode", "prebuilt-image"),
        ("Reuse override", "BEATER_GATE2_REUSE=0"),
        ("Outside-run wrapper", "yes"),
        ("Timing start source", "external-clone"),
        ("Git branch", "main"),
        ("Git origin", EXPECTED_CLONE_URL),
        ("Git worktree clean", "yes"),
        ("Prebuilt pull policy", "always"),
        ("Compose project", "beater-stopwatch"),
        ("Quickstart browser proof", "passed"),
        ("All-kind waterfall browser proof", "passed"),
        ("Browser recording", "passed"),
        ("Quickstart click source", "manual-outside-runner"),
        ("Manual quickstart confirmation", "yes"),
        ("API endpoint", DEFAULT_API_ENDPOINT),
        ("OTLP endpoint", DEFAULT_OTLP_ENDPOINT),
        ("Dashboard base", DEFAULT_DASHBOARD_BASE),
        ("Quickstart snippet", EXPECTED_QUICKSTART_SNIPPET),
    ]:
        actual = field_value_from(stopwatch_text, field, "stopwatch proof")
        if actual != expected:
            fail(f"{field} in stopwatch proof must be {expected!r}, got {actual!r}")
    stopwatch_quickstart_release_id = field_value_from(
        stopwatch_text, "Quickstart release ID", "stopwatch proof"
    )
    require_quickstart_release_id(
        stopwatch_quickstart_release_id, commit_sha, "stopwatch proof"
    )
    require_equal(
        "quickstart release ID",
        quickstart_release_id,
        stopwatch_quickstart_release_id,
    )
    require_timeline(
        stopwatch_text,
        "stopwatch proof",
        "Started",
        "Ended",
        "Total duration",
    )

    stopwatch_first_trace = duration_seconds(
        stopwatch_text, "Time-to-first-trace", "stopwatch proof"
    )
    stopwatch_quickstart_click = duration_seconds(
        stopwatch_text, "Time-to-quickstart-click", "stopwatch proof"
    )
    require_max_300(stopwatch_first_trace, "Time-to-first-trace", "stopwatch proof")
    require_max_300(
        stopwatch_quickstart_click, "Time-to-quickstart-click", "stopwatch proof"
    )
    require_equal(
        "time-to-first-trace",
        field_value("Time-to-first-trace"),
        field_value_from(stopwatch_text, "Time-to-first-trace", "stopwatch proof"),
    )
    require_equal(
        "time-to-quickstart-click",
        field_value("Time-to-quickstart-click"),
        field_value_from(stopwatch_text, "Time-to-quickstart-click", "stopwatch proof"),
    )
    require_equal(
        "total proof duration",
        field_value("Total proof duration"),
        field_value_from(stopwatch_text, "Total duration", "stopwatch proof"),
    )
    require_equal(
        "Docker version",
        field_value("Docker version"),
        field_value_from(stopwatch_text, "Docker", "stopwatch proof"),
    )
    require_equal(
        "Docker Compose version",
        field_value("Docker Compose version"),
        field_value_from(stopwatch_text, "Docker Compose", "stopwatch proof"),
    )
    require_equal(
        "OS/arch",
        field_value("OS/arch"),
        field_value_from(stopwatch_text, "OS/arch", "stopwatch proof"),
    )
    require_equal(
        "started time",
        field_value("Started at"),
        field_value_from(stopwatch_text, "Started", "stopwatch proof"),
    )
    require_equal(
        "ended time",
        field_value("Ended at"),
        field_value_from(stopwatch_text, "Ended", "stopwatch proof"),
    )
    stopwatch_quickstart_trace = field_value_from(
        stopwatch_text, "Quickstart trace", "stopwatch proof"
    )
    stopwatch_quickstart_span = field_value_from(
        stopwatch_text, "Quickstart span", "stopwatch proof"
    )
    stopwatch_manual_confirmation_code = field_value_from(
        stopwatch_text, "Manual confirmation code", "stopwatch proof"
    )
    stopwatch_manual_confirmation_salt = field_value_from(
        stopwatch_text, "Manual confirmation salt", "stopwatch proof"
    )
    stopwatch_all_kind_trace = field_value_from(
        stopwatch_text, "All-kind nested trace", "stopwatch proof"
    )
    require_trace_id("Quickstart trace", stopwatch_quickstart_trace, "stopwatch proof")
    require_span_id("Quickstart span", stopwatch_quickstart_span, "stopwatch proof")
    require_confirmation_salt(
        "Manual confirmation salt",
        stopwatch_manual_confirmation_salt,
        "stopwatch proof",
    )
    require_trace_id("All-kind nested trace", stopwatch_all_kind_trace, "stopwatch proof")
    require_confirmation_code(
        "Manual confirmation code",
        stopwatch_manual_confirmation_code,
        stopwatch_manual_confirmation_salt,
        stopwatch_quickstart_trace,
        stopwatch_quickstart_span,
        "stopwatch proof",
    )
    require_equal("quickstart trace id", quickstart_trace_id, stopwatch_quickstart_trace)
    require_equal("quickstart span id", quickstart_span_id, stopwatch_quickstart_span)
    require_equal(
        "manual confirmation code",
        manual_confirmation_code,
        stopwatch_manual_confirmation_code,
    )
    require_equal(
        "manual confirmation salt",
        manual_confirmation_salt,
        stopwatch_manual_confirmation_salt,
    )
    require_equal("all-kind trace id", all_kind_trace_id, stopwatch_all_kind_trace)

    stopwatch_quickstart_url = field_value_from(
        stopwatch_text, "Quickstart dashboard", "stopwatch proof"
    )
    stopwatch_all_kind_url = field_value_from(
        stopwatch_text, "All-kind dashboard", "stopwatch proof"
    )
    require_default_dashboard_url(
        "Quickstart dashboard", stopwatch_quickstart_url, stopwatch_quickstart_trace
    )
    require_default_dashboard_url(
        "All-kind dashboard", stopwatch_all_kind_url, stopwatch_all_kind_trace
    )
    require_equal("quickstart dashboard URL", quickstart_url, stopwatch_quickstart_url)
    require_equal("all-kind dashboard URL", all_kind_url, stopwatch_all_kind_url)

    stopwatch_api_endpoint = field_value_from(
        stopwatch_text, "API endpoint", "stopwatch proof"
    )
    stopwatch_dashboard_base = field_value_from(
        stopwatch_text, "Dashboard base", "stopwatch proof"
    )
    require_equal("API endpoint", api_endpoint, stopwatch_api_endpoint)
    require_equal("Dashboard base", dashboard_base, stopwatch_dashboard_base)
    require_equal(
        "timing start source",
        timing_start_source,
        field_value_from(stopwatch_text, "Timing start source", "stopwatch proof"),
    )
    require_equal(
        "clone start time",
        clone_started_at,
        field_value_from(stopwatch_text, "Clone started at", "stopwatch proof"),
    )
    require_equal(
        "script start time",
        field_value("Script started at"),
        field_value_from(stopwatch_text, "Script started at", "stopwatch proof"),
    )
    require_equal(
        "script-to-first-trace",
        field_value("Script-to-first-trace"),
        field_value_from(stopwatch_text, "Script-to-first-trace", "stopwatch proof"),
    )
    require_equal(
        "script-to-quickstart-click",
        field_value("Script-to-quickstart-click"),
        field_value_from(stopwatch_text, "Script-to-quickstart-click", "stopwatch proof"),
    )
    require_equal(
        "quickstart click source",
        quickstart_click_source,
        field_value_from(stopwatch_text, "Quickstart click source", "stopwatch proof"),
    )
    require_equal(
        "manual quickstart confirmation",
        manual_quickstart_confirmation,
        field_value_from(
            stopwatch_text,
            "Manual quickstart confirmation",
            "stopwatch proof",
        ),
    )
    require_equal(
        "script duration",
        field_value("Script duration"),
        field_value_from(stopwatch_text, "Script duration", "stopwatch proof"),
    )

    stopwatch_outside_wrapper = field_value_from(
        stopwatch_text, "Outside-run wrapper", "stopwatch proof"
    )
    require_equal("outside-run wrapper", outside_wrapper, stopwatch_outside_wrapper)

    stopwatch_beater_image_ref = field_value_from(
        stopwatch_text, "Beater image reference", "stopwatch proof"
    )
    stopwatch_dashboard_image_ref = field_value_from(
        stopwatch_text, "Dashboard image reference", "stopwatch proof"
    )
    stopwatch_dashboard_e2e_image_ref = field_value_from(
        stopwatch_text, "Dashboard e2e image reference", "stopwatch proof"
    )
    stopwatch_otel_python_image_ref = field_value_from(
        stopwatch_text, "OTEL Python image reference", "stopwatch proof"
    )
    require_ghcr_sha_image_ref(
        "Beater image reference",
        stopwatch_beater_image_ref,
        "stopwatch proof",
        "beaterd",
        commit_sha,
    )
    require_ghcr_sha_image_ref(
        "Dashboard image reference",
        stopwatch_dashboard_image_ref,
        "stopwatch proof",
        "dashboard",
        commit_sha,
    )
    require_ghcr_sha_image_ref(
        "Dashboard e2e image reference",
        stopwatch_dashboard_e2e_image_ref,
        "stopwatch proof",
        "dashboard-e2e",
        commit_sha,
    )
    require_ghcr_sha_image_ref(
        "OTEL Python image reference",
        stopwatch_otel_python_image_ref,
        "stopwatch proof",
        "otel-python",
        commit_sha,
    )
    require_equal("beater image reference", beater_image_ref, stopwatch_beater_image_ref)
    require_equal(
        "dashboard image reference", dashboard_image_ref, stopwatch_dashboard_image_ref
    )
    require_equal(
        "dashboard e2e image reference",
        dashboard_e2e_image_ref,
        stopwatch_dashboard_e2e_image_ref,
    )
    require_equal(
        "otel python image reference",
        otel_python_image_ref,
        stopwatch_otel_python_image_ref,
    )

    stopwatch_recording = field_value_from(
        stopwatch_text, "Browser recording artifact", "stopwatch proof"
    )
    stopwatch_notes = field_value_from(
        stopwatch_text, "Browser recording notes", "stopwatch proof"
    )
    stopwatch_sha = field_value_from(
        stopwatch_text, "Browser recording SHA256", "stopwatch proof"
    )
    require_equal("screen recording path", recording, stopwatch_recording)
    require_equal("screen recording notes path", notes, stopwatch_notes)
    require_equal("screen recording sha256", sha, stopwatch_sha)

    stopwatch_beater_image_digest = field_value_from(
        stopwatch_text, "Beater image digest", "stopwatch proof"
    )
    stopwatch_dashboard_image_digest = field_value_from(
        stopwatch_text, "Dashboard image digest", "stopwatch proof"
    )
    stopwatch_dashboard_e2e_image_digest = field_value_from(
        stopwatch_text, "Dashboard e2e image digest", "stopwatch proof"
    )
    stopwatch_otel_python_image_digest = field_value_from(
        stopwatch_text, "OTEL Python image digest", "stopwatch proof"
    )
    require_ghcr_image_digest(
        "Beater image digest",
        stopwatch_beater_image_digest,
        "stopwatch proof",
        "beaterd",
    )
    require_ghcr_image_digest(
        "Dashboard image digest",
        stopwatch_dashboard_image_digest,
        "stopwatch proof",
        "dashboard",
    )
    require_ghcr_image_digest(
        "Dashboard e2e image digest",
        stopwatch_dashboard_e2e_image_digest,
        "stopwatch proof",
        "dashboard-e2e",
    )
    require_ghcr_image_digest(
        "OTEL Python image digest",
        stopwatch_otel_python_image_digest,
        "stopwatch proof",
        "otel-python",
    )
    require_equal(
        "beater image digest", beater_image_digest, stopwatch_beater_image_digest
    )
    require_equal(
        "dashboard image digest",
        dashboard_image_digest,
        stopwatch_dashboard_image_digest,
    )
    require_equal(
        "dashboard e2e image digest",
        dashboard_e2e_image_digest,
        stopwatch_dashboard_e2e_image_digest,
    )
    require_equal(
        "otel python image digest",
        otel_python_image_digest,
        stopwatch_otel_python_image_digest,
    )

    outside_commit_sha = field_value("Commit SHA")
    stopwatch_commit_sha = field_value_from(stopwatch_text, "Git SHA", "stopwatch proof")
    require_equal("commit SHA", outside_commit_sha, stopwatch_commit_sha)
    stopwatch_branch = field_value_from(stopwatch_text, "Git branch", "stopwatch proof")
    require_equal("branch", branch, stopwatch_branch)
    stopwatch_origin = field_value_from(stopwatch_text, "Git origin", "stopwatch proof")
    require_equal("clone URL", clone_url, stopwatch_origin)
    stopwatch_worktree_clean = field_value_from(
        stopwatch_text, "Git worktree clean", "stopwatch proof"
    )
    require_equal("worktree clean", worktree_clean, stopwatch_worktree_clean)

if errors:
    print("Gate 2 outside-person proof is not valid:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    raise SystemExit(1)

if diagnostic_mode:
    print(f"Gate 2 diagnostic proof is valid: {proof_path}")
elif ALLOW_UNTRACKED_ARTIFACTS:
    print(f"Gate 2 outside-person proof draft is internally consistent: {proof_path}")
else:
    print(f"Gate 2 outside-person proof is complete and valid: {proof_path}")
PY
