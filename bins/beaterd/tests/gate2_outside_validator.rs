use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

const QUICKSTART_TRACE: &str = "11111111111111111111111111111111";
const ALL_KIND_TRACE: &str = "22222222222222222222222222222222";
const RECORDING_SHA: &str = "8dbf7556fbb705c3e00d5ec7604b3ce82482f7743f937d008d0913a96d0284dc";
const BEATER_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/beaterd@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DASHBOARD_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/dashboard@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const DASHBOARD_E2E_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/dashboard-e2e@sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const OTEL_PYTHON_IMAGE_DIGEST: &str =
    "ghcr.io/jadenfix/beater/otel-python@sha256:abababababababababababababababababababababababababababababababab";
const OUTSIDE_RUN_ATTESTATION: &str = "I attest that I am not a Beater project maintainer, I received no step-by-step help beyond public repository instructions, I used a fresh clone, and I completed the Gate 2 flow unaided.";

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

    assert_success(output, "Gate 2 outside-person proof is complete and valid");
}

#[test]
fn gate2_outside_generator_builds_valid_completed_proof() {
    let fixture = ValidatorFixture::new();
    let generated = fixture.dir.path().join("generated-outside-proof.md");

    let output = run_generator(&fixture.stopwatch_path, &generated);

    assert_success(output, "Wrote Gate 2 outside-person proof");
    assert_success(
        run_validator(&generated),
        "Gate 2 outside-person proof is complete and valid",
    );
    let generated_text = fs::read_to_string(&generated)
        .unwrap_or_else(|err| panic!("read {}: {err}", generated.display()));
    assert!(generated_text.contains("- Name: Validator Fixture Runner"));
    assert!(generated_text.contains(OUTSIDE_RUN_ATTESTATION));
    assert!(generated_text.contains("- API endpoint: http://127.0.0.1:8080"));
    assert!(generated_text.contains("- Dashboard base: http://127.0.0.1:3000"));
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
    assert!(generated_text.contains(
        r#"BEATER_GATE2_CLONE_STARTED_EPOCH="$BEATER_GATE2_CLONE_STARTED_EPOCH" scripts/gate2-outside-run.sh"#
    ));
    assert!(generated_text.contains("- Outside-run wrapper: yes"));
    assert!(generated_text.contains("- `docker compose images` excerpt:"));
    assert!(generated_text.contains("ghcr.io/jadenfix/beater/beaterd"));
    assert!(generated_text.contains("ghcr.io/jadenfix/beater/dashboard"));
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
    assert_success(
        run_validator(&pending),
        "Gate 2 outside-person proof is complete and valid",
    );
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
    assert!(script.contains("socket.create_connection"));
    assert!(script.contains("def port_owner_hint"));
    assert!(script.contains("lsof"));
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
}

#[test]
fn gate2_stopwatch_outside_next_steps_separate_dashboard_targets() {
    let script = fs::read_to_string(repo_root().join("scripts/gate2-compose-stopwatch.sh"))
        .unwrap_or_else(|err| panic!("read Gate 2 compose stopwatch script: {err}"));

    assert!(script.contains("Open $dashboard_url in a normal browser for the quickstart trace."));
    assert!(script
        .contains("Confirm prompt, completion, model, tokens, cost, and latency are visible."));
    assert!(script.contains(
        "Open ${all_kind_dashboard_url:-not requested} in a normal browser for the all-kind waterfall."
    ));
    assert!(script.contains("Confirm run -> turn -> step -> tool -> MCP nesting is visible."));
}

