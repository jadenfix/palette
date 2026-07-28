---
name: review-pr
description: High-recall, high-precision independent review of a beater PR. Use when asked to review a PR in jadenfix/beater (e.g. "/review-pr 575"). Reviews must be done by an agent that did NOT author the PR.
---

# beater PR review

You are an independent, non-author reviewer for `jadenfix/beater`. The argument is a PR number: `$ARGUMENTS`. Several agents work this repo concurrently — assume nothing about freshness, and never rubber-stamp. This rubric teaches you *how* to find bugs on any PR; it is deliberately not a list of past bugs to grep for.

## Ground rules

- **Non-author only.** Check `gh pr view <N> -R jadenfix/beater --json commits -q '.commits[].messageHeadline'` — if you recognize any commit as your own work from this session, stop and hand the review to another agent.
- Read-only: do not modify the main clone, do not run `cargo` in a directory another agent may be building in. CI (backend, sdk-contract, frontend workflows) already builds per-PR; review by reading.
- Precision: every **blocker** carries a concrete traced failure scenario (specific input/state → specific wrong behavior, with `file:line`). If you cannot trace one, it is a nit.
- Recall: read the ENTIRE diff, the referenced issues, and the surrounding code of every touched file at current `main`. Bugs live at the seams the diff doesn't show.

## Procedure

1. `gh pr view <N> -R jadenfix/beater --json title,body,author,files,mergeStateStatus,statusCheckRollup`
2. `gh pr diff <N> -R jadenfix/beater` — all of it.
3. `gh issue view <issue> -R jadenfix/beater` for every referenced issue; the issue defines the intended scope (did the PR do more or less than it needs?).
4. **Supersession check:** `git log origin/main --oneline -30` plus targeted `git log -p` on touched files. Concurrent agents mean main may already contain an equivalent fix → REJECT (superseded).
5. **Freshness check:** after any wait, force-push, PR body edit, or CI rerun, re-read PR state, head SHA, base SHA, check rollup, and linked issue state. A closed PR, stale-head check run, or already-closed issue is not merge evidence.
6. **Overlap check:** `gh pr list -R jadenfix/beater --state open` — flag open PRs touching the same paths and whether merge order matters.
7. Hunt for bugs using the method below.
8. Post the review (format at the bottom) and return a structured verdict.

## How to find bugs (do this — don't just tick boxes)

- **Trace one path end to end.** Pick the main value or state the PR changes and follow it through the code — into the error and edge branches, not just the happy path. Most blockers live on the path the tests don't take.
- **Review from three seats.** beater serves an **instrumented agent developer** (SDK/OTLP caller), a **CI gate** (a wrong eval verdict ships a regression or blocks a good change), and a **beaterd operator** (tenant isolation, storage growth, ingest backpressure). For the code in the diff, ask how it hurts each of the three.
- **Enumerate failure modes** for every new input, call, or state transition: empty · malformed · oversized · slow/hung · repeated/retried · concurrent · out-of-order · partial failure · adversarial/untrusted.
- **Follow the seams the diff hides:** the callers of every changed signature, the callees it now leans on, and any invariant elsewhere that assumed the old behavior.
- **Reverted-fix test:** would any test in the PR still pass if the fix itself were reverted? If yes, the test proves nothing — that's a blocker for a bugfix PR.
- **Adversarially verify** each candidate blocker before it goes in the review: try to refute it against the code. Survives refutation → blocker. Can't build a concrete trace → nit.
- **Preserve durable lessons without breaking read-only review.** Call out persistent, non-overfit invariants under `Durable guidance`; a follow-up author lands accepted guidance in this file from a separate worktree/PR.

## What to look for (general bug classes)

Correctness & honesty of the contract:
- [ ] Return values and status flags tell the caller the truth — a failure or a no-op is never reported as success; output that drops, truncates, samples, or rate-limits says so.
- [ ] Docs, comments, and declared schemas match what the code actually does — no present-tense claims for a stub; `[contract]`-marked requirements are not claimed as implemented.
- [ ] Handles/IDs the caller reuses across calls are stable, or their churn is handled.

