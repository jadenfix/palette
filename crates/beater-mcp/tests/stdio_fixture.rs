// Test code uses unwrap/expect freely on known-good fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Protocol fixture / golden tests for the MCP stdio transport (§21).
//!
//! ## Status
//!
//! All tests in this file are **`#[ignore]`d** because the stdio transport is
//! not yet implemented. They compile in CI and are skipped — they exist as the
//! machine-readable specification for the future implementation.
//!
//! When the transport is implemented (`crates/beater-mcp/src/stdio.rs` +
//! `beaterd mcp --stdio`), un-ignore the tests and they become the acceptance
//! gate (§22.3 row: "`beaterd mcp --stdio` `[planned]`").
//!
//! ## Design reference
//!
//! See `docs/mcp-stdio.md` for the full design: framing contract, lifecycle,
//! logging-to-stderr rule, entrypoint shape, and phased implementation plan.
//!
//! ## Golden fixtures
//!
//! `tests/fixtures/stdio_initialize.json` — `initialize` request/response pair.
//! `tests/fixtures/stdio_session.json`    — full session: init → tools/list →
//!                                          ping → shutdown → EOF.
//!
//! ## Running the ignored tests
//!
//! ```text
//! cargo test -p beater-mcp -- --ignored stdio
//! ```
//!
//! Default CI run (`cargo test -p beater-mcp`) skips this file entirely.

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Helpers shared by the ignored tests (compiled even when tests are ignored,
// so any type/import errors are caught in CI).
// ---------------------------------------------------------------------------

/// Load a fixture file relative to this test file's crate root.
fn load_fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("failed to parse fixture {name} as JSON: {e}"))
}

/// Minimal JSON-RPC 2.0 shape check: must be an object with `"jsonrpc": "2.0"`.
fn assert_jsonrpc_envelope(value: &Value, context: &str) {
    assert!(value.is_object(), "{context}: must be a JSON object");
    assert_eq!(
        value["jsonrpc"], "2.0",
        "{context}: jsonrpc field must be \"2.0\""
    );
}

/// Assert a JSON-RPC response envelope: has `id` and `result`, no `error`.
fn assert_response(value: &Value, expected_id: i64, context: &str) {
    assert_jsonrpc_envelope(value, context);
    assert_eq!(
        value["id"], expected_id,
        "{context}: id must be {expected_id}"
    );
    assert!(
        value.get("result").is_some(),
        "{context}: must have a result field"
    );
    assert!(
        value.get("error").is_none(),
        "{context}: must not have an error field"
    );
}

// ---------------------------------------------------------------------------
// Fixture validity (always runs — not ignored)
// ---------------------------------------------------------------------------

/// The golden fixtures are valid JSON and have the expected top-level shape.
/// This test is NOT ignored so fixture regressions surface in the default run.
#[test]
fn fixture_files_are_valid_json() {
    let init = load_fixture("stdio_initialize.json");
    assert!(
        init.get("request").is_some(),
        "stdio_initialize.json must have a 'request' key"
    );
    assert!(
        init.get("response").is_some(),
        "stdio_initialize.json must have a 'response' key"
    );

    let session = load_fixture("stdio_session.json");
    let frames = session["frames"]
        .as_array()
        .expect("stdio_session.json must have a 'frames' array");
    assert!(
        !frames.is_empty(),
        "stdio_session.json frames array must not be empty"
    );
    for (i, frame) in frames.iter().enumerate() {
        let dir = frame["direction"].as_str().unwrap_or("");
        assert!(
            dir == "in" || dir == "out",
            "frame {i}: direction must be 'in' or 'out', got {dir:?}"
        );
        // Every "out" frame and every "in" request frame must be a valid JSON-RPC
        // object. Notifications ("in" frames without an id) are also objects.
        assert!(
            frame.get("frame").map(Value::is_object).unwrap_or(false),
            "frame {i}: must have a 'frame' object"
        );
    }

    let tools_list = load_fixture("stdio_tools_list.json");
    assert!(
        tools_list.get("request").is_some(),
        "stdio_tools_list.json must have a 'request' key"
    );
    assert!(
        tools_list.get("response_shape").is_some(),
        "stdio_tools_list.json must have a 'response_shape' key"
    );
}

