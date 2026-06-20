# Beater Ship Requirements

This file converts the architecture into an auditable checklist. A requirement is
done only when the evidence listed here exists in the current repo, deployed
system, test output, or runtime behavior.

## R0. Core Product Loop

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R0.1 | A user can send agent telemetry without a Beater-specific SDK. | OTLP HTTP/gRPC endpoint docs, `beaterctl smoke` local/remote tests, live `beaterd` OTLP HTTP/gRPC test; trace visible in UI still required |
| R0.2 | A user can inspect a trace as an agent-native span tree. | UI screenshots/tests covering `agent.*`, `llm.call`, `tool.call`, `mcp.request`, memory, guardrail, evaluator spans |
| R0.3 | A failure can be promoted to a dataset case. | Trace-to-dataset API test, human-review annotation promotion API test, and `beaterctl review-fixture` |
| R0.4 | Offline evals run over a dataset version. | Deterministic and judge-backed dataset eval integration tests plus `beaterctl judge-dataset-fixture` |
| R0.5 | Candidate and baseline releases can be compared. | Experiment output with per-case scores, aggregate deltas, stored gate policy, deterministic/judge experiment API tests |
| R0.6 | CI can be blocked on regression gates. | Persisted `GateStore` tests, `/v1/gates` API test, `beaterctl gate-run` nonzero exit test, deterministic and judge-backed experiment fixture tests, and example GitHub Actions workflow |
| R0.7 | Production traffic can be sampled for online evals and alerts. | Sampling policy tests and alert delivery tests |

## R1. Architecture Shape

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R1.1 | MVP ships as one all-in-one `beaterd` binary. | Cargo workspace with `bins/beaterd`; compose uses one Beater process by default |
| R1.2 | Future service split is logical, not operational. | Optional thin bins behind features; no mandatory 10-service deployment |
| R1.3 | OSS runs without Beater Cloud. | Offline compose test with network calls disabled except configured providers |
| R1.4 | Vercel is used only for stateless/control-plane work. | Vercel config contains Rust functions/UI only; stateful workers run outside Vercel or in `beaterd` |

## R2. Data Model and Schema Evolution

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R2.1 | Canonical entities exist: Run, Span, Event, Artifact, DatasetVersion, Experiment, EvaluatorVersion, EvalResult, Gate, GateRun, ReviewQueue, ReviewTask, Annotation, CalibrationReport, UsageRecord, AuditEvent. | Schema definitions and migrations |
| R2.2 | Every raw event has `schema_version`, source dialect, source schema version/URL, payload hash, and raw artifact ref. | Unit tests and sample stored raw envelopes |
| R2.3 | Normalized spans store `normalizer_version`, canonical attrs, unmapped attrs, and raw ref. | Golden normalizer tests |
| R2.4 | Old raw traces can be re-normalized after schema changes. | Migration/replay test that reprojects an old fixture into a new canonical version |
| R2.5 | Standards projections do not claim false losslessness. | Docs and export tests showing raw preservation for round-trip |

