//! Fail-closed Paddle facts. Callers authenticate the account and environment before use.
use super::{
    evidence::{AdverseState, Cancellation, EvidenceError, Settlement},
    lifecycle_entities::{payment, purchase},
};
use std::collections::BTreeMap;
use suprnova::serde_json::Value;

type Result<T> = std::result::Result<T, EvidenceError>;
fn check(ok: bool, code: &'static str) -> Result<()> {
    if ok { Ok(()) } else { Err(EvidenceError(code)) }
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(EvidenceError("paddle_missing_field"))
}
fn money(v: &Value) -> Result<i64> {
    let s = v.as_str().ok_or(EvidenceError("paddle_missing_amount"))?;
    check(
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()),
        "paddle_invalid_amount",
    )?;
    s.parse()
        .map_err(|_| EvidenceError("paddle_invalid_amount"))
}
fn add(a: i64, b: i64) -> Result<i64> {
    a.checked_add(b)
        .ok_or(EvidenceError("paddle_amount_overflow"))
}
fn time(v: &Value) -> Result<i64> {
    let s = v.as_str().ok_or(EvidenceError("paddle_missing_time"))?;
    let t = chrono::DateTime::parse_from_rfc3339(s)
        .map_err(|_| EvidenceError("paddle_invalid_time"))?
        .timestamp();
    check(t >= 0, "paddle_invalid_time")?;
    Ok(t)
}
fn bound(p: &purchase::Model, v: &Value) -> Result<()> {
    check(
        p.provider == "paddle" && matches!(p.mode.as_str(), "test" | "live"),
        "paddle_wrong_environment",
    )?;
    if let Some(mode) = v["custom_data"].get("mode") {
        check(
            mode.as_str() == Some(p.mode.as_str()),
            "paddle_wrong_environment",
        )?;
    }
    check(
        v["custom_data"]["purchase_id"].as_str() == Some(p.id.as_str()),
        "paddle_wrong_purchase",
    )?;
    check(
        p.customer_ref.as_deref() == Some(string(v, "customer_id")?),
        "paddle_wrong_customer",
    )
}
fn item(p: &purchase::Model, v: &Value) -> Result<()> {
    check(v["quantity"].as_u64() == Some(1), "paddle_wrong_quantity")?;
    let price = &v["price"];
    check(
        p.price_id.as_deref() == Some(string(price, "id")?),
        "paddle_wrong_price",
    )?;
    check(
        money(&price["unit_price"]["amount"])? == p.amount
            && price["unit_price"]["currency_code"].as_str() == Some(p.currency.as_str()),
        "paddle_wrong_price_amount",
    )?;
    let cycle = price
        .get("billing_cycle")
        .ok_or(EvidenceError("paddle_missing_cycle"))?;
    check(
        match p.billing_type.as_str() {
            "one_time" => cycle.is_null(),
            "monthly" => cycle["interval"] == "month" && cycle["frequency"].as_u64() == Some(1),
            "annual" => cycle["interval"] == "year" && cycle["frequency"].as_u64() == Some(1),
            _ => false,
        },
        "paddle_wrong_cycle",
    )?;
    check(
        v.get("proration").is_none_or(Value::is_null),
        "paddle_proration_unsupported",
    )
}
/// A completed transaction with actual captured funds, tied to immutable purchase terms.
pub fn settlement(p: &purchase::Model, data: &Value, observed_at: i64) -> Result<Settlement> {
    bound(p, data)?;
    let id = string(data, "id")?;
    let subscription = data
        .get("subscription_id")
        .ok_or(EvidenceError("paddle_missing_subscription"))?;
    let subscription_ref = if p.billing_type == "one_time" {
        check(subscription.is_null(), "paddle_wrong_subscription")?;
        None
    } else {
        Some(
            subscription
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or(EvidenceError("paddle_missing_subscription"))?
                .to_owned(),
        )
    };
    if let Some(expected) = &p.subscription_ref {
        check(
            subscription_ref.as_ref() == Some(expected),
            "paddle_wrong_subscription",
        )?;
    }
    check(
        p.session_ref.as_deref() == Some(id)
            || (p.subscription_ref.is_some() && p.subscription_ref == subscription_ref),
        "paddle_wrong_transaction",
    )?;
    check(data["status"] == "completed", "paddle_not_completed")?;
    check(
        data["currency_code"].as_str() == Some(p.currency.as_str()),
        "paddle_wrong_currency",
    )?;
    let items = data["items"]
        .as_array()
        .ok_or(EvidenceError("paddle_missing_items"))?;
    check(items.len() == 1, "paddle_wrong_items")?;
    item(p, &items[0])?;
    let lines = data["details"]["line_items"]
        .as_array()
        .ok_or(EvidenceError("paddle_missing_lines"))?;
    check(lines.len() == 1, "paddle_wrong_lines")?;
    let line = &lines[0];
    check(
        line.get("proration").is_none_or(Value::is_null),
        "paddle_proration_unsupported",
    )?;
    check(
        line["price_id"].as_str() == p.price_id.as_deref() && line["quantity"].as_u64() == Some(1),
        "paddle_wrong_line",
    )?;
    let totals = &data["details"]["totals"];
    check(
        totals["currency_code"].as_str() == Some(p.currency.as_str()),
        "paddle_wrong_currency",
    )?;
    for key in ["subtotal", "discount", "tax", "total"] {
        check(
            money(&line["totals"][key])? == money(&totals[key])?,
            "paddle_inconsistent_lines",
        )?;
    }
    let subtotal = money(&totals["subtotal"])?;
    let discount = money(&totals["discount"])?;
    let tax = money(&totals["tax"])?;
    let total = money(&totals["total"])?;
    check(
        discount <= subtotal && add(subtotal - discount, tax)? == total,
        "paddle_inconsistent_totals",
    )?;
    let tax_mode = string(&items[0]["price"], "tax_mode")?;
    let external = subtotal == p.amount;
    // Inclusive discounted prices need a tax allocation proof; retain pending until supported.
    let internal = discount == 0 && add(subtotal, tax)? == p.amount;
    check(
        match tax_mode {
            "external" => external,
            "internal" => internal,
            "location" | "account_setting" => external || internal,
            _ => false,
        },
        "paddle_ambiguous_tax_basis",
    )?;
    let credit = money(&totals["credit"])?;
    let amount_total = money(&totals["grand_total"])?;
    check(
        credit <= total
            && total - credit == amount_total
            && amount_total > 0
            && money(&totals["balance"])? == 0
            && money(&totals["credit_to_balance"])? == 0,
        "paddle_no_retained_capture",
    )?;
    let payments = data["payments"]
        .as_array()
        .ok_or(EvidenceError("paddle_missing_payments"))?;
    let mut seen = BTreeMap::new();
    let mut captured = 0;
    let mut paid_at = 0;
    let mut references = vec![
        ("transaction".into(), id.into()),
        ("line_item".into(), string(line, "id")?.into()),
    ];
    for payment in payments {
        let attempt = string(payment, "payment_attempt_id")?;
        if let Some(previous) = seen.insert(attempt, payment) {
            check(previous == payment, "paddle_conflicting_capture")?;
            continue;
        }
        if payment["status"] != "captured" {
            continue;
        }
        let amount = money(&payment["amount"])?;
        check(amount > 0, "paddle_invalid_capture")?;
        let at = time(&payment["captured_at"])?;
        check(at <= observed_at, "paddle_future_capture")?;
        captured = add(captured, amount)?;
        paid_at = paid_at.max(at);
        references.push(("payment_attempt".into(), attempt.into()));
    }
    check(
        captured >= amount_total && paid_at > 0,
        "paddle_no_retained_capture",
    )?;
    let (period_start, period_end) = if subscription_ref.is_some() {
        let start = time(&data["billing_period"]["starts_at"])?;
        let end = time(&data["billing_period"]["ends_at"])?;
        check(start < end && start <= observed_at, "paddle_invalid_period")?;
        (start, Some(end))
    } else {
        (paid_at, None)
    };
    Ok(Settlement {
        payment_ref: id.into(),
        customer_ref: string(data, "customer_id")?.into(),
        session_ref: (p.session_ref.as_deref() == Some(id)).then(|| id.into()),
        subscription_ref,
        amount_total,
        period_start,
        period_end,
        paid_at,
        references,
    })
}

