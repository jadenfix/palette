# MCP stdio Transport — Design (§21)

> **Status:** Design-only. streamable-HTTP `/mcp` is **[built]**;
> stdio is the **one [planned] transport gap** called out in §3.3 and §21.
>
> Relevant architecture sections: §3.3 (deployment table), §21 (MCP
> deployability), §22.3 (acceptance criteria row — `beaterd mcp --stdio
> [planned]`), §22.4 (DoD traceability).

---

## 1. Why stdio

Local MCP clients — Claude Code, Cursor, Codex, and any editor that follows the
[MCP specification](https://spec.modelcontextprotocol.io/) — launch tools as
child processes and communicate over stdin/stdout using newline-delimited JSON-RPC
2.0. No HTTP server, no port, no TLS, no OAuth dance is required for this path:
the user runs `beaterd mcp --stdio` (or an alias), the client forks it, and the
MCP handshake happens in-process.

The streamable-HTTP transport (served at `POST /mcp` by `beaterd`) handles the
hosted, multi-tenant, OAuth-secured path for ChatGPT connectors and remote IDEs.
stdio is the complementary local path. Together they satisfy §21's "same tool set,
two transports" requirement.

---

## 2. Protocol contract: newline-delimited JSON-RPC 2.0

### 2.1 Framing

```
stdin  → server: one complete JSON object per line, UTF-8, `\n`-terminated
stdout ← server: one complete JSON object per line, UTF-8, `\n`-terminated
stderr ← server: human-readable log lines (NOT JSON-RPC — stdout is protocol-only)
```

Each line is an independent JSON-RPC 2.0 envelope:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}
```

The server MUST NOT write partial lines, MUST flush stdout after each response
line, and MUST NOT interleave output from concurrent requests on a single line.
Newlines inside JSON values are escaped (`\n`); a bare ASCII LF is always a
record separator.

### 2.2 Message types (per JSON-RPC 2.0)

| Type | Has `id` | Has `method` | Direction |
|------|-----------|--------------|-----------|
| Request | yes | yes | client → server |
| Response | yes | no | server → client |
| Notification | no | yes | either direction |
| Error response | yes | no | server → client |

The server ignores unknown notifications gracefully (no response, no error). It
MUST respond to every request (even shutdown) with either a result or an error.

### 2.3 Lifecycle

```
client                             server
  │── initialize ─────────────────►│  (request, id=1)
  │◄─ result (capabilities) ───────│  (response, id=1)
  │── notifications/initialized ──►│  (notification — no id, no response)
  │
  │   [normal tool traffic]
  │── tools/list ────────────────►│
  │◄─ result ───────────────────── │
  │── tools/call ───────────────►│
  │◄─ result ───────────────────── │
  │
  │── shutdown ────────────────────►│  (request, id=N)
  │◄─ result({}) ──────────────────│  (response, id=N)
  │── [close stdin / EOF] ─────────►│  server exits 0
```

**EOF handling:** when stdin reaches EOF the server MUST flush any buffered
responses, emit no further output, and exit with code `0`. This is the primary
signal from MCP clients that trigger a clean teardown (e.g. Claude Code closing
the tool).

**`shutdown` request:** explicit shutdown request; the server responds `{}`, then
waits for EOF (or exits immediately if stdin is already closed). A server MUST
NOT exit on receiving `shutdown` alone — it must wait for EOF or a timeout.

### 2.4 `initialize` handshake (golden shape)

See `crates/beater-mcp/tests/fixtures/stdio_initialize.json` for the canonical
golden fixture (request + response pair).

Request:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "clientInfo": { "name": "claude-code", "version": "1.0.0" },
    "capabilities": {}
  }
}
```

Response:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-06-18",
    "serverInfo": { "name": "beater-mcp", "version": "<semver>" },
    "capabilities": { "tools": { "listChanged": false } }
  }
}
```

The `protocolVersion` is negotiated exactly as the HTTP transport does it today
(see `SUPPORTED_PROTOCOL_VERSIONS` in `crates/beater-mcp/src/lib.rs`): the server
echoes the client's requested version if supported, otherwise falls back to the
latest.

---

## 3. Reuse of existing dispatch logic

The existing `dispatch_rpc` function in `crates/beater-mcp/src/lib.rs` already:

- handles the full JSON-RPC 2.0 envelope (parse, route, respond)
- implements `initialize`, `ping`, `tools/list`, `tools/call`, `shutdown`
- resolves tools from the live OpenAPI spec at startup (zero drift)
- forwards auth headers verbatim to the real handlers

The stdio transport is a **thin I/O loop** that wraps `dispatch_rpc`:

```
loop {
    read one line from stdin
    parse JSON
    call dispatch_rpc(&state, &headers, &request).await
    if Some(response) { write line to stdout; flush }
}
```

No business logic, no tool catalog changes, no new auth paths. The `ApiState`
used in stdio mode is the same struct built by `beaterd` for the HTTP server.

### 3.1 What does NOT change

- `crates/beater-mcp/src/lib.rs` — no changes to the dispatch core; the stdio
  runner consumes the public API.
- The tool catalog (`build_tools`, `tools()`) — reused unchanged.
- Auth: stdio mode passes the user's Beater API key (read from env var
  `BEATER_API_KEY` or a config file) as the `x-beater-api-key` header on every
  synthesized inner request, exactly as a direct HTTP call would.
- The OpenAPI spec / generated SDKs / contract — not touched.

---

## 4. Logging-to-stderr contract

**stdout is reserved for the JSON-RPC protocol.** Nothing else may appear on
stdout. All diagnostics go to stderr:

```
[beater-mcp] listening on stdio (protocol 2025-06-18)
[beater-mcp] tool catalog: 42 operations loaded
[beater-mcp] tools/call traces_list -> 200 OK (12 ms)
```

Structured log output (e.g. `tracing_subscriber` JSON format) is routed to
stderr. The MCP client ignores stderr; it is surfaced in the terminal / IDE log
panel for the developer.

---

## 5. `beaterd` / CLI entrypoint shape (design only)

This section describes the intended command-line surface. **No code is wired yet.**

### 5.1 `beaterd mcp --stdio`

```
beaterd mcp --stdio [--api-url <url>] [--api-key <key>]
```

- `--api-url`: base URL of a running Beater server (default: `http://localhost:8080`).
  The stdio runner is a **client-side proxy**: it opens an `ApiState` backed by
  the remote server, so tools dispatch as HTTP calls to that server.
  Alternatively, when run in-process (e.g. inside `beaterd serve`), it uses the
  in-process `ApiState` directly with no HTTP hop.
