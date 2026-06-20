# Beater

Beater is a Rust-first open-source agent observability, replay, and eval platform.
The target is an OSS self-hostable debugger/evals harness plus a hosted API
product that can compete with Arize Phoenix, Braintrust, LangSmith, Langfuse, and
Judgment-style systems.

The core loop:

```text
instrument agent -> inspect trace -> promote failure to dataset -> run evals
-> compare candidate -> gate CI -> monitor production
```

## Canonical Docs

- [Architecture](ARCHITECTURE.md): system design, Rust workspace, storage,
  Vercel split, evaluator lanes, replay, compliance, and milestones.
- [Ship Requirements](REQUIREMENTS.md): requirement-by-requirement checklist for
  what must be true before the platform can be called shipped.

## Current State

This repo now contains the first tested Rust vertical slice:

- all-in-one `beaterd` HTTP server
- `beaterctl smoke` local OTLP ingest command that drains trace-write and trace-ingested work
- `beaterctl smoke --http-url ...` remote mode for live `beaterd` OTLP HTTP/gRPC smoke checks with measured query lag
- OTLP/HTTP protobuf trace ingest endpoint with raw protobuf preservation
- OTLP/gRPC TraceService ingest mounted by `beaterd` alongside axum
- Tantivy-backed structured and full-text span search
- Parquet/DataFusion cold trace archive export/query
- canonical schema and agent span taxonomy
- immutable raw envelope creation
- raw-envelope lookup by tenant/project/idempotency key
- filesystem artifact storage with hash verification
- trait-only `beater-store` boundary with `TraceStore`/`ArtifactStore`/`MetadataStore`/`QuotaLimiter`
- SQLite and in-memory metadata store conformance coverage for org/project/environment/RBAC boundaries
- SQLite and in-memory quota limiter conformance coverage for shared fixed-window project quotas
- SQLite `TraceStore` implementation in `beater-store-sql`
- filesystem artifact store implementation in `beater-store-obj`
- typed `StoreError`/provider/adapter errors across storage, judge, dataset, experiment, search, and review trait contracts
- currency-checked `Money` math plus injectable core `Clock`
- bounded in-memory bus plus SQLite durable bus with persisted retry and DLQ behavior
- explicit bus ack/inflight semantics with SQLite recovery of unacked leased work after restart
- tenant/project-scoped DLQ replay that resets attempts and requeues work without deleting unreplayed dead letters
- buffered ingest mode that durably queues canonical trace writes before hot-store persistence
- scoped trace-write drain/status API with typed 429 backpressure responses
- trace-write retry semantics for downstream publish failures after successful trace storage, relying on store idempotency to avoid double writes
- API quota 429 responses include retry/reset headers
- `beaterd` quota flags/env (`--per-project-event-quota`, `--quota-window-seconds`, `--quota-db-path`) with live two-replica shared-counter/reset smoke coverage
- API duplicate native-ingest fixture proves idempotent raw/span writes and downstream dedupe
- native ingest pipeline with payload/attribute governance and trait-backed windowed quotas
- `trace.ingested` downstream drain API and `beaterd` worker for off-hot-path search indexing with retry/DLQ handling
- deterministic evaluator lane and judge-broker budget model
- encrypted-at-rest BYOK provider-secret store for judge/model providers
- judge broker with preflight budget reservation, request-hash cache hits, and SQLite audit ledger
- persisted usage ledger that meters judge charged cost idempotently by judge call
- persisted audit event store for privileged PII unmask attempts
- OpenAI-compatible and Anthropic HTTP judge providers with retry/backoff behavior and structured-score parsing
- trace-to-dataset promotion, immutable dataset versions, and offline deterministic plus judge-backed dataset evals
- baseline-vs-candidate experiment runs with per-case scores, judge-backed scoring, and confidence-bound gates
- in-process agent harness adapters that run baseline/candidate releases over dataset versions
- experiment reports that capture per-case baseline/candidate agent traces and the gate policy used for the decision
- persisted CI gate policies and gate-run audit reports over latest or explicit experiment reports
- persisted human review queues, review tasks, annotations, and annotation-to-dataset promotion
- persisted judge/human calibration reports with agreement, confusion counts, and Cohen's kappa
- online sampling decisions and signed webhook alert delivery with dedupe/suppression
- Wasmtime-backed deterministic WASM evaluator sandbox with fuel limits and no host imports
- experiment gate confidence-bound logic
- persisted replay cassette event store, deterministic/forked replay execution, and failure attribution
- API-key hashing/scoping, persisted SQLite API-key metadata, last-used audit timestamp, and HMAC webhook signing primitives
- optional strict API auth mode on `beaterd` with environment-bound scoped keys
- checksummed SQLite migration runner wired into `beaterd` startup for local OSS stores
- API route for native trace ingest and trace readback
- API routes for trace lists, selected span detail, redaction-aware span I/O inspection, and audited unmask reads
- `/openapi.json` documents the dashboard read surface and generates the dashboard TypeScript client
- multi-stage cargo-chef Dockerfile and `docker-compose.yml` for the current self-host topology
- migration contracts for SQLite local runtime plus Postgres and ClickHouse scale/control-plane paths
- stock OpenTelemetry Python examples: a literal five-line quickstart snippet and an all-kind agent trace fixture
- GHCR prebuilt image workflow plus a Compose override for the five-minute clean-machine stopwatch path
- Gate 2 proof scripts for OpenAPI drift, local clone-to-browser smoke, compose smoke, and browser demo recording
- API route for tenant-scoped span search
- API routes for admin key creation/revocation and strict trace/search/dataset/eval/alert authorization
- API routes for provider-secret create/list/revoke, judge evaluation, and judge ledger readback
- API route for tenant/project usage summaries over metered judge spend
- API route for tenant/project audit event readback and audited trace unmasking
- API route for scoped ingest DLQ replay
- API routes for archiving hot traces and querying cold spans
- API routes for dataset creation, trace promotion, versioning, deterministic eval runs, and judge-backed eval runs
- API route and CLI fixtures for deterministic and judge-backed experiment comparison plus local agent harness runs
- API routes for persisted gate creation and gate runs over stored experiment reports
- API routes for review queue creation, trace review task enqueue, annotation submission, and annotation promotion to dataset cases
- API route for calibration over persisted dataset eval reports and human-labeled dataset cases
- API routes and CLI fixture for online sampling and signed alert webhooks
- `beaterd` defaults to a persistent SQLite bus backend, with an in-memory backend still available
- `beaterd` runs configurable trace-write and trace-ingested background workers for buffered ingest and downstream indexing
- live `beaterd` integration test proving OTLP HTTP and gRPC traces become readable and searchable through public APIs
- live `beaterd` integration test proving trace-ingested work recovers after consumer kill, restart, DLQ, and replay
- live `beaterd` integration test proving storage write failure accounts events as explicit error, DLQ, or recovered with no silent loss
- `beaterctl bus-fixture` validates durable queue reopen, retry, DLQ, and replay behavior
- `beaterctl ingest-outage-fixture` validates no-silent-drop accounting across explicit error, DLQ, and recovery during a simulated TraceStore outage
- `beaterctl replay-fixture` validates persisted cassette replay without live provider/tool calls
- `beaterctl judge-fixture` validates encrypted BYOK secret persistence, cached judge calls, budget metering, and ledger redaction
- `beaterctl usage-fixture` validates idempotent judge usage records and cached zero-cost audit records
- `beaterctl audit-fixture` validates persisted allowed/denied PII-unmask audit events
- `beaterctl judge-dataset-fixture` validates judge broker evaluation over a persisted dataset version
- `beaterctl judge-experiment-fixture` validates judge-backed candidate-vs-baseline gates
- `beaterctl gate-policy-create`, `gate-run`, and `gate-run-fixture` validate CI blocking on persisted latest experiment regressions
- `beaterctl review-fixture` validates human annotation promotion into an eval-ready dataset case
- `beaterctl calibration-fixture` validates persisted judge/human agreement and kappa reports
- `beaterctl api-key-create` / `api-key-revoke` bootstrap helpers for local and hosted deployments
- `web/dashboard` Next.js trace debugger with generated OpenAPI types, status/kind/time/model/cost/latency/release filters, trace table, icon-coded agent span waterfall, span detail, and audited I/O unmask controls

