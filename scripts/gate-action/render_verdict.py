#!/usr/bin/env python3
"""Render a `beaterctl gate-run` JSON report as a CI-facing Markdown verdict.

Consumed by the repo-root composite action (`action.yml`). Reads the raw
`GateRunReport` JSON that `beaterctl gate-run` prints, and emits:

  - a Markdown verdict block (step summary and/or sticky PR comment body),
  - GitHub Actions outputs (`verdict`, `passed`, `decision`, `reason`).

The verdict is deliberately three-valued — pass / fail / **inconclusive** —
mirroring `GateDecision` in `beater-eval`. Inconclusive is a first-class
outcome, not a soft pass: the block reports the minimum detectable effect at
the current sample size and how many paired cases would have resolved the
comparison (`mde` / `required_n` from `ExperimentComparison`), so the PR
answers "how much more data do I need?" instead of shipping on noise.

Stdlib only: this must run on a bare GitHub runner with no pip installs.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Stable snake_case names from beater-eval's GateDecision.
DECISION_PASS = "pass"
DECISION_FAIL = "fail_regression"
DECISION_INCONCLUSIVE = "inconclusive"

VERDICT_STYLE = {
    "pass": ("✅", "PASS"),
    "fail": ("❌", "FAIL"),
    "inconclusive": ("⚪", "INCONCLUSIVE"),
}


def verdict_of(report: dict) -> str:
    """Collapse the report to pass | fail | inconclusive.

    `passed` already folds the gate's inconclusive policy into a boolean, but
    the three-valued decision is the message: an inconclusive that the policy
    fails must not render as a regression.
    """
    decision = report.get("experiment_decision")
    if decision == DECISION_INCONCLUSIVE:
        return "inconclusive"
    if report.get("passed"):
        return "pass"
    return "fail"


def fmt_num(value, digits: int = 4) -> str:
    if value is None:
        return "—"
    if isinstance(value, float):
        return f"{value:.{digits}g}"
    return str(value)


def cell(value) -> str:
    """Make a value safe inside a Markdown table cell: `|` starts a new
    column and a newline ends the row."""
    return str(value).replace("|", "\\|").replace("\n", " ")


def inline(value) -> str:
    """Make a value safe inside `inline code`: a backtick would close the
    span and leak the rest as raw Markdown."""
    return str(value).replace("`", "'").replace("\n", " ")


def fmt_p_value(p) -> str:
    if p is None:
        return "—"
    if p < 0.0001:
        return "<0.0001"
    return f"{p:.4f}"


def render_markdown(report: dict, comment_tag: str | None) -> str:
    verdict = verdict_of(report)
    emoji, label = VERDICT_STYLE[verdict]
    comparison = report.get("comparison") or {}

    lines: list[str] = []
    if comment_tag:
        # Hidden marker that keeps the PR comment sticky across pushes.
        lines.append(f"<!-- beater-eval-gate:{comment_tag} -->")
    lines.append(f"## {emoji} Beater eval gate: {label}")
    lines.append("")

    gate_name = inline(report.get("gate_name", "?"))
    dataset = inline(report.get("dataset_id", "?"))
    evaluator = inline(report.get("evaluator_version_id", "?"))
    lines.append(
        f"**Gate** `{gate_name}` · dataset `{dataset}` · evaluator `{evaluator}`"
    )
    baseline = inline(report.get("baseline_release_id", "?"))
    candidate = inline(report.get("candidate_release_id", "?"))
    lines.append(f"Baseline `{baseline}` → candidate `{candidate}`")
    lines.append("")

    if comparison:
        alpha = comparison.get("adjusted_alpha")
        ci_label = (
            f"{100 * (1 - alpha):g}% CI" if isinstance(alpha, (int, float)) else "CI"
        )
        test = comparison.get("test")
        test_name = test if isinstance(test, str) else json.dumps(test)
        lines.append(
            f"| n | baseline | candidate | Δ | {ci_label} | p | test |"
        )
        lines.append("|---|---|---|---|---|---|---|")
        lines.append(
            "| {n} | {b} | {c} | {d} | [{lo}, {hi}] | {p} | {t} |".format(
                n=comparison.get("sample_size", "—"),
                b=fmt_num(comparison.get("baseline_mean")),
                c=fmt_num(comparison.get("candidate_mean")),
                d=fmt_num(comparison.get("delta")),
                lo=fmt_num(comparison.get("ci_low")),
                hi=fmt_num(comparison.get("ci_high")),
                p=fmt_p_value(comparison.get("p_value")),
                t=cell(test_name),
            )
        )
        lines.append("")

    if verdict == "inconclusive":
        n = comparison.get("sample_size", "?")
        mde = comparison.get("mde")
        required_n = comparison.get("required_n")
        detail = [
            f"This comparison lacked the power to resolve the regression bound at n={n}."
        ]
        if mde is not None:
            detail.append(
                f"Effects smaller than **{fmt_num(mde)}** (in the metric's own units) are invisible at this sample size."
            )
        if required_n is not None:
            detail.append(
                f"About **{required_n} paired cases** would make the observed effect conclusive."
            )
        policy = report.get("inconclusive_policy")
        outcome = "fails" if not report.get("passed") else "passes"
        detail.append(
            f"The gate's inconclusive policy (`{policy}`) {outcome} this run — an underpowered comparison is not a pass."
        )
        lines.append(" ".join(detail))
        lines.append("")

    reason = report.get("reason")
    if reason:
        # Prefix every line so a multi-line reason stays inside the quote
        # instead of leaking as raw Markdown after the first line.
        quoted = str(reason).replace("\r", "").replace("\n", "\n> ")
        lines.append(f"> {quoted}")
        lines.append("")

    run_id = inline(report.get("gate_run_id", "?"))
    experiment = inline(report.get("experiment_run_id", "?"))
    lines.append(f"<sub>gate run `{run_id}` · experiment `{experiment}`</sub>")
    return "\n".join(lines) + "\n"


def append_file(path: str, content: str) -> None:
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(content)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True, help="gate-run JSON report path")
    parser.add_argument("--summary", help="file to append the Markdown to (GITHUB_STEP_SUMMARY)")
    parser.add_argument("--outputs", help="file to append action outputs to (GITHUB_OUTPUT)")
    parser.add_argument("--comment-file", help="write the Markdown comment body here")
    parser.add_argument("--comment-tag", help="sticky-comment marker tag")
    args = parser.parse_args()

    raw = Path(args.report).read_text(encoding="utf-8")
    report = json.loads(raw)
    # `gate-run-fixture` wraps the report; accept both shapes so the demo mode
    # and the real mode share one renderer.
    if "gate_run" in report and "experiment_decision" not in report:
        report = report["gate_run"]

    verdict = verdict_of(report)
    markdown = render_markdown(report, args.comment_tag)

    if args.summary:
        append_file(args.summary, markdown + "\n")
    if args.comment_file:
        Path(args.comment_file).write_text(markdown, encoding="utf-8")
    if args.outputs:
        # GITHUB_OUTPUT is line-oriented: both \n and \r would terminate or
        # corrupt the value.
        reason = str(report.get("reason", "")).replace("\r", " ").replace("\n", " ")
        append_file(
            args.outputs,
            "verdict={v}\npassed={p}\ndecision={d}\nreason={r}\n".format(
                v=verdict,
                p="true" if report.get("passed") else "false",
                d=report.get("experiment_decision", ""),
                r=reason,
            ),
        )

    print(f"beater eval gate verdict: {verdict}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
