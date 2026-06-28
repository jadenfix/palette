import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const root = new URL("..", import.meta.url).pathname;

function loadTsModule(relativePath, { context = {}, requireMap = {} } = {}) {
  const source = readFileSync(join(root, relativePath), "utf8");
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
      require(specifier) {
        if (Object.hasOwn(requireMap, specifier)) {
          const value = requireMap[specifier];
          return typeof value === "function" ? value() : value;
        }
        throw new Error(`unexpected test import from ${relativePath}: ${specifier}`);
      },
      ...context
    },
    { filename: relativePath }
  );
  return module.exports;
}

function loadGate2SessionModule(context = {}) {
  return loadTsModule("lib/gate2-session.ts", { context });
}

function loadGate2ClickProofModule(context = {}) {
  return loadTsModule("lib/gate2-click-proof.ts", { context });
}

function loadGate2ConfirmationRequestModule(context = {}) {
  return loadTsModule("lib/gate2-confirmation-request.ts", {
    context,
    requireMap: {
      "./gate2-click-proof": () => loadGate2ClickProofModule(context)
    }
  });
}

function loadGate2ConfirmationContractModule(context = {}) {
  return loadTsModule("lib/gate2-confirmation-contract.ts", { context });
}

function loadGate2ConfirmationModule(context = {}) {
  return loadTsModule("lib/gate2-confirmation.ts", {
    context: { process, ...context },
    requireMap: {
      "node:crypto": { createHash },
      "./gate2-confirmation-contract": () => loadGate2ConfirmationContractModule(context)
    }
  });
}

function loadSpanKindsModule(context = {}) {
  return loadTsModule("lib/span-kinds.ts", { context });
}

function loadDashboardQueryModule(context = {}) {
  return loadTsModule("lib/dashboard-query.ts", {
    context: { URLSearchParams, ...context },
    requireMap: {}
  });
}

function loadDashboardApiModule(context = {}) {
  return loadTsModule("lib/api.ts", {
    context: { process, URLSearchParams, ...context },
    requireMap: {
      "./span-kinds": () => loadSpanKindsModule(context),
      "./dashboard-query": () => loadDashboardQueryModule(context)
    }
  });
}

function loadGate2ConfirmRouteModule(context = {}) {
  return loadTsModule("app/api/gate2/confirm/route.ts", {
    context: { process, Response, URL, ...context },
    requireMap: {
      "node:crypto": { createHash },
      "../../../../lib/gate2-click-proof": () => loadGate2ClickProofModule(context),
      "../../../../lib/gate2-confirmation": () => loadGate2ConfirmationModule(context),
      "../../../../lib/gate2-confirmation-request": () =>
        loadGate2ConfirmationRequestModule(context),
      "../../../../lib/gate2-session": () => loadGate2SessionModule(context)
    }
  });
}

function gate2ConfirmRequest({
  session = "0123456789abcdef0123456789abcdef",
  origin = "http://127.0.0.1:3000",
  host = "127.0.0.1:3000",
  fetchMetadata = true,
  body = gate2ConfirmBody()
} = {}) {
  const { GATE2_SESSION_COOKIE } = loadGate2SessionModule();
  const headers = new Map();
  if (origin) headers.set("origin", origin);
  if (host) headers.set("host", host);
  if (fetchMetadata) {
    headers.set("sec-fetch-site", "same-origin");
    headers.set("sec-fetch-mode", "cors");
    headers.set("sec-fetch-dest", "empty");
  }
  return {
    url: "http://localhost:3000/api/gate2/confirm",
    cookies: {
      get(name) {
        return name === GATE2_SESSION_COOKIE && session ? { value: session } : undefined;
      }
    },
    headers: {
      get(name) {
        return headers.get(name.toLowerCase()) ?? null;
      }
    },
    json: async () => body
  };
}

function gate2ConfirmBody({
  traceId = "00000000000000000000000000000000",
  spanId = "0000000000000000",
  nonce = "abcdef0123456789abcdef0123456789",
  capturedAtMs = Date.now(),
  click = {}
} = {}) {
  return {
    traceId,
    spanId,
    click: {
      nonce,
      capturedAtMs,
      isTrusted: true,
      button: 0,
      detail: 1,
      clientX: 10,
      clientY: 20,
      screenX: 30,
      screenY: 40,
      ...click
    }
  };
}

async function responseJson(response) {
  return {
    status: response.status,
    body: await response.json()
  };
}

