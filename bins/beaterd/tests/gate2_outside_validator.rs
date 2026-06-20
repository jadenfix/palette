use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const QUICKSTART_TRACE: &str = "11111111111111111111111111111111";
const ALL_KIND_TRACE: &str = "22222222222222222222222222222222";
const COMMIT_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECORDING_BYTES: &[u8] = b"beater gate2 validator video\n";
const RECORDING_SHA: &str = "996b14a456ef7d971a97600ecf240cc5f22eb5b5c05235719a64a042a4fdb29e";
const BEATER_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/beaterd@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DASHBOARD_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/dashboard@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

#[test]
fn gate2_outside_validator_allows_pending_template_with_allow_pending() {
    let output = run_default_validator(&["--allow-pending"]);

    assert_success(
        output,
        "Gate 2 outside-person proof is pending but structurally valid",
    );
}

#[test]
fn gate2_outside_validator_accepts_matching_default_port_artifacts() {
    let fixture = ValidatorFixture::new();

    let output = run_validator(&fixture.proof_path);

    assert_success(output, "Gate 2 outside-person proof is complete and valid");
}

#[test]
fn gate2_outside_validator_rejects_missing_stopwatch_proof() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        fixture.stopwatch_path.to_str().unwrap(),
        fixture
            .dir
            .path()
            .join("missing-stopwatch.md")
            .to_str()
            .unwrap(),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "stopwatch proof file does not exist");
}

#[test]
fn gate2_outside_validator_rejects_alternate_port_stopwatch_artifact() {
    let fixture = ValidatorFixture::new();
    replace(&fixture.stopwatch_path, "127.0.0.1:3000", "127.0.0.1:13080");
    replace(&fixture.stopwatch_path, "127.0.0.1:4317", "127.0.0.1:14317");

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "stopwatch proof must not use alternate/warm-loop evidence",
    );
}

#[test]
fn gate2_outside_validator_rejects_dashboard_url_trace_mismatch() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!("trace={QUICKSTART_TRACE}`"),
        &format!("trace={ALL_KIND_TRACE}`"),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        &format!("Quickstart dashboard URL must include trace={QUICKSTART_TRACE}"),
    );
}

#[test]
fn gate2_outside_validator_rejects_stale_recording_notes() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.notes_path,
        "Dashboard base: `http://127.0.0.1:3000`",
        "Dashboard base: `http://127.0.0.1:13080`",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording notes dashboard base must be http://127.0.0.1:3000",
    );
}

#[test]
fn gate2_outside_validator_rejects_bad_recording_hash() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        RECORDING_SHA,
        "87269612de7fdfdc9e671c7d4bb96b5b0b8d034ef799580e91c7e5d9d3ee6ab2",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "screen recording sha mismatch");
}

#[test]
fn gate2_outside_validator_rejects_image_digest_mismatch() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        BEATER_IMAGE_DIGEST,
        "ghcr.io/jadenfix/beater/beaterd@sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "beater image digest mismatch between proof artifacts",
    );
}

struct ValidatorFixture {
    dir: TempDir,
    proof_path: PathBuf,
    stopwatch_path: PathBuf,
    notes_path: PathBuf,
}

impl ValidatorFixture {
    fn new() -> Self {
        let dir = TempDir::new().expect("create validator fixture tempdir");
        let proof_path = dir.path().join("outside-proof.md");
        let stopwatch_path = dir.path().join("stopwatch-proof.md");
        let notes_path = dir.path().join("recording-notes.md");
        let recording_path = dir.path().join("recording.webm");

        fs::write(&recording_path, RECORDING_BYTES)
            .unwrap_or_else(|err| panic!("write {}: {err}", recording_path.display()));
        fs::write(
            &notes_path,
            recording_notes(&recording_path.file_name().unwrap().to_string_lossy()),
        )
        .unwrap_or_else(|err| panic!("write {}: {err}", notes_path.display()));
        fs::write(
            &stopwatch_path,
            stopwatch_proof(&recording_path, &notes_path),
        )
        .unwrap_or_else(|err| panic!("write {}: {err}", stopwatch_path.display()));
        fs::write(
            &proof_path,
            outside_proof(&stopwatch_path, &recording_path, &notes_path),
        )
        .unwrap_or_else(|err| panic!("write {}: {err}", proof_path.display()));

        Self {
            dir,
            proof_path,
            stopwatch_path,
            notes_path,
        }
    }
}

