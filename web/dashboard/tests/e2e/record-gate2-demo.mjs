import { chromium } from "@playwright/test";
import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const dashboardRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const repoRoot = resolve(dashboardRoot, "../..");
const baseUrl = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:3000";
const publicDashboardBase = process.env.BEATER_GATE2_PUBLIC_DASHBOARD_BASE ?? baseUrl;
const mode = process.env.BEATER_GATE2_RECORD_MODE ?? "all-kind";
const allKindTraceId =
  process.env.BEATER_E2E_ALL_KIND_TRACE_ID ??
  (mode === "all-kind" ? process.env.BEATER_E2E_TRACE_ID : undefined);
const quickstartTraceId =
  process.env.BEATER_E2E_QUICKSTART_TRACE_ID ??
  (mode === "quickstart" ? process.env.BEATER_E2E_TRACE_ID : undefined);
const demoDir = resolve(repoRoot, "docs/demos");
const scratchDir = resolve(dashboardRoot, "test-results/gate2-demo-video");
const videoPath = outputPath(
  process.env.BEATER_GATE2_RECORD_VIDEO,
  mode === "compose" ? "gate2-compose-browser-demo.webm" : "gate2-browser-demo.webm"
);
const notesPath = outputPath(
  process.env.BEATER_GATE2_RECORD_NOTES,
  mode === "compose" ? "gate2-compose-browser-demo.md" : "gate2-browser-demo.md"
);

await rm(scratchDir, { force: true, recursive: true });
await mkdir(scratchDir, { recursive: true });
await mkdir(demoDir, { recursive: true });

const browser = await chromium.launch();
const context = await browser.newContext({
  recordVideo: {
    dir: scratchDir,
    size: { width: 1440, height: 1000 }
  },
  viewport: { width: 1440, height: 1000 }
});
const page = await context.newPage();

if (mode === "compose") {
  await recordQuickstartFlow(page);
  await recordAllKindFlow(page);
} else if (mode === "quickstart") {
  await recordQuickstartFlow(page);
} else if (mode === "all-kind") {
  await recordAllKindFlow(page);
} else {
  throw new Error(`unknown BEATER_GATE2_RECORD_MODE '${mode}'`);
}

const video = page.video();
await context.close();
await browser.close();

if (!video) {
  throw new Error("Playwright did not produce a video artifact");
}

await copyFile(await video.path(), videoPath);
const videoSha256 = createHash("sha256").update(await readFile(videoPath)).digest("hex");
await writeFile(
  notesPath,
  mode === "compose"
    ? composeNotes(videoSha256)
    : mode === "quickstart"
      ? quickstartNotes(videoSha256)
      : allKindNotes(videoSha256)
);

console.log(`Recorded Gate 2 browser demo to ${videoPath}`);

function outputPath(value, fallbackName) {
  if (!value) return join(demoDir, fallbackName);
  return resolve(repoRoot, value);
}

async function recordQuickstartFlow(page) {
  await page.goto(
    `${baseUrl}/?tenant=demo&project=demo&environment=local&kind=llm.call&model=gpt-quickstart`
  );
  await page.getByRole("heading", { name: "Agent Trace Debugger" }).waitFor();
  const traceList = page.getByLabel("Traces");
  await traceList.getByText("five-line-llm-call").waitFor();
  await traceList.getByText("openai/gpt-quickstart").waitFor();
  const traceRow = quickstartTraceId
    ? traceList.locator(`a.run-row[href*="trace=${encodeURIComponent(quickstartTraceId)}"]`)
    : traceList.getByRole("link").filter({ hasText: "five-line-llm-call" }).first();
  await traceRow.click();
  const waterfall = page.getByLabel("Agent span waterfall");
  const llm = waterfall.locator('[data-span-name="five-line-llm-call"]');
  await requireAttribute(llm, "data-kind", "llm.call");
  await requireAttribute(llm, "data-depth", "0");
  await requireAttribute(llm.locator(".kind-icon"), "data-icon", "llm");
  await llm.click();
  const detail = page.getByLabel("Span detail");
  await detail.getByText("openai/gpt-quickstart").waitFor();
  await detail.getByText("USD 0.001200").waitFor();
  await detail
    .locator(".io")
    .filter({ hasText: "Input" })
    .getByText("hello from stock OpenTelemetry")
    .waitFor();
  await detail
    .locator(".io")
    .filter({ hasText: "Output" })
    .getByText("hello from Beater")
    .waitFor();
  await page.waitForTimeout(1000);
}

