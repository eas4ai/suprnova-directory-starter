#[allow(dead_code)]
mod common;

use common::Client;
use directory::{commands::admin_access::ADMIN_PERMISSIONS, models::user::User};
use suprnova::{eloquent::Model, serde_json::json};

#[tokio::test(flavor = "current_thread")]
async fn seo_workflows_contract() {
    let _mail = common::setup().await;
    let mut admin = User::create(
        "SEO administrator",
        "seo-admin@example.test",
        "fixture-password-123",
    )
    .await
    .unwrap();
    admin.email_verified_at = Some(chrono::Utc::now());
    admin.save().await.unwrap();
    for permission in ADMIN_PERMISSIONS {
        suprnova::rbac::give_permission_to_model(
            "directory.user",
            &admin.id.to_string(),
            permission,
        )
        .await
        .unwrap();
    }
    let mut client = Client::new();
    client.get("/login").await;
    assert_eq!(
        client
            .post(
                "/login",
                json!({"email":admin.email,"password":"fixture-password-123"})
            )
            .await
            .status,
        302
    );
    let response = client.inertia_get("/admin/seo").await;
    assert_eq!(
        response.status, 200,
        "SEO settings must be available to the full administrator: {}",
        response.body
    );
    let page: suprnova::serde_json::Value = suprnova::serde_json::from_str(&response.body).unwrap();
    assert_eq!(page["component"], "admin/Seo");
    assert!(page["props"]["settings"].is_object());
}
