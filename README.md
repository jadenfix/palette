# Beater

<p align="center">
  <img src="docs/assets/beater-logo.svg" width="104" alt="Beater logo">
</p>

<p align="center">
  <strong>Rust-first OSS agent observability, replay, eval, and CI gate platform.</strong>
</p>

<p align="center">
  <a href="https://github.com/jadenfix/beater/actions/workflows/backend.yml"><img alt="backend" src="https://github.com/jadenfix/beater/actions/workflows/backend.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/jadenfix/beater/actions/workflows/sdk-contract.yml"><img alt="sdk-contract" src="https://github.com/jadenfix/beater/actions/workflows/sdk-contract.yml/badge.svg?branch=main"></a>
  <a href="https://github.com/jadenfix/beater/actions/workflows/frontend.yml"><img alt="frontend" src="https://github.com/jadenfix/beater/actions/workflows/frontend.yml/badge.svg?branch=main"></a>
  <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache--2.0-3fb5ff"></a>
</p>

Beater is a local-first platform for understanding, replaying, and improving AI
agent behavior. It preserves agent traces, lets you inspect the trace/span tree,
turns failures into versioned datasets, runs evals over candidate releases, and
uses experiment reports as CI gates.

```text
instrument agent -> inspect trace/span tree -> promote failure to dataset -> run evals
-> compare candidate -> gate CI -> monitor production
```

The OSS runtime is centered on one all-in-one Rust binary, `beaterd`, plus a
Next.js dashboard and generated API/SDK surfaces. Hosted scale, SSO, billing,
managed cells, and enterprise governance are architecture tracks, not required
for the local OSS loop.

## Current Scope

| Area | In this repo today |
| --- | --- |
| Runtime | `beaterd` serves API, OTLP ingest, local jobs, SQLite-backed state, and durable bus workers. |
| Ingest | OTLP HTTP/gRPC, native trace ingest, source import primitives, raw envelope preservation, and canonical projections. |
| Dashboard | `web/dashboard` reads generated OpenAPI types and provides trace lists, span waterfalls, span detail, filters, and audited I/O unmask controls. |
| Datasets and evals | Trace-to-dataset promotion, immutable dataset versions, deterministic and judge-backed eval primitives, experiments, reports, and CI gate runs. |
| Contract | `/v1` handlers in `crates/beater-api` generate `sdks/openapi/beater-api.json`, dashboard snapshots, generated SDK clients, MCP/CLI surfaces, and semantic conventions. |
| Storage path | SQLite and in-memory conformance coverage for the OSS path; Postgres/ClickHouse migrations are scale-path contracts. |

## Quickstart

Run the current self-host topology:

```bash
git clone https://github.com/jadenfix/beater.git
cd beater
docker compose up
```

Default local endpoints:

| Service | URL |
| --- | --- |
| Dashboard | `http://127.0.0.1:3000` |
| API | `http://127.0.0.1:8080` |
| OTLP gRPC | `http://127.0.0.1:4317` |
| OTLP HTTP/protobuf | `http://127.0.0.1:8080/v1/otlp/<tenant>/<project>/<environment>/v1/traces` |

Smoke an OTLP round trip without running the full test suite:

```bash
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
```

Or run the all-in-one server and smoke it remotely:

```bash
cargo run -q -p beaterd -- --data-dir /tmp/beaterd
cargo run -q -p beaterctl -- smoke --http-url http://127.0.0.1:8080
cargo run -q -p beaterctl -- ingest test --http-url http://127.0.0.1:8080
```

Point a stock OpenTelemetry exporter at the local OTLP port:

```bash
python3 -m venv /tmp/beater-otel
/tmp/beater-otel/bin/pip install opentelemetry-sdk opentelemetry-exporter-otlp-proto-grpc
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 \
  /tmp/beater-otel/bin/python examples/python/five_line_otel.py
```

## Repository Map

