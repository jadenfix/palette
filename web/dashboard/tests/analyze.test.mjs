import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const root = new URL("..", import.meta.url).pathname;

// analyze.ts is dependency-free, so transpile + run it in a bare sandbox.
function loadAnalyze() {
  const source = readFileSync(join(root, "lib/analyze.ts"), "utf8");
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  });
  const module = { exports: {} };
  vm.runInNewContext(outputText, { exports: module.exports, module, require() {
    throw new Error("analyze.ts must stay dependency-free");
  } });
  return module.exports;
}

function span(id, parent, start, end, name) {
  return { span_id: id, parent_span_id: parent, start_time: start, end_time: end, name: name ?? id };
}

test("toMicros keeps sub-millisecond precision", () => {
  const { toMicros } = loadAnalyze();
  assert.equal(
    toMicros("2026-01-01T00:00:00.000410Z") - toMicros("2026-01-01T00:00:00.000100Z"),
    310,
  );
  assert.equal(toMicros("bad"), null);
  assert.equal(toMicros(null), null);
});

test("critical path follows the latest-finishing chain to the root", () => {
  const { criticalPathSpanIds } = loadAnalyze();
  // root[0..100] contains fast[10..20] and slow[10..100]; slow contains leaf[20..100]
  const spans = [
    span("root", null, "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.100Z"),
    span("fast", "root", "2026-01-01T00:00:00.010Z", "2026-01-01T00:00:00.020Z"),
    span("slow", "root", "2026-01-01T00:00:00.010Z", "2026-01-01T00:00:00.100Z"),
    span("leaf", "slow", "2026-01-01T00:00:00.020Z", "2026-01-01T00:00:00.100Z"),
  ];
  const path = criticalPathSpanIds(spans);
  assert.deepEqual([...path].sort(), ["leaf", "root", "slow"]);
  assert.equal(path.has("fast"), false);
});

test("critical path is empty for no spans and safe on cycles", () => {
  const { criticalPathSpanIds } = loadAnalyze();
  assert.equal(criticalPathSpanIds([]).size, 0);
  const cyclic = [
    span("a", "b", "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.020Z"),
    span("b", "a", "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.030Z"),
  ];
  // Must terminate (no infinite loop) and include the latest-ending span.
  assert.ok(criticalPathSpanIds(cyclic).has("b"));
});

test("critical path stats report wall-clock and a sequential-sibling hint", () => {
  const { criticalPathStats } = loadAnalyze();
  // two siblings run back-to-back under root: toolA[0..40], toolB[40..70]
  const spans = [
    span("root", null, "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.070Z"),
    span("toolA", "root", "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.040Z", "lookup-order"),
    span("toolB", "root", "2026-01-01T00:00:00.040Z", "2026-01-01T00:00:00.070Z", "charge-card"),
  ];
  const stats = criticalPathStats(spans);
  assert.equal(stats.totalMs, 70);
  assert.ok(stats.spanCount >= 2);
  assert.ok(stats.parallelizable);
  // shorter of the two sequential siblings (30ms) is the saving
  assert.equal(stats.parallelizable.savingsMs, 30);
  assert.equal(stats.parallelizable.a, "lookup-order");
  assert.equal(stats.parallelizable.b, "charge-card");
});

test("no parallelization hint when siblings overlap", () => {
  const { criticalPathStats } = loadAnalyze();
  const spans = [
    span("root", null, "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.050Z"),
    span("x", "root", "2026-01-01T00:00:00.000Z", "2026-01-01T00:00:00.040Z"),
    span("y", "root", "2026-01-01T00:00:00.010Z", "2026-01-01T00:00:00.050Z"),
  ];
  assert.equal(criticalPathStats(spans).parallelizable, null);
});

test("formatMs spans microseconds to seconds", () => {
  const { formatMs } = loadAnalyze();
  assert.equal(formatMs(0.4), "400 µs");
  assert.equal(formatMs(5.2), "5.2 ms");
  assert.equal(formatMs(420), "420 ms");
  assert.equal(formatMs(3400), "3.40 s");
  assert.equal(formatMs(-1), "—");
});