## R3. TraceStore and Storage

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R3.1 | Product code depends on a `TraceStore` trait. | `beater-store` is trait/error/fake only; concrete SQLite/filesystem implementations live in `beater-store-sql` and `beater-store-obj`; `cargo tree -p beater-store` has no `rusqlite` |
| R3.2 | Local backend exists before ClickHouse assumptions leak. | `beater-store-sql` TraceStore conformance suite runs identical cases against SQLite and `InMemoryTraceStore`; `MetadataStore` conformance suite runs identical org/project/environment/RBAC cases against SQLite and `InMemoryMetadataStore`; `QuotaLimiter` conformance suite runs identical fixed-window cases against SQLite and `InMemoryQuotaLimiter`; filesystem artifact store has its own round-trip/hash test |
| R3.3 | ClickHouse backend exists for scale. | ClickHouse migrations and testcontainers integration test |
| R3.4 | Large payloads are artifact refs, not hot-row blobs. | Size-cap tests and object-store fixture |
| R3.5 | Cold retention uses Parquet plus DataFusion. | Export/archive test and query fixture |
| R3.6 | Public storage/eval trait contracts use typed errors. | Store/search/dataset/experiment/judge/review/auth/secret/usage/audit/replay/calibration/gate traits return `StoreResult`, `EvalError`, `JudgeProviderError`, or `AgentAdapterError`; `rg "async fn .*anyhow::Result" ...` only reports tests/helpers, not trait methods |
| R3.7 | Foundational time and money primitives are replay-safe. | `beater-core` exposes `Clock`, `SystemClock`, `FixedClock`, typed `Currency`, and checked `Money::try_add`/`try_sub`; `rg "Utc::now()" crates/beater-core crates/beater-schema` returns no matches |
| R3.8 | Canonical schema owns span/status mapping and run rollups. | `AgentSpanKind::parse`, `SpanStatus::{as_str,parse}`, `span_matches`, `span_summary`, and `roll_up_runs` live in `beater-schema`; storage/search/archive/API callers delegate to schema helpers |

## R4. Ingest Survival

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R4.1 | Backpressure is bounded and observable. | In-memory/SQLite bus capacity tests; buffered ingest API 429 full-stack test; load test still required before GA |
| R4.2 | Durable buffer exists. | SQLite durable bus reopen/dedupe tests, `beaterd` SQLite bus default, buffered trace-write queue; NATS JetStream and Vercel Queues adapters still required for scale/hosted GA |
| R4.3 | DLQ captures invalid or repeatedly failed events. | Bus DLQ tests, trace-write and trace-ingested worker invalid payload DLQ tests, `beaterctl bus-fixture`; replay path still required |
| R4.4 | At-least-once delivery is reconciled by idempotency keys. | SQLite bus idempotent publish tests, SQLite `TraceStore` duplicate write tests; API duplicate-batch fixture still required |
| R4.5 | Cardinality and payload governance are enforced. | Attribute cardinality, allow/deny, and truncation-to-artifact tests |
| R4.6 | Tail-based sampling keeps errors, slow traces, and high-cost traces. | Policy tests with full-trace buffering |
| R4.7 | Trace completion handles root-end, idle timeout, late spans, and clock skew. | Out-of-order distributed trace fixtures |
| R4.8 | Per-project quotas produce explicit 429 semantics. | Ingest quota test exercises typed 429 error semantics through a trait-backed fixed-window limiter; SQLite/in-memory limiter conformance proves window reset behavior; API reset headers still required |
| R4.9 | Poison messages cannot stall a queue shard or consumer group. | Lane-aware and scoped bus consumption tests plus trace-write and trace-ingested worker invalid-payload DLQ tests |
| R4.10 | ClickHouse or TraceStore outage does not silently drop accepted events. | `buffer_native` outage/recovery unit test, buffered ingest + scoped drain full-stack API test, `beaterd` background trace-write drain worker, `beaterd` background trace-ingested downstream worker, `beaterctl smoke` OTLP trace-write/downstream drain test, live `beaterd` OTLP HTTP/gRPC smoke test, and `beaterctl ingest-outage-fixture` |

## R5. Evaluators

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R5.1 | Deterministic scorers run in WASI Component Model sandbox. | Wasmtime integration tests with no network access |
| R5.2 | LLM/embedding judges run through judge broker, never inside WASM. | `beater-judge` broker tests, HTTP provider mock-server tests, dataset and experiment judge eval tests, plus hosted API test for BYOK refs, cache hits, budget metering, and redacted ledger output |
| R5.3 | Evaluator catalog is explicitly lane-classified. | Registry with deterministic vs judge metadata |
| R5.4 | `EvalResult` pins reproducibility metadata. | Schema fields and tests asserting code hash/model/rubric/dataset/prompt versions |
| R5.5 | Non-reproducible reruns state why. | Test for deprecated/missing judge model or artifact |
| R5.6 | Judge spend is metered and budgeted. | `beater-usage` idempotent ledger tests, `/v1/usage` full-stack tests for direct/dataset/experiment judge spend, `beaterctl usage-fixture`, and judge budget-failure tests |

