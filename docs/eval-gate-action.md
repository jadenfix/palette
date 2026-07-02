# Beater Eval Gate — GitHub Action

The repo-root `action.yml` turns a Beater statistical gate into a CI check
(#154). It wraps `beaterctl gate-run` and reports a three-valued verdict —
**pass / fail / inconclusive** — to the job's step summary and, optionally,
a sticky pull-request comment.

The point of the third verdict: when a comparison is underpowered, the
comment doesn't guess. It reports the minimum detectable effect at the
current sample size and how many paired cases would have made the observed
effect conclusive (`mde` / `required_n` from `ExperimentComparison`), and the
gate's inconclusive policy decides whether that blocks the merge. An
underpowered eval is not a pass.

Everything runs against the job-local `--data-dir` SQLite store. No Beater
server, no API keys, no network: the eval steps earlier in the job produce
`gates.sqlite` / `experiments.sqlite`, and the action reads them.

## Usage

```yaml
permissions:
  contents: read
  pull-requests: write   # only needed for the sticky PR comment

jobs:
  eval-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - name: Install Rust
        run: rustup toolchain install stable --profile minimal && rustup default stable

      # ... your steps that run evals and record an experiment into .beater ...

      - name: Beater eval gate
        uses: jadenfix/beater@main
        with:
          data-dir: .beater
          tenant-id: my-tenant
          project-id: my-project
          gate-id: main
          github-token: ${{ secrets.GITHUB_TOKEN }}
```

Zero-config demo (seeds a deterministic fixture gate whose latest experiment
is a regression, so you can see the failure rendering without any setup):

```yaml
      - uses: jadenfix/beater@main
        with:
          demo-fixture: "true"
          fail-on-gate-failure: "false"   # report-only
```

## Inputs

| Input | Default | Meaning |
|---|---|---|
| `data-dir` | `.beater` | Local store with `gates.sqlite` / `experiments.sqlite`. |
| `tenant-id` / `project-id` / `gate-id` | `demo` / `demo` / `main` | Which gate to run. |
| `experiment-run-id` | latest | Gate a specific experiment run. |
| `beaterctl-path` | build from source | Prebuilt `beaterctl` binary. Without it the action builds `beaterctl` from its own checkout with cargo — correct but slow on a cold runner; prefer a prebuilt binary or a cargo cache. |
| `demo-fixture` | `false` | Seed the deterministic demo gate first (`gate-run-fixture`). |
| `fail-on-gate-failure` | `true` | Fail the step when the gate doesn't pass, so the gate can be a required check. `false` = report-only. |
| `github-token` | none | Enables the sticky PR comment (upserted in place across pushes). |
| `comment-tag` | `default` | Distinguishes comments when one PR runs several gates. |

## Outputs

| Output | Meaning |
|---|---|
| `verdict` | `pass` \| `fail` \| `inconclusive` |
| `passed` | Whether the gate passed under its inconclusive policy. |
| `decision` | Raw `GateDecision` (`pass` \| `fail_regression` \| `inconclusive`). |
| `reason` | The gate's human-readable reason string. |
| `report-path` | Raw `GateRunReport` JSON, for downstream steps. |

## Proof gate

`.github/workflows/eval-gate-action.yml` keeps the action honest on every
change to it: a renderer self-check covers all three `GateDecision` variants
from synthetic reports, and an end-to-end job runs the action against the
demo fixture and asserts `verdict=fail` / `passed=false` /
`decision=fail_regression` on the fixture's seeded regression.

## Non-goals (for now)

- No hosted-API mode; this is the local-first gate. When a `/v1` gate-run
  surface is the right shape, the action grows a `base-url` input.
- No cassette-backed judge reruns yet — that's the #152 keystone; this action
  is its delivery vehicle once the demo fixture covers judge-lane scoring.
