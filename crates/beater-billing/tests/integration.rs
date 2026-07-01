//! Integration tests for `beater-billing`:
//! * optimistic-concurrency proof for plan changes (`multi_thread`);
//! * a full crate-level e2e: subscribe -> record usage -> roll up -> finalize ->
//!   push to (mock) Stripe idempotently -> deliver the same webhook twice ->
//!   assert state changed exactly once.

use beater_billing::store::BillingStore;
use beater_billing::stripe::{
    EventApplication, StripeClient, StripeError, StripeSync, StripeUsagePush,
};
use beater_billing::{
    AdjustmentKind, Billing, BillingAdjustment, BillingPeriod, BillingScope, InvoiceStatus, Plan,
    PlanId, PlanTier, SqliteBillingStore, Subscription, SubscriptionStatus,
};
use beater_core::{Money, OrganizationId, ProjectId, TenantId};
use beater_security::sign_webhook;
use beater_usage::{
    SqliteUsageLedger, UsageLedgerStore, UsageMeter, UsageRecordInsert, UsageRecordSourceKind,
};
use chrono::{Duration, Utc};
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Mutex};

fn org() -> anyhow::Result<OrganizationId> {
    OrganizationId::new("org-e2e").map_err(|e| anyhow::anyhow!(e.to_string()))
}

fn plan_id(id: &str) -> anyhow::Result<PlanId> {
    PlanId::new(id).map_err(|e| anyhow::anyhow!(e.to_string()))
}

