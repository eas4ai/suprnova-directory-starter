use super::{MANAGE_PERMISSION, entities::redirect};
use crate::listings::{database_error, invalid, missing};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use suprnova::{DB, FrameworkError};

pub const LIMIT: u64 = 1000;
const MAX_CHAIN: usize = 5;

/// One canonical spelling. Encoded separators, dot segments, query strings and credentials are rejected.
pub fn safe_path(path: &str) -> bool {
    path.len() <= 240
        && path.starts_with('/')
        && !path.starts_with("//")
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.' | b'~'))
        && !path.contains("//")
        && !path.split('/').any(|segment| matches!(segment, "." | ".."))
}

pub fn reserved(path: &str) -> bool {
    path == "/"
        || path.ends_with(".md")
        || [
            "/listings",
            "/articles",
            "/media",
            "/admin",
            "/dashboard",
            "/billing",
            "/login",
            "/logout",
            "/register",
            "/forgot-password",
            "/reset-password",
            "/verify-email",
            "/email",
            "/feed.xml",
            "/robots.txt",
            "/sitemap.xml",
            "/sitemaps",
            "/assets",
            "/_suprnova",
            "/favicon.ico",
            "/storage",
        ]
        .iter()
        .any(|prefix| {
            path == *prefix
                || path
                    .strip_prefix(prefix)
                    .is_some_and(|tail| tail.starts_with('/'))
        })
}

async fn live_on<C: ConnectionTrait>(db: &C, path: &str) -> Result<bool, FrameworkError> {
    if matches!(path, "/" | "/listings" | "/articles") {
        return Ok(true);
    }
    if let Some(slug) = path.strip_prefix("/listings/") {
        use crate::listings::{entities::listing, queries::eligible};
        return Ok(listing::Entity::find()
            .filter(listing::Column::Slug.eq(slug))
            .filter(eligible(chrono::Utc::now().timestamp()))
            .one(db)
            .await
            .map_err(database_error)?
            .is_some());
    }
    if let Some(slug) = path.strip_prefix("/articles/") {
        use crate::articles::{entities::article, queries::published};
        return Ok(article::Entity::find()
            .filter(article::Column::Slug.eq(slug))
            .filter(published())
            .one(db)
            .await
            .map_err(database_error)?
            .is_some());
    }
    Ok(false)
}

