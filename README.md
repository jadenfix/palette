# Beater

Beater is a Rust-first open-source agent observability, replay, and eval platform.
The target is an OSS self-hostable debugger/evals harness plus a hosted API
product that can compete with Arize Phoenix, Braintrust, LangSmith, Langfuse, and
Judgment-style systems.

The core loop:

```text
instrument agent -> inspect trace -> promote failure to dataset -> run evals
-> compare candidate -> gate CI -> monitor production
```

## Canonical Docs

- [Architecture](ARCHITECTURE.md): system design, Rust workspace, storage,
  Vercel split, evaluator lanes, replay, compliance, and milestones.
- [Ship Requirements](REQUIREMENTS.md): requirement-by-requirement checklist for
  what must be true before the platform can be called shipped.
- [Gate 2 Outside Runner Card](docs/demos/gate2-outside-runner-card.md): the
  one-screen unaided runner instructions for the current hard-stop proof.

## Outside Runner Quickstart

This is the public clean-clone path Gate 2 is measured on. The one-screen
handoff is
[docs/demos/gate2-outside-runner-card.md](docs/demos/gate2-outside-runner-card.md).
Prerequisites:
Docker Desktop or another local Docker daemon, Docker Compose v2, `git`, `curl`,
`ffprobe`, `shasum` or `sha256sum`, and `python3` 3.9+; local ports `8080`,
`4317`, and `3000` free. Remote `DOCKER_HOST` values and
remote Docker contexts are rejected because the browser proof connects to
`127.0.0.1`.
On macOS, `brew install ffmpeg` provides `ffprobe`; on Ubuntu/Debian, use
`sudo apt-get install ffmpeg`.
The public Compose path uses prebuilt Beater images. Optional third-party
topology services remain digest-pinned for deterministic diagnostics, but they
are not started in the timed default path until the Rust runtime uses them.

Run this from Bash, zsh, Git Bash, or WSL2 before cloning:

```bash
bash -o pipefail -lc 'sha_line="$(git ls-remote --exit-code https://github.com/jadenfix/beater.git refs/heads/main)" && sha="${sha_line%%[[:space:]]*}" && test -n "$sha" && preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && curl -fsSL "https://raw.githubusercontent.com/jadenfix/beater/$sha/scripts/gate2-outside-local-preflight.sh" -o "$preflight" && bash "$preflight" && t="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && test "$(git rev-parse HEAD)" = "$sha" && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'
```

Run it from a directory that does not already contain `beater/`; reruns should
start from a new or empty parent directory. If an aborted previous attempt left
default ports occupied by `beater-stopwatch`, use the cleanup hint printed by
the preflight before rerunning. The command resolves the public `main` commit,
downloads `scripts/gate2-outside-local-preflight.sh` from that immutable SHA
into a temp file, and runs it before the stopwatch starts, so missing local
tooling, remote Docker contexts, and occupied default ports fail before the
timed attempt. After cloning, it verifies the clone HEAD still matches the
resolved SHA before running the outside wrapper. The cloned wrapper repeats
those checks before Compose startup. As soon as the first
`Open this quickstart trace-list URL first:` URL appears, open that filtered
trace-list URL in a normal browser; do not wait for the script to finish. The
terminal checkpoint prints the seconds remaining in the 5-minute clone-to-click
SLO, which already includes clone and image-pull time. Click the quickstart
trace, then click the `llm.call` span. You should see the prompt, completion,
model, token breakdown, cost, latency, and the `Confirm` code. Type that
confirmation code in the terminal and press Enter only after that manual
click-through is complete; the stopwatch records that as the quickstart-click
SLO. Then keep the script running for the post-SLO automated
browser proof, all-kind and recording evidence, open the printed all-kind
dashboard URL, and verify the run -> turn -> step -> tool -> MCP waterfall. Gate 2 is not closed
until someone outside the project reaches the first trace and confirms the
quickstart browser click unaided in 5 minutes or less, completes the post-SLO
all-kind/recording evidence, and fills
[docs/demos/gate2-outside-person-proof.md](docs/demos/gate2-outside-person-proof.md).

## Current State

This repo now contains the first tested Rust vertical slice:

- all-in-one `beaterd` HTTP server
- `beaterctl smoke` local OTLP ingest command that drains trace-write and trace-ingested work
- `beaterctl smoke --http-url ...` remote mode for live `beaterd` OTLP HTTP/gRPC smoke checks with measured query lag
- OTLP/HTTP protobuf trace ingest endpoint with raw protobuf preservation
- OTLP/gRPC TraceService ingest mounted by `beaterd` alongside axum
- Tantivy-backed structured and full-text span search
- Parquet/DataFusion cold trace archive export/query
- canonical schema and agent span taxonomy
- immutable raw envelope creation
- raw-envelope lookup by tenant/project/idempotency key
- filesystem artifact storage with hash verification
- trait-only `beater-store` boundary with `TraceStore`/`ArtifactStore`/`MetadataStore`/`QuotaLimiter`
- SQLite and in-memory metadata store conformance coverage for org/project/environment/RBAC boundaries
- SQLite and in-memory quota limiter conformance coverage for shared fixed-window project quotas
- SQLite `TraceStore` implementation in `beater-store-sql`
- filesystem artifact store implementation in `beater-store-obj`
- typed `StoreError`/provider/adapter errors across storage, judge, dataset, experiment, search, and review trait contracts
- currency-checked `Money` math plus injectable core `Clock`
- bounded in-memory bus plus SQLite durable bus with persisted retry and DLQ behavior
- explicit bus ack/inflight semantics with SQLite recovery of unacked leased work after restart
- tenant/project-scoped DLQ replay that resets attempts and requeues work without deleting unreplayed dead letters
- buffered ingest mode that durably queues canonical trace writes before hot-store persistence
- scoped trace-write drain/status API with typed 429 backpressure responses
- trace-write retry semantics for downstream publish failures after successful trace storage, relying on store idempotency to avoid double writes
- API quota 429 responses include retry/reset headers
- `beaterd` quota flags/env (`--per-project-event-quota`, `--quota-window-seconds`, `--quota-db-path`) with live two-replica shared-counter/reset smoke coverage
- API duplicate native-ingest fixture proves idempotent raw/span writes and downstream dedupe
- native ingest pipeline with payload/attribute governance and trait-backed windowed quotas
- `trace.ingested` downstream drain API and `beaterd` worker for off-hot-path search indexing with retry/DLQ handling
- deterministic evaluator lane and judge-broker budget model
- encrypted-at-rest BYOK provider-secret store for judge/model providers
- judge broker with preflight budget reservation, request-hash cache hits, and SQLite audit ledger
- persisted usage ledger that meters judge charged cost idempotently by judge call
- persisted audit event store for privileged PII unmask attempts
- OpenAI-compatible and Anthropic HTTP judge providers with retry/backoff behavior and structured-score parsing
- trace-to-dataset promotion, immutable dataset versions, and offline deterministic plus judge-backed dataset evals
- baseline-vs-candidate experiment runs with per-case scores, judge-backed scoring, and confidence-bound gates
- in-process agent harness adapters that run baseline/candidate releases over dataset versions
- experiment reports that capture per-case baseline/candidate agent traces and the gate policy used for the decision
- persisted CI gate policies and gate-run audit reports over latest or explicit experiment reports
- persisted human review queues, review tasks, annotations, and annotation-to-dataset promotion
- persisted judge/human calibration reports with agreement, confusion counts, and Cohen's kappa
- online sampling decisions and signed webhook alert delivery with dedupe/suppression
- Wasmtime-backed deterministic WASM evaluator sandbox with fuel limits and no host imports
- experiment gate confidence-bound logic
- persisted replay cassette event store, deterministic/forked replay execution, and failure attribution
- API-key hashing/scoping, persisted SQLite API-key metadata, last-used audit timestamp, and HMAC webhook signing primitives
- optional strict API auth mode on `beaterd` with environment-bound scoped keys
- checksummed SQLite migration runner wired into `beaterd` startup for local OSS stores
- API route for native trace ingest and trace readback
- API routes for trace lists, selected span detail, redaction-aware span I/O inspection, and audited unmask reads
- `/openapi.json` documents the dashboard read surface and generates the dashboard TypeScript client
- multi-stage cargo-chef Dockerfile and `docker-compose.yml` for the current self-host topology
- migration contracts for SQLite local runtime plus Postgres and ClickHouse scale/control-plane paths
- stock OpenTelemetry Python examples: a literal five-line quickstart snippet and an all-kind agent trace fixture
- GHCR prebuilt image workflow plus a Compose override for the five-minute clean-machine stopwatch path
- Gate 2 proof scripts for OpenAPI drift, local clone-to-browser smoke, compose smoke, and browser demo recording
- API route for tenant-scoped span search
- API routes for admin key creation/revocation and strict trace/search/dataset/eval/alert authorization
- API routes for provider-secret create/list/revoke, judge evaluation, and judge ledger readback
- API route for tenant/project usage summaries over metered judge spend
- API route for tenant/project audit event readback and audited trace unmasking
- API route for scoped ingest DLQ replay
- API routes for archiving hot traces and querying cold spans
- API routes for dataset creation, trace promotion, versioning, deterministic eval runs, and judge-backed eval runs
- API route and CLI fixtures for deterministic and judge-backed experiment comparison plus local agent harness runs
- API routes for persisted gate creation and gate runs over stored experiment reports
- API routes for review queue creation, trace review task enqueue, annotation submission, and annotation promotion to dataset cases
- API route for calibration over persisted dataset eval reports and human-labeled dataset cases
- API routes and CLI fixture for online sampling and signed alert webhooks
- `beaterd` defaults to a persistent SQLite bus backend, with an in-memory backend still available
- `beaterd` runs configurable trace-write and trace-ingested background workers for buffered ingest and downstream indexing
- live `beaterd` integration test proving OTLP HTTP and gRPC traces become readable and searchable through public APIs
- live `beaterd` integration test proving accepted buffered trace-write work survives worker kill/restart, can DLQ on TraceStore outage, and replays to a readable/searchable trace
- live `beaterd` integration test proving accepted buffered trace-write work survives an unavailable external TraceStore endpoint, lands in DLQ with no canonical write, and replays after storage returns
- live `beaterd` integration test proving trace-ingested work recovers after consumer kill, restart, DLQ, and replay
- live `beaterd` integration test proving storage write failure accounts events as explicit error, DLQ, or recovered with no silent loss
- `beaterctl bus-fixture` validates durable queue reopen, retry, DLQ, and replay behavior
- `beaterctl ingest-outage-fixture` validates no-silent-drop accounting across explicit error, DLQ, and recovery during a simulated TraceStore outage
- `beaterctl replay-fixture` validates persisted cassette replay without live provider/tool calls
- `beaterctl judge-fixture` validates encrypted BYOK secret persistence, cached judge calls, budget metering, and ledger redaction
- `beaterctl usage-fixture` validates idempotent judge usage records and cached zero-cost audit records
- `beaterctl audit-fixture` validates persisted allowed/denied PII-unmask audit events
- `beaterctl judge-dataset-fixture` validates judge broker evaluation over a persisted dataset version
- `beaterctl judge-experiment-fixture` validates judge-backed candidate-vs-baseline gates
- `beaterctl gate-policy-create`, `gate-run`, and `gate-run-fixture` validate CI blocking on persisted latest experiment regressions
- `beaterctl review-fixture` validates human annotation promotion into an eval-ready dataset case
- `beaterctl calibration-fixture` validates persisted judge/human agreement and kappa reports
- `beaterctl api-key-create` / `api-key-revoke` bootstrap helpers for local and hosted deployments
- `web/dashboard` Next.js trace debugger with generated OpenAPI types, status/kind/time/model/cost/latency/release filters, trace table, icon-coded agent span waterfall, span detail, and audited I/O unmask controls

