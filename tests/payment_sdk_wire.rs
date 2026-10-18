#![recursion_limit = "256"]
//! Real SDK request/response contracts against a loopback-only HTTP server.
use directory::billing::gateway::{GatewayError, Resource, read_paddle, read_stripe};
use std::sync::{Arc, Mutex};
use suprnova::serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

const STRIPE_KEY: &str = "sk_test_fixture_marker";
const PADDLE_KEY: &str = "pdl_fixture_marker";
struct FixtureServer {
    base: String,
    requests: Arc<Mutex<Vec<String>>>,
    task: JoinHandle<()>,
}
impl FixtureServer {
    async fn new(responses: Vec<Value>) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let task = tokio::spawn(async move {
            for response in responses {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let mut chunk = [0; 1024];
                while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                    let read = stream.read(&mut chunk).await.unwrap();
                    assert!(read > 0);
                    bytes.extend_from_slice(&chunk[..read]);
                    assert!(bytes.len() < 32768);
                }
                captured
                    .lock()
                    .unwrap()
                    .push(String::from_utf8(bytes).unwrap());
                let body = response.to_string();
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nRequest-Id: req_fixture\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).await.unwrap();
                stream.write_all(body.as_bytes()).await.unwrap();
                stream.shutdown().await.unwrap();
            }
        });
        Self {
            base,
            requests,
            task,
        }
    }
    fn stripe(&self) -> stripe::Client {
        stripe::ClientBuilder::new(STRIPE_KEY)
            .url(&self.base)
            .build()
            .unwrap()
    }
    fn paddle(&self) -> paddle_rust_sdk::Paddle {
        paddle_rust_sdk::Paddle::new(PADDLE_KEY, &self.base).unwrap()
    }
    fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}
impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn request(request: &str, path: &str, marker: &str) -> Vec<(String, String)> {
    let mut first = request.lines().next().unwrap().split_whitespace();
    assert_eq!(first.next(), Some("GET"));
    let target = first.next().unwrap();
    let (actual, query) = target.split_once('?').unwrap_or((target, ""));
    assert_eq!(actual, path);
    assert!(request.lines().any(|line| line.to_ascii_lowercase()
        == format!("authorization: bearer {marker}").to_ascii_lowercase()));
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (k, v) = part.split_once('=').unwrap();
            (decode(k), decode(v))
        })
        .collect()
}
fn decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            result.push(u8::from_str_radix(&value[i + 1..i + 3], 16).unwrap());
            i += 3;
        } else {
            result.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
            i += 1;
        }
    }
    String::from_utf8(result).unwrap()
}
fn list(data: Vec<Value>, more: bool, url: &str) -> Value {
    json!({"object":"list","data":data,"has_more":more,"url":url})
}
fn stripe_price() -> Value {
    json!({"id":"price_fixture","object":"price","active":true,"billing_scheme":"per_unit","created":1700000000,"currency":"usd","livemode":false,"metadata":{},"product":"prod_fixture","type":"one_time","unit_amount":1000,"recurring":null})
}
fn checkout() -> Value {
    json!({"id":"cs_fixture","object":"checkout.session","automatic_tax":{"enabled":false},"created":1700000000,"custom_fields":[],"custom_text":{},"expires_at":1700003600,"livemode":false,"mode":"payment","payment_method_types":["card"],"payment_status":"paid","shipping_options":[],"customer":"cus_fixture","metadata":{"purchase_id":"purchase_fixture"},"currency":"usd","amount_subtotal":1000,"amount_total":1100,"status":"complete","payment_intent":"pi_fixture","line_items":list(vec![json!({"id":"li_fixture","object":"item","amount_discount":0,"amount_subtotal":1000,"amount_tax":100,"amount_total":1100,"currency":"usd","quantity":1,"price":stripe_price()})],false,"/v1/checkout/sessions/cs_fixture/line_items")})
}
fn invoice() -> Value {
    json!({"id":"in_fixture","object":"invoice","amount_due":1100,"amount_overpaid":0,"amount_paid":1100,"amount_remaining":0,"amount_shipping":0,"attempt_count":1,"attempted":true,"automatic_tax":{"enabled":false},"collection_method":"charge_automatically","created":1700000000,"currency":"usd","default_tax_rates":[],"discounts":[],"issuer":{"type":"self"},"lines":list(vec![json!({"id":"il_fixture","object":"line_item","amount":1000,"currency":"usd","discountable":true,"discounts":[],"livemode":false,"metadata":{},"period":{"start":1700000000,"end":1702592000},"subtotal":1000,"quantity":1,"pricing":{"type":"price_details","price_details":{"price":"price_fixture","product":"prod_fixture"}},"parent":{"type":"subscription_item_details","subscription_item_details":{"subscription":"sub_fixture","subscription_item":"si_fixture","proration":false}}})],false,"/v1/invoices/in_fixture/lines"),"livemode":false,"customer":"cus_fixture","metadata":{},"payment_settings":{},"period_end":1702592000,"period_start":1700000000,"post_payment_credit_notes_amount":0,"pre_payment_credit_notes_amount":0,"starting_balance":0,"status":"paid","status_transitions":{"paid_at":1700000010},"subtotal":1000,"total":1100,"parent":{"type":"subscription_details","subscription_details":{"subscription":"sub_fixture","metadata":{"purchase_id":"purchase_fixture"}}},"payments":list(vec![json!({"id":"inpay_fixture","object":"invoice_payment","amount_paid":1100,"amount_requested":1100,"created":1700000010,"currency":"usd","invoice":"in_fixture","is_default":true,"livemode":false,"payment":{"type":"payment_intent","payment_intent":"pi_fixture"},"status":"paid","status_transitions":{"paid_at":1700000010}})],false,"/v1/invoice_payments")})
}
fn dispute(id: &str, status: &str) -> Value {
    json!({"id":id,"object":"dispute","amount":1100,"balance_transactions":[],"charge":"ch_fixture","created":1700000000,"currency":"usd","enhanced_eligibility_types":[],"evidence":{"enhanced_evidence":{}},"evidence_details":{"enhanced_eligibility":{},"has_evidence":false,"past_due":false,"submission_count":0},"is_charge_refundable":true,"livemode":false,"metadata":{},"reason":"fraudulent","status":status,"payment_intent":"pi_fixture"})
}
fn paddle_page(data: Vec<Value>, more: bool, next: &str) -> Value {
    json!({"data":data,"meta":{"request_id":"request_fixture","pagination":{"per_page":50,"next":next,"has_more":more,"estimated_total":2}}})
}
fn paddle_transaction() -> Value {
    suprnova::serde_json::from_str(include_str!("fixtures/payment-sdk/paddle-transaction.json"))
        .unwrap()
}

