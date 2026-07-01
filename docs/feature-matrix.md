# Beater feature matrix and open-core boundary

This document is the **public, before-launch** statement of what is in the
open-source core versus what is part of the (future) commercial Beater Cloud
offering. It pairs with the root [`LICENSE`](../LICENSE) (Apache-2.0) and
[`GOVERNANCE.md`](../GOVERNANCE.md) (the no-rug-pull promise). It satisfies
requirement **R12.1** (license + open-core boundary are public before launch).

## License

The entire repository — the `beaterd` server, all 7 SDK clients, the MCP tools,
the CLI, and the dashboard — is licensed under **Apache-2.0**. There is no
"open core that quietly relicenses." See [`GOVERNANCE.md`](../GOVERNANCE.md) for
the durability promise.

## What "open core" means here

The **contract and the engine are open.** The OpenAPI contract
(`sdks/openapi/beater-api.json`), the canonical schema/semantic conventions
(`crates/beater-schema`, `sdks/semconv`), and the self-hostable `beaterd` server
are all Apache-2.0 and will stay that way. Commercial offerings are *operational
convenience* (hosting, scale connectors, support) layered on the same contract —
never a fork of the data model or a paywall on the SDK protocol.

## Feature matrix

| Capability | Open-source core (Apache-2.0, self-host) | Beater Cloud (future, hosted) |
| --- | --- | --- |
| All-in-one `beaterd` server | Yes | Yes (managed) |
| OpenAPI `/v1` contract + 7 SDK clients | Yes | Yes (identical contract) |
| MCP tools + `beater` CLI | Yes | Yes |
| Zero-SDK OTLP ingest (HTTP + gRPC) | Yes | Yes |
| SQLite + filesystem local backends | Yes | n/a (managed storage) |
| Postgres backend | Yes | Yes (managed) |
| ClickHouse backend (scale) | Yes (self-managed) | Yes (managed, autoscaled) |
| Parquet + DataFusion cold retention | Yes | Yes |
| Tail-based sampling, DLQ, replay | Yes | Yes |
| Eval / judge / gates / calibration | Yes | Yes |
| Dashboard (Next.js UI) | Yes | Yes (hosted control plane) |
| Security primitives (scoped keys, audit events, redaction) | Yes | Yes (managed governance/export UX) |
| Self-host usage telemetry | Opt-out (off by default) | n/a |
| SSO / SCIM / enterprise RBAC | Community RBAC primitives | Managed enterprise add-ons |
| SLA, dedicated support, hosted scale | Community support | Commercial |
| Managed multi-region / autoscaling | Bring-your-own infra | Yes |

Anything in the "Cloud" column is *operational*, not a capability removed from
the core: the open-source server can do everything the protocol allows. The
commercial line is hosting, scale, and support — see the promise in
[`GOVERNANCE.md`](../GOVERNANCE.md).

## Where the boundary is enforced

- The contract is the single source of truth (`CONTRIBUTING.md`). The same spec,
  SDKs, MCP tools, and CLI work against self-host and Cloud.
- Self-host telemetry is **opt-out** (R12.5): `beaterd` makes no outbound
  telemetry call unless `BEATER_SELF_HOST_TELEMETRY` is explicitly set. The
  single source of truth is `beater_core::SelfHostTelemetryConfig`.
- The future service split is **logical, not operational** (R1.2): the all-in-one
  `beaterd` is the only mandatory binary; thin role binaries are opt-in behind
  the `thin-bins` cargo feature.
