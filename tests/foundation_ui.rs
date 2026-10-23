#[allow(dead_code)] // This shared fixture also serves other integration suites.
mod common;
use common::Client;
use directory::models::user::User;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, Query};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use suprnova::serde_json::json;

#[tokio::test]
async fn administration_requires_an_explicit_permission() {
    let _mail = common::setup().await;
    let mut guest = Client::new();
    assert_eq!(
        guest.get("/admin").await.location.as_deref(),
        Some("/login")
    );
    let ordinary = User::create(
        "Directory Member",
        "member@example.test",
        "member-password-123",
    )
    .await
    .unwrap();
    let admin = User::create(
        "Directory Operator",
        "operator@example.test",
        "operator-password-123",
    )
    .await
    .unwrap();
    guest.get("/login").await;
    assert_eq!(
        guest
            .post(
                "/login",
                json!({"email": ordinary.email, "password": "member-password-123"})
            )
            .await
            .status,
        302
    );
    assert_eq!(
        guest.get("/admin").await.location.as_deref(),
        Some("/dashboard")
    );
    // The starter now seeds this role with explicit permissions. Strip those in
    // this disposable fixture so the same name-only denial remains meaningful.
    use suprnova::rbac::entity::{RoleColumn, RoleEntity};
    let role = RoleEntity::find()
        .filter(RoleColumn::Name.eq("administrator"))
        .filter(RoleColumn::GuardName.eq("web"))
        .one(suprnova::DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap();
    let connection = suprnova::DB::connection().unwrap();
    let db = connection.inner();
    let remove_permissions = Query::delete()
        .from_table(Alias::new("role_has_permissions"))
        .and_where(Expr::col(Alias::new("role_id")).eq(u64::from(role.id)))
        .to_owned();
    db.execute_raw(db.get_database_backend().build(&remove_permissions))
        .await
        .unwrap();
    // A role name alone grants nothing; only the explicit permission does.
    suprnova::rbac::assign_role_to_model(
        "directory.user",
        &ordinary.id.to_string(),
        "administrator",
    )
    .await
    .unwrap();
    assert_eq!(
        guest.get("/admin").await.location.as_deref(),
        Some("/dashboard")
    );
    suprnova::rbac::give_permission_to_model(
        "directory.user",
        &admin.id.to_string(),
        "admin.access",
    )
    .await
    .unwrap();
    let mut allowed = Client::new();
    allowed.get("/login").await;
    assert_eq!(
        allowed
            .post(
                "/login",
                json!({"email": admin.email, "password": "operator-password-123"})
            )
            .await
            .status,
        302
    );
    let page = allowed.get("/admin").await;
    assert_eq!(page.status, 200, "{}", page.body);
    assert!(
        page.body.contains(r#""component":"admin\/Overview""#),
        "Admin component must be in the page payload"
    );
    assert_eq!(allowed.inertia_post("/logout", json!({})).await.status, 302);
}
