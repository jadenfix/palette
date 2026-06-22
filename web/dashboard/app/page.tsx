import Link from "next/link";
import type { LucideIcon } from "lucide-react";
import {
  Activity,
  BadgePercent,
  Bot,
  BrainCircuit,
  CircleDot,
  ClipboardList,
  Database,
  DatabaseZap,
  ListChecks,
  MessageSquareText,
  Network,
  RotateCcw,
  Search as SearchIcon,
  ShieldCheck,
  SlidersHorizontal,
  UserCheck,
  Wrench
} from "lucide-react";
import {
  CanonicalSpan,
  DashboardQuery,
  Money,
  RunSummary,
  SpanIoResponse,
  TraceView,
  formatCost,
  durationMs,
  formatDuration,
  formatLatency,
  formatModels,
  formatReleases,
  loadDashboardData,
  spanDepth,
  statusLabel
} from "../lib/api";

type SearchParams = Record<string, string | string[] | undefined>;

export default async function DashboardPage({
  searchParams
}: {
  searchParams?: Promise<SearchParams>;
}) {
  const params = (await searchParams) ?? {};
  const query: DashboardQuery = {
    tenantId: value(params.tenant) || "demo",
    projectId: value(params.project) || "demo",
    environmentId: value(params.environment) || "local",
    traceId: value(params.trace),
    selectedSpanId: value(params.span),
    status: value(params.status),
    kind: value(params.kind),
    startedAfter: value(params.started_after),
    startedBefore: value(params.started_before),
    model: value(params.model),
    release: value(params.release),
    minCostMicros: numberValue(params.min_cost_micros),
    maxCostMicros: numberValue(params.max_cost_micros),
    minLatencyMs: numberValue(params.min_latency_ms),
    maxLatencyMs: numberValue(params.max_latency_ms),
    unmask: boolValue(params.unmask),
    unmaskReason: value(params.reason)
  };
  const data = await loadDashboardData(query);
  const spans = data.trace?.spans ?? [];
  const listedSelectedRun = data.trace
    ? data.runs.items.find((run) => run.trace_id === data.trace?.trace_id)
    : undefined;
  const selectedRun = listedSelectedRun ?? runSummaryFromTrace(data.trace);
  const selectedTraceOutsideFilters = Boolean(data.trace && selectedRun && !listedSelectedRun);
  const activeRun = selectedRun ?? data.runs.items[0];
  const runRows =
    selectedTraceOutsideFilters && selectedRun ? [selectedRun, ...data.runs.items] : data.runs.items;
  const failedSpanCount = spans.filter((span) => span.status === "error").length;
  const spanSummaryMeta = activeRun
    ? failedSpanCount > 0
      ? `${failedSpanCount} failed`
      : "no failures"
    : "no trace";
  const spanSummaryTone = failedSpanCount > 0 ? "error" : "structure";
  const tokenTotal = spans.reduce((total, span) => total + spanTokenTotal(span), 0);
  const activeFilters = filterChips(data.query);
  const traceLabel = traceBreadcrumbLabel(data.query.traceId, data.trace?.trace_id);
  const traceInputPlaceholder = tracePlaceholder(data.trace?.trace_id);

  return (
    <main className="shell">
      <header className="command-bar">
        <div className="product-lockup">
          <div className="brand-mark" aria-hidden="true">
            B
          </div>
          <div className="product-copy">
            <p className="eyebrow">
              Beater
              <span>Trace console</span>
            </p>
            <h1>Agent Trace Debugger</h1>
            <p className="scope-breadcrumb" aria-label="Current scope">
              <span>{data.query.tenantId}</span>
              <span>/</span>
              <span>{data.query.projectId ?? "all-projects"}</span>
              <span>/</span>
              <span>{data.query.environmentId ?? "environment"}</span>
              <span>/</span>
              <span>{traceLabel}</span>
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
          <Link
            className="refresh-action"
            href={hrefFor(data.query, {
              trace: data.query.traceId,
              span: data.query.selectedSpanId && data.selectedSpan ? data.selectedSpan.span_id : undefined
            })}
          >
            <RotateCcw aria-hidden="true" />
            <span>Refresh</span>
          </Link>
        </div>
      </header>

      <section className="summary-strip" aria-label="Trace summary">
        <SummaryItem
          label="Run status"
          value={activeRun ? statusLabel(activeRun.status) : "No trace"}
          meta={data.query.environmentId ?? "environment"}
          tone={activeRun?.status ?? "unset"}
        />
        <SummaryItem
          label="Spans"
          value={activeRun ? String(activeRun.span_count) : "0"}
          meta={spanSummaryMeta}
          tone={spanSummaryTone}
        />
        <SummaryItem
          label="Model"
          value={activeRun ? formatModels(activeRun.models) : "none"}
          meta={activeRun ? formatReleases(activeRun.release_ids) : "no release"}
          tone="model"
        />
        <SummaryItem
          label="Cost"
          value={activeRun ? formatCost(activeRun.total_cost) : "none"}
          meta="run total"
          tone="cost"
        />
        <SummaryItem
          label="Latency"
          value={activeRun ? formatLatency(activeRun.duration_ms) : "open"}
          meta="wall clock"
          tone="latency"
        />
        <SummaryItem
          label="Tokens"
          value={tokenTotal > 0 ? tokenTotal.toLocaleString("en-US") : "none"}
          meta="input + output + cached + reasoning"
          tone="release"
        />
      </section>

      <section className="toolbar" aria-label="Trace filters">
        <div className="query-row">
          <div className="query-heading">
            <SearchIcon aria-hidden="true" />
            <span>Query</span>
          </div>
          <div className="query-chips" aria-label="Active filters">
            {activeFilters.map((chip) => (
              <span className="query-chip" key={`${chip.label}:${chip.value}`}>
                {chip.label}
                <strong>{chip.value}</strong>
              </span>
            ))}
          </div>
        </div>
        <form className="filters">
          <div className="filter-primary">
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
            <label className="trace-filter">
              <span>Trace</span>
              <input name="trace" defaultValue={data.query.traceId} placeholder={traceInputPlaceholder} />
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
                <option value="agent.run">agent.run</option>
                <option value="agent.turn">agent.turn</option>
                <option value="agent.plan">agent.plan</option>
                <option value="agent.step">agent.step</option>
                <option value="llm.call">llm.call</option>
                <option value="tool.call">tool.call</option>
                <option value="mcp.request">mcp.request</option>
                <option value="retrieval.query">retrieval.query</option>
                <option value="memory.read">memory.read</option>
                <option value="memory.write">memory.write</option>
                <option value="guardrail.check">guardrail.check</option>
                <option value="human.review">human.review</option>
                <option value="evaluator.run">evaluator.run</option>
                <option value="replay.run">replay.run</option>
              </select>
            </label>
            <button className="filter-submit" type="submit">
              <SearchIcon aria-hidden="true" />
              <span>Apply</span>
            </button>
            <Link className="filter-reset" href={scopeHref(data.query)}>
              Reset
            </Link>
          </div>
          <details className="advanced-filters" open={advancedFiltersActive(data.query)}>
            <summary>
              <SlidersHorizontal aria-hidden="true" />
              Advanced filters
            </summary>
            <div className="filter-secondary">
              <label>
                <span>Started After</span>
                <input
                  name="started_after"
                  defaultValue={data.query.startedAfter}
                  placeholder="2026-01-01T00:00:00Z"
                />
              </label>
              <label>
                <span>Started Before</span>
                <input
                  name="started_before"
                  defaultValue={data.query.startedBefore}
                  placeholder="2026-01-01T01:00:00Z"
                />
              </label>
              <label>
                <span>Model</span>
                <input name="model" defaultValue={data.query.model} placeholder="gpt" />
              </label>
              <label>
                <span>Release</span>
                <input name="release" defaultValue={data.query.release} placeholder="release-a" />
              </label>
              <label>
                <span>Min Cost</span>
                <input
                  name="min_cost_micros"
                  type="number"
                  min="0"
                  defaultValue={numberInput(data.query.minCostMicros)}
                  placeholder="micros"
                />
              </label>
              <label>
                <span>Max Cost</span>
                <input
                  name="max_cost_micros"
                  type="number"
                  min="0"
                  defaultValue={numberInput(data.query.maxCostMicros)}
                  placeholder="micros"
                />
              </label>
              <label>
                <span>Min Latency</span>
                <input
                  name="min_latency_ms"
                  type="number"
                  min="0"
                  defaultValue={numberInput(data.query.minLatencyMs)}
                  placeholder="ms"
                />
              </label>
              <label>
                <span>Max Latency</span>
                <input
                  name="max_latency_ms"
                  type="number"
                  min="0"
                  defaultValue={numberInput(data.query.maxLatencyMs)}
                  placeholder="ms"
                />
              </label>
            </div>
          </details>
        </form>
      </section>

      {data.error ? <div className="notice">{data.error}</div> : null}

      <section className="workspace">
        <aside className="trace-list" aria-label="Traces">
          <div className="section-heading">
            <h2>Traces</h2>
            <span>
              {selectedTraceOutsideFilters
                ? `${data.runs.items.length} + selected`
                : data.runs.items.length}
            </span>
          </div>
          <div className="run-table">
            <div className="run-table-head" aria-hidden="true">
              <span>Trace</span>
              <span>Signals</span>
            </div>
            {runRows.map((run) => {
              const isSelected = run.trace_id === data.trace?.trace_id;
              const isOutsideFilters = isSelected && selectedTraceOutsideFilters;
              return (
                <Link
                  key={run.trace_id}
                  className={isSelected ? "run-row active" : "run-row"}
                  aria-current={isSelected ? "location" : undefined}
                  data-status={run.status}
                  data-outside-filters={isOutsideFilters ? "true" : undefined}
                  href={hrefFor(data.query, { trace: run.trace_id, span: undefined })}
                >
                  <span className="run-state" aria-hidden="true" />
                  <span className="run-body">
                    <span className="run-title-line">
                      <span className="run-name">
                        <strong>{run.first_span_name}</strong>
                        <small>{run.trace_id}</small>
                      </span>
                      <span className="run-badges">
                        <span className={`status ${run.status}`}>{statusLabel(run.status)}</span>
                        {isOutsideFilters ? (
                          <span className="run-filter-note">outside filters</span>
                        ) : null}
                      </span>
                    </span>
                    <span className="run-metrics">
                      <span className="run-cell metric-emphasis" data-label="Spans">
                        <span className="sr-only">Spans </span>
                        {run.span_count}
                      </span>
                      <span className="run-cell" data-label="Latency">
                        <span className="sr-only">Latency </span>
                        {formatLatency(run.duration_ms)}
                      </span>
                      <span className="run-cell" data-label="Cost">
                        <span className="sr-only">Cost </span>
                        {formatCost(run.total_cost)}
                      </span>
                      <span className="run-cell" data-label="Model">
                        <span className="sr-only">Model </span>
                        {formatModels(run.models)}
                      </span>
                      <span className="run-cell" data-label="Release">
                        <span className="sr-only">Release </span>
                        {formatReleases(run.release_ids)}
                      </span>
                    </span>
                  </span>
                </Link>
              );
            })}
            {runRows.length === 0 ? (
              <div className="empty">No traces match this scope.</div>
            ) : null}
          </div>
        </aside>

        <section className="trace-pane" aria-label="Trace detail">
          <div className="section-heading">
            <h2>Waterfall</h2>
            <span>{spans.length} spans</span>
          </div>
          <div className="waterfall" aria-label="Agent span waterfall">
            {spans.length > 0 ? <TimelineAxis spans={spans} /> : null}
            {spans.length > 0 ? (
              <div className="waterfall-head" aria-hidden="true">
                <span>Span</span>
                <span>Kind</span>
                <span>Status</span>
                <span>Duration</span>
              </div>
            ) : null}
            {spans.map((span) => {
              const depth = spanDepth(span, spans);
              const icon = kindIcon(span.kind);
              const KindGlyph = icon.Icon;
              const timing = spanTimeline(span, spans);
              return (
                <Link
                  key={span.span_id}
                  href={hrefFor(data.query, { trace: span.trace_id, span: span.span_id })}
                  aria-current={
                    data.selectedSpan?.span_id === span.span_id ? "location" : undefined
                  }
                  className={
                    data.selectedSpan?.span_id === span.span_id ? "span-line selected" : "span-line"
                  }
                  data-depth={depth}
                  data-kind={span.kind}
                  data-status={span.status}
                  data-span-id={span.span_id}
                  data-span-seq={span.seq}
                  style={
                    {
                      "--depth": depth,
                      "--offset": timing.offset,
                      "--bar": timing.width
                    } as React.CSSProperties
                  }
                >
                  <span className="span-name">
                    <span
                      className={`kind-icon ${kindClass(span.kind)}`}
                      aria-hidden="true"
                      data-icon={icon.key}
                      title={icon.title}
                    >
                      <KindGlyph aria-hidden="true" />
                    </span>
                    <span className="span-title">
                      <span>{span.name}</span>
                    </span>
                  </span>
                  <span className="span-kind">{span.kind}</span>
                  <span className={`status ${span.status}`}>{statusLabel(span.status)}</span>
                  <span className="duration">
                    <span className="span-track" role="presentation">
                      <span className="span-bar" />
                    </span>
                    <span>{formatDuration(span.start_time, span.end_time)}</span>
                  </span>
                </Link>
              );
            })}
            {spans.length === 0 ? (
              <div className="empty">Send an OTLP trace, then refresh this view.</div>
            ) : null}
          </div>
        </section>

        <aside className="span-detail" aria-label="Span detail">
          <div className="section-heading">
            <h2>Span</h2>
            <span>{data.selectedSpan?.kind ?? "none"}</span>
          </div>
          {data.selectedSpan ? (
            <SpanDetail
              span={data.selectedSpan}
              io={data.selectedIo}
              query={data.query}
              spans={spans}
            />
          ) : (
            <div className="empty">Select a span in the waterfall.</div>
          )}
        </aside>
      </section>
    </main>
  );
}

