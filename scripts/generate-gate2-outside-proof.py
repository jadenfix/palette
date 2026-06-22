#!/usr/bin/env python3
import argparse
import datetime as dt
import os
import re
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path


CANONICAL_COMMAND = "scripts/gate2-outside-run.sh"
OUTSIDE_RUN_ATTESTATION = (
    "I attest that I am not a Beater project maintainer, I received no "
    "step-by-step help beyond public repository instructions, I used a fresh "
    "clone, and I completed the Gate 2 flow unaided."
)
DIAGNOSTIC_ATTESTATION = (
    "Diagnostic maintainer full-run auto-confirmed the manual checkpoint; "
    "this is not outside-person evidence and cannot close Gate 2."
)
UNRESOLVED_REQUIRED_VALUES = {
    "...",
    "…",
    "unknown",
    "not requested",
    "not reported",
    "tbd",
    "todo",
}
EMBEDDED_PLACEHOLDER = re.compile(r"(^|[\s:;,/()_-])(\.\.\.|…|tbd|todo)($|[\s:;,/()_-])", re.I)
IMMUTABLE_LOG_URL = re.compile(
    r"https://github\.com/jadenfix/beater/actions/runs/[0-9]+(?:/job/[0-9]+)?"
)


def clean_value(value):
    return value.strip().strip("`").strip()


def require_meaningful_arg(name, value, *, allow_none=False):
    cleaned = clean_value(value)
    if (
        not cleaned
        or cleaned.lower() in UNRESOLVED_REQUIRED_VALUES
        or (cleaned.lower() == "none" and not allow_none)
        or EMBEDDED_PLACEHOLDER.search(cleaned)
    ):
        raise SystemExit(f"{name} must be provided with a concrete value")
    return cleaned


def require_date_arg(name, value):
    cleaned = require_meaningful_arg(name, value)
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", cleaned):
        raise SystemExit(f"{name} must be YYYY-MM-DD")
    try:
        dt.date.fromisoformat(cleaned)
    except ValueError:
        raise SystemExit(f"{name} must be YYYY-MM-DD") from None
    return cleaned


def field_value(source_text, name, source_name):
    matches = re.findall(
        r"^- " + re.escape(name) + r":[ \t]*(.*)$", source_text, re.MULTILINE
    )
    if not matches:
        raise SystemExit(f"missing field in {source_name}: {name}")
    if len(matches) > 1:
        raise SystemExit(f"duplicate field in {source_name}: {name}")
    value = clean_value(matches[0])
    if not value or value.lower() in UNRESOLVED_REQUIRED_VALUES:
        raise SystemExit(f"unusable field in {source_name}: {name}={value!r}")
    if EMBEDDED_PLACEHOLDER.search(value):
        raise SystemExit(f"unusable field in {source_name}: {name} contains placeholder text")
    return value


def require_source_field_equal(source_text, source_name, name, expected):
    value = field_value(source_text, name, source_name)
    if value != expected:
        raise SystemExit(
            f"{name} in {source_name} must be {expected!r} before generating proof; "
            f"got {value!r}"
        )
    return value


def require_source_sha256(source_text, source_name, name):
    value = field_value(source_text, name, source_name)
    if not re.fullmatch(r"[0-9a-f]{64}", value):
        raise SystemExit(
            f"{name} in {source_name} must be a lowercase 64-character sha256 "
            "before generating proof"
        )
    return value


