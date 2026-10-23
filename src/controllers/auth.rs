//! Authentication controller.
//!
//! Renders the login/register Inertia pages on GET, validates and
//! persists credentials on POST and continues to the intended path or `/dashboard`.
//! Form bodies are extracted via `FormRequest`. On an Inertia visit a
//! per-field failure comes back as a `303` to the form page with the
//! errors flashed, so `useForm().errors` fills in; a plain REST client
//! still gets the `422` `{ message, errors }` envelope.

use std::sync::Arc;

use serde::Deserialize;
use suprnova::{
    Auth, Credentials, FormRequest, FrameworkError, Inertia, InertiaProps, Redirect, Request,
    Response, Validate, ValidationErrors, handler, inertia_response, redirect,
};

use crate::models::user::User;

// ============================================================================
// Login
// ============================================================================

#[derive(InertiaProps)]
pub struct LoginProps {}

#[handler]
pub async fn show_login(req: Request) -> Response {
    crate::middleware::authenticate::remember_return_to(&req);
    inertia_response!(&req, "auth/Login", LoginProps {})
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(
        email(message = "Please enter a valid email address"),
        length(max = 254, message = "Email must be at most 254 characters")
    )]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

impl FormRequest for LoginRequest {
    fn prepare_for_validation(req: &mut Request) -> Result<(), FrameworkError> {
        req.transform_input("email", |email| crate::email_address::normalize(&email));
        Ok(())
    }
}

/// Build a `FrameworkError::Validation` that pins the failure to the
/// `email` field, mirroring how the bundled validators surface errors.
fn invalid_credentials() -> suprnova::FrameworkError {
    let mut errs = ValidationErrors::new();
    errs.add("email", "These credentials do not match our records.");
    suprnova::FrameworkError::Validation(errs)
}

#[handler]
pub async fn login(form: LoginRequest) -> Response {
    // `Auth::attempt` verifies the password through the registered user
    // provider, logs the user into the session on success, and issues a
    // remember-me token when requested - all via the named-guard system
    // wired in bootstrap.rs.
    match Auth::attempt(
        &Credentials::password(&form.email, &form.password),
        form.remember,
    )
    .await?
    {
        Some(_user) => {
            if Redirect::intended_url().is_some() {
                return Ok(Inertia::location(Redirect::intended("/dashboard")));
            }
            redirect!("/dashboard").into()
        }
        None => Err(invalid_credentials().into()),
    }
}

// ============================================================================
// Registration
// ============================================================================

#[derive(InertiaProps)]
pub struct RegisterProps {}

#[handler]
pub async fn show_register(req: Request) -> Response {
    inertia_response!(&req, "auth/Register", RegisterProps {})
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 255, message = "Name must be 2 to 255 characters"))]
    pub name: String,
    #[validate(
        email(message = "Please enter a valid email address"),
        length(max = 254, message = "Email must be at most 254 characters")
    )]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub password_confirmation: String,
}

impl FormRequest for RegisterRequest {
    fn prepare_for_validation(req: &mut Request) -> Result<(), FrameworkError> {
        req.transform_input("email", |email| crate::email_address::normalize(&email));
        Ok(())
    }

    /// Cross-field check: confirm the password and its confirmation
    /// match. Runs after the per-field rules pass, so we know each
    /// individual value is well-formed before comparing them.
    fn after_validation(&self) -> Result<(), ValidationErrors> {
        if self.password != self.password_confirmation {
            let mut errs = ValidationErrors::new();
            errs.add("password_confirmation", "Passwords do not match.");
            return Err(errs);
        }
        Ok(())
    }
}

#[handler]
pub async fn register(form: RegisterRequest) -> Response {
    if User::find_by_email(&form.email).await?.is_some() {
        let mut errs = ValidationErrors::new();
        errs.add("email", "This email is already registered.");
        return Err(suprnova::FrameworkError::Validation(errs).into());
    }

    let user = User::create(&form.name, &form.email, &form.password).await?;
    suprnova::auth_flows::EmailVerification::send_link(
        &user,
        &super::account_links::mail_url("/verify-email/verify")?,
    )
    .await?;
    // Log the freshly-created user into the session (fires the Login event).
    Auth::login(Arc::new(user), false).await?;

    if Redirect::intended_url().is_some() {
        return Ok(Inertia::location(Redirect::intended("/dashboard")));
    }
    redirect!("/dashboard").into()
}

// ============================================================================
// Logout
// ============================================================================

#[handler]
pub async fn logout(_req: Request) -> Response {
    Auth::logout().await?;
    redirect!("/").into()
}
