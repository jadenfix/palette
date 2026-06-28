/**
 * Beater TypeScript SDK — ergonomic agent observability (OpenTelemetry-native).
 *
 *   import * as beater from "@beater/sdk";
 *   beater.init({ tenantId: "acme", projectId: "support-bot", environmentId: "prod" });
 *   const handle = beater.observe(rawHandle, { kind: beater.SpanKind.AGENT_RUN });
 *
 * This is the hand-written ergonomic (Layer 2) SDK. The generated control-plane
 * client (Layer 1) ships separately as `@beater/client`, generated from the
 * OpenAPI contract so it never drifts from the API.
 */

export { init, getConfig, flush, shutdown } from "./tracing";
export { observe, span, setInput, setOutput } from "./observe";
export { wrapOpenAI } from "./providers/openai";
export { wrapAnthropic } from "./providers/anthropic";
export { SpanKind, Attr, SPAN_KINDS } from "./semconv";
export type { SpanKindValue } from "./semconv";
export type { BeaterConfig, BeaterOptions } from "./config";
export { BeaterCallbackHandler } from "./integrations/langchain";
export { instrumentLlamaIndex } from "./integrations/llamaindex";
export type {
  LlamaIndexCallbackManager,
  LlamaIndexEventHandler,
  LlamaIndexEventName,
  LlamaIndexInstrumentation,
} from "./integrations/llamaindex";

export const VERSION = "0.1.0";
