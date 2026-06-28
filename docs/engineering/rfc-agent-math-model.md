# RFC: Honest math for agent observability

Status: Draft / thinking document
Companion issues: see "Tracked work" at the bottom.

## The one defensible thesis

Beater stores each agent run losslessly as a **timed, labeled span forest**
(`crates/beater-schema` `CanonicalSpan`: 14 `AgentSpanKind`s, each with
cost/tokens/latency/status/model). So the runs are an *already-observed stochastic
process*, and the platform's job is inference on it.

But the product's value is narrow and concrete: **"ship / don't ship this agent
change."** A tool that answers that must be **honest about uncertainty at small
sample sizes** — most eval setups have *tens* of cases, not thousands. That single
property (don't declare a false green at n=12) is where almost all the real value
is, and it needs humble statistics, not impressive ones.

Design rule: **compress every concern to one calibrated, uncertainty-bearing
scalar**, with progressive disclosure (scalar → distribution → raw trace). Heavy
machinery only earns its place if it changes that scalar at realistic n. Most of
it doesn't.

This document is deliberately small. An earlier draft proposed a new crate and
~11 endpoints of branching-process / causal / off-policy machinery; that was math
looking for a problem. The triage below records what survived and *why the rest
was cut*, so it isn't relitigated.

---

## Build now (small, high-value, data already exists)

### 1. Report required-n instead of a binary "underpowered"
`beater-eval::compare_paired_scores` returns `EvalError::Underpowered` — all or
nothing. From the observed difference variance `σ̂²` and the
`max_regression` already in `GatePolicy` as the minimum detectable effect `δ`:

```
n* ≈ 2 (z_α + z_β)² σ̂² / δ²
```

Surface **"need ~24 more cases to detect a 1% regression (95%/80%)"** instead of a
dead end. ~50 lines in `beater-eval`; no new crate. This is the flagship: cheap,
honest, and the thing most eval tools get wrong.

### 2. Judge trust = agreement + a reliability diagram (display only)
`beater-calibration` currently reduces judge-vs-human to a scalar
`pass_threshold`. Add, as *display*, the measurement-error view:

- **Cohen's κ** (agreement beyond chance) between judge and human labels.
- A **reliability diagram + ECE**: does a judge score of 0.7 mean a 70% pass rate?

No recalibration, no latent-variable model yet — just show whether the judge can
be trusted. Recalibration (isotonic/Platt) is deferred until label volume exists.

---

## Build soon (useful, slightly larger)

### 3. Critical-path latency on the waterfall
Wall-clock latency is the **longest weighted path in the timed span DAG**, not
`Σ d_v`. Highlight it on the existing waterfall and, when two critical spans are
independent, report "parallelizing X and Y saves ≈Δt." Caveat: many agent traces
are sequential (LLM→tool→LLM), where critical path ≈ the sum — so this pays off
only with real tool fan-out. Build it, but don't oversell it.

### 4. Simple regression alerting
A windowed error-rate / quality monitor with **CUSUM** (not Bayesian online
changepoint — too heavy for the payoff) → "regression began near commit X," posted
through `crates/beater-alerts`.

### 5. Failure grouping without ML
Group failing traces by **error string / failing tool / status** first. This
captures most of the "find the failure mode" value of embedding-based clustering
with zero ML infrastructure. Embedding clusters are a later upgrade, not the MVP.

---

## Deferred — right idea, needs data or scale we don't yet have

| Idea | Blocker |
|---|---|
| Anytime-valid confidence sequences for gating | Solves *peeking on streaming data*; CI gating is a single look on a fixed dataset. Revisit when production/online evals stream in. |
| Isotonic / Platt judge recalibration | Needs a sizeable labeled calibration set. |
| Dawid–Skene latent-truth rater model | Needs *multiple raters per item*; real data is ~1 human + 1 judge (degenerate, weakly identified). |
| Active learning by uncertainty for dataset promotion | Depends on calibrated judge uncertainty (circular until #2 lands); start with "label one trace per failure group" from #5. |
| Wasserstein / distributional regression check | Marginal over mean-of-differences at small n. |

---

## Cut — mathematically pretty, practically inert or infeasible

| Idea | Why it's cut |
|---|---|
| **Spectral radius ρ(M)** of the offspring matrix as a runaway gauge | Real agents have hard step caps, so the branching process is sub-critical *by construction* — ρ(M) is near-meaningless. And estimating M requires already having logged the runaways you'd detect directly. A `steps > N` / `depth > D` / `cost > budget` counter strictly dominates: exact, per-run, immediate. |
| Observational causal inference (backdoor/IPW over the trace DAG) | Severe confounding, no positivity. Self-defeating: if you can **replay**, just run the A/B (replay *is* the intervention); if you can't, the observational estimate is too confounded to trust. Keep "replay = experiment"; drop the causal-DAG machinery. |
| Off-policy routing (IPW / doubly-robust) | Requires logged action **propensities** (P(model chosen)). Traces don't record them — the model was picked deterministically/by hand — so IPW is undefined and DR degrades to a hand-built simulator. |
| Mahalanobis / MMD drift scores | Agent features are heavy-tailed and mixed-type; Σ⁻¹ is unstable and "MMD = 0.03" is uninterpretable. Per-metric drift (p99 latency moved, error rate moved) beats it on interpretability and ROI. |
| Latent-mode HMM ("explore/stuck/recover") | Speculative; no evidence the modes are real or actionable. |

---

## Why not more sophistication?

Two facts kill the fancy tail:

1. **n is tiny.** Confidence sequences, MMD, DR estimators, and latent-rater models
   need hundreds-to-thousands of samples to beat a t-test and a reliability
   diagram. At n≈30, simple *is* optimal and far more interpretable.
2. **The one-scalar UX makes sophistication invisible *and* unnecessary.** If the
   user sees only a gauge, they can't tell a z-test from an e-process — and at
   realistic n the two scalars are nearly identical. Conditioned on compression,
   the marginal value of the heavy math is ~zero.

The defensible edge is **honesty**, not sophistication: power analysis that
refuses to over-claim, calibrated judge scores, and an explicit "inconclusive —
need more data" instead of a false green.

---

## Delivery note (unchanged, and still good)

Per `CLAUDE.md`, any `/v1` endpoint auto-projects into MCP tools + 7 SDKs + CLI
from one OpenAPI artifact. So the build-now items, once they're handlers in
`crates/beater-api`, become agent-callable MCP tools for free — no separate math
service, staying offline/self-hostable (R11). No new crate until there are enough
estimators to justify extracting one; for now they live in `beater-eval` and
`beater-calibration`.

---

## Tracked work

- #61 — Gate: report required-n / minimum detectable effect (Build now #1)
- #62 — Judge trust: Cohen's κ + reliability diagram / ECE, display-only (Build now #2)
- #63 — Critical-path latency on the trace waterfall (Build soon #3)

(Deferred and cut items above are intentionally *not* issues — they're recorded
here so they aren't rebuilt by reflex.)