def require_compose_logs_saved_arg(value):
    cleaned = require_meaningful_arg("--compose-logs-saved", value)
    normalized = cleaned.lower()
    if (
        normalized in {"not saved", "none", "n/a", "na"}
        or normalized.startswith("not saved")
        or "not saved" in normalized
    ):
        raise SystemExit("--compose-logs-saved must identify saved logs")
    if cleaned.startswith("https://"):
        if not IMMUTABLE_LOG_URL.fullmatch(cleaned):
            raise SystemExit(
                "--compose-logs-saved must be a repo-relative docs/demos log file "
                "or immutable GitHub Actions run/job URL"
            )
        return cleaned
    path = Path(cleaned)
    if path.is_absolute() or ".." in path.parts:
        raise SystemExit("--compose-logs-saved must be a repo-relative path under docs/demos")
    if len(path.parts) < 2 or path.parts[0] != "docs" or path.parts[1] != "demos":
        raise SystemExit("--compose-logs-saved must live under docs/demos")
    log_path = repo_root() / path
    if not log_path.is_file():
        raise SystemExit(f"--compose-logs-saved file does not exist: {cleaned}")
    try:
        log_path.resolve().relative_to(repo_root().resolve())
    except ValueError:
        raise SystemExit("--compose-logs-saved must resolve inside the repository") from None
    probe = repo_root()
    for part in path.parts:
        probe = probe / part
        if probe.is_symlink():
            raise SystemExit(f"--compose-logs-saved must not be a symlink: {cleaned}")
    return cleaned


def timestamp_date(source_text, name, source_name):
    value = field_value(source_text, name, source_name)
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", value):
        raise SystemExit(f"{name} in {source_name} must be UTC ISO-8601")
    try:
        return dt.datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").date().isoformat()
    except ValueError:
        raise SystemExit(f"{name} in {source_name} must be a valid UTC timestamp") from None


def repo_root():
    return Path(__file__).resolve().parent.parent


def relative_or_absolute(path):
    try:
        return str(path.resolve().relative_to(repo_root()))
    except ValueError:
        return str(path)


def compose_images_excerpt(stopwatch_text, stopwatch_path):
    match = re.search(r"## Compose Images\s+```text\n(.*?)\n```", stopwatch_text, re.DOTALL)
    if not match:
        return f"see {relative_or_absolute(stopwatch_path)}"
    lines = [line.strip() for line in match.group(1).splitlines() if line.strip()]
    if not lines:
        return f"see {relative_or_absolute(stopwatch_path)}"
    gate2_repos = {
        "ghcr.io/jadenfix/beater/beaterd",
        "ghcr.io/jadenfix/beater/dashboard",
        "ghcr.io/jadenfix/beater/dashboard-e2e",
        "ghcr.io/jadenfix/beater/otel-python",
    }
    services = [line for line in lines if any(repo in line for repo in gate2_repos)]
    if services:
        return " | ".join(services)
    return " | ".join(lines[:3])


def terminal_output_excerpt_from_stopwatch(stopwatch_text, stopwatch_rel):
    quickstart_dashboard = field_value(stopwatch_text, "Quickstart dashboard", stopwatch_rel)
    all_kind_dashboard = field_value(stopwatch_text, "All-kind dashboard", stopwatch_rel)
    recording_status = field_value(stopwatch_text, "Browser recording", stopwatch_rel)
    return (
        f"Gate 2 compose stopwatch passed; Browser recording: {recording_status}; "
        f"Quickstart dashboard: {quickstart_dashboard}; "
        f"All-kind dashboard: {all_kind_dashboard}"
    )


def compose_logs_from_stopwatch(stopwatch_text, stopwatch_rel):
    return field_value(stopwatch_text, "Compose logs artifact", stopwatch_rel)


