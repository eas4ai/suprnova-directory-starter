#[allow(dead_code)]
mod common;

use common::Client;
use directory::{
    articles,
    commands::admin_access::ADMIN_PERMISSIONS,
    listings::{
        entities::{category, entitlement, listing},
        workflow,
    },
    models::user::User,
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};
use suprnova::{
    DB,
    eloquent::Model,
    serde_json::{self, Value, json},
};

const PASSWORD: &str = "fixture-password-123";

async fn account(name: &str, permissions: &[&str]) -> User {
    let mut user = User::create(name, &format!("overview-{name}@example.test"), PASSWORD)
        .await
        .unwrap();
    user.email_verified_at = Some(chrono::Utc::now());
    user.save().await.unwrap();
    for permission in permissions {
        suprnova::rbac::give_permission_to_model(
            "directory.user",
            &user.id.to_string(),
            permission,
        )
        .await
        .unwrap();
    }
    user
}

async fn login(user: &User) -> Client {
    let mut client = Client::new();
    assert_eq!(client.get("/login").await.status, 200);
    let response = client
        .post("/login", json!({"email":user.email,"password":PASSWORD}))
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    client
}

async fn props(client: &mut Client, path: &str) -> Value {
    let response = client.inertia_get(path).await;
    assert_eq!(response.status, 200, "{path}: {}", response.body);
    serde_json::from_str::<Value>(&response.body).unwrap()["props"].clone()
}

fn listing_input(title: &str, category: i64, version: i64) -> directory::listings::SaveListing {
    serde_json::from_value(json!({
        "version":version,"title":title,"summary":"Overview publication fixture",
        "description":"Content used to verify the admin overview.","url":"https://example.test/resource",
        "category_ids":[category],"media_id":null,"media_alt":""
    }))
    .unwrap()
}

async fn publish_listing(actor: i64, owner: i64, id: i64) {
    let db = DB::connection().unwrap();
    let row = listing::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    workflow::submit(owner, id, row.version).await.unwrap();
    workflow::decide(
        actor,
        id,
        workflow::Decision {
            version: row.version,
            revision_id: row.current_revision_id.unwrap(),
            decision: "approve".into(),
            reason: "".into(),
        },
    )
    .await
    .unwrap();
    let now = chrono::Utc::now().timestamp();
    entitlement::ActiveModel {
        id: Set(format!("overview-{id}")),
        listing_id: Set(id),
        mode: Set("free".into()),
        status: Set("active".into()),
        valid_from: Set(now - 60),
        valid_until: Set(None),
        updated_at: Set(now),
    }
    .insert(db.inner())
    .await
    .unwrap();
}

