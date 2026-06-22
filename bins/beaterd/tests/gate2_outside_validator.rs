use std::fs;
use std::io::Write;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const QUICKSTART_TRACE: &str = "11111111111111111111111111111111";
const ALL_KIND_TRACE: &str = "22222222222222222222222222222222";
const RECORDING_SHA: &str = "3dac802bc8f2db03406d0d76e4e1618ed5b516a2cf3d286589e1a588cf6e6534";
const BEATER_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/beaterd@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DASHBOARD_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/dashboard@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const DASHBOARD_E2E_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/dashboard-e2e@sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const OTEL_PYTHON_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/otel-python@sha256:abababababababababababababababababababababababababababababababab";
const LLM_OBSERVATION: &str =
    "clicked llm.call and saw prompt, completion, model, token breakdown, cost, and latency";
const WATERFALL_OBSERVATION: &str =
    "opened all-kind trace and saw run -> turn -> step -> tool -> MCP nesting";
const OUTSIDE_RUN_ATTESTATION: &str = "I attest that I am not a Beater project maintainer, I received no step-by-step help beyond public repository instructions, I used a fresh clone, and I completed the Gate 2 flow unaided.";
const DIAGNOSTIC_ATTESTATION: &str = "Diagnostic maintainer full-run auto-confirmed the manual checkpoint; this is not outside-person evidence and cannot close Gate 2.";
const DRAFT_VALID: &str = "Gate 2 outside-person proof draft is internally consistent";
const CLOSURE_VALID: &str = "Gate 2 outside-person proof is complete and valid";

#[test]
fn gate2_outside_validator_allows_pending_template_with_allow_pending() {
    let output = run_default_validator(&["--allow-pending"]);

    assert_success(
        output,
        "Gate 2 outside-person proof is pending but structurally valid",
    );
}

#[test]
fn gate2_outside_validator_resolves_default_template_from_script_path() {
    let root = repo_root();
    let cwd = tempdir("create non-repo validator cwd");
    let output = Command::new("bash")
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .arg("--allow-pending")
        .current_dir(cwd.path())
        .env_remove("BEATER_GATE2_OUTSIDE_PROOF")
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside validator outside repo cwd: {err}"));

    assert_success(
        output,
        "Gate 2 outside-person proof is pending but structurally valid",
    );
}