def print_prefilled_command(args, stopwatch_path, output_path, stopwatch_text):
    stopwatch_rel = relative_or_absolute(stopwatch_path)
    require_source_field_equal(
        stopwatch_text, stopwatch_rel, "Browser recording", "passed"
    )
    compose_logs_saved = compose_logs_from_stopwatch(stopwatch_text, stopwatch_rel)
    terminal_excerpt = terminal_output_excerpt_from_stopwatch(stopwatch_text, stopwatch_rel)
    command = [
        "python3",
        "scripts/generate-gate2-outside-proof.py",
        "--stopwatch-proof",
        relative_or_absolute(stopwatch_path),
        "--output",
        relative_or_absolute(output_path),
        "--runner-name",
        "... runner full name ...",
        "--relationship",
        "... external relationship to Beater; no project role ...",
        "--prior-exposure",
        "none",
        "--machine-os",
        "... OS and architecture ...",
        "--browser",
        "... browser and version ...",
        "--network-notes",
        "... network used; mention VPN/proxy if any ...",
        "--llm-observation",
        "... clicked llm.call and saw prompt, completion, model, token breakdown, cost, and latency ...",
        "--waterfall-observation",
        "... opened all-kind trace and saw run -> turn -> step -> tool -> MCP nesting ...",
        "--terminal-output-excerpt",
        terminal_excerpt,
        "--compose-logs-saved",
        compose_logs_saved,
        "--preflight-status",
        "passed",
        "--failure-notes",
        "none",
        "--runner-notes",
        "... add any confusing step, or say no extra runner notes ...",
        "--attest-outside-run",
    ]
    print("Start from this command after replacing every ... field with runner-specific facts:")
    for index in range(0, len(command), 2):
        if index == 0:
            print(f"{shlex.quote(command[0])} {shlex.quote(command[1])} \\")
            continue
        suffix = " \\" if index + 2 < len(command) else ""
        if index + 1 < len(command):
            print(f"  {shlex.quote(command[index])} {shlex.quote(command[index + 1])}{suffix}")
        else:
            print(f"  {shlex.quote(command[index])}{suffix}")


def proof_status(text, output_path):
    matches = re.findall(r"^Status:\s*(.+)$", text, re.MULTILINE)
    if len(matches) != 1:
        raise SystemExit(
            f"{output_path} must contain exactly one top-level Status line; pass --force"
        )
    return matches[0].strip()


def require_pending_or_force(output_path, force):
    if not output_path.exists() or force:
        return
    text = output_path.read_text()
    if proof_status(text, output_path) == "not yet completed.":
        return
    raise SystemExit(f"{output_path} already exists and is not the pending template; pass --force")


