use std::process::Command;

#[test]
fn api_unknown_operation_fails_before_network() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("--base-url")
        .arg("http://127.0.0.1:9")
        .arg("api")
        .arg("definitelyNotAnOperation")
        .output()?;

    assert!(!output.status.success(), "unknown operation should fail");
    assert!(
        output.stdout.is_empty(),
        "unknown operation should not print a response: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("definitelyNotAnOperation"),
        "stderr should name the bad operation id: {stderr}"
    );
    assert!(
        stderr.contains("OpenAPI spec"),
        "stderr should explain that operation lookup uses the OpenAPI spec: {stderr}"
    );

    Ok(())
}

#[test]
fn api_get_operation_rejects_body_before_network() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("--base-url")
        .arg("http://127.0.0.1:9")
        .arg("api")
        .arg("traces.list-traces")
        .arg("--param")
        .arg("tenant_id=demo")
        .arg("--body")
        .arg("{}")
        .output()?;

    assert!(!output.status.success(), "GET operation body should fail");
    assert!(
        output.stdout.is_empty(),
        "GET body validation should fail before a response is printed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        stderr.contains("operation `traces.list-traces` is GET"),
        "stderr should name the GET operation: {stderr}"
    );
    assert!(
        stderr.contains("--body is not allowed"),
        "stderr should explain the invalid flag combination: {stderr}"
    );

    Ok(())
}
