import assert from "node:assert";
import { test } from "node:test";

import { InMemorySpanExporter } from "@opentelemetry/sdk-trace-base";

import * as beater from "../src/index";
import { Attr, SpanKind } from "../src/semconv";
import type { LlamaIndexEventHandler, LlamaIndexEventName } from "../src/integrations/llamaindex";

const exporter = new InMemorySpanExporter();
beater.init({ tenantId: "t", projectId: "p", environmentId: "e", exporter });

class FakeCallbackManager {
  private handlers = new Map<LlamaIndexEventName, Set<LlamaIndexEventHandler>>();

  on(event: LlamaIndexEventName, handler: LlamaIndexEventHandler): this {
    const handlers = this.handlers.get(event) ?? new Set<LlamaIndexEventHandler>();
    handlers.add(handler);
    this.handlers.set(event, handlers);
    return this;
  }

  off(event: LlamaIndexEventName, handler: LlamaIndexEventHandler): this {
    this.handlers.get(event)?.delete(handler);
    return this;
  }

  emit(event: LlamaIndexEventName, detail: unknown): void {
    for (const handler of this.handlers.get(event) ?? []) {
      handler({ detail });
    }
  }
}

function setup(): InMemorySpanExporter {
  exporter.reset();
  return exporter;
}

function byName(spans: ReturnType<InMemorySpanExporter["getFinishedSpans"]>, name: string) {
  const span = spans.find((s) => s.name === name);
  assert.ok(span, `expected a span named ${name}`);
  return span!;
}

function parentSpanId(span: unknown): string | undefined {
  const readable = span as { parentSpanContext?: { spanId?: string }; parentSpanId?: string };
  return readable.parentSpanContext?.spanId ?? readable.parentSpanId;
}

test("instrumentLlamaIndex records a query, synthesis, and LLM span tree", async () => {
  const exporter = setup();
  const manager = new FakeCallbackManager();
  beater.instrumentLlamaIndex(manager);

  manager.emit("query-start", { id: "query-1", query: "What is Beater?" });
  manager.emit("synthesize-start", { id: "synth-1", query: { query: "What is Beater?", nodes: [] } });
  manager.emit("llm-start", {
    id: "llm-1",
    messages: [{ role: "user", content: "What is Beater?" }],
  });
  manager.emit("llm-end", {
    id: "llm-1",
    response: {
      message: { role: "assistant", content: "An agent observability platform." },
      raw: {
        model: "gpt-4o-mini",
        usage: { prompt_tokens: 17, completion_tokens: 6 },
      },
    },
  });
  manager.emit("synthesize-end", { id: "synth-1", response: { response: "An agent observability platform." } });
  manager.emit("query-end", { id: "query-1", response: { response: "An agent observability platform." } });
  await beater.flush();

  const spans = exporter.getFinishedSpans();
  const query = byName(spans, "llamaindex.query");
  const synth = byName(spans, "llamaindex.synthesize");
  const llm = byName(spans, "llamaindex.llm");

  assert.equal(query.attributes[Attr.SPAN_KIND], SpanKind.AGENT_RUN);
  assert.equal(synth.attributes[Attr.SPAN_KIND], SpanKind.AGENT_STEP);
  assert.equal(llm.attributes[Attr.SPAN_KIND], SpanKind.LLM_CALL);
  assert.equal(llm.attributes[Attr.LLM_MODEL_NAME], "gpt-4o-mini");
  assert.equal(llm.attributes[Attr.LLM_TOKEN_PROMPT], 17);
  assert.equal(llm.attributes[Attr.LLM_TOKEN_COMPLETION], 6);
  assert.match(String(llm.attributes[Attr.OUTPUT_VALUE]), /observability/);
  assert.equal(parentSpanId(synth), query.spanContext().spanId);
  assert.equal(parentSpanId(llm), synth.spanContext().spanId);
});

test("instrumentLlamaIndex records retrieval and tool spans", async () => {
  const exporter = setup();
  const manager = new FakeCallbackManager();
  beater.instrumentLlamaIndex(manager);

  manager.emit("query-start", { id: "query-2", query: "Find policy" });
  manager.emit("retrieve-start", { id: "retrieve-1", query: { query: "Find policy" } });
  manager.emit("retrieve-end", {
    id: "retrieve-1",
    nodes: [{ node: { id_: "doc-1", text: "Refund policy" }, score: 0.9 }],
  });
  manager.emit("llm-tool-call", { toolCall: { id: "tool-1", name: "lookupPolicy", input: { topic: "refund" } } });
  manager.emit("llm-tool-result", {
    toolCall: { id: "tool-1", name: "lookupPolicy", input: { topic: "refund" } },
    toolResult: { output: "30 days", isError: false },
  });
  manager.emit("query-end", { id: "query-2", response: { response: "30 days" } });
  await beater.flush();

  const spans = exporter.getFinishedSpans();
  const retrieve = byName(spans, "llamaindex.retrieve");
  const tool = byName(spans, "llamaindex.tool.lookupPolicy");

  assert.equal(retrieve.attributes[Attr.SPAN_KIND], SpanKind.RETRIEVAL_QUERY);
  assert.match(String(retrieve.attributes[Attr.OUTPUT_VALUE]), /doc-1/);
  assert.equal(tool.attributes[Attr.SPAN_KIND], SpanKind.TOOL_CALL);
  assert.equal(tool.attributes[Attr.OUTPUT_VALUE], "30 days");
});

test("instrumentLlamaIndex unregisters callback handlers", async () => {
  const exporter = setup();
  const manager = new FakeCallbackManager();
  const instrumentation = beater.instrumentLlamaIndex(manager);
  instrumentation.uninstall();

  manager.emit("query-start", { id: "query-3", query: "ignored" });
  manager.emit("query-end", { id: "query-3", response: "ignored" });
  await beater.flush();

  assert.equal(exporter.getFinishedSpans().length, 0);
});
