//! Synthetic account state behind the same application gateway used by HTTP.
use async_trait::async_trait;
use directory::billing::{
    gateway::{Gateway, GatewayError, Resource},
    lifecycle_entities::purchase,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use suprnova::{
    payments::{
        CheckoutSessionState, CreateCustomerRequest, SessionPayload, StartSessionRequest,
        SubscriptionResult, SubscriptionStatus,
    },
    serde_json::{self, Value, json},
};

pub const STRIPE_SIGNING: &str = "whsec_PAYMENT_SIGNATURE_FIXTURE";
pub const PADDLE_SIGNING: &str = "pdl_ntfset_PAYMENT_SIGNATURE_FIXTURE";

#[derive(Default)]
pub struct FakeGateway {
    pub starts: Mutex<Vec<(String, Value)>>,
    pub customers: AtomicUsize,
    pub cancels: AtomicUsize,
    pub reads: AtomicUsize,
    pub fail_start: AtomicUsize,
    pub fail_reads: AtomicUsize,
    pub fail_cancel: AtomicUsize,
    pub observed_profiles: Mutex<Vec<(String, String, Option<String>)>>,
    resources: Mutex<BTreeMap<String, Value>>,
    gate: Mutex<Option<(String, Arc<ReadGate>)>>,
}
#[derive(Default)]
pub struct ReadGate {
    pub arrived: tokio::sync::Notify,
    pub release: tokio::sync::Notify,
}

pub fn suffix(p: &purchase::Model) -> String {
    p.id.replace('-', "")
}
pub fn reference(p: &purchase::Model, kind: &str, n: usize) -> String {
    format!("{kind}_{}_{}", suffix(p), n)
}
fn key(provider: &str, kind: &str, id: &str) -> String {
    format!("{provider}:{kind}:{id}")
}
fn fail(counter: &AtomicUsize) -> bool {
    counter
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
        .is_ok()
}

impl FakeGateway {
    pub fn set(&self, provider: &str, kind: &str, id: &str, value: Value) {
        self.resources
            .lock()
            .unwrap()
            .insert(key(provider, kind, id), value);
    }
    pub fn get(&self, provider: &str, kind: &str, id: &str) -> Value {
        self.resources
            .lock()
            .unwrap()
            .get(&key(provider, kind, id))
            .unwrap_or_else(|| panic!("missing {provider}/{kind}/{id}"))
            .clone()
    }
    pub fn gate_read(&self, provider: &str, kind: &str, id: &str) -> Arc<ReadGate> {
        let gate = Arc::new(ReadGate::default());
        *self.gate.lock().unwrap() = Some((key(provider, kind, id), gate.clone()));
        gate
    }
    pub fn start_count(&self) -> usize {
        self.starts.lock().unwrap().len()
    }

    /// Actual paid facts are separate from creating an unpaid checkout.
    pub fn settle(&self, p: &purchase::Model, n: usize, start: i64, end: Option<i64>) -> String {
        let now = chrono::Utc::now().timestamp();
        let customer = p.customer_ref.as_ref().unwrap();
        let session = p
            .session_ref
            .clone()
            .unwrap_or_else(|| reference(p, if p.provider == "stripe" { "cs" } else { "txn" }, 0));
        let subscription = end.map(|_| {
            p.subscription_ref
                .clone()
                .unwrap_or_else(|| reference(p, "sub", 0))
        });
        let recurring = end.is_some();
        let total = p.amount + 100;
        let currency = p.currency.to_ascii_lowercase();
        if p.provider == "stripe" {
            let payment = reference(p, "pi", n);
            let charge = reference(p, "ch", n);
            let invoice = reference(p, "in", n);
            let price = json!({"id":p.price_id,"currency":currency,"unit_amount":p.amount,"type":if recurring {"recurring"} else {"one_time"},"recurring":if recurring {json!({"interval":if p.billing_type == "monthly" {"month"} else {"year"},"interval_count":1,"usage_type":"licensed"})} else {Value::Null}});
            self.set(
                "stripe",
                "price",
                p.price_id.as_deref().unwrap(),
                price.clone(),
            );
            self.set("stripe", "checkout", &session, json!({"id":session,"livemode":p.mode=="live","customer":customer,"metadata":{"purchase_id":p.id},"mode":if recurring {"subscription"} else {"payment"},"status":"complete","payment_status":"paid","currency":currency,"amount_subtotal":p.amount,"amount_total":total,"payment_intent":payment,"invoice":if recurring {json!(invoice)} else {Value::Null},"subscription":subscription,"line_items":{"has_more":false,"data":[{"quantity":1,"price":price,"currency":currency,"amount_subtotal":p.amount,"amount_total":total}]}}));
            self.set("stripe", "intent", &payment, json!({"id":payment,"livemode":p.mode=="live","customer":customer,"status":"succeeded","amount_received":total,"currency":currency,"latest_charge":charge}));
            self.set("stripe", "charge", &charge, json!({"id":charge,"livemode":p.mode=="live","customer":customer,"currency":currency,"payment_intent":payment,"paid":true,"status":"succeeded","amount":total,"amount_refunded":0,"disputed":false}));
            self.set("stripe", "disputes", &charge, json!([]));
            if let Some(sub) = subscription {
                self.set("stripe", "invoice", &invoice, json!({"id":invoice,"livemode":p.mode=="live","customer":customer,"parent":{"subscription_details":{"subscription":sub,"metadata":{"purchase_id":p.id}}},"status":"paid","paid_out_of_band":false,"amount_remaining":0,"amount_paid":total,"amount_due":total,"currency":currency,"payments":{"has_more":false,"data":[{"status":"paid","payment":{"payment_intent":payment}}]},"lines":{"has_more":false,"data":[{"id":reference(p,"il",n),"pricing":{"price_details":{"price":p.price_id}},"quantity":1,"parent":{"subscription_item_details":{"proration":false}},"amount":p.amount,"currency":currency,"period":{"start":start,"end":end}}]},"status_transitions":{"paid_at":now}}));
                self.set("stripe", "subscription", &sub, json!({"id":sub,"livemode":p.mode=="live","customer":customer,"metadata":{"purchase_id":p.id},"status":"active","cancel_at_period_end":false,"cancel_at":null,"latest_invoice":invoice,"current_period_start":start,"current_period_end":end}));
            }
            if recurring { invoice } else { session }
        } else {
            let transaction = if n == 0 {
                session
            } else {
                reference(p, "txn", n)
            };
            let price = json!({"id":p.price_id,"unit_price":{"amount":p.amount.to_string(),"currency_code":p.currency},"billing_cycle":if recurring {json!({"interval":if p.billing_type=="monthly" {"month"} else {"year"},"frequency":1})} else {Value::Null},"tax_mode":"external"});
            let totals = json!({"subtotal":p.amount.to_string(),"discount":"0","tax":"100","total":total.to_string(),"credit":"0","grand_total":total.to_string(),"balance":"0","credit_to_balance":"0","currency_code":p.currency});
            self.set("paddle", "checkout", &transaction, json!({"id":transaction,"customer_id":customer,"custom_data":{"purchase_id":p.id},"subscription_id":subscription,"status":"completed","currency_code":p.currency,"items":[{"quantity":1,"price":price}],"details":{"line_items":[{"id":reference(p,"txnitm",n),"price_id":p.price_id,"quantity":1,"totals":totals}],"totals":totals,"adjusted_totals":{"grand_total":total.to_string(),"currency_code":p.currency}},"payments":[{"payment_attempt_id":reference(p,"pay",n),"status":"captured","amount":total.to_string(),"captured_at":iso(now)}],"billing_period":end.map(|end|json!({"starts_at":iso(start),"ends_at":iso(end)}))}));
            self.set("paddle", "adjustments", &transaction, json!([]));
            if let Some(sub) = subscription {
                self.set("paddle", "subscription", &sub, json!({"id":sub,"customer_id":customer,"custom_data":{"purchase_id":p.id},"status":"active","updated_at":iso(now),"scheduled_change":null,"current_billing_period":{"starts_at":iso(start),"ends_at":iso(end.unwrap())},"items":[{"quantity":1,"price":price}]}));
                self.set("paddle", "latest", &sub, json!({"id":transaction}));
            }
            transaction
        }
    }
}

pub fn iso(time: i64) -> String {
    chrono::DateTime::from_timestamp(time, 0)
        .unwrap()
        .to_rfc3339()
}

#[async_trait]
impl Gateway for FakeGateway {
    async fn create_customer(
        &self,
        p: &purchase::Model,
        request: CreateCustomerRequest,
    ) -> Result<String, GatewayError> {
        self.customers.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.metadata.unwrap()["purchase_id"], p.id);
        Ok(reference(
            p,
            if p.provider == "stripe" { "cus" } else { "ctm" },
            0,
        ))
    }
    async fn start_session(
        &self,
        p: &purchase::Model,
        request: StartSessionRequest,
    ) -> Result<SessionPayload, GatewayError> {
        self.starts
            .lock()
            .unwrap()
            .push((p.id.clone(), serde_json::to_value(&request).unwrap()));
        let id = reference(p, if p.provider == "stripe" { "cs" } else { "txn" }, 0);
        self.set(&p.provider, "checkout", &id, if p.provider == "stripe" {
            json!({"id":id,"status":"open","metadata":{"purchase_id":p.id},"customer":p.customer_ref,"livemode":p.mode=="live","payment_status":"unpaid"})
        } else { json!({"id":id,"status":"ready","custom_data":{"purchase_id":p.id},"customer_id":p.customer_ref}) });
        if fail(&self.fail_start) {
            return Err(GatewayError("provider_timeout"));
        }
        if p.provider == "stripe" {
            Ok(SessionPayload::StripeCheckoutRedirect {
                url: format!("https://checkout.stripe.com/c/pay/{id}"),
                provider_session_id: id,
            })
        } else {
            Ok(SessionPayload::PaddleInline {
                transaction_id: id,
                client_token: p.public_key.clone().unwrap(),
                customer_token: None,
            })
        }
    }
    async fn session_state(
        &self,
        _: &purchase::Model,
        _: &str,
    ) -> Result<CheckoutSessionState, GatewayError> {
        Err(GatewayError("fixture_uses_detailed_evidence"))
    }
    async fn cancel(
        &self,
        p: &purchase::Model,
        id: &str,
    ) -> Result<SubscriptionResult, GatewayError> {
        self.cancels.fetch_add(1, Ordering::SeqCst);
        let mut subscription = self.get(&p.provider, "subscription", id);
        let now = chrono::Utc::now();
        let end = if p.provider == "stripe" {
            subscription["current_period_end"].as_i64().unwrap()
        } else {
            chrono::DateTime::parse_from_rfc3339(
                subscription["current_billing_period"]["ends_at"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap()
            .timestamp()
        };
        if p.provider == "stripe" {
            subscription["cancel_at_period_end"] = json!(true);
            subscription["cancel_at"] = json!(end);
        } else {
            subscription["scheduled_change"] = json!({"action":"cancel","effective_at":iso(end)});
        }
        self.set(&p.provider, "subscription", id, subscription.clone());
        if fail(&self.fail_cancel) {
            return Err(GatewayError("provider_timeout"));
        }
        Ok(SubscriptionResult {
            provider_subscription_id: id.into(),
            provider_customer_id: p.customer_ref.clone().unwrap(),
            status: SubscriptionStatus::Active,
            items: vec![],
            current_period_start: now,
            current_period_end: chrono::DateTime::from_timestamp(end, 0).unwrap(),
            cancel_at_period_end: true,
            provider_metadata: subscription,
        })
    }
    async fn read(
        &self,
        p: &purchase::Model,
        resource: Resource<'_>,
    ) -> Result<Value, GatewayError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.observed_profiles.lock().unwrap().push((
            p.provider.clone(),
            p.mode.clone(),
            p.public_key.clone(),
        ));
        if fail(&self.fail_reads) {
            return Err(GatewayError("provider_unavailable"));
        }
        let (kind, id) = match resource {
            Resource::Checkout(id) => ("checkout", id),
            Resource::Invoice(id) => ("invoice", id),
            Resource::Charge(id) => ("charge", id),
            Resource::Subscription(id) => ("subscription", id),
            Resource::Adjustments(id) => ("adjustments", id),
            Resource::Price(id) => ("price", id),
            Resource::PaymentIntent(id) => ("intent", id),
            Resource::Dispute(id) => ("dispute", id),
            Resource::Disputes(id) => ("disputes", id),
            Resource::LatestTransaction(id) => ("latest", id),
        };
        let key = key(&p.provider, kind, id);
        let value = self
            .resources
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or(GatewayError("fixture_missing_resource"))?;
        let gate = {
            let mut guard = self.gate.lock().unwrap();
            if guard.as_ref().is_some_and(|(expected, _)| expected == &key) {
                guard.take().map(|(_, gate)| gate)
            } else {
                None
            }
        };
        if let Some(gate) = gate {
            gate.arrived.notify_one();
            gate.release.notified().await;
        }
        Ok(value)
    }
}

pub fn signature(provider: &str, bytes: &[u8], secret: &str, timestamp: i64) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(
        format!(
            "{timestamp}{}",
            if provider == "stripe" { "." } else { ":" }
        )
        .as_bytes(),
    );
    mac.update(bytes);
    let digest = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if provider == "stripe" {
        format!("t={timestamp},v1={digest}")
    } else {
        format!("ts={timestamp};h1={digest}")
    }
}
pub fn event(
    p: &purchase::Model,
    kind: &str,
    resource: &str,
    event_id: &str,
    timestamp: i64,
) -> Vec<u8> {
    let data = json!({"id":resource,"object":if kind.starts_with("invoice") {"invoice"} else if kind.starts_with("charge") {"charge"} else {"checkout.session"},"metadata":{"purchase_id":p.id},"custom_data":{"purchase_id":p.id},"customer":p.customer_ref,"customer_id":p.customer_ref});
    serde_json::to_vec(&if p.provider=="stripe" {json!({"id":event_id,"object":"event","api_version":"2025-03-31.basil","created":timestamp,"livemode":p.mode=="live","type":kind,"data":{"object":data},"pending_webhooks":1,"request":null})}
        else {json!({"event_id":event_id,"event_type":kind,"occurred_at":iso(timestamp),"notification_id":"ntf_fixture","data":{"id":resource,"status":"completed","customer_id":p.customer_ref,"custom_data":{"purchase_id":p.id},"currency_code":p.currency,"origin":"api","collection_mode":"automatic","items":[],"payments":[],"checkout":{"url":"https://checkout.example.test"},"created_at":iso(timestamp),"updated_at":iso(timestamp),"details":{"tax_rates_used":[],"line_items":[],"totals":{"subtotal":"1000","discount":"0","tax":"0","total":"1000","credit":"0","credit_to_balance":"0","balance":"0","grand_total":"1000","currency_code":p.currency},"adjusted_totals":{"subtotal":"1000","tax":"0","total":"1000","grand_total":"1000","currency_code":p.currency}}}})}).unwrap()
}
