use super::{
    evidence::{AdverseState, Cancellation, EvidenceError, Settlement},
    lifecycle_entities::{payment, purchase},
};
use suprnova::serde_json::Value;

fn check(condition: bool, code: &'static str) -> Result<(), EvidenceError> {
    if condition {
        Ok(())
    } else {
        Err(EvidenceError(code))
    }
}
pub(crate) fn id(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("id").and_then(Value::as_str))
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 255
                && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        })
}
fn number(value: &Value) -> Result<i64, EvidenceError> {
    value
        .as_i64()
        .filter(|n| *n >= 0)
        .ok_or(EvidenceError("missing_payment_amount"))
}
fn currency(p: &purchase::Model, value: &Value) -> Result<(), EvidenceError> {
    check(
        value
            .as_str()
            .is_some_and(|c| c.eq_ignore_ascii_case(&p.currency)),
        "wrong_currency",
    )
}
fn identity(p: &purchase::Model, data: &Value) -> Result<(), EvidenceError> {
    check(
        p.provider == "stripe" && matches!(p.mode.as_str(), "test" | "live"),
        "wrong_provider_mode",
    )?;
    check(
        data["livemode"].as_bool() == Some(p.mode == "live"),
        "wrong_mode",
    )?;
    check(
        p.customer_ref.is_some() && id(&data["customer"]) == p.customer_ref.as_deref(),
        "wrong_customer",
    )
}
fn correlated(p: &purchase::Model, metadata: &Value) -> Result<(), EvidenceError> {
    check(
        metadata["purchase_id"].as_str() == Some(&p.id),
        "wrong_purchase",
    )
}
fn single(data: &Value) -> Result<&Value, EvidenceError> {
    check(
        data["has_more"].as_bool() == Some(false),
        "incomplete_line_items",
    )?;
    let items = data["data"]
        .as_array()
        .ok_or(EvidenceError("missing_line_items"))?;
    check(items.len() == 1, "wrong_quantity")?;
    Ok(&items[0])
}
fn price(p: &purchase::Model, value: &Value) -> Result<(), EvidenceError> {
    check(id(value) == p.price_id.as_deref(), "wrong_price")?;
    currency(p, &value["currency"])?;
    check(
        number(&value["unit_amount"])? == p.amount,
        "wrong_base_amount",
    )?;
    match p.billing_type.as_str() {
        "one_time" => check(
            value["type"] == "one_time" && value["recurring"].is_null(),
            "wrong_billing_type",
        ),
        "monthly" | "annual" => check(
            value["type"] == "recurring"
                && value["recurring"]["interval"]
                    == if p.billing_type == "monthly" {
                        "month"
                    } else {
                        "year"
                    }
                && value["recurring"]["interval_count"].as_i64() == Some(1)
                && value["recurring"]["usage_type"] == "licensed",
            "wrong_billing_type",
        ),
        _ => Err(EvidenceError("wrong_billing_type")),
    }
}

pub fn one_time(
    p: &purchase::Model,
    session: &Value,
    observed_at: i64,
) -> Result<Settlement, EvidenceError> {
    identity(p, session)?;
    correlated(p, &session["metadata"])?;
    check(
        p.billing_type == "one_time" && session["mode"] == "payment",
        "wrong_billing_type",
    )?;
    let session_id = id(&session["id"])
        .filter(|id| id.starts_with("cs_"))
        .ok_or(EvidenceError("missing_session"))?;
    check(
        p.session_ref
            .as_deref()
            .is_none_or(|known| known == session_id),
        "wrong_session",
    )?;
    check(
        session["status"] == "complete" && session["payment_status"] == "paid",
        "payment_not_settled",
    )?;
    let line = single(&session["line_items"])?;
    check(line["quantity"].as_i64() == Some(1), "wrong_quantity")?;
    price(p, &line["price"])?;
    currency(p, &line["currency"])?;
    currency(p, &session["currency"])?;
    let subtotal = number(&line["amount_subtotal"])?;
    check(
        subtotal == p.amount && number(&session["amount_subtotal"])? == subtotal,
        "wrong_base_amount",
    )?;
    let total = number(&session["amount_total"])?;
    check(
        total > 0 && number(&line["amount_total"])? == total,
        "payment_not_settled",
    )?;
    let payment = id(&session["payment_intent"])
        .filter(|id| id.starts_with("pi_"))
        .ok_or(EvidenceError("missing_payment_reference"))?;
    check(
        observed_at >= p.created_at && observed_at > 0,
        "invalid_payment_time",
    )?;
    Ok(Settlement {
        payment_ref: payment.to_owned(),
        customer_ref: p
            .customer_ref
            .clone()
            .ok_or(EvidenceError("wrong_customer"))?,
        session_ref: Some(session_id.to_owned()),
        subscription_ref: None,
        amount_total: total,
        period_start: observed_at,
        period_end: None,
        paid_at: observed_at,
        references: vec![
            ("session".to_owned(), session_id.to_owned()),
            ("payment".to_owned(), payment.to_owned()),
        ],
    })
}

