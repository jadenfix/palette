//! HTTP surface for self-observability: the Prometheus `/metrics` route and the
//! axum middleware that records per-request query latency (R13.5).
//!
//! This is a Prometheus exposition endpoint, NOT part of the typed `/v1` OpenAPI
//! contract, so it is intentionally registered outside `beater_api::router`.

use axum::Router;
use axum::body::Body;
use axum::extract::{MatchedPath, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::metrics::{Metrics, Stopwatch};

/// Build a router exposing `GET /metrics` in Prometheus text format. Merged into
/// the main app alongside `beater_api::router` and the MCP router.
pub fn router(metrics: Metrics) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(metrics)
}

async fn metrics_handler(State(metrics): State<Metrics>) -> Response {
    let body = metrics.render();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
        .into_response()
}

/// R13.5 — axum middleware that times every request and records its latency into
/// the query-latency histogram, labelled by the matched route template (NOT the
/// raw path, to keep label cardinality bounded) and HTTP method.
pub async fn track_query_latency(metrics: Metrics, request: Request<Body>, next: Next) -> Response {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "unmatched".to_string());
    let method = request.method().as_str().to_string();
    let sw = Stopwatch::start();
    let response = next.run(request).await;
    metrics.observe_query_latency(&route, &method, sw.elapsed_seconds());
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::middleware;
    use tower::ServiceExt;

    #[tokio::test]
    async fn metrics_endpoint_renders_prometheus_text() {
        let metrics = Metrics::new();
        metrics.record_write(crate::metrics::OpResult::Success, 5);
        let app = router(metrics);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("build request: {e}")),
            )
            .await
            .unwrap_or_else(|e| panic!("oneshot: {e}"));
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(ct.starts_with("text/plain"));
        let bytes = to_bytes(response.into_body(), 1 << 20)
            .await
            .unwrap_or_else(|e| panic!("body: {e}"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("beater_trace_writes_total{result=\"success\"} 5"));
    }

    #[tokio::test]
    async fn middleware_records_latency_for_matched_route() {
        let metrics = Metrics::new();
        let m = metrics.clone();
        let app = Router::new()
            .route("/v1/ping", get(|| async { "pong" }))
            .layer(middleware::from_fn(move |req, next| {
                let m = m.clone();
                async move { track_query_latency(m, req, next).await }
            }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/ping")
                    .body(Body::empty())
                    .unwrap_or_else(|e| panic!("build request: {e}")),
            )
            .await
            .unwrap_or_else(|e| panic!("oneshot: {e}"));
        assert_eq!(response.status(), StatusCode::OK);
        let out = metrics.render();
        assert!(out.contains("beater_http_request_duration_seconds"));
        assert!(out.contains("route=\"/v1/ping\""));
        assert!(out.contains("method=\"GET\""));
    }
}
