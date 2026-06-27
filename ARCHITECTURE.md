# Beater Architecture

Beater is a Rust-first, open-source agent observability, replay, and eval platform.
It is designed to be credible as an OSS project, viable as a hosted API product, and
agent-native enough to compete with Arize Phoenix, Braintrust, LangSmith, Langfuse,
and Judgment-style agent debugging systems.

The core product loop is the definition of "shipped":

```text
instrument agent
  -> inspect trace/span tree
  -> promote failure to dataset
  -> run evals
  -> compare candidate change
  -> block or allow CI
  -> monitor production after deploy
```

If that loop is not excellent end to end, the rest of the platform is premature.

## 1. Non-Negotiable Principles

1. Ship one Rust binary first.
   Crates are modular, but the MVP runs as one `beaterd` process. Later service
   splits are thin bins over the same crates and are justified only by measured
   scale pressure.

2. Use standards at the edge, not proprietary lock-in.
   OTLP, W3C trace-context, OpenInference, OpenTelemetry GenAI conventions, and
   common framework exporters are first-class. The native Rust SDK is an
   accelerator, not the adoption gate.

3. Store immutable raw data and normalized projections.
   Every ingest event keeps its raw envelope, source schema URL/version, payload
   hash, normalizer version, and unmapped attributes. The normalized model is a
   canonical superset used for queries and UI. Lossless round-trip is promised
   only through the raw copy, not through lossy standards projections.

4. Design privacy and tenancy before hosted ingest.
   Tenant isolation, regional pinning, redaction, crypto-shredding, retention,
   scoped keys, audited PII access, and artifact encryption are v1 constraints.

5. Split deterministic evals from model-dependent evals.
   User-supplied deterministic evaluators run in a WASI Component Model sandbox
   via Wasmtime with no network. LLM-judge and embedding evals run through a
   judge broker with budgets, redaction, model versioning, and audit trails.

6. Be honest about replay.
   Deterministic replay requires provider, tool, memory, and clock cassettes.
   Without cassettes, the product calls it forked replay or simulation.

7. Scope Vercel correctly.
   Vercel runs the dashboard, stateless Rust functions, edge/control-plane API,
   webhooks, Blob, Queues, and Cron. Long-running ingest listeners, ClickHouse
   writers, eval pools, replay pools, and stateful workers run in hosted cells or
   in the OSS all-in-one process.

8. No cloud dependency in OSS.
   The open-source edition must run without calling Beater Cloud. Self-host
   telemetry is opt-out, and all hosted-specific APIs degrade cleanly to local
   equivalents or are absent.

9. Be honest about every number.
   No aggregate is reported as if it were an unbiased population estimate when it
   is not. Tail-sampled roll-ups are **inverse-probability weighted**
   (`sampling_weight = 1/keep_probability`, §9) or explicitly **labeled biased** —
   never silently averaged. Nominal alpha **must equal** actual alpha: the gate's
   confidence/p-value is computed with a method whose stated error rate is its true
   error rate (§10.3), not a hard-coded normal-approximation z. Determinism is the
   **caching** story (request-hash judge caches, cassettes), *not* a claim that
   `temperature=0` makes a model deterministic.

