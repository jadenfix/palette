# Security Regression Fixture Catalog

Maps every existing auth/RBAC/API-key/tenant-isolation/secret-handling test
to a threat category.  Gaps — threat categories with no current test — are
marked **GAP** and linked to the source security review.

**Reference:** [`docs/security-review-2026-06-24.md`](security-review-2026-06-24.md)
(automated multi-lens audit, 2026-06-24).

---

## Architecture contract map

This catalog tracks regression coverage for security invariants already called
out in `ARCHITECTURE.md`. It does not change any Covered/GAP status below.

| Catalog area | Architecture contract | Notes |
|---|---|---|
| AuthZ / RBAC scope escalation | §20.7 #5.2 enforced RBAC; A20 tenant isolation | A non-owner must be denied mutating routes; scoped keys are not a substitute for enforced role resolution. |
| Tenant cross-talk / IDOR | §20.7 #5.4 storage-layer tenant isolation; A20 tenant isolation | App-layer scoping is built today; DB-layer RLS is still planned, so API and store tests both matter. |
| API-key leakage / scope and secret storage | §14 security; §20.7 #5.7 tamper-evident audit; §20.7 #5.12 BYOK/key rotation | Key material must stay scoped, rotatable, encrypted, and auditable across API responses and storage. |
| PII redaction | §14 redaction and audited PII access | `pii:unmask` must remain separate from ordinary trace reads, and sensitive-data access must emit audit evidence. |
| Data deletion / crypto-shred | §20.7 #5.5 crypto-shred | Unreadable-after-delete is planned contract work; fixture gaps should not be marked Covered until a deletion path exists. |

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
| Wrong scope → `MissingScope` error | `crates/beater-security/src/lib.rs :: api_keys_are_hashed_scoped_and_rotatable` | `verify_api_key` with `trace:write` key checked for `pii:unmask` returns `Err(MissingScope)` | Covered |
| Scope forgery in payload body overwritten | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | Client-supplied `auth_context` with forged `"admin"` scope is discarded; server-side auth context written to the stored artifact with the actual verified scopes | Covered |
| Trace key blocked from read routes missing scope headers | same test | `GET /v1/traces/{tenant}/{trace}` with a trace-scoped key and no `x-beater-project-id` returns `400 Bad Request` | Covered |
| `pii:unmask` scope gate on unmask param | same test | `GET /v1/traces/{tenant}/{trace}?unmask=true` with a `trace:write+trace:read` key (no `pii:unmask`) returns `403 Forbidden`; same request with a `pii:unmask` key returns `200` with unredacted data | Covered |
| `trace:write` key attempting admin-only endpoints | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | A same-tenant `trace:write`-only key is rejected with `403 Forbidden` by `POST /v1/api-keys/…`, `GET /v1/audit/…`, and `GET /v1/usage/…`. | Covered |

### Tenant cross-talk / IDOR

Can tenant A access or modify tenant B's resources using tenant A's own valid key?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| Cross-tenant key revoke returns 404 | `crates/beater-api/tests/full_stack.rs :: api_key_revoke_is_scoped_to_path_tenant_project_environment` | Admin key scoped to `tenant-a/project-a` attempts to revoke a key belonging to `tenant-b/project-b`; returns `404 Not Found`; victim key is still active | Covered |
| Environment mismatch on archive route → 403 | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | Archive span query with `environment_id=dev` on a key scoped to `prod` returns `403 Forbidden` | Covered |
| Store-layer scoped revoke does not cross tenants | `crates/beater-auth/src/lib.rs :: sqlite_store_scoped_revoke_does_not_revoke_other_scope` | `revoke_key_in_scope(tenant-a, project-a, …)` on a key owned by `tenant-b/project-b` returns `None`; victim key's `active` and `rotated_at` fields unchanged | Covered |
| Cross-tenant trace read (IDOR on read path) | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | A valid `trace:read` key for `other-tenant/project/prod` receives `403 Forbidden` when it attempts `GET /v1/traces/tenant/trace` with matching project/environment scope headers. | Covered |

### API-key leakage / scope

