use async_trait::async_trait;
use sea_orm::{EntityTrait, TransactionTrait};
use suprnova::{Command, DB, FrameworkError, TypedCommand};

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
    crate::accounts::lock(&transaction).await?;
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
    let permissions = crate::accounts::roles::seed(&transaction)
        .await
        .map_err(database_error)?;
    crate::accounts::roles::clear(&transaction, user_id, &permissions)
        .await
        .map_err(database_error)?;
    match action {
        AccessAction::Grant => {
            crate::accounts::roles::assign(&transaction, user_id, &["administrator".into()])
                .await
                .map_err(database_error)?;
            crate::accounts::write_state(&transaction, user_id, false).await?;
        }
        AccessAction::Revoke => {
            let suspended = !crate::accounts::active_on(&transaction, user_id).await?;
            crate::accounts::write_state(&transaction, user_id, suspended).await?;
        }
    }
    crate::accounts::record_access(
        &transaction,
        None,
        user_id,
        match action {
            AccessAction::Grant => "operator_granted",
            AccessAction::Revoke => "operator_revoked",
        },
        match action {
            AccessAction::Grant => "Suspended: false; predefined roles: administrator; verification unchanged.",
            AccessAction::Revoke => "Starter administrative permissions and role memberships removed; suspension and verification unchanged.",
        },
    )
    .await?;
    transaction.commit().await.map_err(database_error)
}

pub const ADMIN_PERMISSIONS: [&str; 7] = [
    ADMIN_PERMISSION,
    BILLING_PERMISSION,
    crate::listings::MODERATE_PERMISSION,
    crate::articles::EDIT_PERMISSION,
    crate::articles::TAXONOMY_PERMISSION,
    crate::accounts::MANAGE_PERMISSION,
    crate::accounts::AUDIT_PERMISSION,
];

fn database_error(error: sea_orm::DbErr) -> FrameworkError {
    tracing::error!(error = %error, "Administrative access transaction failed");
    FrameworkError::database(
        "Administrative access could not be changed; the transaction was not completed.",
    )
}
