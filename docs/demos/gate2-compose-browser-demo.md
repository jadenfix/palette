# Gate 2 Compose Browser Demo

Recorded from the Docker Compose stopwatch path using the literal five-line
stock OpenTelemetry quickstart and the all-kind stock OpenTelemetry agent trace.

- Artifact: `gate2-compose-browser-demo.webm`
- SHA256: `d4b3864cd3a5a1b2c2c70b329a949c1215b8a07e85203650841a07be95177248`
- Recording mode: compose
- Dashboard base: `http://127.0.0.1:3001`
- Quickstart trace: `3725033cbe940f1674a9d19ab72d3904`
- All-kind trace: `4849bef9a116057c6f481016b3d604f7`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, token breakdown, cost, latency, and confirmation code -> inspect run -> turn -> step -> tool -> MCP waterfall.

This run used alternate host ports; the outside-person proof must still use the default dashboard URL `http://127.0.0.1:3000`.

The mandate still requires the outside-person run recorded in
`docs/demos/gate2-outside-person-proof.md` before Gate 2 can close.

Regenerate with:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```
