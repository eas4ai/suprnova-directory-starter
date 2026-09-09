use async_trait::async_trait;
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set, TransactionTrait,
    sea_query::OnConflict,
};
use suprnova::{Command, DB, FrameworkError, TypedCommand, rbac::entity::*};

use crate::{
    billing::{ADMIN_PERMISSION, BILLING_PERMISSION},
    models::user,
};

#[derive(clap::Parser, Command)]
#[console(
    name = "admin:access",
    description = "Grant or revoke starter administrative access for a verified account"
)]
pub struct AdminAccess {
    #[arg(value_enum)]
    pub action: AccessAction,
    /// Exact numeric account ID. No email matching or account creation.
    #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
    pub user_id: i64,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum AccessAction {
    Grant,
    Revoke,
}

#[async_trait]
impl TypedCommand for AdminAccess {
    async fn run(self) -> Result<(), FrameworkError> {
        change_access(self.user_id, self.action).await?;
        let action = match self.action {
            AccessAction::Grant => "granted",
            AccessAction::Revoke => "revoked",
        };
        println!(
            "Administrative access {action} for account {}.",
            self.user_id
        );
        Ok(())
    }
}

/// Use the framework-owned RBAC entities so the permission bundle commits as one
/// unit. Checks read the same tables on every request, including existing sessions.
pub async fn change_access(user_id: i64, action: AccessAction) -> Result<(), FrameworkError> {
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    let account = user::Entity::find_by_id(user_id)
        .one(&transaction)
        .await
        .map_err(database_error)?
        .ok_or_else(|| {
            FrameworkError::bad_request(
                "No account has that ID. Create and verify the account first.",
            )
        })?;
    if account.email_verified_at.is_none() {
        return Err(FrameworkError::bad_request(
            "The account must verify its email before administrative access can change.",
        ));
    }
    let mut permissions = Vec::with_capacity(2);
    for name in [ADMIN_PERMISSION, BILLING_PERMISSION] {
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
        .exec(&transaction)
        .await
        .map_err(database_error)?;
        let permission = PermissionEntity::find()
            .filter(PermissionColumn::Name.eq(name))
            .filter(PermissionColumn::GuardName.eq("web"))
            .one(&transaction)
            .await
            .map_err(database_error)?
            .ok_or_else(|| {
                FrameworkError::internal("The administrative permission could not be created.")
            })?;
        permissions.push(permission.id);
    }
    let model_id = user_id.to_string();
    match action {
        AccessAction::Grant => {
            for permission_id in permissions {
                ModelPermissionEntity::insert(ModelPermissionActiveModel {
                    model_type: Set("directory.user".to_owned()),
                    model_id: Set(model_id.clone()),
                    permission_id: Set(permission_id),
                    ..Default::default()
                })
                .on_conflict(
                    OnConflict::columns([
                        ModelPermissionColumn::ModelType,
                        ModelPermissionColumn::ModelId,
                        ModelPermissionColumn::PermissionId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .try_insert()
                .exec(&transaction)
                .await
                .map_err(database_error)?;
            }
        }
        AccessAction::Revoke => {
            ModelPermissionEntity::delete_many()
                .filter(ModelPermissionColumn::ModelType.eq("directory.user"))
                .filter(ModelPermissionColumn::ModelId.eq(&model_id))
                .filter(ModelPermissionColumn::PermissionId.is_in(permissions.clone()))
                .exec(&transaction)
                .await
                .map_err(database_error)?;
            // A direct revocation must not leave the same permission inherited from
            // a role. Remove only memberships that confer this administrative bundle.
            let roles: Vec<i64> = RolePermissionEntity::find()
                .select_only()
                .column(RolePermissionColumn::RoleId)
                .filter(RolePermissionColumn::PermissionId.is_in(permissions))
                .into_tuple()
                .all(&transaction)
                .await
                .map_err(database_error)?;
            if !roles.is_empty() {
                ModelRoleEntity::delete_many()
                    .filter(ModelRoleColumn::ModelType.eq("directory.user"))
                    .filter(ModelRoleColumn::ModelId.eq(&model_id))
                    .filter(ModelRoleColumn::RoleId.is_in(roles))
                    .exec(&transaction)
                    .await
                    .map_err(database_error)?;
            }
        }
    }
    transaction.commit().await.map_err(database_error)
}

fn database_error(error: sea_orm::DbErr) -> FrameworkError {
    tracing::error!(error = %error, "Administrative access transaction failed");
    FrameworkError::database(
        "Administrative access could not be changed; the transaction was not completed.",
    )
}
