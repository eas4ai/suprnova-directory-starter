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
                json!({"name": user.name, "email": user.email,
                    "verified": user.email_verified_at.is_some(), "can_admin": can_admin, "can_billing": can_billing, "can_moderate": can_moderate})
            }
            None => suprnova::serde_json::Value::Null,
        };
        Ok(IndexMap::from([(
            "auth".to_owned(),
            Prop::eager(json!({"user": user})),
        )]))
    }
}
