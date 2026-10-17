use crate::listings::{
    database_error, invalid,
    queries::{Page, Pagination},
};
use sea_orm::{
    ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Statement, Value,
};
use serde::Serialize;
use std::collections::HashMap;
use suprnova::{DB, FrameworkError};

#[derive(Serialize)]
pub struct Finding {
    pub path: String,
    pub preview: super::Preview,
    pub sitemap_included: bool,
    pub findings: Vec<String>,
}
#[derive(Serialize)]
pub struct Report {
    pub kind: String,
    pub rows: Vec<Finding>,
    pub pagination: Pagination,
    pub scope: String,
}

fn catalog(backend: sea_orm::DbBackend) -> (String, Vec<Value>) {
    use crate::{articles, listings};
    let eligible = listings::entities::listing::Entity::find()
        .select_only()
        .column(listings::entities::listing::Column::Id)
        .filter(listings::queries::eligible(chrono::Utc::now().timestamp()))
        .build(backend);
    let published = articles::entities::article::Entity::find()
        .select_only()
        .column(articles::entities::article::Column::Id)
        .filter(articles::queries::published())
        .build(backend);
    let sql = format!(
        r#"WITH eligible_listings AS ({}), published_articles AS ({}), pages AS (
SELECT 'listing' AS kind, '/listings/' || l.slug AS path, r.title, r.summary AS description, CASE WHEN r.media_id IS NULL THEN '' ELSE '/media/listings/' || l.slug || '/' || r.media_id END AS image, r.seo
 FROM listings l JOIN listing_revisions r ON r.id = l.approved_revision_id WHERE l.id IN (SELECT id FROM eligible_listings)
UNION ALL SELECT 'article', '/articles/' || a.slug, r.title, r.summary, CASE WHEN r.media_id IS NULL THEN '' ELSE '/media/articles/' || a.slug || '/' || r.media_id END, r.seo
 FROM articles a JOIN article_revisions r ON r.id = a.published_revision_id WHERE a.id IN (SELECT id FROM published_articles)
UNION ALL SELECT 'listing_category', '/listings?category=' || c.slug, c.name, '', '', c.seo FROM listing_categories c WHERE c.active = TRUE
 AND EXISTS (SELECT 1 FROM listing_revision_categories rc JOIN listings l ON l.approved_revision_id = rc.revision_id WHERE rc.category_id = c.id AND l.id IN (SELECT id FROM eligible_listings))
UNION ALL SELECT t.kind, '/articles?' || t.kind || '=' || t.slug, t.name, '', '', t.seo FROM article_terms t WHERE t.active = TRUE
 AND EXISTS (SELECT 1 FROM article_revision_terms rt JOIN articles a ON a.published_revision_id = rt.revision_id WHERE rt.term_id = t.id AND a.id IN (SELECT id FROM published_articles))
), resolved AS (SELECT *, COALESCE(NULLIF(json_extract(seo, '$.title'), ''), title) AS search_title,
COALESCE(NULLIF(json_extract(seo, '$.description'), ''), NULLIF(description, ''), ?) AS search_description FROM pages) "#,
        eligible.sql, published.sql
    );
    let mut values = eligible.values.map(|v| v.0).unwrap_or_default();
    values.extend(published.values.map(|v| v.0).unwrap_or_default());
    (sql, values)
}

