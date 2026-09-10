use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::Serialize;
use suprnova::{
    Auth, DB, FrameworkError, InertiaProps, Request, Response, handler, inertia_response,
    rbac::HasRoles,
};

use crate::{
    accounts::queries::AuditEntry,
    articles,
    listings::{self, database_error, entities::listing, queries::Page},
    models::user::User,
};

#[derive(Serialize)]
pub struct ListingSummary {
    pub published: u64,
    pub awaiting_review: u64,
    pub total: u64,
}

#[derive(Serialize)]
pub struct ArticleSummary {
    pub published: u64,
    pub total: u64,
}

#[derive(InertiaProps)]
pub struct AdminOverviewProps {
    pub listing_summary: Option<ListingSummary>,
    pub article_summary: Option<ArticleSummary>,
    pub recent_activity: Option<Vec<AuditEntry>>,
}

async fn listing_summary(now: i64) -> Result<ListingSummary, FrameworkError> {
    let db = DB::connection()?;
    Ok(ListingSummary {
        published: listing::Entity::find()
            .filter(listings::queries::eligible(now))
            .count(db.inner())
            .await
            .map_err(database_error)?,
        awaiting_review: listing::Entity::find()
            .filter(listings::queries::awaiting_review())
            .count(db.inner())
            .await
            .map_err(database_error)?,
        total: listing::Entity::find()
            .filter(listing::Column::Archived.eq(false))
            .count(db.inner())
            .await
            .map_err(database_error)?,
    })
}

async fn article_summary() -> Result<ArticleSummary, FrameworkError> {
    use articles::entities::article;
    let db = DB::connection()?;
    Ok(ArticleSummary {
        published: article::Entity::find()
            .filter(articles::queries::published())
            .count(db.inner())
            .await
            .map_err(database_error)?,
        total: article::Entity::find()
            .count(db.inner())
            .await
            .map_err(database_error)?,
    })
}

#[handler]
pub async fn index(req: Request) -> Response {
    let user = Auth::user_as::<User>()
        .await?
        .ok_or(FrameworkError::Unauthorized)?;
    articles::require_permission(user.id, crate::billing::ADMIN_PERMISSION).await?;
    let listing_summary = if user
        .has_permission_to(listings::MODERATE_PERMISSION)
        .await?
    {
        Some(listing_summary(chrono::Utc::now().timestamp()).await?)
    } else {
        None
    };
    let article_summary = if user.has_permission_to(articles::EDIT_PERMISSION).await? {
        Some(article_summary().await?)
    } else {
        None
    };
    let recent_activity = if user
        .has_permission_to(crate::accounts::AUDIT_PERMISSION)
        .await?
    {
        Some(
            crate::accounts::queries::audit(user.id, Page { number: 1, size: 5 })
                .await?
                .0,
        )
    } else {
        None
    };
    inertia_response!(
        &req,
        "admin/Overview",
        AdminOverviewProps {
            listing_summary,
            article_summary,
            recent_activity
        }
    )
}
