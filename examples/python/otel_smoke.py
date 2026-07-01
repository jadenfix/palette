import os

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.trace import Status, StatusCode


def main() -> None:
    endpoint = os.getenv("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:4317")
    headers = (
        ("x-beater-tenant-id", os.getenv("BEATER_TENANT_ID", "demo")),
        ("x-beater-project-id", os.getenv("BEATER_PROJECT_ID", "demo")),
        ("x-beater-environment-id", os.getenv("BEATER_ENVIRONMENT_ID", "local")),
    )

    provider = TracerProvider(
        resource=Resource.create({"service.name": "beater-otel-python-smoke"})
    )
    provider.add_span_processor(
        BatchSpanProcessor(OTLPSpanExporter(endpoint=endpoint, insecure=True, headers=headers))
    )
    trace.set_tracer_provider(provider)
    tracer = trace.get_tracer("beater.examples.python")

    with tracer.start_as_current_span(
        "refund-agent-run",
        attributes={
            "openinference.span.kind": "agent.run",
            "beater.seq": 1,
            "agent.release_id": "compose-demo",
        },
    ) as run:
        run.set_status(Status(StatusCode.OK))
        with tracer.start_as_current_span(
            "customer-refund-turn",
            attributes={"openinference.span.kind": "agent.turn", "beater.seq": 2},
        ) as turn:
            turn.set_status(Status(StatusCode.OK))
            with tracer.start_as_current_span(
                "plan-refund-resolution",
                attributes={
                    "openinference.span.kind": "agent.plan",
                    "beater.seq": 3,
                    "output.value": '["retrieve policy","read memory","call model","lookup order"]',
                },
            ) as plan:
                plan.set_status(Status(StatusCode.OK))
            with tracer.start_as_current_span(
                "execute-refund-step",
                attributes={"openinference.span.kind": "agent.step", "beater.seq": 4},
            ) as step:
                step.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "retrieve-refund-policy",
                    attributes={
                        "openinference.span.kind": "retrieval.query",
                        "beater.seq": 5,
                        "input.value": "refund policy late delivery exception",
                        "output.value": "Refunds after 30 days require supervisor approval.",
                    },
                ) as retrieval:
                    retrieval.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "read-customer-memory",
                    attributes={
                        "openinference.span.kind": "memory.read",
                        "beater.seq": 6,
                        "input.value": "customer_id=cus_123",
                        "output.value": "Customer has two prior late-delivery refunds.",
                    },
                ) as memory_read:
                    memory_read.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "guardrail-refund-policy",
                    attributes={
                        "openinference.span.kind": "guardrail.check",
                        "beater.seq": 7,
                        "input.value": "refund request outside policy window",
                        "output.value": "manual approval required",
                    },
                ) as guardrail:
                    guardrail.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "call-policy-model",
                    attributes={
                        "openinference.span.kind": "llm.call",
                        "beater.seq": 8,
                        "llm.provider": "openai",
                        "llm.model_name": "gpt-demo",
                        "llm.token_count.prompt": 18,
                        "llm.token_count.completion": 11,
                        "llm.token_count.reasoning": 4,
                        "llm.cost.amount_micros": 2500,
                        "llm.cost.currency": "USD",
                        "input.value": "Can this order be refunded after 31 days?",
                        "output.value": "Escalate because the order is outside the standard window.",
                    },
                ) as llm:
                    llm.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "lookup-order-tool",
                    attributes={
                        "openinference.span.kind": "tool.call",
                        "beater.seq": 9,
                        "input.value": '{"order_id":"ord_123"}',
                        "output.value": '{"status":"delivered","age_days":31}',
                    },
                ) as tool:
                    tool.set_status(Status(StatusCode.OK))
                    with tracer.start_as_current_span(
                        "mcp-order-service",
                        attributes={
                            "openinference.span.kind": "mcp.request",
                            "beater.seq": 10,
                            "input.value": '{"server":"orders","method":"get_order"}',
                            "output.value": '{"order_id":"ord_123","status":"delivered"}',
                        },
                    ) as mcp:
                        mcp.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "write-customer-memory",
                    attributes={
                        "openinference.span.kind": "memory.write",
                        "beater.seq": 11,
                        "input.value": "customer_id=cus_123",
                        "output.value": "Stored escalation note for refund review.",
                    },
                ) as memory_write:
                    memory_write.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "evaluate-refund-answer",
                    attributes={
                        "openinference.span.kind": "evaluator.run",
                        "beater.seq": 12,
                        "output.value": '{"score":1.0,"label":"policy_compliant"}',
                    },
                ) as evaluator:
                    evaluator.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "human-refund-review",
                    attributes={
                        "openinference.span.kind": "human.review",
                        "beater.seq": 13,
                        "output.value": "Approved escalation path; no immediate refund issued.",
                    },
                ) as human:
                    human.set_status(Status(StatusCode.OK))
                with tracer.start_as_current_span(
                    "replay-refund-run",
                    attributes={
                        "openinference.span.kind": "replay.run",
                        "beater.seq": 14,
                        "output.value": "Replay matched the original policy decision.",
                    },
                ) as replay:
                    replay.set_status(Status(StatusCode.OK))

    provider.force_flush()
    provider.shutdown()
    print("sent stock OpenTelemetry Python trace to", endpoint)


if __name__ == "__main__":
    main()
