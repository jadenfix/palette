"""Instrument the ``browser-use`` agent framework and emit canonical
``browser.*`` spans over OTLP into Beater.

The public surface is small and defensive:

- :class:`BeaterBrowserUseTracer` owns an OpenTelemetry ``TracerProvider`` and
  knows how to turn a per-step decision/action into an ``llm.call`` +
  ``tool.call`` span pair.
- :func:`make_hooks` returns ``(on_step_start, on_step_end)`` async hooks for
  ``await agent.run(on_step_start=..., on_step_end=...)``.
- :func:`instrument` wires a tracer onto an ``Agent`` and returns the hooks (and
  also registers a ``register_new_step_callback`` when constructing fresh).

Everything reads from the agent via ``getattr`` so the SDK survives
``browser-use`` API drift. ``browser-use`` itself is *not* imported here, which
is what lets the unit tests run with only ``opentelemetry`` installed.
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from typing import Any, Awaitable, Callable, Optional, Tuple

from opentelemetry import trace
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import (
    BatchSpanProcessor,
    SpanProcessor,
)

from .semconv import Attr, Browser, SpanKind, STATUS_ERROR, STATUS_OK

__all__ = [
    "BeaterBrowserUseTracer",
    "StepRecord",
    "instrument",
    "make_hooks",
]

DEFAULT_ENDPOINT = "localhost:4317"
ENV_ENDPOINT = "BEATER_OTLP_ENDPOINT"
DEFAULT_SERVICE_NAME = "browser-use-agent"


# --------------------------------------------------------------------------- #
# Defensive extraction from a browser-use Agent / its history.
# --------------------------------------------------------------------------- #


def _first(seq: Any) -> Any:
    """Return the last element of a sequence-ish thing, or ``None``.

    browser-use history accessors return the *full* list across all steps; the
    entry for the current step is the most recent one.
    """
    if seq is None:
        return None
    try:
        items = list(seq)
    except TypeError:
        return None
    return items[-1] if items else None


def _get(obj: Any, *names: str, default: Any = None) -> Any:
    """First non-None attribute/key among ``names`` on ``obj``."""
    if obj is None:
        return default
    for name in names:
        if isinstance(obj, dict) and name in obj:
            val = obj[name]
        else:
            val = getattr(obj, name, None)
        if val is not None:
            return val
    return default


def _coerce_str(val: Any) -> Optional[str]:
    if val is None:
        return None
    if isinstance(val, str):
        return val
    try:
        return str(val)
    except Exception:  # pragma: no cover - defensive
        return None


@dataclass
class StepRecord:
    """Normalized projection of one browser-agent step.

    This is the duck-typed intermediate the span mapping consumes. It is built
    either from a live ``browser-use`` Agent (:meth:`from_agent`) or directly in
    tests/fixtures.
    """

    seq: int
    # llm.call side
    model: Optional[str] = None
    reasoning: Optional[str] = None
    # tool.call side
    action: Optional[str] = None
    selector: Optional[str] = None
    url: Optional[str] = None
    title: Optional[str] = None
    engine: Optional[str] = None
    status: str = STATUS_OK
    selector_existed: Optional[bool] = None
    matched_element: Optional[bool] = None
    error: Optional[str] = None
    dom_artifact_id: Optional[str] = None
    screenshot_artifact_id: Optional[str] = None
    extra: dict = field(default_factory=dict)

    # ----- builders ----------------------------------------------------- #

    @classmethod
    def from_agent(cls, agent: Any, seq: Optional[int] = None) -> "StepRecord":
        """Build a :class:`StepRecord` from a live ``browser-use`` Agent.

        Uses only ``getattr`` access against the documented history accessors:
        ``model_thoughts()``, ``model_actions()``, ``model_outputs()`` and the
        browser session. Anything missing is simply omitted from the span.
        """
        history = getattr(agent, "history", None)

        reasoning = _extract_reasoning(_first(_call(history, "model_thoughts")))
        action_obj = _first(_call(history, "model_actions"))
        output_obj = _first(_call(history, "model_outputs"))

        if seq is None:
            seq = _infer_seq(history, agent)

        # browser-use carries the per-action success/error on the action RESULT,
        # not on the action itself, so read both to detect failed steps.
        result_obj = _first(_call(history, "action_results"))
        action_name, selector = _extract_action(action_obj)
        status, selector_existed, matched, error = _extract_grounding(
            action_obj, result_obj
        )
        url = _extract_url(agent, action_obj)

        return cls(
            seq=seq,
            model=_extract_model(agent),
            reasoning=reasoning,
            action=action_name,
            selector=selector,
            url=url,
            title=_coerce_str(_get(action_obj, "title")),
            engine=_extract_engine(agent),
            status=status,
            selector_existed=selector_existed,
            matched_element=matched,
            error=error,
        )

    @classmethod
    def from_outcome(cls, step: Any, seq: int) -> "StepRecord":
        """Build from a ``StepTriple``/outcome-shaped mapping.

        Accepts the ``{"action", "outcome": {...}}`` fixture shape (see
        ``fixtures/recorded_run.json``) so the same projection that backs the
        downstream Rust integration test can be replayed through the exporter.
        """
        outcome = _get(step, "outcome", default={}) or {}
        grounding = _get(outcome, "grounding", default={}) or {}
        observation = _get(outcome, "observation", default={}) or {}
        decision = _get(step, "decision", default={}) or {}

        status_raw = _coerce_str(_get(outcome, "status")) or STATUS_OK
        status = STATUS_ERROR if status_raw == STATUS_ERROR else STATUS_OK

        action_val = _get(step, "action")
        action_name, selector = _normalize_action(action_val)
        # selector may also live in grounding
        selector = selector or _coerce_str(_get(grounding, "selector"))

        return cls(
            seq=seq,
            model=_coerce_str(_get(decision, "model")),
            reasoning=_coerce_str(_get(decision, "reasoning")),
            action=action_name,
            selector=selector,
            url=_coerce_str(_get(observation, "url")),
            title=_coerce_str(_get(observation, "title")),
            status=status,
            selector_existed=_get(grounding, "selector_existed"),
            matched_element=_get(grounding, "matched_element"),
            error=_coerce_str(_get(outcome, "error")),
        )


def _call(obj: Any, name: str) -> Any:
    """Call ``obj.name()`` if it's callable; else return the attribute."""
    attr = getattr(obj, name, None)
    if attr is None:
        return None
    if callable(attr):
        try:
            return attr()
        except Exception:  # pragma: no cover - defensive
            return None
    return attr


