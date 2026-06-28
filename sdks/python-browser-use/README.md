# beater-browser-use

Beater instrumentation for the [`browser-use`](https://github.com/browser-use/browser-use)
agent framework. It hooks an `Agent` and emits **canonical `browser.*` browser-step
spans** over OTLP/gRPC into Beater, so browser-agent runs land in the same
observability/eval pipeline as everything else.

## Architecture fit

Generic OTLP/OpenInference/OpenLLMetry ingest remains Beater's zero-code floor:
any already-instrumented app should export to Beater without this package. Use
`beater-browser-use` when you need the first-class browser-use vertical from
`ARCHITECTURE.md` §28.1: browser step lifecycle hooks, grounded action metadata,
and canonical `browser.*` spans that generic LLM-only exporters usually cannot
see.

Per browser step it emits a span pair:

- **`llm.call`** — the decision: `browser.reasoning` (from `model_thoughts`), an
  `llm.model` attribute, and `browser.step_seq`.
- **`tool.call`** — the action (child of the `llm.call`): `browser.action` (verb),
  `browser.selector`, `browser.url`, `browser.step_seq`, `browser.step_status`
  (`ok`/`error`), plus grounding signals `browser.selector_existed` /
  `browser.matched_element`, optional `browser.title` / `browser.engine`, and
  optional `browser.dom_artifact_id` / `browser.screenshot_artifact_id`.

The `browser.*` attribute keys are the cross-language ingest contract, mirrored
from the Rust source of truth in `crates/beater-browser/src/semconv.rs`
(see [`beater_browser_use/semconv.py`](beater_browser_use/semconv.py)).

## Install

```bash
pip install beater-browser-use            # SDK + opentelemetry deps
pip install beater-browser-use[browser-use]   # + browser-use for live runs
```

`browser-use` is an **optional extra** — the SDK and its unit tests run with only
`opentelemetry` installed.

## Quickstart

```python
import asyncio
from browser_use import Agent
from beater_browser_use import instrument

async def main():
    agent = Agent(task="Subscribe to the newsletter on example.com", llm=...)

    # Defaults to $BEATER_OTLP_ENDPOINT or localhost:4317.
    inst = instrument(agent, endpoint="localhost:4317")

    await agent.run(**inst.run_kwargs())   # splats on_step_start / on_step_end
    inst.tracer.shutdown()                 # flush spans to Beater

asyncio.run(main())
```

You must pass the hooks to `run()` (via `**inst.run_kwargs()` or the unpacked
`on_step_start, on_step_end`). If you cannot, call
`instrument(agent, register_step_callback=True)` to instead wire a
`register_new_step_callback` on the agent — but do not enable both, or every
step is recorded twice.

### Lower-level: build hooks yourself

```python
from beater_browser_use import BeaterBrowserUseTracer, make_hooks

tracer = BeaterBrowserUseTracer(endpoint="localhost:4317")
on_step_start, on_step_end = make_hooks(tracer)
await agent.run(on_step_start=on_step_start, on_step_end=on_step_end)
tracer.shutdown()
```

## Configuration

| Env var | Default | Meaning |
| --- | --- | --- |
| `BEATER_OTLP_ENDPOINT` | `localhost:4317` | OTLP/gRPC endpoint Beater ingests on |

`endpoint=` passed to `instrument()` / `BeaterBrowserUseTracer` overrides the env var.

## Example

See [`examples/run_agent.py`](examples/run_agent.py) for a full runnable script.

## Fixtures

[`fixtures/recorded_run.json`](fixtures/recorded_run.json) is a small, representative
3-step run in the `{"browser_steps": [...]}` shape (a grounded success ending at a
confirmation page, plus a grounding miss). It is reused by a downstream Rust
integration test and can be replayed through the exporter via
`StepRecord.from_outcome(step, seq)`.

## Tests

The unit tests require only `opentelemetry-sdk` + `pytest` (no live browser, no
`browser-use`). They feed fake/duck-typed agent-history objects into the mapping
and assert the emitted spans via OpenTelemetry's `InMemorySpanExporter`.

```bash
cd sdks/python-browser-use
pip install opentelemetry-sdk opentelemetry-exporter-otlp pytest
python -m pytest
```
