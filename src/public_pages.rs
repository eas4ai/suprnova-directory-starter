//! Public metadata and rendering use configured origin, never request Host.
use crate::{
    config::site,
    listings::queries::{Pagination, PublicCard, PublicDetail},
};
use serde::Serialize;
use suprnova::{
    FrameworkError, Frontend, InertiaConfig,
    serde_json::{self, Value, json},
};

#[derive(Serialize)]
pub struct Seo {
    pub title: String,
    pub description: String,
    pub canonical: String,
    pub image: Option<String>,
    pub kind: &'static str,
    pub structured_data: String,
}

pub fn metadata(
    title: &str,
    description: &str,
    path: &str,
    image: Option<&str>,
    kind: &'static str,
    data: Value,
) -> Result<Seo, FrameworkError> {
    let site = site::read()?;
    let encoded = serde_json::to_string(&data)
        .map_err(|_| FrameworkError::internal("Could not render public metadata."))?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    Ok(Seo {
        title: format!("{title} | {}", site.name),
        description: description.into(),
        canonical: format!("{}{path}", site.origin),
        image: image.map(|path| format!("{}{path}", site.origin)),
        kind,
        structured_data: encoded,
    })
}

pub fn list_path(base: &str, query: &[(&str, &str)], pagination: &Pagination) -> String {
    let mut pairs = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in query {
        if !value.is_empty() {
            pairs.append_pair(key, value);
        }
    }
    if pagination.page > 1 {
        pairs.append_pair("page", &pagination.page.to_string());
    }
    if pagination.per_page != 24 {
        pairs.append_pair("per_page", &pagination.per_page.to_string());
    }
    let query = pairs.finish();
    if query.is_empty() {
        base.into()
    } else {
        format!("{base}?{query}")
    }
}

pub fn breadcrumbs(items: &[(&str, String)]) -> Result<Value, FrameworkError> {
    let origin = site::origin()?;
    Ok(
        json!({"@context":"https://schema.org", "@type":"BreadcrumbList", "itemListElement": items.iter().enumerate()
        .map(|(index, (name, path))| json!({"@type":"ListItem","position":index + 1,"name":name,"item":format!("{origin}{path}")})).collect::<Vec<_>>()}),
    )
}

pub fn directory_index(
    heading: &str,
    cards: &[PublicCard],
    q: &str,
    category: &str,
    page: &Pagination,
    home: bool,
) -> Result<Seo, FrameworkError> {
    let site = site::read()?;
    let path = list_path(
        if home { "/" } else { "/listings" },
        &[("q", q), ("category", category)],
        page,
    );
    let data = json!({"@context":"https://schema.org","@type":"ItemList","itemListElement": cards.iter().enumerate()
        .map(|(index, row)| json!({"@type":"ListItem","position":(page.page - 1)*page.per_page + index as u64 + 1,"url":format!("{}/listings/{}",site.origin,row.slug),"name":row.title})).collect::<Vec<_>>()});
    metadata(heading, &site.description, &path, None, "website", data)
}

pub fn listing(detail: &PublicDetail) -> Result<Seo, FrameworkError> {
    let card = &detail.card;
    let path = format!("/listings/{}", card.slug);
    metadata(
        &card.title,
        &card.summary,
        &path,
        card.media_url.as_deref(),
        "website",
        breadcrumbs(&[
            ("Home", "/".into()),
            ("Directory", "/listings".into()),
            (&card.title, path.clone()),
        ])?,
    )
}

pub fn article_index(
    cards: &[crate::articles::queries::PublicArticle],
    q: &str,
    category: &str,
    tag: &str,
    page: &Pagination,
) -> Result<Seo, FrameworkError> {
    let origin = site::origin()?;
    metadata(
        "Articles",
        "Stories, guides and ideas from the directory.",
        &list_path(
            "/articles",
            &[("q", q), ("category", category), ("tag", tag)],
            page,
        ),
        None,
        "website",
        json!({"@context":"https://schema.org","@type":"ItemList","itemListElement": cards.iter().enumerate()
            .map(|(index,row)| json!({"@type":"ListItem","position":(page.page - 1)*page.per_page + index as u64 + 1,"url":format!("{origin}/articles/{}",row.slug),"name":row.title})).collect::<Vec<_>>()}),
    )
}

pub fn article(detail: &crate::articles::queries::PublicDetail) -> Result<Seo, FrameworkError> {
    let site = site::read()?;
    let card = &detail.card;
    let path = format!("/articles/{}", card.slug);
    let date = |time| {
        chrono::DateTime::from_timestamp(time, 0)
            .map(|v| v.to_rfc3339())
            .unwrap_or_default()
    };
    let mut article = json!({"@context":"https://schema.org","@type":"Article","headline":card.title,"description":card.summary,
        "mainEntityOfPage":format!("{}{path}",site.origin),"datePublished":date(card.published_at),"dateModified":date(std::cmp::max(card.modified_at,card.published_at)),
        "publisher":{"@type":"Organization","name":site.name}});
    if let Some(image) = &card.media_url {
        article["image"] = json!(format!("{}{image}", site.origin));
    }
    metadata(
        &card.title,
        &card.summary,
        &path,
        card.media_url.as_deref(),
        "article",
        json!([
            article,
            breadcrumbs(&[
                ("Home", "/".into()),
                ("Articles", "/articles".into()),
                (&card.title, path.clone())
            ])?
        ]),
    )
}

/// Public pages require actual server-rendered content. Private forms keep CSR.
pub fn config() -> Result<InertiaConfig, FrameworkError> {
    let url = std::env::var("SSR_URL").unwrap_or_else(|_| "http://127.0.0.1:13714".into());
    let parsed = url::Url::parse(&url).map_err(|_| {
        FrameworkError::internal("SSR_URL must be an HTTP URL for the internal rendering worker.")
    })?;
    if parsed.scheme() != "http"
        || parsed.host_str().is_none()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(FrameworkError::internal(
            "SSR_URL must be an HTTP origin for the internal rendering worker.",
        )
        .into());
    }
    let config = InertiaConfig::new()
        .frontend(Frontend::Vue)
        .ssr(url)
        .ssr_timeout(std::time::Duration::from_secs(3))
        .ssr_throw_on_error(true)
        .on_ssr_error(|_| tracing::error!("Public page rendering failed; check the SSR worker."));
    Ok(config)
}