#[test]
fn gate2_outside_docs_use_fail_fast_clone_command() {
    let root = repo_root();
    for rel in [
        "README.md",
        "docs/demos/gate2-outside-person-proof.md",
        "scripts/generate-gate2-outside-proof.py",
    ] {
        let text =
            fs::read_to_string(root.join(rel)).unwrap_or_else(|err| panic!("read {rel}: {err}"));
        assert!(
            !text.contains("git clone https://github.com/jadenfix/beater.git; cd beater"),
            "{rel} must not allow stale-clone semicolon chaining"
        );
    }

    let readme = fs::read_to_string(root.join("README.md"))
        .unwrap_or_else(|err| panic!("read README.md: {err}"));
    assert!(readme.contains(r#"git clone https://github.com/jadenfix/beater.git && cd beater &&"#));
    assert!(readme.contains("confirms the\nquickstart browser click unaided in 5"));
    assert!(readme.contains("`scripts/check-gate2-public-handoff.py` without `--full-run`"));
    assert!(readme.contains("and `python3` 3.9+; local ports"));
    assert!(readme.contains("`ffprobe`, `shasum` or `sha256sum`"));
    assert!(readme.contains("curl, `ffprobe`,\nand SHA tooling"));
    assert!(readme.contains("Docker Compose v2, `curl`, `ffprobe`, local Docker daemon"));
    assert!(readme.contains("requires `python3` 3.9+ before the timed run"));
    assert!(!readme.contains("`python3` for post-run proof generation"));
    assert!(!readme.contains("`python3` is required afterward"));
    assert!(readme.contains("--llm-observation"));
    assert!(readme.contains("--waterfall-observation"));
    assert!(readme.contains("Do not leave placeholder values such as `...`"));
    assert!(readme.contains("uncommitted non-evidence worktree changes"));
    assert!(readme.contains(r#"--runner-name "Jane Outside Runner""#));
    assert!(readme.contains(r#"--relationship "external evaluator; no Beater project role""#));
    assert!(!readme.contains(r#"--relationship "external evaluator; no Beater maintainer role""#));
    assert!(!readme.contains(r#"--runner-name "...""#));
    assert!(!readme.contains(r#"--relationship "...""#));
    assert!(!readme.contains(r#"--machine-os "...""#));
    assert!(!readme.contains(r#"--browser "...""#));
    assert!(!readme.contains(r#"--network-notes "...""#));
    let proof_template = fs::read_to_string(root.join("docs/demos/gate2-outside-person-proof.md"))
        .unwrap_or_else(|err| panic!("read outside proof template: {err}"));
    assert!(proof_template.contains("`scripts/check-gate2-public-handoff.py` without `--full-run`"));
    assert!(proof_template.contains("Python 3.9 or newer is required"));
    assert!(proof_template.contains("curl\nor `ffprobe` is missing"));
    assert!(proof_template.contains("Docker Compose v2, `curl`, `ffprobe`, local Docker daemon"));
    assert!(proof_template.contains("`ffprobe` playable-video metadata"));
    assert!(proof_template.contains("uncommitted non-evidence worktree changes"));
    assert!(proof_template.contains("playable WebM"));
    assert!(!proof_template.contains("none / describe"));
    assert!(!proof_template.contains("`python3` is required after the timed run"));
    assert!(!proof_template.contains("http://127.0.0.1:3000/..."));
    assert!(proof_template.contains("--llm-observation"));
    assert!(proof_template.contains("--waterfall-observation"));
    assert!(proof_template.contains("placeholder values such as `...`"));
    assert!(proof_template.contains(r#"--runner-name "Jane Outside Runner""#));
    assert!(
        proof_template.contains(r#"--relationship "external evaluator; no Beater project role""#)
    );
    assert!(!proof_template
        .contains(r#"--relationship "external evaluator; no Beater maintainer role""#));
    assert!(!proof_template.contains(r#"--runner-name "...""#));
    assert!(!proof_template.contains(r#"--relationship "...""#));
    assert!(!proof_template.contains(r#"--machine-os "...""#));
    assert!(!proof_template.contains(r#"--browser "...""#));
    assert!(!proof_template.contains(r#"--network-notes "...""#));
}

#[test]
fn gate2_pending_template_rejects_missing_required_field_label() {
    let fixture =
        TempDir::new().unwrap_or_else(|err| panic!("create pending proof fixture: {err}"));
    let proof_path = fixture.path().join("pending-proof.md");
    let source = fs::read_to_string(repo_root().join("docs/demos/gate2-outside-person-proof.md"))
        .unwrap_or_else(|err| panic!("read pending proof template: {err}"));
    fs::write(
        &proof_path,
        source.replace("- `docker compose images` excerpt:\n", ""),
    )
    .unwrap_or_else(|err| panic!("write {}: {err}", proof_path.display()));

    let output = run_validator_with_args(&proof_path, &["--allow-pending"]);

    assert_failure(
        output,
        "missing field in pending outside-person proof template: `docker compose images` excerpt",
    );
}

#[test]
fn gate2_pending_template_rejects_duplicate_status_line() {
    let fixture =
        TempDir::new().unwrap_or_else(|err| panic!("create pending proof fixture: {err}"));
    let proof_path = fixture.path().join("pending-proof.md");
    let source = fs::read_to_string(repo_root().join("docs/demos/gate2-outside-person-proof.md"))
        .unwrap_or_else(|err| panic!("read pending proof template: {err}"));
    fs::write(&proof_path, source)
        .unwrap_or_else(|err| panic!("write {}: {err}", proof_path.display()));
    append(&proof_path, "\nStatus: completed.\n");

    let output = run_validator_with_args(&proof_path, &["--allow-pending"]);

    assert_failure(output, "duplicate Status line");
}

#[test]
fn gate2_outside_validator_accepts_matching_default_port_artifacts() {
    let fixture = ValidatorFixture::new();

    let output = run_validator(&fixture.proof_path);

    assert_success(output, DRAFT_VALID);
}

#[test]
fn gate2_outside_validator_untracked_escape_does_not_claim_closure() {
    let fixture = ValidatorFixture::new();

    let output = run_validator(&fixture.proof_path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(DRAFT_VALID),
        "draft validation must describe itself as non-closure\nstdout:\n{stdout}"
    );
    assert!(
        !stdout.contains(CLOSURE_VALID),
        "untracked draft validation must not use closure wording\nstdout:\n{stdout}"
    );
}

#[test]
fn gate2_outside_validator_rejects_diagnostic_status_for_closure() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "Status: completed.",
        "Status: diagnostic.",
    );
    replace(
        &fixture.proof_path,
        OUTSIDE_RUN_ATTESTATION,
        DIAGNOSTIC_ATTESTATION,
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Status must be 'completed.' for Gate 2 closure");
}

#[test]
fn gate2_outside_validator_accepts_diagnostic_status_only_in_diagnostic_mode() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "Status: completed.",
        "Status: diagnostic.",
    );
    replace(
        &fixture.proof_path,
        OUTSIDE_RUN_ATTESTATION,
        DIAGNOSTIC_ATTESTATION,
    );
    replace(
        &fixture.proof_path,
        "The runner completed the flow using only public repository instructions.",
        "The maintainer diagnostic full-run auto-confirmed the manual checkpoint. This is not outside-person evidence.",
    );

    let output = run_validator_with_args(&fixture.proof_path, &["--diagnostic"]);

    assert_success(output, "Gate 2 diagnostic proof is valid");
}

#[test]
fn gate2_outside_validator_rejects_missing_ffprobe_for_completed_proof() {
    let fixture = ValidatorFixture::new();

    let output = run_validator_without_ffprobe(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording validation requires ffprobe on PATH",
    );
}

#[test]
fn gate2_outside_generator_builds_valid_completed_proof() {
    let fixture = ValidatorFixture::new();
    let generated = fixture.dir.path().join("generated-outside-proof.md");

    let output = run_generator(&fixture.stopwatch_path, &generated);

    assert_success(output, "Wrote Gate 2 outside-person proof");
    assert_success(run_validator(&generated), DRAFT_VALID);
    let generated_text = fs::read_to_string(&generated)
        .unwrap_or_else(|err| panic!("read {}: {err}", generated.display()));
    assert!(generated_text.contains("- Name: Validator Fixture Runner"));
    assert!(generated_text.contains(OUTSIDE_RUN_ATTESTATION));
    assert!(generated_text.contains("- API endpoint: http://127.0.0.1:8080"));
    assert!(generated_text.contains("- Dashboard base: http://127.0.0.1:3000"));
    assert!(generated_text.contains("- Quickstart snippet: examples/python/five_line_otel.py"));
    assert!(generated_text.contains("- Timing start source: external-clone"));
    assert!(generated_text.contains("- Clone started at: 2026-06-20T11:59:55Z"));
    assert!(generated_text.contains("- Script-to-first-trace: 7s"));
    assert!(generated_text.contains("- Clone URL: https://github.com/jadenfix/beater.git"));
    assert!(generated_text.contains("- Branch: main"));
    assert!(generated_text.contains("- Worktree clean: yes"));
    assert!(generated_text.contains(&format!(
        "- Beater image reference: ghcr.io/jadenfix/beater/beaterd:{}",
        current_head()
    )));
    assert!(generated_text.contains("- Beater image digest: ghcr.io/jadenfix/beater/beaterd@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
    assert!(generated_text.contains("- Dashboard e2e image digest: ghcr.io/jadenfix/beater/dashboard-e2e@sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"));
    assert!(generated_text.contains("- OTEL Python image digest: ghcr.io/jadenfix/beater/otel-python@sha256:abababababababababababababababababababababababababababababababab"));
    assert!(generated_text.contains(r#"bash -lc 't="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'"#));
    assert!(!generated_text.contains(r#"BEATER_GATE2_CLONE_STARTED_EPOCH="$(date +%s)""#));
    assert!(generated_text.contains("- Outside-run wrapper: yes"));
    assert!(generated_text.contains("Gate 2 compose stopwatch passed; Browser recording: passed"));
    assert!(generated_text.contains(&format!(
        "Quickstart dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={QUICKSTART_TRACE}"
    )));
    assert!(generated_text.contains(&format!(
        "All-kind dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={ALL_KIND_TRACE}"
    )));
    assert!(generated_text.contains("- `docker compose images` excerpt:"));
    assert!(generated_text.contains(&format!("- Runner llm.call observation: {LLM_OBSERVATION}")));
    assert!(generated_text.contains(&format!(
        "- Runner waterfall observation: {WATERFALL_OBSERVATION}"
    )));
    assert!(generated_text.contains("ghcr.io/jadenfix/beater/beaterd"));
    assert!(generated_text.contains("ghcr.io/jadenfix/beater/dashboard"));
    assert!(generated_text.contains("beater-stopwatch-dashboard-e2e-run-1"));
    assert!(generated_text.contains("beater-stopwatch-otel-python-quickstart-run-1"));
    assert!(generated_text.contains(
        "- [x] The runner completed the flow using only public repository instructions."
    ));
}

#[test]
fn gate2_outside_generator_does_not_write_invalid_completed_proof() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("invalid-generated-outside-proof.md");
    replace(&fixture.stopwatch_path, "127.0.0.1:3000", "127.0.0.1:13080");

    let output = run_generator(&fixture.stopwatch_path, &generated);

    assert_failure(
        output,
        "stopwatch proof must not use alternate/warm-loop evidence",
    );
    assert!(
        !generated.exists(),
        "generator must not leave an invalid completed proof at {}",
        generated.display()
    );
}

#[test]
fn gate2_outside_generator_requires_explicit_attestation() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("missing-attestation-generated-proof.md");

    let output = run_generator_with_attestation(&fixture.stopwatch_path, &generated, false);

    assert_failure(output, "--attest-outside-run is required");
    assert!(
        !generated.exists(),
        "generator must not write completed proof without explicit attestation"
    );
}

#[test]
fn gate2_outside_generator_requires_network_notes() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("missing-network-notes-generated-proof.md");

    let output = run_generator_without_network_notes(&fixture.stopwatch_path, &generated);

    assert_failure(output, "--network-notes");
    assert!(
        !generated.exists(),
        "generator must not write completed proof without network notes"
    );
}

#[test]
fn gate2_outside_generator_requires_runner_observations() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("missing-observations-generated-proof.md");

    let output = run_generator_without_observations(&fixture.stopwatch_path, &generated);

    assert_failure(output, "--llm-observation");
    assert!(
        !generated.exists(),
        "generator must not write completed proof without runner observations"
    );
}

#[test]
fn gate2_outside_generator_rejects_placeholder_runner_identity() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("placeholder-runner-generated-proof.md");

    let output = run_generator_with_runner_name(&fixture.stopwatch_path, &generated, "...");

    assert_failure(
        output,
        "--runner-name must be provided with a concrete value",
    );
    assert!(
        !generated.exists(),
        "generator must not write completed proof with placeholder runner identity"
    );
}

#[test]
fn gate2_outside_generator_rejects_invalid_date() {
    let fixture = ValidatorFixture::new();
    let generated = fixture.dir.path().join("invalid-date-generated-proof.md");

    let output = run_generator_with_date(&fixture.stopwatch_path, &generated, "06/20/2026");

    assert_failure(output, "--date must be YYYY-MM-DD");
    assert!(
        !generated.exists(),
        "generator must not write completed proof with a non-ISO date"
    );
}

#[test]
fn gate2_outside_generator_rejects_date_not_matching_clone_start() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("mismatched-date-generated-proof.md");

    let output = run_generator_with_date(&fixture.stopwatch_path, &generated, "2026-06-21");

    assert_failure(
        output,
        "--date must match Clone started at UTC date 2026-06-20",
    );
    assert!(
        !generated.exists(),
        "generator must not write completed proof with a mismatched run date"
    );
}

#[test]
fn gate2_outside_generator_accepts_prior_exposure_none_and_defaults_run_date() {
    let fixture = ValidatorFixture::new();
    let generated = fixture.dir.path().join("prior-none-generated-proof.md");

    let output = run_generator_with_prior_exposure(&fixture.stopwatch_path, &generated, "none");

    assert_success(output, "Wrote Gate 2 outside-person proof");
    assert_success(run_validator(&generated), DRAFT_VALID);
    let generated_text = fs::read_to_string(&generated)
        .unwrap_or_else(|err| panic!("read {}: {err}", generated.display()));
    assert!(generated_text.contains("- Prior Beater repo exposure: none"));
    assert!(generated_text.contains("- Date: 2026-06-20"));
}

#[test]
fn gate2_outside_generator_rejects_placeholder_source_field() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("placeholder-source-generated-proof.md");
    replace(
        &fixture.stopwatch_path,
        "- Docker: `Docker version 29.2.0`",
        "- Docker: `Docker TBD version`",
    );

    let output = run_generator(&fixture.stopwatch_path, &generated);

    assert_failure(output, "Docker contains placeholder text");
    assert!(
        !generated.exists(),
        "generator must not write completed proof from placeholder stopwatch evidence"
    );
}

#[test]
fn gate2_outside_generator_rejects_duplicate_source_field_without_writing() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("duplicate-source-field-generated-proof.md");
    append(
        &fixture.stopwatch_path,
        "\n- Docker: `Docker version hidden-conflict`\n",
    );

    let output = run_generator(&fixture.stopwatch_path, &generated);

    assert_failure(output, "duplicate field in");
    assert!(
        !generated.exists(),
        "generator must not write proof from duplicate source fields"
    );
}

#[test]
fn gate2_outside_generator_reports_validation_failure_without_traceback() {
    let fixture = ValidatorFixture::new();
    let generated = fixture
        .dir
        .path()
        .join("validation-failed-generated-proof.md");

    let output = run_generator_without_fake_ffprobe(&fixture.stopwatch_path, &generated);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Traceback"),
        "generator must report validator failure without a Python traceback:\n{stderr}"
    );
    assert_failure(
        output,
        "screen recording validation requires ffprobe on PATH",
    );
    assert!(
        !generated.exists(),
        "generator must not write proof when post-generation validation fails"
    );
}

#[test]
fn gate2_outside_generator_refuses_non_pending_output_with_pending_phrase() {
    let fixture = ValidatorFixture::new();
    let existing = fixture.dir.path().join("existing-outside-proof.md");
    let existing_text = "# Gate 2 Outside-Person Proof\n\nStatus: completed.\n\nRunner note mentions Status: not yet completed.\n";
    fs::write(&existing, existing_text)
        .unwrap_or_else(|err| panic!("write {}: {err}", existing.display()));

    let output = run_generator(&fixture.stopwatch_path, &existing);

    assert_failure(output, "already exists and is not the pending template");
    let after = fs::read_to_string(&existing)
        .unwrap_or_else(|err| panic!("read {}: {err}", existing.display()));
    assert_eq!(
        after, existing_text,
        "generator must not overwrite non-pending proof"
    );
}

#[test]
fn gate2_outside_generator_overwrites_pending_template() {
    let fixture = ValidatorFixture::new();
    let pending = fixture.dir.path().join("pending-outside-proof.md");
    let source = fs::read_to_string(repo_root().join("docs/demos/gate2-outside-person-proof.md"))
        .unwrap_or_else(|err| panic!("read pending proof template: {err}"));
    fs::write(&pending, source).unwrap_or_else(|err| panic!("write {}: {err}", pending.display()));

    let output = run_generator(&fixture.stopwatch_path, &pending);

    assert_success(output, "Wrote Gate 2 outside-person proof");
    assert_success(run_validator(&pending), DRAFT_VALID);
}

#[test]
fn gate2_outside_readiness_accepts_fixture_registry_manifests() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());

    let output = run_readiness_with_fixture(registry.path());

    assert_success(output, "Gate 2 outside-run readiness passed");
}

#[test]
fn gate2_outside_readiness_rejects_missing_image_platform() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    write_registry_manifest(registry.path(), "dashboard", &["linux/amd64"]);

    let output = run_readiness_with_fixture(registry.path());

    assert_failure(output, "platforms mismatch for dashboard");
}

#[test]
fn gate2_outside_readiness_reports_missing_origin_without_traceback() {
    let root = repo_root();
    let fixture = tempdir("create readiness fixture without origin");
    copy_fixture_file(
        &root,
        fixture.path(),
        "scripts/check-gate2-outside-readiness.py",
    );
    git_success(fixture.path(), &["init"]);
    git_success(
        fixture.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git_success(fixture.path(), &["config", "user.name", "Gate 2 Fixture"]);
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "fixture"]);
    git_success(fixture.path(), &["branch", "-M", "main"]);

    let output = Command::new("python3")
        .arg(
            fixture
                .path()
                .join("scripts/check-gate2-outside-readiness.py"),
        )
        .current_dir(fixture.path())
        .output()
        .unwrap_or_else(|err| panic!("run readiness checker without origin: {err}"));

    assert_failure(output, "git remote get-url origin failed");
}

#[test]
fn gate2_public_handoff_verifier_accepts_clean_clone_fixture() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_with_fixture(&source_url, &fixture_head, registry.path());

    assert_success(output, "Gate 2 public handoff clone passed");
}

#[test]
fn gate2_public_handoff_verifier_rejects_missing_ffprobe_from_cloned_wrapper() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());
    let path_dir = path_without_ffprobe(
        "create public handoff PATH without ffprobe",
        &["python3", "git", "bash", "dirname"],
    );
    let mut command =
        run_public_handoff_with_fixture_command(&source_url, &fixture_head, registry.path());
    command.env("PATH", path_with_isolated_tempdir(&path_dir));

    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 public handoff without ffprobe: {err}"));

    assert!(
        !output.status.success(),
        "public handoff succeeded without ffprobe\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.contains("Gate 2 outside-run preflight failed"),
        "expected public handoff failure to expose wrapper preflight failure, got:\n{output}"
    );
    assert!(
        output.contains("missing required command 'ffprobe'"),
        "expected public handoff failure to expose missing ffprobe, got:\n{output}"
    );
}

#[test]
fn gate2_public_handoff_verifier_rejects_missing_quickstart_timing_guard() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    let readme_path = fixture_repo.path().join("README.md");
    replace(
        &readme_path,
        "not wait for the script to finish",
        "wait for the script to finish",
    );
    git_success(fixture_repo.path(), &["add", "README.md"]);
    git_success(
        fixture_repo.path(),
        &["commit", "-m", "break quickstart timing guard"],
    );
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_with_fixture(&source_url, &fixture_head, registry.path());

    assert_failure(
        output,
        "README.md must contain timing guard for outside runners",
    );
}

#[test]
fn gate2_public_handoff_verifier_clears_post_slo_timeout_override() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_with_fixture_env(
        &source_url,
        &fixture_head,
        registry.path(),
        ("BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS", "1"),
    );

    assert_success(output, "Gate 2 public handoff clone passed");
}

#[test]
fn gate2_public_handoff_verifier_ignores_alternate_outside_proof_env() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    let proof_path = fixture_repo
        .path()
        .join("docs/demos/gate2-outside-person-proof.md");
    let broken = fs::read_to_string(&proof_path)
        .unwrap_or_else(|err| panic!("read fixture outside proof: {err}"))
        .replace("- `docker compose images` excerpt:\n", "");
    fs::write(&proof_path, broken)
        .unwrap_or_else(|err| panic!("write broken fixture outside proof: {err}"));
    git_success(
        fixture_repo.path(),
        &["add", "docs/demos/gate2-outside-person-proof.md"],
    );
    git_success(
        fixture_repo.path(),
        &["commit", "-m", "break outside proof template"],
    );
    let alternate_proof_dir = tempdir("create alternate proof env dir");
    let alternate_proof = alternate_proof_dir
        .path()
        .join("alternate-outside-proof.md");
    fs::copy(
        repo_root().join("docs/demos/gate2-outside-person-proof.md"),
        &alternate_proof,
    )
    .unwrap_or_else(|err| panic!("copy alternate outside proof: {err}"));
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_with_fixture_env(
        &source_url,
        &fixture_head,
        registry.path(),
        (
            "BEATER_GATE2_OUTSIDE_PROOF",
            alternate_proof
                .to_str()
                .unwrap_or_else(|| panic!("alternate proof path must be UTF-8")),
        ),
    );

    assert_failure(
        output,
        "missing field in pending outside-person proof template: `docker compose images` excerpt",
    );
}

#[test]
fn gate2_public_handoff_verifier_rejects_invalid_stopwatch_shell() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    fs::write(
        fixture_repo
            .path()
            .join("scripts/gate2-compose-stopwatch.sh"),
        "#!/usr/bin/env bash\nif true; then\n",
    )
    .unwrap_or_else(|err| panic!("write invalid stopwatch fixture: {err}"));
    git_success(
        fixture_repo.path(),
        &["add", "scripts/gate2-compose-stopwatch.sh"],
    );
    git_success(
        fixture_repo.path(),
        &["commit", "-m", "break stopwatch syntax"],
    );
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_with_fixture(&source_url, &fixture_head, registry.path());

    assert_failure(output, "scripts/gate2-compose-stopwatch.sh");
}

#[test]
fn gate2_public_handoff_verifier_rejects_invalid_smoke_shell() {
    let registry = tempdir("create registry fixture dir");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    fs::write(
        fixture_repo.path().join("scripts/smoke-compose.sh"),
        "#!/usr/bin/env bash\nif true; then\n",
    )
    .unwrap_or_else(|err| panic!("write invalid smoke fixture: {err}"));
    git_success(fixture_repo.path(), &["add", "scripts/smoke-compose.sh"]);
    git_success(fixture_repo.path(), &["commit", "-m", "break smoke syntax"]);
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_with_fixture(&source_url, &fixture_head, registry.path());

    assert_failure(output, "scripts/smoke-compose.sh");
}

#[test]
fn gate2_public_handoff_verifier_full_run_rejects_noncanonical_fixture_source() {
    let registry = tempdir("create registry fixture dir");
    let clone_parent = tempdir("create public handoff clone parent");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let source_url = format!("file://{}", fixture_repo.path().display());

    let output = run_public_handoff_full_run_with_fixture(
        &source_url,
        &fixture_head,
        registry.path(),
        clone_parent.path(),
    );

    assert_failure(
        output,
        "--full-run executes the exact scripts/gate2-outside-run.sh path",
    );
    assert!(
        !clone_parent.path().join("beater").exists(),
        "noncanonical --full-run rejection must happen before creating a clone"
    );
}

#[test]
fn gate2_public_handoff_verifier_full_run_rejects_registry_fixture_without_fixture_flag() {
    let registry = tempdir("create registry fixture dir");
    let clone_parent = tempdir("create public handoff clone parent");

    let output = run_public_handoff_full_run_with_fixture(
        "https://github.com/jadenfix/beater.git",
        &current_head(),
        registry.path(),
        clone_parent.path(),
    );

    assert_failure(
        output,
        "--full-run verifies canonical public GHCR images and does not support --registry-fixture",
    );
    assert!(
        !clone_parent.path().join("beater").exists(),
        "canonical --full-run must reject registry fixtures before creating a clone"
    );
}

#[test]
fn gate2_public_handoff_verifier_full_run_accepts_rewritten_canonical_fixture() {
    let registry = tempdir("create registry fixture dir");
    let clone_parent = tempdir("create public handoff clone parent");
    write_registry_fixtures(registry.path());
    let fixture_repo = write_public_handoff_fixture_repo();
    write_stopwatch_env_stub(fixture_repo.path());
    git_success(
        fixture_repo.path(),
        &["add", "scripts/gate2-compose-stopwatch.sh"],
    );
    git_success(
        fixture_repo.path(),
        &["commit", "-m", "stub stopwatch runtime"],
    );
    let fixture_head = git_output(fixture_repo.path(), &["rev-parse", "HEAD"]);
    let runtime = fake_public_handoff_runtime(true, "unix:///var/run/docker.sock");
    let fixture_url = format!("file://{}", fixture_repo.path().display());
    let git_rewrite_key = format!("url.{fixture_url}.insteadOf");
    let root = repo_root();
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/check-gate2-public-handoff.py"))
        .arg("--skip-local-readiness")
        .arg("--expected-commit")
        .arg(&fixture_head)
        .arg("--registry-fixture")
        .arg(registry.path())
        .arg("--clone-parent")
        .arg(clone_parent.path())
        .arg("--full-run")
        .current_dir(&root)
        .env("PATH", path_with_public_handoff_runtime(&runtime))
        .env_remove("DOCKER_HOST")
        .env("BEATER_GATE2_FIXTURE_FULL_RUN", "1")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", git_rewrite_key)
        .env(
            "GIT_CONFIG_VALUE_0",
            "https://github.com/jadenfix/beater.git",
        );

    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 public handoff full-run fixture: {err}"));

    if !output.status.success() {
        panic!(
            "Gate 2 public handoff full-run fixture failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Gate 2 public handoff fixture full run passed"),
        "stdout did not contain full-run pass line\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Gate 2 generated proof diagnostic passed"),
        "stdout did not contain generated-proof diagnostic pass line\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Gate 2 diagnostic proof is valid"),
        "stdout did not contain diagnostic validator pass line\nstdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Auto-confirming the manual quickstart checkpoint"),
        "full-run must disclose diagnostic-only manual checkpoint auto-confirmation\nstdout:\n{stdout}"
    );
    let checks_clone_dir = clone_parent.path().join("beater-checks");
    assert!(
        checks_clone_dir.exists(),
        "full-run verifier must use a separate static-check clone"
    );
    assert!(
        !checks_clone_dir
            .join("docs/demos/wrapper-real-env.txt")
            .exists(),
        "static-check clone must not execute the timed outside wrapper"
    );
    let clone_dir = clone_parent.path().join("beater");
    let clone_origin = git_output(&clone_dir, &["remote", "get-url", "origin"]);
    assert_eq!(clone_origin, "https://github.com/jadenfix/beater.git");
    let env_marker = fs::read_to_string(clone_dir.join("docs/demos/wrapper-real-env.txt"))
        .unwrap_or_else(|err| panic!("read cloned wrapper runtime marker: {err}"));
    assert!(env_marker.contains("manual_checkpoint_confirmed=yes"));
    assert!(env_marker.contains("write=1"));
    assert!(env_marker.contains("browser=1"));
    assert!(env_marker.contains("record=1"));
    assert!(env_marker.contains("post_slo=unset"));
    assert!(env_marker.contains("reuse=0"));
    assert!(env_marker.contains("local_build=0"));
    assert!(env_marker.contains("pull_policy=always"));
    assert!(env_marker.contains("keep=1"));
    assert!(env_marker.contains("outside_wrapper=1"));
    assert!(env_marker.contains("dry=unset"));
    assert!(env_marker.contains("expected_origin=unset"));
    assert!(env_marker.contains("dashboard_port=unset"));
    assert!(env_marker.contains("fixture_full_run=unset"));
    assert!(env_marker.contains("git_config_count=unset"));
    assert!(
        env_marker.contains("clone_started=") && !env_marker.contains("clone_started=unset"),
        "full-run fixture must pass clone-start timing into the wrapper\n{env_marker}"
    );
    let diagnostic_proof =
        fs::read_to_string(clone_dir.join("docs/demos/gate2-public-handoff-diagnostic-proof.md"))
            .unwrap_or_else(|err| panic!("read generated diagnostic proof: {err}"));
    assert!(diagnostic_proof.contains("Status: diagnostic."));
    assert!(diagnostic_proof.contains("Public Handoff Diagnostic"));
    assert!(diagnostic_proof.contains("not outside-person evidence"));
    assert!(diagnostic_proof.contains("diagnostic auto-confirmed the manual checkpoint"));
    assert!(diagnostic_proof.contains(DIAGNOSTIC_ATTESTATION));
    assert!(diagnostic_proof.contains("token breakdown"));
    let docker_log = fs::read_to_string(&runtime.docker_log)
        .unwrap_or_else(|err| panic!("read fake Docker log: {err}"));
    assert!(docker_log.contains("info"));
    assert!(docker_log.contains("compose version"));
    assert!(docker_log.contains("context inspect"));
    assert_eq!(
        docker_log.matches("down -v --remove-orphans").count(),
        2,
        "full-run fixture should clean Compose before and after the wrapper\n{docker_log}"
    );
}

#[test]
fn gate2_public_handoff_full_run_rejects_missing_sha_tooling_before_clone() {
    let clone_parent = tempdir("create public handoff clone parent");
    let runtime = fake_public_handoff_runtime(false, "unix:///var/run/docker.sock");
    let mut command = public_handoff_full_run_preflight_command(clone_parent.path());
    command.env("PATH", &runtime.path_env);

    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run full-run missing-SHA preflight: {err}"));

    assert_failure(output, "shasum or sha256sum");
    assert!(
        !clone_parent.path().join("beater").exists(),
        "missing SHA tooling must fail before creating a clone"
    );
    assert!(
        !runtime.docker_log.exists(),
        "missing SHA tooling must fail before invoking Docker"
    );
}

#[test]
fn gate2_public_handoff_full_run_rejects_remote_docker_host_before_clone() {
    let clone_parent = tempdir("create public handoff clone parent");
    let runtime = fake_public_handoff_runtime(true, "unix:///var/run/docker.sock");
    let mut command = public_handoff_full_run_preflight_command(clone_parent.path());
    command
        .env("PATH", &runtime.path_env)
        .env("DOCKER_HOST", "ssh://builder.example.invalid");

    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run full-run remote DOCKER_HOST preflight: {err}"));

    assert_failure(output, "requires a local Docker daemon");
    assert!(
        !clone_parent.path().join("beater").exists(),
        "remote DOCKER_HOST must fail before creating a clone"
    );
    assert!(
        !runtime.docker_log.exists(),
        "remote DOCKER_HOST must fail before invoking Docker"
    );
}

#[test]
fn gate2_public_handoff_full_run_rejects_remote_docker_context_before_cleanup() {
    let clone_parent = tempdir("create public handoff clone parent");
    let runtime = fake_public_handoff_runtime(true, "ssh://builder.example.invalid");
    let mut command = public_handoff_full_run_preflight_command(clone_parent.path());
    command.env("PATH", &runtime.path_env);

    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run full-run remote Docker context preflight: {err}"));

    assert_failure(output, "requires a local Docker context");
    assert!(
        !clone_parent.path().join("beater").exists(),
        "remote Docker context must fail before creating a clone"
    );
    let docker_log = fs::read_to_string(&runtime.docker_log)
        .unwrap_or_else(|err| panic!("read fake Docker log: {err}"));
    assert!(docker_log.contains("info"));
    assert!(docker_log.contains("compose version"));
    assert!(docker_log.contains("context inspect"));
    assert!(
        !docker_log.contains("down -v --remove-orphans"),
        "remote Docker context must fail before Compose cleanup"
    );
}

#[test]
fn gate2_public_handoff_full_run_has_local_runtime_preflight_contract() {
    let script = fs::read_to_string(repo_root().join("scripts/check-gate2-public-handoff.py"))
        .unwrap_or_else(|err| panic!("read Gate 2 public handoff verifier: {err}"));

    assert!(script.contains("preflight_full_run_runtime"));
    assert!(script.contains("require_full_run_source(args)"));
    assert!(script.contains("shutil.which"));
    assert!(script.contains("\"ffprobe\""));
    assert!(script.contains("socket.create_connection"));
    assert!(script.contains("def port_owner_hint"));
    assert!(script.contains("def process_owner_details"));
    assert!(script.contains("def process_command"));
    assert!(script.contains("def process_cwd"));
    assert!(script.contains("lsof"));
    assert!(script.contains("\"ps\", \"-p\", pid, \"-o\", \"command=\""));
    assert!(script.contains("\"lsof\", \"-a\", \"-p\", pid, \"-d\", \"cwd\", \"-Fn\""));
    assert!(script.contains("process {pid} command"));
    assert!(script.contains("process {pid} cwd"));
    assert!(script.contains("install lsof or ss to identify the process holding TCP"));
    assert!(script.contains("(8080, \"beaterd HTTP\", \"BEATER_HTTP_PORT\")"));
    assert!(script.contains("(4317, \"OTLP gRPC\", \"BEATER_OTLP_GRPC_PORT\")"));
    assert!(script.contains("(3000, \"dashboard\", \"BEATER_DASHBOARD_PORT\")"));
    assert!(script.contains("run([\"docker\", \"info\"]"));
    assert!(script.contains("run([\"docker\", \"compose\", \"version\"]"));
    assert!(script.contains("shasum or sha256sum"));
    assert!(script.contains("DOCKER_HOST"));
    assert!(script.contains("docker_endpoint_is_local"));
    assert!(script.contains("require_local_docker_host_env"));
    assert!(script.contains("require_local_docker_context"));
    assert!(script.contains("[\"docker\", \"context\", \"inspect\""));
    assert!(script.contains("requires a local Docker daemon"));
    assert!(script.contains("requires a local Docker context"));
    assert!(script.contains("STOPWATCH_COMPOSE_DOWN"));
    assert!(script.contains("def cleanup_stopwatch_compose"));
    assert!(script.contains("cleanup_stopwatch_compose(repo_root(), fatal=True)"));
    assert!(script.contains("cleanup_stopwatch_compose(clone_dir, fatal=False)"));
    assert!(script.contains("cleanup_local_stopwatch_compose"));
    assert!(script.contains("free it rather than setting"));
    assert!(script.contains("cleaning the beater-stopwatch Compose project"));
    assert!(script.contains("preflight_full_run_runtime(args)"));
    assert!(script.contains("\"beater-checks\" if args.full_run else \"beater\""));
    assert!(script.contains("full_clone_dir, full_temp_owner, full_clone_started_epoch"));
    assert!(script.contains("expected_commit, \"beater\""));
}

#[test]
fn gate2_public_handoff_port_owner_hint_reports_command_and_cwd() {
    let root = repo_root();
    let tools = tempdir("create fake port owner tools");
    write_executable(
        &tools.path().join("lsof"),
        r#"#!/bin/sh
for arg in "$@"; do
  if [ "$arg" = "-t" ]; then
    printf '43210\n43210\n'
    exit 0
  fi
  if [ "$arg" = "-Fn" ]; then
    printf 'p43210\nn/tmp/outside-dashboard\n'
    exit 0
  fi
done
printf 'COMMAND   PID USER FD TYPE DEVICE SIZE/OFF NODE NAME\n'
printf 'node    43210 user 13u IPv6 0x1 0t0 TCP *:3000 (LISTEN)\n'
"#,
    );
    write_executable(
        &tools.path().join("ps"),
        "#!/bin/sh\nprintf 'next-server (v14.2.15)\\n'\n",
    );

    let output = Command::new(python3_executable())
        .arg("-c")
        .arg(
            r#"
import importlib.util
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("handoff", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(module.occupied_port_message(3000, "dashboard", "BEATER_DASHBOARD_PORT"))
"#,
        )
        .arg(root.join("scripts/check-gate2-public-handoff.py"))
        .current_dir(&root)
        .env("PATH", path_with_isolated_tempdir(&tools))
        .output()
        .unwrap_or_else(|err| panic!("run port owner hint fixture: {err}"));

    if !output.status.success() {
        panic!(
            "port owner hint fixture failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("node    43210"),
        "port owner hint must include raw listener output\n{stdout}"
    );
    assert!(
        stdout.contains("process 43210 command: next-server (v14.2.15)"),
        "port owner hint must include process command\n{stdout}"
    );
    assert!(
        stdout.contains("process 43210 cwd: /tmp/outside-dashboard"),
        "port owner hint must include process cwd\n{stdout}"
    );
    assert!(
        stdout.contains("Stop the process or app listening on TCP 3000"),
        "port owner hint must include a non-destructive remediation hint\n{stdout}"
    );
    assert!(
        stdout.contains("do not set BEATER_DASHBOARD_PORT"),
        "port owner hint must preserve default-port evidence semantics\n{stdout}"
    );
    assert_eq!(
        stdout.matches("process 43210 command").count(),
        1,
        "port owner hint must dedupe listener pids\n{stdout}"
    );
}

#[test]
fn gate2_public_handoff_sanitizes_git_config_rewrites_for_public_clone() {
    let root = repo_root();
    let output = Command::new(python3_executable())
        .arg("-c")
        .arg(
            r#"
import argparse
import importlib.util
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("handoff", path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
args = argparse.Namespace(
    full_run=True,
    source_url=module.REMOTE_URL,
    registry_fixture="/tmp/registry",
    skip_local_readiness=False,
)
env = module.public_clone_env(args)
print("GIT_CONFIG_COUNT=" + env.get("GIT_CONFIG_COUNT", "unset"))
print("GIT_CONFIG_GLOBAL=" + env.get("GIT_CONFIG_GLOBAL", "unset"))
print("GIT_CONFIG_NOSYSTEM=" + env.get("GIT_CONFIG_NOSYSTEM", "unset"))
args.skip_local_readiness = True
module.os.environ["BEATER_GATE2_FIXTURE_FULL_RUN"] = "1"
fixture_env = module.public_clone_env(args)
print("fixture_GIT_CONFIG_COUNT=" + fixture_env.get("GIT_CONFIG_COUNT", "unset"))
"#,
        )
        .arg(root.join("scripts/check-gate2-public-handoff.py"))
        .current_dir(&root)
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "url.file:///tmp/fake.insteadOf")
        .env(
            "GIT_CONFIG_VALUE_0",
            "https://github.com/jadenfix/beater.git",
        )
        .output()
        .unwrap_or_else(|err| panic!("run public clone env fixture: {err}"));

    if !output.status.success() {
        panic!(
            "public clone env fixture failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GIT_CONFIG_COUNT=unset"));
    assert!(stdout.contains("GIT_CONFIG_GLOBAL=/dev/null"));
    assert!(stdout.contains("GIT_CONFIG_NOSYSTEM=1"));
    assert!(stdout.contains("fixture_GIT_CONFIG_COUNT=1"));
}

#[test]
fn gate2_stopwatch_service_image_digest_selects_matching_service_repo_digest() {
    let repo_digests = format!(
        "{BEATER_IMAGE_DIGEST}\n{DASHBOARD_IMAGE_DIGEST}\n{}",
        "ghcr.io/example/unrelated@sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
    );

    let output = run_service_image_digest_fixture("dashboard", &repo_digests);

    assert_eq!(output, DASHBOARD_IMAGE_DIGEST);
}

#[test]
fn gate2_stopwatch_service_image_digest_rejects_wrong_service_repo_digest() {
    let output = run_service_image_digest_fixture("dashboard", BEATER_IMAGE_DIGEST);

    assert_eq!(output, "unknown");
}

#[test]
fn gate2_stopwatch_outside_next_steps_separate_dashboard_targets() {
    let script = fs::read_to_string(repo_root().join("scripts/gate2-compose-stopwatch.sh"))
        .unwrap_or_else(|err| panic!("read Gate 2 compose stopwatch script: {err}"));

    assert!(script.contains("Open the quickstart URL above in a normal browser now"));
    assert!(script.contains("do not wait for the script to finish"));
    assert!(script.contains("Gate 2 recording proof requires ffprobe before the stopwatch starts."));
    assert!(script.contains("If another app is listed below, stop that app before rerunning"));
    assert!(script.contains("do not set\n$env_name for outside-person evidence"));
    assert!(script.contains(
        "Confirm prompt, completion, model, token breakdown, cost, and latency are visible."
    ));
    assert!(script.contains(
        "If you have not already done so, open $dashboard_url in a normal browser for the quickstart trace."
    ));
    assert!(script.contains(
        "Open ${all_kind_dashboard_url:-not requested} in a normal browser for the all-kind waterfall."
    ));
    assert!(script.contains("Confirm run -> turn -> step -> tool -> MCP nesting is visible."));
    assert!(script.contains(
        "Maintainer diagnostic overrides are intentionally suppressed for outside-person evidence."
    ));
    assert!(script.contains("if [[ \"$outside_wrapper\" == \"1\" ]]; then"));
}

#[test]
fn gate2_outside_wrapper_real_run_executes_stopwatch_with_clone_timer() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    write_stopwatch_env_stub(fixture.path());
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "add stopwatch stub"]);

    let output = run_outside_wrapper_real_with_clone_timer_in_repo(fixture.path(), "1800000000");

    assert_success(output, "fixture outside wrapper runtime executed");
    let env_marker = fs::read_to_string(fixture.path().join("docs/demos/wrapper-real-env.txt"))
        .unwrap_or_else(|err| panic!("read outside wrapper runtime marker: {err}"));
    assert!(env_marker.contains("write=1"));
    assert!(env_marker.contains("browser=1"));
    assert!(env_marker.contains("record=1"));
    assert!(env_marker.contains("post_slo=unset"));
    assert!(env_marker.contains("reuse=0"));
    assert!(env_marker.contains("local_build=0"));
    assert!(env_marker.contains("pull_policy=always"));
    assert!(env_marker.contains("keep=1"));
    assert!(env_marker.contains("outside_wrapper=1"));
    assert!(env_marker.contains("dry=unset"));
    assert!(env_marker.contains("expected_origin=unset"));
    assert!(env_marker.contains("clone_started=1800000000"));
    assert!(env_marker.contains("dashboard_port=unset"));
}

#[test]
fn gate2_outside_wrapper_accepts_default_dry_run() {
    let output = run_outside_wrapper_dry_run(None);

    assert_success(output, "Gate 2 outside-run wrapper preflight passed");
}

#[test]
fn gate2_outside_wrapper_dry_run_rejects_missing_python3() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    let path_dir = tempdir("create outside wrapper dry-run PATH without python3");
    symlink(&command_executable("git"), path_dir.path().join("git"))
        .unwrap_or_else(|err| panic!("symlink git fixture: {err}"));
    symlink(
        &command_executable("dirname"),
        path_dir.path().join("dirname"),
    )
    .unwrap_or_else(|err| panic!("symlink dirname fixture: {err}"));

    let mut command = Command::new("/bin/bash");
    command
        .arg(fixture.path().join("scripts/gate2-outside-run.sh"))
        .current_dir(fixture.path());
    clear_outside_env(&mut command);
    command
        .env("PATH", path_dir.path())
        .env("BEATER_GATE2_OUTSIDE_RUN_DRY_RUN", "1");
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper dry-run without python3: {err}"));

    assert_failure(output, "missing required command 'python3'");
}

#[test]
fn gate2_outside_wrapper_dry_run_rejects_unusable_python3() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    let path_dir = tempdir("create outside wrapper dry-run PATH with unusable python3");
    symlink(&command_executable("git"), path_dir.path().join("git"))
        .unwrap_or_else(|err| panic!("symlink git fixture: {err}"));
    symlink(
        &command_executable("dirname"),
        path_dir.path().join("dirname"),
    )
    .unwrap_or_else(|err| panic!("symlink dirname fixture: {err}"));
    write_executable(
        &path_dir.path().join("python3"),
        "#!/bin/sh\nif [ \"$1\" = \"-c\" ]; then printf '3.8.18\\n'; exit 0; fi\nexit 1\n",
    );

    let mut command = Command::new("/bin/bash");
    command
        .arg(fixture.path().join("scripts/gate2-outside-run.sh"))
        .current_dir(fixture.path());
    clear_outside_env(&mut command);
    command
        .env("PATH", path_dir.path())
        .env("BEATER_GATE2_OUTSIDE_RUN_DRY_RUN", "1");
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper dry-run with bad python3: {err}"));

    assert_failure(
        output,
        "python3 must be version 3.9 or newer for proof generation and validation; got '3.8.18'",
    );
}

#[test]
fn gate2_outside_wrapper_dry_run_rejects_missing_ffprobe() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    let path_dir = tempdir("create outside wrapper dry-run PATH without ffprobe");
    for name in ["git", "dirname", "python3"] {
        symlink(&command_executable(name), path_dir.path().join(name))
            .unwrap_or_else(|err| panic!("symlink {name} fixture: {err}"));
    }

    let mut command = Command::new("/bin/bash");
    command
        .arg(fixture.path().join("scripts/gate2-outside-run.sh"))
        .current_dir(fixture.path());
    clear_outside_env(&mut command);
    command
        .env("PATH", path_dir.path())
        .env("BEATER_GATE2_OUTSIDE_RUN_DRY_RUN", "1");
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper dry-run without ffprobe: {err}"));

    assert_failure(output, "missing required command 'ffprobe'");
}

#[test]
fn gate2_outside_wrapper_rejects_alternate_dashboard_port() {
    let output = run_outside_wrapper_dry_run(Some(("BEATER_DASHBOARD_PORT", "13080")));

    assert_failure(output, "BEATER_DASHBOARD_PORT must be unset or '3000'");
}

#[test]
fn gate2_outside_wrapper_rejects_post_slo_timeout_override() {
    let output =
        run_outside_wrapper_dry_run(Some(("BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS", "120")));

    assert_failure(
        output,
        "BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS must be unset or '300'",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_image_override() {
    let output = run_outside_wrapper_dry_run(Some((
        "BEATERD_IMAGE",
        "ghcr.io/jadenfix/beater/beaterd:main",
    )));

    assert_failure(
        output,
        "BEATERD_IMAGE must be unset for outside-person evidence",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_registry_fixture_override() {
    let output = run_outside_wrapper_dry_run(Some((
        "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS",
        "/tmp/registry",
    )));

    assert_failure(
        output,
        "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS must be unset for outside-person evidence",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_artifact_path_override() {
    let output = run_outside_wrapper_dry_run(Some((
        "BEATER_GATE2_RECORD_VIDEO",
        "/tmp/gate2-recording.webm",
    )));

    assert_failure(
        output,
        "BEATER_GATE2_RECORD_VIDEO must be unset for outside-person evidence",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_compose_project_override() {
    let output = run_outside_wrapper_dry_run(Some(("COMPOSE_PROJECT_NAME", "beater-alt")));

    assert_failure(
        output,
        "COMPOSE_PROJECT_NAME must be unset for outside-person evidence",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_compose_cleanup_override() {
    let output = run_outside_wrapper_dry_run(Some(("KEEP_BEATER_COMPOSE", "0")));

    assert_failure(output, "KEEP_BEATER_COMPOSE must be unset or '1'");
}

#[test]
fn gate2_outside_wrapper_rejects_non_main_branch() {
    let fixture = write_outside_wrapper_fixture_repo("feature-proof");

    let output = run_outside_wrapper_dry_run_in_repo(fixture.path(), None);

    assert_failure(
        output,
        "outside-person evidence must run from the main branch; got 'feature-proof'",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_wrong_origin() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    git_success(
        fixture.path(),
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/jadenfix/beater-fork.git",
        ],
    );

    let output = run_outside_wrapper_dry_run_in_repo(fixture.path(), None);

    assert_failure(
        output,
        "outside-person evidence must run from origin 'https://github.com/jadenfix/beater.git'",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_dirty_worktree() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    fs::write(fixture.path().join("dirty.txt"), "dirty")
        .unwrap_or_else(|err| panic!("write dirty fixture file: {err}"));

    let output = run_outside_wrapper_dry_run_in_repo(fixture.path(), None);

    assert_failure(
        output,
        "outside-person evidence must run from a clean worktree",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_missing_clone_timer_for_real_run() {
    let fixture = write_outside_wrapper_fixture_repo("main");

    let output = run_outside_wrapper_real_preflight_in_repo(fixture.path());

    assert_failure(
        output,
        "BEATER_GATE2_CLONE_STARTED_EPOCH must be set before git clone",
    );
}

#[test]
fn gate2_outside_wrapper_rejects_missing_ffprobe_before_stopwatch() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    write_stopwatch_env_stub(fixture.path());
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "add stopwatch stub"]);
    let path_dir = tempdir("create outside wrapper PATH without ffprobe");
    for name in ["git", "dirname", "python3"] {
        symlink(&command_executable(name), path_dir.path().join(name))
            .unwrap_or_else(|err| panic!("symlink {name} fixture: {err}"));
    }

    let mut command = Command::new("/bin/bash");
    command
        .arg(fixture.path().join("scripts/gate2-outside-run.sh"))
        .current_dir(fixture.path());
    clear_outside_env(&mut command);
    command
        .env("PATH", path_dir.path())
        .env("BEATER_GATE2_CLONE_STARTED_EPOCH", "1800000000");
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper without ffprobe: {err}"));

    assert_failure(output, "missing required command 'ffprobe'");
    assert!(
        !fixture.path().join("wrapper-real-env.txt").exists(),
        "outside wrapper must fail before executing the stopwatch script"
    );
}

#[test]
fn gate2_outside_wrapper_rejects_missing_python3_before_stopwatch() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    write_stopwatch_env_stub(fixture.path());
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "add stopwatch stub"]);
    let path_dir = tempdir("create outside wrapper PATH without python3");
    symlink(&command_executable("git"), path_dir.path().join("git"))
        .unwrap_or_else(|err| panic!("symlink git fixture: {err}"));
    symlink(
        &command_executable("dirname"),
        path_dir.path().join("dirname"),
    )
    .unwrap_or_else(|err| panic!("symlink dirname fixture: {err}"));

    let mut command = Command::new("/bin/bash");
    command
        .arg(fixture.path().join("scripts/gate2-outside-run.sh"))
        .current_dir(fixture.path());
    clear_outside_env(&mut command);
    command
        .env("PATH", path_dir.path())
        .env("BEATER_GATE2_CLONE_STARTED_EPOCH", "1800000000");
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper without python3: {err}"));

    assert_failure(output, "missing required command 'python3'");
    assert!(
        !fixture.path().join("wrapper-real-env.txt").exists(),
        "outside wrapper must fail before executing the stopwatch script"
    );
}

#[test]
fn gate2_outside_validator_rejects_stopwatch_without_wrapper_marker() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Outside-run wrapper: yes",
        "- Outside-run wrapper: no",
    );
    replace(
        &fixture.stopwatch_path,
        "- Outside-run wrapper: yes",
        "- Outside-run wrapper: no",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Outside-run wrapper must be yes");
}

#[test]
fn gate2_outside_validator_rejects_local_stopwatch_footer_for_outside_evidence() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "outside-run stopwatch source artifact",
        "automated local stopwatch proof",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "stopwatch proof for outside-person evidence must identify itself as outside-run source evidence",
    );
}

#[test]
fn gate2_outside_validator_rejects_stopwatch_missing_source_artifact_marker() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "outside-run stopwatch source artifact",
        "outside-run stopwatch evidence artifact",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "stopwatch proof must identify itself as outside-run source evidence",
    );
}

#[test]
fn gate2_outside_validator_rejects_missing_stopwatch_proof() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &fixture.stopwatch_field,
        "docs/demos/missing-stopwatch.md",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "stopwatch proof file does not exist");
}

#[test]
fn gate2_outside_validator_rejects_wrong_quickstart_snippet() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Quickstart snippet: `examples/python/five_line_otel.py`",
        "- Quickstart snippet: `examples/python/custom_agent.py`",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Quickstart snippet in stopwatch proof must be 'examples/python/five_line_otel.py'",
    );
}

#[test]
fn gate2_outside_validator_rejects_missing_quickstart_snippet() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Quickstart snippet: `examples/python/five_line_otel.py`\n",
        "",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "missing field in stopwatch proof: Quickstart snippet",
    );
}

#[test]
fn gate2_outside_validator_rejects_mismatched_quickstart_release_id() {
    let fixture = ValidatorFixture::new();
    let head = current_head();
    replace(
        &fixture.stopwatch_path,
        &format!("- Quickstart release ID: `{}`", quickstart_release_id()),
        &format!(
            "- Quickstart release ID: `gate2-{}-1780000000-99999`",
            &head[..12]
        ),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "quickstart release ID mismatch between proof artifacts",
    );
}

#[test]
fn gate2_outside_validator_rejects_missing_quickstart_release_id() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!("- Quickstart release ID: {}\n", quickstart_release_id()),
        "",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "missing field in outside-person proof: Quickstart release ID",
    );
}

#[test]
fn gate2_outside_validator_rejects_split_clone_command() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        r#"bash -lc 't="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'"#,
        r#"BEATER_GATE2_CLONE_STARTED_EPOCH="$(date +%s)"
git clone https://github.com/jadenfix/beater.git && cd beater
BEATER_GATE2_CLONE_STARTED_EPOCH="$BEATER_GATE2_CLONE_STARTED_EPOCH" scripts/gate2-outside-run.sh"#,
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "missing fail-fast clone-to-browser command");
}

#[test]
fn gate2_outside_validator_rejects_untracked_recording_artifacts() {
    let fixture = ValidatorFixture::new();

    let output = run_validator_without_untracked_artifact_escape(&fixture.proof_path);

    assert_failure(
        output,
        "Screen recording must be tracked by git before Gate 2 closure",
    );
}

#[test]
fn gate2_outside_validator_rejects_missing_compose_images_excerpt() {
    let fixture = ValidatorFixture::new();
    replace(&fixture.proof_path, &compose_images_excerpt_line(), "");

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "missing field in outside-person proof: `docker compose images` excerpt",
    );
}