def build_proof(args, stopwatch_path, stopwatch_text):
    stopwatch_rel = relative_or_absolute(stopwatch_path)
    require_source_field_equal(
        stopwatch_text, stopwatch_rel, "Browser recording", "passed"
    )
    recording = field_value(stopwatch_text, "Browser recording artifact", stopwatch_rel)
    notes = field_value(stopwatch_text, "Browser recording notes", stopwatch_rel)
    recording_sha = require_source_sha256(
        stopwatch_text, stopwatch_rel, "Browser recording SHA256"
    )
    quickstart_dashboard_url = field_value(stopwatch_text, "Quickstart dashboard", stopwatch_rel)
    all_kind_dashboard_url = field_value(stopwatch_text, "All-kind dashboard", stopwatch_rel)

    terminal_excerpt = require_meaningful_arg(
        "--terminal-output-excerpt", args.terminal_output_excerpt
    )
    logs_saved = require_compose_logs_saved_arg(args.compose_logs_saved)
    failure_notes = args.failure_notes or "none"
    runner_notes = args.runner_notes or "No extra runner notes."
    network_notes = require_meaningful_arg("--network-notes", args.network_notes)
    llm_observation = require_meaningful_arg("--llm-observation", args.llm_observation)
    waterfall_observation = require_meaningful_arg(
        "--waterfall-observation", args.waterfall_observation
    )
    runner_name = require_meaningful_arg("--runner-name", args.runner_name)
    relationship = require_meaningful_arg("--relationship", args.relationship)
    prior_exposure = require_meaningful_arg(
        "--prior-exposure", args.prior_exposure, allow_none=True
    )
    machine_os = require_meaningful_arg("--machine-os", args.machine_os)
    browser = require_meaningful_arg("--browser", args.browser)
    preflight_status = require_meaningful_arg(
        "--preflight-status", args.preflight_status
    )
    run_date = timestamp_date(stopwatch_text, "Clone started at", stopwatch_rel)
    proof_date = require_date_arg("--date", args.date) if args.date else run_date
    if proof_date != run_date:
        raise SystemExit(
            f"--date must match Clone started at UTC date {run_date}, got {proof_date}"
        )
    status = "diagnostic." if args.diagnostic_report else "completed."
    attestation = DIAGNOSTIC_ATTESTATION if args.diagnostic_report else OUTSIDE_RUN_ATTESTATION
    if args.diagnostic_report:
        proof_intro = (
            "Maintainer diagnostic report generated from the stopwatch proof listed below. "
            "This is not outside-person evidence, auto-confirms the manual checkpoint for "
            "automation, and cannot close Gate 2."
        )
        command_note = (
            "The maintainer diagnostic full-run exercised the public wrapper path and "
            "auto-confirmed the manual quickstart checkpoint. This is not outside-person "
            "evidence."
        )
        recording_checklist = (
            "- [x] A screen recording of the diagnostic full-run was generated under "
            "`docs/demos/`."
        )
        runner_checklist = (
            "- [x] The diagnostic verifier auto-confirmed the manual quickstart "
            "checkpoint; not outside-person evidence."
        )
    else:
        proof_intro = (
            "Gate 2 evidence generated from the stopwatch proof listed below. This file is "
            "valid only when the named runner is outside the project and completed the run "
            "unaided using public repository instructions."
        )
        command_note = "The runner completed the flow using only public repository instructions."
        recording_checklist = "- [x] A screen recording of the full flow is committed under `docs/demos/`."
        runner_checklist = (
            "- [x] The runner completed the flow using only public repository instructions."
        )

    return f"""# Gate 2 Outside-Person Proof

Status: {status}

{proof_intro}

## Runner

- Name: {runner_name}
- Organization or relationship to project: {relationship}
- Prior Beater repo exposure: {prior_exposure}
- Date: {proof_date}
- Machine and OS: {machine_os}
- Docker version: {field_value(stopwatch_text, "Docker", stopwatch_rel)}
- Docker Compose version: {field_value(stopwatch_text, "Docker Compose", stopwatch_rel)}
- Browser: {browser}
- Network notes: {network_notes}
- Preflight status: {preflight_status}
- Outside-run attestation: {attestation}

## Repository

- Clone URL: {field_value(stopwatch_text, "Git origin", stopwatch_rel)}
- Commit SHA: {field_value(stopwatch_text, "Git SHA", stopwatch_rel)}
- Branch: {field_value(stopwatch_text, "Git branch", stopwatch_rel)}
- Worktree clean: {field_value(stopwatch_text, "Git worktree clean", stopwatch_rel)}
- OS/arch: {field_value(stopwatch_text, "OS/arch", stopwatch_rel)}
- Beater image reference: {field_value(stopwatch_text, "Beater image reference", stopwatch_rel)}
- Dashboard image reference: {field_value(stopwatch_text, "Dashboard image reference", stopwatch_rel)}
- Dashboard e2e image reference: {field_value(stopwatch_text, "Dashboard e2e image reference", stopwatch_rel)}
- OTEL Python image reference: {field_value(stopwatch_text, "OTEL Python image reference", stopwatch_rel)}
- Beater image digest: {field_value(stopwatch_text, "Beater image digest", stopwatch_rel)}
- Dashboard image digest: {field_value(stopwatch_text, "Dashboard image digest", stopwatch_rel)}
- Dashboard e2e image digest: {field_value(stopwatch_text, "Dashboard e2e image digest", stopwatch_rel)}
- OTEL Python image digest: {field_value(stopwatch_text, "OTEL Python image digest", stopwatch_rel)}
- API endpoint: {field_value(stopwatch_text, "API endpoint", stopwatch_rel)}
- Dashboard base: {field_value(stopwatch_text, "Dashboard base", stopwatch_rel)}
- Quickstart snippet: {field_value(stopwatch_text, "Quickstart snippet", stopwatch_rel)}
- Quickstart release ID: {field_value(stopwatch_text, "Quickstart release ID", stopwatch_rel)}
- Timing start source: {field_value(stopwatch_text, "Timing start source", stopwatch_rel)}
- Clone started at: {field_value(stopwatch_text, "Clone started at", stopwatch_rel)}
- Script started at: {field_value(stopwatch_text, "Script started at", stopwatch_rel)}
- Started at: {field_value(stopwatch_text, "Started", stopwatch_rel)}
- Ended at: {field_value(stopwatch_text, "Ended", stopwatch_rel)}
- Time-to-first-trace: {field_value(stopwatch_text, "Time-to-first-trace", stopwatch_rel)}
- Script-to-first-trace: {field_value(stopwatch_text, "Script-to-first-trace", stopwatch_rel)}
- Time-to-quickstart-click: {field_value(stopwatch_text, "Time-to-quickstart-click", stopwatch_rel)}
- Script-to-quickstart-click: {field_value(stopwatch_text, "Script-to-quickstart-click", stopwatch_rel)}
- Quickstart click source: {field_value(stopwatch_text, "Quickstart click source", stopwatch_rel)}
- Manual quickstart confirmation: {field_value(stopwatch_text, "Manual quickstart confirmation", stopwatch_rel)}
- Total proof duration: {field_value(stopwatch_text, "Total duration", stopwatch_rel)}
- Script duration: {field_value(stopwatch_text, "Script duration", stopwatch_rel)}
- Outside-run wrapper: {field_value(stopwatch_text, "Outside-run wrapper", stopwatch_rel)}

## Commands

```bash
bash -o pipefail -lc 'curl -fsSL https://raw.githubusercontent.com/jadenfix/beater/main/scripts/gate2-outside-local-preflight.sh | bash && t="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" {CANONICAL_COMMAND}'
```

{command_note}

## Required Evidence

- Stopwatch proof file: {stopwatch_rel}
- Screen recording: `{recording}`
- Screen recording notes: `{notes}`
- Screen recording SHA256: {recording_sha}
- Terminal output excerpt: {terminal_excerpt}
- Runner llm.call observation: {llm_observation}
- Runner waterfall observation: {waterfall_observation}
- `docker compose images` excerpt: {compose_images_excerpt(stopwatch_text, stopwatch_path)}
- Quickstart trace ID: {field_value(stopwatch_text, "Quickstart trace", stopwatch_rel)}
- Quickstart dashboard URL: `{quickstart_dashboard_url}`
- All-kind nested trace ID: {field_value(stopwatch_text, "All-kind nested trace", stopwatch_rel)}
- All-kind dashboard URL: `{all_kind_dashboard_url}`
- `docker compose` logs saved: {logs_saved}
- Failure notes, if any: {failure_notes}

## Pass Checklist

- [x] Fresh clone was used.
- [x] Docker was running before the stopwatch started.
- [x] curl was available before the stopwatch started.
- [x] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`, dashboard `127.0.0.1:3000`.
- [x] `BEATER_GATE2_REUSE` was not set.
- [x] `COMPOSE_FILE`, `COMPOSE_PROJECT_NAME`, and `COMPOSE_PROFILES` were not set.
- [x] The script reported `Clean start: yes`.
- [x] Time-to-first-trace was 300 seconds or less.
- [x] Time-to-first-trace includes clone time.
- [x] Manual quickstart click confirmation was recorded before 300 seconds.
- [x] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [x] Clicking the `llm.call` span showed prompt, completion, model, token breakdown, cost, and latency.
- [x] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in the waterfall.
- [x] The browser proof passed for both the quickstart trace and all-kind waterfall.
- [x] The stopwatch script generated and reported the browser recording.
{recording_checklist}
{runner_checklist}

## Runner Notes

{runner_notes}
"""


