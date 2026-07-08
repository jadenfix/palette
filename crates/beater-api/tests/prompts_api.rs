//! End-to-end coverage for the `/v1/prompts` surface, driving the real
//! `beater-api` router with an `InMemoryPromptRegistry` wired into app state.
//!
//! This is the live consumer that makes `beater-prompts` a real feature: every
//! assertion goes through the same `router()` beaterd serves.

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use beater_api::{ApiState, router};
use beater_bus::InMemoryBus;
use beater_ingest::{IngestPolicy, IngestService};
use beater_prompts::InMemoryPromptRegistry;
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

fn build_app() -> (Router, tempfile::TempDir) {
    let tempdir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let artifacts = Arc::new(
        FsArtifactStore::new(tempdir.path().join("artifacts"))
            .unwrap_or_else(|err| panic!("{err}")),
    );
    let traces = Arc::new(SqliteTraceStore::in_memory().unwrap_or_else(|err| panic!("{err}")));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let prompts = Arc::new(InMemoryPromptRegistry::new());
    let state = ApiState::new(ingest, traces).with_prompts(prompts);
    (router(state), tempdir)
}

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = match body {
        Some(payload) => Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap_or_else(|err| panic!("{err}")),
        None => Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap_or_else(|err| panic!("{err}")),
    };
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("{err}"));
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|err| panic!("{err}"))
    };
    (status, value)
}

fn str_field<'a>(value: &'a Value, pointer: &str) -> &'a str {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string at {pointer} in {value}"))
}

#[tokio::test]
async fn prompts_lifecycle_create_version_list_and_diff() {
    let (app, _tempdir) = build_app();
    let base = "/v1/prompts/tenant-a/project-a";

    // create_prompt -> initial version 1
    let (status, created) = send(
        &app,
        "POST",
        base,
        Some(json!({
            "name": "answer-support-question",
            "description": "Support prompt",
            "template": {
                "body": "system\nanswer briefly",
                "variables": [{"name": "question", "required": true, "default": null, "description": null}],
                "tags": ["support"]
            },
            "created_by": "agent",
            "message": "initial"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create body: {created}");
    let prompt_id = str_field(&created, "/prompt/prompt_id").to_string();
    let version_one = str_field(&created, "/version/version_id").to_string();
    assert_eq!(created.pointer("/version/version_number"), Some(&json!(1)));

    // list_prompts shows the new prompt
    let (status, listed) = send(&app, "GET", base, None).await;
    assert_eq!(status, StatusCode::OK);
    let prompts = listed
        .pointer("/prompts")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing prompts array: {listed}"));
    assert_eq!(prompts.len(), 1);
    assert_eq!(str_field(&prompts[0], "/prompt_id"), prompt_id);

    // get_prompt by id
    let (status, prompt) = send(&app, "GET", &format!("{base}/{prompt_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(str_field(&prompt, "/name"), "answer-support-question");

    // add_version -> version 2
    let (status, version_two_body) = send(
        &app,
        "POST",
        &format!("{base}/{prompt_id}/versions"),
        Some(json!({
            "template": {
                "body": "system\nanswer with detail",
                "variables": [],
                "tags": ["support"]
            },
            "message": "expand"
        })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "add version body: {version_two_body}"
    );
    assert_eq!(version_two_body.pointer("/version_number"), Some(&json!(2)));
    let version_two = str_field(&version_two_body, "/version_id").to_string();

    // list_versions returns both, oldest-first
    let (status, versions_body) =
        send(&app, "GET", &format!("{base}/{prompt_id}/versions"), None).await;
    assert_eq!(status, StatusCode::OK);
    let versions = versions_body
        .pointer("/versions")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing versions array: {versions_body}"));
    assert_eq!(versions.len(), 2);

    // diff_versions reports the body change
    let (status, diff) = send(
        &app,
        "GET",
        &format!("{base}/{prompt_id}/diff?from={version_one}&to={version_two}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "diff body: {diff}");
    let lines = diff
        .pointer("/lines")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing diff lines: {diff}"));
    let texts: Vec<&str> = lines.iter().map(|line| str_field(line, "/text")).collect();
    assert!(texts.contains(&"answer briefly"), "diff texts: {texts:?}");
    assert!(
        texts.contains(&"answer with detail"),
        "diff texts: {texts:?}"
    );
}

#[tokio::test]
async fn get_unknown_prompt_returns_404() {
    let (app, _tempdir) = build_app();
    let (status, body) = send(
        &app,
        "GET",
        "/v1/prompts/tenant-a/project-a/prompt_does_not_exist",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert_eq!(body.pointer("/status"), Some(&json!(404)));
}

#[tokio::test]
async fn add_version_to_unknown_prompt_returns_404() {
    let (app, _tempdir) = build_app();
    let (status, body) = send(
        &app,
        "POST",
        "/v1/prompts/tenant-a/project-a/prompt_missing/versions",
        Some(json!({"template": {"body": "x", "variables": [], "tags": []}})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
}