function SummaryItem({
  label,
  value,
  meta,
  tone
}: {
  label: string;
  value: string;
  meta: string;
  tone: string;
}) {
  return (
    <div className={`summary-item tone-${tone}`} aria-label={label}>
      <span className="summary-copy">
        <span>{label}</span>
        <strong>{value}</strong>
        <small>{meta}</small>
      </span>
    </div>
  );
}

function advancedFiltersActive(query: DashboardQuery): boolean {
  return Boolean(
    query.startedAfter ||
      query.startedBefore ||
      query.model ||
      query.release ||
      query.minCostMicros !== undefined ||
      query.maxCostMicros !== undefined ||
      query.minLatencyMs !== undefined ||
      query.maxLatencyMs !== undefined
  );
}

function SpanDetail({
  span,
  io,
  query,
  spans
}: {
  span: CanonicalSpan;
  io: SpanIoResponse | null;
  query: DashboardQuery;
  spans: CanonicalSpan[];
}) {
  const hasRedactedIo = io ? isRedactedIo(io.input) || isRedactedIo(io.output) : false;
  const icon = kindIcon(span.kind);
  const KindGlyph = icon.Icon;
  const artifacts = spanArtifactRefs(span);
  const ancestry = spanAncestry(span, spans);
  return (
    <div className="detail-stack">
      <div className="span-identity">
        <span
          className={`kind-icon detail-kind ${kindClass(span.kind)}`}
          aria-hidden="true"
          data-icon={icon.key}
          title={icon.title}
        >
          <KindGlyph aria-hidden="true" />
        </span>
        <div>
          <h3>{span.name}</h3>
          <p>
            <span>{span.kind}</span>
            <span>{span.model ? `${span.model.provider}/${span.model.name}` : "no model"}</span>
          </p>
          <p>{span.span_id}</p>
        </div>
        <span className={`status ${span.status}`}>{statusLabel(span.status)}</span>
      </div>
      <div className="span-path" aria-label="Selected span path">
        <span className="span-path-label">Path</span>
        {ancestry.map((node, index) => (
          <span className="path-fragment" key={node.span_id}>
            {index > 0 ? (
              <span className="path-separator" aria-hidden="true">
                /
              </span>
            ) : null}
            <span className={`path-node ${kindClass(node.kind)}`}>
              <span>{node.kind}</span>
              <strong>{node.name}</strong>
            </span>
          </span>
        ))}
      </div>
      <dl className="metrics" aria-label="Span metrics">
        <div>
          <dt>Depth</dt>
          <dd>{String(ancestry.length - 1)}</dd>
        </div>
        <div>
          <dt>Model</dt>
          <dd>{span.model ? `${span.model.provider}/${span.model.name}` : "none"}</dd>
        </div>
        <div>
          <dt>Tokens</dt>
          <dd>{tokenSummary(span)}</dd>
        </div>
        <div>
          <dt>Cost</dt>
          <dd>{formatCost(span.cost)}</dd>
        </div>
        <div>
          <dt>Latency</dt>
          <dd>{formatDuration(span.start_time, span.end_time)}</dd>
        </div>
        <div>
          <dt>Parent</dt>
          <dd>{span.parent_span_id ?? "root"}</dd>
        </div>
        <div>
          <dt>Started</dt>
          <dd>{formatTimestamp(span.start_time)}</dd>
        </div>
        <div>
          <dt>Ended</dt>
          <dd>{span.end_time ? formatTimestamp(span.end_time) : "open"}</dd>
        </div>
      </dl>
      <RedactionControls span={span} query={query} hasRedactedIo={hasRedactedIo} />
      <section className="detail-section" aria-label="Span I/O">
        <div className="detail-section-head">
          <h3>I/O</h3>
          <span>{hasRedactedIo && !query.unmask ? "redacted" : "captured"}</span>
        </div>
        <div className="io-grid">
          <IoBlock label="Input" value={io?.input} />
          <IoBlock label="Output" value={io?.output} />
        </div>
      </section>
      <section className="detail-section" aria-label="Artifact references">
        <div className="detail-section-head">
          <h3>Artifacts</h3>
          <span>{artifacts.length}</span>
        </div>
        <div className="artifact-list">
          {artifacts.map((artifact) => (
            <div className="artifact-row" key={`${artifact.label}:${artifact.ref.artifact_id}`}>
              <span>{artifact.label}</span>
              <code>{artifact.ref.uri}</code>
              <small>
                {artifact.ref.mime_type} | {formatBytes(artifact.ref.size_bytes)} |{" "}
                {shortHash(artifact.ref.sha256)}
              </small>
            </div>
          ))}
        </div>
      </section>
      <section className="detail-section" aria-label="Span attributes">
        <div className="detail-section-head">
          <h3>Attributes</h3>
          <span>canonical + unmapped</span>
        </div>
        <JsonPanel label="Canonical" value={span.attributes} />
        <JsonPanel label="Unmapped" value={span.unmapped_attrs} />
      </section>
    </div>
  );
}

