use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use suprnova::{DB, FrameworkError};

use super::{checkout, gateway::Gateway, lifecycle_entities::purchase, reconcile};
use crate::listings::{database_error, workflow::require_verified};

pub async fn request(actor: i64, id: &str, gateway: &dyn Gateway) -> Result<(), FrameworkError> {
    require_verified(actor).await?;
    let row = checkout::owned(actor, id).await?;
    let subscription = row.subscription_ref.as_deref().ok_or_else(|| {
        super::invalid("purchase", "There is no confirmed subscription to cancel.")
    })?;
    let now = chrono::Utc::now().timestamp();
    if row.state == "canceled"
        || row.cancel_at.is_some()
        || row.lease_until.is_some_and(|until| until > now)
    {
        return Ok(());
    }
    let db = DB::connection()?;
    let claimed = purchase::Entity::update_many()
        .col_expr(purchase::Column::CancelRequested, Expr::value(true))
        .col_expr(purchase::Column::Version, Expr::value(row.version + 1))
        .col_expr(purchase::Column::LeaseUntil, Expr::value(now + 30))
        .filter(purchase::Column::Id.eq(id))
        .filter(purchase::Column::Version.eq(row.version))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    if claimed.rows_affected == 0 {
        return Ok(());
    }
    // Repeating this action requests the same end-of-period cancellation. It
    // cannot create a checkout, extend a subscription or issue a refund.
    let outcome = gateway.cancel(&row, subscription).await;
    purchase::Entity::update_many()
        .col_expr(
            purchase::Column::ErrorCode,
            Expr::value(outcome.as_ref().err().map(|e| e.0)),
        )
        .col_expr(
            purchase::Column::LeaseUntil,
            Expr::value(if outcome.is_ok() {
                None
            } else {
                Some(now + 30)
            }),
        )
        .filter(purchase::Column::Id.eq(id))
        .filter(purchase::Column::Version.eq(row.version + 1))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    // Read confirmation is durable even when the mutation response was lost.
    reconcile::enqueue_recovery(id, Some(subscription), false).await?;
    Ok(())
}
