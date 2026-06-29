import { cookies } from "next/headers";

import { dashboardApiBaseUrl, dashboardApiHeaders } from "./api";
import { SESSION_COOKIE } from "./auth";
import type { components } from "./generated/api-types";

export type UsageSummary = components["schemas"]["UsageSummary"];
export type UsageTotal = components["schemas"]["UsageTotal"];

/**
 * Read the per-project usage summary from beaterd (`GET /v1/usage/{tenant}/{project}`),
 * forwarding the user's session cookie plus any server-side API credentials.
 * Never throws — returns `{ summary: null, error }` so the page can render a
 * graceful empty/local-mode state instead of crashing.
 */
export async function fetchUsageSummary(
  tenantId: string,
  projectId = "default",
): Promise<{ summary: UsageSummary | null; error: string | null }> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  const base = dashboardApiBaseUrl();
  const headers: Record<string, string> = {
    ...(dashboardApiHeaders({ tenantId, projectId }) as Record<string, string>),
  };
  if (token) headers.cookie = `${SESSION_COOKIE}=${token}`;

  try {
    const res = await fetch(
      `${base}/v1/usage/${encodeURIComponent(tenantId)}/${encodeURIComponent(projectId)}`,
      { headers, cache: "no-store" },
    );
    if (!res.ok) return { summary: null, error: `API ${res.status}` };
    return { summary: (await res.json()) as UsageSummary, error: null };
  } catch {
    return { summary: null, error: "unreachable" };
  }
}

/** Humanize a usage metric key like `written_spans` → `Written spans`. */
export function humanizeMetric(key: string): string {
  const spaced = key.replace(/[_.]/g, " ").trim();
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}

/** Format a quantity for display, special-casing byte units. */
export function formatQuantity(quantity: number, unit: string): string {
  if (/byte/i.test(unit)) return formatBytes(quantity);
  return new Intl.NumberFormat("en-US").format(quantity);
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i += 1;
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[i]}`;
}
