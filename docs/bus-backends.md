# Bus Backend Adapter Contract

> **Status:** `DurableBus` trait is built and both `InMemoryBus` and
> `SqliteDurableBus` implement it (see `crates/beater-bus/src/lib.rs`).
> NATS JetStream and Kafka adapters are **[planned]** — no client deps are
> in-tree yet.  This document is the design reference for those adapters and
> for anyone adding a new backend.

---

## 1. Why a trait boundary?

The architecture contract (§8.1, §8.2) is that _the same code paths must work
whether the bus backend is the embedded SQLite store, NATS JetStream at the
edge, or a Kafka cluster in a worker cell_.  All ingest, eval-job, and replay
callers depend on `DurableBus`, never on a concrete type.  Swapping backends is
an ops decision made at process startup, not a product-code change.

This mirrors the `TraceStore` boundary (§8.1): one trait, multiple impls, one
conformance suite.

---

## 2. The adapter contract — `DurableBus`

```rust
#[async_trait::async_trait]
pub trait DurableBus: Send + Sync {
    async fn publish(&self, message: BusMessage) -> Result<PublishAck, BusError>;
    async fn consume_batch(&self, limit: usize) -> Result<Vec<BusMessage>, BusError>;
    async fn consume_kind_batch(&self, kind: &str, limit: usize) -> Result<Vec<BusMessage>, BusError>;
    async fn consume_scoped_kind_batch(
        &self, tenant_id: &TenantId, project_id: &ProjectId, kind: &str, limit: usize,
    ) -> Result<Vec<BusMessage>, BusError>;
    async fn ack(&self, message: BusMessage) -> Result<(), BusError>;
    async fn retry_or_dlq(&self, message: BusMessage, reason: String) -> Result<(), BusError>;
    async fn replay_dead_letter(
        &self, tenant_id: &TenantId, project_id: &ProjectId, message_id: &str, reset_attempts: bool,
    ) -> Result<PublishAck, BusError>;
    async fn dlq(&self) -> Result<Vec<DeadLetter>, BusError>;
    async fn depth(&self) -> Result<usize, BusError>;
    async fn depth_for_kind(&self, kind: &str) -> Result<usize, BusError>;
}
```

All callers hold `Arc<dyn DurableBus>`.

### 2.1 Delivery guarantees every backend must honour

| Property | Required behaviour |
|---|---|
| **At-least-once delivery** | A message consumed but not acked must be re-delivered after a configurable lease timeout (crash recovery). |
| **Idempotent publish** | A second `publish` with the same `(tenant_id, project_id, kind, idempotency_key)` while the first is active (queued or inflight) returns `PublishAck::duplicate()` without inserting a second copy. |
| **Ordered within a kind** | Messages for the same `kind` are delivered in `enqueued_at` / insertion order.  Cross-kind ordering is not guaranteed. |
| **Partition isolation** | `consume_scoped_kind_batch` must not return messages belonging to a different `(tenant_id, project_id)` pair. |
| **Poison-message isolation** | After `max_attempts` failures, `retry_or_dlq` moves the message to DLQ; it must not block or starve other messages in the same queue. |
| **DLQ replayability** | `replay_dead_letter` re-enqueues a DLQ entry, optionally resetting `attempts`, and removes it from DLQ on success. |
| **Depth accounting** | `depth` returns `queued + inflight` total; `depth_for_kind` returns the same scoped to one message kind. |
| **Backpressure** | When `depth >= capacity`, `publish` returns `BusError::Backpressure`; callers must apply back-pressure rather than retry tightly. |

### 2.2 What backends are NOT required to provide

- **Exactly-once delivery** — callers use idempotency keys to collapse
  duplicates; the bus only guarantees at-least-once.
- **Global total ordering** — only per-kind ordering is required.
- **Synchronous persistence** — a backend may buffer in memory as long as crash
  recovery re-queues inflight messages before serving new consumers.

---

## 3. Existing backends

| Backend | Crate | Status | Use case |
|---|---|---|---|
| `InMemoryBus` | `beater-bus` | Built | Tests, ephemeral dev, conformance harness |
| `SqliteDurableBus` | `beater-bus` | Built — runtime default | OSS all-in-one, local dev, CI |

Both are validated by the generic `trait_round_trip` helper in
`crates/beater-bus/src/lib.rs` (tests `backend_pluggability_in_memory_bus` and
`backend_pluggability_sqlite_durable_bus`).

---

## 4. Planned adapters

### 4.1 NATS JetStream adapter (`NatsJetStreamBus`)

**Target crate:** `beater-bus-nats` (new, no client deps in-tree yet).

**Mapping:**

| `DurableBus` operation | NATS JetStream primitive |
|---|---|
| `publish` | `js.publish(subject, payload)` with `Nats-Msg-Id` header set to the idempotency key for server-side dedup (JetStream dedup window). |
| `consume_batch` | `Consumer::fetch(limit)` from a durable pull consumer on the stream. |
| `consume_kind_batch` | A per-kind subject filter; one consumer per kind subject prefix. |
| `consume_scoped_kind_batch` | Subject hierarchy `bus.{tenant}.{project}.{kind}` with a per-tenant-project consumer. |
| `ack` | `msg.ack()` — removes from consumer's pending set. |
| `retry_or_dlq` | `msg.nak_with_delay(backoff)` for retry; move to a `$JS.EVENT.BUS.DLQ` stream on max attempts using an `ack+publish` pattern. |
| `replay_dead_letter` | Republish from DLQ stream to main stream with `reset_attempts` reflected in a header. |
| `dlq` | Consume from the DLQ stream with `0` ack timeout (read-only scan). |
| `depth` / `depth_for_kind` | `stream.info().state.num_pending` filtered by subject. |

