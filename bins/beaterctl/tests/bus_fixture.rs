use std::process::Command;

#[test]
fn bus_fixture_retries_dlqs_and_replays_poison_work() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("bus-fixture")
        .arg("--data-dir")
        .arg(tempdir.path())
        .output()?;

    assert!(
        output.status.success(),
        "fixture failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains(r#""dlq_len_before_replay": 1"#));
    assert!(stdout.contains(r#""dlq_len": 0"#));
    assert!(stdout.contains(r#""accepted": true"#));
    assert!(stdout.contains(r#""replayed_attempts": 0"#));
    Ok(())
}
