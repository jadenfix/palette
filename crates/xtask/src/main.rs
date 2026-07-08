//! Workspace automation tasks, invoked via `cargo xtask <command>`.
//!
//! The project standard is `cargo xtask` over ad-hoc shell scripts. This crate
//! owns the OpenAPI spec / SDK regeneration pipeline:
//!
//! * `regen-spec`: regenerate `sdks/openapi/beater-api.json`, the dashboard
//!   snapshot, AND the dashboard's typed client
//!   (`web/dashboard/lib/generated/api-types.ts`) directly from the `beater-api`
//!   handlers — every spec-derived artifact in one step, so nothing the CI drift
//!   checks enforce is left stale. The dashboard client step needs Node/npx.
//! * `regen-sdks`: regenerate the spec *and* every language client by shelling
//!   out to `scripts/regen-sdks.sh` (needs Docker).
//! * `check-drift`: regenerate the spec into a temp file and `git diff` the
//!   committed spec/dashboard snapshot, failing if stale. This is the fast,
//!   Docker-free drift check; the full check is `cargo xtask regen-sdks`
//!   followed by `git diff`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

/// Canonical spec location, relative to the workspace root.
const SPEC_PATH: &str = "sdks/openapi/beater-api.json";
/// Cross-language semantic-conventions contract, regenerated from beater-schema.
const SEMCONV_PATH: &str = "sdks/semconv/conventions.json";
/// Dashboard snapshot that must stay byte-identical to the canonical spec.
const DASHBOARD_SPEC_PATH: &str = "web/dashboard/openapi/beater-read-api.json";

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Beater workspace automation tasks")]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Regenerate the OpenAPI spec from the beater-api handlers (no Docker).
    RegenSpec,
    /// Regenerate the spec and every language SDK client (requires Docker).
    RegenSdks,
    /// Fail if the committed spec is stale versus the handlers (no Docker).
    CheckDrift,
    /// Regenerate the cross-language semconv contract from beater-schema.
    RegenSemconv,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Cmd::RegenSpec => regen_spec(),
        Cmd::RegenSdks => regen_sdks(),
        Cmd::CheckDrift => check_drift(),
        Cmd::RegenSemconv => regen_semconv(),
    }
}

/// Write `sdks/semconv/conventions.json` from `beater_schema::conventions`, the
/// single source of truth for span kinds + attribute keys. CI regenerates this
/// and fails on `git diff`, so the server contract can't drift from the file the
/// SDK drift checker validates against.
fn regen_semconv() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let path = root.join(SEMCONV_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create semconv dir {}", parent.display()))?;
    }
    std::fs::write(&path, beater_schema::conventions::conventions_json())
        .with_context(|| format!("write semconv {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

/// Locate the workspace root from this crate's manifest directory.
///
/// `crates/xtask` is two levels below the workspace root.
fn workspace_root() -> anyhow::Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .context("locate workspace root from xtask manifest dir")
}

/// Render the canonical OpenAPI document directly from the handlers.
fn render_spec() -> anyhow::Result<String> {
    let mut json =
        beater_api::openapi::openapi_json_pretty().context("render OpenAPI spec to JSON")?;
    // Match the trailing newline that shell redirection (`> file`) produced for
    // the committed artifact so drift checks compare equal.
    json.push('\n');
    Ok(json)
}

fn regen_spec() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let spec = render_spec()?;
    let spec_path = root.join(SPEC_PATH);
    let dashboard_path = root.join(DASHBOARD_SPEC_PATH);

    if let Some(parent) = spec_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create spec dir {}", parent.display()))?;
    }
    if let Some(parent) = dashboard_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dashboard spec dir {}", parent.display()))?;
    }

    std::fs::write(&spec_path, &spec)
        .with_context(|| format!("write spec {}", spec_path.display()))?;
    std::fs::write(&dashboard_path, &spec)
        .with_context(|| format!("write dashboard spec {}", dashboard_path.display()))?;

    println!("wrote {}", spec_path.display());
    println!("wrote {}", dashboard_path.display());

    // The dashboard's typed client is a THIRD spec-derived artifact. Regenerate
    // it here so `regen-spec` leaves nothing stale — otherwise CI's
    // `check-openapi-drift.sh` (the `verify` job) flags drift even after the
    // documented regen steps were run.
    regen_dashboard_client(&root)?;
    Ok(())
}

