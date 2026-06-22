import { expect, test } from "@playwright/test";

test("renders the five-line stock OTLP quickstart trace in a browser", async ({ page }) => {
  const traceId = process.env.BEATER_E2E_QUICKSTART_TRACE_ID;
  const release = process.env.BEATER_E2E_QUICKSTART_RELEASE;
  const releaseParam = release ? `&release=${encodeURIComponent(release)}` : "";
  await page.goto(`/?tenant=demo&project=demo&environment=local&kind=llm.call&model=gpt-quickstart${releaseParam}`);

  await expect(page.getByRole("heading", { name: "Agent Trace Debugger" })).toBeVisible();
  await expect(page.locator(".advanced-filters summary")).toContainText(
    release ? "2 active" : "1 active"
  );
  await expect(page.locator(".filter-secondary")).not.toBeVisible();
  const traceList = page.getByLabel("Traces");
  await expect(traceList).toContainText("five-line-llm-call");
  await expect(traceList).toContainText("openai/gpt-quickstart");
  const traceFilter = page.locator('input[name="trace"]');
  await expect(traceFilter).toHaveValue("");
  await expect(traceFilter).toHaveAttribute("placeholder", /latest: /);
  await expect(page).not.toHaveURL(/trace=/);

  const traceRow = traceId
    ? traceList.locator(`a.run-row[href*="trace=${encodeURIComponent(traceId)}"]`)
    : traceList.getByRole("link").filter({ hasText: "five-line-llm-call" }).first();
  await expect(traceRow).toContainText("five-line-llm-call");
  await traceRow.click();
  await expect(page).toHaveURL(traceId ? new RegExp(`trace=${traceId}`) : /trace=/);
  await expect(traceFilter).not.toHaveValue("");

  const waterfall = page.getByLabel("Agent span waterfall");
  await expect(waterfall).toContainText("five-line-llm-call");
  const llm = waterfall.locator('[data-kind="llm.call"]');
  await expect(llm).toHaveCount(1);
  await expect(llm).toContainText("five-line-llm-call");
  await expect(llm).toHaveAttribute("data-span-id", /.+/);
  await expect(llm).toHaveAttribute("data-kind", "llm.call");
  await expect(llm).toHaveAttribute("data-depth", "0");
  await expect(llm.locator(".kind-icon")).toHaveAttribute("data-icon", "llm");
  await llm.click();
  await expect(llm).toHaveAttribute("aria-current", "location");

  const detail = page.getByLabel("Span detail");
  const selectedPath = page.getByLabel("Selected span path");
  await expect(selectedPath).toContainText("llm.call");
  await expect(selectedPath).toContainText("five-line-llm-call");
  await expect(detail).toContainText("openai/gpt-quickstart");
  await expect(detail).toContainText("Tokens");
  await expect(detail).toContainText("12 total, 5 prompt, 7 completion");
  const essentials = detail.getByLabel("Selected span essentials");
  await expect(essentials.locator("div").filter({ hasText: "Model" })).toContainText(
    "openai/gpt-quickstart"
  );
  await expect(essentials.locator("div").filter({ hasText: "Tokens" })).toContainText(
    "12 total, 5 prompt, 7 completion"
  );
  await expect(essentials.locator("div").filter({ hasText: "Cost" })).toContainText(
    "USD 0.001200"
  );
  await expect(essentials.locator("div").filter({ hasText: "Latency" })).toContainText(
    /(?:\d+ ms|\d+\.\d+ s)/
  );
  await expect(
    detail.getByLabel("Span metrics").locator("div").filter({ hasText: "Latency" })
  ).toContainText(/(?:\d+ ms|\d+\.\d+ s)/);
  await expect(detail).toContainText("USD 0.001200");
  await expect(detail.getByRole("heading", { name: "Prompt" })).toBeVisible();
  await expect(detail.getByRole("heading", { name: "Completion" })).toBeVisible();
  await expect(detail).toContainText("hello from stock OpenTelemetry");
  await expect(detail).toContainText("hello from Beater");
});
