"""Behavioral tests for the framework integration adapters.

Drives the LangChain and LlamaIndex callback handlers through a simulated run
and asserts they emit real Beater spans (kinds, I/O, token usage) via an
in-memory exporter -- no live beaterd and no framework install required.
"""

from uuid import uuid4

import pytest
from opentelemetry import trace
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter

import beater
from beater import BeaterCallbackHandler, BeaterLlamaIndexHandler, SpanKind
from beater.semconv import Attr


@pytest.fixture
def exporter(monkeypatch):
    provider = TracerProvider(resource=Resource.create({"service.name": "test"}))
    mem = InMemorySpanExporter()
    provider.add_span_processor(SimpleSpanProcessor(mem))
    trace.set_tracer_provider(provider)
    monkeypatch.setattr(beater.tracing, "_config", beater.BeaterConfig.resolve(release_id="rel-1"))
    monkeypatch.setattr(beater.tracing.trace, "get_tracer", lambda name: provider.get_tracer(name))
    yield mem
    mem.clear()


def _by_name(spans, name):
    return next(s for s in spans if s.name == name)


def test_adapters_are_public_exports():
    # Mirrors the TypeScript SDK, which exports the same adapters from its root.
    from beater.integrations import BeaterCallbackHandler as FromPkg

    assert FromPkg is BeaterCallbackHandler
    assert "BeaterCallbackHandler" in beater.__all__
    assert "BeaterLlamaIndexHandler" in beater.__all__


def test_langchain_handler_emits_chain_llm_tool_spans(exporter):
    handler = BeaterCallbackHandler()

    chain_id = uuid4()
    llm_id = uuid4()
    tool_id = uuid4()

    # A chain that runs an LLM call and a tool call beneath it.
    handler.on_chain_start({"name": "my_chain"}, {"question": "weather?"}, run_id=chain_id)

    handler.on_llm_start(
        {"name": "ChatOpenAI"},
        ["weather?"],
        run_id=llm_id,
        parent_run_id=chain_id,
        invocation_params={"model": "gpt-4.1"},
    )

    class _Gen:
        text = "it is sunny"

    class _Response:
        generations = [[_Gen()]]
        llm_output = {"token_usage": {"prompt_tokens": 9, "completion_tokens": 4}}

    handler.on_llm_end(_Response(), run_id=llm_id)

    handler.on_tool_start({"name": "lookup"}, "san francisco", run_id=tool_id, parent_run_id=chain_id)
    handler.on_tool_end("72F", run_id=tool_id)

    handler.on_chain_end({"answer": "sunny"}, run_id=chain_id)

    spans = exporter.get_finished_spans()
    names = {s.name for s in spans}
    assert {"my_chain", "llm", "lookup"} <= names

    chain_span = _by_name(spans, "my_chain")
    assert chain_span.attributes[Attr.SPAN_KIND] == SpanKind.AGENT_STEP
    assert "weather" in chain_span.attributes[Attr.INPUT_VALUE]
    assert "sunny" in chain_span.attributes[Attr.OUTPUT_VALUE]

    llm_span = _by_name(spans, "llm")
    assert llm_span.attributes[Attr.SPAN_KIND] == SpanKind.LLM_CALL
    assert llm_span.attributes[Attr.LLM_MODEL_NAME] == "gpt-4.1"
    assert llm_span.attributes[Attr.LLM_TOKEN_PROMPT] == 9
    assert llm_span.attributes[Attr.LLM_TOKEN_COMPLETION] == 4
    assert "sunny" in llm_span.attributes[Attr.OUTPUT_VALUE]

    tool_span = _by_name(spans, "lookup")
    assert tool_span.attributes[Attr.SPAN_KIND] == SpanKind.TOOL_CALL
    assert "72F" in tool_span.attributes[Attr.OUTPUT_VALUE]

    # The LLM and tool spans nest under the chain span (same trace).
    assert llm_span.parent.span_id == chain_span.context.span_id
    assert tool_span.parent.span_id == chain_span.context.span_id


def test_langchain_handler_records_llm_error(exporter):
    handler = BeaterCallbackHandler()
    run_id = uuid4()
    handler.on_llm_start({"name": "ChatOpenAI"}, ["hi"], run_id=run_id)
    handler.on_llm_error(ValueError("rate limited"), run_id=run_id)

    span = _by_name(exporter.get_finished_spans(), "llm")
    assert span.status.status_code.name == "ERROR"


def test_llamaindex_handler_emits_event_spans(exporter):
    handler = BeaterLlamaIndexHandler()

    class _Event:
        value = "llm"

    handler.on_event_start(_Event(), payload={"messages": "summarize this"}, event_id="e1")
    handler.on_event_end(_Event(), payload={"response": "a summary"}, event_id="e1")

    span = _by_name(exporter.get_finished_spans(), "llamaindex.llm")
    assert span.attributes[Attr.SPAN_KIND] == SpanKind.LLM_CALL
    assert "summarize" in span.attributes[Attr.INPUT_VALUE]
    assert "summary" in span.attributes[Attr.OUTPUT_VALUE]
    assert span.status.status_code.name == "OK"
