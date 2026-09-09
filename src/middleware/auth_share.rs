use crate::models::user::User;
use suprnova::{
    Auth, FrameworkError, InertiaRequestExt, InertiaSharedData, Prop, indexmap::IndexMap,
    rbac::HasRoles, serde_json::json,
};

pub struct AuthShare;

#[async_trait::async_trait]
impl InertiaSharedData for AuthShare {
    async fn share(
        &self,
        _req: &dyn InertiaRequestExt,
        _component: &str,
    ) -> Result<IndexMap<String, Prop>, FrameworkError> {
        let user = match Auth::user_as::<User>().await? {
            Some(user) => {
                let can_admin = user.has_permission_to("admin.access").await?;
                let can_billing = can_admin
                    && user
                        .has_permission_to(crate::billing::BILLING_PERMISSION)
                        .await?;
                let can_moderate = can_admin
                    && user
                        .has_permission_to(crate::listings::MODERATE_PERMISSION)
                        .await?;
                let can_edit = can_admin
                    && user
                        .has_permission_to(crate::articles::EDIT_PERMISSION)
                        .await?;
                let can_taxonomy = can_admin
                    && user
                        .has_permission_to(crate::articles::TAXONOMY_PERMISSION)
                        .await?;
                json!({"name": user.name, "email": user.email,
                    "verified": user.email_verified_at.is_some(), "can_admin": can_admin, "can_billing": can_billing, "can_moderate": can_moderate,
                    "can_edit": can_edit, "can_taxonomy": can_taxonomy})
            }
            None => suprnova::serde_json::Value::Null,
        };
        Ok(IndexMap::from([
            ("auth".to_owned(), Prop::eager(json!({"user": user}))),
            (
                "site".to_owned(),
                Prop::eager(
                    suprnova::serde_json::to_value(crate::config::site::read()?).map_err(|_| {
                        FrameworkError::internal("Could not share site configuration.")
                    })?,
                ),
            ),
        ]))
    }
}
