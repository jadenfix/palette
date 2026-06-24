# Temporal → Beater

Beater meets Temporal users where they already are. There are **two language-agnostic
paths**, both landing in the same canonical span model (workflow run → `agent.run`
root, activity → `tool.call`, child workflow → nested `agent.run`, timer/signal →
`agent.step`) and the same downstream debug/eval/gate pipeline as every other source.

You do **not** need a Beater SDK in any language.

## Path A — Live capture (no Beater code, any SDK)

Every Temporal SDK (Go, Java, Python, TypeScript, .NET) ships an OpenTelemetry
tracing interceptor. Point it at Beater's existing OTLP endpoint and Beater
recognizes Temporal's spans automatically (by `temporal.*` attributes and by the
interceptor's span names like `RunWorkflow:`, `StartActivity:`, `RunActivity:`,
`StartChildWorkflow:`).

Python example:

```python
from temporalio.client import Client
from temporalio.contrib.opentelemetry import TracingInterceptor
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter

provider = TracerProvider()
provider.add_span_processor(BatchSpanProcessor(OTLPSpanExporter(
    endpoint="http://127.0.0.1:4317",
    headers=(
        ("x-beater-tenant-id", "demo"),
        ("x-beater-project-id", "demo"),
        ("x-beater-environment-id", "local"),
    ),
)))
trace.set_tracer_provider(provider)

# Beater needs nothing else — just register Temporal's OTel interceptor.
client = await Client.connect("localhost:7233", interceptors=[TracingInterceptor()])
```

Recognized spans are tagged `beater.framework=temporal` for easy filtering.

## Path B — History import (backfill, completed/in-flight workflows)

Fetch a workflow's durable history and POST it to the unified, source-agnostic
import endpoint. Works for any worker language because history is a server-side
artifact — the worker's language is irrelevant.

```bash
# Export a workflow's history as JSON (Temporal CLI):
temporal workflow show --workflow-id order-A-100 --output json > history.json

# Import it into Beater (source selector picks the Temporal normalizer):
jq '{source: "temporal_history", payload: .}' history.json \
  | curl -sS -X POST "http://127.0.0.1:8080/v1/import/demo/demo/local" \
      -H "content-type: application/json" --data @- | jq
```

The same endpoint accepts `"source": "native"` (a `{ "spans": [...] }` body) — so
imports are pluggable: native or from Temporal, by a single field.

A ready-to-try fixture lives at
[`../../crates/beater-temporal/tests/fixtures/order_workflow_history.json`](../../crates/beater-temporal/tests/fixtures/order_workflow_history.json).

## Mapping reference

| Temporal | Beater span |
| --- | --- |
| `WorkflowExecutionStarted` | `agent.run` (trace root; `trace_id` = run id) |
| `ActivityTaskScheduled/Started/Completed/Failed/TimedOut/Canceled` | `tool.call` |
| `StartChildWorkflowExecutionInitiated` + `ChildWorkflowExecution*` | nested `agent.run` |
| `TimerStarted/Fired/Canceled` | `agent.step` (`timer:<id>`) |
| `WorkflowExecutionSignaled` | `agent.step` (`signal:<name>`) |
| `MarkerRecorded` | `agent.step` (`marker:<name>`) |
| workflow-task / update / search-attr / nexus / external-wf bookkeeping | recognized, no span |
| anything not in the pinned contract | counted as unmapped, preserved in the raw envelope |

## Anti-drift

The History converter is pinned to a declared Temporal schema
(`beater_temporal::TEMPORAL_HISTORY_CONTRACT`). Every targeted event type is listed
in `KNOWN_EVENT_TYPES` and classified explicitly — there is no silent wildcard. A new
Temporal event type is counted as unmapped (never dropped) and surfaced by the
crate's exhaustiveness test and the `check_temporal_contract` CI gate, so the
mapping cannot silently drift from Temporal.