**Crash recovery:** inflight messages with unacked `Consumer` entries are
automatically re-delivered by JetStream after the consumer's `ack_wait` expires.
`SqliteDurableBus`'s explicit `recover_inflight` step is not needed.

**No client deps yet:** add `async-nats = "0.34"` to `beater-bus-nats/Cargo.toml`
only when wiring begins; do not add it to `beater-bus` (keeps the core crate
dependency-free of network clients).

**Acceptance criteria:**

1. `NatsJetStreamBus` passes the `trait_round_trip` helper (copy to a
   conformance module or re-export and call from `beater-bus-nats/tests/`).
2. A container test (`testcontainers-modules::nats`) runs the full round-trip
   including crash recovery (kill consumer, restart, re-deliver inflight).
3. `depth` matches `stream.info().state.num_pending + num_pending_pull_consumers`
   within ±1 across all test scenarios.
4. The `beater-ingest` and `beater-api` crates compile and pass their existing
   tests with `NatsJetStreamBus` wired via `Arc<dyn DurableBus>` — zero
   product-code changes.

### 4.2 Kafka adapter (`KafkaBus`)

**Target crate:** `beater-bus-kafka` (new, no client deps in-tree yet).

**Mapping:**

| `DurableBus` operation | Kafka primitive |
|---|---|
| `publish` | Transactional `producer.send(record)` with idempotent producer enabled; idempotency key in header; dedup by Kafka's idempotent-producer sequence numbers per partition. |
| `consume_batch` | `consumer.poll(timeout)` up to `limit` records from a consumer group. Store offset advance only after ack. |
| `consume_kind_batch` | Each `kind` is a dedicated topic; assign only the matching topic partition. |
| `consume_scoped_kind_batch` | Topic-per-kind with a partition key of `{tenant_id}/{project_id}`; consumer group per-tenant if strict isolation needed. |
| `ack` | Commit offset for the acked message's partition. |
| `retry_or_dlq` | Retry: re-produce to the same topic with incremented `attempts` header; DLQ: produce to `bus-dlq` topic. |
| `replay_dead_letter` | Consume from `bus-dlq`, filter by `message_id`, re-produce to the source topic. |
| `dlq` | Subscribe to `bus-dlq` topic with `earliest` offset and drain. |
| `depth` / `depth_for_kind` | Admin API `list_offsets` to compute consumer lag per partition; depth = sum of lag values. |

**Ordering note:** Kafka partitions deliver ordered within a partition.
`consume_scoped_kind_batch` relies on a stable partition key; consumers must not
reassign partitions mid-batch.

**No client deps yet:** add `rdkafka` or `rskafka` to `beater-bus-kafka/Cargo.toml`
only when wiring begins; keep out of `beater-bus` core.

**Acceptance criteria:**

1. `KafkaBus` passes the `trait_round_trip` helper.
2. A container test (`testcontainers-modules::kafka` / Redpanda) runs round-trip
   including DLQ and replay paths.
3. `depth` is computed from consumer lag, not from a broker-side count, so it
   reflects unprocessed messages only.
4. The `beater-ingest` and `beater-api` crates compile and pass their tests with
   `KafkaBus` wired — zero product-code changes required.

---

## 5. Phased rollout plan

### Phase 0 — Trait seam (done, on `main`)

- `DurableBus` trait in `beater-bus` with `InMemoryBus` and `SqliteDurableBus`
  implementations.
- Pluggability proof tests: `backend_pluggability_in_memory_bus` and
  `backend_pluggability_sqlite_durable_bus`.
- All callers (`beater-api`, `beater-ingest`, `beater-mcp`, `beater-otlp`) hold
  `Arc<dyn DurableBus>`.

### Phase 1 — NATS JetStream adapter (OSS/hosted parity)

- Prerequisite: §20.2 #0.1 backend selector in `beaterd` (runtime
  `BUS_BACKEND` env var).
- New crate `beater-bus-nats`; no changes to `beater-bus` or callers.
- Container test suite gating CI merge.
- Ship when NATS is added to the `beaterd` deployment manifest.

### Phase 2 — Kafka adapter (enterprise)

- Prerequisite: Phase 1 complete and at least one NATS-backed deployment
  running in production for 30 days.
- New crate `beater-bus-kafka`; no changes to `beater-bus` or callers.
- Acceptance tested against Redpanda in CI.
- Ship as an enterprise add-on behind a `BUS_BACKEND=kafka` flag.

### Phase 3 — Vercel Queues adapter (hosted edge)

- Lightweight HTTP adapter wrapping the Vercel Queues REST API.
- Durability semantics (at-least-once, idempotency key in body) satisfied
  by Vercel's delivery guarantee.
- No persistent state on the adapter side; lease/inflight tracking delegated
  to the queue service.

---

## 6. Adding a new backend — checklist

1. Create `crates/beater-bus-{name}/` with `Cargo.toml` depending on
   `beater-bus = { path = "../beater-bus" }` and your broker client crate.
   Do **not** add broker deps to `beater-bus` itself.
2. Implement `DurableBus` for your type.
3. Call the shared `trait_round_trip` helper (or replicate it in an integration
   test) to prove the seam.
4. Add a crash-recovery test (kill and restart your backend mid-inflight; verify
   unacked messages re-deliver).
5. Wire via `Arc<dyn DurableBus>` in `beaterd`'s startup block behind an env
   var — no product-code changes should be required.
6. Update the §8.2 data-planes table in `ARCHITECTURE.md` status column from
   `[planned]` to `[built]` or `[built, unwired]` as appropriate.
