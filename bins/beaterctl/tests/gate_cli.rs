use std::process::Command;

#[test]
fn gate_exits_zero_for_passing_comparison() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("gate")
        .arg("--baseline")
        .arg("1.0")
        .arg("--candidate")
        .arg("1.0")
        .arg("--min-sample-size")
        .arg("1")
        .output()?;

    assert!(
        output.status.success(),
        "passing gate stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["decision"], "pass");
    Ok(())
}

#[test]
fn gate_exits_nonzero_for_regression() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("gate")
        .arg("--baseline")
        .arg("1.0")
        .arg("--candidate")
        .arg("0.0")
        .arg("--min-sample-size")
        .arg("1")
        .output()?;

    assert!(!output.status.success(), "regression gate should fail CI");
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["decision"], "fail_regression");
    Ok(())
}
