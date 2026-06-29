import sys
import types

import pytest
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter

import beater
from beater.semconv import Attr


@pytest.fixture
def exporter(monkeypatch):
    provider = TracerProvider(resource=Resource.create({"service.name": "test"}))
    mem = InMemorySpanExporter()
    provider.add_span_processor(SimpleSpanProcessor(mem))
    monkeypatch.setattr(beater.tracing, "_config", beater.BeaterConfig.resolve(release_id="rel-1"))
    monkeypatch.setattr(beater.tracing, "_provider", provider)
    monkeypatch.setattr(beater.tracing.trace, "get_tracer", lambda name: provider.get_tracer(name))
    yield mem
    mem.clear()


def _by_name(spans, name):
    return next(s for s in spans if s.name == name)


def _fake_openai_module():
    module = types.ModuleType("openai")

    class _Usage:
        prompt_tokens = 3
        completion_tokens = 5
        completion_tokens_details = None

    class _Message:
        content = "hello from auto"

    class _Choice:
        message = _Message()

    class _Response:
        model = "gpt-auto"
        usage = _Usage()
        choices = [_Choice()]

    class _Completions:
        def create(self, **kwargs):
            return _Response()

    class _Chat:
        def __init__(self):
            self.completions = _Completions()

    class OpenAI:
        def __init__(self):
            self.chat = _Chat()

    module.OpenAI = OpenAI
    return module


def _fake_anthropic_module():
    module = types.ModuleType("anthropic")

    class _Usage:
        input_tokens = 7
        output_tokens = 11
        cache_read_input_tokens = None

    class _Block:
        text = "anthropic auto"

    class _Response:
        model = "claude-auto"
        usage = _Usage()
        content = [_Block()]

    class _Messages:
        def create(self, **kwargs):
            return _Response()

    class Anthropic:
        def __init__(self):
            self.messages = _Messages()

    module.Anthropic = Anthropic
    return module


def test_instrument_openai_patches_new_clients(monkeypatch, exporter):
    module = _fake_openai_module()
    monkeypatch.setitem(sys.modules, "openai", module)

    result = beater.instrument(providers=["openai"])

    assert result[0].provider == "openai"
    assert result[0].instrumented is True
    assert result[0].status == "patched openai.OpenAI"
    client = module.OpenAI()
    assert client._beater_wrapped is True

    client.chat.completions.create(model="gpt-auto", messages=[{"role": "user", "content": "hi"}])

    span = _by_name(exporter.get_finished_spans(), "openai.chat.completions.create")
    assert span.attributes[Attr.LLM_PROVIDER] == "openai"
    assert span.attributes[Attr.LLM_MODEL_NAME] == "gpt-auto"
    assert span.attributes[Attr.LLM_TOKEN_PROMPT] == 3
    assert span.attributes[Attr.LLM_TOKEN_COMPLETION] == 5
    assert "hello from auto" in span.attributes[Attr.OUTPUT_VALUE]


def test_instrument_anthropic_patches_new_clients(monkeypatch, exporter):
    module = _fake_anthropic_module()
    monkeypatch.setitem(sys.modules, "anthropic", module)

    result = beater.instrument(providers=["anthropic"])

    assert result[0].provider == "anthropic"
    assert result[0].instrumented is True
    assert result[0].status == "patched anthropic.Anthropic"
    client = module.Anthropic()
    assert client._beater_wrapped is True

    client.messages.create(model="claude-auto", messages=[{"role": "user", "content": "hi"}])

    span = _by_name(exporter.get_finished_spans(), "anthropic.messages.create")
    assert span.attributes[Attr.LLM_PROVIDER] == "anthropic"
    assert span.attributes[Attr.LLM_MODEL_NAME] == "claude-auto"
    assert span.attributes[Attr.LLM_TOKEN_PROMPT] == 7
    assert span.attributes[Attr.LLM_TOKEN_COMPLETION] == 11
    assert "anthropic auto" in span.attributes[Attr.OUTPUT_VALUE]


def test_instrument_is_idempotent(monkeypatch):
    module = _fake_openai_module()
    monkeypatch.setitem(sys.modules, "openai", module)

    first = beater.instrument(providers="openai")
    second = beater.instrument(providers=["openai"])

    assert first[0].status == "patched openai.OpenAI"
    assert second[0].instrumented is True
    assert second[0].status == "already instrumented"


def test_instrument_rejects_unknown_provider():
    with pytest.raises(ValueError, match="unsupported provider"):
        beater.instrument(providers=["not-a-provider"])
