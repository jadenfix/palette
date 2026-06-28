//! Snapshot test: the /v1 route+method inventory in sdks/openapi/beater-api.json
//! must exactly match the committed golden file.
//!
//! If a route is added, removed, or renamed, the test fails with a clear diff
//! and instructions to regenerate the golden.
//!
//! To regenerate the golden file after a legitimate spec change:
//!   cargo test -p beater-api --test route_inventory -- --ignored update_golden

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Absolute path to `sdks/openapi/beater-api.json` (workspace root).
fn spec_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/beater-api  →  workspace root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("sdks/openapi/beater-api.json")
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/route_inventory.golden.txt")
}

/// Read the spec and return sorted "METHOD /v1/path" strings.
fn collect_routes() -> BTreeSet<String> {
    let path = spec_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let spec: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("invalid JSON in spec: {e}"));

    let mut routes = BTreeSet::new();
    if let Some(paths) = spec["paths"].as_object() {
        for (path, item) in paths {
            if !path.starts_with("/v1") {
                continue;
            }
            if let Some(item) = item.as_object() {
                for method in ["get", "post", "put", "delete", "patch"] {
                    if item.contains_key(method) {
                        routes.insert(format!("{} {}", method.to_uppercase(), path));
                    }
                }
            }
        }
    }
    routes
}

#[test]
fn route_inventory_matches_golden() {
    let routes = collect_routes();
    let actual: String = routes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n") + "\n";

    let golden_path = golden_path();
    let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|e| {
        panic!(
            "cannot read golden file {}: {e}\n\
             Generate it with:\n  \
             cargo test -p beater-api --test route_inventory -- --ignored update_golden",
            golden_path.display()
        )
    });

    if actual == golden {
        return;
    }

    let actual_set: BTreeSet<&str> = actual.lines().collect();
    let golden_set: BTreeSet<&str> = golden.lines().collect();

    let mut msg = String::from(
        "Route inventory has changed — regenerate the golden file:\n  \
         cargo test -p beater-api --test route_inventory -- --ignored update_golden\n\n",
    );
    for line in actual_set.difference(&golden_set) {
        msg.push_str(&format!("  ADDED:   {line}\n"));
    }
    for line in golden_set.difference(&actual_set) {
        msg.push_str(&format!("  REMOVED: {line}\n"));
    }
    panic!("{msg}");
}

/// Regenerate the golden file from the current spec.
///
/// Run with:
///   cargo test -p beater-api --test route_inventory -- --ignored update_golden
#[test]
#[ignore]
fn update_golden() {
    let routes = collect_routes();
    let content: String = routes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n") + "\n";
    let path = golden_path();
    std::fs::write(&path, &content)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    println!("Wrote {} routes to {}", routes.len(), path.display());
}
