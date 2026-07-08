#![cfg(feature = "billing")]

use axum::Router;
use axum::body::{Body, to_bytes};
use beater_api::{ApiState, router};
use beater_archive::ParquetTraceArchive;
use beater_auth::{ApiKeyStore, CreateApiKeyRequest, SqliteApiKeyStore};
use beater_billing::store::BillingStore;
use beater_billing::{Billing, BillingScope, Plan, PlanId, PlanTier, SqliteBillingStore};
use beater_bus::InMemoryBus;
use beater_core::{EnvironmentId, Money, OrganizationId, ProjectId, TenantId};
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_ingest::{IngestPolicy, IngestService};
use beater_search::TantivySearchIndex;
use beater_security::{ApiScope, CreatedApiKey, sign_webhook};
use beater_store_obj::FsArtifactStore;
use beater_store_sql::SqliteTraceStore;
use beater_usage::{
    SqliteUsageLedger, UsageLedgerStore, UsageMeter, UsageRecordInsert, UsageRecordSourceKind,
};
use chrono::{Duration, TimeZone, Utc};
use http::{Method, Request, StatusCode};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tower::ServiceExt;

const ORG_ID: &str = "org-http";
const PROJECT_ID: &str = "project-http";
const ENVIRONMENT_ID: &str = "local";
const STRIPE_SECRET: &[u8] = b"whsec_api_billing_test";

struct BillingApiHarness {
    app: Router,
    billing: Arc<SqliteBillingStore>,
    usage: Arc<SqliteUsageLedger>,
    admin: CreatedApiKey,
    _tempdir: tempfile::TempDir,
}

fn must<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("test setup failed: {err}"),
    }
}

fn id<T, F>(value: &str, make: F) -> T
where
    F: FnOnce(String) -> Result<T, beater_core::IdError>,
{
    must(make(value.to_string()))
}

fn plan(id: &str, tier: PlanTier, base_micros: i64, included: u64, rate_micros: i64) -> Plan {
    let mut included_map = BTreeMap::new();
    included_map.insert(UsageMeter::JudgeCostMicros, included);
    let mut rates = BTreeMap::new();
    rates.insert(UsageMeter::JudgeCostMicros, Money::usd_micros(rate_micros));
    Plan {
        id: must(PlanId::new(id)),
        tier,
        included: included_map,
        base_price: Money::usd_micros(base_micros),
        overage_rates: rates,
    }
}

async fn harness() -> BillingApiHarness {
    let tempdir = must(tempfile::tempdir());
    let artifacts = Arc::new(must(FsArtifactStore::new(tempdir.path().join("artifacts"))));
    let traces = Arc::new(must(SqliteTraceStore::in_memory()));
    let search = Arc::new(must(TantivySearchIndex::in_memory()));
    let archive = must(ParquetTraceArchive::new(tempdir.path().join("archive")));
    let datasets = Arc::new(must(SqliteDatasetStore::in_memory()));
    let experiments = Arc::new(must(SqliteExperimentStore::in_memory()));
    let bus = Arc::new(InMemoryBus::new(32));
    let ingest = IngestService::new(artifacts, traces.clone(), bus, IngestPolicy::default());
    let billing = Arc::new(must(SqliteBillingStore::in_memory()));
    let usage = Arc::new(must(SqliteUsageLedger::in_memory()));
    let api_keys = Arc::new(must(SqliteApiKeyStore::in_memory()));

    billing
        .put_plan(plan("local", PlanTier::Free, 0, 0, 0))
        .await
        .unwrap_or_else(|err| panic!("seed local plan: {err}"));
    billing
        .put_plan(plan("pro", PlanTier::Pro, 10_000, 1_000, 2))
        .await
        .unwrap_or_else(|err| panic!("seed pro plan: {err}"));
    billing
        .put_plan(plan("enterprise", PlanTier::Enterprise, 60_000, 10_000, 1))
        .await
        .unwrap_or_else(|err| panic!("seed enterprise plan: {err}"));

    let mut scopes = BTreeSet::new();
    scopes.insert(ApiScope::Admin);
    let admin = api_keys
        .create_key(CreateApiKeyRequest {
            tenant_id: id(ORG_ID, TenantId::new),
            project_id: id(PROJECT_ID, ProjectId::new),
            environment_id: id(ENVIRONMENT_ID, EnvironmentId::new),
            scopes,
        })
        .await
        .unwrap_or_else(|err| panic!("create admin key: {err}"));

    let app = router(
        ApiState::with_integrations(ingest, traces, search, archive, datasets, experiments)
            .with_usage(usage.clone())
            .with_billing(billing.clone(), STRIPE_SECRET.to_vec())
            .require_auth(api_keys),
    );

    BillingApiHarness {
        app,
        billing,
        usage,
        admin,
        _tempdir: tempdir,
    }
}