fn plan(id: &str, base: i64, included: u64, rate: i64) -> anyhow::Result<Plan> {
    let mut included_map = BTreeMap::new();
    included_map.insert(UsageMeter::JudgeCostMicros, included);
    let mut rates = BTreeMap::new();
    rates.insert(UsageMeter::JudgeCostMicros, Money::usd_micros(rate));
    Ok(Plan {
        id: plan_id(id)?,
        tier: PlanTier::Pro,
        included: included_map,
        base_price: Money::usd_micros(base),
        overage_rates: rates,
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_plan_change_has_one_winner_no_lost_update() -> anyhow::Result<()> {
    let store = Arc::new(SqliteBillingStore::in_memory()?);
    store.put_plan(plan("free", 0, 0, 0)?).await?;
    store.put_plan(plan("pro", 10_000, 1000, 2)?).await?;
    store.put_plan(plan("ent", 50_000, 10_000, 1)?).await?;

    let now = Utc::now();
    store
        .create_subscription(Subscription {
            org_id: org()?,
            plan_id: plan_id("free")?,
            status: SubscriptionStatus::Active,
            period_start: now,
            period_end: now + Duration::days(30),
            version: 1,
        })
        .await?;

    // Both writers observe the same version (1) and race the optimistic update.
    // This is exactly the lost-update scenario `Billing::change_plan` guards
    // against via `change_subscription_plan(expected_version, ...)`.
    let credit = BillingAdjustment {
        adjustment_id: "adj_a".to_string(),
        org_id: org()?,
        kind: AdjustmentKind::ProrationCredit,
        amount: Money::usd_micros(0),
        reason: "race a".to_string(),
        period_key: None,
        created_at: now,
    };
    let store_a = store.clone();
    let store_b = store.clone();
    let org_a = org()?;
    let org_b = org()?;
    let pro = plan_id("pro")?;
    let ent = plan_id("ent")?;
    let credit_a = credit.clone();
    let credit_b = credit.clone();

    let a = tokio::spawn(async move {
        store_a
            .change_subscription_plan(&org_a, 1, &pro, &[credit_a])
            .await
    });
    let b = tokio::spawn(async move {
        store_b
            .change_subscription_plan(&org_b, 1, &ent, &[credit_b])
            .await
    });

    let res_a = a.await?;
    let res_b = b.await?;

    let successes = [&res_a, &res_b].iter().filter(|r| r.is_ok()).count();
    let conflicts = [&res_a, &res_b]
        .iter()
        .filter(|r| matches!(r, Err(beater_store::StoreError::Conflict(_))))
        .count();
    assert_eq!(successes, 1, "exactly one writer wins");
    assert_eq!(conflicts, 1, "the other gets a conflict, not a lost update");

    // The surviving subscription is at version 2 with exactly one proration
    // adjustment (the loser wrote nothing).
    let sub = store
        .get_subscription(&org()?)
        .await?
        .ok_or_else(|| anyhow::anyhow!("subscription missing"))?;
    assert_eq!(sub.version, 2);
    assert_eq!(store.list_adjustments(&org()?).await?.len(), 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn change_plan_service_records_proration_and_bumps_version() -> anyhow::Result<()> {
    let store = Arc::new(SqliteBillingStore::in_memory()?);
    store.put_plan(plan("pro", 30_000, 1000, 2)?).await?;
    store.put_plan(plan("ent", 90_000, 10_000, 1)?).await?;
    let usage: Arc<dyn UsageLedgerStore> = Arc::new(SqliteUsageLedger::in_memory()?);
    let billing = Billing::new(store.clone() as Arc<dyn BillingStore>, usage);

    // 30-day period, switch exactly half way through.
    let start = Utc::now();
    let end = start + Duration::days(30);
    let at = start + Duration::days(15);
    store
        .create_subscription(Subscription {
            org_id: org()?,
            plan_id: plan_id("pro")?,
            status: SubscriptionStatus::Active,
            period_start: start,
            period_end: end,
            version: 1,
        })
        .await?;

    let change = billing.change_plan(&org()?, &plan_id("ent")?, at).await?;
    // ~half of each base price (allow for whole-second rounding).
    assert!((change.credit.amount_micros - 15_000).abs() <= 1);
    assert!((change.charge.amount_micros - 45_000).abs() <= 1);
    assert_eq!(change.net, change.charge.try_sub(&change.credit)?);
    assert_eq!(change.subscription.version, 2);
    assert_eq!(change.subscription.plan_id, plan_id("ent")?);
    // Two append-only proration adjustments.
    let adjustments = store.list_adjustments(&org()?).await?;
    assert_eq!(adjustments.len(), 2);
    Ok(())
}

#[derive(Default)]
struct MockStripeClient {
    seen: Mutex<HashSet<String>>,
    effective: Mutex<Vec<StripeUsagePush>>,
}

#[async_trait::async_trait]
impl StripeClient for MockStripeClient {
    async fn push_usage(
        &self,
        idempotency_key: &str,
        push: &StripeUsagePush,
    ) -> Result<(), StripeError> {
        let mut seen = self
            .seen
            .lock()
            .map_err(|e| StripeError::Transport(e.to_string()))?;
        if seen.insert(idempotency_key.to_string()) {
            self.effective
                .lock()
                .map_err(|e| StripeError::Transport(e.to_string()))?
                .push(push.clone());
        }
        Ok(())
    }
}

#[tokio::test]
async fn e2e_subscribe_rollup_finalize_push_and_webhook_exactly_once() -> anyhow::Result<()> {
    let store = Arc::new(SqliteBillingStore::in_memory()?);
    let usage = Arc::new(SqliteUsageLedger::in_memory()?);
    store.put_plan(plan("pro", 10_000, 1000, 2)?).await?;

    // Period spans "now" so the recorded usage falls inside it.
    let now = Utc::now();
    let period = BillingPeriod::new(now - Duration::days(1), now + Duration::days(1))?;
    store
        .create_subscription(Subscription {
            org_id: org()?,
            plan_id: plan_id("pro")?,
            status: SubscriptionStatus::Active,
            period_start: period.start,
            period_end: period.end,
            version: 1,
        })
        .await?;

    let tenant = TenantId::new("tenant-e2e").map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let project = ProjectId::new("project-e2e").map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Record 1500 units of usage (included 1000 -> 500 overage @ 2 = 1000).
    usage
        .record_usage(UsageRecordInsert {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            meter: UsageMeter::JudgeCostMicros,
            quantity: 1500,
            unit: "usd_micros".to_string(),
            source_kind: UsageRecordSourceKind::Manual,
            source_id: "e2e-usage".to_string(),
            attributes: serde_json::json!({}),
        })
        .await?;

    let billing = Billing::new(
        store.clone() as Arc<dyn BillingStore>,
        usage.clone() as Arc<dyn UsageLedgerStore>,
    );
    let scope = BillingScope {
        org_id: org()?,
        tenant_id: tenant,
        project_id: project,
    };

    // Roll up twice: identical result, exactly one stored invoice (idempotent).
    let invoice1 = billing.roll_up_period(&scope, &period).await?;
    let invoice2 = billing.roll_up_period(&scope, &period).await?;
    assert_eq!(invoice1, invoice2);
    assert_eq!(invoice1.total, Money::usd_micros(11_000)); // 10_000 base + 1_000 overage
    assert_eq!(store.list_invoices(&org()?).await?.len(), 1);

    // Finalize (idempotent).
    let finalized = billing.finalize_invoice(&org()?, &period.key()).await?;
    assert_eq!(finalized.status, InvoiceStatus::Finalized);
    let finalized_again = billing.finalize_invoice(&org()?, &period.key()).await?;
    assert_eq!(finalized_again.status, InvoiceStatus::Finalized);

    // Push metered usage to (mock) Stripe twice -> one effective push.
    let secret = b"whsec_e2e".to_vec();
    let stripe = StripeSync::new(store.clone() as Arc<dyn BillingStore>, secret.clone());
    let client = MockStripeClient::default();
    stripe.push_invoice_usage(&client, &finalized).await?;
    stripe.push_invoice_usage(&client, &finalized).await?;
    {
        let effective = client
            .effective
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        assert_eq!(effective.len(), 1);
        assert_eq!(effective[0].quantity, 500);
    }

    // Deliver a payment_succeeded webhook twice -> applied exactly once.
    let sign_now = Utc::now();
    let body = serde_json::to_vec(&serde_json::json!({
        "id": "evt_e2e",
        "created": sign_now.timestamp(),
        "type": "invoice.payment_succeeded",
        "data": { "object": {
            "id": "in_e2e",
            "org_id": "org-e2e",
            "amount_micros": 11_000,
            "period_key": period.key()
        }}
    }))?;
    let header = sign_webhook(&secret, &body, sign_now)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .header_value();

    let first = stripe.apply_event(&body, &header).await?;
    let second = stripe.apply_event(&body, &header).await?;
    assert_eq!(first, EventApplication::Applied);
    assert_eq!(second, EventApplication::Duplicate);

    // Exactly one Stripe-sync charge adjustment, and the invoice is now Paid.
    let adjustments = store.list_adjustments(&org()?).await?;
    let stripe_charges = adjustments
        .iter()
        .filter(|a| a.kind == AdjustmentKind::Charge)
        .count();
    assert_eq!(stripe_charges, 1);
    let paid = store
        .get_invoice(&org()?, &period.key())
        .await?
        .ok_or_else(|| anyhow::anyhow!("invoice missing"))?;
    assert_eq!(paid.status, InvoiceStatus::Paid);
    Ok(())
}