def _extract_reasoning(thought: Any) -> Optional[str]:
    if thought is None:
        return None
    if isinstance(thought, str):
        return thought
    # browser-use AgentBrain: evaluation_previous_goal / memory / next_goal /
    # thinking. Join whatever textual fields are present.
    parts = []
    for name in ("thinking", "evaluation_previous_goal", "memory", "next_goal"):
        val = _get(thought, name)
        if val:
            parts.append(f"{name}: {val}" if name != "thinking" else str(val))
    if parts:
        return "\n".join(parts)
    return _coerce_str(thought)


def _normalize_action(action_val: Any) -> Tuple[Optional[str], Optional[str]]:
    """Normalize an action value into ``(verb, selector)``.

    Accepts a plain verb string, or a ``{verb: {args}}`` browser-use action dict.
    """
    if action_val is None:
        return None, None
    if isinstance(action_val, str):
        return action_val, None
    if isinstance(action_val, dict):
        # browser-use action dicts look like {"click_element": {"index": 3}} or
        # {"go_to_url": {"url": "..."}}; the first key is the verb.
        if len(action_val) == 1:
            verb, args = next(iter(action_val.items()))
            selector = None
            if isinstance(args, dict):
                selector = _coerce_str(
                    _get(args, "selector", "css", "xpath", "index")
                )
            return _coerce_str(verb), selector
        return _coerce_str(_get(action_val, "action", "name")), _coerce_str(
            _get(action_val, "selector")
        )
    return _coerce_str(action_val), None


