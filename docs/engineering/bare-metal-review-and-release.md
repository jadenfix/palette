# Bare-metal/e2e review and release operations

This document defines the review and release workflow for PRs that affect the
bare-metal lane, including any PRs that alter e2e hardware execution,
benchmarks, or release artifacts for physical targets.

## 1. Separation of responsibilities

Implementation PR authors must not review their own PR.

1. The PR author owns implementation work in the PR body under `1.) agent` and
   `2.) purpose` as usual.
2. The PR body must declare a distinct reviewer in
   `3.) reviewed-by: @<github handle>`.
3. Review lane responsibility is owned by the declared `reviewed-by` handle and
   must be independent from the implementation author.
4. Merge is allowed only after that reviewer approves the current HEAD commit.

## 2. Bare-metal/e2e PR merge checklist

Use this section in bare-metal/e2e PRs and keep it fully checked before merge.

### 2.1 Hardware policy

- [ ] Target platform and exact hardware class recorded (example: board/revision
  or machine model + serial/asset tag).
- [ ] Firmware/toolchain baseline recorded for the test run.
- [ ] Required hardware constraints documented:
  - secure boot / provisioning state,
  - networking mode,
  - environment assumptions,
  - destructive operations or resets required.
- [ ] Reviewer confirms hardware policy does not conflict with active runbook
  constraints for the target lane.

### 2.2 e2e artifacts

- [ ] Full e2e artifact bundle attached in PR body (logs + summary + raw
  outputs).
- [ ] Artifacts include:
  - execution logs,
  - console/serial logs for hardware sessions (if applicable),
  - screenshots/videos or equivalent capture proving observable behavior,
  - test manifests, environment manifests, and failure reproduction context.
- [ ] The reviewer can reproduce from artifact links alone without private local
  knowledge.
- [ ] All artifact links are immutable and scoped to this PR commit/sha where
  possible.
- [ ] Bare-metal optimization profile is recorded when applied (`bare-metal-optimize-env.txt`)
  and environment decisions are reviewed.

### 2.3 Gate proof

- [ ] PR body includes a non-self reviewer declaration that matches a real
  approval on the HEAD commit.
- [ ] Required branch protection and platform gates are green for this head SHA.
- [ ] Non-blocking failures are explicitly documented with a remediation plan and
  explicit owner.
- [ ] A reviewer confirms there are no open review-action gaps.

### 2.4 Benchmark notes

- [ ] Pre-change benchmark baseline and post-change results are included.
- [ ] Resource/latency claims are tied to a specific run (artifact or log path +
  timestamp).
- [ ] Regression analysis is included for changed metrics (or explicit "no
  change" rationale).
- [ ] Reviewer signs off that benchmark notes cover impacted bare-metal/e2e critical
  paths (boot, actuation, observation, and teardown).

## 3. Review responsibilities and merge sign-off

For bare-metal/e2e lanes, the merge decision requires both:

- Implementation PR author: code + evidence completeness.
- Declared review owner: independent review sign-off on all four sections above.

If either party identifies uncertainty, the PR must stay open until:

- uncertainty is resolved in-thread with linked evidence, or
- a follow-up PR is created and explicitly linked before merge.

## 4. Reviewer handoff workflow (codex + human review split)

Use this process when the implementation PR is created by the coding agent:

- Set the PR body `3.) reviewed-by` to the designated reviewer before opening.
- Use the checker helper to generate the template body and include checklist artifacts:

```bash
bash scripts/bare-metal-pr-helper.sh \
  --title "[bare-metal] ..." \
  --reviewer <github_handle> \
  --plan-file docs/engineering/bare-metal-operations-map.md
```

- Keep code-review and merge ownership separate from implementation author.
- Require explicit approval from the declared reviewer before merge.

## 5. Copy/paste checklist block

Suggested PR-body section (paste as-is and fill):

```text
## Bare-metal/e2e merge checklist
- [ ] Reviewed-by: @<github handle>

### Hardware policy
- [ ] Platform/revision: 
- [ ] Firmware/tooling baseline:
- [ ] Constraints and assumptions:

### e2e artifacts
- [ ] Artifact bundle:
- [ ] Logs:
- [ ] Screenshots/captures:
- [ ] Repro notes:

### Gate proof
- [ ] non-self-review check passed on HEAD:
- [ ] Required gates passed:
- [ ] Open follow-ups:

### Benchmark notes
- [ ] Baseline:
- [ ] Post-change results:
- [ ] Regression/impact summary:
```
