"""OpenAI-compatible provider instrumentation.

Use this for providers that expose ``client.chat.completions.create`` with
OpenAI-style request/response objects, such as Groq and Mistral.
"""

from __future__ import annotations

from typing import Any, Iterable, Optional

from opentelemetry.trace import Status, StatusCode

from ..observe import _apply_common, _to_value
from ..semconv import Attr, SpanKind
from ..tracing import get_tracer


def _get(value: Any, key: str, default: Any = None) -> Any:
    if isinstance(value, dict):
        return value.get(key, default)
    return getattr(value, key, default)


def _first(value: Any, keys: Iterable[str]) -> Any:
    for key in keys:
        found = _get(value, key)
        if found is not None:
            return found
    return None


def _params(args: tuple[Any, ...], kwargs: dict[str, Any]) -> dict[str, Any]:
    if args and isinstance(args[0], dict):
        return {**args[0], **kwargs}
    return kwargs


def _record_usage(span, response: Any) -> None:
    usage = _first(response, ["usage", "usage_metadata", "usageMetadata"])
    if usage is None:
        return

    prompt = _first(usage, ["prompt_tokens", "promptTokens", "input_tokens", "inputTokens"])
    completion = _first(
        usage,
        ["completion_tokens", "completionTokens", "output_tokens", "outputTokens"],
    )
    details = _first(usage, ["completion_tokens_details", "completionTokensDetails"])
    reasoning = _first(details, ["reasoning_tokens", "reasoningTokens"]) if details is not None else None
    cache_read = _first(usage, ["cache_read_input_tokens", "cacheReadInputTokens"])

    if prompt is not None:
        span.set_attribute(Attr.LLM_TOKEN_PROMPT, int(prompt))
    if completion is not None:
        span.set_attribute(Attr.LLM_TOKEN_COMPLETION, int(completion))
    if reasoning is not None:
        span.set_attribute(Attr.LLM_TOKEN_REASONING, int(reasoning))
    if cache_read is not None:
        span.set_attribute(Attr.LLM_TOKEN_CACHE_READ, int(cache_read))


def _message_content(message: Any) -> str:
    content = _get(message, "content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "".join(str(_get(part, "text", "")) for part in content)
    return "" if content is None else str(content)


def _output_text(response: Any) -> str:
    choices = _get(response, "choices") or []
    try:
        choice = choices[0]
    except (IndexError, TypeError):
        return ""
    message = _get(choice, "message")
    if message is not None:
        return _message_content(message)
    return _get(choice, "text", "") or ""


def wrap_openai_compatible(
    client: Any,
    *,
    provider: str,
    span_name: Optional[str] = None,
) -> Any:
    """Instrument an OpenAI-compatible chat-completions client in place."""

    marker = f"_beater_wrapped_{provider}"
    if getattr(client, marker, False):
        return client

    completions = client.chat.completions
    original = completions.create
    operation_name = span_name or f"{provider}.chat.completions.create"

    def create(*args: Any, **kwargs: Any) -> Any:
        call_params = _params(args, kwargs)
        tracer = get_tracer()
        with tracer.start_as_current_span(operation_name) as span:
            _apply_common(span, SpanKind.LLM_CALL)
            span.set_attribute(Attr.LLM_PROVIDER, provider)
            span.set_attribute(Attr.LLM_MODEL_NAME, str(call_params.get("model", "unknown")))
            messages = call_params.get("messages")
            if messages is not None:
                span.set_attribute(Attr.INPUT_VALUE, _to_value(messages))
            try:
                response = original(*args, **kwargs)
                response_model = _get(response, "model")
                if response_model:
                    span.set_attribute(Attr.LLM_MODEL_NAME, str(response_model))
                _record_usage(span, response)
                span.set_attribute(Attr.OUTPUT_VALUE, _output_text(response))
                span.set_status(Status(StatusCode.OK))
                return response
            except Exception as exc:  # noqa: BLE001
                span.set_status(Status(StatusCode.ERROR, str(exc)))
                span.record_exception(exc)
                raise

    completions.create = create  # type: ignore[method-assign]
    setattr(client, marker, True)
    return client


def wrap_groq(client: Any) -> Any:
    """Instrument a Groq OpenAI-compatible client in place."""

    return wrap_openai_compatible(client, provider="groq")


def wrap_mistral(client: Any) -> Any:
    """Instrument a Mistral OpenAI-compatible client in place."""

    return wrap_openai_compatible(client, provider="mistral")
