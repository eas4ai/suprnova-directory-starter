use suprnova::{
    Auth, FrameworkError, InertiaProps, Request, Response, handler, inertia_response,
    rbac::HasRoles,
    redirect,
    serde_json::{Value, to_value},
};

use crate::{
    billing::{self, BillingError, Mode, SaveSettings},
    models::user::User,
};

#[derive(InertiaProps)]
pub struct BillingProps {
    pub mode: String,
    pub settings: Option<Value>,
    pub configuration_error: Option<String>,
}

async fn authorize() -> Result<(), FrameworkError> {
    let user = Auth::user_as::<User>()
        .await?
        .ok_or(FrameworkError::Unauthorized)?;
    if !user.has_permission_to(billing::ADMIN_PERMISSION).await?
        || !user.has_permission_to(billing::BILLING_PERMISSION).await?
    {
        return Err(
            suprnova::AppError::forbidden("Billing configuration permission is required.").into(),
        );
    }
    Ok(())
}

#[handler]
pub async fn index(req: Request) -> Response {
    authorize().await?;
    let mode = Mode::parse(&req.query_param("mode").unwrap_or_else(|| "test".to_owned()))
        .map_err(BillingError::into_framework)?;
    let (settings, configuration_error) = match billing::load(mode).await {
        Ok(settings) => (
            Some(
                to_value(settings)
                    .map_err(|_| FrameworkError::internal("Could not display billing settings."))?,
            ),
            None,
        ),
        Err(BillingError::Unavailable(message)) => (None, Some(message.to_owned())),
        Err(error) => return Err(error.into_framework().into()),
    };
    inertia_response!(
        &req,
        "admin/Billing",
        BillingProps {
            mode: mode.as_str().to_owned(),
            settings,
            configuration_error,
        }
    )
}

#[handler]
pub async fn update(req: Request) -> Response {
    authorize().await?;
    let mode = Mode::parse(&req.query_param("mode").unwrap_or_else(|| "test".to_owned()))
        .map_err(BillingError::into_framework)?;
    // Never return the deserializer's message: unexpected string values may be secrets.
    let input: SaveSettings = req.json().await.map_err(|_| {
        billing::invalid(
            "configuration",
            "The submitted configuration is invalid. Reload the page and try again.",
        )
    })?;
    billing::save(mode, input)
        .await
        .map_err(BillingError::into_framework)?;
    match mode {
        Mode::Test => redirect!("/admin/billing?mode=test").into(),
        Mode::Live => redirect!("/admin/billing?mode=live").into(),
    }
}
