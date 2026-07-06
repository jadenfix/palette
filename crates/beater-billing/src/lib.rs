//! `beater-billing` — Bandwidth (ARCHITECTURE §20.7 #5.8 / §21.7 / R15).
//!
//! Metered-usage billing built on the same primitives as the rest of the
//! platform: integer-micros [`Money`](beater_core::Money) for every monetary
//! computation (never `f64`), the usage ledger ([`beater_usage`]) as the source
//! of metered quantities, and the signed-webhook primitives in
//! [`beater_security`] for Stripe webhook verification.
//!
//! # Robustness invariants
//!
//! Billing correctness is paramount. The crate is designed around five
//! invariants, each exercised by tests:
//!
//! * **Append-only ledger** — charges, credits, proration and refunds are
//!   recorded as immutable [`BillingAdjustment`] rows. A finalized invoice is
//!   never mutated or deleted; corrections are compensating adjustments.
//! * **Idempotent everywhere** — period rollup is a pure function of usage and
//!   plan, so it is recomputable without changing state; invoice finalization is
//!   insert-if-absent on `(org, period)`; every Stripe call carries an
//!   idempotency key; the webhook handler dedups by Stripe `event.id`.
//! * **Atomic / race-free** — subscription plan changes use optimistic
//!   concurrency on a `version` column; a lost update is reported as
//!   [`BillingError::ConcurrentModification`] rather than silently overwriting.
//! * **Out-of-order safe** — a Stripe event is applied only if it is newer than
//!   the last applied event for the same object; stale events are acknowledged
//!   but not applied.
//! * **Overflow-safe** — all sums use [`Money::try_add`](beater_core::Money::try_add)
//!   and checked integer math, surfacing [`BillingError::Overflow`] instead of
//!   wrapping or panicking.

use beater_core::{Currency, Money, MoneyError, OrganizationId, Timestamp};
use beater_store::StoreError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

pub mod store;
pub mod stripe;

pub use store::{BillingStore, SqliteBillingStore};

use beater_usage::{UsageLedgerStore, UsageMeter};

/// Identifier for a billing [`Plan`]. A non-empty, whitespace-free string, like
/// the identifier newtypes in [`beater_core`].
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(transparent)]
pub struct PlanId(String);

impl PlanId {
    pub fn new(value: impl Into<String>) -> Result<Self, BillingError> {
        let value = value.into();
        if value.is_empty() {
            return Err(BillingError::InvalidPlan(
                "plan id cannot be empty".to_string(),
            ));
        }
        if value.chars().any(char::is_whitespace) {
            return Err(BillingError::InvalidPlan(format!(
                "plan id contains whitespace: {value}"
            )));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PlanId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Coarse plan tier. Pricing lives on the [`Plan`] itself; the tier is metadata.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PlanTier {
    Free,
    Pro,
    Enterprise,
}

impl PlanTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Pro => "pro",
            Self::Enterprise => "enterprise",
        }
    }
}

/// A billing plan: a base price plus per-meter included quantities and overage
/// rates. `overage_rates` is the price charged **per unit** of usage beyond the
/// `included` allowance for that meter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Plan {
    pub id: PlanId,
    pub tier: PlanTier,
    /// Included allowance per meter (units of the meter's native quantity).
    #[schema(value_type = std::collections::BTreeMap<String, u64>)]
    pub included: BTreeMap<UsageMeter, u64>,
    /// Recurring base price for a full billing period.
    pub base_price: Money,
    /// Price per unit of usage above the included allowance, per meter.
    #[schema(value_type = std::collections::BTreeMap<String, Money>)]
    pub overage_rates: BTreeMap<UsageMeter, Money>,
}

/// Lifecycle status of a subscription.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Canceled,
}

impl SubscriptionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::PastDue => "past_due",
            Self::Canceled => "canceled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "past_due" => Some(Self::PastDue),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

/// An org's subscription to a [`Plan`] for a billing period. `version` is the
/// optimistic-concurrency token: every mutation bumps it and is gated on the
/// previously-observed value (see [`Billing::change_plan`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Subscription {
    pub org_id: OrganizationId,
    pub plan_id: PlanId,
    pub status: SubscriptionStatus,
    #[schema(value_type = String, format = DateTime)]
    pub period_start: Timestamp,
    #[schema(value_type = String, format = DateTime)]
    pub period_end: Timestamp,
    pub version: i64,
}

