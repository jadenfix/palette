import Link from "next/link";
import {
  Activity,
  RotateCcw,
  Search as SearchIcon,
  SquareArrowOutUpRight
} from "lucide-react";
import {
  loadSearchData,
  statusLabel
} from "../../lib/api";
import type { SearchHit, SearchQuery } from "../../lib/api";
import { AGENT_SPAN_KINDS } from "../../lib/span-kinds";

type SearchParams = Record<string, string | string[] | undefined>;

export default async function SearchPage({
  searchParams
}: {
  searchParams?: Promise<SearchParams>;
}) {
  const params = (await searchParams) ?? {};
  const query: SearchQuery = {
    tenantId: textValue(params.tenant) || "demo",
    projectId: textValue(params.project) || "demo",
    environmentId: textValue(params.environment) || "local",
    q: textValue(params.q),
    status: textValue(params.status),
    kind: textValue(params.kind),
    model: textValue(params.model),
    tool: textValue(params.tool),
    traceId: textValue(params.trace_id) ?? textValue(params.trace),
    spanId: textValue(params.span_id) ?? textValue(params.span),
    limit: boundedLimit(params.limit)
  };
  const data = await loadSearchData(query);
  const hits = data.response.hits;
  const chips = searchChips(data.query);

  return (
    <main className="shell search-shell">
      <header className="command-bar">
        <div className="product-lockup">
          <div className="brand-mark" aria-hidden="true">
            B
          </div>
          <div className="product-copy">
            <p className="eyebrow">
              Beater
              <span>Crate Dig</span>
            </p>
            <h1>Span Search</h1>
            <p className="scope-breadcrumb" aria-label="Current scope">
              <span>{data.query.tenantId}</span>
              <span>/</span>
              <span>{data.query.projectId ?? "all-projects"}</span>
              <span>/</span>
              <span>{data.query.environmentId ?? "environment"}</span>
              <span>/</span>
              <span>{data.query.q ?? "all-spans"}</span>
            </p>
          </div>
        </div>
        <div className="command-actions">
          <div className="connection-control">
            <Activity aria-hidden="true" />
            <span className="live-dot" aria-hidden="true" />
            <span>Read API</span>
            <code title={data.apiBaseUrl}>{apiHostLabel(data.apiBaseUrl)}</code>
          </div>
          <Link className="refresh-action" href={traceConsoleHref(data.query)}>
            <SquareArrowOutUpRight aria-hidden="true" />
            <span>Traces</span>
          </Link>
          <Link className="refresh-action" href={searchHref(data.query)}>
            <RotateCcw aria-hidden="true" />
            <span>Refresh</span>
          </Link>
        </div>
      </header>

      <section className="summary-strip search-summary" aria-label="Search summary">
        <SummaryItem label="Hits" value={String(hits.length)} meta={`limit ${data.query.limit ?? 50}`} />
        <SummaryItem label="Query" value={data.query.q || "all spans"} meta={data.query.kind || "any kind"} />
        <SummaryItem label="Status" value={data.query.status ? statusLabel(data.query.status) : "Any"} meta={data.query.model || "any model"} />
        <SummaryItem label="Tool" value={data.query.tool || "Any"} meta={data.query.traceId ? shortHash(data.query.traceId) : "any trace"} />
      </section>

      <section className="toolbar search-toolbar" aria-label="Span search filters">
        <div className="query-row">
          <div className="query-heading">
            <SearchIcon aria-hidden="true" />
            <span>Search</span>
          </div>
          <div className="query-chips" aria-label="Active filters">
            {chips.map((chip) => (
              <span className="query-chip" key={`${chip.label}:${chip.value}`}>
                {chip.label}
                <strong>{chip.value}</strong>
              </span>
            ))}
          </div>
        </div>
        <form className="filters">
          <div className="filter-primary search-filter-grid">
            <label>
              <span>Tenant</span>
              <input name="tenant" defaultValue={data.query.tenantId} />
            </label>
            <label>
              <span>Project</span>
              <input name="project" defaultValue={data.query.projectId} />
            </label>
            <label>
              <span>Environment</span>
              <input name="environment" defaultValue={data.query.environmentId} />
            </label>
            <label className="search-query-field">
              <span>Query</span>
              <input name="q" defaultValue={data.query.q} placeholder="prompt error" />
            </label>
            <label>
              <span>Status</span>
              <select name="status" defaultValue={data.query.status ?? ""}>
                <option value="">Any</option>
                <option value="ok">OK</option>
                <option value="error">Error</option>
                <option value="unset">Unset</option>
              </select>
            </label>
            <label>
              <span>Kind</span>
              <select name="kind" defaultValue={data.query.kind ?? ""}>
                <option value="">Any</option>
                {AGENT_SPAN_KINDS.map((kind) => (
                  <option key={kind} value={kind}>
                    {kind}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Model</span>
              <input name="model" defaultValue={data.query.model} placeholder="gpt" />
            </label>
            <label>
              <span>Tool</span>
              <input name="tool" defaultValue={data.query.tool} placeholder="browser" />
            </label>
            <label>
              <span>Trace</span>
              <input name="trace_id" defaultValue={data.query.traceId} placeholder="trace id" />
            </label>
            <label>
              <span>Span</span>
              <input name="span_id" defaultValue={data.query.spanId} placeholder="span id" />
            </label>
            <label>
              <span>Limit</span>
              <input
                name="limit"
                type="number"
                min="1"
                max="100"
                defaultValue={data.query.limit ?? 50}
              />
            </label>
            <button className="filter-submit" type="submit">
              <SearchIcon aria-hidden="true" />
              <span>Apply</span>
            </button>
            <Link className="filter-reset" href={searchScopeHref(data.query)}>
              Reset
            </Link>
          </div>
        </form>
      </section>

      {data.error ? <div className="notice">{data.error}</div> : null}

      <section className="search-results" aria-label="Span search results">
        <div className="section-heading">
          <h2>Results</h2>
          <span>{hits.length} hits</span>
        </div>
        <div className="search-table">
          {hits.length > 0 ? (
            <div className="search-table-head" aria-hidden="true">
              <span>Span</span>
              <span>Scope</span>
              <span>Signals</span>
              <span>Score</span>
            </div>
          ) : null}
          {hits.map((hit) => (
            <SearchResultRow hit={hit} key={`${hit.trace_id}:${hit.span_id}`} />
          ))}
          {hits.length === 0 ? <div className="empty">No search hits match this query.</div> : null}
        </div>
      </section>
    </main>
  );
}

function SummaryItem({ label, value, meta }: { label: string; value: string; meta: string }) {
  return (
    <div className="summary-item tone-structure" aria-label={label}>
      <span className="summary-copy">
        <span>{label}</span>
        <strong>{value}</strong>
        <small>{meta}</small>
      </span>
    </div>
  );
}

function SearchResultRow({ hit }: { hit: SearchHit }) {
  return (
    <Link
      className="search-row"
      data-status={hit.status}
      href={traceHitHref(hit)}
      title={`${hit.trace_id}/${hit.span_id}`}
    >
      <span className="search-hit-main">
        <strong>{hit.name}</strong>
        <small>
          {hit.trace_id}/{hit.span_id}
        </small>
      </span>
      <span className="search-hit-scope">
        <span>{hit.project_id}</span>
        <small>{hit.environment_id}</small>
      </span>
      <span className="search-hit-signals">
        <span className={`status ${hit.status}`}>{statusLabel(hit.status)}</span>
        <span>{hit.kind || "span"}</span>
        <span>{hit.model || "no model"}</span>
        <span>{hit.tool || "no tool"}</span>
      </span>
      <span className="search-score">{formatScore(hit.score)}</span>
    </Link>
  );
}

function textValue(input: string | string[] | undefined): string | undefined {
  const raw = Array.isArray(input) ? input[0] : input;
  const trimmed = raw?.trim();
  return trimmed ? trimmed : undefined;
}

function boundedLimit(input: string | string[] | undefined): number {
  const raw = textValue(input);
  if (!raw) return 50;
  const parsed = Number(raw);
  if (!Number.isFinite(parsed)) return 50;
  return Math.min(100, Math.max(1, Math.trunc(parsed)));
}

function searchChips(query: SearchQuery): { label: string; value: string }[] {
  const chips: { label: string; value: string }[] = [];
  if (query.q) chips.push({ label: "Query", value: query.q });
  if (query.status) chips.push({ label: "Status", value: query.status });
  if (query.kind) chips.push({ label: "Kind", value: query.kind });
  if (query.model) chips.push({ label: "Model", value: query.model });
  if (query.tool) chips.push({ label: "Tool", value: query.tool });
  if (query.traceId) chips.push({ label: "Trace", value: shortHash(query.traceId) });
  if (query.spanId) chips.push({ label: "Span", value: shortHash(query.spanId) });
  return chips;
}

function searchHref(query: SearchQuery): string {
  const params = searchUiParams(query);
  return `/search?${params.toString()}`;
}

function searchScopeHref(query: SearchQuery): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  return `/search?${params.toString()}`;
}

function traceConsoleHref(query: SearchQuery): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  if (query.traceId) params.set("trace", query.traceId);
  if (query.spanId) params.set("span", query.spanId);
  return `/?${params.toString()}`;
}

function traceHitHref(hit: SearchHit): string {
  const params = new URLSearchParams();
  params.set("tenant", hit.tenant_id);
  params.set("project", hit.project_id);
  params.set("environment", hit.environment_id);
  params.set("trace", hit.trace_id);
  params.set("span", hit.span_id);
  return `/?${params.toString()}`;
}

function searchUiParams(query: SearchQuery): URLSearchParams {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  if (query.q) params.set("q", query.q);
  if (query.status) params.set("status", query.status);
  if (query.kind) params.set("kind", query.kind);
  if (query.model) params.set("model", query.model);
  if (query.tool) params.set("tool", query.tool);
  if (query.traceId) params.set("trace_id", query.traceId);
  if (query.spanId) params.set("span_id", query.spanId);
  if (query.limit !== undefined) params.set("limit", String(query.limit));
  return params;
}

function formatScore(value: number): string {
  if (!Number.isFinite(value)) return "0";
  if (value >= 100) return value.toFixed(0);
  if (value >= 10) return value.toFixed(1);
  return value.toFixed(2);
}

function shortHash(value: string): string {
  return value.length > 18 ? `${value.slice(0, 12)}...${value.slice(-6)}` : value;
}

function apiHostLabel(value: string): string {
  try {
    return new URL(value).host;
  } catch {
    return value.replace(/^https?:\/\//, "").replace(/\/$/, "");
  }
}