#[test]
fn gate2_outside_wrapper_real_run_executes_stopwatch_with_clone_timer() {
    let fixture = write_outside_wrapper_fixture_repo("main");
    write_stopwatch_env_stub(fixture.path());
    git_success(fixture.path(), &["add", "."]);
    git_success(fixture.path(), &["commit", "-m", "add stopwatch stub"]);

    let output = run_outside_wrapper_real_with_clone_timer_in_repo(fixture.path(), "1800000000");

    assert_success(output, "fixture outside wrapper runtime executed");
    let env_marker = fs::read_to_string(fixture.path().join("wrapper-real-env.txt"))
        .unwrap_or_else(|err| panic!("read outside wrapper runtime marker: {err}"));
    assert!(env_marker.contains("write=1"));
    assert!(env_marker.contains("browser=1"));
    assert!(env_marker.contains("record=1"));
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
fn gate2_outside_wrapper_rejects_alternate_dashboard_port() {
    let output = run_outside_wrapper_dry_run(Some(("BEATER_DASHBOARD_PORT", "13080")));

    assert_failure(output, "BEATER_DASHBOARD_PORT must be unset or '3000'");
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
fn gate2_outside_validator_rejects_first_trace_less_than_script_runtime() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.stopwatch_path,
        "- Time-to-first-trace: 12s",
        "- Time-to-first-trace: 6s",
    );

    let output = run_validator(&fixture.proof_path);

    assert_failure(
        output,
        "Time-to-first-trace must include at least the script runtime",
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
fn gate2_outside_validator_rejects_recording_notes_without_full_gate2_flow() {
    let fixture = ValidatorFixture::new();
    replace(
        &fixture.notes_path,
        "read prompt, completion, model, tokens, cost, and latency -> ",
        "",
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
- Timing start source: external-clone
- Clone started at: 2026-06-20T11:59:55Z
- Script started at: 2026-06-20T12:00:00Z
- Started at: 2026-06-20T12:00:00Z
- Ended at: 2026-06-20T12:00:40Z
- Time-to-first-trace: 12s
- Script-to-first-trace: 7s
- Time-to-quickstart-click: 20s
- Script-to-quickstart-click: 15s
- Total proof duration: 40s
- Script duration: 35s
- Outside-run wrapper: yes

## Commands

```bash
BEATER_GATE2_CLONE_STARTED_EPOCH="$(date +%s)"
git clone https://github.com/jadenfix/beater.git
cd beater
BEATER_GATE2_CLONE_STARTED_EPOCH="$BEATER_GATE2_CLONE_STARTED_EPOCH" scripts/gate2-outside-run.sh
```

The runner completed the flow using only public repository instructions.

## Required Evidence

- Stopwatch proof file: {stopwatch}
- Screen recording: `{recording}`
- Screen recording notes: `{notes}`
- Screen recording SHA256: {RECORDING_SHA}
- Terminal output excerpt: generated proof says browser recording passed
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
- [x] Time-to-quickstart-click was 300 seconds or less.
- [x] The five-line stock OpenTelemetry trace appeared in `localhost:3000`.
- [x] Clicking the `llm.call` span showed prompt, completion, model, tokens, cost, and latency.
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
        "- `docker compose images` excerpt: beater-stopwatch-beaterd-1 ghcr.io/jadenfix/beater/beaterd {commit_sha} | beater-stopwatch-dashboard-1 ghcr.io/jadenfix/beater/dashboard {commit_sha}\n"
    )
}

fn stopwatch_proof(recording: &str, notes: &str) -> String {
    let commit_sha = current_head();
    format!(
        r#"# Gate 2 Compose Stopwatch Proof

- Timing start source: external-clone
- Clone started at: 2026-06-20T11:59:55Z
- Script started at: 2026-06-20T12:00:00Z
- Started: 2026-06-20T12:00:00Z
- Ended: 2026-06-20T12:00:40Z
- Time-to-first-trace: 12s
- Script-to-first-trace: 7s
- Time-to-quickstart-click: 20s
- Script-to-quickstart-click: 15s
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
```
"#,
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
- Shows: open dashboard -> click five-line trace -> click `llm.call` span -> read prompt, completion, model, tokens, cost, and latency -> inspect run -> turn -> step -> tool -> MCP waterfall.
"#
    )
}

fn run_validator(proof_path: &Path) -> Output {
    run_validator_with_args(proof_path, &[])
}

fn run_validator_with_args(proof_path: &Path, args: &[&str]) -> Output {
    let root = repo_root();
    Command::new("bash")
        .arg(root.join("scripts/validate-gate2-outside-proof.sh"))
        .args(args)
        .current_dir(root)
        .env("BEATER_GATE2_OUTSIDE_PROOF", proof_path)
        .env("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS", "1")
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

fn run_generator(stopwatch_path: &Path, output_path: &Path) -> Output {
    run_generator_with_attestation(stopwatch_path, output_path, true)
}

fn run_generator_with_attestation(
    stopwatch_path: &Path,
    output_path: &Path,
    attest: bool,
) -> Output {
    run_generator_with_options(stopwatch_path, output_path, attest, true)
}

fn run_generator_without_network_notes(stopwatch_path: &Path, output_path: &Path) -> Output {
    run_generator_with_options(stopwatch_path, output_path, true, false)
}

fn run_generator_with_options(
    stopwatch_path: &Path,
    output_path: &Path,
    attest: bool,
    include_network_notes: bool,
) -> Output {
    let root = repo_root();
    let mut command = Command::new("python3");
    command
        .arg(root.join("scripts/generate-gate2-outside-proof.py"))
        .arg("--stopwatch-proof")
        .arg(stopwatch_path)
        .arg("--output")
        .arg(output_path)
        .arg("--runner-name")
        .arg("Validator Fixture Runner")
        .arg("--relationship")
        .arg("external validation fixture")
        .arg("--prior-exposure")
        .arg("no prior exposure")
        .arg("--machine-os")
        .arg("macOS arm64")
        .arg("--browser")
        .arg("Chromium")
        .arg("--preflight-status")
        .arg("passed")
        .arg("--date")
        .arg("2026-06-20")
        .arg("--terminal-output-excerpt")
        .arg("generated proof says browser recording passed")
        .arg("--compose-logs-saved")
        .arg("temp fixture")
        .arg("--failure-notes")
        .arg("none")
        .arg("--runner-notes")
        .arg("No extra runner notes.")
        .current_dir(root)
        .env("BEATER_GATE2_ALLOW_UNTRACKED_ARTIFACTS", "1");
    if attest {
        command.arg("--attest-outside-run");
    }
    if include_network_notes {
        command.arg("--network-notes").arg("public docs only");
    }
    command
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside proof generator: {err}"))
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
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 public handoff checker: {err}"))
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

fn python3_executable() -> PathBuf {
    let output = Command::new("python3")
        .arg("-c")
        .arg("import sys; print(sys.executable)")
        .output()
        .unwrap_or_else(|err| panic!("resolve python3 executable: {err}"));
    if !output.status.success() {
        panic!(
            "resolve python3 executable failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
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
    let mut command = Command::new("bash");
    command
        .arg(repo.join("scripts/gate2-outside-run.sh"))
        .current_dir(repo);
    clear_outside_env(&mut command);
    command.env("BEATER_GATE2_OUTSIDE_RUN_DRY_RUN", "1");
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
    let mut command = Command::new("bash");
    command
        .arg(repo.join("scripts/gate2-outside-run.sh"))
        .current_dir(repo);
    clear_outside_env(&mut command);
    command
        .env("BEATER_GATE2_CLONE_STARTED_EPOCH", clone_started)
        .output()
        .unwrap_or_else(|err| panic!("run Gate 2 outside wrapper fixture real run: {err}"))
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
        "BEATERD_IMAGE",
        "BEATER_DASHBOARD_IMAGE",
        "BEATER_DASHBOARD_E2E_IMAGE",
        "BEATER_OTEL_PYTHON_IMAGE",
        "BEATER_GATE2_STOPWATCH_PROOF",
        "BEATER_GATE2_RECORD_VIDEO",
        "BEATER_GATE2_RECORD_NOTES",
        "KEEP_BEATER_COMPOSE",
        "COMPOSE_PROJECT_NAME",
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
    let manifests = platforms
        .iter()
        .map(|platform| {
            let (os, architecture) = platform
                .split_once('/')
                .unwrap_or_else(|| panic!("invalid platform fixture: {platform}"));
            format!(r#"{{"platform":{{"os":"{os}","architecture":"{architecture}"}}}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        dir.join(format!("{image}.json")),
        format!(r#"{{"manifests":[{manifests}]}}"#),
    )
    .unwrap_or_else(|err| panic!("write registry fixture for {image}: {err}"));
}

fn recording_bytes() -> Vec<u8> {
    let mut bytes =
        bytes_from_hex("1a45dfa39f4286810142f7810142f2810442f381084282847765626d4287810242858102");
    bytes.resize(70_000, 0);
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
        "docker-compose.yml",
        "docker-compose.prebuilt.yml",
        "docs/demos/gate2-outside-person-proof.md",
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

fn write_stopwatch_env_stub(repo: &Path) {
    let stub = repo.join("scripts/gate2-compose-stopwatch.sh");
    fs::write(
        &stub,
        r#"#!/usr/bin/env bash
set -euo pipefail
{
  echo "write=${BEATER_GATE2_WRITE_PROOF:-unset}"
  echo "browser=${BEATER_GATE2_BROWSER_PROOF:-unset}"
  echo "record=${BEATER_GATE2_RECORD_DEMO:-unset}"
  echo "reuse=${BEATER_GATE2_REUSE:-unset}"
  echo "local_build=${BEATER_GATE2_LOCAL_BUILD:-unset}"
  echo "pull_policy=${BEATER_GATE2_PULL_POLICY:-unset}"
  echo "keep=${KEEP_BEATER_COMPOSE:-unset}"
  echo "outside_wrapper=${BEATER_GATE2_OUTSIDE_WRAPPER:-unset}"
  echo "dry=${BEATER_GATE2_OUTSIDE_RUN_DRY_RUN:-unset}"
  echo "expected_origin=${BEATER_GATE2_EXPECTED_ORIGIN:-unset}"
  echo "clone_started=${BEATER_GATE2_CLONE_STARTED_EPOCH:-unset}"
  echo "dashboard_port=${BEATER_DASHBOARD_PORT:-unset}"
} > wrapper-real-env.txt
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

fn tempdir(context: &str) -> TempDir {
    TempDir::new().unwrap_or_else(|err| panic!("{context}: {err}"))
}
