# Security Review — Beater (high-recall)

**Date:** 2026-06-24
**Reviewer:** automated multi-lens audit (5 parallel auditors: auth, capture-leakage, injection/storage, CI/CD, browser/SSRF)
**Scope:** full workspace, with emphasis on the browser-agent observability surface that is new to `main`.
**Bias:** high recall — latent/uncertain issues are included and labelled with confidence + reachability.

> Note: the local branch is identical to `origin/main`; these findings describe code already on `main`, not an isolated diff. The only open PR (#8, CI consolidation) is unrelated to these findings and is **red** (see end).

---

## Critical / High

### H1 — Redaction never engages on the capture path (secrets persisted in cleartext)
**Confidence: high.** Read-path redaction (`crates/beater-api/src/lib.rs:3382-3455`, `is_sensitive_redaction`) only fires for `RedactionClass::Sensitive|Secret`. **Every** writer in the capture/ingest path hardcodes `RedactionClass::Internal`, which is treated as safe-to-show:
- `crates/beater-otlp/src/lib.rs:121,355`
- `crates/beater-ingest/src/lib.rs:116,1416,1665,1761`
- `crates/beater-browser-capture/src/lib.rs:150,157,174,234,238,259`

Net effect: console logs, full DOM, raw LLM prompts/outputs, and network URLs are stored and returned to any `TraceRead` caller unredacted. The `PiiUnmask`-scope + audit-trail control is effectively **dead code** for browser/OTLP traces. **Fix:** treat `Internal` as "unclassified → scrub conservatively", or add a mandatory ingest-time content classifier that promotes `RedactionClass` to `Sensitive`.

### H2 — OTLP ingest copies all inbound attributes (allow-all default)
**Confidence: high.** `crates/beater-otlp/src/lib.rs:295-297` copies every attribute; `IngestPolicy::default` (`crates/beater-ingest/src/lib.rs:1081-1094`) has `allowed_attributes: None` (allow-all) and an empty `denied_attributes`. Standard OTel keys carrying secrets — `http.request.header.authorization`, `http.request.header.cookie`, `url.full` (query secrets), `gen_ai.prompt` — are retained verbatim. **Fix:** ship a default deny-list of sensitive OTel keys + a value scrubber (not just key-name drop).

### H3 — Network URLs / console / DOM / prompts captured with no scrubbing or size cap
**Confidence: high.** `NetworkRequest.url` keeps full query strings incl. `?access_token=…` (`crates/beater-browser/src/lib.rs:137-150`); console text verbatim (`:128-133`); full before/after DOM and the raw `decision.prompt` are written into `StepTriple` artifacts and the replay cassette (`crates/beater-browser-capture/src/lib.rs:163-260`). Per-value length is unbounded (only attribute *count* is capped at 128). The external SDKs emit the same unscrubbed `browser.url`/`browser.reasoning`/`browser.selector` (`sdks/python-browser-use/beater_browser_use/instrumentation.py:407-453`, `sdks/ts-stagehand/src/tracer.ts:96-188`). **Fix:** strip sensitive query params, scrub secret patterns in console/prompt/DOM, truncate, and class authenticated-page artifacts `Sensitive`.

### H4 — Default auth mode is `Local` = fully disabled
**Confidence: high.** `bins/beaterd/src/main.rs:70` defaults `--auth-mode` to `Local`; `:277-279` only installs `require_auth` when `Required`. With `!auth_required()` every `authorize_*` helper short-circuits to anonymous (`crates/beater-api/src/lib.rs:3020,3041,3065,3093`), so `createApiKey`, `createProviderSecret`, `listProviderSecrets`, `getTrace?unmask=true`, audit, usage, and all tenant reads/writes serve with **no credentials and no tenant scoping**. Prod is saved only by docs telling operators to pass `--auth-mode required`. **Fix:** make the insecure mode opt-in (`--auth-mode insecure-local`) and emit a startup `warn!`, or default to `Required`.

### H5 — Unbounded span materialization → memory-exhaustion DoS
**Confidence: high.** `crates/beater-store/src/lib.rs:101-131` queries spans with `PageRequest { limit: u32::MAX }`; the SQLite query (`crates/beater-store-sql/src/lib.rs:829-882`) has **no SQL `LIMIT`** and deserializes every matching `span_json` into memory, then clones the vector. Backs `listTraces` (`crates/beater-api/src/lib.rs:1247`); user `page.limit` is applied only *after* full materialization. **Fix:** push `LIMIT`/pagination into SQL; never query with `u32::MAX`.

### H6 — DataFusion archive query built by string concatenation
**Confidence: medium.** `crates/beater-archive/src/lib.rs:342-379` (`build_query_sql`) assembles `tenant_id = '{}'`-style predicates with only `sql_literal` (`:595`, doubles `'`) as defense — no bound params. Values are attacker-influenced (native-ingested IDs that bypass validation, re-queried via `queryArchiveSpans`, `crates/beater-api/src/lib.rs:1551-1562`). Any sqlparser quoting edge case → WHERE-clause injection. **Fix:** use DataFusion's DataFrame/`Expr` filter API (`col(..).eq(lit(..))`); stop interpolating into SQL text.

### H7 — Tantivy parses raw user query → operator injection / DoS
**Confidence: high.** `crates/beater-search/src/lib.rs:244,285,295` pass `?q=`/`?model=`/`?tool=` straight to `parser.parse_query`. Callers can inject query-DSL operators (wildcard/fuzzy/range) → expensive-query DoS over large indexes. Tenant isolation itself holds (separate `Must` TermQuery). **Fix:** escape DSL metacharacters or use a restricted/tokenized phrase query; cap query cost.

### H8 — flyctl action pinned to a mutable `@master` ref (supply-chain)
**Confidence: high.** `.github/workflows/deploy-backend.yml:50` uses `superfly/flyctl-actions/setup-flyctl@master` in a job holding `FLY_API_TOKEN` (`:36`). A compromise/force-push of that branch runs attacker code with the production Fly deploy credential. **Fix:** pin to a commit SHA (Dependabot to bump).

---

## Medium

- **M1 — ID newtypes deserialize with zero validation (root cause).** `crates/beater-core/src/lib.rs:47-103` (`id_type!`, `#[serde(transparent)]`) accepts quotes/slashes/`..`/control chars from request bodies (e.g. `ingestNative`). Enables H6 and the path-traversal below. **Fix:** add a validating `Deserialize` (reject quotes, path separators, control chars).
- **M2 — Path traversal in FS artifact store.** `crates/beater-store-obj/src/lib.rs:24-35` blocks `..` but not absolute paths — `artifact:///etc/passwd` reads outside root; `put_bytes` (`:52-62`) doesn't sanitize tenant/project segments. **Fix:** reject absolute paths + canonicalize-and-check inside root; `safe_segment` the tenant/project.
- **M3 — Browser `goto` has no scheme/host allowlist (latent SSRF + local-file read).** All three drivers forward URLs verbatim (`beater-browser-cdp:253`, `-webdriver:156`, `-playwright:278`/`runner.js:203`). Not wired to any API/MCP handler **today**, so latent — but the moment a tenant supplies `start_url`, it's full SSRF (IMDS, loopback, `file://`). **Fix:** shared URL guard in `beater-browser` before navigation.
- **M4 — Webhook `endpoint_url` SSRF (latent).** `crates/beater-alerts/src/lib.rs:103,215` accepts arbitrary tenant URL; HMAC-signs the payload but doesn't restrict the target. No delivery worker exists yet, so no live request — validate before one is added. **Fix:** require `https`, block private/loopback/link-local, re-resolve at send.
- **M5 — `fly.toml` ships the `tools` build-target (beaterctl) into prod.** `fly.toml:17`, `Dockerfile:52-55`. In-container compromise / `fly ssh` → forge Admin API keys directly against `/data`, bypassing HTTP auth. **Fix:** default prod to `runtime`; bootstrap the first key via a one-off job.
- **M6 — Bogus `actions/checkout@v7` pins.** `dashboard-ui.yml`, `browser-e2e.yml`, `container-images.yml`, `gate1/gate2-*.yml` — checkout's latest major is v4; v7 doesn't exist. Inconsistent/unverified pins. **Fix:** standardize on `@v4` (or SHA).
- **M7 — Third-party actions on mutable major tags.** `docker/*@v4/@v7`, `browser-actions/setup-chrome@v1`, `dtolnay/rust-toolchain@stable` — several run with `packages: write` + GHCR login. **Fix:** pin to SHAs.
- **M8 — Plaintext provider-secret store is `pub`-exported.** `crates/beater-secrets/src/lib.rs:166-274` stores `secret_value` unencrypted; only used in tests today but reachable by integrators. **Fix:** gate behind `#[cfg(test)]`/`test-util`.

---

## Low / Info

- **L1 — `dashboard` standing credential is admin+pii-unmask.** `docs/hosting.md:50-57,73` tells operators to paste an `admin,trace-read,trace-write,pii-unmask` key into Vercel. **Fix:** recommend a least-privilege read key for the web tier.
- **L2 — `release.yml` grants workflow-wide `contents: write`.** `:16-17` — only `release-notes` needs it. **Fix:** scope per-job.
- **L3 — `beater-sandbox` has no memory-growth ceiling.** `crates/beater-sandbox/src/lib.rs:88-95` — fuel-metered but no `wasmtime::StoreLimits` max-memory → guest `memory.grow` DoS. **Fix:** attach `Store::limiter` with max memory/tables/instances.
- **L4 — DTOs derive `Debug` over plaintext secrets.** `CreateProviderSecretHttpRequest` (`crates/beater-api/src/lib.rs:2858`) and `ApiKeyCreatedResponse` (`:2872`) — leak if ever debug-logged. **Fix:** custom redacting `Debug`.
- **L5 — CDP `Scroll` interpolates coords into evaluated JS.** `crates/beater-browser-cdp/src/lib.rs:281` — safe today (`i64`), unsafe pattern. **Fix:** bound args via `evaluate_function`.
- **L6 — `/health` unauthenticated.** `fly.toml:38,51` — intended; confirm the handler returns only `{"ok":true}`, no version/build detail.

---

## Verified clean (recall completeness)
- API keys: Argon2 verify (constant-time); webhook sigs via `subtle::ct_eq`; no `==` on secret material.
- Provider secrets at rest: ChaCha20-Poly1305, per-row `OsRng` nonce, AAD tenant/project binding; key file `0o600`; no-plaintext-in-sqlite test.
- Tenant IDOR on reads: `verify_api_key` + `ensure_trace_auth_scope` enforce project/env scoping on every read/archive/span route. MCP inherits the same `authorize()` path (no weaker surface).
- No `${{ github.event.* }}` → `run:` script injection; no `pull_request_target`; deploy-skip guard pattern is correct; Dockerfiles run non-root; no secrets echoed.
- Replay re-issues no outbound requests; judge `endpoint_url` is server-fixed (no user `base_url` exfil vector); OTLP IDs are hex-only (no injection on that path); runtime SQLite uses bound `params!`.

---

## Top 5 to fix first
1. **H1/H2/H3** — engage redaction by default on capture/OTLP/native (the single highest-leverage fix; the whole new browser-observability surface leaks secrets today).
2. **H4** — make auth-disabled opt-in / loud.
3. **H5** — bound `listTraces` with a real SQL `LIMIT`.
4. **H8 + M6/M7** — pin CI actions to SHAs (fix the bogus `checkout@v7`).
5. **M1** — validating `Deserialize` on `id_type!` (hardens H6 + M2 at once).
</content>
</invoke>
