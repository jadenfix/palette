/** Dependency-free registry of TypeScript SDK integrations. */

export const INTEGRATION_AVAILABLE = "available" as const;
export const INTEGRATION_PLANNED = "planned" as const;

export type IntegrationStatus = typeof INTEGRATION_AVAILABLE | typeof INTEGRATION_PLANNED;
export type IntegrationKind = "framework" | "provider" | "provider-router";

export interface IntegrationSpec {
  readonly slug: string;
  readonly name: string;
  readonly kind: IntegrationKind;
  readonly status: IntegrationStatus;
  readonly packageName: string;
  readonly module?: string;
  readonly notes: string;
}

const CATALOG: readonly IntegrationSpec[] = [
  {
    slug: "openai",
    name: "OpenAI",
    kind: "provider",
    status: INTEGRATION_AVAILABLE,
    packageName: "@beater/sdk",
    module: "@beater/sdk",
    notes: "wrapOpenAI emits llm.call spans with model and token usage.",
  },
  {
    slug: "anthropic",
    name: "Anthropic",
    kind: "provider",
    status: INTEGRATION_AVAILABLE,
    packageName: "@beater/sdk",
    module: "@beater/sdk",
    notes: "wrapAnthropic emits llm.call spans with model and token usage.",
  },
  {
    slug: "langchain",
    name: "LangChain.js",
    kind: "framework",
    status: INTEGRATION_AVAILABLE,
    packageName: "@beater/sdk",
    module: "@beater/sdk",
    notes: "Callback handler maps chains, LLMs, tools, and retrievers to Beater spans.",
  },
  {
    slug: "vercel-ai-sdk",
    name: "Vercel AI SDK",
    kind: "framework",
    status: INTEGRATION_AVAILABLE,
    packageName: "@beater/sdk",
    module: "@beater/sdk",
    notes: "vercelAiTelemetry enables AI SDK OpenTelemetry spans for Beater ingest.",
  },
  {
    slug: "llamaindex",
    name: "LlamaIndex.TS",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: TypeScript callback/span-tree instrumentation.",
  },
  {
    slug: "langgraph",
    name: "LangGraph.js",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: graph node and edge span-tree instrumentation.",
  },
  {
    slug: "openai-agents-sdk",
    name: "OpenAI Agents SDK",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: agent span-tree instrumentation.",
  },
  {
    slug: "mastra",
    name: "Mastra",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: auto-instrumentation breadth parity.",
  },
  {
    slug: "livekit-agents",
    name: "LiveKit Agents",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: voice and real-time agent instrumentation.",
  },
  {
    slug: "agentscope",
    name: "AgentScope",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: multi-agent span-tree instrumentation.",
  },
  {
    slug: "google-adk",
    name: "Google ADK",
    kind: "framework",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: agent span-tree instrumentation.",
  },
  {
    slug: "litellm",
    name: "LiteLLM",
    kind: "provider-router",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: provider-router and gateway instrumentation.",
  },
  {
    slug: "bedrock",
    name: "Amazon Bedrock",
    kind: "provider",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: provider wrapper and token usage extraction.",
  },
  {
    slug: "mistral",
    name: "Mistral",
    kind: "provider",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: provider wrapper and token usage extraction.",
  },
  {
    slug: "groq",
    name: "Groq",
    kind: "provider",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: provider wrapper and token usage extraction.",
  },
  {
    slug: "gemini",
    name: "Gemini",
    kind: "provider",
    status: INTEGRATION_PLANNED,
    packageName: "@beater/sdk",
    notes: "Architecture backlog: provider wrapper and token usage extraction.",
  },
];

export function integrationCatalog(): readonly IntegrationSpec[] {
  return [...CATALOG].sort((left, right) => left.slug.localeCompare(right.slug));
}

export function availableIntegrations(): readonly IntegrationSpec[] {
  return integrationCatalog().filter((spec) => spec.status === INTEGRATION_AVAILABLE);
}

export function plannedIntegrations(): readonly IntegrationSpec[] {
  return integrationCatalog().filter((spec) => spec.status === INTEGRATION_PLANNED);
}

export function findIntegration(slug: string): IntegrationSpec | undefined {
  const normalized = slug.trim().toLowerCase();
  return integrationCatalog().find((spec) => spec.slug === normalized);
}
