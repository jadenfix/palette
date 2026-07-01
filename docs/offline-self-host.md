# Offline self-host: running Beater without Beater Cloud

Beater is built to run fully self-hosted, **offline**, with no dependency on
Beater Cloud. This document describes the offline posture and how to prove it,
and satisfies requirement **R1.3** (OSS runs without Beater Cloud).

## TL;DR

The default `docker compose up` runs a single `beaterd` process plus the
dashboard. It makes **no outbound calls except to configured providers**
(the providers you configure)
(e.g. the LLM provider your agents call, or an OTLP collector you explicitly
point at). Beater itself never phones home:

- Self-host usage telemetry is **opt-out** and off by default (R12.5). `beaterd`
  contacts no `telemetry.beater.dev` endpoint unless you set
  `BEATER_SELF_HOST_TELEMETRY` (see below).
- There is no license-key check, no mandatory Beater Cloud account, and no
  "community edition can't run in production" gate (see `GOVERNANCE.md`).
- External backends (Postgres, NATS, MinIO, ClickHouse) are **opt-in** behind
  compose profiles. The default path uses embedded SQLite + the local
  filesystem, so there is nothing extra to reach over the network.
- Vercel is not required for self-host. The dashboard runs in the local compose
  `dashboard` service and talks to the local `beaterd`; hosted Vercel deploys
  are dashboard/control-plane surfaces only, not the `beaterd` runtime.
- The compose quickstart explicitly runs `beaterd --auth-mode local` for offline
  demo ergonomics. Run `beaterd` with its default `--auth-mode required` and
  bootstrap API keys before exposing it beyond local development.

## What egress exists, and why

The only outbound traffic from a default offline deployment is the egress *you*
configure:

| Egress | When it happens | How to control it |
| --- | --- | --- |
| LLM provider API | Only when your agent code calls a provider | Your agent code / provider keys |
| OTLP export to beaterd | Your apps -> `beaterd:4317` (inside the network) | `OTEL_EXPORTER_OTLP_ENDPOINT` |
| Self-host telemetry | Never, unless explicitly opted in | `BEATER_SELF_HOST_TELEMETRY` (default off) |

Everything internal (dashboard -> `beaterd:8080`, smoke tools -> `beaterd:4317`,
health probes -> `127.0.0.1`) stays inside the compose network.

## Self-host telemetry (opt-out)

The single source of truth is `beater_core::SelfHostTelemetryConfig`:

- Default (`BEATER_SELF_HOST_TELEMETRY` unset / `0` / `false` / `off`): disabled,
  no outbound telemetry call, `endpoint()` is `None`.
- Explicit opt-in (`1` / `true` / `on` / `yes` / `enabled`): enabled, reports to
  the fixed, inspectable endpoint `https://telemetry.beater.dev/v1/usage`.
- Any unrecognized value fails closed (disabled).

`beaterd` logs its posture on startup so you can confirm it offline.

## Proving it offline

You can firewall all egress and Beater still works end to end:

1. Block outbound traffic except to your LLM provider (and, if you use it, your
   own OTLP collector).
2. `docker compose up beaterd dashboard`.
3. Run `examples/python/five_line_otel.py` (points at `beaterd:4317`).
4. Open the dashboard and click the trace.

No call to any `*.beater.dev` / `beater.cloud` host is required for any of this.
The test `bins/beaterd/tests/offline_compose.rs` enforces that the default
compose topology never wires a Beater Cloud host and keeps external backends
opt-in.

See also: [`GOVERNANCE.md`](../GOVERNANCE.md),
[`docs/feature-matrix.md`](feature-matrix.md).
