"""Beater Python SDK -- ergonomic agent observability.

Quickstart::

    import beater

    beater.init(tenant_id="acme", project_id="support-bot", environment_id="prod")

    @beater.observe(kind=beater.SpanKind.AGENT_RUN)
    def handle(query): ...

Drop-in provider instrumentation::

    from openai import OpenAI
    client = beater.wrap_openai(OpenAI())
    from groq import Groq
    client = beater.wrap_groq(Groq())

Auto-instrument installed providers::

    beater.instrument(providers=["openai", "anthropic"])

This is the hand-written ergonomic (Layer 2) SDK. The generated control-plane
client (Layer 1, for datasets/experiments/gates/etc.) lives in the
``beater_client`` package generated from the OpenAPI contract.
"""

from __future__ import annotations

from .auto import InstrumentationResult, instrument
from .config import BeaterConfig
from .integrations import BeaterCallbackHandler, BeaterLlamaIndexHandler
from .observe import observe, set_input, set_output, span
from .providers.anthropic import wrap_anthropic
from .providers.openai import wrap_openai
from .providers.openai_compatible import wrap_groq, wrap_mistral, wrap_openai_compatible
from .semconv import Attr, SpanKind
from .tracing import flush, get_config, init, shutdown

__all__ = [
    "init",
    "observe",
    "span",
    "set_input",
    "set_output",
    "instrument",
    "wrap_openai",
    "wrap_anthropic",
    "wrap_openai_compatible",
    "wrap_groq",
    "wrap_mistral",
    "BeaterCallbackHandler",
    "BeaterLlamaIndexHandler",
    "flush",
    "shutdown",
    "get_config",
    "InstrumentationResult",
    "BeaterConfig",
    "SpanKind",
    "Attr",
]

__version__ = "0.1.0"
