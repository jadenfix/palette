import { expect, test } from "@playwright/test";

test("renders a stock OTLP llm span through table, waterfall, detail, and I/O", async ({
  page
}) => {
  const traceParam = process.env.BEATER_E2E_TRACE_ID
    ? `&trace=${encodeURIComponent(process.env.BEATER_E2E_TRACE_ID)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";
  await page.goto(`/?tenant=demo&project=demo&environment=local${traceParam}`);

  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();
  const traceList = page.getByLabel("Traces");
  await expect(traceList).toContainText("Spans");
  await expect(traceList).toContainText("Latency");
  await expect(traceList).toContainText("Release");
  await expect(traceList).toContainText("refund-agent-run");
  await expect(traceList).toContainText("openai/gpt-demo");

  const waterfall = page.getByLabel("Agent span waterfall");
  for (const kind of [
    "agent.run",
    "agent.turn",
    "agent.plan",
    "agent.step",
    "retrieval.query",
    "memory.read",
    "guardrail.check",
    "llm.call",
    "tool.call",
    "mcp.request",
    "memory.write",
    "evaluator.run",
    "human.review",
    "replay.run"
  ]) {
    await expect(waterfall).toContainText(kind);
  }
  await expect(waterfall).toContainText("lookup-order-tool");

  const run = waterfall.locator('[data-span-name="refund-agent-run"]');
  const turn = waterfall.locator('[data-span-name="customer-refund-turn"]');
  const step = waterfall.locator('[data-span-name="execute-refund-step"]');
  const tool = waterfall.locator('[data-span-name="lookup-order-tool"]');
  const mcp = waterfall.locator('[data-span-name="mcp-order-service"]');

  await expect(run).toHaveAttribute("data-kind", "agent.run");
  await expect(turn).toHaveAttribute("data-kind", "agent.turn");
  await expect(step).toHaveAttribute("data-kind", "agent.step");
  await expect(tool).toHaveAttribute("data-kind", "tool.call");
  await expect(mcp).toHaveAttribute("data-kind", "mcp.request");
  await expect(run).toHaveAttribute("data-depth", "0");
  await expect(turn).toHaveAttribute("data-depth", "1");
  await expect(step).toHaveAttribute("data-depth", "2");
  await expect(tool).toHaveAttribute("data-depth", "3");
  await expect(mcp).toHaveAttribute("data-depth", "4");
  await expect(run.locator(".kind-icon")).toHaveAttribute("data-icon", "agent-run");
  await expect(waterfall.locator('[data-span-name="call-policy-model"] .kind-icon')).toHaveAttribute(
    "data-icon",
    "llm"
  );
  await expect(waterfall.locator('[data-span-name="call-policy-model"] .span-track')).toBeVisible();
  await expect(waterfall.locator('[data-span-name="call-policy-model"] .span-bar')).toBeVisible();
  await expect(tool.locator(".kind-icon")).toHaveAttribute("data-icon", "tool");
  await expect(mcp.locator(".kind-icon")).toHaveAttribute("data-icon", "mcp");

  const axisRail = await waterfall.locator(".axis-rail").boundingBox();
  const rootTrack = await run.locator(".span-track").boundingBox();
  const mcpTrack = await mcp.locator(".span-track").boundingBox();
  expect(axisRail).not.toBeNull();
  expect(rootTrack).not.toBeNull();
  expect(mcpTrack).not.toBeNull();
  expect(Math.abs((rootTrack?.x ?? 0) - (axisRail?.x ?? 0))).toBeLessThanOrEqual(2);
  expect(Math.abs((mcpTrack?.x ?? 0) - (axisRail?.x ?? 0))).toBeLessThanOrEqual(2);

  const orderedNames = await waterfall.locator("[data-span-name]").evaluateAll((rows) =>
    rows.map((row) => row.getAttribute("data-span-name"))
  );
  expect(orderedNames.indexOf("refund-agent-run")).toBeLessThan(
    orderedNames.indexOf("customer-refund-turn")
  );
  expect(orderedNames.indexOf("customer-refund-turn")).toBeLessThan(
    orderedNames.indexOf("execute-refund-step")
  );
  expect(orderedNames.indexOf("execute-refund-step")).toBeLessThan(
    orderedNames.indexOf("lookup-order-tool")
  );
  expect(orderedNames.indexOf("lookup-order-tool")).toBeLessThan(
    orderedNames.indexOf("mcp-order-service")
  );

  await waterfall.getByText("call-policy-model").click();

  const detail = page.getByLabel("Span detail");
  await expect(page.getByLabel("Detail sections")).toHaveCount(0);
  await expect(detail).toContainText("openai/gpt-demo");
  await expect(detail).toContainText("Tokens");
  await expect(detail).toContainText("Latency");
  await expect(detail).toContainText("USD 0.002500");
  await expect(detail.getByRole("heading", { name: "Input" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Output" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Attributes", exact: true })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Canonical" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Unmapped" })).toBeVisible();
  await expect(detail.locator(".io").filter({ hasText: "Input" }).locator("pre")).toHaveText(
    "Can this order be refunded after 31 days?"
  );
  await expect(detail.locator(".io").filter({ hasText: "Output" }).locator("pre")).toHaveText(
    "Escalate because the order is outside the standard window."
  );
  await expect(detail).toContainText("Can this order be refunded after 31 days?");
  await expect(detail).toContainText("Escalate because the order is outside the standard window.");

  await waterfall.getByText("lookup-order-tool").click();
  await expect(detail.locator(".io").filter({ hasText: "Input" }).locator("pre")).toHaveText(
    '{\n  "order_id": "ord_123"\n}'
  );
  await expect(detail.locator(".io").filter({ hasText: "Output" }).locator("pre")).toHaveText(
    '{\n  "status": "delivered",\n  "age_days": 31\n}'
  );
});

test("keeps the trace console inside the viewport on desktop and mobile", async ({ page }) => {
  const traceParam = process.env.BEATER_E2E_TRACE_ID
    ? `&trace=${encodeURIComponent(process.env.BEATER_E2E_TRACE_ID)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";

  for (const viewport of [
    { width: 1440, height: 1000 },
    { width: 390, height: 900 }
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`/?tenant=demo&project=demo&environment=local${traceParam}`);
    await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();
    const overflow = await page.evaluate(() => ({
      documentWidth: document.documentElement.scrollWidth,
      viewportWidth: document.documentElement.clientWidth,
      offenders: Array.from(document.querySelectorAll("*"))
        .filter((element) => element.getBoundingClientRect().right > window.innerWidth + 1)
        .map((element) =>
          typeof element.className === "string" ? element.className : element.tagName
        )
        .slice(0, 5)
    }));
    expect(overflow).toEqual({
      documentWidth: overflow.viewportWidth,
      viewportWidth: overflow.viewportWidth,
      offenders: []
    });
  }
});