function RedactionControls({
  span,
  query,
  hasRedactedIo
}: {
  span: CanonicalSpan;
  query: DashboardQuery;
  hasRedactedIo: boolean;
}) {
  if (query.unmask) {
    return (
      <div className="redaction-controls active">
        <span>Unmasked</span>
        <small>{query.unmaskReason || "no reason"}</small>
        <Link href={hrefFor(query, { trace: span.trace_id, span: span.span_id, unmask: false })}>
          Redacted view
        </Link>
      </div>
    );
  }
  if (!hasRedactedIo) return null;
  return (
    <form className="redaction-controls" aria-label="Unmask redacted I/O">
      <HiddenQueryInputs query={query} traceId={span.trace_id} spanId={span.span_id} />
      <input type="hidden" name="unmask" value="true" />
      <label>
        <span>Reason</span>
        <input name="reason" required minLength={3} placeholder="incident-123" />
      </label>
      <button type="submit">Unmask</button>
    </form>
  );
}

function HiddenQueryInputs({
  query,
  traceId,
  spanId
}: {
  query: DashboardQuery;
  traceId: string;
  spanId: string;
}) {
  const entries: [string, string | number | undefined][] = [
    ["tenant", query.tenantId],
    ["project", query.projectId],
    ["environment", query.environmentId],
    ["trace", traceId],
    ["span", spanId],
    ["status", query.status],
    ["kind", query.kind],
    ["started_after", query.startedAfter],
    ["started_before", query.startedBefore],
    ["model", query.model],
    ["release", query.release],
    ["min_cost_micros", query.minCostMicros],
    ["max_cost_micros", query.maxCostMicros],
    ["min_latency_ms", query.minLatencyMs],
    ["max_latency_ms", query.maxLatencyMs]
  ];
  return (
    <>
      {entries
        .filter((entry): entry is [string, string | number] => entry[1] !== undefined)
        .map(([name, fieldValue]) => (
          <input key={name} type="hidden" name={name} value={String(fieldValue)} />
        ))}
    </>
  );
}

