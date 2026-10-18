//! Explicit permissions, using only the framework-owned RBAC schema.
use crate::commands::admin_access::ADMIN_PERMISSIONS;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set,
    sea_query::{Alias, Expr, ExprTrait, OnConflict, Query},
};
use suprnova::rbac::entity::*;
pub const PREDEFINED: [&str; 3] = ["administrator", "moderator", "editor"];
pub async fn seed<C: ConnectionTrait>(db: &C) -> Result<Vec<i64>, sea_orm::DbErr> {
    let mut permissions = Vec::with_capacity(ADMIN_PERMISSIONS.len());
    for name in ADMIN_PERMISSIONS {
        let now = chrono::Utc::now();
        PermissionEntity::insert(PermissionActiveModel {
            name: Set(name.to_owned()),
            guard_name: Set("web".to_owned()),
            created_at: Set(Some(now.into())),
            updated_at: Set(Some(now.into())),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([PermissionColumn::Name, PermissionColumn::GuardName])
                .do_nothing()
                .to_owned(),
        )
        .try_insert()
        .exec(db)
        .await?;
        let permission = PermissionEntity::find()
            .filter(PermissionColumn::Name.eq(name))
            .filter(PermissionColumn::GuardName.eq("web"))
            .one(db)
            .await?
            .ok_or_else(|| {
                sea_orm::DbErr::Custom("The administrative permission could not be created.".into())
            })?;
        permissions.push(i64::try_from(u64::from(permission.id)).map_err(|_| {
            sea_orm::DbErr::Custom(
                "Permission identifier exceeds the supported integer range".into(),
            )
        })?);
    }

    for name in PREDEFINED {
        let now = chrono::Utc::now();
        RoleEntity::insert(RoleActiveModel {
            name: Set(name.into()),
            guard_name: Set("web".into()),
            created_at: Set(Some(now.into())),
            updated_at: Set(Some(now.into())),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([RoleColumn::Name, RoleColumn::GuardName])
                .do_nothing()
                .to_owned(),
        )
        .try_insert()
        .exec(db)
        .await?;
        let role = RoleEntity::find()
            .filter(RoleColumn::Name.eq(name))
            .filter(RoleColumn::GuardName.eq("web"))
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom("Missing starter role".into()))?;
        let names: &[&str] = match name {
            "moderator" => &["admin.access", "listings.moderate"],
            "editor" => &["admin.access", "articles.manage"],
            _ => &ADMIN_PERMISSIONS,
        };
        for (permission_name, permission_id) in ADMIN_PERMISSIONS.iter().zip(permissions.iter()) {
            if names.contains(permission_name) {
                let role_id = i64::try_from(u64::from(role.id)).map_err(|_| {
                    sea_orm::DbErr::Custom(
                        "Role identifier exceeds the supported integer range".into(),
                    )
                })?;
                let insert = Query::insert()
                    .into_table(Alias::new("role_has_permissions"))
                    .columns([Alias::new("role_id"), Alias::new("permission_id")])
                    .values_panic([role_id.into(), (*permission_id).into()])
                    .on_conflict(
                        OnConflict::columns([Alias::new("role_id"), Alias::new("permission_id")])
                            .do_nothing()
                            .to_owned(),
                    )
                    .to_owned();
                db.execute_raw(db.get_database_backend().build(&insert))
                    .await?;
            }
        }
    }
    Ok(permissions)
}

pub async fn clear<C: ConnectionTrait>(
    db: &C,
    id: i64,
    permissions: &[i64],
) -> Result<(), sea_orm::DbErr> {
    let direct = Query::delete()
        .from_table(Alias::new("model_has_permissions"))
        .and_where(Expr::col(Alias::new("model_type")).eq("directory.user"))
        .and_where(Expr::col(Alias::new("model_id")).eq(id))
        .and_where(Expr::col(Alias::new("permission_id")).is_in(permissions.iter().copied()))
        .to_owned();
    db.execute_raw(db.get_database_backend().build(&direct))
        .await?;
    let roles = Query::select()
        .column(Alias::new("role_id"))
        .from(Alias::new("role_has_permissions"))
        .and_where(Expr::col(Alias::new("permission_id")).is_in(permissions.iter().copied()))
        .to_owned();
    let inherited = Query::delete()
        .from_table(Alias::new("model_has_roles"))
        .and_where(Expr::col(Alias::new("model_type")).eq("directory.user"))
        .and_where(Expr::col(Alias::new("model_id")).eq(id))
        .and_where(Expr::col(Alias::new("role_id")).in_subquery(roles))
        .to_owned();
    db.execute_raw(db.get_database_backend().build(&inherited))
        .await?;
    Ok(())
}

pub async fn assign<C: ConnectionTrait>(
    db: &C,
    id: i64,
    names: &[String],
) -> Result<(), sea_orm::DbErr> {
    for name in names {
        let role = RoleEntity::find()
            .filter(RoleColumn::Name.eq(name))
            .filter(RoleColumn::GuardName.eq("web"))
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom("Missing starter role".into()))?;
        let role_id = i64::try_from(u64::from(role.id)).map_err(|_| {
            sea_orm::DbErr::Custom("Role identifier exceeds the supported integer range".into())
        })?;
        let insert = Query::insert()
            .into_table(Alias::new("model_has_roles"))
            .columns([
                Alias::new("model_type"),
                Alias::new("model_id"),
                Alias::new("role_id"),
            ])
            .values_panic(["directory.user".into(), id.into(), role_id.into()])
            .on_conflict(
                OnConflict::columns([
                    Alias::new("model_type"),
                    Alias::new("model_id"),
                    Alias::new("role_id"),
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned();
        db.execute_raw(db.get_database_backend().build(&insert))
            .await?;
    }
    Ok(())
}