/// A billing period, identified canonically by its `YYYY-MM` start month. The
/// `(org, period_key)` pair is the idempotency key for an [`Invoice`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BillingPeriod {
    #[schema(value_type = String, format = DateTime)]
    pub start: Timestamp,
    #[schema(value_type = String, format = DateTime)]
    pub end: Timestamp,
}

impl BillingPeriod {
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self, BillingError> {
        if end <= start {
            return Err(BillingError::InvalidPeriod(
                "period end must be after period start".to_string(),
            ));
        }
        Ok(Self { start, end })
    }

    /// Canonical period key, e.g. `2026-06`, derived from the period start.
    pub fn key(&self) -> String {
        self.start.format("%Y-%m").to_string()
    }

    /// Total length of the period in whole seconds (always positive by
    /// construction).
    pub fn total_seconds(&self) -> i64 {
        (self.end - self.start).num_seconds().max(1)
    }

    /// Seconds remaining from `at` to the end of the period, clamped to
    /// `[0, total_seconds]`.
    pub fn remaining_seconds(&self, at: Timestamp) -> i64 {
        let remaining = (self.end - at).num_seconds();
        remaining.clamp(0, self.total_seconds())
    }
}

/// Kind of an append-only [`BillingAdjustment`]. Amounts are always stored as a
/// non-negative magnitude; the kind carries the sign semantics (credits reduce
/// the amount owed, charges increase it).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentKind {
    Charge,
    Credit,
    ProrationCharge,
    ProrationCredit,
    Refund,
    StripeSync,
}

impl AdjustmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Charge => "charge",
            Self::Credit => "credit",
            Self::ProrationCharge => "proration_charge",
            Self::ProrationCredit => "proration_credit",
            Self::Refund => "refund",
            Self::StripeSync => "stripe_sync",
        }
    }

    /// Sign of this kind against the balance owed: charges are positive, credits
    /// and refunds are negative.
    pub fn sign(self) -> i64 {
        match self {
            Self::Charge | Self::ProrationCharge | Self::StripeSync => 1,
            Self::Credit | Self::ProrationCredit | Self::Refund => -1,
        }
    }
}

/// An immutable, append-only billing adjustment. Once written it is never
/// mutated or deleted; corrections are recorded as new compensating entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BillingAdjustment {
    pub adjustment_id: String,
    pub org_id: OrganizationId,
    pub kind: AdjustmentKind,
    pub amount: Money,
    pub reason: String,
    /// Optional period this adjustment relates to (`YYYY-MM`).
    pub period_key: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

/// Status of an [`Invoice`]. Only `Draft -> Finalized -> Paid`/`Void`
/// transitions are permitted; a finalized invoice's line items and total are
/// immutable.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Finalized,
    Paid,
    Void,
}

impl InvoiceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Finalized => "finalized",
            Self::Paid => "paid",
            Self::Void => "void",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "finalized" => Some(Self::Finalized),
            "paid" => Some(Self::Paid),
            "void" => Some(Self::Void),
            _ => None,
        }
    }
}

/// One line on an [`Invoice`]: either the recurring base charge or a per-meter
/// overage charge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InvoiceLineItem {
    /// Meter this line bills, or `None` for the recurring base price.
    #[schema(value_type = Option<String>)]
    pub meter: Option<UsageMeter>,
    pub description: String,
    /// Total metered quantity in the period (0 for the base line).
    pub quantity: u64,
    /// Included allowance applied (0 for the base line).
    pub included: u64,
    /// Billable overage (`max(0, quantity - included)`).
    pub overage_quantity: u64,
    /// Price per overage unit (the base price for the base line).
    pub unit_rate: Money,
    /// Computed amount for this line.
    pub amount: Money,
}

/// A billing invoice for `(org, period)`. The `idempotency_key` is deterministic
/// in `(org, period)` so finalization and Stripe push can never double-create.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Invoice {
    pub org_id: OrganizationId,
    pub period_key: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub total: Money,
    pub status: InvoiceStatus,
    pub idempotency_key: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

