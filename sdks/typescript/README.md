# Beater TypeScript SDK

Ergonomic OpenTelemetry-native instrumentation for Beater.

```ts
import * as beater from "@beater/sdk";

beater.init({
  tenantId: "acme",
  projectId: "support-bot",
  environmentId: "prod",
});

beater.instrument({ providers: ["openai", "anthropic"] });
```

`instrument()` monkeypatches installed provider SDK constructors so clients
created after the call are wrapped automatically:

```ts
beater.instrument({ providers: ["openai"] });

const OpenAI = require("openai");
const openai = new OpenAI();
await openai.chat.completions.create({
  model: "gpt-4.1",
  messages: [{ role: "user", content: "hello" }],
});
```

Supported providers:

- `openai`: patches `openai` exports and wraps `chat.completions.create` plus
  `responses.create` when present.
- `anthropic`: patches `@anthropic-ai/sdk` or `anthropic` exports and wraps
  `messages.create`.

OpenAI and Anthropic are optional runtime dependencies. If a provider package is
not installed, `instrument()` skips it and returns the skip reason instead of
throwing. The direct wrappers remain available when you already have a client
instance:

```ts
const wrapped = beater.wrapOpenAI(existingOpenAIClient);
```

Call `instrument()` before constructing provider clients. For the broadest
CommonJS monkeypatch coverage, call it before importing provider SDKs.

## Vercel AI SDK

The AI SDK emits OpenTelemetry spans when telemetry is enabled on a request. Beater
can ingest those spans through the same OTLP pipeline as the rest of the SDK.

```ts
import { generateText } from "ai";
import { openai } from "@ai-sdk/openai";
import { withVercelAiTelemetry } from "@beater/sdk";

const result = await generateText(
  withVercelAiTelemetry(
    {
      model: openai("gpt-4o-mini"),
      prompt: "Draft a concise support reply.",
    },
    { functionId: "support-reply", recordInputs: false },
  ),
);
```

For AI SDK versions or observability providers that use the newer option name:

```ts
withVercelAiTelemetry(request, { optionName: "telemetry" });
```

## Integration Registry

```ts
import { availableIntegrations, plannedIntegrations } from "@beater/sdk";

availableIntegrations().map((spec) => spec.slug);
plannedIntegrations().map((spec) => spec.slug);
```