async function recordAllKindFlow(page) {
  const traceParam = allKindTraceId
    ? `&trace=${encodeURIComponent(allKindTraceId)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";
  await page.goto(`${baseUrl}/?tenant=demo&project=demo&environment=local${traceParam}`);
  await page.getByRole("heading", { name: "Agent Trace Debugger" }).waitFor();
  const waterfall = page.getByLabel("Agent span waterfall");
  await requireAttribute(waterfall.locator('[data-span-name="refund-agent-run"]'), "data-depth", "0");
  await requireAttribute(
    waterfall.locator('[data-span-name="customer-refund-turn"]'),
    "data-depth",
    "1"
  );
  await requireAttribute(
    waterfall.locator('[data-span-name="execute-refund-step"]'),
    "data-depth",
    "2"
  );
  await requireAttribute(waterfall.locator('[data-span-name="lookup-order-tool"]'), "data-depth", "3");
  await requireAttribute(waterfall.locator('[data-span-name="mcp-order-service"]'), "data-depth", "4");
  await requireAttribute(
    waterfall.locator('[data-span-name="call-policy-model"] .kind-icon'),
    "data-icon",
    "llm"
  );
  await requireAttribute(
    waterfall.locator('[data-span-name="mcp-order-service"] .kind-icon'),
    "data-icon",
    "mcp"
  );
  await waterfall.getByText("call-policy-model").click();
  const detail = page.getByLabel("Span detail");
  await detail
    .locator(".io")
    .filter({ hasText: "Input" })
    .getByText("Can this order be refunded after 31 days?")
    .waitFor();
  await page.waitForTimeout(1000);
  await page.getByLabel("Agent span waterfall").getByText("lookup-order-tool").click();
  await detail.locator(".io").filter({ hasText: "Input" }).getByText("ord_123").waitFor();
  await page.waitForTimeout(1000);
}

function allKindNotes(videoSha256) {
  const traceParam = allKindTraceId
    ? `&trace=${encodeURIComponent(allKindTraceId)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";
  return `# Gate 2 Browser Demo

Recorded from the stock OpenTelemetry Python trace produced by \`examples/python/otel_smoke.py\`.

- Artifact: \`gate2-browser-demo.webm\`
- SHA256: \`${videoSha256}\`
- Dashboard: \`${baseUrl}/?tenant=demo&project=demo&environment=local${traceParam}\`
- Shows: trace table, color/icon-coded all-kind agent waterfall, run -> turn -> step -> tool -> MCP nesting, \`llm.call\` prompt/completion/model/tokens/cost/latency, and tool-call I/O.

Regenerate with:

\`\`\`bash
BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh
\`\`\`

For the Docker Compose stopwatch proof that uses the literal five-line snippet,
run the prebuilt-image path:

\`\`\`bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
\`\`\`

For a local source build measurement, add \`BEATER_GATE2_LOCAL_BUILD=1\`.
`;
}

function quickstartNotes(videoSha256) {
  const traceParam = quickstartTraceId
    ? `&trace=${encodeURIComponent(quickstartTraceId)}`
    : "&kind=llm.call&model=gpt-quickstart";
  return `# Gate 2 Browser Demo

Recorded from the literal five-line stock OpenTelemetry quickstart trace.

- Artifact: \`gate2-browser-demo.webm\`
- SHA256: \`${videoSha256}\`
- Dashboard: \`${baseUrl}/?tenant=demo&project=demo&environment=local${traceParam}\`
- Shows: trace table, click five-line trace, click \`llm.call\` span, read prompt, completion, model, tokens, cost, and latency.

Regenerate with:

\`\`\`bash
BEATER_E2E_TRACE_ID=<quickstart-trace-id> BEATER_GATE2_RECORD_MODE=quickstart npm run record:gate2
\`\`\`

For the Docker Compose stopwatch proof that records the full quickstart plus
all-kind waterfall flow, run the prebuilt-image path:

\`\`\`bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
\`\`\`
`;
}

function composeNotes(videoSha256) {
  const quickstartTrace = quickstartTraceId ?? "latest matching quickstart trace";
  const allKindTrace = allKindTraceId ?? "latest matching all-kind trace";
  const defaultDashboardBase = "http://127.0.0.1:3000";
  const portNote =
    publicDashboardBase === defaultDashboardBase
      ? `This automated maintainer run used the default dashboard URL \`${defaultDashboardBase}\`; no alternate host ports were needed.`
      : `This automated maintainer run used alternate host ports; the outside-person proof must still use the default dashboard URL \`${defaultDashboardBase}\`.`;

  return `# Gate 2 Compose Browser Demo

Recorded from the Docker Compose stopwatch path using the literal five-line
stock OpenTelemetry quickstart and the all-kind stock OpenTelemetry agent trace.

- Artifact: \`gate2-compose-browser-demo.webm\`
- SHA256: \`${videoSha256}\`
- Dashboard base: \`${publicDashboardBase}\`
- Quickstart trace: \`${quickstartTrace}\`
- All-kind trace: \`${allKindTrace}\`
- Shows: open dashboard -> click five-line trace -> click \`llm.call\` span -> read prompt, completion, model, tokens, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.

${portNote}

Regenerate with:

\`\`\`bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
\`\`\`

The mandate still requires the outside-person run recorded in
\`docs/demos/gate2-outside-person-proof.md\` before Gate 2 can close.
`;
}

async function requireAttribute(locator, name, expected) {
  await locator.waitFor();
  const actual = await locator.getAttribute(name);
  if (actual !== expected) {
    throw new Error(`expected ${name}=${expected}, got ${actual}`);
  }
}
