use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, de::DeserializeOwned};
use suprnova::{
    Auth, DB, FrameworkError, HttpResponse, InertiaProps, Request, Response, handler,
    inertia_response, redirect, serde_json::json,
};

use crate::{
    listings::{
        self, entities, media,
        queries::{self, Category, OwnerListing, Page, Pagination, PublicCard, PublicDetail},
        workflow,
    },
    models::user::User,
};

#[derive(InertiaProps)]
pub struct DirectoryIndexProps {
    pub heading: String,
    pub listings: Vec<PublicCard>,
    pub categories: Vec<Category>,
    pub q: String,
    pub category: String,
    pub pagination: Pagination,
    pub seo: crate::public_pages::Seo,
}
#[derive(InertiaProps)]
pub struct DirectoryDetailProps {
    pub listing: PublicDetail,
    pub seo: crate::public_pages::Seo,
}
#[derive(InertiaProps)]
pub struct OwnerListingsProps {
    pub listings: Vec<OwnerListing>,
    pub pagination: Pagination,
}
#[derive(InertiaProps)]
pub struct ListingEditProps {
    pub notifications: Vec<crate::notifications::Notice>,
    pub listing: Option<OwnerListing>,
    pub categories: Vec<Category>,
}
#[derive(InertiaProps)]
pub struct ListingReviewProps {
    pub listing: OwnerListing,
    pub categories: Vec<Category>,
}

pub(crate) async fn actor_id() -> Result<i64, FrameworkError> {
    Auth::user_as::<User>()
        .await?
        .map(|user| user.id)
        .ok_or(FrameworkError::Unauthorized)
}
pub(crate) fn route_id(req: &Request) -> Result<i64, FrameworkError> {
    req.param("id")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|id| *id > 0)
        .ok_or_else(listings::missing)
}
fn page(req: &Request) -> Result<Page, FrameworkError> {
    Page::parse(req.query_param("page"), req.query_param("per_page"))
}
async fn input<T: DeserializeOwned>(req: Request) -> Result<T, FrameworkError> {
    req.json().await.map_err(|_| {
        listings::invalid(
            "listing",
            "The submitted listing is invalid. Check the fields and try again.",
        )
    })
}

#[handler]
pub async fn index(req: Request) -> Response {
    render_index(req, "Explore the directory").await
}

pub(super) async fn render_index(req: Request, heading: &str) -> Response {
    let q = req.query_param("q").unwrap_or_default();
    let category = req.query_param("category").unwrap_or_default();
    let (listings, pagination) =
        queries::search(&q, &category, page(&req)?, chrono::Utc::now().timestamp()).await?;
    let seo = crate::public_pages::directory_index(
        heading,
        &listings,
        &q,
        &category,
        &pagination,
        req.path() == "/",
    )
    .await?;
    inertia_response!(
        &req,
        "directory/Index",
        DirectoryIndexProps {
            heading: heading.to_owned(),
            listings,
            categories: queries::public_categories(chrono::Utc::now().timestamp()).await?,
            q,
            category,
            pagination,
            seo,
        },
        crate::public_pages::config()?
    )
}

#[handler]
pub async fn show(req: Request) -> Response {
    let slug = req.param("slug").map_err(|_| listings::missing())?;
    let listing = queries::detail(slug, chrono::Utc::now().timestamp()).await?;
    let seo = crate::public_pages::listing(&listing).await?;
    inertia_response!(
        &req,
        "directory/Show",
        DirectoryDetailProps { listing, seo },
        crate::public_pages::config()?
    )
}

#[handler]
pub async fn owner_index(req: Request) -> Response {
    let (listings, pagination) = queries::owner_search(actor_id().await?, page(&req)?).await?;
    inertia_response!(
        &req,
        "owner/Listings",
        OwnerListingsProps {
            listings,
            pagination
        }
    )
}

#[handler]
pub async fn create(req: Request) -> Response {
    workflow::require_verified(actor_id().await?).await?;
    inertia_response!(
        &req,
        "owner/ListingEdit",
        ListingEditProps {
            listing: None,
            notifications: Vec::new(),
            categories: queries::categories().await?
        }
    )
}

#[handler]
pub async fn edit(req: Request) -> Response {
    let actor = actor_id().await?;
    let listing = queries::owner_listing(actor, route_id(&req)?).await?;
    let notifications = crate::notifications::recent(actor, listing.id).await?;
    inertia_response!(
        &req,
        "owner/ListingEdit",
        ListingEditProps {
            listing: Some(listing),
            notifications,
            categories: queries::categories().await?
        }
    )
}

#[handler]
pub async fn store(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = workflow::save(actor, None, input(req).await?).await?;
    suprnova::Redirect::to(format!("/dashboard/listings/{id}/edit")).into()
}

