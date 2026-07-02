# Issue-Backlog Signal Audit — 2026-07-01

A full-corpus review of all 194 issues in `jadenfix/beater` (numbers #61–#529;
153 open, 41 closed as of 2026-07-01), cross-verified against the codebase at
commit `ae7a209`. Seven parallel analyses covered the corpus by theme (RSI
program, eval statistics, agent-eval packs, strategy/moats, incremental-sync,
product surfaces, platform hygiene); every load-bearing claim below was checked
against code, not just issue text.

Goal: draw a defensible line between the ideas that are truly useful and the
ones that are noise, and name the patterns that let you draw that line again
without re-reading 194 issues.

---

## 1. Anatomy of the corpus

**The backlog is a single generation event, not accumulated demand.** 193 of
194 issues were created in one ~15-hour window on 2026-06-28, in themed waves:

| Wave (UTC hour) | Issues | Theme |
|---|---|---|
| 03:00 | #61–#116 (32) | hygiene, security, stats gaps |
| 04:00 | #119–#200 (51) | gap-analysis vs ARCHITECTURE.md, eval-pack research batch |
| 05:00 | #201–#277 (64) | product-surface ideas (~1/minute), connectors, refactors |
| 06:00 | #278–#334 (34) | strategy docs, incremental-sync decomposition |
| 19:55 | #433–#446 (11) | RSI-optimization program + perf batch |

