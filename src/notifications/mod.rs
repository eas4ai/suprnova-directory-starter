//! Domain intents commit with their change. Email acknowledgment is at least once.
pub mod delivery;
pub mod entity;
mod payment;
use crate::listings::{database_error, entities::listing, missing};
pub(crate) use payment::facts as payment_facts;
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    sea_query::OnConflict,
};
use serde::Serialize;
use suprnova::{DB, FrameworkError};

pub struct Intent<'a> {
    pub event_key: &'a str,
    pub owner_id: i64,
    pub listing_id: i64,
    pub purchase_id: Option<&'a str>,
    pub title: &'a str,
    pub body: &'a str,
}
pub async fn record(tx: &DatabaseTransaction, intent: Intent<'_>) -> Result<(), FrameworkError> {
    let now = chrono::Utc::now().timestamp();
    let path = intent.purchase_id.map_or_else(
        || format!("/dashboard/listings/{}/edit", intent.listing_id),
        |id| format!("/dashboard/purchases/{id}"),
    );
    entity::Entity::insert(entity::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        event_key: Set(intent.event_key.into()),
        owner_id: Set(intent.owner_id),
        listing_id: Set(intent.listing_id),
        purchase_id: Set(intent.purchase_id.map(str::to_owned)),
        title: Set(intent.title.into()),
        body: Set(intent.body.into()),
        path: Set(path),
        status: Set("pending".into()),
        attempts: Set(0),
        next_attempt_at: Set(now),
        lease_token: Set(None),
        lease_until: Set(None),
        delivered_at: Set(None),
        last_error: Set(None),
        created_at: Set(chrono::Utc::now().timestamp_micros()),
    })
    .on_conflict(
        OnConflict::column(entity::Column::EventKey)
            .do_nothing()
            .to_owned(),
    )
    .try_insert()
    .exec(tx)
    .await
    .map_err(database_error)?;
    Ok(())
}
pub async fn moderation(
    tx: &DatabaseTransaction,
    listing_id: i64,
    event_key: &str,
    action: &str,
    reason: &str,
) -> Result<(), FrameworkError> {
    let row = listing::Entity::find_by_id(listing_id)
        .one(tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    let title = match action {
        "approved" => "Listing approved",
        "rejected" => "Listing rejected",
        "suspended" => "Listing suspended",
        _ => "Listing reinstated",
    };
    let mut body = match action {
        "approved" => "Your submitted revision was approved. Publication also requires an eligible publishing plan.",
        "rejected" => "Your submitted revision needs changes before approval.",
        "suspended" => "Your listing has been suspended and is no longer public.",
        _ => "Your listing was reinstated. Publication still depends on moderation and plan eligibility.",
    }.to_owned();
    if !reason.is_empty() {
        body.push_str("\n\nReason: ");
        body.push_str(reason);
    }
    record(
        tx,
        Intent {
            event_key,
            owner_id: row.owner_id,
            listing_id,
            purchase_id: None,
            title,
            body: &body,
        },
    )
    .await
}
/// Safe owner-facing snapshot; provider payloads and credentials never enter mail.
pub async fn purchase(
    tx: &DatabaseTransaction,
    id: &str,
    event_key: &str,
) -> Result<(), FrameworkError> {
    payment_event(tx, id, event_key, "").await
}
pub async fn payment_event(
    tx: &DatabaseTransaction,
    id: &str,
    event_key: &str,
    detail: &str,
) -> Result<(), FrameworkError> {
    use crate::billing::lifecycle_entities::purchase;
    let row = purchase::Entity::find_by_id(id)
        .one(tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    let state = if row.provider == "free" {
        "Your free publishing plan is active, subject to moderation. No payment was charged."
    } else {
        match row.state.as_str() {
            "active" => {
                "Your purchase has confirmed payment history. Current publication depends on payment and moderation eligibility."
            }
            "open" => "Your checkout is ready. Complete payment to activate paid publishing.",
            "canceled" => {
                "Cancellation was confirmed. Access ends at the confirmed paid-period cutoff."
            }
            "expired" => "This checkout expired. Review your publishing options.",
            "customer_unknown" => {
                "Checkout preparation needs recovery. Your purchase remains saved."
            }
            _ => "Your purchase is being prepared. Payment has not yet been confirmed.",
        }
    };
    let body = format!(
        "{detail}\n{state}\nMode: {}.\nReview your purchase page for current payment and publication status.",
        row.mode
    );
    let title = match row.state.as_str() {
        "reserved" => "Publishing request saved",
        "customer_creating" => "Preparing payment account",
        "checkout_creating" => "Preparing checkout",
        "open" => "Checkout ready",
        "customer_unknown" => "Checkout needs recovery",
        "expired" => "Checkout expired",
        "canceled" => "Subscription canceled",
        _ => "Publishing payment updated",
    };
    record(
        tx,
        Intent {
            event_key,
            owner_id: row.owner_id,
            listing_id: row.listing_id,
            purchase_id: Some(id),
            title,
            body: &body,
        },
    )
    .await
}
#[derive(Serialize)]
pub struct Notice {
    pub id: String,
    pub title: String,
    pub body: String,
    pub created_at: i64,
}
pub async fn recent(owner_id: i64, listing_id: i64) -> Result<Vec<Notice>, FrameworkError> {
    let db = DB::connection()?;
    // Scope independently even when the caller has already authorized its page.
    let rows = entity::Entity::find()
        .filter(entity::Column::OwnerId.eq(owner_id))
        .filter(entity::Column::ListingId.eq(listing_id))
        .order_by_desc(entity::Column::CreatedAt)
        .order_by_desc(entity::Column::Id)
        .limit(20)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    Ok(rows
        .into_iter()
        .map(|n| Notice {
            id: n.id,
            title: n.title,
            body: n.body,
            created_at: n.created_at / 1_000_000,
        })
        .collect())
}
