use super::{AUDIT_PERMISSION, MANAGE_PERMISSION, entity};
use crate::listings::{
    database_error, invalid, missing,
    queries::{Page, Pagination},
};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Statement, Value,
};
use serde::Serialize;
use std::collections::HashMap;
use suprnova::{DB, FrameworkError};

// This is the same explicit web-guard permission relationship used by Suprnova RBAC.
const EFFECTIVE: &str = "((EXISTS (SELECT 1 FROM model_permissions mp WHERE mp.permission_id = p.id AND mp.model_type = 'directory.user' AND mp.model_id = CAST(users.id AS TEXT))) OR (EXISTS (SELECT 1 FROM model_roles mr JOIN roles r ON r.id = mr.role_id JOIN role_permissions rp ON rp.role_id = r.id WHERE rp.permission_id = p.id AND r.guard_name = 'web' AND mr.model_type = 'directory.user' AND mr.model_id = CAST(users.id AS TEXT))))";
fn statement<C: ConnectionTrait>(db: &C, sql: String, values: Vec<Value>) -> Statement {
    // Supported install target is SQLite. Also bind native numbered placeholders on PostgreSQL.
    let sql = if db.get_database_backend() == sea_orm::DbBackend::Postgres {
        let mut index = 0;
        sql.chars()
            .map(|c| {
                if c == '?' {
                    index += 1;
                    format!("${index}")
                } else {
                    c.to_string()
                }
            })
            .collect()
    } else {
        sql
    };
    Statement::from_sql_and_values(db.get_database_backend(), sql, values)
}
pub async fn has_permission_on<C: ConnectionTrait>(
    db: &C,
    id: i64,
    permission: &str,
) -> Result<bool, FrameworkError> {
    let sql = format!(
        "SELECT users.id FROM users WHERE users.id = ? AND EXISTS (SELECT 1 FROM permissions p WHERE p.name = ? AND p.guard_name = 'web' AND {EFFECTIVE})"
    );
    Ok(db
        .query_one_raw(statement(db, sql, vec![id.into(), permission.into()]))
        .await
        .map_err(database_error)?
        .is_some())
}
pub async fn administrator_count<C: ConnectionTrait>(db: &C) -> Result<i64, FrameworkError> {
    let names = crate::commands::admin_access::ADMIN_PERMISSIONS;
    let placeholders = vec!["?"; names.len()].join(",");
    let sql = format!(
        "SELECT COUNT(*) AS total FROM users WHERE email_verified_at IS NOT NULL AND NOT EXISTS (SELECT 1 FROM account_access a WHERE a.user_id = users.id AND a.suspended = TRUE) AND (SELECT COUNT(DISTINCT p.name) FROM permissions p WHERE p.guard_name = 'web' AND p.name IN ({placeholders}) AND {EFFECTIVE}) = ?"
    );
    let mut values: Vec<Value> = names.into_iter().map(Into::into).collect();
    values.push((names.len() as i64).into());
    db.query_one_raw(statement(db, sql, values))
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?
        .try_get("", "total")
        .map_err(database_error)
}
#[derive(Serialize)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub verified: bool,
    pub suspended: bool,
    pub version: i64,
    pub roles: Vec<String>,
}
async fn rows(
    users: Vec<<crate::models::user::Entity as EntityTrait>::Model>,
) -> Result<Vec<Account>, FrameworkError> {
    let db = DB::connection()?;
    let ids: Vec<i64> = users.iter().map(|u| u.id).collect();
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let states: HashMap<_, _> = entity::Entity::find()
        .filter(entity::Column::UserId.is_in(ids.clone()))
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(|s| (s.user_id, s))
        .collect();
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT mr.model_id, r.name FROM model_roles mr JOIN roles r ON r.id = mr.role_id WHERE mr.model_type = 'directory.user' AND r.guard_name = 'web' AND r.name IN ('administrator', 'moderator', 'editor') AND mr.model_id IN ({placeholders}) ORDER BY r.name"
    );
    let role_rows = db
        .inner()
        .query_all_raw(statement(
            db.inner(),
            sql,
            ids.iter().map(|id| id.to_string().into()).collect(),
        ))
        .await
        .map_err(database_error)?;
    let mut roles: HashMap<i64, Vec<String>> = HashMap::new();
    for row in role_rows {
        let id: String = row.try_get("", "model_id").map_err(database_error)?;
        if let Ok(id) = id.parse::<i64>() {
            roles
                .entry(id)
                .or_default()
                .push(row.try_get("", "name").map_err(database_error)?);
        }
    }
    Ok(users
        .into_iter()
        .map(|u| {
            let state = states.get(&u.id);
            Account {
                id: u.id,
                name: u.name,
                email: u.email,
                verified: u.email_verified_at.is_some(),
                suspended: state.is_some_and(|s| s.suspended),
                version: state.map_or(0, |s| s.version),
                roles: roles.remove(&u.id).unwrap_or_default(),
            }
        })
        .collect())
}
pub async fn search(
    actor: i64,
    q: &str,
    page: Page,
) -> Result<(Vec<Account>, Pagination), FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    if q.chars().count() > 200 || q.contains('\0') {
        return Err(invalid("q", "Use at most 200 characters."));
    }
    let db = DB::connection()?;
    let mut query = crate::models::user::Entity::find();
    if !q.trim().is_empty() {
        let pattern = format!(
            "%{}%",
            q.trim()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        query = query.filter(
            Condition::any()
                .add(sea_orm::sea_query::Expr::cust_with_values(
                    "LOWER(name) LIKE LOWER(?) ESCAPE '\\'",
                    [pattern.clone()],
                ))
                .add(sea_orm::sea_query::Expr::cust_with_values(
                    "LOWER(email) LIKE LOWER(?) ESCAPE '\\'",
                    [pattern],
                )),
        );
    }
    let total = query
        .clone()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let users = query
        .order_by_asc(crate::models::user::Column::Id)
        .offset((page.number - 1) * page.size)
        .limit(page.size)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    Ok((
        rows(users).await?,
        Pagination {
            page: page.number,
            per_page: page.size,
            total,
        },
    ))
}
pub async fn detail(actor: i64, id: i64) -> Result<Account, FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    let user = crate::models::user::Entity::find_by_id(id)
        .one(DB::connection()?.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    rows(vec![user]).await?.pop().ok_or_else(missing)
}
#[derive(Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub actor: String,
    pub actor_id: i64,
    pub actor_type: String,
    pub target_type: String,
    pub target_id: String,
    pub action: String,
    pub summary: String,
    pub created_at: i64,
}
pub async fn audit(
    actor: i64,
    page: Page,
) -> Result<(Vec<AuditEntry>, Pagination), FrameworkError> {
    crate::articles::require_permission(actor, AUDIT_PERMISSION).await?;
    let db = DB::connection()?;
    let total = crate::listings::entities::audit::Entity::find()
        .count(db.inner())
        .await
        .map_err(database_error)?;
    let sql="SELECT a.*, u.name AS actor_name FROM administrative_audit a LEFT JOIN users u ON a.actor_type = 'user' AND u.id = a.actor_id ORDER BY a.id DESC LIMIT ? OFFSET ?".into();
    let raw = db
        .inner()
        .query_all_raw(statement(
            db.inner(),
            sql,
            vec![
                (page.size as i64).into(),
                (((page.number - 1) * page.size) as i64).into(),
            ],
        ))
        .await
        .map_err(database_error)?;
    let mut entries = Vec::with_capacity(raw.len());
    for row in raw {
        let actor_type: String = row.try_get("", "actor_type").map_err(database_error)?;
        let actor_name: Option<String> = row.try_get("", "actor_name").map_err(database_error)?;
        let target_type: String = row.try_get("", "target_type").map_err(database_error)?;
        // Earlier listing audits included owner-facing reasons. Preserve history in
        // storage while keeping that free text out of the shared audit output.
        let summary = if target_type == "listing" {
            "Listing moderation state and owner-facing reason saved.".into()
        } else {
            row.try_get("", "summary").map_err(database_error)?
        };
        entries.push(AuditEntry {
            id: row.try_get("", "id").map_err(database_error)?,
            actor: if actor_type == "operator" {
                "Host operator".into()
            } else {
                actor_name.unwrap_or_else(|| "Removed account".into())
            },
            actor_type,
            actor_id: row.try_get("", "actor_id").map_err(database_error)?,
            target_type,
            target_id: row.try_get("", "target_id").map_err(database_error)?,
            action: row.try_get("", "action").map_err(database_error)?,
            summary,
            created_at: row.try_get("", "created_at").map_err(database_error)?,
        });
    }
    Ok((
        entries,
        Pagination {
            page: page.number,
            per_page: page.size,
            total,
        },
    ))
}
