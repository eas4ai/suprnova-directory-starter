pub mod entities;
pub mod queries;
pub mod taxonomy;
pub mod validation;
pub mod workflow;

use suprnova::FrameworkError;
pub const EDIT_PERMISSION: &str = "articles.manage";
pub const TAXONOMY_PERMISSION: &str = "taxonomy.manage";

pub async fn require_permission(actor: i64, capability: &str) -> Result<(), FrameworkError> {
    crate::listings::workflow::require_verified(actor).await?;
    for permission in [crate::billing::ADMIN_PERMISSION, capability] {
        if !suprnova::rbac::has_permission_for_model(
            "directory.user",
            &actor.to_string(),
            permission,
        )
        .await?
        {
            return Err(suprnova::AppError::forbidden(
                "This administrative permission is required.",
            )
            .into());
        }
    }
    Ok(())
}
