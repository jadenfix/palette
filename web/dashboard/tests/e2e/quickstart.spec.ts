import { expect, test } from "@playwright/test";

test("renders the five-line stock OTLP quickstart trace in a browser", async ({ page }) => {
  const traceParam = process.env.BEATER_E2E_TRACE_ID
    ? `&trace=${encodeURIComponent(process.env.BEATER_E2E_TRACE_ID)}`
    : "&kind=llm.call&model=gpt-quickstart";
  await page.goto(`/?tenant=demo&project=demo&environment=local${traceParam}`);

  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();
  await expect(page.getByLabel("Traces")).toContainText("five-line-llm-call");
  await expect(page.getByLabel("Traces")).toContainText("openai/gpt-quickstart");

  const waterfall = page.getByLabel("Agent span waterfall");
  await expect(waterfall).toContainText("five-line-llm-call");
  const llm = waterfall.locator('[data-span-name="five-line-llm-call"]');
  if ((await llm.count()) > 0) {
    await expect(llm).toHaveAttribute("data-kind", "llm.call");
    await expect(llm).toHaveAttribute("data-depth", "0");
    await expect(llm.locator(".kind-icon")).toHaveAttribute("data-icon", "llm");
    await llm.click();
  } else {
    await waterfall.getByText("five-line-llm-call").click();
  }

  const detail = page.getByLabel("Span detail");
  await expect(detail).toContainText("openai/gpt-quickstart");
  await expect(detail).toContainText("Tokens");
  await expect(detail).toContainText("USD 0.001200");
  await expect(detail).toContainText("hello from stock OpenTelemetry");
  await expect(detail).toContainText("hello from Beater");
});
