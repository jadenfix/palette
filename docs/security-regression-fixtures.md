# Security Regression Fixture Catalog

Maps every existing auth/RBAC/API-key/tenant-isolation/secret-handling test
to a threat category.  Gaps — threat categories with no current test — are
marked **GAP** and linked to the source security review.

**Reference:** [`docs/security-review-2026-06-24.md`](security-review-2026-06-24.md)
(automated multi-lens audit, 2026-06-24).

---

## Catalog

### AuthN bypass

Can an attacker reach a protected endpoint without valid credentials?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| No credentials → 401 | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | `POST /v1/traces/native` without `Authorization` header returns `401 Unauthorized` | Covered |
| Revoked key → 403 | same test (latter half) | After revoking the key, the same secret now returns `403 Forbidden` | Covered |
| Default auth mode disabled (H4) | `bins/beaterd/src/main.rs :: auth_default_tests::{auth_mode_defaults_to_required, auth_mode_local_is_explicit_opt_in, auth_mode_required_parses}` | The CLI parser defaults `beaterd` to `AuthModeArg::Required`, while `--auth-mode local` remains an explicit insecure opt-in. | Covered |

### AuthZ / RBAC scope escalation

Can an authenticated caller acquire permissions beyond those on their key?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| Wrong scope → `MissingScope` error | `crates/beater-security/src/lib.rs :: api_keys_are_hashed_scoped_and_rotatable` | `verify_api_key` with `trace_write` key checked for `pii_unmask` returns `Err(MissingScope)` | Covered |
| Scope forgery in payload body overwritten | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | Client-supplied `auth_context` with forged `"admin"` scope is discarded; server-side auth context written to the stored artifact with the actual verified scopes | Covered |
| `trace_write` key blocked from read routes missing scope headers | same test | `GET /v1/traces/{tenant}/{trace}` with a `trace_write`-only key and no `x-beater-project-id` returns `400 Bad Request` | Covered |
| `pii_unmask` scope gate on unmask param | same test | `GET /v1/traces/{tenant}/{trace}?unmask=true` with a `trace_write+trace_read` key (no `pii_unmask`) returns `403 Forbidden`; same request with a `pii_unmask` key returns `200` with unredacted data | Covered |
| `trace_write` key attempting admin-only endpoints | — | No test verifying a `trace_write` key is rejected by `POST /v1/api-keys/…`, `GET /v1/audit/…`, `GET /v1/usage/…` etc. | **GAP** |

### Tenant cross-talk / IDOR

Can tenant A access or modify tenant B's resources using tenant A's own valid key?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| Cross-tenant key revoke returns 404 | `crates/beater-api/tests/full_stack.rs :: api_key_revoke_is_scoped_to_path_tenant_project_environment` | Admin key scoped to `tenant-a/project-a` attempts to revoke a key belonging to `tenant-b/project-b`; returns `404 Not Found`; victim key is still active | Covered |
| Environment mismatch on archive route → 403 | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | Archive span query with `environment_id=dev` on a key scoped to `prod` returns `403 Forbidden` | Covered |
| Store-layer scoped revoke does not cross tenants | `crates/beater-auth/src/lib.rs :: sqlite_store_scoped_revoke_does_not_revoke_other_scope` | `revoke_key_in_scope(tenant-a, project-a, …)` on a key owned by `tenant-b/project-b` returns `None`; victim key's `active` and `rotated_at` fields unchanged | Covered |
| Cross-tenant trace read (IDOR on read path) | — | No test that tenant-a's valid key cannot retrieve tenant-b's trace via `GET /v1/traces/{tenant-b}/{trace}`. The security review notes `verify_api_key` is called on every read route, but no regression test exercises this boundary directly. | **GAP** |

### API-key leakage / scope

Is plaintext secret material ever exposed in API responses or on disk?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| `secret_hash` absent from create response | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | `created_key.get("secret_hash").is_none()` after `POST /v1/api-keys/…` | Covered |
| Provider secret never returned in responses | `crates/beater-api/tests/full_stack.rs :: hosted_judge_api_uses_byok_refs_cache_and_never_returns_secret` | POST /v1/provider-secrets response body does not contain the secret value string; `secret_value` field absent; judge response and ledger response likewise secret-free | Covered |
| Provider secret not stored as plaintext on disk | `crates/beater-secrets/src/encrypted.rs :: encrypted_store_round_trips_without_plaintext_in_sqlite_file` | After writing a secret, raw bytes of the SQLite file and its WAL do not contain the plaintext value | Covered |
| Wrong AEAD key rejected | `crates/beater-secrets/src/encrypted.rs :: encrypted_store_rejects_wrong_key_material` | Opening the store with a key of different bytes returns `Err` on `get_secret` | Covered |
| Key rotation re-wraps ciphertext | `crates/beater-secrets/src/encrypted.rs :: rotating_re_wraps_ciphertext_under_new_active_key` | After rotation, the new key alone can decrypt; the old key can no longer be used in isolation to read the row | Covered |
| `secret_hash` absent from list/get responses | — | The create-response check exists but no test GETs or lists an already-stored key and asserts `secret_hash` is absent from the returned JSON. | **GAP** |

### Replay attacks