pub async fn load(actor: i64, kind: &str, page: Page) -> Result<Report, FrameworkError> {
    crate::articles::require_permission(actor, super::MANAGE_PERMISSION).await?;
    if !matches!(
        kind,
        "listing" | "article" | "listing_category" | "category" | "tag"
    ) {
        return Err(invalid("kind", "Choose a public content type."));
    }
    let site = crate::config::site::read()?;
    let defaults = super::settings::load().await?.defaults;
    let db = DB::connection()?;
    let backend = db.inner().get_database_backend();
    let (sql, mut values) = catalog(backend);
    values.push(
        if defaults.description.is_empty() {
            site.description.clone()
        } else {
            defaults.description.clone()
        }
        .into(),
    );
    let mut scoped_values = values.clone();
    scoped_values.push(kind.into());
    let total: i64 = db
        .inner()
        .query_one_raw(Statement::from_sql_and_values(
            backend,
            format!("{sql} SELECT COUNT(*) AS total FROM resolved WHERE kind = ?"),
            scoped_values.clone(),
        ))
        .await
        .map_err(database_error)?
        .ok_or_else(|| FrameworkError::internal("SEO report count is unavailable."))?
        .try_get("", "total")
        .map_err(database_error)?;
    scoped_values.extend([page.size.into(), ((page.number - 1) * page.size).into()]);
    let rows = db
        .inner()
        .query_all_raw(Statement::from_sql_and_values(
            backend,
            format!("{sql} SELECT * FROM resolved WHERE kind = ? ORDER BY path LIMIT ? OFFSET ?"),
            scoped_values,
        ))
        .await
        .map_err(database_error)?;
    let mut duplicate_counts = Vec::new();
    for field in ["search_title", "search_description"] {
        let keys = rows
            .iter()
            .map(|row| row.try_get::<String>("", field).map_err(database_error))
            .collect::<Result<Vec<_>, _>>()?;
        let mut counts = HashMap::new();
        if !keys.is_empty() {
            let mut query_values = values.clone();
            query_values.extend(keys.iter().cloned().map(Into::into));
            let marks = vec!["?"; keys.len()].join(",");
            let query = format!(
                "{sql} SELECT {field} AS value, COUNT(*) AS total FROM resolved WHERE {field} IN ({marks}) GROUP BY {field} HAVING COUNT(*) > 1"
            );
            for row in db
                .inner()
                .query_all_raw(Statement::from_sql_and_values(backend, query, query_values))
                .await
                .map_err(database_error)?
            {
                counts.insert(
                    row.try_get::<String>("", "value").map_err(database_error)?,
                    row.try_get::<i64>("", "total").map_err(database_error)?,
                );
            }
        }
        duplicate_counts.push(counts);
    }
    let mut findings = Vec::with_capacity(rows.len());
    for row in rows {
        let path: String = row.try_get("", "path").map_err(database_error)?;
        let title: String = row.try_get("", "title").map_err(database_error)?;
        let description: String = row.try_get("", "description").map_err(database_error)?;
        let image: String = row.try_get("", "image").map_err(database_error)?;
        let seo: String = row.try_get("", "seo").map_err(database_error)?;
        let overrides = super::Overrides::decode(&seo)?;
        let preview = super::preview(
            &site,
            &defaults,
            &overrides,
            &title,
            &description,
            (!image.is_empty()).then_some(image.as_str()),
            &path,
        );
        let mut messages = Vec::new();
        if preview.image.is_none() {
            messages.push("Add a public social image to this content or the site defaults.".into());
        }
        if preview.title.trim().is_empty() {
            messages.push("Add a search title or content title.".into());
        }
        if preview.description.trim().is_empty() {
            messages.push("Add a search description, content summary or site description.".into());
        }
        for (field, label, counts) in [
            ("search_title", "title", &duplicate_counts[0]),
            ("search_description", "description", &duplicate_counts[1]),
        ] {
            let value: String = row.try_get("", field).map_err(database_error)?;
            if let Some(count) = counts.get(&value) {
                messages.push(format!("This search {label} is shared by {count} public pages. Use specific text if these pages serve different purposes."));
            }
        }
        if preview.noindex {
            messages.push("Excluded from search and sitemaps by content or site settings. Clear noindex if this page should be indexed.".into());
        }
        findings.push(Finding {
            path,
            sitemap_included: !preview.noindex,
            preview,
            findings: messages,
        });
    }
    Ok(Report { kind: kind.into(), rows: findings, pagination: Pagination { page: page.number, per_page: page.size, total: total as u64 },
        scope: "Current eligible public content, paginated by type. Duplicate checks compare each displayed value across all eligible content and taxonomy pages. Private drafts are excluded. Previews show saved metadata; search and social services may display it differently.".into() })
}
