#[path = "common/paid_browser.rs"]
mod browser;
#[allow(dead_code)]
mod common;
#[path = "common/payments.rs"]
mod fixtures;

use common::Client;
use directory::{
    billing::{
        self, Mode, Provider, SaveSettings, checkout, events,
        gateway::Gateway,
        lifecycle_entities::{event, payment, plan, purchase, receipt, slot},
        plans, reconcile,
    },
    commands::{
        admin_access::{AccessAction, change_access},
        billing_reconcile::BillingReconcile,
    },
    listings::{
        entities::{entitlement, listing},
        queries, workflow,
    },
    models::user::User,
};
use fixtures::{FakeGateway, PADDLE_SIGNING, STRIPE_SIGNING};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Statement,
    sea_query::Expr,
};
use std::sync::{Arc, atomic::Ordering};
use suprnova::{
    App, DB, TypedCommand,
    eloquent::Model,
    serde_json::{self, Value, json},
};

async fn account(name: &str, verified: bool) -> User {
    let mut user = User::create(
        name,
        &format!("{name}@example.test"),
        "fixture-password-123",
    )
    .await
    .unwrap();
    if verified {
        user.email_verified_at = Some(chrono::Utc::now());
        user.save().await.unwrap();
    }
    user
}
async fn login(user: &User) -> Client {
    let mut http = Client::new();
    http.get("/login").await;
    let response = http
        .post(
            "/login",
            json!({"email":user.email,"password":"fixture-password-123"}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    http
}
async fn listing_row(id: i64) -> listing::Model {
    listing::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
async fn purchase_row(id: &str) -> purchase::Model {
    purchase::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
async fn create(owner: &mut Client, title: &str, category: i64) -> i64 {
    let r=owner.post("/dashboard/listings",json!({"version":0,"title":title,"summary":"Publishing contract fixture","description":"A **reviewed** resource.","url":"https://example.test/item","category_ids":[category],"media_id":null,"media_alt":""})).await;
    assert_eq!(r.status, 302, "{}", r.body);
    r.location
        .unwrap()
        .trim_end_matches("/edit")
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
async fn approve(owner: &mut Client, admin: &mut Client, id: i64) {
    let r = owner
        .post(
            &format!("/dashboard/listings/{id}/submit"),
            json!({"version":listing_row(id).await.version}),
        )
        .await;
    assert_eq!(r.status, 302, "{}", r.body);
    let row = listing_row(id).await;
    let r=admin.post(&format!("/admin/listings/{id}/decision"),json!({"version":row.version,"revision_id":row.current_revision_id,"decision":"approve","reason":""})).await;
    assert_eq!(r.status, 302, "{}", r.body);
}
fn plan_input(key: &str, kind: &str, amount: i64, version: i64) -> Value {
    json!({"key":key,"name":format!("{key} publishing"),"description":"Publishing terms","enabled":true,"billing_type":kind,"amount":amount,"currency":"USD","version":version})
}
fn settings_input(revision: i64, mode: Mode) -> Value {
    let s = mode.as_str();
    let paddle = if mode == Mode::Test { "sdbx" } else { "live" };
    json!({"revision":revision,"default_provider":"stripe","stripe":{"enabled":true,"public_key":format!("pk_{s}_PAYMENT_PUBLIC"),"api_key":format!("sk_{s}_PAYMENT_API_SECRET"),"webhook_key":STRIPE_SIGNING,"clear_secrets":false},"paddle":{"enabled":true,"public_key":format!("{s}_PAYMENT_CLIENT"),"api_key":format!("pdl_{paddle}_apikey_PAYMENT_API_SECRET"),"webhook_key":PADDLE_SIGNING,"clear_secrets":false},"mappings":[{"plan":"once","stripe":"price_once","paddle":"pri_once"},{"plan":"monthly","stripe":"price_monthly","paddle":"pri_monthly"},{"plan":"annual","stripe":"price_annual","paddle":"pri_annual"}]})
}
async fn settings(mode: Mode, value: Value) {
    billing::save(mode, serde_json::from_value::<SaveSettings>(value).unwrap())
        .await
        .unwrap();
}
async fn start(http: &mut Client, id: i64, key: &str, provider: Option<&str>) -> String {
    let response = http
        .post(
            &format!("/dashboard/listings/{id}/checkout"),
            json!({"plan_key":key,"provider":provider}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    response
        .location
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap()
        .to_owned()
}
async fn expiry(id: &str) {
    purchase::Entity::update_many()
        .col_expr(purchase::Column::LeaseUntil, Expr::value(0))
        .filter(purchase::Column::Id.eq(id))
        .exec(DB::connection().unwrap().inner())
        .await
        .unwrap();
}
async fn count_entitlements(id: i64) -> u64 {
    entitlement::Entity::find()
        .filter(entitlement::Column::ListingId.eq(id))
        .count(DB::connection().unwrap().inner())
        .await
        .unwrap()
}
async fn visible(id: i64, expected: bool) {
    let now = chrono::Utc::now().timestamp();
    let row = listing_row(id).await;
    let (search, _) = queries::search(
        "",
        "",
        queries::Page {
            number: 1,
            size: 100,
        },
        now,
    )
    .await
    .unwrap();
    let (category, _) = queries::search(
        "",
        "software",
        queries::Page {
            number: 1,
            size: 100,
        },
        now,
    )
    .await
    .unwrap();
    assert_eq!(
        search.iter().any(|x| x.id == id),
        expected,
        "search eligibility {id}"
    );
    assert_eq!(
        category.iter().any(|x| x.id == id),
        expected,
        "category eligibility {id}"
    );
    assert_eq!(
        queries::detail(&row.slug, now).await.is_ok(),
        expected,
        "detail eligibility {id}"
    );
}
async fn signed(
    http: &mut Client,
    p: &purchase::Model,
    kind: &str,
    resource: &str,
    id: &str,
) -> String {
    let now = chrono::Utc::now().timestamp();
    let body = fixtures::event(p, kind, resource, id, now);
    let (header, secret) = if p.provider == "stripe" {
        ("stripe-signature", STRIPE_SIGNING)
    } else {
        ("paddle-signature", PADDLE_SIGNING)
    };
    let signature = fixtures::signature(&p.provider, &body, secret, now);
    let response = http
        .signed_request(
            &format!("/billing/webhooks/{}/{}", p.provider, p.mode),
            body,
            header,
            &signature,
        )
        .await;
    assert_eq!(response.status, 202, "{}", response.body);
    event::Entity::find()
        .filter(event::Column::ProviderEventId.eq(id))
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
        .id
}
async fn processed(id: &str, fake: &FakeGateway) {
    let result = reconcile::process_event(id, fake, true, false)
        .await
        .unwrap();
    assert_eq!(result.status, "applied", "{:?}", result);
}

async fn signature_contract(
    http: &mut Client,
    p: &purchase::Model,
    resource: &str,
    fake: &FakeGateway,
) {
    let now = chrono::Utc::now().timestamp();
    let stripe = p.provider == "stripe";
    let provider = if stripe {
        Provider::Stripe
    } else {
        Provider::Paddle
    };
    let mode = Mode::parse(&p.mode).unwrap();
    let kind = if stripe {
        "checkout.session.completed"
    } else {
        "transaction.completed"
    };
    let header = if stripe {
        "stripe-signature"
    } else {
        "paddle-signature"
    };
    let secret = if stripe {
        STRIPE_SIGNING
    } else {
        PADDLE_SIGNING
    };
    let path = format!("/billing/webhooks/{}/{}", p.provider, p.mode);
    let bytes = fixtures::event(
        p,
        kind,
        resource,
        &format!("evt_signatures_{}", p.provider),
        now,
    );
    for signature in [
        fixtures::signature(&p.provider, &bytes, "wrong-key", now),
        fixtures::signature(&p.provider, &bytes, secret, now - 600),
        if stripe {
            format!("t={now},v1=f")
        } else {
            format!("ts={now};h1=f")
        },
        if stripe {
            format!("t={now},v1=é")
        } else {
            format!("ts={now};h1=é")
        },
    ] {
        assert_eq!(
            http.signed_request(&path, bytes.clone(), header, &signature)
                .await
                .status,
            400
        );
    }
    let sig = fixtures::signature(&p.provider, &bytes, secret, now);
    let first = events::accept(provider, mode, &bytes, &sig).await.unwrap();
    assert_eq!(
        events::accept(provider, mode, &bytes, &sig).await.unwrap(),
        first
    );
    let saved = event::Entity::find_by_id(&first)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.raw_body, bytes);
    let mut altered: Value = serde_json::from_slice(&bytes).unwrap();
    altered["extra"] = json!(true);
    let altered = serde_json::to_vec(&altered).unwrap();
    let signature = fixtures::signature(&p.provider, &altered, secret, now);
    assert!(
        events::accept(provider, mode, &altered, &signature)
            .await
            .is_err()
    );
    assert!(
        events::accept(
            provider,
            mode,
            &vec![b' '; events::MAX_EVENT_BYTES + 1],
            &sig
        )
        .await
        .is_err()
    );
    if stripe {
        let opposite = if mode == Mode::Live {
            Mode::Test
        } else {
            Mode::Live
        };
        assert!(
            events::accept(provider, opposite, &bytes, &sig)
                .await
                .is_err()
        );
    } else {
        // Exercise the exact SDK schema gap and its signature fallback through ingress.
        for action in ["chargeback_warning_reverse", "credit_reverse"] {
            let mut reversal: Value = serde_json::from_slice(&bytes).unwrap();
            reversal["event_id"] = json!(format!("evt_{action}"));
            reversal["event_type"] = json!("adjustment.updated");
            reversal["data"] = json!({"id":format!("adj_{action}"),"action":action,"transaction_id":resource,"customer_id":p.customer_ref,"custom_data":{"purchase_id":p.id}});
            let bytes = serde_json::to_vec(&reversal).unwrap();
            for (key, time, valid) in [
                (secret, now, true),
                ("wrong-key", now, false),
                (secret, now - 600, false),
                (secret, now + 600, false),
            ] {
                let signature = fixtures::signature("paddle", &bytes, key, time);
                assert_eq!(
                    http.signed_request(&path, bytes.clone(), header, &signature)
                        .await
                        .status,
                    if valid { 202 } else { 400 }
                );
            }
            for signature in [format!("ts={now};h1=f"), format!("ts={now};h1=é")] {
                assert_eq!(
                    http.signed_request(&path, bytes.clone(), header, &signature)
                        .await
                        .status,
                    400
                );
            }
        }
    }
    // A real signature authenticates the sender, not an unknown purchase.
    let mut unknown = p.clone();
    unknown.id = uuid::Uuid::new_v4().to_string();
    unknown.customer_ref = Some("cus_unknown".into());
    let bytes = fixtures::event(
        &unknown,
        kind,
        "unknown_resource",
        &format!("evt_unknown_{}", p.provider),
        now,
    );
    let sig = fixtures::signature(&p.provider, &bytes, secret, now);
    let unknown_id = events::accept(provider, mode, &bytes, &sig).await.unwrap();
    let reads = fake.reads.load(Ordering::SeqCst);
    for _ in 0..reconcile::MAX_ATTEMPTS {
        event::Entity::update_many()
            .col_expr(event::Column::NextAttemptAt, Expr::value(0))
            .filter(event::Column::Id.eq(&unknown_id))
            .exec(DB::connection().unwrap().inner())
            .await
            .unwrap();
        let result = reconcile::process_event(&unknown_id, fake, false, false)
            .await
            .unwrap();
        assert_eq!(
            result.error_code.as_deref(),
            Some("purchase_not_correlated")
        );
    }
    let exhausted = event::Entity::find_by_id(&unknown_id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(exhausted.status, "failed");
    assert_eq!(exhausted.attempts, reconcile::MAX_ATTEMPTS);
    reconcile::process_event(&unknown_id, fake, false, false)
        .await
        .unwrap();
    assert_eq!(fake.reads.load(Ordering::SeqCst), reads);
    // An interrupted worker's expired lease can be reclaimed and fulfilled.
    event::Entity::update_many()
        .col_expr(event::Column::Status, Expr::value("processing"))
        .col_expr(event::Column::LeaseToken, Expr::value("crashed-worker"))
        .col_expr(event::Column::LeaseUntil, Expr::value(now - 1))
        .filter(event::Column::Id.eq(&first))
        .exec(DB::connection().unwrap().inner())
        .await
        .unwrap();
    processed(&first, fake).await;
}

#[tokio::test(flavor = "current_thread")]
async fn paid_lifecycle_contract() {
    let _mail = common::setup().await;
    let mode = checkout::checkout_mode().unwrap();
    let live = mode == Mode::Live;
    let fake = Arc::new(FakeGateway::default());
    App::bind::<dyn Gateway>(fake.clone());
    workflow::seed_categories().await.unwrap();
    let category = queries::categories()
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.slug == "software")
        .unwrap()
        .id;
    let owner = account("pay-owner", true).await;
    let other = account("pay-other", true).await;
    let unverified = account("pay-unverified", false).await;
    let admin = account("pay-admin", true).await;
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    let mut owner_http = login(&owner).await;
    let mut other_http = login(&other).await;
    let mut unverified_http = login(&unverified).await;
    let mut admin_http = login(&admin).await;
    let mut guest = Client::new();
    guest.get("/login").await;
    for client in [&mut guest, &mut owner_http] {
        assert_eq!(
            client
                .post("/admin/plans", plan_input("forbidden", "free", 0, 0))
                .await
                .status,
            302
        );
    }
    suprnova::rbac::give_permission_to_model(
        "directory.user",
        &other.id.to_string(),
        "admin.access",
    )
    .await
    .unwrap();
    assert_eq!(
        other_http
            .post("/admin/plans", plan_input("forbidden", "free", 0, 0))
            .await
            .status,
        403
    );
    assert_eq!(
        admin_http
            .request(
                "POST",
                "/admin/plans",
                Some(plan_input("csrf", "free", 0, 0)),
                false
            )
            .await
            .status,
        419
    );
    assert_eq!(
        admin_http
            .post("/admin/plans", plan_input("bad", "monthly", 0, 0))
            .await
            .status,
        422
    );
    for (key, kind, amount) in [
        ("free", "free", 0),
        ("once", "one_time", 1000),
        ("monthly", "monthly", 1000),
        ("annual", "annual", 1000),
    ] {
        let r = admin_http
            .post("/admin/plans", plan_input(key, kind, amount, 0))
            .await;
        assert_eq!(r.status, 302, "{}", r.body);
    }
    assert_eq!(plans::list(admin.id).await.unwrap().len(), 4);
    let db = DB::connection().unwrap();
    db.inner().execute_raw(Statement::from_string(sea_orm::DatabaseBackend::Sqlite, "CREATE TRIGGER fail_plan_audit BEFORE INSERT ON administrative_audit BEGIN SELECT RAISE(ABORT, 'controlled plan audit interruption'); END")).await.unwrap();
    assert_eq!(
        admin_http
            .post("/admin/plans", plan_input("rollback", "free", 0, 0))
            .await
            .status,
        500
    );
    assert_eq!(
        plans::list(admin.id).await.unwrap().len(),
        4,
        "an audit failure must roll back the plan"
    );
    db.inner()
        .execute_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "DROP TRIGGER fail_plan_audit",
        ))
        .await
        .unwrap();
    assert_eq!(
        admin_http
            .post("/admin/plans", plan_input("once", "one_time", 1000, 0))
            .await
            .status,
        422
    );
    let version = plan::Entity::find_by_id("once")
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
        .version;
    let (first, second) = tokio::join!(
        plans::save(
            admin.id,
            Some("once"),
            serde_json::from_value(plan_input("once", "one_time", 1000, version)).unwrap()
        ),
        plans::save(
            admin.id,
            Some("once"),
            serde_json::from_value(plan_input("once", "one_time", 1000, version)).unwrap()
        )
    );
    assert_ne!(
        first.is_ok(),
        second.is_ok(),
        "one conflicting plan save must lose"
    );
    for m in [Mode::Test, Mode::Live] {
        settings(
            m,
            settings_input(billing::load(m).await.unwrap().revision, m),
        )
        .await;
    }

    // Approval, ownership, CSRF and input boundaries are exercised through HTTP.
    let draft = create(&mut owner_http, "Pending paid resource", category).await;
    for provider in ["stripe", "paddle"] {
        assert_eq!(
            owner_http
                .post(
                    &format!("/dashboard/listings/{draft}/checkout"),
                    json!({"plan_key":"once","provider":provider})
                )
                .await
                .status,
            422
        );
    }
    let unapproved = create(&mut owner_http, "Unapproved payment states", category).await;
    assert_eq!(
        owner_http
            .post(
                &format!("/dashboard/listings/{unapproved}/submit"),
                json!({"version":listing_row(unapproved).await.version})
            )
            .await
            .status,
        302
    );
    for state in ["submitted", "rejected"] {
        if state == "rejected" {
            let row = listing_row(unapproved).await;
            assert_eq!(admin_http.post(&format!("/admin/listings/{unapproved}/decision"), json!({"version":row.version,"revision_id":row.current_revision_id,"decision":"reject","reason":"Please revise before payment."})).await.status, 302);
        }
        for provider in ["stripe", "paddle"] {
            assert_eq!(
                owner_http
                    .post(
                        &format!("/dashboard/listings/{unapproved}/checkout"),
                        json!({"plan_key":"once","provider":provider})
                    )
                    .await
                    .status,
                422,
                "{state} cannot start checkout"
            );
        }
    }
    approve(&mut owner_http, &mut admin_http, draft).await;
    let mut disabled = plan_input("disabled", "one_time", 1000, 0);
    disabled["enabled"] = json!(false);
    assert_eq!(admin_http.post("/admin/plans", disabled).await.status, 302);
    assert_eq!(
        admin_http
            .post("/admin/plans", plan_input("unmapped", "one_time", 1000, 0))
            .await
            .status,
        302
    );
    for key in ["disabled", "unmapped"] {
        for provider in ["stripe", "paddle"] {
            assert_eq!(
                owner_http
                    .post(
                        &format!("/dashboard/listings/{draft}/checkout"),
                        json!({"plan_key":key,"provider":provider})
                    )
                    .await
                    .status,
                422
            );
        }
    }
    for client in [&mut other_http, &mut unverified_http] {
        assert!(matches!(
            client
                .post(
                    &format!("/dashboard/listings/{draft}/checkout"),
                    json!({"plan_key":"once","provider":"stripe"})
                )
                .await
                .status,
            403 | 404
        ));
    }
    assert_eq!(
        owner_http
            .request(
                "POST",
                &format!("/dashboard/listings/{draft}/checkout"),
                Some(json!({"plan_key":"once","provider":"stripe"})),
                false
            )
            .await
            .status,
        419
    );
    for field in [
        "price",
        "currency",
        "customer",
        "return_url",
        "mode",
        "amount",
    ] {
        let mut data = json!({"plan_key":"once","provider":"stripe"});
        data[field] = json!("attacker");
        assert_eq!(
            owner_http
                .post(&format!("/dashboard/listings/{draft}/checkout"), data)
                .await
                .status,
            422
        );
    }
    assert_eq!(fake.start_count(), 0);
    assert_eq!(count_entitlements(draft).await, 0);
    visible(draft, false).await;

    // Free is an entitlement, never a synthetic payment or provider request.
    let free = start(&mut owner_http, draft, "free", None).await;
    assert_eq!(count_entitlements(draft).await, 1);
    visible(draft, true).await;
    assert_eq!(
        payment::Entity::find()
            .filter(payment::Column::PurchaseId.eq(&free))
            .count(DB::connection().unwrap().inner())
            .await
            .unwrap(),
        0
    );
    assert_eq!(fake.start_count(), 0);

    for provider in ["stripe", "paddle"] {
        let listing = create(
            &mut owner_http,
            &format!("{provider} one time resource"),
            category,
        )
        .await;
        approve(&mut owner_http, &mut admin_http, listing).await;
        let before = fake.start_count();
        let mut a = owner_http.clone();
        let mut b = owner_http.clone();
        let (one, two) = tokio::join!(
            start(&mut a, listing, "once", Some(provider)),
            start(&mut b, listing, "once", Some(provider))
        );
        assert_eq!(one, two);
        assert_eq!(
            fake.start_count(),
            before + 1,
            "concurrent requests share one payable session"
        );
        let p = purchase_row(&one).await;
        assert_eq!(p.mode, mode.as_str());
        assert_eq!(p.amount, 1000);
        assert_eq!(p.provider, provider);
        assert_eq!(
            slot::Entity::find()
                .filter(slot::Column::ListingId.eq(listing))
                .count(DB::connection().unwrap().inner())
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            other_http
                .get(&format!("/dashboard/purchases/{}", p.id))
                .await
                .status,
            404
        );
        let shown = owner_http
            .get(&format!(
                "/dashboard/purchases/{}?paid=true&session_id=forged",
                p.id
            ))
            .await;
        assert_eq!(shown.status, 200, "{}", shown.body);
        assert!(!shown.body.contains("PAYMENT_API_SECRET"));
        assert!(!shown.body.contains(STRIPE_SIGNING));
        assert_eq!(count_entitlements(listing).await, 0);
        visible(listing, false).await;
        let resource = fake.settle(&p, 0, chrono::Utc::now().timestamp() - 30, None);
        // Signature and crashed-worker checks run after the initial no-fulfillment assertions below.
        let kind = if provider == "stripe" {
            "checkout.session.completed"
        } else {
            "transaction.completed"
        };
        let now = chrono::Utc::now().timestamp();
        let body = fixtures::event(&p, kind, &resource, &format!("evt_bad_{provider}"), now);
        let header = if provider == "stripe" {
            "stripe-signature"
        } else {
            "paddle-signature"
        };
        let response = guest
            .signed_request(
                &format!("/billing/webhooks/{provider}/{}", p.mode),
                body.clone(),
                header,
                "bad",
            )
            .await;
        assert_eq!(response.status, 400);
        assert_eq!(count_entitlements(listing).await, 0);
        let reads = fake.reads.load(Ordering::SeqCst);
        let eid = signed(
            &mut guest,
            &p,
            kind,
            &resource,
            &format!("evt_paid_{provider}"),
        )
        .await;
        assert_eq!(
            fake.reads.load(Ordering::SeqCst),
            reads,
            "acceptance must not contact the gateway"
        );
        assert_eq!(count_entitlements(listing).await, 0);
        // Signed labels are insufficient: wrong actual price fails closed.
        let original = fake.get(provider, "checkout", &resource);
        for field in [
            if provider == "stripe" {
                "customer"
            } else {
                "customer_id"
            },
            if provider == "stripe" {
                "currency"
            } else {
                "currency_code"
            },
        ] {
            let mut wrong = original.clone();
            wrong[field] = json!(if field.starts_with("currency") {
                "EUR"
            } else {
                "cus_other"
            });
            fake.set(provider, "checkout", &resource, wrong);
            assert_eq!(
                reconcile::process_event(&eid, fake.as_ref(), true, false)
                    .await
                    .unwrap()
                    .status,
                "deferred"
            );
            assert_eq!(count_entitlements(listing).await, 0);
        }
        let mut wrong = original.clone();
        if provider == "stripe" {
            wrong["line_items"]["data"][0]["price"]["id"] = json!("price_other");
        } else {
            wrong["items"][0]["price"]["id"] = json!("pri_other");
        }
        fake.set(provider, "checkout", &resource, wrong);
        let failed = reconcile::process_event(&eid, fake.as_ref(), true, false)
            .await
            .unwrap();
        assert_eq!(failed.status, "deferred");
        assert_eq!(count_entitlements(listing).await, 0);
        fake.set(provider, "checkout", &resource, original);
        // Simulate interruption at receipt insertion: no payment or entitlement can commit alone.
        DB::connection().unwrap().inner().execute_raw(Statement::from_string(sea_orm::DatabaseBackend::Sqlite,"CREATE TRIGGER fail_payment_receipt BEFORE INSERT ON publishing_receipts BEGIN SELECT RAISE(ABORT, 'controlled fulfillment interruption'); END" )).await.unwrap();
        let interrupted = reconcile::process_event(&eid, fake.as_ref(), true, false)
            .await
            .unwrap();
        assert_eq!(
            interrupted.error_code.as_deref(),
            Some("fulfillment_transaction_failed")
        );
        assert_eq!(count_entitlements(listing).await, 0);
        DB::connection()
            .unwrap()
            .inner()
            .execute_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "DROP TRIGGER fail_payment_receipt",
            ))
            .await
            .unwrap();
        processed(&eid, &fake).await;
        visible(listing, live).await;
        assert_eq!(count_entitlements(listing).await, 1);
        assert!(
            receipt::Entity::find_by_id(&eid)
                .one(DB::connection().unwrap().inner())
                .await
                .unwrap()
                .is_some()
        );
        let paid = payment::Entity::find()
            .filter(payment::Column::PurchaseId.eq(&p.id))
            .one(DB::connection().unwrap().inner())
            .await
            .unwrap()
            .unwrap();
        let before_start = fake.start_count();
        let (r1, r2) = tokio::join!(
            reconcile::recover(&p.id, Some(&resource), fake.as_ref(), false),
            reconcile::recover(&p.id, Some(&resource), fake.as_ref(), false)
        );
        assert_eq!(r1.unwrap().status, "applied");
        assert_eq!(r2.unwrap().status, "applied");
        assert_eq!(fake.start_count(), before_start);
        assert_eq!(count_entitlements(listing).await, 1);
        let replay = payment::Entity::find_by_id(&paid.id)
            .one(DB::connection().unwrap().inner())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            paid.period_start, replay.period_start,
            "one-time start does not slide with replay"
        );
        let owner_list = queries::owner_listing(owner.id, listing).await.unwrap();
        assert_eq!(owner_list.purchase_id.as_deref(), Some(p.id.as_str()));
        assert_eq!(owner_list.next_action, "manage_payment");
        signature_contract(&mut guest, &p, &resource, fake.as_ref()).await;

        if provider == "stripe" {
            let charge = fixtures::reference(&p, "ch", 0);
            let mut state = fake.get(provider, "charge", &charge);
            state["amount_refunded"] = json!(400);
            fake.set(provider, "charge", &charge, state.clone());
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, live).await;
            let dispute = fixtures::reference(&p, "dp", 0);
            state["disputed"] = json!(true);
            fake.set(provider, "charge", &charge, state.clone());
            let mut d = json!({"id":dispute,"charge":charge,"livemode":live,"currency":"usd","status":"needs_response"});
            fake.set(provider, "disputes", &charge, json!([d]));
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, false).await;
            d["status"] = json!("won");
            fake.set(provider, "disputes", &charge, json!([d]));
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, live).await;
            // A slow older authoritative read must not restore a newer full refund.
            let gate = fake.gate_read(provider, "disputes", &charge);
            let oldfake = fake.clone();
            let id = p.id.clone();
            let older = tokio::spawn(async move {
                reconcile::recover(&id, None, oldfake.as_ref(), false)
                    .await
                    .unwrap()
            });
            gate.arrived.notified().await;
            state["amount_refunded"] = json!(1100);
            fake.set(provider, "charge", &charge, state);
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            gate.release.notify_one();
            assert_eq!(older.await.unwrap().status, "applied");
            visible(listing, false).await;
        } else {
            let mut transaction = fake.get(provider, "checkout", &resource);
            let line = transaction["details"]["line_items"][0]["id"].clone();
            let adjustment = |id: &str, action: &str, status: &str, amount: i64| json!({"id":id,"transaction_id":resource,"customer_id":p.customer_ref,"subscription_id":null,"currency_code":"USD","action":action,"status":status,"type":if amount==1100 {"full"} else {"partial"},"totals":{"total":amount.to_string()},"items":[{"item_id":line}]});
            transaction["details"]["adjusted_totals"]["grand_total"] = json!("700");
            fake.set(provider, "checkout", &resource, transaction.clone());
            fake.set(
                provider,
                "adjustments",
                &resource,
                json!([adjustment("adj_partial", "refund", "approved", 400)]),
            );
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, live).await;
            fake.set(
                provider,
                "adjustments",
                &resource,
                json!([
                    adjustment("adj_partial", "refund", "approved", 400),
                    adjustment("adj_dispute", "chargeback_warning", "approved", 1100)
                ]),
            );
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, false).await;
            fake.set(
                provider,
                "adjustments",
                &resource,
                json!([
                    adjustment("adj_partial", "refund", "approved", 400),
                    adjustment("adj_dispute", "chargeback_warning", "reversed", 1100),
                    adjustment(
                        "adj_reverse",
                        "chargeback_warning_reverse",
                        "approved",
                        1100
                    )
                ]),
            );
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, live).await;
            transaction["details"]["adjusted_totals"]["grand_total"] = json!("0");
            fake.set(provider, "checkout", &resource, transaction);
            fake.set(
                provider,
                "adjustments",
                &resource,
                json!([adjustment("adj_full", "refund", "approved", 1100)]),
            );
            assert_eq!(
                reconcile::recover(&p.id, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(listing, false).await;
        }
        assert_eq!(
            checkout::view(owner.id, &p.id)
                .await
                .unwrap()
                .payment_status,
            "refunded"
        );
    }
    renewal_contract(
        &owner,
        &mut owner_http,
        &mut admin_http,
        &mut guest,
        category,
        &fake,
        mode,
    )
    .await;
    recovery_contract(
        &owner,
        &mut owner_http,
        &mut admin_http,
        &mut guest,
        category,
        &fake,
        mode,
    )
    .await;
    let count = plans::list(admin.id).await.unwrap().len();
    for n in count..99 {
        plans::save(
            admin.id,
            None,
            serde_json::from_value(plan_input(&format!("capacity-{n}"), "free", 0, 0)).unwrap(),
        )
        .await
        .unwrap();
    }
    let (first, second) = tokio::join!(
        plans::save(
            admin.id,
            None,
            serde_json::from_value(plan_input("capacity-a", "free", 0, 0)).unwrap()
        ),
        plans::save(
            admin.id,
            None,
            serde_json::from_value(plan_input("capacity-b", "free", 0, 0)).unwrap()
        ),
    );
    assert_ne!(
        first.is_ok(),
        second.is_ok(),
        "concurrent creation must respect the 100-plan bound"
    );
    assert_eq!(
        plan::Entity::find()
            .count(DB::connection().unwrap().inner())
            .await
            .unwrap(),
        100
    );
    println!(
        "paid lifecycle: plans, HTTP ownership/CSRF, immutable checkout, authenticated acceptance, crash rollback, replay, refunds and disputes passed for {}",
        mode.as_str()
    );
}

