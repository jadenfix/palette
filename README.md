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
- `beaterctl smoke --http-url ...` remote mode for live `beaterd` OTLP HTTP/gRPC smoke checks
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
- buffered ingest mode that durably queues canonical trace writes before hot-store persistence
- scoped trace-write drain/status API with typed 429 backpressure responses
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
- API route for native trace ingest and trace readback
- API route for tenant-scoped span search
- API routes for admin key creation/revocation and strict trace/search/dataset/eval/alert authorization
- API routes for provider-secret create/list/revoke, judge evaluation, and judge ledger readback
- API route for tenant/project usage summaries over metered judge spend
- API route for tenant/project audit event readback and audited trace unmasking
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
- `beaterctl bus-fixture` validates durable queue reopen, retry, and DLQ behavior
- `beaterctl ingest-outage-fixture` validates accepted buffered ingest, retry during TraceStore outage, and recovery drain
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

With `beaterd` running in local auth mode, remote smoke can target the live
server:

```bash
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080 --otlp-grpc-url http://127.0.0.1:4317
```

`beaterd` reads `BEATER_PROVIDER_SECRET_KEY` as a base64 32-byte provider-secret
encryption key when set. Without it, local OSS/dev mode creates
`provider-secrets.key` under the data directory and uses that key to encrypt
`provider-secrets.sqlite`.
