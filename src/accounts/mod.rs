pub mod entity;
pub mod provider;
pub mod queries;
pub mod roles;

use crate::listings::{database_error, invalid, missing};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, Set, TransactionTrait,
};
use serde::Deserialize;
use suprnova::{DB, FrameworkError};
pub const MANAGE_PERMISSION: &str = "accounts.manage";
pub const AUDIT_PERMISSION: &str = "audit.view";

pub async fn active_on<C: ConnectionTrait>(db: &C, id: i64) -> Result<bool, FrameworkError> {
    Ok(entity::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(database_error)?
        .is_none_or(|a| !a.suspended))
}
pub async fn is_active(id: i64) -> Result<bool, FrameworkError> {
    active_on(DB::connection()?.inner(), id).await
}
pub async fn require_active(id: i64) -> Result<(), FrameworkError> {
    if !is_active(id).await? {
        return Err(suprnova::AppError::forbidden("This account is suspended.").into());
    }
    Ok(())
}
/// Serialize human access changes before reading state, including the last-admin check.
pub async fn lock(tx: &DatabaseTransaction) -> Result<(), FrameworkError> {
    tx.execute_unprepared("UPDATE account_write_lock SET version = version + 1 WHERE id = 1")
        .await
        .map_err(database_error)?;
    Ok(())
}
/// Called before other writes: suspension and protected mutations share this lock.
pub async fn guard_mutation(tx: &DatabaseTransaction, id: i64) -> Result<(), FrameworkError> {
    lock(tx).await?;
    if !active_on(tx, id).await? {
        return Err(suprnova::AppError::forbidden("This account is suspended.").into());
    }
    Ok(())
}
/// Authorization under the same lock as role edits prevents stale capability checks.
pub async fn guard_permission(
    tx: &DatabaseTransaction,
    id: i64,
    capability: &str,
) -> Result<(), FrameworkError> {
    guard_mutation(tx, id).await?;
    if !queries::has_permission_on(tx, id, "admin.access").await?
        || !queries::has_permission_on(tx, id, capability).await?
    {
        return Err(
            suprnova::AppError::forbidden("This administrative permission is required.").into(),
        );
    }
    Ok(())
}
pub async fn write_state(
    tx: &DatabaseTransaction,
    id: i64,
    suspended: bool,
) -> Result<(), FrameworkError> {
    let row = entity::Entity::find_by_id(id)
        .one(tx)
        .await
        .map_err(database_error)?;
    let version = row
        .as_ref()
        .map_or(0, |r| r.version)
        .checked_add(1)
        .ok_or_else(|| invalid("version", "Account version is exhausted."))?;
    let state = entity::ActiveModel {
        user_id: Set(id),
        version: Set(version),
        suspended: Set(suspended),
    };
    if row.is_some() {
        state.update(tx).await.map_err(database_error)?;
    } else {
        state.insert(tx).await.map_err(database_error)?;
    }
    Ok(())
}
pub async fn record_access(
    tx: &DatabaseTransaction,
    actor: Option<i64>,
    id: i64,
    action: &str,
    summary: &str,
) -> Result<(), FrameworkError> {
    crate::audit::record(
        tx,
        actor
            .map(crate::audit::Actor::User)
            .unwrap_or(crate::audit::Actor::Operator),
        "account",
        id.to_string(),
        action,
        summary,
    )
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveAccess {
    pub version: i64,
    pub roles: Vec<String>,
    pub suspended: bool,
}
pub async fn save(actor: i64, id: i64, mut input: SaveAccess) -> Result<(), FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    if input.version < 0
        || input.roles.len() > 3
        || input
            .roles
            .iter()
            .any(|r| !roles::PREDEFINED.contains(&r.as_str()))
    {
        return Err(invalid(
            "roles",
            "Choose only the predefined starter roles.",
        ));
    }
    input.roles.sort();
    input.roles.dedup();
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    guard_mutation(&tx, actor).await?;
    // Recheck authorization after the access lock; another admin may have revoked it.
    if !queries::has_permission_on(&tx, actor, MANAGE_PERMISSION).await?
        || !queries::has_permission_on(&tx, actor, "admin.access").await?
    {
        return Err(suprnova::AppError::forbidden(
            "Account administration permission is required.",
        )
        .into());
    }
    let account = crate::models::user::Entity::find_by_id(id)
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if !input.roles.is_empty() && account.email_verified_at.is_none() {
        return Err(invalid(
            "roles",
            "The account must verify its email before receiving a role.",
        ));
    }
    let state = entity::Entity::find_by_id(id)
        .one(&tx)
        .await
        .map_err(database_error)?;
    if state.map_or(0, |s| s.version) != input.version {
        return Err(invalid(
            "version",
            "This account changed. Reload before saving.",
        ));
    }
    let before = queries::administrator_count(&tx).await?;
    let permissions = roles::seed(&tx).await.map_err(database_error)?;
    roles::clear(&tx, id, &permissions)
        .await
        .map_err(database_error)?;
    roles::assign(&tx, id, &input.roles)
        .await
        .map_err(database_error)?;
    write_state(&tx, id, input.suspended).await?;
    if before > 0 && queries::administrator_count(&tx).await? == 0 {
        return Err(invalid(
            "roles",
            "Keep at least one active verified administrator. Use the host command for operator recovery.",
        ));
    }
    let summary = format!(
        "Suspended: {}; predefined roles: {}; verification unchanged.",
        input.suspended,
        if input.roles.is_empty() {
            "none".into()
        } else {
            input.roles.join(", ")
        }
    );
    record_access(&tx, Some(actor), id, "access_updated", &summary).await?;
    tx.commit().await.map_err(database_error)
}
