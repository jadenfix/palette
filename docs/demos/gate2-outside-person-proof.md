# Gate 2 Outside-Person Proof

Status: not yet completed.

Gate 2 is not passed until this file is replaced with evidence from a person
outside the project who runs the flow unaided from a fresh clone.
For the short unaided runner instructions, use
[gate2-outside-runner-card.md](gate2-outside-runner-card.md).

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
- Quickstart release ID:
- Timing start source:
- Clone started at:
- Script started at:
- Started at:
- Ended at:
- Time-to-first-trace:
- Script-to-first-trace:
- Time-to-quickstart-click:
- Script-to-quickstart-click:
- Quickstart click source:
- Manual quickstart confirmation:
- Manual confirmation source:
- Manual confirmation code:
- Manual confirmation salt:
- Total proof duration:
- Script duration:
- Outside-run wrapper:

## Commands

```bash
bash -o pipefail -lc 'sha_line="$(GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git ls-remote --exit-code https://github.com/jadenfix/beater.git refs/heads/main)" && sha="${sha_line%%[[:space:]]*}" && test -n "$sha" && preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && curl -fsSL "https://raw.githubusercontent.com/jadenfix/beater/$sha/scripts/gate2-outside-local-preflight.sh" -o "$preflight" && BEATER_GATE2_EXPECTED_COMMIT="$sha" bash "$preflight" && t="$(date +%s)" && GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git clone https://github.com/jadenfix/beater.git && cd ./beater && test "$(GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 git rev-parse HEAD)" = "$sha" && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0 scripts/gate2-outside-run.sh'
```

No project maintainer may provide step-by-step help beyond public repo docs
during the timed run.
Run the command from a directory that does not already contain `beater/`; reruns
must start from a new or empty parent directory. If an aborted previous attempt
left default ports occupied by `beater-stopwatch`, use the cleanup hint printed
by the preflight before rerunning. If preflight reports another app on a default
port, stop that app and rerun; do not set alternate Beater ports.

The wrapper sets the required proof/browser/recording flags and rejects
non-`main` checkouts, non-canonical GitHub origins, dirty worktrees, warm-loop reuse,
local source builds, alternate ports, mutable pull-policy overrides,
prebuilt image overrides, evidence artifact path overrides, Compose file/profile/project
overrides, and teardown overrides before the stopwatch starts. It also requires
`BEATER_GATE2_CLONE_STARTED_EPOCH` from before `git clone`, so
`Time-to-first-trace` and `Time-to-quickstart-click` include clone time. The
stopwatch proof records
`Outside-run wrapper: yes`, `Git branch: main`, the Git origin, and
`Git worktree clean: yes`; completed outside-person evidence is invalid without
those markers. The stopwatch proof must also identify itself as an
outside-run stopwatch source artifact, not an automated local stopwatch proof.

The command runs public Git operations with global/system config disabled, then
runs `scripts/gate2-outside-local-preflight.sh` from the public repo before the
stopwatch starts, so missing local tooling, unpublished SHA-tagged GHCR images,
remote Docker contexts, and occupied default ports fail before the timed attempt.
The cloned wrapper then repeats the same checks. The script fails before Compose startup if local Docker is unavailable, if curl
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
port environment variables. If preflight reports another process on a default
port, stop that app and rerun. Do not set `COMPOSE_FILE`, `COMPOSE_PROJECT_NAME`,
or `COMPOSE_PROFILES`; the public command controls the Compose topology.

