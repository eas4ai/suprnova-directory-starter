//! Account-access checks around Suprnova's model-backed authentication.

use std::sync::Arc;

use async_trait::async_trait;
use suprnova::{
    EloquentUserProvider, FrameworkError,
    auth::{AuthFlowUser, Authenticatable, UserProvider},
    serde_json::Value,
};

use crate::models::user::User;

#[derive(Default)]
pub struct AccountUserProvider {
    inner: EloquentUserProvider<User>,
}

impl AccountUserProvider {
    pub fn new() -> Self {
        Self::default()
    }

    async fn active_user(
        user: Option<Arc<dyn Authenticatable>>,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        let Some(user) = user else {
            return Ok(None);
        };
        let id = user.get_auth_identifier().parse::<i64>().map_err(|_| {
            FrameworkError::internal("account provider returned an invalid user identifier")
        })?;
        if super::is_active(id).await? {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl UserProvider for AccountUserProvider {
    async fn retrieve_by_id(
        &self,
        id: &str,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        Self::active_user(self.inner.retrieve_by_id(id).await?).await
    }

    async fn retrieve_by_credentials(
        &self,
        credentials: &Value,
    ) -> Result<Option<Arc<dyn Authenticatable>>, FrameworkError> {
        Self::active_user(self.inner.retrieve_by_credentials(credentials).await?).await
    }

    async fn validate_credentials(
        &self,
        user: &dyn Authenticatable,
        credentials: &Value,
    ) -> Result<bool, FrameworkError> {
        self.inner.validate_credentials(user, credentials).await
    }

    async fn dummy_verify(&self) -> Result<bool, FrameworkError> {
        self.inner.dummy_verify().await
    }

    // Suspension limits sign-in and protected actions. Recovery and verification
    // keep their framework-owned token and verified-email requirements.
    async fn retrieve_by_email(&self, email: &str) -> Result<Option<AuthFlowUser>, FrameworkError> {
        self.inner.retrieve_by_email(email).await
    }

    fn supports_password_reset(&self) -> bool {
        self.inner.supports_password_reset()
    }

    async fn retrieve_verified_user_for_password_reset(
        &self,
        email: &str,
    ) -> Result<Option<AuthFlowUser>, FrameworkError> {
        self.inner
            .retrieve_verified_user_for_password_reset(email)
            .await
    }

    async fn flow_user_by_id(&self, id: &str) -> Result<Option<AuthFlowUser>, FrameworkError> {
        self.inner.flow_user_by_id(id).await
    }

    async fn mark_email_verified(&self, id: &str) -> Result<(), FrameworkError> {
        self.inner.mark_email_verified(id).await
    }

    async fn set_password(&self, id: &str, hashed: &str) -> Result<(), FrameworkError> {
        self.inner.set_password(id, hashed).await
    }

    async fn is_email_verified(&self, id: &str) -> Result<bool, FrameworkError> {
        self.inner.is_email_verified(id).await
    }
}