- `--api-key`: Beater API key forwarded on every inner request. Can also be set
  via `BEATER_API_KEY` env var.

### 5.2 MCP client config (Claude Code example)

```json
{
  "mcpServers": {
    "beater": {
      "command": "beaterd",
      "args": ["mcp", "--stdio"],
      "env": { "BEATER_API_KEY": "<your-key>" }
    }
  }
}
```

Cursor `.cursor/mcp.json` and Codex `codex.json` follow the same shape.

### 5.3 In-process vs proxy modes

| Mode | How it runs | When to use |
|------|-------------|-------------|
| **In-process** | `beaterd` embeds both the HTTP server and the stdio loop in one binary; the MCP client forks `beaterd mcp --stdio` and the inner dispatch hits handlers in the same process (zero network hops) | local OSS self-host where latency matters |
| **Client proxy** | stdio loop makes real HTTP calls to a remote `beaterd` instance | developer using the hosted tier; `--api-url` points at `api.beater.dev` |

For the initial implementation, the client-proxy mode is sufficient and simpler:
the stdio binary wraps `reqwest` calls rather than embedding `ApiState` directly.

---

## 6. Phased implementation plan

### Phase 1 — stdio I/O loop (the concrete gap, §21)

**Files to create / modify:**

- `crates/beater-mcp/src/stdio.rs` — new file: the async stdin read loop,
  stdout write-and-flush, stderr logging, EOF/shutdown handling.
- `bins/beaterd/src/main.rs` — add `beaterd mcp --stdio` subcommand that
  initializes `ApiState` (proxy mode) and calls `beater_mcp::stdio::run()`.

**Acceptance criteria (from §22.3):**

- `beaterd mcp --stdio` is launchable by a local MCP client.
- `tools/list` over stdio returns the full tool set (same as HTTP `/mcp`).
- `tools/call` over stdio and the equivalent `POST /mcp` return identical JSON
  for the same operation + arguments.
- EOF on stdin causes a clean exit (code 0, no panic).
- All log output goes to stderr; stdout contains only JSON-RPC lines.

**Test gate:** `cargo test -p beater-mcp -- stdio` — the currently-ignored
fixture tests in `crates/beater-mcp/tests/stdio_fixture.rs` become un-ignored
and green.

### Phase 2 — in-process mode

- Wire `ApiState` in-process (no HTTP hop) when `--api-url` is not given and
  `beaterd` embeds the server.
- Add a `beater_mcp::stdio::run_with_state(state: ApiState)` variant.

### Phase 3 — polish & CI gate

- Add `beaterd mcp --stdio` to `sdk-contract` CI (§22.3 row).
- Add to quickstart guide and `docs/local-dev.md`.
- Register `tools/list` count check in the DoD table (§22.3).

---

## 7. Security notes

- **stdio is a local transport.** The process inherits the user's OS permissions.
  There is no network exposure; no TLS, no ports.
- **API key from env only.** Never log the key. Never write it to stdout.
- **No repo writes by default** (§21.6 bounded-autonomy policy). The MCP tools
  are read/query tools; `apply_change` requires explicit opt-in.
- The stdio server process SHOULD be the same binary as `beaterd` (not a
  separate download) to keep the attack surface minimal.

---

## 8. References

- MCP specification: <https://spec.modelcontextprotocol.io/>
- JSON-RPC 2.0: <https://www.jsonrpc.org/specification>
- `crates/beater-mcp/src/lib.rs` — current HTTP MCP implementation (dispatch core)
- `crates/beater-mcp/tests/mcp.rs` — existing HTTP integration tests
- `crates/beater-mcp/tests/stdio_fixture.rs` — ignored protocol fixture (this doc's companion)
- `crates/beater-mcp/tests/fixtures/stdio_initialize.json` — golden initialize handshake
- §3.3 deployment table (ARCHITECTURE.md:195)
- §21 MCP deployability / stdio gap (ARCHITECTURE.md:2431)
- §22.3 acceptance criteria row (ARCHITECTURE.md:3338)
- §22.4 DoD traceability (ARCHITECTURE.md:3342)
