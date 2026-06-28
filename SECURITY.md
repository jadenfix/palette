# Security Policy

Beater handles agent traces that can contain prompts, completions, tool I/O, and
PII. We take vulnerabilities — in the platform, the SDKs, the MCP server, and the
hosted edition — seriously.

## Supported versions

Beater is **pre-1.0**. Security fixes land on `main` and the latest tagged
release. There is no long-term-support branch yet; once 1.0 ships this section
will pin a support window.

## Reporting a vulnerability (coordinated disclosure)

**Do not open a public GitHub issue for a security problem.**

Report privately, in order of preference:

1. **GitHub Security Advisories** — use the repository's
   *Security → Report a vulnerability* (private advisory) workflow.
2. **Message a project admin privately** — reach out to a maintainer/admin via
   the contact listed in `GOVERNANCE.md`. PGP key on request.

Please include:

- the affected component (API, a specific SDK, MCP, CLI, dashboard, hosted),
- the version / commit SHA,
- a minimal reproduction or proof-of-concept,
- the impact you believe it has (data exposure, tenant crossover, auth bypass,
  RCE in the WASI sandbox, judge-credential leakage, etc.).

## Our commitments

- **Acknowledge** your report within **3 business days**.
- **Triage and assign a severity** (CVSS-style) within **7 business days**.
- Keep you updated on remediation progress and **credit you** in the advisory
  unless you prefer to remain anonymous.
- Target **90 days** to a fix and public disclosure; critical issues are
  expedited. We coordinate the disclosure date with you.

## Scope

In scope: the Rust workspace (`crates/*`, `bins/*`), the 7 generated SDK clients
and the native Rust SDK (`sdks/*`), the MCP server (`beater-mcp`), the CLI
(`beaterctl`), the dashboard (`web/dashboard`), the OpenAPI/semconv contract
artifacts, and the hosted edition.

Particularly sensitive areas — please prioritize these in any review:

- **Tenant isolation** — cross-tenant trace/dataset/eval reads or writes (§14).
- **PII unmask & audit** — bypassing the unmask RBAC scope or its audit trail
  (§14).
- **BYOK / provider secrets** — leakage of encrypted-at-rest judge/model
  credentials (§14, `beater-secrets`/`beater-security`).
- **WASI evaluator sandbox** — any escape, host-import access, or network egress
  from a user-supplied deterministic scorer (§10.1).
- **OAuth 2.1 / MCP auth** — token, PKCE, or scope handling on `/oauth/*` and
  `/mcp` (§3.2, §21).
- **Webhook signing** — forgeable HMAC alert/webhook payloads (§14).

Out of scope: volumetric DoS against a self-hosted instance you control, issues
that require a privileged local account on the host, and findings in
third-party dependencies that are already publicly tracked (report those
upstream, but tell us so we can pin/patch).

## Self-host hardening notes

- `beaterd` defaults to `--auth-mode required`. Use `--auth-mode local` only for
  explicit local development; it prints a startup warning because mutating and
  sensitive routes are anonymous in that mode (§20.7 #5.4).
- Set `BEATER_PROVIDER_SECRET_KEY` (base64 32-byte key) so provider secrets are
  encrypted with a key you control rather than an on-disk generated key.
- Keep the dashboard and OTLP ports off the public internet unless fronted by
  your own auth/proxy.