async fn content_fixtures(actor: i64, owner: i64) -> (i64, i64) {
    let db = DB::connection().unwrap();
    let category = category::ActiveModel {
        slug: Set("overview-category".into()),
        name: Set("Overview category".into()),
        active: Set(true),
        version: Set(1),
        ..Default::default()
    }
    .insert(db.inner())
    .await
    .unwrap()
    .id;
    let mut ids = Vec::new();
    for title in [
        "Published listing",
        "Published with changes",
        "New submission",
        "Private draft",
    ] {
        ids.push(
            workflow::save(owner, None, listing_input(title, category, 0))
                .await
                .unwrap(),
        );
    }
    for id in &ids[..2] {
        publish_listing(actor, owner, *id).await;
    }
    let changed = listing::Entity::find_by_id(ids[1])
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    workflow::save(
        owner,
        Some(changed.id),
        listing_input("Proposed change", category, changed.version),
    )
    .await
    .unwrap();
    let changed = listing::Entity::find_by_id(changed.id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    workflow::submit(owner, changed.id, changed.version)
        .await
        .unwrap();
    workflow::submit(owner, ids[2], 1).await.unwrap();

    for (slug, publish) in [("overview-published", true), ("overview-draft", false)] {
        let input = json!({"version":0,"slug":slug,"title":slug,"summary":"Overview article",
            "body":"Article publication proof.","term_ids":[],"media_id":null,"media_alt":""});
        let id =
            articles::workflow::save(actor, None, serde_json::from_value(input.clone()).unwrap())
                .await
                .unwrap();
        if publish {
            articles::workflow::publish(actor, id, 1, true)
                .await
                .unwrap();
            let row = articles::entities::article::Entity::find_by_id(id)
                .one(db.inner())
                .await
                .unwrap()
                .unwrap();
            let mut draft = input;
            draft["version"] = json!(row.version);
            draft["title"] = json!("Unpublished article edit");
            articles::workflow::save(actor, Some(id), serde_json::from_value(draft).unwrap())
                .await
                .unwrap();
        }
    }
    (ids[0], ids[2])
}

async fn assert_counts(client: &mut Client, published: u64, pending: u64, total: u64) {
    let data = props(client, "/admin").await;
    assert_eq!(
        data["listing_summary"],
        json!({"published":published,"awaiting_review":pending,"total":total})
    );
    assert_eq!(data["article_summary"], json!({"published":1,"total":2}));
    let queue = props(client, "/admin/listings?per_page=1").await;
    assert_eq!(queue["pagination"]["total"], pending);
    let public = props(client, "/listings?per_page=1").await;
    assert_eq!(public["pagination"]["total"], published);
}

async fn publication_boundaries(
    client: &mut Client,
    owner: i64,
    published_id: i64,
    pending_id: i64,
) {
    let db = DB::connection().unwrap();
    for (condition, restore) in [
        ("valid_until = 0", "valid_until = NULL"),
        ("valid_from = 4102444800", "valid_from = 0"),
        ("mode = 'test'", "mode = 'free'"),
        ("status = 'revoked'", "status = 'active'"),
    ] {
        db.inner()
            .execute_unprepared(&format!(
                "UPDATE publication_entitlements SET {condition} WHERE listing_id = {published_id}"
            ))
            .await
            .unwrap();
        assert_counts(client, 1, 2, 4).await;
        db.inner()
            .execute_unprepared(&format!(
                "UPDATE publication_entitlements SET {restore} WHERE listing_id = {published_id}"
            ))
            .await
            .unwrap();
    }
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE listings SET suspended = TRUE WHERE id = {published_id}"
        ))
        .await
        .unwrap();
    assert_counts(client, 1, 2, 4).await;
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE listings SET suspended = FALSE WHERE id = {published_id}"
        ))
        .await
        .unwrap();
    directory::accounts::entity::ActiveModel {
        user_id: Set(owner),
        version: Set(1),
        suspended: Set(true),
    }
    .insert(db.inner())
    .await
    .unwrap();
    assert_counts(client, 0, 2, 4).await;
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE account_access SET suspended = FALSE WHERE user_id = {owner}"
        ))
        .await
        .unwrap();
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE listings SET archived = TRUE WHERE id = {pending_id}"
        ))
        .await
        .unwrap();
    assert_counts(client, 2, 1, 3).await;
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE listings SET archived = FALSE WHERE id = {pending_id}"
        ))
        .await
        .unwrap();
    assert_counts(client, 2, 2, 4).await;
}

async fn delegated_overview() {
    for (name, permission, field, allowed_path) in [
        (
            "moderator",
            "listings.moderate",
            "listing_summary",
            "/admin/listings",
        ),
        (
            "editor",
            "articles.manage",
            "article_summary",
            "/admin/articles",
        ),
        ("auditor", "audit.view", "recent_activity", "/admin/audit"),
        ("billing", "billing.configure", "", "/admin/billing"),
    ] {
        let user = account(name, &["admin.access", permission]).await;
        let mut client = login(&user).await;
        let data = props(&mut client, "/admin").await;
        for candidate in ["listing_summary", "article_summary", "recent_activity"] {
            assert_eq!(
                !data[candidate].is_null(),
                candidate == field,
                "{name} received {candidate}"
            );
        }
        assert_eq!(client.get(allowed_path).await.status, 200);
        for path in ["/admin/listings", "/admin/articles", "/admin/audit"] {
            if path != allowed_path {
                assert_eq!(client.get(path).await.status, 403, "{name} reached {path}");
            }
        }
    }
    let user = account("shell", &["admin.access"]).await;
    let mut client = login(&user).await;
    let data = props(&mut client, "/admin").await;
    assert!(data["listing_summary"].is_null());
    assert!(data["article_summary"].is_null());
    assert!(data["recent_activity"].is_null());
}

