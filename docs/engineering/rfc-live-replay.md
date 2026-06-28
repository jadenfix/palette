# RFC: Live Replay Implementation Plan

Status: Draft PR plan
Companion issues: #216, #236, #239, #240, #243, #244, #245

## Thesis

Live replay should be an append-only, durable event log first and a browser stream
second. The stream is only a delivery adapter over persisted run/session events.
That keeps replay honest: a client that disconnects, opens late, or watches after
`beaterd` restarts gets the same ordered history instead of whatever happened to
be in memory.

This matches Beater's core constraints:

- one Rust binary first
- no cloud dependency in OSS
- immutable raw data plus normalized projections
- privacy and tenant isolation before hosted ingest
- honest replay labels when cassettes are incomplete

## Primary-source research

- MDN documents Server-Sent Events as a browser-native `EventSource` stream over
  `text/event-stream`, with event fields such as `event`, `data`, `id`, and
  reconnection behavior: https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
- axum exposes SSE through `axum::response::sse::{Sse, Event, KeepAlive}`, which
  fits Beater's existing axum API stack: https://docs.rs/axum/latest/axum/response/sse/index.html
- Tokio's broadcast channel is useful for best-effort fanout to current
  subscribers, but it is not a durable log and must not be the source of truth:
  https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html
- MCP Streamable HTTP treats agent-tool communication as a sessioned transport
  with resumability concerns, which supports the same product direction for live
  agent runs: https://modelcontextprotocol.io/specification/2025-06-18/basic/transports
- Next.js client components and route handlers give the dashboard a clean split:
  a client component owns the live subscription, while a same-origin route can
  proxy auth-sensitive stream requests if needed:
  https://nextjs.org/docs/app/api-reference/directives/use-client and
  https://nextjs.org/docs/app/api-reference/file-conventions/route
- AG-UI is converging on event-shaped agent UI integration. Beater should keep a
  native event model and map to AG-UI later instead of making AG-UI a storage
  dependency: https://github.com/ag-ui-protocol/ag-ui

## Current repo state

Built:

- `beaterd` already serves the API with axum.
- `crates/beater-replay` has persisted replay events, cassette construction, and
  deterministic/forked replay planning primitives.
- `crates/beater-api` exposes trace list/get/span read APIs and auth/redaction
  flows for static reads.
- The dashboard renders a trace waterfall and already has redaction-oriented
  tests.

Missing:

- no live replay or live trace-tail route
- no stream cursor, event id, or reconnect contract
- no dashboard `EventSource`/streaming client
- no CLI `trace tail` or `session watch`
- no E2E gate proving events appear before a run finishes
- no live-link privacy/regression test matrix

## First-principles design

The invariant is:

```text
persist event -> notify live subscribers -> client catches up from persisted log
```

Never:

```text
notify subscriber -> maybe persist later
```

That ordering means a dropped notification, slow subscriber, API restart, or page
reload can be repaired by backfill. The live stream improves latency; the event
store provides correctness.

## Event model

Introduce a `LiveReplayEvent` domain DTO, separate from `ReplayEvent` cassettes:

```rust
pub struct LiveReplayEvent {
    pub event_id: LiveReplayEventId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub environment_id: Option<EnvironmentId>,
    pub session_id: Option<SessionId>,
    pub trace_id: TraceId,
    pub span_id: Option<SpanId>,
    pub seq: u64,
    pub kind: LiveReplayEventKind,
    pub status: LiveReplayEventStatus,
    pub payload: LiveReplayPayload,
    pub redaction: RedactionState,
    pub source: LiveReplaySource,
    pub recorded_at: Timestamp,
}
```

Initial event kinds:

- `run.started`
- `span.started`
- `span.updated`
- `span.completed`
- `tool.requested`
- `tool.completed`
- `mcp.requested`
- `mcp.completed`
- `approval.requested`
- `approval.resolved`
- `artifact.created`
- `replay.mode_changed`
- `run.completed`
- `run.failed`
- `stream.heartbeat`

`seq` is monotonic inside `(tenant_id, project_id, trace_id)` and stable across
restarts. `event_id` should encode enough to resume safely, for example:

```text
lr_<trace_id>_<zero_padded_seq>_<content_hash_prefix>
```

## Storage and fanout

Add a small `beater-live` crate or a `beater-replay::live` module. Prefer a crate
once the API, CLI, and dashboard all depend on the type.

Core trait:

```rust
#[async_trait]
pub trait LiveReplayStore: Send + Sync {
    async fn append(&self, event: LiveReplayEvent) -> StoreResult<LiveReplayEvent>;
    async fn list_after(
        &self,
        scope: LiveReplayScope,
        after: Option<LiveReplayCursor>,
        limit: u32,
    ) -> StoreResult<Vec<LiveReplayEvent>>;
}
```

SQLite table:

```sql
CREATE TABLE IF NOT EXISTS live_replay_events (
  tenant_id TEXT NOT NULL,
  project_id TEXT NOT NULL,
  environment_id TEXT,
  session_id TEXT,
  trace_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  event_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  recorded_at TEXT NOT NULL,
  event_json TEXT NOT NULL,
  PRIMARY KEY (tenant_id, project_id, trace_id, seq)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_live_replay_event_id
ON live_replay_events (event_id);

CREATE INDEX IF NOT EXISTS idx_live_replay_session_order
ON live_replay_events (tenant_id, project_id, session_id, seq);
```