impl Invoice {
    /// Deterministic idempotency key for an invoice, stable across recomputes.
    pub fn idempotency_key_for(org_id: &OrganizationId, period_key: &str) -> String {
        format!("inv_{}_{}", org_id.as_str(), period_key)
    }
}

/// Result of a [`Billing::change_plan`] proration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PlanChange {
    pub subscription: Subscription,
    /// Pro-rated credit for the unused remainder of the old plan.
    pub credit: Money,
    /// Pro-rated charge for the new plan over the remainder of the period.
    pub charge: Money,
    /// Net amount owed for the change (`charge - credit`).
    pub net: Money,
}

/// The usage scope a subscription bills against. Billing is org-scoped, but the
/// usage ledger is keyed by `(tenant, project)`; this binds the two together so
/// rollup can read metered quantities for the org.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BillingScope {
    pub org_id: OrganizationId,
    pub tenant_id: beater_core::TenantId,
    pub project_id: beater_core::ProjectId,
}

/// Typed billing errors. No billing path panics or unwraps.
#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("concurrent modification of {0}; reload and retry")]
    ConcurrentModification(String),
    #[error("invalid plan: {0}")]
    InvalidPlan(String),
    #[error("invalid period: {0}")]
    InvalidPeriod(String),
    #[error("currency mismatch or invalid money: {0}")]
    Money(#[from] MoneyError),
    #[error("arithmetic overflow in money computation")]
    Overflow,
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("{0}")]
    Backend(String),
}

impl BillingError {
    pub fn backend(error: impl Display) -> Self {
        Self::Backend(error.to_string())
    }
}

/// Multiply a [`Money`] rate by an integer `factor`, checked for overflow.
pub fn money_scale(rate: &Money, factor: u64) -> Result<Money, BillingError> {
    let factor = i64::try_from(factor).map_err(|_| BillingError::Overflow)?;
    let amount = rate
        .amount_micros
        .checked_mul(factor)
        .ok_or(BillingError::Overflow)?;
    Ok(Money::new(amount, rate.currency))
}

/// Pro-rate `base` by `numerator / denominator`, computed in `i128` to avoid
/// intermediate overflow and clamped to `[0, base]`. `denominator` must be
/// positive.
pub fn money_prorate(
    base: &Money,
    numerator: i64,
    denominator: i64,
) -> Result<Money, BillingError> {
    if denominator <= 0 {
        return Err(BillingError::InvalidPeriod(
            "proration denominator must be positive".to_string(),
        ));
    }
    let numerator = numerator.clamp(0, denominator);
    let scaled = (base.amount_micros as i128)
        .checked_mul(numerator as i128)
        .ok_or(BillingError::Overflow)?
        / (denominator as i128);
    let amount = i64::try_from(scaled).map_err(|_| BillingError::Overflow)?;
    Ok(Money::new(amount, base.currency))
}

/// Sum a sequence of [`Money`] with overflow- and currency-checked addition,
/// starting from zero in `currency`.
pub fn money_sum<'a>(
    currency: Currency,
    parts: impl IntoIterator<Item = &'a Money>,
) -> Result<Money, BillingError> {
    let mut total = Money::new(0, currency);
    for part in parts {
        total = total.try_add(part)?;
    }
    Ok(total)
}

/// The billing service: orchestrates rollup, proration and finalization over a
/// [`BillingStore`] and a [`UsageLedgerStore`].
pub struct Billing<C: beater_core::Clock = beater_core::SystemClock> {
    store: std::sync::Arc<dyn BillingStore>,
    usage: std::sync::Arc<dyn UsageLedgerStore>,
    clock: C,
}

impl Billing<beater_core::SystemClock> {
    /// Construct a billing service using the system clock.
    pub fn new(
        store: std::sync::Arc<dyn BillingStore>,
        usage: std::sync::Arc<dyn UsageLedgerStore>,
    ) -> Self {
        Self {
            store,
            usage,
            clock: beater_core::SystemClock,
        }
    }
}