test("dashboard page exposes the trace inspection surface", () => {
  const page = readFileSync(join(root, "app/page.tsx"), "utf8");
  const spanKinds = readFileSync(join(root, "lib/span-kinds.ts"), "utf8");
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
  assert.match(page, /TokenBreakdown/);
  assert.match(page, /aria-label="Token breakdown"/);
  assert.match(page, /className="token-chip"/);
  assert.match(page, /label: "Reasoning", value: span\.tokens\.reasoning/);
  assert.match(page, /label: "Cached", value: span\.tokens\.cache_read/);
  assert.doesNotMatch(page, /label: "AI"/);
  assert.doesNotMatch(page, /label: "Fn"/);
  assert.match(page, /data-label="Spans"/);
  assert.match(page, /data-label="Latency"/);
  assert.match(page, /spanTimeline/);
  assert.match(page, /"--offset"/);
  assert.match(page, /span-track/);
  assert.match(page, /SpanDetail/);
  assert.match(page, /IoBlock/);
  assert.match(page, /displaySpanIoLabels/);
  assert.match(page, /spanKindClass/);
  assert.match(page, /isLlmCallKind/);
  assert.match(page, /isRedactedIoValue/);
  assert.match(page, /ioVisibilityLabel/);
  assert.match(page, /Still redacted/);
  assert.match(page, /Unmask requested/);
  assert.match(page, /redacted by policy/);
  assert.doesNotMatch(page, />Unmasked</);
  assert.match(spanKinds, /LLM_CALL_SPAN_KIND = "llm\.call"/);
  assert.match(spanKinds, /input: "Prompt", output: "Completion"/);
  assert.match(spanKinds, /input: "prompt", output: "completion"/);
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
  assert.match(page, /spanKindMeta/);
  assert.match(spanKinds, /human\.review/);
  assert.match(spanKinds, /replay\.run/);
  assert.match(spanKinds, /key: "human"/);
  assert.match(spanKinds, /key: "replay"/);
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
  assert.match(proof, /seed-gate2-redaction-trace\.py/);
  assert.match(proof, /tests\/e2e\/redaction\.spec\.ts/);
  assert.match(proof, /BEATER_E2E_REDACTION_TRACE_ID/);
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
  assert.match(api, /const activeRun = query\.traceId/);
  assert.match(api, /const activeRunMatchesTrace = activeRun !== undefined && activeRun\.trace_id === activeTraceId/);
  assert.match(api, /activeRunMatchesTrace && activeRun\.project_id && !query\.projectId/);
  assert.match(api, /tracePath\(traceQuery, activeTraceId\)/);
  assert.match(api, /spanPath\(traceQuery, trace\.trace_id, activeSpanId\)/);
  assert.match(api, /query,\n\s+runs,/);
  assert.doesNotMatch(api, /query: \{ \.\.\.query, traceId: activeTraceId/);
  assert.match(api, /BEATER_API_TOKEN/);
  assert.match(api, /x-beater-project-id/);
  assert.match(api, /x-beater-environment-id/);
  assert.match(api, /formatApiError\(response\.status, response\.statusText/);
  assert.match(api, /let runs: RunSummaryPage;/);
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
    items: [{ tenant_id: "demo", project_id: "demo", trace_id: "trace-1", first_span_name: "run", span_count: 1 }],
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
    items: [{ tenant_id: "demo", project_id: "demo", trace_id: "trace-1", first_span_name: "run", span_count: 2 }],
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
    items: [{ tenant_id: "demo", project_id: "demo", trace_id: "trace-1", first_span_name: "run", span_count: 1 }],
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

test("dashboard loader scopes tenant-wide trace details to the selected run project", async () => {
  const runs = {
    items: [
      {
        tenant_id: "demo",
        project_id: "project-b",
        trace_id: "trace-1",
        first_span_name: "run",
        span_count: 1
      }
    ],
    next_cursor: null
  };
  const span = {
    ...spanFixture("span-1", null, "2026-01-01T00:00:00Z", 1),
    project_id: "project-b",
    trace_id: "trace-1",
    kind: "llm.call"
  };
  const trace = { tenant_id: "demo", trace_id: "trace-1", spans: [span] };
  const requests = [];
  const { loadDashboardData } = loadDashboardApiModule({
    fetch: async (url, init) => {
      const href = String(url);
      requests.push({ href, headers: init?.headers ?? {} });
      if (href.includes("/v1/traces/demo?")) return okJson(runs);
      if (href.includes("/v1/traces/demo/trace-1")) return okJson(trace);
      if (href.includes("/v1/spans/demo/trace-1/span-1/io")) {
        return okJson({ input: { kind: "missing" }, output: { kind: "missing" } });
      }
      if (href.includes("/v1/spans/demo/trace-1/span-1")) return okJson(span);
      throw new Error(`unexpected fetch ${href}`);
    }
  });

  const data = await loadDashboardData({ tenantId: "demo" });

  assert.equal(data.selectedSpan?.project_id, "project-b");
  const detailRequests = requests.filter(({ href }) => href.includes("/v1/traces/demo/trace-1") || href.includes("/v1/spans/demo/trace-1"));
  assert.equal(detailRequests.length, 3);
  assert.ok(
    detailRequests.every(
      ({ headers }) => headers["x-beater-project-id"] === "project-b"
    )
  );
});

test("dashboard loader does not scope explicit trace details to an unrelated fallback run", async () => {
  const runs = {
    items: [
      {
        tenant_id: "demo",
        project_id: "project-b",
        trace_id: "trace-2",
        first_span_name: "other-run",
        span_count: 1
      }
    ],
    next_cursor: null
  };
  const span = {
    ...spanFixture("span-1", null, "2026-01-01T00:00:00Z", 1),
    project_id: "project-a",
    trace_id: "trace-1",
    kind: "llm.call"
  };
  const trace = { tenant_id: "demo", trace_id: "trace-1", spans: [span] };
  const requests = [];
  const { loadDashboardData } = loadDashboardApiModule({
    fetch: async (url, init) => {
      const href = String(url);
      requests.push({ href, headers: init?.headers ?? {} });
      if (href.includes("/v1/traces/demo?")) return okJson(runs);
      if (href.includes("/v1/traces/demo/trace-1")) return okJson(trace);
      if (href.includes("/v1/spans/demo/trace-1/span-1/io")) {
        return okJson({ input: { kind: "missing" }, output: { kind: "missing" } });
      }
      if (href.includes("/v1/spans/demo/trace-1/span-1")) return okJson(span);
      throw new Error(`unexpected fetch ${href}`);
    }
  });

  const data = await loadDashboardData({ tenantId: "demo", traceId: "trace-1" });

  assert.equal(data.selectedSpan?.project_id, "project-a");
  const detailRequests = requests.filter(({ href }) => href.includes("/v1/traces/demo/trace-1") || href.includes("/v1/spans/demo/trace-1"));
  assert.equal(detailRequests.length, 3);
  assert.ok(
    detailRequests.every(
      ({ headers }) => headers["x-beater-project-id"] === undefined
    )
  );
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
    tenant_id: "demo",
    project_id: "demo",
    environment_id: "local",
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
  assert.match(generated, /listTraces/);
  assert.match(generated, /started_after/);
  assert.match(generated, /min_cost_micros/);
});

test("in-app docs runtime surfaces are generated from the openapi contract", () => {
  const spec = JSON.parse(readFileSync(join(root, "openapi/beater-read-api.json"), "utf8"));
  const operationIds = Object.values(spec.paths)
    .flatMap((methods) => Object.values(methods))
    .map((operation) => operation.operationId)
    .filter(Boolean);

  assert.ok(operationIds.includes("listTraces"));
  assert.ok(operationIds.includes("createDataset"));

  const openapiRoute = readFileSync(join(root, "app/api/openapi/route.ts"), "utf8");
  assert.match(openapiRoute, /openapi", "beater-read-api\.json"/);
  assert.match(openapiRoute, /readFile\(specPath, "utf8"\)/);
  assert.match(openapiRoute, /"content-type": "application\/json"/);

  const docsPage = readFileSync(join(root, "app/docs/page.tsx"), "utf8");
  assert.match(docsPage, /setAttribute\("data-url", "\/api\/openapi"\)/);
  assert.match(docsPage, /@scalar\/api-reference/);
  assert.doesNotMatch(docsPage, /beater-read-api\.json/);

  const mcpPage = readFileSync(join(root, "app/docs/mcp/page.tsx"), "utf8");
  assert.match(mcpPage, /fetch\("\/api\/openapi"\)/);
  assert.match(mcpPage, /spec\.paths/);
  assert.match(mcpPage, /op\?\.operationId/);
  assert.match(mcpPage, /name: op\.operationId/);
  assert.match(mcpPage, /tool name = operationId/);

  const quickstartsPage = readFileSync(join(root, "app/docs/quickstarts/page.tsx"), "utf8");
  assert.match(quickstartsPage, /href="\/docs"/);
  assert.match(quickstartsPage, /one OpenAPI contract/);
  assert.match(quickstartsPage, /MCP server/);
  assert.match(quickstartsPage, /tools\/list/);
  assert.match(quickstartsPage, /CLI/);
  assert.match(quickstartsPage, /Control-plane clients \(7 languages\)/);
});

test("browser proof covers all canonical span kinds and can record a demo", () => {
  const e2e = readFileSync(join(root, "tests/e2e/dashboard.spec.ts"), "utf8");
  const spanKinds = loadSpanKindsModule();
  const tokenBreakdown = readFileSync(join(root, "tests/e2e/token-breakdown.ts"), "utf8");
  assert.match(tokenBreakdown, /export async function expectTokenBreakdown/);
  assert.match(tokenBreakdown, /toHaveCount\(expected\.length\)/);
  assert.match(tokenBreakdown, /expect\(actual\)\.toEqual\(expected\)/);
  assert.match(e2e, /import \{ expectTokenBreakdown \} from "\.\/token-breakdown"/);
  assert.doesNotMatch(e2e, /async function expectTokenBreakdown/);
  for (const kind of spanKinds.AGENT_SPAN_KINDS) {
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
  assert.match(recorder, /BEATER_E2E_QUICKSTART_RELEASE/);
  assert.match(recorder, /recordQuickstartFlow/);
  assert.match(recorder, /recordAllKindFlow/);
  assert.match(recorder, /recordRedactionFlow/);
  assert.match(recorder, /BEATER_E2E_REDACTION_TRACE_ID/);
  assert.match(recorder, /redactionUnmaskReason = "gate2-redaction-review"/);
  assert.match(recorder, /quickstartNotes/);
  assert.match(recorder, /Quickstart release ID/);
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
  assert.match(recorder, /waitForTokenBreakdown/);
  assert.match(recorder, /filter\(\{ hasText: "Prompt" \}\)/);
  assert.match(recorder, /filter\(\{ hasText: "Completion" \}\)/);
  assert.match(recorder, /hello from stock OpenTelemetry/);
  assert.match(recorder, /color\/icon-coded all-kind agent waterfall/);
  assert.match(recorder, /redacted prompt\/completion/);
  assert.match(recorder, /Redacted view/);
  assert.match(recorder, /gate2-browser-demo\.webm/);
  const redaction = readFileSync(join(root, "tests/e2e/redaction.spec.ts"), "utf8");
  assert.match(redaction, /BEATER_E2E_REDACTION_TRACE_ID/);
  assert.match(redaction, /BEATER_E2E_REDACTION_SPAN_ID/);
  assert.match(redaction, /gpt-redaction/);
  assert.match(redaction, /gate2-redaction-review/);
  assert.match(redaction, /not\.toContainText\(rawCard\)/);
  assert.match(redaction, /Redacted view/);
  const quickstart = readFileSync(join(root, "tests/e2e/quickstart.spec.ts"), "utf8");
  assert.match(quickstart, /BEATER_E2E_QUICKSTART_TRACE_ID/);
  assert.match(quickstart, /import \{ expectTokenBreakdown \} from "\.\/token-breakdown"/);
  assert.doesNotMatch(quickstart, /async function expectTokenBreakdown/);
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
  assert.match(quickstart, /gate2ConfirmationCode\(\{/);
  assert.doesNotMatch(quickstart, /function confirmationCode/);
  assert.match(quickstart, /rawDetailResponse/);
  assert.match(quickstart, /not\.toContain\(expectedConfirmationCode\)/);
  assert.match(quickstart, /Span metrics/);
  assert.match(quickstart, /Latency/);
});

test("gate2 confirmation code is fetched only after a browser span click", () => {
  const component = readFileSync(join(root, "app/Gate2Confirmation.tsx"), "utf8");
  assert.match(component, /"use client"/);
  assert.match(component, /event\.isTrusted/);
  assert.match(component, /event\.button !== 0/);
  assert.match(component, /event\.detail < 1/);
  assert.match(component, /randomHex\(16\)/);
  assert.match(component, /GATE2_CONFIRMATION_CODE/);
  assert.match(component, /isBrowserClickProof/);
  assert.match(component, /sessionStorage\.setItem\(storageKey\(traceId, spanId\), JSON\.stringify\(click\)\)/);
  assert.match(component, /new CustomEvent<ClickDetail>\(CLICK_EVENT/);
  assert.match(component, /fetch\("\/api\/gate2\/confirm"/);
  assert.match(component, /requestedNonce/);
  assert.match(component, /aria-live="polite"/);
  assert.match(component, /: "pending"/);
  assert.match(component, />Confirm</);

  const route = readFileSync(join(root, "app/api/gate2/confirm/route.ts"), "utf8");
  const confirmation = readFileSync(join(root, "lib/gate2-confirmation.ts"), "utf8");
  const confirmationContract = readFileSync(join(root, "lib/gate2-confirmation-contract.ts"), "utf8");
  const clickProof = readFileSync(join(root, "lib/gate2-click-proof.ts"), "utf8");
  const confirmationRequest = readFileSync(join(root, "lib/gate2-confirmation-request.ts"), "utf8");
  const session = readFileSync(join(root, "lib/gate2-session.ts"), "utf8");
  assert.match(route, /gate2ConfirmationCode/);
  assert.match(route, /isGate2ConfirmationRequest/);
  assert.doesNotMatch(route, /isBrowserClickProof/);
  assert.match(route, /cache-control/);
  assert.match(route, /GATE2_SESSION_COOKIE/);
  assert.match(route, /isGate2SessionId/);
  assert.match(route, /sec-fetch-site/);
  assert.match(route, /USED_NONCES/);
  assert.match(route, /browser click proof expired/);
  assert.match(clickProof, /record\.button === 0/);
  assert.match(clickProof, /record\.detail >= 1/);
  assert.match(clickProof, /GATE2_CLICK_PROOF_NONCE = \/\^\[0-9a-f\]\{32\}\$\//);
  assert.match(confirmationRequest, /Gate2ConfirmationRequest/);
  assert.match(confirmationRequest, /isGate2ConfirmationRequest/);
  assert.match(confirmationRequest, /isBrowserClickProof\(record\.click\)/);
  assert.match(confirmationRequest, /GATE2_TRACE_ID = \/\^\[0-9a-f\]\{32\}\$\//);
  assert.match(confirmationRequest, /GATE2_SPAN_ID = \/\^\[0-9a-f\]\{16\}\$\//);
  assert.match(confirmationContract, /GATE2_CONFIRMATION_SALT_ENV = "BEATER_GATE2_CONFIRMATION_SALT"/);
  assert.match(confirmationContract, /GATE2_CONFIRMATION_HASH_PREFIX = "gate2"/);
  assert.match(confirmationContract, /GATE2_CONFIRMATION_CODE = \/\^\[0-9A-F\]\{8\}\$\//);
  assert.match(confirmation, /gate2ConfirmationCode/);
  assert.match(session, /GATE2_SESSION_COOKIE = "beater_gate2_session"/);
  assert.match(session, /GATE2_SESSION_MAX_AGE_SECONDS = 60 \* 60/);
  assert.match(session, /isGate2SessionId/);
  const css = readFileSync(join(root, "app/globals.css"), "utf8");
  assert.match(css, /\.confirmation-code\[data-confirmation-status="hidden"\] dd/);
  assert.match(css, /\.confirmation-code\[data-confirmation-status="ready"\] dd/);
  assert.match(css, /\.confirmation-code\[data-confirmation-status="error"\] dd/);
  const proxy = readFileSync(join(root, "proxy.ts"), "utf8");
  assert.match(proxy, /export function proxy/);
  assert.match(proxy, /GATE2_SESSION_COOKIE/);
  assert.match(proxy, /GATE2_SESSION_MAX_AGE_SECONDS/);
  assert.match(proxy, /isGate2SessionId/);
  assert.match(proxy, /httpOnly: true/);
});

test("gate2 confirmation route rejects non-browser requests and nonce replay", async () => {
  const previousSalt = process.env.BEATER_GATE2_CONFIRMATION_SALT;
  process.env.BEATER_GATE2_CONFIRMATION_SALT = "unit-salt";
  try {
    const { POST } = loadGate2ConfirmRouteModule();

    assert.deepEqual(
      await responseJson(await POST(gate2ConfirmRequest({ session: "" }))),
      {
        status: 403,
        body: { error: "missing browser session" }
      }
    );
    assert.deepEqual(
      await responseJson(await POST(gate2ConfirmRequest({ fetchMetadata: false }))),
      {
        status: 403,
        body: { error: "missing browser fetch metadata" }
      }
    );
    assert.deepEqual(
      await responseJson(await POST(gate2ConfirmRequest({ body: { traceId: "bad" } }))),
      {
        status: 400,
        body: { error: "traceId, spanId, and browser click proof are required" }
      }
    );
    const keyboardClickBody = gate2ConfirmBody({ click: { detail: 0 } });
    assert.deepEqual(await responseJson(await POST(gate2ConfirmRequest({ body: keyboardClickBody }))), {
      status: 400,
      body: { error: "traceId, spanId, and browser click proof are required" }
    });
    const nonPrimaryClickBody = gate2ConfirmBody({ click: { button: 1 } });
    assert.deepEqual(await responseJson(await POST(gate2ConfirmRequest({ body: nonPrimaryClickBody }))), {
      status: 400,
      body: { error: "traceId, spanId, and browser click proof are required" }
    });

    const body = gate2ConfirmBody();
    const { gate2ConfirmationCode } = loadGate2ConfirmationModule();
    const expectedCode = gate2ConfirmationCode({
      salt: "unit-salt",
      traceId: body.traceId,
      spanId: body.spanId
    });
    assert.deepEqual(
      await responseJson(await POST(gate2ConfirmRequest({ body }))),
      {
        status: 200,
        body: { code: expectedCode }
      }
    );
    assert.deepEqual(
      await responseJson(await POST(gate2ConfirmRequest({ body }))),
      {
        status: 409,
        body: { error: "browser click proof was already used" }
      }
    );
  } finally {
    if (previousSalt === undefined) {
      delete process.env.BEATER_GATE2_CONFIRMATION_SALT;
    } else {
      process.env.BEATER_GATE2_CONFIRMATION_SALT = previousSalt;
    }
  }
});

test("gate2 recorder confirmation helper matches the app helper", async () => {
  const recorder = await import("./e2e/gate2-confirmation-code.mjs");
  const confirmation = loadGate2ConfirmationModule();
  const vector = recorder.GATE2_CONFIRMATION_TEST_VECTOR;

  assert.equal(recorder.GATE2_CONFIRMATION_HASH_PREFIX, confirmation.GATE2_CONFIRMATION_HASH_PREFIX);
  assert.equal(recorder.gate2ConfirmationCode(vector), vector.code);
  assert.equal(
    confirmation.gate2ConfirmationCode({
      salt: vector.salt,
      traceId: vector.traceId,
      spanId: vector.spanId
    }),
    vector.code
  );
});

// ── dashboard-query round-trip tests ─────────────────────────────────────────

test("dashboard query field table covers every filter field exactly once", () => {
  const { FILTER_FIELDS } = loadDashboardQueryModule();

  // Canonical list of all filter fields (URL param name → DashboardQuery key)
  const expected = [
    { field: "status",         urlParam: "status" },
    { field: "kind",           urlParam: "kind" },
    { field: "model",          urlParam: "model" },
    { field: "release",        urlParam: "release" },
    { field: "startedAfter",   urlParam: "started_after" },
    { field: "startedBefore",  urlParam: "started_before" },
    { field: "minCostMicros",  urlParam: "min_cost_micros" },
    { field: "maxCostMicros",  urlParam: "max_cost_micros" },
    { field: "minLatencyMs",   urlParam: "min_latency_ms" },
    { field: "maxLatencyMs",   urlParam: "max_latency_ms" },
  ];

  assert.equal(FILTER_FIELDS.length, expected.length, "FILTER_FIELDS length mismatch");
  for (const { field, urlParam } of expected) {
    const d = FILTER_FIELDS.find((d) => d.field === field);
    assert.ok(d, `missing descriptor for field "${field}"`);
    assert.equal(d.urlParam, urlParam, `wrong urlParam for field "${field}"`);
  }
});

test("dashboard query parse → API params round-trip for every filter field", () => {
  const { parseQueryFromSearchParams, applyFilterParams } = loadDashboardQueryModule();

  // Build a URL params record with every filter field set
  const rawParams = {
    tenant: "acme",
    project: "myproj",
    environment: "prod",
    trace: "trace-abc",
    span: "span-xyz",
    status: "error",
    kind: "agent.run",
    model: "openai/gpt-4o",
    release: "v1.2.3",
    started_after: "2026-01-01T00:00:00Z",
    started_before: "2026-06-01T00:00:00Z",
    min_cost_micros: "500",
    max_cost_micros: "9999",
    min_latency_ms: "100",
    max_latency_ms: "5000",
    unmask: "true",
    reason: "incident-007",
  };

  const query = parseQueryFromSearchParams(rawParams);

  // Scope + selection + unmask fields
  assert.equal(query.tenantId, "acme");
  assert.equal(query.projectId, "myproj");
  assert.equal(query.environmentId, "prod");
  assert.equal(query.traceId, "trace-abc");
  assert.equal(query.selectedSpanId, "span-xyz");
  assert.equal(query.unmask, true);
  assert.equal(query.unmaskReason, "incident-007");

  // Filter fields
  assert.equal(query.status, "error");
  assert.equal(query.kind, "agent.run");
  assert.equal(query.model, "openai/gpt-4o");
  assert.equal(query.release, "v1.2.3");
  assert.equal(query.startedAfter, "2026-01-01T00:00:00Z");
  assert.equal(query.startedBefore, "2026-06-01T00:00:00Z");
  assert.equal(query.minCostMicros, 500);
  assert.equal(query.maxCostMicros, 9999);
  assert.equal(query.minLatencyMs, 100);
  assert.equal(query.maxLatencyMs, 5000);

  // Serialize back to API params (applyFilterParams only touches filter fields)
  const apiParams = new URLSearchParams();
  applyFilterParams(query, apiParams);

  assert.equal(apiParams.get("status"), "error");
  assert.equal(apiParams.get("kind"), "agent.run");
  assert.equal(apiParams.get("model"), "openai/gpt-4o");
  assert.equal(apiParams.get("release"), "v1.2.3");
  assert.equal(apiParams.get("started_after"), "2026-01-01T00:00:00Z");
  assert.equal(apiParams.get("started_before"), "2026-06-01T00:00:00Z");
  assert.equal(apiParams.get("min_cost_micros"), "500");
  assert.equal(apiParams.get("max_cost_micros"), "9999");
  assert.equal(apiParams.get("min_latency_ms"), "100");
  assert.equal(apiParams.get("max_latency_ms"), "5000");
});

test("dashboard query parse → href params round-trip for every filter field", () => {
  const { parseQueryFromSearchParams, applyFilterParams } = loadDashboardQueryModule();

  const rawParams = {
    tenant: "t1",
    project: "p1",
    environment: "staging",
    status: "ok",
    kind: "llm.call",
    model: "anthropic/claude-3-5-sonnet",
    release: "r42",
    started_after: "2026-03-01T00:00:00Z",
    started_before: "2026-03-31T00:00:00Z",
    min_cost_micros: "0",
    max_cost_micros: "1000000",
    min_latency_ms: "50",
    max_latency_ms: "2000",
  };

  const query = parseQueryFromSearchParams(rawParams);

  // Simulate hrefFor: scope fields first, then applyFilterParams
  const hrefParams = new URLSearchParams();
  hrefParams.set("tenant", query.tenantId);
  if (query.projectId) hrefParams.set("project", query.projectId);
  if (query.environmentId) hrefParams.set("environment", query.environmentId);
  applyFilterParams(query, hrefParams);

  assert.equal(hrefParams.get("tenant"), "t1");
  assert.equal(hrefParams.get("project"), "p1");
  assert.equal(hrefParams.get("environment"), "staging");
  assert.equal(hrefParams.get("status"), "ok");
  assert.equal(hrefParams.get("kind"), "llm.call");
  assert.equal(hrefParams.get("model"), "anthropic/claude-3-5-sonnet");
  assert.equal(hrefParams.get("release"), "r42");
  assert.equal(hrefParams.get("started_after"), "2026-03-01T00:00:00Z");
  assert.equal(hrefParams.get("started_before"), "2026-03-31T00:00:00Z");
  assert.equal(hrefParams.get("min_cost_micros"), "0");
  assert.equal(hrefParams.get("max_cost_micros"), "1000000");
  assert.equal(hrefParams.get("min_latency_ms"), "50");
  assert.equal(hrefParams.get("max_latency_ms"), "2000");
});

test("dashboard query filterChips returns correct labels and display values", () => {
  const { parseQueryFromSearchParams, filterChips } = loadDashboardQueryModule();

  const query = parseQueryFromSearchParams({
    trace: "abcdef123456789012345678",
    status: "error",
    kind: "llm.call",
    model: "gpt-4o",
    release: "v1",
    started_after: "2026-01-01T00:00:00Z",
    started_before: "2026-06-01T00:00:00Z",
    min_cost_micros: "100",
    max_cost_micros: "200",
    min_latency_ms: "50",
    max_latency_ms: "2000",
  });

  const chips = filterChips(query);
  const byLabel = Object.fromEntries(chips.map((c) => [c.label, c.value]));

  // traceId uses shortHash
  assert.ok(byLabel["Trace"].includes("..."), "traceId chip uses shortHash");
  // plain string fields
  assert.equal(byLabel["Status"], "error");
  assert.equal(byLabel["Kind"], "llm.call");
  assert.equal(byLabel["Model"], "gpt-4o");
  assert.equal(byLabel["Release"], "v1");
  assert.equal(byLabel["After"], "2026-01-01T00:00:00Z");
  assert.equal(byLabel["Before"], "2026-06-01T00:00:00Z");
  // numeric fields: raw string for cost, " ms" suffix for latency
  assert.equal(byLabel["Min cost"], "100");
  assert.equal(byLabel["Max cost"], "200");
  assert.equal(byLabel["Min latency"], "50 ms");
  assert.equal(byLabel["Max latency"], "2000 ms");
  assert.equal(chips.length, 11, "one chip per active filter field + traceId");
});

test("dashboard query filterChips returns empty when no filters set", () => {
  const { parseQueryFromSearchParams, filterChips } = loadDashboardQueryModule();
  const query = parseQueryFromSearchParams({ tenant: "demo", project: "demo", environment: "local" });
  const chips = filterChips(query);
  // Use .length check rather than deepEqual — chips is a VM-context array
  // (cross-realm Array prototype), so deepStrictEqual would fail.
  assert.equal(chips.length, 0, "no chips when no filters set");
});

test("dashboard query advancedFilterCount counts only advanced fields", () => {
  const { parseQueryFromSearchParams, advancedFilterCount } = loadDashboardQueryModule();

  // status and kind are primary (not advanced), so they should not be counted
  const primaryOnly = parseQueryFromSearchParams({ status: "error", kind: "llm.call" });
  assert.equal(advancedFilterCount(primaryOnly), 0);

  // All 8 advanced filter fields set
  const allAdvanced = parseQueryFromSearchParams({
    model: "gpt-4o",
    release: "v1",
    started_after: "2026-01-01T00:00:00Z",
    started_before: "2026-06-01T00:00:00Z",
    min_cost_micros: "100",
    max_cost_micros: "200",
    min_latency_ms: "50",
    max_latency_ms: "2000",
  });
  assert.equal(advancedFilterCount(allAdvanced), 8);

  // Empty query
  const empty = parseQueryFromSearchParams({});
  assert.equal(advancedFilterCount(empty), 0);
});

test("dashboard query parseQueryFromSearchParams uses correct defaults", () => {
  const { parseQueryFromSearchParams } = loadDashboardQueryModule();

  const query = parseQueryFromSearchParams({});
  assert.equal(query.tenantId, "demo");
  assert.equal(query.projectId, "demo");
  assert.equal(query.environmentId, "local");
  assert.equal(query.traceId, undefined);
  assert.equal(query.selectedSpanId, undefined);
  assert.equal(query.unmask, undefined);
  assert.equal(query.unmaskReason, undefined);
  assert.equal(query.status, undefined);
  assert.equal(query.kind, undefined);
});

test("dashboard query parseQueryFromSearchParams handles string-array params (multi-value)", () => {
  const { parseQueryFromSearchParams } = loadDashboardQueryModule();

  // Next.js may pass repeated params as string[]; first value wins
  const query = parseQueryFromSearchParams({
    tenant: ["acme", "other"],
    status: ["ok", "error"],
  });
  assert.equal(query.tenantId, "acme");
  assert.equal(query.status, "ok");
});

test("dashboard query searchParamsForTraceList includes all filter fields", () => {
  const { searchParamsForTraceList } = loadDashboardApiModule();
  const query = {
    tenantId: "demo",
    projectId: "proj",
    environmentId: "prod",
    traceId: "t1",
    status: "error",
    kind: "llm.call",
    model: "gpt-4o",
    release: "v1",
    startedAfter: "2026-01-01T00:00:00Z",
    startedBefore: "2026-06-01T00:00:00Z",
    minCostMicros: 100,
    maxCostMicros: 200,
    minLatencyMs: 50,
    maxLatencyMs: 2000,
  };
  const params = searchParamsForTraceList(query);

  // Scope/selection → different API param names
  assert.equal(params.get("project_id"), "proj");
  assert.equal(params.get("environment_id"), "prod");
  assert.equal(params.get("trace_id"), "t1");
  // Filter fields → same names in URL and API
  assert.equal(params.get("status"), "error");
  assert.equal(params.get("kind"), "llm.call");
  assert.equal(params.get("model"), "gpt-4o");
  assert.equal(params.get("release"), "v1");
  assert.equal(params.get("started_after"), "2026-01-01T00:00:00Z");
  assert.equal(params.get("started_before"), "2026-06-01T00:00:00Z");
  assert.equal(params.get("min_cost_micros"), "100");
  assert.equal(params.get("max_cost_micros"), "200");
  assert.equal(params.get("min_latency_ms"), "50");
  assert.equal(params.get("max_latency_ms"), "2000");
  assert.equal(params.get("limit"), "50");
});

test("dashboard query module is table-driven and delegates contract to api.ts", () => {
  const dq = readFileSync(join(root, "lib/dashboard-query.ts"), "utf8");
  const api = readFileSync(join(root, "lib/api.ts"), "utf8");

  // dashboard-query.ts imports DashboardQuery as a type-only reference from api.ts
  // so the OpenAPI contract (TraceListQuery / TraceReadQuery) stays in one place.
  assert.match(dq, /import type \{ DashboardQuery \} from "\.\/api"/, "DashboardQuery sourced from api.ts via type-only import");

  // api.ts delegates filter-field serialization to dashboard-query.ts
  assert.match(api, /dashboard-query/, "api.ts imports from dashboard-query");
  assert.match(api, /applyFilterParams/, "api.ts calls applyFilterParams");

  // Table-driven exports are all present
  assert.match(dq, /FILTER_FIELDS/, "exports FILTER_FIELDS table");
  assert.match(dq, /parseQueryFromSearchParams/, "exports parseQueryFromSearchParams");
  assert.match(dq, /applyFilterParams/, "exports applyFilterParams");
  assert.match(dq, /filterChips/, "exports filterChips");
  assert.match(dq, /advancedFilterCount/, "exports advancedFilterCount");
  assert.match(dq, /shortHash/, "exports shortHash");
});
