<!--
Thanks for contributing to Beater! Please read CONTRIBUTING.md first.
The HTTP API, the 11 SDK clients, the MCP tools, the CLI, and the docs are ALL
generated from one artifact: sdks/openapi/beater-api.json.
-->

## What does this PR do?

<!-- A short description of the change and why. Link any issue: Closes #123 -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Docs / examples only
- [ ] Refactor / internal
- [ ] CI / tooling

## Contract checklist (required)

- [ ] This PR does **not** change a `/v1` endpoint, request/response type, span
      kind, or attribute — **OR** —
- [ ] It does, and I regenerated everything in the same change:
  - [ ] `cargo xtask regen-spec` (OpenAPI spec + dashboard snapshot)
  - [ ] `scripts/regen-sdks.sh` (all 11 generated clients)
  - [ ] `cargo xtask regen-semconv` (if span kinds / attributes changed)
- [ ] `scripts/check-contract-sync.sh` passes locally (no drift)

## Tests

- [ ] `cargo build --workspace` and `cargo test --workspace` (or the affected
      crates) pass locally
- [ ] `cargo clippy --workspace --all-targets` is clean
- [ ] Added/updated tests for the change

## Notes for reviewers

<!-- Anything reviewers should focus on, follow-ups, or known limitations. -->
