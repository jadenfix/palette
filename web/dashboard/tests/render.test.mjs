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
  assert.match(page, /BrainCircuit/);
  assert.match(page, /const KindGlyph = icon\.Icon/);
  assert.match(page, /data-depth/);
  assert.match(page, /data-kind/);
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
  assert.match(page, /RedactionControls/);
  assert.match(page, /Unmask redacted I\/O/);
  assert.match(page, /name="unmask"/);
  assert.match(page, /name="reason"/);
  assert.match(page, /name="status"/);
  assert.match(page, /name="kind"/);
  assert.match(page, /name="started_after"/);
  assert.match(page, /name="model"/);
  assert.match(page, /name="release"/);
  assert.match(page, /name="min_cost_micros"/);
  assert.match(page, /name="min_latency_ms"/);
  assert.match(page, /human\.review/);
  assert.match(page, /replay\.run/);
  assert.match(page, /kind === "human\.review"/);
  assert.match(page, /kind === "replay\.run"/);
});

test("dashboard chrome stays dense and tool-like", () => {
  const css = readFileSync(join(root, "app/globals.css"), "utf8");
  assert.match(css, /--canvas: #f5f6f3/);
  assert.match(css, /\.workspace \{\n  display: grid;/);
  assert.match(css, /\.summary-strip \{\n  background: rgb\(255 255 255 \/ 0\.72\);/);
  assert.match(css, /grid-template-columns:\n    62px minmax\(180px, 1\.35fr\)/);
  assert.match(css, /\.run-cell::before/);
  assert.match(css, /\.waterfall-head,\n\.span-line \{/);
  assert.match(css, /\.span-line\[data-kind="llm\.call"\] \{/);
  assert.match(css, /--kind-color: var\(--teal\);/);
  assert.match(css, /\.kind-icon svg \{/);
  assert.match(css, /\.span-track \{/);
  assert.match(css, /left: var\(--offset\);/);
  assert.match(css, /var\(--kind-color\),/);
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
  const recorder = readFileSync(join(root, "tests/e2e/record-gate2-demo.mjs"), "utf8");
  assert.match(recorder, /recordVideo/);
  assert.match(recorder, /requireAttribute/);
  assert.match(recorder, /BEATER_GATE2_RECORD_MODE/);
  assert.match(recorder, /recordQuickstartFlow/);
  assert.match(recorder, /recordAllKindFlow/);
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
  assert.match(quickstart, /traceRow\.click\(\)/);
  assert.match(quickstart, /toHaveURL/);
  assert.match(quickstart, /hello from stock OpenTelemetry/);
  assert.match(quickstart, /hello from Beater/);
  assert.match(quickstart, /data-icon/);
});
