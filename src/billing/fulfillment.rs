//! Apply verified provider facts and a fulfillment receipt in one transaction.
use std::collections::BTreeMap;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, ExprTrait, QueryFilter, Set,
    TransactionTrait, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use suprnova::{DB, FrameworkError, serde_json};

use super::{
    checkout::link_reference,
    evidence::{AdverseState, Cancellation, Settlement},
    lifecycle_entities::{event, payment, purchase, receipt, slot},
};
use crate::listings::{database_error, entities::entitlement, missing};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Fact {
    Settled(Settlement),
    Adverse {
        payment_ref: String,
        source: String,
        state: AdverseState,
    },
    Cancellation(Cancellation),
    CheckoutEnded,
    Ignored,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Observation {
    read_started_at: i64,
    state: AdverseState,
}

fn conflict() -> FrameworkError {
    super::invalid(
        "payment",
        "Payment evidence conflicts with retained purchase terms.",
    )
}

/// The request-start marker orders overlapping authoritative reads, regardless of
/// webhook delivery order. All facts in a batch came from the same account/mode.
pub(crate) async fn apply(
    event_id: &str,
    lease_token: &str,
    purchase_id: Option<&str>,
    facts: &[Fact],
    read_started_at: i64,
) -> Result<bool, FrameworkError> {
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    // Claim the event write lock first. A worker whose lease was replaced cannot
    // commit either its facts or its receipt.
    let changed = event::Entity::update_many()
        .col_expr(event::Column::Status, Expr::value("applying"))
        .filter(event::Column::Id.eq(event_id))
        .filter(event::Column::LeaseToken.eq(lease_token))
        .filter(event::Column::Status.eq("processing"))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    if changed.rows_affected == 0 {
        return Ok(false);
    }
    if receipt::Entity::find_by_id(event_id)
        .one(&tx)
        .await
        .map_err(database_error)?
        .is_some()
    {
        return Err(conflict());
    }
    if let Some(id) = purchase_id {
        purchase::Entity::update_many()
            .col_expr(
                purchase::Column::Version,
                Expr::col(purchase::Column::Version).add(1),
            )
            .filter(purchase::Column::Id.eq(id))
            .exec(&tx)
            .await
            .map_err(database_error)?;
        let mut row = purchase::Entity::find_by_id(id)
            .one(&tx)
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        for fact in facts {
            match fact {
                Fact::Settled(value) => settled(&tx, &mut row, value).await?,
                Fact::Adverse {
                    payment_ref,
                    source,
                    state,
                } => adverse(&tx, &row, payment_ref, source, state, read_started_at).await?,
                Fact::Cancellation(value) => {
                    cancellation(&tx, &mut row, value, read_started_at).await?
                }
                Fact::CheckoutEnded => {
                    // A terminal initial session must not close a settled purchase.
                    if payment::Entity::find()
                        .filter(payment::Column::PurchaseId.eq(&row.id))
                        .one(&tx)
                        .await
                        .map_err(database_error)?
                        .is_none()
                    {
                        row.state = "expired".into();
                        release(&tx, &row).await?;
                    }
                }
                Fact::Ignored => {}
            }
        }
        row.updated_at = chrono::Utc::now().timestamp();
        row.error_code = None;
        row.lease_until = None;
        // Converting a Model marks its values Unchanged, including values
        // mutated above. Mark the workflow fields explicitly for persistence.
        purchase::ActiveModel {
            id: Set(row.id),
            state: Set(row.state),
            session_ref: Set(row.session_ref),
            subscription_ref: Set(row.subscription_ref),
            cancel_at: Set(row.cancel_at),
            cancel_event_at: Set(row.cancel_event_at),
            cancel_requested: Set(row.cancel_requested),
            updated_at: Set(row.updated_at),
            error_code: Set(row.error_code),
            lease_until: Set(row.lease_until),
            ..Default::default()
        }
        .update(&tx)
        .await
        .map_err(database_error)?;
    }
    if let Some(id) = purchase_id {
        crate::notifications::payment_facts(&tx, id, event_id, facts).await?;
    }
    let now = chrono::Utc::now().timestamp();
    receipt::ActiveModel {
        event_record_id: Set(event_id.to_owned()),
        purchase_id: Set(purchase_id.map(str::to_owned)),
        outcome: Set(if facts.iter().all(|f| matches!(f, Fact::Ignored)) {
            "ignored"
        } else {
            "applied"
        }
        .into()),
        applied_at: Set(now),
    }
    .insert(&tx)
    .await
    .map_err(database_error)?;
    event::Entity::update_many()
        .col_expr(event::Column::Status, Expr::value("applied"))
        .col_expr(
            event::Column::Enrichment,
            Expr::value(
                serde_json::to_string(
                    &serde_json::json!({"read_started_at": read_started_at, "facts": facts}),
                )
                .map_err(|_| conflict())?,
            ),
        )
        .col_expr(event::Column::PurchaseId, Expr::value(purchase_id))
        .col_expr(event::Column::LeaseToken, Expr::value(None::<String>))
        .col_expr(event::Column::LeaseUntil, Expr::value(None::<i64>))
        .col_expr(event::Column::ErrorCode, Expr::value(None::<String>))
        .filter(event::Column::Id.eq(event_id))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    tx.commit().await.map_err(database_error)?;
    Ok(true)
}

async fn settled(
    tx: &DatabaseTransaction,
    row: &mut purchase::Model,
    value: &Settlement,
) -> Result<(), FrameworkError> {
    if row.customer_ref.as_deref() != Some(&value.customer_ref)
        || value.amount_total <= 0
        || value
            .period_end
            .is_some_and(|end| end <= value.period_start)
        || row
            .subscription_ref
            .as_ref()
            .zip(value.subscription_ref.as_ref())
            .is_some_and(|(a, b)| a != b)
        || row
            .session_ref
            .as_ref()
            .zip(value.session_ref.as_ref())
            .is_some_and(|(a, b)| a != b)
    {
        return Err(conflict());
    }
    let existing = payment::Entity::find()
        .filter(payment::Column::PurchaseId.eq(&row.id))
        .filter(payment::Column::ProviderPaymentId.eq(&value.payment_ref))
        .one(tx)
        .await
        .map_err(database_error)?;
    let paid = if let Some(paid) = existing {
        if paid.amount_total != value.amount_total
            || paid.period_end != value.period_end
            || (value.period_end.is_some() && paid.period_start != value.period_start)
        {
            return Err(conflict());
        }
        paid
    } else {
        // A replaced purchase can retain late money evidence for recovery, but
        // must not become another active publishing purchase for this listing.
        let owns_slot = slot::Entity::find_by_id(row.listing_id)
            .one(tx)
            .await
            .map_err(database_error)?
            .is_some_and(|slot| slot.purchase_id == row.id);
        let id = uuid::Uuid::new_v4().to_string();
        let end = value.period_end.map(|end| {
            row.cancel_at
                .filter(|_| row.state == "canceled")
                .map_or(end, |cutoff| std::cmp::min(end, cutoff))
        });
        let paid = payment::ActiveModel {
            id: Set(id.clone()),
            purchase_id: Set(row.id.clone()),
            provider_payment_id: Set(value.payment_ref.clone()),
            amount_total: Set(value.amount_total),
            period_start: Set(value.period_start),
            period_end: Set(value.period_end),
            status: Set("paid".into()),
            paid_at: Set(value.paid_at),
            refund_event_at: Set(0),
            dispute_event_at: Set(0),
            adverse: Set("{}".into()),
            created_at: Set(chrono::Utc::now().timestamp()),
        }
        .insert(tx)
        .await
        .map_err(database_error)?;
        entitlement::ActiveModel {
            id: Set(id),
            listing_id: Set(row.listing_id),
            mode: Set(row.mode.clone()),
            status: Set(if owns_slot { "active" } else { "superseded" }.into()),
            valid_from: Set(value.period_start),
            valid_until: Set(end),
            updated_at: Set(chrono::Utc::now().timestamp()),
        }
        .insert(tx)
        .await
        .map_err(database_error)?;
        paid
    };
    link_reference(tx, row, "payment", &value.payment_ref, Some(&paid.id)).await?;
    link_reference(tx, row, "customer", &value.customer_ref, None).await?;
    if let Some(id) = &value.session_ref {
        link_reference(tx, row, "session", id, None).await?;
        row.session_ref = Some(id.clone());
    }
    if let Some(id) = &value.subscription_ref {
        link_reference(tx, row, "subscription", id, None).await?;
        row.subscription_ref = Some(id.clone());
    }
    for (kind, id) in &value.references {
        let shared = matches!(kind.as_str(), "session" | "subscription" | "customer");
        link_reference(
            tx,
            row,
            kind,
            id,
            if shared { None } else { Some(&paid.id) },
        )
        .await?;
    }
    if !matches!(row.state.as_str(), "canceled" | "expired") {
        row.state = "active".into();
    }
    Ok(())
}

async fn adverse(
    tx: &DatabaseTransaction,
    row: &purchase::Model,
    payment_ref: &str,
    source: &str,
    value: &AdverseState,
    marker: i64,
) -> Result<(), FrameworkError> {
    let paid = payment::Entity::find()
        .filter(payment::Column::ProviderPaymentId.eq(payment_ref))
        .filter(payment::Column::PurchaseId.eq(&row.id))
        .one(tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if value.refund_total < 0 || value.refund_total > paid.amount_total {
        return Err(conflict());
    }
    let mut observations: BTreeMap<String, Observation> =
        serde_json::from_str(&paid.adverse).map_err(|_| conflict())?;
    if observations
        .get(source)
        .is_some_and(|prior| prior.read_started_at >= marker)
    {
        return Ok(());
    }
    observations.insert(
        source.into(),
        Observation {
            read_started_at: marker,
            state: value.clone(),
        },
    );
    let refunded = observations
        .values()
        .any(|v| v.state.refund_total >= paid.amount_total);
    let lost = observations.values().any(|v| v.state.lost_dispute);
    let disputed = observations.values().any(|v| v.state.disputed);
    let status = if refunded {
        "refunded"
    } else if lost {
        "revoked"
    } else if disputed {
        "disputed"
    } else {
        "paid"
    };
    let owns_slot = slot::Entity::find_by_id(row.listing_id)
        .one(tx)
        .await
        .map_err(database_error)?
        .is_some_and(|slot| slot.purchase_id == row.id);
    let eligibility = if status == "paid" {
        if owns_slot { "active" } else { "superseded" }
    } else {
        status
    };
    let encoded = serde_json::to_string(&observations).map_err(|_| conflict())?;
    payment::Entity::update_many()
        .col_expr(payment::Column::Status, Expr::value(status))
        .col_expr(payment::Column::Adverse, Expr::value(encoded))
        .filter(payment::Column::Id.eq(&paid.id))
        .exec(tx)
        .await
        .map_err(database_error)?;
    entitlement::Entity::update_many()
        .col_expr(entitlement::Column::Status, Expr::value(eligibility))
        .col_expr(
            entitlement::Column::UpdatedAt,
            Expr::value(chrono::Utc::now().timestamp()),
        )
        .filter(entitlement::Column::Id.eq(&paid.id))
        .exec(tx)
        .await
        .map_err(database_error)?;
    Ok(())
}

async fn cancellation(
    tx: &DatabaseTransaction,
    row: &mut purchase::Model,
    value: &Cancellation,
    marker: i64,
) -> Result<(), FrameworkError> {
    if row.cancel_event_at >= marker {
        return Ok(());
    }
    row.cancel_event_at = marker;
    if let Some(end) = value.effective_at {
        // Effective cancellation is terminal; a stale active subscription read
        // cannot erase it. Each period retains its original payment record.
        let cutoff = row
            .cancel_at
            .filter(|_| row.state == "canceled")
            .map_or(end, |old| std::cmp::min(old, end));
        row.cancel_at = Some(cutoff);
        row.state = "canceled".into();
        let payment_ids = payment::Entity::find()
            .select_only()
            .column(payment::Column::Id)
            .filter(payment::Column::PurchaseId.eq(&row.id))
            .into_query();
        entitlement::Entity::update_many()
            .col_expr(entitlement::Column::ValidUntil, Expr::value(cutoff))
            .col_expr(
                entitlement::Column::UpdatedAt,
                Expr::value(chrono::Utc::now().timestamp()),
            )
            .filter(entitlement::Column::Id.in_subquery(payment_ids))
            .filter(
                sea_orm::Condition::any()
                    .add(entitlement::Column::ValidUntil.is_null())
                    .add(entitlement::Column::ValidUntil.gt(cutoff)),
            )
            .exec(tx)
            .await
            .map_err(database_error)?;
        if cutoff <= chrono::Utc::now().timestamp() {
            release(tx, row).await?;
        }
    } else if row.state != "canceled" {
        row.cancel_at = value.scheduled_at;
        if value.scheduled_at.is_none() {
            // A confirmed active subscription permits another cancellation
            // request when the earlier mutation did not take effect.
            row.cancel_requested = false;
        }
    }
    Ok(())
}

use sea_orm::{QuerySelect, QueryTrait};

async fn release(tx: &DatabaseTransaction, row: &purchase::Model) -> Result<(), FrameworkError> {
    slot::Entity::delete_many()
        .filter(slot::Column::ListingId.eq(row.listing_id))
        .filter(slot::Column::PurchaseId.eq(&row.id))
        .exec(tx)
        .await
        .map_err(database_error)?;
    Ok(())
}
