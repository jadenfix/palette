/**
 * LlamaIndex.TS integration: register Beater spans on a CallbackManager.
 *
 *   import { CallbackManager, Settings } from "llamaindex";
 *   import { instrumentLlamaIndex } from "@beater/sdk";
 *
 *   const callbackManager = instrumentLlamaIndex(new CallbackManager()).callbackManager;
 *   Settings.callbackManager = callbackManager;
 *
 * Implemented structurally so `@beater/sdk` does not hard-depend on `llamaindex`.
 */

import { context, Span, SpanStatusCode, trace } from "@opentelemetry/api";

import { _internal } from "../observe";
import { Attr, SpanKind, SpanKindValue } from "../semconv";
import { getTracer } from "../tracing";

const { toValue } = _internal;

export type LlamaIndexEventName =
  | "agent-start"
  | "agent-end"
  | "llm-start"
  | "llm-end"
  | "llm-stream"
  | "llm-tool-call"
  | "llm-tool-result"
  | "query-start"
  | "query-end"
  | "retrieve-start"
  | "retrieve-end"
  | "synthesize-start"
  | "synthesize-end";

export type LlamaIndexEventHandler = (event: { detail?: unknown } | unknown) => void;

export interface LlamaIndexCallbackManager {
  on(event: LlamaIndexEventName, handler: LlamaIndexEventHandler): unknown;
  off?(event: LlamaIndexEventName, handler: LlamaIndexEventHandler): unknown;
}

export interface LlamaIndexInstrumentation {
  callbackManager: LlamaIndexCallbackManager;
  uninstall(): void;
}

interface OpenSpan {
  id: string;
  span: Span;
}

const pairedEvents = [
  ["agent-start", "agent-end", "llamaindex.agent", SpanKind.AGENT_RUN],
  ["query-start", "query-end", "llamaindex.query", SpanKind.AGENT_RUN],
  ["retrieve-start", "retrieve-end", "llamaindex.retrieve", SpanKind.RETRIEVAL_QUERY],
  ["synthesize-start", "synthesize-end", "llamaindex.synthesize", SpanKind.AGENT_STEP],
] as const;

/**
 * Register Beater handlers on a LlamaIndex.TS `CallbackManager`.
 *
 * Returns an object with `uninstall()` for tests, hot reloads, and apps that
 * swap callback managers. The manager itself is returned for one-line setup.
 */
export function instrumentLlamaIndex(callbackManager: LlamaIndexCallbackManager): LlamaIndexInstrumentation {
  const openSpans = new Map<string, OpenSpan>();
  const openStack: string[] = [];
  const streamedOutput = new Map<string, string>();
  let fallbackSeq = 0;

  function activeParent(): Span | undefined {
    for (let idx = openStack.length - 1; idx >= 0; idx -= 1) {
      const current = openSpans.get(openStack[idx]);
      if (current) return current.span;
    }
    return undefined;
  }

  function fallbackId(prefix: string): string {
    fallbackSeq += 1;
    return `${prefix}:${fallbackSeq}`;
  }

  function start(id: string | undefined, name: string, kind: SpanKindValue, input: unknown): string {
    const spanId = id ?? fallbackId(name);
    const parent = activeParent();
    const ctx = parent ? trace.setSpan(context.active(), parent) : undefined;
    const span = ctx ? getTracer().startSpan(name, undefined, ctx) : getTracer().startSpan(name);
    span.setAttribute(Attr.SPAN_KIND, kind);
    if (input !== undefined && input !== null) {
      span.setAttribute(Attr.INPUT_VALUE, toValue(input));
    }
    openSpans.set(spanId, { id: spanId, span });
    openStack.push(spanId);
    return spanId;
  }

  function end(id: string | undefined, output: unknown): void {
    const spanId = id ?? openStack[openStack.length - 1];
    if (!spanId) return;
    const current = openSpans.get(spanId);
    if (!current) return;
    openSpans.delete(spanId);
    const stackIndex = openStack.lastIndexOf(spanId);
    if (stackIndex >= 0) openStack.splice(stackIndex, 1);
    if (output !== undefined && output !== null) {
      current.span.setAttribute(Attr.OUTPUT_VALUE, toValue(output));
    }
    current.span.setStatus({ code: SpanStatusCode.OK });
    current.span.end();
  }

  function onStart(name: string, kind: SpanKindValue, inputKey: string, event: unknown): void {
    const detail = eventDetail(event);
    start(extractEventId(detail), name, kind, valueAt(detail, inputKey));
  }

  function onEnd(outputKey: string, event: unknown): void {
    const detail = eventDetail(event);
    end(extractEventId(detail), valueAt(detail, outputKey));
  }

  const handlers: Array<[LlamaIndexEventName, LlamaIndexEventHandler]> = [];
  for (const [startEvent, endEvent, spanName, kind] of pairedEvents) {
    const inputKey = inputKeyFor(startEvent);
    const outputKey = outputKeyFor(endEvent);
    handlers.push([startEvent, (event) => onStart(spanName, kind, inputKey, event)]);
    handlers.push([endEvent, (event) => onEnd(outputKey, event)]);
  }
  handlers.push(["llm-start", (event) => onStart("llamaindex.llm", SpanKind.LLM_CALL, "messages", event)]);

  handlers.push([
    "llm-stream",
    (event) => {
      const detail = eventDetail(event);
      const id = extractEventId(detail);
      const chunk = valueAt(detail, "chunk");
      const delta = stringAt(chunk, "delta");
      if (id && delta) streamedOutput.set(id, `${streamedOutput.get(id) ?? ""}${delta}`);
    },
  ]);

  handlers.push([
    "llm-end",
    (event) => {
      const detail = eventDetail(event);
      const id = extractEventId(detail);
      const span = id ? openSpans.get(id)?.span : undefined;
      if (span) {
        applyLlmResponseAttributes(span, valueAt(detail, "response"));
      }
      const streamed = id ? streamedOutput.get(id) : undefined;
      if (id) streamedOutput.delete(id);
      end(id, responseOutput(valueAt(detail, "response")) ?? streamed);
    },
  ]);

  handlers.push([
    "llm-tool-call",
    (event) => {
      const detail = eventDetail(event);
      const toolCall = valueAt(detail, "toolCall");
      const name = stringAt(toolCall, "name") ?? "tool";
      const id = stringAt(toolCall, "id") ?? fallbackId(`tool:${name}`);
      start(id, `llamaindex.tool.${name}`, SpanKind.TOOL_CALL, valueAt(toolCall, "input"));
    },
  ]);

  handlers.push([
    "llm-tool-result",
    (event) => {
      const detail = eventDetail(event);
      const toolCall = valueAt(detail, "toolCall");
      const toolResult = valueAt(detail, "toolResult");
      end(stringAt(toolCall, "id"), valueAt(toolResult, "output") ?? toolResult);
    },
  ]);

  for (const [event, handler] of handlers) {
    callbackManager.on(event, handler);
  }

  return {
    callbackManager,
    uninstall(): void {
      if (!callbackManager.off) return;
      for (const [event, handler] of handlers) {
        callbackManager.off(event, handler);
      }
    },
  };
}