As soon as the first `Open this quickstart trace-list URL first:` URL appears,
the runner must open that filtered trace-list URL in a normal browser; do not
wait for the script to finish. The manual checkpoint prints the seconds
remaining in the 5-minute clone-to-click SLO, which already includes clone and
image-pull time. Click the quickstart trace, click the `llm.call` span, and
capture prompt, completion, model, token breakdown, cost, latency, and the
`Confirm` code shown in the selected detail. Type that confirmation code in the
terminal and press Enter only after the manual click-through is complete; this
records `Quickstart click source: manual-outside-runner`,
`Manual quickstart confirmation: yes`,
`Manual confirmation source: browser-selected-llm-detail`,
`Manual confirmation code: <code>`, the per-run manual confirmation salt, and
the 5-minute quickstart-click SLO.
Then keep the script running for the post-SLO automated browser proof, all-kind
trace, and recording evidence, open the all-kind dashboard URL, and capture the
run -> turn -> step -> tool -> MCP waterfall. The same post-SLO proof seeds a
sensitive native `llm.call` trace, verifies prompt/completion are redacted by
default, submits the required unmask reason, verifies unmasked I/O, and returns
to Redacted view. Cleanup can happen after the recording.

After the stopwatch command finishes, the one-liner returns the runner's parent
shell to the directory that contains `beater/`. Run `cd ./beater`, then use the
prefilled `scripts/generate-gate2-outside-proof.py --print-command` output
printed in the terminal. It copies the stopwatch-derived dashboard URLs,
terminal excerpt, outside-run terminal transcript, and compose-log artifact into
a ready-to-edit command. Replace every `...` field with the runner's actual
values before running it; the generator and validator reject unresolved
evidence. `--prior-exposure "none"` is valid when the runner has never seen the
repository before, and the proof date defaults to the UTC date captured in the
stopwatch proof's `Clone started at` field. Save the outside-run terminal
transcript and compose logs as repo-relative, committed/clean, non-symlink files
under `docs/demos/` (for example `docs/demos/gate2-outside-terminal.log` and
`docs/demos/gate2-outside-compose.log`), or use an immutable GitHub Actions
run/job URL for compose logs such as
`https://github.com/jadenfix/beater/actions/runs/<run_id>`. The outside-run
wrapper writes `docs/demos/gate2-outside-compose.log` automatically and
pre-fills that path with `--compose-logs-saved`; it also writes
`docs/demos/gate2-outside-terminal.log` and pre-fills
`--terminal-transcript-saved`. For local compose-log files, the validator checks
the stopwatch-written `# Gate 2 Compose Logs` header, `beater-stopwatch`
project, `prebuilt-image` startup mode, and timestamped compose logs command.

If you need to reprint the command, run:

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

After replacing this template with completed evidence, stay in the `beater/`
clone and run:

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

Maintainers should run this full public-clone verifier before handing the repo
to an outside runner, after the `container-images` workflow has published the
current commit:

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
path. The readiness check verifies clean `main`, the expected
GitHub remote, this proof file's structure, and public multi-arch GHCR images
for the exact commit. The verifier executes the second clone's
`scripts/gate2-outside-run.sh` wrapper with the clone-start timestamp captured
immediately before that second `git clone`, waits until the wrapper prints the
manual quickstart checkpoint, uses a browser click to read and enter the confirmation
code from the selected `llm.call` detail for diagnostic automation only, and cleans up the `beater-stopwatch` Compose project after the
wrapper exits. It proves the exact public outside-run path and images can run,
but it is not outside-person evidence and does not close this proof file. Its
generated report is `Status: diagnostic.` and default outside-person validation
rejects it as closure evidence. `--full-run` is intentionally supported only
for the canonical public GitHub/GHCR handoff,
not fixture or fork URLs.

If Docker is unavailable on the maintainer machine, run
`scripts/check-gate2-public-handoff.py` without `--full-run`; that still
performs the public clone, exact-commit, wrapper dry-run, proof-structure, and
multi-arch GHCR-image checks, but it is not a runtime handoff proof.

