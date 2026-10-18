use serde::{Deserialize, de::DeserializeOwned};
use suprnova::{
    FrameworkError, HttpResponse, InertiaProps, Request, Response, handler, inertia_response,
    serde_json::json,
};

use super::listings::{actor_id, route_id};
use crate::billing::{self, Mode, Provider, cancellation, checkout, events, gateway, plans};

#[derive(InertiaProps)]
pub struct PlansProps {
    pub plans: Vec<plans::Plan>,
}
#[derive(InertiaProps)]
pub struct OffersProps {
    pub listing: checkout::ListingSummary,
    pub plans: Vec<checkout::Offer>,
    pub mode: Mode,
    pub purchase_id: Option<String>,
}
#[derive(InertiaProps)]
pub struct PurchaseProps {
    pub notifications: Vec<crate::notifications::Notice>,
    pub purchase: checkout::PurchaseView,
}

async fn input<T: DeserializeOwned>(req: Request) -> Result<T, FrameworkError> {
    req.json().await.map_err(|_| {
        billing::invalid(
            "purchase",
            "The submitted publishing request is invalid. Check the fields and try again.",
        )
    })
}
fn purchase_id(req: &Request) -> Result<String, FrameworkError> {
    let id = req.param("id").map_err(|_| crate::listings::missing())?;
    uuid::Uuid::parse_str(id).map_err(|_| crate::listings::missing())?;
    Ok(id.into())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}

#[handler]
pub async fn plans_index(req: Request) -> Response {
    let plans = plans::list(actor_id().await?).await?;
    inertia_response!(&req, "admin/PublishingPlans", PlansProps { plans })
}
#[handler]
pub async fn plans_store(req: Request) -> Response {
    let actor = actor_id().await?;
    plans::save(actor, None, input(req).await?).await?;
    suprnova::Redirect::to("/admin/plans").into()
}
#[handler]
pub async fn plans_update(req: Request) -> Response {
    let actor = actor_id().await?;
    let key = req
        .param("key")
        .map_err(|_| crate::listings::missing())?
        .to_owned();
    plans::save(actor, Some(&key), input(req).await?).await?;
    suprnova::Redirect::to("/admin/plans").into()
}
#[handler]
pub async fn offers(req: Request) -> Response {
    let checkout::Offers {
        listing,
        plans,
        mode,
        purchase_id,
    } = checkout::offers(actor_id().await?, route_id(&req)?).await?;
    inertia_response!(
        &req,
        "owner/PublishingPlans",
        OffersProps {
            listing,
            plans,
            mode,
            purchase_id
        }
    )
}
#[handler]
pub async fn start(req: Request) -> Response {
    let actor = actor_id().await?;
    let listing = route_id(&req)?;
    let gateway = gateway::service()?;
    let id = checkout::start(actor, listing, input(req).await?, gateway.as_ref()).await?;
    suprnova::Redirect::to(format!("/dashboard/purchases/{id}")).into()
}
#[handler]
pub async fn show(req: Request) -> Response {
    // Return URLs and query parameters never trigger provider writes or fulfillment.
    let actor = actor_id().await?;
    let purchase = checkout::view(actor, &purchase_id(&req)?).await?;
    let notifications = crate::notifications::recent(actor, purchase.listing_id).await?;
    inertia_response!(
        &req,
        "owner/Purchase",
        PurchaseProps {
            purchase,
            notifications
        }
    )
}
#[handler]
pub async fn continue_purchase(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = purchase_id(&req)?;
    let _: Empty = input(req).await?;
    let gateway = gateway::service()?;
    checkout::continue_purchase(actor, &id, gateway.as_ref()).await?;
    suprnova::Redirect::to(format!("/dashboard/purchases/{id}")).into()
}
#[handler]
pub async fn cancel(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = purchase_id(&req)?;
    let _: Empty = input(req).await?;
    let gateway = gateway::service()?;
    cancellation::request(actor, &id, gateway.as_ref()).await?;
    suprnova::Redirect::to(format!("/dashboard/purchases/{id}")).into()
}

async fn webhook(req: Request, provider: Provider, mode: Mode) -> Response {
    let header = if provider == Provider::Stripe {
        "stripe-signature"
    } else {
        "paddle-signature"
    };
    let signature = req
        .header(header)
        .ok_or_else(|| FrameworkError::bad_request("Payment signature is required."))?
        .to_owned();
    let (_, bytes) = req.body_bytes_with_cap(events::MAX_EVENT_BYTES).await?;
    events::accept(provider, mode, &bytes, &signature).await?;
    // Bounded workers perform provider reads after durable acceptance. A provider
    // timeout cannot keep webhook delivery open or lose a fulfillment retry.
    Ok(HttpResponse::json(json!({"accepted": true}))
        .status(202)
        .header("Cache-Control", "no-store"))
}
#[handler]
pub async fn stripe_test(req: Request) -> Response {
    webhook(req, Provider::Stripe, Mode::Test).await
}
#[handler]
pub async fn stripe_live(req: Request) -> Response {
    webhook(req, Provider::Stripe, Mode::Live).await
}
#[handler]
pub async fn paddle_test(req: Request) -> Response {
    webhook(req, Provider::Paddle, Mode::Test).await
}
#[handler]
pub async fn paddle_live(req: Request) -> Response {
    webhook(req, Provider::Paddle, Mode::Live).await
}