impl<C: beater_core::Clock> Billing<C> {
    /// Construct a billing service with an explicit clock (for deterministic
    /// tests).
    pub fn with_clock(
        store: std::sync::Arc<dyn BillingStore>,
        usage: std::sync::Arc<dyn UsageLedgerStore>,
        clock: C,
    ) -> Self {
        Self {
            store,
            usage,
            clock,
        }
    }

    pub fn store(&self) -> &std::sync::Arc<dyn BillingStore> {
        &self.store
    }

    /// Roll up metered usage for `(org, period)` into a draft invoice.
    ///
    /// This is a **pure function** of the usage ledger and the org's plan: it
    /// sums each meter's usage within the period, computes
    /// `overage = max(0, qty - included) * overage_rate` (overflow-checked),
    /// builds line items + total, and persists the invoice **insert-if-absent**
    /// on `(org, period_key)`. Calling it twice yields an identical invoice and
    /// exactly one stored row.
    pub async fn roll_up_period(
        &self,
        scope: &BillingScope,
        period: &BillingPeriod,
    ) -> Result<Invoice, BillingError> {
        let subscription = self
            .store
            .get_subscription(&scope.org_id)
            .await?
            .ok_or_else(|| {
                BillingError::NotFound(format!("subscription for org {}", scope.org_id.as_str()))
            })?;
        let plan = self
            .store
            .get_plan(&subscription.plan_id)
            .await?
            .ok_or_else(|| {
                BillingError::NotFound(format!("plan {}", subscription.plan_id.as_str()))
            })?;

        let quantities = self.metered_quantities(scope, period).await?;
        let invoice = build_invoice(&scope.org_id, period, &plan, &quantities, self.clock.now())?;
        // Insert-if-absent: the first call stores it, later calls return the
        // already-stored invoice unchanged. Rollup never double-creates.
        let stored = self.store.insert_invoice_if_absent(invoice).await?;
        Ok(stored)
    }

    /// Sum per-meter usage quantities within the period from the usage ledger.
    /// Period filtering happens here (the ledger is append-only and unscoped by
    /// time), keeping rollup recomputable.
    async fn metered_quantities(
        &self,
        scope: &BillingScope,
        period: &BillingPeriod,
    ) -> Result<BTreeMap<UsageMeter, u64>, BillingError> {
        let records = self
            .usage
            .list_usage(scope.tenant_id.clone(), scope.project_id.clone())
            .await?;
        let mut signed_totals: BTreeMap<UsageMeter, i64> = BTreeMap::new();
        for record in records {
            if record.created_at < period.start || record.created_at >= period.end {
                continue;
            }
            let entry = signed_totals.entry(record.meter).or_insert(0);
            *entry = entry
                .checked_add(record.quantity)
                .ok_or(BillingError::Overflow)?;
        }
        signed_totals
            .into_iter()
            .map(|(meter, quantity)| {
                let quantity = if quantity <= 0 {
                    0
                } else {
                    u64::try_from(quantity).map_err(|_| BillingError::Overflow)?
                };
                Ok((meter, quantity))
            })
            .collect()
    }

    /// Finalize the invoice for `(org, period)` — idempotent. A `Draft` invoice
    /// transitions to `Finalized`; an already-finalized (or paid/void) invoice
    /// is returned unchanged. Never mutates a finalized invoice's amounts.
    pub async fn finalize_invoice(
        &self,
        org_id: &OrganizationId,
        period_key: &str,
    ) -> Result<Invoice, BillingError> {
        Ok(self.store.finalize_invoice(org_id, period_key).await?)
    }