## R6. Statistical Rigor

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R6.1 | Candidate-vs-baseline comparisons include confidence intervals. | Experiment tests with expected CI output |
| R6.2 | Noisy judge evals support N-trial repetition. | Multi-trial fixture and aggregate variance |
| R6.3 | Gates can fail on confidence-bound regressions. | Gate policy tests plus deterministic/judge-backed experiment reports, persisted gate-run audit reports, and CLI nonzero regression test |
| R6.4 | Pairwise judges mitigate position bias. | A/B swap tests |
| R6.5 | Judge calibration stores agreement with human labels. | Persisted calibration report schema, Cohen's kappa unit test, `/v1/calibrations` full-stack test, and `beaterctl calibration-fixture` over judge-backed eval results |
| R6.6 | Experiment policies define minimum sample sizes. | Gate tests refusing underpowered comparisons |
| R6.7 | Statistical tests are chosen by metric type and assumptions. | Fixtures covering paired bootstrap/permutation/McNemar or documented equivalents |
| R6.8 | Multiple comparisons are controlled across metrics/cohorts. | Experiment fixture with adjusted decision output |

## R7. Replay and Attribution

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R7.1 | Provider, tool, memory, retrieval, clock, and random cassettes exist. | Cassette schema and recorder tests |
| R7.2 | Deterministic replay is allowed only when cassettes are complete. | Replay-mode tests |
| R7.3 | Forked replay and simulation are labeled differently. | API/UI tests |
| R7.4 | Root-cause attribution highlights earliest likely failing span. | Seeded multi-step failure fixture |
| R7.5 | Failed-vs-passed diffing works on similar traces. | Diff fixture and UI/API output |

## R8. Compliance and Data Lifecycle

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R8.1 | Deletion is compatible with immutable raw storage. | Crypto-shredding design, key hierarchy tests, deletion verification |
| R8.2 | Project-level data residency is enforced. | Region pinning tests and storage routing config |
| R8.3 | PII unmask is a separate audited scope. | `/v1/traces` full-stack test for default redaction, denied `pii_unmask`, allowed scoped unmask, and `/v1/audit` readback; `beater-audit` SQLite store test; `beaterctl audit-fixture` |
| R8.4 | Retention tiers are consistent across hot, cold, and artifact stores. | TTL/sweeper tests |
| R8.5 | Orphaned artifacts are detected and cleaned. | Sweeper integration test |

## R9. Security and Secrets

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R9.1 | API keys are scoped, rotatable, hashed, environment-bound, and audited. | Auth tests and migration fields |
| R9.2 | BYOK provider keys are never exposed to client or WASM code. | Encrypted provider-secret store tests, API response tests, CLI persistence leak check, broker credential redaction tests, and dependency checks |
| R9.3 | Encryption at rest and key rotation are designed. | KMS/local key manager tests; OSS SQLite path uses envelope metadata with key IDs and ChaCha20-Poly1305 ciphertext |
| R9.4 | Outbound webhooks are signed with replay protection. | HMAC/timestamp/idempotency tests |
| R9.5 | Tenant isolation is mandatory in storage APIs. | Type signatures requiring `TenantId`; negative tests for missing tenant |

## R10. Query, Search, and Alerting

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R10.1 | Structured search filters by status/time/model/tool/cost/latency/tags/evaluator. | Query API tests |
| R10.2 | Full-text search covers selected inputs, outputs, errors, and tool names. | Tantivy or equivalent fixture |
| R10.3 | Query latency SLOs are measured. | Load test report or benchmark dashboard |
| R10.4 | Alerts have baselines, dedupe, grouping, and maintenance windows. | Alert policy tests |
| R10.5 | Alert output links to traces, clusters, datasets, and gates. | Webhook/Slack payload tests |

