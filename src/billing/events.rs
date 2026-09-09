use std::collections::BTreeSet;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set, sea_query::OnConflict};
use suprnova::{
    DB, FrameworkError,
    payments::WebhookContext,
    serde_json::{self, Value},
};

use super::{
    Mode, Provider, adapters, gateway,
    lifecycle_entities::{event, purchase, reference},
    settings, stripe_evidence,
};
use crate::listings::database_error;

pub const MAX_EVENT_BYTES: usize = 256 * 1024;

/// The adapter's typed unmarshal predates these documented actions. Use its
/// pinned SDK signature primitive only for that schema gap, on original bytes.
fn verify_paddle_reversal(body: &Value, bytes: &[u8], signature: &str, key: &str) -> bool {
    if key.trim().is_empty()
        || !matches!(
            body["event_type"].as_str(),
            Some("adjustment.created" | "adjustment.updated")
        )
        || !matches!(
            body["data"]["action"].as_str(),
            Some("chargeback_warning_reverse" | "credit_reverse")
        )
    {
        return false;
    }
    let digest = signature
        .split(';')
        .find_map(|part| part.strip_prefix("h1="));
    // The SDK's hex parser slices byte pairs and can panic on odd or Unicode input.
    if !digest
        .is_some_and(|value| value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return false;
    }
    let timestamp = signature
        .split(';')
        .find_map(|part| part.strip_prefix("ts="))
        .and_then(|v| v.parse::<i64>().ok());
    if !timestamp.is_some_and(|time| time <= chrono::Utc::now().timestamp() + 5) {
        return false;
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    signature
        .parse::<paddle_rust_sdk::webhooks::Signature>()
        .is_ok_and(|signature| {
            signature
                .verify(
                    text,
                    key,
                    paddle_rust_sdk::webhooks::MaximumVariance::default(),
                )
                .is_ok()
        })
}

pub(crate) fn data<'a>(provider: &str, body: &'a Value) -> &'a Value {
    if provider == "stripe" {
        &body["data"]["object"]
    } else {
        &body["data"]
    }
}

/// Untrusted identifiers narrow key selection only. They never establish payment.
pub(crate) async fn find_purchase(
    provider: Provider,
    mode: Mode,
    body: &Value,
) -> Result<Option<purchase::Model>, FrameworkError> {
    let data = data(provider.as_str(), body);
    let mut purchases = BTreeSet::new();
    for metadata in [
        &data["metadata"],
        &data["custom_data"],
        &data["subscription_details"]["metadata"],
        &data["parent"]["subscription_details"]["metadata"],
    ] {
        if let Some(id) = metadata["purchase_id"]
            .as_str()
            .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        {
            purchases.insert(id.to_owned());
        }
    }
    let mut identifiers = BTreeSet::new();
    for field in [
        "id",
        "customer",
        "customer_id",
        "subscription",
        "subscription_id",
        "payment_intent",
        "charge",
        "invoice",
        "transaction_id",
    ] {
        if let Some(id) = stripe_evidence::id(&data[field]) {
            identifiers.insert(id.to_owned());
        }
    }
    if let Some(id) = stripe_evidence::invoice_subscription(data) {
        identifiers.insert(id.to_owned());
    }
    let db = DB::connection()?;
    if !identifiers.is_empty() {
        let matches = reference::Entity::find()
            .select_only()
            .column(reference::Column::PurchaseId)
            .distinct()
            .filter(reference::Column::Provider.eq(provider.as_str()))
            .filter(reference::Column::Mode.eq(mode.as_str()))
            .filter(reference::Column::Reference.is_in(identifiers))
            .limit(2)
            .into_tuple::<String>()
            .all(db.inner())
            .await
            .map_err(database_error)?;
        purchases.extend(matches);
    }
    if purchases.len() > 1 {
        return Err(FrameworkError::bad_request(
            "Payment references identify different purchases.",
        ));
    }
    let Some(id) = purchases.into_iter().next() else {
        return Ok(None);
    };
    purchase::Entity::find_by_id(id)
        .filter(purchase::Column::Provider.eq(provider.as_str()))
        .filter(purchase::Column::Mode.eq(mode.as_str()))
        .one(db.inner())
        .await
        .map_err(database_error)
}