function inputKeyFor(event: LlamaIndexEventName): string {
  switch (event) {
    case "agent-start":
      return "startStep";
    case "llm-start":
      return "messages";
    case "query-start":
    case "retrieve-start":
    case "synthesize-start":
      return "query";
    default:
      return "";
  }
}

function outputKeyFor(event: LlamaIndexEventName): string {
  switch (event) {
    case "agent-end":
      return "endStep";
    case "llm-end":
      return "response";
    case "query-end":
    case "synthesize-end":
      return "response";
    case "retrieve-end":
      return "nodes";
    default:
      return "";
  }
}

function eventDetail(event: unknown): unknown {
  const record = asRecord(event);
  return record && "detail" in record ? record.detail : event;
}

function extractEventId(detail: unknown): string | undefined {
  return stringAt(detail, "id") ?? stringAt(valueAt(detail, "startStep"), "id") ?? stringAt(valueAt(detail, "endStep"), "id");
}

function responseOutput(response: unknown): unknown {
  return (
    valueAt(valueAt(response, "message"), "content") ??
    valueAt(response, "response") ??
    valueAt(response, "delta") ??
    valueAt(response, "text") ??
    response
  );
}

function applyLlmResponseAttributes(span: Span, response: unknown): void {
  const raw = valueAt(response, "raw") ?? response;
  const model = stringAt(raw, "model") ?? stringAt(raw, "model_name") ?? stringAt(raw, "modelName");
  if (model) span.setAttribute(Attr.LLM_MODEL_NAME, model);

  const usage = extractUsage(response);
  if (usage.prompt != null) span.setAttribute(Attr.LLM_TOKEN_PROMPT, usage.prompt);
  if (usage.completion != null) span.setAttribute(Attr.LLM_TOKEN_COMPLETION, usage.completion);
}

function extractUsage(value: unknown): { prompt?: number; completion?: number } {
  for (const candidate of usageCandidates(value)) {
    const prompt = firstNumber(candidate, ["prompt_tokens", "promptTokens", "input_tokens", "inputTokens"]);
    const completion = firstNumber(candidate, [
      "completion_tokens",
      "completionTokens",
      "output_tokens",
      "outputTokens",
    ]);
    if (prompt != null || completion != null) {
      return { prompt, completion };
    }
  }
  return {};
}

function usageCandidates(value: unknown): unknown[] {
  const raw = valueAt(value, "raw");
  return [
    valueAt(value, "usage"),
    valueAt(value, "usage_metadata"),
    valueAt(value, "usageMetadata"),
    valueAt(raw, "usage"),
    valueAt(raw, "usage_metadata"),
    valueAt(raw, "usageMetadata"),
    valueAt(valueAt(value, "message"), "options"),
    valueAt(valueAt(raw, "response_metadata"), "tokenUsage"),
    valueAt(valueAt(raw, "responseMetadata"), "tokenUsage"),
  ];
}

function firstNumber(value: unknown, keys: string[]): number | undefined {
  for (const key of keys) {
    const num = numberAt(value, key);
    if (num != null) return num;
  }
  return undefined;
}

function valueAt(value: unknown, key: string): unknown {
  if (!key) return value;
  return asRecord(value)?.[key];
}

function stringAt(value: unknown, key: string): string | undefined {
  const found = valueAt(value, key);
  return typeof found === "string" && found.length > 0 ? found : undefined;
}

function numberAt(value: unknown, key: string): number | undefined {
  const found = valueAt(value, key);
  return typeof found === "number" && Number.isFinite(found) ? found : undefined;
}

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null ? (value as Record<string, unknown>) : undefined;
}
