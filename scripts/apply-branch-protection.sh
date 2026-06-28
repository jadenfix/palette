#!/usr/bin/env bash
# apply-branch-protection.sh
#
# One-time admin script: apply the branch-protection ruleset documented in
# docs/engineering/merge-queue-policy.md to the main branch of this repo.
#
# Prerequisites:
#   - gh CLI authenticated as a repo admin (gh auth login)
#   - jq installed
#
# What this does (see docs/engineering/merge-queue-policy.md for full rationale):
#   1. Marks the six fast CI checks as REQUIRED status checks on main.
#   2. Enables "Require branches to be up to date before merging" (strict mode).
#   3. Enables the GitHub merge queue on main (the recommended option — renders
#      step 2 redundant but both are safe to have together).
#   4. Disallows force-pushes and branch deletion on main.
#   5. Requires at least one approving review.
#
# The heavy/Docker gates (live-smoke, gate2-*, storage-backends, container-images)
# are left advisory: they run on every PR but do NOT block merge.  They are
# included in the merge queue so the queue sees their result, but a failure does
# not hard-stop the queue (set merge_method below if you want them to block).
#
# Run:
#   bash scripts/apply-branch-protection.sh
#
# To preview the current ruleset before and after:
#   gh api repos/jadenfix/beater/branches/main/protection | jq .

set -euo pipefail

REPO="jadenfix/beater"
BRANCH="main"

echo "Applying branch-protection ruleset for ${REPO}@${BRANCH} …"

# ---------------------------------------------------------------------------
# 1. Branch protection rule via the REST API
#    (Covers: required checks, strict up-to-date, no force-push, review req.)
# ---------------------------------------------------------------------------
gh api \
  --method PUT \
  "/repos/${REPO}/branches/${BRANCH}/protection" \
  --header "Accept: application/vnd.github+json" \
  --input - <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "backend tests",
      "backend lint",
      "contract in sync (semconv + 7 SDKs + additive-only)",
      "frontend tests",
      "frontend lint",
      "validate"
    ]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": false,
    "require_code_owner_reviews": false,
    "required_approving_review_count": 1
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON

echo "Branch protection applied."

# ---------------------------------------------------------------------------
# 2. Enable the GitHub merge queue (Rulesets API — requires GitHub Enterprise
#    Cloud or public repos on GitHub.com with merge queue enabled in settings).
#
#    If the Rulesets API returns 404, your plan does not include merge queues;
#    fall back to the strict "up to date" setting applied in step 1 above,
#    which already prevents the two-green-PR failure class (with a small race
#    window).  Enable merge queue manually in:
#      Settings → Branches → Edit rule → "Require merge queue"
# ---------------------------------------------------------------------------
echo ""
echo "Attempting to enable merge queue via Rulesets API…"

RULESET_PAYLOAD=$(cat <<'JSON'
{
  "name": "main-merge-queue",
  "target": "branch",
  "enforcement": "active",
  "conditions": {
    "ref_name": {
      "include": ["refs/heads/main"],
      "exclude": []
    }
  },
  "rules": [
    {
      "type": "merge_queue",
      "parameters": {
        "check_response_timeout_minutes": 60,
        "entry_by_passage_of_time": false,
        "entry_delay_minutes": 0,
        "grouping_strategy": "ALLGREEN",
        "max_entries_to_build": 5,
        "max_entries_to_merge": 1,
        "merge_method": "SQUASH",
        "min_entries_to_merge": 1,
        "min_entries_to_merge_wait_minutes": 0
      }
    }
  ]
}
JSON
)

if gh api \
     --method POST \
     "/repos/${REPO}/rulesets" \
     --header "Accept: application/vnd.github+json" \
     --input - <<< "${RULESET_PAYLOAD}" 2>/dev/null; then
  echo "Merge queue ruleset created."
else
  echo ""
  echo "WARNING: Rulesets API returned an error (plan may not support merge"
  echo "queues, or a ruleset already exists).  The strict up-to-date setting"
  echo "applied in step 1 still prevents the two-green-PR breakage class."
  echo ""
  echo "To enable the merge queue manually:"
  echo "  GitHub UI → Settings → Branches → Edit protection rule for main"
  echo "            → Check 'Require merge queue'"
  echo ""
  echo "See docs/engineering/merge-queue-policy.md for the full options."
fi

echo ""
echo "Done. Verify with:"
echo "  gh api repos/${REPO}/branches/${BRANCH}/protection | jq '.required_status_checks'"