Several waves are visibly templated (identical section skeletons: *Research
signal → Why → Proposed contract change → Acceptance criteria*). After merging
near-duplicates filed from parallel templates (#202/#204 vs #206, #211 vs #212,
#210/#227/#229, #216/#236/#239-245, #222 vs #253, #184 vs #434), the 194 issues
reduce to roughly **120 distinct ideas**.

**Execution is real and fast.** Closed ≈ shipped: every spot-checked closed
issue has its fix verifiably in the tree (keyset cursors, SSRF guard wired into
all three real browser drivers, auth-required default, DataFusion expression
filters, pinned action SHAs, the #442–446 perf batch, `rust-toolchain.toml`).
All six SECURITY issues were fixed fail-closed, including a residual-leak
follow-up (#314 after #126). A handful were correctly **closed-as-pruned**
(#198 exotic OOD math, #200 uncalibrated embedding scorer) — the tracker
already self-triages.

**Citation hubs** (most referenced by other issue bodies): #202 session cockpit
(21×), #109 StatisticalDesign manifests (17×), #73 product-UI gap (16×), #204
approval cockpit (12×), #71 close-the-RSI-loop (11×). The stats and product-UI
hubs are the load-bearing ideas the rest of the backlog hangs off.

---

## 2. The line: five first-principles tests

These tests, derived from what actually got built vs. what stayed paper,
separate signal from noise across all seven clusters:

1. **Evidence test.** An eval/analysis idea is signal only if the platform can
   already capture the evidence it scores. Tool-contract conformance (#165)
   scores `tool.call`/`mcp.request` spans that exist in
   `sdks/semconv/conventions.json` → buildable. Voice evals (#259), memory
   lifecycle (#162), multi-agent handoffs (#159) need evidence classes (audio,
   store snapshots, agent-channel attributes) that don't exist → premature.

2. **Consumer test.** Code with no consumer is landed paper. `beater-knowledge`
   shipped well-tested and is imported by nothing — and forked the Merkle
   implementation in the process. Conversely `beater-core/src/merkle.rs` is 150
   honest lines whose docstring explicitly rejects the SOTA recommendations of
   #317–#325 because "Beater has none of those consumers today." That docstring
   is the best triage artifact in the repo.

3. **Loop test.** Beater's differentiation is instrument → trace → promote
   failure to dataset → eval → gate → monitor. Ideas that compound that loop
   (capture more evidence in, act on gate output) are signal; ideas that are
   separate products in disguise (ROI dashboards #222, vendor-comparison
   cockpit #223, collaboration suite #241, comms audit #218) dilute a small
   team.

4. **Grounding test.** Issues that cite real `file:line` evidence (the
   live-replay batch #239–#245, the perf batch #442–#446, #146, #263) are the
   highest-precision issues in the corpus. Issues whose "research signal"
   sections lean on decorative citations (several unverifiable 2026 arXiv IDs)
   are idea-generation output whose value lives entirely in the acceptance
   criteria, if anywhere.

5. **Execution-path test.** Observability reads; control planes execute.
   Ideas that put Beater in the agent's execution path (MCP gateway #232, live
   approval cockpits #204/#206) are strategic pivots with real blast radius,
   not features — they need a deliberate decision, not a backlog slot.

---

## 3. Cross-cutting patterns

1. **Code outran the tracker — badly.** At least ten open issues are
   substantially done: #62 (Brier/ECE/reliability bins shipped in
   `beater-calibration`), #72 (overfit-rejection test exists in `rsi.rs`),
   #109 (`beater-design` implements the manifest, load-bearing since PR #456),
   #111 (~90% of the stats checklist), #112 (`MonotoneBisect` landed), #142
   (`split.rs` implements exactly the fix), #147 (its "no mSPRT exists" premise
   is false since `sequential.rs`), #153 (onboarding is now
   `docker compose up`), #436 (all six techniques landed), #140 (trajectory
   scorers in `EVALUATOR_CATALOG`), #166 (flip-search shipped). Commit messages
   cite PR numbers, not issue numbers, so nothing closes automatically. This
   is the strongest argument in the corpus for #296 (generated/CI-checked
   architecture status).

2. **"Merged but unwired" is the house failure mode.** SSRF guard merged but
   only wired into `MockDriver` (#113), artifact cap merged but inert (#116),
   gateway spans → `NoopSpanSink` (#164/#182), ClickHouse store implemented but
   never selectable in `beaterd` (#74), six governance crates scaffolded in one
   commit (`7bee84c`) with zero API routes. Features pass review; production
   wiring is the systematically missed step. Any "done" claim in this repo
   needs a wiring check, and acceptance criteria should demand an end-to-end
   proof gate (the repo's own #240 pattern).

3. **Safety brakes are filed alongside ambition — and honored.** #143 demanded
   no `apply_change` until real stats + Test split + forked replay exist;
   #198 demanded prune-or-prove on speculative math. The code trail shows both
   bound behavior: gate/stats/split landed first, `apply_change` still has zero
   implementations, and only #198's retained guardrail set (gap CI,
   Thresholdout, Ladder in `overfit.rs`) was built. The RSI program is
   executing in its safety-mandated order.

4. **The tracker peer-reviews itself.** #147 catches the spec's acceptance
   criteria contradicting its own statistics section; #197 demotes the metric
   #62 introduced; #111 audits a merged PR line-by-line; #529 catches
   fingerprint drift between two features merged days apart. This
   self-policing is why the closed-as-pruned issues exist and is worth
   preserving as house style.

5. **Honesty is the only uncopyable moat.** "Never pass inconclusive" (tested
   at `beater-gates/src/lib.rs:673`), replay modes that confess cassette
   incompleteness, family-wise withdrawal of borderline wins, refusal-based
   estimators (`EvalDesign::permit_pass`). Incumbents' demo/growth motions
   depend on green checkmarks; a product that prints "inconclusive — you don't
   have enough data to ship" is structurally against their interests to copy.
   Every top-ranked idea below is a projection of this invariant into a
   customer-visible surface.

6. **Engine surplus, distribution deficit.** The hard, hard-to-copy machinery
   exists: 13-module `beater-stats` (~5.4k LOC), cassette replay with honest
   completeness, WASI-deterministic scorer sandbox, judge request-hash cache,
   scenario mining, pre-registration gate design. What does not exist is every
   thin adoption surface: no GitHub Action (`action.yml` absent), no
   `ReleaseEvidencePacket` (zero grep hits), one competitor importer of four,
   no demonstrated $0-keyless CI rerun. The strategy issues keep re-litigating
   positioning (#149 vs #152 vs #332 vs #320) while the wrappers stay unshipped.

7. **Research decomposition ≠ buildable backlog.** The incremental-sync
   program (#317–#325) is excellent literature review with zero code impact;
   the RSI program (#433–#438) is the counterexample — same deep-research
   method, but each substep names concrete types, files, and invariants, and
   half of it landed within days. The difference: the RSI issues were
   decomposed against the repo's actual types; the sync issues were decomposed
   against other companies' systems.

---

## 4. Signal: ranked build list

### Tier 1 — the wedge-completing five (build next)

1. **Eval-CI GitHub Action with a statistical verdict (#154, first slice of
   #152/#92).** A composite action wrapping the already-existing
   `beaterctl gate-run` / `gate-run-fixture`, printing pass / fail /
   **inconclusive** with CI/power in the PR comment, with $0 keyless reruns via
   the judge request-hash cache + cassettes. Everything hard exists; this is
   an action.yml plus a Markdown formatter, and it is the distribution vehicle
   for the entire engine. No incumbent can print "inconclusive" without
   breaking their demo culture.

2. **Perturbation-execution layer for scenario replay (#138, completing
   #309).** `Scenario.perturbation_knobs` ship on every promoted scenario but
   nothing executes them. Cassette mutation (stale source, timeout, schema
   drift, injected content) + clean-vs-faulted paired scoring via the existing
   `compare_paired_scores_cuped`/`GatePolicy` machinery turns the scenario
   catalog from a filing cabinet into an AV-style regression gate. Highest
   leverage-to-effort in the eval clusters.

3. **`ReleaseEvidencePacket` (#326).** Deterministic aggregation of gate
   decision + stats CIs + cassette-completeness + redaction status into a
   customer/security-reviewer-shareable artifact that refuses to overclaim.
   All inputs exist as typed data; the honesty rules are a pure projection of
   invariants already enforced in `beater-gates`/`beater-replay`/`beater-stats`.
   Completes #320's wedge ("customer-shareable report") and pairs naturally
   with #234 (reviewer handoff packets on agent-authored PRs).

4. **Terminal-agent transcript import (#227, with #225 `beaterctl agent init`
   as front door).** The largest untapped evidence corpus — Claude
   Code/Codex/Cursor session JSONL — already sits on users' disks. A parser +
   span normalizer + fixtures (exactly the deterministic-crate shape the owner
   demonstrably ships), feeding the existing
   `/v1/datasets/{...}/cases/from-trace` promotion path. Cheapest possible
   capture expansion; zero execution-path risk.

5. **GEPA proposer on the now-real RSI substrate (#434, fed by #435
   phase 2).** The gate, split, stats, broker plumbing, and optimization-round
   driver all exist; `ProposalContext` already carries failure signatures. A
   trace-reflective Pareto proposer is now an enum-variant implementation, not
   a platform build — and it is the shortest path to making the README's
   headline demonstrably true. Hard constraint: #143's brake stays on
   (`apply_change` only through the held-out gate).

### Tier 2 — validity and identity debt that silently corrupts decisions

- **#263 human-label provenance** — labels come from a priority-DESC
  convenience queue and feed calibration as unqualified ground truth; the last
  unguarded input to an otherwise-hardened inference chain.
- **#129 pass^k / flakiness-bounded gates** — the platform models agents as
  stochastic policies but gates on one draw; `RepetitionPlan.trial_count`
  already reserves the slot.
- **#146 finish at the telemetry layer** — `sampling_weight` on
  `CanonicalSpan` + `biased` tag on unweighted roll-ups; the gate layer refuses
  biased samples but the daily-visible roll-up API is still a silently biased
  average.
- **#529 unify content-fingerprint vocabulary** — three incompatible hash/
  Merkle notions in-tree (`split.rs`, `datasets::case_content_hash`,
  `beater-knowledge`), under a gate that now depends on `corpus_root`; do it
  before roots are load-bearing externally.
- **#165 tool-contract conformance scorers (+#163 abstention rider)** —
  deterministic WASI-lane scorers over existing span kinds; catches
  hallucinated tools structurally where competitors reach for a judge.
- **#260 end-user feedback → dataset evidence** — thumbs/escalations as
  trace-linked, promotable evidence; mostly one record type over the existing
  `beater-human` promotion path.

### Tier 3 — platform bets (each needs one deliberate decision, then pays)

- **#90 embedded DataFusion HOT TraceStore + synchronous-consistency SLO** —
  ClickHouse-class analytics in the single OSS binary; deps already vendored,
  trait + conformance suite ready. The one hygiene-cluster idea that is a moat.
- **#255 agent-native canonical trace fields** (sessions, messages, media,
  sampling weights) — schema-as-product; unlocks the session layer that
  #202/#216/#250 all presuppose; rides the existing contract-regen machinery.
- **#239 cursored live-stream contract (SSE)** — one primitive unblocking seven
  live-surface issues; build the contract, not the cockpits; #240/#244/#245
  are its proof gates.
- **#155 Switzerland importers** — `LangfuseImporter` proves the
  `SourceImporter` pattern; LangSmith/Phoenix are pattern-stamping. Anti-lock-in
  is the one positioning incumbents cannot copy.
- **#296 (+#261) generated architecture status + optimality scorecard** — the
  repo's own history (phantom §20.10, three drifted status ledgers, ten
  stale-open done issues) proves hand-maintained status lies; generalizes an
  existing drift-gate habit.

---

## 5. Noise: what to close or park

**Close as pruned (research shelf-ware / wrong stage):**
- #317, #321, #323, #324, #325 — FastCDC/RBSR/CRDT/ANN-churn machinery for a
  distributed corpus-sync product with zero users; keep #327 as reference
  index only.
- #433 — io_uring/rkyv/PGO ingest optimization with no measured bottleneck;
  keep only the cheap build-flags item.
- #136 adaptive eval scheduling, #168 routing policies, #169 simulator
  calibration — statistically literate but wrong ordering; revisit when the
  capabilities they meta-manage exist.
- #159 multi-agent metrics, #162 memory lifecycle, #259 voice — fail the
  evidence test today; real someday, unbuildable now.
- #218 comms audit, #219 AG-UI adapter, #222 ROI dashboard, #223
  vendor-comparison cockpit, #235 dynamic tool discovery, #241 shared live
  sessions — separate products in disguise; #253 is the grounded survivor of
  the ROI theme.
- #332's headline (lead with RSI) — refuted by its own evidence (optimizer is
  the least-built pillar; Braintrust Loop is live); keep its risk list. #320
  won the strategy argument and the README already reflects it.

**Close as done (tracker hygiene; ~30 minutes of clicking):**
#62, #72, #109, #111 (file the `statrs` residue narrowly if wanted), #112,
#140 (rescope to deterministic sub-scores + UI), #142, #147 (residue is
#167's wiring), #153, #166 (rescope to intervention taxonomy + report
surface), #197, #436. Also re-scope #91 to its cassette-CAS remainder and
#184 as superseded by #434.

**Genuine bugs from the corpus that remain open and true:** #141 (live
side-effect safety before forked replay on the no-cassette path — a real
pre-RSI blocker), #203 (sync mutexes in async paths), #278 (non-atomic Stripe
webhook apply), #293 (connector invocation authorized by broad `eval:run`),
#284 (C conformance bypasses the SDK it claims to test).

---

## 6. Meta-observation

This backlog is itself an experiment in agent-generated planning, and the
result is legible: **repo-grounded decomposition executes; category-list
decomposition shelves.** The waves that cited actual types and file:line
evidence (#239–#245, #442–#446, the RSI program) were substantially built
within days; the waves that enumerated a research field's categories
(#317–#325, most tier:optional product surfaces) produced zero code. The
corpus also contains its own quality-control loop — brake issues, prune
issues, peer-review issues — which is rarer and more valuable than any single
feature idea in it.

The single highest-leverage fact: the engine is built and the distribution
layer is not. Five thin surfaces (Action, perturbation gate, evidence packet,
transcript import, GEPA) stand between the current repo and its stated wedge
being externally visible. Everything else in the backlog is either validity
debt worth paying (Tier 2), a deliberate platform bet (Tier 3), or noise that
the repo's own triage habits are already equipped to shed.
