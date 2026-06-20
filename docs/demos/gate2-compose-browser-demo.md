# Gate 2 Compose Browser Demo

Recorded from the Docker Compose stopwatch path using the literal five-line
stock OpenTelemetry quickstart and the all-kind stock OpenTelemetry agent trace.
This automated maintainer run used alternate host ports because port 3000 was
already occupied; the outside-person proof must still use
`http://127.0.0.1:3000`.

- Artifact: `gate2-compose-browser-demo.webm`
- SHA256: `f08729933ca69fa7ca1753422f1ce3ba322ccd7d58d689a3b4f267b29c04052a`
- Dashboard base: `http://127.0.0.1:13080`
- Quickstart trace: `2489bdad6d709b2d8ac9fbb80accffbd`
- All-kind trace: `4bf122b6b07c0faa9efe9a887efd63e8`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, tokens, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.

Regenerate with:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```

The mandate still requires the outside-person run recorded in
`docs/demos/gate2-outside-person-proof.md` before Gate 2 can close.