#[test]
fn gate2_outside_validator_rejects_duplicate_proof_field() {
    let fixture = ValidatorFixture::new();
    append(&fixture.proof_path, "\n- Branch: feature-hidden\n");

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "duplicate field in outside-person proof: Branch");
}

#[test]
fn gate2_outside_validator_rejects_duplicate_status_line() {
    let fixture = ValidatorFixture::new();
    append(&fixture.proof_path, "\nStatus: not yet completed.\n");

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "duplicate Status line");
}

#[test]
fn gate2_outside_validator_rejects_missing_network_notes() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Network notes: public docs only\n",
        "",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "missing field in outside-person proof: Network notes",
    );
}

#[test]
fn gate2_outside_validator_rejects_placeholder_network_notes() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Network notes: public docs only",
        "- Network notes: not reported",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "unresolved required fields: Network notes");
}

#[test]
fn gate2_outside_validator_rejects_placeholder_required_field_value() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Browser: Chromium",
        "- Browser: unknown",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "unresolved required fields: Browser");
}

#[test]
fn gate2_outside_validator_rejects_embedded_placeholder_required_field_value() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Machine and OS: macOS arm64",
        "- Machine and OS: Ubuntu TBD machine",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "unresolved required fields: Machine and OS");
}

#[test]
fn gate2_outside_validator_rejects_ellipsis_required_field_value() {
    let fixture = ValidatorFixture::new();
    replace(&fixture.proof_path, "- Browser: Chromium", "- Browser: ...");

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "unresolved required fields: Browser");
}

