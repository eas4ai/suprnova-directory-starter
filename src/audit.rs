//! Administrative summaries use field names and identifiers, never submitted secrets.
use crate::listings::{database_error, entities::audit};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};
use suprnova::FrameworkError;

pub async fn record(
    tx: &DatabaseTransaction,
    actor: i64,
    target_type: &'static str,
    target_id: String,
    action: &'static str,
    summary: &'static str,
) -> Result<(), FrameworkError> {
    audit::ActiveModel {
        actor_id: Set(actor),
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
