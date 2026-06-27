//! Guard test for R1.2: the future service split is logical, not operational.
//!
//! The all-in-one `beaterd` is the only mandatory binary. Optional thin role
//! binaries exist but are gated behind the `thin-bins` cargo feature
//! (`required-features`) and are NOT compiled by default, so a self-host is
//! never forced into a multi-service deployment.

use std::path::PathBuf;

fn read(rel: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join(rel)).unwrap_or_else(|err| panic!("read {rel}: {err}"))
}

/// A thin role binary must declare an explicit `[[bin]]` block gated behind the
/// `thin-bins` feature via `required-features`.
fn assert_thin_bin_is_feature_gated(manifest: &str, name: &str) {
    let needle = format!("name = \"{name}\"");
    let start = manifest
        .find(&needle)
        .unwrap_or_else(|| panic!("Cargo.toml must declare a [[bin]] for {name}"));
    // The block continues until the next [[ or [ section header.
    let rest = &manifest[start..];
    let end = rest[1..].find("\n[").map(|i| i + 1).unwrap_or(rest.len());
    let block = &rest[..end];
    assert!(
        block.contains("required-features = [\"thin-bins\"]"),
        "thin bin {name} must be gated behind required-features = [\"thin-bins\"], got:\n{block}"
    );
}

#[test]
fn default_build_ships_only_the_all_in_one_beaterd() {
    let manifest = read("Cargo.toml");

    // The default feature set must be empty so nothing pulls in the thin bins.
    assert!(
        manifest.contains("default = []"),
        "beaterd default features must be empty so the all-in-one is the only default build"
    );
    assert!(
        manifest.contains("thin-bins = []"),
        "beaterd must declare an opt-in `thin-bins` feature"
    );

    // The optional thin role binaries must be feature-gated, never default.
    for thin in ["beater-ingestd", "beater-queryd"] {
        assert_thin_bin_is_feature_gated(&manifest, thin);
    }
}

#[test]
fn thin_role_sources_point_back_at_the_all_in_one() {
    // The thin bins are stubs that document the role and defer to `beaterd`,
    // so the supported, tested path stays the single-process server.
    for (src, role) in [
        ("src/bin/beater-ingestd.rs", "ingest"),
        ("src/bin/beater-queryd.rs", "query"),
    ] {
        let body = read(src);
        assert!(
            body.contains("R1.2"),
            "{src} should reference R1.2 (logical-not-operational split)"
        );
        assert!(
            body.contains("all-in-one `beaterd`"),
            "{src} ({role}) must point operators back at the all-in-one beaterd"
        );
        assert!(
            body.contains("thin-bins"),
            "{src} ({role}) must document that it is opt-in behind the thin-bins feature"
        );
    }
}

#[test]
fn default_compose_does_not_mandate_thin_bins_or_external_services() {
    // The default self-host compose path must run one Beater process, not a
    // mandatory 10-service split. External deps stay opt-in via profiles.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = root
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root is two levels above bins/beaterd");
    let compose = std::fs::read_to_string(repo_root.join("docker-compose.yml"))
        .expect("read docker-compose.yml");
    for thin in ["beater-ingestd", "beater-queryd"] {
        assert!(
            !compose.contains(thin),
            "default compose must not require the thin role binary {thin}"
        );
    }
    // postgres/nats/minio are opt-in (deps profile), proving no mandatory split.
    for dep in ["postgres", "nats", "minio"] {
        assert!(
            compose.contains(dep),
            "compose should still define optional {dep} service"
        );
    }
    assert!(
        compose.contains("profiles: [\"deps\"]"),
        "external services must stay opt-in behind the deps profile"
    );
}
