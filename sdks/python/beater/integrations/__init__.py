"""Framework integrations.

These adapters bridge agent frameworks into Beater spans. They are part of the
SDK's public surface, mirroring the TypeScript SDK which exports the same
adapters from its package root::

    from beater import BeaterCallbackHandler        # LangChain
    from beater import BeaterLlamaIndexHandler       # LlamaIndex

Both adapters import their framework lazily behind an import guard, so importing
this module never requires LangChain or LlamaIndex to be installed.
"""

from __future__ import annotations

from .langchain import BeaterCallbackHandler
from .llamaindex import BeaterLlamaIndexHandler

__all__ = [
    "BeaterCallbackHandler",
    "BeaterLlamaIndexHandler",
]
