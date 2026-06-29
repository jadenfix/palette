//! OpenAPI 3.1 contract for the Beater API.
//!
//! This is the single source of truth for the multi-language SDK pipeline. Every
//! route registered in [`crate::router`] is documented here via `#[utoipa::path]`
//! annotations placed on the real handler functions in `lib.rs`, and every schema
//! is derived from the real request/response types (no hand-maintained mirrors).

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde_json::{Map, Value};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Beater API",
        version = "0.1.0",
        description = "Agent observability, evaluation, gating, and human-review APIs for Beater"
    ),
    paths(
        crate::health,
        crate::ingest_native,
        crate::ingest_otlp_http,
        crate::import_source_route,
        crate::create_api_key_route,
        crate::revoke_api_key_route,
        crate::list_provider_secrets_route,
        crate::create_provider_secret_route,
        crate::revoke_provider_secret_route,
        crate::run_judge_eval_route,
        crate::list_judge_ledger_route,
        crate::get_usage_summary_route,
        crate::list_audit_events_route,
        crate::get_ingest_queue_status_route,
        crate::reconcile_trace_ingested_route,
        crate::replay_dead_letter_route,
        crate::drain_trace_writes_route,
        crate::drain_trace_ingested_route,
        crate::search_spans,
        crate::list_traces,
        crate::get_trace,
        crate::get_span_route,
        crate::get_span_io_route,
        crate::archive_trace,
        crate::query_archive_spans,
        crate::create_dataset,
        crate::promote_dataset_case,
        crate::create_dataset_version,
        crate::run_deterministic_dataset_eval,
        crate::run_judge_dataset_eval,
        crate::run_calibration_route,
        crate::run_deterministic_experiment_route,
        crate::run_judge_experiment_route,
        crate::create_gate_route,
        crate::run_gate_route,
        crate::create_review_queue_route,
        crate::list_review_tasks_route,
        crate::enqueue_review_task_from_trace_route,
        crate::submit_review_annotation_route,
        crate::promote_review_annotation_route,
        crate::decide_online_sampling,
        crate::evaluate_alert,
    ),
    tags(
        (name = "health", description = "Runtime health"),
        (name = "ingest", description = "Trace ingestion, queue, and durability"),
        (name = "traces", description = "Trace read APIs"),
        (name = "spans", description = "Span read APIs"),
        (name = "search", description = "Span search"),
        (name = "archive", description = "Trace archival and archived-span queries"),
        (name = "datasets", description = "Datasets, cases, and versions"),
        (name = "evals", description = "Dataset evaluations"),
        (name = "experiments", description = "Baseline/candidate experiments"),
        (name = "calibrations", description = "Evaluator calibration runs"),
        (name = "gates", description = "Release gates"),
        (name = "judge", description = "Ad-hoc judge evaluation and ledger"),
        (name = "reviews", description = "Human review queues and annotations"),
        (name = "online", description = "Online sampling decisions"),
        (name = "alerts", description = "Alert policy evaluation"),
        (name = "usage", description = "Usage summaries"),
        (name = "audit", description = "Audit events"),
        (name = "apiKeys", description = "API key management"),
        (name = "providerSecrets", description = "Provider secret management")
    )
)]
pub struct BeaterApi;

/// Billing/Stripe is a hosted concern. Its paths, schemas, and tag live in this
/// separate document that is only compiled — and only merged into the public
/// contract — under the non-default `billing` cargo feature, so the open-source
/// API contract never advertises Stripe endpoints.
#[cfg(feature = "billing")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::get_plans_route,
        crate::get_plan_route,
        crate::get_subscription_route,
        crate::create_subscription_route,
        crate::change_subscription_plan_route,
        crate::get_org_invoices_route,
        crate::get_invoice_route,
        crate::stripe_webhook_route,
    ),
    tags(
        (name = "billing", description = "Plans, subscriptions, invoices, and Stripe billing"),
    )
)]
pub struct BillingApi;

