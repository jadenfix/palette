# Beater SDKs

Every SDK, the MCP server, the CLI, and the docs derive from **one** artifact:
[`sdks/openapi/beater-api.json`](openapi/beater-api.json), generated from the
Rust API handlers. This is what makes drift structurally impossible.

```
crates/beater-api handlers  (#[utoipa::path] + ToSchema on the real types)
            │  cargo run --example dump_openapi
            ▼
   sdks/openapi/beater-api.json   ← THE single source of truth
   ├── sdks/clients/<lang>/   11 generated control-plane clients (Layer 1)
   ├── /mcp tools             one tool per operationId
   ├── beater api <op>        CLI generic invoker
   └── web/dashboard /docs    rendered API reference + tool catalog
```

## Quickstart (for consumers)

Want to *call* the Beater API from your language? Install the generated
control-plane client, point it at your Beater host, set an API key, and go. Each
client's full, per-endpoint method reference lives in its own README (linked
below).

| Language | Install | Import / namespace | Reference |
| --- | --- | --- | --- |
| Python | `pip install beater_client` | `import beater_client` | [README](clients/python/README.md) |
| TypeScript | `npm install @beater/client` | `import { Configuration, DatasetsApi } from '@beater/client'` | [README](clients/typescript/README.md) |
| Go | `go get github.com/jadenfix/beater/sdks/clients/go` | `import beaterclient ".../sdks/clients/go"` | [README](clients/go/README.md) |
| Java | Maven `ai.beater:beater-client:0.1.0` | `import ai.beater.client.api.DatasetsApi;` | [README](clients/java/README.md) |
| Kotlin | Gradle `implementation("ai.beater:beater-client-kotlin:0.1.0")` | `import ai.beater.client.kotlin.apis.DatasetsApi` | [README](clients/kotlin/README.md) |
| Rust | `cargo add beater-client` | `use beater_client::apis::datasets_api;` | [README](clients/rust/README.md) |
| Ruby | `gem install beater_client` | `require 'beater_client'` (module `BeaterClient`) | [README](clients/ruby/README.md) |
| PHP | `composer require beater/client` | namespace `Beater\Client` | [README](clients/php/README.md) |
| C# | `dotnet add package Beater.Client` | `using Beater.Client.Api;` | [README](clients/csharp/README.md) |
| C | build from source | — | [README](clients/c/README.md) |
| C++ | build from source | — | [README](clients/cpp/README.md) |