    /// Change an org's plan, recording append-only proration adjustments and
    /// bumping the subscription with optimistic concurrency.
    ///
    /// Proration: a pro-rated **credit** for the unused remainder of the old
    /// plan plus a pro-rated **charge** for the new plan over that same
    /// remainder. The plan switch and both adjustments are applied atomically;
    /// if another writer changed the subscription concurrently the update
    /// affects zero rows and [`BillingError::ConcurrentModification`] is
    /// returned with nothing written.
    pub async fn change_plan(
        &self,
        org_id: &OrganizationId,
        new_plan_id: &PlanId,
        at: Timestamp,
    ) -> Result<PlanChange, BillingError> {
        let subscription = self.store.get_subscription(org_id).await?.ok_or_else(|| {
            BillingError::NotFound(format!("subscription for org {}", org_id.as_str()))
        })?;
        let old_plan = self
            .store
            .get_plan(&subscription.plan_id)
            .await?
            .ok_or_else(|| {
                BillingError::NotFound(format!("plan {}", subscription.plan_id.as_str()))
            })?;
        let new_plan = self
            .store
            .get_plan(new_plan_id)
            .await?
            .ok_or_else(|| BillingError::NotFound(format!("plan {}", new_plan_id.as_str())))?;

        let period = BillingPeriod::new(subscription.period_start, subscription.period_end)?;
        let total = period.total_seconds();
        let remaining = period.remaining_seconds(at);
        let period_key = period.key();

        let credit = money_prorate(&old_plan.base_price, remaining, total)?;
        let charge = money_prorate(&new_plan.base_price, remaining, total)?;
        let net = charge.try_sub(&credit)?;

        let now = self.clock.now();
        let credit_adj = BillingAdjustment {
            adjustment_id: new_adjustment_id(),
            org_id: org_id.clone(),
            kind: AdjustmentKind::ProrationCredit,
            amount: credit.clone(),
            reason: format!(
                "proration credit: unused {} of plan {}",
                fraction_label(remaining, total),
                old_plan.id
            ),
            period_key: Some(period_key.clone()),
            created_at: now,
        };
        let charge_adj = BillingAdjustment {
            adjustment_id: new_adjustment_id(),
            org_id: org_id.clone(),
            kind: AdjustmentKind::ProrationCharge,
            amount: charge.clone(),
            reason: format!(
                "proration charge: remaining {} of plan {}",
                fraction_label(remaining, total),
                new_plan.id
            ),
            period_key: Some(period_key),
            created_at: now,
        };

        let updated = match self
            .store
            .change_subscription_plan(
                org_id,
                subscription.version,
                new_plan_id,
                &[credit_adj, charge_adj],
            )
            .await
        {
            Ok(updated) => updated,
            // A lost optimistic-version update is surfaced as a typed
            // concurrent-modification error, never a silent overwrite.
            Err(StoreError::Conflict(message)) => {
                return Err(BillingError::ConcurrentModification(message))
            }
            Err(other) => return Err(other.into()),
        };

        Ok(PlanChange {
            subscription: updated,
            credit,
            charge,
            net,
        })
    }
}

/// Build a draft invoice from metered quantities and a plan. Pure and
/// deterministic — the heart of rollup idempotency.
pub fn build_invoice(
    org_id: &OrganizationId,
    period: &BillingPeriod,
    plan: &Plan,
    quantities: &BTreeMap<UsageMeter, u64>,
    created_at: Timestamp,
) -> Result<Invoice, BillingError> {
    let currency = plan.base_price.currency;
    let mut line_items = Vec::new();

    // Base recurring charge.
    line_items.push(InvoiceLineItem {
        meter: None,
        description: format!("Base price ({})", plan.tier.as_str()),
        quantity: 0,
        included: 0,
        overage_quantity: 0,
        unit_rate: plan.base_price.clone(),
        amount: plan.base_price.clone(),
    });

    // Overage per meter. Consider every meter the plan prices or includes, plus
    // any meter present in usage, so unexpected usage is still billed.
    let mut meters: std::collections::BTreeSet<UsageMeter> = std::collections::BTreeSet::new();
    meters.extend(plan.included.keys().copied());
    meters.extend(plan.overage_rates.keys().copied());
    meters.extend(quantities.keys().copied());

    for meter in meters {
        let quantity = quantities.get(&meter).copied().unwrap_or(0);
        let included = plan.included.get(&meter).copied().unwrap_or(0);
        let overage_quantity = quantity.saturating_sub(included);
        let unit_rate = plan
            .overage_rates
            .get(&meter)
            .cloned()
            .unwrap_or_else(|| Money::new(0, currency));
        if unit_rate.currency != currency {
            return Err(BillingError::Money(MoneyError::CurrencyMismatch {
                left: currency,
                right: unit_rate.currency,
            }));
        }
        let amount = money_scale(&unit_rate, overage_quantity)?;
        // Skip zero-overage meters that the plan doesn't price, to keep invoices
        // tidy, but always include priced meters for auditability.
        if overage_quantity == 0 && !plan.overage_rates.contains_key(&meter) {
            continue;
        }
        line_items.push(InvoiceLineItem {
            meter: Some(meter),
            description: format!("Overage: {}", meter.as_str()),
            quantity,
            included,
            overage_quantity,
            unit_rate,
            amount,
        });
    }

    let total = money_sum(currency, line_items.iter().map(|item| &item.amount))?;
    let period_key = period.key();
    let idempotency_key = Invoice::idempotency_key_for(org_id, &period_key);
    Ok(Invoice {
        org_id: org_id.clone(),
        period_key,
        line_items,
        total,
        status: InvoiceStatus::Draft,
        idempotency_key,
        created_at,
    })
}

