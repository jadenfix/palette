# Beater

<p align="center">
  <img src="docs/assets/beater-logo.svg" width="104" alt="Beater logo">
</p>

<p align="center">
  <strong>Rust-first OSS agent observability, replay, eval, and CI-gate platform.</strong>
</p>

<p align="center">
  <a href="https://github.com/jadenfix/beater/actions/workflows/backend.yml"><img alt="backend" src="https://github.com/jadenfix/beater/actions/workflows/backend.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/jadenfix/beater/actions/workflows/sdk-contract.yml"><img alt="sdk-contract" src="https://github.com/jadenfix/beater/actions/workflows/sdk-contract.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/jadenfix/beater/actions/workflows/frontend.yml"><img alt="frontend" src="https://github.com/jadenfix/beater/actions/workflows/frontend.yml/badge.svg?branch=main"></a>
  <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache--2.0-3fb5ff"></a>
</p>

## What it does

Beater records what your AI agent did, turns its failures into datasets, and then
**only lets a change ship when it measurably improves behavior *and* generalizes** —
not just because it looked good on the cases you already had. The whole loop is one
local Rust binary (`beaterd`) plus a Next.js dashboard:

```text
instrument agent → inspect trace/span tree → promote failure to dataset
→ run evals → compare candidate → gate CI → monitor production
```

Ingest is plain OpenTelemetry, so a stock OTEL exporter works with no Beater-specific
code. The HTTP API, 7 SDK clients, MCP tools, and CLI are all generated from one
OpenAPI contract, so they never drift.

## Quickstart

```bash
git clone https://github.com/jadenfix/beater.git && cd beater
docker compose up
```

| Service | URL |
| --- | --- |
| Dashboard | `http://127.0.0.1:3000` |
| API | `http://127.0.0.1:8080` |
| OTLP gRPC | `http://127.0.0.1:4317` |

Prove an end-to-end OTLP round trip without the full test suite:

```bash
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
```

`beaterd` defaults to `--auth-mode required`; add `--auth-mode local` for anonymous
local use.

## Connect your agent

The SDK, CLI, and MCP tools all talk to the same running `beaterd`
(`http://127.0.0.1:8080`). Every `init()` argument falls back to a `BEATER_*` env var.

**SDK — instrument (Python / TypeScript):**

```python
import beater  # pip install beater-sdk
beater.init(tenant_id="acme", project_id="support-bot", environment_id="prod")

@beater.observe(kind=beater.SpanKind.AGENT_RUN)
def handle(query): ...
```

```ts
import * as beater from "@beater/sdk";  // npm install @beater/sdk
beater.init({ tenantId: "acme", projectId: "support-bot", environmentId: "prod" });
beater.instrument({ providers: ["openai", "anthropic"] });  // auto-wraps clients
```

**CLI — drive any `/v1` operation from a terminal:**

```bash
export BEATER_BASE_URL=http://127.0.0.1:8080
cargo run -q -p beaterctl -- api listTraces --param tenant_id=demo --param project_id=demo
```

**MCP — use Beater as agent tools.** Every API operation is exposed as one MCP tool at
`POST http://127.0.0.1:8080/mcp` (or `beaterd mcp --stdio` for a local editor agent).

## The RSI loop, and the math that gates it

RSI (recursive self-improvement) here means one honest thing: a proposed fix is
accepted only if it beats the **held-out test split** *and* clears an
**anti-overfitting guardrail**. The optimizer never gets to accept its own work — the
gate decides. Run a full round (deterministic, no network, no key):

```bash
cargo run -q -p beaterctl -- rsi-round-fixture --record-trace --data-dir /tmp/beater-rsi

# or live, with a real model proposing (bring your own key):
ANTHROPIC_API_KEY=sk-ant-... cargo run -q -p beaterctl -- rsi-round \
  --model claude-haiku-4-5-20251001 --record-trace --data-dir /tmp/beater-rsi
```