// Adjustment item IDs identify different adjustment rows, not the original paid item.
// The SDK omits those IDs; matching must have identical meaning for raw and SDK JSON.
fn same_adjustment_items(left: &Value, right: &Value) -> bool {
    let (Some(left), Some(right)) = (left.as_array(), right.as_array()) else {
        return false;
    };
    left.len() == right.len()
        && left.iter().all(|a| {
            right
                .iter()
                .filter(|b| {
                    ["item_id", "type", "amount", "proration", "totals"]
                        .iter()
                        .all(|key| a[*key] == b[*key])
                })
                .count()
                == 1
        })
}

/// Derive adverse state from the complete current adjustment list, never one webhook delta.
pub fn adverse(
    p: &purchase::Model,
    payment: &payment::Model,
    transaction: &Value,
    adjustments: &Value,
    observed_at: i64,
) -> Result<AdverseState> {
    let settled = settlement(p, transaction, observed_at)?;
    check(
        payment.purchase_id == p.id
            && settled.payment_ref == payment.provider_payment_id
            && settled.amount_total == payment.amount_total
            && settled.period_start == payment.period_start
            && settled.period_end == payment.period_end,
        "paddle_wrong_payment",
    )?;
    let rows = adjustments
        .as_array()
        .ok_or(EvidenceError("paddle_missing_adjustments"))?;
    let mut seen = BTreeMap::new();
    let mut refund_total = 0;
    let mut disputed = false;
    let mut lost_dispute = false;
    for row in rows {
        let id = string(row, "id")?;
        if let Some(previous) = seen.insert(id, row) {
            check(previous == row, "paddle_conflicting_adjustment")?;
            continue;
        }
        check(
            row["transaction_id"].as_str() == Some(payment.provider_payment_id.as_str())
                && row["customer_id"].as_str() == p.customer_ref.as_deref()
                && row["currency_code"].as_str() == Some(p.currency.as_str()),
            "paddle_wrong_adjustment",
        )?;
        check(
            row.get("subscription_id").is_some()
                && row["subscription_id"].as_str() == settled.subscription_ref.as_deref(),
            "paddle_wrong_adjustment",
        )?;
        let amount = money(&row["totals"]["total"])?;
        check(
            amount > 0 && amount <= payment.amount_total,
            "paddle_invalid_adjustment_amount",
        )?;
        let row_items = row["items"]
            .as_array()
            .ok_or(EvidenceError("paddle_missing_adjustment_items"))?;
        for adjusted in row_items {
            check(
                adjusted["item_id"].as_str()
                    == transaction["details"]["line_items"][0]["id"].as_str(),
                "paddle_wrong_adjustment_item",
            )?;
        }
        match string(row, "type")? {
            "full" => check(
                amount == payment.amount_total,
                "paddle_invalid_full_adjustment",
            )?,
            "partial" => check(!row_items.is_empty(), "paddle_missing_adjustment_items")?,
            _ => return Err(EvidenceError("paddle_unknown_adjustment_type")),
        }
        let action = string(row, "action")?;
        check(
            matches!(
                action,
                "refund"
                    | "credit"
                    | "credit_reverse"
                    | "chargeback"
                    | "chargeback_warning"
                    | "chargeback_reverse"
                    | "chargeback_warning_reverse"
            ),
            "paddle_unknown_adjustment_action",
        )?;
        let status = string(row, "status")?;
        check(
            matches!(
                status,
                "approved" | "pending_approval" | "rejected" | "reversed"
            ),
            "paddle_unknown_adjustment_status",
        )?;
        if action.ends_with("_reverse") {
            let original_action = action.strip_suffix("_reverse").unwrap_or_default();
            let matches = rows
                .iter()
                .filter(|candidate| {
                    candidate["action"] == original_action
                        && candidate["status"] == "reversed"
                        && candidate["totals"]["total"] == row["totals"]["total"]
                        && same_adjustment_items(&candidate["items"], &row["items"])
                })
                .count();
            check(matches == 1, "paddle_ambiguous_reversal")?;
        }
        if status != "approved" {
            continue;
        }
        match action {
            "refund" => refund_total = add(refund_total, amount)?,
            "chargeback_warning" => disputed = true,
            "chargeback" => {
                disputed = true;
                lost_dispute = true;
            }
            _ => {}
        }
    }
    check(refund_total <= payment.amount_total, "paddle_excess_refund")?;
    if !disputed && refund_total < payment.amount_total {
        let adjusted = &transaction["details"]["adjusted_totals"];
        check(
            adjusted["currency_code"].as_str() == Some(p.currency.as_str()),
            "paddle_wrong_currency",
        )?;
        let retained = money(&adjusted["grand_total"])?;
        check(
            retained > 0
                && retained <= payment.amount_total
                && retained == payment.amount_total - refund_total,
            "paddle_retention_unproven",
        )?;
    }
    Ok(AdverseState {
        refund_total,
        disputed,
        lost_dispute,
        observed_at,
    })
}

