//! Mailbox verification and account recovery through Suprnova.

use crate::models::user::User;
use serde::Deserialize;
use suprnova::{
    Auth, FrameworkError, InertiaProps, MustVerifyEmail, Request, Response,
    auth_flows::EmailVerification, handler, inertia_response, redirect,
};
use suprnova::{FormRequest, Validate, ValidationErrors, auth_flows::PasswordReset};

pub fn mail_url(path: &str) -> Result<String, FrameworkError> {
    let base = std::env::var("APP_URL")
        .map_err(|_| FrameworkError::internal("APP_URL is required for account mail"))?;
    Ok(format!("{}{path}", base.trim_end_matches('/')))
}

#[derive(InertiaProps)]
pub struct VerificationProps {
    pub email: String,
}

#[handler]
pub async fn notice(req: Request) -> Response {
    let user = Auth::user_as::<User>()
        .await?
        .ok_or(FrameworkError::Unauthorized)?;
    if user.is_email_verified() {
        return redirect!("/dashboard").into();
    }
    inertia_response!(
        &req,
        "auth/VerifyEmail",
        VerificationProps { email: user.email }
    )
}

#[handler]
pub async fn resend(_req: Request) -> Response {
    let user = Auth::user_as::<User>()
        .await?
        .ok_or(FrameworkError::Unauthorized)?;
    if !user.is_email_verified() {
        EmailVerification::send_link(&user, &mail_url("/verify-email/verify")?).await?;
    }
    redirect!("/verify-email").into()
}

#[handler]
pub async fn verify(req: Request) -> Response {
    EmailVerification::verify(&req.query_param("token").unwrap_or_default()).await?;
    redirect!("/dashboard").into()
}

#[derive(InertiaProps)]
pub struct ForgotPasswordProps {}

#[derive(InertiaProps)]
pub struct ResetPasswordProps {
    pub token: String,
}

#[derive(Deserialize, Validate)]
pub struct SendResetRequest {
    #[validate(email(message = "Enter a valid email address."))]
    pub email: String,
}
impl FormRequest for SendResetRequest {}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters."))]
    pub password: String,
    pub password_confirmation: String,
}

impl FormRequest for ResetPasswordRequest {
    fn after_validation(&self) -> Result<(), ValidationErrors> {
        if self.password != self.password_confirmation {
            let mut errors = ValidationErrors::new();
            errors.add("password_confirmation", "Passwords do not match.");
            return Err(errors);
        }
        Ok(())
    }
}

#[handler]
pub async fn forgot(req: Request) -> Response {
    inertia_response!(&req, "auth/ForgotPassword", ForgotPasswordProps {})
}

#[handler]
pub async fn send_reset(form: SendResetRequest) -> Response {
    PasswordReset::send_link(&form.email, &mail_url("/reset-password")?).await?;
    redirect!("/forgot-password").into()
}

#[handler]
pub async fn reset_form(req: Request) -> Response {
    inertia_response!(
        &req,
        "auth/ResetPassword",
        ResetPasswordProps {
            token: req.query_param("token").unwrap_or_default(),
        }
    )
}

#[handler]
pub async fn reset_password(form: ResetPasswordRequest) -> Response {
    let outcome = PasswordReset::complete_with_outcome(&form.token, &form.password)
        .await
        .map_err(|error| {
            if error.status_code() == 400 {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "token",
                    "This reset link is invalid or expired. Request another link.",
                );
                FrameworkError::Validation(errors)
            } else {
                error
            }
        })?;
    // A changed password is not sufficient if old credentials still work.
    outcome.sessions_revoked?;
    outcome.remember_tokens_revoked?;
    redirect!("/login").into()
}
