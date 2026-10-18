#[allow(dead_code)] // This suite uses the shared HTTP client without every account helper.
mod common;

use common::Client;
use directory::{
    billing::{self, BillingError, Mode, Provider, SaveSettings},
    commands::admin_access::{AccessAction, change_access},
    models::user::User,
};
use sea_orm::{
    ConnectionTrait, Statement,
    sea_query::{Alias, Expr, ExprTrait, Query},
};
use suprnova::{
    DB,
    eloquent::Model,
    serde_json::{Value, json},
};

fn blank_profile() -> Value {
    json!({"enabled":false,"public_key":"","api_key":"","webhook_key":"","clear_secrets":false})
}

fn input(revision: i64, mode: Mode) -> Value {
    let (stripe, paddle, client) = match mode {
        Mode::Test => ("test", "sdbx", "test"),
        Mode::Live => ("live", "live", "live"),
    };
    json!({"revision":revision,"default_provider":"stripe",
        "stripe":{"enabled":true,"public_key":format!("pk_{stripe}_FixturePublic"),
            "api_key":format!("sk_{stripe}_SECRET_API_MARKER"),"webhook_key":"whsec_SECRET_WEBHOOK_MARKER","clear_secrets":false},
        "paddle":{"enabled":true,"public_key":format!("{client}_FixtureClient"),
            "api_key":format!("pdl_{paddle}_apikey_SECRET_PADDLE_API"),"webhook_key":"pdl_ntfset_SECRET_PADDLE_WEBHOOK","clear_secrets":false},
        "mappings":[{"plan":"featured","stripe":format!("price_{stripe}_featured"),"paddle":format!("pri_{stripe}_featured")}]})
}

fn parse(value: Value) -> SaveSettings {
    suprnova::serde_json::from_value(value).unwrap()
}

async fn verified(name: &str, email: &str) -> User {
    let mut user = User::create(name, email, "fixture-password-123")
        .await
        .unwrap();
    user.email_verified_at = Some(chrono::Utc::now());
    user.save().await.unwrap();
    user
}

async fn login(user: &User) -> Client {
    let mut client = Client::new();
    client.get("/login").await;
    assert_eq!(
        client
            .post(
                "/login",
                json!({"email":user.email,"password":"fixture-password-123"})
            )
            .await
            .status,
        302
    );
    client
}

async fn payload(mode: Mode) -> String {
    let db = DB::connection().unwrap();
    let query = Query::select()
        .column(Alias::new("payload"))
        .from(Alias::new("billing_settings"))
        .and_where(Expr::col(Alias::new("mode")).eq(mode.as_str()))
        .to_owned();
    db.inner()
        .query_one(&query)
        .await
        .unwrap()
        .unwrap()
        .try_get("", "payload")
        .unwrap()
}

async fn set_payload(mode: Mode, value: &str) {
    let db = DB::connection().unwrap();
    let query = Query::update()
        .table(Alias::new("billing_settings"))
        .value(Alias::new("payload"), value)
        .and_where(Expr::col(Alias::new("mode")).eq(mode.as_str()))
        .to_owned();
    db.inner().execute(&query).await.unwrap();
}

async fn permission_count(user: &User) -> i64 {
    let mut count = 0;
    for permission in directory::commands::admin_access::ADMIN_PERMISSIONS {
        if suprnova::rbac::has_permission_for_model(
            "directory.user",
            &user.id.to_string(),
            permission,
        )
        .await
        .unwrap()
        {
            count += 1;
        }
    }
    count
}

