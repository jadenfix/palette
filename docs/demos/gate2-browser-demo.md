# Gate 2 Browser Demo

Recorded from the stock OpenTelemetry Python trace produced by `examples/python/otel_smoke.py`.

- Artifact: `gate2-browser-demo.webm`
- Dashboard: `http://127.0.0.1:13000/?tenant=demo&project=demo&environment=local&trace=d7c3eca856125443d6f85384b1441ca0`
- Shows: trace table, all-kind agent waterfall, `llm.call` prompt/completion/model/tokens/cost, and tool-call I/O.

Regenerate with:

```bash
BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh
```

For the Docker Compose stopwatch proof that uses the literal five-line snippet,
run the prebuilt-image path:

```bash
BEATER_GATE2_WRITE_PROOF=1 KEEP_BEATER_COMPOSE=0 scripts/gate2-compose-stopwatch.sh
```

For a local source build measurement, add `BEATER_GATE2_LOCAL_BUILD=1`.
