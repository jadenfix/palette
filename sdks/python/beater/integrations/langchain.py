"""LangChain integration: a one-line callback handler.

>>> from beater.integrations.langchain import BeaterCallbackHandler
>>> chain.invoke(inputs, config={"callbacks": [BeaterCallbackHandler()]})
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional
from uuid import UUID

from opentelemetry import context as otel_context
from opentelemetry import trace as otel_trace
from opentelemetry.trace import Status, StatusCode

from ..observe import _to_value
from ..semconv import Attr, SpanKind
from ..tracing import get_tracer

try:  # pragma: no cover - import guard
    from langchain_core.callbacks.base import BaseCallbackHandler
except Exception:  # pragma: no cover - missing or import-incompatible framework
    try:
        from langchain.callbacks.base import BaseCallbackHandler  # type: ignore
    except Exception:  # pragma: no cover
        BaseCallbackHandler = object  # type: ignore


class BeaterCallbackHandler(BaseCallbackHandler):  # type: ignore[misc]
    """Maps LangChain runs (chains, LLMs, tools, retrievers) to Beater spans."""

    def __init__(self) -> None:
        self._spans: Dict[UUID, Any] = {}
        self._tokens: Dict[UUID, Any] = {}

    def _start(self, run_id: UUID, parent_run_id: Optional[UUID], name: str, kind: str, input_value: Any) -> None:
        tracer = get_tracer()
        parent_span = self._spans.get(parent_run_id) if parent_run_id else None
        ctx = otel_trace.set_span_in_context(parent_span) if parent_span else None
        span = tracer.start_span(name, context=ctx)
        span.set_attribute(Attr.SPAN_KIND, kind)
        if input_value is not None:
            span.set_attribute(Attr.INPUT_VALUE, _to_value(input_value))
        self._spans[run_id] = span
        self._tokens[run_id] = otel_context.attach(otel_trace.set_span_in_context(span))

    def _end(self, run_id: UUID, output_value: Any = None, error: Optional[BaseException] = None) -> None:
        span = self._spans.pop(run_id, None)
        token = self._tokens.pop(run_id, None)
        if token is not None:
            otel_context.detach(token)
        if span is None:
            return
        if output_value is not None:
            span.set_attribute(Attr.OUTPUT_VALUE, _to_value(output_value))
        if error is not None:
            span.set_status(Status(StatusCode.ERROR, str(error)))
            span.record_exception(error)
        else:
            span.set_status(Status(StatusCode.OK))
        span.end()

    # -- chains --
    def on_chain_start(self, serialized, inputs, *, run_id, parent_run_id=None, **kwargs):  # type: ignore[no-untyped-def]
        name = (serialized or {}).get("name", "chain") if isinstance(serialized, dict) else "chain"
        self._start(run_id, parent_run_id, name, SpanKind.AGENT_STEP, inputs)

    def on_chain_end(self, outputs, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        self._end(run_id, outputs)

    def on_chain_error(self, error, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        self._end(run_id, error=error)

    # -- LLMs --
    def on_llm_start(self, serialized, prompts, *, run_id, parent_run_id=None, **kwargs):  # type: ignore[no-untyped-def]
        self._start(run_id, parent_run_id, "llm", SpanKind.LLM_CALL, prompts)
        span = self._spans.get(run_id)
        invocation = kwargs.get("invocation_params") or {}
        model = invocation.get("model") or invocation.get("model_name")
        if span is not None and model:
            span.set_attribute(Attr.LLM_MODEL_NAME, str(model))

    def on_chat_model_start(self, serialized, messages, *, run_id, parent_run_id=None, **kwargs):  # type: ignore[no-untyped-def]
        self._start(run_id, parent_run_id, "chat_model", SpanKind.LLM_CALL, messages)

    def on_llm_end(self, response, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        span = self._spans.get(run_id)
        if span is not None:
            usage = {}
            llm_output = getattr(response, "llm_output", None) or {}
            if isinstance(llm_output, dict):
                usage = llm_output.get("token_usage") or llm_output.get("usage") or {}
            if usage.get("prompt_tokens") is not None:
                span.set_attribute(Attr.LLM_TOKEN_PROMPT, int(usage["prompt_tokens"]))
            if usage.get("completion_tokens") is not None:
                span.set_attribute(Attr.LLM_TOKEN_COMPLETION, int(usage["completion_tokens"]))
        output = None
        try:
            output = response.generations[0][0].text
        except (AttributeError, IndexError, TypeError):
            output = None
        self._end(run_id, output)

    def on_llm_error(self, error, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        self._end(run_id, error=error)

    # -- tools --
    def on_tool_start(self, serialized, input_str, *, run_id, parent_run_id=None, **kwargs):  # type: ignore[no-untyped-def]
        name = (serialized or {}).get("name", "tool") if isinstance(serialized, dict) else "tool"
        self._start(run_id, parent_run_id, name, SpanKind.TOOL_CALL, input_str)

    def on_tool_end(self, output, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        self._end(run_id, output)

    def on_tool_error(self, error, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        self._end(run_id, error=error)

    # -- retrievers --
    def on_retriever_start(self, serialized, query, *, run_id, parent_run_id=None, **kwargs):  # type: ignore[no-untyped-def]
        self._start(run_id, parent_run_id, "retriever", SpanKind.RETRIEVAL_QUERY, query)

    def on_retriever_end(self, documents, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        summary = [getattr(doc, "page_content", str(doc))[:200] for doc in (documents or [])]
        self._end(run_id, summary)

    def on_retriever_error(self, error, *, run_id, **kwargs):  # type: ignore[no-untyped-def]
        self._end(run_id, error=error)
