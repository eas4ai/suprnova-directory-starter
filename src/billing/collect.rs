//! Read existing resources through the retained account, then validate their facts.
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use suprnova::{DB, FrameworkError, serde_json::Value};

use super::{
    evidence::{EvidenceError, Settlement},
    fulfillment::Fact,
    gateway::{Gateway, GatewayError, Resource},
    lifecycle_entities::{payment, purchase},
    paddle_evidence, stripe_evidence as stripe,
};

#[derive(Debug)]
pub(crate) enum CollectError {
    Evidence(&'static str),
    Database(FrameworkError),
}
impl From<EvidenceError> for CollectError {
    fn from(value: EvidenceError) -> Self {
        Self::Evidence(value.0)
    }
}
impl From<GatewayError> for CollectError {
    fn from(value: GatewayError) -> Self {
        Self::Evidence(value.0)
    }
}
impl From<FrameworkError> for CollectError {
    fn from(value: FrameworkError) -> Self {
        Self::Database(value)
    }
}

type Result<T> = std::result::Result<T, CollectError>;

fn reference(value: &Value) -> Result<&str> {
    stripe::id(value).ok_or(CollectError::Evidence("missing_provider_reference"))
}
fn same(value: &Value, id: &str) -> Result<()> {
    if stripe::id(&value["id"]) == Some(id) {
        Ok(())
    } else {
        Err(CollectError::Evidence("wrong_resource"))
    }
}

/// Unsupported notifications receive a receipt without changing the purchase.
pub(crate) fn supported(provider: &str, kind: &str) -> bool {
    if kind.starts_with("reconcile.") {
        return true;
    }
    match provider {
        "stripe" => {
            kind.starts_with("checkout.session.")
                || kind.starts_with("invoice.")
                || kind.starts_with("charge.refund")
                || kind.starts_with("charge.dispute.")
                || kind.starts_with("customer.subscription.")
        }
        "paddle" => {
            kind.starts_with("transaction.")
                || kind.starts_with("subscription.")
                || kind.starts_with("adjustment.")
        }
        _ => false,
    }
}

pub(crate) async fn collect(
    p: &purchase::Model,
    kind: &str,
    data: &Value,
    gateway: &dyn Gateway,
    now: i64,
) -> Result<Vec<Fact>> {
    if !supported(&p.provider, kind) {
        return Ok(vec![Fact::Ignored]);
    }
    match p.provider.as_str() {
        "stripe" => {
            if kind.starts_with("checkout.session.") || kind == "reconcile.checkout" {
                stripe_checkout(p, reference(&data["id"])?, gateway, now).await
            } else if kind.starts_with("invoice.") || kind == "reconcile.invoice" {
                stripe_invoice(p, reference(&data["id"])?, gateway, now, None).await
            } else if kind.starts_with("charge.dispute.") {
                let id = reference(&data["id"])?;
                let dispute = gateway.read(p, Resource::Dispute(id)).await?;
                same(&dispute, id)?;
                let charge_id = reference(&dispute["charge"])?;
                let charge = gateway.read(p, Resource::Charge(charge_id)).await?;
                same(&charge, charge_id)?;
                let paid = stored_payment(p, reference(&charge["payment_intent"])?).await?;
                stripe::dispute(p, &paid, &charge, &dispute, now)?;
                charge_facts(p, &paid, &charge, gateway, now).await
            } else if kind.starts_with("charge.refund") {
                let id = reference(&data["id"])?;
                let charge = gateway.read(p, Resource::Charge(id)).await?;
                same(&charge, id)?;
                let paid = stored_payment(p, reference(&charge["payment_intent"])?).await?;
                charge_facts(p, &paid, &charge, gateway, now).await
            } else {
                stripe_subscription(p, reference(&data["id"])?, gateway, now).await
            }
        }
        "paddle" => {
            if kind.starts_with("subscription.") || kind == "reconcile.subscription" {
                let id = reference(&data["id"])?;
                let subscription = gateway.read(p, Resource::Subscription(id)).await?;
                same(&subscription, id)?;
                let mut facts = vec![Fact::Cancellation(paddle_evidence::cancellation(
                    p,
                    &subscription,
                    now,
                )?)];
                let latest = gateway.read(p, Resource::LatestTransaction(id)).await?;
                if !latest.is_null() {
                    facts.extend(
                        paddle_transaction(p, reference(&latest["id"])?, gateway, now).await?,
                    );
                }
                Ok(facts)
            } else {
                let id = if kind.starts_with("adjustment.") {
                    reference(&data["transaction_id"])?
                } else {
                    reference(&data["id"])?
                };
                paddle_transaction(p, id, gateway, now).await
            }
        }
        _ => Err(CollectError::Evidence("invalid_provider")),
    }
}

async fn stored_payment(p: &purchase::Model, id: &str) -> Result<payment::Model> {
    let db = DB::connection()?;
    payment::Entity::find()
        .filter(payment::Column::PurchaseId.eq(&p.id))
        .filter(payment::Column::ProviderPaymentId.eq(id))
        .one(db.inner())
        .await
        .map_err(crate::listings::database_error)?
        .ok_or(CollectError::Evidence("payment_not_correlated"))
}

fn payment_from(p: &purchase::Model, s: &Settlement) -> payment::Model {
    payment::Model {
        id: String::new(),
        purchase_id: p.id.clone(),
        provider_payment_id: s.payment_ref.clone(),
        amount_total: s.amount_total,
        period_start: s.period_start,
        period_end: s.period_end,
        status: "paid".into(),
        paid_at: s.paid_at,
        refund_event_at: 0,
        dispute_event_at: 0,
        adverse: "{}".into(),
        created_at: s.paid_at,
    }
}

async fn charge_facts(
    p: &purchase::Model,
    paid: &payment::Model,
    charge: &Value,
    gateway: &dyn Gateway,
    now: i64,
) -> Result<Vec<Fact>> {
    let disputes = gateway
        .read(p, Resource::Disputes(reference(&charge["id"])?))
        .await?;
    let state = stripe::adverse(p, paid, charge, &disputes, now)?;
    Ok(vec![Fact::Adverse {
        payment_ref: paid.provider_payment_id.clone(),
        source: "stripe_all".into(),
        state,
    }])
}

async fn stripe_payment_facts(
    p: &purchase::Model,
    settlement: Settlement,
    intent: &Value,
    gateway: &dyn Gateway,
    now: i64,
) -> Result<Vec<Fact>> {
    stripe::captured(p, intent, &settlement.payment_ref, settlement.amount_total)?;
    let paid = payment_from(p, &settlement);
    let mut facts = vec![Fact::Settled(settlement)];
    let charge_id = reference(&intent["latest_charge"])?;
    let charge = gateway.read(p, Resource::Charge(charge_id)).await?;
    same(&charge, charge_id)?;
    facts.extend(charge_facts(p, &paid, &charge, gateway, now).await?);
    Ok(facts)
}

async fn stripe_checkout(
    p: &purchase::Model,
    id: &str,
    gateway: &dyn Gateway,
    now: i64,
) -> Result<Vec<Fact>> {
    let session = gateway.read(p, Resource::Checkout(id)).await?;
    same(&session, id)?;
    if session["status"] == "expired" {
        // Terminal status alone cannot close another purchase's attempt.
        if session["metadata"]["purchase_id"] != p.id
            || stripe::id(&session["customer"]) != p.customer_ref.as_deref()
            || session["livemode"].as_bool() != Some(p.mode == "live")
            || p.session_ref.as_deref().is_some_and(|known| known != id)
        {
            return Err(CollectError::Evidence("wrong_checkout"));
        }
        return Ok(vec![Fact::CheckoutEnded]);
    }
    if p.billing_type == "one_time" {
        let settlement = stripe::one_time(p, &session, now)?;
        let intent = gateway
            .read(p, Resource::PaymentIntent(&settlement.payment_ref))
            .await?;
        stripe_payment_facts(p, settlement, &intent, gateway, now).await
    } else {
        let subscription_id = stripe::checkout_subscription(p, &session)?;
        let subscription = gateway
            .read(p, Resource::Subscription(&subscription_id))
            .await?;
        same(&subscription, &subscription_id)?;
        let invoice_id = stripe::id(&session["invoice"])
            .or_else(|| stripe::id(&subscription["latest_invoice"]))
            .ok_or(CollectError::Evidence("missing_invoice"))?
            .to_owned();
        let mut correlated = p.clone();
        correlated.session_ref = Some(id.into());
        correlated.subscription_ref = Some(subscription_id);
        let mut facts =
            stripe_invoice(&correlated, &invoice_id, gateway, now, Some(subscription)).await?;
        for fact in &mut facts {
            if let Fact::Settled(value) = fact {
                value.session_ref = Some(id.into());
            }
        }
        Ok(facts)
    }
}

async fn stripe_invoice(
    p: &purchase::Model,
    id: &str,
    gateway: &dyn Gateway,
    now: i64,
    subscription: Option<Value>,
) -> Result<Vec<Fact>> {
    let invoice = gateway.read(p, Resource::Invoice(id)).await?;
    same(&invoice, id)?;
    let price_id = p
        .price_id
        .as_deref()
        .ok_or(CollectError::Evidence("missing_price"))?;
    let price = gateway.read(p, Resource::Price(price_id)).await?;
    let intent_id = stripe::invoice_intent(&invoice)
        .ok_or(CollectError::Evidence("missing_payment_reference"))?;
    let intent = gateway.read(p, Resource::PaymentIntent(intent_id)).await?;
    let settlement = stripe::recurring(p, &invoice, &price, &intent, now)?;
    let mut correlated = p.clone();
    correlated.subscription_ref = settlement.subscription_ref.clone();
    let mut facts = stripe_payment_facts(p, settlement, &intent, gateway, now).await?;
    let subscription = match subscription {
        Some(value) => value,
        None => {
            gateway
                .read(
                    p,
                    Resource::Subscription(
                        correlated
                            .subscription_ref
                            .as_deref()
                            .ok_or(CollectError::Evidence("missing_subscription"))?,
                    ),
                )
                .await?
        }
    };
    facts.push(Fact::Cancellation(stripe::cancellation(
        &correlated,
        &subscription,
        now,
    )?));
    Ok(facts)
}

async fn stripe_subscription(
    p: &purchase::Model,
    id: &str,
    gateway: &dyn Gateway,
    now: i64,
) -> Result<Vec<Fact>> {
    let subscription = gateway.read(p, Resource::Subscription(id)).await?;
    same(&subscription, id)?;
    // Validate ownership before using the subscription to find missed invoices.
    let cancellation = stripe::cancellation(p, &subscription, now)?;
    let mut facts = vec![Fact::Cancellation(cancellation)];
    if let Some(invoice) = stripe::id(&subscription["latest_invoice"]) {
        // Unpaid renewal never removes an existing paid period or adds time.
        match stripe_invoice(p, invoice, gateway, now, Some(subscription.clone())).await {
            Ok(paid) => facts.extend(paid),
            Err(CollectError::Evidence("missing_payment_reference" | "payment_not_settled")) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(facts)
}

async fn paddle_transaction(
    p: &purchase::Model,
    id: &str,
    gateway: &dyn Gateway,
    now: i64,
) -> Result<Vec<Fact>> {
    let transaction = gateway.read(p, Resource::Checkout(id)).await?;
    same(&transaction, id)?;
    let mut correlated = p.clone();
    // A lost create response can be repaired from an authenticated transaction
    // with this attempt's unique customer and immutable custom_data correlation.
    if correlated.session_ref.is_none() {
        correlated.session_ref = Some(id.into());
    }
    if transaction["status"] == "canceled" {
        if transaction["custom_data"]["purchase_id"] != p.id
            || transaction["customer_id"].as_str() != p.customer_ref.as_deref()
            || p.session_ref.as_deref().is_some_and(|known| known != id)
        {
            return Err(CollectError::Evidence("wrong_checkout"));
        }
        return Ok(vec![Fact::CheckoutEnded]);
    }
    let settlement = paddle_evidence::settlement(&correlated, &transaction, now)?;
    let paid = payment_from(p, &settlement);
    correlated.subscription_ref = settlement.subscription_ref.clone();
    let adjustments = gateway.read(p, Resource::Adjustments(id)).await?;
    let state = paddle_evidence::adverse(&correlated, &paid, &transaction, &adjustments, now)?;
    let mut facts = vec![
        Fact::Settled(settlement),
        Fact::Adverse {
            payment_ref: id.into(),
            source: "paddle_all".into(),
            state,
        },
    ];
    if let Some(id) = correlated.subscription_ref.as_deref() {
        let subscription = gateway.read(p, Resource::Subscription(id)).await?;
        same(&subscription, id)?;
        facts.push(Fact::Cancellation(paddle_evidence::cancellation(
            &correlated,
            &subscription,
            now,
        )?));
    }
    Ok(facts)
}