10. Scoring quantifies its own uncertainty and cancels known bias.
    A score is reported with an interval, and noisy/model-dependent scores quantify
    run-to-run uncertainty via **N-trial self-consistency** (§6 dim #12, §10.3).
    Pairwise judge bias is cancelled **structurally** by the A/B order swap
    (§10.1.1), never assumed away.

11. Statistical validity is a product invariant.
    A deploy gate may return *pass* only on a real p-value with the correct test
    for the metric type and **FWER/FDR multiplicity control** across metrics/slices
    (§10.3). An underpowered comparison returns *inconclusive*, never *pass*.

12. Held-out generalization is enforced.
    Every dataset version carries a frozen **Train/Dev/Test** split (§5.3, §6.4); a
    self-improvement change is accepted only on the untouched **Test** split, behind
    a contamination guard. The ruler does not move while the agent is being
    optimized.

13. Zero-code OTLP bootstrap is the default onboarding.
    The first-class adoption path is pointing a standards-based OTLP exporter at
    Beater with **no Beater SDK and no code edits** (§15, §20.8). The native SDK is
    an accelerator, not the gate. The DX SLO is **time to first *scored failure***,
    not time to first trace.

## 2. Editions

| Capability | OSS self-host | Hosted |
| --- | --- | --- |
| All-in-one Rust binary | Required | Used for cells and local dev |
| OTLP and native ingest | Required | Required |
| Trace viewer and span tree | Required | Required |
| Datasets, experiments, offline evals | Required | Required |
| WASI deterministic evaluator sandbox | Required | Required |
| Judge broker with BYOK | Required | Required |
| Managed judge routing and shared judge fleet | Optional BYOK only | Required |
| Online eval sampling and alerts | OSS local/basic | Hosted scaled/commercial |
| Multi-tenant orgs, billing, quotas | Single-cell/basic | Required |
| SSO/SAML, regional pinning UI, advanced audit export | Optional enterprise | Commercial |
| No-cloud operation | Required | Not applicable |

Open-core boundary:

- Apache-2.0 core: ingest, canonical schema, local auth, trace UI, datasets,
  deterministic evals, WASI ABI, judge broker BYOK, replay cassettes, CI gate,
  import/export, Docker Compose, plugin APIs.
- Commercial/hosted: managed multi-region cells, billing, SSO/SAML, enterprise
  audit reporting, regional compliance controls, managed alerts, hosted judge
  fleet, high-scale replay/eval pools, support SLAs.
- Security-critical primitives such as audit event generation and redaction are
  not withheld from OSS. The paid boundary is managed scale and governance UX,
  not the ability to self-host safely.

## 3. Deployment Shape

### 3.1 OSS

Target compose (the shape we are building toward — items not yet wired are
marked):

```text
docker compose up
  beaterd       # one Rust process: API, OTLP, jobs, eval, replay   [built]
  postgres      # metadata + optional local TraceStore               [PgTraceStore implemented, not runtime-wired]
  clickhouse    # optional scale TraceStore                          [ClickHouseTraceStore implemented, not runtime-wired]
  nats          # JetStream durable bus                              [planned: bus is SqliteDurableBus today]
  minio         # object storage                                     [planned: artifacts are local filesystem today]
```

As of `origin/main`, `beaterd` boots with SQLite stores, a `SqliteDurableBus`,
and a filesystem `FsArtifactStore` only; there is no backend selector that wires
Postgres/ClickHouse, NATS, or S3 into the running service (see §20.2 Phase 0
#0.1, #0.5 and §8.2). The architecture contract is that the same code paths
*must* work against Postgres, ClickHouse, a NATS/Kafka bus, and S3-compatible
storage once those backends are wired — the trait boundaries (§8.1) exist
precisely so that wiring is additive, not a rewrite.

### 3.2 Hosted

```text
Vercel
  dashboard
  Rust Functions for stateless /v1 API routes
  Vercel Queues for Vercel-native async boundaries
  Vercel Blob for small/control-plane artifacts where appropriate
  Cron for short scheduled tasks

Hosted Rust cells
  long-running OTLP gRPC/HTTP ingest
  queue consumers
  ClickHouse writers
  replay/eval/judge workers
  WebSocket/SSE fanout if needed

Managed data
  Postgres for metadata
  ClickHouse for hot trace analytics
  S3/R2/GCS for artifacts and raw envelopes
  Parquet/DataFusion for cold retention
```

Current platform constraints, verified June 19, 2026:

- Vercel's official Rust runtime is beta, runs on Fluid Compute, and uses
  `vercel_runtime` handlers in `api/`.
- Vercel Functions have request/response and body-size limits; the documented
  payload limit is 4.5 MB.
- Vercel Functions do not act as WebSocket servers.
- Vercel Queues are beta durable append-only topics with at-least-once delivery,
  idempotency keys, push consumers, and poll consumers. Poll mode is suitable for
  off-Vercel workers.
- Therefore Vercel can be the hosted control plane and stateless API surface, but
  not the only runtime for stateful ingest and long-running workers.

Primary source links:

- https://vercel.com/docs/functions/runtimes/rust
- https://vercel.com/docs/functions/limitations
- https://vercel.com/docs/queues
- https://vercel.com/docs/queues/poll-mode

### 3.3 Deployment & Distribution (server, docs, 7 SDKs, MCP, CLI)

Beater is not one artifact — it is a server, a documentation site, seven
generated SDK clients plus a native Rust SDK, an MCP server, and a CLI. Each is
built, versioned, published, and deployed differently, but they all derive from
the same contract (the `CLAUDE.md` single-source-of-truth rule), so they cannot
drift. What is **[built]** today vs **[planned]** is marked.

| Artifact | Built from | Published as | Versioning | Deployed / consumed | Status |
| --- | --- | --- | --- | --- | --- |
| **`beaterd` server** (+ `beaterctl`) | Rust workspace, multi-stage `cargo-chef` Dockerfile | multi-arch GHCR image (`container-images` workflow); also a raw binary | git SHA tag per build; semver tag at release | OSS: `docker compose up`. Hosted: Rust cells (§3.2). | [built] |
| **Dashboard** | `web/dashboard` (Next.js) consuming the generated TS client | GHCR image (`container-images`); Vercel deploy for hosted | git SHA / release tag | OSS: compose service on `:3000`. Hosted: Vercel (§3.2). | [built] |
| **Docs site** | renders the committed `sdks/openapi/beater-api.json` | static site / hosted docs | tracks the spec version | published from `main`; the committed spec is the source so docs never drift | [planned site; spec is built] |
| **7 generated SDK clients** (`sdks/clients/*`: py, ts, go, java, c, cpp, …) | OpenAPI spec via `scripts/regen-sdks.sh` (+ reproducible C/C++ patches) | committed in-repo; published to each language registry (PyPI / npm / pkg.go.dev / Maven, etc.) by `scripts/publish-sdk.sh` | spec/contract version; per-language package version | `pip`/`npm`/`go get`/Maven by users; `sdk-contract` CI blocks any drift from the spec | clients [built]; registry publish [planned] |
| **Native Rust SDK** (`sdks/rust`) | hand-written, `tracing`/OTel layers; **excluded** from the cargo workspace | crates.io package | semver | `cargo add beater` (accelerator, not the adoption gate, §1 #2, §15) | [built in-repo; crates.io publish planned] |
| **MCP server** (`beater-mcp`) | every `/v1` operation resolved from the spec at runtime, + composite recipes + RSI tools (§21) | served by `beaterd` at `POST /mcp` | tracks the spec (operations resolved at runtime → auto-in-sync) | **stdio** for local clients (Claude Code/Cursor/Codex) and **streamable-HTTP + OAuth 2.1** for hosted (§21). streamable-HTTP is [built]; **stdio is the one [planned] transport gap (§21).** | streamable-HTTP [built]; stdio [planned] |
| **CLI** (`beaterctl`) | resolves `/v1` operations from the spec at runtime (`beater api`), plus local fixtures/smoke | the server image, and a standalone binary | tracks server/spec | `cargo run -p beaterctl` or the released binary; used in CI smoke + local dev | [built] |

The discipline that keeps these consistent is one workflow (`CLAUDE.md`,
`CONTRIBUTING.md`): a `/v1` change runs `cargo xtask regen-spec` →
`scripts/regen-sdks.sh` → `cargo xtask regen-semconv`, and
`scripts/check-contract-sync.sh` (mirrored by the `sdk-contract` CI gate) blocks
any drift across the spec, all 7 clients, semconv, MCP, CLI, and docs. Because
the MCP and CLI resolve operations from the committed spec at runtime, they
update automatically; the docs render the committed spec, so they update too. The
per-artifact "how to verify it's deployed/in-sync" commands are in §22.

## 4. Rust Workspace

The operational split is logical first, physical later.

### Naming convention — the "beat-boxes"

The **project** is **Beater**. Its **crates are collectively the "beat-boxes"**, and
every crate carries a **rhythm/beat-themed name** — the platform keeps the beat of an
agent's behavior, so the components that make up the platform are named for the parts
of a beat. The beat name is the **primary** name used throughout this document; the
underlying Cargo crate (`beater-*`) is given in parentheses on first use and is the
**crosswalk** back to the code.

This table is the **source of truth** for the mapping. Each beat name maps to exactly
one role, and no two roles share a name. Crate directories still ship under their
`beater-*` paths today; the **physical directory rename to the beat names is a
pre-1.0 follow-up** (tracked as the "beat-boxes rename" task, §20.9) — this document
establishes the naming now and the code adopts it next. Until then, reach for a
component by its crate path; reason about it by its beat name. Format below:
**Beatname** — role — (crate).

**Conductor & foundation**

- **Beater** — the conductor: the product and the default all-in-one binary that runs
  every box in one process — (bin: `beaterd`).
- **Downbeat** — foundational primitives: IDs, entity types, typed money, clocks,
  tenant scope (the "one" every other beat counts from) — (crate: `beater-core`).
- **Beatmap** — the canonical span/run/eval schema, mappings, roll-ups, conventions,
  `sampling_weight`, DatasetCase `split`; the single source of truth the contract is
  generated from — (crate: `beater-schema`).

**Signal in (ingest) & cadence**

- **Upbeat** — the pickup: incoming signal — OTLP receive/export and the
  OTLP/OpenInference/GenAI → canonical normalizer, plus auth/quota/raw-append,
  tail-sampling, and `sampling_weight` stamping — (crates: `beater-otlp` +
  `beater-ingest`).
- **Syncopation** — off-grid signal made to fit the grid: Temporal workflow-history →
  canonical span normalization — (crate: `beater-temporal`).
- **Drumbeat** — durable cadence: the queue/job bus that keeps work moving on tempo
  (`SqliteDurableBus` today; NATS/Kafka planned) — (crate: `beater-bus`).

**Trace storage — the Groove**

- **Groove** — trace storage: the `TraceStore`/`MetadataStore`/`ArtifactStore`/
  `QuotaLimiter` trait boundary and weighted roll-up queries (the groove every trace
  lays down) — (crate: `beater-store`).
- **Soundcheck** — the shared trait-conformance suite run against every Groove backend
  (you sound-check every box before the show) — (crate: `beater-store-conformance`).
- **Sample** — the in-memory Groove backend for tests/dev (a quick captured sample) —
  (crate: `beater-store-memory`).
- **Vinyl** — the durable SQL Groove backends: SQLite (runtime default) plus the
  ClickHouse/Postgres trace stores (the records the groove is pressed onto) — (crate:
  `beater-store-sql`).
- **Crate** — the artifact/raw-envelope object store (`FsArtifactStore`); a record
  crate where the raw pressings are filed — (crate: `beater-store-obj`).
- **Cold Storage** — the Parquet cold-tier archive over Arrow/DataFusion (the back-room
  crate of old records) — (crate: `beater-archive`).
- **Crate Dig** — full-text search (Tantivy) over spans; digging the crates for the
  record you want — (crate: `beater-search`).

**Scoring backbone — the Backbeat**

- **Backbeat** — the scoring backbone: the evaluator catalog, scoring contracts,
  paired comparison and aggregation, the LLM/embedding judge broker, and the
  statistics correctness layer (real p-values, CIs, power, FWER/FDR) — (crates:
  `beater-eval` + `beater-judge` + `beater-stats` [planned]).
- **Soundproof** — the WASI/Wasmtime sandbox that runs user evaluators isolated from
  network/host (the soundproof booth) — (crate: `beater-sandbox`).
- **Riff** — the custom-scorer registry over the Soundproof sandbox (player-supplied
  riffs) — (crate: `beater-scorers` [planned]).
- **Tuning** — agent/score proper-scoring calibration (Brier/ECE/reliability +
  recalibration map) and judge-vs-human agreement; tuning the instrument so the
  reading is true — (crate: `beater-calibration`).
- **Encore** — datasets, versions, examples, trace promotion, and the Train/Dev/Test
  `split` + contamination guard; the failures you bring back for another take —
  (crate: `beater-datasets`).
- **Setlist** — review queues, annotations, and human labels; the curated list a human
  works through — (crate: `beater-human`).

**Improvement & replay**

- **Beatboxing** — the recursive-self-improvement loop and experiment harness:
  candidate-vs-baseline comparisons, the propose→simulate→accept episode, and the
  agent run harness (improvising on the beat to make it better) — (crates:
  `beater-experiments` + the RSI tools, §21).
- **Cue** — the CI/CD deploy gates and policy evaluation: a gate cues the deploy only
  on a real, powered, multiplicity-corrected, held-out win — (crate: `beater-gates`).
- **Rewind** — cassettes + deterministic/forked replay and earliest-failing-span
  attribution (rewind the tape to the moment it went wrong) — (crate: `beater-replay`).
- **Backspin** — online-eval scoring worker: continuously re-scoring sampled production
  traces (the turntable's backspin re-reads the groove) — (crate: `beater-online`
  [planned]).
- **Mixdown** — prompt registry/versioning/playground: where prompt versions are mixed
  and committed — (crate: `beater-prompts` [planned]).

**Anomaly, health & metering**

- **Offbeat** — anomaly/drift alerting over trace/score signals (it fires when the
  agent falls off the beat) — (crate: `beater-alerts`).
- **Heartbeat** — self-observability: the Prometheus metrics facade, health, and SLO
  instrumentation that proves the platform itself is alive — (in `beaterd`:
  `metrics.rs` / `metrics_http.rs`; there is no separate crate).
- **Tempo** — the usage ledger, billing meters, and spend summaries (the rate at which
  value/spend accrues) — (crate: `beater-usage`).
- **Bandwidth** — plans/subscriptions/Stripe metered sync; how much throughput a plan
  buys — (crate: `beater-billing` [planned]).
- **Tip Jar** — the autonomy-credits metering layer (deferred productization, §21.6);
  what you pay into for verified gains — (crate: `beater-credits` [deferred]).

**Identity, secrets & trust**

- **Backstage** — API keys, JWT/session, RBAC types, and audit scopes; the
  who's-allowed-backstage door — (crate: `beater-auth`).
- **Guestlist** — users, password auth, browser sessions, org membership; the named
  people on the list — (crate: `beater-accounts`).
- **Wristband** — OAuth 2.1 core (clients, PKCE codes, access/refresh tokens); the
  scoped wristband you're issued at the door — (crate: `beater-oauth`).
- **Door** — the OAuth 2.1 HTTP surface wired into Beater (where the wristband is
  checked) — (crate: `beater-oauth-server`).
- **All-Access** — role/permission resolution inside `authorize()` (enforced RBAC) —
  (crate: `beater-rbac` [planned]).
- **Passport** — OIDC/SAML/SCIM enterprise identity (your credentials from another
  venue) — (crate: `beater-identity` [planned]).
- **Stash** — opaque provider-secret refs, BYOK metadata, and revocation (where keys
  are stashed, never shown) — (crate: `beater-secrets`).
- **Vault** — crypto primitives: Argon2 keys, ChaCha20 envelope encryption, signed
  webhooks (the vault the Stash relies on) — (crate: `beater-security`).
- **Logbook** — privileged-access audit events and tamper-evident readback (the venue's
  signed logbook) — (crate: `beater-audit`).

**Browser-agent family — the Liveset**

- **Liveset** — the browser-agent observability contract: the shared foundation that
  turns browser-driving agents into first-class observed agents (the live performance
  on stage) — (crate: `beater-browser`).
- **Liveset: DJ Deck** — the Chrome DevTools Protocol driver backend — (crate:
  `beater-browser-cdp`).
- **Liveset: Turntable** — the Playwright driver backend — (crate:
  `beater-browser-playwright`).
- **Liveset: Mixer** — the WebDriver/fantoccini driver backend — (crate:
  `beater-browser-webdriver`).
- **Liveset: Bootleg** — per-step console + network + DOM capture (the bootleg
  recording of the live set) — (crate: `beater-browser-capture`).
- **Liveset: Roadie** — the browser-agent run harness that drives the set — (crate:
  `beater-browser-harness`).

**Surfaces, tooling & tracked-but-deferred**

- **Beatbox** — the MCP tool-belt: the MCP server exposing every `/v1` operation as a
  tool, the composite "recipe" tools, and the folded-in improvement loop (§21) (the
  box you reach into for a tool) — (crate: `beater-mcp`).
- **Mixing Board** — the axum routers, OpenAPI surface, SSE/read APIs, the mapping
  importer, and bulk promote (every signal routed and faded to the right output) —
  (crate: `beater-api`).
- **Stomp Box** — the CLI: `init`, `ingest test`, `eval run`, `gate`, `export`, and
  `beater api` over the spec (the foot pedal you stomp to trigger an action) — (bin:
  `beaterctl`).
- **Roadcase** — build/regen tasks: `regen-spec`, `regen-semconv`, loadgen (the case
  that holds the road crew's tools) — (crate: `xtask`).
- **Tech Rider** — the criterion benches + load-test fixtures (the rider that pins the
  performance requirements) — (crate: `beater-bench` [planned]).
- **Studio** — the deferred visual agent-design canvas (front-end ↔ back-end map,
  live traces, drag-to-add); design-only, idea preserved (§21.5b) — (crate:
  `beater-studio` [deferred]).
- **Backline** — the deferred auto-provisioned tool-belt (managed vector memory, SQL,
  web search, scrapers); the gear the venue provides on demand (§21.5c) — (crate:
  `beater-toolbelt` [deferred]).

**Pipeline (not a crate):**

- **Metronome** — the single combined CI/CD pipeline that keeps every box on tempo and
  makes drift impossible to merge silently (§22.5).

The crate list below reflects the workspace as it exists on `origin/main`
(verified 2026-06-27). Crates marked **[planned]** are described elsewhere in
this document as future work and do not yet exist; everything else is a real
workspace member in `Cargo.toml`. Where this section once named a crate that the
code never grew (`beater-normalize`, `beater-store-ch`, `beater-sdk`,
`beater-telemetry`), the note explains where that responsibility actually lives.

```text
beater/
  Cargo.toml
  crates/
    beater-core/          # Downbeat — IDs, entity types, typed money, clocks, tenant scope
    beater-schema/        # Beatmap [CHANGED] canonical event/run/span/eval schemas, mappings,
                          #   rollups, conventions PLUS `sampling_weight` on the keep path
                          #   and WEIGHTED roll-ups/aggregates (§9, §13); DatasetCase `split`
    beater-otlp/          # Upbeat (with beater-ingest) — tonic/prost OTLP HTTP/gRPC
                          #   receive/export AND the OTLP/OpenInference/GenAI -> canonical
                          #   normalizer (there is no separate beater-normalize crate)
    beater-temporal/      # Syncopation — Temporal workflow-history -> canonical span normalization
    beater-ingest/        # Upbeat [CHANGED] auth, quota, raw append, normalization, tail-sampling
                          #   PLUS recording `sampling_weight = 1/keep_probability` on every
                          #   kept span so downstream aggregates can be unbiased (§9)
    beater-store/         # Groove [CHANGED] TraceStore, MetadataStore, ArtifactStore, QuotaLimiter
                          #   traits and StoreError; roll-up/aggregate queries become WEIGHTED
                          #   by `sampling_weight` (§9, §13) so tail-sampled totals are unbiased
    beater-store-conformance/ # Soundcheck — shared trait-conformance test suite run against every backend
    beater-store-memory/  # Sample — in-memory TraceStore/MetadataStore/QuotaLimiter for tests/dev
    beater-store-sql/     # Vinyl — SQLite stores (runtime default) PLUS PgTraceStore and
                          #   ClickHouseTraceStore (implemented, NOT yet runtime-wired);
                          #   ClickHouse lives here, not in a beater-store-ch crate
    beater-store-obj/     # Crate — FsArtifactStore (filesystem) for artifacts/raw envelopes
    beater-bus/           # Drumbeat — SqliteDurableBus (the durable bus today); NATS/Kafka are [planned]
    beater-eval/          # Backbeat (with beater-judge/-stats) — evaluator catalog, scoring
                          #   contracts, paired comparison, aggregation
                          #   [CHANGED] hardcoded-z `compare_paired_scores` is DELETED;
                          #   it now delegates to Backbeat/beater-stats (§10.3, §20.5)
    beater-calibration/   # Tuning [CHANGED] judge-vs-human agreement + Cohen's-kappa reports
                          #   PLUS agent/score proper-scoring calibration: Brier, ECE,
                          #   reliability curve, persisted recalibration map (§10.5; kappa
                          #   becomes a secondary signal). Distinct from the §10.1.1 judge
                          #   Wasserstein calibration, which lives in the judge broker.
    beater-usage/         # Tempo — usage ledger, billing meters, spend summaries
    beater-audit/         # Logbook — privileged access audit events and readback
    beater-sandbox/       # Soundproof — Wasmtime/WASI Component Model evaluator runtime
    beater-secrets/       # Stash — opaque provider-secret refs, BYOK metadata, revocation
    beater-security/      # Vault — crypto primitives: Argon2 keys, ChaCha20 envelope, signed webhooks
    beater-judge/         # Backbeat (with beater-eval/-stats) — LLM/embedding judge broker, BYOK, calibration
    beater-replay/        # Rewind [CHANGED] cassettes + deterministic replay PLUS real forked
                          #   replay and earliest-failing-span attribution (§11); the
                          #   current `attribute_failure` first-error heuristic is replaced
    beater-datasets/      # Encore [CHANGED] datasets, versions, examples, trace promotion PLUS a
                          #   seeded-hash Train/Dev/Test `split` on DatasetCase + min-sample
                          #   gate + contamination guard (§5.3, §6.4); bulk promote-from-query
    beater-experiments/   # Beatboxing (with the §21 RSI tools) — candidate-vs-baseline
                          #   comparisons and statistics
    beater-gates/         # Cue [CHANGED] CI/CD gates and policy evaluation; the deploy-gate
                          #   number now comes from Backbeat/beater-stats (real p-value + power +
                          #   FWER/FDR) and a gate accepts only on the frozen Test split (§10.3)
    beater-human/         # Setlist — review queues, annotations, human labels
    beater-search/        # Crate Dig — Tantivy full-text index over spans
    beater-archive/       # Cold Storage — Parquet cold-tier archive (Arrow/DataFusion read path)
    beater-alerts/        # Offbeat — alert evaluation over trace/score signals
    beater-auth/          # Backstage — API keys, JWT/session, RBAC types, audit scopes
    beater-accounts/      # Guestlist — users, password auth, browser sessions, org membership
    beater-oauth/         # Wristband — OAuth 2.1 core: clients, PKCE codes, access/refresh tokens
    beater-oauth-server/  # Door — OAuth 2.1 HTTP surface (wired into Beater/beaterd)
    beater-mcp/           # Beatbox [CHANGED] MCP server exposing every /v1 operation as a tool,
                          #   PLUS composite "recipe" tools, "suggest scorers" advisory, and
                          #   the FOLDED-IN self-improvement loop (§21). stdio transport for
                          #   local + streamable-HTTP/OAuth 2.1 for hosted (§3.2, §20.7)
    beater-browser/       # Liveset — browser-agent observability contract (shared foundation)
    beater-browser-cdp/         # Liveset: DJ Deck — Chrome DevTools Protocol backend
    beater-browser-playwright/  # Liveset: Turntable — Playwright driver backend
    beater-browser-webdriver/   # Liveset: Mixer — WebDriver/fantoccini backend
    beater-browser-capture/     # Liveset: Bootleg — console + network + DOM capture per browser step
    beater-browser-harness/     # Liveset: Roadie — browser-agent run harness
    beater-api/           # Mixing Board [CHANGED] axum routers, OpenAPI, SSE/read APIs PLUS the
                          #   config-driven MAPPING importer boundary (§7) and the bulk
                          #   "promote cases from query" endpoint (§20.4, §21)
    xtask/                # Roadcase — build/regen tasks (regen-spec, regen-semconv, loadgen)
    beater-stats/         # Backbeat (with beater-eval/-judge) [planned, NEW] over `statrs`: real
                          #   p-values, Wilson + bootstrap CIs, paired-t/McNemar/Wilcoxon test
                          #   selection, Holm-Bonferroni + Benjamini-Hochberg, power/MDE gating;
                          #   mSPRT/confidence-sequences are the required online follow-on (§6, §10.3, §20.5)
    beater-scorers/       # Riff [planned] custom-scorer registry over the Soundproof WASI sandbox (§20.5)
    beater-online/        # Backspin [planned] online-eval scoring worker (§20.6)
    beater-prompts/       # Mixdown [planned] prompt registry/versioning/playground (§20.6)
    beater-rbac/          # All-Access [planned] role/permission resolution inside authorize() (§20.7)
    beater-identity/      # Passport [planned] OIDC/SAML/SCIM (§20.7)
    beater-billing/       # Bandwidth [planned] plans/subscriptions/Stripe metered sync (§20.7)
    beater-bench/         # Tech Rider [planned] criterion benches + load fixtures (§20.2)
  bins/
    beaterd/              # Beater — default all-in-one binary (also holds Heartbeat: metrics.rs / Prometheus facade)
    beaterctl/            # Stomp Box — CLI: init, ingest test, eval run, gate, export
    beater-worker/        # [planned] later thin bin over worker modules
    beater-ingestd/       # [planned] later thin bin over ingest modules
  sdks/
    rust/                 # native Rust SDK + tracing layers (a standalone package,
                          #   excluded from the workspace; there is no beater-sdk crate)
    clients/*             # 7 generated SDK clients (py/ts/go/java/c/cpp/...) from the OpenAPI spec
    openapi/, semconv/    # single-source contract artifacts
  api/
    *.rs                  # [planned] Vercel Rust Function entrypoints where needed
  web/
    dashboard/            # Next.js dashboard consuming generated OpenAPI client
  migrations/             # SQLite migrations today; Postgres/ClickHouse migrations [planned] (§20.2 #0.6)
  docker-compose.yml
```

Metrics, health, and SLO instrumentation are NOT a separate `beater-telemetry`
crate; they live in `bins/beaterd/src/metrics.rs` (the Prometheus facade) and
`metrics_http.rs`. The native Rust SDK is `sdks/rust`, intentionally **excluded
from the cargo workspace** (`exclude = ["sdks"]`) so generated and hand-written
SDK packages stay out of the core build/test graph; it is not a `beater-sdk`
workspace crate.

Browser-observability family note: `beater-browser*` is a six-crate family that
turns browser-driving agents into first-class observed agents. `beater-browser`
defines the shared contract; `-cdp`, `-playwright`, and `-webdriver` are
interchangeable driver backends; `-capture` records console, network, and DOM
state per step (perception + economics + timing); `-harness` runs browser-agent
cases. Each browser step normalizes into the same canonical spans (§5.2) so the
eval, replay, and statistics machinery applies unchanged.

The dashboard can use TypeScript/React for product velocity, but all platform
logic, ingestion, storage, eval, replay, API contracts, and SDK primitives remain
Rust-owned.

**Crate deltas from the staff-SWE refactor (summary; details in the cited
sections).** NEW: `beater-stats` (the statistics correctness layer, §10.3).
CHANGED: `beater-calibration` (adds agent/score proper-scoring calibration, §10.5),
`beater-eval` (deletes the hardcoded-z path, §10.3), `beater-datasets` +
`beater-schema` + `beater-store` + `beater-ingest` (Train/Dev/Test split +
`sampling_weight` + weighted aggregates, §5.3/§6.4/§9), `beater-replay` (real
forked replay + earliest-failing-span attribution, §11), `beater-gates` (gate
number sourced from `beater-stats`, §10.3), `beater-api` (mapping importer + bulk
promote, §7/§20.4), `beater-mcp` (composite recipe tools + folded-in improvement
loop, §21). DEFERRED (design-only, ideas preserved, not dropped): full
evolutionary/population search over agent configs; a skill library on a vector
store; and a standalone Studio / toolbelt / credits productization as separate
products (§21). An MVP foundation for the latter (`beater-credits`,
`beater-mcp-improve`) already exists on the `feat/mcp-improve-foundation` branch,
but the architecture now prefers folding improvement into `beater-mcp` and
deferring credits productization (§21.6).

### 4.1 Implementation Picks

The default Rust stack should be boring and production-proven:

| Concern | Pick |
| --- | --- |
| async runtime | `tokio` |
| HTTP API | `axum`, `tower`, `hyper` |
| OTLP gRPC | `tonic`, `prost`, `opentelemetry-proto` |
| Vercel Rust Functions | `vercel_runtime` |
| serialization | `serde`, `serde_json`, `rmp-serde` where useful |
| schema/OpenAPI | `utoipa` |
| metadata DB | SQLite via `rusqlite` today (runtime default); Postgres via `tokio-postgres` for the unwired `PgTraceStore`; `sqlx` is aspirational, not yet adopted |
| ClickHouse | driven over its HTTP interface via `reqwest` (no native driver), in `beater-store-sql` — there is no separate `clickhouse`-crate dependency |
| object storage | `FsArtifactStore` (filesystem) today; `object_store`/S3 is **[planned]**, no dependency yet |
| durable bus | `SqliteDurableBus` today; `async-nats` JetStream / Kafka are **[planned]** with no dependency in-tree |
| Vercel queue adapter | **[planned]** — Vercel Queues HTTP API |
| WASI sandbox | `wasmtime` Component Model |
| cold analytics | `arrow`, `parquet`, `datafusion` |
| statistics | `statrs` (distributions/CDFs for p-values, Wilson, power) in the new `beater-stats`; bootstrap/permutation are hand-rolled over a seeded RNG |
| full-text search | `tantivy` |
| auth/secrets | `argon2`, `jsonwebtoken`, KMS/Vault-compatible traits |
| CLI | `clap` |
| testing | `cargo-nextest`, `testcontainers`, `insta`, `proptest` |
| build/release | `cargo-chef`, multi-stage Docker, `cargo-deny` |

The stack can change when implementation evidence demands it, but each
replacement must preserve the architecture contracts in this document.

## 5. Canonical Data Model

### 5.1 Entity Set

- `Organization`
- `User`
- `Project`
- `Environment`
- `Agent`
- `AgentRelease`
- `Run`
- `Span`
- `Event`
- `Artifact`
- `ReplayCassette`
- `Dataset`
- `DatasetVersion`
- `DatasetCase`
- `Experiment`
- `ExperimentRun`
- `Evaluator`
- `EvaluatorVersion`
- `EvalResult`
- `Gate`
- `GateRun`
- `ReviewQueue`
- `ReviewTask`
- `Annotation`
- `Prompt`
- `PromptVersion`
- `UsageLedger`
- `UsageRecord`
- `ApiKey`
- `WebhookEndpoint`
- `RedactionPolicy`
- `RetentionPolicy`
- `AuditEvent`

### 5.2 Agent Span Taxonomy

These span kinds are canonical, regardless of incoming dialect:

```text
agent.run
agent.turn
agent.plan
agent.step
llm.call
tool.call
mcp.request
retrieval.query
memory.read
memory.write
guardrail.check
human.review
evaluator.run
replay.run
```

Additional provider/framework kinds are retained as original attributes and can
be projected into OpenInference or OTel GenAI exports.

### 5.3 Raw Envelope and Normalized Projection

Every write path stores both forms:

```rust
pub struct RawEnvelope {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub source: SourceDialect,
    pub source_schema_url: Option<String>,
    pub source_schema_version: Option<String>,
    pub received_at: Timestamp,
    pub idempotency_key: IdempotencyKey,
    pub payload_hash: Sha256,
    pub body_ref: ArtifactRef,
    pub auth_context: AuthContext,
}

pub struct CanonicalSpan {
    pub schema_version: u32,
    pub normalizer_version: String,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: EnvironmentId,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    // Cross-span relationships (causal/follows-from, e.g. a tool result feeding a
    // later llm.call, or an agent.run linked to its replay.run). See SpanLink below.
    pub links: Vec<SpanLink>,
    pub seq: u64,
    pub kind: AgentSpanKind,
    pub name: String,
    pub status: SpanStatus,
    // Agent-native grouping (Phase 1, §20.3 #1.1). Populated from session.id /
    // thread.id / user.id + OpenInference session attrs; the conversation/thread
    // cluster id used by §6.3 dim #2/#12 and §10.3 #1 clustered standard errors.
    pub session_id: Option<SessionId>,
    pub thread_id: Option<ThreadId>,
    pub user_id: Option<UserId>,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub model: Option<ModelRef>,
    pub cost: Option<Money>,
    pub tokens: Option<TokenCounts>,
    // Structured role/message/tool-call I/O (Phase 1, §20.3 #1.2). The flat
    // input_ref/output_ref artifacts remain for raw bodies; `messages` is the
    // canonical typed projection read by message-aware evals and the UI.
    pub messages: Option<CanonicalMessages>,
    pub input_ref: Option<ArtifactRef>,
    pub output_ref: Option<ArtifactRef>,
    pub attributes: CanonicalAttrs,
    pub unmapped_attrs: serde_json::Value,
    pub raw_ref: ArtifactRef,
    // Honesty-about-numbers invariant (§1 #9, §9). Inverse-probability weight set
    // by tail-sampling on the keep path: `sampling_weight = 1/keep_probability`.
    // 1.0 for an unsampled (kept-with-certainty) span; >1.0 for a span kept under
    // probabilistic sampling. WEIGHTED roll-ups/aggregates (§13, beater-store)
    // multiply by this so tail-sampled totals are unbiased; a `weighted=false`
    // aggregate path exists only when explicitly labeled biased.
    pub sampling_weight: f64,
}

/// A typed edge between two spans (within or across traces). `links` on
/// CanonicalSpan is a required field — pre-1.0 we add it directly to the canonical
/// type and update every construction site rather than bolting it on as an
/// optional compat shim. An empty `Vec` means "no links", which is the common case.
pub struct SpanLink {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub kind: SpanLinkKind,   // follows_from | caused_by | replays | derived_from
    pub attributes: CanonicalAttrs,
}

/// Structured message I/O (the typed projection of §20.3 #1.2). Defined here so
/// every reference in §6.3 (dimensions), §10 (evals), and §13 (UI) resolves to one
/// type. Multimodal parts (§20.3 #1.3) ride inside `MessageContent`.
pub struct CanonicalMessages {
    pub input: Vec<CanonicalMessage>,
    pub output: Vec<CanonicalMessage>,
}

pub struct CanonicalMessage {
    pub role: MessageRole,                 // system | user | assistant | tool
    pub content: Vec<MessageContent>,      // text, or a MediaArtifact part (image|audio|file, §20.3 #1.3)
    pub tool_calls: Vec<CanonicalToolCall>,// { name, arguments, result_ref } per parsed tool call
}
```

Leaf types: `SessionId`/`ThreadId`/`UserId` are `beater-core` ID newtypes
(like the other typed IDs); `MessageRole`, `SpanLinkKind`, and `MessageContent`
are the inline-enumerated enums shown above; `CanonicalToolCall` and the
`MediaArtifact` content part are the structured forms produced by the Phase 1
normalizer (§20.3 #1.2/#1.3).

Idempotency key:

```text
tenant_id + project_id + trace_id + span_id + seq + payload_hash
```

Late spans are accepted. Out-of-order writes are normal. Trace completeness is a
state machine, not a boolean.

**Pre-1.0 schema-evolution stance.** Beater is not deployed anywhere yet, so the
canonical and API schemas evolve *freely*: a change bumps `schema_version`
(`CANONICAL_SCHEMA_VERSION` / `RAW_SCHEMA_VERSION` in `beater-schema`) and
re-normalizes stored raw envelopes into the new canonical version (`xtask
renormalize`, §20.2 #0.6). **No wire/SDK backward-compatibility is promised before
1.0.** When a canonical type needs a new field we add it to the type directly and
update every construction site — we do *not* accrete `#[serde(default)]` /
optional-for-compat shims to avoid a workspace edit; the canonical model is kept
clean. This is *only* about the normalized/canonical/API schemas. The
**immutable-RAW-envelope guarantee (§1 principle 3) is untouched**: raw bodies,
source schema URL/version, payload hash, and normalizer version are preserved
forever, which is precisely what makes free re-normalization safe — the lossless
source is never destroyed. The single-source-of-truth contract regen
(spec → 7 SDKs → MCP → CLI → docs, `CLAUDE.md`) still runs on every contract change;
dropping wire compat does **not** drop the regen discipline.

### 5.4 DatasetCase Train/Dev/Test split (held-out discipline)

Every `DatasetCase` carries a `split` tag. This is the schema-level foundation of
the held-out-generalization invariant (§1 #12) consumed by the agent model (§6.4)
and the RSI loop (§21):

```rust
pub enum DatasetSplit { Train, Dev, Test }

pub struct DatasetCase {
    // ... existing fields (id, input/expected, artifact hashes, code/wasm hash) ...
    /// Assigned by a SEEDED hash so the split is stable and reproducible across
    /// re-versioning: split = bucket( hash(dataset_version_seed ++ case_id) ).
    /// Default proportions ~Train 0.7 / Dev 0.15 / Test 0.15 are policy, not law.
    pub split: DatasetSplit,
}
```

Rules (pre-1.0, designed cleanly — `split` is a required field, no compat shim):

- **Seeded, stable assignment.** The split is a deterministic function of a
  per-dataset-version seed and the case id, so it does not churn when cases are
  added and is reproducible from the version alone. New cases hash into a split
  without re-shuffling existing ones.
- **Min-sample gate.** A dataset version is usable for an accept/reject decision
  only when each consumed split clears a minimum size (ties into the §10.3 #5 power
  check — too few Test cases ⇒ *inconclusive*, never *pass*).
- **Contamination guard.** Near-duplicate detection prevents a Test case (or a
  near-dup) from leaking into Train, into few-shot exemplars, into memory, or into
  tool fixtures. A suspected-compromised Test split is rotated/refreshed.
- **Who reads what.** Propose/simulate (§21) read **Train** (and may tune on
  **Dev**); acceptance gates (§6.4, §10.3, §12, §21) read the **untouched Test**
  split only. This is the single mechanism that makes the RSI objective `J(π)`
  (§6.2) overfit-resistant.

`DatasetVersion` is the unit the split seed is pinned to, so an `ExperimentRun` or
gate decision can name exactly which frozen split it scored against.

## 6. The Agent Model (the object under evaluation)

Everything else in this document — ingest, storage, evals, replay, statistics,
the RSI loop (§21) — exists to **measure and improve one thing: an agent.** This
section formalizes that agent from first principles as a statistical object, so a
developer iterating on an agent and the RSI loop both have a precise target. It is
a measurement view over §5, not a second storage schema: the agent is *projected
onto* the canonical entities and spans of §5.

### 6.1 An agent is a policy; a run is a sampled trajectory

Model the agent under evaluation as a **policy** `π` — a (usually stochastic)
mapping from context to actions. Executing `π` once on an input produces a
**trajectory** `τ`: an ordered sequence of canonical spans

```text
τ = [ agent.plan, agent.step, llm.call, tool.call, retrieval.query,
      memory.read, memory.write, guardrail.check, ... ]
```

i.e. exactly the §5.2 taxonomy. A `Run` (§5.1) is one realized sample
`τ ~ π(· | case)` for a `DatasetCase`; an `ExperimentRun` is a batch of such
samples for a fixed `(π, dataset version)`. Because `π` is stochastic, **a single
run is one draw from a distribution** — never the agent. Any honest claim about an
agent is a claim about the *distribution of τ*, which is why §10.3's N-trial
repetition and standard errors are not optional polish but the definition of
measuring `π` at all.

`π` is not monolithic. Its **mutable components** are the levers the platform and
the RSI loop can change:

```text
π = f( system_prompt, customer/user_prompt, code, tool_set,
       memory_config, model_params )
```

These map one-to-one onto the agent-mutating variants of §21.1's `ChangeKind`
(`SystemPrompt`, `CustomerPrompt`, `Code`, `ToolAdd`/`ToolRemove` for `tool_set`,
`MemoryConfig`, `ModelParams`). `ChangeKind::DataLabel` is the one variant that is
*not* a lever of `π` — it challenges a dataset label (dim #16, §6.3), never the
agent itself.

### 6.2 RSI as constrained optimization over π

Recursive self-improvement (§21) is, formally, a constrained optimization:

```text
maximize    J(π)        = E_{case ~ D_test, τ ~ π}[ objective(τ, case) ]  // frozen Test split (§5.4)
over        the mutable components of π  (§6.1)
subject to  policy constraints C  (load-bearing prompts/tools unchanged unless
                                    contradictory; safety/guardrail invariants)
```

where `J(π)` is estimated on a **held-out** objective, never the data the loop
proposed against. Two anti-Goodhart invariants make the optimization honest:

- **The evaluator is frozen during an optimization episode.** The judge model,
  rubric (locked JSON, §10.1.1), deterministic scorers, *and the dataset split* do
  not change while a loop is improving `π`. If the ruler can move, the loop
  optimizes the ruler, not the agent.
- **Propose/simulate read TRAIN; acceptance reads untouched HOLDOUT.** See §6.4.

**Convergence criteria** (the loop stops, rather than churning): no proposed
change clears the §10.3 confidence-bound *and* power bar on holdout (the gain is
indistinguishable from noise or underpowered); or a fixed episode budget
(iterations / AI-credits, §21.6) is exhausted; or every remaining candidate
touches a policy-constrained component. A change is **accepted only** when its
holdout improvement is statistically significant under §10.3 *and* does not regress
any guardrail/safety dimension below threshold.

### 6.3 Measurable agent dimensions

An agent is not a scalar. Beater measures it along many **typed dimensions**, each
a metric with: a **definition**, an **estimator** (point + the CI method from
§10.3), the **assumptions** that estimator needs, and **where it attaches** in the
canonical schema (§5). "Attaches" names the span/entity the evidence is read from.

| # | Dimension | Definition (point estimate) | Estimator + CI (§10.3) | Assumptions | Attaches to |
| --- | --- | --- | --- | --- | --- |
| 1 | **Task success (outcome)** | P(final output meets the case's success criterion) | proportion; **Wilson**, clustered if multi-turn | a checkable success criterion per case | `agent.run` outcome vs `DatasetCase.expected` |
| 2 | **Trajectory / process quality** | joint promise+progress score over the step sequence (NOT a mean of independent per-step scores) [arXiv:2511.08325; arXiv:2507.21504] | process-reward score; **bootstrap, trajectory-clustered SE** | steps within a trajectory are correlated (so: cluster) | `agent.plan`/`agent.step` chain |
| 3 | **Tool-call correctness** | fraction of tool calls that, *executed*, produce the correct effect (EXECUTION-based, not AST/syntax) | per-call binary → **Wilson**; per-trajectory clustered | a seeded/replayable tool environment | `tool.call`/`mcp.request` spans |
| 4 | **Planning / decomposition quality** | does the plan cover the sub-goals with no redundant/missing steps | rubric judge or structural check; bootstrap | a reference decomposition or rubric | `agent.plan` span |
| 5 | **Reasoning faithfulness** | does the stated reasoning actually entail the action/answer | judge (faithfulness); calibrated → bootstrap | judge calibration valid (§10.1.1) | `llm.call` reasoning vs `output_ref` |
| 6 | **Instruction / policy adherence** | fraction of explicit constraints obeyed | per-constraint binary → **Wilson** | constraints are enumerable & checkable | `guardrail.check`, system_prompt vs trajectory |
| 7 | **Self-calibration** | agreement between stated confidence and actual correctness | **Brier score** + **ECE** + reliability curve — proper scoring rules (§10.5); bootstrap CI | the agent emits a confidence/probability | confidence attr on `llm.call` vs outcome (#1) |
| 8 | **Robustness (distribution shift / adversarial)** | success on perturbed/adversarial inputs vs clean | paired delta clean→shifted; **paired test (§10.3 #3)** | a defined perturbation/adversarial set | run pairs over original vs perturbed case |
| 9 | **Cost** | spend per successful task (and per run) | mean/quantiles; **bootstrap** (skewed) | cost field populated & trustworthy | `cost` on `llm.call`/`tool.call`, rolled to run |
| 10 | **Latency** | wall-clock per run / per step | p50/p95/p99; **bootstrap** | clock-skew corrected (§9) | span `start/end_time` |
| 11 | **Token efficiency** | tokens (or tokens/success) per task | mean/quantiles; bootstrap | token counts populated | `tokens` on `llm.call` |
| 12 | **Reliability / variance** | run-to-run outcome variance at fixed input (N-trial) | variance / success-rate spread across N draws; bootstrap | repeated draws are exchangeable | N `Run`s of the same case |
| 13 | **Safety / guardrail conformance** | rate of guardrail violations (jailbreak, PII leak, unsafe action) | proportion; **Wilson** (one-sided, conservative) | violation is detectable by a check/judge | `guardrail.check` spans + output scans |
| 14 | **Memory / retrieval quality** | did retrieval surface the relevant context; was memory written/read correctly | retrieval relevance (judge) + write/read consistency (deterministic) | a relevance label or reference | `retrieval.query`, `memory.read/write` |
| 15 | **Generalization** | holdout success − train success (the gap) | paired/Δ with CI; flag if gap CI excludes 0 | a genuine train/holdout split (§6.4) | runs partitioned by split |
| 16 | **Data-label trust** | fraction of dataset labels the evidence contradicts (challenged labels) | proportion of disputed labels; Wilson | labels are independently checkable | `DatasetCase` vs human review (§10.1.1, §21.1 `challenge_labels`) |

Every dimension is scored by a §10.4 grading algorithm and aggregated by §10.3.
Dimensions are not collapsed into one number by default: an agent that is cheaper
but less safe is *worse* on the safety axis, and the gate (§10.3, §12) can veto on
any single axis. This is the multi-comparison setting of §10.3 #4 — improving 16
dimensions at once *requires* FWER/FDR control or the loop will manufacture false
wins.

### 6.4 Anti-overfit / generalization discipline for RSI

Because the RSI loop actively searches over `π`, it is a textbook overfitting
risk: given enough proposals it *will* find a change that beats a fixed dataset by
chance. The discipline that prevents this is mandatory, not advisory:

- **Train/Dev/Test split on `DatasetCase`.** Every dataset version carries a
  stable, seeded-hash split (the schema-level definition is §5.4). Propose/simulate
  steps (§21.1 `propose_change`, `simulate`) read **Train** and may tune on **Dev**;
  the **Test** split is the held-out judge. ("Holdout" throughout this document
  means the frozen **Test** split.)
- **Acceptance gates run on the untouched Test split.** A change is accepted only
  on Test evidence that clears §10.3's significance *and* power bars. The Test split
  is never shown to the proposal/Dev-tuning steps in the same episode.
- **Contamination controls.** Prevent leakage of holdout cases (or near-duplicates)
  into prompts, few-shot exemplars, memory, or tool fixtures; detect near-dup
  overlap between train and holdout; rotate/refresh holdout if it is suspected
  compromised.
- **Freeze the evaluator during an episode** (§6.2): judge model, rubric,
  deterministic scorers, and split are pinned for the whole optimization episode,
  so the measured gain is attributable to `π`, not to a moved ruler.

### 6.5 Modeling assumptions (stated, checked, relaxed)

The agent model rests on assumptions; naming them is what separates measurement
from wishful thinking. For each, how Beater checks or relaxes it:

- **Independence vs clustering.** Default analyses assume i.i.d. cases. This is
  *violated* for multi-turn conversations and shared prompt templates — handled by
  **clustered standard errors** (§10.3 #1). Checked by: declaring a cluster id on
  every case; relaxed by coarsening clusters when they are themselves correlated.
- **Stationarity.** Estimates assume the agent, judge, and providers are stable
  over the measurement window. *Violated* by model deprecation/provider drift —
  handled by recalibration triggers (§10.1.1, §10.3) and by freezing the evaluator
  within an episode (§6.2). Checked by: re-running a fixed canary set over time
  and watching for kappa/score drift.
- **Judge-calibration validity.** Judge-derived dimensions assume the §10.1.1
  distributional calibration still holds. *Violated* when the human reference set
  is stale or too small (the open questions flagged in §10.1.1). Checked by:
  periodic judge-vs-human agreement (`beater-calibration`, Cohen's kappa);
  relaxed by re-fitting `F_human`/`F_model`.
- **Sampling / representativeness.** `J(π)` generalizes only if the dataset is a
  representative sample of the deployment distribution. *Violated* by a biased or
  tiny dataset — handled by power/MDE planning (§10.3 #5, refuse underpowered),
  generalization-gap monitoring (dim #15), and online evals (§20.6) that compare
  offline estimates against production score distributions.

The payoff: a developer can read off *exactly* which dimension regressed, with a
real interval and a stated assumption, and the RSI loop (§21) has a precise,
overfit-resistant objective `J(π)` to optimize against rather than a single fragile
score. These four assumptions, plus every other quantitative assumption in this
document, are gathered with their checks and enforcing tests in the consolidated
table at §21.9 (rows A1, A9, A10, A12).

## 7. Standards and Normalization

Input dialects:

- OTLP traces over gRPC and HTTP.
- OpenInference attributes and span kinds.
- OpenTelemetry GenAI conventions.
- Vercel AI SDK telemetry shapes.
- OpenLLMetry/Traceloop-compatible attributes.
- Native Beater `/v1` JSON ingest.
- Future imports from Phoenix, LangSmith, Langfuse, and Braintrust exports.

**The normalizer algorithm (dialect → canonical projection).** Each dialect is a
**deterministic projection**, not parsing-by-guess: a static **mapping table** keyed
by `(dialect, source_span_kind)` → canonical span kind (§5.2), plus an
**attribute-rename map** `source_attr → canonical_attr`, plus typed **unit/timestamp
coercions** (epoch-ns vs RFC3339; tokens/cost field names; ms vs s). The pipeline per
span is: (1) detect the dialect (explicit `?source=` or signature attributes); (2)
look up the span-kind mapping (unknown kinds map to `agent.step` with the raw kind
preserved as an attribute, never dropped); (3) apply the attribute-rename map and
coerce units/timestamps; (4) emit the canonical span tagged with the **pinned
normalizer version** (§10.2). The mapping is **table-driven** so adding/auditing a
dialect is data, and every projection is re-derivable from the immutable raw envelope
(§1 #3). The hard-coded normalizers (OTLP/OpenInference/GenAI/Vercel-AI) are
hand-written tables on this same shape; the declarative MAPPING importer below is the
same table supplied as config instead of code.

Config-driven mapping importer (`SourceImporter` boundary). The hand-written
normalizers above (OTLP/OpenInference/GenAI/Vercel-AI) cover the standard dialects,
but a long tail of custom and *older* framework shapes will never get a bespoke Rust
normalizer. For those, Beater exposes a **declarative MAPPING importer** on the
`SourceImporter` trait boundary: a user supplies a config (field-path mapping, span-
kind mapping, attribute renames, timestamp/units coercion) — **no code** — that
projects a foreign dialect into the canonical model (§5.2). The hard-coded
normalizers remain the fast path; the mapping importer is the escape hatch that
makes "bring your weird exporter" a config task, not a PR. It rides the
single-source contract (the `/v1` import endpoint is **[contract]**, §20.4) and,
like every other importer, preserves the immutable raw envelope (§1 #3) so a
mis-configured mapping is always re-projectable.

Output dialects:

- OTLP export.
- OpenInference-compatible export.
- Parquet export.
- JSONL dataset/eval export.
- Post-training export for SFT/RL pipelines.

OpenTelemetry GenAI note:

- The old OpenTelemetry docs now point to the standalone
  `open-telemetry/semantic-conventions-genai` repository.
- The repository currently contains generated docs/model definitions for GenAI
  clients, MCP, events, spans, metrics, and provider-specific conventions.
- Pin by commit or generated schema snapshot. Do not assume stability. The
  normalizer must support both old and new forms.

Source links:

- https://opentelemetry.io/docs/specs/semconv/gen-ai/
- https://github.com/open-telemetry/semantic-conventions-genai

## 8. Storage Architecture

### 8.1 Trait Boundary

`TraceStore` exists before any ClickHouse-specific behavior leaks into product
code.

```rust
#[async_trait::async_trait]
pub trait TraceStore: Send + Sync {
    async fn write_batch(&self, batch: CanonicalTraceBatch) -> anyhow::Result<WriteAck>;
    async fn get_trace(
        &self,
        tenant: TenantId,
        trace: TraceId,
    ) -> anyhow::Result<TraceView>;
    async fn query_runs(
        &self,
        tenant: TenantId,
        filter: RunFilter,
        page: PageRequest,
    ) -> anyhow::Result<Page<RunSummary>>;
    async fn query_spans(
        &self,
        tenant: TenantId,
        filter: SpanFilter,
        page: PageRequest,
    ) -> StoreResult<Page<SpanSummary>>;
}
```

Backends (status as of `origin/main`, 2026-06-27):

- `SqliteTraceStore` in `beater-store-sql`: **[built, runtime default]** — the
  only `TraceStore` `beaterd` actually constructs today.
- `InMemoryTraceStore` in `beater-store-memory`: **[built]** — used by tests and
  for ephemeral dev.
- `PgTraceStore` (Postgres, `tokio-postgres`) and `ClickHouseTraceStore`
  (ClickHouse over HTTP via `reqwest`), both in `beater-store-sql`:
  **[built but NOT runtime-wired]** — the types and trait impls exist and pass
  the `beater-store-conformance` suite, but `beaterd` has no backend selector,
  so neither is reachable from the running service yet (§20.2 #0.1).
- `ParquetTraceArchive` in `beater-archive`: **[built, local-fs only]** — cold
  tier with an Arrow/DataFusion read path; not yet writing to object storage or
  scheduled (§20.2 #0.5).

Every backend is validated against one shared trait-conformance test suite in
`beater-store-conformance`, so a newly wired backend must satisfy the same
contract before it can be selected. Product code depends on `TraceStore`, not
concrete backend crates.

SQLite and memory stores may use
`beater_store::query_runs_by_materializing_spans` as a dev/local fallback. That
helper intentionally reads matching spans and rolls up run summaries in Rust.
ClickHouse or any hosted hot-store backend must not use that fallback for normal
paths; it must aggregate run summaries, run-level filters, and pagination in the
backend over tenant-leading sort keys.

### 8.2 Data Planes

The table is the target topology. The **Built today** column states what
`origin/main` actually runs; "→" marks the planned migration the trait boundary
is designed to absorb without product-code changes.

| Plane | Built today (OSS) | Target OSS / Hosted scale | Purpose |
| --- | --- | --- | --- |
| Metadata | SQLite (`SqliteMetadataStore`) | → Postgres | orgs, projects, prompts, datasets, RBAC, billing metadata |
| Hot traces | SQLite (`SqliteTraceStore`) | → Postgres/SQLite dev, ClickHouse for scale (impls exist, unwired) | runs, spans, events, scores, indexed attrs |
| Raw/artifacts | filesystem (`FsArtifactStore`) | → MinIO/S3, S3/R2/GCS/Vercel Blob | raw envelopes, payloads, cassettes, exports |
| Durable bus | `SqliteDurableBus` | → NATS JetStream / Vercel Queues at edge / Kafka in cells | ingest buffering, eval jobs, replay jobs |
| Cold traces | Parquet + Arrow/DataFusion, local-fs (`beater-archive`) | → Parquet on object store, scheduled demotion | long retention and export |
| Full text | Tantivy (`beater-search`) | → Tantivy or managed equivalent | prompt/output/error search |

Redis is optional cache/pubsub, not the default durability primitive — and is
not in-tree today. The §8.3 ClickHouse rules and §3.2 "Managed data" describe the
hosted target, not the current runtime.

### 8.3 ClickHouse Rules

- Tenant ID leads sort keys and all query filters.
- Updates are modeled as new events or versioned replacing rows.
- Object storage holds large inputs, outputs, attachments, raw payloads, and
  cassettes. ClickHouse stores refs, hashes, sizes, MIME types, and redaction
  classifications.
- TTL moves hot rows to cold Parquet before deletion.
- Query APIs must not require `FINAL` for normal paths.

### 8.4 Queue and Job Lanes

Do not collapse every async concern into one queue. The platform has distinct
lanes with different guarantees:

| Lane | OSS default | Hosted default | Required semantics |
| --- | --- | --- | --- |
| in-process smoothing | bounded Tokio channels | bounded Tokio channels | absorbs short spikes, rejects when full |
| ingest durability | NATS JetStream | Vercel Queues at edge, NATS/Kafka in cells | at-least-once, idempotency keys, replay until retention |
| background jobs | NATS JetStream or Postgres-backed scheduler | NATS/Kafka/cell scheduler | retries, backoff, poison-message isolation |
| DLQ | NATS stream plus object refs | regional DLQ stream plus object refs | reason codes, replay tooling, alerts |
| cache/pubsub | optional Redis or in-process | Redis/managed cache where needed | never source of durability |
| enterprise bus | Kafka adapter | Kafka adapter | large customer integration and audit needs |

The current OSS all-in-one slice uses the same lane model on the SQLite durable
bus. `?durability=buffered` on native or OTLP ingest writes a canonical
`trace.write_batch` message before hot trace persistence. A scoped drain API and
the `beaterd` background worker consume only that lane, write through
`TraceStore`, publish downstream `trace.ingested` work, and move invalid or
exhausted messages to DLQ without consuming other tenants' queued work. Hosted
deployments replace the SQLite bus implementation with Vercel Queues at the edge
and NATS/Kafka in worker cells without changing the ingest contract.

Poison messages are messages that repeatedly fail for deterministic reasons.
They must be moved to DLQ with a reason, source envelope ref, attempt history,
and replay command. They must not block a partition or consumer group forever.

## 9. Ingest Pipeline

```text
receive OTLP/native request
  -> authenticate API key and project/environment
  -> enforce per-project quotas and payload limits
  -> create RawEnvelope and artifact refs
  -> normalize with pinned normalizer version
  -> enforce cardinality/payload governance
  -> buffer for tail-sampling and trace completion
  -> on keep, stamp sampling_weight = 1/keep_probability (§1 #9)
  -> direct mode: write canonical projection through TraceStore
  -> buffered mode: enqueue canonical trace.write_batch for the drain worker
  -> enqueue online eval/replay/alert jobs
  -> acknowledge or DLQ
```

Required survivability behavior:

- Backpressure with bounded queues.
- **At-least-once delivery reconciled by idempotency keys (the dedup algorithm).**
  Each ingest unit carries an **idempotency key** = a stable content hash of its
  identity — `blake3(tenant_id ‖ trace_id ‖ span_id ‖ canonical-payload-hash)`,
  or a client-supplied `Idempotency-Key` when present. Dedup is an
  **existence-check-then-insert**: the write path does an atomic
  insert-if-absent on that key (a `UNIQUE` constraint / `INSERT … ON CONFLICT DO
  NOTHING` in SQL backends, the natural primary key in ClickHouse's
  versioned-replacing model, §8.3); a key already present is acknowledged as a
  no-op, never written twice. Because the key is a deterministic content hash, a
  retried or fanned-out delivery (at-least-once bus, §8.4) collapses to **exactly
  once** in storage without coordination. This is what makes "no silent drops" and
  "at-least-once" coexist: redelivery is safe and observable.
- Dead-letter queue for invalid, unauthenticated, unnormalizable, or repeatedly
  unwritable events.
- Poison-message isolation so one bad tenant payload cannot stall a shard.
- No silent drops.
- Per-project ingest quotas with explicit 429 semantics.
- Payload size caps. Oversized prompts/completions truncate to artifact refs.
- Per-attribute cardinality budgets.
- Attribute allow/deny lists at project and environment scope.
- **Tail-based sampling — the decision algorithm.** Decision is made once the
  trace is buffered to completion (so the whole trace is visible), in priority
  order: **keep with probability 1** if the trace has any error span, exceeds the
  latency threshold, exceeds the cost threshold, or matches a policy keep-rule;
  **otherwise keep routine traffic with probability `p`** by a *deterministic stable
  hash* — `keep ⇔ (xxhash64(trace_id) mod 2⁶⁴) / 2⁶⁴ < p`. Hashing the immutable
  `trace_id` (rather than drawing a fresh random number) makes the decision
  **deterministic, stateless, and consistent across shards/retries**: the same trace
  always gets the same keep verdict, so retried or fanned-out ingest never
  double-counts and the kept set is reproducible. `p` is the per-project
  routine-sampling rate.
- **Inverse-probability sampling weights on the keep path (honesty invariant §1
  #9).** Every kept span records `sampling_weight = 1/keep_probability`: 1.0 for a
  span kept with certainty (errors/slow/high-cost/policy keeps), and `1/p` for a
  span kept under probabilistic routine-traffic sampling at rate `p`. Without this,
  any roll-up over a tail-sampled population is *biased* — routine traffic is
  systematically under-counted and error/cost rates are inflated. The keep
  decision and `p` are known at sampling time, so the weight is recorded then;
  downstream aggregates (§13, `beater-store`) are **weighted by default** and an
  unweighted path must be explicitly labeled biased. This is a correctness fix, not
  an analytics nicety. The estimator is **Horvitz-Thompson**: a population total is
  `T̂ = Σ_{kept i} wᵢ·yᵢ` (`wᵢ = 1/pᵢ`) and a population mean is
  `μ̂ = (Σ wᵢ·yᵢ)/(Σ wᵢ)`; under independent keep decisions its variance is
  `Var(T̂) = Σ_i (1−pᵢ)/pᵢ² · yᵢ²`, which `beater-stats` reports as the weighted
  standard error and which the weighted bootstrap (§10.3 #2, resampling in
  proportion to `wᵢ`) recovers without the closed form. This is the §10.3 "sampling
  weights flow into the estimators" path.
- Trace completion semantics based on root-span end, idle timeout, and late-span
  window.
- Clock-skew correction and out-of-order handling across distributed agents.

Trace states:

```text
open
root_ended
idle_complete
late_window_closed
complete
incomplete
```

Online evals trigger only when policy says the trace is complete enough.

## 10. Evaluator Architecture

### 10.1 Execution Lanes

Deterministic lane:

- Runtime: Wasmtime using the WASI Component Model.
- Network: none.
- Inputs: trace/case data, expected outputs, evaluator config, artifact reads
  through explicit host functions.
- Outputs: structured score, labels, evidence refs, diagnostics.
- Examples: exact match, regex, JSON schema, tool-call correctness,
  trajectory-structure checks, latency budgets, cost budgets, token budgets,
  loop detection, citation presence, PII-pattern checks.

Judge lane:

- Runtime: `judge-broker` Rust service/module.
- Network: broker only, never evaluator WASM.
- Secrets: provider keys held behind opaque `ProviderSecretId` refs. The OSS
  SQLite implementation stores ChaCha20-Poly1305 ciphertext with tenant/project/
  secret/provider associated data; hosted cells can swap the same trait for KMS
  or customer vault unwrap. API responses and ledgers expose metadata only,
  never secret material.
- Controls: redaction, preflight budget reservation, rate limit, model pinning,
  retries, request-hash caching, and append-only audit logging.
- Usage: judge calls emit idempotent `UsageRecord`s keyed by tenant/project,
  meter, source kind, and source ID. Billing summaries use charged cost, while
  cached calls remain visible as zero-cost audit records.
- Provider clients: OpenAI-compatible chat completions and Anthropic messages
  are first-class Rust judge providers behind the same broker trait, with
  retry/backoff for `429` and `5xx` responses and structured JSON score parsing.
- Dataset execution: dataset-version evals support both deterministic WASI
  scorers and judge-broker scorers. Judge-backed reports persist through the
  same `DatasetEvalReport`/`EvalResult` schema, with model/provider/request
  hashes, cache status, and charged cost pinned in reproducibility metadata.
- Experiment execution: candidate-vs-baseline gates support judge-broker
  scorers as well as deterministic scorers. Per-case experiment scores retain
  judge call IDs, charged costs, and cache status so noisy/model-dependent gates
  remain auditable. Experiment reports also retain the `GatePolicy` that
  produced the stored pass/fail/inconclusive decision.
- Examples: faithfulness, pairwise judge, trajectory quality, retrieval
  relevance, handoff quality, rubric grading, semantic safety checks.

#### 10.1.1 Judge Reliability & Debiasing Protocol

An LLM judge is itself a noisy, biased measurement instrument. Treating its raw
score as ground truth is the single most common way an eval platform silently
lies. Beater's judge broker therefore implements a debiasing protocol as a
first-class part of the lane, not an optional add-on. The protocol below is the
*default recipe*; every clause is grounded in the literature and several
magnitudes come from 2026 preprints that have not been independently replicated —
those are flagged **[directional]** and the magnitude, not the direction, is what
should be treated as uncertain.

**The biases are real, model-dependent, and must be mitigated — not assumed
away.** Position bias, verbosity/length bias, and self-preference bias all
persist in current frontier judges and vary by model:

- *Self-preference* (a judge scoring its own family's outputs higher) is not a
  fixed constant: across a 20-model study the self-preference coefficient ranges
  from roughly **+0.307 to −0.229**, i.e. some models actively *dis*-prefer their
  own outputs. Crucially, **higher capability does not guarantee fairness** — the
  most capable judge is not automatically the least biased
  [arXiv:2404.18796; arXiv:2410.21819].
- *Position bias* (preferring the first- or second-presented answer) and
  *verbosity bias* (rewarding length irrespective of quality) are likewise
  present and model-dependent [arXiv:2411.15594].

Because the magnitude and even the *sign* are model-specific, Beater never hard-
codes a bias correction; it measures bias per judge model on the calibration set
(§10.1.1 calibration) and applies mitigation structurally.

**Default single-judge recipe (the broker's out-of-the-box judge):**

1. **Merged chain-of-thought + a LOCKED JSON rubric.** The judge reasons step by
   step *before* emitting a structured score against a rubric whose criteria,
   weights, and scale are frozen for the duration of an eval/optimization episode.
   CoT is the strongest single mitigation on adversarial data
   [arXiv:2604.23178] **[directional]**.
2. **Position-swap ON TOP OF CoT+rubric only.** Each pairwise comparison is run in
   both A/B orders and reconciled. *Position-swap applied alone can HURT accuracy*
   — it is only safe layered on top of CoT+rubric, so the broker refuses to enable
   swap without them [arXiv:2604.23178] **[directional]**.
3. **Mid-tier judge model (~$0.001/eval).** A locked, mid-tier judge is the
   default; capability beyond mid-tier buys little fairness (see above) at large
   cost.

**Distributional calibration is the single biggest accuracy lever.** Raw judge
scores are mapped to a human-anchored distribution by Wasserstein quantile-
matching:

```text
g(z) = F_human^{-1}( F_model(z) )
```

where `F_model` is the empirical CDF of the judge's raw scores and `F_human` the
empirical CDF of human reference labels. Removing this calibration step collapses
judge-human agreement — reported quadratic-weighted kappa falling from **0.73 to
0.26** when calibration is dropped [arXiv:2601.08654, "Rulers"] **[directional]**.
In Beater this calibration **lives in the judge broker** (alongside the existing
`beater-calibration` agreement/kappa reporting), is fit from a **human reference-
label set**, and is pinned into `EvalResult` reproducibility metadata so a score's
calibration provenance is auditable.

**Procedure + parameters.** `F_model` and `F_human` are **empirical CDFs** over the
calibration set; `g` is their composition, evaluated by linear interpolation between
the empirical quantiles (monotone non-decreasing by construction). Suggested
operating points, to be confirmed by Beater's own measurement (below): a **minimum
of ~200 paired human reference labels** for a stable `F_human` (the empirical
quantiles are too coarse below that — fall back to the raw judge score and flag
"uncalibrated"), and **recalibration cadence is event-driven** — re-fit on any
§10.3 trigger (model deprecation, provider/judge drift, rubric change) and otherwise
whenever judge-vs-human kappa on a fixed canary set (A9) drops past a configured
threshold, rather than on a fixed schedule. These two numbers (min label count,
cadence) remain **open questions to confirm empirically before treating calibration
as load-bearing in a production gate** — they should be measured against Beater's own
reference set, not hard-coded from a preprint.

**Ensemble policy — small calibrated panels, NOT large ones.** A small calibrated
panel of ~3 diverse *smaller* judges (the "Panel of LLM evaluators", PoLL) can
beat a single large judge at **>7× lower cost** [arXiv:2404.18796]. But the gain
saturates fast because **judge errors are strongly correlated**: an analysis of a
~9-judge panel found an *effective* sample size of only **≈2.18 independent
votes**, and **model-family diversity does NOT restore independence**
[arXiv:2605.29800] **[directional]**. The design consequence is explicit: **do
not build large panels.** Prefer a small panel (≈3) and spend the diversity budget
on **decorrelated prompts/rubrics** rather than more models.

**Per-dimension forced-choice decomposition.** Decomposing a holistic judgment
into per-dimension forced-choice comparisons reduces self-preference bias by
about **31%** [arXiv:2604.22891] **[directional]**. The structured-rubric judge
(§20.5 #3.2) emits `per_criterion` scores precisely so this decomposition is the
default shape, not a special case.

**Refuted assumptions — do NOT design around these.** Two intuitions that older
eval folklore relied on were measured to fail and Beater must not assume either:

- *"Pairwise comparison is strictly better than pointwise scoring"* — **refuted**
  (held in 0 of 3 tested settings). Beater treats pairwise vs pointwise as an
  empirical, per-task choice, not a default.
- *"Position bias is negligible in modern judges"* — **refuted** (held in 0 of 3
  settings). Position mitigation stays mandatory.

**Honesty caveat.** Several single-recipe magnitudes above (the CoT/position-swap
interaction, the QWK 0.73→0.26 calibration collapse, the ≈2.18 effective votes,
the 31% self-preference reduction) come from **unreplicated 2026 preprints**.
Treat the *directions* as well-supported and the *magnitudes* as directional;
Beater's own calibration reports (§10.1.1, §10.3) are the source of truth for any
gate, not these published numbers.

### 10.2 EvalResult Reproducibility Contract

Every `EvalResult` pins:

- dataset version
- dataset case ID
- candidate agent release
- prompt version
- evaluator ID and version
- evaluator code hash or WASM hash
- WASI ABI version if deterministic
- judge model ID, provider, parameters, seed when available
- judge prompt/rubric version
- normalizer version
- trace schema version
- input artifact hashes
- execution timestamp
- cost and token usage
- reason if an old result cannot be reproduced

### 10.3 Statistical Rigor

Every eval is an **experiment**, and the platform must report it like one:
standard errors, not bare point estimates, and a decision rule that knows its own
assumptions.

**This is a correctness fix, not an enhancement — today's deploy-gate number is
wrong.** The current statistics are a single hand-rolled normal-approximation:
`compare_paired_scores` in `beater-eval` computes a paired delta, a sample
variance, a standard error, and then a **hard-coded** Wald interval with
`z = if adjusted_alpha <= 0.01 { 2.576 } else { 1.96 }` and a crude
`alpha / comparison_count` Bonferroni split, with **no real p-value**. The
consequence is not "less precise" — it is **nominal alpha ≠ actual alpha**: for the
binary, small-N, clustered, multi-metric situations the platform actually faces, a
Wald interval's true coverage is *not* its stated coverage, so a gate set to
"reject regressions at 5%" does not actually hold a 5% error rate. The number the
CI gate blocks or ships on is therefore **wrong**, and silently so. **The
hard-coded-`z` line and the `StatisticalTest::PairedNormalApproximation` path are
DELETED**, not retained as a fallback; `compare_paired_scores` is replaced by a
call into `beater-stats` that selects the correct test (below). This restores the
§1 #9/#11 invariant that nominal alpha equals actual alpha.

This subsection specifies the replacement statistics layer as a concrete,
assumption-aware algorithm spec. It lives in a new **`beater-stats`** crate
(built on `statrs`; §20.5 #3.4) that `beater-experiments`, the gate runner
(§12), the online-eval worker (§20.6), and the RSI loop (§6, §21) all call. Each
estimator below states **what it computes, the assumption it requires, and when
it is invalid** — a gate that cannot satisfy an estimator's assumptions must
refuse to decide, not silently use the wrong test.

**Implementation phasing (so this section agrees with §16/§20/§21).** The
*fixed-horizon* core of `beater-stats` — Wilson + bootstrap CIs, clustered SEs,
paired-t/McNemar/Wilcoxon test selection with real p-values, Holm-Bonferroni +
Benjamini-Hochberg, and power/MDE gating (items #1–#5 below) — **ships first** and
is what the offline CI gate and §20.5 #3.4 deliver. Anytime-valid / sequential
inference (item #6: mSPRT and confidence sequences) is **REQUIRED for the
online/continuous path** and ships as the **required follow-on**, not an optional
extra — peeking at a fixed-horizon test inflates false positives 5–10× (§10.3 #6),
so the online-eval worker (§20.6) and live alerting (§13) must not declare a
fixed-N result on a continuously-inspected stream. Phasing means "fixed-n first,
sequential next," **not** "sequential is optional." Until mSPRT lands, the online
path may *display* a running estimate but must not emit an accept/alert *decision*
with fixed-horizon confidence on a peeked stream.

**1. Report standard errors; cluster them when questions are not independent.**
Point estimates are never reported without an error bar. When questions are
non-independent — multi-turn conversations sharing context, or many cases drawn
from the same prompt template — naive i.i.d. standard errors are *too small* and
inflate false wins. `beater-stats` computes **clustered standard errors** with
the cluster id being the conversation/template/seed group
[Miller, "Adding Error Bars to Evals", arXiv:2411.00640]. *Assumption:* clusters
are independent of each other even if items within a cluster are not. *Invalid
when:* clusters themselves are correlated (e.g. all from one adversarial seed) —
then the cluster definition must be coarsened.

**2. Confidence intervals by metric type — prefer Wilson/bootstrap over CLT at
small N.**

- **Binary / proportion metrics** (pass-rate, exact-match): **Wilson score
  interval**, not the normal/Wald interval the current code uses. *Assumption:*
  Bernoulli trials. *Invalid when:* trials are clustered (combine with #1) or N is
  effectively tiny — report the interval but flag low power (#5).
- **Bounded / continuous metrics** (judge scores in [0,1], latency, cost):
  **bootstrap interval** (resample cases, or resample whole clusters for clustered
  data, §10.3 #1). *Defaults:* `n_resamples = 10_000` (the standard
  bias-stable default; fewer than ~2_000 makes the tail quantiles noisy) over a
  **seeded** RNG so a reported interval is reproducible. The default is the
  **percentile** interval; for skewed metrics (cost, latency) `beater-stats` uses
  the **BCa (bias-corrected and accelerated)** interval, which adjusts the
  percentile endpoints by a **bias correction** `z₀ = Φ^{-1}(fraction of bootstrap
  replicates below the observed estimate)` and an **acceleration** `a` estimated by
  jackknife skewness `a = (Σ (θ̄−θ₍ᵢ₎)³) / (6·(Σ (θ̄−θ₍ᵢ₎)²)^{3/2})`; BCa restores
  ~nominal coverage on skewed/biased statistics where the plain percentile interval
  is shifted. *Assumption:* the sample is representative of the population of cases.
  *Invalid when:* N is so small the empirical distribution is degenerate — fall back
  to reporting raw spread and refusing a significance claim.
- Naive CLT/normal intervals are used **only** when N is large and the metric is
  unbounded and roughly symmetric; otherwise they are disallowed.

**3. Significance test selection by metric type AND satisfied assumptions.** The
test is chosen by the data, and `beater-stats` records which assumption justified
the choice:

| Metric / situation | Test | Required assumption |
| --- | --- | --- |
| Paired continuous, ~normal differences | paired *t*-test | normal-ish paired differences, n not tiny |
| Paired binary (pass/fail flips) | **McNemar / exact binomial** | paired Bernoulli outcomes |
| Paired continuous, non-normal | **Wilcoxon signed-rank** | symmetric difference distribution |
| Any, assumptions unclear / small N | **paired bootstrap / permutation** | exchangeability under the null |

A paired *t*-test is used *only when its normality assumption is met*; otherwise
the engine selects Wilcoxon or bootstrap. Pairwise judge comparisons retain the
position-swap mitigation from §10.1.1 before any of these tests see the scores.

**4. Multiple-comparison control — Holm-Bonferroni (FWER) and Benjamini-Hochberg
(FDR), not naive division.** When one experiment evaluates many metrics, cohorts,
or slices, raw per-comparison alpha inflates false wins. `beater-stats` applies
**Holm-Bonferroni** when the goal is to control the family-wise error rate
(strict: "no false win anywhere") and **Benjamini-Hochberg** when the goal is to
control the false-discovery rate (exploratory: "most of the flagged wins are
real"). The current crude Bonferroni *division* of alpha is replaced; it is both
too conservative and applied at the wrong layer.

**5. Power / MDE / minimum-sample planning before declaring a win.** Before a gate
can return *pass*, `beater-stats` checks that the comparison was adequately
powered to detect the minimum detectable effect (MDE) at the gate's alpha/power.
`power.rs` exposes `required_sample_size(effect, alpha, power)` and
`achieved_power(n, effect, alpha)`. **Gates refuse underpowered comparisons** with
an explicit *inconclusive* (not *pass*), so a green CI never means "we ran too few
cases to see a regression."

**6. Online / continuous monitoring MUST use anytime-valid (sequential)
inference (REQUIRED for the online path; phased to ship after #1–#5).** Offline
experiments have a fixed horizon; online evals (§13 alerting, §20.5, §20.6) are
*peeked at continuously*. Fixed-horizon tests under
peeking inflate false-positives by **5–10× even at n=10,000**
[arXiv:1512.04922]. Therefore any continuously-monitored signal uses
**always-valid p-values / confidence sequences** — mixture-SPRT (mSPRT) and
betting-style confidence sequences [arXiv:2402.03683] — which remain valid no
matter how often they are inspected. *Tradeoff:* anytime-valid intervals are
**wider** than fixed-horizon intervals at the same nominal coverage; that is the
price of unlimited peeking and is accepted. *Assumption:* observations are
bounded or sub-Gaussian — **satisfied automatically by 0–1 eval scores**, which is
why this is tractable for Beater's metrics. This ties directly to §13 alert
baselines, §20.5 online statistics, and the §20.6 online-eval worker: alert
conditions on a live score stream are evaluated against a confidence sequence, not
a fixed-N test.

**7. The exact formulas `beater-stats` implements.** So the layer is buildable
without a second reference, the named methods above are pinned to their standard
forms (`statrs` supplies the CDFs/quantiles `Φ`, `Φ^{-1}`, `t`, `χ²`):

- **Wilson score interval** for a proportion `p̂ = k/n` at level `z = Φ^{-1}(1−α/2)`:

  ```text
  center = (p̂ + z²/2n) / (1 + z²/n)
  half   = ( z/(1 + z²/n) ) · sqrt( p̂(1−p̂)/n + z²/4n² )
  CI     = center ± half
  ```

  (vs the deleted Wald `p̂ ± z·sqrt(p̂(1−p̂)/n)`, which under-covers at small `n`
  or `p̂` near 0/1 — exactly Beater's regime).
- **Paired *t*-test** on the `n` differences `dᵢ`: `t = d̄ / (s_d/√n)` with `n−1`
  df, `s_d` the sample SD of the differences; two-sided `p = 2·(1 − F_t(|t|; n−1))`.
- **McNemar exact** on the discordant pairs `(b, c)` of a paired-binary 2×2 (b =
  pass→fail, c = fail→pass): exact two-sided binomial p-value of `b` under
  `Binomial(b+c, ½)` (preferred over the `χ²=(b−c)²/(b+c)` approximation when
  `b+c` is small).
- **Wilcoxon signed-rank**: rank `|dᵢ|`, `W = Σ rank(dᵢ)·sign(dᵢ)`; for small `n`
  the exact null distribution, otherwise the normal approximation
  `z = W / sqrt( n(n+1)(2n+1)/6 )`.
- **Holm-Bonferroni (FWER) — step-down.** Sort the `m` p-values ascending
  `p₍₁₎ ≤ … ≤ p₍ₘ₎`; reject `p₍ᵢ₎` while `p₍ᵢ₎ ≤ α/(m−i+1)`, and **stop at the
  first failure** (all subsequent stay non-rejected). Uniformly more powerful than
  the crude `α/m` division the old code used.
- **Benjamini-Hochberg (FDR) — step-up.** Same sorted p-values; find the
  **largest** `i` with `p₍ᵢ₎ ≤ (i/m)·α`, reject all `p₍₁₎…p₍ᵢ₎`. Controls the
  expected false-discovery proportion at `α` for the exploratory multi-slice case.
- **Power / MDE.** For a two-sample proportion test at `α`, power
  `1−β`, the required per-arm `n` for effect `δ` (`= p₁−p₀`, pooled SD `σ`) is the
  textbook
  `n = ( (z_{1−α/2} + z_{1−β})·σ / δ )²`; `power.rs` exposes
  `required_sample_size(δ, α, 1−β)` (invert for `n`) and `achieved_power(n, δ, α)`
  (solve for `1−β`). A gate refuses *pass* when `achieved_power < target` (A6).
- **Anytime-valid: mSPRT / confidence sequence (the e-process).** For a stream of
  bounded scores testing `H₀: μ = μ₀`, `beater-stats` maintains a non-negative
  **e-process** `Eₜ` (an e-value that is a martingale under `H₀`, so
  `E[Eₜ] ≤ 1`). The **mixture-SPRT** form mixes the simple-vs-simple likelihood
  ratio over a prior on the alternative (a Normal mixture for sub-Gaussian scores),
  giving `Eₜ = ∫ Λₜ(θ) dπ(θ)`; **Ville's inequality** guarantees
  `P(∃t : Eₜ ≥ 1/α) ≤ α` — so rejecting the first time `Eₜ ≥ 1/α` is valid **no
  matter how often the stream is peeked.** The dual **confidence sequence** is the
  set `{μ : Eₜ(μ) < 1/α}`, a running interval valid at every `t` simultaneously.
  The betting-style alternative replaces `Λₜ` with a capital process
  `Eₜ = Πₛ (1 + λₛ·(Xₛ − μ₀))` over predictable bets `λₛ ∈ [−1/(1−μ₀), 1/μ₀]`
  (well-defined because scores are bounded in [0,1], A8). Both are wider than the
  fixed-horizon CI at the same `α` — the accepted price of unlimited peeking.

**Carried-over requirements** (unchanged in intent, now with a home in
`beater-stats` and the §10.1.1 calibration):

- candidate-vs-baseline deltas; variance reported by case and metric.
- N-trial repetition for noisy evaluators (reliability/variance is itself a
  measured agent dimension — see §6).
- judge calibration artifact: judge-vs-human agreement, confusion counts, Cohen's
  kappa where applicable, plus the distributional-calibration map of §10.1.1
  (`beater-calibration` already persists kappa/agreement).
- recalibration triggers for model deprecation, provider drift, rubric changes,
  and kappa degradation.

The CI gate must be able to fail on **confidence-bound** regressions (and refuse
*inconclusive* underpowered ones), not only raw mean-score deltas.

**Sampling weights flow into the estimators.** When an estimate is computed over
production traffic rather than a balanced dataset, the per-span `sampling_weight`
(§9, §1 #9) is carried through: proportions/means become Horvitz-Thompson weighted
estimates and bootstrap resampling resamples in proportion to weight, so the
reported interval is an honest estimate of the *population* rate, not the
tail-sampled *kept* rate. Offline dataset evals run on balanced cases where every
weight is 1.0, so this only changes production/online aggregates — but where it
applies, an unweighted number is simply wrong.

Every assumption named in this subsection (clustering, interval validity, test
selection, nominal=actual alpha, multiplicity, power, no-peeking, weighting) is
gathered with its check and the §22 test that enforces it in the consolidated
assumptions table at §21.9 (rows A1–A8, A19).

### 10.4 Grading Algorithms & Assumptions

A score is only as trustworthy as the algorithm that produced it. This catalogue
pins each scorer as a concrete algorithm with its **assumptions**, the conditions
under which it is **invalid** (so the platform can refuse to emit a misleading
score), and its **CI / aggregation** path into §10.3. The **Lane** column says
whether it runs in the deterministic WASI sandbox (no network, §10.1 deterministic
lane) or the judge broker (§10.1 judge lane). Scorers marked **[planned]** are in
the §20.5 catalog-breadth work; the rest exist in `EVALUATOR_CATALOG` today.

| Scorer | Computes | Key assumption | Invalid when | CI / aggregation | Lane |
| --- | --- | --- | --- | --- | --- |
| **Exact match** | 1 if output == expected (after normalization) else 0 | a single canonical correct string exists | free-form/multi-valid answers; whitespace/casing matters but isn't normalized | Wilson (binary), §10.3 #2 | WASI |
| **Regex match** | 1 if pattern matches output | the pattern captures all-and-only correct outputs | pattern over/under-matches; catastrophic backtracking on adversarial input | Wilson (binary) | WASI |
| **Fuzzy match (strsim)** [planned] | similarity ratio ≥ `min_ratio` (Levenshtein/Jaro-Winkler) | edit distance correlates with semantic correctness | semantics diverge from surface form (paraphrase, reordering) | threshold→binary Wilson, or ratio→bootstrap | WASI |
| **JSON-schema** [planned] | 1 if output validates against a JSON Schema | the schema fully encodes "valid" structure | schema is laxer/stricter than true validity; valid JSON, wrong meaning | Wilson (binary) | WASI |
| **JSON-object (current)** | 1 if output parses as a JSON object | object-shape ⇒ correct (weak) | checks shape only, *not* schema — a wrong-but-well-formed object passes | Wilson (binary) | WASI |
| **Numeric tolerance** [planned] | 1 if `|out−exp| ≤ abs` or `≤ rel·|exp|` | a numeric ground truth with a known tolerance | unit mismatch; tolerance mis-set; non-numeric output | Wilson (binary) | WASI |
| **Cost / latency / token budget** | 1 if measured ≤ budget | the measured field is populated and trustworthy | missing/estimated cost or tokens; clock skew on latency | Wilson (binary); raw values → bootstrap | WASI |
| **Embedding similarity** [planned] | cosine(sim(out), sim(exp)) ≥ `min_cosine` | the embedding space separates correct from incorrect | out-of-domain text; threshold not calibrated; model drift | threshold→Wilson, or cosine→bootstrap; recalibrate on model change | **judge** (needs an embedding provider) |
| **SQL-result match** [planned] | 1 if executing the candidate SQL yields the expected result set | a fixed seeded DB and order-insensitive set compare | schema/data drift; nondeterministic queries; ORDER BY semantics | Wilson (binary) | WASI (execution against a sandboxed/seeded store) |
| **Execution-based tool correctness** | 1 if the tool call, *executed*, produces the correct effect/result | tool calls are checked by EXECUTION, not by AST/argument syntax | judging only the *syntactic* call shape (a syntactically valid call can be semantically wrong, and a differently-shaped call can be correct) | Wilson (binary); per-call then per-trajectory aggregation | WASI (replayed/sandboxed) |
| **Trajectory / process-reward** | a process score over the span sequence (plan→step→tool→…) | progress is jointly modeled across steps, *not* independent per-step scores (AgentPRM-style promise+progress) | scoring steps independently double-counts shared context and misattributes credit | per-step scores aggregated with clustered SE (§10.3 #1, cluster = trajectory) | WASI for structural checks; **judge** for quality |
| **Rubric LLM judge** | weighted per-criterion score from a locked rubric + CoT | the §10.1.1 debiasing protocol holds (calibration, position-swap, small panel) | calibration stale; rubric unlocked mid-episode; large uncalibrated panel | distributional calibration (§10.1.1) → bootstrap CI; FWER across criteria (§10.3 #4) | **judge** |

Exact algorithm per scorer (the surface forms behind the table):

- **Exact match** — compare after a fixed normalization pipeline (Unicode NFC,
  trim, optional case-fold and whitespace-collapse, all flags pinned in the eval);
  score `1` iff equal.
- **Fuzzy match** — the **normalized Levenshtein ratio**
  `ratio = 1 − lev(a,b)/max(|a|,|b|)` ∈ [0,1] (or Jaro-Winkler where prefix
  matching matters), via `strsim`; score `1` iff `ratio ≥ min_ratio` (default
  `0.9`). The threshold collapses to a binary scored by Wilson; the raw ratio is
  kept for a bootstrap CI when reported continuously.
- **Numeric tolerance** — parse both to `f64`; score `1` iff
  `|out − exp| ≤ abs_tol` **or** `|out − exp| ≤ rel_tol·|exp|` (default
  `abs_tol = 0`, `rel_tol = 1e-6`); both bounds let "within ε" and "within X%"
  coexist.
- **JSON-schema** — validate against a draft-2020-12 JSON Schema (vs the weaker
  current "parses as an object" check); score `1` iff valid.
- **Embedding similarity** — cosine `sim = (u·v)/(‖u‖‖v‖)` ∈ [−1,1] between
  embeddings of output and expected from a **pinned** embedding model; score `1`
  iff `sim ≥ min_cosine` (a model-specific threshold, **recalibrated on model
  change** — there is no universal cutoff). Judge lane (needs a provider).
- **SQL-result match** — execute candidate SQL against a fixed **seeded** DB and
  compare result sets as **multisets** (order-insensitive unless the query has an
  explicit `ORDER BY`, in which case order is compared); score `1` iff equal.
- **Execution-based tool correctness** — replay/execute the tool call against the
  seeded environment and check the **effect/result**, never the serialized call
  shape (a syntactically valid call can be wrong, a differently-shaped call can be
  right). Per-call binary → Wilson, then per-trajectory clustered (§10.3 #1).
- **Trajectory / process-reward** — an **AgentPRM-style promise+progress** joint
  score over the span sequence: each step is scored for *progress* (did it advance
  the goal) and *promise* (is the path still on track), combined across the
  trajectory rather than averaging independent per-step scores; aggregated with
  trajectory-clustered SE [arXiv:2511.08325; arXiv:2507.21504].

Two cross-cutting rules:

- **Tool-call correctness is execution-based, never AST/syntactic.** A scorer that
  only diffs the serialized tool call against an expected call confuses *form* for
  *effect*; Beater scores the call by replaying/executing it (deterministic lane,
  seeded) and checking the result.
- **Trajectory quality is jointly modeled, not a mean of independent per-step
  scores.** Independent per-step scoring violates the clustering assumption of
  §10.3 #1 (steps within a trajectory share context) and mis-assigns credit; the
  process-reward scorer models promise/progress across the sequence and aggregates
  with trajectory-clustered standard errors [arXiv:2511.08325; arXiv:2507.21504].

Aggregation always flows back through §10.3: per-case scores → metric-appropriate
CI → clustered when non-independent → significance test by type → multiplicity
control across scorers → power check before any *pass*.

### 10.5 Agent / Score Calibration (proper scoring rules)

A score or a confidence is only useful if it *means* what it claims. There are two
**distinct** calibration problems in this platform, and they coexist without
conflict:

- **Judge calibration (§10.1.1)** maps a noisy *judge model's* raw scores onto a
  human-anchored distribution by Wasserstein quantile-matching. It lives in the
  **judge broker** and answers "is the ruler reading right?"
- **Agent / score calibration (this section)** asks whether a *probabilistic
  signal* — the agent's own stated confidence, or a continuous judge/confidence
  score used as a probability — is *well-calibrated against outcomes*: when the
  signal says 0.8, is the event true ~80% of the time? It lives in
  `beater-calibration` and ties directly to agent dimension #7 (self-calibration,
  §6.3).

These are orthogonal: §10.1.1 corrects the measuring instrument; §10.5 measures and
corrects a probability's calibration. Both run; neither replaces the other.

**Why this is near-free.** The continuous judge/confidence signal needed for proper
scoring is **already produced and then discarded today** — the platform thresholds
it to a label and throws away the probability. Persisting that probability and
scoring it with proper rules is mostly plumbing, not new modeling.

**Proper-scoring metrics (replacing kappa as the primary calibration signal):**

- **Brier score** — mean squared error between the stated probability and the 0/1
  outcome; a strictly proper scoring rule, so it is minimized only by honest
  probabilities. Reported with a §10.3 bootstrap CI.
- **Expected Calibration Error (ECE)** — the binned gap between confidence and
  observed accuracy; the headline "is it calibrated" number. *Binning:* the standard
  **M = 10 equal-width bins** over [0,1] (`B_m = (\,(m-1)/10,\ m/10\,]`); with
  `acc(B_m)` the empirical accuracy and `conf(B_m)` the mean confidence in bin `m`,
  `ECE = Σ_m (|B_m|/n)·|acc(B_m) − conf(B_m)|`. Equal-width is the default; an
  equal-mass (adaptive) binning is the alternative when confidences pile up in a few
  bins, and `beater-stats` reports which was used so the number is comparable
  across runs.
- **Reliability curve** — the per-bin confidence-vs-accuracy plot the dashboard
  renders, the visual form of ECE.
- **Cohen's kappa becomes a secondary signal.** The existing `beater-calibration`
  kappa/agreement report (the judge-vs-human agreement artifact, §10.1.1, §10.3) is
  retained for backward continuity and inter-rater context, but the *primary*
  calibration verdict is now Brier/ECE, because kappa neither rewards honest
  probabilities nor yields a recalibration map.

**Persisted recalibration map.** From the reliability data `beater-calibration`
fits and **persists** a monotone recalibration map `c(p) → p'` that corrects
systematically over- or under-confident signals. The map is versioned and pinned
into `EvalResult` reproducibility metadata (like the §10.1.1 judge calibration) so a
corrected probability's provenance is auditable, and it is re-fit on the same
recalibration triggers as §10.1.1 (model deprecation, provider/judge drift, rubric
change, kappa/ECE degradation).

**Which map to fit — a concrete selection rule** (the three are standard,
named methods, chosen by reference-label budget and the shape of the miscalibration):

- **Platt scaling** — fit a 1-D logistic `p' = σ(a·logit(p) + b)` by maximum
  likelihood on `(p, outcome)` pairs (2 parameters). *Use when* labels are scarce
  (roughly **n < 1000**) or the reliability curve is a smooth monotone sigmoid;
  its 2-parameter form is low-variance and will not overfit a small set, but it
  *cannot* correct a non-sigmoidal distortion.
- **Isotonic regression** — fit the best non-decreasing step function minimizing
  squared error via the **Pool-Adjacent-Violators Algorithm (PAVA)**, `O(n)` after
  an `O(n log n)` sort. *Use when* labels are plentiful (roughly **n ≥ 1000**) and
  the miscalibration is non-monotone-in-shape but order-preserving; it is
  non-parametric and strictly more flexible than Platt but overfits and produces
  ragged steps on small `n`. This is the default for the abundant-label case.
- **Wasserstein quantile-matching** `g(z) = F_human^{-1}(F_model(z))` (§10.1.1) —
  used for the *judge-vs-human distribution* problem, **not** the
  probability-vs-outcome problem. It matches a whole *score distribution* to a human
  reference rather than mapping a confidence to an empirical accuracy, so it is the
  right tool for §10.1.1 (judge broker) and the wrong tool here; the two coexist
  (§10.5 intro).

**Default selection:** Platt below ~1000 reference labels, isotonic at or above it;
both require a **minimum of ~50 labeled outcomes** to fit at all (below that the map
abstains and `c(p)=p` is used, flagged "uncalibrated"). **Cadence:** re-fit on every
§10.1.1 trigger and otherwise on a **rolling window** when ECE on a fixed canary set
(A9) degrades past a configured threshold — i.e. event-driven, not a fixed calendar.
The RSI
loop's self-calibration dimension (§6.3 #7) reads ECE/Brier on the held-out Test
split; a change that improves task success while *degrading* calibration is visible
as a regression on this axis rather than hidden inside a single score.

## 11. Replay and Failure Attribution

Replay substrate:

- Immutable event stream.
- Provider cassettes for LLM requests/responses, streaming chunks, tool calls,
  embeddings, retrievals, memory reads/writes, guardrail checks, random seeds,
  clock reads, and human-review stubs.
- Versioned tool schemas and MCP request/response bodies.

Replay modes:

- `deterministic_replay`: all required cassettes present and hashes match.
- `forked_replay`: starts from a captured trace, then calls live providers/tools
  after a fork point.
- `simulation`: substitutes models/tools with configured simulators.

Failure attribution pipeline:

```text
failed trace
  -> span-level deterministic checks
  -> judge or human checks where needed
  -> compare against similar passing traces
  -> replay or fork candidate corrections
  -> identify earliest span whose correction flips outcome
  -> write root-cause annotation and regression candidate
```

**Real forked replay + earliest-failing-span attribution (replaces the current
first-error heuristic).** Today `attribute_failure` in `beater-replay` is a stub:
it sorts spans by `seq` and returns the first span that is `Status::Error` or whose
evidence score `< 0.5`. That is "the first thing that looked bad," which is *not*
the same as "the earliest change that flips the outcome" — an early low-score span
may be irrelevant while a later one is causal, and a trace can fail with no errored
span at all. The replacement is a real **forked-replay search**:

```text
for candidate fork points, earliest-first along the causal span order:
  fork the captured trace at that span (deterministic_replay of the prefix
    from cassettes, §11 replay modes)
  apply the candidate correction at the fork point (corrected tool result,
    corrected llm.call output, alternate plan step)
  resume forked_replay from the fork point (live or simulated, labeled honestly)
  re-score the resumed trajectory with the SAME frozen evaluator (§6.2)
  if the outcome FLIPS (fail -> pass):
    record this fork point as a root-cause candidate
return the EARLIEST fork point whose correction flips the outcome
  (a counterfactual minimal cause), with the replay mode + guarantee level
  that produced it
```

This is a counterfactual definition — the root cause is the *earliest* span whose
correction is *sufficient* to flip the outcome — so it survives the cases the
heuristic fails on (no errored span; misleading early low score).

**Complexity + the bisection optimization.** The naive scan tries each of the `n`
candidate fork points earliest-first and stops at the first flip: worst case `O(n)`
forked replays, each costing one prefix-replay + resume + re-score. When the outcome
is **monotone in the fork point** — correcting an *earlier* span never *un*-flips a
later success (the common case for a single propagating fault) — the earliest
flipping span is found by **binary search (bisection) over the span order in
`O(log n)` replays**: replay-and-score at the midpoint, recurse left if it flips,
right if it does not. `beater-stats` is not involved; this is a deterministic search.
Monotonicity is an assumption, not a guarantee, so bisection is the fast path and the
linear earliest-first scan is the **fallback** whenever the cheap monotonicity check
fails (e.g. interacting faults), preserving correctness at `O(n)`. The search is also
bounded by a fork budget. Attribution
confidence is reported with its replay guarantee level: a flip found under
`deterministic_replay` (all cassettes present, hashes match) is high-confidence; a
flip found under `forked_replay`/`simulation` is labeled as such (§1 #6). The
search is bounded by a fork budget; when no single-span correction flips the
outcome it returns "no single-span root cause" rather than a false attribution. The
flipped run is the natural seed for a regression dataset case (`split` assigned per
§5.4).

The product should surface:

- root-cause span (the earliest outcome-flipping fork point)
- confidence/evidence and the replay guarantee level behind the attribution
- failed-vs-passed diff
- replay mode and guarantee level
- one-click "add to dataset"

## 12. Agent Harness

The harness lets the platform become an open-source eval runner, not just a
trace viewer.

```rust
#[async_trait::async_trait]
pub trait AgentAdapter {
    async fn run_case(
        &self,
        case: DatasetCase,
        ctx: HarnessContext,
    ) -> anyhow::Result<AgentRunOutput>;
}

#[async_trait::async_trait]
pub trait ToolProxy {
    async fn call_tool(&self, request: ToolRequest) -> anyhow::Result<ToolResponse>;
}

#[async_trait::async_trait]
pub trait TraceEmitter {
    async fn emit(&self, event: CanonicalEvent) -> anyhow::Result<()>;
}
```

Harness components:

- `ScenarioRunner`: runs dataset cases, seeds, timeouts, and concurrency.
- `ProviderProxy`: records provider cassettes and enforces budgets.
- `ToolProxy`: records tool/MCP cassettes and validates schemas.
- `JudgeRunner`: routes model-dependent evals through judge broker.
- `ExperimentRunner`: runs baseline and candidate releases.
- `GateRunner`: converts persisted experiment reports into CI pass/fail. A gate
  run may target an explicit experiment run ID or the latest tenant/project
  report matching the gate's dataset/evaluator selectors; the gate-run report
  snapshots the gate definition, experiment gate policy, comparison, decision,
  and pass/fail reason.
- `HumanReviewRunner`: queues trace/span review tasks, stores human annotations,
  and promotes corrected human references into dataset cases through the same
  dataset store used by automated evals.
- `TraceEmitter`: emits canonical traces and raw refs.

The same harness must run locally, in CI, and in hosted workers.

## 13. Query, UI, and Alerting

Core UI requirements:

- trace table with project/environment/release filters
- span tree and waterfall
- agent turn/plan/step view
- MCP/tool-call visibility
- prompt/input/output/artifact inspector with redaction controls
- cost/token/latency analytics (**weighted by `sampling_weight`** so tail-sampled
  traffic produces unbiased population totals, §1 #9, §9; an unweighted view is
  available only when explicitly labeled biased)
- dataset promotion from trace/span (including **bulk promote-from-query**, §20.4)
- experiment comparison
- eval result drilldown
- replay/cassette view
- human review queues
- failure clustering and root-cause annotations

Search:

- structured filters by status, time, trace ID, span kind, model, tool, cost,
  latency, token counts, environment, agent release, evaluator, and tags
- full-text search over inputs, outputs, errors, tool names, and selected attrs
  through Tantivy (Crate Dig), ranked by **Okapi BM25** — Tantivy's default
  scorer — with the standard parameters `k₁ = 1.2` (term-frequency saturation) and
  `b = 0.75` (document-length normalization), tenant-scoped at query time. The
  score of document `D` for query terms `qᵢ` is
  `Σᵢ IDF(qᵢ)·( f(qᵢ,D)·(k₁+1) ) / ( f(qᵢ,D) + k₁·(1 − b + b·|D|/avgdl) )`. The
  index stores only what §8.3 permits (refs/redaction-aware fields), never
  unredacted payloads it should not hold.
- natural-language search is later; fast structured BM25 search is v1

Alerting:

- online eval sampling policies
- baselines by project/environment/release, evaluated against an **anytime-valid
  confidence sequence** on the live score stream, not a fixed-N test — a
  continuously-peeked alert condition that used a fixed-horizon test would inflate
  false alerts 5–10× (§10.3 #6); this is the required online follow-on of the
  §10.3 phasing
- dedupe and grouping
- maintenance windows
- Slack/webhook integrations
- alert budgets and suppression
- links back to trace clusters, dataset candidates, and gates

## 14. Compliance, Security, and Data Lifecycle

Data lifecycle:

- per-project retention policies
- hot ClickHouse -> cold Parquet -> delete
- referential consistency across rows, artifacts, cassettes, indexes, and exports
- orphan artifact sweeps
- restore drills before hosted GA

Deletion:

- immutable raw rows conflict with subject deletion unless encryption is designed
  correctly
- use envelope encryption with per-tenant keys and, where enterprise requires,
  per-subject data encryption keys
- deletion can be implemented as crypto-shredding plus deletion of lookup rows
  and object refs
- deleted data must become unreadable across hot, cold, and artifact stores

Security:

- API keys are scoped, rotatable, hashed, environment-bound, and audited
- PII unmask is a separate RBAC scope from ordinary trace read
- sensitive-data access emits audit events
- BYOK for judge/model providers
- encryption at rest for metadata, artifacts, cassettes, and cold files
- key rotation and key revocation workflows
- outbound webhooks use HMAC signatures, timestamp/replay protection, retries,
  and idempotency keys
- all tenant-scoped storage APIs take `TenantId` explicitly
- hosted cells enforce region/project pinning; PII does not cross regions

## 15. Public API and DX

DX SLO:

```text
time to first SCORED FAILURE <= 15 minutes
  (subsumes the older "time to first trace <= 5 minutes" milestone)
```

The DX SLO is **time to first *scored failure***, not time to first trace. A trace
on screen proves ingestion; it does not prove the product's value loop. The thing a
user must reach fast is the moment Beater shows them a *failing* agent behavior with
a *score* on it (the §0 core loop's "promote failure → run evals" inflection) —
that is when the platform has demonstrably done something a log viewer cannot.
"Time to first trace ≤ 5 min" remains a useful internal sub-milestone (and the §18
v0 `beaterctl smoke` target) but is no longer the headline DX number.

The **default** onboarding path is **zero-code OTLP bootstrap** (§1 #13, §20.8):
point a standards-based OTLP exporter at Beater via environment variables, with no
Beater SDK and no code edits. The native SDK is an accelerator offered second, not
the adoption gate.

Required onboarding paths (zero-code OTLP first):

- **zero-code env-var OTLP bootstrap (DEFAULT)** — any OpenInference/OpenLLMetry/
  OTel app exports to Beater by setting `BEATER_*`/OTLP env vars; no code, no SDK
  (§20.8 #6.2)
- zero-SDK OTLP endpoint for any OpenInference/OpenLLMetry/OTel app
- native Rust SDK with `tracing`, `opentelemetry-rust`, `reqwest`, `axum`,
  `tonic`, MCP client/server, and tool-call helpers
- Python and TypeScript examples through standards-based OTLP first
- seeded demo project
- one-command Docker Compose
- copy/paste framework quickstarts
- `beaterctl smoke` to emit a known trace and verify ingestion

Public API:

- stable `/v1`
- OpenAPI-generated clients
- deprecation policy
- request IDs and idempotency keys
- pagination and time-bounded queries
- export endpoints for OTLP, Parquet, and JSONL
- import paths from Phoenix/LangSmith/Langfuse where feasible, plus the
  config-driven MAPPING importer (§7) for custom/older dialects with no code

No lock-in:

- export traces and evals without hosted dependency
- import existing datasets/traces
- keep raw source attributes for migration and round-trip use

## 16. Self-Observability SLOs

Beater dogfoods itself. Hosted cannot launch without dashboards and alerts for:

| Signal | Initial target |
| --- | --- |
| ingest accepted write success | >= 99.9 percent over 30 days |
| ingest-to-queryable lag p95 | <= 5s hosted hot path, <= 15s OSS compose |
| DLQ age p95 | <= 5m before alert |
| query p95 for indexed 24h searches | <= 1s for normal tenant workloads |
| query p95 for 30d filtered searches | <= 3s for normal tenant workloads |
| eval queue depth | bounded by project policy and worker count |
| judge spend | attributed to org/project/experiment/evaluator |
| artifact orphan rate | zero tolerated after sweeper window |
| tail-sampling decision lag p95 | <= trace completion target |

These numbers will change with load testing, but the product must expose them
from day one.

## 17. Execution Reality

### 17.1 Minimum Shippable OSS Product

The first serious open-source release needs all of this, not a smaller demo:

- Rust all-in-one `beaterd`
- OTLP and native ingest
- canonical trace schema
- immutable raw envelopes
- `TraceStore` abstraction *(built)*
- SQLite local mode *(built)*; Postgres local mode *(impl exists, unwired)*
- ClickHouse scale backend *(impl exists, unwired — §20.2 #0.1)*
- artifact storage *(built: filesystem; object store planned)*
- durable bus *(built: `SqliteDurableBus`; NATS JetStream planned)*
- trace table and span tree UI
- Rust SDK and tracing layer
- dataset creation from traces
- offline deterministic evals
- WASI evaluator ABI
- judge broker with BYOK
- experiment comparison with confidence intervals
- CI gate command
- Docker Compose
- import/export for OTLP, Parquet, and JSONL

That is the smallest version that can be called a serious OSS project.

### 17.2 Team Needed for Hosted SOTA

A solo founder can ship a focused OSS MVP, but not a hosted SOTA platform quickly.
The realistic team is:

- Rust infra lead
- backend/product engineer
- frontend/product engineer
- data/observability engineer
- evals/agent-systems engineer
- infra/security engineer part-time early and full-time before hosted GA

### 17.3 Hardest Problems

The hard parts are not CRUD:

- schema evolution: re-normalizing stored raw envelopes into each new canonical
  `schema_version` correctly and at scale (pre-1.0 we change canonical types freely
  and re-project from immutable raw — §5.3; the hard part is the re-normalization
  pass, not preserving a frozen wire format)
- multi-tenant privacy and artifact security
- accurate standards translation
- ingest survivability during storage outages and traffic spikes
- evaluator reproducibility
- judge reliability and calibration
- replay correctness
- query speed over high-cardinality span volumes
- UX that makes agent failures obvious
- open-core trust and governance

## 18. Milestones

### v0: Substrate

Acceptance:

- `beaterd` starts as one binary.
- OTLP HTTP/gRPC and native ingest accept traces.
- Raw envelopes and canonical projections are both stored.
- `TraceStore` exists with SQL and ClickHouse implementations. *(Status: SQLite
  is the runtime default; Postgres/ClickHouse impls exist but are not yet
  selectable at runtime — see §20.2 #0.1.)*
- A durable bus buffers writes and DLQ paths are visible. *(Status: today this is
  the `SqliteDurableBus`; NATS JetStream is planned — see §8.4, §20.2.)*
- A seeded demo trace renders in the UI.
- `beaterctl smoke` proves time-to-first-trace.

### v1: OSS Observability and Offline Evals

Acceptance:

- trace table, span tree, waterfall, and agent step views work
- dataset promotion from trace/span works
- deterministic evals run in WASI sandbox
- judge broker runs BYOK LLM evals
- experiments compare baseline vs candidate with confidence intervals
- CI gate blocks regressions
- Docker Compose is the supported self-host path

### v2: Agent-Native Debugging

Acceptance:

- replay cassettes support deterministic replay where complete
- forked replay and simulation are labeled honestly
- root-cause attribution identifies the earliest likely failing span on seeded
  multi-step agent failures
- human review queues feed datasets
- online eval sampling and alert grouping work
- MCP/tool/memory spans are first-class

### v3: Hosted GA

Acceptance:

- Vercel-hosted dashboard and Rust control-plane functions
- hosted Rust cells for ingest/eval/replay
- org/project/environment isolation
- quotas, rate limits, billing ledger, and usage metering
- retention, crypto-shredding, data residency, audit, BYOK, and key rotation
- backups and restore drills
- SLO dashboards
- Slack/webhook alerts
- regional deployment story

## 19. Bar for Done

The platform is real when a team can replace ad hoc Phoenix, LangSmith,
Braintrust, and spreadsheet workflows and answer:

- What did my agent do?
- Why did it fail?
- Has this happened before?
- Can I reproduce it?
- Can I test the fix?
- Did the fix regress anything?
- Can I block deploys on that?
- Can I monitor it in production?
- Can I self-host without calling your cloud?

If any answer is no, that area is not shipped.

## 20. Planned: Execution to Parity-Grade GA

This section is the concrete, technical execution plan to take Beater from its
current state to feature parity with Arize Phoenix, Braintrust, LangSmith, and
Langfuse for deep agent evaluation. It builds on — and does not replace — the
milestones in §17–18. The milestones describe *what* must exist; this section
describes the *current measured gap* and the *specific work* to close it, at the
crate/type/endpoint level.

Every contract-touching item (new or changed `/v1` route, request/response type,
or span kind/attribute) MUST follow the `CLAUDE.md` contract regen workflow
(`cargo xtask regen-spec` → `scripts/regen-sdks.sh` → `cargo xtask regen-semconv`
→ `scripts/check-contract-sync.sh`). Those items are tagged **[contract]** below.

### 20.1 Readiness Baseline (audited 2026-06-27)

A six-dimension audit of `main` against the parity bar. Overall readiness ≈ 33%:
strong primitives, missing product/scale/control-plane pillars.

| Dimension | Readiness | Headline gap |
| --- | --: | --- |
| Ingestion, SDKs & instrumentation | 58% | no session/thread grouping; flat scalar I/O (no message/tool-call/multimodal); no auto-instrumentation; no CrewAI/DSPy/Vercel-AI/OpenAI-Agents |
| Evaluations, datasets & reproducibility | 38% | no read APIs; no eval/dataset UI; thin scorer catalog; no prompt registry; no CI plugins |
| Security, multi-tenancy & hosted ops | 38% | OAuth 2.1 + accounts/sessions now exist (`beater-oauth`/`-oauth-server`/`-accounts`, wired into `beaterd`) but enforced RBAC, SSO/SAML/SCIM are absent; RBAC data model never consulted by `authorize()`; audit covers one action; no deletion/retention/billing/backups |
| Experiments, statistics, online evals & alerting | 34% | one hand-rolled normal-approx; online evals sampled but never scored; alerts computed but never delivered; no Slack |
| Data model, storage, scale & query performance | 22% | SQLite-only runtime (ClickHouse/Pg unwired); full-scan queries, no LIMIT/keyset pushdown; zero benchmarks/SLOs; no runtime TTL |
| Product surface (UI, replay, annotation, prompt) | 22% | one read-only trace-waterfall page is the entire product |

Already genuinely strong (do not rebuild): OTLP HTTP+gRPC core; dual
OpenInference + OTel `gen_ai` normalizer; 4 tracing SDKs with `@observe`;
reproducibility/lineage pinning; WASI scorer sandbox; judge broker with
cost/ledger/audit; tail-sampling; crypto primitives (Argon2 keys, ChaCha20
envelope + online re-wrap, signed webhooks, BYOK); OAuth 2.1 authorization server
(PKCE, accounts, sessions) wired into `beaterd`; quota limiter; single-source
OpenAPI → 7 SDKs + MCP + CLI with a CI drift gate; Apache-2.0 + governance.

Biggest missing pillars: prompt management; hosted control plane
(identity/SSO/enforced RBAC); load-tested scale; product UI beyond the waterfall;
data lifecycle & compliance; online evaluation scoring; real statistics + alert
delivery; auto-instrumentation & modern-framework coverage.

### 20.2 Phase 0 — Scale & Data Plane

Goal: make a scale claim defensible. Wire the columnar store into the running
service, push filtering/pagination into the backend, prove latency, enforce TTL.

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 0.1 | Columnar store wired into `beaterd` | `ClickHouseTraceStore`/`PgTraceStore` implemented but dead code; runtime hardcodes `SqliteTraceStore` | Add `TraceStoreBackend` env/CLI arg (`sqlite\|postgres\|clickhouse`) + `build_trace_store(cfg) -> Arc<dyn TraceStore>` in `beater-store-sql`; thread through `ApiState` and the ingest/query bins; non-ignored compose integration test booting `beaterd` on ClickHouse | L | docker |
| 0.2 | Server-side pagination + pushdown | `query_spans` appends no `LIMIT`, paginates in memory; `query_runs` materializes all spans (`limit u32::MAX`) | Push `PageRequest.limit` + time-window into SQL; keyset (seek) cursors on `(start_time, span_id)`; reimplement `query_runs` as backend `GROUP BY`; add `start_after/before` to `SpanFilter`/`RunFilter` | XL | none |
| 0.3 | Measured query p95 SLOs | no `benches/`, no criterion, no load test, no SLO evidence | New `beater-bench` crate: criterion benches for `write_batch` throughput + `query_*` latency on seeded 1M/10M/100M-span fixtures; `xtask loadgen` emitting OTLP at sustained RPS → p50/p95/p99; codify §16 SLOs + CI regression gate | XL | evidence |
| 0.4 | Runtime retention/TTL | TTL exists only as ClickHouse DDL that never runs | `RetentionPolicy{hot_days,archive_days}` in `beater-core`/`beater-schema`; retention sweeper (extend `beater-archive`) on an interval in `beaterd` demoting-then-deleting expired hot rows; `GET/PUT /v1/projects/:id/retention` **[contract]** | L | design |
| 0.5 | Automated cold-tier archival | `ParquetTraceArchive` exists, local-fs only | Write partitioned append-only Parquet (`tenant/project/yyyymm/uuid`) to object store via `beater-store-obj`; scheduled demotion job; DataFusion read path over cold files | L | design |
| 0.6 | Backend-agnostic migrations + re-normalization | versioned framework exists for SQLite only | Generalize the `SqliteMigration` version/checksum `Migrator` to ClickHouse + Postgres (`_beater_schema_migrations` on each); `xtask renormalize` reprojecting historical `RawEnvelope`s to a new canonical version | L | none |

Acceptance: `beaterd --trace-store clickhouse` boots and serves traces; a 10M-span
seeded search returns under the §16 p95 SLO in CI; expired rows are demoted then
deleted by the sweeper; benches run in CI and gate regressions.

### 20.3 Phase 1 — Agent-Native Trace Data Model

Goal: close the table-stakes agent concepts the data model lacks.

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 1.1 | Session/thread/conversation grouping | absent from schema, normalizer, SDKs | Add `session_id/thread_id/user_id` to `CanonicalSpan`; map `session.id`/`thread.id`/`user.id` + OpenInference session attrs in `beater-otlp`; sessions index in `beater-store`; `/v1/sessions` list/get **[contract]**; `session_id` param on SDK `observe()/span()` (py/ts/go/java) | L | contract |
| 1.2 | Structured message/role/tool-call I/O | only flat `input.value/output.value` scalars | Parse OpenInference `llm.input_messages/output_messages/tool_calls` + `gen_ai.*` message events into a `CanonicalMessages` structure on `CanonicalSpan`; golden fixture tests for both dialects **[contract]** | L | contract |
| 1.3 | Multimodal (image/audio/file) I/O | stringified scalars only | `MediaArtifact{mime_type,uri-or-inline,role}` on canonical messages; parse OpenInference content-part `image_url`/`audio`; store large media via `beater-store-obj` with size caps + redaction class **[contract]** | L | design |
| 1.4 | Full-text over artifact-backed I/O | tantivy indexes only inline attrs, not artifact bodies | In `beater-search`, have the ingest processor resolve `input_ref`/`output_ref` via `ArtifactStore` and index their text into dedicated `input_body`/`output_body`/`error` fields; per-tenant shards | L | evidence |
| 1.5 | OTLP/JSON + canonical `/v1/traces` alias | OTLP HTTP is protobuf-only on a tenant-scoped path | Content-type negotiation in `ingest_otlp_http` (deserialize `ExportTraceServiceRequest` from JSON); gRPC `partial_success` population; optionally `/v1/logs` for events **[contract]** | M | contract |
| 1.6 | Sampling weights + weighted aggregates (**honesty fix**, §9, §1 #9) | tail-sampling keeps/drops but records no weight; roll-ups average kept spans (biased) | Add `sampling_weight: f64` to `CanonicalSpan` (`beater-schema`); stamp `1/keep_probability` on the keep path in `beater-ingest`; make `beater-store` roll-up/aggregate queries weighted (Horvitz-Thompson); label any unweighted view biased **[contract]** | M | contract |
| 1.7 | DatasetCase Train/Dev/Test split + contamination guard (§5.4, §6.4) | `DatasetCase` has no split; no held-out discipline | Add `split: DatasetSplit` (seeded hash off `dataset_version_seed ++ case_id`) to `DatasetCase` (`beater-datasets`/`beater-schema`); min-sample gate; near-dup contamination detection train↔test; gates/RSI read Test-only **[contract]** | M | contract |
| 1.8 | Config-driven MAPPING importer (§7) | hand-written normalizers only; custom/older dialects need a PR | `SourceImporter` config dialect (field-path/span-kind/attr/units mapping) projecting a foreign shape to canonical with no code; `/v1/import` mapping endpoint; raw envelope preserved for re-projection **[contract]** | L | contract |

Acceptance: a multi-turn agent trace groups by session in the API; a vision LLM
call renders its image; full-text search hits prompt/output bodies stored as
artifacts; a stock OTel JSON exporter ingests with no Beater SDK.

### 20.4 Phase 2 — Read APIs & Product UI

Goal: make the eval/observability backend usable as a product, not just POST
endpoints. The dashboard today is one server-rendered trace-waterfall page.

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 2.1 | Dataset CRUD + read APIs | create-only POST; no GET | `DatasetStore` `list_datasets/get_dataset/list_versions/update_case/delete_case/import_cases`; `GET /v1/datasets[...]`, versions, cases; CSV/JSONL import **[contract]** | M | contract |
| 2.1b | Bulk promote cases from query (§21 MCP UX) | one-trace-at-a-time promotion only | `POST /v1/datasets/:id/promote-from-query` taking a span/run filter (§13 search) + target version, materializing matching failures as `DatasetCase`s with seeded `split` (§5.4); the outcome-shaped MCP "promote failures" recipe (§21) calls this **[contract]** | M | contract |
| 2.2 | Eval-report read API | reports only readable inside POST handlers | `GET /v1/datasets/.../eval-reports/{id}`, `.../versions/{vid}/eval-reports` (list+latest), paged per-case results **[contract]** | M | contract |
| 2.3 | Experiment comparison UI (with CIs) | rich backend, no UI | `web/dashboard/app/experiments/[id]` rendering `ExperimentRunReport`: per-case score table, baseline-vs-candidate deltas with `ci_low/ci_high`, gate badge, trace deep-links | L | contract |
| 2.4 | Dataset / eval-result browse UI | none | `web/dashboard/app/datasets[...]` routes: versions, cases, eval drilldown with judge rationale | XL | contract |
| 2.5 | Human annotation queues + inline scoring UI | full `beater-human` backend, no UI | `web/dashboard/app/review` (queue + task inbox) + inline `AnnotationPanel` on span detail posting `submitReviewAnnotation`; keyboard labeling | L | none |
| 2.6 | Failed-vs-passed trace diff | none | `GET /v1/traces/:tenant/:a/diff/:b` aligning spans by name/kind/seq emitting per-span deltas **[contract]**; `web/dashboard/app/diff` side-by-side view | L | contract |
| 2.7 | Cost/latency analytics dashboard | single-run summary strip only | `GET /v1/metrics/:tenant` timeseries (p50/p95/p99, cost/token trends, model/release breakdown) **[contract]**; `web/dashboard/app/analytics` charts | L | contract |
| 2.8 | Search UI + saved views | strong filter form, no full-text UI | `web/dashboard/app/search` + `searchSpansPath()` calling `/v1/search/:tenant/spans`; attribute-predicate query bar; saved views | M | none |
| 2.9 | Client interactivity (live tail, virtualized) | fully server-rendered, GET-form nav | client components (SWR/react-query) over read APIs; SSE/websocket live-tail on `/v1/traces`; virtualized span lists | L | none |

Acceptance: a user can browse datasets, open an experiment and see per-case
deltas with CIs and a gate badge, annotate a trace in a review queue, diff a
failed vs passing trace, and watch cost/latency trends — all in the UI.

### 20.5 Phase 3 — Eval Depth & Statistics

Goal: scorer breadth and statistically defensible experiments.

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 3.1 | Scorer catalog breadth | 10 scorers; `json_object` checks object-ness not schema | Add `FuzzyMatch{min_ratio}` (strsim), `JsonSchema{schema}`, `NumericTolerance{abs,rel}`, `EmbeddingSimilarity{model,min_cosine}` (judge lane), SQL-result match to `EvaluatorKind`/`EVALUATOR_CATALOG` **[contract]** | L | contract |
| 3.2 | Structured-rubric LLM judge | `LlmJudge{rubric:String}` free-text | `JudgeRubric{criteria:[{name,weight,scale}],reference_mode,exemplars}`; `JudgeResponse.per_criterion`; reference-guided + CoT rationale **[contract]** | L | contract |
| 3.3 | Custom scorer registry | WASI sandbox runs components, no upload/registry | `beater-scorers` (or extend `beater-eval`): `ScorerStore` (upload component bytes → `Sha256Hash`, version, list/get) on `beater-store-obj`+sqlite; `/v1/scorers` CRUD **[contract]**; resolve by `wasm_hash` into the sandbox; add memory/epoch limits to `SandboxConfig` | XL | contract |
| 3.4 | Real statistics module (**correctness fix**, §10.3) | single paired normal-approx, **hardcoded z (1.96/2.576), Bonferroni-only, no p-value → nominal alpha ≠ actual alpha** | New `beater-stats` on `statrs`: paired-t / bootstrap-percentile / Wilson CIs; test selection `{PairedT, McNemarExact, WilcoxonSignedRank, Bootstrap}` with real `p_value`; Holm-Bonferroni + Benjamini-Hochberg; `power.rs` (`required_sample_size`, `achieved_power`); **DELETE `compare_paired_scores`'s hardcoded-z path + `StatisticalTest::PairedNormalApproximation`**, route `beater-eval`/`beater-experiments`/`beater-gates` through `beater-stats`. mSPRT/confidence-sequences are the REQUIRED online follow-on (Phase 4, §10.3 #6) | L | none |
| 3.5 | Experiment depth | single metric, no segments | Multi-named-metric + segment tags on `ExperimentRunReport`; `ExperimentStore::list_runs` + `GET /v1/experiments/:tenant/:project` **[contract]**; per-slice comparison | M | contract |
| 3.6 | CI integration | none | `sdks/python/beater/pytest_plugin.py` (`@beater.eval` marker running cases through the API, asserting via `GatePolicy`); TS vitest reporter; `beater eval` gating CLI subcommand | L | contract |
| 3.7 | Agent/score calibration (proper scoring, §10.5) | `beater-calibration` reports kappa/agreement only; the continuous confidence signal is stored then discarded | Persist the probability signal; add Brier + ECE + reliability curve and a persisted, versioned isotonic/Platt recalibration map to `beater-calibration` (kappa demoted to secondary); pin the map into `EvalResult` repro metadata; reliability-curve UI; feeds agent dim #7 (§6.3) on the Test split **[contract]** | M | contract |

Acceptance: an experiment reports a delta with a method-appropriate CI and real
p-value, FWER-corrected across metrics, refusing underpowered comparisons; a
custom uploaded scorer runs sandboxed; `pytest`/`beater eval` fails CI on
regression.

### 20.6 Phase 4 — Online Evals, Alerting & Prompt Management

Goal: production scoring, real alert delivery, and the missing prompt pillar.

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 4.1 | Online evals that score | sampling decision only, never scored | `beater-online` worker (or `beater-temporal` workflow) consuming tail-sampled traces, running configured deterministic+judge evaluators, persisting online-tagged `EvalResult`s (weighted by `sampling_weight`, §9); `GET /v1/online/.../scores` timeseries **[contract]** | XL | design |
| 4.2 | Alert policy persistence + CRUD | policies passed inline; nothing stored | `AlertPolicyStore` (sqlite+sql) + `POST/GET/PATCH/DELETE /v1/alert-policies/...`; persist `OnlineSamplingPolicy` per project; load in `evaluate_alert`/ingest **[contract]** | L | contract |
| 4.3 | Actual webhook delivery | `WebhookDelivery` computed, never sent | delivery worker POSTing with retry/backoff + `beater-security` HMAC signature; persist attempts/status; delivery-history endpoint | M | evidence |
| 4.4 | Slack integration | zero references | `SlackChannel` formatting `AlertInput` into Block Kit (severity, score-vs-baseline, trace deep-link button); stored incoming-webhook config | M | evidence |
| 4.5 | Baseline/anomaly/drift alerting (anytime-valid) | static threshold only | `AlertCondition{AbsoluteThreshold, BaselineDeviation, Drift}` with rolling EWMA/z-score/percentile baseline over recent project scores, decided against a **`beater-stats` confidence sequence (mSPRT)** not a fixed-N test — the REQUIRED online follow-on of §10.3 #6 (peeking a fixed-horizon test inflates false alerts 5–10×) | L | design |
| 4.6 | Durable dedupe/grouping | in-memory `AlertState` | back `AlertState` with the store so dedupe survives restarts + is shared across workers; group rollups in payload | M | none |
| 4.7 | Prompt management | `prompt_version_id` is a dangling pin, no producer | New `beater-prompts`: `PromptRegistry`, versioned `PromptTemplate`, variable schema, tags, diff; `/v1/prompts` CRUD + `runPrompt` (playground) **[contract]**; `web/dashboard/app/prompts` registry + playground + prompt-from-trace; resolve `prompt_version_id` at eval time | XL | contract |

Acceptance: sampled production traces get scored on a schedule with a visible
trend; an alert policy persists, fires on baseline deviation, and is actually
delivered to Slack with a trace link; a prompt can be created, versioned,
diffed, run in a playground, and linked to an eval run.

### 20.7 Phase 5 — Hosted Control Plane & Compliance (Enterprise GA)

Goal: everything required before hosted multi-tenant GA can be sold (see §14, §18 "v3: Hosted GA").

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 5.1 | Orgs/projects/environments CRUD | id types + `TenantScope` only | `POST/GET/DELETE /v1/organizations\|projects\|environments` on `MetadataStore`; membership; org/project switcher in UI/SDK **[contract]** | L | contract |
| 5.2 | Human identity + enforced RBAC | `RoleBinding` data model never consulted by `authorize()` | `beater-rbac` (or extend `beater-auth`): `Role`/`Permission` + `resolve_permissions(principal, scope)` called inside `authorize()` on every mutating route; users + memberships; member/role-grant endpoints **[contract]**; conformance tests | XL | contract |
| 5.3 | SSO / SAML / SCIM / OIDC | none | `beater-identity`: OIDC (auth-code+PKCE) + SAML2 SP + SCIM 2.0 `/Users`/`/Groups`; session/JWT issuance; per-org IdP config; enforced-SSO toggle; JIT provisioning | XL | design |
| 5.4 | Storage-layer tenant isolation + secure default | app-enforced `WHERE tenant_id=?`; default auth effectively open | hosted store on Postgres with Row-Level Security keyed on per-request `SET app.tenant_id`; conformance test proving cross-tenant reads fail at the DB; make `Required` auth the default for non-localhost binds | XL | design |
| 5.5 | Data deletion / crypto-shred / GDPR | no DELETE routes, no erasure | per-tenant data-encryption keys for crypto-shred; `DELETE /v1/tenants/{id}` (key destroy + cascade), `DELETE /v1/traces/{id}`; background purge worker; deletion audit events **[contract]** | XL | contract |
| 5.6 | Data residency / regional | single-region placeholder | `region` on `OrganizationMetadata`; region-aware gateway routing to home-region backends; per-region object/DB stores; EU/US topology doc | XL | design |
| 5.7 | Comprehensive tamper-evident audit | covers exactly one action (`PiiUnmask`) | expand `AuditAction` (key/secret/role/config/export/login/auth-failure); emit from `beater-auth`/`beater-secrets`/RBAC/login; hash-chained tamper-evident column; `GET /v1/audit-events` **[contract]** | L | contract |
| 5.8 | Billing / usage ledger | idempotent ledger exists; no plans/invoicing | meters for ingest/storage/eval/judge; per-org rollups; `beater-billing` (plan/subscription + Stripe metered sync) linked to `QuotaLimiter` | L | contract |
| 5.9 | Backups + restore drills | none | hosted on Postgres+object store with PITR; `beaterctl backup`/`restore` for self-host; CI restore-drill job with documented RPO/RTO | L | evidence |
| 5.10 | SLO dashboards + dogfooding | Prometheus facade exists | Grafana dashboard JSON + Prometheus alert rules under `ops/`; self-trace OTLP exporter so `beaterd` traces into a Beater project; load test producing the §16 numbers | M | evidence |
| 5.11 | Governance / SOC2 controls | LICENSE + GOVERNANCE only | `SECURITY.md` (coordinated disclosure); `docs/compliance/` SOC2 control matrix, access-review runbook, incident-response plan, subprocessor list, DPA template | M | evidence |
| 5.12 | KMS-backed BYOK + at-rest rotation for blobs | ChaCha20 envelope for secrets only | KMS `Keyring` (AWS/GCP CMK wrap) behind `SecretKeyring`; extend envelope encryption to trace I/O blobs + PII fields; concurrency-safe rotation across stores | XL | design |

Acceptance: a non-owner is denied a mutating route by enforced RBAC; SSO login
provisions a user; a cross-tenant query fails at the database; a tenant can be
crypto-shredded and proven unreadable across hot/cold/artifact stores; billing
totals drive quota; a restore drill passes; SLO dashboards show live numbers.

### 20.8 Phase 6 — Auto-Instrumentation & Ecosystem Breadth

Goal: lower adoption friction to match the incumbents' framework coverage.

| # | Requirement | Now | Target / concrete task | Effort | Blocker |
| --- | --- | --- | --- | --- | --- |
| 6.1 | Auto-instrumentation (OpenAI/Anthropic) | one-line `wrap_*` wrappers only | `beater.auto.instrument(providers=[...])` monkeypatching `openai`/`anthropic` (incl streaming + tool calls) in py + ts | L | none |
| 6.2 | Zero-code env-var bootstrap (**DEFAULT onboarding**, §1 #13, §15) | all paths require code | `opentelemetry-distro`/configurator (py) + TS `--require` preload reading `BEATER_*` env, setting OTLP exporter+headers, enabling installed auto-instrumentors; promoted to the documented first path | M | none |
| 6.3 | Modern framework coverage | LangChain (py+ts), LlamaIndex (py) only | examples + instrumentation for Vercel AI SDK (TS), OpenAI Agents SDK, CrewAI, DSPy, Pydantic AI, AutoGen, Haystack; TS LlamaIndex; token-usage extraction; 3-level span-tree integration tests | XL | evidence |
| 6.4 | `beaterctl quickstart` (time to first SCORED FAILURE) | manual compose + snippet | one command boots compose, provisions tenant/key, prints exporter snippet + dashboard URL; timed e2e asserting not just a trace but a *scored failing case* visible < the §15 SLO | M | evidence |

Acceptance: an env-var-only Python app produces traces with zero code edits;
each named framework has a working example emitting a correct agent span tree;
`beaterctl quickstart` demonstrates **time to first scored failure** under the §15
SLO (a failing case shown with a score, not merely a trace rendered).

### 20.9 New Crates, Contracts & Sequencing

New crates introduced by this plan (all under the §4 workspace conventions):

- `beater-bench` — criterion benches + load-test fixtures (Phase 0).
- `beater-stats` — CIs, test selection, p-values, power, FWER/FDR (Phase 3); the
  correctness layer that DELETES the hardcoded-z gate path (§10.3). mSPRT /
  confidence sequences are its required online follow-on (Phase 4, §10.3 #6).
- `beater-scorers` — custom-scorer registry over the WASI sandbox (Phase 3).
- `beater-online` — online-eval scoring worker (Phase 4).
- `beater-prompts` — prompt registry/versioning/playground (Phase 4).
- `beater-rbac` — role/permission resolution wired into `authorize()` (Phase 5).
- `beater-identity` — OIDC/SAML/SCIM (Phase 5).
- `beater-billing` — plans/subscriptions/Stripe metered sync (Phase 5).

**The beat-boxes rename (pre-1.0 follow-up, cross-cutting).** §4 establishes the
beat-themed names now; the **physical directory rename** of each `beater-*` crate to
its beat name is deferred to a pre-1.0 follow-up because crate-path renames touch the
contract and semconv sources (§4 references this task here as "§20.9"). It is a
single regenerated change that must pass Metronome's drift gates (§22.5): rename the
crate dirs + `Cargo.toml` members, then `cargo xtask regen-spec` + `regen-sdks.sh` +
`regen-semconv` and `scripts/check-contract-sync.sh` green in the same commit. Until
then, reach for a component by its `beater-*` path and reason about it by its beat
name.

Sequencing rationale (each phase unblocks the next):

```text
Phase 0  scale & data plane     -> every scale/latency claim depends on it
Phase 1  agent data model       -> sessions/messages/multimodal feed UI + evals
Phase 2  read APIs + product UI -> makes the eval/observability backend usable
Phase 3  eval depth + stats     -> defensible experiments and scorer breadth
Phase 4  online + alerts + prompts -> production loop + the prompt pillar
Phase 5  hosted control plane   -> enterprise multi-tenant GA (gates §18 hosted)
Phase 6  ecosystem breadth      -> adoption parity; can run partly in parallel
```

Cross-cutting bar for every item (no exceptions):

- Contract-touching changes regenerate spec + 7 SDKs + semconv and pass
  `scripts/check-contract-sync.sh` (CI-gated). These need Docker for
  `regen-sdks.sh`.
- Every non-trivial change lands with a runnable test; `cargo clippy
  --all-targets -D warnings` is clean (the workspace denies `unwrap`/`expect`,
  including in tests).
- New scale/perf claims ship with a benchmark or load test, never an assertion.
- Tenant isolation, redaction, and audit are never weakened to ship a feature.

Done, per §19, is when a team can replace ad-hoc Phoenix/LangSmith/Braintrust
workflows end to end. This plan is the path from 33% to that bar.

## 21. Planned: Recursive Self-Improvement (folded into `beater-mcp`)

This is the recursive-self-improvement (RSI) loop layered on the Beater
eval/judge/trace/dataset/replay/stats primitives (§5–§13, §20): an MCP-driven loop
that lets an agent — driven by Claude Code, Cursor, a ChatGPT connector, Codex, or
any MCP client — improve a *target* agent's policy `π` (§6.1) under statistical and
autonomy guardrails. It reuses Beater for traces, evals, judges, datasets, replay,
and statistics; it does not reinvent them.

**Architecture decision: the improvement loop is FOLDED INTO `beater-mcp`, not a
standalone server.** The §20 MCP already exposes every `/v1` operation as a tool;
the RSI tools (`index_agent`, `propose_change`, `simulate`, `apply_change`,
`track_evolution`, `challenge_labels`) are added as *additional* tools on that same
server, sharing its auth, transport, and contract-sync discipline. An MVP
foundation (`beater-credits`, `beater-mcp-improve`) already exists on the
`feat/mcp-improve-foundation` branch, but the architecture now prefers folding
improve into `beater-mcp` over shipping a separate improve server, and **defers**
the standalone Studio / toolbelt / credits productization to a later phase (§21.6,
§21.7). The thesis ("a tool belt that generates tool belts") is retained as a
direction, not a near-term standalone product.

**MCP deployability (required).** The MCP is reachable two ways, with the same tool
set: **stdio** for local clients (Claude Code / Cursor / Codex running on the
developer's machine), and **streamable-HTTP secured by OAuth 2.1** for the hosted
tier so ChatGPT connectors and remote IDEs can connect via `/mcp`. The OAuth 2.1
HTTP endpoints already exist on `main` (`beater-oauth`/`beater-oauth-server` wired
into `beaterd`: `/.well-known/oauth-authorization-server`, `/oauth/authorize`,
`/oauth/token`, dynamic client registration) and the MCP is already served at
`POST /mcp` alongside them; the streamable-HTTP transport reuses exactly that
surface. **stdio is the one transport not yet present and is the concrete MCP
deployability gap to close.**

Design invariants (carried from §1):

- **Human-in-the-loop by default; bounded autonomy when opted in.** The loop runs
  as plan → approve → execute: the MCP indexes the agent, reports what it found
  ("is this correct? which of the §6.1 levers are you OK changing?"), and only then
  iterates. Autonomy is opt-in and **bounded** — spend and confidence bounds, with
  **repo writes OFF by default** (§21.5 bounded-autonomy policy).
- **Generalize, do not overfit — accept only on the frozen Test split.** A change
  is accepted only on the untouched **Test** split (§5.4, §6.4) clearing a real
  confidence interval *and* power bar (§10.3). The loop is policy-aware: load-
  bearing prompts/tools are not changed unless contradictory. There is **no
  "gradient"**: the loop runs *sequential evaluation* gated on a real CI, not a
  differentiable score signal.
- **Standards + reuse at the edge.** Scoring is Beater's existing LLM-judge +
  deterministic WASI evals; statistics are `beater-stats`; memory/tools are
  provisioned, not hand-rolled.
- **MCP-first, SDK-second.** Recommend the MCP to learn the workflow, then expose
  a deterministic SDK for repeatable monitoring/improvement pipelines.

### 21.1 The improvement tools (added to `beater-mcp`)

The RSI tool-belt is a set of tools on the existing `beater-mcp` server (not a
separate binary). Every tool call is a metered self-improvement action (see §21.6).
Core tools:

- `index_agent` — discover the agent's code, config, system/UI/customer prompts,
  policy, tools, and runtime (localhost, API logs, browser) and build a map from
  symptom → corresponding code/prompt/data.
- `propose_change` — given a goal + traces + evals, propose a typed change. The
  change set is one enum, `ChangeKind`, used by `propose_change`, `apply_change`,
  and `track_evolution`:

  ```text
  ChangeKind =
    | SystemPrompt | CustomerPrompt        // the prompt levers of π (§6.1)
    | Code                                  // agent code
    | ToolAdd | ToolRemove                  // the tool_set lever of π (§6.1)
    | MemoryConfig                          // the memory lever of π (§6.1)
    | ModelParams                           // the model-params lever of π (§6.1)
    | DataLabel                             // NOT a π lever — challenges a dataset
                                            //   label (dim #16, see challenge_labels)
  ```

  Each proposal carries a rationale and the exact file/symbol/span it targets.
  Returns a plan, never a silent edit.
- `simulate` — run N candidate iterations through Beater's harness (§12) on the
  **Train** split (and Dev for tuning), scoring with the frozen evaluator
  (LLM-judge + deterministic WASI), and return a **typed reward estimate** (§21.2:
  verifier gain vs judge gain, position-bias-cancelled) **with a `beater-stats`
  confidence interval** — *not* a "score gradient." `simulate` answers "is this
  change worth proposing to the Test gate?", it never decides acceptance.
- `apply_change` — wire the approved change at a chosen integration depth
  (suggest-only → wire a node → edit repo code), collaborating with Claude Code for
  the actual code write. **Repo writes are OFF by default** and a write is
  materialized to the repo **only after a held-out Test win** clears §10.3 (§21.5).
- `track_evolution` — record the agent's version history (tools added/removed,
  prompts rewritten, labels challenged) so the loop can see its own trajectory.
- `challenge_labels` — flag dataset labels the evidence contradicts; route to the
  human grader (§21.5).
- `suggest_scorers` — **advisory**: given the indexed agent + its traces, suggest
  an archetype ("RAG agent", "tool-using planner", "browser agent") and a starter
  set of §10.4 scorers/dimensions (§6.3) to measure it. Outcome-shaped advice, not
  an API call the user must assemble.

**Composite MCP tools (named recipes over operation-ids).** On top of the raw
per-operation tools (§20), `beater-mcp` exposes a small set of **outcome-shaped
composite tools** — named recipes that chain several `/v1` operations so the client
asks for an *outcome*, not an API sequence. Examples: `promote_failures`
(query failing traces → bulk `promote-from-query`, §20.4 #2.1b → assign Test
split) and `gate_candidate` (run experiment → `beater-stats` CI on Test → return
pass/inconclusive/fail with the interval). Recipes are versioned tools resolved
from the spec like everything else, so they stay in contract-sync.

### 21.2 The typed reward model (no gradient)

The loop needs a *reward* to optimize `J(π)` (§6.2). It is **not** a scalar
"gradient" — that framing is deleted because there is nothing differentiable here
and a single collapsed score is exactly what Goodhart exploits. The reward is a
**typed decomposition** with a `beater-stats` interval on each component:

```text
Reward(change) = {
  verifier_gain : Δ on DETERMINISTIC scorers (WASI lane, §10.1/§10.4) — trusted
                  where state is known-correct (exact/regex/schema/numeric/cost/
                  tool-execution/SQL-result). High-trust, cheap, reproducible.
  judge_gain    : Δ on JUDGE-lane scorers (§10.1.1) — needed for open-ended
                  quality, but noisy/biased, so always position-bias-cancelled
                  and reported with its CI.
  per_dimension : the §6.3 dimension vector (success, cost, latency, safety,
                  calibration, ...) — NOT collapsed; a safety/guardrail
                  regression vetoes regardless of verifier/judge gain.
}
```

Why typed: a change that lifts the *judge* score while the *verifier* score is flat
or down is the classic "the model talked the judge into it" failure; separating
verifier gain from judge gain makes that visible instead of hiding it in one number.
The verifier component is weighted higher precisely because it is the
harder-to-game, deterministic signal.

**Position-bias cancellation via the A/B order swap.** Every pairwise judge
comparison feeding `judge_gain` is run in **both** A/B orders and reconciled, so the
known position bias of LLM judges (§10.1.1) is **cancelled structurally**, not
assumed away (§1 #10). This is the same swap §10.1.1 mandates; here it doubles as
the order-bias control on the reward signal. Each reward component carries a
`beater-stats` CI, and acceptance (§21.3) reads these on the **Test** split only.

### 21.3 The Self-Improvement Loop (sequential, CI-gated)

```text
goal + params + few examples
  -> index_agent (code + prompts + policy + runtime)
  -> collect traces/evals (Beater) + classify failures
  -> propose_change (typed §6.1 lever, goal-targeted, generalizable)
  -> simulate on TRAIN/Dev (judge + deterministic) -> typed reward + CI (§21.2)
  -> human approve (which changes; autonomy bounds, §21.5)
  -> evaluate on the untouched TEST split -> beater-stats CI + power check (§10.3)
  -> ACCEPT iff Test CI clears the bar AND no safety dimension regresses
  -> only then apply_change (materialize to repo, §21.5) + record
  -> track_evolution -> repeat (stop on §6.2 convergence/budget)
```

**The optimization strategy, named concretely.** This is **reflective proposal
selection with sequential, CI-gated acceptance** — *not* gradient descent and *not*
(yet) population search. Each round the proposer reflects on the indexed agent +
classified failures (§21.1) to emit a small set of typed candidate changes; because
the objective is the multi-dimensional §6.3 vector, candidates are ranked by
**Pareto-style dominance** on the typed reward (§21.2) — a candidate that improves
some dimensions without regressing any guardrail dimension dominates, and ties are
broken toward the higher-trust **verifier_gain** over the noisier **judge_gain**.
The single best non-dominated candidate is then put to the **sequential acceptance
gate**: simulate on Train/Dev for a typed-reward CI, then evaluate on the **untouched
Test split**, accepting iff the Test CI clears §10.3's significance *and* power bar
with no safety regression (the §6.2 accept/reject rule). Rejected candidates inform
the next reflection round; the loop is single-candidate-sequential, with full
population/evolutionary search deferred (§21.5c).

This is **sequential evaluation gated on a real confidence interval over the frozen
Test split** — propose/simulate read Train (Dev for tuning), acceptance reads the
untouched Test split (§5.4, §6.4), and a *pass* requires a real `beater-stats`
p-value at adequate power (§10.3), never a raw mean delta. Deterministic evals are
trusted where state is known-correct; the judge component is position-bias-cancelled
and CI'd. Anti-overfit, the frozen evaluator (§6.2), and policy-awareness gate every
accepted change.

### 21.x Anti-Overfitting & Generalization Guardrail — [research-backed spec pending: smoothness/over-optimization detection + auto-generated OOD probes]

### 21.4 Integrations & Code-Awareness

- **Runtime introspection:** aware of where localhost runs; can open the browser,
  read API logs from the user's codebase, and locate the responsible stack layer.
- **Frameworks:** direct link to browser-use; Temporal (sub-agent trace steps map
  cleanly to canonical spans); LangChain / LangGraph. Auto-discover internal
  workflows and classify their traces into improvement candidates.
- **Integration depths:** (1) suggest-only, (2) wire a node (Studio, deferred —
  §21.5b), (3) change actual repo code — chosen per change. Depth (3) is gated by
  the bounded-autonomy policy (§21.5) and a held-out Test win.

### 21.5 Bounded-autonomy policy

Autonomy is opt-in and **bounded**; the loop never silently rewrites a repo. The
policy is a hard guard around `apply_change`:

- **Repo writes OFF by default.** The default integration depth is suggest-only
  (§21.4). A repo write requires an explicit opt-in *and* satisfies the conditions
  below; until then the loop produces plans and simulated/Test results, not edits.
- **Materialize only after a held-out win.** An accepted change is written to the
  repo **only after** its improvement on the untouched **Test** split clears the
  §10.3 confidence-bound *and* power bar with no safety-dimension regression (§6.2,
  §21.3). A simulate-only (Train/Dev) win is never sufficient to write code.
- **Spend bound.** Each episode runs under a budget (AI-credits / model spend, §21.6)
  enforced by `QuotaLimiter` (§8.4); exhausting it stops the loop (a §6.2
  convergence criterion), it does not silently overspend.
- **Confidence bound.** A change below a configured confidence threshold on its
  typed reward (§21.2) is not auto-applied even within budget; it is surfaced for
  human approval. Full autonomy raises the bound, it does not remove it.
- **Frozen evaluator + policy constraints.** The ruler does not move during an
  episode (§6.2) and load-bearing prompts/tools are not changed unless contradictory.

Together these make the autonomous mode *bounded* — it can spend up to a budget,
act only above a confidence bound, and touch the repo only after a real held-out
win — rather than an open-ended self-rewriting agent.

### 21.5b Deferred: Agent Studio (`beater-studio`)

**Deferred — design-only, idea preserved, not a near-term product.** A visual
surface that maps front-end ↔ back-end. Kept here as direction; it is *not* on the
critical path and is part of the deferred standalone-Studio productization (§21.6):

- **Canvas** (Excalidraw-style, mostly native): agent design auto-drawn as nodes,
  **topologically sorted left→right**, with explicit visualization of recursive
  self-improvement loops.
- **JSON-schema-first:** every node/edge is backed by JSON schema stored in the
  backend; Claude Code assists with the schema via the MCP. A canonical
  "good workflow" example + a skills doc the MCP/Claude Code pull from.
- **Studio mode:** watch the agent run, see traces live, drag tools in; Claude
  Code wires them (AI tier: a hosted agent wires them).
- **Human grading:** an expert feedback area to grade right/wrong inline, feeding
  `challenge_labels` and calibration (§10.5).

### 21.5c Deferred: auto-provisioned tool-belt (`beater-toolbelt`)

**Deferred — design-only, idea preserved.** OAuth in, and the platform
auto-provisions agent capabilities on demand (the "pop-up" experience): one-click
managed **vector memory**, **SQL store**, **web search**, **scrapers**, addressable
by `propose_change`/`apply_change` and metered. Also deferred and *not deleted*: a
**skill library on a vector store** and **full evolutionary/population search over
agent configs** (the §21.3 loop ships as a single-candidate sequential search first;
population search is a later generalization). These are future generalizations of
the loop, not MVP.

### 21.6 Commercial Model & Metering (DRAFT — design-only, productization DEFERRED)

**Status: the commercial model is kept as a design, but the standalone Studio +
toolbelt + credits *productization* is DEFERRED to a later phase.** An MVP
foundation (`beater-credits`, `beater-mcp-improve`) already exists on the
`feat/mcp-improve-foundation` branch; the architecture now prefers folding
improvement into `beater-mcp` (§21) and treating credits as a later commercial
layer rather than a launch dependency. The numbers below are illustrative, not a
committed price sheet.

**Bill on VERIFIED GAIN / an autonomy budget, not raw tool-call effort.** The key
refinement over the original "count every MCP tool call" model: charging for effort
rewards the platform for *churning* (more simulate calls = more revenue) and
punishes the user for the loop's own inefficiency. Instead the primary commercial
meter is an **autonomy budget** spent toward **verified improvement** — credits are
consumed against AI/model spend within an episode, and the value narrative is
"credits spent per accepted, held-out-verified gain" (§21.2 typed reward on the
Test split), not per tool invocation. Raw tool-call counts remain a *rate-limit /
abuse* signal, not the value meter.

Two dimensions:

- **Autonomy budget (AI credits)** — model spend (judge + code-writer) inside an
  episode, bounded per the §21.5 spend bound; this is what the user is really
  buying (verified gains), with episodes that fail to clear the Test gate costing
  the platform's margin, not silently the user.
- **Rate-limit requests** — MCP tool calls / endpoint calls, used to bound abuse
  and smooth bursts, **not** as the primary value meter.

| Plan (illustrative) | Price | Rate-limit requests/mo | Included AI credits | Overage |
| --- | --- | --- | --- | --- |
| Free | $0 | 5,000 | $5 | — |
| Starter | $8/mo | 8,000 | — | — |
| Pro / AI | $20/mo | 50,000 | $40 | pay-as-you-go credits |
| Usage (AI) | metered | — | per plan above | pay-as-you-go |

**Rolling-window rate limiting (Claude-Code/Codex-style).** On top of monthly caps,
both tiers enforce **rolling 5-hour and weekly windows** computed from a
multi-factor cost (tool-call count, tokens, model tier, simulation depth), so bursty
usage is smoothed and abuse is bounded without a hard monthly cliff. Windows reset
continuously (seek-based), not on calendar boundaries.

Requires (when productized): a metering/credits service (`beater-credits`, MVP
exists on branch) over the existing `beater-usage` ledger (§10 usage records) +
`QuotaLimiter` (§8.4) with rolling 5h/weekly windows, plan tiers, and Stripe metered
billing (ties into §20.7 5.8). Until productization, the §21.5 spend bound is
enforced directly through `QuotaLimiter` without the commercial layer.

### 21.7 Crates & SDK

- **`beater-mcp` (CHANGED, primary)** — the improvement tools (§21.1) and composite
  recipes are added here; the loop is *not* a separate server. stdio + streamable-
  HTTP/OAuth 2.1 transports (§21 intro).
- **`beater-stats` (NEW, §10.3)** — supplies the CI/p-value/power the loop gates on.
- **`beater-replay` (CHANGED, §11)** — forked replay backs `simulate` and root-cause.
- **`beater-mcp-improve` (DEFERRED / branch foundation)** — exists on
  `feat/mcp-improve-foundation` as the MVP; its logic folds into `beater-mcp` rather
  than shipping standalone.
- **`beater-credits` (DEFERRED / branch foundation)** — metering exists on branch;
  productization deferred (§21.6).
- **`beater-toolbelt` (DEFERRED, §21.5c)**, **`beater-studio` (DEFERRED, §21.5b)** —
  design-only, ideas preserved.
- Deterministic **improvement SDK** (py/ts) over the same endpoints for repeatable
  monitoring/improvement pipelines (later phase).

### 21.8 Phasing & Acceptance

- **MVP:** the improvement tools on `beater-mcp` — `index_agent`/`propose_change`/
  `simulate`/`apply_change` — wired to Beater evals/judge/harness/`beater-stats`,
  plan→approve→execute, **repo writes off by default**, accept only on a held-out
  Test win. Acceptance: from a goal + a small agent (system prompt + policy), the
  MCP indexes it, proposes a generalizable change, simulates a typed reward with a
  CI on Train/Dev, **verifies a statistically significant win on the untouched Test
  split**, and only then applies it via Claude Code with human approval.
- **+1:** browser-use/Temporal integration; stdio transport for local clients.
- **+2 (deferred):** auto-provisioned tool-belt (vector/SQL/web); Studio canvas
  (topo-sorted nodes, JSON schema, live traces, drag-to-add) + human grading.
- **+3 (deferred):** deterministic SDK, LangGraph integration, credits/billing tiers
  GA; later still, population/evolutionary search and a skill library (§21.5c).

This loop depends on Phases 0–4 of §20 (scale, data model, read APIs, evals/stats,
online evals) being far enough along that traces and evals are real inputs to it.

### 21.9 Assumptions & how they are validated (consolidated)

Every quantitative claim in this document rests on assumptions. They are stated
in context (§6.5, §10.3, §10.4, §10.5, §11), but a reader should be able to see
**all of them in one place**, paired with **how each is checked or relaxed** and
**the §22 test that enforces it** — so there is no dangling or implicit
assumption anywhere in the platform. If an assumption cannot be satisfied for a
given comparison, the affected estimator/gate **refuses to decide** (returns
*inconclusive* / "no single-span root cause" / "biased, unweighted") rather than
emitting a wrong number.

| # | Assumption | Where it lives | How it is checked | How it is relaxed when violated | §22 test that enforces it |
| --- | --- | --- | --- | --- | --- |
| A1 | **Cases are i.i.d.** (independent observations) | §6.5, §10.3 #1 | declare a cluster id per case; detect multi-turn/shared-template groups | **clustered standard errors** (§10.3 #1); coarsen clusters if clusters are themselves correlated | `beater-stats` unit: clustered SE on a correlated fixture is wider than naive SE; gate refuses if no cluster id where one is required |
| A2 | **CLT/normal interval is valid** (large N, unbounded, symmetric) | §10.3 #2 | metric type + N check before choosing an interval | use **Wilson** (binary) / **bootstrap** (bounded/continuous); CLT only when its preconditions hold | unit: Wald/normal path is *deleted*; a property test asserts Wilson/bootstrap coverage ≈ nominal at small N |
| A3 | **Significance test matches the metric** (normality / pairing / symmetry) | §10.3 #3 | test-selection records which satisfied assumption justified the choice | auto-select McNemar/Wilcoxon/bootstrap when t-test normality fails | unit: each branch of the §10.3 #3 table selected on its matching fixture; mismatched fixture falls back to bootstrap |
| A4 | **Nominal alpha = actual alpha** (the gate's stated error rate is its true rate) | §1 #9/#11, §10.3 | the hardcoded-z path is deleted; real p-values from `beater-stats` | n/a — this is an invariant, not a relaxable assumption; gate is wrong if violated | conformance: simulate many null comparisons, assert empirical false-positive rate ≈ alpha (calibration test) |
| A5 | **Multiplicity is controlled** across metrics/slices | §10.3 #4, §6.3 | count comparisons in an experiment; apply Holm-Bonferroni (FWER) or BH (FDR) | switch FWER↔FDR by goal; never raw alpha division | unit: 16-dimension multi-metric fixture (§6.3) shows corrected vs uncorrected false-win rate |
| A6 | **The comparison is adequately powered** | §10.3 #5 | `achieved_power(n,effect,alpha)` before any *pass* | return **inconclusive**, never *pass*, when underpowered; `required_sample_size` tells the user how many more cases | gate test: an underpowered fixture returns `inconclusive`; CI green never means "too few cases" |
| A7 | **Fixed-horizon test is being read once** (no peeking) | §10.3 #6, §13 | online/continuous path flagged distinct from offline | use **anytime-valid mSPRT / confidence sequences** on any continuously-peeked stream | online test: a continuously-peeked stream using a fixed-N test inflates FP ≫ alpha; confidence-sequence path holds coverage |
| A8 | **Observations are bounded / sub-Gaussian** (needed for A7) | §10.3 #6 | 0–1 eval scores satisfy this automatically | restrict anytime-valid claims to bounded metrics; refuse on unbounded heavy-tailed signals | unit: confidence-sequence validity asserted on 0–1 scores |
| A9 | **Stationarity** — agent/judge/providers stable over the window | §6.5, §10.1.1 | re-run a fixed canary set over time; watch kappa/score/ECE drift | recalibration triggers on model deprecation/provider drift; freeze evaluator within an episode (§6.2) | integration: canary drift detector fires a recalibration trigger on injected drift |
| A10 | **Judge calibration is valid** (judge reads right) | §6.5, §10.1.1, dim #5 | periodic judge-vs-human agreement (Cohen's kappa) + Wasserstein calibration | re-fit `F_human`/`F_model`; demote/abstain dimensions if reference set is stale/tiny | calibration test: kappa/agreement report persisted; stale reference set flags affected dimensions |
| A11 | **Probability signals are calibrated** (0.8 means ~80%) | §10.5, dim #7 | Brier + ECE + reliability curve on held-out Test | persisted isotonic/Platt **recalibration map** `c(p)→p'`, versioned into `EvalResult` | unit: ECE improves after applying the recalibration map on a miscalibrated fixture |
| A12 | **Dataset is representative** of the deployment distribution | §6.5, dim #15 | generalization-gap monitoring (dim #15); online vs offline score comparison (§20.6) | power/MDE planning refuses tiny/biased sets; flag if holdout−train gap CI excludes 0 | integration: offline estimate vs production score-distribution divergence raises a representativeness flag |
| A13 | **Held-out Test is uncontaminated** (no leakage into prompts/few-shot/memory) | §5.4, §6.4, §6.2 | seeded-hash Train/Dev/Test split; near-dup overlap detection train↔test; min-sample gate | rotate/refresh holdout if compromised; acceptance reads Test only | conformance: contamination guard rejects a near-dup leaked into Test; gate accepts only on untouched Test |
| A14 | **Evaluator is frozen during an optimization episode** | §6.2, §6.4, §21.3 | pin judge model, rubric (locked JSON), scorers, and split for the episode | n/a — invariant; a moved ruler invalidates the episode | RSI test: mutating the rubric mid-episode aborts the episode rather than scoring against a moved ruler |
| A15 | **Determinism = caching, not `temperature=0`** | §1 #9, §11 | request-hash judge cache + provider/tool/clock/seed cassettes; hashes must match | label `forked_replay`/`simulation` honestly when cassettes are absent/mismatched | replay test: deterministic_replay requires hash match; missing cassette downgrades the mode label |
| A16 | **Tool correctness is execution-based, not syntactic** | §10.4, dim #3 | score the tool call by *executing/replaying* it against a seeded env, check the effect | refuse a syntactic-only score; require a seeded/replayable tool environment | unit: a syntactically valid but semantically wrong call scores 0; a differently-shaped correct call scores 1 |
| A17 | **Trajectory quality is jointly modeled** (not a mean of independent steps) | §10.4, dim #2 | process-reward scorer over the span sequence; trajectory = cluster (A1) | aggregate with trajectory-clustered SE; never average independent per-step scores | unit: per-step independent scoring vs joint scoring diverge on a shared-context fixture; clustered SE used |
| A18 | **Root cause = earliest outcome-flipping span**, not first error | §11 | counterfactual forked-replay search, earliest-first, bounded by a fork budget | return "no single-span root cause" when no single correction flips the outcome | replay test: a trace that fails with no errored span still yields the causal fork point; first-error heuristic would miss it |
| A19 | **Aggregates over production traffic are unbiased** | §1 #9, §9, §10.3 | carry `sampling_weight = 1/keep_probability`; Horvitz-Thompson weighted estimates | label any unweighted production view **biased**; offline balanced datasets use weight 1.0 | unit: weighted vs unweighted aggregate diverge on a tail-sampled fixture; weighted matches the population rate |
| A20 | **Tenant isolation holds** (no cross-tenant read/write) | §14, §20.7 #5.4 | app-enforced `WHERE tenant_id=?` today; Postgres RLS on `SET app.tenant_id` (hosted) | n/a — invariant; a crossover is a security bug (SECURITY.md scope) | conformance: cross-tenant read/write fails (store-conformance today, at the DB under RLS for hosted) |

The payoff: every number the platform ships traces back to a row here, and every
row names the test in §22 that fails if the assumption is silently broken.

## 22. Testing, Verification & Acceptance

This section is the actionable test plan. It is written so a developer can (a)
know **what tests to write** for each major component, (b) run a **"how to verify
it's running" check** at any moment, and (c) see **every §20/§21 plan item mapped
to its acceptance test plus a verification command and the CI gate that enforces
it.** Commands assume `beaterd` on `:8080` (API) / `:4317`/`:4318` (OTLP) and the
dashboard on `:3000`, matching §3.1. `[built]` commands run against `origin/main`
today; `[planned]` commands are the acceptance check for an unbuilt item and are
marked.

### 22.0 The five test layers

| Layer | Purpose | Tooling |
| --- | --- | --- |
| **unit** | one function/estimator/scorer in isolation | `cargo test` / `cargo nextest`, `proptest`, `insta` snapshots |
| **integration** | one crate boundary wired to a real store/bus | `cargo test` with `testcontainers`, SQLite temp dirs |
| **conformance** | the *same* suite run against every backend impl | `beater-store-conformance` (trait suite) |
| **e2e** | live `beaterd`: ingest → query → eval → gate → UI | `beaterctl` fixtures, `gate1-live-smoke`, `gate2-*`, Playwright |
| **load / bench** | throughput + p95 SLO evidence | `beater-bench` (criterion), `xtask loadgen` **[planned]** |

A change without the right layer is incomplete: a scale claim needs load, an
estimator needs unit + a calibration test, a contract change needs the
`sdk-contract` gate, a UI change needs Playwright.

### 22.1 Per-component test plan + "how to verify it's running"

For each component: the concrete tests to write, then a runnable verification.

**Ingest (`beater-ingest`, `beater-otlp`, `beater-temporal`).**
*Tests:* unit — OTLP/OpenInference/GenAI → canonical normalizer golden fixtures
(both dialects); raw-envelope immutability + payload-hash; idempotent duplicate
ingest; quota 429 with retry/reset headers; tail-sampling stamps
`sampling_weight = 1/keep_probability` (A19). integration — buffered trace-write
survives worker kill/restart, DLQs on store outage, replays to a
readable/searchable trace; no-silent-drop accounting across error/DLQ/recovery.
e2e — OTLP HTTP **and** gRPC trace becomes queryable and searchable.
*Verify it's running:* `[built]`

```bash
curl -fsS http://127.0.0.1:8080/health
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080            # OTLP round-trip + query lag
cargo run -q -p beaterctl -- ingest-outage-fixture --data-dir /tmp/beater-io  # no silent drop
```

**Store (`beater-store*`).**
*Tests:* the `beater-store-conformance` suite is the contract — run it against
in-memory, SQLite, and (as wired, §20.2 #0.1) Postgres/ClickHouse: org/project/
environment/RBAC boundaries, shared fixed-window quotas, trace write/read,
**cross-tenant read/write fails (A20)**. integration — keyset pagination +
pushdown (§20.2 #0.2). load — `beater-bench` write/query p95 on 1M/10M-span
fixtures (§20.2 #0.3).
*Verify:* `[built]` `cargo test -p beater-store-conformance --workspace` (the
`storage-backends` CI gate runs this); `[planned]`
`cargo run -p beaterd -- --trace-store clickhouse` boots and serves.

**Evals / judge (`beater-eval`, `beater-judge`, `beater-sandbox`, `beater-scorers`).**
*Tests:* unit — each §10.4 scorer on a passing + a failing fixture, **plus its
invalid-when case** (e.g. execution-based tool correctness scores a
syntactically-valid-but-wrong call 0, A16; trajectory joint vs independent
diverge, A17); WASI sandbox denies network/host imports, enforces fuel/memory/
epoch limits. integration — judge broker preflight budget reservation,
request-hash cache hit (determinism = caching, A15), SQLite audit ledger,
idempotent usage metering. e2e — deterministic + judge-backed dataset eval.
*Verify:* `[built]`

```bash
cargo run -q -p beaterctl -- judge-fixture --data-dir /tmp/beater-judge          # encrypted BYOK + cached judge + ledger
cargo run -q -p beaterctl -- judge-dataset-fixture --data-dir /tmp/beater-jds    # judge eval over a dataset version
```

**Statistics (`beater-stats`, §10.3).**
*Tests:* the assumption table A2–A8 each map to a unit/conformance test —
Wilson/bootstrap coverage at small N (A2); test selection per §10.3 #3 (A3);
**the calibration test: many null comparisons ⇒ empirical false-positive rate ≈
alpha (A4)**; Holm-Bonferroni/BH false-win control (A5); underpowered ⇒
`inconclusive` not `pass` (A6); confidence-sequence validity under continuous
peeking (A7/A8). A regression test asserts the deleted hardcoded-z path and
`StatisticalTest::PairedNormalApproximation` do **not** exist.
*Verify:* `[planned]` `cargo test -p beater-stats` (alpha-calibration +
power-refusal tests); until then the gate uses the deleted-and-wrong path and
must not be trusted (§10.3).

**Experiments + gates (`beater-experiments`, `beater-gates`).**
*Tests:* integration — baseline-vs-candidate per-case scores; gate report
snapshots policy/comparison/decision; gate **fails** on a real confidence-bound
regression and returns **inconclusive** when underpowered (A6); accepts only on
the frozen Test split (A13/A14). e2e — judge-backed candidate-vs-baseline gate.
*Verify:* `[built]`

```bash
cargo run -q -p beaterctl -- judge-experiment-fixture --data-dir /tmp/beater-jx  # judge-backed gate
cargo run -q -p beaterctl -- gate-run-fixture --data-dir /tmp/beater-gate        # CI gate passes on no-regression
! cargo run -q -p beaterctl -- gate-run --data-dir /tmp/beater-gate \
    --tenant-id demo --project-id demo --gate-id main                            # gate BLOCKS a regression (non-zero exit)
```

**Calibration (`beater-calibration`, §10.5).**
*Tests:* unit — judge-vs-human agreement + Cohen's kappa (A10); Brier/ECE +
reliability curve; **ECE improves after applying the persisted isotonic/Platt
recalibration map (A11)**; the map is versioned into `EvalResult` repro metadata.
*Verify:* `[built]`
`cargo run -q -p beaterctl -- calibration-fixture --data-dir /tmp/beater-cal`
(kappa/agreement today; Brier/ECE/map is §20.5 #3.7 `[planned]`).

**Replay & attribution (`beater-replay`, §11).**
*Tests:* unit — `deterministic_replay` requires cassette hash match, missing/
mismatched cassette downgrades the mode label honestly (A15); **forked-replay
search finds the earliest outcome-flipping span even when no span errored, and
returns "no single-span root cause" when no single correction flips it (A18)**.
*Verify:* `[built]`
`cargo run -q -p beaterctl -- replay-fixture --data-dir /tmp/beater-replay`
(persisted-cassette replay with no live calls; real forked-replay search is §11
`[planned]`).

**MCP (`beater-mcp`).**
*Tests:* unit — `tools/list` is deterministic and covers every `/v1` operation +
composite recipes; `tools/call` routes to the right handler; the synthetic help
tool. integration — streamable-HTTP `/mcp` behind OAuth 2.1; **stdio transport
(the §21 gap)**. conformance — MCP tool set stays in sync with the spec (the
`sdk-contract` discipline).
*Verify:* `[built]` streamable-HTTP:

```bash
curl -fsS -X POST http://127.0.0.1:8080/mcp \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | head
```

`[planned]` stdio: `beaterd mcp --stdio` then a `tools/list` JSON-RPC frame on
stdin returns the same tool set.

**SDKs (7 generated clients + native Rust, §3.3).**
*Tests:* the `sdk-contract` gate — spec ↔ served routes, spec ↔ each of the 7
clients, semconv ↔ 5 SDKs, `oasdiff` breaking-change check. live conformance —
each client round-trips a real call against `beaterd`.
*Verify:* `[built]`

```bash
scripts/check-contract-sync.sh        # zero drift across spec/clients/semconv/MCP/CLI/docs
scripts/check-openapi-drift.sh        # spec ↔ served routes only
scripts/e2e-clients-live.sh           # live SDK conformance (needs Docker + toolchains)
```

**Agent-model dimensions (§6.3).**
*Tests:* each of the 16 dimensions ships with a fixture exercising its estimator
+ CI + its stated assumption — e.g. dim #2 trajectory uses trajectory-clustered
SE (A17); dim #3 is execution-based (A16); dim #7 reads ECE/Brier on Test (A11);
dim #15 flags a generalization gap whose CI excludes 0 (A12); improving multiple
dimensions at once triggers FWER/FDR control (A5).
*Verify:* `[planned]` `cargo test -p beater-eval dimensions::` exercising the
§6.3 estimator vector against a seeded multi-dimension fixture.

**RSI loop (`beater-mcp` improvement tools, §21).**
*Tests:* integration — plan→approve→execute with **repo writes off by default**;
`propose_change` returns a typed plan never a silent edit; `simulate` reads
Train/Dev and returns a typed reward + CI, never decides acceptance; **acceptance
reads the untouched Test split and requires a real `beater-stats` p-value at
power (A6/A13)**; mutating the evaluator mid-episode aborts the episode (A14);
spend/confidence bounds enforced by `QuotaLimiter` (§21.5).
*Verify:* `[planned]` an MCP `gate_candidate` recipe over a small seeded agent
returns `pass`/`inconclusive`/`fail` with the interval, and `apply_change`
refuses to write the repo without a held-out Test win.

**Hosted control plane (§20.7).**
*Tests:* conformance — cross-tenant query fails at the DB under Postgres RLS
(A20); enforced RBAC denies a non-owner a mutating route; SSO login JIT-provisions
a user; crypto-shred makes a tenant unreadable across hot/cold/artifact stores;
restore drill meets documented RPO/RTO; audit is hash-chained tamper-evident.
*Verify:* `[planned]` `cargo run -p beaterd -- --auth-mode required` then an
unauthorized mutating call returns 401/403; a cross-tenant read returns empty/403.

**Self-observability (§16).**
*Tests:* the `/metrics` Prometheus facade exposes ingest success, ingest→queryable
lag, DLQ age, query p95; a load run produces the §16 numbers.
*Verify:* `[built]` `curl -fsS http://127.0.0.1:8080/metrics | head`.

### 22.2 One-command local verification (the developer smoke loop)

The fast "is everything healthy" sequence, all `[built]`:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings    # unwrap/expect denied
cargo test --workspace                                    # or: cargo nextest run --workspace
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
cargo run -q -p beaterctl -- judge-fixture --data-dir /tmp/beater-judge
cargo run -q -p beaterctl -- gate-run-fixture --data-dir /tmp/beater-gate
scripts/check-contract-sync.sh
```

The containerized self-host equivalent: `scripts/smoke-compose.sh`. The
clean-clone-to-browser proof: see the README "Clean Clone To Browser" path
(enforced by `gate2-proof-contract`).

### 22.3 Plan item → acceptance test → verification command → CI gate

Every §20/§21 item maps to a concrete acceptance test and a verification command.
The CI gate is the workflow that blocks merge if the item regresses.

| Item | Acceptance test | Verification command | CI gate |
| --- | --- | --- | --- |
| §20.2 #0.1 columnar store wired | `beaterd --trace-store clickhouse` boots + serves; non-ignored compose integration test | `cargo run -p beaterd -- --trace-store clickhouse && curl /health` `[planned]` | `storage-backends` |
| §20.2 #0.2 pagination/pushdown | keyset cursor + `LIMIT` pushed to SQL; `query_runs` is a backend `GROUP BY` | conformance test asserts no in-memory full-scan `[planned]` | `storage-backends` |
| §20.2 #0.3 query p95 SLOs | criterion bench on 1M/10M-span fixtures meets §16 p95 in CI | `cargo bench -p beater-bench` `[planned]` | `backend` (bench gate) |
| §20.2 #0.4 retention/TTL | sweeper demotes-then-deletes expired hot rows | retention integration test `[planned]` | `backend` |
| §20.2 #0.5 cold archival | partitioned Parquet to object store; DataFusion read path | archive round-trip test `[planned]` | `backend` |
| §20.2 #0.6 backend-agnostic migrations | `Migrator` runs on ClickHouse/Postgres; `xtask renormalize` reprojects raw | migration checksum test per backend `[planned]` | `storage-backends` |
| §20.3 #1.1 sessions **[contract]** | multi-turn trace groups by session in the API | `curl /v1/sessions` + normalizer golden test `[planned]` | `sdk-contract` |
| §20.3 #1.2 structured message I/O **[contract]** | OpenInference/`gen_ai` messages parse into `CanonicalMessages` | golden fixture both dialects `[planned]` | `sdk-contract` |
| §20.3 #1.3 multimodal **[contract]** | a vision LLM call renders its image | media-artifact parse + render test `[planned]` | `sdk-contract` / `frontend` |
| §20.3 #1.6 sampling weights **[contract]** | weighted aggregate matches population rate (A19) | unit: weighted vs unweighted diverge on tail-sampled fixture `[planned]` | `sdk-contract` / `backend` |
| §20.3 #1.7 Train/Dev/Test split **[contract]** | seeded split + contamination guard rejects a near-dup in Test (A13) | dataset split + contamination unit test `[planned]` | `sdk-contract` |
| §20.3 #1.8 mapping importer **[contract]** | a foreign dialect projects to canonical with no code | `/v1/import` mapping fixture `[planned]` | `sdk-contract` |
| §20.4 #2.x read APIs + UI | browse datasets, open an experiment with per-case CIs + gate badge, annotate, diff, analytics | Playwright e2e over the dashboard `[planned]` | `frontend` |
| §20.4 #2.1b bulk promote **[contract]** | `promote-from-query` materializes failures as cases with seeded split | `curl /v1/datasets/:id/promote-from-query` `[planned]` | `sdk-contract` |
| §20.5 #3.1 scorer breadth **[contract]** | new scorers pass on valid + invalid-when fixtures (§10.4) | per-scorer unit tests `[planned]` | `sdk-contract` / `backend` |
| §20.5 #3.3 custom scorer registry **[contract]** | an uploaded WASM scorer runs sandboxed with memory/epoch limits | `/v1/scorers` upload + sandbox limit test `[planned]` | `sdk-contract` |
| §20.5 #3.4 real statistics | delta with method-appropriate CI + real p-value, FWER-corrected, refuses underpowered (A2–A6) | `cargo test -p beater-stats` (alpha-calibration) `[planned]` | `backend` |
| §20.5 #3.6 CI integration | `pytest`/`beater eval` fails CI on regression | the pytest plugin / `beater eval` subcommand `[planned]` | `sdk-contract` |
| §20.5 #3.7 proper-scoring calibration **[contract]** | Brier/ECE + recalibration map improves ECE (A11) | `calibration-fixture` extended `[planned]`; today `[built]` runs kappa | `backend` |
| §20.6 #4.1 online evals score | sampled production traces scored on a schedule, weighted (A19) | `curl /v1/online/.../scores` timeseries `[planned]` | `backend` |
| §20.6 #4.3/#4.4 delivery + Slack | alert actually delivered (HMAC webhook / Slack Block Kit) | delivery-history endpoint + signed-payload test `[planned]`; today `[built]` `alert-fixture` computes a signed webhook | `backend` |
| §20.6 #4.5 anytime-valid alerting | alert decided against an mSPRT confidence sequence, not fixed-N (A7) | continuous-peek FP test `[planned]` | `backend` |
| §20.6 #4.7 prompt management **[contract]** | create/version/diff/run a prompt; resolve `prompt_version_id` at eval | `/v1/prompts` CRUD + playground `[planned]` | `sdk-contract` |
| §20.7 #5.2 enforced RBAC **[contract]** | a non-owner is denied a mutating route by `authorize()` (A20) | unauthorized mutate returns 403 `[planned]` | `sdk-contract` |
| §20.7 #5.4 tenant isolation at DB | cross-tenant read fails under Postgres RLS (A20) | store-conformance cross-tenant test `[built]` (app-layer) → DB-layer `[planned]` | `storage-backends` |
| §20.7 #5.5 crypto-shred **[contract]** | a shredded tenant is unreadable across hot/cold/artifact | deletion + unreadable-after assertion `[planned]` | `sdk-contract` |
| §20.7 #5.9 backups/restore | restore drill meets documented RPO/RTO | CI restore-drill job `[planned]` | `backend` |
| §20.7 #5.11 governance/SECURITY | `SECURITY.md` + compliance docs present | repo presence check; **`SECURITY.md` now exists `[built]`** | `backend` |
| §20.8 #6.2 zero-code bootstrap | env-var-only app produces traces with no code (§1 #13) | the README zero-code OTLP snippet `[built]` (manual); env-var distro `[planned]` | `gate1-live-smoke` |
| §20.8 #6.4 `beaterctl quickstart` | timed e2e shows a *scored failing case* under the §15 SLO | `beaterctl quickstart` `[planned]` | `gate1-live-smoke` |
| §21 MCP stdio transport | `tools/list` over stdio returns the full tool set | `beaterd mcp --stdio` `[planned]`; streamable-HTTP `/mcp` `[built]` | `sdk-contract` |
| §21.1 RSI tools | propose→simulate(Train)→accept(Test) only on a stat-sig held-out win (A13/A14) | `gate_candidate` MCP recipe `[planned]` | `backend` |
| §21.8 RSI MVP acceptance | indexes a small agent, proposes a generalizable change, verifies a Test win, applies via Claude Code with approval | end-to-end MCP episode `[planned]` | `backend` |

### 22.4 Acceptance-to-milestone traceability

The §18 milestone acceptance bullets and the §19 "Bar for Done" questions are
satisfied exactly when their §22.1/§22.3 rows are green:

- **v0 Substrate** → ingest + store + `beaterctl smoke` rows (§22.1) all `[built]`.
- **v1 OSS Observability & Offline Evals** → evals/judge, experiments+gates,
  calibration, and the read-API/UI rows; the offline path is `[built]` through the
  gate fixtures, the product UI is §20.4 `[planned]`.
- **v2 Agent-Native Debugging** → replay/attribution, sessions/messages, online
  evals + alert delivery, MCP rows.
- **v3 Hosted GA** → the entire hosted control-plane block (§20.7) plus
  self-observability load evidence (§16).

A milestone is "shipped" only when every row it depends on has a green
verification command **and** the CI gate that guards it is passing — which is the
same standard CONTRIBUTING.md enforces on every PR.

### 22.5 Metronome — the combined CI/CD pipeline

**Metronome** is the single combined CI/CD pipeline that keeps every beat-box on
tempo: it is the union of the GitHub Actions workflows under
`.github/workflows/` plus the local `scripts/check-*` drift gates, governed by one
rule — **a change that is not regenerated, tested, and drift-free cannot merge, and
only a green `main` deploys.** It has two halves: **CI** (the merge gates below,
run on every PR and on `main`) and **CD** (the deploy/release workflows, triggered
only by a push to `main` or a `v*` tag *after* the CI gates are green). The gate
set here is the same one §22.1/§22.3 map every component and plan item to, the same
verify-commands as §22.2, and is consistent with the README/CONTRIBUTING gate list
— there is one source of truth for "what must pass," not three.

#### CI — required merge gates

These **block merge**. Each maps to a workflow in `.github/workflows/` and a local
equivalent a developer runs before pushing (§22.2). A PR is mergeable only when all
required gates are green.

| Gate (workflow) | What it enforces | Local equivalent |
| --- | --- | --- |
| **`backend`** | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -D warnings` (the workspace denies `unwrap`/`expect`, incl. tests); `cargo test --workspace`; the `sqlite_migrations` test (schema/migration drift, below) | `cargo fmt`, `cargo clippy …`, `cargo test --workspace` |
| **`sdk-contract`** | the **whole contract chain has zero drift**: spec == served routes, spec == all **7** generated SDK clients (`regen-sdks.sh --check`), API-shape audit, semconv == all **5** semconv-carrying SDKs, and the additive-only `oasdiff` breaking-change check; MCP tools and the CLI resolve operations from the spec at runtime so they stay in sync automatically (drift coverage detailed below) | `scripts/check-contract-sync.sh` |
| **`storage-backends`** | the `beater-store-conformance` trait suite runs against every backend (in-memory, SQLite today; Postgres/ClickHouse as wired, §20.2 #0.1), incl. tenant-isolation (A20) and the `#[ignore]`d container-backed store tests | `cargo test -p beater-store-conformance --workspace`; `cargo test -p beater-store-sql -- --ignored` |
| **`frontend`** | dashboard build/lint/typecheck against the **generated** OpenAPI client, plus `check-openapi-drift.sh` so a UI change cannot silently diverge from the served spec | `scripts/check-openapi-drift.sh` |
| **`browser`** | the `beater-browser*` family (Liveset) builds and its driver/capture tests pass | `cargo test` over the browser crates |
| **`gate1-live-smoke`** | a live `beaterd` ingest → query round-trip (`beaterd --test live_smoke`); the zero-code-bootstrap and `quickstart` acceptance live here (§22.3) | `cargo run -p beaterctl -- smoke …` |
| **`gate2-proof-contract`** | the clean-clone-to-browser proof contract: `fmt`, `check-openapi-drift.sh`, the gate-0 foundations check, and the self-host/outside-validator tests that back the README "Clean Clone To Browser" path | `scripts/gate2-proof.sh` |
| **`gate2-browser-proof`** | the recorded browser demo proof (Playwright over the dashboard) | `scripts/browser-e2e.sh` |

The required set above is exactly the standard CONTRIBUTING.md applies to every PR;
admin review + squash-merge is the human gate layered on top, never a way around a
red required gate.

#### CI — advisory gates (informational, do not block merge)

- **Benchmarks / load (`beater-bench`, `xtask loadgen`) [planned].** Scale/p95 SLO
  evidence (§16, §20.2 #0.3). Advisory until the `[planned]` bench fixtures exist;
  a regression here is surfaced, not merge-blocking, because perf numbers are noisy
  on shared CI runners. Promote to required once a stable bench baseline lands.
- **`apple-container-build` / `container-images`.** Image builds for distribution;
  a build break is reported but does not block a source-only PR. On `main` these
  feed CD (below).

#### Explicit DRIFT coverage (the anti-silent-drift guarantee)

Metronome's defining job is that **no generated artifact can fall out of sync with
its single source without a gate going red.** The three drift surfaces of §1 #2:

1. **Contract drift — `spec → 7 SDKs → MCP → CLI → docs`** (the `sdk-contract`
   gate / `scripts/check-contract-sync.sh`, §22.2). The Rust `#[utoipa::path]`
   handlers in `beater-api` (Mixing Board) generate `sdks/openapi/beater-api.json`
   via `cargo xtask regen-spec`; that spec then generates the 7 clients
   (`scripts/regen-sdks.sh`). `check-contract-sync.sh` proves, in one command:
   spec == served routes (`openapi_coverage`), spec == all 7 regenerated clients
   (`regen-sdks.sh --check`), the API-shape audit, and an additive-only `oasdiff`
   breaking check. **MCP tools and the `beater api` CLI are not separately
   regenerated** — they resolve operations from the spec *at runtime*, so a spec
   change propagates to both automatically and a coverage test asserts the MCP
   `tools/list` covers every `/v1` operation (§22.1 MCP row). Docs that name a
   contract artifact ride the same gate.
2. **Semantic-convention drift — `beater-schema (Beatmap) → conventions.json → 5
   SDKs`** (`cargo xtask regen-semconv` + `scripts/check-semconv-drift.py`, part of
   `check-contract-sync.sh` step 4/4). Span kinds, attribute keys, defaults, and
   env-var names have one source — the `conventions` module of `beater-schema` —
   regenerated into `sdks/semconv/conventions.json`; the drift check parses the
   values actually assigned in each SDK's semconv file and fails if any of the 5
   carrying SDKs has drifted from the source.
3. **Schema / migration drift** (the `sqlite_migrations` test in the `backend`
   gate). Migrations are the source of the persisted schema; the migration test
   asserts the runtime schema matches the migration set and that migrations apply
   cleanly (and, per §20.2 #0.6, a per-backend migration-checksum test extends this
   to Postgres/ClickHouse as those backends are wired). `xtask renormalize`
   reprojects the immutable raw envelopes (§1 #3) when the normalizer or canonical
   schema changes, so a schema change is always re-derivable rather than a
   destructive migration.

If any source changes without its generated artifact being regenerated and
committed, the matching gate is red and the PR is unmergeable — drift cannot merge
silently. This is the same guarantee the §4 naming note relies on: the **physical
beat-boxes crate rename is a pre-1.0 follow-up** (§20.9) precisely because renaming
crate paths touches the contract/semconv sources and must pass Metronome's drift
gates in one regenerated change.

#### CD — deploy & release (green-`main`-only)

CD is the second half of the same pipeline; it never runs on a red tree.

- **`deploy-backend`** — on push to `main` touching `crates/**`/`bins/**`/Dockerfile/
  `fly.toml`, deploys `beaterd` (Fly.io; runtime secrets live in `fly secrets`, the
  workflow holds only `FLY_API_TOKEN`). Forks without the token skip the deploy
  rather than fail.
- **`deploy-dashboard`** — on push to `main` touching `web/dashboard/**`, deploys
  the Next.js dashboard.
- **`container-images`** — on push to `main`, builds and publishes the GHCR images
  used by the clean-machine compose path.
- **`release`** — on a `v*` tag (or manual `workflow_dispatch`), re-runs the
  contract verification (`verify-contract`) and cuts the tagged release artifacts.

Because every deploy workflow is `main`/tag-triggered and `main` only advances
through the required CI gates above, **CD inherits CI's guarantees**: nothing
deploys that has not passed the full required gate set, including zero contract /
semconv / migration drift. That closure — sources → generated artifacts → gates →
`main` → deploy — is what "Metronome keeps every box on tempo" means concretely.


