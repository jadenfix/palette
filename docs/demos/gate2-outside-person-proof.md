# Gate 2 Outside-Person Proof

Status: not yet completed.

Gate 2 is not passed until this file is replaced with evidence from a person
outside the project who runs the flow unaided from a fresh clone.

## Runner

- Name:
- Organization or relationship to project:
- Prior Beater repo exposure:
- Date:
- Machine and OS:
- Docker version:
- Docker Compose version:
- Browser:
- Network notes:
- Preflight status:
- Outside-run attestation:

## Repository

- Clone URL: `https://github.com/jadenfix/beater.git`
- Commit SHA:
- Branch:
- Worktree clean:
- OS/arch:
- Beater image reference:
- Dashboard image reference:
- Dashboard e2e image reference:
- OTEL Python image reference:
- Beater image digest:
- Dashboard image digest:
- Dashboard e2e image digest:
- OTEL Python image digest:
- API endpoint:
- Dashboard base: `http://127.0.0.1:3000`
- Timing start source:
- Clone started at:
- Script started at:
- Started at:
- Ended at:
- Time-to-first-trace:
- Script-to-first-trace:
- Time-to-quickstart-click:
- Script-to-quickstart-click:
- Total proof duration:
- Script duration:
- Outside-run wrapper:

## Commands

```bash
bash -lc 't="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'
```

No project maintainer may provide step-by-step help beyond public repo docs
during the timed run.
Run the command from a directory that does not already contain `beater/`; reruns
must start from a new or empty parent directory.

The wrapper sets the required proof/browser/recording flags and rejects
non-`main` checkouts, non-canonical GitHub origins, dirty worktrees, warm-loop reuse,
local source builds, alternate ports, mutable pull-policy overrides,
prebuilt image overrides, evidence artifact path overrides, Compose project overrides,
and teardown overrides before the stopwatch starts. It also requires
`BEATER_GATE2_CLONE_STARTED_EPOCH` from before `git clone`, so
`Time-to-first-trace` and `Time-to-quickstart-click` include clone time. The
stopwatch proof records
`Outside-run wrapper: yes`, `Git branch: main`, the Git origin, and
`Git worktree clean: yes`; completed outside-person evidence is invalid without
those markers.

The script fails before Compose startup if local Docker is unavailable, if curl
or `ffprobe` is missing, if recording SHA tooling is missing, or if API `8080`,
OTLP `4317`, or dashboard `3000` are still in use after it removes any previous
Beater stopwatch project. It also requires `python3` before the timed run so
proof generation and validation cannot fail late on missing local tooling;
Python 3.9 or newer is required. The stock
OpenTelemetry Python snippet runs in the prebuilt `otel-python` container, and
browser proof runs in the prebuilt `dashboard-e2e` container. Remote
`DOCKER_HOST` values and remote Docker contexts are rejected because the browser
proof connects to `127.0.0.1`. For
this outside-person proof, free those default ports instead of using alternate
port environment variables.

After the script prints the dashboard URLs, the runner must open the
quickstart dashboard URL in a normal browser, click the quickstart trace, click
the `llm.call` span, and capture prompt, completion, model, tokens, cost, and
latency. Then the runner must open the all-kind dashboard URL and capture the
run -> turn -> step -> tool -> MCP waterfall. Cleanup can happen after the
recording.

After the stopwatch command finishes, prefer generating completed evidence from
the stopwatch proof instead of manually copying fields. Replace every identity
and environment example below with the runner's actual values. Do not leave
placeholder values such as `...`; the generator and validator reject unresolved
evidence.

```bash
scripts/generate-gate2-outside-proof.py \
  --runner-name "Jane Outside Runner" \
  --relationship "external evaluator; no Beater project role" \
  --prior-exposure "none" \
  --machine-os "Ubuntu 24.04 x86_64" \
  --browser "Chrome stable" \
  --network-notes "home Wi-Fi; no VPN" \
  --llm-observation "clicked llm.call and saw prompt, completion, model, tokens, cost, and latency" \
  --waterfall-observation "opened all-kind trace and saw run -> turn -> step -> tool -> MCP nesting" \
  --preflight-status "passed" \
  --attest-outside-run
```

After replacing this template with completed evidence, run:

```bash
scripts/validate-gate2-outside-proof.sh
```

Maintainers should run this full public-clone verifier before handing the repo
to an outside runner, after the `container-images` workflow has published the
current commit:

