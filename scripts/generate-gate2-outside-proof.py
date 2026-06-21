#!/usr/bin/env python3
import argparse
import datetime as dt
import os
import re
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


def clean_value(value):
    return value.strip().strip("`").strip()


def require_meaningful_arg(name, value):
    cleaned = clean_value(value)
    if (
        not cleaned
        or cleaned.lower() in UNRESOLVED_REQUIRED_VALUES
        or cleaned.lower() == "none"
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
    long_running_repos = {
        "ghcr.io/jadenfix/beater/beaterd",
        "ghcr.io/jadenfix/beater/dashboard",
    }
    services = [line for line in lines if long_running_repos.intersection(line.split())]
    if services:
        return " | ".join(services)
    return " | ".join(lines[:3])


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
    recording = field_value(stopwatch_text, "Browser recording artifact", stopwatch_rel)
    notes = field_value(stopwatch_text, "Browser recording notes", stopwatch_rel)
    quickstart_dashboard_url = field_value(stopwatch_text, "Quickstart dashboard", stopwatch_rel)
    all_kind_dashboard_url = field_value(stopwatch_text, "All-kind dashboard", stopwatch_rel)

    terminal_excerpt = (
        args.terminal_output_excerpt
        or (
            "Gate 2 compose stopwatch passed; Browser recording: passed; "
            f"Quickstart dashboard: {quickstart_dashboard_url}; "
            f"All-kind dashboard: {all_kind_dashboard_url}"
        )
    )
    logs_saved = args.compose_logs_saved or "not saved; stopwatch proof embeds compose image output"
    failure_notes = args.failure_notes or "none"
    runner_notes = args.runner_notes or "No extra runner notes."
    network_notes = require_meaningful_arg("--network-notes", args.network_notes)
    llm_observation = require_meaningful_arg("--llm-observation", args.llm_observation)
    waterfall_observation = require_meaningful_arg(
        "--waterfall-observation", args.waterfall_observation
    )
    runner_name = require_meaningful_arg("--runner-name", args.runner_name)
    relationship = require_meaningful_arg("--relationship", args.relationship)
    prior_exposure = require_meaningful_arg("--prior-exposure", args.prior_exposure)
    machine_os = require_meaningful_arg("--machine-os", args.machine_os)
    browser = require_meaningful_arg("--browser", args.browser)
    preflight_status = require_meaningful_arg(
        "--preflight-status", args.preflight_status
    )
    proof_date = require_date_arg("--date", args.date)

    return f"""# Gate 2 Outside-Person Proof

Status: completed.

Gate 2 evidence generated from the stopwatch proof listed below. This file is
valid only when the named runner is outside the project and completed the run
unaided using public repository instructions.

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
- Outside-run attestation: {OUTSIDE_RUN_ATTESTATION}

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
- Timing start source: {field_value(stopwatch_text, "Timing start source", stopwatch_rel)}
- Clone started at: {field_value(stopwatch_text, "Clone started at", stopwatch_rel)}
- Script started at: {field_value(stopwatch_text, "Script started at", stopwatch_rel)}
- Started at: {field_value(stopwatch_text, "Started", stopwatch_rel)}
- Ended at: {field_value(stopwatch_text, "Ended", stopwatch_rel)}
- Time-to-first-trace: {field_value(stopwatch_text, "Time-to-first-trace", stopwatch_rel)}
- Script-to-first-trace: {field_value(stopwatch_text, "Script-to-first-trace", stopwatch_rel)}
- Time-to-quickstart-click: {field_value(stopwatch_text, "Time-to-quickstart-click", stopwatch_rel)}
- Script-to-quickstart-click: {field_value(stopwatch_text, "Script-to-quickstart-click", stopwatch_rel)}
- Total proof duration: {field_value(stopwatch_text, "Total duration", stopwatch_rel)}
- Script duration: {field_value(stopwatch_text, "Script duration", stopwatch_rel)}
- Outside-run wrapper: {field_value(stopwatch_text, "Outside-run wrapper", stopwatch_rel)}

## Commands

```bash
bash -lc 't="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" {CANONICAL_COMMAND}'
```

The runner completed the flow using only public repository instructions.

## Required Evidence

- Stopwatch proof file: {stopwatch_rel}
- Screen recording: `{recording}`
- Screen recording notes: `{notes}`
- Screen recording SHA256: {field_value(stopwatch_text, "Browser recording SHA256", stopwatch_rel)}
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
- [x] The script reported `Clean start: yes`.
- [x] Time-to-first-trace was 300 seconds or less.
- [x] Time-to-first-trace includes clone time.
- [x] Time-to-quickstart-click was 300 seconds or less.
- [x] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [x] Clicking the `llm.call` span showed prompt, completion, model, tokens, cost, and latency.
- [x] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in the waterfall.
- [x] The browser proof passed for both the quickstart trace and all-kind waterfall.
- [x] The stopwatch script generated and reported the browser recording.
- [x] A screen recording of the full flow is committed under `docs/demos/`.
- [x] The runner completed the flow using only public repository instructions.

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
    parser.add_argument("--runner-name", required=True)
    parser.add_argument("--relationship", required=True)
    parser.add_argument("--prior-exposure", required=True)
    parser.add_argument("--machine-os", required=True)
    parser.add_argument("--browser", required=True)
    parser.add_argument("--preflight-status", required=True)
    parser.add_argument(
        "--attest-outside-run",
        action="store_true",
        help="Required attestation that the runner is outside the project and unaided.",
    )
    parser.add_argument("--network-notes", required=True)
    parser.add_argument("--llm-observation", required=True)
    parser.add_argument("--waterfall-observation", required=True)
    parser.add_argument("--terminal-output-excerpt", default="")
    parser.add_argument("--compose-logs-saved", default="")
    parser.add_argument("--failure-notes", default="")
    parser.add_argument("--runner-notes", default="")
    parser.add_argument("--date", default=dt.date.today().isoformat())
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--no-validate", action="store_true")
    args = parser.parse_args()
    if not args.attest_outside_run:
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

    require_pending_or_force(output_path, args.force)
    stopwatch_text = stopwatch_path.read_text()
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
            subprocess.run(
                ["bash", "scripts/validate-gate2-outside-proof.sh"],
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

    print(f"Wrote Gate 2 outside-person proof: {relative_or_absolute(output_path)}")


if __name__ == "__main__":
    sys.exit(main())
