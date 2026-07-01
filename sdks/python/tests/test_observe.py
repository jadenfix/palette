"""Offline tests: verify wrappers emit spans with the correct Beater attributes.

Uses an in-memory exporter so no live beaterd is needed.
"""

import asyncio

import pytest
from opentelemetry import trace
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter

import beater
from beater import SpanKind
from beater.semconv import Attr


@pytest.fixture
def exporter(monkeypatch):
    provider = TracerProvider(resource=Resource.create({"service.name": "test"}))
    mem = InMemorySpanExporter()
    provider.add_span_processor(SimpleSpanProcessor(mem))
    trace.set_tracer_provider(provider)
    # Point the SDK at this provider without building a real exporter.
    monkeypatch.setattr(beater.tracing, "_config", beater.BeaterConfig.resolve(release_id="rel-1"))
    monkeypatch.setattr(beater.tracing.trace, "get_tracer", lambda name: provider.get_tracer(name))
    yield mem
    mem.clear()


def _by_name(spans, name):
    return next(s for s in spans if s.name == name)


def test_observe_sync_records_kind_and_io(exporter):
    @beater.observe(kind=SpanKind.LLM_CALL, name="call")
    def call(prompt):
        return "answer"

    assert call("hi") == "answer"
    span = _by_name(exporter.get_finished_spans(), "call")
    assert span.attributes[Attr.SPAN_KIND] == SpanKind.LLM_CALL
    assert span.attributes[Attr.RELEASE_ID] == "rel-1"
    assert "answer" in span.attributes[Attr.OUTPUT_VALUE]
    assert span.attributes[Attr.SEQ] >= 1


def test_observe_async(exporter):
    @beater.observe(kind=SpanKind.AGENT_RUN, name="arun")
    async def arun(x):
        return x * 2

    assert asyncio.run(arun(3)) == 6
    span = _by_name(exporter.get_finished_spans(), "arun")
    assert span.attributes[Attr.SPAN_KIND] == SpanKind.AGENT_RUN


def test_observe_records_error(exporter):
    @beater.observe(name="boom")
    def boom():
        raise ValueError("nope")

    with pytest.raises(ValueError):
        boom()
    span = _by_name(exporter.get_finished_spans(), "boom")
    assert span.status.status_code.name == "ERROR"


def test_wrap_openai_emits_llm_span(exporter):
    class _Usage:
        prompt_tokens = 11
        completion_tokens = 7
        completion_tokens_details = None

    class _Msg:
        content = "hello there"

    class _Choice:
        message = _Msg()

    class _Resp:
        model = "gpt-4.1"
        usage = _Usage()
        choices = [_Choice()]

    class _Completions:
        def create(self, **kwargs):
            return _Resp()

    class _Chat:
        completions = _Completions()

    class _Client:
        chat = _Chat()

    client = beater.wrap_openai(_Client())
    client.chat.completions.create(model="gpt-4.1", messages=[{"role": "user", "content": "hi"}])

    span = _by_name(exporter.get_finished_spans(), "openai.chat.completions.create")
    assert span.attributes[Attr.LLM_PROVIDER] == "openai"
    assert span.attributes[Attr.LLM_MODEL_NAME] == "gpt-4.1"
    assert span.attributes[Attr.LLM_TOKEN_PROMPT] == 11
    assert span.attributes[Attr.LLM_TOKEN_COMPLETION] == 7
    assert "hello there" in span.attributes[Attr.OUTPUT_VALUE]


def test_wrap_groq_emits_openai_compatible_llm_span(exporter):
    class _Completions:
        def create(self, **kwargs):
            return {
                "model": "llama-3.3-70b-versatile",
                "usage": {"prompt_tokens": 5, "completion_tokens": 3},
                "choices": [{"message": {"content": "groq answer"}}],
            }

    class _Chat:
        completions = _Completions()

    class _Client:
        chat = _Chat()

    client = beater.wrap_groq(_Client())
    client.chat.completions.create(
        model="llama-3.3-70b-versatile",
        messages=[{"role": "user", "content": "hi"}],
    )

    span = _by_name(exporter.get_finished_spans(), "groq.chat.completions.create")
    assert span.attributes[Attr.LLM_PROVIDER] == "groq"
    assert span.attributes[Attr.LLM_MODEL_NAME] == "llama-3.3-70b-versatile"
    assert span.attributes[Attr.LLM_TOKEN_PROMPT] == 5
    assert span.attributes[Attr.LLM_TOKEN_COMPLETION] == 3
    assert "groq answer" in span.attributes[Attr.OUTPUT_VALUE]


def test_wrap_mistral_emits_openai_compatible_llm_span(exporter):
    class _Usage:
        input_tokens = 13
        output_tokens = 8

    class _Message:
        content = "mistral answer"

    class _Choice:
        message = _Message()

    class _Resp:
        model = "mistral-large-latest"
        usage = _Usage()
        choices = [_Choice()]

    class _Completions:
        def create(self, request):
            return _Resp()

    class _Chat:
        completions = _Completions()

    class _Client:
        chat = _Chat()

    client = beater.wrap_mistral(_Client())
    client.chat.completions.create(
        {
            "model": "mistral-large-latest",
            "messages": [{"role": "user", "content": "hi"}],
        }
    )

    span = _by_name(exporter.get_finished_spans(), "mistral.chat.completions.create")
    assert span.attributes[Attr.LLM_PROVIDER] == "mistral"
    assert span.attributes[Attr.LLM_MODEL_NAME] == "mistral-large-latest"
    assert span.attributes[Attr.LLM_TOKEN_PROMPT] == 13
    assert span.attributes[Attr.LLM_TOKEN_COMPLETION] == 8
    assert "mistral answer" in span.attributes[Attr.OUTPUT_VALUE]


def test_semconv_kinds_match_normalizer():
    # Guard: every kind the SDK can emit is one the server normalizer accepts.
    expected = {
        "agent.run", "agent.turn", "agent.plan", "agent.step", "llm.call",
        "tool.call", "mcp.request", "retrieval.query", "memory.read",
        "memory.write", "guardrail.check",
    }
    from beater.semconv import SPAN_KINDS

    assert set(SPAN_KINDS) == expected
