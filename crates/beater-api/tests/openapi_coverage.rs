//! Coverage guard that ties the OpenAPI spec to the real router.
//!
//! 1. Every documented path+method is reachable on the real router (not 404).
//! 2. The number of documented `/v1` operations equals the number of `/v1`
//!    routes registered in `router()` (via [`beater_api::v1_route_count`]).

use std::sync::Arc;

use beater_api::{ApiState, router, v1_route_count};
use beater_audit::SqliteAuditStore;
use beater_auth::SqliteApiKeyStore;
use beater_bus::InMemoryBus;
use beater_calibration::SqliteCalibrationStore;
use beater_core::Money;
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_ingest::{IngestPolicy, IngestService};
use beater_judge::{JudgeBrokerService, KeywordJudgeProvider, SqliteJudgeLedger};
use beater_prompts::SqlitePromptRegistry;
use beater_search::TantivySearchIndex;
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use beater_usage::SqliteUsageLedger;

use axum::Router;
use axum::body::{Body, to_bytes};
use beater_archive::ParquetTraceArchive;
use http::{Request, StatusCode};
use tower::ServiceExt;

fn unwrap<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("test setup failed: {err}"),
    }
}

/// Build a fully-integrated in-memory `ApiState` so that every documented route
/// is wired (otherwise an un-wired feature would 503/501, never 404 — but we
/// want the strongest possible "route exists and runs" assertion).
fn build_app() -> (Router, tempfile::TempDir) {
    let tempdir = unwrap(tempfile::tempdir());
    let artifacts = Arc::new(unwrap(FsArtifactStore::new(
        tempdir.path().join("artifacts"),
    )));
    let traces = Arc::new(unwrap(SqliteTraceStore::in_memory()));
    let search = Arc::new(unwrap(TantivySearchIndex::in_memory()));
    let archive = unwrap(ParquetTraceArchive::new(tempdir.path().join("archive")));
    let datasets = Arc::new(unwrap(SqliteDatasetStore::in_memory()));
    let experiments = Arc::new(unwrap(SqliteExperimentStore::in_memory()));
    let gates = Arc::new(unwrap(SqliteGateStore::in_memory()));
    let human_reviews = Arc::new(unwrap(SqliteHumanReviewStore::in_memory()));
    let calibrations = Arc::new(unwrap(SqliteCalibrationStore::in_memory()));
    let usage = Arc::new(unwrap(SqliteUsageLedger::in_memory()));
    let audit = Arc::new(unwrap(SqliteAuditStore::in_memory()));
    let api_keys = Arc::new(unwrap(SqliteApiKeyStore::in_memory()));
    let provider_secrets = Arc::new(unwrap(EncryptedSqliteProviderSecretStore::in_memory(
        unwrap(SecretKeyring::generated_for_tests()),
    )));
    let judge_ledger = Arc::new(unwrap(SqliteJudgeLedger::in_memory()));
    let judge_broker = Arc::new(JudgeBrokerService::new(
        provider_secrets.clone(),
        judge_ledger.clone(),
        KeywordJudgeProvider::new(Money::usd_micros(25)),
        Money::usd_micros(100),
    ));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());

    let state = ApiState::with_integrations(ingest, traces, search, archive, datasets, experiments)
        .with_gates(gates)
        .with_human_reviews(human_reviews)
        .with_calibrations(calibrations)
        .with_usage(usage)
        .with_audit(audit)
        .with_prompts(Arc::new(unwrap(SqlitePromptRegistry::in_memory())))
        .require_auth(api_keys)
        .with_judge(provider_secrets, judge_broker, judge_ledger);

    (router(state), tempdir)
}

/// Collect every (method, path) operation in the spec.
fn spec_operations() -> Vec<(String, String)> {
    let spec = beater_api::openapi::openapi();
    let mut ops = Vec::new();
    for (path, item) in spec.paths.paths.iter() {
        if item.get.is_some() {
            ops.push(("GET".to_string(), path.clone()));
        }
        if item.post.is_some() {
            ops.push(("POST".to_string(), path.clone()));
        }
        if item.put.is_some() {
            ops.push(("PUT".to_string(), path.clone()));
        }
        if item.delete.is_some() {
            ops.push(("DELETE".to_string(), path.clone()));
        }
        if item.patch.is_some() {
            ops.push(("PATCH".to_string(), path.clone()));
        }
    }
    ops
}

/// Replace `{param}` placeholders with a concrete value so the path matches a
/// real axum route.
fn concretize(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            // skip until '}'
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
            }
            out.push_str("placeholder");
        } else {
            out.push(c);
        }
    }
    out
}

#[tokio::test]
async fn every_documented_route_is_reachable() {
    let (app, _tempdir) = build_app();
    let ops = spec_operations();
    assert!(!ops.is_empty(), "spec produced no operations");

    for (method, path) in &ops {
        let uri = concretize(path);
        let body = if method == "POST" || method == "PUT" || method == "PATCH" {
            // Send an empty JSON object so handlers reach auth/validation rather
            // than failing body extraction with a 4xx that still proves routing.
            Body::from("{}")
        } else {
            Body::empty()
        };
        let request = unwrap(
            Request::builder()
                .method(method.as_str())
                .uri(&uri)
                .header("content-type", "application/json")
                .body(body),
        );
        let response = unwrap(app.clone().oneshot(request).await);
        let status = response.status();
        // Drain the body to avoid leaking connections in the test harness.
        let _ = to_bytes(response.into_body(), 1024 * 1024).await;
        assert_ne!(
            status,
            StatusCode::NOT_FOUND,
            "documented route {method} {path} (as {uri}) returned 404 — it is not wired in router()",
        );
        assert_ne!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "documented route {method} {path} (as {uri}) returned 405 — method mismatch with router()",
        );
    }
}

#[tokio::test]
async fn documented_v1_operation_count_matches_router() {
    let ops = spec_operations();
    let v1 = ops.iter().filter(|(_, p)| p.starts_with("/v1")).count();
    assert_eq!(
        v1,
        v1_route_count(),
        "spec documents {v1} /v1 operations but router() registers {} — update V1_ROUTE_COUNT or the spec paths(...) list",
        v1_route_count(),
    );
}

#[test]
fn health_is_documented() {
    let spec = beater_api::openapi::openapi();
    assert!(
        spec.paths.paths.contains_key("/health"),
        "/health must be documented",
    );
}

#[test]
fn default_openapi_does_not_advertise_hosted_billing_paths() {
    let spec = beater_api::openapi::openapi();
    let billing_paths: Vec<_> = spec
        .paths
        .paths
        .keys()
        .filter(|path| path.starts_with("/v1/billing") || path.starts_with("/v1/plans"))
        .collect();

    assert!(
        billing_paths.is_empty(),
        "default OSS OpenAPI must not advertise hosted billing paths: {billing_paths:?}",
    );
}
