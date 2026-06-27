# Contributing to Beater

Beater is an open-source (Apache-2.0 core) Rust agent observability, replay, and
eval platform with a hosted edition. Contributions are welcome — from a typo fix
to a new evaluator, a storage backend, or an SDK improvement. Please read
[ARCHITECTURE.md](ARCHITECTURE.md) first; it is the build-ready plan, and
[ARCHITECTURE.md §22](ARCHITECTURE.md#22-testing-verification--acceptance) tells
you exactly which tests and verification commands a given change needs.

## How every change ships: the Pull Request

**All code changes go through a Pull Request — no direct pushes to `main`.**

**Every PR description must state, explicitly:**

1. **WHAT it is** — the change in one or two sentences.
2. **WHY we have it** — the problem it solves or the capability it adds (link the
   relevant ARCHITECTURE.md section or the issue).
3. **HOW you tested it** — the exact commands you ran and what you observed
   (unit/integration tests, a `beaterctl` fixture, a live smoke, a UI click-through).
   "It compiles" is not a test.

PRs that do not answer all three are sent back. Keep PRs focused; a reviewer
should be able to hold the whole change in their head.

## CI/CD must be green before a PR is merged

A PR cannot merge until **every** required CI gate passes. The gates (under
`.github/workflows/`) are:

| Workflow | What it guards |
| --- | --- |
| `backend` | `cargo fmt`, `cargo clippy -D warnings` (unwrap/expect denied), `cargo test --workspace` |
| `sdk-contract` | spec ↔ served routes, spec ↔ all 7 SDK clients, semconv ↔ SDKs, `oasdiff` breaking-change check — **single-source-of-truth contract must show zero drift** |
| `storage-backends` | trait-conformance suite across SQLite / in-memory (and the wired columnar backends as they land) |
| `browser` | browser-agent observability crates and the Playwright/CDP/WebDriver harness |
| `frontend` | `web/dashboard` build, lint, and generated-OpenAPI-client checks |
| `gate1-live-smoke` | live `beaterd` OTLP HTTP/gRPC round-trip becomes queryable/searchable |
| `gate2-proof-contract` | the clean-clone-to-browser proof template and proof-artifact fixtures |
| `container-images` | multi-arch GHCR image build/publish for `beaterd`, dashboard, and demo runners |

The **single-source-of-truth contract** — `sdks/openapi/beater-api.json` →
7 SDK clients → MCP tools → CLI → docs (and `sdks/semconv/conventions.json` for
span kinds/attributes) — must regenerate to **zero drift**. Run the full local
check before you push:

```bash
scripts/check-contract-sync.sh
```

If you cannot get CI green, mark the PR a draft and say what is failing. Do not
ask a maintainer to override a red gate.

## Local development setup

```bash
# 1. Rust toolchain (stable) + the test/bench tooling
rustup toolchain install stable
cargo install cargo-nextest

# 2. Build & run the test suite
cargo build --workspace
cargo test --workspace            # or: cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# 3. Run the all-in-one server locally
cargo run -q -p beaterd -- --data-dir /tmp/beaterd

# 4. Smoke an OTLP round-trip (see ARCHITECTURE.md §22 for the full matrix)
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke

# 5. Full self-host stack via Docker Compose
docker compose up
```

Docker is required for `scripts/regen-sdks.sh`, the live SDK conformance suite,
and the compose-based gates. See
[ARCHITECTURE.md §22](ARCHITECTURE.md#22-testing-verification--acceptance) for
the per-component test plan and the "how to verify it's running" commands, and
[SECURITY.md](SECURITY.md) for how to report a vulnerability.

## The one rule: the OpenAPI contract is the single source of truth

The HTTP API, the 7 SDK clients, the MCP tools, the CLI, and the docs are **all
generated from one artifact** — `sdks/openapi/beater-api.json`, which is itself
generated from the Rust handlers in `crates/beater-api`. Never hand-edit a
generated client, the spec snapshot, or `sdks/semconv/conventions.json`.

### When you add or change a `/v1` endpoint

Do all of this in the same change (CI enforces it — see below):

1. **Handler + contract.** Add the axum route in `crates/beater-api/src/lib.rs`
   and annotate the handler with `#[utoipa::path(...)]`: a unique camelCase
   `operation_id`, a resource `tag`, params, `request_body`, and **every**
   response it can return (including `4xx`/`429`/`413`/`422`), each error using
   the shared `ErrorResponse` body. Request/response types must derive
   `utoipa::ToSchema`.
2. **Regenerate everything from the contract** — one command:
   ```bash
   cargo xtask regen-spec      # spec + dashboard snapshot
   scripts/regen-sdks.sh       # 7 clients (+ reproducible C/C++ patches)
   cargo xtask regen-semconv   # sdks/semconv/conventions.json (if conventions changed)
   ```
3. **If you touched span kinds / attribute keys**, change them ONLY in
   `crates/beater-schema` (`conventions` module) and re-run `regen-semconv`; then
   update each SDK's `semconv` to match (the gate will tell you what's missing).
4. **Commit the regenerated output** (generated clients are committed on purpose).

### Verify there is no drift — one command

```bash
scripts/check-contract-sync.sh
```

This runs every drift gate: spec ↔ served routes, spec ↔ all 7 SDK clients,
the API-shape audit, and semantic-conventions ↔ all 5 SDKs. CI runs the same
gates in `.github/workflows/sdk-contract.yml`, so **a handler change that isn't
regenerated into the spec, SDKs, MCP tools, and docs cannot merge**, and
`oasdiff` blocks breaking contract changes.

Because the MCP tools and CLI resolve operations from the spec at runtime, they
update automatically — no separate step. The docs site renders the committed
spec, so it updates too.

## Building & testing

- `cargo build --workspace` / `cargo test --workspace`
- `cargo clippy --workspace --all-targets` (unwrap/expect are denied in non-test code)
- Live SDK conformance (needs Docker + language toolchains):
  `scripts/e2e-clients-live.sh`
