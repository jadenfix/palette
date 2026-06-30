use std::process::Command;

/// The live `rsi-round` command performs REAL Anthropic calls (proposer rewrite +
/// per-case baseline/candidate evaluation) and is BYOK: it requires a real
/// `ANTHROPIC_API_KEY`. With the key UNSET, it must fail cleanly (non-zero exit,
/// no panic) with the BYOK guidance pointing at the no-network fixture — proving
/// the key-gating WITHOUT making any network call.
#[test]
fn rsi_round_requires_anthropic_key() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("rsi-round")
        // Unset the key in the child env so no network call is attempted.
        .env_remove("ANTHROPIC_API_KEY")
        .output()?;

    assert!(
        !output.status.success(),
        "rsi-round must exit non-zero when ANTHROPIC_API_KEY is unset; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ANTHROPIC_API_KEY"),
        "expected the BYOK error to mention ANTHROPIC_API_KEY, got: {stderr}"
    );
    assert!(
        stderr.contains("rsi-round-fixture"),
        "expected the BYOK error to point at the no-network fixture, got: {stderr}"
    );
    Ok(())
}