function IoBlock({ label, value }: { label: string; value: SpanIoResponse["input"] | undefined }) {
  let body = "No captured I/O";
  if (value?.kind === "inline") body = prettyJson(value.value);
  if (value?.kind === "artifact") {
    body = `${value.artifact_ref.mime_type}\n${value.artifact_ref.uri}\n${formatBytes(
      value.artifact_ref.size_bytes
    )}`;
  }
  if (value?.kind === "redacted") body = value.reason;
  return (
    <div className={ioClassName(value)} aria-label={`${label} I/O`}>
      <div className="io-head">
        <h4>{label}</h4>
        <span>{ioStatus(value)}</span>
      </div>
      <pre>{body}</pre>
    </div>
  );
}

function JsonPanel({ label, value }: { label: string; value: unknown }) {
  return (
    <div className="json-panel">
      <div className="json-panel-head">
        <h4>{label}</h4>
      </div>
      <pre>{prettyJson(value)}</pre>
    </div>
  );
}

function prettyJson(value: unknown): string {
  if (value === undefined || value === null) return "{}";
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
      try {
        const parsed = JSON.parse(trimmed);
        if (parsed && typeof parsed === "object") return JSON.stringify(parsed, null, 2);
      } catch {
        return value;
      }
    }
    return value;
  }
  return JSON.stringify(value, null, 2);
}