/// Cancellation changes only the ending boundary; it never establishes paid access.
pub fn cancellation(p: &purchase::Model, data: &Value, observed_at: i64) -> Result<Cancellation> {
    bound(p, data)?;
    check(
        p.subscription_ref.as_deref() == Some(string(data, "id")?),
        "paddle_wrong_subscription",
    )?;
    if let Some(items) = data.get("items") {
        let items = items
            .as_array()
            .ok_or(EvidenceError("paddle_missing_items"))?;
        check(items.len() == 1, "paddle_wrong_items")?;
        item(p, &items[0])?;
    }
    let updated = time(&data["updated_at"])?;
    check(updated <= observed_at, "paddle_future_update")?;
    let mut result = Cancellation {
        effective_at: None,
        scheduled_at: None,
        observed_at: updated,
    };
    match string(data, "status")? {
        "canceled" => {
            let at = time(&data["canceled_at"])?;
            check(at <= observed_at, "paddle_future_cancellation")?;
            result.effective_at = Some(at);
        }
        "active" | "past_due" | "trialing" => {
            let change = data
                .get("scheduled_change")
                .ok_or(EvidenceError("paddle_missing_scheduled_change"))?;
            if !change.is_null() {
                check(
                    change["action"] == "cancel",
                    "paddle_unsupported_scheduled_change",
                )?;
                result.scheduled_at = Some(time(&change["effective_at"])?);
            }
        }
        _ => return Err(EvidenceError("paddle_unsupported_subscription_status")),
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use suprnova::serde_json::json;
    fn purchase() -> purchase::Model {
        purchase::Model {
            id: "purchase".into(),
            listing_id: 1,
            owner_id: 2,
            approved_revision_id: 3,
            plan_key: "plan".into(),
            plan_name: "Plan".into(),
            billing_type: "one_time".into(),
            amount: 1000,
            currency: "USD".into(),
            provider: "paddle".into(),
            mode: "test".into(),
            price_id: Some("pri_1".into()),
            public_key: None,
            credentials: None,
            profile_revision: None,
            customer_ref: Some("ctm_1".into()),
            session_ref: Some("txn_1".into()),
            subscription_ref: None,
            checkout_payload: None,
            return_origin: "https://directory.test".into(),
            state: "pending".into(),
            error_code: None,
            version: 0,
            lease_until: None,
            cancel_requested: false,
            cancel_at: None,
            cancel_event_at: 0,
            created_at: 0,
            updated_at: 0,
        }
    }
    fn transaction() -> Value {
        json!({"id":"txn_1", "customer_id":"ctm_1", "custom_data":{"purchase_id":"purchase"}, "subscription_id":null, "status":"completed", "currency_code":"USD", "items":[{"quantity":1,"price":{"id":"pri_1","unit_price":{"amount":"1000","currency_code":"USD"},"billing_cycle":null,"tax_mode":"external"}}], "details":{"line_items":[{"id":"txnitm_1","price_id":"pri_1","quantity":1,"totals":{"subtotal":"1000","discount":"0","tax":"100","total":"1100"}}],"totals":{"subtotal":"1000","discount":"0","tax":"100","total":"1100","credit":"0","grand_total":"1100","balance":"0","credit_to_balance":"0","currency_code":"USD"},"adjusted_totals":{"grand_total":"1100","currency_code":"USD"}}, "payments":[{"payment_attempt_id":"pay_1","status":"captured","amount":"1100","captured_at":"2026-01-02T00:00:00Z"}]})
    }
    const NOW: i64 = 1_800_000_000;
    fn payment(p: &purchase::Model, t: &Value) -> payment::Model {
        let s = settlement(p, t, NOW).unwrap();
        payment::Model {
            id: "payment".into(),
            purchase_id: p.id.clone(),
            provider_payment_id: s.payment_ref,
            amount_total: s.amount_total,
            period_start: s.period_start,
            period_end: s.period_end,
            status: "paid".into(),
            paid_at: s.paid_at,
            refund_event_at: 0,
            dispute_event_at: 0,
            adverse: "{}".into(),
            created_at: 0,
        }
    }
    fn adjustment(id: &str, action: &str, status: &str, amount: &str) -> Value {
        json!({"id":id,"transaction_id":"txn_1","customer_id":"ctm_1","subscription_id":null,"currency_code":"USD","action":action,"status":status,"type":if amount=="1100" {"full"} else {"partial"},"totals":{"total":amount},"items":[{"item_id":"txnitm_1"}]})
    }
    #[test]
    fn completed_capture_and_inclusive_tax() {
        let p = purchase();
        let mut t = transaction();
        assert_eq!(settlement(&p, &t, NOW).unwrap().amount_total, 1100);
        t["items"][0]["price"]["tax_mode"] = json!("internal");
        for totals in ["/details/totals", "/details/line_items/0/totals"] {
            let v = t.pointer_mut(totals).unwrap();
            v["subtotal"] = json!("909");
            v["tax"] = json!("91");
            v["total"] = json!("1000");
        }
        t["details"]["totals"]["grand_total"] = json!("1000");
        t["payments"][0]["amount"] = json!("1000");
        assert_eq!(settlement(&p, &t, NOW).unwrap().amount_total, 1000);
    }
    #[test]
    fn rejects_foreign_or_incomplete_capture() {
        let p = purchase();
        for (path, value) in [
            ("/customer_id", json!("other")),
            ("/items/0/price/id", json!("other")),
            ("/currency_code", json!("EUR")),
            ("/custom_data/mode", json!("live")),
            ("/status", json!("billed")),
            ("/status", json!("paid")),
            ("/payments", json!([])),
            ("/details/totals/credit", json!("1100")),
            ("/details/totals/grand_total", json!("0")),
        ] {
            let mut t = transaction();
            if path == "/custom_data/mode" {
                t["custom_data"]["mode"] = value;
            } else {
                *t.pointer_mut(path).unwrap() = value;
            }
            assert!(settlement(&p, &t, NOW).is_err(), "{path}");
        }
    }
    #[test]
    fn recurring_requires_transaction_period_and_bound_subscription() {
        let mut p = purchase();
        p.billing_type = "monthly".into();
        let mut t = transaction();
        t["subscription_id"] = json!("sub_1");
        t["items"][0]["price"]["billing_cycle"] = json!({"interval":"month","frequency":1});
        assert!(settlement(&p, &t, NOW).is_err());
        t["billing_period"] =
            json!({"starts_at":"2026-01-01T00:00:00Z","ends_at":"2026-02-01T00:00:00Z"});
        assert!(settlement(&p, &t, NOW).unwrap().period_end.is_some());
        t["id"] = json!("txn_renewal");
        assert!(settlement(&p, &t, NOW).is_err());
        p.subscription_ref = Some("sub_1".into());
        assert!(settlement(&p, &t, NOW).unwrap().session_ref.is_none());
    }
    #[test]
    fn refunds_accumulate_and_duplicate_ids_do_not_double_count() {
        let p = purchase();
        let mut t = transaction();
        let pay = payment(&p, &t);
        let partial = adjustment("a", "refund", "approved", "400");
        t["details"]["adjusted_totals"]["grand_total"] = json!("700");
        assert_eq!(
            adverse(&p, &pay, &t, &json!([partial, partial]), NOW)
                .unwrap()
                .refund_total,
            400
        );
        assert_eq!(
            adverse(
                &p,
                &pay,
                &t,
                &json!([partial, adjustment("b", "refund", "approved", "700")]),
                NOW
            )
            .unwrap()
            .refund_total,
            1100
        );
        assert_eq!(
            adverse(
                &p,
                &pay,
                &t,
                &json!([adjustment("full", "refund", "approved", "1100")]),
                NOW
            )
            .unwrap()
            .refund_total,
            1100
        );
        assert!(
            adverse(
                &p,
                &pay,
                &t,
                &json!([partial, adjustment("a", "refund", "approved", "700")]),
                NOW
            )
            .is_err()
        );
    }
    #[test]
    fn reversal_never_clears_an_unrelated_dispute() {
        let p = purchase();
        let t = transaction();
        let pay = payment(&p, &t);
        let reversed = adjustment("old", "chargeback_warning", "reversed", "1100");
        let reversal = adjustment("rev", "chargeback_warning_reverse", "approved", "1100");
        let active = adjustment("new", "chargeback", "approved", "1100");
        let state = adverse(&p, &pay, &t, &json!([reversed, reversal, active]), NOW).unwrap();
        assert!(state.disputed && state.lost_dispute);
        assert!(
            !adverse(&p, &pay, &t, &json!([reversed, reversal]), NOW)
                .unwrap()
                .disputed
        );
        assert!(adverse(&p, &pay, &t, &json!([reversal]), NOW).is_err());
        let mut missing = t.clone();
        missing["details"]["adjusted_totals"] = Value::Null;
        assert!(adverse(&p, &pay, &missing, &json!([reversed, reversal]), NOW).is_err());
    }
    #[test]
    fn cancellation_only_supplies_end_boundaries() {
        let mut p = purchase();
        p.subscription_ref = Some("sub_1".into());
        let mut s = json!({"id":"sub_1","customer_id":"ctm_1","custom_data":{"purchase_id":"purchase"},"status":"active","updated_at":"2026-01-03T00:00:00Z","scheduled_change":{"action":"cancel","effective_at":"2026-02-01T00:00:00Z"}});
        let c = cancellation(&p, &s, NOW).unwrap();
        assert!(c.effective_at.is_none() && c.scheduled_at.is_some());
        s["status"] = json!("canceled");
        assert!(cancellation(&p, &s, NOW).is_err());
        s["canceled_at"] = json!("2026-01-04T00:00:00Z");
        assert!(cancellation(&p, &s, NOW).unwrap().effective_at.is_some());
    }
    #[test]
    fn reversal_matches_paid_item_not_unique_adjustment_item_id() {
        let p = purchase();
        let t = transaction();
        let pay = payment(&p, &t);
        let mut original = adjustment("old", "chargeback", "reversed", "1100");
        let mut reverse = adjustment("reverse", "chargeback_reverse", "approved", "1100");
        original["items"][0]["id"] = json!("adjitm_original");
        reverse["items"][0]["id"] = json!("adjitm_reverse");
        assert!(
            !adverse(&p, &pay, &t, &json!([original, reverse]), NOW)
                .unwrap()
                .disputed
        );
        reverse["items"][0]["item_id"] = json!("txnitm_other");
        assert!(adverse(&p, &pay, &t, &json!([original, reverse]), NOW).is_err());
    }

    #[test]
    fn sdk_adjusted_totals_round_trip_retains_required_evidence() {
        use paddle_rust_sdk::entities::TransactionTotalsAdjusted;
        let raw = json!({"subtotal":"1000", "tax":"100", "total":"1100", "grand_total":"1100", "fee":null, "earnings":null, "currency_code":"USD"});
        let typed: TransactionTotalsAdjusted =
            suprnova::serde_json::from_value(raw.clone()).unwrap();
        let serialized = suprnova::serde_json::to_value(typed).unwrap();
        assert_eq!(serialized, raw);
        assert!(
            suprnova::serde_json::from_value::<TransactionTotalsAdjusted>(Value::Null).is_err()
        );
        let mut missing = raw;
        missing.as_object_mut().unwrap().remove("grand_total");
        assert!(suprnova::serde_json::from_value::<TransactionTotalsAdjusted>(missing).is_err());
    }
}
