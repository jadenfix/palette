import type { components, operations } from "./generated/api-types";

export type Page<T> = {
  items: T[];
  next_cursor: string | null;
};

type TraceListOperation = operations["openapi_list_traces"];
type TraceOperation = operations["openapi_get_trace"];
type SpanIoOperation = operations["openapi_get_span_io"];
type TraceListQuery = NonNullable<TraceListOperation["parameters"]["query"]>;
type TraceListPathParams = TraceListOperation["parameters"]["path"];
type TracePathParams = TraceOperation["parameters"]["path"];
type SpanIoPathParams = SpanIoOperation["parameters"]["path"];

export type RunSummary = components["schemas"]["RunSummaryDoc"];
export type Money = components["schemas"]["MoneyDoc"];
export type CanonicalSpan = components["schemas"]["CanonicalSpanDoc"];
export type TraceView = components["schemas"]["TraceViewDoc"];
export type SpanIoResponse = components["schemas"]["SpanIoResponseDoc"];

export type DashboardQuery = {
  tenantId: string;
  projectId?: string;
  environmentId?: string;
  traceId?: string;
  selectedSpanId?: string;
  status?: TraceListQuery["status"];
  kind?: TraceListQuery["kind"];
  startedAfter?: TraceListQuery["started_after"];
  startedBefore?: TraceListQuery["started_before"];
  model?: TraceListQuery["model"];
  release?: TraceListQuery["release"];
  minCostMicros?: TraceListQuery["min_cost_micros"];
  maxCostMicros?: TraceListQuery["max_cost_micros"];
  minLatencyMs?: TraceListQuery["min_latency_ms"];
  maxLatencyMs?: TraceListQuery["max_latency_ms"];
};

export type DashboardData = {
  apiBaseUrl: string;
  query: DashboardQuery;
  runs: Page<RunSummary>;
  trace: TraceView | null;
  selectedSpan: CanonicalSpan | null;
  selectedIo: SpanIoResponse | null;
  error: string | null;
};

export function dashboardApiBaseUrl(): string {
  return (
    process.env.BEATER_API_BASE_URL ??
    process.env.NEXT_PUBLIC_BEATER_API_BASE_URL ??
    "http://127.0.0.1:8080"
  ).replace(/\/$/, "");
}

export function dashboardApiHeaders(query: DashboardQuery): HeadersInit {
  const headers: Record<string, string> = {};
  const bearerToken = process.env.BEATER_API_TOKEN ?? process.env.BEATER_API_BEARER_TOKEN;
  const apiKey = process.env.BEATER_API_KEY;
  if (bearerToken) {
    headers.authorization = bearerToken.startsWith("Bearer ")
      ? bearerToken
      : `Bearer ${bearerToken}`;
  } else if (apiKey) {
    headers["x-beater-api-key"] = apiKey;
  }
  if (query.projectId) headers["x-beater-project-id"] = query.projectId;
  if (query.environmentId) headers["x-beater-environment-id"] = query.environmentId;
  return headers;
}

export function searchParamsForTraceList(query: DashboardQuery): URLSearchParams {
  const params = new URLSearchParams();
  if (query.projectId) params.set("project_id", query.projectId);
  if (query.environmentId) params.set("environment_id", query.environmentId);
  if (query.traceId) params.set("trace_id", query.traceId);
  if (query.status) params.set("status", query.status);
  if (query.kind) params.set("kind", query.kind);
  if (query.startedAfter) params.set("started_after", query.startedAfter);
  if (query.startedBefore) params.set("started_before", query.startedBefore);
  if (query.model) params.set("model", query.model);
  if (query.release) params.set("release", query.release);
  if (query.minCostMicros !== undefined) params.set("min_cost_micros", String(query.minCostMicros));
  if (query.maxCostMicros !== undefined) params.set("max_cost_micros", String(query.maxCostMicros));
  if (query.minLatencyMs !== undefined) params.set("min_latency_ms", String(query.minLatencyMs));
  if (query.maxLatencyMs !== undefined) params.set("max_latency_ms", String(query.maxLatencyMs));
  params.set("limit", "50");
  return params;
}

