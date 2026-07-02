#!/usr/bin/env python3
"""Deterministic self-check for render_verdict.py — no network, no fixtures.

Builds one synthetic `GateRunReport` per `GateDecision` variant (pass /
fail_regression / inconclusive), renders each, and asserts the load-bearing
strings: the three-valued verdict, the stats row, and — for inconclusive —
the mde / required_n "how much more data" sentence that is the whole point
of the verdict. Run directly: `python3 scripts/gate-action/test_render.py`.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from render_verdict import render_markdown, verdict_of  # noqa: E402


def base_report(decision: str, passed: bool, **comparison_overrides) -> dict:
    comparison = {
        "sample_size": 40,
        "baseline_mean": 0.700,
        "candidate_mean": 0.775,
        "delta": 0.075,
        "ci_low": 0.012,
        "ci_high": 0.138,
        "p_value": 0.0213,
        "decision": decision,
        "test": "paired_t",
        "adjusted_alpha": 0.05,
    }
    comparison.update(comparison_overrides)
    return {
        "gate_run_id": "gr-1",
        "gate_name": "main",
        "inconclusive_policy": "fail",
        "experiment_run_id": "exp-1",
        "dataset_id": "ds-1",
        "evaluator_version_id": "exact-v1",
        "baseline_release_id": "rel-base",
        "candidate_release_id": "rel-cand",
        "experiment_decision": decision,
        "passed": passed,
        "reason": f"synthetic {decision} fixture",
        "comparison": comparison,
    }


def expect(markdown: str, needle: str, case: str) -> None:
    if needle not in markdown:
        print(f"FAIL [{case}] missing: {needle!r}\n---\n{markdown}", file=sys.stderr)
        raise SystemExit(1)


def main() -> int:
    passing = base_report("pass", True)
    assert verdict_of(passing) == "pass"
    md = render_markdown(passing, comment_tag="demo")
    expect(md, "✅ Beater eval gate: PASS", "pass")
    expect(md, "<!-- beater-eval-gate:demo -->", "pass")
    expect(md, "| 40 | 0.7 | 0.775 | 0.075 | [0.012, 0.138] | 0.0213 | paired_t |", "pass")

    failing = base_report("fail_regression", False, delta=-0.25, ci_low=-0.31, ci_high=-0.19)
    assert verdict_of(failing) == "fail"
    md = render_markdown(failing, comment_tag=None)
    expect(md, "❌ Beater eval gate: FAIL", "fail")
    expect(md, "Baseline `rel-base` → candidate `rel-cand`", "fail")

    inconclusive = base_report(
        "inconclusive",
        False,
        delta=0.01,
        p_value=0.41,
        mde=0.083,
        required_n=214,
    )
    assert verdict_of(inconclusive) == "inconclusive"
    md = render_markdown(inconclusive, comment_tag=None)
    expect(md, "⚪ Beater eval gate: INCONCLUSIVE", "inconclusive")
    expect(md, "Effects smaller than **0.083**", "inconclusive")
    expect(md, "About **214 paired cases**", "inconclusive")
    expect(md, "an underpowered comparison is not a pass", "inconclusive")

    # Markdown-hostile report values must not corrupt the rendering: `|`
    # would open a table column, a backtick would close an inline-code span,
    # and a multi-line reason would leak out of the blockquote.
    hostile = base_report("fail_regression", False, test="pipe|d")
    hostile["dataset_id"] = "ds`x"
    hostile["reason"] = "line one\nline two"
    md = render_markdown(hostile, comment_tag=None)
    expect(md, "pipe\\|d |", "hostile")
    expect(md, "dataset `ds'x`", "hostile")
    expect(md, "> line one\n> line two", "hostile")

    print("render_verdict self-check: 4/4 cases OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
