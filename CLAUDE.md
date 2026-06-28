# Beater — working rules

Start with `AGENTS.md` for the compact repo map and cross-agent context. This
file keeps the non-negotiable contract-generation rule close to Claude Code.

## The contract is the single source of truth (do not break this)

The HTTP API, the 7 SDK clients (`sdks/clients/*`), the MCP tools (`/mcp`), the
CLI (`beater api`), and the docs are ALL generated from one artifact —
`sdks/openapi/beater-api.json`, generated from the Rust handlers in
`crates/beater-api`. Span kinds + attribute keys come from one source too
(`crates/beater-schema` `conventions` module → `sdks/semconv/conventions.json`).

**When you add or change a `/v1` endpoint** (or a request/response type, or a
span kind/attribute), you MUST regenerate everything in the same change:

```bash
cargo xtask regen-spec      # OpenAPI spec + dashboard snapshot
scripts/regen-sdks.sh       # all 7 generated clients (+ reproducible C/C++ patches)
cargo xtask regen-semconv   # sdks/semconv/conventions.json (if conventions changed)
```

Annotate every handler with `#[utoipa::path]` (unique camelCase `operation_id`,
resource `tag`, all responses incl. 4xx using the shared `ErrorResponse`); never
hand-edit generated clients, the spec snapshot, or `conventions.json`.

**Verify no drift before pushing — one command:**

```bash
scripts/check-contract-sync.sh
```

CI (`.github/workflows/sdk-contract.yml`) runs the same gates, so a handler change
that isn't regenerated into the spec/SDKs/MCP/docs/conventions cannot merge.
MCP tools and the CLI resolve operations from the spec at runtime, so they stay in
sync automatically. See `CONTRIBUTING.md` for the full workflow.