def parse_args():
    parser = argparse.ArgumentParser(
        description="Generate completed Gate 2 outside-person proof from a stopwatch proof."
    )
    parser.add_argument(
        "--stopwatch-proof",
        default="docs/demos/gate2-compose-stopwatch.md",
        help="Path to the stopwatch proof generated by scripts/gate2-compose-stopwatch.sh.",
    )
    parser.add_argument(
        "--output",
        default="docs/demos/gate2-outside-person-proof.md",
        help="Proof file to write.",
    )
    parser.add_argument("--runner-name", default="")
    parser.add_argument("--relationship", default="")
    parser.add_argument("--prior-exposure", default="")
    parser.add_argument("--machine-os", default="")
    parser.add_argument("--browser", default="")
    parser.add_argument("--preflight-status", default="")
    parser.add_argument(
        "--attest-outside-run",
        action="store_true",
        help="Required attestation that the runner is outside the project and unaided.",
    )
    parser.add_argument(
        "--diagnostic-report",
        action="store_true",
        help=(
            "Generate a maintainer diagnostic report from full-run artifacts. "
            "This cannot validate as outside-person closure evidence."
        ),
    )
    parser.add_argument(
        "--print-command",
        action="store_true",
        help=(
            "Print a ready-to-edit proof-generation command using values from "
            "the stopwatch proof. Does not write or validate a proof."
        ),
    )
    parser.add_argument("--network-notes", default="")
    parser.add_argument("--llm-observation", default="")
    parser.add_argument("--waterfall-observation", default="")
    parser.add_argument("--terminal-output-excerpt", default="")
    parser.add_argument("--compose-logs-saved", default="")
    parser.add_argument("--failure-notes", default="")
    parser.add_argument("--runner-notes", default="")
    parser.add_argument(
        "--date",
        default=None,
        help="Proof date. Defaults to the UTC date from stopwatch Clone started at.",
    )
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--no-validate", action="store_true")
    args = parser.parse_args()
    if args.print_command and (args.attest_outside_run or args.diagnostic_report):
        parser.error("--print-command cannot be combined with proof-writing attestation flags")
    if args.attest_outside_run and args.diagnostic_report:
        parser.error("--diagnostic-report cannot be combined with --attest-outside-run")
    if not args.print_command and not args.attest_outside_run and not args.diagnostic_report:
        parser.error("--attest-outside-run is required for completed Gate 2 proof generation")
    return args