## R11. Adoption and Public API

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R11.1 | Time-to-first-trace is under 5 minutes. | Fresh-machine quickstart runbook and timed smoke test |
| R11.2 | Zero-SDK OTLP onboarding works. | Live `beaterd` OTLP HTTP/gRPC smoke test uses stock OpenTelemetry protobuf service types; OpenInference/OpenLLMetry/OTel fixture apps still required |
| R11.3 | Rust SDK is first-class. | `tracing`, OTLP, reqwest, axum, tonic, MCP examples |
| R11.4 | Python/TS adoption is supported through standards on day one. | OTLP examples for common Python/TS frameworks |
| R11.5 | `/v1` API is stable and versioned. | OpenAPI spec and deprecation policy |
| R11.6 | Import/export avoids lock-in. | OTLP/Parquet/JSONL export tests and import fixtures |

## R12. Open-Core Governance

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R12.1 | License and open-core boundary are public before launch. | `LICENSE`, `GOVERNANCE.md`, feature matrix |
| R12.2 | No-rug-pull promise is documented. | Governance doc |
| R12.3 | Evaluator/framework extension API is semvered. | Plugin ABI docs and version tests |
| R12.4 | Contributing path exists. | `CONTRIBUTING.md`, issue templates, local dev instructions |
| R12.5 | Self-host telemetry is opt-out. | Config default and test |

## R13. Self-Observability

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R13.1 | Beater emits its own traces, metrics, and logs. | Dogfood config and dashboards |
| R13.2 | Ingest-to-queryable lag is measured. | Metric and alert |
| R13.3 | Write success rate is measured. | Metric and alert |
| R13.4 | Eval queue depth and age are measured. | Metric and alert |
| R13.5 | Query p95 is measured. | Metric and alert |
| R13.6 | DLQ age and count are measured. | Metric and alert |
| R13.7 | Normalizer failures are measured by source dialect and version. | Metric and alert |
| R13.8 | Queue lag is measured per lane and tenant/project where safe. | Metric and alert |
| R13.9 | Object-store read/write/delete failures are measured. | Metric and alert |

## R14. Trace Context Propagation

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R14.1 | W3C trace-context is preserved across HTTP, gRPC, queues, and async tasks. | Propagation integration tests |
| R14.2 | Baggage carries tenant/project/release context safely. | Baggage tests with redaction of sensitive values |
| R14.3 | Span links model fan-out and multi-agent handoffs. | Fixture rendering handoffs and fan-out in trace view |
| R14.4 | MCP request spans avoid duplicate tool spans when outer instrumentation exists. | MCP fixture based on OTel GenAI guidance |

## R15. Hosted GA Gate

Hosted cannot be sold seriously until all are true:

- orgs/projects/environments
- scoped API keys
- tenant isolation
- quotas and rate limits
- billing ledger
- audit logs
- retention policies
- background workers
- object lifecycle
- backups and restore drills
- SLO dashboards
- Slack/webhook alerts
- regional deployment
- support runbooks

## R16. Completion Audit Rule

Before marking Beater "shipped", audit every requirement above against direct
evidence. Passing tests are evidence only when the tests cover the requirement's
full scope. Search results, intent, partial implementation, and plausible docs do
not prove completion.

## R17. Execution Readiness

| ID | Requirement | Evidence required |
| --- | --- | --- |
| R17.1 | OSS MVP scope is not reduced below the documented minimum shippable product. | Milestone checklist includes all MVP items and release notes map each missing item to a known non-GA status |
| R17.2 | Hosted GA has named ownership for Rust infra, backend/product, frontend/product, data/observability, evals/agent systems, and infra/security. | Staffing/ownership plan before hosted GA |
| R17.3 | Hard engineering risks have explicit owners and test plans. | Risk register covering schema evolution, privacy, standards translation, ingest survivability, eval reproducibility, judge calibration, replay correctness, query speed, UX, and governance |
