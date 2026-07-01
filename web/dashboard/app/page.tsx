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
  ioVisibilityLabel,
  isRedactedIoValue,
  loadDashboardData,
  orderSpansForWaterfall,
  spanDepth,
  spanTokenSummary,
  spanTokenTotal,
  statusLabel,
  timestampMicros
} from "../lib/api";
import {
  AGENT_SPAN_KINDS,
  displaySpanIoLabels,
  isLlmCallKind,
  spanKindClass,
  spanKindMeta
} from "../lib/span-kinds";
import {
  advancedFilterCount,
  applyFilterParams,
  filterChips,
  parseQueryFromSearchParams,
  shortHash,
} from "../lib/dashboard-query";
import { Gate2ConfirmationCode, Gate2SpanClickTracker } from "./Gate2Confirmation";
import { getSession } from "../lib/auth";
import { criticalPathSpanIds, criticalPathStats, formatMs } from "../lib/analyze";
import { AccountMenu } from "../components/AccountMenu";

type SearchParams = Record<string, string | string[] | undefined>;

export default async function DashboardPage({
  searchParams
}: {
  searchParams?: Promise<SearchParams>;
}) {
  const params = (await searchParams) ?? {};
  const query: DashboardQuery = parseQueryFromSearchParams(params);
  const data = await loadDashboardData(query);
  const account = await getSession();
  const spans = data.trace ? orderSpansForWaterfall(data.trace.spans) : [];
  const criticalIds = criticalPathSpanIds(spans);
  const criticalStats = criticalPathStats(spans);
  const selectedTraceProjectId = traceProjectId(data.trace);
  const listedSelectedRun = data.trace
    ? data.runs.items.find(
        (run) =>
          run.trace_id === data.trace?.trace_id &&
          (!selectedTraceProjectId || run.project_id === selectedTraceProjectId)
      )
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
  const advancedFilterTotal = advancedFilterCount(data.query);
  const traceLabel = traceBreadcrumbLabel(data.query.traceId, data.trace?.trace_id);
  const traceInputPlaceholder = tracePlaceholder(data.trace?.trace_id);

  return (
    <main className="shell">
      <Gate2SpanClickTracker />
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
          <Link className="refresh-action" href={searchHref(data.query)}>
            <SearchIcon aria-hidden="true" />
            <span>Search</span>
          </Link>
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
          {account ? (
            <AccountMenu account={account} />
          ) : (
            <Link href="/login" className="btn btn-primary btn-sm">
              Sign in
            </Link>
          )}
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
                {AGENT_SPAN_KINDS.map((kind) => (
                  <option key={kind} value={kind}>
                    {kind}
                  </option>
                ))}
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
          <details
            className="advanced-filters"
            data-active={advancedFilterTotal > 0 ? "true" : undefined}
          >
            <summary>
              <SlidersHorizontal aria-hidden="true" />
              <span>Advanced filters</span>
              <strong>
                {advancedFilterTotal > 0 ? `${advancedFilterTotal} active` : "optional"}
              </strong>
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
              const isSelected =
                run.trace_id === data.trace?.trace_id &&
                (!selectedTraceProjectId || run.project_id === selectedTraceProjectId);
              const isOutsideFilters = isSelected && selectedTraceOutsideFilters;
              return (
                <Link
                  key={`${run.project_id}:${run.trace_id}`}
                  className={isSelected ? "run-row active" : "run-row"}
                  aria-current={isSelected ? "location" : undefined}
                  data-status={run.status}
                  data-outside-filters={isOutsideFilters ? "true" : undefined}
                  href={hrefFor(data.query, {
                    project: run.project_id,
                    trace: run.trace_id,
                    span: undefined
                  })}
                >
                  <span className="run-state" aria-hidden="true" />
                  <span className="run-body">
                    <span className="run-title-line">
                      <span className="run-name">
                        <strong>{run.first_span_name}</strong>
                        <small>
                          {run.project_id}/{run.trace_id}
                        </small>
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
            <span>
              {spans.length} spans
              {criticalStats.spanCount > 0 ? (
                <span
                  className="cp-chip"
                  title={`Critical path: the ${criticalStats.spanCount} nested spans that determine when this run finishes (longest path through the timed span tree).`}
                >
                  <span className="cp-dot" aria-hidden="true" />
                  critical path {formatMs(criticalStats.totalMs)}
                </span>
              ) : null}
            </span>
          </div>
          {criticalStats.parallelizable ? (
            <p className="cp-hint">
              <span className="cp-dot" aria-hidden="true" />
              <span>
                <strong>{criticalStats.parallelizable.a}</strong> and{" "}
                <strong>{criticalStats.parallelizable.b}</strong> ran back-to-back — if they&apos;re
                independent, running them in parallel could save ~{formatMs(criticalStats.parallelizable.savingsMs)}.
              </span>
            </p>
          ) : null}
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
              const isLlmCall = isLlmCallKind(span.kind);
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
                  data-critical={criticalIds.has(span.span_id) ? "true" : undefined}
                  data-span-id={span.span_id}
                  data-span-seq={span.seq}
                  data-gate2-confirm-span={isLlmCall ? "true" : undefined}
                  data-trace-id={isLlmCall ? span.trace_id : undefined}
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
                      className={`kind-icon ${spanKindClass(span.kind)}`}
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
  const hasRedactedIo = io ? isRedactedIoValue(io.input) || isRedactedIoValue(io.output) : false;
  const icon = kindIcon(span.kind);
  const KindGlyph = icon.Icon;
  const artifacts = spanArtifactRefs(span);
  const ancestry = spanAncestry(span, spans);
  const ioLabels = displaySpanIoLabels(span.kind);
  const showConfirmationSlot = isLlmCallKind(span.kind) && query.selectedSpanId === span.span_id;
  return (
    <div className="detail-stack">
      <div className="span-identity">
        <span
          className={`kind-icon detail-kind ${spanKindClass(span.kind)}`}
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
      <dl
        className={showConfirmationSlot ? "span-proof-strip with-confirmation" : "span-proof-strip"}
        aria-label="Selected span essentials"
      >
        <div>
          <dt>Model</dt>
          <dd>{span.model ? `${span.model.provider}/${span.model.name}` : "none"}</dd>
        </div>
        <div>
          <dt>Tokens</dt>
          <dd className="token-cell">
            <span className="token-summary">{spanTokenSummary(span)}</span>
            <TokenBreakdown span={span} />
          </dd>
        </div>
        <div>
          <dt>Cost</dt>
          <dd>{formatCost(span.cost)}</dd>
        </div>
        <div>
          <dt>Latency</dt>
          <dd>{formatDuration(span.start_time, span.end_time)}</dd>
        </div>
        {showConfirmationSlot ? (
          <Gate2ConfirmationCode traceId={span.trace_id} spanId={span.span_id} />
        ) : null}
      </dl>
      <RedactionControls span={span} query={query} hasRedactedIo={hasRedactedIo} />
      <section className="detail-section" aria-label="Span I/O">
        <div className="detail-section-head">
          <h3>I/O</h3>
          <span>{ioVisibilityLabel(hasRedactedIo, query.unmask)}</span>
        </div>
        <div className="io-grid">
          <IoBlock label={ioLabels.input} value={io?.input} />
          <IoBlock label={ioLabels.output} value={io?.output} />
        </div>
      </section>
      <div className="span-path" aria-label="Selected span path">
        <span className="span-path-label">Path</span>
        {ancestry.map((node, index) => (
          <span className="path-fragment" key={node.span_id}>
            {index > 0 ? (
              <span className="path-separator" aria-hidden="true">
                /
              </span>
            ) : null}
            <span className={`path-node ${spanKindClass(node.kind)}`}>
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
          <dd>
            <span>{spanTokenSummary(span)}</span>
          </dd>
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

function TokenBreakdown({ span }: { span: Pick<CanonicalSpan, "kind" | "tokens"> }) {
  if (!span.tokens) return null;
  const { input: inputLabel, output: outputLabel } = displaySpanIoLabels(span.kind);
  const items = [
    { label: inputLabel, value: span.tokens.input },
    { label: outputLabel, value: span.tokens.output },
    { label: "Reasoning", value: span.tokens.reasoning },
    { label: "Cached", value: span.tokens.cache_read }
  ];

  return (
    <span className="token-breakdown" aria-label="Token breakdown">
      {items.map((item) => (
        <span className="token-chip" key={item.label}>
          <b>{item.label}</b>
          <span>{item.value.toLocaleString("en-US")}</span>
        </span>
      ))}
    </span>
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
    const state = hasRedactedIo ? "Still redacted" : "Unmask requested";
    return (
      <div className="redaction-controls active">
        <span>{state}</span>
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
  if (isRedactedIoValue(value) && value?.kind !== "redacted") body = "redacted by policy";
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
  if (isRedactedIoValue(value)) return "redacted";
  if (value.kind === "artifact") return "artifact";
  return "inline";
}

function ioClassName(value: SpanIoResponse["input"] | undefined): string {
  if (!value || value.kind === "missing") return "io missing";
  return isRedactedIoValue(value) ? "io redacted" : "io";
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
    project_id: firstSpan.project_id,
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

function traceProjectId(trace: TraceView | null): string | undefined {
  return trace?.spans[0]?.project_id;
}

function compareSpansByStart(left: CanonicalSpan, right: CanonicalSpan): number {
  const leftStart = parsedTimeOrMax(left.start_time);
  const rightStart = parsedTimeOrMax(right.start_time);
  if (leftStart !== rightStart) return leftStart - rightStart;
  return left.seq - right.seq;
}

function parsedTimeOrMax(value: string): number {
  return timestampMicros(value) ?? Number.MAX_SAFE_INTEGER;
}

function latestEndedAt(spans: CanonicalSpan[]): string | null {
  let latest: { value: string; micros: number } | null = null;
  for (const span of spans) {
    if (!span.end_time) continue;
    const micros = timestampMicros(span.end_time);
    if (micros === null) continue;
    if (!latest || micros > latest.micros) latest = { value: span.end_time, micros };
  }
  return latest?.value ?? null;
}

function aggregateRunStatus(spans: CanonicalSpan[]): RunSummary["status"] {
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


function scopeHref(query: DashboardQuery): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  return `/?${params.toString()}`;
}

function searchHref(query: DashboardQuery): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (query.projectId) params.set("project", query.projectId);
  if (query.environmentId) params.set("environment", query.environmentId);
  if (query.traceId) params.set("trace_id", query.traceId);
  if (query.selectedSpanId) params.set("span_id", query.selectedSpanId);
  if (query.status) params.set("status", query.status);
  if (query.kind) params.set("kind", query.kind);
  if (query.model) params.set("model", query.model);
  return `/search?${params.toString()}`;
}

function hrefFor(
  query: DashboardQuery,
  next: {
    project?: string | undefined;
    trace?: string;
    span?: string | undefined;
    unmask?: boolean | undefined;
  }
): string {
  const params = new URLSearchParams();
  params.set("tenant", query.tenantId);
  if (next.project ?? query.projectId) params.set("project", next.project ?? query.projectId ?? "");
  if (query.environmentId) params.set("environment", query.environmentId);
  if (next.trace ?? query.traceId) params.set("trace", next.trace ?? query.traceId ?? "");
  if (next.span) params.set("span", next.span);
  applyFilterParams(query, params);
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
      const start = timestampMicros(candidate.start_time);
      const end = candidate.end_time ? timestampMicros(candidate.end_time) : start;
      if (start === null || end === null) return null;
      return { start, end: Math.max(start, end) };
    })
    .filter((candidate): candidate is { start: number; end: number } => candidate !== null);

  if (bounds.length === 0) return { offset: "0%", width: "100%" };

  const traceStart = Math.min(...bounds.map((bound) => bound.start));
  const traceEnd = Math.max(...bounds.map((bound) => bound.end));
  const traceDuration = traceEnd - traceStart;
  const spanStart = timestampMicros(span.start_time);
  const spanEnd = span.end_time ? timestampMicros(span.end_time) : spanStart;

  if (spanStart === null || spanEnd === null) {
    return { offset: "0%", width: "8%" };
  }

  if (traceDuration <= 0) {
    return { offset: "0%", width: "4%" };
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
      const start = timestampMicros(span.start_time);
      const end = span.end_time ? timestampMicros(span.end_time) : start;
      if (start === null || end === null) return null;
      return { start, end: Math.max(start, end) };
    })
    .filter((span): span is { start: number; end: number } => span !== null);
  if (bounds.length === 0) return null;
  const start = Math.min(...bounds.map((span) => span.start));
  const end = Math.max(...bounds.map((span) => span.end));
  const duration = end - start;
  if (duration <= 0) {
    return {
      ticks: [
        { offset: "0.0%", label: "0 ms" },
        { offset: "100.0%", label: "same instant" }
      ]
    };
  }
  const tickCount = duration < 1000 ? 2 : 4;
  const ticks = Array.from({ length: tickCount + 1 }, (_, index) => {
    const ratio = index / tickCount;
    return {
      offset: `${(ratio * 100).toFixed(1)}%`,
      label: formatAxisDuration((duration * ratio) / 1000)
    };
  });
  return { ticks };
}

function formatAxisDuration(ms: number): string {
  if (ms > 0 && ms < 10) return `${ms.toFixed(1)} ms`;
  if (ms < 1000) return `${Math.round(ms)} ms`;
  return `${(ms / 1000).toFixed(1)} s`;
}

type KindIcon = { key: string; Icon: LucideIcon; title: string };

const SPAN_KIND_ICONS: Record<string, LucideIcon> = {
  "agent-run": Bot,
  "agent-turn": MessageSquareText,
  "agent-plan": ClipboardList,
  "agent-step": ListChecks,
  llm: BrainCircuit,
  tool: Wrench,
  mcp: Network,
  retrieval: SearchIcon,
  "memory-read": Database,
  "memory-write": DatabaseZap,
  guardrail: ShieldCheck,
  human: UserCheck,
  eval: BadgePercent,
  replay: RotateCcw,
  other: CircleDot
};

function kindIcon(kind: string): KindIcon {
  const meta = spanKindMeta(kind);
  return { ...meta, Icon: SPAN_KIND_ICONS[meta.key] ?? CircleDot };
}