## Verify

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
cargo run -q -p beaterctl -- judge-fixture --data-dir /tmp/beater-judge
cargo run -q -p beaterctl -- usage-fixture --data-dir /tmp/beater-usage
cargo run -q -p beaterctl -- audit-fixture --data-dir /tmp/beater-audit
cargo run -q -p beaterctl -- ingest-outage-fixture --data-dir /tmp/beater-ingest-outage
cargo run -q -p beaterctl -- judge-dataset-fixture --data-dir /tmp/beater-judge-dataset
cargo run -q -p beaterctl -- judge-experiment-fixture --data-dir /tmp/beater-judge-experiment
cargo run -q -p beaterctl -- gate-run-fixture --data-dir /tmp/beater-gate
! cargo run -q -p beaterctl -- gate-run --data-dir /tmp/beater-gate --tenant-id demo --project-id demo --gate-id main
cargo run -q -p beaterctl -- review-fixture --data-dir /tmp/beater-review
cargo run -q -p beaterctl -- calibration-fixture --data-dir /tmp/beater-calibration
cargo run -q -p beaterd -- --data-dir /tmp/beaterd --judge-provider http-routing --auth-mode required
```

## Clean Clone To Browser

Exact Docker Compose stopwatch proof for the mandate's clean-machine path:

```bash
git clone https://github.com/jadenfix/beater.git
cd beater
scripts/gate2-outside-run.sh
```

The outside-run wrapper rejects warm-loop reuse, local source builds, alternate
ports, mutable pull-policy overrides, prebuilt image overrides, and evidence
artifact path overrides, rejects alternate Compose project names and teardown
overrides, then runs the stopwatch script with proof writing, browser proof,
and browser recording enabled. It also sets an
`Outside-run wrapper: yes` marker in the stopwatch proof; completed
outside-person proof validation rejects evidence without that marker. The script
first removes any previous Compose project/volumes, then runs
`docker compose up`, sends `examples/python/five_line_otel.py` from the
prebuilt stock OpenTelemetry Python runner container, waits until the trace is
visible in `localhost:3000`, and fails if time-to-first-trace exceeds 300
seconds. It also records time-to-quickstart-click when browser proof is
enabled. It leaves the dashboard running by default so a human can click
through the trace.
Before starting Compose it checks Docker and curl. It removes any previous Beater stopwatch project, then checks the required host ports. For
outside-person evidence, free the default `8080`/`4317`/`3000` ports rather than
using alternate ports.
By default it uses `docker-compose.prebuilt.yml` and pulls current GHCR images
published by `.github/workflows/container-images.yml`. The stopwatch script
pins `beaterd`, `dashboard`, `dashboard-e2e`, and `otel-python` to the checked-out commit SHA
tags by default, then records the image references and resolved GHCR digests in
the proof. Set
`BEATER_GATE2_LOCAL_BUILD=1` when you intentionally want to build the server and
dashboard images from source. Set `BEATER_GATE2_REUSE=1` only for local
warm-loop debugging. Set `BEATER_GATE2_BROWSER_PROOF=1` to also run the
prebuilt `dashboard-e2e` Playwright browser proof for both the five-line trace
and the all-kind nested agent waterfall in the same proof run. The five-minute
SLO is enforced for time-to-first-trace and time-to-quickstart-click; all-kind
and recording steps run afterward with per-step timeouts. Set
`BEATER_GATE2_RECORD_DEMO=1` to write `docs/demos/gate2-compose-browser-demo.webm`
and its SHA-pinned notes from the same browser session.

The five-line snippet is intentionally plain OpenTelemetry. To run the exact
manual step after `docker compose up -d --build`, install stock OTEL packages
and execute it against the local OTLP port:

```bash
python3 -m venv /tmp/beater-otel
/tmp/beater-otel/bin/pip install opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 /tmp/beater-otel/bin/python examples/python/five_line_otel.py
sed -n '1,5p' examples/python/five_line_otel.py
```

Fast local proof with built binaries, the all-kind stock Python OpenTelemetry
trace, and the Next dashboard:

```bash
scripts/gate2-proof.sh
```

Containerized self-host proof:

```bash
scripts/smoke-compose.sh
```

The compose topology starts `beaterd`, the dashboard, Postgres, NATS JetStream,
and MinIO; ClickHouse is available with the `clickhouse` profile. The current
`beaterd` runtime still stores local OSS state in SQLite under `--data-dir`.
Postgres, NATS, MinIO, and ClickHouse are present as self-host topology and
schema-contract services, not yet as fully wired Rust runtime backends.

The browser proof should finish by opening:

```text
http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local
```

Run OpenAPI drift detection separately with:

```bash
scripts/check-openapi-drift.sh
```

To regenerate the committed Gate 2 browser capture under `docs/demos/`:

```bash
BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh
```

To write the automated compose stopwatch artifact under `docs/demos/`:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 KEEP_BEATER_COMPOSE=0 scripts/gate2-compose-stopwatch.sh
```

