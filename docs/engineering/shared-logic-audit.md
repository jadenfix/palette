# Shared Logic Audit

Status: active audit for keeping Beater's Gate 2, dashboard, and Rust service
contracts narrow, shared, and testable.

## Shipped in this pass

- `scripts/gate2_proof_contract.py` owns the Gate 2 image catalog, required
  image platforms, default outside-run endpoints, full-run port contract, and
  confirmation-code hash function plus test vector.
- Gate 2 readiness, public handoff, proof generation, and proof validation now
  consume that shared contract instead of each owning independent image/platform
  constants.
- `bins/beaterd/tests/self_host_contract.rs` now asserts that scripts use the
  shared contract while still keeping independent black-box sentinel coverage for
  public handoff behavior.
- `web/dashboard/lib/gate2-confirmation-request.ts` owns the browser
  confirmation request shape plus trace/span id validation, and the route/client
  now consume that shared contract.
- `web/dashboard/tests/e2e/gate2-confirmation-code.mjs` owns the recorder-side
  confirmation helper and test vector; dashboard tests assert parity with the
  TypeScript app helper.
- `web/dashboard/lib/api.ts` now uses the generated OpenAPI
  `PageRunSummaryDoc` shape instead of a hand-rolled generic page wrapper.
- Rust JSON/content hashes now use `beater_core::sha256_hex` in judge,
  dataset, replay, store, and ingest code instead of local implementations.
- `beater_core::lower_hex` owns generic lowercase byte-to-hex formatting for
  OTLP id normalization, CLI smoke fixtures, and live-smoke tests.
- Gate 2 self-host contract tests now assert the shared dashboard confirmation
  helper directly, rather than the removed quickstart-local wrapper.
- `beater-search::index_project_trace` owns trace-ingested readback plus search
  indexing, so API drain and `beaterd` background workers share the same
  downstream processing path.
- `ApiState` now has one private base constructor plus explicit opt-in setters
  for search, archive, dataset, and experiment integrations. The public
  constructors delegate through that path, and a regression test pins default
  optional-service behavior.

## Keep Independent

- Rust string-contract tests should keep independent expected outcomes for
  outside-run evidence, docs, workflow snippets, and validator failures. Those
  tests are guardrails against accidental coupling, not production helpers.
- Shell preflight scripts may keep local operational implementations until a
  dedicated shell library can be introduced without hiding user-facing failure
  messages.
- Runtime-specific Gate 2 confirmation implementations may exist in TypeScript,
  Python, shell, and Node ESM, but each implementation must stay tied to the
  shared prefix and golden vector.

## Audit Findings

- SQLite schema ownership is the largest remaining duplication. `beaterd`
  applies one local all-tables migration across several store databases while
  individual store crates also own `CREATE TABLE` blocks. The safe direction is
  either one local SQLite database with one migration manager, or per-store
  migrations exported by the owning crate and consumed by runtime wiring.
- Store boilerplate is repeated across auth, audit, datasets, gates, human,
  replay, search, secrets, usage, and experiments. A small SQLite support module
  should own connection setup, `IntoStoreResult`, JSON column helpers, and
  timestamp/id decoding without becoming an ORM.
- Ingest preparation still has parallel native/raw/OTLP paths. The right shared
  contract is one internal canonical span input plus shared artifact hashing,
  idempotency, redaction, and span assembly.
- Trace-ingested search processing now shares the readback/index helper, but
  higher-level queue draining, retry reporting, and worker hooks still live at
  their API/runtime boundaries. The API drain route and `beaterd` worker still
  each build the async search-indexing closure.
- JSON value hashing still repeats `serde_json::to_vec` plus SHA-256 wrapping in
  dataset, judge, and replay code. `beater_core` owns byte hashing now; a small
  JSON-hash helper can remove the last drift-prone copies without pulling
  serialization policy into higher-level crates.
- OpenAPI doc schemas mirror real API and schema DTOs. Prefer deriving or
  sharing public response DTOs rather than maintaining doc-only copies for
  canonical spans, run summaries, money, artifact refs, and query params.
- API handlers repeat route id parsing for tenant, project, environment,
  dataset, version, trace, span, queue, task, annotation, gate, and experiment
  ids. Introduce typed path structs or small `parse_*_id` helpers at the API
  boundary before splitting handlers, so auth checks and domain calls receive
  validated ids consistently.