Runtime fanout:

- A bounded Tokio broadcast channel per active trace/session can wake subscribers.
- If a receiver lags, it must reconnect/backfill from `LiveReplayStore`.
- Ingest and replay code must not block on a subscriber.
- `beaterd` restart loses only in-memory fanout, not stream history.

## API contract

Add one endpoint first:

```text
GET /v1/traces/{tenant_id}/{project_id}/{trace_id}/events
```

Query:

- `environment_id`
- `after`
- `limit` for finite backfill
- `follow=true` for live tail
- `heartbeat_ms`

Headers:

- accept `Last-Event-ID` as an alias for `after`
- return `text/event-stream` for live follow
- return JSON page for `follow=false` if generated SDK support is needed

SSE event shape:

```text
id: lr_trace_00000000042_abcd1234
event: span.completed
data: {"event_id":"...","trace_id":"...","span_id":"...","seq":42,...}
```

Authorization:

- use the same trace-read project scope as static trace reads
- re-check auth before initial backfill
- close or downgrade the stream if long-lived permissions become invalid
- never expose unmasked payloads without the existing audited unmask path

## Dashboard implementation

Add a focused client component rather than rewriting the whole dashboard:

- `web/dashboard/app/live/[tenant]/[project]/[trace]/page.tsx`
- `LiveReplayTimeline.tsx` as a `"use client"` component
- same-origin route handler only if the browser needs cookie/header mediation
- `useLiveReplayEvents({ trace, after })` hook using `EventSource`

UI states:

- live
- connecting
- backfilling
- reconnecting
- stale
- completed
- forbidden
- expired
- replay-only

The selected span detail must remain stable while new events arrive. Large traces
should use virtualized rows before the UI claims readiness for production.

## CLI implementation

Add:

```bash
beaterctl trace tail --tenant demo --project demo --trace <id>
beaterctl session watch --tenant demo --project demo --session <id>
```

CLI behavior:

- print a compact event line for each event
- persist the last event id only when explicitly asked, not by default
- reconnect with exponential backoff
- show whether output is live, backfilling, or replay-only
- exit non-zero on auth/scope errors

## Test plan

Rust contract tests:

- appending events assigns monotonic ids
- `list_after` returns stable ordered backfill
- duplicate append is idempotent
- cross-tenant/project cursor use is rejected
- lagged broadcast receiver recovers via store backfill

API tests:

- `GET .../events?follow=false` returns ordered JSON backfill
- `GET .../events?follow=true` emits SSE `id`, `event`, and `data`
- `Last-Event-ID` resumes from the next event
- unauthorized tenant/project/trace subscriptions do not disclose existence
- redacted static trace reads and live events expose consistent payload visibility

Runtime smoke:

- start `beaterd`
- emit a delayed multi-span run
- subscribe before completion
- prove the stream receives intermediate events before `run.completed`
- kill/restart `beaterd`
- reconnect from last id and receive the missing suffix

Dashboard E2E:

- page updates without manual refresh
- reconnecting/backfilling states are visible
- selected detail remains stable while events append
- pending approval resolves from another tab
- forbidden/expired link does not leak tenant/project details

CLI E2E:

- `beaterctl trace tail` prints event order
- reconnect resumes without duplicates
- `--follow=false` exits after backfill

## Edge-case matrix

| Case | Required behavior |
| --- | --- |
| no live subscribers | events persist and are replayable later |
| slow subscriber | server drops or closes that subscriber without blocking ingest |
| duplicate reconnect | client de-dupes by `event_id` |
| out-of-order projection | UI shows pending parent/state until canonical trace catches up |
| large payload | stream emits redacted summary or artifact ref |
| trace archived/deleted | link degrades to a scoped unavailable state |
| permission revoked mid-stream | stream closes or emits safe terminal auth event |
| two approval decisions | one wins; the other sees resolved/conflict |
| incomplete cassettes | replay mode is labeled `forked_replay` or `simulation`, never deterministic |

## Suggested PR slices

1. Storage and DTOs: `LiveReplayEvent`, SQLite store, conformance tests.
2. API backfill: JSON `follow=false` endpoint plus auth/redaction tests.
3. API SSE: `follow=true`, `Last-Event-ID`, keepalive, lag recovery tests.
4. Runtime event emission: ingest/replay/span lifecycle emits live events.
5. CLI tail/watch commands with reconnect.
6. Dashboard live page and client hook.
7. E2E gate: local delayed-run fixture through API, CLI, and dashboard.
8. Share-link hardening: expiring scoped links and permission-change tests.
9. AG-UI adapter: map native events to AG-UI only after the native contract is
   stable.

## Non-goals for the first implementation

- WebSockets. SSE fits current Vercel/Rust-cell constraints and browser needs.
- Cross-region fanout. The first OSS implementation is single `beaterd`.
- Making AG-UI a storage dependency.
- Re-executing live tools during replay without the side-effect safety contract
  tracked in #141.
- Claiming deterministic replay when cassettes are missing or mismatched.

## Done definition

Live replay is done only when:

- a run can be watched before it finishes
- a late subscriber can replay the same history
- reconnects do not lose or duplicate semantic events
- static and live views enforce the same auth/redaction rules
- dashboard and CLI both use the same cursor contract
- CI has a deterministic E2E proof that fails on manual-refresh-only behavior
