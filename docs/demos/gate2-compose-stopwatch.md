# Gate 2 Compose Stopwatch Proof

- Timing start source: script
- Clone started at: not provided
- Script started at: 2026-06-22T16:24:34Z
- Started: 2026-06-22T16:24:34Z
- Ended: 2026-06-22T16:26:24Z
- Time-to-first-trace: 42s
- Script-to-first-trace: 42s
- Time-to-quickstart-click: 56s
- Script-to-quickstart-click: 56s
- Quickstart click source: automated-browser-proof
- Manual quickstart confirmation: not requested
- Manual confirmation source: not requested
- Manual confirmation code: not requested
- Manual confirmation salt: `not requested`
- Total duration: 110s
- Script duration: 110s
- Limit: 300s
- Git SHA: `3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab`
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
- Compose logs artifact: `not requested`
- Beater image reference: `ghcr.io/jadenfix/beater/beaterd:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab`
- Dashboard image reference: `ghcr.io/jadenfix/beater/dashboard:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab`
- Dashboard e2e image reference: `ghcr.io/jadenfix/beater/dashboard-e2e:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab`
- OTEL Python image reference: `ghcr.io/jadenfix/beater/otel-python:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab`
- Beater image digest: `ghcr.io/jadenfix/beater/beaterd@sha256:2f3f45f686c4a5e159b4683ae0b6a7ca72a88fee2be2feacbbd770e95d671ee9`
- Dashboard image digest: `ghcr.io/jadenfix/beater/dashboard@sha256:2cfdc628375ebeacbd88fd2a0ef540e382da5a5c0400b4950b56472685e92053`
- Dashboard e2e image digest: `ghcr.io/jadenfix/beater/dashboard-e2e@sha256:4d7cd50903f708c2c29328707abb721ddff1c098034a38a6090ef0f9eb8b56cd`
- OTEL Python image digest: `ghcr.io/jadenfix/beater/otel-python@sha256:ee6510fbb35fe59acb84817b68485527bcbd1f067226ad61a3bee091597010d0`
- Quickstart snippet: `examples/python/five_line_otel.py`
- API endpoint: `http://127.0.0.1:8080`
- OTLP endpoint: `http://127.0.0.1:4317`
- Dashboard base: `http://127.0.0.1:3001`
- Quickstart release ID: `gate2-3d9c7bc5ad38-1782145474-83545`
- Quickstart trace: `3725033cbe940f1674a9d19ab72d3904`
- Quickstart span: `not requested`
- Quickstart dashboard: http://127.0.0.1:3001/?tenant=demo&project=demo&environment=local&trace=3725033cbe940f1674a9d19ab72d3904
- Quickstart browser proof: passed
- All-kind nested trace: `4849bef9a116057c6f481016b3d604f7`
- All-kind dashboard: http://127.0.0.1:3001/?tenant=demo&project=demo&environment=local&trace=4849bef9a116057c6f481016b3d604f7
- All-kind waterfall browser proof: passed
- Browser recording: passed
- Browser recording artifact: `docs/demos/gate2-compose-browser-demo.webm`
- Browser recording notes: `docs/demos/gate2-compose-browser-demo.md`
- Browser recording SHA256: `d4b3864cd3a5a1b2c2c70b329a949c1215b8a07e85203650841a07be95177248`

## Compose Images

```text
CONTAINER                      REPOSITORY                          TAG                                        PLATFORM            IMAGE ID            SIZE                CREATED
beater-stopwatch-beaterd-1     ghcr.io/jadenfix/beater/beaterd     3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab   linux/arm64         2f3f45f686c4        88.4MB              2 hours ago
beater-stopwatch-dashboard-1   ghcr.io/jadenfix/beater/dashboard   3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab   linux/arm64         2cfdc628375e        99.2MB              8 hours ago
proof-image beaterd ghcr.io/jadenfix/beater/beaterd:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab ghcr.io/jadenfix/beater/beaterd@sha256:2f3f45f686c4a5e159b4683ae0b6a7ca72a88fee2be2feacbbd770e95d671ee9
proof-image dashboard ghcr.io/jadenfix/beater/dashboard:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab ghcr.io/jadenfix/beater/dashboard@sha256:2cfdc628375ebeacbd88fd2a0ef540e382da5a5c0400b4950b56472685e92053
proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab ghcr.io/jadenfix/beater/dashboard-e2e@sha256:4d7cd50903f708c2c29328707abb721ddff1c098034a38a6090ef0f9eb8b56cd
proof-image otel-python ghcr.io/jadenfix/beater/otel-python:3d9c7bc5ad38fabc828462d64e8fbe8f0b1521ab ghcr.io/jadenfix/beater/otel-python@sha256:ee6510fbb35fe59acb84817b68485527bcbd1f067226ad61a3bee091597010d0
```

This is an automated local stopwatch proof. The mandate still requires an
outside-person run to fully close Gate 2.

Regenerate:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```
