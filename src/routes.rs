use suprnova::{StaticFiles, fallback, get, group, post, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // Public routes
    get!("/", controllers::home::index),

    // Guest-only routes (redirect to dashboard if logged in)
    group!("/", {
        get!("/login", controllers::auth::show_login),
        post!("/login", controllers::auth::login),
        get!("/register", controllers::auth::show_register),
        post!("/register", controllers::auth::register),
        get!("/forgot-password", controllers::account_links::forgot),
        post!("/forgot-password", controllers::account_links::send_reset),
        get!("/reset-password", controllers::account_links::reset_form),
        post!("/reset-password", controllers::account_links::reset_password),
    }).middleware(middleware::authenticate::guest()),

    // Protected routes (require authentication)
    group!("/", {
        get!("/dashboard", controllers::dashboard::index),
        post!("/logout", controllers::auth::logout),
        get!("/verify-email", controllers::account_links::notice),
        post!("/email/verification-notification", controllers::account_links::resend),
        get!("/verify-email/verify", controllers::account_links::verify),
    }).middleware(middleware::authenticate::auth()),

    group!("/admin", {
        get!("/", controllers::admin::index),
    }).middleware(middleware::authenticate::auth())
      .middleware(suprnova::rbac::PermissionMiddleware::<crate::models::user::User>::redirect_to("admin.access", "/dashboard")),
    fallback!(StaticFiles::public().handler()),
}