pub(crate) fn invoice_subscription(invoice: &Value) -> Option<&str> {
    id(&invoice["subscription"])
        .or_else(|| id(&invoice["parent"]["subscription_details"]["subscription"]))
}
pub(crate) fn invoice_intent(invoice: &Value) -> Option<&str> {
    id(&invoice["payment_intent"]).or_else(|| {
        invoice["payments"]["data"]
            .as_array()?
            .iter()
            .find_map(|payment| {
                (payment["status"] == "paid")
                    .then(|| id(&payment["payment"]["payment_intent"]))
                    .flatten()
            })
    })
}

pub(crate) fn checkout_subscription(
    p: &purchase::Model,
    session: &Value,
) -> Result<String, EvidenceError> {
    identity(p, session)?;
    correlated(p, &session["metadata"])?;
    check(
        session["mode"] == "subscription"
            && matches!(p.billing_type.as_str(), "monthly" | "annual"),
        "wrong_billing_type",
    )?;
    check(
        id(&session["id"]).is_some()
            && p.session_ref
                .as_deref()
                .is_none_or(|known| id(&session["id"]) == Some(known)),
        "wrong_session",
    )?;
    let subscription = id(&session["subscription"]).ok_or(EvidenceError("missing_subscription"))?;
    check(
        p.subscription_ref
            .as_deref()
            .is_none_or(|known| known == subscription),
        "wrong_subscription",
    )?;
    Ok(subscription.to_owned())
}

pub(crate) fn captured(
    p: &purchase::Model,
    intent: &Value,
    payment: &str,
    amount: i64,
) -> Result<(), EvidenceError> {
    identity(p, intent)?;
    currency(p, &intent["currency"])?;
    check(
        id(&intent["id"]) == Some(payment)
            && intent["status"] == "succeeded"
            && number(&intent["amount_received"])? == amount,
        "payment_not_settled",
    )
}

pub fn recurring(
    p: &purchase::Model,
    invoice: &Value,
    catalog_price: &Value,
    intent: &Value,
    observed_at: i64,
) -> Result<Settlement, EvidenceError> {
    identity(p, invoice)?;
    identity(p, intent)?;
    check(
        matches!(p.billing_type.as_str(), "monthly" | "annual"),
        "wrong_billing_type",
    )?;
    let metadata = if invoice["parent"]["subscription_details"]["metadata"].is_object() {
        &invoice["parent"]["subscription_details"]["metadata"]
    } else {
        &invoice["subscription_details"]["metadata"]
    };
    correlated(p, metadata)?;
    let subscription =
        invoice_subscription(invoice).ok_or(EvidenceError("missing_subscription"))?;
    check(
        p.subscription_ref
            .as_deref()
            .is_none_or(|known| known == subscription),
        "wrong_subscription",
    )?;
    check(
        invoice["status"] == "paid"
            // Basil removed this field. The succeeded PaymentIntent and exact
            // received amount below establish collection on current API versions.
            && invoice["paid_out_of_band"].as_bool() != Some(true)
            && number(&invoice["amount_remaining"])? == 0,
        "payment_not_settled",
    )?;
    let amount = number(&invoice["amount_paid"])?;
    check(
        amount > 0 && amount == number(&invoice["amount_due"])?,
        "payment_not_settled",
    )?;
    currency(p, &invoice["currency"])?;
    let payment = invoice_intent(invoice).ok_or(EvidenceError("missing_payment_reference"))?;
    check(
        id(&intent["id"]) == Some(payment)
            && intent["status"] == "succeeded"
            && number(&intent["amount_received"])? == amount,
        "payment_not_settled",
    )?;
    currency(p, &intent["currency"])?;
    let line = single(&invoice["lines"])?;
    let line_price = id(&line["price"]).or_else(|| id(&line["pricing"]["price_details"]["price"]));
    check(line_price == p.price_id.as_deref(), "wrong_price")?;
    price(p, catalog_price)?;
    check(line["quantity"].as_i64() == Some(1), "wrong_quantity")?;
    check(
        line["proration"]
            .as_bool()
            .or_else(|| line["parent"]["subscription_item_details"]["proration"].as_bool())
            == Some(false),
        "unsupported_proration",
    )?;
    check(number(&line["amount"])? == p.amount, "wrong_base_amount")?;
    currency(p, &line["currency"])?;
    let start = number(&line["period"]["start"])?;
    let end = number(&line["period"]["end"])?;
    let paid_at = number(&invoice["status_transitions"]["paid_at"])?;
    check(
        start > 0 && end > start && paid_at > 0 && paid_at <= observed_at + 300,
        "invalid_paid_period",
    )?;
    let invoice_id = id(&invoice["id"]).ok_or(EvidenceError("missing_invoice"))?;
    let mut references = vec![
        ("invoice".to_owned(), invoice_id.to_owned()),
        ("payment".to_owned(), payment.to_owned()),
        ("subscription".to_owned(), subscription.to_owned()),
    ];
    if let Some(charge) = id(&intent["latest_charge"]) {
        references.push(("charge".to_owned(), charge.to_owned()));
    }
    Ok(Settlement {
        payment_ref: payment.to_owned(),
        customer_ref: p
            .customer_ref
            .clone()
            .ok_or(EvidenceError("wrong_customer"))?,
        session_ref: None,
        subscription_ref: Some(subscription.to_owned()),
        amount_total: amount,
        period_start: start,
        period_end: Some(end),
        paid_at,
        references,
    })
}

