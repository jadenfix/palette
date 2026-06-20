# Gate 2 Compose Stopwatch Proof

Status: automated local proof using alternate host ports because port 3000 was
already occupied on the maintainer machine. This does not close Gate 2; the
outside-person proof must use the default dashboard URL `http://127.0.0.1:3000`.

- Started: 2026-06-20T11:38:14Z
- Ended: 2026-06-20T11:38:52Z
- Time-to-first-trace: 17s
- Time-to-quickstart-click: 30s
- Total duration: 38s
- Limit: 300s
- Git SHA: `de2271d781e452e2b8a82624bab3cf1883d0e4ef`
- OS/arch: `Darwin arm64`
- Docker: `Docker version 29.2.0, build 0b9d198`
- Docker Compose: `Docker Compose version v5.0.2`
- Startup mode: prebuilt-image
- Clean start: yes
- Reuse override: `BEATER_GATE2_REUSE=0`
- Prebuilt pull policy: `always`
- Compose project: beater-stopwatch
- Quickstart snippet: `examples/python/five_line_otel.py`
- OTLP endpoint: `http://127.0.0.1:14317`
- Quickstart trace: `2489bdad6d709b2d8ac9fbb80accffbd`
- Quickstart dashboard: http://127.0.0.1:13080/?tenant=demo&project=demo&environment=local&trace=2489bdad6d709b2d8ac9fbb80accffbd
- Quickstart browser proof: passed
- All-kind nested trace: `4bf122b6b07c0faa9efe9a887efd63e8`
- All-kind dashboard: http://127.0.0.1:13080/?tenant=demo&project=demo&environment=local&trace=4bf122b6b07c0faa9efe9a887efd63e8
- All-kind waterfall browser proof: passed
- Browser recording: passed
- Browser recording artifact: `docs/demos/gate2-compose-browser-demo.webm`
- Browser recording notes: `docs/demos/gate2-compose-browser-demo.md`
- Browser recording SHA256: `f08729933ca69fa7ca1753422f1ce3ba322ccd7d58d689a3b4f267b29c04052a`

## Compose Images

```text
CONTAINER                      REPOSITORY                          TAG                 PLATFORM            IMAGE ID            SIZE                CREATED
beater-stopwatch-beaterd-1     ghcr.io/jadenfix/beater/beaterd     main                linux/arm64         557bb44a2dcb        88.4MB              17 minutes ago
beater-stopwatch-dashboard-1   ghcr.io/jadenfix/beater/dashboard   main                linux/arm64         59a22f24ad93        99.2MB              About an hour ago
beater-stopwatch-minio-1       minio/minio                         latest              linux/arm64         14cea493d9a3        57.5MB              9 months ago
beater-stopwatch-nats-1        nats                                2.11-alpine         linux/arm64/v8      e4bf19f15fd3        10.5MB              7 weeks ago
beater-stopwatch-postgres-1    postgres                            17-alpine           linux/arm64/v8      dc17045ccfd3        115MB               3 days ago
```

This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```
