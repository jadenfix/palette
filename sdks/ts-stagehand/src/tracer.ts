import {
  context,
  SpanStatusCode,
  trace,
  type Span,
  type Tracer,
} from "@opentelemetry/api";

import {
  BEATER_SPAN_KIND,
  BrowserAttr,
  SpanKind,
  StepStatus,
  type BrowserActionName,
} from "./semconv.js";
import type { ModelDecision, PageLike } from "./types.js";

const TRACER_NAME = "@beater/stagehand-instrumentation";
const TRACER_VERSION = "0.1.0";

/** Best-effort read of a possibly-async page getter, swallowing errors. */
async function safeCall(
  fn: ((...a: unknown[]) => unknown) | undefined,
): Promise<string | undefined> {
  if (typeof fn !== "function") return undefined;
  try {
    const value = await Promise.resolve(fn());
    return typeof value === "string" ? value : undefined;
  } catch {
    return undefined;
  }
}

/** Pull a selector hint out of a Stagehand call's first argument. */
function selectorFromArgs(args: unknown[]): string | undefined {
  const first = args[0];
  if (typeof first === "string") return first;
  if (first && typeof first === "object") {
    const obj = first as Record<string, unknown>;
    for (const key of ["selector", "action", "instruction"]) {
      const v = obj[key];
      if (typeof v === "string") return v;
    }
  }
  return undefined;
}

/** Extract a model decision (reasoning, grounded selector) from a result, if present. */
export function decisionFromResult(result: unknown): ModelDecision | undefined {
  if (!result || typeof result !== "object") return undefined;
  const obj = result as Record<string, unknown>;
  const reasoning =
    pickString(obj, ["reasoning", "model_thoughts", "thoughts", "description"]);
  const selector = pickString(obj, ["selector", "xpath"]);
  if (reasoning === undefined && selector === undefined) return undefined;
  return { reasoning, selector };
}

function pickString(
  obj: Record<string, unknown>,
  keys: string[],
): string | undefined {
  for (const key of keys) {
    const v = obj[key];
    if (typeof v === "string" && v.length > 0) return v;
  }
  return undefined;
}

/**
 * Emits canonical Beater `browser.*` spans for Stagehand AI primitives.
 *
 * Each wrapped `act`/`observe`/`extract` call produces a `tool.call` span. When
 * the call (or its result) carries a model decision, an `llm.call` child span
 * carrying `browser.reasoning` is also emitted.
 */
export class BeaterStagehandTracer {
  private readonly tracer: Tracer;
  private readonly engine: string;
  private seq = 0;

  constructor(opts: { tracer?: Tracer; engine?: string } = {}) {
    this.tracer = opts.tracer ?? trace.getTracer(TRACER_NAME, TRACER_VERSION);
    this.engine = opts.engine ?? "chromium";
  }

  /** Monotonic step sequence shared across all instrumented primitives. */
  nextSeq(): number {
    return this.seq++;
  }

  /**
   * Wrap a single primitive invocation in a `tool.call` span.
   * `invoke` performs the real Stagehand call and returns its result.
   */
  async traceCall<T>(
    action: BrowserActionName,
    page: PageLike,
    args: unknown[],
    invoke: () => Promise<T>,
  ): Promise<T> {
    const seq = this.nextSeq();
    // Bind to `page` so the real Playwright `url()`/`title()` methods keep their
    // receiver — passing the bare method reference calls them with `this`
    // undefined and they throw (silently swallowed -> url/title always missing).
    const url = await safeCall(page.url?.bind(page));
    const title = await safeCall(page.title?.bind(page));
    const selector = selectorFromArgs(args);

    const span = this.tracer.startSpan(`browser.${action}`, {
      attributes: {
        [BEATER_SPAN_KIND]: SpanKind.TOOL_CALL,
        [BrowserAttr.ENGINE]: this.engine,
        [BrowserAttr.ACTION]: action,
        [BrowserAttr.STEP_SEQ]: seq,
        [BrowserAttr.STEP_STATUS]: StepStatus.OK,
        ...(url !== undefined ? { [BrowserAttr.URL]: url } : {}),
        ...(title !== undefined ? { [BrowserAttr.TITLE]: title } : {}),
        ...(selector !== undefined
          ? { [BrowserAttr.SELECTOR]: selector }
          : {}),
      },
    });

    const ctx = trace.setSpan(context.active(), span);
    try {
      const result = await context.with(ctx, invoke);

      // Surface a model decision as a child llm.call span when available.
      const decision = decisionFromResult(result);
      if (decision) {
        this.emitDecision(ctx, action, seq, url, decision);
        // Only claim grounding when Stagehand actually resolved a selector and
        // the action did not throw. A decision that carries only reasoning (no
        // selector) is NOT evidence of grounding, so don't fabricate it — that
        // would make every such step score a perfect grounding ratio.
        if (decision.selector) {
          span.setAttribute(BrowserAttr.SELECTOR, decision.selector);
          span.setAttribute(BrowserAttr.SELECTOR_EXISTED, true);
          span.setAttribute(BrowserAttr.MATCHED_ELEMENT, true);
        }
      }
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (err) {
      span.setAttribute(BrowserAttr.STEP_STATUS, StepStatus.ERROR);
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: err instanceof Error ? err.message : String(err),
      });
      if (err instanceof Error) span.recordException(err);
      throw err;
    } finally {
      span.end();
    }
  }

  /** Emit an `llm.call` span carrying the model's `browser.reasoning`. */
  emitDecision(
    parentCtx: ReturnType<typeof context.active>,
    action: BrowserActionName,
    seq: number,
    url: string | undefined,
    decision: ModelDecision,
  ): void {
    const span: Span = this.tracer.startSpan(
      `browser.${action}.decision`,
      {
        attributes: {
          [BEATER_SPAN_KIND]: SpanKind.LLM_CALL,
          [BrowserAttr.ENGINE]: this.engine,
          [BrowserAttr.ACTION]: action,
          [BrowserAttr.STEP_SEQ]: seq,
          [BrowserAttr.STEP_STATUS]: StepStatus.OK,
          ...(url !== undefined ? { [BrowserAttr.URL]: url } : {}),
          ...(decision.reasoning !== undefined
            ? { [BrowserAttr.REASONING]: decision.reasoning }
            : {}),
          ...(decision.selector !== undefined
            ? { [BrowserAttr.SELECTOR]: decision.selector }
            : {}),
        },
      },
      parentCtx,
    );
    span.setStatus({ code: SpanStatusCode.OK });
    span.end();
  }
}
