# Gate 2 Compose Browser Demo

Status: pending regeneration. The previous checked-in recording was a
pre-hardening 3.68s capture and has been removed from the canonical
`docs/demos/gate2-compose-browser-demo.webm` evidence path because it does not
satisfy the current 8-second reviewability floor enforced by
`scripts/gate2-compose-stopwatch.sh` and
`scripts/validate-gate2-outside-proof.sh`. Regenerate the recording from the
default `http://127.0.0.1:3000` compose stopwatch path before using this file as
Gate 2 evidence.

Recorded from the Docker Compose stopwatch path using the literal five-line
stock OpenTelemetry quickstart and the all-kind stock OpenTelemetry agent trace.

- Artifact: `gate2-compose-browser-demo.webm` (not currently committed)
- SHA256: pending regenerated recording
- Recording mode: compose
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart trace: `c8fd1651c8ea514803dc1b86bd6c5411`
- All-kind trace: `42bfb21a2a4dc58046869a20f079b9ec`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, token breakdown, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.

This run used the default dashboard URL `http://127.0.0.1:3000`; no alternate host ports were needed.

Regenerate with:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```

The mandate still requires the outside-person run recorded in
`docs/demos/gate2-outside-person-proof.md` before Gate 2 can close.
