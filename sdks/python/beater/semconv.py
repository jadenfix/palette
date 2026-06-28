"""Beater semantic conventions.

This is the SINGLE place the Python SDK defines span-kind and attribute keys.
Every wrapper (`observe`, provider wrappers, and framework callbacks) imports
from here so the instrumentation can never drift within the SDK.

These strings mirror the Rust normalizer in ``crates/beater-otlp`` and the
canonical kinds in ``crates/beater-schema``. They are part of the cross-language
ingest contract; keep them in lockstep with the server. (A future codegen step
can emit this file from the Rust source -- see ``sdks/README.md``.)
"""

from __future__ import annotations


class SpanKind:
    """OpenInference ``openinference.span.kind`` values accepted by the normalizer."""

    AGENT_RUN = "agent.run"
    AGENT_TURN = "agent.turn"
    AGENT_PLAN = "agent.plan"
    AGENT_STEP = "agent.step"
    LLM_CALL = "llm.call"
    TOOL_CALL = "tool.call"
    MCP_REQUEST = "mcp.request"
    RETRIEVAL_QUERY = "retrieval.query"
    MEMORY_READ = "memory.read"
    MEMORY_WRITE = "memory.write"
    GUARDRAIL_CHECK = "guardrail.check"


#: Every accepted span kind, for validation.
SPAN_KINDS = frozenset(
    value
    for name, value in vars(SpanKind).items()
    if not name.startswith("_") and isinstance(value, str)
)


class Attr:
    """Canonical span attribute keys."""

    SPAN_KIND = "openinference.span.kind"
    SEQ = "beater.seq"
    RELEASE_ID = "agent.release_id"

    INPUT_VALUE = "input.value"
    OUTPUT_VALUE = "output.value"

    LLM_PROVIDER = "llm.provider"
    LLM_MODEL_NAME = "llm.model_name"
    LLM_TOKEN_PROMPT = "llm.token_count.prompt"
    LLM_TOKEN_COMPLETION = "llm.token_count.completion"
    LLM_TOKEN_REASONING = "llm.token_count.reasoning"
    LLM_TOKEN_CACHE_READ = "llm.token_count.cache_read"
    LLM_COST_MICROS = "llm.cost.amount_micros"
    LLM_COST_CURRENCY = "llm.cost.currency"


#: OTLP ingest headers used to scope traces when exporting over gRPC.
HEADER_TENANT = "x-beater-tenant-id"
HEADER_PROJECT = "x-beater-project-id"
HEADER_ENVIRONMENT = "x-beater-environment-id"