```bash
scripts/check-gate2-public-handoff.py --full-run
```

That mode first preflights the local runtime: canonical public source URL only,
`docker`, Docker Compose v2, `curl`, `ffprobe`, local Docker daemon, SHA tooling,
and free default ports after removing any previous `beater-stopwatch` Compose project.
Remote `DOCKER_HOST` values and remote Docker contexts fail before clone or
Compose cleanup. It runs `scripts/check-gate2-outside-readiness.py`, uses one
fresh clone from `https://github.com/jadenfix/beater.git` for exact-commit,
cloned readiness, and wrapper dry-run checks, then uses a second fresh clone for
the timed runtime path. The readiness check verifies clean `main`, the expected
GitHub remote, this proof file's structure, and public multi-arch GHCR images
for the exact commit. The verifier executes the second clone's
`scripts/gate2-outside-run.sh` wrapper with the clone-start timestamp captured
immediately before that second `git clone`, and cleans up the `beater-stopwatch`
Compose project after the wrapper exits. It proves the exact public outside-run
path and images can run, but it is not outside-person evidence and does not
close this proof file. `--full-run` is intentionally supported only for the
canonical public GitHub/GHCR handoff, not fixture or fork URLs.

If Docker is unavailable on the maintainer machine, run
`scripts/check-gate2-public-handoff.py` without `--full-run`; that still
performs the public clone, exact-commit, wrapper dry-run, proof-structure, and
multi-arch GHCR-image checks, but it is not a runtime handoff proof.

The validator reads the listed stopwatch proof file, screen-recording notes, and
`ffprobe` playable-video metadata,
then cross-checks default API/OTLP/dashboard endpoints, clean-start status,
browser-proof status, trace IDs, dashboard URLs, SHA-pinned prebuilt GHCR image
references, prebuilt GHCR image digests, stock quickstart snippet markers, and
the tested public GitHub origin,
`main` branch, clean-worktree state, and commit SHA. If the proof commit is newer
than the tested SHA, every later committed change must be under `docs/demos/`;
uncommitted non-evidence worktree changes are rejected at closure. It verifies
screen-recording SHA256 against the committed artifact, requires the artifact to
be a playable WebM capture of at least 64 KiB with EBML/WebM, Segment, Info,
Tracks, and Cluster structure plus a video track, and requires the recording
notes to describe the full click-through: quickstart trace, `llm.call`, prompt,
completion, model, tokens, cost, latency, and run -> turn -> step -> tool -> MCP
waterfall. Stopwatch, recording, and notes paths must be repo-relative paths
under `docs/demos/` and must not resolve through symlinks.

## Required Evidence

- Stopwatch proof file:
- Screen recording: `docs/demos/gate2-compose-browser-demo.webm`
- Screen recording notes: `docs/demos/gate2-compose-browser-demo.md`
- Screen recording SHA256:
- Terminal output excerpt:
- Runner llm.call observation:
- Runner waterfall observation:
- `docker compose images` excerpt:
- Quickstart trace ID:
- Quickstart dashboard URL:
- All-kind nested trace ID:
- All-kind dashboard URL:
- `docker compose` logs saved:
- Failure notes, if any:

## Pass Checklist

- [ ] Fresh clone was used.
- [ ] Docker was running before the stopwatch started.
- [ ] curl was available before the stopwatch started.
- [ ] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`,
      dashboard `127.0.0.1:3000`.
- [ ] `BEATER_GATE2_REUSE` was not set.
- [ ] The script reported `Clean start: yes`.
- [ ] Time-to-first-trace was 300 seconds or less.
- [ ] Time-to-first-trace includes clone time.
- [ ] Time-to-quickstart-click was 300 seconds or less.
- [ ] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [ ] Clicking the `llm.call` span showed prompt, completion, model, tokens,
      cost, and latency.
- [ ] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in
      the waterfall.
- [ ] The browser proof passed for both the quickstart trace and all-kind
      waterfall.
- [ ] The stopwatch script generated and reported the browser recording.
- [ ] A screen recording of the full flow is committed under `docs/demos/`.
- [ ] The runner completed the flow using only public repository instructions.

## Runner Notes

Add any confusing step, missing prerequisite, slow pull, failing command, or UI
ambiguity here. These notes are blocking feedback until fixed or explicitly
triaged.
