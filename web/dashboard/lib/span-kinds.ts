export const LLM_CALL_SPAN_KIND = "llm.call";

export const AGENT_SPAN_KINDS = [
  "agent.run",
  "agent.turn",
  "agent.plan",
  "agent.step",
  LLM_CALL_SPAN_KIND,
  "tool.call",
  "mcp.request",
  "retrieval.query",
  "memory.read",
  "memory.write",
  "guardrail.check",
  "human.review",
  "evaluator.run",
  "replay.run"
] as const;

export type AgentSpanKindValue = (typeof AGENT_SPAN_KINDS)[number];

export type SpanKindMeta = {
  key: string;
  title: string;
};

export function isLlmCallKind(kind: string): boolean {
  return kind === LLM_CALL_SPAN_KIND;
}

export function apiSpanIoLabels(kind: string): { input: string; output: string } {
  if (isLlmCallKind(kind)) return { input: "prompt", output: "completion" };
  return { input: "input", output: "output" };
}

export function displaySpanIoLabels(kind: string): { input: string; output: string } {
  if (isLlmCallKind(kind)) return { input: "Prompt", output: "Completion" };
  return { input: "Input", output: "Output" };
}

export function spanKindClass(kind: string): string {
  if (kind.startsWith("agent.")) return "agent";
  if (isLlmCallKind(kind)) return "llm";
  if (kind === "tool.call" || kind === "mcp.request") return "tool";
  if (kind.startsWith("memory.")) return "memory";
  if (kind.includes("guardrail")) return "guardrail";
  if (kind.includes("evaluator")) return "eval";
  if (kind === "human.review") return "human";
  if (kind === "replay.run") return "replay";
  return "other";
}

export function spanKindMeta(kind: string): SpanKindMeta {
  if (kind === "agent.run") return { key: "agent-run", title: "Agent run" };
  if (kind === "agent.turn") return { key: "agent-turn", title: "Agent turn" };
  if (kind === "agent.plan") return { key: "agent-plan", title: "Agent plan" };
  if (kind === "agent.step") return { key: "agent-step", title: "Agent step" };
  if (isLlmCallKind(kind)) return { key: "llm", title: "LLM call" };
  if (kind === "tool.call") return { key: "tool", title: "Tool call" };
  if (kind === "mcp.request") return { key: "mcp", title: "MCP request" };
  if (kind === "retrieval.query") return { key: "retrieval", title: "Retrieval query" };
  if (kind === "memory.read") return { key: "memory-read", title: "Memory read" };
  if (kind === "memory.write") return { key: "memory-write", title: "Memory write" };
  if (kind === "guardrail.check") return { key: "guardrail", title: "Guardrail check" };
  if (kind === "human.review") return { key: "human", title: "Human review" };
  if (kind === "evaluator.run") return { key: "eval", title: "Evaluator run" };
  if (kind === "replay.run") return { key: "replay", title: "Replay run" };
  return { key: "other", title: kind };
}