#[test]
fn gate2_outside_validator_rejects_embedded_placeholder_in_copied_source_field() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Docker version: Docker version 29.2.0",
        "- Docker version: Docker TBD version",
    );
    replace(
        &fixture.stopwatch_path,
        "- Docker: `Docker version 29.2.0`",
        "- Docker: `Docker TBD version`",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "unresolved required fields: Docker version");
}

#[test]
fn gate2_outside_validator_rejects_placeholder_compose_images_excerpt() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &compose_images_excerpt_line(),
        "- `docker compose images` excerpt: beaterd and dashboard images present\n",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "`docker compose images` excerpt must include ghcr.io/jadenfix/beater/beaterd",
    );
}

#[test]
fn gate2_outside_validator_accepts_compose_images_excerpt_from_all_gate2_services() {
    let fixture = ValidatorFixture::new();
    let commit_sha = current_head();
    replace(
        &fixture.proof_path,
        &compose_images_excerpt_line(),
        &format!(
            "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd {commit_sha} | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard {commit_sha} | beater-stopwatch-dashboard-e2e-run-1 ghcr.io/jadenfix/beater/dashboard-e2e {commit_sha} | beater-stopwatch-otel-python-quickstart-run-1 ghcr.io/jadenfix/beater/otel-python {commit_sha} | proof-image beaterd ghcr.io/jadenfix/beater/beaterd:{commit_sha} {BEATER_IMAGE_DIGEST} | proof-image dashboard ghcr.io/jadenfix/beater/dashboard:{commit_sha} {DASHBOARD_IMAGE_DIGEST} | proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha} {DASHBOARD_E2E_IMAGE_DIGEST} | proof-image otel-python ghcr.io/jadenfix/beater/otel-python:{commit_sha} {OTEL_PYTHON_IMAGE_DIGEST}\n"
        ),
    );

    let output = run_validator(&fixture.proof_path);

    assert_success(output, DRAFT_VALID);
}

#[test]
fn gate2_outside_validator_rejects_compose_images_without_proof_image_rows() {
    let fixture = ValidatorFixture::new();
    let commit_sha = current_head();
    replace(
        &fixture.proof_path,
        &compose_images_excerpt_line(),
        &format!(
            "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd {commit_sha} | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard {commit_sha} | beater-stopwatch-dashboard-e2e-run-1 ghcr.io/jadenfix/beater/dashboard-e2e {commit_sha} | beater-stopwatch-otel-python-quickstart-run-1 ghcr.io/jadenfix/beater/otel-python {commit_sha}\n"
        ),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "`docker compose images` excerpt must include proof-image row for ghcr.io/jadenfix/beater/beaterd",
    );
}

#[test]
fn gate2_outside_validator_rejects_compose_images_missing_dashboard_e2e() {
    let fixture = ValidatorFixture::new();
    let commit_sha = current_head();
    replace(
        &fixture.proof_path,
        &compose_images_excerpt_line(),
        &format!(
            "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd {commit_sha} | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard {commit_sha} | beater-stopwatch-otel-python-quickstart-run-1 ghcr.io/jadenfix/beater/otel-python {commit_sha} | proof-image beaterd ghcr.io/jadenfix/beater/beaterd:{commit_sha} {BEATER_IMAGE_DIGEST} | proof-image dashboard ghcr.io/jadenfix/beater/dashboard:{commit_sha} {DASHBOARD_IMAGE_DIGEST} | proof-image otel-python ghcr.io/jadenfix/beater/otel-python:{commit_sha} {OTEL_PYTHON_IMAGE_DIGEST}\n"
        ),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "`docker compose images` excerpt must include ghcr.io/jadenfix/beater/dashboard-e2e tagged with the checked-out commit SHA",
    );
}

#[test]
fn gate2_outside_validator_rejects_compose_images_missing_otel_python() {
    let fixture = ValidatorFixture::new();
    let commit_sha = current_head();
    replace(
        &fixture.proof_path,
        &compose_images_excerpt_line(),
        &format!(
            "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd {commit_sha} | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard {commit_sha} | beater-stopwatch-dashboard-e2e-run-1 ghcr.io/jadenfix/beater/dashboard-e2e {commit_sha} | proof-image beaterd ghcr.io/jadenfix/beater/beaterd:{commit_sha} {BEATER_IMAGE_DIGEST} | proof-image dashboard ghcr.io/jadenfix/beater/dashboard:{commit_sha} {DASHBOARD_IMAGE_DIGEST} | proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha} {DASHBOARD_E2E_IMAGE_DIGEST}\n"
        ),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "`docker compose images` excerpt must include ghcr.io/jadenfix/beater/otel-python tagged with the checked-out commit SHA",
    );
}

#[test]
fn gate2_outside_validator_rejects_compose_images_with_stale_service_tags() {
    let fixture = ValidatorFixture::new();
    let commit_sha = current_head();
    replace(
        &fixture.proof_path,
        &compose_images_excerpt_line(),
        &format!(
            "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd stale-sha | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard stale-sha | unrelated-image {commit_sha} | proof-image beaterd ghcr.io/jadenfix/beater/beaterd:{commit_sha} {BEATER_IMAGE_DIGEST} | proof-image dashboard ghcr.io/jadenfix/beater/dashboard:{commit_sha} {DASHBOARD_IMAGE_DIGEST} | proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha} {DASHBOARD_E2E_IMAGE_DIGEST} | proof-image otel-python ghcr.io/jadenfix/beater/otel-python:{commit_sha} {OTEL_PYTHON_IMAGE_DIGEST}\n"
        ),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "`docker compose images` excerpt must include ghcr.io/jadenfix/beater/beaterd tagged with the checked-out commit SHA",
    );
}

#[test]
fn gate2_outside_validator_rejects_noncanonical_compose_project() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Compose project: beater-stopwatch",
        "- Compose project: beater-alt",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Compose project in stopwatch proof must be 'beater-stopwatch'",
    );
}

#[test]
fn gate2_outside_validator_rejects_non_main_stopwatch_branch() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Git branch: `main`",
        "- Git branch: `feature-proof`",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Git branch in stopwatch proof must be 'main'");
}

#[test]
fn gate2_outside_validator_rejects_duplicate_stopwatch_field() {
    let fixture = ValidatorFixture::new();
    append(
        &fixture.stopwatch_path,
        "\n- Git branch: `feature-hidden`\n",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "duplicate field in stopwatch proof: Git branch");
}

#[test]
fn gate2_outside_validator_rejects_copied_stopwatch_metadata_mismatch() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Docker version: Docker version 29.2.0",
        "- Docker version: Docker version 0.0.0",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Docker version mismatch between proof artifacts");
}

#[test]
fn gate2_outside_validator_rejects_wrong_clone_url() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Clone URL: `https://github.com/jadenfix/beater.git`",
        "- Clone URL: `https://github.com/jadenfix/beater-fork.git`",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Clone URL must be https://github.com/jadenfix/beater.git",
    );
}

#[test]
fn gate2_outside_validator_rejects_wrong_stopwatch_origin() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Git origin: `https://github.com/jadenfix/beater.git`",
        "- Git origin: `https://github.com/jadenfix/beater-fork.git`",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Git origin in stopwatch proof must be 'https://github.com/jadenfix/beater.git'",
    );
}

#[test]
fn gate2_outside_validator_rejects_dirty_stopwatch_worktree() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Git worktree clean: yes",
        "- Git worktree clean: no",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Git worktree clean in stopwatch proof must be 'yes'",
    );
}

#[test]
fn gate2_outside_validator_rejects_script_only_timing_source() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Timing start source: external-clone",
        "- Timing start source: script",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Timing start source must be external-clone for outside-person evidence",
    );
}

#[test]
fn gate2_outside_validator_rejects_invalid_utc_timestamp() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Ended at: 2026-06-20T12:00:35Z",
        "- Ended at: 2026/06/20 12:00:35",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Ended at in outside-person proof must be UTC ISO-8601",
    );
}

