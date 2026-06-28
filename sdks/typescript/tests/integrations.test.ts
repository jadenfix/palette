import assert from "node:assert";
import { test } from "node:test";

import * as beater from "../src/index";

test("TypeScript integration registry exposes current SDK support", () => {
  const available = Object.fromEntries(beater.availableIntegrations().map((spec) => [spec.slug, spec]));

  assert.equal(available.openai.module, "@beater/sdk");
  assert.equal(available.anthropic.module, "@beater/sdk");
  assert.equal(available.langchain.status, beater.INTEGRATION_AVAILABLE);
  assert.match(available["vercel-ai-sdk"].notes, /OpenTelemetry/);
  assert.ok(beater.availableIntegrations().every((spec) => spec.status === beater.INTEGRATION_AVAILABLE));
});

test("TypeScript integration registry tracks architecture backlog candidates", () => {
  const planned = new Set(beater.plannedIntegrations().map((spec) => spec.slug));

  for (const slug of [
    "llamaindex",
    "langgraph",
    "openai-agents-sdk",
    "mastra",
    "livekit-agents",
    "agentscope",
    "google-adk",
    "litellm",
    "bedrock",
    "mistral",
    "groq",
    "gemini",
  ]) {
    assert.ok(planned.has(slug), `expected ${slug} to be tracked`);
  }
});

test("TypeScript integration registry is stable and searchable", () => {
  const catalog = beater.integrationCatalog();
  const slugs = catalog.map((spec) => spec.slug);

  assert.equal(slugs.length, new Set(slugs).size);
  assert.deepEqual(slugs, [...slugs].sort());
  assert.equal(beater.findIntegration("  Vercel-AI-SDK  ")?.status, beater.INTEGRATION_AVAILABLE);
  assert.equal(beater.findIntegration("missing"), undefined);
});

test("Vercel AI SDK telemetry helper builds supported telemetry option shapes", () => {
  const telemetry = beater.vercelAiTelemetry({
    functionId: "support-reply",
    recordInputs: false,
    recordOutputs: false,
    metadata: { tenant: "acme" },
  });

  assert.deepEqual(telemetry, {
    isEnabled: true,
    functionId: "support-reply",
    recordInputs: false,
    recordOutputs: false,
    metadata: { tenant: "acme" },
  });

  const request = { prompt: "Write a short reply" };
  const current = beater.withVercelAiTelemetry(request, { functionId: "current" });
  const stable = beater.withVercelAiTelemetry(request, {
    optionName: "telemetry",
    enabled: false,
  });

  assert.deepEqual(request, { prompt: "Write a short reply" });
  assert.equal(current.experimental_telemetry?.functionId, "current");
  assert.equal(stable.telemetry?.isEnabled, false);
});
