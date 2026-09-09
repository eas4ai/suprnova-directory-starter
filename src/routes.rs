use suprnova::{StaticFiles, fallback, get, group, post, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // Public routes
    get!("/", controllers::home::index),
    get!("/listings", controllers::listings::index),
    get!("/listings/{slug}", controllers::listings::show),
    get!("/media/listings/{slug}/{media_id}", controllers::listings::public_media),
    post!("/billing/webhooks/stripe/test", controllers::publishing::stripe_test),
    post!("/billing/webhooks/stripe/live", controllers::publishing::stripe_live),
    post!("/billing/webhooks/paddle/test", controllers::publishing::paddle_test),
    post!("/billing/webhooks/paddle/live", controllers::publishing::paddle_live),

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
        get!("/dashboard/listings", controllers::listings::owner_index),
        get!("/dashboard/listings/create", controllers::listings::create),
        post!("/dashboard/listings", controllers::listings::store),
        post!("/dashboard/listings/media", controllers::listings::upload),
        get!("/dashboard/listings/media/{media_id}", controllers::listings::private_media),
        get!("/dashboard/listings/{id}/edit", controllers::listings::edit),
        post!("/dashboard/listings/{id}", controllers::listings::update),
        post!("/dashboard/listings/{id}/submit", controllers::listings::submit),
        post!("/dashboard/listings/{id}/archive", controllers::listings::archive),
        get!("/dashboard/listings/{id}/plans", controllers::publishing::offers),
        post!("/dashboard/listings/{id}/checkout", controllers::publishing::start),
        get!("/dashboard/purchases/{id}", controllers::publishing::show),
        post!("/dashboard/purchases/{id}/continue", controllers::publishing::continue_purchase),
        post!("/dashboard/purchases/{id}/cancel", controllers::publishing::cancel),
        post!("/logout", controllers::auth::logout),
        get!("/verify-email", controllers::account_links::notice),
        post!("/email/verification-notification", controllers::account_links::resend),
        get!("/verify-email/verify", controllers::account_links::verify),
    }).middleware(middleware::authenticate::auth()),

    group!("/admin", {
        get!("/", controllers::admin::index),
        get!("/billing", controllers::billing::index),
        post!("/billing", controllers::billing::update),
        get!("/plans", controllers::publishing::plans_index),
        post!("/plans", controllers::publishing::plans_store),
        post!("/plans/{key}", controllers::publishing::plans_update),
        get!("/listings", controllers::listings::queue),
        get!("/listings/{id}", controllers::listings::review),
        post!("/listings/{id}/decision", controllers::listings::decide),
        post!("/listings/{id}/suspension", controllers::listings::suspension),
    }).middleware(middleware::authenticate::auth())
      .middleware(suprnova::rbac::PermissionMiddleware::<crate::models::user::User>::redirect_to("admin.access", "/dashboard")),
    fallback!(StaticFiles::public().handler()),
}