Resource, lifecycle & availability:
- [ ] Everything that can grow is bounded: ingest queues, span/artifact sizes, maps, caches, retries, spawned tasks. Unbounded growth on remote-driven input is a blocker; a size check that runs only after allocating the full payload is not a bound.
- [ ] Every external round-trip has a timeout **and** a recovery path; cleanup runs on every exit path including error and cancel.
- [ ] Locks are narrow, ordered, and never held across `.await`; ingest and read paths must not serialize behind each other.

Tests:
- [ ] A test exercises the actual failure mode (survives the reverted-fix question); caps/timeouts/limits are tested at, below, and above the boundary.

Fit & simplicity (more code is not better):
- [ ] The change does exactly what its issue needs — no speculative abstraction, dead branch, unused config knob, or new dependency without justification.
- [ ] It fits ARCHITECTURE.md and REQUIREMENTS.md; layer direction respected (schema/core crates never depend upward on api/ingest).

## beater-specific bug classes (check every one the diff touches)

Contract-first (the sdk-contract gate is the product):
- [ ] Any runtime route, status, or response-field change regenerates the OpenAPI contract (`sdks/openapi/beater-api.json`) and SDK-facing fixtures in the SAME PR. A route or field that exists at runtime but not in the contract — or vice versa — is a blocker: 7 SDKs, the CLI, and MCP tools are generated from it.
- [ ] Semconv changes go through `crates/beater-schema` conventions + `xtask regen-semconv`, never hand-edited into `sdks/semconv/conventions.json`.

Tenant scoping & auth:
- [ ] Every query and mutation is scoped by tenant/project/environment derived from auth, not from a caller-supplied body field. Any path where tenant A can read or write tenant B's traces, datasets, or artifacts is a critical blocker.
- [ ] `--auth-mode required` remains the default; new endpoints are registered under auth, not accidentally public.

Canonical schema & ingest:
- [ ] `CanonicalSpan` shape changes bump `schema_version` and ship a reprojection (the v1→v2 `reproject.rs` pattern); old stored spans must still read correctly.
- [ ] Attribute mapping keeps OpenInference-first, `gen_ai.*`-fallback resolution order; unmapped attributes go to `unmapped_attrs` losslessly; `raw_ref` is never dropped.
- [ ] Ingest idempotency keys are honored on retry paths; dead-letter replay preserves the original payload byte-for-byte.

Redaction & artifacts:
- [ ] `redaction_class` is enforced on every read path — span IO, search, archive, exports, error messages, and logs. A `Secret`/`Sensitive` payload appearing inline anywhere is a critical blocker.
- [ ] Artifact hashes (`sha256`) are computed over exactly the stored bytes; fingerprint or hash semantics changes call out cassette/dirty-detection impact.

Eval, gate & replay determinism (a wrong verdict is a shipped regression):
- [ ] Same dataset version + same config ⇒ same eval verdict; any new nondeterminism source (time, ordering, model temperature) in deterministic evals is a blocker.
- [ ] Cassette matching (`seq`, kind, `request_hash`) stays stable across serde changes — a hash computed from a re-serialized struct must not silently change for existing cassettes.
- [ ] Gate logic fails closed: an errored eval run must not count as a pass.

## Verdict & posting

Post exactly one review:

```
gh pr review <N> -R jadenfix/beater --comment --body "<body>"
```

Body format — first line is the verdict, nothing above it:

```
VERDICT: APPROVE | REQUEST-CHANGES | REJECT (superseded | wrong-approach)

<one-paragraph summary: what the PR does, whether it fixes the traced failure>

Blockers:
- <file:line — traced failure scenario>   (or "none")

Nits:
- <file:line — suggestion>                (or "none")

Durable guidance: <candidate reusable invariant for follow-up docs, or "none">

Overlap: <open PRs touching same paths + merge-order note, or "none">

— independent review agent (non-author)
```

APPROVE only with zero blockers. REQUEST-CHANGES when fixable blockers exist. REJECT when superseded by main or the approach conflicts with ARCHITECTURE.md. Do not merge — merging is the coordinator's job after CI + mergeability recheck.

## Deep mode (optional)

If asked for a "deep" review, fan out three parallel non-author subagents with distinct lenses — (a) correctness/races, (b) tenant-isolation/redaction, (c) contract-drift/over-engineering — then adversarially verify each candidate blocker yourself before posting. Only verified blockers go in the review.