async fn renewal_contract(
    owner: &User,
    owner_http: &mut Client,
    admin_http: &mut Client,
    guest: &mut Client,
    category: i64,
    fake: &Arc<FakeGateway>,
    mode: Mode,
) {
    let live = mode == Mode::Live;
    for provider in ["stripe", "paddle"] {
        for plan_key in ["monthly", "annual"] {
            let id = create(
                owner_http,
                &format!("{provider} {plan_key} resource"),
                category,
            )
            .await;
            approve(owner_http, admin_http, id).await;
            let purchase = start(owner_http, id, plan_key, Some(provider)).await;
            let p = purchase_row(&purchase).await;
            let now = chrono::Utc::now().timestamp();
            let first = fake.settle(&p, 0, now - 7200, Some(now - 3600));
            let kind = if provider == "stripe" {
                "invoice.paid"
            } else {
                "transaction.completed"
            };
            let eid = signed(
                guest,
                &p,
                kind,
                &first,
                &format!("evt_{provider}_{plan_key}_old"),
            )
            .await;
            processed(&eid, fake).await;
            visible(id, false).await;
            assert_eq!(
                checkout::view(owner.id, &purchase)
                    .await
                    .unwrap()
                    .paid_through,
                Some(now - 3600)
            );
            let p = purchase_row(&purchase).await;
            assert!(p.subscription_ref.is_some());
            let second = fake.settle(&p, 1, now - 3600, Some(now + 3600));
            let resource_kind = if provider == "stripe" {
                "invoice"
            } else {
                "checkout"
            };
            let settled = fake.get(provider, resource_kind, &second);
            let mut unpaid = settled.clone();
            unpaid["status"] = json!(if provider == "stripe" {
                "open"
            } else {
                "billed"
            });
            fake.set(provider, resource_kind, &second, unpaid);
            let renewal = signed(
                guest,
                &p,
                kind,
                &second,
                &format!("evt_{provider}_{plan_key}_renewal"),
            )
            .await;
            assert_eq!(
                reconcile::process_event(&renewal, fake.as_ref(), true, false)
                    .await
                    .unwrap()
                    .status,
                "deferred"
            );
            assert_eq!(count_entitlements(id).await, 1);
            visible(id, false).await;
            fake.set(provider, resource_kind, &second, settled);
            processed(&renewal, fake).await;
            assert_eq!(count_entitlements(id).await, 2);
            visible(id, live).await;
            assert_eq!(
                checkout::view(owner.id, &purchase)
                    .await
                    .unwrap()
                    .paid_through,
                Some(now + 3600)
            );
            // Reordered old invoices use their absolute period, never replace or add to the new boundary.
            assert_eq!(
                reconcile::recover(&purchase, Some(&first), fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            assert_eq!(count_entitlements(id).await, 2);
            assert_eq!(
                checkout::view(owner.id, &purchase)
                    .await
                    .unwrap()
                    .paid_through,
                Some(now + 3600)
            );
            let starts = fake.start_count();
            assert_eq!(
                owner_http
                    .post(
                        &format!("/dashboard/listings/{id}/checkout"),
                        json!({"plan_key":"once","provider":provider})
                    )
                    .await
                    .status,
                422
            );
            assert_eq!(fake.start_count(), starts);
            let cancel_path = format!("/dashboard/purchases/{purchase}/cancel");
            assert_eq!(
                owner_http
                    .request("POST", &cancel_path, Some(json!({})), false)
                    .await
                    .status,
                419
            );
            assert_eq!(
                owner_http
                    .post(&cancel_path, json!({"at_period_end":false}))
                    .await
                    .status,
                422
            );
            // The provider can acknowledge a request without scheduling it.
            // Until read-back confirms a date, the owner must not see "scheduled".
            let sub = p.subscription_ref.as_deref().unwrap();
            let active = fake.get(provider, "subscription", sub);
            assert_eq!(owner_http.post(&cancel_path, json!({})).await.status, 302);
            let requested = checkout::view(owner.id, &purchase).await.unwrap();
            assert!(requested.cancel_requested);
            assert_eq!(requested.cancel_at, None);
            fake.set(provider, "subscription", sub, active);
            assert_eq!(
                reconcile::recover(&purchase, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            let confirmed_active = checkout::view(owner.id, &purchase).await.unwrap();
            assert!(!confirmed_active.cancel_requested);
            assert!(confirmed_active.can_cancel);
            let before = fake.cancels.load(Ordering::SeqCst);
            assert_eq!(owner_http.post(&cancel_path, json!({})).await.status, 302);
            assert_eq!(fake.cancels.load(Ordering::SeqCst), before + 1);
            let marked = purchase_row(&purchase).await;
            assert!(marked.cancel_requested);
            visible(id, live).await;
            assert_eq!(
                reconcile::recover(&purchase, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            assert_eq!(purchase_row(&purchase).await.cancel_at, Some(now + 3600));
            visible(id, live).await;
            assert_eq!(owner_http.post(&cancel_path, json!({})).await.status, 302);
            assert_eq!(
                fake.cancels.load(Ordering::SeqCst),
                before + 1,
                "scheduled cancellation is not sent twice"
            );
            // Immediate provider cancellation caps existing paid periods at its effective time.
            let sub = marked.subscription_ref.unwrap();
            let mut canceled = fake.get(provider, "subscription", &sub);
            canceled["status"] = json!("canceled");
            if provider == "stripe" {
                canceled["ended_at"] = json!(now - 1);
                canceled["canceled_at"] = json!(now - 200);
            } else {
                canceled["canceled_at"] = json!(fixtures::iso(now - 1));
            }
            fake.set(provider, "subscription", &sub, canceled);
            assert_eq!(
                reconcile::recover(&purchase, None, fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(id, false).await;
            assert_eq!(purchase_row(&purchase).await.state, "canceled");
            assert!(
                slot::Entity::find_by_id(id)
                    .one(DB::connection().unwrap().inner())
                    .await
                    .unwrap()
                    .is_none()
            );
            // A later replay of the old payment cannot cross the terminal cutoff.
            assert_eq!(
                reconcile::recover(&purchase, Some(&first), fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            visible(id, false).await;
            let replacement = start(owner_http, id, "free", None).await;
            assert_ne!(replacement, purchase);
            visible(id, true).await;
            assert_eq!(
                reconcile::recover(&purchase, Some(&second), fake.as_ref(), false)
                    .await
                    .unwrap()
                    .status,
                "applied"
            );
            assert_eq!(
                slot::Entity::find_by_id(id)
                    .one(DB::connection().unwrap().inner())
                    .await
                    .unwrap()
                    .unwrap()
                    .purchase_id,
                replacement
            );
            visible(id, true).await;
        }
    }
}

async fn recovery_contract(
    owner: &User,
    owner_http: &mut Client,
    admin_http: &mut Client,
    guest: &mut Client,
    category: i64,
    fake: &Arc<FakeGateway>,
    mode: Mode,
) {
    for provider in ["stripe", "paddle"] {
        let id = create(
            owner_http,
            &format!("{provider} ambiguous checkout"),
            category,
        )
        .await;
        approve(owner_http, admin_http, id).await;
        fake.fail_start.store(1, Ordering::SeqCst);
        let before = fake.start_count();
        let purchase = start(owner_http, id, "once", Some(provider)).await;
        let original = purchase_row(&purchase).await;
        assert_eq!(original.state, "checkout_creating");
        assert!(original.session_ref.is_none());
        assert_eq!(fake.start_count(), before + 1);
        checkout::continue_purchase(owner.id, &purchase, fake.as_ref())
            .await
            .unwrap();
        assert_eq!(
            fake.start_count(),
            before + 1,
            "an unexpired create lease cannot retry"
        );
        expiry(&purchase).await;
        checkout::continue_purchase(owner.id, &purchase, fake.as_ref())
            .await
            .unwrap();
        assert_eq!(
            fake.start_count(),
            before + if provider == "stripe" { 2 } else { 1 },
            "only explicit Stripe idempotency permits an ambiguous create retry"
        );
        let history = fake
            .starts
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _)| id == &purchase)
            .map(|(_, request)| request.clone())
            .collect::<Vec<_>>();
        assert_eq!(history[0]["price_refs"], json!([original.price_id]));
        assert_eq!(history[0]["customer_ref"], json!(original.customer_ref));
        assert_eq!(history[0]["amount_hint"], Value::Null);
        if provider == "stripe" {
            assert_eq!(history[0], history[1]);
            assert_eq!(history[0]["idempotency_key"], purchase);
        } else {
            assert_eq!(history[0]["idempotency_key"], Value::Null);
        }
        let p = purchase_row(&purchase).await;
        let resource = fake.settle(&p, 0, chrono::Utc::now().timestamp() - 30, None);
        let calls = fake.start_count();
        let repaired = reconcile::recover(&purchase, Some(&resource), fake.as_ref(), false)
            .await
            .unwrap();
        assert_eq!(repaired.status, "applied", "{:?}", repaired);
        assert_eq!(fake.start_count(), calls);
        assert_eq!(count_entitlements(id).await, 1);
        visible(id, mode == Mode::Live).await;
        assert_eq!(
            purchase_row(&purchase).await.session_ref.as_deref(),
            Some(resource.as_str())
        );

        // Existing purchases retain immutable terms and encrypted account material.
        let held = purchase_row(&purchase).await;
        let terms = plan::Entity::find_by_id("once")
            .one(DB::connection().unwrap().inner())
            .await
            .unwrap()
            .unwrap();
        let mut edited = plan_input("once", "one_time", 2000, terms.version);
        edited["enabled"] = json!(false);
        assert_eq!(
            admin_http.post("/admin/plans/once", edited).await.status,
            302
        );
        let settings_revision = billing::load(mode).await.unwrap().revision;
        let mut cleared = settings_input(settings_revision, mode);
        cleared[provider] = json!({"enabled":false,"public_key":"","api_key":"","webhook_key":"","clear_secrets":true});
        cleared["default_provider"] = json!(if provider == "stripe" {
            "paddle"
        } else {
            "stripe"
        });
        cleared["mappings"][0][provider] = json!(if provider == "stripe" {
            "price_changed"
        } else {
            "pri_changed"
        });
        settings(mode, cleared).await;
        let restored = purchase_row(&purchase).await;
        assert_eq!(held.credentials, restored.credentials);
        assert_eq!(held.amount, restored.amount);
        assert_eq!(held.price_id, restored.price_id);
        assert_eq!(held.currency, restored.currency);
        let kind = if provider == "stripe" {
            "checkout.session.completed"
        } else {
            "transaction.completed"
        };
        let eid = signed(
            guest,
            &restored,
            kind,
            &resource,
            &format!("evt_retained_{provider}"),
        )
        .await;
        processed(&eid, fake).await;
        assert_eq!(count_entitlements(id).await, 1);
        // The owner sees disabled plans as unavailable; retained fulfillment still succeeds.
        let fresh = create(owner_http, &format!("{provider} disabled plan"), category).await;
        approve(owner_http, admin_http, fresh).await;
        assert_eq!(
            owner_http
                .post(
                    &format!("/dashboard/listings/{fresh}/checkout"),
                    json!({"plan_key":"once","provider":provider})
                )
                .await
                .status,
            422
        );
        assert!(
            !checkout::offers(owner.id, fresh)
                .await
                .unwrap()
                .plans
                .iter()
                .any(|plan| plan.key == "once")
        );
        let mut replacement = settings_input(billing::load(mode).await.unwrap().revision, mode);
        replacement[provider]["public_key"] = json!(if provider == "stripe" {
            format!("pk_{}_REPLACED_PUBLIC", mode.as_str())
        } else {
            format!("{}_REPLACED_CLIENT", mode.as_str())
        });
        replacement[provider]["webhook_key"] = json!(format!(
            "{}_REPLACED",
            if provider == "stripe" {
                STRIPE_SIGNING
            } else {
                PADDLE_SIGNING
            }
        ));
        settings(mode, replacement).await;
        assert_eq!(
            reconcile::recover(&purchase, Some(&resource), fake.as_ref(), true)
                .await
                .unwrap()
                .status,
            "applied"
        );
        let observed = fake
            .observed_profiles
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .clone();
        assert_eq!(observed.0, provider);
        assert_eq!(observed.1, mode.as_str());
        assert!(observed.2.unwrap().contains("REPLACED"));
        assert_eq!(purchase_row(&purchase).await.credentials, held.credentials);
        // Replay uses stored acceptance, not a newly rotated signing secret.
        assert_eq!(
            reconcile::process_event(&eid, fake.as_ref(), true, false)
                .await
                .unwrap()
                .status,
            "applied"
        );
        // Restore catalog/settings for the next provider.
        let terms = plan::Entity::find_by_id("once")
            .one(DB::connection().unwrap().inner())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            admin_http
                .post(
                    "/admin/plans/once",
                    plan_input("once", "one_time", 1000, terms.version)
                )
                .await
                .status,
            302
        );
        settings(
            mode,
            settings_input(billing::load(mode).await.unwrap().revision, mode),
        )
        .await;

        // Missing current state defers evidence without removing its paid record.
        fake.fail_reads.store(1, Ordering::SeqCst);
        let failed = reconcile::recover(&purchase, Some(&resource), fake.as_ref(), false)
            .await
            .unwrap();
        assert_eq!(failed.status, "deferred");
        assert_eq!(failed.error_code.as_deref(), Some("provider_unavailable"));
        visible(id, mode == Mode::Live).await;
        let before = fake.start_count();
        BillingReconcile {
            limit: 1,
            status: false,
            purchase: None,
            resource: None,
            event: Some(failed.event_id.clone()),
            use_current_credentials: false,
        }
        .run()
        .await
        .unwrap();
        assert_eq!(fake.start_count(), before);
        assert_eq!(count_entitlements(id).await, 1);
        assert!(reconcile::pending(0, fake.as_ref()).await.is_err());
        assert!(reconcile::pending(101, fake.as_ref()).await.is_err());
    }
}
