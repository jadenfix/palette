/**
 * Static route smoke manifest for the Beater dashboard (Soundstage, §25).
 *
 * What this file checks (all statically — no server, no build):
 *   1. Every leaf directory under app/ (excluding app/api/) has a page.tsx —
 *      no orphan route folders that create dead URL segments.
 *   2. Every EXPECTED_ROUTES entry (routes confirmed to exist on main) resolves
 *      to a page.tsx at the expected path.
 *   3. PENDING_ROUTES (documented in §25.4 but not yet on main — blocked on
 *      read-API work, §20.2/§20.4) are logged as informational and do NOT fail
 *      the test. When a pending route ships, move it into EXPECTED_ROUTES and
 *      it will automatically be enforced on every subsequent run.
 *
 * Route discrepancies vs §25.4 as of the commit this test was added:
 *   - /settings    → §25.4 says [partial/built] but app/settings/page.tsx is
 *                    absent on main (settings/ is a namespace dir; api-keys page
 *                    lives at /settings/api-keys). Added to PENDING_ROUTES with a
 *                    note so it is promoted when the settings landing page ships.
 *   All other documented-as-planned routes (§25.4) are in PENDING_ROUTES.
 */

import assert from "node:assert/strict";
import { existsSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

const dashboardRoot = new URL("..", import.meta.url).pathname;
const appDir = join(dashboardRoot, "app");

// ---------------------------------------------------------------------------
// Routes confirmed to exist on main (checked via git ls-tree origin/main).
// Each entry is relative to app/ and must resolve to page.tsx.
// ---------------------------------------------------------------------------
const EXPECTED_ROUTES = [
  { route: "/", file: "page.tsx", note: "Traces — §25.4 [built]" },
  { route: "/login", file: "login/page.tsx", note: "Login — §25.4 [built]" },
  {
    route: "/connect",
    file: "connect/page.tsx",
    note: "Coding-agent connect screen — §25.4 [built]",
  },
  {
    route: "/settings/api-keys",
    file: "settings/api-keys/page.tsx",
    note: "API key management — §25.4 [partial, built]",
  },
  {
    route: "/docs",
    file: "docs/page.tsx",
    note: "In-app API/MCP docs (Scalar) — §25.4 [built]",
  },
  {
    route: "/docs/mcp",
    file: "docs/mcp/page.tsx",
    note: "MCP reference docs — §25.4 [built]",
  },
  {
    route: "/docs/quickstarts",
    file: "docs/quickstarts/page.tsx",
    note: "Quickstart guides — §25.4 [built]",
  },
  {
    route: "/search",
    file: "search/page.tsx",
    note: "Crate Dig — §25.4 [partial, built against /v1/search/:tenant/spans]",
  },
];

// ---------------------------------------------------------------------------
// Routes documented in §25.4 but NOT yet on main — blocked on read-API work
// (§20.2/§20.4) or other planned work. These are reported but never fail.
// Promote an entry to EXPECTED_ROUTES once the route ships to main.
// ---------------------------------------------------------------------------
const PENDING_ROUTES = [
  {
    route: "/settings",
    file: "settings/page.tsx",
    note: "Settings landing page — §25.4 [partial]; settings/ is currently a namespace dir only (api-keys child exists)",
  },
  {
    route: "/sessions",
    file: "sessions/page.tsx",
    note: "Sessions — §25.4 [needs read-API: /v1/sessions, §20.3 #1.1]",
  },
  {
    route: "/datasets",
    file: "datasets/page.tsx",
    note: "Encore datasets — §25.4 [needs read-API: GET /v1/datasets, §20.4 #2.1]",
  },
  {
    route: "/experiments/[id]",
    file: "experiments/[id]/page.tsx",
    note: "Beatboxing A/B — §25.4 [needs read-API: §20.4 #2.3]",
  },
  {
    route: "/evals/[id]",
    file: "evals/[id]/page.tsx",
    note: "Backbeat drilldown — §25.4 [needs read-API: §20.4 #2.2]",
  },
  {
    route: "/analytics",
    file: "analytics/page.tsx",
    note: "Tempo/Heartbeat analytics — §25.4 [needs read-API: GET /v1/metrics/:tenant, §20.4 #2.7]",
  },
  {
    route: "/prompts",
    file: "prompts/page.tsx",
    note: "Mixdown prompt registry — §25.4 [needs read-API: /v1/prompts, §20.6 #4.7]",
  },
  {
    route: "/review",
    file: "review/page.tsx",
    note: "Setlist review queue — §25.4 [planned; backend built, UI not]",
  },
  {
    route: "/diff",
    file: "diff/page.tsx",
    note: "Rewind diff — §25.4 [needs read-API: GET /v1/traces/:tenant/:a/diff/:b, §20.4 #2.6]",
  },
  {
    route: "/studio",
    file: "studio/page.tsx",
    note: "Agent Studio — §25.4 [planned; depends on §11 forked-replay + §21 simulate]",
  },
  {
    route: "/evolution",
    file: "evolution/page.tsx",
    note: "Beatboxing RSI — §25.4 [planned; depends on §21 RSI]",
  },
];

// ---------------------------------------------------------------------------
// Helper: recursively collect leaf directories (no child dirs) under a path,
// excluding the api/ subtree (those use route.ts, not page.tsx).
// ---------------------------------------------------------------------------
function leafDirs(dir, relativeTo = dir) {
  const results = [];
  const entries = readdirSync(dir, { withFileTypes: true });
  const childDirs = entries.filter(
    (e) => e.isDirectory() && e.name !== "api"
  );
  if (childDirs.length === 0) {
    // This is a leaf directory (no non-api child directories)
    results.push(dir);
  } else {
    for (const childDir of childDirs) {
      results.push(...leafDirs(join(dir, childDir.name), relativeTo));
    }
  }
  return results;
}

// ---------------------------------------------------------------------------
// Test 1: every EXPECTED route has its page.tsx on disk.
// ---------------------------------------------------------------------------
test("expected routes have page.tsx on main", () => {
  const missing = [];
  for (const { route, file, note } of EXPECTED_ROUTES) {
    const fullPath = join(appDir, file);
    if (!existsSync(fullPath)) {
      missing.push({ route, file, note });
    }
  }
  if (missing.length > 0) {
    const list = missing
      .map((m) => `  ${m.route}  →  app/${m.file}  (${m.note})`)
      .join("\n");
    assert.fail(`Expected page.tsx files are missing on main:\n${list}`);
  }
});

// ---------------------------------------------------------------------------
// Test 2: no orphan leaf directories under app/ (excluding api/).
//
// An "orphan" is a leaf route directory that has neither page.tsx nor route.ts.
// In Next.js App Router every leaf route segment must export at least one of
// those; a folder with neither is a dead stub.
// ---------------------------------------------------------------------------
test("no orphan leaf route directories under app/", () => {
  const orphans = [];
  for (const leafDir of leafDirs(appDir)) {
    const relPath = leafDir.slice(appDir.length + 1); // relative to app/
    const hasPage = existsSync(join(leafDir, "page.tsx"));
    const hasRoute = existsSync(join(leafDir, "route.ts"));
    if (!hasPage && !hasRoute) {
      orphans.push(relPath || "(app root)");
    }
  }
  if (orphans.length > 0) {
    assert.fail(
      `Orphan route directories (no page.tsx or route.ts):\n${orphans.map((d) => `  app/${d}`).join("\n")}`
    );
  }
});

// ---------------------------------------------------------------------------
// Test 3: pending routes — logged but never fail.
//
// This test always passes. It reports which §25.4-documented routes are still
// absent from main, so the gap is visible in CI output without breaking the
// build. Move an entry into EXPECTED_ROUTES once the route ships.
// ---------------------------------------------------------------------------
test("pending routes (documented in §25 but not on main — informational only)", () => {
  const present = [];
  const absent = [];
  for (const { route, file, note } of PENDING_ROUTES) {
    const fullPath = join(appDir, file);
    if (existsSync(fullPath)) {
      present.push({ route, file, note });
    } else {
      absent.push({ route, note });
    }
  }

  if (present.length > 0) {
    // A pending route has appeared on this branch — great, but it should be
    // promoted to EXPECTED_ROUTES so future runs enforce it.
    console.log(
      `[routes] NOTE: ${present.length} pending route(s) now exist on this branch — ` +
        `consider promoting to EXPECTED_ROUTES:`
    );
    for (const { route, note } of present) {
      console.log(`  [PROMOTE] ${route}  (${note})`);
    }
  }

  if (absent.length > 0) {
    console.log(
      `[routes] Pending routes not yet on main (§25.4, expected — not failures):`
    );
    for (const { route, note } of absent) {
      console.log(`  [pending] ${route}  —  ${note}`);
    }
  }

  // Intentionally always passes — the information above is the signal.
  assert.ok(true, "pending routes check is informational only");
});