fn terminal<'a>(
    start: &'a str,
    redirects: &'a HashMap<String, String>,
) -> Result<&'a str, FrameworkError> {
    let mut path = start;
    let mut seen = HashSet::new();
    for _ in 0..=MAX_CHAIN {
        if !seen.insert(path) {
            return Err(invalid("destination", "Redirects cannot form a cycle."));
        }
        match redirects.get(path) {
            Some(next) => path = next,
            None => return Ok(path),
        }
    }
    Err(invalid(
        "destination",
        "Redirect chains must have at most five steps.",
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveRedirect {
    pub version: i64,
    pub source: String,
    pub destination: String,
}

pub async fn list(actor: i64) -> Result<Vec<redirect::Model>, FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    redirect::Entity::find()
        .order_by_asc(redirect::Column::Source)
        .limit(LIMIT)
        .all(DB::connection()?.inner())
        .await
        .map_err(database_error)
}

pub async fn save(actor: i64, id: Option<i64>, input: SaveRedirect) -> Result<(), FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    if !safe_path(&input.source) || reserved(&input.source) || !safe_path(&input.destination) {
        return Err(invalid(
            "source",
            "Use plain same-site paths. Existing route namespaces, queries, encoded paths and Markdown twins are reserved.",
        ));
    }
    if tokio::fs::metadata(
        std::path::Path::new("public").join(input.source.trim_start_matches('/')),
    )
    .await
    .is_ok()
    {
        return Err(invalid(
            "source",
            "A public file or directory already uses this path.",
        ));
    }
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&tx, actor, MANAGE_PERMISSION).await?;
    // Serialize validation and writes so concurrent edits cannot create a cycle or exceed the cap.
    tx.execute_unprepared("UPDATE seo_settings SET id = id WHERE id = 1")
        .await
        .map_err(database_error)?;
    let rows = redirect::Entity::find()
        .limit(LIMIT + 1)
        .all(&tx)
        .await
        .map_err(database_error)?;
    if id.is_none() && rows.len() >= LIMIT as usize {
        return Err(invalid(
            "source",
            "At most 1000 redirects are supported. Remove an unused redirect first.",
        ));
    }
    let existing = id.and_then(|id| rows.iter().find(|row| row.id == id));
    if existing.is_some_and(|row| row.source != input.source) {
        return Err(invalid(
            "source",
            "Existing redirect sources are stable. Remove an unused redirect before changing its source.",
        ));
    }
    if id.is_some() && existing.is_none() {
        return Err(missing());
    }
    if existing.map_or(input.version != 0, |row| row.version != input.version) {
        return Err(crate::listings::conflict());
    }
    if rows
        .iter()
        .any(|row| Some(row.id) != id && row.source == input.source)
    {
        return Err(invalid("source", "This path already has a redirect."));
    }
    let mut map: HashMap<_, _> = rows
        .iter()
        .filter(|row| Some(row.id) != id)
        .map(|row| (row.source.clone(), row.destination.clone()))
        .collect();
    map.insert(input.source.clone(), input.destination.clone());
    for source in map.keys() {
        terminal(source, &map)?;
    }
    if !live_on(&tx, terminal(&input.source, &map)?).await? {
        return Err(invalid(
            "destination",
            "Choose a currently public listing, article or index page, directly or through an existing redirect.",
        ));
    }
    let mut model: redirect::ActiveModel = existing.cloned().map(Into::into).unwrap_or_default();
    model.source = Set(input.source);
    model.destination = Set(input.destination);
    model.version = Set(input.version + 1);
    model.updated_at = Set(chrono::Utc::now().timestamp());
    let saved = model.save(&tx).await.map_err(database_error)?;
    crate::audit::record(
        &tx,
        actor,
        "seo_redirect",
        saved.id.unwrap().to_string(),
        "seo_redirect_saved",
        "Changed same-site redirect paths after conflict and cycle validation.",
    )
    .await?;
    tx.commit().await.map_err(database_error)
}

pub async fn remove(actor: i64, id: i64, version: i64) -> Result<(), FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&tx, actor, MANAGE_PERMISSION).await?;
    tx.execute_unprepared("UPDATE seo_settings SET id = id WHERE id = 1")
        .await
        .map_err(database_error)?;
    let row = redirect::Entity::find_by_id(id)
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if row.version != version {
        return Err(crate::listings::conflict());
    }
    if redirect::Entity::find()
        .filter(redirect::Column::Destination.eq(&row.source))
        .count(&tx)
        .await
        .map_err(database_error)?
        > 0
    {
        return Err(invalid(
            "source",
            "Update incoming redirects before removing this destination.",
        ));
    }
    redirect::Entity::delete_by_id(id)
        .exec(&tx)
        .await
        .map_err(database_error)?;
    crate::audit::record(
        &tx,
        actor,
        "seo_redirect",
        id.to_string(),
        "seo_redirect_removed",
        "Removed an unused manual redirect.",
    )
    .await?;
    tx.commit().await.map_err(database_error)
}

pub async fn resolve(path: &str) -> Result<Option<String>, FrameworkError> {
    if !safe_path(path) || reserved(path) {
        return Ok(None);
    }
    let db = DB::connection()?;
    let mut current = path.to_owned();
    let mut seen = HashSet::new();
    for step in 0..=MAX_CHAIN {
        if !seen.insert(current.clone()) {
            return Err(FrameworkError::internal("Stored redirect cycle detected."));
        }
        let row = redirect::Entity::find()
            .filter(redirect::Column::Source.eq(&current))
            .one(db.inner())
            .await
            .map_err(database_error)?;
        let Some(row) = row else {
            return if step > 0 && live_on(db.inner(), &current).await? {
                Ok(Some(current))
            } else {
                Ok(None)
            };
        };
        if !safe_path(&row.destination) {
            return Err(FrameworkError::internal(
                "Stored redirect destination is invalid.",
            ));
        }
        current = row.destination;
    }
    Err(FrameworkError::internal(
        "Stored redirect chain exceeds its limit.",
    ))
}
