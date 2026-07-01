/** Drop-in OpenAI instrumentation: `const client = wrapOpenAI(new OpenAI())`. */

import { context, Span, SpanStatusCode, trace } from "@opentelemetry/api";

import { Attr, SpanKind } from "../semconv";
import { getTracer } from "../tracing";
import { _internal } from "../observe";

const { toValue, applyCommon } = _internal;

/* eslint-disable @typescript-eslint/no-explicit-any */

const OPENAI_CLIENT_MARK = Symbol.for("@beater/sdk.openai.client");
const OPENAI_METHOD_MARK = Symbol.for("@beater/sdk.openai.method");

function recordUsage(span: any, response: any): void {
  const usage = response?.usage;
  if (!usage) return;
  if (usage.prompt_tokens != null) span.setAttribute(Attr.LLM_TOKEN_PROMPT, Number(usage.prompt_tokens));
  if (usage.completion_tokens != null) span.setAttribute(Attr.LLM_TOKEN_COMPLETION, Number(usage.completion_tokens));
  if (usage.input_tokens != null) span.setAttribute(Attr.LLM_TOKEN_PROMPT, Number(usage.input_tokens));
  if (usage.output_tokens != null) span.setAttribute(Attr.LLM_TOKEN_COMPLETION, Number(usage.output_tokens));
  const reasoning = usage.completion_tokens_details?.reasoning_tokens;
  if (reasoning != null) span.setAttribute(Attr.LLM_TOKEN_REASONING, Number(reasoning));
}

function textFromContent(content: any): string {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return "";
  return content.map((part) => part?.text ?? part?.content ?? "").join("");
}

function simplifyToolCalls(toolCalls: any): unknown[] {
  if (!Array.isArray(toolCalls)) return [];
  return toolCalls.map((call) => ({
    id: call?.id,
    type: call?.type,
    name: call?.function?.name ?? call?.name,
    arguments: call?.function?.arguments ?? call?.arguments,
  }));
}

function outputText(response: any): string {
  try {
    if (response.output_text) return String(response.output_text);
    if (Array.isArray(response.output)) {
      const text = response.output
        .map((item: any) => textFromContent(item?.content) || item?.text || "")
        .join("");
      if (text) return text;
      const toolCalls = response.output.filter((item: any) => item?.type === "function_call");
      if (toolCalls.length > 0) return toValue({ tool_calls: simplifyToolCalls(toolCalls) });
    }
    const choice = response.choices?.[0];
    const content = textFromContent(choice?.message?.content) || choice?.text || "";
    if (content) return content;
    const toolCalls = simplifyToolCalls(choice?.message?.tool_calls);
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
  toolCalls: unknown[];
}

function recordChunk(capture: StreamCapture, span: Span, chunk: any): void {
  if (chunk?.model) span.setAttribute(Attr.LLM_MODEL_NAME, String(chunk.model));
  recordUsage(span, chunk);

  if (chunk?.type === "response.output_text.delta" && chunk.delta) {
    capture.text.push(String(chunk.delta));
  }
  if (chunk?.type === "response.output_item.done" && chunk.item?.type === "function_call") {
    capture.toolCalls.push(...simplifyToolCalls([chunk.item]));
  }

  for (const choice of chunk?.choices ?? []) {
    const delta = choice?.delta ?? {};
    const text = textFromContent(delta.content);
    if (text) capture.text.push(text);
    capture.toolCalls.push(...simplifyToolCalls(delta.tool_calls));
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
    for await (const chunk of stream) {
      recordChunk(capture, span, chunk);
      yield chunk;
    }
  } catch (err) {
    failed = true;
    recordError(span, err);
    throw err;
  } finally {
    if (!failed) finishStream(span, capture);
  }
}

function patchCreate(owner: any, spanName: string): boolean {
  if (!owner?.create || owner.create[OPENAI_METHOD_MARK]) return false;
  const original = owner.create;

  owner.create = async function create(...args: any[]): Promise<any> {
    const params = args[0] ?? {};
    const tracer = getTracer();
    const span = tracer.startSpan(spanName);
    applyCommon(span, SpanKind.LLM_CALL);
    span.setAttribute(Attr.LLM_PROVIDER, "openai");
    span.setAttribute(Attr.LLM_MODEL_NAME, String(params.model ?? "unknown"));
    const input = params.messages ?? params.input;
    if (input !== undefined) span.setAttribute(Attr.INPUT_VALUE, toValue(input));

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
  owner.create[OPENAI_METHOD_MARK] = true;
  return true;
}

/** Instrument an OpenAI client in place and return it. */
export function wrapOpenAI<T extends Record<string, any>>(client: T): T {
  if ((client as any)[OPENAI_CLIENT_MARK]) return client;
  patchCreate(client.chat?.completions, "openai.chat.completions.create");
  patchCreate(client.responses, "openai.responses.create");

  (client as any)[OPENAI_CLIENT_MARK] = true;
  return client;
}