## Verify

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
cargo run -q -p beaterctl -- judge-fixture --data-dir /tmp/beater-judge
cargo run -q -p beaterctl -- usage-fixture --data-dir /tmp/beater-usage
cargo run -q -p beaterctl -- audit-fixture --data-dir /tmp/beater-audit
cargo run -q -p beaterctl -- ingest-outage-fixture --data-dir /tmp/beater-ingest-outage
cargo run -q -p beaterctl -- judge-dataset-fixture --data-dir /tmp/beater-judge-dataset
cargo run -q -p beaterctl -- judge-experiment-fixture --data-dir /tmp/beater-judge-experiment
cargo run -q -p beaterctl -- gate-run-fixture --data-dir /tmp/beater-gate
! cargo run -q -p beaterctl -- gate-run --data-dir /tmp/beater-gate --tenant-id demo --project-id demo --gate-id main
cargo run -q -p beaterctl -- review-fixture --data-dir /tmp/beater-review
cargo run -q -p beaterctl -- calibration-fixture --data-dir /tmp/beater-calibration
cargo run -q -p beaterd -- --data-dir /tmp/beaterd --judge-provider http-routing --auth-mode required
```

## Clean Clone To Browser

Exact Docker Compose stopwatch proof for the mandate's clean-machine path:

Run this from Bash, zsh, Git Bash, or WSL2 before cloning:

```bash
bash -o pipefail -lc 'sha_line="$(git ls-remote --exit-code https://github.com/jadenfix/beater.git refs/heads/main)" && sha="${sha_line%%[[:space:]]*}" && test -n "$sha" && preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && curl -fsSL "https://raw.githubusercontent.com/jadenfix/beater/$sha/scripts/gate2-outside-local-preflight.sh" -o "$preflight" && bash "$preflight" && t="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && test "$(git rev-parse HEAD)" = "$sha" && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'
```

Run it from a directory that does not already contain `beater/`; reruns should
start from a new or empty parent directory. If an aborted previous attempt left
default ports occupied by `beater-stopwatch`, use the cleanup hint printed by
the preflight before rerunning. The one-liner resolves `refs/heads/main`,
downloads the public `scripts/gate2-outside-local-preflight.sh` from that
immutable SHA before `t="$(date +%s)"`, and verifies the cloned checkout still
matches that SHA before running the wrapper. Missing tools, remote Docker
contexts, and occupied default ports fail before the timed attempt starts;
moving-target clone races fail before the wrapper starts.
As soon as the first `Open this quickstart trace-list URL first:` URL appears,
open that filtered trace-list URL in a normal browser and click the quickstart
trace, then click the `llm.call` span. The manual checkpoint prints the
remaining seconds in the 5-minute clone-to-click SLO. Type the `Confirm` code
shown in the selected span detail and press Enter only after prompt,
completion, model, token breakdown, cost, latency, and the code are visible.
Do not wait for the script to finish; it continues with automated browser
proof, the all-kind waterfall trace, and the recording after the timed manual
quickstart click. Keep the command running until those post-SLO evidence steps
finish.

The outside-run wrapper rejects non-`main` checkouts, non-canonical GitHub
origins, dirty worktrees, warm-loop reuse, local source builds, alternate ports,
mutable pull-policy overrides, prebuilt image overrides, evidence
artifact path overrides, alternate Compose file/profile/project settings, and teardown overrides,
then runs the stopwatch script with proof writing, browser proof, and browser recording
enabled. It rejects a pre-set `BEATER_GATE2_RUN_ID`; the stopwatch creates a
fresh per-run quickstart release ID and filters the five-line trace on that ID
so stale traces cannot satisfy the proof. Test-only registry fixtures are
rejected; outside evidence must validate image digests against public GHCR. The
clone-start environment variable must be captured before
`git clone`, so `Time-to-first-trace` and `Time-to-quickstart-click` include
clone time. For outside-person evidence, `Time-to-quickstart-click` is captured
from the runner's Enter confirmation after manually clicking the trace and
`llm.call` span, not from the automated Playwright proof. It also sets an
`Outside-run wrapper: yes` marker in the stopwatch proof; completed
outside-person proof validation rejects evidence without that marker, rejects
local automated stopwatch footers, requires an outside-run stopwatch source artifact
marker, and cross-checks the stopwatch branch, origin, and worktree-clean
status. The script first removes any previous Compose
project/volumes and fails if that clean start does not complete, then runs
`docker compose up`, sends `examples/python/five_line_otel.py` from the
prebuilt stock OpenTelemetry Python runner container, waits until the trace is
visible in `localhost:3000`, and fails if time-to-first-trace exceeds 300
seconds. It then requires the outside runner to confirm the manual quickstart
click-through before 300 seconds by reading the code from the selected
`llm.call` detail; automated browser proof still runs afterward as secondary
evidence. It leaves the dashboard running by default so a human can click
through the trace.
Before starting Compose it checks local Docker, Docker Compose, curl, `ffprobe`,
and SHA tooling, and it requires `python3` 3.9+ before the timed run so proof
generation and validation cannot fail late on missing local tooling.
It removes any previous Beater stopwatch project, then checks the required host
ports. For outside-person evidence, free the default
`8080`/`4317`/`3000` ports rather than using alternate ports. If preflight
reports another process on one of those ports, stop that app and rerun; do not
set `BEATER_HTTP_PORT`, `BEATER_OTLP_GRPC_PORT`, or `BEATER_DASHBOARD_PORT` for
outside-person evidence. Do not set `COMPOSE_FILE`, `COMPOSE_PROJECT_NAME`, or
`COMPOSE_PROFILES`; the public command controls the Compose topology.
By default it uses `docker-compose.prebuilt.yml` and pulls current GHCR images
published by `.github/workflows/container-images.yml`. The stopwatch script
pins `beaterd`, `dashboard`, `dashboard-e2e`, and `otel-python` to the checked-out commit SHA
tags by default, then records the image references, Beater image service rows,
and structured `proof-image` rows with resolved GHCR digests in the proof. Closure validation
requires those digests to match the public GHCR manifest digest set for the
exact checked-out SHA tag. Set
`BEATER_GATE2_LOCAL_BUILD=1` when you intentionally want to build the server and
dashboard images from source. Set `BEATER_GATE2_REUSE=1` only for local
warm-loop debugging. Set `BEATER_GATE2_BROWSER_PROOF=1` to also run the
prebuilt `dashboard-e2e` Playwright browser proof for both the five-line trace
and the all-kind nested agent waterfall in the same proof run. The five-minute
SLO is enforced for time-to-first-trace and, in outside-run mode, the manual
quickstart click confirmation; all-kind and recording steps run afterward with
per-step timeouts. Set
`BEATER_GATE2_RECORD_DEMO=1` to write `docs/demos/gate2-compose-browser-demo.webm`
and its SHA-pinned notes from the same browser session.

The five-line snippet is intentionally plain OpenTelemetry. To run the exact
manual step after `docker compose up -d --build`, install stock OTEL packages
and execute it against the local OTLP port:

```bash
python3 -m venv /tmp/beater-otel
/tmp/beater-otel/bin/pip install opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 /tmp/beater-otel/bin/python examples/python/five_line_otel.py
sed -n '1,5p' examples/python/five_line_otel.py
```

Fast local proof with built binaries, the all-kind stock Python OpenTelemetry
trace, and the Next dashboard:

```bash
scripts/gate2-proof.sh
```

Containerized self-host proof:

```bash
scripts/smoke-compose.sh
```

The default compose proof starts `beaterd` and the dashboard, then uses one-shot
`otel-python` and `dashboard-e2e` containers for trace generation and browser
proof. Postgres, NATS JetStream, and MinIO are available with the `deps` profile;
ClickHouse is available with the `clickhouse` profile. The current `beaterd`
runtime still stores local OSS state in SQLite under `--data-dir`, so those
external services are kept as self-host topology and schema-contract services,
not started in the timed default path.

The browser proof should finish by opening:

```text
http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local
```

Run OpenAPI drift detection separately with:

```bash
scripts/check-openapi-drift.sh
```

To regenerate the committed Gate 2 browser capture under `docs/demos/`:

```bash
BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-proof.sh
```

To write the automated compose stopwatch artifact under `docs/demos/`:

```bash
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 KEEP_BEATER_COMPOSE=0 scripts/gate2-compose-stopwatch.sh
```

The committed compose recording in
[docs/demos/gate2-compose-browser-demo.webm](docs/demos/gate2-compose-browser-demo.webm)
is a maintainer diagnostic capture from the SHA-pinned prebuilt Compose path.
It is reviewable demo evidence, but it does not close Gate 2. The outside-person
proof still must use default `127.0.0.1:3000` evidence captured from a run where
that port is genuinely Beater, not another local app or an alternate-port
diagnostic.

Gate 2 still requires an unaided outside-person run before it can be called
passed. Before handing the repo to the outside runner, a maintainer should run
the full public handoff verifier after the `container-images` workflow has
published the current commit:

```bash
scripts/check-gate2-public-handoff.py --full-run
```

That mode first preflights the local runtime: canonical public source URL only,
`docker`, Docker Compose v2, `curl`, `ffprobe`, local Docker daemon, SHA tooling,
and free default ports after removing any previous `beater-stopwatch` Compose project.
It then downloads the raw public preflight from the expected immutable commit
and runs it under `bash -o pipefail -lc` before any clone. Remote `DOCKER_HOST` values and
remote Docker contexts fail before clone or Compose cleanup. It runs
`scripts/check-gate2-outside-readiness.py`, uses one fresh clone from
`https://github.com/jadenfix/beater.git` for exact-commit, cloned readiness, and
wrapper dry-run checks, then uses a second fresh clone for the timed runtime
path. The readiness check verifies clean `main`, the expected GitHub remote,
this proof file's structure, and public current-SHA multi-arch
`beaterd`, `dashboard`, `dashboard-e2e`, and `otel-python` GHCR images for the
exact commit. The verifier executes the second clone's
`scripts/gate2-outside-run.sh` wrapper with the clone-start timestamp captured
immediately before that second `git clone`, waits until the wrapper prints the
manual quickstart checkpoint, uses a browser click to read and enter the confirmation
code from the selected `llm.call` detail for diagnostic automation only, and cleans up the `beater-stopwatch` Compose project after the
wrapper exits. This is maintainer runtime evidence that the exact public
outside-run path, current GHCR images, OTLP ingest, dashboard render, browser
proof, and browser recording work; it is not a substitute for the required
outside-person proof below. Its generated report is `Status: diagnostic.` and
default outside-person validation rejects it as closure evidence. `--full-run`
is intentionally supported only for the canonical public GitHub/GHCR handoff,
not fixture or fork URLs.

