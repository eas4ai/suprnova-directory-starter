#[allow(dead_code)]
mod common;

use common::Client;
use directory::{accounts, accounts::provider::AccountUserProvider, models::user::User};
use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait, TransactionTrait};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use suprnova::{
    DB, LaravelDatabase, auth::UserProvider, eloquent::Model, hashing, serde_json::json,
};

const PASSWORD: &str = "upgrade-password-123";

#[derive(Clone)]
struct ChangeMailboxAfterRead {
    id: i64,
    writer: sea_orm::DatabaseConnection,
    attempted: Arc<AtomicBool>,
    committed: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl suprnova::eloquent::observers::Observer<User> for ChangeMailboxAfterRead {
    async fn retrieved(&self, user: &User) -> Result<(), suprnova::FrameworkError> {
        if user.id != self.id || self.attempted.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        // The model has been read but verification has not been stamped. This
        // independent connection deterministically attempts the competing write.
        let result = self
            .writer
            .execute_raw(sea_orm::Statement::from_sql_and_values(
                sea_orm::DbBackend::Sqlite,
                "UPDATE users SET email = ?, email_verified_at = NULL WHERE id = ?",
                ["unproven-mailbox@example.test".into(), self.id.into()],
            ))
            .await;
        match result {
            Ok(result) => {
                assert_eq!(result.rows_affected(), 1);
                self.committed.store(true, Ordering::SeqCst);
            }
            Err(error) => {
                let message = error.to_string().to_ascii_lowercase();
                assert!(
                    message.contains("locked") || message.contains("busy"),
                    "unexpected competing-writer failure: {error}"
                );
            }
        }
        Ok(())
    }
}

async fn mailbox_verification_preserves_the_atomic_provider_boundary() {
    let user = User::create("Mailbox fixture", "proven-mailbox@example.test", PASSWORD)
        .await
        .unwrap();
    let provider = AccountUserProvider::new();
    let id = user.id.to_string();
    assert!(
        !provider
            .mark_email_verified_for(&id, "wrong@example.test")
            .await
            .unwrap()
    );
    assert!(
        !provider
            .mark_email_verified_for("9223372036854775807", &user.email)
            .await
            .unwrap()
    );
    assert!(!provider.is_email_verified(&id).await.unwrap());

    let mut options = sea_orm::ConnectOptions::new(std::env::var("DATABASE_URL").unwrap());
    options.max_connections(1);
    let writer = sea_orm::Database::connect(options).await.unwrap();
    writer
        .execute_unprepared("PRAGMA busy_timeout = 100")
        .await
        .unwrap();
    let attempted = Arc::new(AtomicBool::new(false));
    let committed = Arc::new(AtomicBool::new(false));
    User::observe(ChangeMailboxAfterRead {
        id: user.id,
        writer,
        attempted: attempted.clone(),
        committed: committed.clone(),
    })
    .await;
    let outcome = provider.mark_email_verified_for(&id, &user.email).await;
    assert!(attempted.load(Ordering::SeqCst), "the race hook must run");
    let stored = directory::models::user::Entity::find_by_id(user.id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap();
    if committed.load(Ordering::SeqCst) {
        assert_eq!(stored.email, "unproven-mailbox@example.test");
        assert!(
            stored.email_verified_at.is_none(),
            "the old mailbox must not verify the replacement: {outcome:?}"
        );
        assert!(!matches!(outcome, Ok(true)));
    } else {
        // SQLite may serialize the competing writer, or allow its WAL write
        // and reject the stale verification transaction. Both outcomes are safe.
        assert_eq!(stored.email, user.email);
        assert!(stored.email_verified_at.is_some());
        assert!(outcome.unwrap());
    }

    let normal = User::create(
        "Normal verification",
        "normal-mailbox@example.test",
        PASSWORD,
    )
    .await
    .unwrap();
    assert!(
        provider
            .mark_email_verified_for(&normal.id.to_string(), &normal.email)
            .await
            .unwrap()
    );
    assert!(
        provider
            .is_email_verified(&normal.id.to_string())
            .await
            .unwrap()
    );
}

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

    // Exercise the new sign-in hook through the session guard, not just the provider.
    let signing_in = account("rehash-sign-in@example.test").await;
    LaravelDatabase::share(true);
    let mut client = Client::new();
    login(&mut client, &signing_in, PASSWORD).await;
    LaravelDatabase::share(false);
    let signed_in = User::find_by_email(&signing_in.email)
        .await
        .unwrap()
        .unwrap();
    assert!(signed_in.password.starts_with("$2y$"));
    assert_ne!(signed_in.password, signing_in.password);
    assert!(hashing::verify(PASSWORD, &signed_in.password).unwrap());

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

async fn normalized_email_and_safe_login_returns() {
    let mut client = Client::new();
    assert_eq!(client.get("/register").await.status, 200);
    let registered = client
        .post(
            "/register",
            json!({
                "name": "Normalized mailbox", "email": "  Mixed.Case@Example.Test  ",
                "password": PASSWORD, "password_confirmation": PASSWORD,
            }),
        )
        .await;
    assert_eq!(registered.status, 302, "{}", registered.body);
    let user = User::find_by_email("MIXED.CASE@EXAMPLE.TEST")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.email, "mixed.case@example.test");

    let mut guest = Client::new();
    guest.get("/login?return_to=/dashboard/listings").await;
    let signed_in = guest
        .post(
            "/login",
            json!({"email": " MIXED.CASE@EXAMPLE.TEST ", "password": PASSWORD}),
        )
        .await;
    assert_eq!(signed_in.status, 302);
    assert_eq!(signed_in.location.as_deref(), Some("/dashboard/listings"));

    let mut inertia = Client::new();
    inertia.get("/login?return_to=/dashboard/listings").await;
    let signed_in = inertia
        .inertia_post("/login", json!({"email": user.email, "password": PASSWORD}))
        .await;
    assert_eq!(signed_in.status, 409);
    assert_eq!(
        signed_in.headers["x-inertia-location"],
        "/dashboard/listings"
    );
    let rejected = guest.get("/login?return_to=//outside.example/path").await;
    assert_eq!(rejected.location.as_deref(), Some("/dashboard"));

    let mut external = Client::new();
    external
        .get("/login?return_to=https://outside.example/path")
        .await;
    let signed_in = external
        .post("/login", json!({"email": user.email, "password": PASSWORD}))
        .await;
    assert_eq!(signed_in.location.as_deref(), Some("/dashboard"));

    assert_inertia_validation_errors(&user.email).await;
}

async fn assert_inertia_validation_errors(registered_email: &str) {
    let mut duplicate = Client::new();
    for (path, body, field) in [
        (
            "/register",
            json!({
                "name": "Duplicate mailbox", "email": registered_email.to_ascii_uppercase(),
                "password": PASSWORD, "password_confirmation": PASSWORD,
            }),
            "email",
        ),
        (
            "/register",
            json!({
                "name": "n".repeat(256), "email": "long-name@example.test",
                "password": PASSWORD, "password_confirmation": PASSWORD,
            }),
            "name",
        ),
        (
            "/forgot-password",
            json!({
                "email": format!("{}@example.test", "a".repeat(250)),
            }),
            "email",
        ),
    ] {
        duplicate.get(path).await;
        let refused = duplicate.inertia_post(path, body).await;
        assert_eq!(refused.status, 303);
        assert_eq!(refused.location.as_deref(), Some(path));
        let page = duplicate.inertia_get(path).await;
        let page: suprnova::serde_json::Value = suprnova::serde_json::from_str(&page.body).unwrap();
        assert!(page["props"]["errors"][field].is_string());
    }
}

#[tokio::test]
async fn upgrade_preserves_authentication_boundaries() {
    let _mail = common::setup().await;
    mailbox_verification_preserves_the_atomic_provider_boundary().await;
    required_rehash_is_forwarded().await;
    normalized_email_and_safe_login_returns().await;
    csrf_rotation_rejects_old_tokens().await;
    password_change_rejects_existing_sessions().await;
    suspension_blocks_existing_and_new_sign_ins().await;
}