def _extract_action(action_obj: Any) -> Tuple[Optional[str], Optional[str]]:
    if action_obj is None:
        return None, None
    if isinstance(action_obj, (str, dict)):
        return _normalize_action(action_obj)
    # ActionModel-like object: try a model_dump, else attribute access.
    dump = _call(action_obj, "model_dump")
    if isinstance(dump, dict):
        # ActionModel dumps to {verb: {...}} with Nones elsewhere.
        non_null = {k: v for k, v in dump.items() if v is not None}
        if non_null:
            return _normalize_action(non_null)
    verb = _coerce_str(_get(action_obj, "action", "name", "type"))
    selector = _coerce_str(_get(action_obj, "selector", "css", "xpath"))
    return verb, selector


def _extract_grounding(
    action_obj: Any,
    result_obj: Any = None,
) -> Tuple[str, Optional[bool], Optional[bool], Optional[str]]:
    """Return ``(status, selector_existed, matched_element, error)``.

    browser-use's ``model_actions()`` does not expose selector resolution, so
    ``selector_existed``/``matched_element`` are typically ``None`` on the live
    path. The failure signal comes from the action RESULT (``error`` string or
    ``success is False``), which we read here so a failed action is not silently
    reported as ``ok``.
    """
    error = _coerce_str(_get(action_obj, "error")) or _coerce_str(
        _get(result_obj, "error")
    )
    existed = _get(action_obj, "selector_existed")
    matched = _get(action_obj, "matched_element")
    success = _get(result_obj, "success")
    if error:
        return STATUS_ERROR, existed, matched, error
    if existed is False or success is False:
        return STATUS_ERROR, existed, matched, error
    return STATUS_OK, existed, matched, error


def _extract_url(agent: Any, action_obj: Any) -> Optional[str]:
    url = _coerce_str(_get(action_obj, "url"))
    if url:
        return url
    session = getattr(agent, "browser_session", None)
    # get_current_page_url may be async; we only use it synchronously when it
    # already returns a plain value (live hooks pass the url in explicitly).
    getter = getattr(session, "get_current_page_url", None)
    if getter is not None and not callable(getter):
        return _coerce_str(getter)
    return None


def _extract_model(agent: Any) -> Optional[str]:
    llm = getattr(agent, "llm", None)
    for name in ("model", "model_name"):
        val = _get(llm, name)
        if val:
            return _coerce_str(val)
    return _coerce_str(_get(agent, "model", "model_name"))


def _extract_engine(agent: Any) -> Optional[str]:
    session = getattr(agent, "browser_session", None)
    engine = _coerce_str(_get(session, "engine", "browser_type"))
    return engine


def _infer_seq(history: Any, agent: Any) -> int:
    n = _get(agent, "n_steps")
    if isinstance(n, int):
        return n
    actions = _call(history, "model_actions")
    try:
        return len(list(actions))
    except TypeError:
        return 0


# --------------------------------------------------------------------------- #
# Tracer.
# --------------------------------------------------------------------------- #


