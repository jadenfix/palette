# Ecosystem Integration Contract

This is the repo-local contract for how Beater integrates with the active
neighboring projects without making them depend on hosted Beater services.

Current neighbor context checked on 2026-07-06:

- `jadenfix/beater.js` exports completed agent runs to Beater with
  `BEATER_TRACE_EXPORT_URL`, `BEATER_OTLP_EXPORT_URL`, or standard
  `OTEL_EXPORTER_OTLP_*` variables.
- `jadenfix/tempo` active PRs are tightening replay order, live E2E evidence,
  session MCP policy, and CI gate proof.
- `jadenfix/beaterOS` models payment authority as mandates, budgets, receipts,
  and journals.
- `jadenfix/aether` models settlement with signed agent authorization and
  `PaymentEnvelope` objects; Beater stores traces/eval evidence, not settlement
  authority.

## Boundary

Beater stays standalone:

- The default OSS build and Docker path do not require Beater Cloud.
- Billing and Stripe stay out of the Beater product API; the central Tempera
  control plane owns checkout, plans, subscriptions, invoices, and webhooks.
- No local runtime may require a license key, mandatory phone-home, or mandatory
  hosted account to ingest, inspect, replay, evaluate, or gate traces.

Hosted Tempera may bill Beater usage through the central control plane, but that
surface is outside Beater's product API. It does not become the source of
authority for local agent execution.

## Inbound Trace Surfaces

Beater accepts ecosystem traces through stable, additive ingress paths:

- Collector-compatible OTLP HTTP/JSON:
  `POST /v1/traces`
- Scoped OTLP HTTP/protobuf:
  `POST /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces`
- Native canonical ingest:
  `POST /v1/traces/native`
- Importer-based source ingest:
  `POST /v1/import/{tenant_id}/{project_id}/{environment_id}`

The zero-lock-in floor is the OTLP trace data model. Collector-style OTLP/JSON
exporters may post directly to `/v1/traces`; Beater resolves tenant, project,
and environment from `x-beater-*` headers or Beater resource attributes.
Protobuf senders can use the scoped OTLP path. Any richer adapter must remain
optional and map back to canonical spans.

## Active Neighbor Repos

| Repo | Current Beater-side contract | Must not require |
| --- | --- | --- |
| Tempo | Send browser/session spans through collector-style OTLP/JSON or Beater's scoped protobuf endpoint; Beater normalizes them into canonical trace views. | Stripe config, hosted account |
| beater.js | Export agent runs and tool calls through `BEATER_OTLP_EXPORT_URL`, `BEATER_TRACE_EXPORT_URL`, collector-style OTLP/JSON, Beater's scoped OTLP/protobuf, or native canonical ingest; Beater displays them as `agent.run`, `llm.call`, and `tool.call` spans. | Billing feature, hosted dashboard, live model credentials |
| beaterOS | Export receipts, journals, and audit spans as traces or artifacts; Beater observes and gates outcomes. | Hosted billing as local payment authority |
| Aether | Anchor agent settlement evidence by carrying run, step, receipt, and `PaymentEnvelope` identifiers as trace attributes; Beater can evaluate and retain off-chain evidence for disputes. | Beater billing as an AIC/SWR wallet, escrow, paymaster, or settlement authority |

beaterOS owns local authority: grants, spend limits, payment mandates, receipts,
and journal verification. Central Tempera billing may meter hosted Beater usage,
but it must not authorize or block local beaterOS actions.

Aether owns settlement authority for AIC/SWR escrow, agent authorization, and
payment-envelope verification. Beater may retain OTLP/native trace evidence such
as `beateros.payment_mandate_id`, `beateros.receipt_requirement`,
`aether.payment_envelope_id`, and `aether.agent_payment_authorization`, but
those fields are observed metadata. They must not cause the OSS Beater runtime
to release funds, enforce payment mandates, or require Aether.

## Billing Boundary

Billing integration is control-plane-owned:

- `crates/beater-billing` may retain internal billing primitives for future
  hosted workers and migrations.
- `beater-api` exposes no billing, subscription, invoice, plan, or Stripe
  webhook routes.
- `beaterd` opens no billing store and wires no Stripe webhook route.
- The usage ledger remains append-only; refunds are compensating entries that
  net down invoice quantities.
- Payment mandates, `PaymentEnvelope` signatures, AIC/SWR escrow, x402-style
  commerce flows, and beaterOS payment receipts remain external authority inputs
  that Beater may observe in traces but never treats as hosted billing state.

This preserves self-hosted operation while giving hosted deployments one
central path to metered billing.

## Verification

The static checker `scripts/check-ecosystem-contract.py` guards this document's
markers against drift from Beater-side code and governance docs. Focused runtime
coverage lives in:

- `cargo test -p beater-api --test openapi_coverage`
- `cargo test -p beater-api api_accepts_collector_otlp_json_and_reads_canonical_trace`
- `cargo test -p beater-otlp --lib`
- `cargo test -p beater-api --test route_inventory`
- `cargo test -p beater-billing`