If Docker is unavailable on the maintainer machine, run
`scripts/check-gate2-public-handoff.py` without `--full-run`; that still
performs the public clone, exact-commit, wrapper dry-run, proof-structure, and
multi-arch GHCR-image checks, but it is not a runtime handoff proof.

Use [docs/demos/gate2-outside-person-proof.md](docs/demos/gate2-outside-person-proof.md)
as the required evidence template for that run. After the outside runner has
completed the stopwatch command, use the prefilled
`scripts/generate-gate2-outside-proof.py --print-command` output printed in the
terminal. It copies the stopwatch-derived dashboard URLs, terminal excerpt, and
compose-log artifact into a ready-to-edit command. Before running the command,
replace every `...` field with the runner's actual values; the generator and
validator reject unresolved evidence. Save the outside-run terminal transcript
or compose logs as a repo-relative, committed/clean, non-symlink file under
`docs/demos/` (for example `docs/demos/gate2-outside-compose.log`), or use an
immutable GitHub Actions run/job URL such as
`https://github.com/jadenfix/beater/actions/runs/<run_id>`. The outside-run
wrapper writes `docs/demos/gate2-outside-compose.log` automatically and
pre-fills that path with `--compose-logs-saved`.

To reprint the ready-to-edit command:

```bash
scripts/generate-gate2-outside-proof.py --print-command
```

