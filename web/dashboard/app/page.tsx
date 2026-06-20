import Link from "next/link";
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
    maxLatencyMs: numberValue(params.max_latency_ms)
  };
  const data = await loadDashboardData(query);
  const spans = data.trace?.spans ?? [];

  return (
    <main className="shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Beater</p>
          <h1>Agent Trace Debugger</h1>
        </div>
        <div className="api-pill">{data.apiBaseUrl}</div>
      </header>

      <section className="toolbar" aria-label="Trace filters">
        <form className="filter-grid">
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
          <label>
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
          <button type="submit">Refresh</button>
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
            {data.runs.items.map((run) => (
              <Link
                key={run.trace_id}
                className={run.trace_id === data.trace?.trace_id ? "run-row active" : "run-row"}
                href={hrefFor(data.query, { trace: run.trace_id, span: undefined })}
              >
                <span className={`status ${run.status}`}>{statusLabel(run.status)}</span>
                <strong>{run.first_span_name}</strong>
                <small>{run.trace_id}</small>
                <span>{run.span_count} spans</span>
                <span>{formatModels(run.models)}</span>
                <span>{formatCost(run.total_cost)}</span>
                <span>{formatLatency(run.duration_ms)}</span>
                <span>{formatReleases(run.release_ids)}</span>
              </Link>
            ))}
            {data.runs.items.length === 0 ? (
              <div className="empty">No traces match this scope.</div>
            ) : null}
          </div>
        </aside>

        <section className="trace-pane" aria-label="Trace detail">
          <div className="section-heading">
            <h2>{data.trace ? data.trace.trace_id : "No trace selected"}</h2>
            <span>{spans.length} spans</span>
          </div>
          <div className="waterfall" aria-label="Agent span waterfall">
            {spans.map((span) => (
              <Link
                key={span.span_id}
                href={hrefFor(data.query, { trace: span.trace_id, span: span.span_id })}
                className={
                  data.selectedSpan?.span_id === span.span_id ? "span-line selected" : "span-line"
                }
                style={{
                  "--depth": spanDepth(span, spans),
                  "--bar": spanWidth(span, spans)
                } as React.CSSProperties}
              >
                <span className={`kind-dot ${kindClass(span.kind)}`} />
                <span className="span-title">{span.name}</span>
                <span className="span-kind">{span.kind}</span>
                <span className={`status ${span.status}`}>{statusLabel(span.status)}</span>
                <span className="span-bar" />
                <span className="duration">{formatDuration(span.start_time, span.end_time)}</span>
              </Link>
            ))}
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
            <SpanDetail span={data.selectedSpan} io={data.selectedIo} />
          ) : (
            <div className="empty">Select a span in the waterfall.</div>
          )}
        </aside>
      </section>
    </main>
  );
}

function SpanDetail({ span, io }: { span: CanonicalSpan; io: SpanIoResponse | null }) {
  return (
    <div className="detail-stack">
      <div>
        <h3>{span.name}</h3>
        <p>{span.span_id}</p>
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
      </dl>
      <IoBlock label="Input" value={io?.input} />
      <IoBlock label="Output" value={io?.output} />
      <div className="attrs">
        <h3>Attributes</h3>
        <pre>{JSON.stringify(span.attributes, null, 2)}</pre>
      </div>
    </div>
  );
}

function IoBlock({ label, value }: { label: string; value: SpanIoResponse["input"] | undefined }) {
  let body = "Missing";
  if (value?.kind === "inline") body = JSON.stringify(value.value, null, 2);
  if (value?.kind === "artifact") {
    body = `${value.artifact_ref.mime_type}\n${value.artifact_ref.uri}\n${value.artifact_ref.size_bytes} bytes`;
  }
  if (value?.kind === "redacted") body = value.reason;
  return (
    <div className="io">
      <h3>{label}</h3>
      <pre>{body}</pre>
    </div>
  );
}

function value(input: string | string[] | undefined): string | undefined {
  return Array.isArray(input) ? input[0] : input;
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
  next: { trace?: string; span?: string | undefined }
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
  return `/?${params.toString()}`;
}

function spanWidth(span: CanonicalSpan, spans: CanonicalSpan[]): string {
  const durations = spans
    .map((candidate) => {
      if (!candidate.end_time) return 0;
      return Math.max(0, new Date(candidate.end_time).getTime() - new Date(candidate.start_time).getTime());
    })
    .filter((duration) => duration > 0);
  const max = Math.max(...durations, 1);
  if (!span.end_time) return "20%";
  const duration = Math.max(0, new Date(span.end_time).getTime() - new Date(span.start_time).getTime());
  return `${Math.max(8, Math.round((duration / max) * 100))}%`;
}

function kindClass(kind: string): string {
  if (kind.startsWith("agent.")) return "agent";
  if (kind === "llm.call") return "llm";
  if (kind === "tool.call" || kind === "mcp.request") return "tool";
  if (kind.startsWith("memory.")) return "memory";
  if (kind.includes("guardrail")) return "guardrail";
  if (kind.includes("evaluator")) return "eval";
  return "other";
}