Can a legitimately observed request or webhook be replayed to achieve an effect?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| Webhook idempotency key stable per delivery | `crates/beater-security/src/lib.rs :: webhook_idempotency_key_is_stable_per_delivery_and_unique_across_deliveries` | Same delivery-id + body → identical idempotency key; different delivery-id → different key; raw body bytes not present in the key | Covered |
| Webhook signature replay window | `crates/beater-security/src/lib.rs :: webhooks_are_signed_and_replay_protected` | `verify_webhook` returns `Err(WebhookReplayWindow)` when `now` is more than 5 minutes past the timestamp in the header | Covered |
| Webhook wrong signing secret rejected | same test | `verify_webhook` with wrong key bytes returns `Err(WebhookSignatureFailed)` | Covered |

### Injection (SQL / query DSL)

Can attacker-controlled input alter query semantics?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| DataFusion archive SQL injection (H6) | — | `build_query_sql` in `crates/beater-archive` assembles `tenant_id = '{}'`-style predicates by string interpolation; no regression test for quoting edge-cases or DSL injection. | **GAP** |
| Tantivy query DSL injection / DoS (H7) | — | `?q=` / `?model=` / `?tool=` values are passed directly to the Tantivy query parser; no test for operator metacharacters (`*`, `~`, `[`, `TO`) or expensive wildcard queries. | **GAP** |
| OTLP attribute injection | `crates/beater-api/tests/full_stack.rs :: api_ingest_store_eval_gate_and_replay_are_integrated` (implicit) | OTLP attribute keys are checked as plain strings after storage; runtime SQLite uses bound `params!` (noted in security review as verified clean). | Covered (partial) |

### PII redaction

Is sensitive data correctly withheld from callers lacking the `pii_unmask` scope?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| `Sensitive`-class trace redacted for `trace_read` key | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | Trace with `RedactionClass::Sensitive` returns `"[redacted]"` for `input.value`/`output.value`/`raw_ref.uri` when retrieved with a `trace_read`-only key; unmasked values returned after `pii_unmask` grant | Covered |
| OTLP/native ingest bypasses redaction (H1) | — | All writers in the OTLP, native-ingest, and browser-capture paths hardcode `RedactionClass::Internal`, which the read path treats as safe-to-show. No test asserts that an OTLP-ingested trace is redacted for a non-`pii_unmask` caller. | **GAP** |
| Browser-captured DOM / console / prompt redacted | — | No test that browser-capture artifacts (DOM snapshots, console text, LLM prompts) are classified `Sensitive` and withheld from non-`pii_unmask` callers. | **GAP** |

### SSRF / path traversal

Can attacker-controlled data cause the server to fetch or open unintended resources?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| FS artifact store path traversal (M2) | — | `FsArtifactStore` blocks `..` but not absolute paths; no test for `artifact:///etc/passwd` or tenant/project segments with path separators. | **GAP** |
| Browser `goto` SSRF (M3) | — | All three browser drivers forward `start_url` verbatim; no URL allowlist and no test for `file://` or `http://169.254.169.254`. Currently latent (no MCP/API handler wires this), but no guard test exists. | **GAP** |
| Webhook `endpoint_url` SSRF (M4) | — | `beater-alerts` accepts arbitrary tenant-supplied webhook URL with no scheme/host restriction. Delivery worker not yet implemented, so no live SSRF today — but no test blocks a private/loopback target when the worker is wired in. | **GAP** |

### Secret storage at rest (key management)

Is the key material lifecycle correctly enforced?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| Keyring requires active key to be present | `crates/beater-secrets/src/encrypted.rs :: keyring_requires_active_key_to_be_present` | `SecretKeyring::with_keys("missing", …)` returns `Err` when the declared active key id is not in the provided set | Covered |
| API key lifecycle (create / verify / revoke) | `crates/beater-auth/src/lib.rs :: sqlite_store_creates_reads_and_revokes_keys` | Create a key, verify it via Argon2, revoke it, confirm `verify_api_key` now returns `Err(InactiveApiKey)` | Covered |

### MCP / session surface

Do alternate access paths enforce the same auth policy?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| MCP auth path equals HTTP path | — | The security review states MCP tools resolve to the same `authorize()` path; the MCP tests run with auth disabled (`build_state()` does not call `.require_auth()`). No test exercises MCP with auth enabled and verifies a missing/invalid key returns 401. | **GAP** |
| OAuth session management / PKCE / redirect URI | — | `crates/beater-oauth-server/src/lib.rs` has no `#[cfg(test)]` block. No tests for session cookie handling, PKCE code-verifier enforcement, redirect-URI allowlist, or scope grant restrictions. | **GAP** |

---

## Summary of gaps

| # | Threat category | Security review finding |
|---|---|---|
| 1 | `trace_write` key not tested against admin-only routes | — |
| 2 | Cross-tenant trace read (IDOR on read path) | — |
| 3 | `secret_hash` absent from list/get key responses | — |
| 4 | OTLP / browser-capture redaction bypass | H1 |
| 5 | Browser DOM / console / prompt redaction | H1 / H3 |
| 6 | DataFusion archive SQL injection | H6 |
| 7 | Tantivy query DSL injection / DoS | H7 |
| 8 | FS artifact store path traversal | M2 |
| 9 | Browser `goto` SSRF | M3 |
| 10 | Webhook `endpoint_url` SSRF | M4 |
| 11 | MCP surface with auth enabled | — |
| 12 | OAuth session / PKCE / redirect-URI | — |

Findings marked H1, H6, H7 are rated **Critical / High** in the security
review. Recommend addressing items 4, 6, and 7 first (aligned with the
security review's Top 5).
