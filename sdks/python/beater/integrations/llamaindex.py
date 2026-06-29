"""LlamaIndex integration: a callback handler mapping events to Beater spans.

>>> from llama_index.core import Settings
>>> from llama_index.core.callbacks import CallbackManager
>>> from beater.integrations.llamaindex import BeaterLlamaIndexHandler
>>> Settings.callback_manager = CallbackManager([BeaterLlamaIndexHandler()])
"""

from __future__ import annotations

from typing import Any, Dict, Optional

from opentelemetry.trace import Status, StatusCode

from ..observe import _to_value
from ..semconv import Attr, SpanKind
from ..tracing import get_tracer

try:  # pragma: no cover - import guard
    from llama_index.core.callbacks.base_handler import BaseCallbackHandler
    from llama_index.core.callbacks.schema import CBEventType
except Exception:  # pragma: no cover - missing or import-incompatible framework
    BaseCallbackHandler = object  # type: ignore
    CBEventType = None  # type: ignore


_KIND_BY_EVENT = {
    "llm": SpanKind.LLM_CALL,
    "function_call": SpanKind.TOOL_CALL,
    "agent_step": SpanKind.AGENT_STEP,
    "retrieve": SpanKind.RETRIEVAL_QUERY,
    "query": SpanKind.AGENT_RUN,
    "embedding": SpanKind.RETRIEVAL_QUERY,
}


class BeaterLlamaIndexHandler(BaseCallbackHandler):  # type: ignore[misc]
    """Bridges LlamaIndex callback events into Beater spans."""

    def __init__(self) -> None:
        try:
            super().__init__(event_starts_to_ignore=[], event_ends_to_ignore=[])
        except TypeError:  # pragma: no cover - base is object in import-guard mode
            pass
        self._spans: Dict[str, Any] = {}

    def _kind(self, event_type: Any) -> str:
        value = getattr(event_type, "value", str(event_type))
        return _KIND_BY_EVENT.get(value, SpanKind.AGENT_STEP)

    def on_event_start(self, event_type, payload=None, event_id="", parent_id="", **kwargs) -> str:  # type: ignore[no-untyped-def]
        tracer = get_tracer()
        value = getattr(event_type, "value", str(event_type))
        span = tracer.start_span(f"llamaindex.{value}")
        span.set_attribute(Attr.SPAN_KIND, self._kind(event_type))
        if payload:
            span.set_attribute(Attr.INPUT_VALUE, _to_value(payload))
        self._spans[event_id] = span
        return event_id

    def on_event_end(self, event_type, payload=None, event_id="", **kwargs) -> None:  # type: ignore[no-untyped-def]
        span = self._spans.pop(event_id, None)
        if span is None:
            return
        if payload:
            span.set_attribute(Attr.OUTPUT_VALUE, _to_value(payload))
        span.set_status(Status(StatusCode.OK))
        span.end()

    def start_trace(self, trace_id: Optional[str] = None) -> None:  # type: ignore[no-untyped-def]
        pass

    def end_trace(self, trace_id: Optional[str] = None, trace_map=None) -> None:  # type: ignore[no-untyped-def]
        pass
