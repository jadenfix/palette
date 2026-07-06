# Bare-metal lane architecture (beaterOS)

## Objective

Build a deterministic, reusable bare-metal validation pipeline that scales from lean x86_64 hosts to CUDA-class machines while preserving safety and reproducibility.

## Control-flow graph

1. `docs/engineering/bare-metal-lane-policy.json`
   - Canonical profile DSL for each lane target.
   - Drives matrix booleans and optimization/e2e toggles.
2. `scripts/bare-metal-dispatch.sh`
   - Validates policy schema strictly.
   - Expands profile target(s) into workflow-dispatch payloads.
   - Supports `--validate-only` for fast preflight.
3. `scripts/bare-metal-run-matrix.sh`
   - Backward-compatible compatibility wrapper for humans and automation.
4. `scripts/bare-metal-lane-fleet.sh`
   - Bounded-parallel lane fan-out for subagent-style dispatch sweeps.
   - Preserves per-target logs and supports explicit worker limits.
5. `.github/workflows/bare-metal-e2e-readiness.yml`
   - Dispatch surface for matrix execution.
   - Executes:
     - readiness + class checks,
     - Gate 2 proof (`check-gate2-outside-readiness.py` + stopwatch),
     - e2e SDK conformance (`scripts/e2e-clients-live.sh`),
     - live smoke (`cargo test -p beaterd --test live_smoke`).
6. `scripts/check-bare-metal-readiness.py`
   - Emits machine-readable summary for each lane run.
   - Records lane name and check flags in the report.
7. `scripts/bare-metal-optimize-env.sh`
   - Emits language/runtime optimization hints for target host.

## Data model

- Lane profile fields (policy):
  - requirement toggles: `require_cuda`, `require_gpu`, `require_avx2`, min resource thresholds.
  - execution toggles: `run_live_smoke`, `check_gate2_readiness`, `run_gate2_compose_stopwatch`,
    `run_e2e_clients_live`, `e2e_clients_live_require`.
  - optimization toggles: `emit_bare_metal_recommendations`, `apply_bare_metal_optimizations`.
- Artifact naming: lane target + matrix fingerprint (`cuda-{bool}|gpu-{bool}|live-{bool}`).

## Optimization strategy

- CPU-class tuning:
  - high parallelism for object build jobs,
  - reduced linker contention via bounded linker worker count,
  - strict `target-cpu=native`.
- Language-stack tuning:
  - Cargo (`CARGO_BUILD_JOBS`, `RUSTFLAGS`),
  - Go (`GOMAXPROCS`, `GOFLAGS`),
  - Java/Maven (`MAVEN_OPTS`),
  - CMake (`CMAKE_BUILD_PARALLEL_LEVEL` when supplied by CI infra),
  - .NET (`DOTNET_MAX_CPU_COUNT`),
  - Make (`MAKEFLAGS`).
- Artifact-first observability:
  - readiness JSON summary,
  - optimization report,
  - Gate 2 proof/log artifacts per matrix leg.
  - post-run assertion (`scripts/bare-metal-assert-report.py`) to guarantee required fields and artifact presence.

## Progress plan and acceptance

- Add/adjust lane profiles in policy and verify with `--validate-only`.
- Dispatch at least one CPU-only and one CUDA-inclusive lane per release cycle.
- Track readiness summaries against historical baselines when reviewing performance deltas.
- Keep policy and workflow in lockstep so new fields are not silently dropped.
