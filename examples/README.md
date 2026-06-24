# Beater examples

Runnable example apps showing how to get traces into Beater. The fastest path is
**zero-SDK OTLP**: point any OpenTelemetry exporter at `beaterd:4317` and you are
done. The ergonomic SDKs (`sdks/rust`, `sdks/python`, `sdks/typescript`) add
convenience on top of the same standard.

Start a local server first: `docker compose up beaterd dashboard` (or
`cargo run -p beaterd`). All examples default to the `demo/demo/local`
tenant/project/environment. The OTLP/gRPC examples (the Python apps) target
`http://127.0.0.1:4317`; the OTLP/HTTP-protobuf examples (the TypeScript apps)
target the HTTP API at `http://127.0.0.1:8080` on the tenant-scoped
`/v1/otlp/<tenant>/<project>/<environment>/v1/traces` path. beaterd does not
expose a separate 4318 collector port — HTTP/proto OTLP rides the 8080 API.

## Zero-SDK instrumentation fixtures (R11.2)

Stock OpenTelemetry with different LLM semantic conventions; Beater ingests them
all natively:

| File | Convention |
| --- | --- |
| `python/instrumentations/openinference_app.py` | OpenInference (`openinference.span.kind`) |
| `python/instrumentations/openllmetry_app.py` | OpenLLMetry / Traceloop (`gen_ai.*`) |
| `python/instrumentations/otel_genai_app.py` | Official OTel GenAI conventions |
| `python/five_line_otel.py` | The five-line quickstart |

## Python / TypeScript framework apps (R11.4)

OTLP from common web frameworks, no Beater SDK required:

| File | Framework |
| --- | --- |
| `python/frameworks/fastapi_app.py` | FastAPI |
| `python/frameworks/flask_app.py` | Flask |
| `typescript/frameworks/express-otlp.mjs` | Express |
| `typescript/frameworks/llamaindex-otlp.mjs` | LlamaIndex.TS |

## Rust SDK examples (R11.3)

First-class Rust adoption via the `beater` SDK (`sdks/rust`):

| File | Integration |
| --- | --- |
| `rust/tracing_app.rs` | `tracing` / ergonomic `observe` |
| `rust/axum_app.rs` | axum HTTP service |
| `rust/tonic_app.rs` | tonic gRPC service |
| `rust/reqwest_app.rs` | reqwest outbound call (LLM client) |
| `rust/mcp_app.rs` | MCP tool calls (`mcp.request` spans) |

The Rust SDK also ships a runnable `cargo run --example quickstart` under
`sdks/rust/examples/`. The files in `examples/rust/` are app templates: copy one
into a binary crate that depends on `beater = { path = ... }` and run it against
a local `beaterd`. They are registered as `[[example]]` targets of the `beater`
SDK crate, so `cargo build --examples` (in `sdks/rust/`) compiles them in CI and
they cannot silently drift from the SDK API.