The validator reads the listed stopwatch proof file, screen-recording notes,
outside-run terminal transcript, and `ffprobe` playable-video metadata,
then cross-checks default API/OTLP/dashboard endpoints, clean-start status,
browser-proof status, trace IDs, dashboard URLs, per-run quickstart release ID,
the same quickstart release ID in the screen-recording notes,
redacted-I/O browser proof status, redaction trace ID, redaction span ID,
redaction dashboard URL, redaction unmask reason,
SHA-pinned prebuilt GHCR image references, structured compose service and
`proof-image` digest rows, prebuilt GHCR image digests bound to the public GHCR
manifest digest set for the exact SHA tag, stock quickstart snippet markers, and
the tested public GitHub origin, Date-to-clone-start consistency,
`main` branch, clean-worktree state, and commit SHA. If the proof commit is newer
than the tested SHA, every later committed change must be under `docs/demos/`;
uncommitted non-evidence worktree changes are rejected at closure. It verifies
the stopwatch proof identifies itself as an outside-run stopwatch source
artifact, recorded a manual outside-runner quickstart click confirmation before
the 5-minute SLO, verifies
screen-recording SHA256 against the committed artifact, requires the artifact to
be a playable WebM capture of at least 64 KiB and at least 8 seconds with
EBML/WebM, Segment, Info, Tracks, and Cluster structure plus a video track, and
requires the outside-run terminal transcript to contain the manual checkpoint
prompt, dashboard URLs, final pass line, and prefilled proof command. It also
requires the recording notes to declare `Recording mode: compose`, the matching
quickstart release ID, and describe
the full click-through: quickstart trace, `llm.call`, prompt, completion, model,
token breakdown, cost, latency, confirmation code, and run -> turn -> step ->
tool -> MCP waterfall, plus the sensitive `llm.call` redacted prompt/completion,
unmask reason, unmasked I/O, and Redacted view.
Stopwatch, recording, notes, saved outside-run terminal transcript, and saved
compose-log paths must be repo-relative paths under `docs/demos/` and must not
resolve through symlinks. Saved compose-log evidence must be a committed/clean
file at closure, or an immutable GitHub Actions run/job URL under
`https://github.com/jadenfix/beater/actions/runs/`. Local compose-log files must
contain the stopwatch-written header, canonical project, prebuilt startup mode,
and timestamped compose logs command.

## Required Evidence

- Stopwatch proof file:
- Screen recording: `docs/demos/gate2-compose-browser-demo.webm`
- Screen recording notes: `docs/demos/gate2-compose-browser-demo.md`
- Screen recording SHA256:
- Terminal output excerpt:
- Outside-run terminal transcript:
- Runner llm.call observation:
- Runner waterfall observation:
- `docker compose images` excerpt:
- Quickstart trace ID:
- Quickstart span ID:
- Quickstart dashboard URL:
- All-kind nested trace ID:
- All-kind dashboard URL:
- Redaction browser proof:
- Redaction trace ID:
- Redaction span ID:
- Redaction dashboard URL:
- Redaction unmask reason:
- `docker compose` logs saved: repo-relative committed/clean non-symlink `docs/demos/` log file or immutable GitHub Actions run/job URL
- Failure notes, if any:

## Pass Checklist

- [ ] Fresh clone was used.
- [ ] Docker was running before the stopwatch started.
- [ ] curl was available before the stopwatch started.
- [ ] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`,
      dashboard `127.0.0.1:3000`.
- [ ] `BEATER_GATE2_REUSE` was not set.
- [ ] `COMPOSE_FILE`, `COMPOSE_PROJECT_NAME`, and `COMPOSE_PROFILES` were not set.
- [ ] The script reported `Clean start: yes`.
- [ ] Time-to-first-trace was 300 seconds or less.
- [ ] Time-to-first-trace includes clone time.
- [ ] Manual quickstart click confirmation code was recorded before 300 seconds.
- [ ] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [ ] Clicking the `llm.call` span showed prompt, completion, model, token
      breakdown, cost, latency, and confirmation code.
- [ ] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in
      the waterfall.
- [ ] The redacted-I/O browser proof showed redacted defaults, reasoned unmask,
      unmasked I/O, and Redacted view.
- [ ] The browser proof passed for the quickstart trace, all-kind waterfall,
      and redacted-I/O controls.
- [ ] The stopwatch script generated and reported the browser recording.
- [ ] A screen recording of the full flow is committed under `docs/demos/`.
- [ ] The runner completed the flow using only public repository instructions.

## Runner Notes

Add any confusing step, missing prerequisite, slow pull, failing command, or UI
ambiguity here. These notes are blocking feedback until fixed or explicitly
triaged.