class BeaterBrowserUseTracer:
    """Owns an OTLP-exporting ``TracerProvider`` and the span mapping.

    Parameters
    ----------
    endpoint:
        OTLP/gRPC endpoint. Defaults to ``$BEATER_OTLP_ENDPOINT`` or
        ``localhost:4317``.
    service_name:
        ``service.name`` resource attribute.
    span_processor:
        Inject a custom ``SpanProcessor`` (e.g. ``SimpleSpanProcessor`` wrapping
        an ``InMemorySpanExporter``) — used by the unit tests so no live Beater
        or browser is required. When given, no OTLP exporter is constructed.
    tracer_provider:
        Reuse an existing provider instead of creating one.
    """

    def __init__(
        self,
        endpoint: Optional[str] = None,
        service_name: str = DEFAULT_SERVICE_NAME,
        span_processor: Optional[SpanProcessor] = None,
        tracer_provider: Optional[TracerProvider] = None,
        insecure: bool = True,
    ) -> None:
        self.endpoint = endpoint or os.environ.get(ENV_ENDPOINT, DEFAULT_ENDPOINT)

        if tracer_provider is not None:
            self._provider = tracer_provider
        else:
            resource = Resource.create({"service.name": service_name})
            self._provider = TracerProvider(resource=resource)
            if span_processor is not None:
                self._provider.add_span_processor(span_processor)
            else:
                self._provider.add_span_processor(
                    BatchSpanProcessor(self._build_otlp_exporter(insecure))
                )

        self._tracer = self._provider.get_tracer("beater-browser-use")
        self._seq = 0

    def _build_otlp_exporter(self, insecure: bool):
        # Imported lazily so the unit tests (which inject a span_processor) never
        # need the grpc exporter installed.
        from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import (
            OTLPSpanExporter,
        )

        return OTLPSpanExporter(endpoint=self.endpoint, insecure=insecure)

    @property
    def provider(self) -> TracerProvider:
        return self._provider

    # ----- span emission ------------------------------------------------ #

    def record_step(self, record: StepRecord) -> None:
        """Emit the ``llm.call`` + ``tool.call`` span pair for one step.

        The ``tool.call`` span is nested under the ``llm.call`` span so a step's
        decision and action share a trace and the action is causally a child of
        the decision that produced it.
        """
        self._emit_llm_call(record)

    def _emit_llm_call(self, record: StepRecord) -> None:
        with self._tracer.start_as_current_span("llm.call") as span:
            span.set_attribute(Attr.SPAN_KIND, SpanKind.LLM_CALL)
            span.set_attribute(Browser.STEP_SEQ, record.seq)
            if record.model:
                span.set_attribute(Attr.LLM_MODEL, record.model)
            if record.reasoning:
                span.set_attribute(Browser.REASONING, record.reasoning)
            # The tool.call is emitted within the llm.call context so it is a
            # child span sharing the trace.
            self._emit_tool_call(record)

    def _emit_tool_call(self, record: StepRecord) -> None:
        with self._tracer.start_as_current_span("tool.call") as span:
            span.set_attribute(Attr.SPAN_KIND, SpanKind.TOOL_CALL)
            span.set_attribute(Browser.STEP_SEQ, record.seq)
            span.set_attribute(Browser.STEP_STATUS, record.status)
            if record.action:
                span.set_attribute(Browser.ACTION, record.action)
            if record.selector is not None:
                span.set_attribute(Browser.SELECTOR, record.selector)
            if record.url:
                span.set_attribute(Browser.URL, record.url)
            if record.title:
                span.set_attribute(Browser.TITLE, record.title)
            if record.engine:
                span.set_attribute(Browser.ENGINE, record.engine)
            if record.selector_existed is not None:
                span.set_attribute(
                    Browser.SELECTOR_EXISTED, bool(record.selector_existed)
                )
            if record.matched_element is not None:
                span.set_attribute(
                    Browser.MATCHED_ELEMENT, bool(record.matched_element)
                )
            if record.dom_artifact_id:
                span.set_attribute(Browser.DOM_ARTIFACT, record.dom_artifact_id)
            if record.screenshot_artifact_id:
                span.set_attribute(
                    Browser.SCREENSHOT_ARTIFACT, record.screenshot_artifact_id
                )
            if record.status == STATUS_ERROR:
                from opentelemetry.trace import Status, StatusCode

                span.set_status(Status(StatusCode.ERROR, record.error or "grounding miss"))
                if record.error:
                    span.add_event("browser.error", {"message": record.error})

    # ----- live-agent integration -------------------------------------- #

    def next_seq(self) -> int:
        seq = self._seq
        self._seq += 1
        return seq

    def record_agent_step(self, agent: Any, seq: Optional[int] = None) -> StepRecord:
        """Build a :class:`StepRecord` from a live agent and emit its spans."""
        if seq is None:
            seq = self.next_seq()
        record = StepRecord.from_agent(agent, seq=seq)
        record.seq = seq
        self.record_step(record)
        return record

    def shutdown(self) -> None:
        """Flush and shut down the provider (flushes any pending OTLP export)."""
        try:
            self._provider.force_flush()
        except Exception:  # pragma: no cover - defensive
            pass
        self._provider.shutdown()


# --------------------------------------------------------------------------- #
# Hook factories / public entrypoints.
# --------------------------------------------------------------------------- #

AsyncHook = Callable[[Any], Awaitable[None]]


