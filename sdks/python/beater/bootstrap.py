"""Environment-driven bootstrap for zero-code Python instrumentation.

Importing this module is intentionally inert. Call ``bootstrap_from_env()`` or
select the ``beater`` OpenTelemetry configurator to initialize tracing.
"""

from __future__ import annotations

import importlib
import os
from dataclasses import dataclass
from typing import Any, Mapping, Optional, Tuple

from .config import BeaterConfig
from . import tracing

try:  # pragma: no cover - depends on OpenTelemetry internals by version.
    from opentelemetry.sdk._configuration import _BaseConfigurator as _ConfiguratorBase
except ImportError:  # pragma: no cover

    class _ConfiguratorBase:  # type: ignore[no-redef]
        def configure(self, **kwargs: Any) -> None:
            self._configure(**kwargs)


_TRUE_VALUES = {"1", "true", "yes", "on", "all"}
_FALSE_VALUES = {"", "0", "false", "no", "off", "none"}


@dataclass(frozen=True)
class AutoInstrumentationStatus:
    """Normalized status for one provider auto-instrumentation request."""

    provider: str
    instrumented: bool
    status: str


@dataclass(frozen=True)
class BootstrapResult:
    """Result returned by ``bootstrap_from_env`` for tests and diagnostics."""

    config: BeaterConfig
    tracing_initialized: bool
    auto_instrumentation: Tuple[AutoInstrumentationStatus, ...] = ()


def _split_csv(value: str) -> Tuple[str, ...]:
    return tuple(part.strip() for part in value.split(",") if part.strip())


def _provider_request(env: Mapping[str, str]) -> Optional[Tuple[str, ...]]:
    requested = env.get("BEATER_AUTO_INSTRUMENT", "")
    normalized = requested.strip().lower()
    providers = env.get("BEATER_AUTO_INSTRUMENT_PROVIDERS")

    if providers:
        return _split_csv(providers)
    if normalized in _FALSE_VALUES:
        return None
    if normalized in _TRUE_VALUES:
        return ()
    return _split_csv(requested)


def _coerce_status(result: Any) -> AutoInstrumentationStatus:
    return AutoInstrumentationStatus(
        provider=str(getattr(result, "provider", "providers")),
        instrumented=bool(getattr(result, "instrumented", False)),
        status=str(getattr(result, "status", result)),
    )


def _apply_auto_instrumentation(env: Mapping[str, str]) -> Tuple[AutoInstrumentationStatus, ...]:
    providers = _provider_request(env)
    if providers is None:
        return ()

    try:
        auto = importlib.import_module("beater.auto")
    except ImportError:
        label = ",".join(providers) if providers else "all"
        return (
            AutoInstrumentationStatus(
                provider=label,
                instrumented=False,
                status="beater.auto is not installed",
            ),
        )

    instrument = getattr(auto, "instrument", None)
    if instrument is None:
        return (
            AutoInstrumentationStatus(
                provider="providers",
                instrumented=False,
                status="beater.auto.instrument is not available",
            ),
        )

    selected = None if not providers else providers
    return tuple(_coerce_status(result) for result in instrument(providers=selected))


def bootstrap_from_env(*, force: bool = False) -> BootstrapResult:
    """Initialize Beater tracing from ``BEATER_*`` environment variables.

    The existing ``beater.tracing.init`` resolver owns connection defaults and
    validation. This wrapper adds an import-safe env-var entry point and optional
    provider auto-instrumentation for deployments that enable it explicitly.
    """

    existing = tracing.get_config()
    tracing_initialized = False
    if existing is None or force:
        config = tracing.init()
        tracing_initialized = True
    else:
        config = existing

    return BootstrapResult(
        config=config,
        tracing_initialized=tracing_initialized,
        auto_instrumentation=_apply_auto_instrumentation(os.environ),
    )


class BeaterConfigurator(_ConfiguratorBase):
    """OpenTelemetry configurator entry point for ``OTEL_PYTHON_CONFIGURATOR``."""

    def _configure(self, **kwargs: Any) -> None:
        bootstrap_from_env()


__all__ = [
    "AutoInstrumentationStatus",
    "BeaterConfigurator",
    "BootstrapResult",
    "bootstrap_from_env",
]
