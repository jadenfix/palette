import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const root = new URL("..", import.meta.url).pathname;

function loadDashboardApiModule(context = {}) {
  const source = readFileSync(join(root, "lib/api.ts"), "utf8");
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: {
      esModuleInterop: true,
      module: ts.ModuleKind.CommonJS,
      target: ts.ScriptTarget.ES2022
    }
  });
  const module = { exports: {} };
  vm.runInNewContext(
    outputText,
    {
      exports: module.exports,
      module,
      process,
      URLSearchParams,
      ...context
    },
    { filename: "lib/api.ts" }
  );
  return module.exports;
}

test("dashboard page exposes the trace inspection surface", () => {
  const page = readFileSync(join(root, "app/page.tsx"), "utf8");
  assert.match(page, /Agent Trace Debugger/);
  assert.match(page, /Trace filters/);
  assert.match(page, /Agent span waterfall/);
  assert.match(page, /lucide-react/);
  assert.match(page, /kindIcon/);
  assert.match(page, /orderSpansForWaterfall/);
  assert.match(page, /data-depth/);
  assert.match(page, /data-kind/);
  assert.match(page, /data-status/);
  assert.match(page, /data-span-id/);
  assert.match(page, /data-span-seq/);
  assert.doesNotMatch(page, /data-span-name/);
  assert.match(page, /data-icon/);
  assert.match(page, /<KindGlyph aria-hidden="true" \/>/);
  assert.match(page, /className="sr-only"/);
  assert.match(page, /aria-hidden="true"\n                      data-icon=\{icon\.key\}/);
  assert.doesNotMatch(page, /aria-label=\{`\$\{span\.kind\} icon`\}/);
  assert.match(page, /input \+ output \+ cached \+ reasoning/);
  assert.match(page, /spanTokenTotal/);
  assert.match(page, /spanTokenSummary/);
  assert.doesNotMatch(page, /label: "AI"/);
  assert.doesNotMatch(page, /label: "Fn"/);
  assert.match(page, /data-label="Spans"/);
  assert.match(page, /data-label="Latency"/);
  assert.match(page, /spanTimeline/);
  assert.match(page, /"--offset"/);
  assert.match(page, /span-track/);
  assert.match(page, /SpanDetail/);
  assert.match(page, /IoBlock/);
  assert.match(page, /spanIoLabels/);
  assert.match(page, /isRedactedIoValue/);
  assert.match(page, /ioVisibilityLabel/);
  assert.match(page, /Still redacted/);
  assert.match(page, /Unmask requested/);
  assert.match(page, /redacted by policy/);
  assert.doesNotMatch(page, />Unmasked</);
  assert.match(page, /kind === "llm\.call"/);
  assert.match(page, /input: "Prompt", output: "Completion"/);
  assert.match(page, /label=\{ioLabels\.input\}/);
  assert.match(page, /label=\{ioLabels\.output\}/);
  assert.doesNotMatch(page, /Detail sections/);
  assert.doesNotMatch(page, /detail-tabs/);
  assert.match(page, /RedactionControls/);
  assert.match(page, /Unmask redacted I\/O/);
  assert.match(page, /name="unmask"/);
  assert.match(page, /name="reason"/);
  assert.match(page, /next\.unmask === true/);
  assert.doesNotMatch(page, /next\.unmask \?\? query\.unmask/);
  assert.match(page, /name="status"/);
  assert.match(page, /name="kind"/);
  assert.match(page, /name="started_after"/);
  assert.match(page, /name="model"/);
  assert.match(page, /name="release"/);
  assert.match(page, /name="min_cost_micros"/);
  assert.match(page, /name="min_latency_ms"/);
  assert.match(page, /traceBreadcrumbLabel/);
  assert.match(page, /tracePlaceholder/);
  assert.match(page, /placeholder=\{traceInputPlaceholder\}/);
  assert.match(page, /runSummaryFromTrace/);
  assert.match(page, /selectedTraceOutsideFilters/);
  assert.match(page, /data-outside-filters/);
  assert.match(page, /aria-current=\{isSelected \? "location" : undefined\}/);
  assert.match(page, /aria-label="Selected span path"/);
  assert.match(page, /advancedFilterCount/);
  assert.doesNotMatch(page, /advancedFiltersActive/);
  assert.match(page, /data-active=\{advancedFilterTotal > 0 \? "true" : undefined\}/);
  assert.match(page, /aria-label="Selected span essentials"/);
  assert.match(page, /Gate2SpanClickTracker/);
  assert.match(page, /Gate2ConfirmationCode/);
  assert.match(page, /showConfirmationSlot/);
  assert.match(page, /data-gate2-confirm-span/);
  assert.doesNotMatch(page, /spanConfirmationCode/);
  assert.doesNotMatch(page, /BEATER_GATE2_CONFIRMATION_SALT/);
  assert.doesNotMatch(page, /createHash/);
  assert.match(page, /"span-proof-strip with-confirmation"/);
  assert.match(page, /aria-label="Span metrics"/);
  assert.match(page, /aria-label=\{`\$\{label\} I\/O`\}/);
  assert.match(page, /spanAncestry/);
  assert.match(page, /apiHostLabel/);
  assert.doesNotMatch(page, /No secondary filters/);
  assert.doesNotMatch(page, /className=\{`run-state \$\{run\.status\}`\}/);
  assert.doesNotMatch(page, /No artifact references/);
  assert.doesNotMatch(page, /detail-section io-section/);
  assert.doesNotMatch(page, /detail-section attrs/);
  assert.match(page, /outside filters/);
  assert.match(page, /agent\.release_id/);
  assert.match(page, /human\.review/);
  assert.match(page, /replay\.run/);
  assert.match(page, /kind === "human\.review"/);
  assert.match(page, /kind === "replay\.run"/);
});