#[test]
fn gate2_outside_validator_rejects_invalid_runner_date() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Date: 2026-06-20",
        "- Date: 06/20/2026",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Date in outside-person proof must be a valid date like 2026-06-20",
    );
}

#[test]
fn gate2_outside_validator_rejects_runner_date_not_matching_clone_start() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Date: 2026-06-20",
        "- Date: 2026-06-21",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Date in outside-person proof must match Clone started at UTC date 2026-06-20",
    );
}

#[test]
fn gate2_outside_validator_rejects_clone_started_after_script_started() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Clone started at: 2026-06-20T11:59:55Z",
        "- Clone started at: 2026-06-20T12:00:05Z",
    );
    replace(
        &fixture.stopwatch_path,
        "- Clone started at: 2026-06-20T11:59:55Z",
        "- Clone started at: 2026-06-20T12:00:05Z",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Clone started at in outside-person proof must not be after Script started at",
    );
}

#[test]
fn gate2_outside_validator_accepts_clone_started_at_script_started() {
    let fixture = ValidatorFixture::new();
    for path in [&fixture.proof_path, &fixture.stopwatch_path] {
        replace(
            path,
            "- Clone started at: 2026-06-20T11:59:55Z",
            "- Clone started at: 2026-06-20T12:00:00Z",
        );
        replace(
            path,
            "- Time-to-first-trace: 12s",
            "- Time-to-first-trace: 7s",
        );
        replace(
            path,
            "- Time-to-quickstart-click: 20s",
            "- Time-to-quickstart-click: 15s",
        );
    }
    replace(
        &fixture.proof_path,
        "- Total proof duration: 40s",
        "- Total proof duration: 35s",
    );
    replace(
        &fixture.stopwatch_path,
        "- Total duration: 40s",
        "- Total duration: 35s",
    );

    let output = run_validator(&fixture.proof_path);

    assert_success(output, DRAFT_VALID);
}

#[test]
fn gate2_outside_validator_rejects_total_duration_mismatched_to_timestamps() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Total proof duration: 40s",
        "- Total proof duration: 37s",
    );
    replace(
        &fixture.stopwatch_path,
        "- Total duration: 40s",
        "- Total duration: 37s",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Total proof duration in outside-person proof must match timestamps",
    );
}

#[test]
fn gate2_outside_validator_rejects_quickstart_click_after_total_duration() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Time-to-quickstart-click: 20s",
        "- Time-to-quickstart-click: 45s",
    );
    replace(
        &fixture.stopwatch_path,
        "- Time-to-quickstart-click: 20s",
        "- Time-to-quickstart-click: 45s",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Time-to-quickstart-click in outside-person proof must be within Total proof duration",
    );
}

#[test]
fn gate2_outside_validator_rejects_script_quickstart_click_after_script_duration() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Script-to-quickstart-click: 15s",
        "- Script-to-quickstart-click: 37s",
    );
    replace(
        &fixture.stopwatch_path,
        "- Script-to-quickstart-click: 15s",
        "- Script-to-quickstart-click: 37s",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Script-to-quickstart-click in outside-person proof must be within Script duration",
    );
}

#[test]
fn gate2_outside_validator_rejects_automated_quickstart_click_source() {
    let fixture = ValidatorFixture::new();
    for path in [&fixture.proof_path, &fixture.stopwatch_path] {
        replace(
            path,
            "- Quickstart click source: manual-outside-runner",
            "- Quickstart click source: automated-browser-proof",
        );
    }

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Quickstart click source must be manual-outside-runner for outside-person evidence",
    );
}

#[test]
fn gate2_outside_validator_rejects_missing_manual_quickstart_confirmation() {
    let fixture = ValidatorFixture::new();
    for path in [&fixture.proof_path, &fixture.stopwatch_path] {
        replace(
            path,
            "- Manual quickstart confirmation: yes",
            "- Manual quickstart confirmation: not requested",
        );
    }

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Manual quickstart confirmation must be yes for outside-person evidence",
    );
}

#[test]
fn gate2_outside_validator_rejects_first_trace_missing_clone_offset() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Time-to-first-trace: 12s",
        "- Time-to-first-trace: 6s",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Time-to-first-trace in stopwatch proof must equal Script-to-first-trace plus clone-to-script time",
    );
}

#[test]
fn gate2_outside_validator_rejects_missing_outside_run_attestation() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!("- Outside-run attestation: {OUTSIDE_RUN_ATTESTATION}"),
        "- Outside-run attestation:",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Outside-run attestation must match");
}

#[test]
fn gate2_outside_validator_rejects_maintainer_relationship() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "external validation fixture",
        "Beater project maintainer",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "runner relationship/prior exposure must not contradict outside-run attestation",
    );
}

#[test]
fn gate2_outside_validator_accepts_negated_employee_relationship_disclosure() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "external validation fixture",
        "external evaluator; not internal; not an employee",
    );

    let output = run_validator(&fixture.proof_path);

    assert_success(output, DRAFT_VALID);
}

#[test]
fn gate2_outside_validator_rejects_missing_prior_exposure() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Prior Beater repo exposure: no prior exposure",
        "- Prior Beater repo exposure:",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "unresolved required fields: Prior Beater repo exposure",
    );
}

#[test]
fn gate2_outside_validator_accepts_prior_exposure_none() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Prior Beater repo exposure: no prior exposure",
        "- Prior Beater repo exposure: none",
    );

    let output = run_validator(&fixture.proof_path);

    assert_success(output, DRAFT_VALID);
}

#[test]
fn gate2_outside_validator_rejects_failed_preflight_status() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "- Preflight status: passed",
        "- Preflight status: failed",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Preflight status must be passed");
}

#[test]
fn gate2_outside_validator_rejects_stale_commit_sha() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &current_head(),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    replace(
        &fixture.stopwatch_path,
        &current_head(),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Commit SHA must match current HEAD or be an evidence-only ancestor",
    );
}

#[test]
fn gate2_outside_validator_accepts_evidence_only_ancestor_closure_repo() {
    let fixture = write_validator_closure_fixture_repo();

    let output = run_default_validator_in_repo(fixture.path());

    assert_success(output, CLOSURE_VALID);
}

#[test]
fn gate2_outside_validator_rejects_dirty_non_evidence_worktree_at_closure() {
    let fixture = write_validator_closure_fixture_repo();
    fs::write(fixture.path().join("source-drift.txt"), "dirty\n")
        .unwrap_or_else(|err| panic!("write dirty non-evidence fixture file: {err}"));

    let output = run_default_validator_in_repo(fixture.path());

    assert_failure(
        output,
        "Completed Gate 2 closure proof has uncommitted non-evidence worktree changes",
    );
}

#[test]
fn gate2_outside_validator_rejects_local_build_evidence() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "Startup mode: prebuilt-image",
        "Startup mode: local-build",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Startup mode in stopwatch proof must be 'prebuilt-image'",
    );
}

#[test]
fn gate2_outside_validator_rejects_bare_image_digest() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        BEATER_IMAGE_DIGEST,
        "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    );
    replace(
        &fixture.stopwatch_path,
        BEATER_IMAGE_DIGEST,
        "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Beater image digest in outside-person proof must be a GHCR repo digest for beaterd",
    );
}

#[test]
fn gate2_outside_validator_rejects_mutable_image_reference() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!("ghcr.io/jadenfix/beater/dashboard-e2e:{}", current_head()),
        "ghcr.io/jadenfix/beater/dashboard-e2e:main",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Dashboard e2e image reference in outside-person proof must be",
    );
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
fn gate2_outside_validator_rejects_alternate_api_endpoint() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "http://127.0.0.1:8080",
        "http://127.0.0.1:18080",
    );
    replace(
        &fixture.stopwatch_path,
        "http://127.0.0.1:8080",
        "http://127.0.0.1:18080",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "API endpoint must be http://127.0.0.1:8080");
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
fn gate2_outside_validator_rejects_dashboard_url_with_non_root_path() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        "http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace=",
        "http://127.0.0.1:3000/not-dashboard?tenant=demo&project=demo&environment=local&trace=",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Quickstart dashboard URL must use the dashboard root path",
    );
}

#[test]
fn gate2_outside_validator_rejects_reused_trace_id_for_both_flows() {
    let fixture = ValidatorFixture::new();
    replace(&fixture.proof_path, ALL_KIND_TRACE, QUICKSTART_TRACE);
    replace(&fixture.stopwatch_path, ALL_KIND_TRACE, QUICKSTART_TRACE);
    replace(&fixture.notes_path, ALL_KIND_TRACE, QUICKSTART_TRACE);

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Quickstart trace ID and All-kind nested trace ID must be distinct",
    );
}

#[test]
fn gate2_outside_validator_rejects_weak_terminal_excerpt() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!(
            "- Terminal output excerpt: Gate 2 compose stopwatch passed; Browser recording: passed; Quickstart dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={QUICKSTART_TRACE}; All-kind dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={ALL_KIND_TRACE}"
        ),
        "- Terminal output excerpt: generated proof says browser recording passed",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Terminal output excerpt must include compose stopwatch pass line",
    );
}

#[test]
fn gate2_outside_validator_rejects_weak_llm_observation() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!("- Runner llm.call observation: {LLM_OBSERVATION}"),
        "- Runner llm.call observation: clicked the trace and it looked good",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Runner llm.call observation must mention: llm.call, prompt, completion, model, token breakdown, cost, latency",
    );
}

#[test]
fn gate2_outside_validator_rejects_weak_waterfall_observation() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &format!("- Runner waterfall observation: {WATERFALL_OBSERVATION}"),
        "- Runner waterfall observation: opened the second dashboard and it looked good",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Runner waterfall observation must mention: run, turn, step, tool, MCP",
    );
}

#[test]
fn gate2_outside_validator_rejects_dirty_evidence_artifact_at_closure() {
    let fixture = write_validator_closure_fixture_repo();
    append(
        &fixture
            .path()
            .join("docs/demos/gate2-closure-fixture/stopwatch-proof.md"),
        "\n<!-- local evidence edit after commit -->\n",
    );

    let output = run_default_validator_in_repo(fixture.path());

    assert_failure(
        output,
        "Stopwatch proof file must be committed and clean before Gate 2 closure",
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
fn gate2_outside_validator_rejects_non_compose_recording_notes() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.notes_path,
        "- Recording mode: compose",
        "- Recording mode: quickstart",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording notes Recording mode must be compose for outside-person proof",
    );
}

#[test]
fn gate2_outside_validator_rejects_recording_notes_without_full_gate2_flow() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.notes_path,
        "read prompt, completion, model, token breakdown, cost, and latency -> ",
        "",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording notes Shows must describe the full Gate 2 flow",
    );
}

#[test]
fn gate2_outside_validator_rejects_recording_notes_with_generic_tokens_only() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.notes_path,
        "read prompt, completion, model, token breakdown, cost, and latency -> ",
        "read prompt, completion, model, tokens, cost, and latency -> ",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording notes Shows must describe the full Gate 2 flow",
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
fn gate2_outside_validator_rejects_non_webm_recording() {
    let fixture = ValidatorFixture::new();
    fs::write(&fixture.recording_path, b"not a webm recording")
        .unwrap_or_else(|err| panic!("write {}: {err}", fixture.recording_path.display()));

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording must start with a WebM/EBML header",
    );
}

#[test]
fn gate2_outside_validator_rejects_padded_webm_header_recording() {
    let fixture = ValidatorFixture::new();
    fs::write(&fixture.recording_path, padded_webm_header_bytes())
        .unwrap_or_else(|err| panic!("write {}: {err}", fixture.recording_path.display()));

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording WebM must contain a Segment element",
    );
}

#[test]
fn gate2_outside_validator_rejects_marker_only_fake_webm_recording() {
    let fixture = ValidatorFixture::new();
    fs::write(&fixture.recording_path, marker_only_fake_webm_bytes())
        .unwrap_or_else(|err| panic!("write {}: {err}", fixture.recording_path.display()));

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording WebM must contain a Segment element",
    );
}

#[test]
fn gate2_outside_validator_rejects_tiny_webm_recording() {
    let fixture = ValidatorFixture::new();
    fs::write(
        &fixture.recording_path,
        [
            bytes_from_hex("1a45dfa39f4286810142f7810142f2810442f381084282847765626d"),
            b" tiny".to_vec(),
        ]
        .concat(),
    )
    .unwrap_or_else(|err| panic!("write {}: {err}", fixture.recording_path.display()));

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "screen recording must be a real WebM capture of at least",
    );
}

#[test]
fn gate2_outside_validator_rejects_unplayable_webm_recording() {
    let fixture = ValidatorFixture::new();

    let output = run_validator_with_ffprobe_script(
        &fixture.proof_path,
        "#!/bin/sh\nprintf 'not playable\\n' >&2\nexit 1\n",
    );

    assert_failure(
        output,
        "screen recording must be a playable WebM video: ffprobe failed",
    );
}

#[test]
fn gate2_outside_validator_rejects_short_review_recording() {
    let fixture = ValidatorFixture::new();

    let output = run_validator_with_ffprobe_script(
        &fixture.proof_path,
        "#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=3.68\\n'\n",
    );

    assert_failure(
        output,
        "screen recording must be a reviewable full-flow capture of at least 8 seconds",
    );
}

#[test]
fn gate2_outside_validator_rejects_symlink_recording_artifact() {
    let fixture = ValidatorFixture::new();
    fs::remove_file(&fixture.recording_path)
        .unwrap_or_else(|err| panic!("remove {}: {err}", fixture.recording_path.display()));
    symlink(
        repo_root().join("docs/demos/gate2-compose-browser-demo.webm"),
        &fixture.recording_path,
    )
    .unwrap_or_else(|err| panic!("symlink recording artifact: {err}"));

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Screen recording must not be a symlink");
}

#[test]
fn gate2_outside_validator_rejects_symlink_recording_notes_artifact_without_traceback() {
    let fixture = ValidatorFixture::new();
    let target = fixture.notes_path.with_file_name("binary-notes-target.md");
    fs::write(&target, [0xff, 0xfe, 0xfd])
        .unwrap_or_else(|err| panic!("write binary notes target: {err}"));
    fs::remove_file(&fixture.notes_path)
        .unwrap_or_else(|err| panic!("remove {}: {err}", fixture.notes_path.display()));
    symlink(&target, &fixture.notes_path)
        .unwrap_or_else(|err| panic!("symlink recording notes artifact: {err}"));

    let output = run_validator(&fixture.proof_path);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Traceback"),
        "validator must report invalid symlink notes without a Python traceback:\n{stderr}"
    );
    assert_failure(output, "Screen recording notes must not be a symlink");
}

#[test]
fn gate2_outside_validator_rejects_symlink_parent_artifact_path() {
    let fixture = ValidatorFixture::new();
    let link_parent = TempDir::new_in(repo_root().join("docs/demos"))
        .unwrap_or_else(|err| panic!("create symlink parent fixture dir: {err}"));
    let linked_dir = link_parent.path().join("linked-artifacts");
    symlink(fixture._artifact_dir.path(), &linked_dir)
        .unwrap_or_else(|err| panic!("symlink artifact parent dir: {err}"));
    let linked_recording = repo_relative_path(&linked_dir.join("recording.webm"));
    replace(
        &fixture.proof_path,
        &fixture.recording_field,
        &linked_recording,
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Screen recording must not be a symlink");
}

#[test]
fn gate2_outside_validator_rejects_symlink_stopwatch_artifact() {
    let fixture = ValidatorFixture::new();
    let target = fixture.stopwatch_path.with_file_name("stopwatch-target.md");
    fs::rename(&fixture.stopwatch_path, &target)
        .unwrap_or_else(|err| panic!("rename stopwatch fixture: {err}"));
    symlink(&target, &fixture.stopwatch_path)
        .unwrap_or_else(|err| panic!("symlink stopwatch artifact: {err}"));

    let output = run_validator(&fixture.proof_path);

    assert_failure(output, "Stopwatch proof file must not be a symlink");
}

#[test]
fn gate2_outside_validator_rejects_absolute_artifact_paths() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        &fixture.recording_field,
        &fixture.recording_path.to_string_lossy(),
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Screen recording must be a repo-relative path under docs/demos",
    );
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

