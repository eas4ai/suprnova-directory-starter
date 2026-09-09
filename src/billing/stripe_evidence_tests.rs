//! Contract examples for authenticated Stripe evidence, independent of transport and persistence.
use super::{
    lifecycle_entities::{payment, purchase},
    stripe_evidence as evidence,
};
use suprnova::serde_json::{Value, json};
const NOW: i64 = 1_800_000_000;
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
        provider: "stripe".into(),
        mode: "test".into(),
        price_id: Some("price_1".into()),
        public_key: None,
        credentials: None,
        profile_revision: None,
        customer_ref: Some("cus_1".into()),
        session_ref: Some("cs_1".into()),
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
        created_at: NOW - 100,
        updated_at: NOW - 100,
    }
}
fn price() -> Value {
    json!({"id":"price_1","currency":"usd","unit_amount":1000,"type":"one_time","recurring":null})
}
fn session() -> Value {
    json!({"id":"cs_1","livemode":false,"customer":"cus_1","metadata":{"purchase_id":"purchase"},"mode":"payment","status":"complete","payment_status":"paid","currency":"usd","amount_subtotal":1000,"amount_total":1100,"payment_intent":"pi_1","line_items":{"has_more":false,"data":[{"quantity":1,"price":price(),"currency":"usd","amount_subtotal":1000,"amount_total":1100}]}})
}
fn recurring() -> (purchase::Model, Value, Value, Value) {
    let mut p = purchase();
    p.billing_type = "monthly".into();
    p.subscription_ref = Some("sub_1".into());
    let mut catalog = price();
    catalog["type"] = json!("recurring");
    catalog["recurring"] = json!({"interval":"month","interval_count":1,"usage_type":"licensed"});
    let invoice = json!({"id":"in_1","livemode":false,"customer":"cus_1","subscription":"sub_1","subscription_details":{"metadata":{"purchase_id":"purchase"}},"status":"paid","paid_out_of_band":false,"amount_remaining":0,"amount_paid":1100,"amount_due":1100,"currency":"usd","payment_intent":"pi_1","lines":{"has_more":false,"data":[{"id":"il_1","price":"price_1","quantity":1,"proration":false,"amount":1000,"currency":"usd","period":{"start":NOW-100,"end":NOW+1000}}]},"status_transitions":{"paid_at":NOW-10}});
    let intent = json!({"id":"pi_1","livemode":false,"customer":"cus_1","status":"succeeded","amount_received":1100,"currency":"usd","latest_charge":"ch_1"});
    (p, invoice, catalog, intent)
}
fn payment() -> payment::Model {
    payment::Model {
        id: "payment".into(),
        purchase_id: "purchase".into(),
        provider_payment_id: "pi_1".into(),
        amount_total: 1100,
        period_start: NOW - 100,
        period_end: None,
        status: "paid".into(),
        paid_at: NOW - 10,
        refund_event_at: 0,
        dispute_event_at: 0,
        adverse: "{}".into(),
        created_at: NOW - 10,
    }
}
fn charge() -> Value {
    json!({"id":"ch_1","livemode":false,"customer":"cus_1","currency":"usd","payment_intent":"pi_1","paid":true,"status":"succeeded","amount":1100,"amount_refunded":0,"disputed":false})
}
fn subscription() -> Value {
    json!({"id":"sub_1","livemode":false,"customer":"cus_1","metadata":{"purchase_id":"purchase"},"status":"active","cancel_at_period_end":false,"cancel_at":null,"current_period_end":NOW+1000})
}
#[test]
fn one_time_paid_checkout_preserves_actual_total_and_never_expires() {
    let s = evidence::one_time(&purchase(), &session(), NOW).unwrap();
    assert_eq!(s.payment_ref, "pi_1");
    assert_eq!(s.amount_total, 1100);
    assert_eq!(s.period_end, None);
    assert_eq!(s.session_ref.as_deref(), Some("cs_1"));
}
#[test]
fn one_time_rejects_foreign_terms_and_unsettled_checkout() {
    for (path, value) in [
        ("/customer", json!("cus_other")),
        ("/livemode", json!(true)),
        ("/currency", json!("eur")),
        ("/metadata/purchase_id", json!("other")),
        ("/id", json!("cs_other")),
        ("/line_items/data/0/price/id", json!("price_other")),
        ("/line_items/data/0/price/unit_amount", json!(900)),
        ("/line_items/data/0/quantity", json!(2)),
        ("/line_items/has_more", json!(true)),
        ("/status", json!("open")),
        ("/payment_status", json!("unpaid")),
        ("/payment_status", json!("no_payment_required")),
        ("/amount_total", json!(0)),
        ("/payment_intent", Value::Null),
    ] {
        let mut s = session();
        *s.pointer_mut(path).unwrap() = value;
        assert!(evidence::one_time(&purchase(), &s, NOW).is_err(), "{path}");
    }
}
#[test]
fn recurring_paid_invoice_uses_its_own_period_and_capture() {
    let (p, i, price, intent) = recurring();
    let s = evidence::recurring(&p, &i, &price, &intent, NOW).unwrap();
    assert_eq!(s.period_start, NOW - 100);
    assert_eq!(s.period_end, Some(NOW + 1000));
    assert_eq!(s.paid_at, NOW - 10);
    assert_eq!(s.payment_ref, "pi_1");
    // Replaying a paid invoice emits the same immutable interval and key. Persistence deduplicates it.
    let replay = evidence::recurring(&p, &i, &price, &intent, NOW + 100).unwrap();
    assert_eq!(
        (s.payment_ref, s.period_start, s.period_end),
        (replay.payment_ref, replay.period_start, replay.period_end)
    );
}
#[test]
fn recurring_rejects_duplicate_period_lines_and_incomplete_or_unpaid_evidence() {
    let (p, invoice, price, intent) = recurring();
    for (path, value) in [
        ("/customer", json!("cus_other")),
        ("/livemode", json!(true)),
        ("/currency", json!("eur")),
        ("/subscription", json!("sub_other")),
        ("/status", json!("open")),
        ("/paid_out_of_band", json!(true)),
        ("/amount_remaining", json!(1)),
        ("/amount_paid", json!(0)),
        ("/lines/has_more", json!(true)),
        ("/lines/data/0/price", json!("price_other")),
        ("/lines/data/0/proration", json!(true)),
        ("/lines/data/0/period/end", json!(NOW - 100)),
        ("/status_transitions/paid_at", Value::Null),
    ] {
        let mut i = invoice.clone();
        *i.pointer_mut(path).unwrap() = value;
        assert!(
            evidence::recurring(&p, &i, &price, &intent, NOW).is_err(),
            "{path}"
        );
    }
    let mut duplicate = invoice.clone();
    let line = duplicate["lines"]["data"][0].clone();
    duplicate["lines"]["data"]
        .as_array_mut()
        .unwrap()
        .push(line);
    assert!(evidence::recurring(&p, &duplicate, &price, &intent, NOW).is_err());
    for (path, value) in [
        ("/status", json!("processing")),
        ("/amount_received", json!(0)),
        ("/customer", json!("cus_other")),
        ("/livemode", json!(true)),
        ("/currency", json!("eur")),
        ("/id", json!("pi_other")),
    ] {
        let mut pi = intent.clone();
        *pi.pointer_mut(path).unwrap() = value;
        assert!(
            evidence::recurring(&p, &invoice, &price, &pi, NOW).is_err(),
            "intent {path}"
        );
    }
}
#[test]
fn recurring_supports_current_invoice_parent_and_payment_shapes() {
    let (p, mut i, price, intent) = recurring();
    i.as_object_mut().unwrap().remove("subscription");
    i.as_object_mut().unwrap().remove("subscription_details");
    i.as_object_mut().unwrap().remove("payment_intent");
    i["parent"] = json!({"subscription_details":{"subscription":"sub_1","metadata":{"purchase_id":"purchase"}}});
    i["payments"] = json!({"has_more":false,"data":[{"status":"paid","payment":{"type":"payment_intent","payment_intent":"pi_1"}}]});
    i["lines"]["data"][0]
        .as_object_mut()
        .unwrap()
        .remove("price");
    i["lines"]["data"][0]["pricing"] = json!({"price_details":{"price":"price_1"}});
    assert!(evidence::recurring(&p, &i, &price, &intent, NOW).is_ok());
}
#[test]
fn refunds_report_cumulative_amount_and_validate_original_payment() {
    let p = purchase();
    let pay = payment();
    for amount in [0, 400, 1100] {
        let mut c = charge();
        c["amount_refunded"] = json!(amount);
        assert_eq!(
            evidence::refund(&p, &pay, &c, NOW).unwrap().refund_total,
            amount
        );
    }
    for (path, value) in [
        ("/payment_intent", json!("pi_other")),
        ("/customer", json!("cus_other")),
        ("/livemode", json!(true)),
        ("/currency", json!("eur")),
        ("/amount", json!(1000)),
        ("/amount_refunded", json!(1101)),
        ("/amount_refunded", json!(-1)),
        ("/paid", json!(false)),
        ("/status", json!("failed")),
    ] {
        let mut c = charge();
        *c.pointer_mut(path).unwrap() = value;
        assert!(evidence::refund(&p, &pay, &c, NOW).is_err(), "{path}");
    }
}
#[test]
fn dispute_states_require_same_charge_and_report_current_resolution() {
    let p = purchase();
    let pay = payment();
    let mut c = charge();
    c["amount_refunded"] = json!(400);
    for status in [
        "needs_response",
        "under_review",
        "warning_needs_response",
        "warning_under_review",
        "lost",
        "won",
        "warning_closed",
    ] {
        let d =
            json!({"id":"dp_1","charge":"ch_1","livemode":false,"currency":"usd","status":status});
        let s = evidence::dispute(&p, &pay, &c, &d, NOW).unwrap();
        assert_eq!(s.refund_total, 400);
        assert_eq!(s.lost_dispute, status == "lost");
        assert_eq!(
            s.disputed,
            matches!(
                status,
                "needs_response"
                    | "under_review"
                    | "warning_needs_response"
                    | "warning_under_review"
            )
        );
    }
    let won = json!({"id":"dp_1","charge":"ch_1","livemode":false,"currency":"usd","status":"won"});
    c["disputed"] = json!(true);
    // This flag records dispute history; the current won status resolves this dispute.
    let resolved = evidence::dispute(&p, &pay, &c, &won, NOW).unwrap();
    assert!(!resolved.disputed && !resolved.lost_dispute);
    c["disputed"] = json!(false);
    for (path, value) in [
        ("/charge", json!("ch_other")),
        ("/livemode", json!(true)),
        ("/currency", json!("eur")),
        ("/status", json!("new_unrecognized_state")),
    ] {
        let mut d = won.clone();
        *d.pointer_mut(path).unwrap() = value;
        assert!(evidence::dispute(&p, &pay, &c, &d, NOW).is_err(), "{path}");
    }
}
#[test]
fn scheduled_cancellation_preserves_access_until_effective_end() {
    let mut p = purchase();
    p.billing_type = "monthly".into();
    p.subscription_ref = Some("sub_1".into());
    let mut s = subscription();
    s["cancel_at_period_end"] = json!(true);
    s["canceled_at"] = json!(NOW - 500);
    let scheduled = evidence::cancellation(&p, &s, NOW).unwrap();
    assert_eq!(scheduled.scheduled_at, Some(NOW + 1000));
    assert_eq!(scheduled.effective_at, None);
    s["status"] = json!("canceled");
    assert!(evidence::cancellation(&p, &s, NOW).is_err());
    s["ended_at"] = json!(NOW - 10);
    let canceled = evidence::cancellation(&p, &s, NOW).unwrap();
    assert_eq!(canceled.effective_at, Some(NOW - 10));
    assert_eq!(canceled.scheduled_at, None);
    for (path, value) in [
        ("/customer", json!("cus_other")),
        ("/livemode", json!(true)),
        ("/id", json!("sub_other")),
        ("/metadata/purchase_id", json!("other")),
        ("/ended_at", json!(NOW + 1000)),
    ] {
        let mut bad = s.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(evidence::cancellation(&p, &bad, NOW).is_err(), "{path}");
    }
}

#[test]
fn foreign_local_payment_row_cannot_supply_refund_or_dispute_evidence() {
    let p = purchase();
    let mut foreign = payment();
    foreign.purchase_id = "another_purchase".into();
    assert!(evidence::refund(&p, &foreign, &charge(), NOW).is_err());
    let dispute =
        json!({"id":"dp_1", "charge":"ch_1", "livemode":false, "currency":"usd", "status":"lost"});
    assert!(evidence::dispute(&p, &foreign, &charge(), &dispute, NOW).is_err());
}

#[test]
fn recurring_requires_explicit_non_prorated_line_evidence() {
    let (p, mut invoice, price, intent) = recurring();
    invoice["lines"]["data"][0]
        .as_object_mut()
        .unwrap()
        .remove("proration");
    assert!(evidence::recurring(&p, &invoice, &price, &intent, NOW).is_err());
    invoice["lines"]["data"][0]["parent"] =
        json!({"subscription_item_details":{"proration":false}});
    assert!(evidence::recurring(&p, &invoice, &price, &intent, NOW).is_ok());
}