- API handlers and CLI fixtures rebuild similar eval, dataset, and experiment
  specs. Keep HTTP request structs local, but centralize conversion into domain
  specs after route-id parsing is shared; otherwise the conversion helpers will
  still own too much HTTP-specific error mapping.
- Local runtime wiring is concentrated in large bin files:
  `bins/beaterd/src/main.rs`, `bins/beaterctl/src/main.rs`, and the full-stack
  API/Gate 2 fixtures. `LocalStorePaths`, `LocalStackBuilder`, `demo_scope()`,
  and fixture setup helpers would reduce main-file size while preserving
  all-in-one operational simplicity.
- OTLP smoke export fixture assembly is repeated in `beaterctl`, `beaterd`
  live-smoke tests, and API tests. Move only the stable test-support pieces
  (`smoke_ids`, metadata values, smoke export construction) into `beater-otlp`;
  keep endpoint assertions local to each runtime.

## Next Shared-Logic Targets

- SQLite schema: choose the one-database versus per-store migration model before
  touching migrations, then remove the duplicate all-tables/per-store DDL.
- Gate 2 proof schema: move required proof field names, fixed values, artifact
  path rules, and SLO field names behind a shared Python proof-schema helper.
- Runtime preflight: extract common Docker endpoint checks, default port checks,
  cleanup command constants, and process-owner hints into a small shell helper
  plus matching Python constants.
- Span taxonomy: add `AgentSpanKind::all()` in `beater-schema`, then validate or
  generate dashboard, migration, shell, and E2E span-kind lists from that source.
- OTLP normalization: keep dialect aliases local, but normalize aliases through
  `AgentSpanKind::parse` so canonical span-kind strings are not re-listed.
- Ingest crate: split internal modules into `normalize`, `queue`, `policy`, and
  `test_support`; extract shared canonical span assembly and retry accounting.
- Store helpers: add small shared helpers for span storage identity and trace
  span ordering while leaving SQLite and memory persistence mechanics separate.
- Trace-ingested search processor: wrap `index_project_trace` in one closure
  factory or processor type consumed by API drain and `beaterd` workers, while
  leaving route status-code selection and worker lifecycle local.
- JSON hash helper: add `beater_core::sha256_json_hash<T: Serialize>()` or an
  equivalent fallible helper returning `Sha256Hash`, then migrate dataset,
  judge, and replay hashes with golden parity tests.
- Eval context: introduce a typed `TraceEvalContext` builder so latency/cost
  evaluators, alerts, datasets, and experiments depend on one trace metric shape.
- API route parsing: add typed path/query helpers for `TenantId`, `ProjectId`,
  `EnvironmentId`, `TraceId`, `SpanId`, `DatasetId`, `DatasetVersionId`,
  `ReviewQueueId`, `ReviewTaskId`, `AnnotationId`, `GateId`, and
  `ExperimentRunId`, then migrate handlers by route family with auth tests.
- Eval/spec conversion: move deterministic/judge dataset and experiment request
  conversion into small API-local builder helpers that return domain specs and
  preserve 400-level id/hash validation.
- Evaluator lanes: add `ensure_deterministic` / `ensure_judge` helpers on
  evaluator specs and use them in datasets, experiments, judge broker paths, and
  API route guards.
- API redaction: move span I/O redaction helpers out of route code and make
  dataset promotion explicitly choose redacted or unmasked capture policy.
- Dashboard query model: replace repeated query field mappings with a
  table-driven helper for search-param parsing, API params, hidden inputs,
  filter chips, and href construction.
- Dashboard Gate 2 confirmation: keep the request/id contract shared and the
  recorder helper pinned to the app helper with the shared vector.
- Dashboard timeline/view helpers: extract pure timeline bounds, axis, run
  summary, artifact formatting, and query helpers before splitting components.
- Rust store result helpers: evaluate whether repeated `IntoStoreResult`
  adapter traits should move into `beater-store` or a small persistence helper.
  Move that trait before attempting timestamp/id decode helpers, so local
  rusqlite row-shape errors stay explicit and readable.
- SQLite timestamp decoding: centralize repeated RFC3339 decode/error mapping
  once the store crates settle on a common error helper.
- CLI/runtime main files: move `beaterctl` command handlers and repeated
  full-stack test setup into focused modules or per-crate test-support helpers.
- OTLP test support: share smoke export construction from `beater-otlp` test
  support once the API, CLI, and runtime tests agree on which fields are
  contractually stable.
