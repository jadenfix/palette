import assert from "node:assert";
import { test } from "node:test";

import { InMemorySpanExporter } from "@opentelemetry/sdk-trace-base";

import * as beater from "../src/index";
import { Attr, SpanKind, SPAN_KINDS } from "../src/semconv";
import { wrapOpenAI } from "../src/providers/openai";
import { wrapAnthropic } from "../src/providers/anthropic";

// OTel's global registration is process-wide, so init() once and reset between tests.
const exporter = new InMemorySpanExporter();
beater.init({ tenantId: "t", projectId: "p", environmentId: "e", releaseId: "rel-1", exporter });

function setup(): InMemorySpanExporter {
  exporter.reset();
  return exporter;
}

function byName(spans: ReturnType<InMemorySpanExporter["getFinishedSpans"]>, name: string) {
  const span = spans.find((s) => s.name === name);
  assert.ok(span, `expected a span named ${name}`);
  return span!;
}

test("observe records kind, release, and output", async () => {
  const exporter = setup();
  const call = beater.observe(async (prompt: string) => `answer:${prompt}`, {
    kind: SpanKind.LLM_CALL,
    name: "call",
  });
  const result = await call("hi");
  assert.equal(result, "answer:hi");
  await beater.flush();

  const span = byName(exporter.getFinishedSpans(), "call");
  assert.equal(span.attributes[Attr.SPAN_KIND], SpanKind.LLM_CALL);
  assert.equal(span.attributes[Attr.RELEASE_ID], "rel-1");
  assert.match(String(span.attributes[Attr.OUTPUT_VALUE]), /answer:hi/);
});

test("observe records errors", async () => {
  const exporter = setup();
  const boom = beater.observe(
    async () => {
      throw new Error("nope");
    },
    { name: "boom" },
  );
  await assert.rejects(boom());
  await beater.flush();
  const span = byName(exporter.getFinishedSpans(), "boom");
  assert.equal(span.status.code, 2 /* ERROR */);
});

test("wrapOpenAI emits llm.call span with tokens", async () => {
  const exporter = setup();
  const client = wrapOpenAI({
    chat: {
      completions: {
        create: async () => ({
          model: "gpt-4.1",
          usage: { prompt_tokens: 11, completion_tokens: 7 },
          choices: [{ message: { content: "hello there" } }],
        }),
      },
    },
  } as any);

  await client.chat.completions.create({ model: "gpt-4.1", messages: [{ role: "user", content: "hi" }] });
  await beater.flush();

  const span = byName(exporter.getFinishedSpans(), "openai.chat.completions.create");
  assert.equal(span.attributes[Attr.LLM_PROVIDER], "openai");
  assert.equal(span.attributes[Attr.LLM_MODEL_NAME], "gpt-4.1");
  assert.equal(span.attributes[Attr.LLM_TOKEN_PROMPT], 11);
  assert.equal(span.attributes[Attr.LLM_TOKEN_COMPLETION], 7);
  assert.match(String(span.attributes[Attr.OUTPUT_VALUE]), /hello there/);
});

test("instrument reports missing optional provider modules", () => {
  const result = beater.instrument({
    providers: ["openai", "anthropic"],
    modules: { openai: null, anthropic: null },
  });

  assert.deepEqual(result.providers, []);
  assert.deepEqual(result.skipped.map((entry) => entry.provider), ["openai", "anthropic"]);
});

test("instrument monkeypatches OpenAI constructors", async () => {
  const exporter = setup();

  class MockOpenAI {
    chat = {
      completions: {
        create: async () => ({
          model: "gpt-4.1",
          usage: { prompt_tokens: 5, completion_tokens: 2 },
          choices: [
            {
              message: {
                tool_calls: [
                  { id: "call-1", type: "function", function: { name: "lookup", arguments: "{\"q\":\"hi\"}" } },
                ],
              },
            },
          ],
        }),
      },
    };
    responses = {
      create: async () => ({
        model: "gpt-4.1",
        usage: { input_tokens: 3, output_tokens: 1 },
        output_text: "response api reply",
      }),
    };
  }

  const providerModule: any = { default: MockOpenAI, OpenAI: MockOpenAI };
  const result = beater.instrument({ providers: ["openai"], modules: { openai: providerModule } });

  assert.deepEqual(result.providers, ["openai"]);
  assert.equal(result.skipped.length, 0);

  const client = new providerModule.OpenAI();
  assert.ok(client instanceof MockOpenAI);
  await client.chat.completions.create({ model: "gpt-4.1", messages: [{ role: "user", content: "hi" }] });
  await client.responses.create({ model: "gpt-4.1", input: "hi" });
  await beater.flush();

  const spans = exporter.getFinishedSpans();
  const span = byName(spans, "openai.chat.completions.create");
  assert.equal(span.attributes[Attr.LLM_PROVIDER], "openai");
  assert.match(String(span.attributes[Attr.OUTPUT_VALUE]), /lookup/);
  const responsesSpan = byName(spans, "openai.responses.create");
  assert.equal(responsesSpan.attributes[Attr.LLM_TOKEN_PROMPT], 3);
  assert.match(String(responsesSpan.attributes[Attr.OUTPUT_VALUE]), /response api reply/);
});