#[test]
fn gate2_outside_validator_rejects_registry_unbound_image_digest_at_closure() {
    let fixture = write_validator_closure_fixture_repo();
    let wrong_digest =
        "ghcr.io/jadenfix/beater/beaterd@sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    replace(
        &fixture
            .path()
            .join("docs/demos/gate2-outside-person-proof.md"),
        BEATER_IMAGE_DIGEST,
        wrong_digest,
    );
    replace(
        &fixture
            .path()
            .join("docs/demos/gate2-closure-fixture/stopwatch-proof.md"),
        BEATER_IMAGE_DIGEST,
        wrong_digest,
    );
    git_success(fixture.path(), &["add", "docs/demos"]);
    git_success(
        fixture.path(),
        &["commit", "-m", "corrupt Gate 2 digest evidence"],
    );

    let output = run_default_validator_in_repo(fixture.path());

    assert_failure(
        output,
        "Beater image digest in outside-person proof must match public GHCR manifest digest",
    );
}

#[test]
fn gate2_outside_validator_rejects_dashboard_e2e_digest_mismatch() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        DASHBOARD_E2E_IMAGE_DIGEST,
        "ghcr.io/jadenfix/beater/dashboard-e2e@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "dashboard e2e image digest mismatch between proof artifacts",
    );
}

#[test]
fn gate2_outside_validator_rejects_otel_python_digest_mismatch() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.proof_path,
        OTEL_PYTHON_IMAGE_DIGEST,
        "ghcr.io/jadenfix/beater/otel-python@sha256:cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "otel python image digest mismatch between proof artifacts",
    );
}

struct ValidatorFixture {
    _artifact_dir: TempDir,
    dir: TempDir,
    proof_path: PathBuf,
    stopwatch_path: PathBuf,
    notes_path: PathBuf,
    recording_path: PathBuf,
    stopwatch_field: String,
    recording_field: String,
}

impl ValidatorFixture {
    fn new() -> Self {
        let root = repo_root();
        let artifact_dir = TempDir::new_in(root.join("docs/demos")).unwrap_or_else(|err| {
            panic!("create validator fixture artifact tempdir under docs/demos: {err}")
        });
        let dir = tempdir("create validator fixture tempdir");
        let proof_path = dir.path().join("outside-proof.md");
        let stopwatch_path = artifact_dir.path().join("stopwatch-proof.md");
        let notes_path = artifact_dir.path().join("recording-notes.md");
        let recording_path = artifact_dir.path().join("recording.webm");
        let artifact_rel = repo_relative_path(artifact_dir.path());
        let stopwatch_field = format!("{artifact_rel}/stopwatch-proof.md");
        let recording_field = format!("{artifact_rel}/recording.webm");
        let notes_field = format!("{artifact_rel}/recording-notes.md");

        fs::write(&recording_path, recording_bytes())
            .unwrap_or_else(|err| panic!("write {}: {err}", recording_path.display()));
        let recording_name = recording_path
            .file_name()
            .unwrap_or_else(|| {
                panic!(
                    "recording path has no file name: {}",
                    recording_path.display()
                )
            })
            .to_string_lossy();
        fs::write(&notes_path, recording_notes(&recording_name))
            .unwrap_or_else(|err| panic!("write {}: {err}", notes_path.display()));
        fs::write(
            &stopwatch_path,
            stopwatch_proof(&recording_field, &notes_field),
        )
        .unwrap_or_else(|err| panic!("write {}: {err}", stopwatch_path.display()));
        fs::write(
            &proof_path,
            outside_proof(&stopwatch_field, &recording_field, &notes_field),
        )
        .unwrap_or_else(|err| panic!("write {}: {err}", proof_path.display()));

        Self {
            _artifact_dir: artifact_dir,
            dir,
            proof_path,
            stopwatch_path,
            notes_path,
            recording_path,
            stopwatch_field,
            recording_field,
        }
    }
}

fn outside_proof(stopwatch: &str, recording: &str, notes: &str) -> String {
    let commit_sha = current_head();
    let quickstart_release_id = quickstart_release_id();
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
- Outside-run attestation: {OUTSIDE_RUN_ATTESTATION}

## Repository

- Clone URL: `https://github.com/jadenfix/beater.git`
- Commit SHA: {commit_sha}
- Branch: main
- Worktree clean: yes
- OS/arch: Darwin arm64
- Beater image reference: ghcr.io/jadenfix/beater/beaterd:{commit_sha}
- Dashboard image reference: ghcr.io/jadenfix/beater/dashboard:{commit_sha}
- Dashboard e2e image reference: ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha}
- OTEL Python image reference: ghcr.io/jadenfix/beater/otel-python:{commit_sha}
- Beater image digest: {BEATER_IMAGE_DIGEST}
- Dashboard image digest: {DASHBOARD_IMAGE_DIGEST}
- Dashboard e2e image digest: {DASHBOARD_E2E_IMAGE_DIGEST}
- OTEL Python image digest: {OTEL_PYTHON_IMAGE_DIGEST}
- API endpoint: http://127.0.0.1:8080
- Dashboard base: http://127.0.0.1:3000
- Quickstart release ID: {quickstart_release_id}
- Timing start source: external-clone
- Clone started at: 2026-06-20T11:59:55Z
- Script started at: 2026-06-20T12:00:00Z
- Started at: 2026-06-20T12:00:00Z
- Ended at: 2026-06-20T12:00:35Z
- Time-to-first-trace: 12s
- Script-to-first-trace: 7s
- Time-to-quickstart-click: 20s
- Script-to-quickstart-click: 15s
- Quickstart click source: manual-outside-runner
- Manual quickstart confirmation: yes
- Total proof duration: 40s
- Script duration: 35s
- Outside-run wrapper: yes

## Commands

```bash
bash -lc 't="$(date +%s)" && git clone https://github.com/jadenfix/beater.git && cd beater && BEATER_GATE2_CLONE_STARTED_EPOCH="$t" scripts/gate2-outside-run.sh'
```

The runner completed the flow using only public repository instructions.

## Required Evidence

- Stopwatch proof file: {stopwatch}
- Screen recording: `{recording}`
- Screen recording notes: `{notes}`
- Screen recording SHA256: {RECORDING_SHA}
- Terminal output excerpt: Gate 2 compose stopwatch passed; Browser recording: passed; Quickstart dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={QUICKSTART_TRACE}; All-kind dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={ALL_KIND_TRACE}
- Runner llm.call observation: {LLM_OBSERVATION}
- Runner waterfall observation: {WATERFALL_OBSERVATION}
{compose_images_excerpt}
- Quickstart trace ID: {QUICKSTART_TRACE}
- Quickstart dashboard URL: `http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={QUICKSTART_TRACE}`
- All-kind nested trace ID: {ALL_KIND_TRACE}
- All-kind dashboard URL: `http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace={ALL_KIND_TRACE}`
- `docker compose` logs saved: temp fixture
- Failure notes, if any: none

## Pass Checklist

- [x] Fresh clone was used.
- [x] Docker was running before the stopwatch started.
- [x] curl was available before the stopwatch started.
- [x] Default ports were used: API `127.0.0.1:8080`, OTLP `127.0.0.1:4317`, dashboard `127.0.0.1:3000`.
- [x] `BEATER_GATE2_REUSE` was not set.
- [x] The script reported `Clean start: yes`.
- [x] Time-to-first-trace was 300 seconds or less.
- [x] Time-to-first-trace includes clone time.
- [x] Manual quickstart click confirmation was recorded before 300 seconds.
- [x] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [x] Clicking the `llm.call` span showed prompt, completion, model, token breakdown, cost, and latency.
- [x] The all-kind trace rendered run -> turn -> step -> tool -> MCP nesting in the waterfall.
- [x] The browser proof passed for both the quickstart trace and all-kind waterfall.
- [x] The stopwatch script generated and reported the browser recording.
- [x] A screen recording of the full flow is committed under `docs/demos/`.
- [x] The runner completed the flow using only public repository instructions.
	"#,
        compose_images_excerpt = compose_images_excerpt_line().trim_end(),
    )
}

fn compose_images_excerpt_line() -> String {
    let commit_sha = current_head();
    format!(
        "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd {commit_sha} | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard {commit_sha} | beater-stopwatch-dashboard-e2e-run-1 ghcr.io/jadenfix/beater/dashboard-e2e {commit_sha} | beater-stopwatch-otel-python-quickstart-run-1 ghcr.io/jadenfix/beater/otel-python {commit_sha} | proof-image beaterd ghcr.io/jadenfix/beater/beaterd:{commit_sha} {BEATER_IMAGE_DIGEST} | proof-image dashboard ghcr.io/jadenfix/beater/dashboard:{commit_sha} {DASHBOARD_IMAGE_DIGEST} | proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha} {DASHBOARD_E2E_IMAGE_DIGEST} | proof-image otel-python ghcr.io/jadenfix/beater/otel-python:{commit_sha} {OTEL_PYTHON_IMAGE_DIGEST}\n"
    )
}

fn stopwatch_proof(recording: &str, notes: &str) -> String {
    let commit_sha = current_head();
    let quickstart_release_id = quickstart_release_id();
    format!(
        r#"# Gate 2 Compose Stopwatch Proof

- Timing start source: external-clone
- Clone started at: 2026-06-20T11:59:55Z
- Script started at: 2026-06-20T12:00:00Z
- Started: 2026-06-20T12:00:00Z
- Ended: 2026-06-20T12:00:35Z
- Time-to-first-trace: 12s
- Script-to-first-trace: 7s
- Time-to-quickstart-click: 20s
- Script-to-quickstart-click: 15s
- Quickstart click source: manual-outside-runner
- Manual quickstart confirmation: yes
- Total duration: 40s
- Script duration: 35s
- Limit: 300s
- Git SHA: `{commit_sha}`
- Git branch: `main`
- Git origin: `https://github.com/jadenfix/beater.git`
- Git worktree clean: yes
- OS/arch: `Darwin arm64`
- Docker: `Docker version 29.2.0`
- Docker Compose: `Docker Compose version v5.0.2`
- Startup mode: prebuilt-image
- Clean start: yes
- Reuse override: `BEATER_GATE2_REUSE=0`
- Outside-run wrapper: yes
- Prebuilt pull policy: `always`
- Compose project: beater-stopwatch
- Beater image reference: `ghcr.io/jadenfix/beater/beaterd:{commit_sha}`
- Dashboard image reference: `ghcr.io/jadenfix/beater/dashboard:{commit_sha}`
- Dashboard e2e image reference: `ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha}`
- OTEL Python image reference: `ghcr.io/jadenfix/beater/otel-python:{commit_sha}`
- Beater image digest: `{BEATER_IMAGE_DIGEST}`
- Dashboard image digest: `{DASHBOARD_IMAGE_DIGEST}`
- Dashboard e2e image digest: `{DASHBOARD_E2E_IMAGE_DIGEST}`
- OTEL Python image digest: `{OTEL_PYTHON_IMAGE_DIGEST}`
- Quickstart snippet: `examples/python/five_line_otel.py`
- API endpoint: `http://127.0.0.1:8080`
- OTLP endpoint: `http://127.0.0.1:4317`
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart release ID: `{quickstart_release_id}`
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

## Compose Images

```text
CONTAINER                      REPOSITORY                          TAG                                        PLATFORM            IMAGE ID            SIZE                CREATED
beater-stopwatch-beaterd-1     ghcr.io/jadenfix/beater/beaterd     {commit_sha}   linux/arm64         bbbbbbbbbbbb        88.4MB              1 minute ago
beater-stopwatch-dashboard-1   ghcr.io/jadenfix/beater/dashboard   {commit_sha}   linux/arm64         cccccccccccc        99.2MB              1 minute ago
beater-stopwatch-dashboard-e2e-run-1 ghcr.io/jadenfix/beater/dashboard-e2e {commit_sha} linux/arm64 eeeeeeeeeeee 132MB 1 minute ago
beater-stopwatch-otel-python-quickstart-run-1 ghcr.io/jadenfix/beater/otel-python {commit_sha} linux/arm64 aaaaaaaaaaaa 116MB 1 minute ago
proof-image beaterd ghcr.io/jadenfix/beater/beaterd:{commit_sha} {BEATER_IMAGE_DIGEST}
proof-image dashboard ghcr.io/jadenfix/beater/dashboard:{commit_sha} {DASHBOARD_IMAGE_DIGEST}
proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:{commit_sha} {DASHBOARD_E2E_IMAGE_DIGEST}
proof-image otel-python ghcr.io/jadenfix/beater/otel-python:{commit_sha} {OTEL_PYTHON_IMAGE_DIGEST}
```

This is an outside-run stopwatch source artifact generated through
`scripts/gate2-outside-run.sh`. Gate 2 closes only after the completed
outside-person proof, this stopwatch proof, the recording, and the recording
notes are committed and `scripts/validate-gate2-outside-proof.sh` passes.
"#,
    )
}

fn recording_notes(recording_name: &str) -> String {
    format!(
        r#"# Gate 2 Compose Browser Demo

- Artifact: `{recording_name}`
- SHA256: `{RECORDING_SHA}`
- Recording mode: compose
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart trace: `{QUICKSTART_TRACE}`
- All-kind trace: `{ALL_KIND_TRACE}`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, token breakdown, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.
"#
    )
}

fn run_validator(proof_path: &Path) -> Output {
    run_validator_with_args(proof_path, &[])
}

fn run_validator_with_args(proof_path: &Path, args: &[&str]) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    run_validator_with_path(proof_path, args, Some(&ffprobe))
}

fn run_validator_with_ffprobe_script(proof_path: &Path, script: &str) -> Output {
    let ffprobe = fake_ffprobe_dir(script);
    run_validator_with_path(proof_path, &[], Some(&ffprobe))
}

fn run_validator_without_ffprobe(proof_path: &Path) -> Output {
    let path_dir = path_without_ffprobe(
        "create validator PATH without ffprobe",
        &["python3", "dirname", "git"],
    );
    let root = repo_root();
    Command::new("/bin/bash")
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .current_dir(root)
        .env("PATH", path_with_isolated_tempdir(&path_dir))
        .env("BEATER_GATE2_OUTSIDE_PROOF", proof_path)
        .env("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS", "1")
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof validator without ffprobe: {err}"))
}

fn run_validator_with_path(proof_path: &Path, args: &[&str], path_dir: Option<&TempDir>) -> Output {
    let root = repo_root();
    let mut command = Command::new("/bin/bash");
    command
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .args(args)
        .current_dir(root)
        .env("BEATER_GATE2_OUTSIDE_PROOF", proof_path)
        .env("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS", "1");
    if let Some(path_dir) = path_dir {
        command.env("PATH", path_with_tempdir(path_dir));
    }
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof validator: {err}"))
}

fn run_validator_without_untracked_artifact_escape(proof_path: &Path) -> Output {
    let root = repo_root();
    Command::new("bash")
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .current_dir(root)
        .env("BEATER_GATE2_OUTSIDE_PROOF", proof_path)
        .env_remove("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS")
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

fn run_default_validator_in_repo(repo: &Path) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    let registry = tempdir("create validator registry fixture");
    write_registry_fixtures(registry.path());
    Command::new("bash")
        .arg(repo.join("scripts/validate-gate2-outside-proof.sh"))
        .current_dir(repo)
        .env("PATH", path_with_tempdir(&ffprobe))
        .env(
            "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS",
            registry.path(),
        )
        .env_remove("BEATER_GATE2_OUTSIDE_PROOF")
        .env_remove("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS")
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof validator in fixture repo: {err}"))
}

fn run_generator(stopwatch_path: &Path, output_path: &Path) -> Output {
    run_generator_with_attestation(stopwatch_path, output_path, true)
}

fn run_generator_with_attestation(
    stopwatch_path: &Path,
    output_path: &Path,
    attest: bool,
) -> Output {
    run_generator_with_options(stopwatch_path, output_path, attest, true, true)
}

fn run_generator_without_network_notes(stopwatch_path: &Path, output_path: &Path) -> Output {
    run_generator_with_options(stopwatch_path, output_path, true, false, true)
}

fn run_generator_without_observations(stopwatch_path: &Path, output_path: &Path) -> Output {
    run_generator_with_options(stopwatch_path, output_path, true, true, false)
}

fn run_generator_without_fake_ffprobe(stopwatch_path: &Path, output_path: &Path) -> Output {
    let path_dir = path_without_ffprobe(
        "create generator PATH without ffprobe",
        &["python3", "bash", "dirname", "git"],
    );
    let mut command = generator_command(
        stopwatch_path,
        output_path,
        "Validator Fixture Runner",
        "Chromium",
    );
    command
        .arg("--attest-outside-run")
        .arg("--network-notes")
        .arg("public docs only")
        .arg("--llm-observation")
        .arg(LLM_OBSERVATION)
        .arg("--waterfall-observation")
        .arg(WATERFALL_OBSERVATION)
        .env("PATH", path_with_isolated_tempdir(&path_dir));
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof generator without ffprobe: {err}"))
}

