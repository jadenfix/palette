# Bare-metal Lane Operations Map

This document is the operating map for the coding agent on bare-metal readiness and optimization work.

## 1. Scope of this lane

Goal: turn hardware-readiness execution into a repeatable, policy-driven workflow that optimizes performance checks without regressing safety gates.

- `docs/engineering/bare-metal-lane-policy.json` defines lane profiles and execution policy.
- `scripts/bare-metal-dispatch.sh` is the policy-to-workflow dispatcher.
- `.github/workflows/bare-metal-e2e-readiness.yml` executes readiness, smoke, Gate2 stopwatch, and language conformance checks.
- `scripts/check-bare-metal-readiness.py` performs local host capability checks and emits a machine-readable summary.
- `scripts/bare-metal-optimize-env.sh` emits deterministic build/runtime tuning recommendations.
- `scripts/bare-metal-run-matrix.sh` is the compatibility wrapper over the dispatcher.
- `scripts/bare-metal-lane-fleet.sh` is the bounded-parallel fleet launcher for multi-target lane rollouts.

## 2. Skills matrix for the agent

### Required engineering disciplines

- Systems engineering: host hardware profiling, concurrency-safe CI orchestration, failure-mode control, and resource caps.
- PR review: independent review of runtime/control-plane and security-sensitive edits.
- Documentation and reproducibility: every lane change records rationale, expected tradeoffs, and evidence artifacts.
- Refactoring discipline: remove duplicated path logic and repeated argument construction only when behavior is preserved by tests/artifacts.

### Repetitive tasks this lane should automate

- Policy updates (new target profiles or check toggles).
- Workflow reruns with lane-specific matrix variations.
- Readiness report triage using JSON summary artifacts.
- Optimization log collection (`bare-metal-optimize-env.txt`) and artifact retention.
- Subagent-style parallel lane dispatch (`--workers`) with bounded concurrency and per-target logging.

## 3. Language + framework + algorithm choices

- Policy/orchestration: Python for `scripts/bare-metal-dispatch.sh` and `scripts/check-bare-metal-readiness.py`.
  - Reason: schema typing, JSON handling, and subprocess orchestration are safer with explicit argument validation.
- Workflow glue and CLI shims: Bash for shell-native GH runners and deterministic env exports.
- Readiness checks and proof runs: existing Rust, Python, and shell tools; keep each in its current stable language to avoid cross-tool semantics drift.
- Matrix algorithm: explicit policy expansion (`targets[]`) + deterministic input projection + workflow_dispatch matrix over boolean axes.

### Performance and robustness policy for this lane

- Always collect optimization hints before running heavy proof checks.
- Use matrix expansion only for orthogonal gate switches.
- Keep local command and artifact paths unique by lane target.
- Fail-fast only on structural violations; runtime checks return structured artifacts for postmortem analysis.

## 4. repo map (code references)

- `scripts/bare-metal-dispatch.sh`
- `scripts/bare-metal-run-matrix.sh`
- `scripts/bare-metal-lane-fleet.sh`
- `scripts/check-bare-metal-readiness.py`
- `scripts/bare-metal-optimize-env.sh`
- `scripts/bare-metal-assert-report.py`
- `scripts/bare-metal-pr-helper.sh`
- `.github/workflows/bare-metal-e2e-readiness.yml`
- `docs/engineering/bare-metal-lane-policy.json`
- `docs/engineering/bare-metal-review-and-release.md`

## 5. Related architecture map

- [docs/engineering/bare-metal-system-architecture.md](/Users/jadenfix/beater/docs/engineering/bare-metal-system-architecture.md)

## 6. Exit criteria for merge-readiness lane work

- New/updated profiles are policy-driven and documented in the JSON policy file.
- Dispatch command and workflow inputs are deterministic across runs.
- Review lane receives independent reviewer sign-off via checklist completion.
- Artifacts are uploaded with lane and matrix isolation.
- Summary + optimization outputs are present for every executed run.

## 7. Practical commands

```bash
./scripts/bare-metal-dispatch.sh --list
python3 scripts/bare-metal-dispatch.sh --dry-run --target cuda
bash scripts/bare-metal-run-matrix.sh --list --json
bash scripts/bare-metal-run-matrix.sh --dry-run --target gpu-lean
bash scripts/bare-metal-lane-fleet.sh --all --workers 3 --dry-run
bash scripts/bare-metal-lane-fleet.sh --target cuda --target cuda-pro
bash scripts/bare-metal-pr-helper.sh --title "[bare-metal] lane hardening" --reviewer reviewer-handle --dry-run
python3 scripts/bare-metal-dispatch.sh --validate-only
python3 scripts/bare-metal-dispatch.sh --dry-run --target-index 2
python3 scripts/bare-metal-assert-report.py --summary <path> --lane-target-name <name>
``` 

Notes:
- `bare-metal-dispatch.sh` is the primary policy expansion/orchestration path.
- `bare-metal-run-matrix.sh` is the stable wrapper for existing callers.
- `bare-metal-lane-fleet.sh` is the parallel dispatch path for larger lane sweeps and review-agent splits.
