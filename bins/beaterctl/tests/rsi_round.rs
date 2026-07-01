use std::process::Command;

/// The `rsi-round-fixture` command must drive `run_optimization_round`
/// end-to-end deterministically (no network / no LLM key) and accept the seeded
/// generalizing candidate: a `pass` held-out gate decision, no overfit flag, and
/// a populated `accepted_candidate`.
#[test]
fn rsi_round_fixture_accepts_the_generalizing_candidate() -> anyhow::Result<()> {
    let fixture = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("rsi-round-fixture")
        .output()?;
    assert!(
        fixture.status.success(),
        "fixture stderr: {}",
        String::from_utf8_lossy(&fixture.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&fixture.stdout)?;

    // An accepted candidate is present and was proposed by the LlmRewrite path.
    let accepted = &report["accepted_candidate"];
    assert!(
        accepted.is_object(),
        "expected an accepted candidate, got {accepted:?}"
    );
    assert_eq!(accepted["proposed_by"], "llm_rewrite");
    assert_eq!(accepted["kind"], "system_prompt");

    // Exactly one candidate was evaluated, it cleared the held-out gate, and the
    // anti-overfit guardrail did not fire.
    let evaluated = report["evaluated"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("evaluated must be an array"))?;
    assert_eq!(evaluated.len(), 1);
    let evaluation = &evaluated[0];
    assert_eq!(evaluation["gate_decision"], "pass");
    assert_eq!(evaluation["overfit_flag"], false);
    assert_eq!(evaluation["accepted"], true);
    Ok(())
}
