/**
 * Table-driven dashboard query-state mapping.
 *
 * Single source of truth for the 10 "filter" fields that appear uniformly in:
 *   - URL parsing  (parseQueryFromSearchParams)
 *   - API serialization  (applyFilterParams)
 *   - Filter chips  (filterChips)
 *   - Href link preservation  (applyFilterParams)
 *
 * Scope fields (tenant / project / environment), selection fields (trace / span),
 * and unmask fields have bespoke logic and are handled separately in each call
 * site. Adding a new filter is one entry in FILTER_FIELDS.
 */

// DashboardQuery is defined in api.ts; import as type-only to avoid a runtime
// circular dependency (api.ts imports applyFilterParams from this module at
// runtime; this module's import is erased by the TypeScript compiler).
import type { DashboardQuery } from "./api";

// ── Exported types ────────────────────────────────────────────────────────────

/** Raw shape Next.js passes to Server Components as searchParams. */
export type RawSearchParams = Record<string, string | string[] | undefined>;

/**
 * A typed descriptor for one filter field.  Each entry in FILTER_FIELDS is
 * a concrete FilterDescriptor<K> where K is the matching DashboardQuery key.
 *
 * Using a generic keeps parse / serialize / chipDisplay types consistent with
 * the actual field value type; the type is erased in the runtime array
 * (AnyDescriptor) via the `field()` factory below.
 */
export type FilterDescriptor<K extends keyof DashboardQuery> = {
  /** DashboardQuery property key */
  readonly field: K;
  /** URL search param name (same as the API query param name for filter fields) */
  readonly urlParam: string;
  /** Parse a URL param raw string → field value */
  readonly parse: (raw: string | undefined) => DashboardQuery[K] | undefined;
  /** Serialize field value → string for URL and API params */
  readonly serialize: (value: NonNullable<DashboardQuery[K]>) => string;
  /** Filter chip label (undefined → no chip for this field) */
  readonly chipLabel: string | undefined;
  /** Override chip display value (default: serialize(value)) */
  readonly chipDisplay: ((value: NonNullable<DashboardQuery[K]>) => string) | undefined;
  /** Counts toward advancedFilterCount */
  readonly advanced: boolean;
};

// ── Internal helpers ──────────────────────────────────────────────────────────

// Type-erased descriptor used in the FILTER_FIELDS heterogeneous array.
// FieldValue covers every possible non-undefined DashboardQuery value.
type FieldValue = NonNullable<DashboardQuery[keyof DashboardQuery]>;
type AnyDescriptor = {
  readonly field: keyof DashboardQuery;
  readonly urlParam: string;
  readonly parse: (raw: string | undefined) => FieldValue | undefined;
  readonly serialize: (value: FieldValue) => string;
  readonly chipLabel: string | undefined;
  readonly chipDisplay: ((value: FieldValue) => string) | undefined;
  readonly advanced: boolean;
};

/** Factory that captures per-entry generics but returns a homogeneous element. */
function field<K extends keyof DashboardQuery>(d: FilterDescriptor<K>): AnyDescriptor {
  return d as unknown as AnyDescriptor;
}

function rawString(input: string | string[] | undefined): string | undefined {
  return Array.isArray(input) ? input[0] : input;
}

function parseString(raw: string | undefined): string | undefined {
  return raw || undefined;
}

function parseNumber(raw: string | undefined): number | undefined {
  if (!raw) return undefined;
  const n = Number(raw);
  return Number.isFinite(n) ? n : undefined;
}

/** Parse "true" / "1" → true, anything else present → false, absent → undefined. */
export function parseBool(raw: string | undefined): boolean | undefined {
  if (raw === undefined || raw === "") return undefined;
  return raw === "true" || raw === "1";
}

// ── Field table ───────────────────────────────────────────────────────────────

/**
 * Table-driven definitions for the 10 filter fields.
 *
 * These fields share an important property: the URL search-param name equals
 * the API query-param name, so one descriptor drives both directions.
 *
 * Fields NOT in this table (handled with bespoke logic in each call site):
 *   Scope:     tenantId / projectId / environmentId
 *   Selection: traceId / selectedSpanId
 *   Unmask:    unmask / unmaskReason
 */