#[tokio::test(flavor = "current_thread")]
async fn overview_workflows_contract() {
    let _mail = common::setup().await;
    let admin = account("admin", &ADMIN_PERMISSIONS).await;
    let owner = account("owner", &[]).await;
    let mut client = login(&admin).await;
    let empty = props(&mut client, "/admin").await;
    assert_eq!(
        empty["listing_summary"],
        json!({"published":0,"awaiting_review":0,"total":0})
    );
    assert_eq!(empty["article_summary"], json!({"published":0,"total":0}));
    assert_eq!(empty["recent_activity"], json!([]));
    let empty_html = client.get("/admin").await;
    assert!(
        empty_html
            .body
            .contains("No listings are public right now.")
    );
    assert!(empty_html.body.contains("No recorded activity yet."));

    let (published_id, pending_id) = content_fixtures(admin.id, owner.id).await;
    assert_counts(&mut client, 2, 2, 4).await;
    publication_boundaries(&mut client, owner.id, published_id, pending_id).await;
    let db = DB::connection().unwrap();
    // Multiple active entitlement records still describe one published listing.
    db.inner().execute_unprepared(&format!("INSERT INTO publication_entitlements (id, listing_id, mode, status, valid_from, valid_until, updated_at) VALUES ('overview-extra', {published_id}, 'free', 'active', 0, NULL, 1)")).await.unwrap();
    assert_counts(&mut client, 2, 2, 4).await;
    const PRIVATE: &str = "overview-private-moderation-marker";
    directory::listings::entities::audit::ActiveModel {
        actor_id: Set(admin.id),
        actor_type: Set("user".into()),
        target_type: Set("listing".into()),
        target_id: Set(published_id.to_string()),
        action: Set("approved".into()),
        summary: Set(PRIVATE.into()),
        private_reason: Set(Some(PRIVATE.into())),
        created_at: Set(chrono::Utc::now().timestamp()),
        ..Default::default()
    }
    .insert(db.inner())
    .await
    .unwrap();
    let data = props(&mut client, "/admin").await;
    let activity = data["recent_activity"].as_array().unwrap();
    assert_eq!(activity.len(), 5, "overview activity is bounded");
    assert!(
        activity
            .windows(2)
            .all(|pair| pair[0]["id"].as_i64() > pair[1]["id"].as_i64())
    );
    assert!(!data.to_string().contains(PRIVATE));
    assert!(
        !client
            .get("/admin")
            .await
            .body
            .contains("Your directory has no published listings yet.")
    );

    delegated_overview().await;
    let mut guest = Client::new();
    assert_eq!(
        guest.get("/admin").await.location.as_deref(),
        Some("/login")
    );
    let mut member = login(&owner).await;
    assert_eq!(
        member.get("/admin").await.location.as_deref(),
        Some("/dashboard")
    );
    let mut unverified = account("unverified", &ADMIN_PERMISSIONS).await;
    unverified.email_verified_at = None;
    unverified.save().await.unwrap();
    assert_eq!(login(&unverified).await.get("/admin").await.status, 403);

    // A database error must stay an error instead of becoming an empty dashboard.
    db.inner()
        .execute_unprepared("ALTER TABLE articles RENAME TO unavailable_overview_articles")
        .await
        .unwrap();
    let failed = client.inertia_get("/admin").await;
    db.inner()
        .execute_unprepared("ALTER TABLE unavailable_overview_articles RENAME TO articles")
        .await
        .unwrap();
    assert_eq!(failed.status, 500, "{}", failed.body);
    assert_counts(&mut client, 2, 2, 4).await;
}
