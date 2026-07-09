use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn run_beaterd_mcp_stdio(request: Value) -> anyhow::Result<Value> {
    let tempdir = tempfile::tempdir()?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_beaterd"))
        .arg("--data-dir")
        .arg(tempdir.path())
        .arg("--bus-backend")
        .arg("memory")
        .arg("--trace-write-drain-interval-ms")
        .arg("0")
        .arg("--trace-ingested-drain-interval-ms")
        .arg("0")
        .arg("mcp")
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("missing beaterd stdin"))?;
        writeln!(stdin, "{request}")?;
    }
    drop(child.stdin.take());

    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "beaterd mcp --stdio failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let mut lines = stdout.lines();
    let first = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("beaterd wrote no stdout"))?;
    assert!(
        lines.next().is_none(),
        "stdio smoke should emit exactly one JSON-RPC line, got:\n{stdout}"
    );

    serde_json::from_str(first).map_err(Into::into)
}

#[test]
fn beaterd_mcp_stdio_lists_tools() -> anyhow::Result<()> {
    let rpc = run_beaterd_mcp_stdio(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    }))?;
    assert_eq!(rpc["jsonrpc"], "2.0");
    assert_eq!(rpc["id"], 1);
    let tools = rpc["result"]["tools"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("tools/list result missing tools array: {rpc}"))?;
    assert_eq!(tools.len(), beater_mcp::tool_names().len() + 1);
    assert!(tools.iter().any(|tool| tool["name"] == "help"));

    Ok(())
}

#[test]
fn beaterd_mcp_stdio_calls_help_tool() -> anyhow::Result<()> {
    let rpc = run_beaterd_mcp_stdio(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "help",
            "arguments": { "tool": "traces.list-traces" }
        }
    }))?;

    assert_eq!(rpc["jsonrpc"], "2.0");
    assert_eq!(rpc["id"], 1);
    let result = &rpc["result"];
    assert_eq!(result["isError"], false, "help tool must not error: {rpc}");
    assert!(
        result["content"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "help tool should include text content: {rpc}"
    );

    let tool = &result["structuredContent"]["tool"];
    assert_eq!(tool["name"], "traces.list-traces");
    assert_eq!(tool["method"], "GET");
    assert_eq!(tool["path"], "/v1/traces/{tenant_id}");
    assert!(
        tool["inputSchema"].is_object(),
        "help should describe traces.list-traces input schema: {rpc}"
    );
    assert!(
        tool["outputSchema"].is_object(),
        "help should describe traces.list-traces output schema: {rpc}"
    );

    Ok(())
}
