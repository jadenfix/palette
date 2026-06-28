use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn root_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("Cargo.toml")
}

fn workspace_array(manifest: &str, key: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut in_workspace = false;
    let mut collecting = false;
    let mut values = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace = trimmed == "[workspace]";
            collecting = false;
            continue;
        }
        if !in_workspace {
            continue;
        }

        if !collecting {
            let Some(rest) = trimmed.strip_prefix(key) else {
                continue;
            };
            let Some(rest) = rest.trim_start().strip_prefix('=') else {
                continue;
            };
            let rest = rest.trim_start();
            if !rest.starts_with('[') {
                return Err(format!("workspace.{key} must be an inline or multiline array").into());
            }
            collecting = true;
            parse_array_line(rest, &mut values);
            if rest.contains(']') {
                return Ok(values);
            }
            continue;
        }

        parse_array_line(trimmed, &mut values);
        if trimmed.contains(']') {
            return Ok(values);
        }
    }

    Err(format!("missing [workspace].{key} array").into())
}

fn parse_array_line(line: &str, values: &mut Vec<String>) {
    let line = line.split('#').next().unwrap_or_default();
    for item in line.split(',') {
        let value = item
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim()
            .trim_matches('"');
        if !value.is_empty() {
            values.push(value.to_owned());
        }
    }
}

#[test]
fn generated_and_native_sdks_stay_out_of_core_workspace() -> Result<(), Box<dyn Error>> {
    let manifest = fs::read_to_string(root_manifest())?;
    let members = workspace_array(&manifest, "members")?;
    let excludes = workspace_array(&manifest, "exclude")?;

    assert!(
        excludes.iter().any(|path| path == "sdks"),
        "[workspace].exclude must include `sdks` so generated and hand-written SDK packages stay standalone"
    );
    assert!(
        members
            .iter()
            .all(|path| path != "sdks" && !path.starts_with("sdks/")),
        "sdks/* must not become core workspace members: {members:?}"
    );
    assert!(
        members.iter().all(|path| !path.ends_with("beater-sdk")),
        "the native Rust SDK belongs under sdks/rust, not as a beater-sdk workspace crate: {members:?}"
    );

    Ok(())
}