The fully expanded form looks like this:

```bash
quickstart_dashboard="$(sed -n 's/^- Quickstart dashboard: //p' docs/demos/gate2-compose-stopwatch.md)"
all_kind_dashboard="$(sed -n 's/^- All-kind dashboard: //p' docs/demos/gate2-compose-stopwatch.md)"

scripts/generate-gate2-outside-proof.py \
  --runner-name "Jane Outside Runner" \
  --relationship "external evaluator; no Beater project role" \
  --prior-exposure "none" \
  --machine-os "Ubuntu 24.04 x86_64" \
  --browser "Chrome stable" \
  --network-notes "home Wi-Fi; no VPN" \
  --llm-observation "clicked llm.call and saw prompt, completion, model, token breakdown, cost, latency, and confirmation code" \
  --waterfall-observation "opened all-kind trace and saw run -> turn -> step -> tool -> MCP nesting" \
  --terminal-output-excerpt "Gate 2 compose stopwatch passed; Browser recording: passed; Quickstart dashboard: $quickstart_dashboard; All-kind dashboard: $all_kind_dashboard" \
  --compose-logs-saved "docs/demos/gate2-outside-compose.log" \
  --preflight-status "passed" \
  --attest-outside-run
```

Then validate it with:

```bash
git add docs/demos/gate2-outside-person-proof.md \
  docs/demos/gate2-compose-stopwatch.md \
  docs/demos/gate2-compose-browser-demo.webm \
  docs/demos/gate2-compose-browser-demo.md \
  docs/demos/gate2-outside-compose.log
git commit -m "add gate2 outside proof"
scripts/validate-gate2-outside-proof.sh
```

