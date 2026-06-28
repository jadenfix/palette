# Beater Python SDK

Ergonomic, OpenTelemetry-native agent observability for [Beater](https://github.com/beater/beater).

This is the **Layer 2** (hand-written, ergonomic) SDK: `@observe` decorators,
drop-in OpenAI/Anthropic wrappers, and LangChain/LlamaIndex callbacks. The
**Layer 1** generated control-plane client (datasets, experiments, gates, evals,
usage, etc.) is published separately as `beater_client`, generated from the
Beater OpenAPI contract so it never drifts from the API.

## Install

```bash
pip install beater-sdk                 # core (OTLP/HTTP export)
pip install "beater-sdk[openai]"       # + OpenAI wrapper
pip install "beater-sdk[anthropic]"    # + Anthropic wrapper
pip install "beater-sdk[langchain]"    # + LangChain callback
pip install "beater-sdk[llamaindex]"   # + LlamaIndex callback
pip install "beater-sdk[grpc]"         # OTLP/gRPC export instead of HTTP
```

## Quickstart (5 lines)

```python
import beater

beater.init(tenant_id="acme", project_id="support-bot", environment_id="prod")

@beater.observe(kind=beater.SpanKind.AGENT_RUN)
def handle(query): ...
```

All `init()` arguments fall back to `BEATER_*` env vars
(`BEATER_BASE_URL`, `BEATER_TENANT_ID`, `BEATER_PROJECT_ID`,
`BEATER_ENVIRONMENT_ID`, `BEATER_API_KEY`), so `beater.init()` works with no args
when the environment is configured.

## Zero-code env bootstrap

For apps launched with OpenTelemetry Python auto-instrumentation, select the
Beater configurator and provide only environment variables. No application code
edits are required. Install the OpenTelemetry launcher and instrumentors
separately for the libraries your app uses.

```bash
export BEATER_BASE_URL=http://127.0.0.1:8080
export BEATER_TENANT_ID=demo
export BEATER_PROJECT_ID=demo
export BEATER_ENVIRONMENT_ID=local
export BEATER_SERVICE_NAME=my-agent
export OTEL_PYTHON_CONFIGURATOR=beater

opentelemetry-instrument python app.py
```

If provider constructor auto-instrumentation is installed, enable it explicitly:

```bash
export BEATER_AUTO_INSTRUMENT=openai,anthropic
```

The bootstrap module is safe to import; it initializes tracing only when the
`beater` OpenTelemetry configurator runs or `beater.bootstrap.bootstrap_from_env()`
is called directly.

## Drop-in provider wrappers

```python
from openai import OpenAI
client = beater.wrap_openai(OpenAI())
# every client.chat.completions.create(...) is now an llm.call span
# with model + token counts

from anthropic import Anthropic
client = beater.wrap_anthropic(Anthropic())
```

## Framework callbacks

```python
from beater.integrations.langchain import BeaterCallbackHandler
chain.invoke(inputs, config={"callbacks": [BeaterCallbackHandler()]})

from beater.integrations.llamaindex import BeaterLlamaIndexHandler
from llama_index.core import Settings
from llama_index.core.callbacks import CallbackManager
Settings.callback_manager = CallbackManager([BeaterLlamaIndexHandler()])
```

## Manual spans

```python
with beater.span("retrieve-policy", kind=beater.SpanKind.RETRIEVAL_QUERY) as s:
    beater.set_input("refund policy")
    beater.set_output(result)
```

## Semantic conventions

Span kinds and attribute keys live in one place — `beater.semconv` — and mirror
the server's OTLP normalizer (`crates/beater-otlp`). This keeps the SDK's emitted
spans in lockstep with what Beater ingests.

## Transport

By default the SDK exports over **OTLP/HTTP** to
`{base_url}/v1/otlp/{tenant}/{project}/{environment}/v1/traces`. Set
`protocol="grpc"` (and install the `grpc` extra) to export to the OTLP gRPC
endpoint instead.
