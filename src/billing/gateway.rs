//! Suprnova mutations/signatures plus bounded SDK reads for omitted evidence fields.
use std::{future::Future, sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Serialize;
use suprnova::{
    payments::{
        CheckoutSessionState, CreateCustomerRequest, PaymentProvider, SessionPayload,
        StartSessionRequest, SubscriptionResult,
    },
    serde_json::Value,
};

use super::{Mode, Provider, adapters, lifecycle_entities::purchase, secrets};

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatewayError(pub &'static str);

pub(crate) fn provider(value: &str) -> Result<Provider, GatewayError> {
    match value {
        "stripe" => Ok(Provider::Stripe),
        "paddle" => Ok(Provider::Paddle),
        _ => Err(GatewayError("invalid_provider")),
    }
}

pub(super) fn credentials(row: &purchase::Model) -> Result<secrets::Credentials, GatewayError> {
    let mode = Mode::parse(&row.mode).map_err(|_| GatewayError("invalid_mode"))?;
    secrets::decrypt(
        mode,
        provider(&row.provider)?,
        row.credentials
            .as_deref()
            .ok_or(GatewayError("missing_credentials"))?,
    )
    .map_err(|_| GatewayError("credentials_unreadable"))
}

pub(crate) fn adapter(row: &purchase::Model) -> Result<Arc<dyn PaymentProvider>, GatewayError> {
    adapters::construct(
        Mode::parse(&row.mode).map_err(|_| GatewayError("invalid_mode"))?,
        provider(&row.provider)?,
        row.public_key
            .as_deref()
            .ok_or(GatewayError("missing_credentials"))?,
        &credentials(row)?,
    )
    .map_err(|_| GatewayError("credentials_unreadable"))
}

async fn bounded<T, E>(operation: impl Future<Output = Result<T, E>>) -> Result<T, GatewayError> {
    tokio::time::timeout(REQUEST_TIMEOUT, operation)
        .await
        .map_err(|_| GatewayError("provider_timeout"))?
        .map_err(|_| GatewayError("provider_unavailable"))
}

fn value(data: impl Serialize) -> Result<Value, GatewayError> {
    suprnova::serde_json::to_value(data).map_err(|_| GatewayError("provider_evidence_invalid"))
}

#[derive(Clone, Copy)]
pub enum Resource<'a> {
    Checkout(&'a str),
    Invoice(&'a str),
    Charge(&'a str),
    Subscription(&'a str),
    Adjustments(&'a str),
    Price(&'a str),
    PaymentIntent(&'a str),
    Dispute(&'a str),
    Disputes(&'a str),
    LatestTransaction(&'a str),
}

/// Explicit dependency seam: HTTP controllers use SdkGateway. Tests can exercise
/// domain retries and transaction failures without contacting a payment account.
#[async_trait]
pub trait Gateway: Send + Sync {
    async fn create_customer(
        &self,
        purchase: &purchase::Model,
        request: CreateCustomerRequest,
    ) -> Result<String, GatewayError>;
    async fn start_session(
        &self,
        purchase: &purchase::Model,
        request: StartSessionRequest,
    ) -> Result<SessionPayload, GatewayError>;
    async fn session_state(
        &self,
        purchase: &purchase::Model,
        id: &str,
    ) -> Result<CheckoutSessionState, GatewayError>;
    async fn cancel(
        &self,
        purchase: &purchase::Model,
        id: &str,
    ) -> Result<SubscriptionResult, GatewayError>;
    async fn read(
        &self,
        purchase: &purchase::Model,
        resource: Resource<'_>,
    ) -> Result<Value, GatewayError>;
}

pub struct SdkGateway;

pub fn service() -> Result<Arc<dyn Gateway>, suprnova::FrameworkError> {
    suprnova::App::make::<dyn Gateway>()
        .ok_or_else(|| suprnova::FrameworkError::internal("The payment gateway is not registered."))
}

#[async_trait]
impl Gateway for SdkGateway {
    async fn create_customer(
        &self,
        row: &purchase::Model,
        request: CreateCustomerRequest,
    ) -> Result<String, GatewayError> {
        Ok(bounded(adapter(row)?.create_customer(request))
            .await?
            .provider_customer_id)
    }
    async fn start_session(
        &self,
        row: &purchase::Model,
        request: StartSessionRequest,
    ) -> Result<SessionPayload, GatewayError> {
        bounded(adapter(row)?.start_session(request)).await
    }
    async fn session_state(
        &self,
        row: &purchase::Model,
        id: &str,
    ) -> Result<CheckoutSessionState, GatewayError> {
        bounded(adapter(row)?.session_status(id)).await
    }
    async fn cancel(
        &self,
        row: &purchase::Model,
        id: &str,
    ) -> Result<SubscriptionResult, GatewayError> {
        bounded(adapter(row)?.cancel(id, true)).await
    }
    async fn read(
        &self,
        row: &purchase::Model,
        resource: Resource<'_>,
    ) -> Result<Value, GatewayError> {
        let credentials = credentials(row)?;
        match provider(&row.provider)? {
            Provider::Stripe => {
                let client = stripe::ClientBuilder::new(credentials.api_key)
                    .build()
                    .map_err(|_| GatewayError("credentials_unreadable"))?;
                read_stripe(&client, resource).await
            }
            Provider::Paddle => {
                let endpoint = match row.mode.as_str() {
                    "test" => paddle_rust_sdk::Paddle::SANDBOX,
                    "live" => paddle_rust_sdk::Paddle::PRODUCTION,
                    _ => return Err(GatewayError("invalid_mode")),
                };
                let client = paddle_rust_sdk::Paddle::new(credentials.api_key, endpoint)
                    .map_err(|_| GatewayError("credentials_unreadable"))?;
                read_paddle(&client, resource).await
            }
        }
    }
}

fn valid_id(id: &str, prefix: &str) -> Result<(), GatewayError> {
    if id.len() > 255
        || !id.starts_with(prefix)
        || id.len() == prefix.len()
        || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(GatewayError("provider_reference_invalid"));
    }
    Ok(())
}

/// Uses the same SDK request path in production and local wire fixtures.
pub async fn read_stripe(
    client: &stripe::Client,
    resource: Resource<'_>,
) -> Result<Value, GatewayError> {
    use stripe_client_core::{RequestBuilder, StripeMethod};
    #[derive(Serialize)]
    struct Expand<'a> {
        expand: [&'a str; 1],
    }
    match resource {
        Resource::Checkout(id) => {
            valid_id(id, "cs_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/checkout/sessions/{id}"))
                        .query(&Expand {
                            expand: ["line_items"],
                        })
                        .customize::<stripe_shared::CheckoutSession>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::Invoice(id) => {
            valid_id(id, "in_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/invoices/{id}"))
                        .query(&Expand {
                            expand: ["payments"],
                        })
                        .customize::<stripe_shared::Invoice>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::Charge(id) => {
            valid_id(id, "ch_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/charges/{id}"))
                        .customize::<stripe_shared::Charge>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::Subscription(id) => {
            valid_id(id, "sub_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/subscriptions/{id}"))
                        .customize::<stripe_shared::Subscription>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::Price(id) => {
            valid_id(id, "price_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/prices/{id}"))
                        .customize::<stripe_shared::Price>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::PaymentIntent(id) => {
            valid_id(id, "pi_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/payment_intents/{id}"))
                        .customize::<stripe_shared::PaymentIntent>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::Dispute(id) => {
            valid_id(id, "dp_")?;
            value(
                bounded(
                    RequestBuilder::new(StripeMethod::Get, format!("/disputes/{id}"))
                        .customize::<stripe_shared::Dispute>()
                        .send(client),
                )
                .await?,
            )
        }
        Resource::Disputes(charge) => {
            valid_id(charge, "ch_")?;
            #[derive(Serialize)]
            struct Page<'a> {
                charge: &'a str,
                limit: u32,
                #[serde(skip_serializing_if = "Option::is_none")]
                starting_after: Option<String>,
            }
            let mut query = Page {
                charge,
                limit: 100,
                starting_after: None,
            };
            let mut rows = Vec::new();
            for _ in 0..5 {
                let page = value(
                    bounded(
                        RequestBuilder::new(StripeMethod::Get, "/disputes")
                            .query(&query)
                            .customize::<stripe_types::List<stripe_shared::Dispute>>()
                            .send(client),
                    )
                    .await?,
                )?;
                let data = page["data"]
                    .as_array()
                    .ok_or(GatewayError("provider_evidence_invalid"))?;
                rows.extend(data.iter().cloned());
                if page["has_more"].as_bool() == Some(false) {
                    return value(rows);
                }
                if page["has_more"].as_bool() != Some(true) {
                    return Err(GatewayError("provider_evidence_invalid"));
                }
                let last = data
                    .last()
                    .and_then(|v| v["id"].as_str())
                    .ok_or(GatewayError("provider_evidence_invalid"))?;
                valid_id(last, "dp_")?;
                if query.starting_after.as_deref() == Some(last) {
                    return Err(GatewayError("provider_evidence_invalid"));
                }
                query.starting_after = Some(last.into());
            }
            Err(GatewayError("disputes_exceed_recovery_limit"))
        }
        Resource::Adjustments(_) | Resource::LatestTransaction(_) => {
            Err(GatewayError("unsupported_evidence_read"))
        }
    }
}

pub async fn read_paddle(
    client: &paddle_rust_sdk::Paddle,
    resource: Resource<'_>,
) -> Result<Value, GatewayError> {
    match resource {
        Resource::Checkout(id) => {
            valid_id(id, "txn_")?;
            value(bounded(client.transaction_get(id).send()).await?.data)
        }
        Resource::Subscription(id) => {
            valid_id(id, "sub_")?;
            value(bounded(client.subscription_get(id).send()).await?.data)
        }
        Resource::Adjustments(id) => {
            valid_id(id, "txn_")?;
            let mut request = client.adjustments_list();
            request.transaction_ids([id]).per_page(50);
            // The SDK's fixed AdjustmentAction enum predates documented reversal
            // types. Its generic pagination retains those fields for validation.
            let mut pages = paddle_rust_sdk::paginated::Paginated::<Vec<Value>>::new(
                client,
                "/adjustments",
                &request,
            );
            let mut rows = Vec::new();
            // Restoration requires a complete adjustment set, never a truncated page.
            for _ in 0..5 {
                let Some(page) = bounded(pages.next()).await? else {
                    return value(rows);
                };
                rows.extend(page.data);
            }
            Err(GatewayError("adjustments_exceed_recovery_limit"))
        }
        Resource::LatestTransaction(id) => {
            valid_id(id, "sub_")?;
            // Only the newest completed period is needed to repair current
            // subscription access. Older transactions remain explicitly recoverable.
            let query = suprnova::serde_json::json!({"subscription_id": id, "status": "completed", "order_by": "created_at[DESC]", "per_page": 1});
            let mut pages = paddle_rust_sdk::paginated::Paginated::<Vec<Value>>::new(
                client,
                "/transactions",
                query,
            );
            Ok(bounded(pages.next())
                .await?
                .and_then(|page| page.data.into_iter().next())
                .unwrap_or(Value::Null))
        }
        _ => Err(GatewayError("unsupported_evidence_read")),
    }
}