/// Build the OpenAPI document, stamping the live crate version at runtime.
///
/// The utoipa derive requires a literal `version`, so we override it here with
/// the actual `CARGO_PKG_VERSION` to keep the spec in lockstep with the crate.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = BeaterApi::openapi();
    #[cfg(feature = "billing")]
    doc.merge(BillingApi::openapi());
    doc.info.version = env!("CARGO_PKG_VERSION").to_string();
    doc
}

pub fn openapi_json_pretty() -> Result<String, serde_json::Error> {
    openapi().to_pretty_json()
}

/// Percent-encode a value for safe interpolation into a request path segment or
/// query-string parameter, escaping everything outside the RFC 3986 *unreserved*
/// set (`ALPHA / DIGIT / "-" / "_" / "." / "~"`).
///
/// Shared by the MCP server and CLI, which both turn a resolved spec operation
/// into a live API request by filling its path template and query string.
pub fn urlencode(value: &str) -> String {
    // RFC 3986 unreserved == NON_ALPHANUMERIC minus the four unreserved marks.
    const UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'~');
    utf8_percent_encode(value, UNRESERVED).to_string()
}

/// HTTP methods that may carry an operation under a path item, per OpenAPI.
const OPERATION_METHODS: [&str; 7] = ["get", "put", "post", "delete", "options", "head", "patch"];

/// One operation discovered in the OpenAPI document, borrowing from the
/// serialized spec.
pub struct SpecOperation<'a> {
    /// The unique `operationId`.
    pub operation_id: &'a str,
    /// Lower-case HTTP method as it appears in the spec (e.g. `"get"`).
    pub method: &'a str,
    /// Path template, e.g. `/v1/traces/{tenant_id}`.
    pub path: &'a str,
    /// The full Operation Object, for callers that need its parameters,
    /// request body, responses, etc.
    pub operation: &'a Map<String, Value>,
}

/// Enumerate every operation in a serialized OpenAPI document (`doc` is the
/// spec rendered to JSON, e.g. `serde_json::to_value(openapi())`), walking each
/// path item × HTTP method that carries an `operationId`.
///
/// This is the single source of truth for "how do we find operations in the
/// spec," shared by the MCP tool catalog and the CLI's operation resolver so
/// neither hand-rolls the paths × methods walk. Returns an empty list when the
/// document has no `paths` object.
pub fn operations(doc: &Value) -> Vec<SpecOperation<'_>> {
    let mut ops = Vec::new();
    let Some(paths) = doc.get("paths").and_then(Value::as_object) else {
        return ops;
    };
    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };
        for method in OPERATION_METHODS {
            let Some(operation) = item.get(method).and_then(Value::as_object) else {
                continue;
            };
            let Some(operation_id) = operation.get("operationId").and_then(Value::as_str) else {
                continue;
            };
            ops.push(SpecOperation {
                operation_id,
                method,
                path,
                operation,
            });
        }
    }
    ops
}

#[cfg(test)]
mod tests {
    use super::{operations, urlencode};

    #[test]
    fn passes_unreserved_and_escapes_the_rest() {
        // Unreserved set is left untouched.
        assert_eq!(urlencode("aZ09-_.~"), "aZ09-_.~");
        // Reserved/space/unicode are percent-escaped with uppercase hex.
        assert_eq!(urlencode("a b/c?d#e"), "a%20b%2Fc%3Fd%23e");
        assert_eq!(urlencode("café"), "caf%C3%A9");
    }

    #[test]
    fn enumerates_operations_from_the_live_spec() -> Result<(), serde_json::Error> {
        let doc = serde_json::to_value(super::openapi())?;
        let ops = operations(&doc);
        // Every advertised operation has an id, a known method, and a path.
        assert!(!ops.is_empty());
        assert!(ops
            .iter()
            .all(|op| !op.operation_id.is_empty() && op.path.starts_with('/')));
        // operationIds are unique across the surface.
        let mut ids: Vec<&str> = ops.iter().map(|op| op.operation_id).collect();
        ids.sort_unstable();
        let unique = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), unique, "duplicate operationId in spec");
        Ok(())
    }
}
