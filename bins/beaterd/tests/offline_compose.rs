//! Offline self-host test for R1.3: OSS runs without Beater Cloud.
//!
//! Asserts that the default compose topology makes **no outbound calls except to
//! configured providers**: every wired endpoint resolves to a container-local
//! service or a `127.0.0.1` health probe, nothing points at a public Beater
//! Cloud / telemetry host, and the only opt-in outbound surface is the
//! operator-configured OTLP/LLM provider endpoint.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| panic!("repo root is two levels above bins/beaterd"))
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// Hosts that would indicate a self-host phoning home to Beater Cloud. None of
/// these may appear anywhere in the default compose file.
const FORBIDDEN_OUTBOUND_HOSTS: &[&str] = &[
    "api.beater.dev",
    "app.beater.dev",
    "cloud.beater.dev",
    "telemetry.beater.dev",
    "beater.cloud",
];

#[test]
fn default_compose_makes_no_outbound_calls_except_configured_providers() {
    let root = repo_root();
    let compose = read(&root.join("docker-compose.yml"));

    // 1. No service points at a Beater Cloud / telemetry host.
    for host in FORBIDDEN_OUTBOUND_HOSTS {
        assert!(
            !compose.contains(host),
            "offline self-host compose must not reference Beater Cloud host {host}"
        );
    }

    // 2. Every wired URL is container-local (service DNS) or a loopback probe.
    //    The OTLP exporter endpoint the example tools use targets `beaterd`, not
    //    an external collector.
    for local in [
        "http://beaterd:8080",
        "http://beaterd:4317",
        "OTEL_EXPORTER_OTLP_ENDPOINT: http://beaterd:4317",
    ] {
        assert!(
            compose.contains(local),
            "compose must wire internal traffic to {local}"
        );
    }

    // 3. External deps (postgres/nats/minio/clickhouse) stay opt-in behind
    //    profiles, so the default `up` path is one Beater process with no extra
    //    network services to reach.
    for service in ["postgres", "nats", "minio"] {
        let block = compose_service_block(&compose, service);
        assert!(
            block.contains("profiles: [\"deps\"]"),
            "{service} must be opt-in (deps profile) so default run stays self-contained"
        );
    }
    assert!(
        compose_service_block(&compose, "clickhouse").contains("profiles: [\"clickhouse\"]"),
        "clickhouse must be opt-in (clickhouse profile)"
    );

    // 4. beaterd does not depend on any external service in the default path.
    assert!(
        !compose_service_block(&compose, "beaterd").contains("depends_on:"),
        "beaterd must not depend on external services in the offline default path"
    );
}

#[test]
fn beaterd_self_host_telemetry_is_opt_out_so_no_outbound_call_by_default() {
    // The single source of truth for the telemetry posture (R12.5) defaults to
    // disabled, which is what makes the offline default genuinely offline: no
    // telemetry endpoint is contacted unless the operator opts in.
    let default = beater_core::SelfHostTelemetryConfig::default();
    assert!(!default.is_enabled());
    assert_eq!(default.endpoint(), None);

    // And nothing in compose silently opts it in.
    let compose = read(&repo_root().join("docker-compose.yml"));
    assert!(
        !compose.contains("BEATER_SELF_HOST_TELEMETRY"),
        "default compose must not enable self-host telemetry"
    );
}

#[test]
fn smoke_compose_script_keeps_runtime_loop_local() {
    // §22.2 / §24.1: the containerized self-host smoke loop must exercise the
    // local compose stack without introducing Beater Cloud or deploy surfaces.
    let script = read(&repo_root().join("scripts/smoke-compose.sh"));

    for local_probe in [
        "api_url=\"http://127.0.0.1:$host_http_port\"",
        "dashboard_url=\"http://127.0.0.1:$host_dashboard_port\"",
        "wait_url \"$api_url/health\" \"beaterd\"",
        "wait_url \"$dashboard_url/?tenant=demo&project=demo&environment=local\" \"dashboard\"",
    ] {
        assert!(
            script.contains(local_probe),
            "smoke-compose.sh must keep host probes on local loopback: {local_probe}"
        );
    }

    for compose_step in [
        "compose up -d --build beaterd dashboard",
        "compose run --rm beaterctl",
        "compose run --rm otel-python-smoke",
    ] {
        assert!(
            script.contains(compose_step),
            "smoke-compose.sh must exercise compose service {compose_step}"
        );
    }

    let lower_script = script.to_ascii_lowercase();
    for forbidden in [
        "BEATER_SELF_HOST_TELEMETRY",
        "beater.cloud",
        "beater.dev",
        "vercel",
        "docker push",
        "docker login",
        "docker compose --profile",
    ] {
        assert!(
            !lower_script.contains(&forbidden.to_ascii_lowercase()),
            "smoke-compose.sh must not reference outbound/deploy surface {forbidden}"
        );
    }
}

#[test]
fn offline_self_host_doc_documents_the_provider_only_egress() {
    // The offline posture is documented for operators who firewall egress.
    let doc = read(&repo_root().join("docs/offline-self-host.md"));
    for needle in [
        "R1.3",
        "no outbound",
        "configured providers",
        "BEATER_SELF_HOST_TELEMETRY",
    ] {
        assert!(
            doc.contains(needle),
            "docs/offline-self-host.md must mention {needle}"
        );
    }
}

/// Return the text of a single compose service block, from its `name:` header to
/// the next top-level (2-space-indented) service header.
fn compose_service_block<'a>(compose: &'a str, service: &str) -> &'a str {
    let header = format!("\n  {service}:");
    let start = compose
        .find(&header)
        .unwrap_or_else(|| panic!("docker-compose.yml must define service {service}"));
    let rest = &compose[start + 1..];
    // Find the next top-level service header (two-space indent, then a word).
    let mut idx = 0;
    let bytes = rest.as_bytes();
    while idx < rest.len() {
        if let Some(nl) = rest[idx..].find('\n') {
            let line_start = idx + nl + 1;
            if line_start < rest.len()
                && rest[line_start..].starts_with("  ")
                && bytes
                    .get(line_start + 2)
                    .is_some_and(|c| c.is_ascii_alphabetic())
            {
                return &rest[..line_start];
            }
            idx = line_start;
        } else {
            break;
        }
    }
    rest
}
