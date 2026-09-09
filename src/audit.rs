//! Shared audit summaries exclude submitted secrets and private moderation detail.
use crate::listings::{database_error, entities::audit};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
use suprnova::FrameworkError;

pub enum Actor {
    User(i64),
    Operator,
}
impl From<i64> for Actor {
    fn from(id: i64) -> Self {
        Self::User(id)
    }
}
struct Entry<'a> {
    actor: Actor,
    target_type: &'a str,
    target_id: String,
    action: &'a str,
    summary: &'a str,
    private_reason: Option<&'a str>,
}
pub async fn record(
    tx: &DatabaseTransaction,
    actor: impl Into<Actor>,
    target_type: &str,
    target_id: String,
    action: &str,
    summary: &str,
) -> Result<(), FrameworkError> {
    insert(
        tx,
        Entry {
            actor: actor.into(),
            target_type,
            target_id,
            action,
            summary,
            private_reason: None,
        },
    )
    .await
}
/// Retain the required reason without exposing it through the shared audit DTO.
pub async fn moderation(
    tx: &DatabaseTransaction,
    actor: i64,
    listing_id: i64,
    action: &str,
    reason: &str,
) -> Result<(), FrameworkError> {
    insert(
        tx,
        Entry {
            actor: Actor::User(actor),
            target_type: "listing",
            target_id: listing_id.to_string(),
            action,
            summary: "Listing moderation state and owner-facing reason saved.",
            private_reason: Some(reason),
        },
    )
    .await
}
async fn insert(tx: &DatabaseTransaction, entry: Entry<'_>) -> Result<(), FrameworkError> {
    let (actor_id, actor_type) = match entry.actor {
        Actor::User(id) => (id, "user"),
        Actor::Operator => (0, "operator"),
    };
    audit::ActiveModel {
        actor_id: Set(actor_id),
        actor_type: Set(actor_type.into()),
        target_type: Set(entry.target_type.into()),
        target_id: Set(entry.target_id),
        action: Set(entry.action.into()),
        summary: Set(entry.summary.into()),
        private_reason: Set(entry.private_reason.map(str::to_owned)),
        created_at: Set(chrono::Utc::now().timestamp()),
        ..Default::default()
    }
    .insert(tx)
    .await
    .map_err(database_error)?;
    Ok(())
}
