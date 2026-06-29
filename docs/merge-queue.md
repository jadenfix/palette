# Merge queue — keeping `main` green (#343)

`main` can go non-compiling even when **every** contributing PR passed CI
individually: two PRs branch from the same `main`, each is green against *that*
base, but their merged result is broken (a classic example: two PRs each add the
same struct field in a different position, or one renames a symbol the other
starts using). CI on the PR branch never tested the combination.

A merge queue closes this. Instead of merging a PR's branch directly, GitHub
enqueues it, creates an ephemeral `gh-readonly-queue/main/<sha>` branch
containing **the PR's changes rebased on the current tip plus any PRs ahead of it
in the queue**, runs the required checks against *that*, and only fast-forwards
`main` if they pass. The combination is always tested before it lands.

## What this repo ships (in code)

Every required, always-on CI gate now also triggers on the `merge_group` event,
so it runs against the queue's merged-result branch:

- `backend.yml` — workspace tests + clippy/fmt (the gate that caught the original
  #343 break)
- `sdk-contract.yml` — zero-drift contract gate (spec + 7 SDKs + semconv)
- `frontend.yml` — dashboard tests/build + generated-client drift
- `browser.yml` — browser-agent suite

`merge_group` does not support `paths:`/`branches-ignore:` filters, so the
path-scoped workflows (`storage-backends.yml`, `action-pins.yml`) are
intentionally left PR-only; they are not part of the required merge-queue gate.

## What a maintainer must enable (repo settings — not expressible in YAML)

These are GitHub branch-protection / repo settings and must be set once by an
admin (Settings → Branches → `main`, or via the API):

1. **Enable the merge queue** for `main` (Branch protection rule → *Require merge
   queue*). Suggested settings: merge method **Squash**, build concurrency **5**,
   only merge if checks pass, group size 1–5.
2. **Require status checks to pass before merging**, and mark these as required:
   `backend tests`, `backend lint`, `contract in sync (semconv + 7 SDKs +
   additive-only)`, `frontend tests`, `frontend lint`, `browser tests`.
3. **Require branches to be up to date before merging** — the belt-and-suspenders
   companion to the queue; with the queue on, GitHub enforces the up-to-date
   merged result automatically.

Once enabled, PRs are merged via **Merge when ready** (the queue), never the
plain Merge button.