async fn request_json(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
    admin: Option<&CreatedApiKey>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(admin) = admin {
        builder = builder
            .header("authorization", format!("Bearer {}", admin.secret))
            .header("x-beater-project-id", PROJECT_ID)
            .header("x-beater-environment-id", ENVIRONMENT_ID);
    }
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(must(builder.body(body)))
        .await
        .unwrap_or_else(|err| panic!("request {uri}: {err}"));
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|err| panic!("read response body: {err}"));
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|err| {
            panic!("decode response body as json ({status} {uri}): {err}; body={bytes:?}")
        })
    };
    (status, value)
}

#[tokio::test]
async fn hosted_billing_routes_round_trip_and_keep_invalid_state_out() {
    let h = harness().await;

    let (status, plans) = request_json(&h.app, Method::GET, "/v1/plans", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(plans.as_array().map(Vec::len), Some(3));

    let start = must(
        Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
            .single()
            .ok_or("bad date"),
    );
    let end = start + Duration::days(31);
    let bad_period = json!({
        "plan_id": "pro",
        "period_start": end.to_rfc3339(),
        "period_end": start.to_rfc3339()
    });
    let (status, body) = request_json(
        &h.app,
        Method::POST,
        "/v1/subscriptions/org-http",
        Some(bad_period),
        Some(&h.admin),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .is_some_and(|error| error.contains("period_end must be after period_start")),
        "unexpected body: {body}"
    );

    let unknown_plan = json!({
        "plan_id": "missing",
        "period_start": start.to_rfc3339(),
        "period_end": end.to_rfc3339()
    });
    let (status, body) = request_json(
        &h.app,
        Method::POST,
        "/v1/subscriptions/org-http",
        Some(unknown_plan),
        Some(&h.admin),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body["error"]
            .as_str()
            .is_some_and(|error| error.contains("plan missing not found")),
        "unexpected body: {body}"
    );

    let subscription = json!({
        "plan_id": "pro",
        "period_start": start.to_rfc3339(),
        "period_end": end.to_rfc3339()
    });
    let (status, body) = request_json(
        &h.app,
        Method::POST,
        "/v1/subscriptions/org-http",
        Some(subscription),
        Some(&h.admin),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["plan_id"], "pro");
    assert_eq!(body["version"], 1);

    let (status, body) = request_json(
        &h.app,
        Method::POST,
        "/v1/subscriptions/org-http/change-plan",
        Some(json!({
            "new_plan_id": "enterprise",
            "at": (start + Duration::days(10)).to_rfc3339()
        })),
        Some(&h.admin),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["subscription"]["plan_id"], "enterprise");
    assert_eq!(body["subscription"]["version"], 2);
    assert!(body["net"]["amount_micros"].as_i64().unwrap_or(0) > 0);

    h.usage
        .record_usage(UsageRecordInsert {
            tenant_id: id(ORG_ID, TenantId::new),
            project_id: id(PROJECT_ID, ProjectId::new),
            meter: UsageMeter::JudgeCostMicros,
            quantity: 12_500,
            unit: "usd_micros".to_string(),
            source_kind: UsageRecordSourceKind::Manual,
            source_id: "api-billing-e2e".to_string(),
            attributes: json!({ "repo": "jadenfix/beater" }),
        })
        .await
        .unwrap_or_else(|err| panic!("record usage: {err}"));
    let service = Billing::new(h.billing.clone(), h.usage.clone());
    let invoice = service
        .roll_up_period(
            &BillingScope {
                org_id: id(ORG_ID, OrganizationId::new),
                tenant_id: id(ORG_ID, TenantId::new),
                project_id: id(PROJECT_ID, ProjectId::new),
            },
            &beater_billing::BillingPeriod::new(start, end)
                .unwrap_or_else(|err| panic!("period: {err}")),
        )
        .await
        .unwrap_or_else(|err| panic!("roll up period: {err}"));
    assert_eq!(invoice.period_key, "2026-07");

    let (status, body) = request_json(
        &h.app,
        Method::GET,
        "/v1/billing/invoices/org-http/2026-07",
        None,
        Some(&h.admin),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["period_key"], "2026-07");
    assert_eq!(body["status"], "draft");

    service
        .finalize_invoice(&id(ORG_ID, OrganizationId::new), "2026-07")
        .await
        .unwrap_or_else(|err| panic!("finalize invoice: {err}"));

    let raw = json!({
        "id": "evt_http_paid_1",
        "created": Utc::now().timestamp(),
        "type": "invoice.payment_succeeded",
        "data": {
            "object": {
                "id": "in_http_1",
                "org_id": ORG_ID,
                "status": "paid",
                "amount_micros": 0,
                "period_key": "2026-07"
            }
        }
    })
    .to_string();
    let signature = sign_webhook(STRIPE_SECRET, raw.as_bytes(), Utc::now())
        .unwrap_or_else(|err| panic!("sign webhook: {err}"))
        .header_value();
    let webhook = Request::builder()
        .method(Method::POST)
        .uri("/v1/billing/webhooks/stripe")
        .header("stripe-signature", signature.clone())
        .body(Body::from(raw.clone()))
        .unwrap_or_else(|err| panic!("build webhook request: {err}"));
    let response = h
        .app
        .clone()
        .oneshot(webhook)
        .await
        .unwrap_or_else(|err| panic!("webhook request: {err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("read webhook response: {err}")),
    )
    .unwrap_or_else(|err| panic!("decode webhook response: {err}"));
    assert_eq!(body["outcome"], "applied");

    let webhook = Request::builder()
        .method(Method::POST)
        .uri("/v1/billing/webhooks/stripe")
        .header("stripe-signature", signature)
        .body(Body::from(raw))
        .unwrap_or_else(|err| panic!("build duplicate webhook request: {err}"));
    let response = h
        .app
        .oneshot(webhook)
        .await
        .unwrap_or_else(|err| panic!("duplicate webhook request: {err}"));
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap_or_else(|err| panic!("read duplicate webhook response: {err}")),
    )
    .unwrap_or_else(|err| panic!("decode duplicate webhook response: {err}"));
    assert_eq!(body["outcome"], "duplicate");

    let invoice = h
        .billing
        .get_invoice(&id(ORG_ID, OrganizationId::new), "2026-07")
        .await
        .unwrap_or_else(|err| panic!("read invoice after webhook: {err}"))
        .unwrap_or_else(|| panic!("invoice missing after webhook"));
    assert_eq!(invoice.status, beater_billing::InvoiceStatus::Paid);

    let subscription = h
        .billing
        .get_subscription(&id(ORG_ID, OrganizationId::new))
        .await
        .unwrap_or_else(|err| panic!("read subscription: {err}"))
        .unwrap_or_else(|| panic!("subscription missing"));
    assert_eq!(subscription.plan_id.as_str(), "enterprise");
}
