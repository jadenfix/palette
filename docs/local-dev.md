# Local development bootstrap

Everything you need to build, run, and test Beater locally. This supports
requirement **R12.4** (a contributing path exists) and complements
[`CONTRIBUTING.md`](../CONTRIBUTING.md) (the contract workflow) and the
[issue/PR templates](../.github/).

## Prerequisites

- **Rust** (stable) with `cargo`, `rustfmt`, and `clippy`:
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup component add rustfmt clippy
  ```
- **Docker** + **Docker Compose v2** (for the self-host compose path).
- **Node 24+** (for the dashboard / Playwright e2e).
- **Python 3.12+** (for the zero-SDK OTLP example apps).

## One-time bootstrap

```sh
git clone https://github.com/jadenfix/beater.git
cd beater

# Rust: build and test the whole workspace.
cargo build --workspace
cargo test --workspace

# Dashboard deps.
cd web/dashboard && npm ci && cd ../..
```

## Run the all-in-one server

The supported deployment is the single `beaterd` binary (R1.2):

```sh
# Native:
cargo run -p beaterd -- --auth-mode local

# Or the full self-host compose (beaterd + dashboard), offline by default (R1.3):
docker compose up beaterd dashboard
```

`beaterd` listens on `:8080` (HTTP API + MCP at `/mcp`), `:4317` (OTLP gRPC), and
`:4318` (OTLP HTTP), and stores data under `.beater/` (SQLite + filesystem) — no
external services needed. The native command and compose files opt into
`--auth-mode local`; the default `beaterd` mode requires API-key auth.

## Send your first trace (zero SDK)

```sh
pip install opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
python examples/python/five_line_otel.py
```

Then open the dashboard (`http://localhost:3000`) and click the trace. More
example apps live under `examples/` (OpenInference, OpenLLMetry, OTel GenAI,
FastAPI, Flask, Express, LlamaIndex, and the Rust `tracing`/axum/tonic/reqwest/MCP
examples).

## The one rule: regenerate from the contract

The HTTP API, the 7 SDK clients, the MCP tools, the CLI, and the docs are all
generated from `sdks/openapi/beater-api.json`. If you change a `/v1` endpoint,
request/response type, span kind, or attribute, regenerate in the same change:

```sh
cargo xtask regen-spec      # OpenAPI spec + dashboard snapshot
scripts/regen-sdks.sh       # all 7 generated clients
cargo xtask regen-semconv   # conventions.json (if conventions changed)
```

Verify there is no drift before pushing:

```sh
scripts/check-contract-sync.sh
```

## Lint and format before a PR

```sh
cargo fmt --all
cargo clippy --workspace --all-targets   # unwrap/expect denied in non-test code
```

## Telemetry posture (offline-friendly)

Self-host telemetry is **opt-out** (R12.5). Local dev makes no outbound telemetry
call. To opt in for testing, set `BEATER_SELF_HOST_TELEMETRY=1`; see
[`docs/offline-self-host.md`](offline-self-host.md).

## Opening a PR

Use the [pull request template](../.github/PULL_REQUEST_TEMPLATE.md). The
contract gates in CI (`.github/workflows/sdk-contract.yml`) run the same checks
as `scripts/check-contract-sync.sh`, so a handler change that is not regenerated
cannot merge.
