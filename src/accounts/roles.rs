//! Explicit permissions, using only the framework-owned RBAC schema.
use crate::commands::admin_access::ADMIN_PERMISSIONS;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set, sea_query::OnConflict,
};
use suprnova::rbac::entity::*;
pub const PREDEFINED: [&str; 3] = ["administrator", "moderator", "editor"];
pub async fn seed<C: ConnectionTrait>(db: &C) -> Result<Vec<i64>, sea_orm::DbErr> {
    let mut permissions = Vec::with_capacity(ADMIN_PERMISSIONS.len());
    for name in ADMIN_PERMISSIONS {
        let now = chrono::Utc::now().to_rfc3339();
        PermissionEntity::insert(PermissionActiveModel {
            name: Set(name.to_owned()),
            display_name: Set(Some(name.to_owned())),
            guard_name: Set("web".to_owned()),
            created_at: Set(now.clone()),
            updated_at: Set(now),
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
        permissions.push(permission.id);
    }

    for name in PREDEFINED {
        let now = chrono::Utc::now().to_rfc3339();
        RoleEntity::insert(RoleActiveModel {
            name: Set(name.into()),
            display_name: Set(Some(name.into())),
            guard_name: Set("web".into()),
            created_at: Set(now.clone()),
            updated_at: Set(now),
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
                RolePermissionEntity::insert(RolePermissionActiveModel {
                    role_id: Set(role.id),
                    permission_id: Set(*permission_id),
                    ..Default::default()
                })
                .on_conflict(
                    OnConflict::columns([
                        RolePermissionColumn::RoleId,
                        RolePermissionColumn::PermissionId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .try_insert()
                .exec(db)
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
    let model_id = id.to_string();
    ModelPermissionEntity::delete_many()
        .filter(ModelPermissionColumn::ModelType.eq("directory.user"))
        .filter(ModelPermissionColumn::ModelId.eq(&model_id))
        .filter(ModelPermissionColumn::PermissionId.is_in(permissions.to_vec()))
        .exec(db)
        .await?;
    let role_ids: Vec<i64> = RolePermissionEntity::find()
        .select_only()
        .column(RolePermissionColumn::RoleId)
        .filter(RolePermissionColumn::PermissionId.is_in(permissions.to_vec()))
        .into_tuple()
        .all(db)
        .await?;
    if !role_ids.is_empty() {
        ModelRoleEntity::delete_many()
            .filter(ModelRoleColumn::ModelType.eq("directory.user"))
            .filter(ModelRoleColumn::ModelId.eq(&model_id))
            .filter(ModelRoleColumn::RoleId.is_in(role_ids))
            .exec(db)
            .await?;
    }
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
        ModelRoleEntity::insert(ModelRoleActiveModel {
            model_type: Set("directory.user".into()),
            model_id: Set(id.to_string()),
            role_id: Set(role.id),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([
                ModelRoleColumn::ModelType,
                ModelRoleColumn::ModelId,
                ModelRoleColumn::RoleId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .try_insert()
        .exec(db)
        .await?;
    }
    Ok(())
}
