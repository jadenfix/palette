//! Snapshot test: the /v1 route+method inventory in sdks/openapi/beater-api.json
//! must exactly match the committed golden file.
//!
//! If a route is added, removed, or renamed, the test fails with a clear diff
//! and instructions to regenerate the golden.
//!
//! To regenerate the golden file after a legitimate spec change:
//!   cargo test -p beater-api --test route_inventory -- --ignored update_golden

use std::collections::BTreeSet;
#[cfg(not(feature = "billing"))]
use std::path::PathBuf;
#[cfg(not(feature = "billing"))]
use std::sync::Arc;

#[cfg(not(feature = "billing"))]
use axum::body::Body;
#[cfg(not(feature = "billing"))]
use beater_api::{router, ApiState};
#[cfg(not(feature = "billing"))]
use beater_bus::InMemoryBus;
#[cfg(not(feature = "billing"))]
use beater_ingest::{IngestPolicy, IngestService};
#[cfg(not(feature = "billing"))]
use beater_store_obj::FsArtifactStore;
#[cfg(not(feature = "billing"))]
use beater_store_sql::SqliteTraceStore;
#[cfg(not(feature = "billing"))]
use http::{Request, StatusCode};
#[cfg(not(feature = "billing"))]
use tower::ServiceExt;

const HOSTED_BILLING_ROUTES: &[&str] = &[
    "GET /v1/plans",
    "GET /v1/plans/{plan_id}",
    "POST /v1/subscriptions/{org_id}",
    "GET /v1/subscriptions/{org_id}",
    "POST /v1/subscriptions/{org_id}/change-plan",
    "GET /v1/billing/invoices/{org_id}",
    "GET /v1/billing/invoices/{org_id}/{period_key}",
    "POST /v1/billing/webhooks/stripe",
];

/// Absolute path to `sdks/openapi/beater-api.json` (workspace root).
#[cfg(not(feature = "billing"))]
fn spec_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/beater-api  →  workspace root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("sdks/openapi/beater-api.json")
}

#[cfg(not(feature = "billing"))]
fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/route_inventory.golden.txt")
}

#[cfg(not(feature = "billing"))]
fn collect_routes_from_spec_text(text: &str) -> BTreeSet<String> {
    let spec: serde_json::Value =
        serde_json::from_str(text).unwrap_or_else(|e| panic!("invalid JSON in spec: {e}"));
    collect_routes_from_spec_value(&spec)
}

fn collect_routes_from_spec_value(spec: &serde_json::Value) -> BTreeSet<String> {
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

/// Read the committed default spec snapshot and return sorted "METHOD /v1/path"
/// strings.
#[cfg(not(feature = "billing"))]
fn collect_routes() -> BTreeSet<String> {
    let path = spec_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    collect_routes_from_spec_text(&text)
}

/// Collect routes from the feature-aware in-process OpenAPI generator.
#[cfg(feature = "billing")]
fn collect_current_openapi_routes() -> BTreeSet<String> {
    let spec = serde_json::to_value(beater_api::openapi::openapi())
        .unwrap_or_else(|e| panic!("serialize generated OpenAPI: {e}"));
    collect_routes_from_spec_value(&spec)
}

#[cfg(not(feature = "billing"))]
fn default_router() -> (axum::Router, tempfile::TempDir) {
    let tempdir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|e| panic!("artifact store: {e}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|e| panic!("{e}")));
    let bus = Arc::new(InMemoryBus::new(8));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    (router(ApiState::new(ingest, traces)), tempdir)
}

#[test]
#[cfg(not(feature = "billing"))]
fn route_inventory_matches_golden() {
    let routes = collect_routes();
    let actual: String = routes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

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

#[test]
#[cfg(not(feature = "billing"))]
fn api_key_routes_do_not_expose_read_surfaces() {
    let routes = collect_routes();
    let api_key_routes = routes
        .iter()
        .filter(|route| route.contains("/v1/api-keys/"))
        .cloned()
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        "POST /v1/api-keys/{tenant_id}/{project_id}/{environment_id}".to_string(),
        "POST /v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke"
            .to_string(),
    ]);

    assert_eq!(
        api_key_routes, expected,
        "API-key read/list routes need an explicit secret_hash response review"
    );
}

#[test]
#[cfg(not(feature = "billing"))]
fn default_route_inventory_excludes_hosted_billing_surfaces() {
    let routes = collect_routes();
    for route in HOSTED_BILLING_ROUTES {
        assert!(
            !routes.contains(*route),
            "default OSS route inventory must not include hosted billing route {route}",
        );
    }
}

#[tokio::test]
#[cfg(not(feature = "billing"))]
async fn default_router_does_not_register_hosted_billing_surfaces() {
    let (app, _tempdir) = default_router();

    for (method, path) in [
        ("GET", "/v1/plans"),
        ("GET", "/v1/plans/pro"),
        ("GET", "/v1/subscriptions/org-1"),
        ("POST", "/v1/subscriptions/org-1"),
        ("POST", "/v1/subscriptions/org-1/change-plan"),
        ("GET", "/v1/billing/invoices/org-1"),
        ("GET", "/v1/billing/invoices/org-1/2026-07"),
        ("POST", "/v1/billing/webhooks/stripe"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("build request {method} {path}: {e}")),
            )
            .await
            .unwrap_or_else(|e| panic!("route request {method} {path}: {e}"));
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "default OSS router must not register hosted billing route {method} {path}",
        );
    }
}

#[test]
#[cfg(feature = "billing")]
fn billing_feature_openapi_route_inventory_includes_hosted_billing_surfaces() {
    let routes = collect_current_openapi_routes();
    for route in HOSTED_BILLING_ROUTES {
        assert!(
            routes.contains(*route),
            "billing feature OpenAPI route inventory must include hosted billing route {route}",
        );
    }
}

/// Regenerate the golden file from the current spec.
///
/// Run with:
///   cargo test -p beater-api --test route_inventory -- --ignored update_golden
#[test]
#[cfg(not(feature = "billing"))]
#[ignore]
fn update_golden() {
    let routes = collect_routes();
    let content: String = routes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let path = golden_path();
    std::fs::write(&path, &content)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    println!("Wrote {} routes to {}", routes.len(), path.display());
}
