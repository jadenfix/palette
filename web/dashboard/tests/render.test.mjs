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
  assert.match(page, /SpanDetail/);
  assert.match(page, /IoBlock/);
  assert.match(page, /name="status"/);
  assert.match(page, /name="kind"/);
  assert.match(page, /name="started_after"/);
  assert.match(page, /name="model"/);
  assert.match(page, /name="release"/);
  assert.match(page, /name="min_cost_micros"/);
  assert.match(page, /name="min_latency_ms"/);
});

test("dashboard client uses public beater read endpoints", () => {
  const api = readFileSync(join(root, "lib/api.ts"), "utf8");
  assert.match(api, /generated\/api-types/);
  assert.match(api, /TraceListPathParams/);
  assert.match(api, /encodeURIComponent\(path\.tenant_id\)/);
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