function ioStatus(value: SpanIoResponse["input"] | undefined): string {
  if (!value || value.kind === "missing") return "missing";
  if (value.kind === "artifact") return "artifact";
  if (value.kind === "redacted") return "redacted";
  return "inline";
}

function ioClassName(value: SpanIoResponse["input"] | undefined): string {
  if (!value || value.kind === "missing") return "io missing";
  return value.kind === "redacted" ? "io redacted" : "io";
}

function value(input: string | string[] | undefined): string | undefined {
  return Array.isArray(input) ? input[0] : input;
}

function boolValue(input: string | string[] | undefined): boolean | undefined {
  const raw = value(input);
  if (!raw) return undefined;
  return raw === "true" || raw === "1";
}

function numberValue(input: string | string[] | undefined): number | undefined {
  const raw = value(input);
  if (!raw) return undefined;
  const parsed = Number(raw);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function numberInput(input: number | undefined): string | undefined {
  return input === undefined ? undefined : String(input);
}

function runSummaryFromTrace(trace: TraceView | null): RunSummary | null {
  if (!trace || trace.spans.length === 0) return null;
  const spans = [...trace.spans].sort(compareSpansByStart);
  const firstSpan = spans[0];
  const startedAt = firstSpan.start_time;
  const endedAt = latestEndedAt(spans);
  return {
    tenant_id: trace.tenant_id,
    trace_id: trace.trace_id,
    first_span_name: firstSpan.name,
    span_count: spans.length,
    status: aggregateRunStatus(spans),
    started_at: startedAt,
    ended_at: endedAt,
    duration_ms: durationMs(startedAt, endedAt),
    total_cost: spans.reduce<Money | null>((total, span) => mergeRunCost(total, span.cost), null),
    models: uniqueModels(spans),
    release_ids: uniqueReleaseIds(spans)
  };
}

function compareSpansByStart(left: CanonicalSpan, right: CanonicalSpan): number {
  const leftStart = parsedTimeOrMax(left.start_time);
  const rightStart = parsedTimeOrMax(right.start_time);
  if (leftStart !== rightStart) return leftStart - rightStart;
  return left.seq - right.seq;
}

function parsedTimeOrMax(value: string): number {
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : Number.MAX_SAFE_INTEGER;
}

function latestEndedAt(spans: CanonicalSpan[]): string | null {
  let latest: { value: string; ms: number } | null = null;
  for (const span of spans) {
    if (!span.end_time) continue;
    const ms = Date.parse(span.end_time);
    if (!Number.isFinite(ms)) continue;
    if (!latest || ms > latest.ms) latest = { value: span.end_time, ms };
  }
  return latest?.value ?? null;
}

function aggregateRunStatus(spans: CanonicalSpan[]): string {
  if (spans.some((span) => span.status === "error")) return "error";
  if (spans.some((span) => span.status === "ok")) return "ok";
  return "unset";
}

function mergeRunCost(total: Money | null, next: Money | null | undefined): Money | null {
  if (!next) return total;
  if (!total) return next;
  if (total.currency !== next.currency) return total;
  return {
    currency: total.currency,
    amount_micros: total.amount_micros + next.amount_micros
  };
}

function uniqueModels(spans: CanonicalSpan[]): RunSummary["models"] {
  const models: RunSummary["models"] = [];
  for (const span of spans) {
    if (!span.model) continue;
    if (
      models.some(
        (model) => model.provider === span.model?.provider && model.name === span.model?.name
      )
    ) {
      continue;
    }
    models.push(span.model);
  }
  return models;
}

function uniqueReleaseIds(spans: CanonicalSpan[]): string[] {
  const releaseIds: string[] = [];
  for (const span of spans) {
    const releaseId = stringAttribute(span.attributes, [
      "agent.release_id",
      "beater.release_id",
      "release_id"
    ]);
    if (releaseId && !releaseIds.includes(releaseId)) releaseIds.push(releaseId);
  }
  return releaseIds;
}

function stringAttribute(attributes: unknown, keys: string[]): string | null {
  if (!attributes || typeof attributes !== "object" || Array.isArray(attributes)) return null;
  const record = attributes as Record<string, unknown>;
  for (const key of keys) {
    const value = record[key];
    if (typeof value === "string" && value.trim()) return value;
  }
  return null;
}

function spanTokenTotal(span: CanonicalSpan): number {
  if (!span.tokens) return 0;
  return span.tokens.input + span.tokens.output + span.tokens.cache_read + span.tokens.reasoning;
}

function spanAncestry(span: CanonicalSpan, spans: CanonicalSpan[]): CanonicalSpan[] {
  const byId = new Map(spans.map((candidate) => [candidate.span_id, candidate]));
  const ancestry = [span];
  const seen = new Set([span.span_id]);
  let parentId = span.parent_span_id;
  while (parentId && byId.has(parentId) && !seen.has(parentId)) {
    const parent = byId.get(parentId);
    if (!parent) break;
    ancestry.unshift(parent);
    seen.add(parent.span_id);
    parentId = parent.parent_span_id;
  }
  return ancestry;
}

function tokenSummary(span: CanonicalSpan): string {
  if (!span.tokens) return "none";
  const total = span.tokens.input + span.tokens.output + span.tokens.reasoning;
  const inputLabel = span.kind === "llm.call" ? "prompt" : "input";
  const outputLabel = span.kind === "llm.call" ? "completion" : "output";
  const parts = [
    `${total.toLocaleString("en-US")} total`,
    `${span.tokens.input.toLocaleString("en-US")} ${inputLabel}`,
    `${span.tokens.output.toLocaleString("en-US")} ${outputLabel}`
  ];
  if (span.tokens.reasoning > 0) {
    parts.push(`${span.tokens.reasoning.toLocaleString("en-US")} reasoning`);
  }
  if (span.tokens.cache_read > 0) {
    parts.push(`${span.tokens.cache_read.toLocaleString("en-US")} cached`);
  }
  return parts.join(", ");
}

function spanArtifactRefs(span: CanonicalSpan) {
  return [
    { label: "raw", ref: span.raw_ref },
    span.input_ref ? { label: "input", ref: span.input_ref } : null,
    span.output_ref ? { label: "output", ref: span.output_ref } : null
  ].filter(
    (artifact): artifact is { label: string; ref: NonNullable<CanonicalSpan["raw_ref"]> } =>
      artifact !== null
  );
}

function shortHash(value: string): string {
  return value.length > 18 ? `${value.slice(0, 12)}...${value.slice(-6)}` : value;
}

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MiB`;
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toISOString().replace("T", " ").replace(/\.\d{3}Z$/, "Z");
}

function traceBreadcrumbLabel(explicitTraceId: string | undefined, resolvedTraceId: string | undefined): string {
  if (explicitTraceId) return explicitTraceId;
  if (resolvedTraceId) return `latest: ${shortHash(resolvedTraceId)}`;
  return "latest trace";
}

function tracePlaceholder(resolvedTraceId: string | undefined): string {
  return resolvedTraceId ? `latest: ${shortHash(resolvedTraceId)}` : "latest";
}

function apiHostLabel(value: string): string {
  try {
    return new URL(value).host;
  } catch {
    return value.replace(/^https?:\/\//, "").replace(/\/$/, "");
  }
}

function filterChips(query: DashboardQuery): { label: string; value: string }[] {
  const chips: { label: string; value: string }[] = [];
  if (query.traceId) chips.push({ label: "Trace", value: shortHash(query.traceId) });
  if (query.status) chips.push({ label: "Status", value: query.status });
  if (query.kind) chips.push({ label: "Kind", value: query.kind });
  if (query.model) chips.push({ label: "Model", value: query.model });
  if (query.release) chips.push({ label: "Release", value: query.release });
  if (query.startedAfter) chips.push({ label: "After", value: query.startedAfter });
  if (query.startedBefore) chips.push({ label: "Before", value: query.startedBefore });
  if (query.minCostMicros !== undefined) {
    chips.push({ label: "Min cost", value: String(query.minCostMicros) });
  }
  if (query.maxCostMicros !== undefined) {
    chips.push({ label: "Max cost", value: String(query.maxCostMicros) });
  }
  if (query.minLatencyMs !== undefined) {
    chips.push({ label: "Min latency", value: `${query.minLatencyMs} ms` });
  }
  if (query.maxLatencyMs !== undefined) {
    chips.push({ label: "Max latency", value: `${query.maxLatencyMs} ms` });
  }
  return chips;
}

function scopeHref(query: DashboardQuery): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  return `/?${params.toString()}`;
}

function hrefFor(
  query: DashboardQuery,
  next: { trace?: string; span?: string | undefined; unmask?: boolean | undefined }
): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  if (next.trace ?? query.traceId) params.set("trace", next.trace ?? query.traceId ?? "");
  if (next.span) params.set("span", next.span);
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
  const unmask = next.unmask === true;
  if (unmask) params.set("unmask", "true");
  if (unmask && query.unmaskReason) params.set("reason", query.unmaskReason);
  return `/?${params.toString()}`;
}

function spanTimeline(
  span: CanonicalSpan,
  spans: CanonicalSpan[]
): { offset: string; width: string } {
  const bounds = spans
    .map((candidate) => {
      const start = Date.parse(candidate.start_time);
      const end = candidate.end_time ? Date.parse(candidate.end_time) : start;
      if (!Number.isFinite(start) || !Number.isFinite(end)) return null;
      return { start, end: Math.max(start, end) };
    })
    .filter((candidate): candidate is { start: number; end: number } => candidate !== null);

  if (bounds.length === 0) return { offset: "0%", width: "100%" };

  const traceStart = Math.min(...bounds.map((bound) => bound.start));
  const traceEnd = Math.max(...bounds.map((bound) => bound.end));
  const traceDuration = Math.max(1, traceEnd - traceStart);
  const spanStart = Date.parse(span.start_time);
  const spanEnd = span.end_time ? Date.parse(span.end_time) : spanStart;

  if (!Number.isFinite(spanStart) || !Number.isFinite(spanEnd)) {
    return { offset: "0%", width: "8%" };
  }

  const offset = Math.min(96, Math.max(0, ((spanStart - traceStart) / traceDuration) * 100));
  const duration = Math.max(1, Math.max(spanEnd, spanStart) - spanStart);
  const available = Math.max(4, 100 - offset);
  const width = Math.min(available, Math.max(4, (duration / traceDuration) * 100));
  return { offset: `${offset.toFixed(1)}%`, width: `${width.toFixed(1)}%` };
}

function TimelineAxis({ spans }: { spans: CanonicalSpan[] }) {
  const axis = traceAxis(spans);
  if (!axis) return null;
  return (
    <div className="timeline-axis" aria-label="Trace timeline scale">
      <span className="axis-label">Timeline</span>
      <span className="axis-rail">
        {axis.ticks.map((tick, index) => (
          <span
            className="axis-tick"
            key={`${tick.label}-${index}`}
            style={{ "--tick": tick.offset } as React.CSSProperties}
          >
            <i aria-hidden="true" />
            <b>{tick.label}</b>
          </span>
        ))}
      </span>
    </div>
  );
}

function traceAxis(
  spans: CanonicalSpan[]
): { ticks: { offset: string; label: string }[] } | null {
  const bounds = spans
    .map((span) => {
      const start = Date.parse(span.start_time);
      const end = span.end_time ? Date.parse(span.end_time) : start;
      if (!Number.isFinite(start) || !Number.isFinite(end)) return null;
      return { start, end: Math.max(start, end) };
    })
    .filter((span): span is { start: number; end: number } => span !== null);
  if (bounds.length === 0) return null;
  const start = Math.min(...bounds.map((span) => span.start));
  const end = Math.max(...bounds.map((span) => span.end));
  const duration = Math.max(1, end - start);
  const tickCount = duration < 10 ? 2 : 4;
  const ticks = Array.from({ length: tickCount + 1 }, (_, index) => {
    const ratio = index / tickCount;
    return {
      offset: `${(ratio * 100).toFixed(1)}%`,
      label: formatAxisDuration(duration * ratio)
    };
  });
  return { ticks };
}

function formatAxisDuration(ms: number): string {
  if (ms > 0 && ms < 10) return `${ms.toFixed(1)} ms`;
  if (ms < 1000) return `${Math.round(ms)} ms`;
  return `${(ms / 1000).toFixed(1)} s`;
}

function kindClass(kind: string): string {
  if (kind.startsWith("agent.")) return "agent";
  if (kind === "llm.call") return "llm";
  if (kind === "tool.call" || kind === "mcp.request") return "tool";
  if (kind.startsWith("memory.")) return "memory";
  if (kind.includes("guardrail")) return "guardrail";
  if (kind.includes("evaluator")) return "eval";
  if (kind === "human.review") return "human";
  if (kind === "replay.run") return "replay";
  return "other";
}

type KindIcon = { key: string; Icon: LucideIcon; title: string };

function kindIcon(kind: string): KindIcon {
  if (kind === "agent.run") return { key: "agent-run", Icon: Bot, title: "Agent run" };
  if (kind === "agent.turn") {
    return { key: "agent-turn", Icon: MessageSquareText, title: "Agent turn" };
  }
  if (kind === "agent.plan") return { key: "agent-plan", Icon: ClipboardList, title: "Agent plan" };
  if (kind === "agent.step") return { key: "agent-step", Icon: ListChecks, title: "Agent step" };
  if (kind === "llm.call") return { key: "llm", Icon: BrainCircuit, title: "LLM call" };
  if (kind === "tool.call") return { key: "tool", Icon: Wrench, title: "Tool call" };
  if (kind === "mcp.request") return { key: "mcp", Icon: Network, title: "MCP request" };
  if (kind === "retrieval.query") {
    return { key: "retrieval", Icon: SearchIcon, title: "Retrieval query" };
  }
  if (kind === "memory.read") return { key: "memory-read", Icon: Database, title: "Memory read" };
  if (kind === "memory.write") {
    return { key: "memory-write", Icon: DatabaseZap, title: "Memory write" };
  }
  if (kind === "guardrail.check") {
    return { key: "guardrail", Icon: ShieldCheck, title: "Guardrail check" };
  }
  if (kind === "human.review") return { key: "human", Icon: UserCheck, title: "Human review" };
  if (kind === "evaluator.run") {
    return { key: "eval", Icon: BadgePercent, title: "Evaluator run" };
  }
  if (kind === "replay.run") return { key: "replay", Icon: RotateCcw, title: "Replay run" };
  return { key: "other", Icon: CircleDot, title: kind };
}

function isRedactedIo(value: SpanIoResponse["input"] | undefined): boolean {
  return value?.kind === "redacted";
}