/// The golden `initialize` fixture has the expected JSON-RPC 2.0 envelope shape.
#[test]
fn fixture_initialize_envelope_shape() {
    let fixture = load_fixture("stdio_initialize.json");

    let req = &fixture["request"];
    assert_jsonrpc_envelope(req, "initialize request");
    assert_eq!(req["method"], "initialize", "request method");
    assert_eq!(req["id"], 1, "request id");
    assert_eq!(
        req["params"]["protocolVersion"], "2025-06-18",
        "request protocol version"
    );

    let resp = &fixture["response"];
    assert_response(resp, 1, "initialize response");
    assert_eq!(
        resp["result"]["protocolVersion"], "2025-06-18",
        "response echoes protocol version"
    );
    assert_eq!(
        resp["result"]["serverInfo"]["name"], "beater-mcp",
        "response server name"
    );
    assert!(
        resp["result"]["capabilities"].is_object(),
        "response capabilities is object"
    );
}

// ---------------------------------------------------------------------------
// Ignored protocol behavior tests (§21 — stdio transport not yet implemented)
// ---------------------------------------------------------------------------

/// The stdio transport MUST respond to `initialize` with a valid JSON-RPC 2.0
/// result that echoes the protocol version and carries server info + capabilities.
///
/// Acceptance criterion (§22.3): `beaterd mcp --stdio` is reachable and
/// completes the MCP handshake.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_initialize_handshake() {
    // When implemented: spawn `beaterd mcp --stdio`, write the fixture request
    // to its stdin, read one line from stdout, assert it matches the fixture
    // response (modulo the dynamic `version` field in serverInfo).
    //
    // For now this documents the expected contract.
    let fixture = load_fixture("stdio_initialize.json");
    let req = &fixture["request"];
    let expected_resp = &fixture["response"];

    // Shape checks on the fixture itself (redundant with the always-on test
    // above, but kept here so the ignore block is self-contained docs).
    assert_eq!(req["method"], "initialize");
    assert_response(expected_resp, 1, "initialize");
    assert_eq!(expected_resp["result"]["protocolVersion"], "2025-06-18");
    assert!(expected_resp["result"]["capabilities"]["tools"].is_object());

    // TODO (Phase 1): replace with a real process-spawn + stdio round-trip:
    //
    //   let mut child = Command::new("beaterd").args(["mcp", "--stdio"]).spawn()?;
    //   write_line(child.stdin.as_mut().unwrap(), req);
    //   let line = read_line(child.stdout.as_mut().unwrap());
    //   let actual: Value = serde_json::from_str(&line)?;
    //   assert_response(&actual, 1, "initialize");
    //   assert_eq!(actual["result"]["protocolVersion"], "2025-06-18");
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// After initialization the client sends `notifications/initialized` (no id).
/// The server MUST NOT respond (notifications receive no reply).
///
/// Acceptance criterion: stdout stays silent for a notification frame.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_notification_receives_no_response() {
    // Verified by: write init + notifications/initialized to stdin, assert
    // stdout only has one line (the initialize response) before the next
    // request.
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// `tools/list` over stdio MUST return the same set of tools as `POST /mcp`
/// with the same `tools/list` body. This is the primary parity assertion.
///
/// Acceptance criterion (§22.3 row): `tools/list` over stdio returns the full
/// tool set.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_tools_list_parity_with_http() {
    // When implemented:
    //   1. Call the HTTP `/mcp` tools/list endpoint to get the reference list.
    //   2. Call `beaterd mcp --stdio` tools/list to get the stdio list.
    //   3. Assert both lists are identical (same names, same count).
    //
    // The tool catalog is built once from the same spec in both transports
    // (crates/beater-mcp/src/lib.rs `tools()`), so they must match exactly.
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// `tools/list` response shape check against the golden fixture.
/// The `tools` array must be non-empty and each entry must have the required fields.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_tools_list_shape() {
    // Expected shape from the fixture (runtime count is dynamic).
    let _fixture = load_fixture("stdio_tools_list.json");

    // When implemented: run tools/list via stdio and assert:
    //   let tools = response["result"]["tools"].as_array().unwrap();
    //   assert!(!tools.is_empty(), "tools list must not be empty");
    //   for tool in tools {
    //       assert!(tool["name"].is_string(), "tool must have a name");
    //       assert!(tool["description"].is_string(), "tool must have a description");
    //       assert!(tool["inputSchema"].is_object(), "tool must have inputSchema");
    //   }
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// `tools/call` over stdio MUST return the same JSON as the equivalent HTTP
/// `POST /mcp` `tools/call` for the same operation + arguments.
///
/// Test case: `listTraces` (GET /v1/traces, no auth, no params → 200 with empty list).
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_tools_call_parity_with_http() {
    // When implemented: call listTraces via HTTP /mcp and via stdio, assert
    // the `content[0].text` and `_meta.httpStatus` fields are identical.
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// EOF on stdin MUST cause the server to exit cleanly (code 0, no panic,
/// no partial output).
///
/// Acceptance criterion: `beaterd mcp --stdio` exits 0 on stdin close.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_eof_exits_cleanly() {
    // When implemented:
    //   let mut child = Command::new("beaterd").args(["mcp", "--stdio"]).spawn()?;
    //   // Close stdin immediately.
    //   drop(child.stdin.take());
    //   let status = child.wait()?;
    //   assert_eq!(status.code(), Some(0), "exit code must be 0 on EOF");
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// `shutdown` request MUST receive a `{}` result, and the server MUST then
/// wait for EOF before exiting (per JSON-RPC lifecycle; see docs/mcp-stdio.md §2.3).
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_shutdown_then_eof() {
    // When implemented:
    //   write init handshake + shutdown request to stdin
    //   assert stdout has init response + shutdown {} response (exactly 2 lines)
    //   close stdin
    //   wait for exit code 0
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// `ping` MUST return `{}` with the same id.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_ping_returns_empty_object() {
    // Session fixture captures this exchange; when implemented, verify via
    // process spawn.
    let session = load_fixture("stdio_session.json");
    let frames = session["frames"].as_array().unwrap();
    // Find the ping request and its expected response in the fixture.
    let ping_req = frames
        .iter()
        .find(|f| f["direction"] == "in" && f["frame"]["method"] == "ping")
        .expect("session fixture must contain a ping request");
    let ping_resp = frames
        .iter()
        .find(|f| f["direction"] == "out" && f["frame"]["id"] == ping_req["frame"]["id"])
        .expect("session fixture must contain a ping response");
    assert_eq!(
        ping_resp["frame"]["result"],
        json!({}),
        "ping result must be empty object"
    );
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// Log output MUST go to stderr only; stdout MUST contain only JSON-RPC lines.
/// Each stdout line MUST be parseable as a JSON object.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_stdout_is_json_rpc_only() {
    // When implemented: run a full session, capture stdout and stderr
    // separately. Assert:
    //   - every stdout line is parseable as JSON.
    //   - no stdout line starts with "[beater" or any non-JSON prefix.
    //   - stderr is non-empty (at least one log line).
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}

/// Full session golden: init → notifications/initialized → tools/list → ping →
/// shutdown → EOF. Verifies the complete lifecycle in one pass.
#[test]
#[ignore = "stdio transport not yet implemented (§21) — see docs/mcp-stdio.md"]
fn stdio_full_session_golden() {
    // When implemented: replay the stdio_session.json frames through a real
    // beaterd mcp --stdio process and assert the "out" frames match.
    let session = load_fixture("stdio_session.json");
    let frames = session["frames"].as_array().unwrap();

    // Shape-check the fixture (belt and suspenders with the always-on test).
    let out_frames: Vec<&Value> = frames.iter().filter(|f| f["direction"] == "out").collect();
    assert!(
        !out_frames.is_empty(),
        "session fixture must have at least one out frame"
    );
    for frame in &out_frames {
        assert_jsonrpc_envelope(&frame["frame"], "session out frame");
    }
    todo!("implement stdio transport (§21, Phase 1) then remove this todo");
}
