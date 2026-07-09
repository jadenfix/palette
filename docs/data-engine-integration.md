# Palette ↔ data-engine integration — the Tempera data flywheel

> Status: design README. This describes how Palette (currently shipped as the
> `beater*` crates and the `beaterd`/`beaterctl` binaries) fits with its sibling
> [`jadenfix/data-engine`](https://github.com/tempera-dev/data-engine) so that
> data flows full-circle: **Palette observes agents → data-engine cleans, labels,
> and emits verified training data → cleaned artifacts feed back into Palette's
> eval/gate loop.** No code changes here; this is the contract + the asks.
>
> Companion to [`docs/ecosystem-roadmap.md`](ecosystem-roadmap.md). Grounded in
> Palette's actual seams (paths cited). The governing rule on both sides is the
> same: **contract first (OTLP / OpenAPI / MCP), native optional** — neither
> repo depends on the other by default.

---

## 1. What each side owns (no overlap, no reimplementing)

| | Palette (Beater) | data-engine |
|---|---|---|
| **Job** | Observe the agent: record what it did, promote failures to datasets, eval, gate CI. The "what happened." | Turn observations into training signal: normalize, dedup, decontaminate, label, emit. The "what to train on." |
| **Owns** | OTLP traces/spans, `DatasetVersionSnapshot` (Merkle `corpus_root`), eval/judge/stats/experiments/gates, `AgentActionReceipt` (hash-chained), replay. | The label cascade, content-addressed compute graph, verifier farm (runs in [`cradle`](https://github.com/jadenfix/cradle)), RLVR/eval/preference/SFT emitters, the gap loop. |
| **Reuses** | — | Palette's traces, datasets, eval scores, receipts as **inputs** (not reimplemented). |

The split is clean: Palette never labels; data-engine never observes/gates. They
meet at two contract seams — **Palette's read API** (collect) and **Palette's
`SourceImporter`** (feed back).

---

## 2. The full-circle data flow

```
                   agent (Beater SDK / any OTLP exporter)
                                     │ OTLP
                                     ▼
   ┌─────────────────────────────────────────────────────────────────┐
   │ PALETTE (beaterd)                                               │
   │  POST /v1/otlp/{t}/{p}/{e}/v1/traces                            │
   │   → immutable RawEnvelope (payload_hash=Sha256)                 │
   │   → CanonicalSpan tree (agent.run / llm.call / tool.call / ...) │
   │  promote failure → DatasetCase → DatasetVersionSnapshot         │
   │   (content-addressed Merkle corpus_root)                        │
   │  eval / judge / experiment / gate → scores + gate decisions     │
   │  AgentActionReceipt (append-only SHA-256 hash chain)            │
   └─────────────────────────────────────────────────────────────────┘
                                     │ COLLECT  (data-engine reads Palette)
                                     ▼
   ┌─────────────────────────────────────────────────────────────────┐
   │ DATA-ENGINE  palette connector                                   │
   │   trace        → artifact (the run)                             │
   │   span         → step                                           │
   │   DatasetCase  → hard negative / preference pair / SFT trace    │
   │   eval score   → partial label w/ confidence                    │
   │   gate decision/ experiment report → delayed ground-truth label │
   │                            (mechanism D — outcome as truth)     │
   │   receipt      → provenance edge (hash-chained, inherited)      │
   │  CLEAN + PREPARE: normalize → dedup → decontaminate →           │
   │   content-address → label (cascade) → EMIT                      │
   │   signed bundles: RLVR / eval / preference / SFT                │
   └─────────────────────────────────────────────────────────────────┘
                                     │ FEED BACK  (full circle)
                                     ▼
   ┌─────────────────────────────────────────────────────────────────┐
   │ PALETTE  receives cleaned/labeled artifacts via a               │
   │  SourceImporter (source = "data_engine", dialect reserved)      │
   │  → canonical spans flow through the SAME ingest pipeline        │
   │  → customer's own eval/gate loop now consumes cleaned data      │
   │  → agent evals + LLM evals materialize as a side effect         │
   └─────────────────────────────────────────────────────────────────┘
```

The customer outcome: an enterprise that instruments agents with Beater gets
their traces **cleaned and turned into evals + training data by data-engine,
with zero new SDK**, and the cleaned data returns into Beater's gate loop. That
is the "full circle."

---

## 3. The concrete seams (cited from Palette's code)

### 3a. INBOUND to Palette — the feedback path

data-engine pushes cleaned artifacts back through Palette's pluggable importer
seam, exactly like the Langfuse importer does today.

- **`SourceImporter` trait** — `crates/beater-ingest/src/lib.rs` (one method):
  `fn normalize(&self, scope: &TenantScope, raw_bytes: &[u8], auth: Option<AuthContext>) -> Result<RawTraceIngestRequest, ImportError>`. Pure, no IO.
- **Register**: `IngestService::with_importer(Arc::new(YourImporter))`.
- **Legacy dispatch over HTTP (built today)**:
  `POST /v1/import/{tenant_id}/{project_id}/{environment_id}` with
  `{"source": "...", "payload": {...}}`. This is the existing importer seam, not
  the stable data-engine feedback contract: it is tenant-leading, uses the
  legacy `importSource` operationId in current OpenAPI, and must be treated as
  compatibility surface only.
- **Target feedback contract**:
  `POST /v1/{parent=projects/*}/imports` with operationId
  `projects.imports.create`, `Authorization: Bearer <token>`, required
  `Idempotency-Key`, and the shared error envelope. The request body should keep
  the current `source` + `payload` shape so `DataEngineImporter` can stay a thin
  adapter over the existing `SourceImporter` trait.
- **Output**: a `RawTraceIngestRequest` of `CanonicalSpanDraft`s; Palette keeps the raw bytes losslessly in a `RawEnvelope` and projects `CanonicalSpan`s.
- **Template to copy**: `crates/beater-langfuse/src/lib.rs` (`LangfuseImporter`, source key `"langfuse"`, sets `RedactionClass::Sensitive`). End-to-end test recipe at `crates/beater-langfuse/tests/import.rs`.

→ data-engine ships a `DataEngineImporter` (source `"data_engine"`). This is the
"full circle" mechanism. The generic `mapping` importer can be used as a
temporary bridge, but the AIP-shaped `projects.imports.create` endpoint is the
uniform contract that data-engine should call long term.

### 3b. OUTBOUND from Palette — the collect path (partially built — this is the gap)

data-engine reads Palette over the read API + committed OpenAPI. **What exists
today** (`GET` routes, `crates/beater-api/src/lib.rs`):

- `/v1/traces/{tenant_id}[/{trace_id}]`, `/v1/spans/{t}/{trace_id}/{span_id}[/io]`
- `/v1/search/{tenant_id}/spans`, `/v1/archive/{tenant_id}/{project_id}/spans`
- `/v1/prompts/...`, `/v1/scenarios/...`, `/v1/judge/{t}/{p}/ledger`,
  `/v1/review-queues/...`, `/v1/audit/...`, `/v1/usage/...`

**The gap — these are WRITE-ONLY over HTTP today (no `GET`):**
datasets, dataset versions, dataset cases, eval reports, experiment runs, gate
runs. They exist as first-class store types but can only be *created* via the
API, not *read*. Reading them currently requires linking the store crates
(`beater_datasets::DatasetStore`, etc.) — i.e. native coupling, which violates
the "contract first" rule.

- **Contract source**: `sdks/openapi/beater-api.json` (OpenAPI 3.1; also mirrored
  at `web/dashboard/openapi/beater-read-api.json`).
- **MCP**: derived at runtime from the OpenAPI at `POST /mcp` — **every** `/v1`
  operation is already an MCP tool, so new `GET` routes become MCP tools for free.

### 3c. The gold data-engine wants (already content-addressed in Palette)

- **`promote_trace_span_to_case`** (`crates/beater-datasets/src/lib.rs`) — turns a
  `TraceView` + span into a `DatasetCase` (input/output/reference/trace + input
  artifact hashes). This is the promoted-failure → hard-negative / preference /
  SFT source.
- **`DatasetVersionSnapshot`** — carries `cases: Vec<DatasetCase>` **and a
  content-addressed Merkle `CorpusRoot`** (`crates/beater-core/src/merkle.rs`,
  history-independent: identical case sets ⇒ identical root). **The `corpus_root`
  is the verifiable handle data-engine labels, decontaminates, and emits against.**
- **`AgentActionReceipt`** (`crates/beater-receipts/src/lib.rs`) — append-only
  SHA-256 hash chain (`prev_hash`/`hash`, `verify_chain`). data-engine inherits
  this as provenance edges for free.

---

## 4. What Palette should add (the asks — minimal, additive, contract-first)

Ordered by leverage. Each keeps Palette standalone (no data-engine dependency).

1. **Add `GET` read routes for the dataset/eval/experiment/gate family.** This is
   the single highest-leverage change — it lets data-engine collect over the
   contract instead of native-coupling to store crates. Proposed routes follow
   the ecosystem API style from data-engine `API_STYLE.md`: project-parent
   scoped, cursor-paginated, drift-gated, and named with dotted `projects.*`
   operationIds. Tenant/org scope remains enforced by auth and Palette's store
   isolation; it should not be a separate leading URL dialect.
   - `GET /v1/{parent=projects/*}/datasets` — list datasets
     (`projects.datasets.list`)
   - `GET /v1/{name=projects/*/datasets/*}` — dataset + latest version ref
     (`projects.datasets.get`)
   - `GET /v1/{parent=projects/*/datasets/*}/versions` — version history
     (`projects.datasetVersions.list`)
   - `GET /v1/{name=projects/*/datasets/*/versions/*}` — full
     `DatasetVersionSnapshot` incl. `corpus_root` and cases
     (`projects.datasetVersions.get`)
   - `GET /v1/{parent=projects/*/datasets/*}/cases` — cases (paginated)
     (`projects.datasetCases.list`)
   - `GET /v1/{parent=projects/*/datasets/*}/evals` — eval reports
     (`projects.datasetEvals.list`)
   - `GET /v1/{parent=projects/*/experiments/*}/runs` and
     `GET /v1/{parent=projects/*/gates/*}/runs` — outcomes, the
     **delayed-truth** labels for data-engine's gap loop
     (`projects.experimentRuns.list`, `projects.gateRuns.list`)

   Mechanically: add the handlers in `crates/beater-api/src/lib.rs`, regenerate
   via `cargo xtask regen-spec && scripts/regen-sdks.sh && scripts/check-contract-sync.sh`
   in the same PR. MCP tools auto-derive from the operationIds. **No behavior
   change; pure additive read surface.**

2. **Reserve a `data_engine` source dialect + importer seat.** Add
   `SourceDialect::DataEngineImport` (`crates/beater-schema/src/lib.rs`, next to
   `LangfuseImport`/`LangSmithImport`/`PhoenixImport`) and document it, even if
   the importer itself is implemented in data-engine. This makes the feedback path
   first-class instead of riding the generic `mapping` importer. (Note: `PhoenixImport`
   and `LangSmithImport` variants already exist without committed importers — same
   pattern: reserve the seat, implement where it belongs.)

3. **Expose a signed dataset-version export bundle.** `DatasetVersionSnapshot`
   already has a Merkle `corpus_root`; expose a content-addressed export (the
   `corpus_root` is the handle). data-engine's emitted products should be **signed
   bundles over these `corpus_root`s** (resolves data-engine's open question in its
   `plan.md` §11: "signed bundles over content-addressed Beater datasets"). No new
   store — the `corpus_root` already is the verifiable handle.

4. **Confirm the redaction + tenancy contract on the feedback path.** data-engine
   pushes back cleaned spans; Palette applies its existing `RedactionClass` +
   tenant/project/env scoping. No new policy — just document that a
   `DataEngineImporter` follows the Langfuse template (`RedactionClass::Sensitive`,
   tenant-scoped, lossless raw kept).

---

## 5. What data-engine adds (the other side — for symmetry)

- A **`palette` connector** that reads Palette's read API (incl. the new `GET`
  dataset/eval/experiment routes), normalizes into data-engine artifacts:
  - trace → artifact, span → step, `DatasetCase` → hard-negative/preference/SFT,
    eval score → partial label, gate/experiment outcome → delayed-truth label.
- A **`DataEngineImporter`** (Langfuse template) for the feedback push.
- **Decontamination** of Palette's exported cases against pretraining corpora
  *before* labeling (data-engine's job; Palette just exports). Don't label
  memorized items.

---

## 6. Provenance + decontamination alignment (no double-bookkeeping)

Palette already produces the provenance primitives data-engine needs:

```
Palette payload_hash (Sha256)  ─┐
Palette corpus_root (Merkle)   ─┼─► become data-engine content-addressed node ids
Palette receipt hash chain     ─┘   (data-engine DESIGN.md §4 compute graph)
```

A data-engine label node's identity can include the Palette `corpus_root` + case
hash directly — **provenance is inherited, not re-minted.** This is the technical
expression of "the two repos are one loop" (`data-engine` BUSINESS_AUDIT.md).

---

## 7. Phasing (aligned to data-engine's ROADMAP)

| data-engine phase | Palette dependency | When |
|---|---|---|
| Phase 1a (collect) | **Palette ask #1** (GET dataset/eval/experiment/gate routes) | now — unblocks the connector |
| Phase 1a | OTLP read (`/v1/traces`, `/v1/spans`) — already exists | now |
| Phase 3b (closed loop) | experiment/gate outcomes → delayed-truth labels (mechanism D) | needs ask #1 |
| Phase 4 (trigger-gated) | **Palette ask #2/#4** — `DataEngineImporter` feedback path | first LOI |

Ask #1 is the only blocker on the critical path. It is pure additive read surface
and can land in Palette independently, ahead of any data-engine code.

---

## 8. What this is NOT

- **Not a native crate dependency** either way. Contract first; both repos run
  standalone from a clean clone.
- **Not Palette doing labeling** — that's data-engine's job (and its moat).
- **Not data-engine doing observability/gates/RSI** — that's Palette's job.
- **No new database, no new auth** — reuse ecosystem auth (`Authorization:
  Bearer <token>` as canonical; legacy API-key headers only as deprecated
  compatibility aliases) + Palette's existing tenant/project/env isolation +
  `RedactionClass`.
- **Not a rename ask** — Palette stays Palette (Beater crates/binaries); this doc
  uses both names since the code still says `beater*`.

---

## 9. Open questions (for Jaden)

1. Should the feedback `DataEngineImporter` push **cleaned spans** back, or only
   **labels/score overlays** (references on existing spans)? Lean: labels as
   overlays first (cheaper, no duplication); full cleaned spans only on demand.
2. Does Palette want a first-class **"training-data export" product kind** (a
   signed bundle over a `corpus_root` tagged with data-engine label provenance),
   or is that purely a data-engine artifact that *references* Palette's
   `corpus_root`? Lean: the latter — Palette owns the corpus, data-engine owns
   the labels-over-corpus.
3. Governance: if a Palette dataset is CUI/export-controlled, does the feedback
   path stay within the same enclave? (Aligns with data-engine DESIGN.md §9 —
   classification tag must round-trip.)
