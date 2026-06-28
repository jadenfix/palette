# Beater

<p align="center">
  <img src="docs/assets/beater-logo.svg" width="104" alt="Beater logo">
</p>

<p align="center">
  <strong>Rust-first OSS agent observability, replay, eval, and CI gate platform.</strong>
</p>

<p align="center">
  <a href="https://github.com/jadenfix/beater/actions/workflows/backend.yml"><img alt="backend" src="https://github.com/jadenfix/beater/actions/workflows/backend.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/jadenfix/beater/actions/workflows/sdk-contract.yml"><img alt="sdk-contract" src="https://github.com/jadenfix/beater/actions/workflows/sdk-contract.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/jadenfix/beater/actions/workflows/frontend.yml"><img alt="frontend" src="https://github.com/jadenfix/beater/actions/workflows/frontend.yml/badge.svg?branch=main"></a>
  <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache--2.0-3fb5ff"></a>
</p>

Beater is a local-first platform for understanding, replaying, and improving AI
agent behavior. It preserves agent traces, lets you inspect the trace/span tree,
turns failures into versioned datasets, runs evals over candidate releases, and
uses experiment reports as CI gates.

```text
instrument agent -> inspect trace/span tree -> promote failure to dataset -> run evals
-> compare candidate -> gate CI -> monitor production
```

The OSS runtime is centered on one all-in-one Rust binary, `beaterd`, plus a
Next.js dashboard and generated API/SDK surfaces. Hosted scale, SSO, billing,
managed cells, and enterprise governance are architecture tracks, not required
for the local OSS loop.

## Current Scope

| Area | In this repo today |
| --- | --- |
| Runtime | `beaterd` serves API, OTLP ingest, local jobs, SQLite-backed state, and durable bus workers. |
| Ingest | OTLP HTTP/gRPC, native trace ingest, source import primitives, raw envelope preservation, and canonical projections. |
| Dashboard | `web/dashboard` reads generated OpenAPI types and provides trace lists, span waterfalls, span detail, filters, and audited I/O unmask controls. |
| Datasets and evals | Trace-to-dataset promotion, immutable dataset versions, deterministic and judge-backed eval primitives, experiments, reports, and CI gate runs. |
| Contract | `/v1` handlers in `crates/beater-api` generate `sdks/openapi/beater-api.json`, dashboard snapshots, generated SDK clients, MCP/CLI surfaces, and semantic conventions. |
| Storage path | SQLite and in-memory conformance coverage for the OSS path; Postgres/ClickHouse migrations are scale-path contracts. |

## Quickstart

Run the current self-host topology:

```bash
git clone https://github.com/jadenfix/beater.git
cd beater
docker compose up
```

Default local endpoints:

| Service | URL |
| --- | --- |
| Dashboard | `http://127.0.0.1:3000` |
| API | `http://127.0.0.1:8080` |
| OTLP gRPC | `http://127.0.0.1:4317` |
| OTLP HTTP/protobuf | `http://127.0.0.1:8080/v1/otlp/<tenant>/<project>/<environment>/v1/traces` |

Smoke an OTLP round trip without running the full test suite:

```bash
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
```

Or run the all-in-one server and smoke it remotely:

```bash
cargo run -q -p beaterd -- --data-dir /tmp/beaterd
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080
```

Point a stock OpenTelemetry exporter at the local OTLP port:

```bash
python3 -m venv /tmp/beater-otel
/tmp/beater-otel/bin/pip install opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 \
  /tmp/beater-otel/bin/python examples/python/five_line_otel.py
```

## Repository Map

| Path | Purpose |
| --- | --- |
| `bins/beaterd` | Main local runtime. |
| `bins/beaterctl` | CLI, smoke commands, and fixtures. |
| `crates/*` | Rust libraries for schema, ingest, storage, bus, API, MCP, evals, replay, auth, datasets, gates, human review, alerts, audit, browser capture, and xtask. |
| `web/dashboard` | Next.js dashboard generated against the read API snapshot. |
| `sdks/openapi` and `sdks/clients/*` | Generated OpenAPI contract and generated clients. Do not hand-edit. |
| `sdks/{python,typescript,rust}` | Hand-written SDK surfaces and examples around the generated contract. |
| `migrations/{sqlite,postgres,clickhouse}` | Durable schema contracts for local runtime and scale paths. |
| `scripts/*` | Contract drift checks, SDK regeneration, compose/browser smoke gates, and release helpers. |

## Development

Focused checks for common README-adjacent and contract work:

```bash
cargo fmt --all
cargo test -p beater-api
cargo test -p beater-api --test openapi_coverage
cargo test -p beater-store-conformance
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
```

Contract changes must regenerate from source and prove zero drift:

```bash
cargo xtask regen-spec
scripts/regen-sdks.sh
cargo xtask regen-semconv
scripts/check-contract-sync.sh
```

Dashboard checks run from `web/dashboard`:

```bash
npm ci
npm run generate:api
npm test
npm run build
```

Run only the checks that match your change. For the full verification matrix,
see [ARCHITECTURE.md](ARCHITECTURE.md) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Project Status

Beater is early and actively evolving, but the local vertical slice is real:
`beaterd`, OTLP/native ingest, trace read APIs, the dashboard, contract
generation, dataset/eval primitives, experiment comparison, and gate policy
fixtures are present in this repository.

The hard rule is contract discipline: generated OpenAPI snapshots, generated
SDK clients, dashboard generated API types, and semantic-convention JSON are
outputs. Change the Rust/API/schema source, regenerate, and commit the generated
result in the same PR.

## Contributor and Agent Notes

- [ARCHITECTURE.md](ARCHITECTURE.md) is the build-ready plan and the source for
  requirement intent.
- [docs/architecture-status.md](docs/architecture-status.md) is the built/partial/planned
  status ledger cross-checked against the actual repo tree.
- [CONTRIBUTING.md](CONTRIBUTING.md) defines PR shape, CI gates, local setup,
  and contract regeneration.
- [CLAUDE.md](CLAUDE.md) states the hard contract-regeneration rule for coding
  agents.
- Honor any local `AGENTS.md` file in your worktree when present.
- Report vulnerabilities privately via [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