/// Regenerate `web/dashboard/lib/generated/api-types.ts` from the dashboard spec
/// snapshot using the exact `openapi-typescript` version pinned in the
/// dashboard's `package.json` (so the output matches CI byte-for-byte). `npx
/// --yes` fetches that version on demand, so a prior `npm ci` is not required.
fn regen_dashboard_client(root: &Path) -> anyhow::Result<()> {
    let dashboard = root.join("web/dashboard");
    let version = openapi_typescript_version(&dashboard)?;
    let out_rel = "lib/generated/api-types.ts";
    let status = Command::new("npx")
        .args([
            "--yes",
            &format!("openapi-typescript@{version}"),
            "openapi/beater-read-api.json",
            "-o",
            out_rel,
        ])
        .current_dir(&dashboard)
        .status()
        .with_context(
            || "run `npx openapi-typescript` in web/dashboard (is Node/npm installed?)",
        )?;
    if !status.success() {
        bail!("dashboard api-types.ts generation failed (openapi-typescript@{version})");
    }
    println!("wrote {}", dashboard.join(out_rel).display());
    Ok(())
}

/// Read the pinned `openapi-typescript` version from the dashboard's
/// `package.json` devDependencies without taking a JSON parser dependency.
fn openapi_typescript_version(dashboard: &Path) -> anyhow::Result<String> {
    let pkg = std::fs::read_to_string(dashboard.join("package.json"))
        .with_context(|| "read web/dashboard/package.json")?;
    for line in pkg.lines() {
        if let Some(rest) = line.trim().strip_prefix("\"openapi-typescript\":") {
            let version: String = rest
                .trim()
                .trim_start_matches('"')
                .chars()
                .take_while(|ch| *ch != '"')
                .collect();
            let version = version.trim_start_matches(['^', '~']).to_string();
            if !version.is_empty() {
                return Ok(version);
            }
        }
    }
    bail!("openapi-typescript is not pinned in web/dashboard/package.json devDependencies")
}

fn regen_sdks() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let script = root.join("scripts/regen-sdks.sh");
    let status = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .status()
        .with_context(|| format!("run {}", script.display()))?;
    if !status.success() {
        bail!(
            "{} failed with {}",
            script.display(),
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".to_string())
        );
    }
    Ok(())
}

fn check_drift() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let fresh = render_spec()?;

    // Render the canonical spec into a temp file, then `git diff --no-index`
    // each committed artifact against it. `--no-index` gives us a real diff
    // (and a clean non-zero exit) regardless of whether the on-disk file is
    // tracked, staged, or untracked -- we only care that the *content* the
    // handlers produce matches what is checked in.
    let dir = std::env::temp_dir().join(format!("beater-xtask-drift-{}", std::process::id()));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create temp drift dir {}", dir.display()))?;
    let result = check_drift_inner(&root, &fresh, &dir);
    // Best-effort cleanup; ignore failures so they never mask the real result.
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn check_drift_inner(root: &Path, fresh: &str, dir: &Path) -> anyhow::Result<()> {
    let temp_spec = dir.join("beater-api.json");
    std::fs::write(&temp_spec, fresh)
        .with_context(|| format!("write temp spec {}", temp_spec.display()))?;

    for relative in [SPEC_PATH, DASHBOARD_SPEC_PATH] {
        let committed = root.join(relative);
        // `git diff --no-index --exit-code` returns 1 when the files differ.
        let status = Command::new("git")
            .args(["diff", "--no-index", "--exit-code", "--"])
            .arg(&committed)
            .arg(&temp_spec)
            .current_dir(root)
            .status()
            .with_context(|| format!("run git diff for {relative}"))?;
        if !status.success() {
            bail!(
                "OpenAPI spec drift: {relative} is stale versus the beater-api handlers.\n\
                 Run `cargo xtask regen-spec` (and `cargo xtask regen-sdks` for the language \
                 clients) and commit. The full check is `cargo xtask regen-sdks` followed by \
                 `git diff` over sdks/clients."
            );
        }
    }

    println!("no drift: OpenAPI spec is current");
    Ok(())
}
