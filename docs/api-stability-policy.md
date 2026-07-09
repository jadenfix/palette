# `/v1` API stability and deprecation policy

This document defines the stability guarantees and deprecation process for the
Beater HTTP API. It satisfies requirement **R11.5** (`/v1` API is stable and
versioned) and complements the OpenAPI spec
(`sdks/openapi/beater-api.json`) and `CONTRIBUTING.md`.

## The contract is the single source of truth

Every `/v1` endpoint, request/response type, MCP tool, CLI command, and SDK
client is generated from one artifact: `sdks/openapi/beater-api.json` (itself
generated from the Rust handlers in `crates/beater-api`). The drift gate
(`scripts/check-contract-sync.sh`) and `oasdiff` in
`.github/workflows/sdk-contract.yml` block any change that is not regenerated
across the spec, all 7 SDK clients, the MCP tools, and the docs.

## Pre-1.0 caveat

Beater has not declared 1.0 yet. Until that milestone, canonical and API
schemas may evolve freely and **no wire/SDK backward-compatibility is promised
before 1.0**. That caveat does not relax the contract discipline: every `/v1`
handler change still regenerates the OpenAPI spec, generated SDK clients,
semantic conventions, MCP/CLI/docs contract surfaces, and then runs
`scripts/check-contract-sync.sh` plus the `sdk-contract` CI gate before merge.
The only pre-1.0 breaking exceptions allowed in CI are explicit contract
alignment breaks filtered by `scripts/filter-oasdiff-breaking.py`; unexpected
`oasdiff` errors still fail the merge gate.

## Stability guarantee for `/v1`

While the API is at `/v1`:

- **No breaking changes are made in place.** A change that removes an endpoint,
  removes a field, narrows an enum, changes a field's type, or makes an optional
  request field required is a breaking change and is **blocked by `oasdiff`** in
  CI.
- **Additive changes are allowed without a version bump.** New endpoints, new
  optional request fields, and new response fields are backward compatible and
  may ship under `/v1`. Clients must tolerate unknown response fields.
- **Error shape is stable.** All error responses use the shared `ErrorResponse`
  body. New error codes may be added; existing codes keep their meaning.

## Versioning model

- The path prefix (`/v1`) is the major version. A breaking change that cannot be
  made additive ships under a **new prefix** (`/v2`) — `/v1` is not mutated.
- During any `/v1` -> `/v2` transition, `/v1` remains served and supported for
  the deprecation window below.
- SDK client packages follow semver. A new minor adds endpoints/fields; a major
  is only released for a new API major version.

## Deprecation process

1. **Announce.** A deprecation is recorded in the changelog and the endpoint is
   marked `deprecated: true` in the OpenAPI spec (which propagates to the SDKs,
   MCP tools, and docs automatically).
2. **Warn at runtime.** Deprecated endpoints continue to function and respond
   with a `Deprecation` header (and, where applicable, a `Sunset` header per
   RFC 8594) so callers can detect usage in the wild.
3. **Provide a migration path.** The replacement endpoint/field ships and is
   documented before the deprecated surface is removed.
4. **Honor the window.** A deprecated `/v1` surface is supported for at least
   **6 months** (and at least one minor SDK release) after the announcement
   before it can be removed, and removal only happens under a new major version
   prefix.

## What this means for self-host

Self-hosted `beaterd` and Beater Cloud serve the identical contract, so the same
guarantees apply to both. Because OSS runs without Beater Cloud (R1.3), the
stability of `/v1` is what your agents, SDKs, and dashboards depend on — it is
governed, versioned, and CI-enforced, not maintained by convention.

See also: [`GOVERNANCE.md`](../GOVERNANCE.md) (no-rug-pull promise) and
[`docs/feature-matrix.md`](feature-matrix.md) (open-core boundary).