def make_hooks(
    tracer: Optional[BeaterBrowserUseTracer] = None,
    endpoint: Optional[str] = None,
    **tracer_kwargs: Any,
) -> Tuple[AsyncHook, AsyncHook]:
    """Return ``(on_step_start, on_step_end)`` async hooks for ``agent.run``.

    The step is recorded on ``on_step_end`` because that is when the action's
    outcome (grounding, resulting URL) is available in ``agent.history``.

    Example
    -------
    >>> tracer = BeaterBrowserUseTracer(endpoint="localhost:4317")
    >>> on_start, on_end = make_hooks(tracer)
    >>> await agent.run(on_step_start=on_start, on_step_end=on_end)
    """
    if tracer is None:
        tracer = BeaterBrowserUseTracer(endpoint=endpoint, **tracer_kwargs)

    async def on_step_start(agent: Any) -> None:  # noqa: ARG001 - signature contract
        # Sequencing only; the span is finalized on step end.
        return None

    async def on_step_end(agent: Any) -> None:
        tracer.record_agent_step(agent)

    return on_step_start, on_step_end


@dataclass
class _Instrumentation:
    """Returned by :func:`instrument`: the tracer plus the run hooks."""

    tracer: BeaterBrowserUseTracer
    on_step_start: AsyncHook
    on_step_end: AsyncHook

    def __iter__(self):
        # Allow ``start, end = instrument(agent)`` style unpacking of the hooks.
        return iter((self.on_step_start, self.on_step_end))

    def run_kwargs(self) -> dict:
        """Kwargs to splat into ``agent.run(**inst.run_kwargs())``."""
        return {
            "on_step_start": self.on_step_start,
            "on_step_end": self.on_step_end,
        }


def instrument(
    agent: Any,
    endpoint: Optional[str] = None,
    tracer: Optional[BeaterBrowserUseTracer] = None,
    register_step_callback: bool = False,
    **tracer_kwargs: Any,
) -> _Instrumentation:
    """Instrument a ``browser-use`` ``Agent`` instance.

    Builds (or reuses) a :class:`BeaterBrowserUseTracer` and returns the run
    hooks. The intended usage is ``await agent.run(**inst.run_kwargs())``.

    Pass ``register_step_callback=True`` ONLY if you cannot forward the hooks to
    ``run()`` — it wires ``register_new_step_callback`` as an alternative capture
    path. Do not combine it with the hooks: both fire once per step, so enabling
    both records every step twice.

    Returns an object exposing ``.tracer``, ``.on_step_start``, ``.on_step_end``
    and ``.run_kwargs()``; it also unpacks as ``(on_step_start, on_step_end)``.
    """
    if tracer is None:
        tracer = BeaterBrowserUseTracer(endpoint=endpoint, **tracer_kwargs)
    on_step_start, on_step_end = make_hooks(tracer=tracer)

    # Opt-in fallback for callers who cannot pass the hooks to run().
    # browser-use calls register_new_step_callback(browser_state, agent_output,
    # step_number). Wire it defensively without importing browser-use.
    if register_step_callback:
        try:
            cb = _make_register_callback(tracer, agent)
            if hasattr(agent, "register_new_step_callback"):
                agent.register_new_step_callback = cb
            elif hasattr(agent, "settings"):
                setattr(agent.settings, "register_new_step_callback", cb)
        except Exception:  # pragma: no cover - defensive, never break the agent
            pass

    return _Instrumentation(
        tracer=tracer, on_step_start=on_step_start, on_step_end=on_step_end
    )


def _make_register_callback(tracer: BeaterBrowserUseTracer, agent: Any):
    """Build a ``register_new_step_callback``-compatible callable.

    Signature per browser-use: ``(browser_state_summary, agent_output, step_number)``.
    May be sync or async depending on the browser-use version; we return an
    async callable (browser-use awaits it when it is a coroutine).
    """

    async def _cb(browser_state: Any, agent_output: Any, step_number: int) -> None:
        record = StepRecord.from_agent(agent, seq=step_number)
        record.seq = step_number
        # Enrich from the directly-provided per-step objects (more reliable than
        # re-reading history mid-run).
        url = _coerce_str(_get(browser_state, "url"))
        if url:
            record.url = url
        title = _coerce_str(_get(browser_state, "title"))
        if title:
            record.title = title
        reasoning = _extract_reasoning(_get(agent_output, "current_state", "brain"))
        if reasoning:
            record.reasoning = reasoning
        tracer.record_step(record)

    return _cb
