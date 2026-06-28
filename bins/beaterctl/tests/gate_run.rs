use std::process::Command;

#[test]
fn gate_run_exits_nonzero_for_latest_regression() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let fixture = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("gate-run-fixture")
        .arg("--data-dir")
        .arg(tempdir.path())
        .output()?;
    assert!(
        fixture.status.success(),
        "fixture stderr: {}",
        String::from_utf8_lossy(&fixture.stderr)
    );

    let gate_run = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("gate-run")
        .arg("--data-dir")
        .arg(tempdir.path())
        .arg("--tenant-id")
        .arg("demo")
        .arg("--project-id")
        .arg("demo")
        .arg("--gate-id")
        .arg("main")
        .output()?;

    assert!(
        !gate_run.status.success(),
        "gate run should fail on latest regression"
    );
    let stdout = String::from_utf8(gate_run.stdout)?;
    assert!(stdout.contains(r#""passed": false"#));
    assert!(stdout.contains(r#""experiment_run_id": "gate-latest-fail""#));
    Ok(())
}

#[test]
fn gate_run_exits_nonzero_for_explicit_inconclusive() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let fixture = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("gate-run-fixture")
        .arg("--data-dir")
        .arg(tempdir.path())
        .output()?;
    assert!(
        fixture.status.success(),
        "fixture stderr: {}",
        String::from_utf8_lossy(&fixture.stderr)
    );

    let fixture_stdout: serde_json::Value = serde_json::from_slice(&fixture.stdout)?;
    assert_eq!(
        fixture_stdout["inconclusive_experiment_run_id"],
        "gate-explicit-inconclusive"
    );

    let gate_run = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("gate-run")
        .arg("--data-dir")
        .arg(tempdir.path())
        .arg("--tenant-id")
        .arg("demo")
        .arg("--project-id")
        .arg("demo")
        .arg("--gate-id")
        .arg("main")
        .arg("--experiment-run-id")
        .arg("gate-explicit-inconclusive")
        .output()?;

    assert!(
        !gate_run.status.success(),
        "gate run should fail when inconclusive policy rejects inconclusive results"
    );
    let stdout: serde_json::Value = serde_json::from_slice(&gate_run.stdout)?;
    assert_eq!(stdout["passed"], false);
    assert_eq!(stdout["experiment_decision"], "inconclusive");
    assert_eq!(
        stdout["reason"],
        "experiment gate-explicit-inconclusive was inconclusive and gate policy fails inconclusive results"
    );
    Ok(())
}
