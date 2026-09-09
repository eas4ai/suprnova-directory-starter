use suprnova::{StaticFiles, fallback, get, group, post, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // Public routes
    get!("/", controllers::home::index),
    get!("/listings", controllers::listings::index),
    get!("/listings/{slug}", controllers::listings::show),
    get!("/media/listings/{slug}/{media_id}", controllers::listings::public_media),
    get!("/articles", controllers::articles::index),
    get!("/articles/{slug}", controllers::articles::show),
    get!("/media/articles/{slug}/{media_id}", controllers::articles::public_media),
    get!("/feed.xml", controllers::feeds::rss),
    get!("/sitemap.xml", controllers::feeds::sitemap),
    get!("/sitemaps/{kind}/{page}", controllers::feeds::sitemap_page),
    get!("/robots.txt", controllers::feeds::robots),
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
        get!("/articles", controllers::articles::admin_index),
        get!("/articles/create", controllers::articles::create),
        post!("/articles", controllers::articles::store),
        post!("/articles/media", controllers::articles::upload),
        get!("/articles/media/{media_id}", controllers::articles::private_media),
        get!("/articles/{id}/edit", controllers::articles::edit),
        get!("/articles/{id}/preview", controllers::articles::preview),
        post!("/articles/{id}", controllers::articles::update),
        post!("/articles/{id}/publish", controllers::articles::publish),
        post!("/articles/{id}/unpublish", controllers::articles::unpublish),
        get!("/taxonomy", controllers::articles::taxonomy_index),
        post!("/taxonomy/{kind}", controllers::articles::taxonomy_store),
        post!("/taxonomy/{kind}/{id}", controllers::articles::taxonomy_update),
        post!("/taxonomy/{kind}/{id}/remove", controllers::articles::taxonomy_remove),
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