fn run_generator_with_runner_name(
    stopwatch_path: &Path,
    output_path: &Path,
    runner_name: &str,
) -> Output {
    run_generator_with_options_and_runner(
        stopwatch_path,
        output_path,
        true,
        true,
        true,
        runner_name,
        "Chromium",
    )
}

fn run_generator_with_prior_exposure(
    stopwatch_path: &Path,
    output_path: &Path,
    prior_exposure: &str,
) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    let mut command = generator_command_with_prior_exposure(
        stopwatch_path,
        output_path,
        "Validator Fixture Runner",
        "Chromium",
        prior_exposure,
    );
    command
        .arg("--attest-outside-run")
        .arg("--network-notes")
        .arg("public docs only")
        .arg("--llm-observation")
        .arg(LLM_OBSERVATION)
        .arg("--waterfall-observation")
        .arg(WATERFALL_OBSERVATION)
        .env("PATH", path_with_tempdir(&ffprobe));
    command.output().unwrap_or_else(|err| {
        panic!("run Gate 2 outside proof generator with prior exposure: {err}")
    })
}

fn run_generator_with_date(stopwatch_path: &Path, output_path: &Path, date: &str) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    let mut command = generator_command(
        stopwatch_path,
        output_path,
        "Validator Fixture Runner",
        "Chromium",
    );
    command
        .arg("--attest-outside-run")
        .arg("--network-notes")
        .arg("public docs only")
        .arg("--llm-observation")
        .arg(LLM_OBSERVATION)
        .arg("--waterfall-observation")
        .arg(WATERFALL_OBSERVATION)
        .arg("--date")
        .arg(date)
        .env("PATH", path_with_tempdir(&ffprobe));
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof generator with date: {err}"))
}

fn run_generator_with_options(
    stopwatch_path: &Path,
    output_path: &Path,
    attest: bool,
    include_network_notes: bool,
    include_observations: bool,
) -> Output {
    run_generator_with_options_and_runner(
        stopwatch_path,
        output_path,
        attest,
        include_network_notes,
        include_observations,
        "Validator Fixture Runner",
        "Chromium",
    )
}

fn run_generator_with_options_and_runner(
    stopwatch_path: &Path,
    output_path: &Path,
    attest: bool,
    include_network_notes: bool,
    include_observations: bool,
    runner_name: &str,
    browser: &str,
) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    let mut command = generator_command(stopwatch_path, output_path, runner_name, browser);
    command.env("PATH", path_with_tempdir(&ffprobe));
    if attest {
        command.arg("--attest-outside-run");
    }
    if include_network_notes {
        command.arg("--network-notes").arg("public docs only");
    }
    if include_observations {
        command
            .arg("--llm-observation")
            .arg(LLM_OBSERVATION)
            .arg("--waterfall-observation")
            .arg(WATERFALL_OBSERVATION);
    }
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof generator: {err}"))
}

fn generator_command(
    stopwatch_path: &Path,
    output_path: &Path,
    runner_name: &str,
    browser: &str,
) -> Command {
    generator_command_with_prior_exposure(
        stopwatch_path,
        output_path,
        runner_name,
        browser,
        "no prior exposure",
    )
}

fn generator_command_with_prior_exposure(
    stopwatch_path: &Path,
    output_path: &Path,
    runner_name: &str,
    browser: &str,
    prior_exposure: &str,
) -> Command {
    let root = repo_root();
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/generate-gate2-outside-proof.py"))
        .arg("--stopwatch-proof")
        .arg(stopwatch_path)
        .arg("--output")
        .arg(output_path)
        .arg("--runner-name")
        .arg(runner_name)
        .arg("--relationship")
        .arg("external validation fixture")
        .arg("--prior-exposure")
        .arg(prior_exposure)
        .arg("--machine-os")
        .arg("macOS arm64")
        .arg("--browser")
        .arg(browser)
        .arg("--preflight-status")
        .arg("passed")
        .arg("--compose-logs-saved")
        .arg("temp fixture")
        .arg("--failure-notes")
        .arg("none")
        .arg("--runner-notes")
        .arg("No extra runner notes.")
        .current_dir(root)
        .env("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS", "1");
    command
}

fn run_readiness_with_fixture(registry_path: &Path) -> Output {
    let root = repo_root();
    Command::new("python3")
        .arg(root.join("scripts/check-gate2-outside-readiness.py"))
        .arg("--skip-repo-shape")
        .arg("--registry-fixture")
        .arg(registry_path)
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside readiness checker: {err}"))
}

fn run_public_handoff_with_fixture(
    source_url: &str,
    expected_commit: &str,
    registry_path: &Path,
) -> Output {
    run_public_handoff_with_fixture_command(source_url, expected_commit, registry_path)
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 public handoff checker: {err}"))
}

fn run_public_handoff_with_fixture_env(
    source_url: &str,
    expected_commit: &str,
    registry_path: &Path,
    extra_env: (&str, &str),
) -> Output {
    let mut command =
        run_public_handoff_with_fixture_command(source_url, expected_commit, registry_path);
    command.env(extra_env.0, extra_env.1);
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 public handoff checker: {err}"))
}

fn run_public_handoff_with_fixture_command(
    source_url: &str,
    expected_commit: &str,
    registry_path: &Path,
) -> Command {
    let root = repo_root();
    let path_env = path_with_ffprobe_fixture(registry_path);
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/check-gate2-public-handoff.py"))
        .arg("--skip-local-readiness")
        .arg("--source-url")
        .arg(source_url)
        .arg("--expected-commit")
        .arg(expected_commit)
        .arg("--registry-fixture")
        .arg(registry_path)
        .current_dir(root)
        .env("PATH", path_env);
    command
}

fn run_public_handoff_full_run_with_fixture(
    source_url: &str,
    expected_commit: &str,
    registry_path: &Path,
    clone_parent: &Path,
) -> Output {
    let root = repo_root();
    Command::new("python3")
        .arg(root.join("scripts/check-gate2-public-handoff.py"))
        .arg("--skip-local-readiness")
        .arg("--source-url")
        .arg(source_url)
        .arg("--expected-commit")
        .arg(expected_commit)
        .arg("--registry-fixture")
        .arg(registry_path)
        .arg("--clone-parent")
        .arg(clone_parent)
        .arg("--full-run")
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 public handoff full-run checker: {err}"))
}

fn public_handoff_full_run_preflight_command(clone_parent: &Path) -> Command {
    let root = repo_root();
    let head = git_output(&root, &["rev-parse", "HEAD"]);
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/check-gate2-public-handoff.py"))
        .arg("--skip-local-readiness")
        .arg("--expected-commit")
        .arg(head)
        .arg("--clone-parent")
        .arg(clone_parent)
        .arg("--full-run")
        .current_dir(root)
        .env_remove("DOCKER_HOST");
    command
}

struct FakePublicHandoffRuntime {
    _dir: TempDir,
    path_env: String,
    docker_log: PathBuf,
}

fn fake_public_handoff_runtime(
    include_sha_tool: bool,
    docker_context_host: &str,
) -> FakePublicHandoffRuntime {
    let dir = tempdir("create fake public handoff runtime PATH");
    let python = python3_executable();
    symlink(&python, dir.path().join("python3")).unwrap_or_else(|err| {
        panic!(
            "symlink fake python3 {} -> {}: {err}",
            dir.path().join("python3").display(),
            python.display()
        )
    });

    let docker_log = dir.path().join("docker.log");
    write_executable(
        &dir.path().join("docker"),
        &format!(
            r#"#!/bin/sh
printf '%s\n' "$*" >> {docker_log}
case "$*" in
  "info")
    exit 0
    ;;
  "compose version")
    exit 0
    ;;
  "context inspect --format {{{{.Endpoints.docker.Host}}}}")
    printf '%s\n' {docker_context_host}
    exit 0
    ;;
  "compose -f docker-compose.prebuilt.yml -p beater-stopwatch down -v --remove-orphans")
    exit 0
    ;;
  *)
    printf 'unexpected docker invocation: %s\n' "$*" >&2
    exit 2
    ;;
esac
"#,
            docker_log = shell_single_quote(&docker_log.to_string_lossy()),
            docker_context_host = shell_single_quote(docker_context_host)
        ),
    );
    write_executable(&dir.path().join("curl"), "#!/bin/sh\nexit 0\n");
    write_executable(
        &dir.path().join("ffprobe"),
        "#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n",
    );
    if include_sha_tool {
        write_executable(
            &dir.path().join("shasum"),
            "#!/bin/sh\nprintf 'fixture  %s\\n' \"$2\"\n",
        );
    }

    FakePublicHandoffRuntime {
        path_env: dir.path().to_string_lossy().into_owned(),
        docker_log,
        _dir: dir,
    }
}

fn path_with_public_handoff_runtime(runtime: &FakePublicHandoffRuntime) -> String {
    path_with_dir(Path::new(&runtime.path_env))
}

fn path_with_ffprobe_fixture(parent: &Path) -> String {
    let bin = parent.join("bin");
    fs::create_dir_all(&bin)
        .unwrap_or_else(|err| panic!("create ffprobe fixture dir {}: {err}", bin.display()));
    write_executable(
        &bin.join("ffprobe"),
        "#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n",
    );
    path_with_dir(&bin)
}

fn fake_ffprobe_dir(script: &str) -> TempDir {
    let dir = tempdir("create fake ffprobe PATH");
    write_executable(&dir.path().join("ffprobe"), script);
    dir
}

fn path_with_tempdir(dir: &TempDir) -> String {
    path_with_dir(dir.path())
}

fn path_with_isolated_tempdir(dir: &TempDir) -> String {
    std::env::join_paths([dir.path()])
        .unwrap_or_else(|err| panic!("build isolated fixture PATH: {err}"))
        .to_string_lossy()
        .into_owned()
}

fn path_with_dir(dir: &Path) -> String {
    let existing_path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![dir.to_path_buf()];
    paths.extend(std::env::split_paths(&existing_path));
    std::env::join_paths(paths)
        .unwrap_or_else(|err| panic!("build fixture PATH: {err}"))
        .to_string_lossy()
        .into_owned()
}

fn path_without_ffprobe(label: &str, tools: &[&str]) -> TempDir {
    let dir = tempdir(label);
    for name in tools {
        symlink(&command_executable(name), dir.path().join(name))
            .unwrap_or_else(|err| panic!("symlink {name} fixture: {err}"));
    }
    dir
}

fn python3_executable() -> PathBuf {
    command_executable("python3")
}

fn command_executable(name: &str) -> PathBuf {
    let output = Command::new("python3")
        .arg("-c")
        .arg("import shutil, sys; print(shutil.which(sys.argv[1]) or '')")
        .arg(name)
        .output()
        .unwrap_or_else(|err| panic!("resolve {name} executable: {err}"));
    if !output.status.success() {
        panic!(
            "resolve {name} executable failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        panic!("could not resolve executable in PATH: {name}");
    }
    PathBuf::from(path)
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    let mut permissions = fs::metadata(path)
        .unwrap_or_else(|err| panic!("stat {}: {err}", path.display()))
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .unwrap_or_else(|err| panic!("chmod +x {}: {err}", path.display()));
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'"'"'"#))
}

fn run_outside_wrapper_dry_run(extra_env: Option<(&str, &str)>) -> Output {
    let fixture = write_outside_wrapper_fixture_repo("main");
    run_outside_wrapper_dry_run_in_repo(fixture.path(), extra_env)
}

fn run_outside_wrapper_dry_run_in_repo(repo: &Path, extra_env: Option<(&str, &str)>) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    let mut command = Command::new("bash");
    command
        .arg(repo.join("scripts/gate2-outside-run.sh"))
        .current_dir(repo);
    clear_outside_env(&mut command);
    command
        .env("PATH", path_with_tempdir(&ffprobe))
        .env("BEATER_GATE2_OUTSIDE_RUN_DRY_RUN", "1");
    if let Some((name, value)) = extra_env {
        command.env(name, value);
    }
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper fixture dry-run: {err}"))
}

fn run_outside_wrapper_real_preflight_in_repo(repo: &Path) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(repo.join("scripts/gate2-outside-run.sh"))
        .current_dir(repo);
    clear_outside_env(&mut command);
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper fixture preflight: {err}"))
}

fn run_outside_wrapper_real_with_clone_timer_in_repo(repo: &Path, clone_started: &str) -> Output {
    let ffprobe =
        fake_ffprobe_dir("#!/bin/sh\nprintf 'codec_type=video\\n'\nprintf 'duration=12.50\\n'\n");
    let mut command = Command::new("bash");
    command
        .arg(repo.join("scripts/gate2-outside-run.sh"))
        .current_dir(repo);
    clear_outside_env(&mut command);
    command
        .env("PATH", path_with_tempdir(&ffprobe))
        .env("BEATER_GATE2_CLONE_STARTED_EPOCH", clone_started)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command
        .spawn()
        .unwrap_or_else(|err| panic!("spawn Gate 2 outside wrapper fixture real run: {err}"));
    child
        .stdin
        .as_mut()
        .expect("fixture wrapper stdin should be piped")
        .write_all(b"\n")
        .expect("write manual checkpoint confirmation to fixture wrapper");
    child
        .wait_with_output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper fixture real run: {err}"))
}

fn run_service_image_digest_fixture(service: &str, repo_digests: &str) -> String {
    let script_text = fs::read_to_string(repo_root().join("scripts/gate2-compose-stopwatch.sh"))
        .unwrap_or_else(|err| panic!("read gate2-compose-stopwatch.sh: {err}"));
    let function_start = script_text
        .find("service_image_digest() {")
        .expect("stopwatch script should define service_image_digest");
    let function_end_marker = "\n}\n\nrequire_command()";
    let function_end = script_text[function_start..]
        .find(function_end_marker)
        .map(|offset| function_start + offset + "\n}\n".len())
        .expect("service_image_digest should end before require_command");
    let function = &script_text[function_start..function_end];
    let mut fixture = String::from("set -euo pipefail\n");
    fixture.push_str(function);
    fixture.push_str(
        r#"
compose() {
  if [[ "$#" -eq 3 && "$1" == "images" && "$2" == "-q" && "$3" == "$SERVICE" ]]; then
    printf '%s\n' image-id
    return 0
  fi
  return 1
}

docker() {
  if [[ "$#" -ge 5 && "$1" == "image" && "$2" == "inspect" && "$3" == "--format" ]]; then
    case "$4" in
      *RepoDigests*)
        printf '%s\n' "$REPO_DIGESTS"
        ;;
      *Id*)
        printf '%s\n' "sha256:fallback"
        ;;
      *)
        return 2
        ;;
    esac
    return 0
  fi
  return 2
}

