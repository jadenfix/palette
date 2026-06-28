# Merge Queue & Required-Up-To-Date Policy

**Status:** Enforced via branch-protection rule on `main`.  
**Applied by:** `scripts/apply-branch-protection.sh` (run once by an admin).  
**Related issue:** #343

---

## Why this policy exists: the two-green-PR breakage class

`main` can go non-compiling even when every contributing PR passes CI
individually. Here is the exact failure sequence that happened twice:

1. A shared type gains new fields (e.g. `ExperimentComparison` gains `mde` and
   `required_n`).
2. Two separate PRs each update the same set of test fixtures to add those
   fields, but they insert them at **different positions** in the struct literal
   (one after `p_value`, one after `adjusted_alpha`).
3. Both PRs branch from the same `main` and both pass CI — neither PR has seen
   the other's change.
4. Both PRs merge (squash-merge). Git sees no conflict markers because the
   additions are at nearby-but-distinct lines.
5. `main` now declares each field twice (`error[E0062]: field specified more
   than once`) and does not compile.  Every open PR goes red through no fault
   of its own.

This is a **semantic merge conflict**: CI on a stale branch cannot detect a
change that landed on `main` after the branch's last CI run.

A second variant: a PR removes or renames a field while another PR adds struct
literals that assume the old field name exists (`error[E0063]: missing field`).
Same root cause, same fix.

---

## The fix: two complementary layers

### Layer 1 — `cargo build --workspace` in CI (code-level, already in repo)

The backend workflow (`.github/workflows/backend.yml`) runs `cargo build
--workspace` **before** `cargo test`. Because the test step excludes several
crates (e.g. `beaterd`) to avoid duplicated work, a compile error in one of
those crates would not be caught by `cargo test` alone. The explicit build step
ensures the entire workspace is type-checked on every PR push, regardless of
which crates are excluded from testing.

This catches E0062/E0063 and similar struct-field errors in ALL crates as soon
as the PR branch is pushed.

### Layer 2 — Merge queue / require-up-to-date (process-level, admin action)

Even with Layer 1, a stale PR branch may compile fine on its own but fail when
combined with another PR that landed on `main` while CI was running. The GitHub
merge queue (preferred) or the "Require branches to be up to date" branch
protection option (fallback) closes this window.

**GitHub merge queue (recommended):**  
Each PR enters a virtual queue. GitHub synthesises a temporary branch that
stacks the PR on top of current `main` (and on top of other queued PRs ahead of
it), reruns all required CI checks against that synthetic branch, and only merges
if all checks pass. No two PRs ever merge without having been tested together.

**"Require branches to be up to date" (fallback):**  
A PR is blocked from merging unless its branch is up to date with `main`. This
forces an explicit rebase/merge-up before the Merge button appears. It closes
most of the window but still has a small race between the final CI run and the
merge click; the merge queue eliminates this race entirely.

---

## Required status checks on `main`

The following CI jobs are marked **required** — a PR cannot merge until all of
them pass on a branch that is up to date with `main`:

| Check name (as GitHub sees it) | Workflow file | What it guards |
|---|---|---|
| `backend tests` | `backend.yml` | `cargo build --workspace` + `cargo test --workspace` (all crates) |
| `backend lint` | `backend.yml` | `cargo fmt --check` + `cargo clippy -D warnings` |
| `contract in sync (semconv + 7 SDKs + additive-only)` | `sdk-contract.yml` | spec ↔ routes ↔ 7 SDK clients ↔ semconv, zero drift |
| `frontend tests` | `frontend.yml` | dashboard build + test suite |
| `frontend lint` | `frontend.yml` | TypeScript type-check + generated-client drift check |
| `validate` | (if present) | any project-level validation gate |

**Advisory gates** (run on every PR but do not block merge):  
`live-smoke`, `gate2-browser-proof`, `gate2-proof-contract`, `storage-backends`,
`container-images` — these require Docker / live infra and would slow the merge
queue unacceptably if required. They run as normal CI jobs; a consistent failure
there is a signal to investigate but is not a hard merge blocker.

---

## How to enable (admin steps)

The `scripts/apply-branch-protection.sh` script encodes all of the above as `gh
api` calls. An admin with write access to the repo settings runs it once:

```bash
# Authenticate as an admin first
gh auth login

# Apply the ruleset
bash scripts/apply-branch-protection.sh
```

The script:
1. Sets the six required status checks (with `strict: true` — up-to-date
   branch required).
2. Attempts to create a merge queue ruleset via the GitHub Rulesets API.
3. Falls back gracefully with manual instructions if merge queues are not
   available on the current plan.

To enable the merge queue manually via the UI:  
**GitHub → Settings → Branches → Edit rule for `main` → "Require merge queue"**

---

## Verifying the configuration

```bash
# Show current branch protection for main
gh api repos/jadenfix/beater/branches/main/protection | jq '.required_status_checks'

# Show current rulesets (merge queue)
gh api repos/jadenfix/beater/rulesets | jq '.[].name'
```

---

## Notes for contributors

You do not need to do anything differently day-to-day. When your PR is ready:

1. Click "Merge" (or "Add to merge queue" if the merge queue is enabled).
2. GitHub will rerun required checks against the current tip of `main`.
3. If something that was green on your branch is now red against `main`, update
   your branch (`git fetch origin main && git rebase origin/main`) and push
   again.

This extra CI round is the cost of preventing a broken `main`.