#[handler]
pub async fn update(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::save(actor, Some(id), input(req).await?).await?;
    suprnova::Redirect::to(format!("/dashboard/listings/{id}/edit")).into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    version: i64,
}

#[handler]
pub async fn submit(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::submit(actor, id, input::<Version>(req).await?.version).await?;
    suprnova::Redirect::to(format!("/dashboard/listings/{id}/edit")).into()
}

#[handler]
pub async fn archive(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::archive(actor, id, input::<Version>(req).await?.version).await?;
    redirect!("/dashboard/listings").into()
}

#[handler]
pub async fn queue(req: Request) -> Response {
    let (listings, pagination) = queries::review_queue(actor_id().await?, page(&req)?).await?;
    inertia_response!(
        &req,
        "admin/ListingQueue",
        OwnerListingsProps {
            listings,
            pagination
        }
    )
}

#[handler]
pub async fn review(req: Request) -> Response {
    let listing = queries::review_listing(actor_id().await?, route_id(&req)?).await?;
    inertia_response!(
        &req,
        "admin/ListingReview",
        ListingReviewProps {
            listing,
            categories: queries::categories().await?
        }
    )
}

#[handler]
pub async fn decide(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::decide(actor, id, input(req).await?).await?;
    suprnova::Redirect::to(format!("/admin/listings/{id}")).into()
}

#[handler]
pub async fn suspension(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::suspend(actor, id, input(req).await?).await?;
    suprnova::Redirect::to(format!("/admin/listings/{id}")).into()
}

#[handler]
pub async fn upload(req: Request) -> Response {
    let actor = actor_id().await?;
    workflow::require_verified(actor).await?;
    let (_, bytes) = req.body_bytes_with_cap(media::MAX_UPLOAD_BYTES).await?;
    let stored = media::persist(&bytes).await.map_err(media_error)?;
    let id = uuid::Uuid::new_v4().to_string();
    let db = DB::connection()?;
    let inserted = entities::media::ActiveModel {
        id: Set(id.clone()),
        owner_id: Set(actor),
        storage_key: Set(stored.key.clone()),
        width: Set(stored.width as i32),
        height: Set(stored.height as i32),
        created_at: Set(chrono::Utc::now().timestamp()),
    }
    .insert(db.inner())
    .await;
    if let Err(error) = inserted {
        if let Err(cleanup) = media::remove(&stored.key).await {
            tracing::error!(error = %cleanup, "Could not remove an uncommitted listing image");
        }
        return Err(listings::database_error(error).into());
    }
    Ok(
        HttpResponse::json(json!({ "id": id, "url": format!("/dashboard/listings/media/{id}") }))
            .status(201)
            .header("Cache-Control", "private, no-store"),
    )
}

pub(crate) fn media_error(error: media::MediaError) -> FrameworkError {
    match error {
        media::MediaError::InvalidImage
        | media::MediaError::TooLarge
        | media::MediaError::InvalidKey => listings::invalid("media_id", &error.to_string()),
        media::MediaError::Busy => FrameworkError::domain(error.to_string(), 503),
        _ => {
            tracing::error!(error = %error, "Private media operation failed");
            FrameworkError::internal(error.to_string())
        }
    }
}

pub(crate) async fn image_response(storage_key: &str) -> Response {
    let bytes = media::read(storage_key).await.map_err(media_error)?;
    Ok(HttpResponse::bytes(bytes.into(), "image/png")
        .header("X-Content-Type-Options", "nosniff")
        .header("Cache-Control", "private, no-store")
        .header("Content-Disposition", "inline"))
}

#[handler]
pub async fn private_media(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = req.param("media_id").map_err(|_| listings::missing())?;
    let db = DB::connection()?;
    let row = entities::media::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(listings::database_error)?
        .ok_or_else(listings::missing)?;
    if row.owner_id != actor {
        workflow::require_moderator(actor).await?;
    }
    image_response(&row.storage_key).await
}

#[handler]
pub async fn public_media(req: Request) -> Response {
    let slug = req.param("slug").map_err(|_| listings::missing())?;
    let id = req.param("media_id").map_err(|_| listings::missing())?;
    let listing = queries::public_listing(slug, chrono::Utc::now().timestamp()).await?;
    let db = DB::connection()?;
    let approved = entities::revision::Entity::find_by_id(
        listing.approved_revision_id.ok_or_else(listings::missing)?,
    )
    .filter(entities::revision::Column::ListingId.eq(listing.id))
    .filter(entities::revision::Column::MediaId.eq(id))
    .one(db.inner())
    .await
    .map_err(listings::database_error)?;
    if approved.is_none() {
        return Err(listings::missing().into());
    }
    let row = entities::media::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(listings::database_error)?
        .ok_or_else(listings::missing)?;
    image_response(&row.storage_key).await
}
