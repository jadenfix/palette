use beater_core::{TenantId, TraceId};
use beater_schema::AgentSpanKind;
use beater_store::TraceStore;
use beater_store_sql::SqliteTraceStore;
use std::process::Command;

/// `rsi-round-fixture --record-trace --data-dir <dir>` must drive the
/// deterministic (no-network) optimization round AND emit one canonical Beater
/// trace into the data-dir store via the same `IngestService::ingest_native`
/// path `smoke` uses. This proves the optimization loop is observable
/// end-to-end with no network: we re-open the store from the data-dir and assert
/// the trace is queryable with the expected span kinds and count.
#[test]
fn rsi_round_fixture_records_a_queryable_trace() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let output = Command::new(env!("CARGO_BIN_EXE_beaterctl"))
        .arg("rsi-round-fixture")
        .arg("--record-trace")
        .arg("--data-dir")
        .arg(tempdir.path())
        .output()?;
    assert!(
        output.status.success(),
        "rsi-round-fixture --record-trace stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    // The round still reports the accepted, generalizing candidate.
    assert_eq!(report["accepted_candidate"]["proposed_by"], "llm_rewrite");
    assert_eq!(report["evaluated"][0]["accepted"], true);

    // The report carries a real trace id + span count from the read-back.
    let trace_id_str = report["trace_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("report must include a string trace_id"))?;
    assert!(
        trace_id_str.starts_with("rsi-round-"),
        "unexpected trace_id: {trace_id_str}"
    );
    assert_eq!(
        report["trace_span_count"], 3,
        "expected root + proposal + one evaluator span"
    );

    // Re-open the SAME data-dir store and query the trace back — exactly the
    // observability claim: the round landed and is queryable with no network.
    let runtime = tokio::runtime::Runtime::new()?;
    let trace = runtime.block_on(async {
        let store = SqliteTraceStore::open(tempdir.path().join("traces.sqlite"))?;
        store
            .get_trace(TenantId::new("demo")?, TraceId::new(trace_id_str)?)
            .await
            .map_err(anyhow::Error::from)
    })?;

    // Root AgentRun + one AgentPlan proposal + >= 1 EvaluatorRun span.
    assert_eq!(trace.spans.len(), 3, "expected exactly three spans");

    let root = trace
        .spans
        .iter()
        .find(|span| span.parent_span_id.is_none())
        .ok_or_else(|| anyhow::anyhow!("trace must have a root span"))?;
    assert_eq!(root.kind, AgentSpanKind::AgentRun);
    assert_eq!(root.name, "rsi optimization round");

    let plan_count = trace
        .spans
        .iter()
        .filter(|span| span.kind == AgentSpanKind::AgentPlan)
        .count();
    assert_eq!(plan_count, 1, "expected one proposal (AgentPlan) span");

    let evaluator_count = trace
        .spans
        .iter()
        .filter(|span| span.kind == AgentSpanKind::EvaluatorRun)
        .count();
    assert!(
        evaluator_count >= 1,
        "expected at least one EvaluatorRun span, got {evaluator_count}"
    );

    // Every child links to the root (a real, well-formed canonical trace tree).
    for span in &trace.spans {
        if span.parent_span_id.is_some() {
            assert_eq!(
                span.parent_span_id.as_ref(),
                Some(&root.span_id),
                "child span {} must parent to the root",
                span.span_id
            );
        }
    }

    Ok(())
}