> **Registry availability.** Packages publish on the first tagged release (see
> [Publishing](#publishing-zero-config-secret-gated)); until then, build from
> `sdks/clients/<lang>/`. C/C++ have no central registry and always ship as
> source + release tarballs.

**Configure host + auth.** The spec is pre-1.0 and ships no `servers` block, so
every client defaults its base URL to `http://localhost`; set your host and API
key on the client's `Configuration`/`ApiClient` object. Python, as the pattern
for the others:

```python
import beater_client

cfg = beater_client.Configuration(host="https://your-beater-host")  # else defaults to http://localhost
cfg.api_key["ApiKeyAuth"] = "YOUR_API_KEY"          # or cfg.access_token = "..." for bearer

with beater_client.ApiClient(cfg) as api:
    health = beater_client.HealthApi(api).health()
    datasets = beater_client.DatasetsApi(api)
    # ... typed calls per resource (datasets, experiments, gates, evals, traces, ...)
```

The same shape holds in every language: build a config with the host + key,
construct a resource API object (`DatasetsApi`, `TracesApi`, …), call typed
methods. See each client's linked README for the exact constructor and method
names. For live, runnable examples in all 11 languages, see
[`sdks/conformance/<lang>/`](conformance/).

## Two layers

**Layer 1 — generated control-plane clients** (`sdks/clients/<lang>/`): typed
CRUD against `/v1` (datasets, experiments, gates, evals, judge, usage, audit,
api-keys, traces read, …). Generated for **rust, python, typescript, go, java,
c, cpp, ruby, php, csharp, kotlin** by [`openapi-generator`](https://openapi-generator.tech) from the
spec.
Resource **tags** become API classes and `removeOperationIdPrefix` yields clean
methods — e.g. `datasets_create` → `DatasetsApi.create()`.

**Layer 2 — hand-written ergonomic SDKs** (`sdks/<lang>/`): the "nice wrappers" —
`init()`, the `@observe`/`observe()` decorators, drop-in `wrap_openai()` /
`wrap_anthropic()`, and LangChain/LlamaIndex callbacks. Built on OpenTelemetry
(ingest is OTLP). Span kinds and attribute keys live in one `semconv` module per
language that mirrors the server normalizer (`crates/beater-otlp`).

| Language | Layer 1 (generated) | Layer 2 (ergonomic) |
| --- | --- | --- |
| Python | `sdks/clients/python` (`beater_client`) | `sdks/python` (`beater-sdk`) ✅ |
| TypeScript | `sdks/clients/typescript` (`@beater/client`) | `sdks/typescript` (`@beater/sdk`) ✅ |
| Rust | `sdks/clients/rust` (`beater-client`) | (uses Layer 1 + tracing) |
| Go / Java / C / C++ | `sdks/clients/{go,java,c,cpp}` | tracing helpers (planned) |
| Ruby | `sdks/clients/ruby` (`beater_client`) | tracing helpers (planned) |

## Generation status

All 11 Layer-1 clients **generate** from the spec, with correct tag-namespaced
shapes (verified: Go emits `DatasetsAPIService.Create()`, one API class per tag).
Compile-clean status per toolchain:

| Target | Generates | Compiles clean | Note |
| --- | --- | --- | --- |
| go, typescript, python, java, ruby, php, csharp, kotlin | ✅ | ✅ (expected) | permissive `oneOf` handling |
| rust, cpp | ✅ | ⚠️ needs polish | the `EvaluatorKind` mixed `oneOf` (string + object variants) trips the Rust/C++ enum templates; the committed C/C++ `sdks/patches/*.patch` re-apply the fix reproducibly after each regen |

## Regenerating (zero-drift)

```bash
scripts/regen-sdks.sh          # regenerate spec + all 11 clients
scripts/regen-sdks.sh --check  # CI mode: fail if anything is stale
```

Requires the pinned `openapi-generator-cli` v7.11.0. By default it runs in the
`openapitools/openapi-generator-cli` Docker image (no local Java needed); set
`BEATER_OPENAPI_GENERATOR_JAR` to the matching JAR to run it with a local JVM
instead (byte-identical output — useful where Docker Hub is unreachable). CI runs
`--check` so a handler change that isn't regenerated cannot merge, and `oasdiff`
blocks breaking contract changes. On release, all packages bump to one
synchronized version and publish together.

## Publishing (zero-config, secret-gated)

A `v*` tag triggers [`.github/workflows/release.yml`](../.github/workflows/release.yml),
which regenerates every client at the tag version and runs
[`scripts/publish-sdk.sh`](../scripts/publish-sdk.sh) per language. Each target
**no-ops with a clear `SKIP` message when its registry secret is absent**, so the
pipeline is wired now and starts publishing the moment tokens are added as repo
secrets:

| Language(s) | Registry | Secret(s) |
| --- | --- | --- |
| rust | crates.io | `CARGO_REGISTRY_TOKEN` |
| python | PyPI | `PYPI_TOKEN` |
| typescript | npm | `NPM_TOKEN` |
| java, kotlin | Maven Central (OSSRH) | `OSSRH_USERNAME`, `OSSRH_PASSWORD` |
| ruby | RubyGems | `RUBYGEMS_API_KEY` |
| csharp | NuGet | `NUGET_API_KEY` |
| go | pkg.go.dev | none (module proxy serves the git tag) |
| php | Packagist | none required (serves the git tag); optional `PACKAGIST_USERNAME` + `PACKAGIST_API_TOKEN` to force reindex |
| c, cpp | — | no central registry; shipped as source + release tarballs |
