import Link from "next/link";
import type { LucideIcon } from "lucide-react";
import {
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
  SpanIoResponse,
  formatCost,
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
  const activeRun =
    data.runs.items.find((run) => run.trace_id === data.trace?.trace_id) ?? data.runs.items[0];

  return (
    <main className="shell">
      <header className="topbar">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">
            B
          </div>
          <div className="brand-block">
            <p className="eyebrow">
              Beater
              <span>{data.query.environmentId ?? "environment"}</span>
            </p>
            <h1>Agent Trace Debugger</h1>
            <p className="scope-line">
              {data.query.tenantId}
              <span>/</span>
              {data.query.projectId ?? "all-projects"}
              <span>/</span>
              {data.query.traceId ?? "latest trace"}
            </p>
          </div>
        </div>
        <div className="api-pill">
          <span className="live-dot" aria-hidden="true" />
          <span>Read API</span>
          <code>{data.apiBaseUrl}</code>
        </div>
      </header>

      <section className="summary-strip" aria-label="Trace summary">
        <SummaryItem label="Status" value={activeRun ? statusLabel(activeRun.status) : "No trace"} />
        <SummaryItem label="Spans" value={activeRun ? String(activeRun.span_count) : "0"} />
        <SummaryItem label="Model" value={activeRun ? formatModels(activeRun.models) : "none"} />
        <SummaryItem label="Cost" value={activeRun ? formatCost(activeRun.total_cost) : "none"} />
        <SummaryItem
          label="Latency"
          value={activeRun ? formatLatency(activeRun.duration_ms) : "open"}
        />
        <SummaryItem
          label="Release"
          value={activeRun ? formatReleases(activeRun.release_ids) : "no release"}
        />
      </section>

      <section className="toolbar" aria-label="Trace filters">
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
              <input name="trace" defaultValue={data.query.traceId} placeholder="latest" />
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
            <span>{data.runs.items.length}</span>
          </div>
          <div className="run-table">
            <div className="run-table-head" aria-hidden="true">
              <span>Status</span>
              <span>Trace</span>
              <span>Spans</span>
              <span>Model</span>
              <span>Cost</span>
              <span>Latency</span>
              <span>Release</span>
            </div>
            {data.runs.items.map((run) => (
              <Link
                key={run.trace_id}
                className={run.trace_id === data.trace?.trace_id ? "run-row active" : "run-row"}
                href={hrefFor(data.query, { trace: run.trace_id, span: undefined })}
              >
                <span className={`status ${run.status}`}>{statusLabel(run.status)}</span>
                <span className="run-name">
                  <strong>{run.first_span_name}</strong>
                  <small>{run.trace_id}</small>
                </span>
                <span className="run-cell metric-emphasis" data-label="Spans">
                  {run.span_count}
                </span>
                <span className="run-cell" data-label="Model">
                  {formatModels(run.models)}
                </span>
                <span className="run-cell" data-label="Cost">
                  {formatCost(run.total_cost)}
                </span>
                <span className="run-cell" data-label="Latency">
                  {formatLatency(run.duration_ms)}
                </span>
                <span className="run-cell" data-label="Release">
                  {formatReleases(run.release_ids)}
                </span>
              </Link>
            ))}
            {data.runs.items.length === 0 ? (
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
                  className={
                    data.selectedSpan?.span_id === span.span_id ? "span-line selected" : "span-line"
                  }
                  data-depth={depth}
                  data-kind={span.kind}
                  data-span-name={span.name}
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
                      aria-label={`${span.kind} icon`}
                      data-icon={icon.key}
                      title={icon.title}
                    >
                      <KindGlyph aria-hidden="true" />
                      <span className="sr-only">{icon.title}</span>
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
            <SpanDetail span={data.selectedSpan} io={data.selectedIo} query={data.query} />
          ) : (
            <div className="empty">Select a span in the waterfall.</div>
          )}
        </aside>
      </section>
    </main>
  );
}

function SummaryItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="summary-item">
      <span>{label}</span>
      <strong>{value}</strong>
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
  query
}: {
  span: CanonicalSpan;
  io: SpanIoResponse | null;
  query: DashboardQuery;
}) {
  const hasRedactedIo = io ? isRedactedIo(io.input) || isRedactedIo(io.output) : false;
  return (
    <div className="detail-stack">
      <div className="span-identity">
        <div>
          <h3>{span.name}</h3>
          <p>{span.span_id}</p>
        </div>
        <span className={`status ${span.status}`}>{statusLabel(span.status)}</span>
      </div>
      <dl className="metrics">
        <div>
          <dt>Status</dt>
          <dd>{statusLabel(span.status)}</dd>
        </div>
        <div>
          <dt>Model</dt>
          <dd>{span.model ? `${span.model.provider}/${span.model.name}` : "none"}</dd>
        </div>
        <div>
          <dt>Tokens</dt>
          <dd>{span.tokens ? span.tokens.input + span.tokens.output + span.tokens.reasoning : "none"}</dd>
        </div>
        <div>
          <dt>Cost</dt>
          <dd>{formatCost(span.cost)}</dd>
        </div>
        <div>
          <dt>Latency</dt>
          <dd>{formatDuration(span.start_time, span.end_time)}</dd>
        </div>
      </dl>
      <RedactionControls span={span} query={query} hasRedactedIo={hasRedactedIo} />
      <IoBlock label="Input" value={io?.input} />
      <IoBlock label="Output" value={io?.output} />
      <div className="attrs">
        <h3>Attributes</h3>
        <pre>{JSON.stringify(span.attributes, null, 2)}</pre>
      </div>
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
  if (value?.kind === "inline") body = JSON.stringify(value.value, null, 2);
  if (value?.kind === "artifact") {
    body = `${value.artifact_ref.mime_type}\n${value.artifact_ref.uri}\n${value.artifact_ref.size_bytes} bytes`;
  }
  if (value?.kind === "redacted") body = value.reason;
  return (
    <div className={ioClassName(value)}>
      <h3>{label}</h3>
      <pre>{body}</pre>
    </div>
  );
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
  const unmask = next.unmask ?? query.unmask;
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