export function traceListPath(query: DashboardQuery): string {
  const path: TraceListPathParams = { tenant_id: query.tenantId };
  const params = searchParamsForTraceList(query);
  const suffix = params.toString();
  return `/v1/traces/${encodeURIComponent(path.tenant_id)}${suffix ? `?${suffix}` : ""}`;
}

export function tracePath(query: DashboardQuery, traceId: string): string {
  const path: TracePathParams = { tenant_id: query.tenantId, trace_id: traceId };
  return `/v1/traces/${encodeURIComponent(path.tenant_id)}/${encodeURIComponent(path.trace_id)}`;
}

export function spanIoPath(query: DashboardQuery, traceId: string, spanId: string): string {
  const path: SpanIoPathParams = {
    tenant_id: query.tenantId,
    trace_id: traceId,
    span_id: spanId
  };
  return `/v1/spans/${encodeURIComponent(path.tenant_id)}/${encodeURIComponent(
    path.trace_id
  )}/${encodeURIComponent(path.span_id)}/io`;
}

export async function loadDashboardData(query: DashboardQuery): Promise<DashboardData> {
  const apiBaseUrl = dashboardApiBaseUrl();
  const headers = dashboardApiHeaders(query);
  try {
    const runs = await fetchJson<Page<RunSummary>>(`${apiBaseUrl}${traceListPath(query)}`, headers);
    const activeTraceId = query.traceId || runs.items[0]?.trace_id;
    const trace = activeTraceId
      ? await fetchJson<TraceView>(`${apiBaseUrl}${tracePath(query, activeTraceId)}`, headers)
      : null;
    const selectedSpan =
      trace?.spans.find((span) => span.span_id === query.selectedSpanId) ??
      trace?.spans[0] ??
      null;
    const selectedIo =
      trace && selectedSpan
        ? await fetchJson<SpanIoResponse>(
            `${apiBaseUrl}${spanIoPath(query, trace.trace_id, selectedSpan.span_id)}`,
            headers
          )
        : null;
    return {
      apiBaseUrl,
      query: { ...query, traceId: activeTraceId },
      runs,
      trace,
      selectedSpan,
      selectedIo,
      error: null
    };
  } catch (error) {
    return {
      apiBaseUrl,
      query,
      runs: { items: [], next_cursor: null },
      trace: null,
      selectedSpan: null,
      selectedIo: null,
      error: error instanceof Error ? error.message : String(error)
    };
  }
}

async function fetchJson<T>(url: string, headers: HeadersInit): Promise<T> {
  const response = await fetch(url, { cache: "no-store", headers });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`${response.status} ${response.statusText}: ${text.slice(0, 240)}`);
  }
  return (await response.json()) as T;
}

export function durationMs(start: string, end: string | null | undefined): number | null {
  if (!end) return null;
  return Math.max(0, new Date(end).getTime() - new Date(start).getTime());
}

export function formatDuration(start: string, end: string | null | undefined): string {
  const ms = durationMs(start, end);
  if (ms === null) return "open";
  if (ms < 1000) return `${ms} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

export function formatCost(cost: Money | null | undefined): string {
  if (!cost) return "none";
  return `${cost.currency} ${(cost.amount_micros / 1_000_000).toFixed(6)}`;
}

export function formatModels(models: RunSummary["models"] | undefined): string {
  if (!models?.length) return "no model";
  return models.map((model) => `${model.provider}/${model.name}`).join(", ");
}

export function formatReleases(releaseIds: string[] | undefined): string {
  if (!releaseIds?.length) return "no release";
  return releaseIds.join(", ");
}

export function formatLatency(durationMs: number | null | undefined): string {
  if (durationMs === null || durationMs === undefined) return "open";
  if (durationMs < 1000) return `${durationMs} ms`;
  return `${(durationMs / 1000).toFixed(2)} s`;
}

export function spanDepth(span: CanonicalSpan, spans: CanonicalSpan[]): number {
  let depth = 0;
  let parent = span.parent_span_id;
  const byId = new Map(spans.map((candidate) => [candidate.span_id, candidate]));
  while (parent && byId.has(parent) && depth < 12) {
    depth += 1;
    parent = byId.get(parent)?.parent_span_id ?? null;
  }
  return depth;
}

export function statusLabel(status: string): string {
  if (status === "ok") return "OK";
  if (status === "error") return "Error";
  return "Unset";
}