#[tokio::test(flavor = "current_thread")]
async fn provider_administration_contract() {
    let _mail = common::setup().await;
    let ordinary = verified("Member", "billing-member@example.test").await;
    let admin = verified("Operator", "billing-operator@example.test").await;
    let unverified = User::create(
        "Unverified",
        "billing-unverified@example.test",
        "fixture-password-123",
    )
    .await
    .unwrap();

    assert!(change_access(i64::MAX, AccessAction::Grant).await.is_err());
    assert!(
        change_access(unverified.id, AccessAction::Grant)
            .await
            .is_err()
    );
    assert_eq!(permission_count(&unverified).await, 0);
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    assert_eq!(
        permission_count(&admin).await,
        8,
        "repeat grants must be idempotent"
    );
    for capability in [
        "admin.access",
        "billing.configure",
        "listings.moderate",
        "articles.manage",
        "taxonomy.manage",
        "accounts.manage",
        "audit.view",
        "seo.manage",
    ] {
        assert!(
            suprnova::rbac::has_permission_for_model(
                "directory.user",
                &admin.id.to_string(),
                capability
            )
            .await
            .unwrap()
        );
    }

    let mut guest = Client::new();
    guest.get("/login").await;
    let mut member = login(&ordinary).await;
    let mut operator = login(&admin).await;
    for client in [&mut guest, &mut member] {
        for mode in ["test", "live"] {
            let path = format!("/admin/billing?mode={mode}");
            assert_eq!(client.get(&path).await.status, 302);
            assert_eq!(client.post(&path, input(0, Mode::Test)).await.status, 302);
        }
    }
    suprnova::rbac::give_permission_to_model(
        "directory.user",
        &ordinary.id.to_string(),
        "admin.access",
    )
    .await
    .unwrap();
    assert_eq!(
        member.get("/admin/billing").await.status,
        403,
        "shell access alone cannot read billing"
    );
    assert_eq!(
        member
            .post("/admin/billing", input(0, Mode::Test))
            .await
            .status,
        403
    );
    let denied_csrf = operator
        .request("POST", "/admin/billing", Some(input(0, Mode::Test)), false)
        .await;
    assert_eq!(denied_csrf.status, 419);
    let empty = billing::load(Mode::Test).await.unwrap();
    assert_eq!(empty.revision, 0);
    assert!(!empty.stripe.enabled && !empty.paddle.enabled && empty.default_provider.is_none());

    let saved = operator
        .post("/admin/billing?mode=test", input(0, Mode::Test))
        .await;
    assert_eq!(saved.status, 302, "{}", saved.body);
    let read = operator.get("/admin/billing?mode=test").await;
    assert_eq!(read.status, 200);
    for marker in [
        "SECRET_API_MARKER",
        "SECRET_WEBHOOK_MARKER",
        "SECRET_PADDLE_API",
        "SECRET_PADDLE_WEBHOOK",
    ] {
        assert!(!read.body.contains(marker), "a response leaked a secret");
        assert!(
            !saved.body.contains(marker),
            "a save response leaked a secret"
        );
        assert!(
            !payload(Mode::Test).await.contains(marker),
            "database payload contains a plaintext secret"
        );
    }
    assert!(billing::load(Mode::Test).await.unwrap().stripe.has_secrets);
    assert_eq!(
        billing::load(Mode::Live).await.unwrap().revision,
        0,
        "test saves must not touch live configuration"
    );
    for provider in [Provider::Stripe, Provider::Paddle] {
        let resolved = billing::resolve(Mode::Test, provider, "featured")
            .await
            .unwrap();
        assert_eq!(resolved.provider.name(), provider.as_str());
        assert_eq!(resolved.mode, Mode::Test);
        assert!(resolved.price_id.contains("test"));
        assert!(
            billing::resolve(Mode::Live, provider, "featured")
                .await
                .is_err()
        );
        assert!(
            billing::resolve(Mode::Test, provider, "missing")
                .await
                .is_err()
        );
    }
    println!(
        "Provider access, direct-request denial, CSRF, encrypted storage and both real adapters passed."
    );

    // A saved provider can be disabled without clearing its secrets, and an
    // unrelated edit with blank replacements preserves credential usability.
    let mut edit = input(1, Mode::Test);
    for provider in ["stripe", "paddle"] {
        edit[provider]["api_key"] = json!("");
        edit[provider]["webhook_key"] = json!("");
    }
    edit["stripe"]["enabled"] = json!(false);
    billing::save(Mode::Test, parse(edit.clone()))
        .await
        .unwrap();
    let disabled = billing::load(Mode::Test).await.unwrap();
    assert!(disabled.default_provider.is_none() && disabled.stripe.has_secrets);
    assert!(
        billing::resolve(Mode::Test, Provider::Stripe, "featured")
            .await
            .is_err()
    );
    edit["revision"] = json!(2);
    edit["stripe"]["enabled"] = json!(true);
    edit["stripe"]["api_key"] = json!("sk_test_REPLACEMENT_SECRET");
    billing::save(Mode::Test, parse(edit.clone()))
        .await
        .unwrap();
    assert!(
        billing::resolve(Mode::Test, Provider::Stripe, "featured")
            .await
            .is_ok()
    );
    assert!(!payload(Mode::Test).await.contains("REPLACEMENT_SECRET"));

    let before = payload(Mode::Test).await;
    let mut invalid = input(3, Mode::Test);
    invalid["stripe"]["api_key"] = json!("sk_live_WRONG_MODE_SECRET");
    let rejected = operator.post("/admin/billing?mode=test", invalid).await;
    assert_eq!(rejected.status, 422, "{}", rejected.body);
    assert!(!rejected.body.contains("WRONG_MODE_SECRET"));
    assert_eq!(payload(Mode::Test).await, before);
    for bad in [
        "",
        "   ",
        "sk_test_hello\r\nInjected: value",
        "sk_test_\u{00e9}",
        "sk_test_bad space",
    ] {
        let mut invalid = input(3, Mode::Test);
        invalid["stripe"]["api_key"] = json!(bad);
        if bad.is_empty() {
            invalid["paddle"] = blank_profile();
            invalid["stripe"]["webhook_key"] = json!("");
            invalid["stripe"]["clear_secrets"] = json!(true);
        }
        assert!(billing::save(Mode::Test, parse(invalid)).await.is_err());
        assert_eq!(payload(Mode::Test).await, before);
    }
    let mut invalid = input(3, Mode::Test);
    invalid["paddle"]["public_key"] = json!("live_wrongmode");
    assert!(billing::save(Mode::Test, parse(invalid)).await.is_err());
    let mut invalid = input(3, Mode::Test);
    invalid["mappings"][0]["stripe"] = json!("pri_wrongprovider");
    assert!(billing::save(Mode::Test, parse(invalid)).await.is_err());
    let mut duplicate = input(3, Mode::Test);
    duplicate["mappings"]
        .as_array_mut()
        .unwrap()
        .push(json!({"plan":"featured","stripe":"price_other","paddle":""}));
    assert!(billing::save(Mode::Test, parse(duplicate)).await.is_err());
    assert_eq!(payload(Mode::Test).await, before);

    // Competing writes start from the same revision. Exactly one may commit.
    let (first, second) = tokio::join!(
        billing::save(Mode::Test, parse(input(3, Mode::Test))),
        billing::save(Mode::Test, parse(input(3, Mode::Test))),
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert!(
        matches!(first, Err(BillingError::Conflict))
            || matches!(second, Err(BillingError::Conflict))
    );
    assert_eq!(billing::load(Mode::Test).await.unwrap().revision, 4);
    let before_failure = payload(Mode::Test).await;
    let db = DB::connection().unwrap();
    db.inner().execute_raw(Statement::from_string(sea_orm::DatabaseBackend::Sqlite,
        "CREATE TRIGGER reject_billing_update BEFORE UPDATE ON billing_settings BEGIN SELECT RAISE(ABORT, 'fixture write failure'); END".to_owned())).await.unwrap();
    assert!(
        billing::save(Mode::Test, parse(input(4, Mode::Test)))
            .await
            .is_err()
    );
    assert_eq!(payload(Mode::Test).await, before_failure);
    assert_eq!(billing::load(Mode::Test).await.unwrap().revision, 4);
    db.inner()
        .execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "DROP TRIGGER reject_billing_update".to_owned(),
        ))
        .await
        .unwrap();
    println!("Local validation, stale/concurrent edits and failed-write atomicity passed.");

    billing::save(Mode::Live, parse(input(0, Mode::Live)))
        .await
        .unwrap();
    for provider in [Provider::Stripe, Provider::Paddle] {
        assert!(
            billing::resolve(Mode::Live, provider, "featured")
                .await
                .unwrap()
                .price_id
                .contains("live")
        );
    }
    let mut changed = input(4, Mode::Test);
    changed["mappings"][0]["stripe"] = json!("price_changed");
    changed["mappings"][0]["paddle"] = json!("");
    billing::save(Mode::Test, parse(changed)).await.unwrap();
    assert_eq!(
        billing::resolve(Mode::Test, Provider::Stripe, "featured")
            .await
            .unwrap()
            .price_id,
        "price_changed"
    );
    assert!(
        billing::resolve(Mode::Test, Provider::Paddle, "featured")
            .await
            .is_err()
    );
    let mut remove = input(5, Mode::Test);
    remove["mappings"] = json!([]);
    billing::save(Mode::Test, parse(remove)).await.unwrap();
    assert!(billing::load(Mode::Test).await.unwrap().mappings.is_empty());
    assert!(
        billing::resolve(Mode::Test, Provider::Stripe, "featured")
            .await
            .is_err()
    );

    let intact = payload(Mode::Test).await;
    let mut corrupt: Value = suprnova::serde_json::from_str(&intact).unwrap();
    corrupt["stripe"]["secrets"] = json!("corrupt-ciphertext");
    set_payload(Mode::Test, &corrupt.to_string()).await;
    assert!(billing::load(Mode::Test).await.is_err());
    assert!(
        billing::save(Mode::Test, parse(input(6, Mode::Test)))
            .await
            .is_err()
    );
    assert_eq!(payload(Mode::Test).await, corrupt.to_string());
    let unavailable = operator.get("/admin/billing?mode=test").await;
    assert_eq!(unavailable.status, 200);
    assert!(unavailable.body.contains("cannot be decrypted"));
    set_payload(Mode::Test, &intact).await;
    // Ciphertext bound to test/Stripe cannot be transplanted to live/Stripe.
    let live_intact = payload(Mode::Live).await;
    let mut live: Value = suprnova::serde_json::from_str(&live_intact).unwrap();
    let test: Value = suprnova::serde_json::from_str(&intact).unwrap();
    live["stripe"]["secrets"] = test["stripe"]["secrets"].clone();
    set_payload(Mode::Live, &live.to_string()).await;
    assert!(billing::load(Mode::Live).await.is_err());
    set_payload(Mode::Live, &live_intact).await;

    // Removal is explicit and only possible after disabling a readable profile.
    let mut clear = input(6, Mode::Test);
    clear["stripe"] = json!({"enabled":false,"public_key":"pk_test_FixturePublic","api_key":"","webhook_key":"","clear_secrets":true});
    billing::save(Mode::Test, parse(clear)).await.unwrap();
    let cleared = billing::load(Mode::Test).await.unwrap();
    assert!(
        !cleared.stripe.has_secrets
            && !cleared.stripe.enabled
            && cleared.stripe.public_key.is_empty()
    );

    suprnova::rbac::give_permission_to_role("billing-admin-fixture", "billing.configure")
        .await
        .unwrap();
    suprnova::rbac::assign_role_to_model(
        "directory.user",
        &admin.id.to_string(),
        "billing-admin-fixture",
    )
    .await
    .unwrap();
    suprnova::rbac::assign_role_to_model("directory.user", &admin.id.to_string(), "unrelated-role")
        .await
        .unwrap();
    change_access(admin.id, AccessAction::Revoke).await.unwrap();
    change_access(admin.id, AccessAction::Revoke).await.unwrap();
    assert_eq!(permission_count(&admin).await, 0);
    assert!(
        suprnova::rbac::has_role_for_model(
            "directory.user",
            &admin.id.to_string(),
            "unrelated-role"
        )
        .await
        .unwrap()
    );
    assert!(
        !suprnova::rbac::has_permission_for_model(
            "directory.user",
            &admin.id.to_string(),
            "billing.configure"
        )
        .await
        .unwrap()
    );
    assert_eq!(
        operator.get("/admin/billing").await.location.as_deref(),
        Some("/dashboard")
    );
    assert_eq!(
        operator
            .post("/admin/billing", input(7, Mode::Test))
            .await
            .status,
        302
    );

    let mut registrant = Client::new();
    registrant.get("/register").await;
    assert_eq!(registrant.post("/register", json!({"name":"Registrant","email":"billing-registrant@example.test","password":"fixture-password-123","password_confirmation":"fixture-password-123","roles":["administrator"],"permissions":["admin.access","billing.configure"]})).await.status, 302);
    let registered = User::find_by_email("billing-registrant@example.test")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(permission_count(&registered).await, 0);
    assert_eq!(registrant.get("/admin/billing").await.status, 302);

    // Leave a verified, non-administrative operator for the real CLI/browser steps.
    println!("BILLING_OPERATOR_ID={}", admin.id);
    println!(
        "Mapping edits/removal, ciphertext integrity, revocation and registration privilege denial passed."
    );
}
