# Gate 2 Browser Demo

Recorded from the stock OpenTelemetry Python trace produced by `examples/python/otel_smoke.py`.

- Artifact: `gate2-browser-demo.webm`
- SHA256: `77ff5820e44f77883c18c111a29d5d376be6141dfe18bb6035fc674fc8b11711`
- Recording mode: all-kind
- Dashboard: `http://127.0.0.1:3001/?tenant=demo&project=demo&environment=local&trace=7f57b4533c178ccc54d3b84a8a5fc76a`
- Shows: trace table, color/icon-coded all-kind agent waterfall, run -> turn -> step -> tool -> MCP nesting, `llm.call` prompt/completion/model/token breakdown/cost/latency/confirmation code, and tool-call I/O.

Regenerate with:

```bash
BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh
```

For the Docker Compose stopwatch proof that uses the literal five-line snippet,
run the prebuilt-image path:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```

For a local source build measurement, add `BEATER_GATE2_LOCAL_BUILD=1`.
