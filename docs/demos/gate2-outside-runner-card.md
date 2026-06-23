# Gate 2 Outside Runner Card

Use this card for the unaided Gate 2 run. It is intentionally shorter than the
full proof template.

## Before You Start

Use a machine with:

- Docker Desktop or another local Docker daemon
- Docker Compose v2
- `git`, `curl`, `python3` 3.9 or newer
- `ffprobe` (installed by common `ffmpeg` packages)
- `shasum` or `sha256sum`
- local ports `8080`, `4317`, and `3000` free

Run from an empty parent directory that does not already contain `beater/`.
Do not set Beater, Docker Compose, or alternate port environment variables.
If preflight reports stale `beater-stopwatch` containers or occupied default
ports, follow the hint it prints: clean stale Beater containers, or stop/move
the reported non-Beater app listening on the port. Do not set alternate Beater
ports. Then rerun this card from a new or empty parent directory.

## Timed Command

Run this exact command from Bash, zsh, Git Bash, or WSL2:

```bash
bash -o pipefail -lc 'sha_line="$(git ls-remote --exit-code https://github.com/jadenfix/beater.git refs/heads/main)" && sha="${sha_line%%[[:space:]]*}" && test -n "$sha" && preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && curl -fsSL "https://raw.githubusercontent.com/jadenfix/beater/$sha/scripts/gate2-outside-local-preflight.sh" -o "$preflight" && bash "$preflight" && t="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && test "$(git rev-parse HEAD)" = "$sha" && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'
```

The command downloads preflight from the resolved public commit SHA, verifies
the clone still matches that SHA, and includes clone and image-pull time in the
timer.

## Timed Browser Step

When the terminal prints `Open this quickstart trace-list URL first:`, open that
URL in a normal browser immediately. Do not wait for the script to finish.

Click the quickstart trace, then click the `llm.call` span. Confirm that the
prompt, completion, model, token breakdown, cost, latency, and the `Confirm`
code are visible. Type that confirmation code in the terminal, then press Enter.
The terminal checkpoint must happen before the 5-minute clone-to-click SLO expires.
This records `Manual confirmation source: browser-selected-llm-detail`; do not
copy the code from terminal logs or generated files.

## Post-SLO Evidence

After the timed click is recorded, leave the command running. It will run the
browser proof, generate the all-kind waterfall trace, seed a sensitive redacted
I/O trace, record the browser video, and save
`docs/demos/gate2-outside-compose.log`.

Open the printed all-kind dashboard URL and confirm the waterfall shows:

```text
run -> turn -> step -> tool -> MCP
```

The automated browser proof and recording also cover the printed redaction
dashboard URL: prompt/completion are redacted by default, the unmask reason is
submitted, unmasked I/O appears, and Redacted view is restored.

## Proof Handoff

After the command exits, use the printed
`scripts/generate-gate2-outside-proof.py --print-command` output. Replace every `...` field with real runner values, then run it.
The generated proof must keep the stopwatch's fresh quickstart release ID,
trace IDs, span IDs, redaction unmask reason, and manual confirmation source;
validation rejects stale or mismatched values.

Commit the evidence before validation:

```bash
git add docs/demos/gate2-outside-person-proof.md \
  docs/demos/gate2-compose-stopwatch.md \
  docs/demos/gate2-compose-browser-demo.webm \
  docs/demos/gate2-compose-browser-demo.md \
  docs/demos/gate2-outside-compose.log
git commit -m "add gate2 outside proof"
scripts/validate-gate2-outside-proof.sh
```

Gate 2 only closes if this proof comes from someone outside the project who
completed the run unaided using public repository instructions.
