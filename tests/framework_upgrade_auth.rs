#[allow(dead_code)]
mod common;

use common::Client;
use directory::{accounts, accounts::provider::AccountUserProvider, models::user::User};
use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait, TransactionTrait};
use suprnova::{
    DB, LaravelDatabase, auth::UserProvider, eloquent::Model, hashing, serde_json::json,
};

const PASSWORD: &str = "upgrade-password-123";

async fn account(email: &str) -> User {
    let mut user = User::create("Upgrade account", email, PASSWORD)
        .await
        .unwrap();
    user.email_verified_at = Some(chrono::Utc::now());
    user.save().await.unwrap();
    user
}

async fn login(client: &mut Client, user: &User, password: &str) {
    assert_eq!(client.get("/login").await.status, 200);
    let response = client
        .post("/login", json!({"email":user.email,"password":password}))
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    assert_eq!(client.get("/dashboard").await.status, 200);
}

async fn required_rehash_is_forwarded() {
    let user = account("rehash@example.test").await;
    let provider = AccountUserProvider::new();
    let credentials = json!({"email":user.email,"password":PASSWORD});
    assert!(
        provider
            .validate_credentials(&user, &credentials)
            .await
            .unwrap()
    );
    // The published provider requires a rewrite in shared-database mode.
    // Set it only for this exercise, in this single-test process.
    LaravelDatabase::share(true);
    assert!(hashing::needs_rehash(&user.password));
    provider
        .rehash_password_if_required(&user, &credentials)
        .await
        .unwrap();
    let rewritten = User::find_by_email(&user.email).await.unwrap().unwrap();
    assert_ne!(
        rewritten.password, user.password,
        "wrapper suppressed the required rehash"
    );
    assert!(rewritten.password.starts_with("$2y$"));
    assert!(hashing::verify(PASSWORD, &rewritten.password).unwrap());
    assert!(!hashing::needs_rehash(&rewritten.password));
    LaravelDatabase::share(false);

    let blocked = account("rehash-storage-failure@example.test").await;
    let db = DB::connection().unwrap();
    db.inner().execute_unprepared("CREATE TRIGGER reject_password_update BEFORE UPDATE OF password ON users BEGIN SELECT RAISE(ABORT, 'fixture password storage failure'); END").await.unwrap();
    LaravelDatabase::share(true);
    let result = provider
        .rehash_password_if_required(&blocked, &json!({"password":PASSWORD}))
        .await;
    LaravelDatabase::share(false);
    db.inner()
        .execute_unprepared("DROP TRIGGER reject_password_update")
        .await
        .unwrap();
    assert!(result.is_err(), "rehash storage errors must propagate");
    assert_eq!(
        User::find_by_email(&blocked.email)
            .await
            .unwrap()
            .unwrap()
            .password,
        blocked.password
    );
}

async fn csrf_rotation_rejects_old_tokens() {
    let user = account("csrf@example.test").await;
    let mut client = Client::new();
    assert_eq!(client.get("/login").await.status, 200);
    let old = client.csrf_token();
    let response = client
        .post("/login", json!({"email":user.email,"password":PASSWORD}))
        .await;
    assert_eq!(response.status, 302);
    let current = client.csrf_token();
    assert_ne!(old, current, "sign-in must rotate the CSRF token");
    assert_eq!(
        client
            .post_with_csrf_token("/logout", json!({}), &old)
            .await
            .status,
        419
    );
    assert_eq!(
        client.get("/dashboard").await.status,
        200,
        "rejected token must not log out the user"
    );
    let response = client
        .post_with_csrf_token("/logout", json!({}), &current)
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    assert_eq!(response.location.as_deref(), Some("/"));
    assert_eq!(
        client.get("/dashboard").await.location.as_deref(),
        Some("/login")
    );
}

async fn password_change_rejects_existing_sessions() {
    let user = account("password-epoch@example.test").await;
    let mut first = Client::new();
    let mut second = Client::new();
    login(&mut first, &user, PASSWORD).await;
    login(&mut second, &user, PASSWORD).await;
    let next_password = "replacement-password-456";
    AccountUserProvider::new()
        .set_password(&user.id.to_string(), &hashing::hash(next_password).unwrap())
        .await
        .unwrap();
    for client in [&mut first, &mut second] {
        assert_eq!(
            client.get("/dashboard").await.location.as_deref(),
            Some("/login"),
            "password change must invalidate old sessions"
        );
    }
    let mut fresh = Client::new();
    login(&mut fresh, &user, next_password).await;
}

async fn suspension_blocks_existing_and_new_sign_ins() {
    let user = account("suspended@example.test").await;
    let mut existing = Client::new();
    login(&mut existing, &user, PASSWORD).await;
    let connection = DB::connection().unwrap();
    connection.inner().execute_unprepared("INSERT INTO listing_categories (id, slug, name, active) VALUES (987, 'upgrade', 'Upgrade', 1)").await.unwrap();
    let mut input = json!({"version":0,"title":"Before suspension","summary":"A valid directory resource",
        "description":"Preserve this listing", "url":"https://example.test/resource", "category_ids":[987],
        "media_id":null,"media_alt":""});
    let created = existing.post("/dashboard/listings", input.clone()).await;
    assert_eq!(created.status, 302, "{}", created.body);
    assert!(created.location.as_deref().unwrap().ends_with("/edit"));
    let before = directory::listings::entities::listing::Entity::find()
        .count(connection.inner())
        .await
        .unwrap();
    let tx = connection.inner().begin().await.unwrap();
    accounts::write_state(&tx, user.id, true).await.unwrap();
    tx.commit().await.unwrap();

    input["title"] = json!("After suspension");
    assert_eq!(
        existing
            .post("/dashboard/listings", input)
            .await
            .location
            .as_deref(),
        Some("/login")
    );
    let mut stranger = Client::new();
    assert_eq!(stranger.get("/login").await.status, 200);
    stranger
        .post("/login", json!({"email":user.email,"password":PASSWORD}))
        .await;
    assert_eq!(
        stranger.get("/dashboard").await.location.as_deref(),
        Some("/login")
    );
    assert!(
        AccountUserProvider::new()
            .retrieve_by_id(&user.id.to_string())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        directory::listings::entities::listing::Entity::find()
            .count(connection.inner())
            .await
            .unwrap(),
        before,
        "suspended account must not create a listing"
    );
}

#[tokio::test]
async fn upgrade_preserves_authentication_boundaries() {
    common::setup().await;
    required_rehash_is_forwarded().await;
    csrf_rotation_rejects_old_tokens().await;
    password_change_rejects_existing_sessions().await;
    suspension_blocks_existing_and_new_sign_ins().await;
}