The validator checks the outside-person template, stopwatch proof file,
screen-recording notes, and playable WebM metadata from the same run. It rejects
alternate ports, warm-loop reuse, placeholder dashboard URLs, mismatched trace IDs,
mismatched recording-note quickstart release IDs,
mismatched commit SHA,
mismatched API/dashboard endpoints, non-main or stale commit evidence,
mismatched SHA-pinned image references, mismatched image digests,
image digests not bound to the exact public GHCR SHA-tag manifest,
wrong or missing stock quickstart snippet markers,
proof dates that do not match the timed clone start,
compose image excerpts missing runner images or structured `proof-image` rows,
non-repo-relative `docs/demos/` artifacts, and non-prebuilt GHCR image digests.
It rejects ambiguous compose-log notes, missing saved log files, non-GitHub
Actions log URLs, symlinked log artifacts, and dirty or uncommitted saved log
artifacts at closure. It rejects recording notes from a different dashboard
session. It rejects
uncommitted non-evidence worktree changes at closure. It rejects any screen
recording hash that does not match the committed file. It requires
`Quickstart click source: manual-outside-runner` and
`Manual quickstart confirmation: yes` in both the completed proof and the
stopwatch artifact, requires
`Manual confirmation source: browser-selected-llm-detail`, recomputes
`Manual confirmation code` from the per-run salt plus quickstart trace and span
IDs, and the stopwatch artifact must identify itself as an outside-run stopwatch
source artifact rather than an automated local proof. The
recording artifact must be a playable WebM capture of
at least 64 KiB and at least 8 seconds with
EBML/WebM, Segment, Info, Tracks, Cluster, and video-track structure, and
artifact paths must not traverse symlinks. The notes must declare
`Recording mode: compose`, the matching quickstart release ID, and describe the full recorded flow:
quickstart trace, `llm.call`, prompt, completion, model, token breakdown, cost,
latency, confirmation code, and run -> turn -> step -> tool -> MCP waterfall. The
completed proof must additionally include
the runner's own `llm.call` detail and waterfall observations, not only the
automated browser recording notes.
The `gate2-proof-contract` GitHub workflow runs the validator template check
and the executable proof-artifact fixture tests on pull requests and `main`.

Warm-loop debugging can skip the pre-run cleanup, but this is not acceptable
evidence for Gate 2:

```bash
BEATER_GATE2_REUSE=1 scripts/gate2-compose-stopwatch.sh
```

Local source builds are measured but are not the SLO path:

```bash
BEATER_GATE2_LOCAL_BUILD=1 scripts/gate2-compose-stopwatch.sh
```

With `beaterd` running in local auth mode, remote smoke can target the live
server and reports `trace_query_lag_ms`:

```bash
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080 --otlp-grpc-url http://127.0.0.1:4317
```

`beaterd` reads `BEATER_PROVIDER_SECRET_KEY` as a base64 32-byte provider-secret
encryption key when set. Without it, local OSS/dev mode creates
`provider-secrets.key` under the data directory and uses that key to encrypt
`provider-secrets.sqlite`.
