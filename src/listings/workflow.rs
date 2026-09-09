use sea_orm::ExprTrait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    PaginatorTrait, QueryFilter, Set, TransactionTrait,
    sea_query::{Expr, OnConflict},
};
use suprnova::{DB, FrameworkError};

use super::{
    MODERATE_PERMISSION, SaveListing, conflict, database_error,
    entities::{category, listing, media, revision, revision_category},
    invalid, missing,
};
use crate::models::user;

pub async fn require_verified(actor_id: i64) -> Result<(), FrameworkError> {
    crate::accounts::require_active(actor_id).await?;
    let db = DB::connection()?;
    let actor = user::Entity::find_by_id(actor_id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or(FrameworkError::Unauthorized)?;
    if actor.email_verified_at.is_none() {
        return Err(
            suprnova::AppError::forbidden("Verify your email before changing listings.").into(),
        );
    }
    Ok(())
}

pub async fn require_moderator(actor_id: i64) -> Result<(), FrameworkError> {
    require_verified(actor_id).await?;
    for permission in [crate::billing::ADMIN_PERMISSION, MODERATE_PERMISSION] {
        if !suprnova::rbac::has_permission_for_model(
            "directory.user",
            &actor_id.to_string(),
            permission,
        )
        .await?
        {
            return Err(suprnova::AppError::forbidden(
                "Listing moderation permission is required.",
            )
            .into());
        }
    }
    Ok(())
}

/// Save a new immutable content revision. The listing pointer changes with it.
pub async fn save(
    actor_id: i64,
    id: Option<i64>,
    input: SaveListing,
) -> Result<i64, FrameworkError> {
    require_verified(actor_id).await?;
    let input = input.validate()?;
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_mutation(&transaction, actor_id).await?;
    let now = chrono::Utc::now().timestamp();
    let row = if let Some(id) = id {
        // Take the write lock through a conditional write before any transaction
        // reads. This also avoids a SQLite read-to-write snapshot upgrade race.
        let changed = listing::Entity::update_many()
            .col_expr(
                listing::Column::Version,
                Expr::col(listing::Column::Version).add(1),
            )
            .filter(listing::Column::Id.eq(id))
            .filter(listing::Column::OwnerId.eq(actor_id))
            .filter(listing::Column::Version.eq(input.version))
            .filter(listing::Column::Archived.eq(false))
            .exec(&transaction)
            .await
            .map_err(database_error)?;
        if changed.rows_affected != 1 {
            owned(&transaction, actor_id, id).await?;
            return Err(conflict());
        }
        owned(&transaction, actor_id, id).await?
    } else {
        if input.version != 0 {
            return Err(conflict());
        }
        listing::ActiveModel {
            owner_id: Set(actor_id),
            slug: Set(stable_slug(&input.title)),
            version: Set(1),
            current_revision_id: Set(None),
            approved_revision_id: Set(None),
            archived: Set(false),
            suspended: Set(false),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&transaction)
        .await
        .map_err(database_error)?
    };
    let count = category::Entity::find()
        .filter(category::Column::Id.is_in(input.category_ids.clone()))
        .filter(category::Column::Active.eq(true))
        .count(&transaction)
        .await
        .map_err(database_error)?;
    if count != input.category_ids.len() as u64 {
        return Err(invalid(
            "category_ids",
            "Choose between one and five active categories.",
        ));
    }
    if let Some(media_id) = &input.media_id {
        if media::Entity::find_by_id(media_id)
            .filter(media::Column::OwnerId.eq(actor_id))
            .one(&transaction)
            .await
            .map_err(database_error)?
            .is_none()
        {
            return Err(invalid(
                "media_id",
                "Upload an image owned by your account.",
            ));
        }
    }
    let proposed = revision::ActiveModel {
        listing_id: Set(row.id),
        search_text: Set(format!("{}\n{}", input.title, input.summary).to_lowercase()),
        title: Set(input.title),
        summary: Set(input.summary),
        description: Set(input.description),
        url: Set(input.url),
        media_id: Set(input.media_id),
        media_alt: Set(input.media_alt),
        status: Set("draft".to_owned()),
        reason: Set(None),
        decided_by: Set(None),
        decided_at: Set(None),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(&transaction)
    .await
    .map_err(database_error)?;
    revision_category::Entity::insert_many(input.category_ids.into_iter().map(|category_id| {
        revision_category::ActiveModel {
            revision_id: Set(proposed.id),
            category_id: Set(category_id),
        }
    }))
    .exec(&transaction)
    .await
    .map_err(database_error)?;
    listing::Entity::update_many()
        .col_expr(listing::Column::CurrentRevisionId, Expr::value(proposed.id))
        .filter(listing::Column::Id.eq(row.id))
        .exec(&transaction)
        .await
        .map_err(database_error)?;
    transaction.commit().await.map_err(database_error)?;
    Ok(row.id)
}

async fn owned<C: ConnectionTrait>(
    db: &C,
    actor_id: i64,
    id: i64,
) -> Result<listing::Model, FrameworkError> {
    listing::Entity::find_by_id(id)
        .filter(listing::Column::OwnerId.eq(actor_id))
        .one(db)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)
}

fn stable_slug(title: &str) -> String {
    let words: String = title
        .chars()
        .take(80)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let prefix = words
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!(
        "{}-{}",
        if prefix.is_empty() {
            "listing"
        } else {
            &prefix
        },
        uuid::Uuid::new_v4().simple()
    )
}

async fn lock_owned(
    transaction: &DatabaseTransaction,
    actor_id: i64,
    id: i64,
    version: i64,
) -> Result<listing::Model, FrameworkError> {
    let changed = listing::Entity::update_many()
        .col_expr(
            listing::Column::Version,
            Expr::col(listing::Column::Version).add(1),
        )
        .filter(listing::Column::Id.eq(id))
        .filter(listing::Column::OwnerId.eq(actor_id))
        .filter(listing::Column::Version.eq(version))
        .filter(listing::Column::Archived.eq(false))
        .exec(transaction)
        .await
        .map_err(database_error)?;
    if changed.rows_affected != 1 {
        owned(transaction, actor_id, id).await?;
        return Err(conflict());
    }
    owned(transaction, actor_id, id).await
}

pub async fn submit(actor_id: i64, id: i64, version: i64) -> Result<(), FrameworkError> {
    require_verified(actor_id).await?;
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_mutation(&transaction, actor_id).await?;
    let row = lock_owned(&transaction, actor_id, id, version).await?;
    let revision_id = row.current_revision_id.ok_or_else(missing)?;
    let changed = revision::Entity::update_many()
        .col_expr(revision::Column::Status, Expr::value("submitted"))
        .filter(revision::Column::Id.eq(revision_id))
        .filter(revision::Column::ListingId.eq(id))
        .filter(revision::Column::Status.eq("draft"))
        .exec(&transaction)
        .await
        .map_err(database_error)?;
    if changed.rows_affected != 1 {
        return Err(invalid(
            "listing",
            "Save a draft before submitting it for review.",
        ));
    }
    transaction.commit().await.map_err(database_error)
}

pub async fn archive(actor_id: i64, id: i64, version: i64) -> Result<(), FrameworkError> {
    require_verified(actor_id).await?;
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_mutation(&transaction, actor_id).await?;
    lock_owned(&transaction, actor_id, id, version).await?;
    listing::Entity::update_many()
        .col_expr(listing::Column::Archived, Expr::value(true))
        .filter(listing::Column::Id.eq(id))
        .exec(&transaction)
        .await
        .map_err(database_error)?;
    transaction.commit().await.map_err(database_error)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub version: i64,
    pub revision_id: i64,
    pub decision: String,
    pub reason: String,
}

pub async fn decide(actor_id: i64, id: i64, input: Decision) -> Result<(), FrameworkError> {
    require_moderator(actor_id).await?;
    if !matches!(input.decision.as_str(), "approve" | "reject") {
        return Err(invalid("decision", "Choose approve or reject."));
    }
    let reason = input.reason.trim();
    if reason.chars().count() > 2_000 || (input.decision == "reject" && reason.is_empty()) {
        return Err(invalid(
            "reason",
            "Provide a rejection reason in 1 to 2000 characters.",
        ));
    }
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&transaction, actor_id, MODERATE_PERMISSION).await?;
    let changed = listing::Entity::update_many()
        .col_expr(
            listing::Column::Version,
            Expr::col(listing::Column::Version).add(1),
        )
        .filter(listing::Column::Id.eq(id))
        .filter(listing::Column::Version.eq(input.version))
        .filter(listing::Column::CurrentRevisionId.eq(input.revision_id))
        .filter(listing::Column::Archived.eq(false))
        .exec(&transaction)
        .await
        .map_err(database_error)?;
    if changed.rows_affected != 1 {
        return Err(conflict());
    }
    let now = chrono::Utc::now().timestamp();
    let status = if input.decision == "approve" {
        "approved"
    } else {
        "rejected"
    };
    let changed = revision::Entity::update_many()
        .col_expr(revision::Column::Status, Expr::value(status))
        .col_expr(revision::Column::Reason, Expr::value(reason))
        .col_expr(revision::Column::DecidedBy, Expr::value(actor_id))
        .col_expr(revision::Column::DecidedAt, Expr::value(now))
        .filter(revision::Column::Id.eq(input.revision_id))
        .filter(revision::Column::ListingId.eq(id))
        .filter(revision::Column::Status.eq("submitted"))
        .exec(&transaction)
        .await
        .map_err(database_error)?;
    if changed.rows_affected != 1 {
        return Err(invalid(
            "listing",
            "Only the exact submitted revision can receive a decision.",
        ));
    }
    if status == "approved" {
        listing::Entity::update_many()
            .col_expr(
                listing::Column::ApprovedRevisionId,
                Expr::value(input.revision_id),
            )
            .filter(listing::Column::Id.eq(id))
            .exec(&transaction)
            .await
            .map_err(database_error)?;
    }
    record_audit(
        &transaction,
        actor_id,
        id,
        status,
        &format!("Revision {}: {}", input.revision_id, reason),
    )
    .await?;
    transaction.commit().await.map_err(database_error)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suspension {
    pub version: i64,
    pub suspended: bool,
    pub reason: String,
}

pub async fn suspend(actor_id: i64, id: i64, input: Suspension) -> Result<(), FrameworkError> {
    require_moderator(actor_id).await?;
    let reason = input.reason.trim();
    if reason.is_empty() || reason.chars().count() > 2_000 {
        return Err(invalid(
            "reason",
            "Record the reason in 1 to 2000 characters.",
        ));
    }
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&transaction, actor_id, MODERATE_PERMISSION).await?;
    let changed = listing::Entity::update_many()
        .col_expr(
            listing::Column::Version,
            Expr::col(listing::Column::Version).add(1),
        )
        .col_expr(listing::Column::Suspended, Expr::value(input.suspended))
        .filter(listing::Column::Id.eq(id))
        .filter(listing::Column::Version.eq(input.version))
        .exec(&transaction)
        .await
        .map_err(database_error)?;
    if changed.rows_affected != 1 {
        return Err(conflict());
    }
    record_audit(
        &transaction,
        actor_id,
        id,
        if input.suspended {
            "suspended"
        } else {
            "reinstated"
        },
        reason,
    )
    .await?;
    transaction.commit().await.map_err(database_error)
}

async fn record_audit(
    transaction: &DatabaseTransaction,
    actor_id: i64,
    id: i64,
    action: &str,
    reason: &str,
) -> Result<(), FrameworkError> {
    crate::audit::moderation(transaction, actor_id, id, action, reason).await
}

/// Idempotent operator setup; existing terms and operator edits are preserved.
pub async fn seed_categories() -> Result<(), FrameworkError> {
    let db = DB::connection()?;
    for (slug, name) in [
        ("software", "Software"),
        ("design", "Design"),
        ("learning", "Learning"),
        ("community", "Community"),
    ] {
        category::Entity::insert(category::ActiveModel {
            slug: Set(slug.to_owned()),
            name: Set(name.to_owned()),
            active: Set(true),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::column(category::Column::Slug)
                .do_nothing()
                .to_owned(),
        )
        .try_insert()
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    }
    Ok(())
}
