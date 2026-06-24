// LlamaIndex.TS + OTLP -> Beater example app (R11.4).
//
// Demonstrates instrumenting a LlamaIndex.TS query engine with stock
// OpenTelemetry and shipping the trace to beaterd over OTLP/HTTP -- the
// standards-first TypeScript adoption path (no Beater SDK required).
//
// This example brackets the LlamaIndex call in OTel spans manually so it runs
// with or without LlamaIndex's own instrumentation. Swap the stub `query()` for
// a real `index.asQueryEngine().query(...)` once you have an index.
//
// This uses the OTLP/HTTP-protobuf exporter, so it targets beaterd's HTTP API
// (:8080) at the tenant-scoped OTLP path, NOT the OTLP/gRPC port (:4317). The
// scope is carried in the URL path, so no x-beater-* headers are needed.
//
// Run a local beaterd (`docker compose up`) and then:
//
//   npm install llamaindex @opentelemetry/api @opentelemetry/sdk-trace-node \
//     @opentelemetry/exporter-trace-otlp-proto
//   node examples/typescript/frameworks/llamaindex-otlp.mjs

import { trace, SpanStatusCode } from "@opentelemetry/api";
import { NodeTracerProvider } from "@opentelemetry/sdk-trace-node";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-proto";

const apiBase =
  process.env.OTEL_EXPORTER_OTLP_ENDPOINT ?? "http://127.0.0.1:8080";
const tenant = process.env.BEATER_TENANT_ID ?? "demo";
const project = process.env.BEATER_PROJECT_ID ?? "demo";
const environment = process.env.BEATER_ENVIRONMENT_ID ?? "local";
const url = `${apiBase}/v1/otlp/${tenant}/${project}/${environment}/v1/traces`;

const provider = new NodeTracerProvider();
provider.addSpanProcessor(
  new BatchSpanProcessor(new OTLPTraceExporter({ url })),
);
provider.register();
const tracer = trace.getTracer("beater.example.llamaindex");
const release = process.env.BEATER_RELEASE_ID ?? "llamaindex-example";

// Stand-in for a real LlamaIndex query engine call.
async function query(question) {
  // const { Document, VectorStoreIndex } = await import("llamaindex");
  // const index = await VectorStoreIndex.fromDocuments([new Document({ text: "..." })]);
  // return (await index.asQueryEngine().query({ query: question })).toString();
  return `answer to: ${question}`;
}

async function main() {
  const question = "What is our refund window?";
  await tracer.startActiveSpan(
    "rag_query",
    { attributes: { "beater.span.kind": "agent.run", "beater.release_id": release, "input.value": question } },
    async (root) => {
      await tracer.startActiveSpan(
        "retrieve",
        { attributes: { "beater.span.kind": "retrieval.query", "beater.release_id": release, "input.value": question } },
        (retrieval) => {
          retrieval.setStatus({ code: SpanStatusCode.OK });
          retrieval.end();
        },
      );
      const answer = await tracer.startActiveSpan(
        "synthesize",
        {
          attributes: {
            "beater.span.kind": "llm.call",
            "llm.provider": "openai",
            "llm.model_name": "gpt-4o-mini",
            "beater.release_id": release,
            "input.value": question,
          },
        },
        async (llm) => {
          const result = await query(question);
          llm.setAttribute("output.value", result);
          llm.setStatus({ code: SpanStatusCode.OK });
          llm.end();
          return result;
        },
      );
      root.setAttribute("output.value", answer);
      root.end();
      console.log("llamaindex trace flushed:", answer);
    },
  );
  await provider.shutdown();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
