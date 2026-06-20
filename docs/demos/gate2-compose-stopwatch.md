# Gate 2 Compose Stopwatch Proof

- Timing start source: script
- Clone started at: not provided
- Script started at: 2026-06-20T22:23:16Z
- Started: 2026-06-20T22:23:16Z
- Ended: 2026-06-20T22:24:25Z
- Time-to-first-trace: 21s
- Script-to-first-trace: 21s
- Time-to-quickstart-click: 29s
- Script-to-quickstart-click: 29s
- Total duration: 69s
- Script duration: 69s
- Limit: 300s
- Git SHA: `9abba1815e0113baf46660fd7384664d43bebd62`
- Git branch: `main`
- Git origin: `https://github.com/jadenfix/beater.git`
- Git worktree clean: yes
- OS/arch: `Darwin arm64`
- Docker: `Docker version 29.2.0, build 0b9d198`
- Docker Compose: `Docker Compose version v5.0.2`
- Startup mode: prebuilt-image
- Clean start: yes
- Reuse override: `BEATER_GATE2_REUSE=0`
- Outside-run wrapper: no
- Prebuilt pull policy: `always`
- Compose project: beater-stopwatch
- Beater image reference: `ghcr.io/jadenfix/beater/beaterd:9abba1815e0113baf46660fd7384664d43bebd62`
- Dashboard image reference: `ghcr.io/jadenfix/beater/dashboard:9abba1815e0113baf46660fd7384664d43bebd62`
- Dashboard e2e image reference: `ghcr.io/jadenfix/beater/dashboard-e2e:9abba1815e0113baf46660fd7384664d43bebd62`
- OTEL Python image reference: `ghcr.io/jadenfix/beater/otel-python:9abba1815e0113baf46660fd7384664d43bebd62`
- Beater image digest: `ghcr.io/jadenfix/beater/beaterd@sha256:9b8ed21a35c9e1b41bbd85ea628234c6ca52a537fcc1ae5ce659082a717967a2`
- Dashboard image digest: `ghcr.io/jadenfix/beater/dashboard@sha256:d78d91dc932aa2157dfc0b07295eb7f9eb16a217b24519832630123b6f1c6e51`
- Dashboard e2e image digest: `ghcr.io/jadenfix/beater/dashboard-e2e@sha256:f61e4d94cd625223d384ec80b819e6f52a7cba1653c2fe7c7ace0ee150ceaa51`
- OTEL Python image digest: `ghcr.io/jadenfix/beater/otel-python@sha256:773fd0d3f8554a4f87ebc71be9f2debbff975aca5863d3f40eda191a145b957f`
- Quickstart snippet: `examples/python/five_line_otel.py`
- API endpoint: `http://127.0.0.1:8080`
- OTLP endpoint: `http://127.0.0.1:4317`
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart trace: `c8fd1651c8ea514803dc1b86bd6c5411`
- Quickstart dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace=c8fd1651c8ea514803dc1b86bd6c5411
- Quickstart browser proof: passed
- All-kind nested trace: `42bfb21a2a4dc58046869a20f079b9ec`
- All-kind dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace=42bfb21a2a4dc58046869a20f079b9ec
- All-kind waterfall browser proof: passed
- Browser recording: passed
- Browser recording artifact: `docs/demos/gate2-compose-browser-demo.webm`
- Browser recording notes: `docs/demos/gate2-compose-browser-demo.md`
- Browser recording SHA256: `3dac802bc8f2db03406d0d76e4e1618ed5b516a2cf3d286589e1a588cf6e6534`

## Compose Images

```text
CONTAINER                      REPOSITORY                          TAG                                        PLATFORM            IMAGE ID            SIZE                CREATED
beater-stopwatch-beaterd-1     ghcr.io/jadenfix/beater/beaterd     9abba1815e0113baf46660fd7384664d43bebd62   linux/arm64         9b8ed21a35c9        88.4MB              11 hours ago
beater-stopwatch-dashboard-1   ghcr.io/jadenfix/beater/dashboard   9abba1815e0113baf46660fd7384664d43bebd62   linux/arm64         d78d91dc932a        99.2MB              8 minutes ago
beater-stopwatch-minio-1       minio/minio                         latest                                     linux/arm64         14cea493d9a3        57.5MB              9 months ago
beater-stopwatch-nats-1        nats                                2.11-alpine                                linux/arm64/v8      e4bf19f15fd3        10.5MB              7 weeks ago
beater-stopwatch-postgres-1    postgres                            17-alpine                                  linux/arm64/v8      dc17045ccfd3        115MB               3 days ago
```

This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```
