# RSI Program — scientific plan for holistic, non-overfitting recursive self-improvement

> Status: in progress. Stacked PRs off `main`. Provenance: deep-research runs
> `wf_03a6f1ee-b78` (sync primitives), `wf_448ac492-e12` (optimizers/stats/sim),
> `wf_0658e8b2-861` (overfitting-resistant evaluation methodology), plus the
> in-repo RSI surface map. Tracking issues: #438 (umbrella), #433–#437.

## Thesis

Beater's differentiator is **gated recursive self-improvement**: optimizers only
*propose* changes; nothing ships unless it beats baseline on a **held-out Test
set** with statistical confidence and survives an **anti-overfitting guardrail**.
The moat is the gate, not the optimizer. Therefore the program is built
**gate-first**, and every optimizer is validated against a simulation that is
deliberately engineered to *catch* overfitting.

## The keystone gap (why we start here)

The RSI surface map found that the "held-out Test gate" the optimizer docs assume
**has no data substrate**: `beater-datasets` models immutable version snapshots
but has **no train/val/test split, no holdout, no contamination/leakage
detection**. The §21.4 anti-overfitting guardrail is referenced but unimplemented.
You cannot claim "non-overfitting RSI" without first being able to *hold data
out* and *prove it wasn't contaminated*. That is PR1.

## First-principles substeps → stacked PRs

Each PR is independently reviewable, tested, and (where it touches `/v1`)
contract-regenerated. Stacked because each depends on the prior substrate.

| PR | Substep | Crate(s) | Depends on | Issue |
|----|---------|----------|-----------|-------|
| **1** | Deterministic split + contamination substrate | `beater-datasets` | — | #334-adjacent |
| **2** | Anti-overfitting guardrail (§21.4): reusable holdout + generalization-gap | `beater-stats`, `beater-experiments` | 1 | #436 |
| **3** | GEPA reflective optimizer + enriched `ProposalContext` | `beater-experiments`, `beater-judge` | 1,2 | #434, #435 |
| **4** | RSI simulation harness + real-OS-repo dataset generator | `beater-bench` (new module) | 1,2,3 | #437-adjacent |
| **5** | Iterate/review loop; expose via MCP/CLI/SDK; validate non-overfitting | all | 1-4 | #438 |

### PR1 — split + contamination (this PR)
- **Deterministic, stable hash split.** A case's split is a pure function of
  `(case_id, seed)` — *independent of dataset composition*. Adding/removing cases
  never reshuffles existing assignments, so a case can never silently migrate
  train→test across re-versioning (the classic leakage source). Mirrors
  production hash-bucketing holdout discipline.
- **Contamination/leakage detection.** Group cases by content fingerprint
  (`sha256_json_hash(input)`); flag any fingerprint whose cases span >1 split —
  i.e. an identical input leaking train↔test. Exact-match in PR1; near-dup
  (n-gram/embedding) is a documented follow-up (contamination from public-repo
  memorization is handled in PR4's generator, per methodology research).
- **No contract change.** Pure library module + tests. Keeps the PR small and the
  review crisp.

### PR2 — anti-overfitting guardrail (§21.4)
- **Reusable holdout (Thresholdout/Dwork ladder).** The optimizer queries the
  Val set many times across rounds; naive reuse overfits. Add a budgeted,
  noise-thresholded holdout mechanism + a strict Train(optimize) /
  Val(select) / Test(gate, queried once) discipline enforced by PR1 splits.
- **Generalization-gap check.** Reuse `beater_stats::bootstrap_diff_ci` to bound
  the Train−Test gap; a candidate whose held-out lift CI doesn't clear the
  regression bound is rejected even if Train improved. Wires into the existing
  `compare_paired_scores` → `GateDecision` path.

### PR3 — GEPA + features
- `OptimizerStrategy::Gepa`: genetic-Pareto search with natural-language
  reflective mutation over execution traces (research: +6–20% vs RL GRPO at up to
  35× fewer rollouts; +>10% vs MIPROv2). Reflection delegates to the judge/LLM
  broker. Pure *proposal* — acceptance still via PR2 gate.
- Enrich `ProposalContext` with `failure_features` (taxonomy label distribution
  via 82%-accuracy classifier, root-cause span attribution from
  `beater-replay`), so proposals target real failure modes.

### PR4 — simulation harness + real-data generator
- Generate eval suites from a **real OS repo** with **controlled
  characteristics**: simple, complex, nuanced, hidden/spurious-correlation traps,
  and genuine **out-of-distribution** splits — plus contamination checks against
  memorized public data (methodology research wf_0658e8b2-861).
- Run the full loop; report **Train vs Val vs Test vs OOD** lift and the
  generalization gap. The harness's job is to *try to make the optimizer
  overfit* and confirm the gate stops it.

### PR5 — iterate/review/expose
- Loop: simulate → advisor-subagent review → fix → repeat until held-out **and**
  OOD improve without an overfitting gap. Expose `propose`/run/gate end-to-end via
  MCP tool + CLI subcommand + SDK (currently library-only). Tag/prune issues.

## Non-negotiable invariant (holds across all PRs)
Optimizers and features make proposals **better and cheaper**; **only the gate
(PR2) decides acceptance**, and it decides on a **Test set held out by PR1** and
**queried under the reusable-holdout budget**. Triage shortcuts (counterfactual
pre-screen, surrogate judges) may never let a candidate bypass the real held-out
gate. That invariant *is* the product.
