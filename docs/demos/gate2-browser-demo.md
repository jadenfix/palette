# Gate 2 Browser Demo

Recorded from the stock OpenTelemetry Python trace produced by `examples/python/otel_smoke.py`.

- Artifact: `gate2-browser-demo.webm`
- SHA256: `359bd521dacc48a6c484b02a2a6fff55bbd9d8ef46c29327919c756d17ae95e1`
- Dashboard: `http://127.0.0.1:13003/?tenant=demo&project=demo&environment=local&trace=a6fde3f82aa86f79f1af5317343d35b4`
- Shows: trace table, color/icon-coded all-kind agent waterfall, run -> turn -> step -> tool -> MCP nesting, `llm.call` prompt/completion/model/token breakdown/cost/latency, and tool-call I/O.

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