Is plaintext secret material ever exposed in API responses or on disk?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| `secret_hash` absent from create response | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | `created_key.get("secret_hash").is_none()` after `POST /v1/api-keys/…` | Covered |
| Provider secret never returned in responses | `crates/beater-api/tests/full_stack.rs :: hosted_judge_api_uses_byok_refs_cache_and_never_returns_secret` | POST /v1/provider-secrets response body does not contain the secret value string; `secret_value` field absent; judge response and ledger response likewise secret-free | Covered |
| Provider secret not stored as plaintext on disk | `crates/beater-secrets/src/encrypted.rs :: encrypted_store_round_trips_without_plaintext_in_sqlite_file` | After writing a secret, raw bytes of the SQLite file and its WAL do not contain the plaintext value | Covered |
| Wrong AEAD key rejected | `crates/beater-secrets/src/encrypted.rs :: encrypted_store_rejects_wrong_key_material` | Opening the store with a key of different bytes returns `Err` on `get_secret` | Covered |
| Key rotation re-wraps ciphertext | `crates/beater-secrets/src/encrypted.rs :: rotating_re_wraps_ciphertext_under_new_active_key` | After rotation, the new key alone can decrypt; the old key can no longer be used in isolation to read the row | Covered |
| API-key read/list routes do not expose stored hashes | `crates/beater-api/tests/route_inventory.rs :: api_key_routes_do_not_expose_read_surfaces` | Current API-key HTTP surface has only create and revoke routes, so there is no list/get response that can expose `secret_hash`; the route inventory test fails if a future API-key read/list route is added without an explicit response review. | Covered |

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
| DataFusion archive SQL injection (H6) | `crates/beater-archive/src/lib.rs :: hostile_id_is_bound_as_literal_not_interpolated_into_sql`; `hostile_tenant_id_with_quotes_matches_no_rows`; `semicolon_injection_in_span_id_does_not_corrupt_query` | Archive filters are built with structured DataFusion `DataFrame::filter(... lit(...))` expressions, not interpolated SQL text; hostile tenant/trace/span values remain opaque literals and return zero rows rather than broadening the query. | Covered |
| Tantivy query DSL injection / DoS (H7) | `crates/beater-search/src/lib.rs :: metacharacter_query_is_tokenized_without_parse_error`; `wildcard_syntax_is_not_executed`; `model_filter_does_not_execute_boolean_syntax`; `oversized_query_returns_error`; `cross_tenant_dsl_*` | Search builds `BooleanQuery`/`TermQuery` clauses from literal analyzer tokens, clamps result limits, rejects overlong query strings, and keeps tenant scope as a mandatory exact field clause that DSL-looking input cannot escape. | Covered |
| OTLP attribute injection | `crates/beater-api/tests/full_stack.rs :: api_ingest_store_eval_gate_and_replay_are_integrated` (implicit) | OTLP attribute keys are checked as plain strings after storage; runtime SQLite uses bound `params!` (noted in security review as verified clean). | Covered (partial) |

### PII redaction

Is sensitive data correctly withheld from callers lacking the `pii:unmask` scope?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| `Sensitive`-class trace redacted for `trace:read` key | `crates/beater-api/tests/full_stack.rs :: strict_auth_enforces_scoped_keys_and_overwrites_ingest_auth_context` | Trace with `RedactionClass::Sensitive` returns `"[redacted]"` for `input.value`/`output.value`/`raw_ref.uri` when retrieved with a `trace:read`-only key; unmasked values returned after `pii:unmask` grant | Covered |
| External source ingest raw payloads are redacted | `crates/beater-api/src/lib.rs :: api_accepts_otlp_http_protobuf_and_reads_canonical_trace`; `crates/beater-api/src/lib.rs :: api_accepts_collector_otlp_json_and_reads_canonical_trace`; `crates/beater-ingest/src/lib.rs :: native_importer_preserves_raw_bytes_as_sensitive`; `crates/beater-api/tests/import_mapping.rs :: import_source_mapping_projects_foreign_trace` | OTLP protobuf, collector OTLP JSON, native-importer, and mapping-importer paths mark raw payloads `Sensitive`; non-`pii:unmask` reads return redacted raw refs/values while stored raw bytes remain durable for authorized access. Direct `/v1/traces/native` remains caller-classified by request contract. | Covered |
| Browser-captured DOM / console / prompt redacted | `crates/beater-browser-capture/src/lib.rs :: sensitive_dom_console_and_prompt_payloads_are_artifacts_not_span_attributes`; `crates/beater-api/src/lib.rs :: redact_trace_view_redacts_sensitive_attribute_values_and_provenance` | Capture stores DOM snapshots, console-bearing step triples, screenshots, and LLM prompt artifacts as `Sensitive`; canonical span JSON carries artifact IDs/metadata without raw DOM, console text, or prompt contents. API redaction then withholds `Sensitive` artifact refs and sensitive span attributes from callers lacking `pii:unmask`. | Covered |

