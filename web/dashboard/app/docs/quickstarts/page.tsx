// Per-language SDK quickstarts + MCP + CLI usage. Static content kept in lockstep
// with the SDK READMEs; the API reference at /docs renders live from the spec.

const PY = `pip install beater-sdk

import beater
beater.init(tenant_id="acme", project_id="bot", environment_id="prod")

@beater.observe(kind=beater.SpanKind.AGENT_RUN)
def handle(q): ...

# drop-in provider:
from openai import OpenAI
client = beater.wrap_openai(OpenAI())`;

const TS = `npm i @beater/sdk

import * as beater from "@beater/sdk";
beater.init({ tenantId: "acme", projectId: "bot", environmentId: "prod" });
const handle = beater.observe(rawHandle, { kind: beater.SpanKind.AGENT_RUN });

// drop-in provider:
const client = beater.wrapOpenAI(new OpenAI());`;

const RUST = `# Cargo.toml: beater = "0.1"

beater::init(beater::BeaterConfig::from_env());
beater::observe("handle", beater::span_kind::AGENT_RUN, || { /* ... */ });`;

const CLIENTS = `# Generated control-plane clients (datasets, experiments, gates, evals, ...)
pip install beater-client        # Python
npm i @beater/client             # TypeScript
# Rust: beater-client (path/git)   Go: go get .../beaterclient
# Java: ai.beater:beater-client    C / C++: source + CMake`;

const MCP = `# Every API operation is an MCP tool, served at /mcp on beaterd.
# Point any MCP client at http://127.0.0.1:8080/mcp
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }
# -> one tool per operationId (datasets.create-dataset, traces.list-traces, judge.evaluate-judge, ...)`;

const CLI = `# The CLI reaches any endpoint via the same contract:
beater api traces.list-traces --param tenant_id=acme --api-key bt_...
beater api datasets.create-dataset --param tenant_id=acme --param project_id=bot \\
  --body '{"name":"regressions"}'`;

function Block({ title, code }: { title: string; code: string }) {
  return (
    <section style={{ marginBottom: 28 }}>
      <h2 style={{ fontSize: 18, marginBottom: 8 }}>{title}</h2>
      <pre style={{ background: "#0d1117", color: "#e6edf3", padding: 16, borderRadius: 8, overflowX: "auto" }}>
        <code>{code}</code>
      </pre>
    </section>
  );
}

export default function Quickstarts() {
  return (
    <main style={{ maxWidth: 880, margin: "0 auto", padding: 32, fontFamily: "system-ui, sans-serif" }}>
      <h1 style={{ fontSize: 26, marginBottom: 6 }}>Beater quickstarts</h1>
      <p style={{ color: "#57606a", marginBottom: 24 }}>
        Two layers: ergonomic SDKs (trace your agent) and generated control-plane
        clients (datasets / experiments / gates / evals). Everything is generated
        from one OpenAPI contract — see the <a href="/docs">API reference</a>.
      </p>
      <Block title="Python (ergonomic)" code={PY} />
      <Block title="TypeScript (ergonomic)" code={TS} />
      <Block title="Rust (ergonomic)" code={RUST} />
      <Block title="Control-plane clients (7 languages)" code={CLIENTS} />
      <Block title="MCP server" code={MCP} />
      <Block title="CLI" code={CLI} />
    </main>
  );
}
