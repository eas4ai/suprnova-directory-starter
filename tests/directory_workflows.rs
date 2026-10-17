#[allow(dead_code)]
mod common;

use common::Client;
use directory::{
    commands::admin_access::{AccessAction, change_access},
    listings::{
        SaveListing,
        entities::{audit, entitlement, listing, media, revision},
        queries::{self, Page},
        workflow,
    },
    models::user::User,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use suprnova::{
    DB,
    eloquent::Model,
    serde_json::{self, Value, json},
};

async fn account(name: &str, email: &str, verified: bool) -> User {
    let mut user = User::create(name, email, "fixture-password-123")
        .await
        .unwrap();
    if verified {
        user.email_verified_at = Some(chrono::Utc::now());
        user.save().await.unwrap();
    }
    user
}
async fn login(user: &User) -> Client {
    let mut client = Client::new();
    assert_eq!(client.get("/login").await.status, 200);
    let response = client
        .post(
            "/login",
            json!({"email": user.email, "password": "fixture-password-123"}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    client
}
fn input(version: i64, title: &str, category: i64, image: Option<&str>) -> Value {
    json!({"version":version,"title":title,"summary":"A useful directory resource",
        "description":"**Useful** resource. <script>alert('private')</script> [bad](javascript:alert(1))",
        "url":"https://example.test/resource","category_ids":[category],
        "media_id":image,"media_alt":if image.is_some() {"Example resource image"} else {""}})
}
async fn row(id: i64) -> listing::Model {
    listing::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
async fn counts() -> (u64, u64, u64) {
    let db = DB::connection().unwrap();
    (
        listing::Entity::find().count(db.inner()).await.unwrap(),
        revision::Entity::find().count(db.inner()).await.unwrap(),
        media::Entity::find().count(db.inner()).await.unwrap(),
    )
}
async fn create(client: &mut Client, data: Value) -> i64 {
    let result = client.post("/dashboard/listings", data).await;
    assert_eq!(result.status, 302, "{}", result.body);
    result
        .location
        .unwrap()
        .trim_end_matches("/edit")
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
async fn submit(client: &mut Client, id: i64) {
    let result = client
        .post(
            &format!("/dashboard/listings/{id}/submit"),
            json!({"version":row(id).await.version}),
        )
        .await;
    assert_eq!(result.status, 302, "{}", result.body);
}
async fn decide(client: &mut Client, id: i64, decision: &str, reason: &str) {
    let current = row(id).await;
    let result = client.post(&format!("/admin/listings/{id}/decision"), json!({"version":current.version,"revision_id":current.current_revision_id,"decision":decision,"reason":reason})).await;
    assert_eq!(result.status, 302, "{}", result.body);
}
async fn entitlement(id: i64, mode: &str, status: &str, until: Option<i64>) {
    let db = DB::connection().unwrap();
    entitlement::Entity::delete_many()
        .filter(entitlement::Column::ListingId.eq(id))
        .exec(db.inner())
        .await
        .unwrap();
    let now = chrono::Utc::now().timestamp();
    entitlement::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        listing_id: Set(id),
        mode: Set(mode.into()),
        status: Set(status.into()),
        valid_from: Set(now - 60),
        valid_until: Set(until),
        updated_at: Set(now),
    }
    .insert(db.inner())
    .await
    .unwrap();
}
async fn upload(client: &mut Client) -> String {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 3)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let response = client
        .raw_request(
            "POST",
            "/dashboard/listings/media",
            bytes.into_inner(),
            "image/png",
            true,
        )
        .await;
    assert_eq!(response.status, 201, "{}", response.body);
    serde_json::from_str::<Value>(&response.body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}
async fn visible(guest: &mut Client, id: i64, image: &str, expected: bool) {
    let saved = row(id).await;
    let now = chrono::Utc::now().timestamp();
    let (all, _) = queries::search(
        "",
        "",
        Page {
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
        Page {
            number: 1,
            size: 100,
        },
        now,
    )
    .await
    .unwrap();
    assert_eq!(
        all.iter().any(|card| card.id == id),
        expected,
        "search eligibility for {}",
        saved.slug
    );
    assert_eq!(
        category.iter().any(|card| card.id == id),
        expected,
        "category eligibility"
    );
    assert_eq!(
        queries::detail(&saved.slug, now).await.is_ok(),
        expected,
        "detail eligibility"
    );
    assert_eq!(
        guest.get(&format!("/listings/{}", saved.slug)).await.status,
        if expected { 200 } else { 404 }
    );
    let response = guest
        .get(&format!("/media/listings/{}/{image}", saved.slug))
        .await;
    assert_eq!(
        response.status,
        if expected { 200 } else { 404 },
        "{}",
        response.body
    );
    if expected {
        assert!(image::load_from_memory(&response.body_bytes).is_ok());
    }
}

// One runtime owns the framework's global application/session state.
#[tokio::test(flavor = "current_thread")]
async fn directory_workflows_contract() {
    let _mail = common::setup().await;
    workflow::seed_categories().await.unwrap();
    workflow::seed_categories().await.unwrap();
    let categories = queries::categories().await.unwrap();
    let software = categories.iter().find(|c| c.slug == "software").unwrap().id;
    let design = categories.iter().find(|c| c.slug == "design").unwrap().id;
    assert_eq!(categories.len(), 4);
    let owner = account("Directory Owner", "directory-owner@example.test", true).await;
    let other = account("Other Owner", "directory-other@example.test", true).await;
    let pending = account(
        "Unverified Owner",
        "directory-unverified@example.test",
        false,
    )
    .await;
    let admin = account(
        "Directory Moderator",
        "directory-moderator@example.test",
        true,
    )
    .await;
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    let mut owner_http = login(&owner).await;
    let mut other_http = login(&other).await;
    let mut pending_http = login(&pending).await;
    let mut admin_http = login(&admin).await;
    let mut guest = Client::new();
    guest.get("/login").await;

    // DIR-001: request middleware, verified account gate, persistent ownership.
    let initial = counts().await;
    assert_eq!(
        guest.get("/dashboard/listings").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        guest
            .post(
                "/dashboard/listings",
                input(0, "Guest draft", software, None)
            )
            .await
            .location
            .as_deref(),
        Some("/login")
    );
    assert_eq!(
        pending_http
            .post(
                "/dashboard/listings",
                input(0, "Unverified draft", software, None)
            )
            .await
            .status,
        403
    );
    assert_eq!(
        owner_http
            .request(
                "POST",
                "/dashboard/listings",
                Some(input(0, "No CSRF", software, None)),
                false
            )
            .await
            .status,
        419
    );
    assert_eq!(counts().await, initial);
    let image = upload(&mut owner_http).await;
    let private_url = format!("/dashboard/listings/media/{image}");
    assert_eq!(owner_http.get(&private_url).await.status, 200);
    assert_eq!(other_http.get(&private_url).await.status, 403);
    assert_eq!(
        guest.get(&private_url).await.location.as_deref(),
        Some("/login")
    );
    let id = create(
        &mut owner_http,
        input(0, "Original 100%_tool", software, Some(&image)),
    )
    .await;
    let original = row(id).await;
    assert_eq!(original.owner_id, owner.id);
    let mut reopened = login(&owner).await;
    assert_eq!(
        reopened
            .get(&format!("/dashboard/listings/{id}/edit"))
            .await
            .status,
        200
    );
    assert_eq!(
        queries::owner_listing(owner.id, id)
            .await
            .unwrap()
            .current
            .title,
        "Original 100%_tool"
    );
    assert_eq!(
        other_http
            .get(&format!("/dashboard/listings/{id}/edit"))
            .await
            .status,
        404
    );
    assert!(queries::owner_listing(other.id, id).await.is_err());
    for (suffix, data) in [
        ("", input(original.version, "Stolen", software, None)),
        ("/submit", json!({"version":original.version})),
        ("/archive", json!({"version":original.version})),
    ] {
        assert_eq!(
            other_http
                .post(&format!("/dashboard/listings/{id}{suffix}"), data)
                .await
                .status,
            404
        );
        assert_eq!(row(id).await, original);
    }
    assert_eq!(
        queries::owner_listing(owner.id, id)
            .await
            .unwrap()
            .next_action,
        "submit"
    );

    // DIR-002: rejected mutations leave no version or partial revision behind.
    let before = counts().await;
    for (field, value) in [
        ("title", json!("")),
        ("title", json!("a".repeat(121))),
        ("summary", json!("a".repeat(281))),
        ("description", json!("a".repeat(20_001))),
        ("url", json!("javascript:alert(1)")),
        ("url", json!("https://user:secret@example.test")),
        ("category_ids", json!([])),
        ("category_ids", json!([999999])),
        ("media_id", json!(uuid::Uuid::new_v4().to_string())),
    ] {
        let mut bad = input(original.version, "Invalid", software, None);
        bad[field] = value;
        let response = owner_http
            .post(&format!("/dashboard/listings/{id}"), bad)
            .await;
        assert_eq!(response.status, 422, "{field}: {}", response.body);
        assert_eq!(row(id).await, original);
        assert_eq!(counts().await, before);
    }
    let other_image = upload(&mut other_http).await;
    let before_foreign = counts().await;
    assert_eq!(
        owner_http
            .post(
                &format!("/dashboard/listings/{id}"),
                input(
                    original.version,
                    "Foreign image",
                    software,
                    Some(&other_image)
                )
            )
            .await
            .status,
        422
    );
    assert_eq!(counts().await, before_foreign);
    assert_eq!(row(id).await, original);
    assert_eq!(
        owner_http
            .raw_request(
                "POST",
                "/dashboard/listings/media",
                b"<svg onload='alert(1)'/>".to_vec(),
                "image/svg+xml",
                true
            )
            .await
            .status,
        422
    );
    assert_eq!(counts().await, before_foreign);
    assert_eq!(
        pending_http
            .raw_request(
                "POST",
                "/dashboard/listings/media",
                vec![1],
                "image/png",
                true
            )
            .await
            .status,
        403
    );
    visible(&mut guest, id, &image, false).await;

    // DIR-003: a moderator needs an exact submitted revision and rejection reason.
    let decision = json!({"version":original.version,"revision_id":original.current_revision_id,"decision":"approve","reason":""});
    assert_eq!(
        admin_http
            .post(&format!("/admin/listings/{id}/decision"), decision.clone())
            .await
            .status,
        422
    );
    assert_eq!(row(id).await, original);
    assert_eq!(
        other_http
            .post(&format!("/admin/listings/{id}/decision"), decision)
            .await
            .location
            .as_deref(),
        Some("/dashboard")
    );
    submit(&mut owner_http, id).await;
    let submitted = row(id).await;
    let view = queries::owner_listing(owner.id, id).await.unwrap();
    assert_eq!(view.moderation_status, "submitted");
    assert_eq!(view.next_action, "await_review");
    assert_eq!(admin_http.post(&format!("/admin/listings/{id}/decision"),json!({"version":submitted.version,"revision_id":submitted.current_revision_id,"decision":"reject","reason":"  "})).await.status,422);
    assert_eq!(row(id).await, submitted);
    decide(&mut admin_http, id, "approve", "").await;
    let approved = row(id).await;
    assert_eq!(approved.approved_revision_id, submitted.current_revision_id);
    let view = queries::owner_listing(owner.id, id).await.unwrap();
    assert_eq!(view.publication_status, "awaiting_payment");
    assert_eq!(view.next_action, "checkout");
    visible(&mut guest, id, &image, false).await;
    assert_eq!(admin_http.post(&format!("/admin/listings/{id}/decision"),json!({"version":submitted.version,"revision_id":submitted.current_revision_id,"decision":"reject","reason":"Stale review"})).await.status,422);
    assert_eq!(row(id).await, approved);

    // DIR-005/008: all public surfaces use the same entitlement matrix.
    let now = chrono::Utc::now().timestamp();
    for (mode, status, until, public, owner_status) in [
        ("free", "active", None, true, "free"),
        ("live", "active", Some(now + 3600), true, "paid"),
        ("test", "active", Some(now + 3600), false, "test"),
        ("live", "active", Some(now - 1), false, "expired"),
        ("live", "revoked", None, false, "revoked"),
        ("live", "disputed", None, false, "disputed"),
    ] {
        entitlement(id, mode, status, until).await;
        visible(&mut guest, id, &image, public).await;
        assert_eq!(
            queries::owner_listing(owner.id, id)
                .await
                .unwrap()
                .payment_status,
            owner_status
        );
    }
    entitlement(id, "free", "active", None).await;
    let detail = queries::detail(&original.slug, now).await.unwrap();
    assert!(!detail.description_html.contains("<script"));
    assert!(!detail.description_html.contains("href=\"javascript:"));
    assert!(detail.description_html.contains("<strong>Useful</strong>"));

    // DIR-004: proposed/rejected edits never replace the approved content or image.
    let proposed_image = upload(&mut owner_http).await;
    let response = owner_http
        .post(
            &format!("/dashboard/listings/{id}"),
            input(
                approved.version,
                "Unreviewed rename",
                design,
                Some(&proposed_image),
            ),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    let proposed = row(id).await;
    assert_ne!(proposed.current_revision_id, approved.current_revision_id);
    assert_eq!(proposed.slug, original.slug);
    assert_eq!(
        queries::detail(&original.slug, now)
            .await
            .unwrap()
            .card
            .title,
        "Original 100%_tool"
    );
    assert_eq!(
        guest
            .get(&format!(
                "/media/listings/{}/{proposed_image}",
                original.slug
            ))
            .await
            .status,
        404
    );
    assert_eq!(
        owner_http
            .post(
                &format!("/dashboard/listings/{id}"),
                input(approved.version, "Stale save", software, None)
            )
            .await
            .status,
        422
    );
    assert_eq!(row(id).await, proposed);
    submit(&mut owner_http, id).await;
    decide(
        &mut admin_http,
        id,
        "reject",
        "Please restore the accurate title",
    )
    .await;
    let rejected = queries::owner_listing(owner.id, id).await.unwrap();
    assert_eq!(
        rejected.current.reason.as_deref(),
        Some("Please restore the accurate title")
    );
    assert_eq!(rejected.next_action, "edit");
    assert_eq!(rejected.publication_status, "published");
    visible(&mut guest, id, &image, true).await;
    assert_eq!(
        queries::detail(&original.slug, now)
            .await
            .unwrap()
            .card
            .title,
        "Original 100%_tool"
    );

    // DIR-006: literal wildcard search, category filtering, bounded stable pages.
    let (found, _) = queries::search(
        "100%_",
        "software",
        Page {
            number: 1,
            size: 24,
        },
        now,
    )
    .await
    .unwrap();
    assert_eq!(found.iter().map(|v| v.id).collect::<Vec<_>>(), vec![id]);
    assert!(
        queries::search(
            "",
            "design",
            Page {
                number: 1,
                size: 24
            },
            now
        )
        .await
        .unwrap()
        .0
        .is_empty()
    );
    assert!(
        queries::search(
            "ORIGINAL",
            "",
            Page {
                number: 1,
                size: 24
            },
            now
        )
        .await
        .unwrap()
        .0
        .iter()
        .any(|v| v.id == id)
    );
    assert!(
        queries::search(
            &"x".repeat(201),
            "",
            Page {
                number: 1,
                size: 24
            },
            now
        )
        .await
        .is_err()
    );
    assert_eq!(guest.get("/listings?per_page=101").await.status, 422);
    assert_eq!(guest.get("/listings?page=0").await.status, 422);
    for title in ["Pagination Alpha", "Pagination Beta"] {
        let extra = create(&mut owner_http, input(0, title, software, None)).await;
        submit(&mut owner_http, extra).await;
        decide(&mut admin_http, extra, "approve", "").await;
        entitlement(extra, "free", "active", None).await;
    }
    let (first, total) = queries::search("Pagination", "", Page { number: 1, size: 1 }, now + 1)
        .await
        .unwrap();
    let (second, _) = queries::search("Pagination", "", Page { number: 2, size: 1 }, now + 1)
        .await
        .unwrap();
    assert_eq!(total.total, 2);
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_ne!(first[0].id, second[0].id);
    assert_eq!(
        first[0].id,
        queries::search("Pagination", "", Page { number: 1, size: 1 }, now + 1)
            .await
            .unwrap()
            .0[0]
            .id
    );
    assert_eq!(
        queries::search(
            "%",
            "",
            Page {
                number: 1,
                size: 24
            },
            now + 1
        )
        .await
        .unwrap()
        .0
        .len(),
        1
    );

    // Competing saves share a version: exactly one immutable proposal wins.
    let version = row(id).await.version;
    let a: SaveListing =
        serde_json::from_value(input(version, "Winner A", software, Some(&image))).unwrap();
    let b: SaveListing =
        serde_json::from_value(input(version, "Winner B", software, Some(&image))).unwrap();
    let before_race = counts().await;
    let (a, b) = tokio::join!(
        workflow::save(owner.id, Some(id), a),
        workflow::save(owner.id, Some(id), b)
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    assert_eq!(counts().await.1, before_race.1 + 1);
    assert_eq!(row(id).await.version, version + 1);
    submit(&mut owner_http, id).await;
    let review = row(id).await;
    // Saving while a reviewer holds the older submission invalidates that decision.
    assert_eq!(
        owner_http
            .post(
                &format!("/dashboard/listings/{id}"),
                input(
                    review.version,
                    "Final approved rename",
                    software,
                    Some(&image)
                )
            )
            .await
            .status,
        302
    );
    assert_eq!(admin_http.post(&format!("/admin/listings/{id}/decision"),json!({"version":review.version,"revision_id":review.current_revision_id,"decision":"approve","reason":""})).await.status,422);
    submit(&mut owner_http, id).await;
    decide(&mut admin_http, id, "approve", "").await;
    assert_eq!(row(id).await.slug, original.slug);
    assert_eq!(
        queries::detail(&original.slug, chrono::Utc::now().timestamp())
            .await
            .unwrap()
            .card
            .title,
        "Final approved rename"
    );

    // DIR-007: suspension does not grant or restore payment eligibility.
    let version = row(id).await.version;
    let suspension = json!({"version":version,"suspended":true,"reason":"Policy review"});
    assert_eq!(
        other_http
            .post(
                &format!("/admin/listings/{id}/suspension"),
                suspension.clone()
            )
            .await
            .location
            .as_deref(),
        Some("/dashboard")
    );
    assert_eq!(row(id).await.version, version);
    assert_eq!(
        admin_http
            .post(&format!("/admin/listings/{id}/suspension"), suspension)
            .await
            .status,
        302
    );
    visible(&mut guest, id, &image, false).await;
    entitlement(id, "live", "active", Some(now - 1)).await;
    assert_eq!(admin_http.post(&format!("/admin/listings/{id}/suspension"),json!({"version":row(id).await.version,"suspended":false,"reason":"Review complete"})).await.status,302);
    visible(&mut guest, id, &image, false).await;
    entitlement(id, "free", "active", None).await;
    visible(&mut guest, id, &image, true).await;
    let audits = audit::Entity::find()
        .filter(audit::Column::TargetType.eq("listing"))
        .filter(audit::Column::TargetId.eq(id.to_string()))
        .all(DB::connection().unwrap().inner())
        .await
        .unwrap();
    assert!(audits.iter().any(|a| a.actor_id == admin.id
        && a.action == "suspended"
        && a.private_reason.as_deref() == Some("Policy review")));
    assert!(audits.iter().any(|a| a.actor_id == admin.id
        && a.action == "reinstated"
        && a.private_reason.as_deref() == Some("Review complete")));
    assert!(audits.iter().any(|a| {
        a.actor_id == admin.id
            && a.action == "rejected"
            && a.private_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("Please restore the accurate title"))
    }));
    let audit_page = admin_http.inertia_get("/admin/audit?per_page=100").await;
    assert_eq!(audit_page.status, 200);
    for private in [
        "Policy review",
        "Review complete",
        "Please restore the accurate title",
        "private_reason",
    ] {
        assert!(
            !audit_page.body.contains(private),
            "Private moderation detail leaked in broad audit output"
        );
    }
    let revision_count = counts().await.1;
    assert_eq!(
        owner_http
            .post(
                &format!("/dashboard/listings/{id}/archive"),
                json!({"version":row(id).await.version})
            )
            .await
            .status,
        302
    );
    assert!(row(id).await.archived);
    assert_eq!(counts().await.1, revision_count);
    visible(&mut guest, id, &image, false).await;
    assert_eq!(
        queries::owner_listing(owner.id, id)
            .await
            .unwrap()
            .publication_status,
        "archived"
    );
}
