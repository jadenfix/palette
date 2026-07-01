//! Stripe synchronization: push metered usage with idempotency keys and apply
//! inbound Stripe webhook events exactly-once and out-of-order-safe.
//!
//! No real network is used; the outbound side is abstracted behind the
//! [`StripeClient`] trait (mock it in tests), and the inbound side is a pure
//! function of the raw bytes + signature header verified with
//! [`beater_security::verify_webhook`].
//!
//! Exactly-once + ordering guarantees (see [`StripeSync::apply_event`]):
//! 1. **Signature** — verified via `beater-security` (HMAC + replay window).
//! 2. **Dedup** — the event id is inserted into a `UNIQUE` table; a repeat
//!    delivery is acknowledged as [`EventApplication::Duplicate`] and not
//!    re-applied.
//! 3. **Ordering** — an event older-or-equal to the last applied event for the
//!    same object is acknowledged as [`EventApplication::Stale`] and not
//!    applied.
//! 4. **Apply** — newer events map to append-only [`BillingAdjustment`]s and, as
//!    appropriate, subscription/invoice status transitions.

use crate::{AdjustmentKind, BillingAdjustment, BillingStore, InvoiceStatus, SubscriptionStatus};
use async_trait::async_trait;
use beater_core::{Clock, Money, OrganizationId, SystemClock};
use beater_security::SecurityError;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Default replay tolerance for inbound webhooks.
pub const DEFAULT_WEBHOOK_TOLERANCE_SECONDS: i64 = 300;

/// Typed Stripe sync errors. No path panics.
#[derive(Debug, thiserror::Error)]
pub enum StripeError {
    #[error("stripe webhook signature/timestamp rejected: {0}")]
    Signature(#[from] SecurityError),
    #[error("malformed stripe event payload: {0}")]
    Malformed(String),
    #[error("store error: {0}")]
    Store(#[from] beater_store::StoreError),
    #[error("billing error: {0}")]
    Billing(String),
    #[error("stripe transport error: {0}")]
    Transport(String),
}

/// Outcome of applying one inbound Stripe event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventApplication {
    /// The event was new and newer than the last applied event; state changed.
    Applied,
    /// The event id had already been delivered; no change (exactly-once).
    Duplicate,
    /// The event was older-or-equal to the last applied event; acknowledged but
    /// not applied (out-of-order-safe).
    Stale,
}

/// A metered-usage push to Stripe. Carries no money — Stripe meters quantities;
/// the rated amount is computed on Stripe's side from the price.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StripeUsagePush {
    pub org_id: OrganizationId,
    pub period_key: String,
    pub meter: String,
    pub quantity: u64,
}

impl StripeUsagePush {
    /// Deterministic Stripe idempotency key for this push. Re-pushing the same
    /// `(org, period, meter)` carries the same key, so Stripe records it once.
    pub fn idempotency_key(&self) -> String {
        format!(
            "usage_{}_{}_{}",
            self.org_id.as_str(),
            self.period_key,
            self.meter
        )
    }
}

/// Outbound Stripe API surface used by [`StripeSync`]. Abstracted so tests use a
/// mock and production uses a real HTTP client; every call carries an
/// idempotency key.
#[async_trait]
pub trait StripeClient: Send + Sync {
    async fn push_usage(
        &self,
        idempotency_key: &str,
        push: &StripeUsagePush,
    ) -> Result<(), StripeError>;
}