service_image_digest "$SERVICE"
"#,
    );
    let output = Command::new("bash")
        .arg("-c")
        .arg(fixture)
        .env("SERVICE", service)
        .env("REPO_DIGESTS", repo_digests)
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|err| panic!("run service_image_digest fixture: {err}"));

    if !output.status.success() {
        panic!(
            "service_image_digest fixture failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout)
        .expect("service_image_digest stdout should be utf-8")
        .trim()
        .to_owned()
}

fn clear_outside_env(command: &mut Command) {
    for name in [
        "BEATER_GATE2_OUTSIDE_RUN_DRY_RUN",
        "BEATER_GATE2_CLONE_STARTED_EPOCH",
        "BEATER_DASHBOARD_PORT",
        "BEATER_HTTP_PORT",
        "BEATER_OTLP_GRPC_PORT",
        "BEATER_GATE2_REUSE",
        "BEATER_GATE2_LOCAL_BUILD",
        "BEATER_GATE2_PULL_POLICY",
        "BEATER_GATE2_WRITE_PROOF",
        "BEATER_GATE2_BROWSER_PROOF",
        "BEATER_GATE2_RECORD_DEMO",
        "BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS",
        "BEATERD_IMAGE",
        "BEATER_DASHBOARD_IMAGE",
        "BEATER_DASHBOARD_E2E_IMAGE",
        "BEATER_OTEL_PYTHON_IMAGE",
        "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS",
        "BEATER_GATE2_STOPWATCH_PROOF",
        "BEATER_GATE2_RECORD_VIDEO",
        "BEATER_GATE2_RECORD_NOTES",
        "KEEP_BEATER_COMPOSE",
        "COMPOSE_PROJECT_NAME",
        "BEATER_GATE2_FIXTURE_FULL_RUN",
    ] {
        command.env_remove(name);
    }
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

fn append(path: &Path, suffix: &str) {
    let mut text =
        fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    text.push_str(suffix);
    fs::write(path, text).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
}

fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = manifest.parent().and_then(|path| path.parent()) else {
        panic!("beaterd manifest must live under bins/beaterd");
    };
    root.to_path_buf()
}

fn repo_relative_path(path: &Path) -> String {
    let root = repo_root();
    path.strip_prefix(&root)
        .unwrap_or_else(|err| panic!("{} must be under repo root: {err}", path.display()))
        .to_string_lossy()
        .replace('\\', "/")
}

fn write_registry_fixtures(dir: &Path) {
    for image in ["beaterd", "dashboard", "dashboard-e2e", "otel-python"] {
        write_registry_manifest(dir, image, &["linux/amd64", "linux/arm64"]);
    }
}

fn write_registry_manifest(dir: &Path, image: &str, platforms: &[&str]) {
    let digest = registry_digest_for_image(image);
    let manifests = platforms
        .iter()
        .map(|platform| {
            let (os, architecture) = platform
                .split_once('/')
                .unwrap_or_else(|| panic!("invalid platform fixture: {platform}"));
            format!(
                r#"{{"digest":"{digest}","platform":{{"os":"{os}","architecture":"{architecture}"}}}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        dir.join(format!("{image}.json")),
        format!(r#"{{"digest":"{digest}","manifests":[{manifests}]}}"#),
    )
    .unwrap_or_else(|err| panic!("write registry fixture for {image}: {err}"));
}

fn registry_digest_for_image(image: &str) -> &'static str {
    match image {
        "beaterd" => "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "dashboard" => "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "dashboard-e2e" => {
            "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
        }
        "otel-python" => "sha256:abababababababababababababababababababababababababababababababab",
        other => panic!("unknown registry fixture image: {other}"),
    }
}

fn recording_bytes() -> Vec<u8> {
    fs::read(repo_root().join("docs/demos/gate2-compose-browser-demo.webm"))
        .unwrap_or_else(|err| panic!("read committed Gate 2 compose recording fixture: {err}"))
}

fn padded_webm_header_bytes() -> Vec<u8> {
    let mut bytes =
        bytes_from_hex("1a45dfa39f4286810142f7810142f2810442f381084282847765626d4287810242858102");
    bytes.resize(70_000, 0);
    bytes
}

fn marker_only_fake_webm_bytes() -> Vec<u8> {
    let mut bytes = padded_webm_header_bytes();
    bytes.extend(bytes_from_hex(
        "0018538067001549a966001654ae6b001f43b6750083810100a388000102030405060708",
    ));
    bytes
}

fn bytes_from_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "hex fixture should have even length");
    (0..hex.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16)
                .unwrap_or_else(|err| panic!("invalid hex fixture at {index}: {err}"))
        })
        .collect()
}

fn write_public_handoff_fixture_repo() -> TempDir {
    let root = repo_root();
    let fixture = tempdir("create public handoff fixture repo");

    for rel in [
        "scripts/check-gate2-outside-readiness.py",
        "scripts/check-gate2-public-handoff.py",
        "scripts/check-openapi-drift.sh",
        "scripts/gate2-proof.sh",
        "scripts/gate2-outside-run.sh",
        "scripts/gate2-compose-stopwatch.sh",
        "scripts/smoke-compose.sh",
        "scripts/generate-gate2-outside-proof.py",
        "scripts/validate-gate2-outside-proof.sh",
        "README.md",
        "docker-compose.yml",
        "docker-compose.prebuilt.yml",
        "docs/demos/gate2-outside-person-proof.md",
        "docs/demos/gate2-compose-browser-demo.webm",
        "docs/demos/gate2-compose-browser-demo.md",
        "docs/demos/gate2-compose-stopwatch.md",
    ] {
        copy_fixture_file(&root, fixture.path(), rel);
    }

    git_success(fixture.path(), &["init"]);
    git_success(
        fixture.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git_success(fixture.path(), &["config", "user.name", "Gate 2 Fixture"]);
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "fixture"]);
    git_success(fixture.path(), &["branch", "-M", "main"]);
    fixture
}

fn write_validator_closure_fixture_repo() -> TempDir {
    let root = repo_root();
    let fixture = tempdir("create validator closure fixture repo");
    copy_fixture_file(
        &root,
        fixture.path(),
        "scripts/validate-gate2-outside-proof.sh",
    );

    git_success(fixture.path(), &["init"]);
    git_success(
        fixture.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git_success(fixture.path(), &["config", "user.name", "Gate 2 Fixture"]);
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "base"]);
    git_success(fixture.path(), &["branch", "-M", "main"]);

    let tested_sha = git_output(fixture.path(), &["rev-parse", "HEAD"]);
    let current_repo_sha = current_head();
    let tested_release_id = quickstart_release_id_for(&tested_sha);
    let current_release_id = quickstart_release_id_for(&current_repo_sha);
    let artifact_rel = "docs/demos/gate2-closure-fixture";
    let artifact_dir = fixture.path().join(artifact_rel);
    fs::create_dir_all(&artifact_dir)
        .unwrap_or_else(|err| panic!("create validator closure artifact dir: {err}"));
    fs::write(artifact_dir.join("recording.webm"), recording_bytes())
        .unwrap_or_else(|err| panic!("write validator closure recording: {err}"));
    fs::write(
        artifact_dir.join("recording-notes.md"),
        recording_notes("recording.webm"),
    )
    .unwrap_or_else(|err| panic!("write validator closure recording notes: {err}"));

    let recording_field = format!("{artifact_rel}/recording.webm");
    let notes_field = format!("{artifact_rel}/recording-notes.md");
    let stopwatch_field = format!("{artifact_rel}/stopwatch-proof.md");
    let stopwatch = stopwatch_proof(&recording_field, &notes_field)
        .replace(&current_repo_sha, &tested_sha)
        .replace(&current_release_id, &tested_release_id);
    fs::write(artifact_dir.join("stopwatch-proof.md"), stopwatch)
        .unwrap_or_else(|err| panic!("write validator closure stopwatch proof: {err}"));
    let outside = outside_proof(&stopwatch_field, &recording_field, &notes_field)
        .replace(&current_repo_sha, &tested_sha)
        .replace(&current_release_id, &tested_release_id);
    fs::write(
        fixture
            .path()
            .join("docs/demos/gate2-outside-person-proof.md"),
        outside,
    )
    .unwrap_or_else(|err| panic!("write validator closure outside proof: {err}"));

    git_success(fixture.path(), &["add", "docs/demos"]);
    git_success(fixture.path(), &["commit", "-m", "add Gate 2 evidence"]);
    fixture
}

fn write_stopwatch_env_stub(repo: &Path) {
    let stub = repo.join("scripts/gate2-compose-stopwatch.sh");
    fs::write(
        &stub,
r#"#!/usr/bin/env bash
set -euo pipefail
mkdir -p docs/demos
cat <<'EOF_PROMPT'
Manual outside-run checkpoint:
  Fixture waits here to prove the diagnostic full-run confirms only after the
  wrapper reaches the timing-critical quickstart browser checkpoint.
EOF_PROMPT
if ! IFS= read -r _manual_checkpoint_confirmation; then
  echo "missing diagnostic manual checkpoint confirmation" >&2
  exit 42
fi
{
  echo "Open the quickstart URL above in a normal browser now; do not wait for the script to finish."
  echo "manual_checkpoint_confirmed=yes"
  echo "write=${BEATER_GATE2_WRITE_PROOF:-unset}"
  echo "browser=${BEATER_GATE2_BROWSER_PROOF:-unset}"
  echo "record=${BEATER_GATE2_RECORD_DEMO:-unset}"
  echo "post_slo=${BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS:-unset}"
  echo "reuse=${BEATER_GATE2_REUSE:-unset}"
  echo "local_build=${BEATER_GATE2_LOCAL_BUILD:-unset}"
  echo "pull_policy=${BEATER_GATE2_PULL_POLICY:-unset}"
  echo "keep=${KEEP_BEATER_COMPOSE:-unset}"
  echo "outside_wrapper=${BEATER_GATE2_OUTSIDE_WRAPPER:-unset}"
  echo "dry=${BEATER_GATE2_OUTSIDE_RUN_DRY_RUN:-unset}"
  echo "expected_origin=${BEATER_GATE2_EXPECTED_ORIGIN:-unset}"
  echo "clone_started=${BEATER_GATE2_CLONE_STARTED_EPOCH:-unset}"
  echo "dashboard_port=${BEATER_DASHBOARD_PORT:-unset}"
  echo "fixture_full_run=${BEATER_GATE2_FIXTURE_FULL_RUN:-unset}"
  echo "git_config_count=${GIT_CONFIG_COUNT:-unset}"
} > docs/demos/wrapper-real-env.txt
cat > docs/demos/gate2-compose-browser-demo.md <<'EOF_NOTES'
# Gate 2 Compose Browser Demo

- Artifact: `gate2-compose-browser-demo.webm`
- SHA256: `3dac802bc8f2db03406d0d76e4e1618ed5b516a2cf3d286589e1a588cf6e6534`
- Recording mode: compose
- Dashboard base: `http://127.0.0.1:3000`
- Quickstart trace: `11111111111111111111111111111111`
- All-kind trace: `22222222222222222222222222222222`
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, token breakdown, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.
EOF_NOTES
commit_sha="$(git rev-parse HEAD)"
cat > docs/demos/gate2-compose-stopwatch.md <<'EOF_PROOF'
# Gate 2 Compose Stopwatch Proof

- Timing start source: external-clone
- Clone started at: 2026-06-20T11:59:55Z
- Script started at: 2026-06-20T12:00:00Z
- Started: 2026-06-20T12:00:00Z
- Ended: 2026-06-20T12:00:35Z
- Time-to-first-trace: 12s
- Script-to-first-trace: 7s
- Time-to-quickstart-click: 20s
- Script-to-quickstart-click: 15s
- Quickstart click source: manual-outside-runner
- Manual quickstart confirmation: yes
- Total duration: 40s
- Script duration: 35s
- Limit: 300s
- Git SHA: __COMMIT_SHA__
- Git branch: main
- Git origin: https://github.com/jadenfix/beater.git
- Git worktree clean: yes
- OS/arch: Darwin arm64
- Docker: Docker version 29.2.0
- Docker Compose: Docker Compose version v5.0.2
- Startup mode: prebuilt-image
- Clean start: yes
- Reuse override: BEATER_GATE2_REUSE=0
- Outside-run wrapper: yes
- Prebuilt pull policy: always
- Compose project: beater-stopwatch
- Beater image reference: ghcr.io/jadenfix/beater/beaterd:__COMMIT_SHA__
- Dashboard image reference: ghcr.io/jadenfix/beater/dashboard:__COMMIT_SHA__
- Dashboard e2e image reference: ghcr.io/jadenfix/beater/dashboard-e2e:__COMMIT_SHA__
- OTEL Python image reference: ghcr.io/jadenfix/beater/otel-python:__COMMIT_SHA__
- Beater image digest: ghcr.io/jadenfix/beater/beaterd@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
- Dashboard image digest: ghcr.io/jadenfix/beater/dashboard@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
- Dashboard e2e image digest: ghcr.io/jadenfix/beater/dashboard-e2e@sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
- OTEL Python image digest: ghcr.io/jadenfix/beater/otel-python@sha256:abababababababababababababababababababababababababababababababab
- Quickstart snippet: examples/python/five_line_otel.py
- API endpoint: http://127.0.0.1:8080
- OTLP endpoint: http://127.0.0.1:4317
- Dashboard base: http://127.0.0.1:3000
- Quickstart release ID: gate2-__COMMIT_SHORT__-1780000000-12345
- Quickstart trace: 11111111111111111111111111111111
- Quickstart dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace=11111111111111111111111111111111
- Quickstart browser proof: passed
- All-kind nested trace: 22222222222222222222222222222222
- All-kind dashboard: http://127.0.0.1:3000/?tenant=demo&project=demo&environment=local&trace=22222222222222222222222222222222
- All-kind waterfall browser proof: passed
- Browser recording: passed
- Browser recording artifact: docs/demos/gate2-compose-browser-demo.webm
- Browser recording notes: docs/demos/gate2-compose-browser-demo.md
- Browser recording SHA256: 3dac802bc8f2db03406d0d76e4e1618ed5b516a2cf3d286589e1a588cf6e6534

## Compose Images

```text
CONTAINER                      REPOSITORY                          TAG              PLATFORM      IMAGE ID      SIZE     CREATED
beater-stopwatch-beaterd-1     ghcr.io/jadenfix/beater/beaterd     __COMMIT_SHA__   linux/arm64   bbbbbbbbbbbb  88.4MB   1 minute ago
beater-stopwatch-dashboard-1   ghcr.io/jadenfix/beater/dashboard   __COMMIT_SHA__   linux/arm64   cccccccccccc  99.2MB   1 minute ago
beater-stopwatch-dashboard-e2e-run-1 ghcr.io/jadenfix/beater/dashboard-e2e __COMMIT_SHA__ linux/arm64 eeeeeeeeeeee 132MB 1 minute ago
beater-stopwatch-otel-python-quickstart-run-1 ghcr.io/jadenfix/beater/otel-python __COMMIT_SHA__ linux/arm64 aaaaaaaaaaaa 116MB 1 minute ago
proof-image beaterd ghcr.io/jadenfix/beater/beaterd:__COMMIT_SHA__ ghcr.io/jadenfix/beater/beaterd@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
proof-image dashboard ghcr.io/jadenfix/beater/dashboard:__COMMIT_SHA__ ghcr.io/jadenfix/beater/dashboard@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
proof-image dashboard-e2e ghcr.io/jadenfix/beater/dashboard-e2e:__COMMIT_SHA__ ghcr.io/jadenfix/beater/dashboard-e2e@sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
proof-image otel-python ghcr.io/jadenfix/beater/otel-python:__COMMIT_SHA__ ghcr.io/jadenfix/beater/otel-python@sha256:abababababababababababababababababababababababababababababababab
```

This is an outside-run stopwatch source artifact generated through
`scripts/gate2-outside-run.sh`. Gate 2 closes only after the completed
outside-person proof, this stopwatch proof, the recording, and the recording
notes are committed and `scripts/validate-gate2-outside-proof.sh` passes.
EOF_PROOF
python3 - "$commit_sha" <<'PY'
from pathlib import Path
import sys
path = Path("docs/demos/gate2-compose-stopwatch.md")
path.write_text(path.read_text().replace("__COMMIT_SHA__", sys.argv[1]).replace("__COMMIT_SHORT__", sys.argv[1][:12]))
PY
echo "fixture outside wrapper runtime executed"
"#,
    )
    .unwrap_or_else(|err| panic!("write stopwatch env stub {}: {err}", stub.display()));
    let mut permissions = fs::metadata(&stub)
        .unwrap_or_else(|err| panic!("stat stopwatch env stub {}: {err}", stub.display()))
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&stub, permissions)
        .unwrap_or_else(|err| panic!("chmod stopwatch env stub {}: {err}", stub.display()));
}

fn write_outside_wrapper_fixture_repo(branch: &str) -> TempDir {
    let root = repo_root();
    let fixture = tempdir("create outside wrapper fixture repo");

    copy_fixture_file(&root, fixture.path(), "scripts/gate2-outside-run.sh");

    git_success(fixture.path(), &["init"]);
    git_success(
        fixture.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git_success(fixture.path(), &["config", "user.name", "Gate 2 Fixture"]);
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "fixture"]);
    git_success(fixture.path(), &["branch", "-M", branch]);
    git_success(
        fixture.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/jadenfix/beater.git",
        ],
    );
    fixture
}

fn copy_fixture_file(root: &Path, fixture_root: &Path, rel: &str) {
    let source = root.join(rel);
    let dest = fixture_root.join(rel);
    let parent = dest
        .parent()
        .unwrap_or_else(|| panic!("fixture destination should have parent: {}", dest.display()));
    fs::create_dir_all(parent)
        .unwrap_or_else(|err| panic!("create fixture dir {}: {err}", parent.display()));
    fs::copy(&source, &dest).unwrap_or_else(|err| {
        panic!(
            "copy fixture file {} -> {}: {err}",
            source.display(),
            dest.display()
        )
    });
}

fn git_success(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|err| panic!("run git {}: {err}", args.join(" ")));
    if !output.status.success() {
        panic!(
            "git {} failed\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn git_output(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|err| panic!("run git {}: {err}", args.join(" ")));
    if !output.status.success() {
        panic!(
            "git {} failed\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn current_head() -> String {
    let root = repo_root();
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("read current git HEAD: {err}"));
    if !output.status.success() {
        panic!(
            "git rev-parse HEAD failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn quickstart_release_id() -> String {
    let head = current_head();
    quickstart_release_id_for(&head)
}

fn quickstart_release_id_for(commit_sha: &str) -> String {
    format!("gate2-{}-1780000000-12345", &commit_sha[..12])
}

fn tempdir(context: &str) -> TempDir {
    TempDir::new().unwrap_or_else(|err| panic!("{context}: {err}"))
}
