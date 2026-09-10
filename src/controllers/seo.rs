use crate::{
    listings::{
        invalid,
        queries::{Page, Pagination},
    },
    seo,
};
use suprnova::{
    FrameworkError, InertiaProps, Request, Response, handler, inertia_response, redirect,
};

#[derive(InertiaProps)]
pub struct SeoProps {
    pub settings: seo::settings::Settings,
    pub report: seo::report::Report,
    pub redirects: Vec<seo::entities::redirect::Model>,
    pub not_found: Vec<seo::entities::not_found::Model>,
    pub not_found_pagination: Pagination,
}

async fn actor() -> Result<i64, FrameworkError> {
    let actor = super::listings::actor_id().await?;
    crate::articles::require_permission(actor, seo::MANAGE_PERMISSION).await?;
    Ok(actor)
}

async fn input<T: serde::de::DeserializeOwned>(req: Request) -> Result<T, FrameworkError> {
    req.json().await.map_err(|_| {
        invalid(
            "seo",
            "The submitted SEO form is invalid. Check its fields and try again.",
        )
    })
}

#[handler]
pub async fn index(req: Request) -> Response {
    let actor = actor().await?;
    let page = Page::parse(req.query_param("page"), req.query_param("per_page"))?;
    let missing_page = Page::parse(req.query_param("missing_page"), Some("25".into()))?;
    let kind = req.query_param("kind").unwrap_or_else(|| "listing".into());
    let (not_found, not_found_pagination) = seo::not_found::report(actor, missing_page).await?;
    inertia_response!(
        &req,
        "admin/Seo",
        SeoProps {
            settings: seo::settings::load().await?,
            report: seo::report::load(actor, &kind, page).await?,
            redirects: seo::redirects::list(actor).await?,
            not_found,
            not_found_pagination,
        }
    )
}

#[handler]
pub async fn update(req: Request) -> Response {
    let actor = actor().await?;
    seo::settings::save(actor, input(req).await?).await?;
    redirect!("/admin/seo").into()
}

#[handler]
pub async fn redirect_store(req: Request) -> Response {
    let actor = actor().await?;
    seo::redirects::save(actor, None, input(req).await?).await?;
    redirect!("/admin/seo").into()
}

#[handler]
pub async fn redirect_update(req: Request) -> Response {
    let actor = actor().await?;
    let id = super::listings::route_id(&req)?;
    seo::redirects::save(actor, Some(id), input(req).await?).await?;
    redirect!("/admin/seo").into()
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    version: i64,
}

#[handler]
pub async fn redirect_remove(req: Request) -> Response {
    let actor = actor().await?;
    let id = super::listings::route_id(&req)?;
    let input: Version = input(req).await?;
    seo::redirects::remove(actor, id, input.version).await?;
    redirect!("/admin/seo").into()
}