fn outside_proof(stopwatch_path: &Path, recording_path: &Path, notes_path: &Path) -> String {
    format!(
        r#"# Gate 2 Outside-Person Proof

Status: completed.

## Runner

- Name: Validator Fixture Runner
- Organization or relationship to project: external validation fixture
- Prior Beater repo exposure: no prior exposure
- Date: 2026-06-20
- Machine and OS: macOS arm64
- Docker version: Docker version 29.2.0
- Docker Compose version: Docker Compose version v5.0.2
- Browser: Chromium
- Network notes: public docs only
- Preflight status: passed

## Repository

- Clone URL: `https://github.com/jadenfix/beater.git`
- Commit SHA: {COMMIT_SHA}
- Branch: main
- OS/arch: Darwin arm64
- Beater image digest: {BEATER_IMAGE_DIGEST}
- Dashboard image digest: {DASHBOARD_IMAGE_DIGEST}
- Started at: 2026-06-20T12:00:00Z
- Ended at: 2026-06-20T12:00:40Z
- Time-to-first-trace: 12s
- Time-to-quickstart-click: 20s
- Total proof duration: 40s

## Commands

```bash
git clone https://github.com/jadenfix/beater.git
cd beater
BEATER_GATE2_WRITE_PROOF=1 BEATER_GATE2_BROWSER_PROOF=1 BEATER_GATE2_RECORD_DEMO=1 scripts/gate2-compose-stopwatch.sh
```

The runner completed the flow using only public repository instructions.

## Required Evidence

- Stopwatch proof file: {stopwatch}
- Screen recording: `{recording}`
- Screen recording notes: `{notes}`
- Screen recording SHA256: {RECORDING_SHA}
- Terminal output excerpt: generated proof says browser recording passed
- `docker compose images` excerpt: beaterd and dashboard images present
- Quickstart trace ID: {QUICKSTART_TRACE}
- Quickstart dashboard URL: `http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={QUICKSTART_TRACE}`
- All-kind nested trace ID: {ALL_KIND_TRACE}
- All-kind dashboard URL: `http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={ALL_KIND_TRACE}`
- `docker compose` logs saved: temp fixture
- Failure notes, if any: none

## Pass Checklist

- [x] Fresh clone was used.
- [x] Docker was running before the stopwatch started.
- [x] Python, curl, and npm were available before the stopwatch started.
- [x] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`, dashboard `127.0.0.1:3000`.
- [x] `BEATER_GATE2_REUSE` was not set.
- [x] The script reported `Clean start: yes`.
- [x] Time-to-first-trace was 300 seconds or less.
- [x] Time-to-quickstart-click was 300 seconds or less.
- [x] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [x] Clicking the `llm.call` span showed prompt, completion, model, tokens, cost, and latency.
- [x] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in the waterfall.
- [x] The browser proof passed for both the quickstart trace and all-kind waterfall.
- [x] The stopwatch script generated and reported the browser recording.
- [x] A screen recording of the full flow is committed under `docs/demos/`.
- [x] The runner completed the flow using only public repository instructions.
"#,
        stopwatch = stopwatch_path.display(),
        recording = recording_path.display(),
        notes = notes_path.display(),
    )
}

fn stopwatch_proof(recording_path: &Path, notes_path: &Path) -> String {
    format!(
        r#"# Gate 2 Compose Stopwatch Proof

- Started: 2026-06-20T12:00:00Z
- Ended: 2026-06-20T12:00:40Z
- Time-to-first-trace: 12s
- Time-to-quickstart-click: 20s
- Total duration: 40s
- Limit: 300s
- Git SHA: `{COMMIT_SHA}`
- OS/arch: `Darwin arm64`
- Docker: `Docker version 29.2.0`
- Docker Compose: `Docker Compose version v5.0.2`
- Startup mode: prebuilt-image
- Clean start: yes
- Reuse override: `BEATER_GATE2_REUSE=0`
- Prebuilt pull policy: `always`
- Compose project: beater-stopwatch
- Beater image digest: `{BEATER_IMAGE_DIGEST}`
- Dashboard image digest: `{DASHBOARD_IMAGE_DIGEST}`
- Quickstart snippet: `examples/python/five_line_otel.py`
- OTLP endpoint: `http://127.0.0.1:4317`
- Quickstart trace: `{QUICKSTART_TRACE}`
- Quickstart dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={QUICKSTART_TRACE}
- Quickstart browser proof: passed
- All-kind nested trace: `{ALL_KIND_TRACE}`
- All-kind dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={ALL_KIND_TRACE}
- All-kind waterfall browser proof: passed
- Browser recording: passed
- Browser recording artifact: `{recording}`
- Browser recording notes: `{notes}`
- Browser recording SHA256: `{RECORDING_SHA}`
"#,
        recording = recording_path.display(),
        notes = notes_path.display(),
    )
}

fn recording_notes(recording_name: &str) -> String {
    format!(
        r#"# Gate 2 Compose Browser Demo

- Artifact: `{recording_name}`
- SHA256: `{RECORDING_SHA}`
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart trace: `{QUICKSTART_TRACE}`
- All-kind trace: `{ALL_KIND_TRACE}`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> inspect run -> turn -> step -> tool -> MCP waterfall.
"#
    )
}

fn run_validator(proof_path: &Path) -> Output {
    let root = repo_root();
    Command::new("bash")
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .current_dir(root)
        .env("BEATER_GATE2_OUTSIDE_PROOF", proof_path)
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof validator: {err}"))
}

fn run_default_validator(args: &[&str]) -> Output {
    let root = repo_root();
    Command::new("bash")
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .args(args)
        .current_dir(root)
        .env_remove("BEATER_GATE2_OUTSIDE_PROOF")
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof validator: {err}"))
}

fn assert_success(output: Output, expected_stdout: &str) {
    if !output.status.success() {
        panic!(
            "validator failed unexpectedly\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(expected_stdout),
        "stdout did not contain {expected_stdout:?}\nstdout:\n{stdout}"
    );
}

fn assert_failure(output: Output, expected_stderr: &str) {
    if output.status.success() {
        panic!(
            "validator succeeded unexpectedly\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_stderr),
        "stderr did not contain {expected_stderr:?}\nstderr:\n{stderr}"
    );
}

fn replace(path: &Path, from: &str, to: &str) {
    let text =
        fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    assert!(
        text.contains(from),
        "fixture {} did not contain replacement source {from:?}",
        path.display()
    );
    fs::write(path, text.replace(from, to))
        .unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
}

fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = manifest.parent().and_then(|path| path.parent()) else {
        panic!("beaterd manifest must live under bins/beaterd");
    };
    root.to_path_buf()
}