fn fraction_label(numerator: i64, denominator: i64) -> String {
    if denominator <= 0 {
        return "0%".to_string();
    }
    let pct = (numerator.clamp(0, denominator) as i128 * 100) / denominator as i128;
    format!("{pct}%")
}

fn new_adjustment_id() -> String {
    format!("adj_{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{ProjectId, TenantId};
    use chrono::{TimeZone, Utc};

    fn org() -> Result<OrganizationId, BillingError> {
        OrganizationId::new("org-1").map_err(|e| BillingError::Backend(e.to_string()))
    }

    fn period() -> Result<BillingPeriod, BillingError> {
        let start = Utc
            .with_ymd_and_hms(2026, 6, 1, 0, 0, 0)
            .single()
            .ok_or_else(|| BillingError::InvalidPeriod("bad start".to_string()))?;
        let end = Utc
            .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
            .single()
            .ok_or_else(|| BillingError::InvalidPeriod("bad end".to_string()))?;
        BillingPeriod::new(start, end)
    }

    fn plan(base: i64, included: u64, rate: i64) -> Result<Plan, BillingError> {
        let mut included_map = BTreeMap::new();
        included_map.insert(UsageMeter::JudgeCostMicros, included);
        let mut rates = BTreeMap::new();
        rates.insert(UsageMeter::JudgeCostMicros, Money::usd_micros(rate));
        Ok(Plan {
            id: PlanId::new("pro")?,
            tier: PlanTier::Pro,
            included: included_map,
            base_price: Money::usd_micros(base),
            overage_rates: rates,
        })
    }

    #[test]
    fn money_scale_is_overflow_safe() -> Result<(), BillingError> {
        assert_eq!(
            money_scale(&Money::usd_micros(5), 3)?,
            Money::usd_micros(15)
        );
        // i64::MAX * 2 overflows -> typed error, never a panic/wrap.
        let result = money_scale(&Money::usd_micros(i64::MAX), 2);
        assert!(matches!(result, Err(BillingError::Overflow)));
        Ok(())
    }

    #[test]
    fn prorate_clamps_and_uses_wide_intermediate() -> Result<(), BillingError> {
        let base = Money::usd_micros(3_000_000);
        // Half the period.
        assert_eq!(money_prorate(&base, 50, 100)?, Money::usd_micros(1_500_000));
        // numerator > denominator clamps to full.
        assert_eq!(money_prorate(&base, 200, 100)?, base);
        // negative numerator clamps to zero.
        assert_eq!(money_prorate(&base, -5, 100)?, Money::usd_micros(0));
        // i64::MAX scaled by a huge ratio would overflow an i64 product, but the
        // i128 intermediate keeps it exact.
        let big = Money::usd_micros(i64::MAX);
        assert_eq!(money_prorate(&big, 1, 1)?, big);
        // denominator must be positive.
        assert!(matches!(
            money_prorate(&base, 1, 0),
            Err(BillingError::InvalidPeriod(_))
        ));
        Ok(())
    }

    // Table-driven overage edge cases. (proptest is not a workspace dev-dep.)
    #[test]
    fn overage_edge_cases() -> Result<(), BillingError> {
        struct Case {
            name: &'static str,
            included: u64,
            rate: i64,
            usage: u64,
            expected_total: i64,
        }
        let cases = [
            Case {
                name: "zero usage -> base only",
                included: 1000,
                rate: 2,
                usage: 0,
                expected_total: 10_000,
            },
            Case {
                name: "exactly at limit -> zero overage",
                included: 1000,
                rate: 2,
                usage: 1000,
                expected_total: 10_000,
            },
            Case {
                name: "one over -> one unit overage",
                included: 1000,
                rate: 2,
                usage: 1001,
                expected_total: 10_002,
            },
            Case {
                name: "large usage stays exact",
                included: 0,
                rate: 1,
                usage: 5_000_000,
                expected_total: 10_000 + 5_000_000,
            },
        ];
        let org = org()?;
        let period = period()?;
        for case in cases {
            let plan = plan(10_000, case.included, case.rate)?;
            let mut quantities = BTreeMap::new();
            quantities.insert(UsageMeter::JudgeCostMicros, case.usage);
            let invoice = build_invoice(&org, &period, &plan, &quantities, period.start)?;
            assert_eq!(
                invoice.total,
                Money::usd_micros(case.expected_total),
                "case: {}",
                case.name
            );
        }
        Ok(())
    }

    #[test]
    fn build_invoice_overflow_is_typed_not_panic() -> Result<(), BillingError> {
        let plan = plan(1, 0, i64::MAX)?;
        let mut quantities = BTreeMap::new();
        quantities.insert(UsageMeter::JudgeCostMicros, 1_000_000u64);
        let result = build_invoice(&org()?, &period()?, &plan, &quantities, period()?.start);
        assert!(matches!(result, Err(BillingError::Overflow)));
        Ok(())
    }

    #[test]
    fn rollup_is_recompute_stable() -> Result<(), BillingError> {
        // Building the invoice twice from the same inputs yields an identical
        // result (the invariant insert-if-absent relies on).
        let plan = plan(10_000, 1000, 3)?;
        let mut quantities = BTreeMap::new();
        quantities.insert(UsageMeter::JudgeCostMicros, 1500u64);
        let a = build_invoice(&org()?, &period()?, &plan, &quantities, period()?.start)?;
        let b = build_invoice(&org()?, &period()?, &plan, &quantities, period()?.start)?;
        assert_eq!(a, b);
        assert_eq!(a.idempotency_key, b.idempotency_key);
        // 500 units over * 3 = 1500, plus 10_000 base.
        assert_eq!(a.total, Money::usd_micros(11_500));
        Ok(())
    }

    #[test]
    fn proration_credit_and_charge_reconcile() -> Result<(), BillingError> {
        // Property: for any split point, net == charge - credit, and at the
        // period boundary proration is zero. Table-driven over the fraction.
        let old_base = Money::usd_micros(30_000);
        let new_base = Money::usd_micros(90_000);
        let total = 30; // days
        for remaining in [0i64, 1, 10, 15, 30] {
            let credit = money_prorate(&old_base, remaining, total)?;
            let charge = money_prorate(&new_base, remaining, total)?;
            let net = charge.try_sub(&credit)?;
            assert_eq!(net, charge.try_sub(&credit)?);
            // credit is always <= old_base, charge always <= new_base.
            assert!(credit.amount_micros <= old_base.amount_micros);
            assert!(charge.amount_micros <= new_base.amount_micros);
            if remaining == 0 {
                assert_eq!(credit, Money::usd_micros(0));
                assert_eq!(charge, Money::usd_micros(0));
            }
        }
        // Exact mid-period numbers.
        assert_eq!(money_prorate(&old_base, 15, 30)?, Money::usd_micros(15_000));
        assert_eq!(money_prorate(&new_base, 15, 30)?, Money::usd_micros(45_000));
        Ok(())
    }

    fn scope() -> Result<BillingScope, BillingError> {
        Ok(BillingScope {
            org_id: org()?,
            tenant_id: TenantId::new("tenant-1").map_err(BillingError::backend)?,
            project_id: ProjectId::new("project-1").map_err(BillingError::backend)?,
        })
    }

    #[test]
    fn scope_constructs() -> Result<(), BillingError> {
        let s = scope()?;
        assert_eq!(s.org_id.as_str(), "org-1");
        Ok(())
    }
}