def main():
    args = parse_args()
    stopwatch_path = Path(args.stopwatch_proof)
    output_path = Path(args.output)
    if not stopwatch_path.is_absolute():
        stopwatch_path = repo_root() / stopwatch_path
    if not output_path.is_absolute():
        output_path = repo_root() / output_path
    if not stopwatch_path.exists():
        raise SystemExit(f"missing stopwatch proof: {stopwatch_path}")

    stopwatch_text = stopwatch_path.read_text()
    if args.print_command:
        print_prefilled_command(args, stopwatch_path, output_path, stopwatch_text)
        return

    require_pending_or_force(output_path, args.force)
    proof = build_proof(args, stopwatch_path, stopwatch_text)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    if args.no_validate:
        output_path.write_text(proof)
    else:
        with tempfile.NamedTemporaryFile(
            "w",
            encoding="utf-8",
            dir=output_path.parent,
            prefix=f".{output_path.name}.",
            suffix=".tmp",
            delete=False,
        ) as temp_proof:
            temp_path = Path(temp_proof.name)
            temp_proof.write(proof)
        env = dict(os.environ)
        try:
            env["BEATER_GATE2_OUTSIDE_PROOF"] = str(temp_path)
            env["BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS"] = "1"
            validate = ["bash", "scripts/validate-gate2-outside-proof.sh"]
            if args.diagnostic_report:
                validate.append("--diagnostic")
            subprocess.run(
                validate,
                cwd=repo_root(),
                env=env,
                check=True,
            )
            temp_path.replace(output_path)
        except subprocess.CalledProcessError as err:
            temp_path.unlink(missing_ok=True)
            raise SystemExit(err.returncode) from None
        except BaseException:
            temp_path.unlink(missing_ok=True)
            raise

    label = "Gate 2 diagnostic proof" if args.diagnostic_report else "Gate 2 outside-person proof"
    print(f"Wrote {label}: {relative_or_absolute(output_path)}")


if __name__ == "__main__":
    sys.exit(main())
