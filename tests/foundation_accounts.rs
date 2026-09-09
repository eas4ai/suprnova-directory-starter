mod common;

use common::Client;
use sea_orm_migration::MigratorTrait;
use suprnova::{DB, Mail, serde_json::json};

fn mail_token(mail: &suprnova::mail::MailFake, path: &str) -> String {
    mail.captured()
        .iter()
        .rev()
        .filter_map(|message| message.text.as_deref())
        .flat_map(str::lines)
        .find_map(|line| {
            line.split_once(&format!("http://directory.test/{path}?token="))
                .map(|(_, token)| token.trim().to_owned())
        })
        .expect("mail must contain the configured application link")
}

#[tokio::test]
async fn account_journey_uses_real_sessions_and_mail_links() {
    // The mechanism supplies a fresh database and environment before runtime
    // startup. Never load an operator's .env in this test binary.
    assert_eq!(std::env::var("APP_ENV").as_deref(), Ok("test"));
    directory::config::register_all();
    suprnova::Crypt::init(suprnova::EncryptionKey::generate());
    directory::bootstrap::register().await;
    directory::migrations::Migrator::up(DB::connection().unwrap().inner(), None)
        .await
        .unwrap();
    suprnova::rate_limit::bootstrap_default().await;
    directory::bootstrap::register_http_stack();
    let mail = Mail::fake();
    let mut client = Client::new();

    let denied = client.get("/dashboard").await;
    assert_eq!(
        denied.location.as_deref(),
        Some("/login"),
        "status {}: {}",
        denied.status,
        denied.body
    );
    assert_eq!(client.get("/register").await.status, 200);
    assert_eq!(client.post("/register", json!({"name": "A", "email": "invalid", "password": "short", "password_confirmation": "different"})).await.status, 422);
    let registered = client
        .post(
            "/register",
            json!({
                "name": "Ada Example", "email": "ada@example.test",
                "password": "first-password-123", "password_confirmation": "first-password-123"
            }),
        )
        .await;
    assert_eq!(registered.status, 302, "{}", registered.body);
    assert_eq!(
        mail.captured().len(),
        1,
        "Registration must send a verification link"
    );
    assert_eq!(client.get("/verify-email").await.status, 200);
    assert_eq!(
        client
            .post("/email/verification-notification", json!({}))
            .await
            .status,
        302
    );
    assert_eq!(mail.captured().len(), 2, "Resend must deliver another link");
    let token = mail_token(&mail, "verify-email/verify");
    let other = directory::models::user::User::create(
        "Other Account",
        "other@example.test",
        "other-password-123",
    )
    .await
    .unwrap();
    let mut stranger = Client::new();
    stranger.get("/login").await;
    assert_eq!(
        stranger
            .post(
                "/login",
                json!({"email": "other@example.test", "password": "other-password-123"})
            )
            .await
            .status,
        302
    );
    assert_eq!(
        stranger
            .get(&format!("/verify-email/verify?token={token}"))
            .await
            .status,
        400,
        "Another account cannot consume the verification token"
    );
    assert!(
        directory::models::user::User::find_by_email(&other.email)
            .await
            .unwrap()
            .unwrap()
            .email_verified_at
            .is_none()
    );
    let verified = client
        .get(&format!("/verify-email/verify?token={token}"))
        .await;
    assert_eq!(
        verified.location.as_deref(),
        Some("/dashboard"),
        "{}",
        verified.body
    );
    assert!(
        directory::models::user::User::find_by_email("ada@example.test")
            .await
            .unwrap()
            .unwrap()
            .email_verified_at
            .is_some()
    );
    assert_eq!(
        client
            .get(&format!("/verify-email/verify?token={token}"))
            .await
            .status,
        400
    );
    assert_eq!(
        client
            .get("/verify-email/verify?token=invalid")
            .await
            .status,
        400
    );
    let user = directory::models::user::User::find_by_email("ada@example.test")
        .await
        .unwrap()
        .unwrap();
    let expired = suprnova::auth_flows::token_store::TokenStore::issue(
        &user.id.to_string(),
        suprnova::auth_flows::token_store::TokenPurpose::EmailVerification,
        chrono::Duration::seconds(-1),
    )
    .await
    .unwrap();
    assert_eq!(
        client
            .get(&format!("/verify-email/verify?token={expired}"))
            .await
            .status,
        400
    );
    assert_eq!(client.get("/dashboard").await.status, 200);
    let mut old_session = client.clone();
    assert_eq!(client.post("/logout", json!({})).await.status, 302);
    assert_eq!(
        client.get("/dashboard").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        old_session.get("/dashboard").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        client
            .post(
                "/login",
                json!({"email": "ada@example.test", "password": "wrong"})
            )
            .await
            .status,
        422
    );
    assert_eq!(
        client
            .request(
                "POST",
                "/login",
                Some(json!({"email": "ada@example.test", "password": "first-password-123"})),
                false
            )
            .await
            .status,
        419
    );
    assert_eq!(
        client
            .post(
                "/login",
                json!({"email": "ada@example.test", "password": "first-password-123"})
            )
            .await
            .status,
        302
    );
    assert_eq!(client.get("/dashboard").await.status, 200);

    let mut recovery = Client::new();
    assert_eq!(recovery.get("/forgot-password").await.status, 200);
    assert_eq!(recovery.inertia_post("/reset-password", json!({"token": "invalid", "password": "next-password-456", "password_confirmation": "next-password-456"})).await.status, 303, "Inertia must receive a validation redirect for an invalid reset token");
    let count = mail.captured().len();
    let unverified = recovery
        .post("/forgot-password", json!({"email": "other@example.test"}))
        .await;
    assert_eq!(
        mail.captured().len(),
        count,
        "Unverified provider accounts must not receive reset links"
    );
    let unknown = recovery
        .post("/forgot-password", json!({"email": "unknown@example.test"}))
        .await;
    assert_eq!(
        (unverified.status, unverified.location, unverified.body),
        (
            unknown.status,
            unknown.location.clone(),
            unknown.body.clone()
        )
    );
    assert_eq!(mail.captured().len(), count);
    let known = recovery
        .post("/forgot-password", json!({"email": "ada@example.test"}))
        .await;
    assert_eq!(known.status, 302, "{}", known.body);
    assert_eq!(
        (unknown.status, unknown.location, unknown.body),
        (known.status, known.location, known.body)
    );
    let reset = mail_token(&mail, "reset-password");
    assert_eq!(
        recovery
            .get(&format!("/reset-password?token={reset}"))
            .await
            .status,
        200
    );
    for token in [
        "invalid".to_owned(),
        suprnova::auth_flows::token_store::TokenStore::issue(
            &user.id.to_string(),
            suprnova::auth_flows::token_store::TokenPurpose::PasswordReset,
            chrono::Duration::seconds(-1),
        )
        .await
        .unwrap(),
    ] {
        assert_eq!(recovery.post("/reset-password", json!({"token": token, "password": "next-password-456", "password_confirmation": "next-password-456"})).await.status, 422);
    }
    assert_eq!(recovery.post("/reset-password", json!({"token": reset, "password": "next-password-456", "password_confirmation": "different"})).await.status, 422);
    let reset_result = recovery.post("/reset-password", json!({"token": reset, "password": "next-password-456", "password_confirmation": "next-password-456"})).await;
    assert_eq!(
        reset_result.location.as_deref(),
        Some("/login"),
        "{}",
        reset_result.body
    );
    assert_eq!(recovery.post("/reset-password", json!({"token": reset, "password": "another-password", "password_confirmation": "another-password"})).await.status, 422);
    assert_eq!(
        client.get("/dashboard").await.location.as_deref(),
        Some("/login"),
        "Password reset must revoke existing authenticated sessions"
    );
    assert_eq!(
        recovery
            .post(
                "/login",
                json!({"email": "ada@example.test", "password": "first-password-123"})
            )
            .await
            .status,
        422
    );
    assert_eq!(
        recovery
            .post(
                "/login",
                json!({"email": "ada@example.test", "password": "next-password-456"})
            )
            .await
            .status,
        302
    );
    assert_eq!(recovery.get("/dashboard").await.status, 200);
}