**1. Paired lift.** For $n$ cases with candidate scores $c_i$ and baseline scores
$b_i$, the per-case lift is $c_i - b_i$ and the mean lift is:

$$\Delta = \frac{1}{n}\sum_{i=1}^{n}(c_i - b_i)$$

**2. Is the lift real?** A two-sided paired $t$-test ($df = n-1$) turns $\Delta$ into a
$p$-value, where $s_\Delta$ is the sample standard deviation of the per-case lifts:

$$t = \frac{\Delta}{\,s_\Delta / \sqrt{n}\,}$$

The held-out gate passes only when this is significant on the **Test** split the
optimizer never saw.

**3. Does it generalize?** Even a Test-passing candidate can be quietly overfitting the
optimization (Train+Val) split. Beater bootstraps a confidence interval for the
**generalization gap** between the optimize-split lift and the held-out lift:

$$\text{gap} = \Delta_{\text{optimize}} - \Delta_{\text{holdout}}$$

A candidate is flagged as **overfit** when the gap's CI lower bound clears a tolerance
$\tau$ — i.e. its optimization-set advantage is *significantly* not reproduced on
held-out data:

$$\text{overfit} \iff \text{gap}_{\text{CI low}} > \tau$$

**4. Accept rule.** Both checks must hold, so neither a lucky Test pass nor a
memorized optimization split alone can ship a change:

$$\text{accept} \iff (\text{held-out gate: Pass}) \;\wedge\; \lnot\,\text{overfit}$$

The command prints each candidate's delta, $p$-value, overfit flag, and final
accept/reject. `--record-trace` writes the round back into Beater as a trace, so the
improvement loop is itself observable. The stats primitives (Wilson intervals, paired
$t$ / exact McNemar, bootstrap CIs, sequential e-values, CUPED) live in
[`crates/beater-stats`](crates/beater-stats/src/lib.rs); the gate wiring is in
[`crates/beater-experiments/src/rsi.rs`](crates/beater-experiments/src/rsi.rs).

The same gate runs as a GitHub Action: `uses: jadenfix/beater@main` posts a
pass / fail / **inconclusive** verdict — with effect size, CI, $p$-value, and
"how many more cases would make this conclusive" when underpowered — as a PR
comment, with no server and no API keys. See
[`docs/eval-gate-action.md`](docs/eval-gate-action.md).

## Repository map

| Path | Purpose |
| --- | --- |
| `bins/beaterd` | Main local runtime (API, OTLP ingest, jobs, SQLite state). |
| `bins/beaterctl` | CLI, smoke commands, and fixtures. |
| `crates/*` | Rust libraries: schema, ingest, storage, bus, API, MCP, evals, replay, auth, datasets, gates, stats, review, audit. |
| `web/dashboard` | Next.js dashboard generated against the read-API snapshot. |
| `sdks/openapi`, `sdks/clients/*` | Generated OpenAPI contract and clients — do not hand-edit. |
| `migrations/{sqlite,postgres,clickhouse}` | Durable schema contracts. |
| `scripts/*` | Contract drift checks, SDK regen, smoke gates. |

## Development

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
```

Any change to a `/v1` handler must regenerate the contract and prove zero drift in the
same PR:

```bash
cargo xtask regen-spec && scripts/regen-sdks.sh && cargo xtask regen-semconv
scripts/check-contract-sync.sh
```

See [ARCHITECTURE.md](ARCHITECTURE.md), [CONTRIBUTING.md](CONTRIBUTING.md), and
[CLAUDE.md](CLAUDE.md) for the full verification matrix and contract rules. The
standalone ecosystem boundary with Tempo, beater.js, and beaterOS is tracked in
[`docs/ecosystem-integration-contract.md`](docs/ecosystem-integration-contract.md).
The clean-clone → browser stopwatch proof lives in the
[Gate 2 Outside Runner Card](docs/demos/gate2-outside-runner-card.md). Report
vulnerabilities privately via [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
