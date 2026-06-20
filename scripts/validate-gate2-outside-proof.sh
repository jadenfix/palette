#!/usr/bin/env bash
set -euo pipefail

proof_path="${BEATER_GATE2_OUTSIDE_PROOF:-docs/demos/gate2-outside-person-proof.md}"
allow_pending=0

if [[ "${1:-}" == "--allow-pending" ]]; then
  allow_pending=1
  shift
fi

if [[ $# -ne 0 ]]; then
  echo "usage: scripts/validate-gate2-outside-proof.sh [--allow-pending]" >&2
  exit 2
fi

python3 - "$proof_path" "$allow_pending" <<'PY'
import hashlib
import re
import subprocess
import sys
from pathlib import Path
from typing import Optional
from urllib.parse import parse_qs, urlparse

proof_path = Path(sys.argv[1])
allow_pending = sys.argv[2] == "1"
repo = Path.cwd()
errors: list[str] = []

if not proof_path.exists():
    raise SystemExit(f"missing outside-person proof file: {proof_path}")

text = proof_path.read_text()
DEFAULT_API_ENDPOINT = "http://127.0.0.1:8080"
DEFAULT_DASHBOARD_BASE = "http://127.0.0.1:3000"
DEFAULT_OTLP_ENDPOINT = "http://127.0.0.1:4317"
OUTSIDE_RUN_ATTESTATION = (
    "I attest that I am not a Beater project maintainer, I received no "
    "step-by-step help beyond public repository instructions, I used a fresh "
    "clone, and I completed the Gate 2 flow unaided."
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
]


def fail(message: str) -> None:
    errors.append(message)


def require_snippet(snippet: str, description: str) -> None:
    if snippet not in text:
        fail(f"missing {description}: {snippet}")


def clean_value(value: str) -> str:
    return value.strip().strip("`").strip()


def field_value_from(source_text: str, name: str, source_name: str) -> str:
    match = re.search(rf"^- {re.escape(name)}:[ \t]*(.*)$", source_text, re.MULTILINE)
    if not match:
        fail(f"missing field in {source_name}: {name}")
        return ""
    return clean_value(match.group(1))


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


def require_max_300(seconds: Optional[int], field_name: str, source_name: str) -> None:
    if seconds is not None and seconds > 300:
        fail(f"{field_name} in {source_name} exceeds 300 seconds")


def require_trace_id(name: str, value: str, source_name: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{32}", value):
        fail(f"{name} in {source_name} must be a lowercase 32-character trace id")


def require_image_digest(name: str, value: str, source_name: str) -> None:
    if not re.fullmatch(r"(?:[^`\s]+@)?sha256:[0-9a-f]{64}", value):
        fail(f"{name} in {source_name} must be a sha256 image digest")


def require_ghcr_image_digest(
    name: str, value: str, source_name: str, expected_image: str
) -> None:
    expected_prefix = f"ghcr.io/jadenfix/beater/{expected_image}@sha256:"
    if not re.fullmatch(re.escape(expected_prefix) + r"[0-9a-f]{64}", value):
        fail(f"{name} in {source_name} must be a GHCR repo digest for {expected_image}")


def require_ghcr_sha_image_ref(
    name: str, value: str, source_name: str, expected_image: str, commit_sha: str
) -> None:
    expected = f"ghcr.io/jadenfix/beater/{expected_image}:{commit_sha}"
    if value != expected:
        fail(f"{name} in {source_name} must be {expected!r}, got {value!r}")


def require_default_dashboard_url(name: str, value: str, trace_id: str) -> None:
    parsed = urlparse(value)
    if parsed.scheme != "http" or parsed.netloc != "127.0.0.1:3000":
        fail(f"{name} must use {DEFAULT_DASHBOARD_BASE}")
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
        "tokens",
        "cost",
        "latency",
        "run -> turn -> step -> tool -> MCP",
    ]
    missing = [fragment for fragment in required_fragments if fragment not in shows]
    if missing:
        fail(
            "screen recording notes Shows must describe the full Gate 2 flow; "
            "missing: " + ", ".join(missing)
        )


def repo_path(value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else repo / path


def repo_artifact_path(value: str, name: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        fail(f"{name} must be a repo-relative path under docs/demos")
    if ".." in path.parts:
        fail(f"{name} must not contain '..'")
    if len(path.parts) < 2 or path.parts[0] != "docs" or path.parts[1] != "demos":
        fail(f"{name} must live under docs/demos")
    return repo / path


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


def forbid_alternate_evidence(source_text: str, source_name: str) -> None:
    for forbidden in FORBIDDEN_EVIDENCE:
        if forbidden in source_text:
            fail(f"{source_name} must not use alternate/warm-loop evidence: {forbidden}")


status_match = re.search(r"^Status:\s*(.+)$", text, re.MULTILINE)
status = status_match.group(1).strip() if status_match else ""
if not status:
    fail("missing Status line")
elif status == "not yet completed." and allow_pending:
    pass
elif status != "completed.":
    fail("Status must be 'completed.' for Gate 2 closure")

for snippet, description in [
    ("scripts/gate2-outside-run.sh", "canonical outside-run command"),
    ("http://127.0.0.1:3000", "default dashboard URL"),
    ("Time-to-first-trace was 300 seconds or less", "first-trace checklist item"),
    ("Time-to-quickstart-click was 300 seconds or less", "browser-click checklist item"),
    ("using only public repository instructions", "unaided-run requirement"),
]:
    require_snippet(snippet, description)

if allow_pending and status == "not yet completed.":
    if errors:
        print("Gate 2 outside-person proof template is invalid:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        raise SystemExit(1)
    print(f"Gate 2 outside-person proof is pending but structurally valid: {proof_path}")
    raise SystemExit(0)

unresolved_fields = []
for field in [
    "Name",
    "Organization or relationship to project",
    "Prior Beater repo exposure",
    "Date",
    "Machine and OS",
    "Docker version",
    "Docker Compose version",
    "Browser",
    "Preflight status",
    "Outside-run attestation",
    "Commit SHA",
    "Branch",
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
    "Started at",
    "Ended at",
    "Time-to-first-trace",
    "Time-to-quickstart-click",
    "Total proof duration",
    "Stopwatch proof file",
    "Screen recording",
    "Screen recording notes",
    "Screen recording SHA256",
    "Terminal output excerpt",
    "Quickstart trace ID",
    "Quickstart dashboard URL",
    "All-kind nested trace ID",
    "All-kind dashboard URL",
]:
    value = field_value(field)
    if not value or value.endswith(":") or "none / describe" in value:
        unresolved_fields.append(field)
if unresolved_fields:
    fail("unresolved required fields: " + ", ".join(unresolved_fields))

outside_run_attestation = field_value("Outside-run attestation")
if outside_run_attestation != OUTSIDE_RUN_ATTESTATION:
    fail("Outside-run attestation must match the required unaided outside-run statement")

relationship = field_value("Organization or relationship to project").lower()
prior_exposure = field_value("Prior Beater repo exposure").lower()
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
commit_sha = field_value("Commit SHA")
require_current_or_evidence_only_commit(commit_sha)

api_endpoint = field_value("API endpoint")
if api_endpoint != DEFAULT_API_ENDPOINT:
    fail(f"API endpoint must be {DEFAULT_API_ENDPOINT}, got {api_endpoint!r}")
dashboard_base = field_value("Dashboard base")
if dashboard_base != DEFAULT_DASHBOARD_BASE:
    fail(f"Dashboard base must be {DEFAULT_DASHBOARD_BASE}, got {dashboard_base!r}")

if "- [ ]" in text:
    fail("all pass-checklist boxes must be checked")

forbid_alternate_evidence(text, "outside-person proof")

quickstart_trace_id = field_value("Quickstart trace ID")
all_kind_trace_id = field_value("All-kind nested trace ID")
require_trace_id("Quickstart trace ID", quickstart_trace_id, "outside-person proof")
require_trace_id("All-kind nested trace ID", all_kind_trace_id, "outside-person proof")
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
quickstart_url = field_value("Quickstart dashboard URL")
all_kind_url = field_value("All-kind dashboard URL")
require_default_dashboard_url("Quickstart dashboard URL", quickstart_url, quickstart_trace_id)
require_default_dashboard_url("All-kind dashboard URL", all_kind_url, all_kind_trace_id)

recording = field_value("Screen recording")
recording_path = repo_artifact_path(recording, "Screen recording") if recording else Path("")
if recording and not recording_path.exists():
    fail(f"screen recording does not exist: {recording}")
notes = field_value("Screen recording notes")
notes_path = repo_artifact_path(notes, "Screen recording notes") if notes else Path("")
if notes and not notes_path.exists():
    fail(f"screen recording notes do not exist: {notes}")
notes_text = notes_path.read_text() if notes and notes_path.exists() else ""
sha = field_value("Screen recording SHA256")
if sha and not re.fullmatch(r"[0-9a-f]{64}", sha):
    fail("Screen recording SHA256 must be a lowercase 64-character sha256")
if recording and recording_path.exists() and re.fullmatch(r"[0-9a-f]{64}", sha):
    actual = hashlib.sha256(recording_path.read_bytes()).hexdigest()
    if actual != sha:
        fail(f"screen recording sha mismatch: expected {sha}, got {actual}")

if notes_text:
    forbid_alternate_evidence(notes_text, "screen recording notes")
    notes_artifact = field_value_from(notes_text, "Artifact", "screen recording notes")
    notes_sha = field_value_from(notes_text, "SHA256", "screen recording notes")
    notes_dashboard_base = field_value_from(
        notes_text, "Dashboard base", "screen recording notes"
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
    repo_artifact_path(stopwatch_proof, "Stopwatch proof file")
    if stopwatch_proof
    else Path("")
)
stopwatch_text = ""
if stopwatch_proof and not stopwatch_path.exists():
    fail(f"stopwatch proof file does not exist: {stopwatch_proof}")
elif stopwatch_proof:
    stopwatch_text = stopwatch_path.read_text()

if stopwatch_text:
    forbid_alternate_evidence(stopwatch_text, "stopwatch proof")
    for field, expected in [
        ("Clean start", "yes"),
        ("Startup mode", "prebuilt-image"),
        ("Reuse override", "BEATER_GATE2_REUSE=0"),
        ("Prebuilt pull policy", "always"),
        ("Quickstart browser proof", "passed"),
        ("All-kind waterfall browser proof", "passed"),
        ("Browser recording", "passed"),
        ("API endpoint", DEFAULT_API_ENDPOINT),
        ("OTLP endpoint", DEFAULT_OTLP_ENDPOINT),
        ("Dashboard base", DEFAULT_DASHBOARD_BASE),
    ]:
        actual = field_value_from(stopwatch_text, field, "stopwatch proof")
        if actual != expected:
            fail(f"{field} in stopwatch proof must be {expected!r}, got {actual!r}")

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

    stopwatch_quickstart_trace = field_value_from(
        stopwatch_text, "Quickstart trace", "stopwatch proof"
    )
    stopwatch_all_kind_trace = field_value_from(
        stopwatch_text, "All-kind nested trace", "stopwatch proof"
    )
    require_trace_id("Quickstart trace", stopwatch_quickstart_trace, "stopwatch proof")
    require_trace_id("All-kind nested trace", stopwatch_all_kind_trace, "stopwatch proof")
    require_equal("quickstart trace id", quickstart_trace_id, stopwatch_quickstart_trace)
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

if errors:
    print("Gate 2 outside-person proof is not valid:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    raise SystemExit(1)

print(f"Gate 2 outside-person proof is complete and valid: {proof_path}")
PY
