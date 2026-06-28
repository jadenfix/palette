# Beater Copilot Instructions

Use `AGENTS.md` as the canonical compact repo context. It explains what Beater is,
the Rust workspace shape, contract artifacts, dashboard, migrations, CI gates,
common commands, and guardrails.

Hard rules:

- Do not casually edit `ARCHITECTURE.md`; it is the build-ready plan.
- Do not hand-edit generated SDKs/specs/dashboard API types/semconv files. Change
  the Rust source of truth, regenerate, and run `scripts/check-contract-sync.sh`.
- Preserve the OSS no-cloud posture, tenant/project/env isolation, raw-envelope
  storage, redaction, and audited PII access constraints.
- Keep changes PR-scoped and avoid broad refactors.
