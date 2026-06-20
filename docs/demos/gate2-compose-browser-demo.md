# Gate 2 Compose Browser Demo

Recorded from the Docker Compose stopwatch path using the literal five-line
stock OpenTelemetry quickstart and the all-kind stock OpenTelemetry agent trace.

- Artifact: `gate2-compose-browser-demo.webm`
- SHA256: `3dac802bc8f2db03406d0d76e4e1618ed5b516a2cf3d286589e1a588cf6e6534`
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart trace: `c8fd1651c8ea514803dc1b86bd6c5411`
- All-kind trace: `42bfb21a2a4dc58046869a20f079b9ec`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, tokens, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.

This automated maintainer run used the default dashboard URL `http://127.0.0.1:3000`; no alternate host ports were needed.

Regenerate with:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```

The mandate still requires the outside-person run recorded in
`docs/demos/gate2-outside-person-proof.md` before Gate 2 can close.
