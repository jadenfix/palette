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