### SSRF / path traversal

Can attacker-controlled data cause the server to fetch or open unintended resources?

| Threat | Test (file :: function) | What it asserts | Status |
|---|---|---|---|
| FS artifact store path traversal (M2) | `crates/beater-store-obj/src/lib.rs :: path_for_uri_rejects_traversal_and_absolute_paths`; `put_bytes_rejects_tenant_id_with_path_separators`; `get_and_delete_bytes_reject_forged_malicious_uris` | `FsArtifactStore` rejects absolute `artifact:///etc/passwd`, `..`, `.`, empty paths, tenant/project IDs with path separators, and forged malicious `ArtifactRef` URIs before reads/deletes can escape the store root. | Covered |
| Browser `goto` SSRF (M3) | `crates/beater-browser/src/url_policy.rs :: block_private_blocks_*`; `crates/beater-browser-cdp/src/lib.rs :: goto_navigation_guard_blocks_ssrf_targets`; same test in Playwright/WebDriver drivers | Live browser drivers default to `UrlPolicy::block_private` and enforce it before `goto` / `act(Goto)` browser I/O; tests cover metadata/link-local, loopback/localhost, alternate IP encodings, and `file://` targets. | Covered |
| Webhook `endpoint_url` SSRF (M4) | `crates/beater-alerts/src/lib.rs :: webhook_endpoint_url_policy_blocks_ssrf_targets`; `alert_engine_rejects_ssrf_webhook_endpoint_before_delivery`; `crates/beater-api/tests/full_stack.rs :: api_ingest_store_eval_gate_and_replay_are_integrated` | Alert webhook endpoints must be public HTTPS targets; loopback, private, link-local/metadata, alternate IPv4 encodings, IPv6 local targets, non-HTTPS schemes, and file URLs are rejected before delivery construction and surface as HTTP `400` through the API. | Covered |

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
| MCP auth path equals HTTP path | `crates/beater-mcp/tests/mcp.rs :: tools_call_forwards_strict_auth_and_scope_headers` | Builds MCP with `ApiState::require_auth`, calls a spec-derived tool through `/mcp`, and verifies missing credentials and invalid bearer tokens surface the underlying HTTP `401`, missing strict-auth scope headers surface `400`, and valid auth reaches the handler. | Covered |
| OAuth session management / PKCE / redirect URI | `crates/beater-oauth-server/src/lib.rs :: auth_register_login_me_logout_flow`; `authorize_rejects_unsafe_registered_redirect_uri_without_redirecting`; `authorize_without_session_redirects_to_login`; `authorize_denies_non_member_of_tenant`; `authorize_rejects_client_with_unsupported_scope`; `authorize_rejects_plain_pkce_method_without_issuing_code`; `token_rejects_wrong_pkce_verifier_and_redirect_uri_mismatch`; `full_authorize_then_token_flow` | OAuth HTTP tests cover secure session cookie issue/clear and authenticated `/auth/me`, login redirects with `return_to`, redirect-URI allowlist enforcement without unsafe redirects, tenant membership checks, unsupported scope denial, `S256`-only authorize requests, token-endpoint PKCE verifier enforcement, redirect-URI binding, and a full authorize-to-token flow. | Covered |

---

## Summary of gaps

No cataloged security fixture gaps remain in this document.