/// Minimal, realistic shape of a Stripe webhook event (`event.data.object`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StripeEvent {
    pub id: String,
    pub created: i64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeEventData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StripeEventData {
    pub object: StripeObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StripeObject {
    /// Stripe object id (e.g. `sub_...`/`in_...`); the ordering key.
    pub id: String,
    /// Our organization id, carried in Stripe metadata.
    pub org_id: Option<String>,
    /// New status, for subscription/invoice lifecycle events.
    pub status: Option<String>,
    /// Amount in micros (our canonical money unit), if the event carries one.
    pub amount_micros: Option<i64>,
    /// Billing period `YYYY-MM`, if applicable.
    pub period_key: Option<String>,
}

/// Synchronizes local billing state with Stripe.
pub struct StripeSync<C: Clock = SystemClock> {
    store: Arc<dyn BillingStore>,
    signing_secret: Vec<u8>,
    tolerance: Duration,
    clock: C,
}

impl StripeSync<SystemClock> {
    pub fn new(store: Arc<dyn BillingStore>, signing_secret: impl Into<Vec<u8>>) -> Self {
        Self {
            store,
            signing_secret: signing_secret.into(),
            tolerance: Duration::seconds(DEFAULT_WEBHOOK_TOLERANCE_SECONDS),
            clock: SystemClock,
        }
    }
}

impl<C: Clock> StripeSync<C> {
    pub fn with_clock(
        store: Arc<dyn BillingStore>,
        signing_secret: impl Into<Vec<u8>>,
        tolerance: Duration,
        clock: C,
    ) -> Self {
        Self {
            store,
            signing_secret: signing_secret.into(),
            tolerance,
            clock,
        }
    }

    /// Push each billable overage line of an invoice to Stripe with a
    /// deterministic idempotency key, so re-pushing is a no-op on Stripe's side.
    pub async fn push_invoice_usage(
        &self,
        client: &dyn StripeClient,
        invoice: &crate::Invoice,
    ) -> Result<Vec<StripeUsagePush>, StripeError> {
        let mut pushes = Vec::new();
        for item in &invoice.line_items {
            let Some(meter) = item.meter else {
                continue; // base line, not metered usage
            };
            if item.overage_quantity == 0 {
                continue;
            }
            let push = StripeUsagePush {
                org_id: invoice.org_id.clone(),
                period_key: invoice.period_key.clone(),
                meter: meter.as_str().to_string(),
                quantity: item.overage_quantity,
            };
            client.push_usage(&push.idempotency_key(), &push).await?;
            pushes.push(push);
        }
        Ok(pushes)
    }

    /// Verify, dedup, order, and apply an inbound Stripe webhook event. See the
    /// module docs for the four-step guarantee.
    pub async fn apply_event(
        &self,
        raw: &[u8],
        signature_header: &str,
    ) -> Result<EventApplication, StripeError> {
        // 1. Signature + replay-window verification.
        beater_security::verify_webhook(
            &self.signing_secret,
            raw,
            signature_header,
            self.clock.now(),
            self.tolerance,
        )?;

        // Parse only after the signature is trusted.
        let event: StripeEvent =
            serde_json::from_slice(raw).map_err(|err| StripeError::Malformed(err.to_string()))?;
        let object_id = event.data.object.id.clone();

        // 2. Dedup by event id (UNIQUE). A repeat delivery is acknowledged but
        //    not re-applied.
        let newly_recorded = self
            .store
            .record_stripe_event(&event.id, &object_id, event.created)
            .await?;
        if !newly_recorded {
            return Ok(EventApplication::Duplicate);
        }

        // 3. Out-of-order guard: ignore events not newer than the last applied
        //    one for this object.
        if let Some(last) = self.store.last_applied_stripe_created(&object_id).await? {
            if event.created <= last {
                return Ok(EventApplication::Stale);
            }
        }

        // 4. Apply the effect, then mark applied (drives future ordering).
        self.apply_effect(&event).await?;
        self.store.mark_stripe_event_applied(&event.id).await?;
        Ok(EventApplication::Applied)
    }

    async fn apply_effect(&self, event: &StripeEvent) -> Result<(), StripeError> {
        let object = &event.data.object;
        let org_id = match &object.org_id {
            Some(org_id) => OrganizationId::new(org_id.clone())
                .map_err(|err| StripeError::Billing(err.to_string()))?,
            // Without our org metadata there is nothing local to update; the
            // event is still recorded+applied for dedup/ordering purposes.
            None => return Ok(()),
        };

        // Lifecycle transitions, where the event type implies one.
        match event.event_type.as_str() {
            "customer.subscription.deleted" => {
                self.store
                    .set_subscription_status(&org_id, SubscriptionStatus::Canceled)
                    .await?;
            }
            "invoice.payment_failed" => {
                self.store
                    .set_subscription_status(&org_id, SubscriptionStatus::PastDue)
                    .await?;
            }
            "customer.subscription.updated" => {
                if let Some(status) = object.status.as_deref().and_then(SubscriptionStatus::parse) {
                    self.store.set_subscription_status(&org_id, status).await?;
                }
            }
            "invoice.payment_succeeded" => {
                if let Some(period_key) = &object.period_key {
                    // Best-effort: only transition an invoice we actually have.
                    if self.store.get_invoice(&org_id, period_key).await?.is_some() {
                        self.store
                            .set_invoice_status(&org_id, period_key, InvoiceStatus::Paid)
                            .await?;
                    }
                }
            }
            _ => {}
        }

        // Always record an append-only adjustment capturing the synced event, so
        // the local ledger reflects exactly the events Stripe applied. Exactly
        // one is written per event id thanks to the dedup guard above.
        let amount = Money::usd_micros(object.amount_micros.unwrap_or(0));
        let kind = match event.event_type.as_str() {
            "invoice.payment_succeeded" => AdjustmentKind::Charge,
            "charge.refunded" | "invoice.voided" => AdjustmentKind::Refund,
            _ => AdjustmentKind::StripeSync,
        };
        let adjustment = BillingAdjustment {
            adjustment_id: format!("adj_stripe_{}", event.id),
            org_id,
            kind,
            amount,
            reason: format!("stripe event {} ({})", event.id, event.event_type),
            period_key: object.period_key.clone(),
            created_at: self.clock.now(),
        };
        self.store.append_adjustment(adjustment).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteBillingStore;
    use beater_core::FixedClock;
    use beater_security::sign_webhook;
    use chrono::{TimeZone, Utc};
    use std::collections::HashSet;
    use std::sync::Mutex;

    const SECRET: &[u8] = b"whsec_test_secret";

    /// A mock Stripe client that records pushes and dedups by idempotency key,
    /// simulating Stripe's own idempotency behaviour.
    #[derive(Default)]
    struct MockStripeClient {
        seen_keys: Mutex<HashSet<String>>,
        effective: Mutex<Vec<StripeUsagePush>>,
    }

    #[async_trait]
    impl StripeClient for MockStripeClient {
        async fn push_usage(
            &self,
            idempotency_key: &str,
            push: &StripeUsagePush,
        ) -> Result<(), StripeError> {
            let mut keys = self
                .seen_keys
                .lock()
                .map_err(|err| StripeError::Transport(err.to_string()))?;
            if keys.insert(idempotency_key.to_string()) {
                self.effective
                    .lock()
                    .map_err(|err| StripeError::Transport(err.to_string()))?
                    .push(push.clone());
            }
            Ok(())
        }
    }

    fn fixed_now() -> beater_core::Timestamp {
        Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0)
            .single()
            .unwrap_or_else(Utc::now)
    }

    fn sync(store: Arc<dyn BillingStore>) -> Result<StripeSync<FixedClock>, anyhow::Error> {
        Ok(StripeSync::with_clock(
            store,
            SECRET.to_vec(),
            Duration::seconds(DEFAULT_WEBHOOK_TOLERANCE_SECONDS),
            FixedClock::new(fixed_now()),
        ))
    }

    fn event_json(id: &str, created: i64, event_type: &str, org: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "id": id,
            "created": created,
            "type": event_type,
            "data": { "object": {
                "id": "sub_a",
                "org_id": org,
                "amount_micros": 5000,
                "period_key": "2026-06"
            }}
        }))
        .unwrap_or_default()
    }

    fn signed_header(body: &[u8]) -> Result<String, anyhow::Error> {
        let sig = sign_webhook(SECRET, body, fixed_now())
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        Ok(sig.header_value())
    }

    #[tokio::test]
    async fn double_delivery_applies_exactly_once() -> anyhow::Result<()> {
        let store = Arc::new(SqliteBillingStore::in_memory()?);
        let sync = sync(store.clone())?;
        let body = event_json("evt_1", 1000, "invoice.payment_succeeded", "org-1");
        let header = signed_header(&body)?;

        let first = sync.apply_event(&body, &header).await?;
        let second = sync.apply_event(&body, &header).await?;
        assert_eq!(first, EventApplication::Applied);
        assert_eq!(second, EventApplication::Duplicate);

        let org = OrganizationId::new("org-1").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let adjustments = store.list_adjustments(&org).await?;
        assert_eq!(
            adjustments.len(),
            1,
            "exactly one adjustment despite re-delivery"
        );
        Ok(())
    }

    #[tokio::test]
    async fn bad_signature_is_rejected() -> anyhow::Result<()> {
        let store = Arc::new(SqliteBillingStore::in_memory()?);
        let sync = sync(store.clone())?;
        let body = event_json("evt_2", 1000, "invoice.payment_succeeded", "org-1");
        // Sign with a different secret -> verification fails.
        let forged = sign_webhook(b"wrong_secret", &body, fixed_now())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .header_value();
        let result = sync.apply_event(&body, &forged).await;
        assert!(matches!(result, Err(StripeError::Signature(_))));
        // Nothing recorded.
        let org = OrganizationId::new("org-1").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        assert!(store.list_adjustments(&org).await?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn out_of_order_older_event_is_not_applied() -> anyhow::Result<()> {
        let store = Arc::new(SqliteBillingStore::in_memory()?);
        let sync = sync(store.clone())?;

        // Apply a newer event first.
        let newer = event_json("evt_new", 2000, "customer.subscription.updated", "org-1");
        let newer_header = signed_header(&newer)?;
        assert_eq!(
            sync.apply_event(&newer, &newer_header).await?,
            EventApplication::Applied
        );

        // Then deliver an older event for the same object.
        let older = event_json("evt_old", 1000, "customer.subscription.updated", "org-1");
        let older_header = signed_header(&older)?;
        assert_eq!(
            sync.apply_event(&older, &older_header).await?,
            EventApplication::Stale
        );

        let org = OrganizationId::new("org-1").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        // Only the newer event produced an adjustment.
        assert_eq!(store.list_adjustments(&org).await?.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn valid_newer_event_updates_state() -> anyhow::Result<()> {
        let store = Arc::new(SqliteBillingStore::in_memory()?);
        // Seed a subscription so the status transition has something to update.
        store
            .put_plan(crate::Plan {
                id: crate::PlanId::new("pro").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                tier: crate::PlanTier::Pro,
                included: std::collections::BTreeMap::new(),
                base_price: Money::usd_micros(10_000),
                overage_rates: std::collections::BTreeMap::new(),
            })
            .await?;
        let org = OrganizationId::new("org-1").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        store
            .create_subscription(crate::Subscription {
                org_id: org.clone(),
                plan_id: crate::PlanId::new("pro").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                status: SubscriptionStatus::Active,
                period_start: fixed_now(),
                period_end: fixed_now() + Duration::days(30),
                version: 1,
            })
            .await?;

        let sync = sync(store.clone())?;
        let body = serde_json::to_vec(&serde_json::json!({
            "id": "evt_cancel",
            "created": 3000,
            "type": "customer.subscription.deleted",
            "data": { "object": { "id": "sub_a", "org_id": "org-1" }}
        }))?;
        let header = signed_header(&body)?;
        assert_eq!(
            sync.apply_event(&body, &header).await?,
            EventApplication::Applied
        );

        let updated = store
            .get_subscription(&org)
            .await?
            .ok_or_else(|| anyhow::anyhow!("subscription missing"))?;
        assert_eq!(updated.status, SubscriptionStatus::Canceled);
        Ok(())
    }

    #[tokio::test]
    async fn usage_push_is_idempotent_on_stripe() -> anyhow::Result<()> {
        let store = Arc::new(SqliteBillingStore::in_memory()?);
        let sync = sync(store)?;
        let client = MockStripeClient::default();
        let org = OrganizationId::new("org-1").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let invoice = crate::Invoice {
            org_id: org,
            period_key: "2026-06".to_string(),
            line_items: vec![crate::InvoiceLineItem {
                meter: Some(beater_usage::UsageMeter::JudgeCostMicros),
                description: "Overage".to_string(),
                quantity: 100,
                included: 0,
                overage_quantity: 100,
                unit_rate: Money::usd_micros(1),
                amount: Money::usd_micros(100),
            }],
            total: Money::usd_micros(100),
            status: InvoiceStatus::Finalized,
            idempotency_key: "inv_org-1_2026-06".to_string(),
            created_at: fixed_now(),
        };
        // Push twice; the mock dedups by idempotency key like Stripe does.
        sync.push_invoice_usage(&client, &invoice).await?;
        sync.push_invoice_usage(&client, &invoice).await?;
        let effective = client
            .effective
            .lock()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        assert_eq!(effective.len(), 1, "duplicate push is a no-op on Stripe");
        Ok(())
    }
}
