use super::listings::{actor_id, image_response, media_error, route_id};
use crate::{
    articles::{
        self, EDIT_PERMISSION, TAXONOMY_PERMISSION, entities,
        queries::{self, ArticleSummary, EditorArticle, PublicArticle, PublicDetail, Term},
        taxonomy::{self, TaxonomyItem},
        workflow,
    },
    listings::{
        database_error, invalid, media, missing,
        queries::{Page, Pagination},
    },
    public_pages::{self, Seo},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, de::DeserializeOwned};
use suprnova::{
    DB, FrameworkError, HttpResponse, InertiaProps, Request, Response, handler, inertia_response,
    serde_json::json,
};

#[derive(InertiaProps)]
pub struct ArticlesProps {
    pub articles: Vec<ArticleSummary>,
    pub q: String,
    pub state: String,
    pub pagination: Pagination,
}
#[derive(InertiaProps)]
pub struct ArticleEditProps {
    pub article: Option<EditorArticle>,
    pub terms: Vec<Term>,
}
#[derive(InertiaProps)]
pub struct ArticlePreviewProps {
    pub article: EditorArticle,
}
#[derive(InertiaProps)]
pub struct ArticleIndexProps {
    pub articles: Vec<PublicArticle>,
    pub terms: Vec<Term>,
    pub q: String,
    pub category: String,
    pub tag: String,
    pub pagination: Pagination,
    pub seo: Seo,
}
#[derive(InertiaProps)]
pub struct ArticleDetailProps {
    pub article: PublicDetail,
    pub seo: Seo,
}
#[derive(InertiaProps)]
pub struct TaxonomyProps {
    pub terms: Vec<TaxonomyItem>,
    pub kind: String,
}

fn page(req: &Request) -> Result<Page, FrameworkError> {
    Page::parse(req.query_param("page"), req.query_param("per_page"))
}
async fn input<T: DeserializeOwned>(req: Request) -> Result<T, FrameworkError> {
    req.json()
        .await
        .map_err(|_| invalid("article", "Check the submitted fields and try again."))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    version: i64,
}

#[handler]
pub async fn index(req: Request) -> Response {
    let q = req.query_param("q").unwrap_or_default();
    let category = req.query_param("category").unwrap_or_default();
    let tag = req.query_param("tag").unwrap_or_default();
    let (articles, pagination) = queries::search(&q, &category, &tag, page(&req)?).await?;
    let seo = public_pages::article_index(&articles, &q, &category, &tag, &pagination)?;
    inertia_response!(
        &req,
        "articles/Index",
        ArticleIndexProps {
            articles,
            terms: queries::terms(true).await?,
            q,
            category,
            tag,
            pagination,
            seo,
        },
        public_pages::config()?
    )
}

#[handler]
pub async fn show(req: Request) -> Response {
    let slug = req.param("slug").map_err(|_| missing())?;
    let article = queries::detail(slug).await?;
    if article.card.slug != slug {
        return Ok(HttpResponse::text("")
            .status(301)
            .header("Location", format!("/articles/{}", article.card.slug))
            .header("Cache-Control", "no-store"));
    }
    let seo = public_pages::article(&article)?;
    inertia_response!(
        &req,
        "articles/Show",
        ArticleDetailProps { article, seo },
        public_pages::config()?
    )
}

#[handler]
pub async fn admin_index(req: Request) -> Response {
    let q = req.query_param("q").unwrap_or_default();
    let state = req.query_param("state").unwrap_or_default();
    let (articles, pagination) =
        queries::admin_search(actor_id().await?, &q, &state, page(&req)?).await?;
    inertia_response!(
        &req,
        "admin/Articles",
        ArticlesProps {
            articles,
            q,
            state,
            pagination
        }
    )
}

#[handler]
pub async fn create(req: Request) -> Response {
    articles::require_permission(actor_id().await?, EDIT_PERMISSION).await?;
    inertia_response!(
        &req,
        "admin/ArticleEdit",
        ArticleEditProps {
            article: None,
            terms: queries::terms(false).await?
        }
    )
}

#[handler]
pub async fn edit(req: Request) -> Response {
    let article = queries::editor(actor_id().await?, route_id(&req)?).await?;
    inertia_response!(
        &req,
        "admin/ArticleEdit",
        ArticleEditProps {
            article: Some(article),
            terms: queries::terms(false).await?
        }
    )
}