| Path | Purpose |
| --- | --- |
| `bins/beaterd` | Main local runtime. |
| `bins/beaterctl` | CLI, smoke commands, and fixtures. |
| `crates/*` | Rust libraries for schema, ingest, storage, bus, API, MCP, evals, replay, auth, datasets, gates, human review, alerts, audit, browser capture, and xtask. |
| `web/dashboard` | Next.js dashboard generated against the read API snapshot. |
| `sdks/openapi` and `sdks/clients/*` | Generated OpenAPI contract and generated clients. Do not hand-edit. |
| `sdks/{python,typescript,rust}` | Hand-written SDK surfaces and examples around the generated contract. |
| `migrations/{sqlite,postgres,clickhouse}` | Durable schema contracts for local runtime and scale paths. |
| `scripts/*` | Contract drift checks, SDK regeneration, compose/browser smoke gates, and release helpers. |

## Development

Focused checks for common README-adjacent and contract work:

```bash
cargo fmt --all
cargo test -p beater-api
cargo test -p beater-api --test openapi_coverage
cargo test -p beater-store-conformance
cargo run -q -p beaterctl -- smoke --data-dir /tmp/beater-smoke
```

Contract changes must regenerate from source and prove zero drift:

```bash
cargo xtask regen-spec
scripts/regen-sdks.sh
cargo xtask regen-semconv
scripts/check-contract-sync.sh
```

Dashboard checks run from `web/dashboard`:

```bash
npm ci
npm run generate:api
npm test
npm run build
```

Run only the checks that match your change. For the full verification matrix,
see [ARCHITECTURE.md](ARCHITECTURE.md) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Outside Runner Quickstart

This is the public clean-clone path Gate 2 is measured on. The one-screen
handoff, the [Gate 2 Outside Runner Card](docs/demos/gate2-outside-runner-card.md),
lives at
[docs/demos/gate2-outside-runner-card.md](docs/demos/gate2-outside-runner-card.md).
Prerequisites:
Docker Desktop or another local Docker daemon, Docker Compose v2, `git`, `curl`,
`ffprobe`, `shasum` or `sha256sum`, and `python3` 3.9+; local ports `8080`,
`4317`, and `3000` free; and a local graphical browser that can reach
`http://127.0.0.1:3000`. Remote `DOCKER_HOST` values and
remote Docker contexts are rejected because the browser proof connects to
`127.0.0.1`.
On macOS, `brew install ffmpeg` provides `ffprobe`; on Ubuntu/Debian, use
`sudo apt-get install ffmpeg`.
The public Compose path uses prebuilt Beater images. Optional third-party
topology services remain digest-pinned for deterministic diagnostics, but they
are not started in the timed default path until the Rust runtime uses them.

Run this from Bash, zsh, Git Bash, or WSL2 before cloning:

```bash
bash -o pipefail -lc 'sha_line="$(GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git ls-remote --exit-code https://github.com/jadenfix/beater.git refs/heads/main)" && sha="${sha_line%%[[:space:]]*}" && test -n "$sha" && preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && curl -fsSL "https://raw.githubusercontent.com/jadenfix/beater/$sha/scripts/gate2-outside-local-preflight.sh" -o "$preflight" && BEATER_GATE2_EXPECTED_COMMIT="$sha" bash "$preflight" && t="$(date +%s)" && GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git clone https://github.com/jadenfix/beater.git && cd ./beater && test "$(GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git rev-parse HEAD)" = "$sha" && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 scripts/gate2-outside-run.sh'
```

