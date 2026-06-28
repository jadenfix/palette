# Beater — working rules

## The contract is the single source of truth (do not break this)

The HTTP API, the 7 SDK clients (`sdks/clients/*`), the MCP tools (`/mcp`), the
CLI (`beater api`), and the docs are ALL generated from one artifact —
`sdks/openapi/beater-api.json`, generated from the Rust handlers in
`crates/beater-api`. Span kinds + attribute keys come from one source too
(`crates/beater-schema` `conventions` module → `sdks/semconv/conventions.json`).

**When you add or change a `/v1` endpoint** (or a request/response type, or a
span kind/attribute), you MUST regenerate everything in the same change:

```bash
./beater-cli update-schema
```

Annotate every handler with `#[utoipa::path]` (unique camelCase `operation_id`,
resource `tag`, all responses incl. 4xx using the shared `ErrorResponse`); never
hand-edit generated clients, the spec snapshot, or `conventions.json`.

**Verify no drift before pushing — one command:**

```bash
./beater-cli update-schema --check
```

CI (`.github/workflows/ci.yml`, job `algorithms`) runs the same gates, so a handler change
that isn't regenerated into the spec/SDKs/MCP/docs/conventions cannot merge.
MCP tools and the CLI resolve operations from the spec at runtime, so they stay in
sync automatically. See `CONTRIBUTING.md` for the full workflow.
