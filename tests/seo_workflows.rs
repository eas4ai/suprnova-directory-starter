#[allow(dead_code)]
mod common;

use common::Client;
use directory::{
    articles,
    commands::admin_access::ADMIN_PERMISSIONS,
    listings::{
        entities::{category, entitlement, listing, revision},
        workflow,
    },
    models::user::User,
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, PaginatorTrait, Set};
use suprnova::{
    DB,
    eloquent::Model,
    serde_json::{self, Value, json},
};
const PASSWORD: &str = "fixture-password-123";

async fn account(name: &str, permissions: &[&str]) -> User {
    let mut user = User::create(name, &format!("seo-{name}@example.test"), PASSWORD)
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
    client.get("/login").await;
    assert_eq!(
        client
            .post("/login", json!({"email":user.email,"password":PASSWORD}))
            .await
            .status,
        302
    );
    client
}
async fn props(client: &mut Client, path: &str) -> Value {
    let response = client.inertia_get(path).await;
    assert_eq!(response.status, 200, "{path}: {}", response.body);
    serde_json::from_str::<Value>(&response.body).unwrap()["props"].clone()
}
async fn listing_row(id: i64) -> listing::Model {
    listing::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
async fn article_row(id: i64) -> articles::entities::article::Model {
    articles::entities::article::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
fn listing_input(category: i64, version: i64, seo: Value) -> Value {
    json!({"version":version,"title":"SEO listing","summary":"A useful public listing summary.","description":"Visible **approved café →** prose.","url":"https://example.test/resource","category_ids":[category],"media_id":null,"media_alt":"","seo":seo})
}
fn article_input(version: i64, slug: &str, seo: Value) -> Value {
    json!({"version":version,"slug":slug,"title":"SEO article <script>marker</script>","summary":"A distinct article summary.","body":"Visible **article café →** prose.","term_ids":[],"media_id":null,"media_alt":"","seo":seo})
}
async fn approve(admin: i64, owner: i64, id: i64) {
    let row = listing_row(id).await;
    workflow::submit(owner, id, row.version).await.unwrap();
    let row = listing_row(id).await;
    workflow::decide(
        admin,
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
}
async fn audit_count() -> u64 {
    directory::listings::entities::audit::Entity::find()
        .count(DB::connection().unwrap().inner())
        .await
        .unwrap()
}
async fn settings(admin: &mut Client) -> Value {
    props(admin, "/admin/seo").await["settings"].clone()
}
async fn save_settings(admin: &mut Client, input: Value) {
    let response = admin.post("/admin/seo", input).await;
    assert_eq!(response.status, 302, "{}", response.body);
}
async fn sitemap(client: &mut Client) -> String {
    let index = client.get("/sitemap.xml").await;
    assert_eq!(index.status, 200, "{}", index.body);
    let mut output = String::new();
    for part in index.body.split("<loc>").skip(1) {
        let url = part.split("</loc>").next().unwrap();
        let path = url
            .strip_prefix(&directory::config::site::origin().unwrap())
            .unwrap();
        let response = client.get(path).await;
        assert_eq!(response.status, 200, "{}", response.body);
        output.push_str(&response.body);
    }
    output
}

async fn settings_contract(admin: &mut Client, owner: &User) {
    let initial = settings(admin).await;
    assert_eq!(initial["version"], 0);
    assert_eq!(initial["defaults"]["title_format"], "{title} | {site}");
    let before = audit_count().await;
    let mut updated = initial.clone();
    updated["defaults"]["title_format"] = json!("{title} · {site}");
    updated["defaults"]["description"] = json!("A saved site description.");
    updated["defaults"]["publisher"] = json!("Example publisher");
    updated["defaults"]["profiles"] = json!(["https://example.test/publisher"]);
    updated["defaults"]["google_verification"] = json!("google-proof-token");
    updated["defaults"]["bing_verification"] = json!("BING123");
    save_settings(admin, updated.clone()).await;
    assert_eq!(audit_count().await, before + 1);
    assert_eq!(settings(admin).await["defaults"], updated["defaults"]);
    assert_eq!(
        admin.post("/admin/seo", updated).await.status,
        422,
        "stale settings must not overwrite"
    );
    assert_eq!(audit_count().await, before + 1);
    for (field, value) in [
        ("title_format", json!("{unknown}")),
        ("image", json!("https://user:secret@example.test/x")),
        ("image", json!("//outside.test/x")),
        ("image", json!("/admin/articles/media/private")),
        ("google_verification", json!("<meta secret-value>")),
        ("profiles", json!(["javascript:alert(1)"])),
    ] {
        let mut invalid = settings(admin).await;
        invalid["defaults"][field] = value;
        assert_eq!(
            admin.post("/admin/seo", invalid).await.status,
            422,
            "accepted {field}"
        );
    }
    assert_eq!(audit_count().await, before + 1);
    let audit = props(admin, "/admin/audit").await;
    assert!(!audit.to_string().contains("google-proof-token"));
    let mut member = login(owner).await;
    assert_eq!(
        member.get("/admin/seo").await.location.as_deref(),
        Some("/dashboard")
    );
    let editor = account("editor", &["admin.access", "articles.manage"]).await;
    let mut editor = login(&editor).await;
    for path in [
        "/admin/seo",
        "/admin/seo?kind=article",
        "/admin/seo?missing_page=1",
    ] {
        assert_eq!(editor.get(path).await.status, 403);
    }
    for path in [
        "/admin/seo",
        "/admin/seo/redirects",
        "/admin/seo/redirects/1",
        "/admin/seo/redirects/1/remove",
    ] {
        assert_eq!(
            editor.post(path, json!({})).await.status,
            403,
            "editor mutated {path}"
        );
    }
    let dedicated = account("manager", &["admin.access", "seo.manage"]).await;
    let mut dedicated = login(&dedicated).await;
    assert_eq!(dedicated.inertia_get("/admin/seo").await.status, 200);
    assert_eq!(dedicated.get("/admin/articles").await.status, 403);
    let mut unverified = account("unverified", &["admin.access", "seo.manage"]).await;
    unverified.email_verified_at = None;
    unverified.save().await.unwrap();
    assert_eq!(login(&unverified).await.get("/admin/seo").await.status, 403);
}

async fn metadata_contract(
    admin: &mut Client,
    guest: &mut Client,
    listing_id: i64,
    article_id: i64,
    owner: &User,
    category_id: i64,
) {
    let listing = listing_row(listing_id).await;
    let path = format!("/listings/{}", listing.slug);
    let public = props(guest, &path).await;
    assert_eq!(public["seo"]["title"], "Shared search title · Directory");
    assert_eq!(public["seo"]["description"], "Listing search description.");
    assert_eq!(
        public["seo"]["canonical"],
        format!("https://catalog.example.test{path}")
    );
    let html = guest.get(&path).await;
    assert_eq!(html.status, 200);
    for marker in [
        "property=\"og:title\"",
        "name=\"twitter:card\"",
        "google-proof-token",
        "BING123",
        "Listing search description.",
        "text/markdown",
        "<strong>approved café →</strong>",
    ] {
        assert!(html.body.contains(marker), "initial HTML omitted {marker}");
    }
    let mut owner_client = login(owner).await;
    let pending = listing_input(
        category_id,
        listing.version,
        json!({"title":"PRIVATE proposed metadata","description":"PRIVATE proposed description","noindex":true}),
    );
    assert_eq!(
        owner_client
            .post(&format!("/dashboard/listings/{listing_id}"), pending)
            .await
            .status,
        302
    );
    assert_eq!(
        props(guest, &path).await["seo"]["title"],
        public["seo"]["title"]
    );
    assert!(!guest.get(&path).await.body.contains("PRIVATE proposed"));
    let current = listing_row(listing_id).await;
    workflow::submit(owner.id, listing_id, current.version)
        .await
        .unwrap();
    let review = props(admin, &format!("/admin/listings/{listing_id}")).await;
    assert_eq!(review["listing"]["current"]["seo"]["noindex"], true);
    assert_eq!(review["listing"]["approved"]["seo"]["noindex"], false);
    let article = article_row(article_id).await;
    let draft = article_input(
        article.version,
        "seo-article",
        json!({"title":"PRIVATE article metadata","description":"PRIVATE article description"}),
    );
    assert_eq!(
        admin
            .post(&format!("/admin/articles/{article_id}"), draft)
            .await
            .status,
        302
    );
    let public = props(guest, "/articles/seo-article").await;
    assert_eq!(public["seo"]["title"], "Shared search title · Directory");
    let data: Value =
        serde_json::from_str(public["seo"]["structured_data"].as_str().unwrap()).unwrap();
    assert_eq!(data[0]["headline"], "SEO article <script>marker</script>");
    assert_eq!(data[0]["publisher"]["name"], "Example publisher");
    assert_eq!(
        data[0]["datePublished"],
        chrono::DateTime::from_timestamp(article.published_at.unwrap(), 0)
            .unwrap()
            .to_rfc3339()
    );
    assert!(
        !public["seo"]["structured_data"]
            .as_str()
            .unwrap()
            .contains("<script>")
    );
    assert!(!data.to_string().contains("aggregateRating"));
    assert!(
        !guest
            .get("/articles/seo-article")
            .await
            .body
            .contains("PRIVATE article")
    );
    let report = props(admin, "/admin/seo?kind=article").await;
    let row = &report["report"]["rows"][0];
    assert_eq!(
        row["preview"]["title"], public["seo"]["title"],
        "SEO-005: preview disagrees with public metadata"
    );
    assert_eq!(row["preview"]["description"], public["seo"]["description"]);
    assert!(
        row["findings"]
            .to_string()
            .contains("shared by 2 public pages")
    );
    assert!(row["findings"].to_string().contains("social image"));
    // No request Host value is used: this client sends directory.test, while all metadata uses configured origin.
    assert!(!public["seo"].to_string().contains("directory.test"));
    assert_eq!(
        props(guest, "/listings?page=2&utm_source=tracking").await["seo"]["canonical"],
        "https://catalog.example.test/listings?page=2"
    );
    for path in [
        "/listings?q=SEO",
        "/articles?q=SEO",
        "/listings?per_page=1",
        "/articles?category=guides&tag=helpful",
    ] {
        assert_eq!(props(guest, path).await["seo"]["robots"], "noindex, follow");
    }
    assert_eq!(
        props(guest, "/listings?page=2").await["seo"]["robots"],
        "index, follow"
    );
}

async fn discovery_contract(
    admin: &mut Client,
    guest: &mut Client,
    listing_id: i64,
    article_id: i64,
    category_id: i64,
) {
    let db = DB::connection().unwrap();
    let row = listing_row(listing_id).await;
    let path = format!("/listings/{}", row.slug);
    let before = sitemap(guest).await;
    assert!(before.contains(&path));
    assert!(before.contains("/articles/seo-article"));
    assert!(before.contains("<lastmod>"));
    assert!(!before.contains("seo-private"));
    let revision_id = row.approved_revision_id.unwrap();
    let revision = revision::Entity::find_by_id(revision_id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    let mut hidden: Value = serde_json::from_str(&revision.seo).unwrap();
    hidden["noindex"] = json!(true);
    revision::ActiveModel {
        id: Set(revision_id),
        seo: Set(hidden.to_string()),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    assert_eq!(
        props(guest, &path).await["seo"]["robots"],
        "noindex, follow"
    );
    assert!(
        !sitemap(guest).await.contains(&path),
        "SEO-003: noindex listing entered sitemap"
    );
    let report = props(admin, "/admin/seo").await;
    assert_eq!(report["report"]["rows"][0]["sitemap_included"], false);
    revision::ActiveModel {
        id: Set(revision_id),
        seo: Set(revision.seo),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    for (condition, restore) in [
        ("valid_until = 0", "valid_until = NULL"),
        ("mode = 'test'", "mode = 'free'"),
        ("status = 'revoked'", "status = 'active'"),
    ] {
        db.inner()
            .execute_unprepared(&format!(
                "UPDATE publication_entitlements SET {condition} WHERE listing_id = {listing_id}"
            ))
            .await
            .unwrap();
        assert!(!sitemap(guest).await.contains(&path));
        assert_eq!(guest.get(&format!("{path}.md")).await.status, 404);
        db.inner()
            .execute_unprepared(&format!(
                "UPDATE publication_entitlements SET {restore} WHERE listing_id = {listing_id}"
            ))
            .await
            .unwrap();
    }
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE listings SET suspended = TRUE WHERE id = {listing_id}"
        ))
        .await
        .unwrap();
    assert!(!sitemap(guest).await.contains(&path));
    assert_eq!(guest.get(&format!("{path}.md")).await.status, 404);
    db.inner()
        .execute_unprepared(&format!(
            "UPDATE listings SET suspended = FALSE WHERE id = {listing_id}"
        ))
        .await
        .unwrap();
    let category = category::Entity::find_by_id(category_id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    let response = admin.post(&format!("/admin/taxonomy/listing_category/{category_id}"), json!({"version":category.version,"slug":category.slug,"name":category.name,"active":true,"seo":{"title":"Category search title","description":"Category search description.","noindex":true}})).await;
    assert_eq!(response.status, 302, "{}", response.body);
    let public = props(guest, "/listings?category=seo-category").await;
    assert_eq!(public["seo"]["title"], "Category search title · Directory");
    assert_eq!(public["seo"]["description"], "Category search description.");
    assert!(!sitemap(guest).await.contains("category=seo-category"));
    let mut global = settings(admin).await;
    global["defaults"]["noindex"] = json!(true);
    save_settings(admin, global).await;
    assert!(sitemap(guest).await.is_empty());
    assert_eq!(
        props(guest, "/articles/seo-article").await["seo"]["robots"],
        "noindex, follow"
    );
    let robots = guest.get("/robots.txt").await;
    assert_eq!(robots.status, 200);
    assert!(robots.body.contains("Allow: /\n"));
    assert!(!robots.body.contains("Disallow: /\n"));
    let mut global = settings(admin).await;
    global["defaults"]["noindex"] = json!(false);
    save_settings(admin, global).await;
    assert!(sitemap(guest).await.contains("/articles/seo-article"));
    // Historical public dates cannot be replaced by request time or a newer private draft.
    let dates_article = article_row(article_id).await;
    let dates_revision = articles::entities::revision::Entity::find_by_id(
        dates_article.published_revision_id.unwrap(),
    )
    .one(db.inner())
    .await
    .unwrap()
    .unwrap();
    revision::ActiveModel {
        id: Set(revision_id),
        decided_at: Set(Some(946684800)),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    articles::entities::article::ActiveModel {
        id: Set(article_id),
        published_at: Set(Some(946684800)),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    articles::entities::revision::ActiveModel {
        id: Set(dates_revision.id),
        created_at: Set(946771200),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    let listing_xml = guest.get("/sitemaps/listings/1.xml").await;
    assert!(
        listing_xml
            .body
            .contains("<lastmod>2000-01-01T00:00:00+00:00</lastmod>")
    );
    let article_xml = guest.get("/sitemaps/articles/1.xml").await;
    assert!(
        article_xml
            .body
            .contains("<lastmod>2000-01-02T00:00:00+00:00</lastmod>")
    );
    assert!(
        !guest
            .get("/sitemaps/pages/1.xml")
            .await
            .body
            .contains("<lastmod>")
    );
    let dated = props(guest, "/articles/seo-article").await;
    let data: Value =
        serde_json::from_str(dated["seo"]["structured_data"].as_str().unwrap()).unwrap();
    assert_eq!(data[0]["datePublished"], "2000-01-01T00:00:00+00:00");
    assert_eq!(data[0]["dateModified"], "2000-01-02T00:00:00+00:00");
    revision::ActiveModel {
        id: Set(revision_id),
        decided_at: Set(revision.decided_at),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    articles::entities::article::ActiveModel {
        id: Set(article_id),
        published_at: Set(dates_article.published_at),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    articles::entities::revision::ActiveModel {
        id: Set(dates_revision.id),
        created_at: Set(dates_revision.created_at),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    // Article noindex and empty overrides follow the same public-revision boundary.
    let published = article_row(article_id).await;
    let published_revision =
        articles::entities::revision::Entity::find_by_id(published.published_revision_id.unwrap())
            .one(db.inner())
            .await
            .unwrap()
            .unwrap();
    for (overrides, indexable) in [(json!({"noindex": true}), false), (json!({}), true)] {
        articles::entities::revision::ActiveModel {
            id: Set(published_revision.id),
            seo: Set(overrides.to_string()),
            ..Default::default()
        }
        .update(db.inner())
        .await
        .unwrap();
        let page = props(guest, "/articles/seo-article").await;
        assert_eq!(
            page["seo"]["title"], "SEO article <script>marker</script> · Directory",
            "blank SEO title must use public content"
        );
        assert_eq!(page["seo"]["description"], "A distinct article summary.");
        assert_eq!(
            sitemap(guest).await.contains("/articles/seo-article"),
            indexable
        );
        assert_eq!(
            guest.get("/articles/seo-article.md").await.status,
            200,
            "noindex is not access control"
        );
    }
    articles::entities::revision::ActiveModel { id: Set(published_revision.id), seo: Set(json!({"title":"Unique useful search title", "description":"Specific complete description.", "image":"https://example.test/social.png"}).to_string()), ..Default::default() }.update(db.inner()).await.unwrap();
    let good = props(admin, "/admin/seo?kind=article").await;
    assert_eq!(
        good["report"]["rows"][0]["findings"],
        json!([]),
        "known-good metadata must have no findings"
    );
    let page = props(guest, "/articles/seo-article").await;
    assert_eq!(
        good["report"]["rows"][0]["preview"]["image"],
        page["seo"]["image"]
    );
    articles::entities::revision::ActiveModel {
        id: Set(published_revision.id),
        seo: Set(published_revision.seo),
        ..Default::default()
    }
    .update(db.inner())
    .await
    .unwrap();
    // The public article remains tied to its publication pointer while private edits exist.
    let article = article_row(article_id).await;
    assert_ne!(article.current_revision_id, article.published_revision_id);
}

async fn redirects_contract(admin: &mut Client, guest: &mut Client) {
    let response = admin
        .post(
            "/admin/seo/redirects",
            json!({"version":0,"source":"/old-guide","destination":"/articles/seo-article"}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    let response = guest.get("/old-guide?token=DO-NOT-STORE").await;
    assert_eq!(response.status, 301);
    assert_eq!(
        response.location.as_deref(),
        Some("/articles/seo-article"),
        "SEO-004: redirect destination changed"
    );
    assert_eq!(response.headers["cache-control"], "no-store");
    for (source, destination) in [
        ("/external", "https://outside.test"),
        ("/external", "//outside.test"),
        ("/a/../b", "/articles"),
        ("/encoded%2fpath", "/articles"),
        ("/admin/seo", "/articles"),
        ("/articles/new-slug", "/articles"),
        ("/login", "/articles"),
        ("/lost.md", "/articles"),
        ("/self-loop", "/self-loop"),
        ("/invalid", "/private-missing"),
        ("/query", "/articles?token=x"),
        ("/backslash", "/\\outside.test"),
    ] {
        assert_eq!(
            admin
                .post(
                    "/admin/seo/redirects",
                    json!({"version":0,"source":source,"destination":destination})
                )
                .await
                .status,
            422,
            "accepted {source} -> {destination}"
        );
    }
    assert_eq!(
        admin
            .post(
                "/admin/seo/redirects",
                json!({"version":0,"source":"/older-guide","destination":"/old-guide"})
            )
            .await
            .status,
        302
    );
    assert_eq!(
        guest.get("/older-guide").await.location.as_deref(),
        Some("/articles/seo-article")
    );
    let all = props(admin, "/admin/seo").await;
    let row = all["redirects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["source"] == "/old-guide")
        .unwrap();
    let id = row["id"].as_i64().unwrap();
    let version = row["version"].as_i64().unwrap();
    assert_eq!(
        admin
            .post(
                &format!("/admin/seo/redirects/{id}"),
                json!({"version":version,"source":"/old-guide","destination":"/older-guide"})
            )
            .await
            .status,
        422
    );
    assert_eq!(
        admin
            .post(
                &format!("/admin/seo/redirects/{id}/remove"),
                json!({"version":version})
            )
            .await
            .status,
        422,
        "cannot break incoming redirects"
    );
    assert_eq!(
        admin
            .post(
                &format!("/admin/seo/redirects/{id}"),
                json!({"version":0,"source":"/old-guide","destination":"/articles"})
            )
            .await
            .status,
        422
    );
    for number in 1..=4 {
        let destination = if number == 1 {
            "/old-guide".into()
        } else {
            format!("/chain-{}", number - 1)
        };
        assert_eq!(admin.post("/admin/seo/redirects", json!({"version":0,"source":format!("/chain-{number}"),"destination":destination})).await.status, 302);
    }
    assert_eq!(
        admin
            .post(
                "/admin/seo/redirects",
                json!({"version":0,"source":"/chain-5","destination":"/chain-4"})
            )
            .await
            .status,
        422
    );
    let db = DB::connection().unwrap();
    let count = directory::seo::entities::redirect::Entity::find()
        .count(db.inner())
        .await
        .unwrap();
    for index in count..1000 {
        db.inner().execute_unprepared(&format!("INSERT INTO seo_redirects(source,destination,version,updated_at) VALUES('/cap-redirect-{index}','/articles',1,0)")).await.unwrap();
    }
    assert_eq!(
        admin
            .post(
                "/admin/seo/redirects",
                json!({"version":0,"source":"/cap-overflow","destination":"/articles"})
            )
            .await
            .status,
        422,
        "redirect cap must reject additional writes"
    );
    assert_eq!(
        directory::seo::entities::redirect::Entity::find()
            .count(db.inner())
            .await
            .unwrap(),
        1000
    );
    db.inner()
        .execute_unprepared("DELETE FROM seo_redirects WHERE source LIKE '/cap-redirect-%'")
        .await
        .unwrap();
}

async fn markdown_contract(
    admin: &mut Client,
    guest: &mut Client,
    listing_id: i64,
    article_id: i64,
) {
    let row = listing_row(listing_id).await;
    for path in [
        format!("/listings/{}.md", row.slug),
        "/articles/seo-article.md".into(),
    ] {
        let markdown = guest.get(&path).await;
        assert_eq!(markdown.status, 200, "{}", markdown.body);
        assert_eq!(
            markdown.headers["content-type"],
            "text/markdown; charset=utf-8"
        );
        assert_eq!(
            markdown.headers["cache-control"], "no-store",
            "SEO-006: Markdown must disable shared caching"
        );
        assert_eq!(markdown.headers["x-robots-tag"], "noindex");
        assert_eq!(markdown.headers["x-content-type-options"], "nosniff");
        assert!(markdown.body.contains("café →"));
        assert!(!markdown.body.contains("PRIVATE"));
        let head = guest.request("HEAD", &path, None, false).await;
        assert_eq!(head.status, 200);
        assert!(head.body.is_empty());
        for method in ["POST", "PUT", "DELETE", "OPTIONS"] {
            assert_eq!(guest.request(method, &path, None, false).await.status, 405);
        }
    }
    for path in [
        "/admin/seo.md",
        "/articles/seo-private.md",
        "/.md",
        "/articles/seo-article.md.md",
        "/articles/seo%2farticle.md",
        "/listings/../private.md",
    ] {
        assert_ne!(guest.get(path).await.status, 200, "accepted {path}");
    }
    let article = article_row(article_id).await;
    let mut updated = article_input(
        article.version,
        "seo-renamed",
        json!({"title":"Renamed search title"}),
    );
    updated["title"] = json!("Renamed article");
    assert_eq!(
        admin
            .post(&format!("/admin/articles/{article_id}"), updated)
            .await
            .status,
        302
    );
    let version = article_row(article_id).await.version;
    assert_eq!(
        admin
            .post(
                &format!("/admin/articles/{article_id}/publish"),
                json!({"version":version})
            )
            .await
            .status,
        302
    );
    assert_eq!(
        guest.get("/articles/seo-article").await.location.as_deref(),
        Some("/articles/seo-renamed")
    );
    assert_eq!(guest.get("/articles/seo-article.md").await.status, 404);
    assert_eq!(guest.get("/articles/seo-renamed.md").await.status, 200);
    assert_eq!(
        props(guest, "/articles/seo-renamed").await["seo"]["markdown"],
        "https://catalog.example.test/articles/seo-renamed.md"
    );
    // Removing publication immediately removes both the twin and manual redirects to that old target.
    let version = article_row(article_id).await.version;
    assert_eq!(
        admin
            .post(
                &format!("/admin/articles/{article_id}/unpublish"),
                json!({"version":version})
            )
            .await
            .status,
        302
    );
    assert_eq!(guest.get("/articles/seo-renamed.md").await.status, 404);
    assert_ne!(guest.get("/old-guide").await.status, 301);
    let version = article_row(article_id).await.version;
    assert_eq!(
        admin
            .post(
                &format!("/admin/articles/{article_id}/publish"),
                json!({"version":version})
            )
            .await
            .status,
        302
    );
    let db = DB::connection().unwrap();
    db.inner()
        .execute_unprepared("ALTER TABLE articles RENAME TO unavailable_seo_articles")
        .await
        .unwrap();
    let unavailable = guest.get("/articles/seo-renamed.md").await;
    db.inner()
        .execute_unprepared("ALTER TABLE unavailable_seo_articles RENAME TO articles")
        .await
        .unwrap();
    assert_eq!(unavailable.status, 500);
    assert_eq!(unavailable.headers["cache-control"], "no-store");
    assert!(!unavailable.body.contains("unavailable_seo_articles"));
}

async fn missing_contract(admin: &mut Client, guest: &mut Client) {
    for path in [
        "/missing-guide?password=PRIVATE404",
        "/missing-guide?token=PRIVATE404",
        "/reset-password/PRIVATE404",
        "/encoded%2fPRIVATE404",
        "/admin/PRIVATE404",
    ] {
        guest.get(path).await;
    }
    let report = props(admin, "/admin/seo?missing_page=1").await;
    assert!(!report["not_found"].to_string().contains("PRIVATE404"));
    let row = report["not_found"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path"] == "/missing-guide")
        .unwrap();
    assert_eq!(row["hits"], 2);
    let db = DB::connection().unwrap();
    db.inner().execute_unprepared("INSERT INTO seo_not_found(path,hits,first_seen,last_seen) VALUES('/expired-observation',1,1,1)").await.unwrap();
    assert!(
        !props(admin, "/admin/seo").await["not_found"]
            .to_string()
            .contains("expired-observation")
    );
    db.inner()
        .execute_unprepared("DELETE FROM seo_not_found")
        .await
        .unwrap();
    let now = chrono::Utc::now().timestamp();
    for index in 0..1000 {
        db.inner().execute_unprepared(&format!("INSERT INTO seo_not_found(path,hits,first_seen,last_seen) VALUES('/cap-{index}',1,{now},{now})")).await.unwrap();
    }
    guest.get("/new-observation").await;
    assert_eq!(
        directory::seo::entities::not_found::Entity::find()
            .count(db.inner())
            .await
            .unwrap(),
        1000
    );
    assert_eq!(
        props(admin, "/admin/seo").await["not_found"]
            .as_array()
            .unwrap()
            .len(),
        25
    );
    db.inner()
        .execute_unprepared("DELETE FROM seo_not_found")
        .await
        .unwrap();
    db.inner()
        .execute_unprepared("ALTER TABLE seo_not_found RENAME TO unavailable_observations")
        .await
        .unwrap();
    let unavailable = guest.get("/observation-unavailable").await;
    db.inner()
        .execute_unprepared("ALTER TABLE unavailable_observations RENAME TO seo_not_found")
        .await
        .unwrap();
    assert_eq!(
        unavailable.status, 404,
        "observational write failure must not replace the original response"
    );
    guest.get("/missing-guide").await;
}

#[tokio::test(flavor = "current_thread")]
async fn seo_workflows_contract() {
    let _mail = common::setup().await;
    let admin_user = account("admin", &ADMIN_PERMISSIONS).await;
    let owner = account("owner", &[]).await;
    let mut admin = login(&admin_user).await;
    let mut guest = Client::new();
    settings_contract(&mut admin, &owner).await;
    let db = DB::connection().unwrap();
    let category = category::ActiveModel {
        slug: Set("seo-category".into()),
        name: Set("SEO category".into()),
        active: Set(true),
        version: Set(1),
        ..Default::default()
    }
    .insert(db.inner())
    .await
    .unwrap();
    let listing_id = workflow::save(
        owner.id,
        None,
        serde_json::from_value(listing_input(
            category.id,
            0,
            json!({"title":"Shared search title","description":"Listing search description."}),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    approve(admin_user.id, owner.id, listing_id).await;
    entitlement::ActiveModel {
        id: Set("seo-entitlement".into()),
        listing_id: Set(listing_id),
        mode: Set("free".into()),
        status: Set("active".into()),
        valid_from: Set(0),
        valid_until: Set(None),
        updated_at: Set(1),
    }
    .insert(db.inner())
    .await
    .unwrap();
    let article_id = articles::workflow::save(
        admin_user.id,
        None,
        serde_json::from_value(article_input(
            0,
            "seo-article",
            json!({"title":"Shared search title"}),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    articles::workflow::publish(admin_user.id, article_id, 1, true)
        .await
        .unwrap();
    articles::workflow::save(
        admin_user.id,
        None,
        serde_json::from_value(article_input(
            0,
            "seo-private",
            json!({"title":"Private article title"}),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
    metadata_contract(
        &mut admin,
        &mut guest,
        listing_id,
        article_id,
        &owner,
        category.id,
    )
    .await;
    discovery_contract(&mut admin, &mut guest, listing_id, article_id, category.id).await;
    redirects_contract(&mut admin, &mut guest).await;
    markdown_contract(&mut admin, &mut guest, listing_id, article_id).await;
    missing_contract(&mut admin, &mut guest).await;
    assert_eq!(
        guest.get("/admin/seo").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        admin
            .request(
                "POST",
                "/admin/seo",
                Some(settings(&mut login(&admin_user).await).await),
                false
            )
            .await
            .status,
        419
    );
    // A report query failure cannot turn into an all-clear report.
    db.inner()
        .execute_unprepared("ALTER TABLE articles RENAME TO unavailable_seo_report")
        .await
        .unwrap();
    let failed = admin.inertia_get("/admin/seo").await;
    db.inner()
        .execute_unprepared("ALTER TABLE unavailable_seo_report RENAME TO articles")
        .await
        .unwrap();
    assert_eq!(failed.status, 500);
}