pub async fn accept(
    provider: Provider,
    mode: Mode,
    bytes: &[u8],
    signature: &str,
) -> Result<String, FrameworkError> {
    if bytes.len() > MAX_EVENT_BYTES || signature.len() > 4096 {
        return Err(FrameworkError::bad_request(
            "Payment event exceeds the supported size.",
        ));
    }
    let body: Value = serde_json::from_slice(bytes)
        .map_err(|_| FrameworkError::bad_request("Invalid payment event body."))?;
    let candidate = find_purchase(provider, mode, &body).await?;
    let mut headers = http::HeaderMap::new();
    let header = if provider == Provider::Stripe {
        "stripe-signature"
    } else {
        "paddle-signature"
    };
    headers.insert(
        header,
        http::HeaderValue::from_str(signature)
            .map_err(|_| FrameworkError::bad_request("Invalid payment signature."))?,
    );
    let context = WebhookContext {
        body: bytes,
        headers: &headers,
        remote_addr: None,
    };
    let mut authenticated = None;
    if let Some(purchase) = &candidate {
        if let Ok(adapter) = gateway::adapter(purchase) {
            if adapter.verify(&context).is_ok()
                || (provider == Provider::Paddle
                    && gateway::credentials(purchase).is_ok_and(|credentials| {
                        verify_paddle_reversal(&body, bytes, signature, &credentials.webhook_key)
                    }))
            {
                authenticated = Some((adapter, format!("purchase:{}", purchase.id)));
            }
        }
    }
    // Key rotation can overlap with retained purchase keys. Disabled profiles
    // still authenticate existing events; enablement controls new checkout only.
    if authenticated.is_none() {
        let (revision, settings) = settings::read(mode)
            .await
            .map_err(super::BillingError::into_framework)?;
        let profile = settings.profile(provider);
        if let Some(credentials) = profile
            .credentials(mode, provider)
            .map_err(super::BillingError::into_framework)?
        {
            let adapter = adapters::construct(mode, provider, &profile.public_key, &credentials)
                .map_err(super::BillingError::into_framework)?;
            if adapter.verify(&context).is_ok()
                || (provider == Provider::Paddle
                    && verify_paddle_reversal(&body, bytes, signature, &credentials.webhook_key))
            {
                authenticated = Some((adapter, format!("settings:{revision}")));
            }
        }
    }
    let (adapter, credential_source) = authenticated
        .ok_or_else(|| FrameworkError::bad_request("Payment signature could not be verified."))?;
    let parsed = adapter
        .parse_event(bytes)
        .map_err(|_| FrameworkError::bad_request("Invalid authenticated payment event."))?;
    if parsed.provider != provider.as_str()
        || parsed.provider_event_id.len() > 255
        || parsed.provider_event_type.len() > 100
    {
        return Err(FrameworkError::bad_request(
            "Invalid payment event identity.",
        ));
    }
    if provider == Provider::Stripe && body["livemode"].as_bool() != Some(mode == Mode::Live) {
        return Err(FrameworkError::bad_request(
            "Payment event mode does not match this endpoint.",
        ));
    }
    let now = chrono::Utc::now().timestamp();
    let occurred = if provider == Provider::Stripe {
        body["created"].as_i64()
    } else {
        body["occurred_at"]
            .as_str()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp())
    }
    .filter(|time| *time > 0 && *time <= now + 300)
    .ok_or_else(|| FrameworkError::bad_request("Payment event has an invalid occurrence time."))?;
    let db = DB::connection()?;
    event::Entity::insert(event::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        provider: Set(provider.as_str().to_owned()),
        mode: Set(mode.as_str().to_owned()),
        provider_event_id: Set(parsed.provider_event_id.clone()),
        event_type: Set(parsed.provider_event_type),
        raw_body: Set(bytes.to_vec()),
        signature: Set(signature.to_owned()),
        occurred_at: Set(occurred),
        received_at: Set(now),
        purchase_id: Set(candidate.map(|p| p.id)),
        credential_source: Set(credential_source),
        status: Set("pending".to_owned()),
        attempts: Set(0),
        next_attempt_at: Set(now),
        error_code: Set(None),
        lease_token: Set(None),
        lease_until: Set(None),
        enrichment: Set(None),
    })
    .on_conflict(
        OnConflict::columns([
            event::Column::Provider,
            event::Column::Mode,
            event::Column::ProviderEventId,
        ])
        .do_nothing()
        .to_owned(),
    )
    .try_insert()
    .exec(db.inner())
    .await
    .map_err(database_error)?;
    let retained = event::Entity::find()
        .filter(event::Column::Provider.eq(provider.as_str()))
        .filter(event::Column::Mode.eq(mode.as_str()))
        .filter(event::Column::ProviderEventId.eq(parsed.provider_event_id))
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(|| FrameworkError::internal("Could not retain payment evidence."))?;
    if retained.raw_body != bytes {
        return Err(FrameworkError::bad_request(
            "This event identifier already has different retained evidence.",
        ));
    }
    Ok(retained.id)
}