test("dashboard chrome stays dense and tool-like", () => {
  const css = readFileSync(join(root, "app/globals.css"), "utf8");
  assert.match(css, /--canvas: #f4f6f8/);
  assert.match(css, /font-variant-numeric: tabular-nums/);
  assert.match(css, /\.command-bar \{/);
  assert.match(css, /\.connection-control/);
  assert.match(css, /\.workspace \{/);
  assert.match(css, /grid-template-areas: "traces waterfall detail";/);
  assert.match(
    css,
    /grid-template-columns: minmax\(286px, 360px\) minmax\(520px, 1fr\) minmax\(334px, 410px\);/
  );
  assert.match(css, /\.summary-strip \{\n  background: var\(--surface\);/);
  assert.match(css, /\.summary-item::before/);
  assert.match(css, /\.summary-item\.tone-model::before/);
  assert.match(css, /\.summary-item\.tone-cost::before/);
  assert.match(css, /\.summary-item\.tone-latency::before/);
  assert.match(css, /\.summary-item\.tone-release::before/);
  assert.match(css, /\.query-chip/);
  assert.match(css, /\.query-chips:empty/);
  assert.match(css, /\.advanced-filters\[data-active="true"\] summary strong/);
  assert.match(css, /\.filter-reset/);
  assert.match(css, /\.span-detail \{\n  align-self: start;/);
  assert.match(css, /\.run-state/);
  assert.match(css, /\.run-metrics/);
  assert.match(css, /\.run-row\[data-status="error"\]/);
  assert.match(css, /\.run-row\[data-outside-filters="true"\]/);
  assert.match(css, /\.run-row\[aria-current="location"\]/);
  assert.match(css, /\.run-filter-note/);
  assert.match(css, /\.run-cell::before/);
  assert.match(css, /\.timeline-axis/);
  assert.match(css, /\.axis-tick/);
  assert.match(css, /\.waterfall-head,\n\.span-line,\n\.timeline-axis \{/);
  assert.match(css, /\.span-line\[data-kind="llm\.call"\] \{/);
  assert.match(css, /--kind-color: var\(--accent\);/);
  assert.match(css, /\.span-line\[data-status="error"\]/);
  assert.match(css, /\.span-line\[aria-current="location"\]/);
  assert.match(css, /\.kind-icon svg \{/);
  assert.match(css, /\.detail-kind svg \{/);
  assert.doesNotMatch(css, /\.muted-copy/);
  assert.doesNotMatch(css, /\.detail-tabs/);
  assert.match(css, /\.span-path \{/);
  assert.match(css, /\.path-node\.llm \{/);
  assert.match(css, /\.path-node strong \{/);
  assert.match(css, /\.span-proof-strip \{/);
  assert.match(css, /\.span-proof-strip\.with-confirmation \{/);
  assert.match(css, /\.span-proof-strip dd \{/);
  assert.match(css, /\.span-proof-strip \.confirmation-code dd \{/);
  assert.match(css, /\.span-track \{/);
  assert.match(css, /left: var\(--offset\);/);
  assert.match(css, /background: var\(--kind-color\);/);
  assert.doesNotMatch(css, /--header:/);
  assert.doesNotMatch(css, /\.summary-icon/);
  assert.doesNotMatch(css, /font-weight: 7[3-9]0/);
  assert.doesNotMatch(css, /--bg-grid/);
  assert.doesNotMatch(css, /linear-gradient\(var\(--bg-grid\)/);
  assert.doesNotMatch(css, /\.workspace,\n\.notice \{\n  background/);
  assert.doesNotMatch(css, /\.span-identity \.status \{\n  display: none;/);
});

test("local Gate 2 proof serves standalone CSS assets", () => {
  const proof = readFileSync(join(root, "..", "..", "scripts/gate2-proof.sh"), "utf8");
  assert.match(proof, /npm run build/);
  assert.ok(proof.includes(".next/standalone/.next/static"));
  assert.ok(proof.includes("cp -R .next/static .next/standalone/.next/static"));
});

test("dashboard client uses public beater read endpoints", () => {
  const api = readFileSync(join(root, "lib/api.ts"), "utf8");
  assert.match(api, /generated\/api-types/);
  assert.match(api, /TraceListPathParams/);
  assert.match(api, /SpanOperation/);
  assert.match(api, /SpanPathParams/);
  assert.match(api, /TraceReadQuery/);
  assert.match(api, /encodeURIComponent\(path\.tenant_id\)/);
  assert.match(api, /spanPath/);
  assert.match(api, /fetchJson<CanonicalSpan>/);
  assert.match(api, /traceReadParams/);
  assert.match(api, /params\.set\("unmask", "true"\)/);
  assert.match(api, /params\.set\("reason"/);
  assert.match(api, /\/v1\/spans\//);
  assert.match(api, /\/io/);
  assert.match(api, /const activeTraceId = query\.traceId \|\| runs\.items\[0\]\?\.trace_id/);
  assert.match(api, /query,\n\s+runs,/);
  assert.doesNotMatch(api, /query: \{ \.\.\.query, traceId: activeTraceId/);
  assert.match(api, /BEATER_API_TOKEN/);
  assert.match(api, /x-beater-project-id/);
  assert.match(api, /x-beater-environment-id/);
  assert.match(api, /formatApiError\(response\.status, response\.statusText/);
  assert.match(api, /let runs: Page<RunSummary>;/);
  assert.match(api, /selectedSpan = selectedSpanFromTrace;/);
  assert.match(api, /selectedIo = await fetchJson<SpanIoResponse>/);
  assert.match(api, /if \(!query\.unmask\) return params;/);
});

test("dashboard API errors stay concise and user-facing", () => {
  const { formatApiError } = loadDashboardApiModule();
  assert.equal(
    formatApiError(404, "Not Found", '{"error":"trace abc not found","status":404}'),
    "API 404 Not Found: trace abc not found"
  );
  assert.equal(
    formatApiError(503, "Service Unavailable", '{"message":"trace store unavailable"}'),
    "API 503 Service Unavailable: trace store unavailable"
  );
  assert.equal(formatApiError(502, "Bad Gateway", ""), "API 502 Bad Gateway: empty response");
  assert.equal(
    formatApiError(500, "Internal Server Error", "\n<html>\n  backend failed\n</html>\n"),
    "API 500 Internal Server Error: <html> backend failed </html>"
  );
  assert.equal(
    formatApiError(400, "", JSON.stringify({ detail: "invalid project id" })),
    "API 400: invalid project id"
  );
  const long = formatApiError(500, "Internal Server Error", "x".repeat(300));
  assert.equal(long.length, "API 500 Internal Server Error: ".length + 240);
  assert.ok(long.endsWith("..."));
});

test("dashboard time formatters hide invalid backend timing", () => {
  const { durationMs, formatDuration, formatLatency, timestampMicros } =
    loadDashboardApiModule();

  assert.equal(durationMs("bad-start", "2026-01-01T00:00:00Z"), null);
  assert.equal(durationMs("2026-01-01T00:00:00Z", "bad-end"), null);
  assert.equal(
    timestampMicros("2026-01-01T00:00:00.000123456Z") -
      timestampMicros("2026-01-01T00:00:00Z"),
    123
  );
  assert.equal(
    durationMs("2026-01-01T00:00:00.000100Z", "2026-01-01T00:00:00.000410Z"),
    0.31
  );
  assert.equal(formatDuration("bad-start", "2026-01-01T00:00:00Z"), "open");
  assert.equal(formatDuration("2026-01-01T00:00:01Z", "2026-01-01T00:00:00Z"), "0 ms");
  assert.equal(
    formatDuration("2026-01-01T00:00:00.000100Z", "2026-01-01T00:00:00.000410Z"),
    "0.310 ms"
  );
  assert.equal(formatLatency(Number.NaN), "open");
  assert.equal(formatLatency(Number.POSITIVE_INFINITY), "open");
  assert.equal(formatLatency(-1), "open");
  assert.equal(formatLatency(999), "999 ms");
});

test("dashboard token helpers include cached reads in UI totals", () => {
  const { spanTokenSummary, spanTokenTotal } = loadDashboardApiModule();
  const span = {
    kind: "llm.call",
    tokens: { input: 3, output: 4, cache_read: 8, reasoning: 2 }
  };
  const toolSpan = {
    kind: "tool.call",
    tokens: { input: 1, output: 2, cache_read: 3, reasoning: 0 }
  };

  assert.equal(spanTokenTotal(span), 17);
  assert.equal(spanTokenSummary(span), "17 total, 3 prompt, 4 completion, 2 reasoning, 8 cached");
  assert.equal(spanTokenSummary(toolSpan), "6 total, 1 input, 2 output, 3 cached");
  assert.equal(spanTokenSummary({ kind: "llm.call", tokens: null }), "none");
});

test("dashboard redaction helpers treat inline sentinels as redacted", () => {
  const { ioVisibilityLabel, isRedactedIoValue } = loadDashboardApiModule();

  assert.equal(isRedactedIoValue({ kind: "inline", value: "[redacted]" }), true);
  assert.equal(isRedactedIoValue({ kind: "redacted", reason: "pii" }), true);
  assert.equal(isRedactedIoValue({ kind: "inline", value: "hello" }), false);
  assert.equal(isRedactedIoValue({ kind: "missing" }), false);
  assert.equal(ioVisibilityLabel(true, false), "redacted");
  assert.equal(ioVisibilityLabel(true, true), "still redacted");
  assert.equal(ioVisibilityLabel(false, true), "unmask requested");
  assert.equal(ioVisibilityLabel(false, false), "captured");
});

test("dashboard span depth stops on malformed parent cycles", () => {
  const { spanDepth } = loadDashboardApiModule();
  const a = {
    span_id: "a",
    parent_span_id: "b"
  };
  const b = {
    span_id: "b",
    parent_span_id: "a"
  };

  assert.equal(spanDepth(a, [a, b]), 1);
  assert.equal(spanDepth(b, [a, b]), 1);
});

test("dashboard waterfall ordering keeps parents before backend-unsorted children", () => {
  const { orderSpansForWaterfall } = loadDashboardApiModule();
  const spans = [
    spanFixture("llm", "step", "2026-01-01T00:00:00.004Z", 4),
    spanFixture("run", null, "2026-01-01T00:00:00.001Z", 1),
    spanFixture("mcp", "tool", "2026-01-01T00:00:00.003500Z", 5),
    spanFixture("turn", "run", "2026-01-01T00:00:00.002Z", 2),
    spanFixture("tool", "step", "2026-01-01T00:00:00.003Z", 3),
    spanFixture("step", "turn", "2026-01-01T00:00:00.002500Z", 6)
  ];

  const orderedIds = Array.from(orderSpansForWaterfall(spans), (span) => span.span_id);

  assert.deepEqual(orderedIds, ["run", "turn", "step", "tool", "mcp", "llm"]);
});

test("dashboard waterfall ordering does not drop malformed cycles or missing parents", () => {
  const { orderSpansForWaterfall } = loadDashboardApiModule();
  const spans = [
    spanFixture("cycle-a", "cycle-b", "2026-01-01T00:00:00.002Z", 2),
    spanFixture("orphan", "missing-parent", "2026-01-01T00:00:00.001Z", 1),
    spanFixture("cycle-b", "cycle-a", "2026-01-01T00:00:00.003Z", 3)
  ];

  const orderedIds = orderSpansForWaterfall(spans).map((span) => span.span_id);

  assert.deepEqual([...orderedIds].sort(), ["cycle-a", "cycle-b", "orphan"]);
  assert.equal(new Set(orderedIds).size, spans.length);
  assert.equal(orderedIds[0], "orphan");
});

test("dashboard read URLs send unmask reason only with unmask=true", () => {
  const { tracePath, spanPath, spanIoPath } = loadDashboardApiModule();
  const redactedQuery = {
    tenantId: "demo",
    projectId: "demo",
    environmentId: "local",
    unmask: false,
    unmaskReason: "incident-123"
  };
  const unmaskedQuery = { ...redactedQuery, unmask: true };

  assert.equal(tracePath(redactedQuery, "trace-1"), "/v1/traces/demo/trace-1");
  assert.equal(spanPath(redactedQuery, "trace-1", "span-1"), "/v1/spans/demo/trace-1/span-1");
  assert.equal(spanIoPath(redactedQuery, "trace-1", "span-1"), "/v1/spans/demo/trace-1/span-1/io");
  assert.equal(
    tracePath(unmaskedQuery, "trace-1"),
    "/v1/traces/demo/trace-1?unmask=true&reason=incident-123"
  );
  assert.equal(
    spanIoPath(unmaskedQuery, "trace-1", "span-1"),
    "/v1/spans/demo/trace-1/span-1/io?unmask=true&reason=incident-123"
  );
});

test("dashboard loader preserves trace context when span I/O fails", async () => {
  const runs = {
    items: [{ trace_id: "trace-1", root_name: "run", span_count: 1 }],
    next_cursor: null
  };
  const span = {
    trace_id: "trace-1",
    span_id: "span-1",
    parent_span_id: null,
    name: "call-policy-model",
    kind: "llm.call",
    status: "ok",
    start_time: "2026-01-01T00:00:00Z",
    end_time: "2026-01-01T00:00:01Z",
    attributes: {},
    unmapped_attrs: {},
    events: [],
    links: [],
    tokens: { input: 1, output: 2, cache_read: 3, reasoning: 4 },
    cost: null,
    model: null
  };
  const trace = { trace_id: "trace-1", spans: [span] };
  const { loadDashboardData } = loadDashboardApiModule({
    fetch: async (url) => {
      const href = String(url);
      if (href.includes("/v1/traces/demo?")) return okJson(runs);
      if (href.includes("/v1/traces/demo/trace-1")) return okJson(trace);
      if (href.includes("/v1/spans/demo/trace-1/span-1/io")) {
        return errorJson(503, "Service Unavailable", { message: "span I/O unavailable" });
      }
      if (href.includes("/v1/spans/demo/trace-1/span-1")) return okJson(span);
      throw new Error(`unexpected fetch ${href}`);
    }
  });

  const data = await loadDashboardData({ tenantId: "demo" });

  assert.equal(data.runs.items.length, 1);
  assert.equal(data.trace?.trace_id, "trace-1");
  assert.equal(data.selectedSpan?.span_id, "span-1");
  assert.equal(data.selectedIo, null);
  assert.match(data.error, /span I\/O unavailable/);
});

test("dashboard loader selects the ordered root span when trace spans arrive unsorted", async () => {
  const runs = {
    items: [{ trace_id: "trace-1", root_name: "run", span_count: 2 }],
    next_cursor: null
  };
  const child = {
    ...spanFixture("child", "root", "2026-01-01T00:00:00.002Z", 2),
    trace_id: "trace-1",
    name: "child-llm",
    kind: "llm.call"
  };
  const root = {
    ...spanFixture("root", null, "2026-01-01T00:00:00.001Z", 1),
    trace_id: "trace-1",
    name: "agent-run",
    kind: "agent.run"
  };
  const trace = { trace_id: "trace-1", spans: [child, root] };
  const requests = [];
  const { loadDashboardData } = loadDashboardApiModule({
    fetch: async (url) => {
      const href = String(url);
      requests.push(href);
      if (href.includes("/v1/traces/demo?")) return okJson(runs);
      if (href.includes("/v1/traces/demo/trace-1")) return okJson(trace);
      if (href.includes("/v1/spans/demo/trace-1/root/io")) {
        return okJson({ input: { kind: "missing" }, output: { kind: "missing" } });
      }
      if (href.includes("/v1/spans/demo/trace-1/root")) return okJson(root);
      throw new Error(`unexpected fetch ${href}`);
    }
  });

  const data = await loadDashboardData({ tenantId: "demo" });

  assert.equal(data.selectedSpan?.span_id, "root");
  assert.equal(requests.some((href) => href.includes("/v1/spans/demo/trace-1/child")), false);
});

test("dashboard loader does not select a fallback span for stale span URLs", async () => {
  const runs = {
    items: [{ trace_id: "trace-1", root_name: "run", span_count: 1 }],
    next_cursor: null
  };
  const span = {
    trace_id: "trace-1",
    span_id: "span-1",
    parent_span_id: null,
    name: "call-policy-model",
    kind: "llm.call",
    status: "ok",
    start_time: "2026-01-01T00:00:00Z",
    end_time: "2026-01-01T00:00:01Z",
    attributes: {},
    unmapped_attrs: {},
    events: [],
    links: [],
    tokens: { input: 1, output: 2, cache_read: 3, reasoning: 4 },
    cost: null,
    model: null
  };
  const trace = { trace_id: "trace-1", spans: [span] };
  const requests = [];
  const { loadDashboardData } = loadDashboardApiModule({
    fetch: async (url) => {
      const href = String(url);
      requests.push(href);
      if (href.includes("/v1/traces/demo?")) return okJson(runs);
      if (href.includes("/v1/traces/demo/trace-1")) return okJson(trace);
      throw new Error(`unexpected fetch ${href}`);
    }
  });

  const data = await loadDashboardData({
    tenantId: "demo",
    traceId: "trace-1",
    selectedSpanId: "missing-span"
  });

  assert.equal(data.trace?.trace_id, "trace-1");
  assert.equal(data.selectedSpan, null);
  assert.equal(data.selectedIo, null);
  assert.match(data.error, /Span missing-span was not found in trace trace-1/);
  assert.equal(requests.some((href) => href.includes("/v1/spans/")), false);
});

function okJson(value) {
  return {
    ok: true,
    json: async () => value
  };
}

function errorJson(status, statusText, value) {
  return {
    ok: false,
    status,
    statusText,
    text: async () => JSON.stringify(value)
  };
}

function spanFixture(spanId, parentSpanId, startTime, seq) {
  return {
    trace_id: "trace-1",
    span_id: spanId,
    parent_span_id: parentSpanId,
    name: spanId,
    kind: "agent.step",
    status: "ok",
    start_time: startTime,
    end_time: "2026-01-01T00:00:00.010Z",
    seq,
    attributes: {},
    unmapped_attrs: {},
    events: [],
    links: [],
    tokens: null,
    cost: null,
    model: null,
    raw_ref: {
      artifact_id: `${spanId}-raw`,
      uri: `artifact://${spanId}`,
      mime_type: "application/json",
      size_bytes: 2,
      sha256: "0".repeat(64)
    }
  };
}

test("generated api client is produced from the checked-in openapi snapshot", () => {
  const spec = readFileSync(join(root, "openapi/beater-read-api.json"), "utf8");
  const generated = readFileSync(join(root, "lib/generated/api-types.ts"), "utf8");
  assert.match(spec, /"\/v1\/traces\/\{tenant_id\}"/);
  assert.match(spec, /"started_after"/);
  assert.match(spec, /"min_cost_micros"/);
  assert.match(generated, /openapi_list_traces/);
  assert.match(generated, /started_after/);
  assert.match(generated, /min_cost_micros/);
});

test("browser proof covers all canonical span kinds and can record a demo", () => {
  const e2e = readFileSync(join(root, "tests/e2e/dashboard.spec.ts"), "utf8");
  for (const kind of [
    "agent.run",
    "agent.turn",
    "agent.plan",
    "agent.step",
    "llm.call",
    "tool.call",
    "mcp.request",
    "retrieval.query",
    "memory.read",
    "memory.write",
    "guardrail.check",
    "human.review",
    "evaluator.run",
    "replay.run"
  ]) {
    assert.match(e2e, new RegExp(kind.replace(".", "\\.")));
  }
  for (const name of [
    "refund-agent-run",
    "customer-refund-turn",
    "execute-refund-step",
    "lookup-order-tool",
    "mcp-order-service"
  ]) {
    assert.match(e2e, new RegExp(name));
  }
  assert.match(e2e, /toHaveAttribute\("data-depth", "4"\)/);
  assert.match(e2e, /toHaveAttribute\("data-icon", "mcp"\)/);
  assert.match(e2e, /model=gpt-demo&release=compose-demo/);
  assert.match(e2e, /status=error/);
  assert.match(e2e, /data-outside-filters="true"/);
  assert.match(e2e, /outside filters/);
  assert.match(e2e, /compose-demo/);
  const recorder = readFileSync(join(root, "tests/e2e/record-gate2-demo.mjs"), "utf8");
  assert.match(recorder, /recordVideo/);
  assert.match(recorder, /requireAttribute/);
  assert.match(recorder, /BEATER_GATE2_RECORD_MODE/);
  assert.match(recorder, /recordQuickstartFlow/);
  assert.match(recorder, /recordAllKindFlow/);
  assert.match(recorder, /quickstartNotes/);
  assert.match(recorder, /literal five-line stock OpenTelemetry quickstart trace/);
  assert.match(recorder, /gate2-compose-browser-demo\.webm/);
  assert.match(recorder, /createHash\("sha256"\)/);
  assert.match(recorder, /minimumRecordingMs = 9000/);
  assert.match(recorder, /llmReviewDwellMs = 4500/);
  assert.match(recorder, /toolReviewDwellMs = 2500/);
  assert.match(recorder, /waitForReviewableRecording/);
  assert.match(recorder, /waitForMetric\(detail, "Model", "openai\/gpt-quickstart"\)/);
  assert.match(recorder, /waitForMetric\(detail, "Model", "openai\/gpt-demo"\)/);
  assert.doesNotMatch(recorder, /detail\.getByText\("openai\/gpt/);
  assert.match(recorder, /Recording mode: compose/);
  assert.match(recorder, /data-depth/);
  assert.match(recorder, /data-icon/);
  assert.doesNotMatch(recorder, /data-span-seq/);
  assert.match(recorder, /five-line-llm-call/);
  assert.match(recorder, /12 total, 5 prompt, 7 completion/);
  assert.match(recorder, /33 total, 18 prompt, 11 completion, 4 reasoning/);
  assert.match(recorder, /token breakdown/);
  assert.match(recorder, /filter\(\{ hasText: "Prompt" \}\)/);
  assert.match(recorder, /filter\(\{ hasText: "Completion" \}\)/);
  assert.match(recorder, /hello from stock OpenTelemetry/);
  assert.match(recorder, /color\/icon-coded all-kind agent waterfall/);
  assert.match(recorder, /gate2-browser-demo\.webm/);
  const quickstart = readFileSync(join(root, "tests/e2e/quickstart.spec.ts"), "utf8");
  assert.match(quickstart, /BEATER_E2E_QUICKSTART_TRACE_ID/);
  assert.match(quickstart, /BEATER_E2E_QUICKSTART_RELEASE/);
  assert.doesNotMatch(quickstart, /BEATER_E2E_TRACE_ID/);
  assert.match(quickstart, /five-line-llm-call/);
  assert.match(quickstart, /gpt-quickstart/);
  assert.match(quickstart, /releaseParam/);
  assert.match(quickstart, /encodeURIComponent\(release\)/);
  assert.match(quickstart, /toHaveValue\(""\)/);
  assert.match(quickstart, /placeholder", \/latest: \//);
  assert.match(quickstart, /toHaveCount\(1\)/);
  assert.match(quickstart, /data-span-id/);
  assert.doesNotMatch(quickstart, /waterfall\.getByText\("five-line-llm-call"\)\.click/);
  assert.match(quickstart, /traceRow\.click\(\)/);
  assert.match(quickstart, /toHaveURL/);
  assert.match(quickstart, /hello from stock OpenTelemetry/);
  assert.match(quickstart, /hello from Beater/);
  assert.match(quickstart, /data-icon/);
  assert.match(quickstart, /Selected span path/);
  assert.match(quickstart, /12 total, 5 prompt, 7 completion/);
  assert.match(quickstart, /Selected span essentials/);
  assert.match(quickstart, /confirmationCode\(selectedTraceId, selectedSpanId\)/);
  assert.match(quickstart, /rawDetailResponse/);
  assert.match(quickstart, /not\.toContain\(expectedConfirmationCode\)/);
  assert.match(quickstart, /Span metrics/);
  assert.match(quickstart, /Latency/);
});

test("gate2 confirmation code is fetched only after a browser span click", () => {
  const component = readFileSync(join(root, "app/Gate2Confirmation.tsx"), "utf8");
  assert.match(component, /"use client"/);
  assert.match(component, /sessionStorage\.setItem\(storageKey\(traceId, spanId\), "clicked"\)/);
  assert.match(component, /window\.dispatchEvent\(new CustomEvent<ClickDetail>\(CLICK_EVENT/);
  assert.match(component, /fetch\("\/api\/gate2\/confirm"/);
  assert.match(component, /x-beater-gate2-browser-click/);
  assert.match(component, /if \(status === "hidden"\) return null/);
  assert.match(component, />Confirm</);

  const route = readFileSync(join(root, "app/api/gate2/confirm/route.ts"), "utf8");
  assert.match(route, /BEATER_GATE2_CONFIRMATION_SALT/);
  assert.match(route, /gate2:\$\{salt\}:\$\{payload\.traceId\}:\$\{payload\.spanId\}/);
  assert.match(route, /cache-control/);
  assert.match(route, /x-beater-gate2-browser-click/);
});
