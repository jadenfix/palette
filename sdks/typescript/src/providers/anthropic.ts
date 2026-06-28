/** Drop-in Anthropic instrumentation: `const client = wrapAnthropic(new Anthropic())`. */

import { context, Span, SpanStatusCode, trace } from "@opentelemetry/api";

import { Attr, SpanKind } from "../semconv";
import { getTracer } from "../tracing";
import { _internal } from "../observe";

const { toValue, applyCommon } = _internal;

/* eslint-disable @typescript-eslint/no-explicit-any */

const ANTHROPIC_CLIENT_MARK = Symbol.for("@beater/sdk.anthropic.client");
const ANTHROPIC_METHOD_MARK = Symbol.for("@beater/sdk.anthropic.method");

function recordUsage(span: any, response: any): void {
  const usage = response?.usage;
  if (!usage) return;
  if (usage.input_tokens != null) span.setAttribute(Attr.LLM_TOKEN_PROMPT, Number(usage.input_tokens));
  if (usage.output_tokens != null) span.setAttribute(Attr.LLM_TOKEN_COMPLETION, Number(usage.output_tokens));
  if (usage.cache_read_input_tokens != null) {
    span.setAttribute(Attr.LLM_TOKEN_CACHE_READ, Number(usage.cache_read_input_tokens));
  }
}

function outputText(response: any): string {
  try {
    const blocks = response.content ?? [];
    const text = blocks.map((b: any) => b?.text ?? "").join("");
    if (text) return text;
    const toolCalls = blocks
      .filter((b: any) => b?.type === "tool_use")
      .map((b: any) => ({ id: b?.id, name: b?.name, input: b?.input }));
    return toolCalls.length > 0 ? toValue({ tool_calls: toolCalls }) : "";
  } catch {
    return "";
  }
}

function isAsyncIterable(value: unknown): value is AsyncIterable<any> {
  return Boolean(value && typeof (value as any)[Symbol.asyncIterator] === "function");
}

interface StreamCapture {
  text: string[];
  toolCalls: Array<{ id?: string; name?: string; input?: unknown; partialJson?: string }>;
}

function recordStreamEvent(capture: StreamCapture, span: Span, event: any): void {
  const message = event?.message;
  if (message?.model) span.setAttribute(Attr.LLM_MODEL_NAME, String(message.model));
  recordUsage(span, message);
  recordUsage(span, event);

  if (event?.type === "content_block_start" && event.content_block?.type === "tool_use") {
    capture.toolCalls.push({
      id: event.content_block.id,
      name: event.content_block.name,
      input: event.content_block.input,
    });
  }

  if (event?.type === "content_block_delta") {
    if (event.delta?.text) capture.text.push(String(event.delta.text));
    if (event.delta?.partial_json) {
      const current = capture.toolCalls[capture.toolCalls.length - 1];
      if (current) current.partialJson = `${current.partialJson ?? ""}${event.delta.partial_json}`;
    }
  }
}

function finishSpan(span: Span, response: any): void {
  if (response?.model) span.setAttribute(Attr.LLM_MODEL_NAME, String(response.model));
  recordUsage(span, response);
  const output = outputText(response);
  if (output) span.setAttribute(Attr.OUTPUT_VALUE, output);
  span.setStatus({ code: SpanStatusCode.OK });
  span.end();
}

function finishStream(span: Span, capture: StreamCapture): void {
  const text = capture.text.join("");
  if (text) {
    span.setAttribute(Attr.OUTPUT_VALUE, text);
  } else if (capture.toolCalls.length > 0) {
    span.setAttribute(Attr.OUTPUT_VALUE, toValue({ tool_calls: capture.toolCalls }));
  }
  span.setStatus({ code: SpanStatusCode.OK });
  span.end();
}

function recordError(span: Span, err: unknown): void {
  span.setStatus({ code: SpanStatusCode.ERROR, message: String(err) });
  span.recordException(err as Error);
  span.end();
}

async function* wrapStream(stream: AsyncIterable<any>, span: Span): AsyncIterable<any> {
  const capture: StreamCapture = { text: [], toolCalls: [] };
  let failed = false;
  try {
    for await (const event of stream) {
      recordStreamEvent(capture, span, event);
      yield event;
    }
  } catch (err) {
    failed = true;
    recordError(span, err);
    throw err;
  } finally {
    if (!failed) finishStream(span, capture);
  }
}

function patchCreate(owner: any): boolean {
  if (!owner?.create || owner.create[ANTHROPIC_METHOD_MARK]) return false;
  const original = owner.create;

  owner.create = async function create(...args: any[]): Promise<any> {
    const params = args[0] ?? {};
    const tracer = getTracer();
    const span = tracer.startSpan("anthropic.messages.create");
    applyCommon(span, SpanKind.LLM_CALL);
    span.setAttribute(Attr.LLM_PROVIDER, "anthropic");
    span.setAttribute(Attr.LLM_MODEL_NAME, String(params.model ?? "unknown"));
    if (params.messages) span.setAttribute(Attr.INPUT_VALUE, toValue(params.messages));

    try {
      const active = trace.setSpan(context.active(), span);
      const response = await context.with(active, () => original.apply(this, args));
      if (isAsyncIterable(response)) {
        return wrapStream(response, span);
      }
      finishSpan(span, response);
      return response;
    } catch (err) {
      recordError(span, err);
      throw err;
    }
  };
  owner.create[ANTHROPIC_METHOD_MARK] = true;
  return true;
}

/** Instrument an Anthropic client in place and return it. */
export function wrapAnthropic<T extends Record<string, any>>(client: T): T {
  if ((client as any)[ANTHROPIC_CLIENT_MARK]) return client;
  patchCreate(client.messages);

  (client as any)[ANTHROPIC_CLIENT_MARK] = true;
  return client;
}
