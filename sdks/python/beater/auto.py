"""Provider auto-instrumentation bootstrap.

This module keeps auto-instrumentation intentionally small: it patches optional
provider client constructors when those packages are installed, then delegates
span emission to the existing manual wrappers.
"""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from functools import wraps
from typing import Any, Callable, Iterable, Optional, Tuple

from .providers.anthropic import wrap_anthropic
from .providers.openai import wrap_openai


SUPPORTED_PROVIDERS: Tuple[str, ...] = ("openai", "anthropic")


@dataclass(frozen=True)
class InstrumentationResult:
    """Result for one provider requested by ``instrument``."""

    provider: str
    instrumented: bool
    status: str


Wrapper = Callable[[Any], Any]


def _provider_names(providers: Optional[Iterable[str]]) -> Tuple[str, ...]:
    if providers is None:
        return SUPPORTED_PROVIDERS
    if isinstance(providers, str):
        return (providers,)
    return tuple(providers)


def _patch_client_class(module: Any, class_name: str, wrapper: Wrapper, provider: str) -> InstrumentationResult:
    client_class = getattr(module, class_name, None)
    if client_class is None:
        return InstrumentationResult(provider, False, f"{class_name} client class not found")
    if getattr(client_class, "_beater_auto_instrumented", False):
        return InstrumentationResult(provider, True, "already instrumented")

    original_init = client_class.__init__

    @wraps(original_init)
    def __init__(self: Any, *args: Any, **kwargs: Any) -> None:
        original_init(self, *args, **kwargs)
        if getattr(self, "_beater_wrapped", False):
            return
        try:
            wrapper(self)
        except Exception as exc:  # noqa: BLE001 - instrumentation must fail open.
            self._beater_auto_instrument_error = repr(exc)

    client_class.__init__ = __init__
    client_class._beater_original_init = original_init
    client_class._beater_auto_instrumented = True
    return InstrumentationResult(provider, True, f"patched {module.__name__}.{class_name}")


def _instrument_openai() -> InstrumentationResult:
    try:
        module = importlib.import_module("openai")
    except ImportError:
        return InstrumentationResult("openai", False, "openai package not installed")
    return _patch_client_class(module, "OpenAI", wrap_openai, "openai")


def _instrument_anthropic() -> InstrumentationResult:
    try:
        module = importlib.import_module("anthropic")
    except ImportError:
        return InstrumentationResult("anthropic", False, "anthropic package not installed")
    return _patch_client_class(module, "Anthropic", wrap_anthropic, "anthropic")


def instrument(providers: Optional[Iterable[str]] = None) -> Tuple[InstrumentationResult, ...]:
    """Patch installed provider clients so new instances emit Beater spans.

    ``providers`` defaults to all supported providers. Optional dependencies are
    not imported by the SDK at module import time; missing packages are reported
    in the returned results instead of raising.
    """

    results = []
    for provider in _provider_names(providers):
        normalized = provider.strip().lower()
        if normalized == "openai":
            results.append(_instrument_openai())
        elif normalized == "anthropic":
            results.append(_instrument_anthropic())
        else:
            raise ValueError(f"unsupported provider: {provider}")
    return tuple(results)


__all__ = ["InstrumentationResult", "SUPPORTED_PROVIDERS", "instrument"]