Gate 2 still requires an unaided outside-person run before it can be called
passed. Before handing the repo to the outside runner, a maintainer should run
the public handoff verifier after the `container-images` workflow has published
the current commit:

```bash
scripts/check-gate2-public-handoff.py
```

That first runs `scripts/check-gate2-outside-readiness.py`, then performs a
fresh clone from `https://github.com/jadenfix/beater.git`, verifies the clone is
on the exact same commit, reruns the cloned readiness check, and dry-runs the
cloned `scripts/gate2-outside-run.sh` wrapper. The readiness check verifies the
repo is on clean `main`, `origin` points at this GitHub repo, the
pending/completed outside-proof file is structurally valid, and the current-SHA
`beaterd`, `dashboard`, `dashboard-e2e`, and `otel-python` GHCR images are
public for both `linux/amd64` and `linux/arm64`.

Use [docs/demos/gate2-outside-person-proof.md](docs/demos/gate2-outside-person-proof.md)
as the required evidence template for that run. After the outside runner has
completed the stopwatch command, generate the proof from the stopwatch artifact:

```bash
scripts/generate-gate2-outside-proof.py \
  --runner-name "..." \
  --relationship "..." \
  --prior-exposure "none" \
  --machine-os "..." \
  --browser "..." \
  --preflight-status "passed" \
  --attest-outside-run
```

