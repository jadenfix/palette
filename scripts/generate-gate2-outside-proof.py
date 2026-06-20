#!/usr/bin/env python3
import argparse
import datetime as dt
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


CANONICAL_COMMAND = (
    "BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 "
    "BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh"
)
OUTSIDE_RUN_ATTESTATION = (
    "I attest that I am not a Beater project maintainer, I received no "
    "step-by-step help beyond public repository instructions, I used a fresh "
    "clone, and I completed the Gate 2 flow unaided."
)


def clean_value(value):
    return value.strip().strip("`").strip()


def field_value(source_text, name, source_name):
    match = re.search(r"^- " + re.escape(name) + r":[ \t]*(.*)$", source_text, re.MULTILINE)
    if not match:
        raise SystemExit(f"missing field in {source_name}: {name}")
    value = clean_value(match.group(1))
    if not value or value == "not requested" or value == "unknown":
        raise SystemExit(f"unusable field in {source_name}: {name}={value!r}")
    return value


def repo_root():
    return Path(__file__).resolve().parent.parent


def relative_or_absolute(path):
    try:
        return str(path.resolve().relative_to(repo_root()))
    except ValueError:
        return str(path)


def git_branch():
    try:
        branch = subprocess.check_output(
            ["git", "branch", "--show-current"],
            cwd=repo_root(),
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        branch = ""
    return branch or "main"


def compose_images_excerpt(stopwatch_text, stopwatch_path):
    match = re.search(r"## Compose Images\s+```text\n(.*?)\n```", stopwatch_text, re.DOTALL)
    if not match:
        return f"see {relative_or_absolute(stopwatch_path)}"
    lines = [line.strip() for line in match.group(1).splitlines() if line.strip()]
    if not lines:
        return f"see {relative_or_absolute(stopwatch_path)}"
    services = [line for line in lines if "beaterd" in line or "dashboard" in line]
    if services:
        return " | ".join(services)
    return " | ".join(lines[:3])


def require_pending_or_force(output_path, force):
    if not output_path.exists() or force:
        return
    text = output_path.read_text()
    if "Status: not yet completed." in text:
        return
    raise SystemExit(f"{output_path} already exists and is not the pending template; pass --force")


def build_proof(args, stopwatch_path, stopwatch_text):
    stopwatch_rel = relative_or_absolute(stopwatch_path)
    recording = field_value(stopwatch_text, "Browser recording artifact", stopwatch_rel)
    notes = field_value(stopwatch_text, "Browser recording notes", stopwatch_rel)

    terminal_excerpt = (
        args.terminal_output_excerpt
        or "Gate 2 compose stopwatch passed; browser recording passed; see stopwatch proof."
    )
    logs_saved = args.compose_logs_saved or "not saved; stopwatch proof embeds compose image output"
    failure_notes = args.failure_notes or "none"
    runner_notes = args.runner_notes or "No extra runner notes."
    network_notes = args.network_notes or "not reported"

    return f"""# Gate 2 Outside-Person Proof

Status: completed.

Gate 2 evidence generated from the stopwatch proof listed below. This file is
valid only when the named runner is outside the project and completed the run
unaided using public repository instructions.

## Runner

- Name: {args.runner_name}
- Organization or relationship to project: {args.relationship}
- Prior Beater repo exposure: {args.prior_exposure}
- Date: {args.date}
- Machine and OS: {args.machine_os}
- Docker version: {field_value(stopwatch_text, "Docker", stopwatch_rel)}
- Docker Compose version: {field_value(stopwatch_text, "Docker Compose", stopwatch_rel)}
- Browser: {args.browser}
- Network notes: {network_notes}
- Preflight status: {args.preflight_status}
- Outside-run attestation: {OUTSIDE_RUN_ATTESTATION}

## Repository

- Clone URL: `https://github.com/jadenfix/beater.git`
- Commit SHA: {field_value(stopwatch_text, "Git SHA", stopwatch_rel)}
- Branch: {args.branch}
- OS/arch: {field_value(stopwatch_text, "OS/arch", stopwatch_rel)}
- Beater image digest: {field_value(stopwatch_text, "Beater image digest", stopwatch_rel)}
- Dashboard image digest: {field_value(stopwatch_text, "Dashboard image digest", stopwatch_rel)}
- API endpoint: {field_value(stopwatch_text, "API endpoint", stopwatch_rel)}
- Dashboard base: {field_value(stopwatch_text, "Dashboard base", stopwatch_rel)}
- Started at: {field_value(stopwatch_text, "Started", stopwatch_rel)}
- Ended at: {field_value(stopwatch_text, "Ended", stopwatch_rel)}
- Time-to-first-trace: {field_value(stopwatch_text, "Time-to-first-trace", stopwatch_rel)}
- Time-to-quickstart-click: {field_value(stopwatch_text, "Time-to-quickstart-click", stopwatch_rel)}
- Total proof duration: {field_value(stopwatch_text, "Total duration", stopwatch_rel)}

## Commands

```bash
git clone https://github.com/jadenfix/beater.git
cd beater
{CANONICAL_COMMAND}
```

The runner completed the flow using only public repository instructions.

## Required Evidence

- Stopwatch proof file: {stopwatch_rel}
- Screen recording: `{recording}`
- Screen recording notes: `{notes}`
- Screen recording SHA256: {field_value(stopwatch_text, "Browser recording SHA256", stopwatch_rel)}
- Terminal output excerpt: {terminal_excerpt}
- `docker compose images` excerpt: {compose_images_excerpt(stopwatch_text, stopwatch_path)}
- Quickstart trace ID: {field_value(stopwatch_text, "Quickstart trace", stopwatch_rel)}
- Quickstart dashboard URL: `{field_value(stopwatch_text, "Quickstart dashboard", stopwatch_rel)}`
- All-kind nested trace ID: {field_value(stopwatch_text, "All-kind nested trace", stopwatch_rel)}
- All-kind dashboard URL: `{field_value(stopwatch_text, "All-kind dashboard", stopwatch_rel)}`
- `docker compose` logs saved: {logs_saved}
- Failure notes, if any: {failure_notes}

## Pass Checklist

- [x] Fresh clone was used.
- [x] Docker was running before the stopwatch started.
- [x] Python, curl, and npm were available before the stopwatch started.
- [x] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`, dashboard `127.0.0.1:3000`.
- [x] `BEATER_GATE2_REUSE` was not set.
- [x] The script reported `Clean start: yes`.
- [x] Time-to-first-trace was 300 seconds or less.
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
    parser.add_argument("--network-notes", default="")
    parser.add_argument("--terminal-output-excerpt", default="")
    parser.add_argument("--compose-logs-saved", default="")
    parser.add_argument("--failure-notes", default="")
    parser.add_argument("--runner-notes", default="")
    parser.add_argument("--date", default=dt.date.today().isoformat())
    parser.add_argument("--branch", default=git_branch())
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
        except BaseException:
            temp_path.unlink(missing_ok=True)
            raise

    print(f"Wrote Gate 2 outside-person proof: {relative_or_absolute(output_path)}")


if __name__ == "__main__":
    sys.exit(main())
