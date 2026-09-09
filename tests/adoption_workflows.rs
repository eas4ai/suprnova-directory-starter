#[allow(dead_code)]
mod common;
#[allow(dead_code)]
#[path = "common/payments.rs"]
mod fixtures;
use common::Client;
use directory::{
    listings::{SaveListing, entities::listing, workflow},
    models::user::User,
    notifications::{self, delivery, entity as notice},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait,
    sea_query::Expr,
};
use suprnova::{
    DB, FrameworkError, Mail,
    eloquent::Model,
    serde_json::{self, json},
};
const PASSWORD: &str = "fixture-password-123";
async fn user(email: &str) -> User {
    let mut u = User::create(email, email, PASSWORD).await.unwrap();
    u.email_verified_at = Some(chrono::Utc::now());
    u.save().await.unwrap();
    u
}
async fn login(u: &User) -> Client {
    let mut client = Client::new();
    client.get("/login").await;
    assert_eq!(
        client
            .post("/login", json!({"email":u.email,"password":PASSWORD}))
            .await
            .status,
        302
    );
    client
}
async fn row(id: i64) -> listing::Model {
    listing::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
fn content(version: i64, category: i64) -> SaveListing {
    SaveListing {
        version,
        title: "Adoption notification listing".into(),
        summary: "A saved notification fixture".into(),
        description: "Safe **content**".into(),
        url: "https://example.test/resource".into(),
        category_ids: vec![category],
        media_id: None,
        media_alt: String::new(),
    }
}
async fn count() -> u64 {
    notice::Entity::find()
        .count(DB::connection().unwrap().inner())
        .await
        .unwrap()
}
async fn sql(query: &str) {
    DB::connection()
        .unwrap()
        .inner()
        .execute_unprepared(query)
        .await
        .unwrap();
}
async fn decide(admin: i64, id: i64, decision: &str, reason: &str) -> Result<(), FrameworkError> {
    let r = row(id).await;
    workflow::decide(
        admin,
        id,
        workflow::Decision {
            version: r.version,
            revision_id: r.current_revision_id.unwrap(),
            decision: decision.into(),
            reason: reason.into(),
        },
    )
    .await
}
struct FailedMail;
#[async_trait::async_trait]
impl suprnova::mail::MailTransport for FailedMail {
    async fn send(&self, _: &suprnova::mail::OutgoingMessage) -> Result<(), FrameworkError> {
        Err(FrameworkError::internal("SECRET_transport_diagnostic"))
    }
    fn name(&self) -> &'static str {
        "controlled-failure"
    }
}
#[tokio::test]
async fn adoption_workflows_contract() {
    let initial_mail = common::setup().await;
    let admin = user("adoption-browser-admin@example.test").await;
    directory::commands::admin_access::change_access(
        admin.id,
        directory::commands::admin_access::AccessAction::Grant,
    )
    .await
    .unwrap();
    let member = user("member@example.test").await;
    let other = user("adoption-other@example.test").await;
    workflow::seed_categories().await.unwrap();
    let category = directory::listings::queries::categories().await.unwrap()[0].id;
    let id = workflow::save(member.id, None, content(0, category))
        .await
        .unwrap();
    workflow::submit(member.id, id, row(id).await.version)
        .await
        .unwrap();
    // Failing intent storage must roll the domain decision and audit back.
    let before = row(id).await;
    sql("CREATE TRIGGER reject_notice BEFORE INSERT ON owner_notifications BEGIN SELECT RAISE(ABORT, 'controlled outbox fault'); END").await;
    assert!(
        decide(admin.id, id, "reject", "needs changes")
            .await
            .is_err()
    );
    assert_eq!(row(id).await, before);
    assert_eq!(count().await, 0);
    sql("DROP TRIGGER reject_notice").await;
    let reason = "<script>window.adoptionInjected=true</script>";
    decide(admin.id, id, "reject", reason).await.unwrap();
    assert_eq!(count().await, 1);
    assert!(decide(admin.id, id, "reject", reason).await.is_err());
    assert_eq!(count().await, 1);
    workflow::save(
        member.id,
        Some(id),
        content(row(id).await.version, category),
    )
    .await
    .unwrap();
    workflow::submit(member.id, id, row(id).await.version)
        .await
        .unwrap();
    decide(admin.id, id, "approve", "").await.unwrap();
    workflow::suspend(
        admin.id,
        id,
        workflow::Suspension {
            version: row(id).await.version,
            suspended: true,
            reason: "Temporary policy review".into(),
        },
    )
    .await
    .unwrap();
    workflow::suspend(
        admin.id,
        id,
        workflow::Suspension {
            version: row(id).await.version,
            suspended: false,
            reason: "Review complete".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(count().await, 4);
    assert!(initial_mail.captured().is_empty());
    let mut owner = login(&member).await;
    let view = owner
        .inertia_get(&format!("/dashboard/listings/{id}/edit"))
        .await;
    assert_eq!(view.status, 200);
    let props: serde_json::Value = serde_json::from_str(&view.body).unwrap();
    assert_eq!(props["props"]["notifications"].as_array().unwrap().len(), 4);
    assert!(!view.body.contains("event_key"));
    assert!(!view.body.contains("lease_token"));
    assert!(
        notifications::recent(other.id, id)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        login(&other)
            .await
            .get(&format!("/dashboard/listings/{id}/edit"))
            .await
            .status,
        404
    );
    // A fresh console process can see committed intents without application memory.
    let console = std::process::Command::new(env!("CARGO_BIN_EXE_console"))
        .args(["notifications:deliver", "--status", "--limit", "100"])
        .output()
        .unwrap();
    assert!(
        console.status.success(),
        "{}",
        String::from_utf8_lossy(&console.stderr)
    );
    let durable: serde_json::Value = serde_json::from_slice(&console.stdout).unwrap();
    assert_eq!(durable.as_array().unwrap().len(), 4);
    let first = durable[0]["id"].as_str().unwrap().to_owned();
    Mail::set_transport(std::sync::Arc::new(FailedMail)).unwrap();
    let failed = delivery::process(&first).await.unwrap();
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.attempts, 1);
    assert_eq!(failed.last_error.as_deref(), Some("mail_delivery_failed"));
    assert!(!serde_json::to_string(&failed).unwrap().contains("SECRET"));
    assert_eq!(count().await, 4);
    // Automatic retries honor the due time and stop at a bounded attempt budget.
    assert_eq!(delivery::process(&first).await.unwrap().attempts, 1);
    for attempt in 2..=delivery::MAX_ATTEMPTS {
        notice::Entity::update_many()
            .col_expr(notice::Column::NextAttemptAt, Expr::value(0))
            .filter(notice::Column::Id.eq(&first))
            .exec(DB::connection().unwrap().inner())
            .await
            .unwrap();
        assert_eq!(delivery::process(&first).await.unwrap().attempts, attempt);
    }
    assert_eq!(
        delivery::process(&first).await.unwrap().attempts,
        delivery::MAX_ATTEMPTS
    );
    let capture = Mail::fake();
    assert_eq!(delivery::retry(&first).await.unwrap().status, "sent");
    assert_eq!(capture.captured().len(), 1);
    let sent = &capture.captured()[0];
    assert_eq!(sent.to[0].email, member.email);
    assert!(
        sent.text
            .as_deref()
            .unwrap()
            .contains("/dashboard/listings/")
    );
    assert_eq!(delivery::retry(&first).await.unwrap().status, "sent");
    assert_eq!(capture.captured().len(), 1);
    let next = delivery::status(100).await.unwrap()[0].id.clone();
    // Two workers may select the same row; only one can hold its live lease.
    let (a, b) = tokio::join!(delivery::process(&next), delivery::process(&next));
    a.unwrap();
    b.unwrap();
    assert_eq!(capture.captured().len(), 2);
    let stale = delivery::status(100).await.unwrap()[0].id.clone();
    notice::Entity::update_many()
        .col_expr(notice::Column::Status, Expr::value("delivering"))
        .col_expr(notice::Column::LeaseToken, Expr::value("dead-worker"))
        .col_expr(notice::Column::LeaseUntil, Expr::value(0))
        .filter(notice::Column::Id.eq(&stale))
        .exec(DB::connection().unwrap().inner())
        .await
        .unwrap();
    assert_eq!(delivery::process(&stale).await.unwrap().status, "sent");
    assert!(delivery::pending(0).await.is_err());
    assert!(delivery::status(101).await.is_err());
    let remaining = delivery::pending(100).await.unwrap();
    assert!(remaining.iter().all(|n| n.status == "sent"));
    payment_notifications(admin.id, member.id, id).await;
    // Bounded owner history independently scopes the owner and listing.
    let mut extra = content(0, category);
    extra.title = "Bounded history fixture".into();
    let second = workflow::save(member.id, None, extra).await.unwrap();
    let db = DB::connection().unwrap();
    let tx = db.inner().begin().await.unwrap();
    for n in 0..25 {
        notifications::record(
            &tx,
            notifications::Intent {
                event_key: &format!("bounded:{n}"),
                owner_id: member.id,
                listing_id: second,
                purchase_id: None,
                title: "Bounded fixture",
                body: "Saved",
            },
        )
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();
    assert_eq!(
        notifications::recent(member.id, second)
            .await
            .unwrap()
            .len(),
        20
    );
    assert!(
        notifications::recent(other.id, second)
            .await
            .unwrap()
            .is_empty()
    );
    println!(
        "Moderation/payment intents, rollback, replay, restart, captured mail, retry limits, leases and private bounded owner history passed."
    );
}
async fn payment_notifications(admin: i64, owner: i64, listing_id: i64) {
    use directory::billing::{
        self, Mode, checkout,
        lifecycle_entities::{purchase, receipt},
        plans, reconcile,
    };
    let gateway = std::sync::Arc::new(fixtures::FakeGateway::default());
    plans::save(admin,None,serde_json::from_value(json!({"key":"adoption-paid","name":"Adoption plan","description":"Fixture","enabled":true,"billing_type":"one_time","amount":1000,"currency":"USD","version":0})).unwrap()).await.unwrap();
    billing::save(Mode::Test,serde_json::from_value(json!({"revision":0,"default_provider":"stripe","stripe":{"enabled":true,"public_key":"pk_test_ADOPTION_PUBLIC","api_key":"sk_test_ADOPTION_SECRET","webhook_key":fixtures::STRIPE_SIGNING,"clear_secrets":false},"paddle":{"enabled":false,"public_key":"","api_key":"","webhook_key":"","clear_secrets":false},"mappings":[{"plan":"adoption-paid","stripe":"price_adoption","paddle":""}]})).unwrap()).await.unwrap();
    let id = checkout::start(
        owner,
        listing_id,
        checkout::StartPurchase {
            plan_key: "adoption-paid".into(),
            provider: Some(billing::Provider::Stripe),
        },
        gateway.as_ref(),
    )
    .await
    .unwrap();
    let db = DB::connection().unwrap();
    let row = purchase::Entity::find_by_id(&id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    let checkout_count = count().await;
    assert!(checkout_count > 4);
    let resource = gateway.settle(&row, 0, chrono::Utc::now().timestamp(), None);
    sql("CREATE TRIGGER reject_payment_notice BEFORE INSERT ON owner_notifications WHEN NEW.event_key LIKE 'payment-event:%' BEGIN SELECT RAISE(ABORT, 'controlled outbox fault'); END").await;
    let failed = reconcile::recover(&id, Some(&resource), gateway.as_ref(), false)
        .await
        .unwrap();
    assert_ne!(failed.status, "applied");
    assert_eq!(
        purchase::Entity::find_by_id(&id)
            .one(db.inner())
            .await
            .unwrap()
            .unwrap()
            .state,
        "open"
    );
    assert!(
        receipt::Entity::find_by_id(&failed.event_id)
            .one(db.inner())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(count().await, checkout_count);
    sql("DROP TRIGGER reject_payment_notice").await;
    assert_eq!(
        reconcile::process_event(&failed.event_id, gateway.as_ref(), true, false)
            .await
            .unwrap()
            .status,
        "applied"
    );
    assert_eq!(count().await, checkout_count + 1);
    assert_eq!(
        reconcile::process_event(&failed.event_id, gateway.as_ref(), true, false)
            .await
            .unwrap()
            .status,
        "applied"
    );
    assert_eq!(count().await, checkout_count + 1);
    let charge = fixtures::reference(&row, "ch", 0);
    for (refunded, disputed, expected) in [
        (400, false, "A refund was recorded"),
        (0, true, "An open dispute suspended"),
    ] {
        let mut state = gateway.get("stripe", "charge", &charge);
        state["amount_refunded"] = json!(refunded);
        state["disputed"] = json!(disputed);
        gateway.set("stripe", "charge", &charge, state);
        gateway.set("stripe", "disputes", &charge, if disputed { json!([{"id":fixtures::reference(&row,"dp",0),"charge":charge,"livemode":false,"currency":"usd","status":"needs_response"}]) } else { json!([]) });
        let result = reconcile::recover(&id, None, gateway.as_ref(), false)
            .await
            .unwrap();
        assert_eq!(result.status, "applied");
        let notification = notice::Entity::find()
            .filter(notice::Column::EventKey.eq(format!("payment-event:{}", result.event_id)))
            .one(db.inner())
            .await
            .unwrap()
            .unwrap();
        assert!(
            notification.body.contains(expected),
            "{}",
            notification.body
        );
    }
    // Retain the newer resolved observation before releasing the older open dispute.
    let gate = gateway.gate_read("stripe", "disputes", &charge);
    let older_gateway = gateway.clone();
    let older_id = id.clone();
    let older = tokio::spawn(async move {
        reconcile::recover(&older_id, None, older_gateway.as_ref(), false)
            .await
            .unwrap()
    });
    gate.arrived.notified().await;
    let mut state = gateway.get("stripe", "charge", &charge);
    state["disputed"] = json!(false);
    gateway.set("stripe", "charge", &charge, state);
    gateway.set("stripe", "disputes", &charge, json!([]));
    let newer = reconcile::recover(&id, None, gateway.as_ref(), false)
        .await
        .unwrap();
    assert_eq!(newer.status, "applied");
    let retained_count = count().await;
    gate.release.notify_one();
    let rejected = older.await.unwrap();
    assert_eq!(rejected.status, "applied");
    assert_eq!(
        directory::billing::lifecycle_entities::payment::Entity::find()
            .filter(directory::billing::lifecycle_entities::payment::Column::PurchaseId.eq(&id))
            .one(db.inner())
            .await
            .unwrap()
            .unwrap()
            .status,
        "paid"
    );
    assert_eq!(
        count().await,
        retained_count,
        "stale observation must not create a notice"
    );
    assert!(
        notice::Entity::find()
            .filter(notice::Column::EventKey.eq(format!("payment-event:{}", rejected.event_id)))
            .one(db.inner())
            .await
            .unwrap()
            .is_none()
    );
    let retained = notice::Entity::find()
        .filter(notice::Column::EventKey.eq(format!("payment-event:{}", newer.event_id)))
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    assert!(!retained.body.contains("open dispute suspended"));
    let captured = Mail::fake();
    assert_eq!(
        delivery::process(&retained.id).await.unwrap().status,
        "sent"
    );
    assert_eq!(captured.captured().len(), 1);
    assert!(
        !captured.captured()[0]
            .text
            .as_deref()
            .unwrap()
            .contains("open dispute suspended")
    );
    let rows = notice::Entity::find()
        .filter(notice::Column::PurchaseId.eq(&id))
        .all(db.inner())
        .await
        .unwrap();
    assert!(
        rows.iter()
            .any(|n| n.body.contains("payment was confirmed"))
    );
    for n in rows {
        assert!(!n.body.contains("ADOPTION_SECRET"));
        assert!(!n.body.contains("customer_ref"));
    }
}

#[test]
fn configuration_probe() {
    let Ok(case) = std::env::var("ADOPTION_CONFIG_PROBE") else {
        return;
    };
    if case == "valid" {
        let site = directory::config::site::read().unwrap();
        assert_eq!(site.name, "Configured adoption");
        assert_eq!(site.description, "Configured description");
        assert_eq!(site.origin, "https://configured.example.test");
        assert_eq!(
            site.logo_url.as_deref(),
            Some("https://configured.example.test/logo.png")
        );
        assert_eq!(site.accent, "#673ab7");
    } else {
        let error = directory::config::site::read()
            .err()
            .expect("invalid setting accepted");
        assert!(error.to_string().contains(&case), "{error}");
    }
    println!("Configuration probe completed");
}
#[tokio::test]
async fn storage_probe() {
    let Ok(key) = std::env::var("ADOPTION_MEDIA_PROBE") else {
        return;
    };
    let bytes = directory::listings::media::read(&key).await.unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!(image.as_raw(), &[255, 0, 0, 255]);
    println!("Storage probe completed");
}
#[tokio::test]
async fn configuration_and_storage_contract() {
    let exe = std::env::current_exe().unwrap();
    for (case, value) in [
        ("valid", ""),
        ("APP_NAME", ""),
        ("SITE_ACCENT", "#ffffff"),
        ("APP_URL", "https://user:secret@example.test/path"),
        ("SITE_LOGO_URL", "javascript:alert(1)"),
    ] {
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["configuration_probe", "--exact", "--nocapture"])
            .env("ADOPTION_CONFIG_PROBE", case)
            .env("APP_NAME", "Configured adoption")
            .env("SITE_DESCRIPTION", "Configured description")
            .env("APP_URL", "https://configured.example.test")
            .env("SITE_LOGO_URL", "/logo.png")
            .env("SITE_ACCENT", "#673ab7");
        if case != "valid" {
            cmd.env(case, value);
        }
        let result = cmd.output().unwrap();
        assert!(
            result.status.success(),
            "{case}: {} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("Configuration probe completed"));
    }
    use image::ImageEncoder;
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&[255, 0, 0, 255], 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    let stored = directory::listings::media::persist(&bytes).await.unwrap();
    let result = std::process::Command::new(&exe)
        .args(["storage_probe", "--exact", "--nocapture"])
        .env("ADOPTION_MEDIA_PROBE", &stored.key)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("Storage probe completed"));
}