#[handler]
pub async fn preview(req: Request) -> Response {
    let article = queries::editor(actor_id().await?, route_id(&req)?).await?;
    let response: Response = inertia_response!(
        &req,
        "admin/ArticlePreview",
        ArticlePreviewProps { article }
    );
    Ok(response?
        .header("Cache-Control", "private, no-store")
        .header("X-Robots-Tag", "noindex, nofollow"))
}

#[handler]
pub async fn store(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = workflow::save(actor, None, input(req).await?).await?;
    suprnova::Redirect::to(format!("/admin/articles/{id}/edit")).into()
}

#[handler]
pub async fn update(req: Request) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::save(actor, Some(id), input(req).await?).await?;
    suprnova::Redirect::to(format!("/admin/articles/{id}/edit")).into()
}

#[handler]
pub async fn publish(req: Request) -> Response {
    change_publication(req, true).await
}
#[handler]
pub async fn unpublish(req: Request) -> Response {
    change_publication(req, false).await
}
async fn change_publication(req: Request, publish: bool) -> Response {
    let actor = actor_id().await?;
    let id = route_id(&req)?;
    workflow::publish(actor, id, input::<Version>(req).await?.version, publish).await?;
    suprnova::Redirect::to(format!("/admin/articles/{id}/edit")).into()
}

#[handler]
pub async fn upload(req: Request) -> Response {
    let actor = actor_id().await?;
    articles::require_permission(actor, EDIT_PERMISSION).await?;
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
            tracing::error!(error = %cleanup, "Could not remove an uncommitted article image");
        }
        return Err(database_error(error).into());
    }
    Ok(
        HttpResponse::json(json!({"id":id,"url":format!("/admin/articles/media/{id}")}))
            .status(201)
            .header("Cache-Control", "private, no-store"),
    )
}

#[handler]
pub async fn private_media(req: Request) -> Response {
    articles::require_permission(actor_id().await?, EDIT_PERMISSION).await?;
    let id = req.param("media_id").map_err(|_| missing())?;
    let db = DB::connection()?;
    let row = entities::media::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    image_response(&row.storage_key).await
}

#[handler]
pub async fn public_media(req: Request) -> Response {
    let slug = req.param("slug").map_err(|_| missing())?;
    let id = req.param("media_id").map_err(|_| missing())?;
    let article = queries::public_article(slug).await?;
    let db = DB::connection()?;
    let attached =
        entities::revision::Entity::find_by_id(article.published_revision_id.ok_or_else(missing)?)
            .filter(entities::revision::Column::ArticleId.eq(article.id))
            .filter(entities::revision::Column::MediaId.eq(id))
            .one(db.inner())
            .await
            .map_err(database_error)?;
    if attached.is_none() {
        return Err(missing().into());
    }
    let row = entities::media::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    image_response(&row.storage_key).await
}

#[handler]
pub async fn taxonomy_index(req: Request) -> Response {
    let kind = req
        .query_param("kind")
        .unwrap_or_else(|| "listing_category".into());
    let terms = taxonomy::list(actor_id().await?, &kind).await?;
    inertia_response!(&req, "admin/Taxonomy", TaxonomyProps { terms, kind })
}

fn kind(req: &Request) -> Result<String, FrameworkError> {
    req.param("kind").map(str::to_owned).map_err(|_| missing())
}
#[handler]
pub async fn taxonomy_store(req: Request) -> Response {
    let actor = actor_id().await?;
    let kind = kind(&req)?;
    articles::require_permission(actor, TAXONOMY_PERMISSION).await?;
    taxonomy::save(actor, &kind, None, input(req).await?).await?;
    suprnova::Redirect::to(format!("/admin/taxonomy?kind={kind}")).into()
}
#[handler]
pub async fn taxonomy_update(req: Request) -> Response {
    let actor = actor_id().await?;
    let kind = kind(&req)?;
    let id = route_id(&req)?;
    taxonomy::save(actor, &kind, Some(id), input(req).await?).await?;
    suprnova::Redirect::to(format!("/admin/taxonomy?kind={kind}")).into()
}
#[handler]
pub async fn taxonomy_remove(req: Request) -> Response {
    let actor = actor_id().await?;
    let kind = kind(&req)?;
    let id = route_id(&req)?;
    taxonomy::remove(actor, &kind, id, input::<Version>(req).await?.version).await?;
    suprnova::Redirect::to(format!("/admin/taxonomy?kind={kind}")).into()
}
