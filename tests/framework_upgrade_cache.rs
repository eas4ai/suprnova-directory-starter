mod common;

use common::Client;
use suprnova::{HttpResponse, serde_json::json};

#[tokio::test]
async fn public_responses_omit_cookies_and_session_writes_become_private() {
    let _mail = common::setup().await;
    let router = directory::routes::register()
        .get("/__cache_fixture", |_| async {
            Ok(HttpResponse::text("shared fixture").header("Cache-Control", "public, max-age=60"))
        })
        .get("/__cache_write_fixture", |_| async {
            suprnova::session::session_mut(|session| {
                session.put("private_fixture", "visitor-specific")
            });
            Ok(HttpResponse::text("changed session").header("Cache-Control", "public, max-age=60"))
        })
        .into();
    let mut client = Client::with_router(router);
    let public = client.get("/__cache_fixture").await;
    assert_eq!(public.status, 200);
    assert_eq!(public.headers["cache-control"], "public, max-age=60");
    assert_eq!(public.headers.get_all("set-cookie").iter().count(), 0);

    directory::models::user::User::create(
        "Cache fixture",
        "cache@example.test",
        "fixture-password-123",
    )
    .await
    .unwrap();
    client.get("/login").await;
    assert_eq!(
        client
            .post(
                "/login",
                json!({"email": "cache@example.test", "password": "fixture-password-123"})
            )
            .await
            .status,
        302
    );
    let public = client.get("/__cache_fixture").await;
    assert_eq!(public.status, 200);
    assert_eq!(public.headers["cache-control"], "public, max-age=60");
    assert_eq!(public.headers.get_all("set-cookie").iter().count(), 0);
    let changed = client.get("/__cache_write_fixture").await;
    assert_eq!(changed.status, 200);
    let cache = changed.headers["cache-control"].to_str().unwrap();
    assert!(
        cache.contains("private") || cache.contains("no-store"),
        "{cache}"
    );
    assert!(!cache.contains("public"), "{cache}");
    assert!(changed.headers.get_all("set-cookie").iter().count() > 0);
}