Run it from a directory that does not already contain `beater/`; reruns should
start from a new or empty parent directory. If an aborted previous attempt left
default ports occupied by `beater-stopwatch`, use the cleanup hint printed by
the preflight before rerunning. If preflight reports another app on a default
port, stop or move that app instead of setting alternate Beater ports. The
command resolves the public `main` commit,
downloads `scripts/gate2-outside-local-preflight.sh` from that immutable SHA
into a temp file, and runs it before the stopwatch starts, so missing local
tooling, unpublished SHA-tagged GHCR images, remote Docker contexts, and
occupied default ports fail before the timed attempt. Public Git operations run
with global/system config disabled so local URL rewrites cannot change the
clone target. After cloning, it verifies the clone HEAD still matches the
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
browser proof, all-kind, redaction, and recording evidence. Open the printed
all-kind dashboard URL and verify the run -> turn -> step -> tool -> MCP
waterfall; the automated proof also seeds a sensitive trace and records
redacted defaults, reasoned unmask, unmasked I/O, and Redacted view. Gate 2 is not closed
until someone outside the project reaches the first trace and confirms the
quickstart browser click unaided in 5 minutes or less, completes the post-SLO
all-kind/recording evidence, and fills
[docs/demos/gate2-outside-person-proof.md](docs/demos/gate2-outside-person-proof.md).

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
bash -o pipefail -lc 'sha_line="$(GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git ls-remote --exit-code https://github.com/jadenfix/beater.git refs/heads/main)" && sha="${sha_line%%[[:space:]]*}" && test -n "$sha" && preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && curl -fsSL "https://raw.githubusercontent.com/jadenfix/beater/$sha/scripts/gate2-outside-local-preflight.sh" -o "$preflight" && BEATER_GATE2_EXPECTED_COMMIT="$sha" bash "$preflight" && t="$(date +%s)" && GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git clone https://github.com/jadenfix/beater.git && cd ./beater && test "$(GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git rev-parse HEAD)" = "$sha" && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 scripts/gate2-outside-run.sh'
```

Run it from a directory that does not already contain `beater/`; reruns should
start from a new or empty parent directory. If an aborted previous attempt left
default ports occupied by `beater-stopwatch`, use the cleanup hint printed by
the preflight before rerunning. If preflight reports another app on a default
port, stop or move that app instead of setting alternate Beater ports. The
one-liner resolves `refs/heads/main`,
downloads the public `scripts/gate2-outside-local-preflight.sh` from that
immutable SHA before `t="$(date +%s)"`, and verifies the cloned checkout still
matches that SHA before running the wrapper. Missing tools, remote Docker
contexts, unpublished SHA-tagged GHCR images, and occupied default ports fail
before the timed attempt starts; moving-target clone races and local Git URL
rewrites fail before the wrapper starts.
As soon as the first `Open this quickstart trace-list URL first:` URL appears,
open that filtered trace-list URL in a normal browser and click the quickstart
trace, then click the `llm.call` span. The manual checkpoint prints the
remaining seconds in the 5-minute clone-to-click SLO. Type the `Confirm` code
shown in the selected span detail and press Enter only after prompt,
completion, model, token breakdown, cost, latency, and the code are visible.
Do not wait for the script to finish; it continues with automated browser
proof, the all-kind waterfall trace, redacted-I/O proof, and the recording
after the timed manual quickstart click. Keep the command running until those
post-SLO evidence steps finish.

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
The outside wrapper also tees its terminal output to
`docs/demos/gate2-outside-terminal.log`, so the manual checkpoint prompt,
dashboard URLs, final pass line, and generated proof command are committed as
evidence alongside the Compose service logs.
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
prebuilt `dashboard-e2e` Playwright browser proof for the five-line trace,
redacted-I/O unmask controls, and the all-kind nested agent waterfall in the
same proof run. The five-minute
SLO is enforced for time-to-first-trace and, in outside-run mode, the manual
quickstart click confirmation; all-kind and recording steps run afterward with
per-step timeouts. Set
`BEATER_GATE2_RECORD_DEMO=1` to write `docs/demos/gate2-compose-browser-demo.webm`
and its SHA-pinned notes from the same browser session, including the redaction
trace and unmask reason.

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
completed the stopwatch command, the one-liner returns their parent shell to the
directory that contains `beater/`; run `cd ./beater`, then use the prefilled
`scripts/generate-gate2-outside-proof.py --print-command` output printed in the
terminal. It copies the stopwatch-derived dashboard URLs, terminal excerpt, and
compose-log artifact into a ready-to-edit command. Before running the command,
replace every `...` field with the runner's actual values; the generator and
validator reject unresolved evidence. Save the outside-run terminal transcript
and compose logs as repo-relative, committed/clean, non-symlink files under
`docs/demos/` (for example `docs/demos/gate2-outside-terminal.log` and
`docs/demos/gate2-outside-compose.log`), or use an immutable GitHub Actions
run/job URL for compose logs such as
`https://github.com/jadenfix/beater/actions/runs/<run_id>`. The outside-run
wrapper writes `docs/demos/gate2-outside-compose.log` automatically and
pre-fills that path with `--compose-logs-saved`; it also writes
`docs/demos/gate2-outside-terminal.log` and pre-fills
`--terminal-transcript-saved`. For local compose-log files, the validator checks
the stopwatch-written `# Gate 2 Compose Logs` header, `beater-stopwatch`
project, `prebuilt-image` startup mode, and timestamped compose logs command.

