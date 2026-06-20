# Gate 2 Outside-Person Proof

Status: not yet completed.

Gate 2 is not passed until this file is replaced with evidence from a person
outside the project who runs the flow unaided from a fresh clone.

## Runner

- Name:
- Organization or relationship to project:
- Prior Beater repo exposure: none / describe:
- Date:
- Machine and OS:
- Docker version:
- Docker Compose version:
- Browser:
- Network notes:
- Preflight status:
- Outside-run attestation:

## Repository

- Clone URL: `https://github.com/jadenfix/beater.git`
- Commit SHA:
- Branch:
- OS/arch:
- Beater image digest:
- Dashboard image digest:
- API endpoint:
- Dashboard base:
- Started at:
- Ended at:
- Time-to-first-trace:
- Time-to-quickstart-click:
- Total proof duration:

## Commands

```bash
git clone https://github.com/jadenfix/beater.git
cd beater
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```

No project maintainer may provide step-by-step help beyond public repo docs
during the timed run.

The script fails before Compose startup if Docker is unavailable, if Python,
curl, or npm are missing, or if API `8080`, OTLP `4317`, or dashboard `3000`
are still in use after it removes any previous Beater stopwatch project. For
this outside-person proof, free those default ports instead of using alternate
port environment variables.

After the script prints the dashboard URLs, the runner must open
`http://127.0.0.1:3000` in a normal browser, click the quickstart trace, click
the `llm.call` span, and capture the evidence below. Cleanup can happen after
the recording.

After the stopwatch command finishes, prefer generating completed evidence from
the stopwatch proof instead of manually copying fields:

```bash
scripts/generate-gate2-outside-proof.py \
  --runner-name "..." \
  --relationship "..." \
  --prior-exposure "none" \
  --machine-os "..." \
  --browser "..." \
  --preflight-status "passed" \
  --attest-outside-run
```

After replacing this template with completed evidence, run:

```bash
scripts/validate-gate2-outside-proof.sh
```

The validator reads the listed stopwatch proof file and screen-recording notes,
then cross-checks default API/OTLP/dashboard endpoints, clean-start status,
browser-proof status, trace IDs, dashboard URLs, prebuilt GHCR image digests,
and the tested `main` commit SHA. If the proof commit is newer than the tested
SHA, every later change must be under `docs/demos/`. It verifies screen-recording SHA256 against the committed artifact. Stopwatch, recording, and notes paths must
be repo-relative paths under `docs/demos/`.

## Required Evidence

- Stopwatch proof file:
- Screen recording: `docs/demos/gate2-compose-browser-demo.webm`
- Screen recording notes: `docs/demos/gate2-compose-browser-demo.md`
- Screen recording SHA256:
- Terminal output excerpt:
- `docker compose images` excerpt:
- Quickstart trace ID:
- Quickstart dashboard URL: `http://127.0.0.1:3000/...`
- All-kind nested trace ID:
- All-kind dashboard URL: `http://127.0.0.1:3000/...`
- `docker compose` logs saved:
- Failure notes, if any:

## Pass Checklist

- [ ] Fresh clone was used.
- [ ] Docker was running before the stopwatch started.
- [ ] Python, curl, and npm were available before the stopwatch started.
- [ ] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`,
      dashboard `127.0.0.1:3000`.
- [ ] `BEATER_GATE2_REUSE` was not set.
- [ ] The script reported `Clean start: yes`.
- [ ] Time-to-first-trace was 300 seconds or less.
- [ ] Time-to-quickstart-click was 300 seconds or less.
- [ ] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [ ] Clicking the `llm.call` span showed prompt, completion, model, tokens,
      cost, and latency.
- [ ] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in
      the waterfall.
- [ ] The browser proof passed for both the quickstart trace and all-kind
      waterfall.
- [ ] The stopwatch script generated and reported the browser recording.
- [ ] A screen recording of the full flow is committed under `docs/demos/`.
- [ ] The runner completed the flow using only public repository instructions.

## Runner Notes

Add any confusing step, missing prerequisite, slow pull, failing command, or UI
ambiguity here. These notes are blocking feedback until fixed or explicitly
triaged.