export const FILTER_FIELDS: readonly AnyDescriptor[] = [
  field({
    field: "status", urlParam: "status",
    parse: parseString, serialize: String,
    chipLabel: "Status", chipDisplay: undefined, advanced: false,
  }),
  field({
    field: "kind", urlParam: "kind",
    parse: parseString, serialize: String,
    chipLabel: "Kind", chipDisplay: undefined, advanced: false,
  }),
  field({
    field: "model", urlParam: "model",
    parse: parseString, serialize: String,
    chipLabel: "Model", chipDisplay: undefined, advanced: true,
  }),
  field({
    field: "release", urlParam: "release",
    parse: parseString, serialize: String,
    chipLabel: "Release", chipDisplay: undefined, advanced: true,
  }),
  field({
    field: "startedAfter", urlParam: "started_after",
    parse: parseString, serialize: String,
    chipLabel: "After", chipDisplay: undefined, advanced: true,
  }),
  field({
    field: "startedBefore", urlParam: "started_before",
    parse: parseString, serialize: String,
    chipLabel: "Before", chipDisplay: undefined, advanced: true,
  }),
  field({
    field: "minCostMicros", urlParam: "min_cost_micros",
    parse: parseNumber, serialize: String,
    chipLabel: "Min cost", chipDisplay: undefined, advanced: true,
  }),
  field({
    field: "maxCostMicros", urlParam: "max_cost_micros",
    parse: parseNumber, serialize: String,
    chipLabel: "Max cost", chipDisplay: undefined, advanced: true,
  }),
  field({
    field: "minLatencyMs", urlParam: "min_latency_ms",
    parse: parseNumber, serialize: String,
    chipLabel: "Min latency", chipDisplay: (v) => `${v} ms`, advanced: true,
  }),
  field({
    field: "maxLatencyMs", urlParam: "max_latency_ms",
    parse: parseNumber, serialize: String,
    chipLabel: "Max latency", chipDisplay: (v) => `${v} ms`, advanced: true,
  }),
];

// ── Public utilities ──────────────────────────────────────────────────────────

/**
 * Parse a Next.js searchParams record into a DashboardQuery.
 *
 * Scope fields use named URL params with hardcoded defaults (tenant/project/
 * environment). Selection and unmask fields are parsed directly. Filter fields
 * are driven by FILTER_FIELDS, preserving exact URL-param names and parse rules.
 */
export function parseQueryFromSearchParams(params: RawSearchParams): DashboardQuery {
  const get = (key: string): string | undefined => rawString(params[key]);

  const query: DashboardQuery = {
    tenantId: get("tenant") ?? "demo",
    projectId: get("project") ?? "demo",
    environmentId: get("environment") ?? "local",
    traceId: get("trace") || undefined,
    selectedSpanId: get("span") || undefined,
    unmask: parseBool(get("unmask")),
    unmaskReason: get("reason") || undefined,
  };

  for (const d of FILTER_FIELDS) {
    const raw = get(d.urlParam);
    const val = d.parse(raw);
    if (val !== undefined) {
      (query as Record<string, unknown>)[d.field] = val;
    }
  }

  return query;
}

/**
 * Derive active filter chips for display in the toolbar.
 * traceId gets a short-hash display; filter fields are table-driven.
 */
export function filterChips(query: DashboardQuery): { label: string; value: string }[] {
  const chips: { label: string; value: string }[] = [];
  if (query.traceId) chips.push({ label: "Trace", value: shortHash(query.traceId) });
  for (const d of FILTER_FIELDS) {
    const val: FieldValue | undefined = query[d.field] as FieldValue | undefined;
    if (val !== undefined && d.chipLabel !== undefined) {
      const display = d.chipDisplay ? d.chipDisplay(val) : d.serialize(val);
      chips.push({ label: d.chipLabel, value: display });
    }
  }
  return chips;
}

/** Count active "advanced" filter fields (drives the disclosure indicator). */
export function advancedFilterCount(query: DashboardQuery): number {
  let count = 0;
  for (const d of FILTER_FIELDS) {
    if (!d.advanced) continue;
    const val = query[d.field];
    if (val !== undefined && val !== "") count++;
  }
  return count;
}

/**
 * Append filter-field params to a URLSearchParams instance.
 *
 * Used by both applyFilterParams in hrefFor (link preservation) and
 * searchParamsForTraceList in api.ts (API serialization). Only touches the 10
 * filter fields; scope, selection, and unmask params must be set by the caller.
 */
export function applyFilterParams(query: DashboardQuery, params: URLSearchParams): void {
  for (const d of FILTER_FIELDS) {
    const val: FieldValue | undefined = query[d.field] as FieldValue | undefined;
    if (val !== undefined) {
      params.set(d.urlParam, d.serialize(val));
    }
  }
}

/** Truncate a long hash ID for display (trace IDs, span IDs, sha256 refs). */
export function shortHash(value: string): string {
  return value.length > 18 ? `${value.slice(0, 12)}...${value.slice(-6)}` : value;
}