To reprint the ready-to-edit command:

```bash
cd ./beater
scripts/generate-gate2-outside-proof.py --print-command
```

The fully expanded form looks like this:

```bash
quickstart_dashboard="$(sed -n 's/^- Quickstart dashboard: //p' docs/demos/gate2-compose-stopwatch.md)"
all_kind_dashboard="$(sed -n 's/^- All-kind dashboard: //p' docs/demos/gate2-compose-stopwatch.md)"
redaction_dashboard="$(sed -n 's/^- Redaction dashboard: //p' docs/demos/gate2-compose-stopwatch.md)"

scripts/generate-gate2-outside-proof.py \
  --runner-name "Jane Outside Runner" \
  --relationship "external evaluator; no Beater project role" \
  --prior-exposure "none" \
  --machine-os "Ubuntu 24.04 x86_64" \
  --browser "Chrome stable" \
  --network-notes "home Wi-Fi; no VPN" \
  --llm-observation "clicked llm.call and saw prompt, completion, model, token breakdown, cost, latency, and confirmation code" \
  --waterfall-observation "opened all-kind trace and saw run -> turn -> step -> tool -> MCP nesting" \
  --terminal-output-excerpt "Gate 2 compose stopwatch passed; Browser recording: passed; Quickstart dashboard: $quickstart_dashboard; All-kind dashboard: $all_kind_dashboard; Redaction dashboard: $redaction_dashboard" \
  --terminal-transcript-saved "docs/demos/gate2-outside-terminal.log" \
  --compose-logs-saved "docs/demos/gate2-outside-compose.log" \
  --preflight-status "passed" \
  --attest-outside-run
```

Then, from the same `beater/` clone, commit the evidence and validate it with:

```bash
git add docs/demos/gate2-outside-person-proof.md \
  docs/demos/gate2-compose-stopwatch.md \
  docs/demos/gate2-compose-browser-demo.webm \
  docs/demos/gate2-compose-browser-demo.md \
  docs/demos/gate2-outside-terminal.log \
  docs/demos/gate2-outside-compose.log
git commit -m "add gate2 outside proof"
scripts/validate-gate2-outside-proof.sh
```

The validator checks the outside-person template, stopwatch proof file,
screen-recording notes, outside-run terminal transcript, and playable WebM
metadata from the same run. It rejects
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
artifacts at closure. Local compose-log files must contain the stopwatch-written
header, canonical project, prebuilt startup mode, and timestamped compose logs
command. It requires the outside-run terminal transcript to be a
committed `docs/demos/` file containing the manual checkpoint prompt, dashboard
URLs, final pass line, and prefilled proof command. It rejects recording notes
from a different dashboard session. It rejects
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
latency, confirmation code, run -> turn -> step -> tool -> MCP waterfall,
redacted prompt/completion, reasoned unmask, unmasked I/O, and Redacted view. The
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

## Project Status

Beater is early and actively evolving, but the local vertical slice is real:
`beaterd`, OTLP/native ingest, trace read APIs, the dashboard, contract
generation, dataset/eval primitives, experiment comparison, and gate policy
fixtures are present in this repository.

The hard rule is contract discipline: generated OpenAPI snapshots, generated
SDK clients, dashboard generated API types, and semantic-convention JSON are
outputs. Change the Rust/API/schema source, regenerate, and commit the generated
result in the same PR.

## Contributor and Agent Notes

- [ARCHITECTURE.md](ARCHITECTURE.md) is the build-ready plan and the source for
  requirement intent.
- [docs/architecture-status.md](docs/architecture-status.md) is the built/partial/planned
  status ledger cross-checked against the actual repo tree.
- [CONTRIBUTING.md](CONTRIBUTING.md) defines PR shape, CI gates, local setup,
  and contract regeneration.
- [CLAUDE.md](CLAUDE.md) states the hard contract-regeneration rule for coding
  agents.
- Honor any local `AGENTS.md` file in your worktree when present.
- Report vulnerabilities privately via [SECURITY.md](SECURITY.md).

## License

Apache-2.0. See [LICENSE](LICENSE).
