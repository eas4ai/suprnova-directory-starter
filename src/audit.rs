//! Administrative summaries use field names and identifiers, never submitted secrets.
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

pub async fn record(
    tx: &DatabaseTransaction,
    actor: impl Into<Actor>,
    target_type: &str,
    target_id: String,
    action: &str,
    summary: &str,
) -> Result<(), FrameworkError> {
    let (actor_id, actor_type) = match actor.into() {
        Actor::User(id) => (id, "user"),
        Actor::Operator => (0, "operator"),
    };
    audit::ActiveModel {
        actor_id: Set(actor_id),
        actor_type: Set(actor_type.into()),
        target_type: Set(target_type.into()),
        target_id: Set(target_id),
        action: Set(action.into()),
        summary: Set(summary.into()),
        created_at: Set(chrono::Utc::now().timestamp()),
        ..Default::default()
    }
    .insert(tx)
    .await
    .map_err(database_error)?;
    Ok(())
}
