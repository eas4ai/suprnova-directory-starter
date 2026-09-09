use crate::{
    articles,
    config::site,
    listings::{self, database_error, missing, queries::Page},
};
use sea_orm::{EntityTrait, PaginatorTrait, QueryFilter};
use suprnova::{DB, FrameworkError, HttpResponse, Request, Response, handler};

const SITEMAP_SIZE: u64 = 100;
const MAX_SITEMAPS: u64 = 50_000;

pub(crate) fn xml(value: &str) -> String {
    value.chars().filter(|c| matches!(*c, '\u{9}' | '\u{a}' | '\u{d}' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'))
        .collect::<String>().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&apos;")
}

fn response(body: String, content_type: &'static str) -> Response {
    Ok(HttpResponse::bytes(body.into_bytes().into(), content_type)
        .header("X-Content-Type-Options", "nosniff")
        .header("Cache-Control", "no-store"))
}

#[handler]
pub async fn rss(_req: Request) -> Response {
    let site = site::read()?;
    let (rows, _) = articles::queries::search(
        "",
        "",
        "",
        Page {
            number: 1,
            size: 50,
        },
    )
    .await?;
    let mut body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>{}</title><link>{}/articles</link><description>{}</description>",
        xml(&site.name),
        xml(&site.origin),
        xml(&site.description)
    );
    for row in rows {
        let url = xml(&format!("{}/articles/{}", site.origin, row.slug));
        let date = chrono::DateTime::from_timestamp(row.published_at, 0)
            .ok_or_else(|| FrameworkError::internal("Article publication time is invalid."))?
            .to_rfc2822();
        body.push_str(&format!("<item><title>{}</title><link>{url}</link><guid isPermaLink=\"true\">{url}</guid><description>{}</description><pubDate>{}</pubDate></item>", xml(&row.title), xml(&row.summary), xml(&date)));
    }
    body.push_str("</channel></rss>");
    response(body, "application/rss+xml; charset=utf-8")
}

async fn page_urls(now: i64) -> Result<Vec<String>, FrameworkError> {
    let mut paths = vec!["/".into(), "/listings".into(), "/articles".into()];
    paths.extend(
        listings::queries::public_categories(now)
            .await?
            .into_iter()
            .map(|term| format!("/listings?category={}", term.slug)),
    );
    paths.extend(
        articles::queries::terms(true)
            .await?
            .into_iter()
            .map(|term| format!("/articles?{}={}", term.kind, term.slug)),
    );
    Ok(paths)
}

#[handler]
pub async fn sitemap(_req: Request) -> Response {
    let origin = site::origin()?;
    let now = chrono::Utc::now().timestamp();
    let db = DB::connection()?;
    let listings = listings::entities::listing::Entity::find()
        .filter(listings::queries::eligible(now))
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let articles = articles::entities::article::Entity::find()
        .filter(articles::queries::published())
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let pages = page_urls(now).await?.len() as u64;
    let groups = [
        ("listings", listings.div_ceil(SITEMAP_SIZE)),
        ("articles", articles.div_ceil(SITEMAP_SIZE)),
        ("pages", pages.div_ceil(SITEMAP_SIZE)),
    ];
    if groups.iter().map(|(_, count)| *count).sum::<u64>() > MAX_SITEMAPS {
        return Err(FrameworkError::internal("The catalog exceeds one sitemap index of 50000 pages. Partition this catalog into smaller indexes before serving its sitemap.").into());
    }
    let mut body = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">".to_owned();
    for (kind, count) in groups {
        for page in 1..=count {
            body.push_str(&format!(
                "<sitemap><loc>{}</loc></sitemap>",
                xml(&format!("{origin}/sitemaps/{kind}/{page}.xml"))
            ));
            if body.len() > 50 * 1024 * 1024 {
                return Err(FrameworkError::internal(
                    "Sitemap index exceeds the XML protocol size limit.",
                )
                .into());
            }
        }
    }
    body.push_str("</sitemapindex>");
    response(body, "application/xml; charset=utf-8")
}

#[handler]
pub async fn sitemap_page(req: Request) -> Response {
    let kind = req.param("kind").map_err(|_| missing())?;
    let number = req
        .param("page")
        .ok()
        .and_then(|value| value.strip_suffix(".xml"))
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|page| *page > 0 && *page <= 1_000_000)
        .ok_or_else(missing)?;
    let page = Page {
        number,
        size: SITEMAP_SIZE,
    };
    let now = chrono::Utc::now().timestamp();
    let paths = match kind {
        "listings" => listings::queries::search("", "", page, now)
            .await?
            .0
            .into_iter()
            .map(|row| format!("/listings/{}", row.slug))
            .collect::<Vec<_>>(),
        "articles" => articles::queries::search("", "", "", page)
            .await?
            .0
            .into_iter()
            .map(|row| format!("/articles/{}", row.slug))
            .collect(),
        "pages" => page_urls(now)
            .await?
            .into_iter()
            .skip(((number - 1) * SITEMAP_SIZE) as usize)
            .take(SITEMAP_SIZE as usize)
            .collect(),
        _ => return Err(missing().into()),
    };
    if paths.is_empty() {
        return Err(missing().into());
    }
    let origin = site::origin()?;
    let mut body = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">".to_owned();
    for path in paths {
        body.push_str(&format!(
            "<url><loc>{}</loc></url>",
            xml(&format!("{origin}{path}"))
        ));
    }
    body.push_str("</urlset>");
    response(body, "application/xml; charset=utf-8")
}

#[handler]
pub async fn robots(_req: Request) -> Response {
    let origin = site::origin()?;
    response(
        format!(
            "User-agent: *\nAllow: /\nDisallow: /admin\nDisallow: /dashboard\nDisallow: /billing\nDisallow: /login\nDisallow: /register\nDisallow: /verify-email\nDisallow: /reset-password\nDisallow: /forgot-password\nSitemap: {origin}/sitemap.xml\n"
        ),
        "text/plain; charset=utf-8",
    )
}
