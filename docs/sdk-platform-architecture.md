# Beater Client Platform — Architecture & API Design

How the **API, MCP server, CLI, and 7 language SDKs** stay coherent, simple, and
impossible to drift. One contract, generated outward.

## Principle: one source of truth, generated outward

```
crates/beater-api handlers  ──#[utoipa::path] + ToSchema on the REAL types──┐
                                                                            v
                                       sdks/openapi/beater-api.json  (OpenAPI 3.1)
        +------------------+------------------+------------------+-----------------+
        v                  v                  v                  v                 v
  7 SDK clients        /mcp tools          beater CLI         docs site      conformance
  (openapi-generator)  (1 per operation)   (shared client)   (renders spec)  (live, per lang)
```

Nothing re-describes the API by hand. A handler change regenerates the spec; CI
(`sdk-contract.yml`) fails if the spec, any SDK client, or MCP tool set is stale,
and `oasdiff` blocks breaking changes. Drift is a merge-blocker, not a hope.

## Modern API design conventions (enforced)

- **Resource-tagged, action-named.** Every operation has a resource `tag`
  (`datasets`, `traces`, ...) and a globally unique dotted/kebab `operationId`
  (`datasets.create-dataset`, `traces.list-traces`, `evals.run-judge-eval`). Tags -> SDK API classes;
  operationIds -> method names. Uniqueness avoids single-package collisions
  (Go/C/Java) so names are consistent across all 7 languages without per-language
  hacks.
- **Uniform error model.** Every non-2xx is one
  `ErrorResponse { error, message, status }` -> one typed error per SDK.
  `error` is the stable snake_case machine code; `status` is retained only as a
  deprecated `/v1` compatibility HTTP status code.
- **Typed everything.** No bare `object`/`any` responses; every response is a
  named schema. Discriminated unions use an internal `type` tag (e.g.
  `EvaluatorKind`) so they generate cleanly in strict languages.
- **Cursor pagination** for list endpoints (`limit` + `next_cursor`).
- **Explicit tenancy.** `tenant`/`project`/`environment` are path-scoped; the SDK
  ergonomic layer binds them once at `init()` so callers never repeat them.
- **Versioned.** All routes under `/v1`; `info.version` tracks the workspace
  version; SDKs publish in lockstep.

## Two SDK layers (simple by default, powerful when needed)

1. **Ergonomic (Layer 2, hand-written):** `init()` + `@observe`/`observe()` +
   `wrap_openai()/wrap_anthropic()` + LangChain/LlamaIndex callbacks. Built on
   OpenTelemetry (ingest is OTLP). One shared `semconv` module per language
   mirrors the server normalizer. This is the 5-line path most users want.
2. **Control-plane (Layer 1, generated):** typed CRUD for datasets, experiments,
   gates, evals, usage, audit, etc. -- `client.datasets.datasets.create-dataset(...)`.

The **CLI** and **Rust SDK** consume the same generated Layer-1 client, and the
**MCP** server dispatches tool calls through the same in-process router with the
same auth -- so all four surfaces are literally the same operations.

## Robust & scalable

- **Robust:** the contract is type-checked in Rust; `openapi_coverage` asserts
  spec == served routes; per-language **live conformance** drives each generated
  client against a running `beaterd`; MCP has parity tests vs direct HTTP; the
  ergonomic SDKs have unit + live E2E.
- **Scalable:** adding an endpoint = annotate one handler + `regen-sdks.sh`; all
  7 SDKs, MCP tools, CLI commands, and docs update from that single change.
  Adding a language = one `sdks/config/<lang>.yaml`.

## Easy to use

- Python: `pip install beater-sdk` -> `beater.init()` -> `@beater.observe(...)`.
- TypeScript: `npm i @beater/sdk` -> `beater.init()` -> `observe(fn)`.
- CLI: `beater api <operationId> --param k=v` reaches any endpoint; typed
  sugar (`beater traces list`) for common ones.
- MCP: point any MCP client at `/mcp`; every API operation is a tool.
- Docs: `/docs` renders the live spec + tool catalog + per-language quickstarts.
