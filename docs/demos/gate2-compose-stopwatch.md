# Gate 2 Compose Stopwatch Proof

- Started: 2026-06-20T09:58:54Z
- Ended: 2026-06-20T09:59:20Z
- Duration: 26s
- Limit: 300s
- Startup mode: prebuilt-image
- Compose project: beater-stopwatch-browser-proof-doc
- Snippet: `examples/python/five_line_otel.py`
- OTLP endpoint: `http://127.0.0.1:14325`
- Trace: `12924879d3bfe1c498a2ea150e895d70`
- Dashboard: http://127.0.0.1:13008/?tenant=demo&project=demo&environment=local&trace=12924879d3bfe1c498a2ea150e895d70
- Browser proof: passed

This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

```bash
BEATER_GATE2_WRITE_PROOF=1 scripts/gate2-compose-stopwatch.sh
```
