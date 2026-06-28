# @beater/stagehand-instrumentation

Instrumentation SDK that wraps the [Stagehand](https://github.com/browserbase/stagehand)
browser-automation SDK (`page.act` / `page.observe` / `page.extract`, built on
Playwright) and emits canonical Beater `browser.*` spans over OTLP gRPC.

One `instrumentStagehand(stagehand)` call turns every AI primitive into a
`tool.call` span, and surfaces any model decision as a child `llm.call` span
carrying `browser.reasoning`. Spans are exported to Beater's OTLP gRPC ingest on
`localhost:4317` (configurable).

## Architecture fit

Generic OTLP/OpenInference/OpenLLMetry ingest remains Beater's zero-code floor:
any already-instrumented app should export to Beater without this package. Use
`@beater/stagehand-instrumentation` when you need the first-class Stagehand
vertical from `ARCHITECTURE.md` §28.1: Playwright-backed browser action hooks,
grounded action metadata, and canonical `browser.*` spans that generic LLM-only
exporters usually cannot see.

## Install

```bash
npm install @beater/stagehand-instrumentation @browserbasehq/stagehand
```

`@browserbasehq/stagehand` is an optional peer dependency — the SDK and its
tests do not require a live browser.

## Quickstart

```ts
import { Stagehand } from "@browserbasehq/stagehand";
import { instrumentStagehand } from "@beater/stagehand-instrumentation";

const stagehand = new Stagehand({ env: "LOCAL" });
await stagehand.init();

// Wrap act/observe/extract. Spans go to Beater over OTLP gRPC.
instrumentStagehand(stagehand, {
  endpoint: process.env.BEATER_OTLP_ENDPOINT ?? "http://localhost:4317",
  serviceName: "my-browser-agent",
});

await stagehand.page.goto("https://news.ycombinator.com");
await stagehand.page.observe("the top story link"); // -> tool.call span
await stagehand.page.act("click the first story");   // -> tool.call span
await stagehand.page.extract({ instruction: "the article title" });
```

### Configuration

| Option        | Env var                | Default                  |
| ------------- | ---------------------- | ------------------------ |
| `endpoint`    | `BEATER_OTLP_ENDPOINT` | `http://localhost:4317`  |
| `serviceName` | —                      | `stagehand-browser-agent`|
| `engine`      | —                      | `chromium`               |
| `tracer`      | —                      | OTLP gRPC pipeline       |

Pass your own `tracer` to reuse an existing OpenTelemetry pipeline (or an
`InMemorySpanExporter` in tests) instead of bootstrapping a new OTLP exporter.

## Emitted spans

Each wrapped call emits a `tool.call` span (`beater.span.kind = tool.call`):

| Attribute                  | Value                                   |
| -------------------------- | --------------------------------------- |
| `browser.engine`           | `chromium` (or configured engine)       |
| `browser.action`           | `act` \| `observe` \| `extract`         |
| `browser.url`              | `page.url()` at call time               |
| `browser.title`            | `page.title()` at call time             |
| `browser.selector`         | selector/instruction hint, if present   |
| `browser.step_seq`         | monotonic step number                   |
| `browser.step_status`      | `ok` \| `error`                         |
| `browser.selector_existed` | `true` when the model grounded a target |
| `browser.matched_element`  | `true` when the model grounded a target |

When a call (or its result) carries a model decision, a child `llm.call` span
(`beater.span.kind = llm.call`) is emitted with `browser.reasoning` and, when
available, `browser.selector`.

These keys match `crates/beater-browser/src/semconv.rs` exactly so Beater's OTLP
ingest normalizes them identically to the native capture layer and the other
instrumentation SDKs.

## Develop / test

Tests use an in-memory span exporter and a mock page — no browser or Stagehand
required:

```bash
cd sdks/ts-stagehand
npm install
npm test
```

```bash
npm run build   # tsc -> dist/
npm run example # requires @browserbasehq/stagehand + a running Beater
```
