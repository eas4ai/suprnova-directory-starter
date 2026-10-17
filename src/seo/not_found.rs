use super::entities::not_found as entity;
use crate::listings::{
    database_error,
    queries::{Page, Pagination},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Statement, TransactionTrait,
};
use suprnova::{DB, FrameworkError};

const RETENTION: i64 = 30 * 86400;
const MAX_ROWS: u64 = 1000;

pub fn reportable(path: &str) -> bool {
    super::redirects::safe_path(path)
        && !path.ends_with(".md")
        && path != "/"
        && ![
            "admin",
            "dashboard",
            "billing",
            "login",
            "logout",
            "register",
            "password",
            "verify",
            "verification",
            "token",
            "secret",
            "session",
            "auth",
            "email",
            "_suprnova",
            "assets",
            "media",
            "storage",
        ]
        .iter()
        .any(|word| {
            path.to_ascii_lowercase()
                .split('/')
                .any(|segment| segment.contains(word))
        })
        && path.split('/').all(|segment| segment.len() <= 64)
}

pub async fn record(path: &str) -> Result<(), FrameworkError> {
    if !reportable(path) {
        return Ok(());
    }
    let now = chrono::Utc::now().timestamp();
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    tx.execute_unprepared("UPDATE seo_settings SET id = id WHERE id = 1")
        .await
        .map_err(database_error)?;
    entity::Entity::delete_many()
        .filter(entity::Column::LastSeen.lt(now - RETENTION))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    // A capped count and a bounded eviction keep unsolicited paths from growing storage without limit.
    if entity::Entity::find_by_id(path)
        .one(&tx)
        .await
        .map_err(database_error)?
        .is_none()
        && entity::Entity::find()
            .count(&tx)
            .await
            .map_err(database_error)?
            >= MAX_ROWS
    {
        let oldest = entity::Entity::find()
            .order_by_asc(entity::Column::LastSeen)
            .order_by_asc(entity::Column::Path)
            .one(&tx)
            .await
            .map_err(database_error)?;
        if let Some(row) = oldest {
            entity::Entity::delete_by_id(row.path)
                .exec(&tx)
                .await
                .map_err(database_error)?;
        }
    }
    tx.execute_raw(Statement::from_sql_and_values(tx.get_database_backend(),
        "INSERT INTO seo_not_found (path, hits, first_seen, last_seen) VALUES (?, 1, ?, ?) ON CONFLICT(path) DO UPDATE SET hits = MIN(seo_not_found.hits + 1, 2147483647), last_seen = excluded.last_seen",
        [path.into(), now.into(), now.into()])).await.map_err(database_error)?;
    tx.commit().await.map_err(database_error)
}

pub async fn report(
    actor: i64,
    page: Page,
) -> Result<(Vec<entity::Model>, Pagination), FrameworkError> {
    crate::articles::require_permission(actor, super::MANAGE_PERMISSION).await?;
    let db = DB::connection()?;
    let query = entity::Entity::find()
        .filter(entity::Column::LastSeen.gte(chrono::Utc::now().timestamp() - RETENTION));
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let rows = query
        .order_by_desc(entity::Column::LastSeen)
        .order_by_asc(entity::Column::Path)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    Ok((
        rows,
        Pagination {
            page: page.number,
            per_page: page.size,
            total,
        },
    ))
}
