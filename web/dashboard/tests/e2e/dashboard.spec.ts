import { expect, test } from "@playwright/test";
import { expectTokenBreakdown } from "./token-breakdown";

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
  const summary = page.getByLabel("Trace summary");
  await expect(summary.locator(".summary-item").filter({ hasText: "Spans" })).toHaveClass(
    /tone-structure/
  );
  await expect(summary.locator(".summary-item").filter({ hasText: "Spans" })).toContainText(
    "no failures"
  );
  const expectedAdvancedFilters = traceParam.includes("model=") ? "2 active" : "optional";
  await expect(page.locator(".advanced-filters summary")).toContainText(expectedAdvancedFilters);
  await expect(page.locator(".filter-secondary")).not.toBeVisible();

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

  const spanRow = (kind: string, name: string) =>
    waterfall.locator(`[data-kind="${kind}"]`).filter({ hasText: name }).first();
  const run = spanRow("agent.run", "refund-agent-run");
  const turn = spanRow("agent.turn", "customer-refund-turn");
  const step = spanRow("agent.step", "execute-refund-step");
  const llm = spanRow("llm.call", "call-policy-model");
  const tool = spanRow("tool.call", "lookup-order-tool");
  const mcp = spanRow("mcp.request", "mcp-order-service");

  await expect(run).toContainText("refund-agent-run");
  await expect(turn).toContainText("customer-refund-turn");
  await expect(step).toContainText("execute-refund-step");
  await expect(llm).toContainText("call-policy-model");
  await expect(tool).toContainText("lookup-order-tool");
  await expect(mcp).toContainText("mcp-order-service");
  await expect(run).toHaveAttribute("data-span-id", /.+/);
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
  await expect(llm.locator(".kind-icon")).toHaveAttribute("data-icon", "llm");
  await expect(llm.locator(".span-track")).toBeVisible();
  await expect(llm.locator(".span-bar")).toBeVisible();
  const llmTrackBox = await llm.locator(".span-track").boundingBox();
  const llmBarBox = await llm.locator(".span-bar").boundingBox();
  expect(llmTrackBox).not.toBeNull();
  expect(llmBarBox).not.toBeNull();
  if (!llmTrackBox || !llmBarBox) throw new Error("missing llm timeline bar geometry");
  expect(llmBarBox.width).toBeGreaterThanOrEqual(4);
  expect(llmBarBox.width).toBeLessThan(llmTrackBox.width * 0.95);
  await expect(tool.locator(".kind-icon")).toHaveAttribute("data-icon", "tool");
  await expect(mcp.locator(".kind-icon")).toHaveAttribute("data-icon", "mcp");

  const axisRail = await waterfall.locator(".axis-rail").boundingBox();
  const rootTrack = await run.locator(".span-track").boundingBox();
  const mcpTrack = await mcp.locator(".span-track").boundingBox();
  expect(axisRail).not.toBeNull();
  expect(rootTrack).not.toBeNull();
  expect(mcpTrack).not.toBeNull();
  if (!axisRail || !rootTrack || !mcpTrack) throw new Error("missing timeline geometry");
  expect(Math.abs(rootTrack.x - axisRail.x)).toBeLessThanOrEqual(2);
  expect(Math.abs(mcpTrack.x - axisRail.x)).toBeLessThanOrEqual(2);
  expect(Math.abs(rootTrack.width - axisRail.width)).toBeLessThanOrEqual(2);
  expect(Math.abs(mcpTrack.width - axisRail.width)).toBeLessThanOrEqual(2);
  expect(Math.abs(rootTrack.x + rootTrack.width - (axisRail.x + axisRail.width))).toBeLessThanOrEqual(
    2
  );
  expect(Math.abs(mcpTrack.x + mcpTrack.width - (axisRail.x + axisRail.width))).toBeLessThanOrEqual(
    2
  );

  const orderedSpans = await waterfall.locator("[data-span-id]").evaluateAll((rows) =>
    rows.map((row) => ({
      kind: row.getAttribute("data-kind"),
      text: row.textContent ?? ""
    }))
  );
  const indexOfSpan = (kind: string, name: string) =>
    orderedSpans.findIndex((span) => span.kind === kind && span.text.includes(name));
  const runIndex = indexOfSpan("agent.run", "refund-agent-run");
  const turnIndex = indexOfSpan("agent.turn", "customer-refund-turn");
  const stepIndex = indexOfSpan("agent.step", "execute-refund-step");
  const toolIndex = indexOfSpan("tool.call", "lookup-order-tool");
  const mcpIndex = indexOfSpan("mcp.request", "mcp-order-service");
  for (const index of [runIndex, turnIndex, stepIndex, toolIndex, mcpIndex]) {
    expect(index).toBeGreaterThanOrEqual(0);
  }
  expect(runIndex).toBeLessThan(turnIndex);
  expect(turnIndex).toBeLessThan(stepIndex);
  expect(stepIndex).toBeLessThan(toolIndex);
  expect(toolIndex).toBeLessThan(mcpIndex);

  await llm.click();

  const detail = page.getByLabel("Span detail");
  await expect(llm).toHaveAttribute("aria-current", "location");
  const selectedPath = page.getByLabel("Selected span path");
  await expect(selectedPath).toContainText("agent.run");
  await expect(selectedPath).toContainText("refund-agent-run");
  await expect(selectedPath).toContainText("agent.turn");
  await expect(selectedPath).toContainText("customer-refund-turn");
  await expect(selectedPath).toContainText("llm.call");
  await expect(selectedPath).toContainText("call-policy-model");
  await expect(detail.getByLabel("Span metrics").filter({ hasText: "Depth" })).toContainText("3");
  await expect(page.getByLabel("Detail sections")).toHaveCount(0);
  await expect(detail).toContainText("openai/gpt-demo");
  await expect(detail).toContainText("Tokens");
  await expect(detail).toContainText("33 total, 18 prompt, 11 completion, 4 reasoning");
  const essentials = detail.getByLabel("Selected span essentials");
  await expect(essentials.locator("div").filter({ hasText: "Model" })).toContainText(
    "openai/gpt-demo"
  );
  await expect(essentials.locator("div").filter({ hasText: "Tokens" })).toContainText(
    "33 total, 18 prompt, 11 completion, 4 reasoning"
  );
  await expectTokenBreakdown(essentials.getByLabel("Token breakdown"), [
    { label: "Prompt", value: "18" },
    { label: "Completion", value: "11" },
    { label: "Reasoning", value: "4" },
    { label: "Cached", value: "0" }
  ]);
  await expect(essentials.locator("div").filter({ hasText: "Cost" })).toContainText(
    "USD 0.002500"
  );
  await expect(essentials.locator("div").filter({ hasText: "Latency" })).toContainText(
    /(?:\d+ ms|\d+\.\d+ s)/
  );
  await expect(
    detail.getByLabel("Span metrics").locator("div").filter({ hasText: "Latency" })
  ).toContainText(/(?:\d+ ms|\d+\.\d+ s)/);
  await expect(detail).toContainText("USD 0.002500");
  await expect(detail.getByRole("heading", { name: "Prompt" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Completion" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Attributes", exact: true })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Canonical" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Unmapped" })).toBeVisible();
  const unmaskSelectedIo = async (reason: string) => {
    const form = detail.getByLabel("Unmask redacted I/O");
    await expect(form).toBeVisible();
    await form.getByLabel("Reason").fill(reason);
    await form.getByRole("button", { name: "Unmask" }).click();
    await expect(page).toHaveURL(/unmask=true/);
    await expect(page).toHaveURL(new RegExp(`reason=${reason}`));
    await expect(detail).toContainText("Unmask requested");
    await expect(detail).toContainText(reason);
  };
  const promptIo = detail.getByLabel("Prompt I/O");
  const completionIo = detail.getByLabel("Completion I/O");
  await expect(detail.getByLabel("Span I/O")).toContainText("redacted");
  await expect(promptIo.locator("pre")).toHaveText("redacted by policy");
  await expect(completionIo.locator("pre")).toHaveText("redacted by policy");
  await unmaskSelectedIo("gate2-stock-llm-review");
  await expect(promptIo.locator("pre")).toHaveText(
    "Can this order be refunded after 31 days?"
  );
  await expect(completionIo.locator("pre")).toHaveText(
    "Escalate because the order is outside the standard window."
  );
  const proofOrder = await detail.evaluate((node) => {
    const essentialsNode = node.querySelector(".span-proof-strip");
    const ioNode = node.querySelector('[aria-label="Span I/O"]');
    const pathNode = node.querySelector('[aria-label="Selected span path"]');
    if (!essentialsNode || !ioNode || !pathNode) return null;
    return {
      essentialsBeforePath: Boolean(essentialsNode.compareDocumentPosition(pathNode) & 4),
      ioBeforePath: Boolean(ioNode.compareDocumentPosition(pathNode) & 4)
    };
  });
  expect(proofOrder).toEqual({ essentialsBeforePath: true, ioBeforePath: true });
  await expect(detail).toContainText("Can this order be refunded after 31 days?");
  await expect(detail).toContainText("Escalate because the order is outside the standard window.");

  await tool.click();
  await expect(detail.getByRole("heading", { name: "Input" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Output" })).toBeVisible();
  const inputIo = detail.getByLabel("Input I/O");
  const outputIo = detail.getByLabel("Output I/O");
  await expect(detail.getByLabel("Span I/O")).toContainText("redacted");
  await expect(inputIo.locator("pre")).toHaveText("redacted by policy");
  await expect(outputIo.locator("pre")).toHaveText("redacted by policy");
  await unmaskSelectedIo("gate2-stock-tool-review");
  await expect(inputIo.locator("pre")).toHaveText(
    '{\n  "order_id": "ord_123"\n}'
  );
  await expect(outputIo.locator("pre")).toHaveText(
    '{\n  "status": "delivered",\n  "age_days": 31\n}'
  );
});

test("keeps an explicitly opened trace coherent when secondary filters exclude it", async ({
  page
}) => {
  test.skip(!process.env.BEATER_E2E_TRACE_ID, "requires the seeded all-kind trace id");

  const traceId = process.env.BEATER_E2E_TRACE_ID!;
  await page.goto(
    `/?tenant=demo&project=demo&environment=local&trace=${encodeURIComponent(traceId)}&status=error`
  );

  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Trace" })).toHaveValue(traceId);
  await expect(page.locator('select[name="status"]')).toHaveValue("error");

  const traceList = page.getByLabel("Traces");
  const selectedRow = traceList.locator('a.run-row[data-outside-filters="true"]');
  await expect(selectedRow).toHaveCount(1);
  await expect(selectedRow).toHaveAttribute("aria-current", "location");
  await expect(selectedRow).toContainText("refund-agent-run");
  await expect(selectedRow).toContainText("outside filters");
  await expect(selectedRow).toContainText("compose-demo");

  const summary = page.getByLabel("Trace summary");
  await expect(summary).not.toContainText("No trace");
  await expect(summary.locator(".summary-item").filter({ hasText: "Spans" })).toContainText("14");
  await expect(summary.locator(".summary-item").filter({ hasText: "Model" })).toContainText(
    "openai/gpt-demo"
  );
  await expect(summary.locator(".summary-item").filter({ hasText: "Model" })).toContainText(
    "compose-demo"
  );

  const waterfall = page.getByLabel("Agent span waterfall");
  await expect(waterfall).toContainText("refund-agent-run");
  await expect(waterfall).toContainText("call-policy-model");
});

test("does not fake-select the first span for a stale span query param", async ({ page }) => {
  const traceParam = process.env.BEATER_E2E_TRACE_ID
    ? `&trace=${encodeURIComponent(process.env.BEATER_E2E_TRACE_ID)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";

  await page.goto(
    `/?tenant=demo&project=demo&environment=local${traceParam}&span=missing-span`
  );

  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();
  await expect(page.locator(".notice")).toContainText(
    "Span missing-span was not found in trace"
  );
  const waterfall = page.getByLabel("Agent span waterfall");
  await expect(waterfall).toContainText("call-policy-model");
  await expect(waterfall.locator('[aria-current="location"]')).toHaveCount(0);
  await expect(page.getByLabel("Span detail")).toContainText("Select a span in the waterfall.");
  await expect(page.getByRole("link", { name: "Refresh" })).not.toHaveAttribute(
    "href",
    /span=missing-span/
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
    const waterfall = page.getByLabel("Agent span waterfall");
    const llm = waterfall
      .locator('[data-kind="llm.call"]')
      .filter({ hasText: "call-policy-model" })
      .first();
    await llm.click();
    await expect(page.getByLabel("Span detail").getByLabel("Token breakdown")).toBeVisible();
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

    if (viewport.width === 390) {
      const timingLayout = await llm
        .locator(".duration")
        .first()
        .evaluate((node) => {
          const track = node.querySelector(".span-track");
          const label = node.querySelector(":scope > span:last-child");
          if (!track || !label) return null;
          const trackRect = track.getBoundingClientRect();
          const labelRect = label.getBoundingClientRect();
          return {
            trackRight: trackRect.right,
            labelLeft: labelRect.left,
            separated:
              trackRect.right <= labelRect.left ||
              labelRect.bottom <= trackRect.top ||
              trackRect.bottom <= labelRect.top
          };
        });
      expect(timingLayout).not.toBeNull();
      expect(timingLayout?.separated).toBe(true);
      expect(timingLayout!.labelLeft - timingLayout!.trackRight).toBeGreaterThanOrEqual(6);
    }
  }
});

test("keeps odd metric grids visually closed", async ({ page }) => {
  const traceParam = process.env.BEATER_E2E_TRACE_ID
    ? `&trace=${encodeURIComponent(process.env.BEATER_E2E_TRACE_ID)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";

  await page.goto(`/?tenant=demo&project=demo&environment=local${traceParam}`);
  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();

  const metrics = await page.evaluate(() => {
    const container = document.createElement("dl");
    container.className = "metrics";
    container.style.width = "400px";
    for (let index = 0; index < 7; index += 1) {
      const cell = document.createElement("div");
      const term = document.createElement("dt");
      const description = document.createElement("dd");
      term.textContent = `Metric ${index + 1}`;
      description.textContent = String(index + 1);
      cell.append(term, description);
      container.append(cell);
    }
    document.body.append(container);
    const cells = Array.from(container.children) as HTMLElement[];
    const rects = cells.map((cell) => cell.getBoundingClientRect());
    const styles = cells.map((cell) => getComputedStyle(cell));
    const result = {
      penultimateBorderWidth: styles[5].borderBottomWidth,
      lastBorderWidth: styles[6].borderBottomWidth,
      normalWidth: rects[0].width,
      lastWidth: rects[6].width
    };
    container.remove();
    return result;
  });

  expect(metrics.penultimateBorderWidth).not.toBe("0px");
  expect(metrics.lastBorderWidth).toBe("0px");
  expect(metrics.lastWidth).toBeGreaterThan(metrics.normalWidth * 1.8);
});

test("supports keyboard focus across filters, traces, spans, and unmask controls", async ({
  page
}) => {
  const traceParam = process.env.BEATER_E2E_TRACE_ID
    ? `&trace=${encodeURIComponent(process.env.BEATER_E2E_TRACE_ID)}`
    : "&kind=llm.call&model=gpt-demo&release=compose-demo";

  await page.goto(`/?tenant=demo&project=demo&environment=local${traceParam}`);
  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();

  const tenant = page.locator('input[name="tenant"]');
  await tenant.focus();
  await expect(tenant).toBeFocused();

  await page.keyboard.press("Tab");
  await expect(page.locator('input[name="project"]')).toBeFocused();

  await page.locator(".run-row").first().focus();
  await expect(page.locator(".run-row").first()).toBeFocused();

  const llmSpan = page
    .getByLabel("Agent span waterfall")
    .locator('[data-kind="llm.call"]')
    .filter({ hasText: "call-policy-model" })
    .first();
  await llmSpan.focus();
  await expect(llmSpan).toBeFocused();

  await page.goto(
    `/?tenant=demo&project=demo&environment=local${traceParam}&unmask=true&reason=keyboard-test`
  );
  const redactedView = page.getByRole("link", { name: "Redacted view" });
  await expect(redactedView).toBeVisible();
  await redactedView.focus();
  await expect(redactedView).toBeFocused();
});