test("instrument monkeypatches Anthropic constructors", async () => {
  const exporter = setup();

  class MockAnthropic {
    messages = {
      create: async () => ({
        model: "claude-3-5-sonnet",
        usage: { input_tokens: 9, output_tokens: 4 },
        content: [{ type: "text", text: "anthropic reply" }],
      }),
    };
  }

  const providerModule: any = { default: MockAnthropic, Anthropic: MockAnthropic };
  const result = beater.instrument({ providers: ["anthropic"], modules: { anthropic: providerModule } });

  assert.deepEqual(result.providers, ["anthropic"]);
  assert.equal(result.skipped.length, 0);

  const client = new providerModule.default();
  assert.ok(client instanceof MockAnthropic);
  await client.messages.create({ model: "claude-3-5-sonnet", messages: [{ role: "user", content: "hi" }] });
  await beater.flush();

  const span = byName(exporter.getFinishedSpans(), "anthropic.messages.create");
  assert.equal(span.attributes[Attr.LLM_PROVIDER], "anthropic");
  assert.equal(span.attributes[Attr.LLM_TOKEN_PROMPT], 9);
  assert.match(String(span.attributes[Attr.OUTPUT_VALUE]), /anthropic reply/);
});

test("wrapOpenAI keeps streaming span open until chunks are consumed", async () => {
  const exporter = setup();

  async function* stream() {
    yield { model: "gpt-4.1", choices: [{ delta: { content: "hello " } }] };
    yield {
      choices: [
        {
          delta: {
            tool_calls: [{ id: "call-1", type: "function", function: { name: "lookup", arguments: "{}" } }],
          },
        },
      ],
    };
    yield { usage: { prompt_tokens: 3, completion_tokens: 5 }, choices: [{ delta: { content: "there" } }] };
  }

  const client = wrapOpenAI({
    chat: {
      completions: {
        create: async () => stream(),
      },
    },
  } as any);

  const response = await client.chat.completions.create({
    model: "gpt-4.1",
    messages: [{ role: "user", content: "hi" }],
    stream: true,
  });

  assert.equal(exporter.getFinishedSpans().length, 0);
  for await (const _chunk of response) {
    // consume the stream
  }
  await beater.flush();

  const span = byName(exporter.getFinishedSpans(), "openai.chat.completions.create");
  assert.equal(span.attributes[Attr.LLM_TOKEN_PROMPT], 3);
  assert.equal(span.attributes[Attr.LLM_TOKEN_COMPLETION], 5);
  assert.match(String(span.attributes[Attr.OUTPUT_VALUE]), /hello there/);
});

test("wrapAnthropic captures streaming text and tool use", async () => {
  const exporter = setup();

  async function* stream() {
    yield { type: "message_start", message: { model: "claude-3-5-sonnet", usage: { input_tokens: 6 } } };
    yield { type: "content_block_start", content_block: { type: "tool_use", id: "tool-1", name: "search" } };
    yield { type: "content_block_delta", delta: { partial_json: "{\"q\":\"hi\"}" } };
    yield { type: "content_block_delta", delta: { text: "done" } };
    yield { type: "message_delta", usage: { output_tokens: 2 } };
  }

  const client = wrapAnthropic({
    messages: {
      create: async () => stream(),
    },
  } as any);

  const response = await client.messages.create({
    model: "claude-3-5-sonnet",
    messages: [{ role: "user", content: "hi" }],
    stream: true,
  });

  for await (const _event of response) {
    // consume the stream
  }
  await beater.flush();

  const span = byName(exporter.getFinishedSpans(), "anthropic.messages.create");
  assert.equal(span.attributes[Attr.LLM_PROVIDER], "anthropic");
  assert.equal(span.attributes[Attr.LLM_TOKEN_PROMPT], 6);
  assert.equal(span.attributes[Attr.LLM_TOKEN_COMPLETION], 2);
  assert.match(String(span.attributes[Attr.OUTPUT_VALUE]), /done/);
});

test("semconv kinds match the server normalizer set", () => {
  const expected = new Set([
    "agent.run", "agent.turn", "agent.plan", "agent.step", "llm.call",
    "tool.call", "mcp.request", "retrieval.query", "memory.read",
    "memory.write", "guardrail.check",
  ]);
  assert.deepEqual(new Set(SPAN_KINDS), expected);
});
