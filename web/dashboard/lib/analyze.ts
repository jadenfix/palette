/**
 * Client-side trace analysis — pure functions over the spans the dashboard has
 * already loaded. No backend, no statistics infra; deterministic and unit
 * tested. These are the dashboard-side previews of the `analyze.*` family that
 * will eventually live behind `/v1` handlers (see issue #63).
 */

export type TimedSpan = {
  span_id: string;
  parent_span_id?: string | null;
  start_time?: string | null;
  end_time?: string | null;
  name?: string | null;
};

/**
 * Parse an RFC3339 timestamp to microseconds. Sub-millisecond precision matters
 * for ordering near-simultaneous spans, so the fractional part is read directly
 * rather than going through `Date` (which is millisecond-resolution). Mirrors
 * `timestampMicros` in lib/api.ts but kept local so this module has no deps.
 */
export function toMicros(value: string | null | undefined): number | null {
  if (!value) return null;
  const dot = value.indexOf(".");
  const head = dot >= 0 ? `${value.slice(0, dot)}Z` : value;
  const ms = Date.parse(head);
  if (Number.isNaN(ms)) return null;
  let micros = ms * 1000;
  if (dot >= 0) {
    const frac = value.slice(dot + 1).replace(/[^0-9]/g, "");
    if (frac) micros += parseInt(frac.padEnd(6, "0").slice(0, 6), 10);
  }
  return micros;
}

/** Build a parent→children map that is safe against cycles and missing parents. */
function childMap<T extends TimedSpan>(spans: T[]): {
  byId: Map<string, T>;
  children: Map<string, T[]>;
  roots: T[];
} {
  const byId = new Map(spans.map((s) => [s.span_id, s]));
  const children = new Map<string, T[]>();
  const roots: T[] = [];
  for (const s of spans) {
    const parent = s.parent_span_id;
    const hasParent =
      parent != null && parent !== s.span_id && byId.has(parent) && !isAncestorCycle(s, byId);
    if (hasParent) {
      const list = children.get(parent as string) ?? [];
      list.push(s);
      children.set(parent as string, list);
    } else {
      roots.push(s);
    }
  }
  return { byId, children, roots };
}

/** Detect a parent chain that loops back on itself (malformed trace). */
function isAncestorCycle<T extends TimedSpan>(span: T, byId: Map<string, T>): boolean {
  const seen = new Set<string>([span.span_id]);
  let parent = span.parent_span_id;
  while (parent && byId.has(parent)) {
    if (seen.has(parent)) return true;
    seen.add(parent);
    parent = byId.get(parent)?.parent_span_id ?? null;
  }
  return false;
}

function endMicros(s: TimedSpan): number {
  return toMicros(s.end_time) ?? toMicros(s.start_time) ?? 0;
}
function startMicros(s: TimedSpan): number {
  return toMicros(s.start_time) ?? endMicros(s);
}

/**
 * The critical path: the chain of nested spans that determines when the trace
 * finishes. Wall-clock latency is the longest path through the timed span DAG,
 * not the sum of span durations — so we find the span that ends last and walk
 * its ancestry to the root. Those are the spans that were still open at the
 * moment the run completed; shortening any of them shortens the run.
 */
export function criticalPathSpanIds(spans: TimedSpan[]): Set<string> {
  if (spans.length === 0) return new Set();
  const { byId } = childMap(spans);
  // The span that finishes last drives completion; ties break on later start
  // (the more deeply nested of two co-ending spans), then on id for determinism.
  let target = spans[0];
  for (const s of spans) {
    const e = endMicros(s);
    const te = endMicros(target);
    if (e > te || (e === te && startMicros(s) > startMicros(target)) ||
      (e === te && startMicros(s) === startMicros(target) && s.span_id < target.span_id)) {
      target = s;
    }
  }
  const path = new Set<string>();
  let cur: TimedSpan | undefined = target;
  const guard = new Set<string>();
  while (cur && !guard.has(cur.span_id)) {
    const node: TimedSpan = cur;
    guard.add(node.span_id);
    path.add(node.span_id);
    const parentId: string | null | undefined = node.parent_span_id;
    cur = parentId && parentId !== node.span_id ? byId.get(parentId) : undefined;
  }
  return path;
}

export type CriticalPathStats = {
  spanCount: number;
  totalMs: number;
  /** A pair of sibling spans that ran back-to-back and, if independent, could be
   *  parallelized. Hedged on purpose: the trace can't prove independence. */
  parallelizable: { a: string; b: string; savingsMs: number } | null;
};

/** Summary stats for the critical path, including a modest parallelization hint. */
export function criticalPathStats(spans: TimedSpan[]): CriticalPathStats {
  const ids = criticalPathSpanIds(spans);
  if (ids.size === 0) return { spanCount: 0, totalMs: 0, parallelizable: null };

  const onPath = spans.filter((s) => ids.has(s.span_id));
  const start = Math.min(...onPath.map(startMicros));
  const end = Math.max(...onPath.map(endMicros));
  const totalMs = Math.max(0, (end - start) / 1000);

  return { spanCount: ids.size, totalMs, parallelizable: findParallelizable(spans) };
}

/**
 * Find the sibling pair with the largest "if you ran these at the same time"
 * saving: two children of one parent whose intervals don't overlap (they ran
 * sequentially). Saving ≈ the shorter of the two. Returns the best candidate or
 * null. Caller must frame this as a hypothesis, not a promise.
 */
function findParallelizable(
  spans: TimedSpan[],
): { a: string; b: string; savingsMs: number } | null {
  const { children } = childMap(spans);
  let best: { a: string; b: string; savingsMs: number } | null = null;
  for (const siblings of children.values()) {
    for (let i = 0; i < siblings.length; i++) {
      for (let j = i + 1; j < siblings.length; j++) {
        const x = siblings[i];
        const y = siblings[j];
        const xs = startMicros(x);
        const xe = endMicros(x);
        const ys = startMicros(y);
        const ye = endMicros(y);
        const sequential = xe <= ys || ye <= xs; // no overlap
        if (!sequential) continue;
        const savingsMs = Math.min(xe - xs, ye - ys) / 1000;
        if (savingsMs <= 0) continue;
        if (!best || savingsMs > best.savingsMs) {
          const [first, second] = xs <= ys ? [x, y] : [y, x];
          best = {
            a: first.name ?? first.span_id,
            b: second.name ?? second.span_id,
            savingsMs,
          };
        }
      }
    }
  }
  return best;
}

/** Format a millisecond duration the way the rest of the console does. */
export function formatMs(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return "—";
  if (ms < 1) return `${(ms * 1000).toFixed(0)} µs`;
  if (ms < 1000) return `${ms < 10 ? ms.toFixed(1) : Math.round(ms)} ms`;
  return `${(ms / 1000).toFixed(ms < 10000 ? 2 : 1)} s`;
}