Then validate it with:

```bash
scripts/validate-gate2-outside-proof.sh
```

The validator checks the outside-person template, stopwatch proof file, and
screen-recording notes from the same run. It rejects alternate ports, warm-loop
reuse, placeholder dashboard URLs, mismatched trace IDs, mismatched commit SHA,
mismatched API/dashboard endpoints, non-main or stale commit evidence,
mismatched SHA-pinned image references, mismatched image digests,
non-repo-relative `docs/demos/` artifacts, and non-prebuilt GHCR image digests.
It rejects recording notes from a different dashboard session and any screen
recording hash that does not match the committed file. The recording artifact
must be a WebM capture of at least 64 KiB with a WebM/EBML header. The notes
must also describe the full recorded flow: quickstart trace, `llm.call`,
prompt, completion, model, tokens, cost, latency, and run -> turn -> step ->
tool -> MCP waterfall.
The `gate2-proof-contract` GitHub workflow runs the validator template check
and the executable proof-artifact fixture tests on pull requests and `main`.

Warm-loop debugging can skip the pre-run cleanup, but this is not acceptable
evidence for Gate 2:

```bash
BEATER_GATE2_REUSE=1 scripts/gate2-compose-stopwatch.sh
```

Local source builds are measured but are not the SLO path:

```bash
BEATER_GATE2_LOCAL_BUILD=1 scripts/gate2-compose-stopwatch.sh
```

With `beaterd` running in local auth mode, remote smoke can target the live
server and reports `trace_query_lag_ms`:

```bash
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080 --otlp-grpc-url http://127.0.0.1:4317
```

`beaterd` reads `BEATER_PROVIDER_SECRET_KEY` as a base64 32-byte provider-secret
encryption key when set. Without it, local OSS/dev mode creates
`provider-secrets.key` under the data directory and uses that key to encrypt
`provider-secrets.sqlite`.