#[tokio::test]
async fn stripe_checkout_expands_and_preserves_paid_line_evidence() {
    let raw = checkout();
    let server = FixtureServer::new(vec![raw]).await;
    let got = read_stripe(&server.stripe(), Resource::Checkout("cs_fixture"))
        .await
        .unwrap();
    assert_eq!(got["line_items"]["data"][0]["price"]["unit_amount"], 1000);
    assert_eq!(got["line_items"]["has_more"], false);
    assert_eq!(got["customer"], "cus_fixture");
    assert_eq!(got["metadata"]["purchase_id"], "purchase_fixture");
    assert_eq!(got["payment_intent"], "pi_fixture");
    assert_eq!(got["payment_status"], "paid");
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let query = request(&requests[0], "/v1/checkout/sessions/cs_fixture", STRIPE_KEY);
    assert!(
        query
            .iter()
            .any(|(k, v)| k.starts_with("expand[") && v == "line_items")
    );
}
#[tokio::test]
async fn stripe_invoice_expands_payments_and_keeps_modern_subscription_and_period_fields() {
    let raw = invoice();
    let server = FixtureServer::new(vec![raw]).await;
    let got = read_stripe(&server.stripe(), Resource::Invoice("in_fixture"))
        .await
        .unwrap();
    assert_eq!(
        got["payments"]["data"][0]["payment"]["payment_intent"],
        "pi_fixture"
    );
    assert_eq!(got["payments"]["has_more"], false);
    assert_eq!(
        got["parent"]["subscription_details"]["metadata"]["purchase_id"],
        "purchase_fixture"
    );
    assert_eq!(got["lines"]["data"][0]["period"]["end"], 1702592000);
    assert_eq!(
        got["lines"]["data"][0]["parent"]["subscription_item_details"]["proration"],
        false
    );
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let query = request(&requests[0], "/v1/invoices/in_fixture", STRIPE_KEY);
    assert!(
        query
            .iter()
            .any(|(k, v)| k.starts_with("expand[") && v == "payments")
    );
}
#[tokio::test]
async fn stripe_disputes_fetch_every_page_with_charge_filter_and_cursor() {
    let first = dispute("dp_first", "won");
    let server = FixtureServer::new(vec![
        list(vec![first], true, "/v1/disputes"),
        list(
            vec![dispute("dp_second", "under_review")],
            false,
            "/v1/disputes",
        ),
    ])
    .await;
    let got = read_stripe(&server.stripe(), Resource::Disputes("ch_fixture"))
        .await
        .unwrap();
    assert_eq!(got.as_array().unwrap().len(), 2);
    assert_eq!(got[0]["status"], "won");
    assert_eq!(got[1]["status"], "under_review");
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    for (index, r) in requests.iter().enumerate() {
        let query = request(r, "/v1/disputes", STRIPE_KEY);
        assert!(query.contains(&("charge".into(), "ch_fixture".into())));
        assert!(query.contains(&("limit".into(), "100".into())));
        if index == 1 {
            assert!(query.contains(&("starting_after".into(), "dp_first".into())));
        }
    }
}
#[tokio::test]
async fn paddle_transaction_keeps_capture_and_adjusted_totals_through_typed_sdk() {
    let raw = paddle_transaction();
    let _: paddle_rust_sdk::entities::Transaction = suprnova::serde_json::from_value(raw.clone())
        .expect("Transaction fixture must satisfy pinned SDK schema");
    let server = FixtureServer::new(vec![
        json!({"data":raw,"meta":{"request_id":"request_fixture"}}),
    ])
    .await;
    let got = read_paddle(
        &server.paddle(),
        Resource::Checkout("txn_01h04vsc0qhwtsbsxh3422wjs4"),
    )
    .await
    .unwrap();
    assert_eq!(got["details"]["adjusted_totals"]["grand_total"], "1100");
    assert_eq!(got["details"]["adjusted_totals"]["currency_code"], "USD");
    assert_eq!(got["payments"][0]["status"], "captured");
    assert!(
        got["payments"][0]["captured_at"]
            .as_str()
            .unwrap()
            .starts_with("2026-01-02T00:00:00")
    );
    assert_eq!(got["custom_data"]["purchase_id"], "purchase_fixture");
    assert!(got["items"][0]["price"]["billing_cycle"].is_null());
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    request(
        &requests[0],
        "/transactions/txn_01h04vsc0qhwtsbsxh3422wjs4",
        PADDLE_KEY,
    );
}
#[tokio::test]
async fn paddle_complete_adjustments_preserve_new_reversal_enum_and_item_ids() {
    let first = json!({"id":"adj_first","transaction_id":"txn_fixture","action":"chargeback_warning","status":"reversed","items":[{"id":"adjitm_first","item_id":"txnitm_paid"}]});
    let second = json!({"id":"adj_second","transaction_id":"txn_fixture","action":"chargeback_warning_reverse","status":"approved","items":[{"id":"adjitm_second","item_id":"txnitm_paid"}]});
    // Pagination follows the returned path/query but retains the configured local origin.
    let server=FixtureServer::new(vec![paddle_page(vec![first],true,"https://api.paddle.com/adjustments?transaction_id=txn_fixture&per_page=50&after=adj_first"),paddle_page(vec![second],false,"https://api.paddle.com/adjustments?after=adj_second")]).await;
    let got = read_paddle(&server.paddle(), Resource::Adjustments("txn_fixture"))
        .await
        .unwrap();
    assert_eq!(got.as_array().unwrap().len(), 2);
    assert_eq!(got[1]["action"], "chargeback_warning_reverse");
    assert_eq!(got[1]["items"][0]["id"], "adjitm_second");
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    for (index, r) in requests.iter().enumerate() {
        let query = request(r, "/adjustments", PADDLE_KEY);
        assert!(query.contains(&("transaction_id".into(), "txn_fixture".into())));
        assert!(query.contains(&("per_page".into(), "50".into())));
        if index == 1 {
            assert!(query.contains(&("after".into(), "adj_first".into())));
        }
    }
}
#[tokio::test]
async fn unsupported_and_invalid_references_never_reach_http() {
    let server = FixtureServer::new(vec![]).await;
    let stripe = server.stripe();
    let paddle = server.paddle();
    for resource in [
        Resource::Checkout("cs_"),
        Resource::Checkout("cs_bad/path"),
        Resource::Invoice("in_bad?expand=x"),
        Resource::Disputes("ch_bad%2fpath"),
        Resource::PaymentIntent("pi_bad\nheader"),
    ] {
        assert_eq!(
            read_stripe(&stripe, resource).await,
            Err(GatewayError("provider_reference_invalid"))
        );
    }
    for resource in [
        Resource::Checkout("txn_"),
        Resource::Adjustments("txn_bad/path"),
        Resource::LatestTransaction("sub_bad?x"),
    ] {
        assert_eq!(
            read_paddle(&paddle, resource).await,
            Err(GatewayError("provider_reference_invalid"))
        );
    }
    assert_eq!(
        read_stripe(&stripe, Resource::Adjustments("txn_fixture")).await,
        Err(GatewayError("unsupported_evidence_read"))
    );
    assert_eq!(
        read_paddle(&paddle, Resource::Invoice("in_fixture")).await,
        Err(GatewayError("unsupported_evidence_read"))
    );
    assert!(server.requests().is_empty());
}
