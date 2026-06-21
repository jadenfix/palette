import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

const root = new URL("..", import.meta.url).pathname;

test("dashboard page exposes the trace inspection surface", () => {
  const page = readFileSync(join(root, "app/page.tsx"), "utf8");
  assert.match(page, /Agent Trace Debugger/);
  assert.match(page, /Trace filters/);
  assert.match(page, /Agent span waterfall/);
  assert.match(page, /lucide-react/);
  assert.match(page, /kindIcon/);
  assert.match(page, /type KindIcon = \{ key: string; Icon: LucideIcon; title: string \}/);
  assert.match(page, /Activity/);
  assert.match(page, /BrainCircuit/);
  assert.match(page, /const KindGlyph = icon\.Icon/);
  assert.match(page, /data-depth/);
  assert.match(page, /data-kind/);
  assert.match(page, /data-status/);
  assert.match(page, /data-span-name/);
  assert.match(page, /data-icon/);
  assert.match(page, /<KindGlyph aria-hidden="true" \/>/);
  assert.match(page, /className="sr-only"/);
  assert.doesNotMatch(page, /label: "AI"/);
  assert.doesNotMatch(page, /label: "Fn"/);
  assert.match(page, /data-label="Spans"/);
  assert.match(page, /data-label="Latency"/);
  assert.match(page, /spanTimeline/);
  assert.match(page, /"--offset"/);
  assert.match(page, /span-track/);
  assert.match(page, /SpanDetail/);
  assert.match(page, /IoBlock/);
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
  assert.match(css, /\.query-chip/);
  assert.match(css, /\.filter-reset/);
  assert.match(css, /\.span-detail \{\n  align-self: start;/);
  assert.match(css, /\.run-state/);
  assert.match(css, /\.run-metrics/);
  assert.match(css, /\.run-row\[data-status="error"\]/);
  assert.match(css, /\.run-row\[data-outside-filters="true"\]/);
  assert.match(css, /\.run-filter-note/);
  assert.match(css, /\.run-cell::before/);
  assert.match(css, /\.timeline-axis/);
  assert.match(css, /\.axis-tick/);
  assert.match(css, /\.waterfall-head,\n\.span-line,\n\.timeline-axis \{/);
  assert.match(css, /\.span-line\[data-kind="llm\.call"\] \{/);
  assert.match(css, /--kind-color: var\(--accent\);/);
  assert.match(css, /\.span-line\[data-status="error"\]/);
  assert.match(css, /\.kind-icon svg \{/);
  assert.match(css, /\.detail-kind svg \{/);
  assert.doesNotMatch(css, /\.detail-tabs/);
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
  assert.match(api, /query,\n      runs,/);
  assert.doesNotMatch(api, /query: \{ \.\.\.query, traceId: activeTraceId/);
  assert.match(api, /BEATER_API_TOKEN/);
  assert.match(api, /x-beater-project-id/);
  assert.match(api, /x-beater-environment-id/);
});

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
  assert.match(recorder, /data-depth/);
  assert.match(recorder, /data-icon/);
  assert.match(recorder, /five-line-llm-call/);
  assert.match(recorder, /hello from stock OpenTelemetry/);
  assert.match(recorder, /color\/icon-coded all-kind agent waterfall/);
  assert.match(recorder, /gate2-browser-demo\.webm/);
  const quickstart = readFileSync(join(root, "tests/e2e/quickstart.spec.ts"), "utf8");
  assert.match(quickstart, /five-line-llm-call/);
  assert.match(quickstart, /gpt-quickstart/);
  assert.match(quickstart, /page\.goto\("\/\?tenant=demo&project=demo&environment=local&kind=llm\.call&model=gpt-quickstart"\)/);
  assert.match(quickstart, /toHaveValue\(""\)/);
  assert.match(quickstart, /placeholder", \/latest: \//);
  assert.match(quickstart, /toHaveCount\(1\)/);
  assert.doesNotMatch(quickstart, /waterfall\.getByText\("five-line-llm-call"\)\.click/);
  assert.match(quickstart, /traceRow\.click\(\)/);
  assert.match(quickstart, /toHaveURL/);
  assert.match(quickstart, /hello from stock OpenTelemetry/);
  assert.match(quickstart, /hello from Beater/);
  assert.match(quickstart, /data-icon/);
});