pub fn refund(
    p: &purchase::Model,
    payment: &payment::Model,
    charge: &Value,
    observed_at: i64,
) -> Result<AdverseState, EvidenceError> {
    identity(p, charge)?;
    currency(p, &charge["currency"])?;
    check(
        payment.purchase_id == p.id
            && id(&charge["payment_intent"]) == Some(&payment.provider_payment_id)
            && charge["paid"] == true
            && charge["status"] == "succeeded",
        "wrong_payment",
    )?;
    check(
        number(&charge["amount"])? == payment.amount_total,
        "wrong_payment_amount",
    )?;
    let refunded = number(&charge["amount_refunded"])?;
    check(refunded <= payment.amount_total, "invalid_refund_amount")?;
    Ok(AdverseState {
        refund_total: refunded,
        disputed: false,
        lost_dispute: false,
        observed_at,
    })
}

pub fn dispute(
    p: &purchase::Model,
    payment: &payment::Model,
    charge: &Value,
    dispute: &Value,
    observed_at: i64,
) -> Result<AdverseState, EvidenceError> {
    let mut state = refund(p, payment, charge, observed_at)?;
    check(
        id(&dispute["charge"]) == id(&charge["id"]) && id(&dispute["charge"]).is_some(),
        "wrong_payment",
    )?;
    check(
        dispute["livemode"].as_bool() == Some(p.mode == "live"),
        "wrong_mode",
    )?;
    currency(p, &dispute["currency"])?;
    match dispute["status"].as_str() {
        Some(
            "needs_response" | "under_review" | "warning_needs_response" | "warning_under_review",
        ) => state.disputed = true,
        Some("lost") => state.lost_dispute = true,
        // Stripe's current won outcome confirms returned funds. Charge.disputed
        // describes dispute history, not whether this dispute remains open.
        Some("won" | "warning_closed") => {}
        _ => return Err(EvidenceError("unknown_dispute_state")),
    }
    Ok(state)
}

/// Call only with the complete current dispute list for this charge. A won
/// dispute cannot clear a different open/lost dispute or a full refund.
pub(crate) fn adverse(
    p: &purchase::Model,
    payment: &payment::Model,
    charge: &Value,
    disputes: &Value,
    observed_at: i64,
) -> Result<AdverseState, EvidenceError> {
    let mut state = refund(p, payment, charge, observed_at)?;
    let rows = disputes
        .as_array()
        .ok_or(EvidenceError("missing_dispute_state"))?;
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let reference = id(&row["id"]).ok_or(EvidenceError("missing_dispute_state"))?;
        check(seen.insert(reference), "duplicate_dispute")?;
        let current = dispute(p, payment, charge, row, observed_at)?;
        state.disputed |= current.disputed;
        state.lost_dispute |= current.lost_dispute;
    }
    check(
        charge["disputed"].as_bool().is_some(),
        "missing_dispute_state",
    )?;
    check(
        charge["disputed"] != true || !rows.is_empty(),
        "dispute_list_pending",
    )?;
    Ok(state)
}

pub fn cancellation(
    p: &purchase::Model,
    subscription: &Value,
    observed_at: i64,
) -> Result<Cancellation, EvidenceError> {
    identity(p, subscription)?;
    correlated(p, &subscription["metadata"])?;
    check(
        p.subscription_ref.is_some() && id(&subscription["id"]) == p.subscription_ref.as_deref(),
        "wrong_subscription",
    )?;
    let mut result = Cancellation {
        effective_at: None,
        scheduled_at: None,
        observed_at,
    };
    if subscription["status"] == "canceled" {
        // canceled_at can be the earlier scheduling time; ended_at is effective.
        let end = number(&subscription["ended_at"])?;
        check(
            end > 0 && end <= observed_at + 300,
            "invalid_cancellation_time",
        )?;
        result.effective_at = Some(end);
    } else if subscription["cancel_at_period_end"].as_bool() == Some(true)
        || subscription["cancel_at"].as_i64().is_some()
    {
        let end = subscription["cancel_at"]
            .as_i64()
            .or_else(|| subscription["current_period_end"].as_i64())
            .or_else(|| {
                subscription["items"]["data"].as_array()?.first()?["current_period_end"].as_i64()
            })
            .ok_or(EvidenceError("missing_cancellation_time"))?;
        check(end > 0, "invalid_cancellation_time")?;
        result.scheduled_at = Some(end);
    }
    Ok(result)
}
