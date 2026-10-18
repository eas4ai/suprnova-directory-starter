#[allow(dead_code)]
mod common;

use common::Client;
use directory::{
    articles::{
        self,
        entities::{article, revision, term},
        queries,
        validation::SaveArticle,
        workflow,
    },
    commands::admin_access::{AccessAction, change_access},
    listings::entities::audit,
    models::user::User,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use suprnova::{
    DB,
    eloquent::Model,
    serde_json::{self, Value, json},
};

async fn account(name: &str, verified: bool) -> User {
    let mut user = User::create(
        name,
        &format!("editorial-{name}@example.test"),
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
    let mut client = Client::new();
    assert_eq!(client.get("/login").await.status, 200);
    let response = client
        .post(
            "/login",
            json!({"email":user.email,"password":"fixture-password-123"}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    client
}
async fn props(client: &mut Client, path: &str) -> Value {
    let response = client.inertia_get(path).await;
    assert_eq!(response.status, 200, "{}", response.body);
    serde_json::from_str::<Value>(&response.body).unwrap()["props"].clone()
}
fn input(version: i64, slug: &str, title: &str, terms: &[i64], image: Option<&str>) -> Value {
    json!({"version":version,"slug":slug,"title":title,"summary":"An editorial proof & useful guide.",
        "body":"**Visible proof**\n\n<script>window.editorialInjected = true</script>\n[unsafe](javascript:alert(1))",
        "term_ids":terms,"media_id":image,"media_alt":if image.is_some() {"Editorial cover"} else {""}})
}
async fn row(id: i64) -> article::Model {
    article::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
}
async fn create(client: &mut Client, data: Value) -> i64 {
    let response = client.post("/admin/articles", data).await;
    assert_eq!(response.status, 302, "{}", response.body);
    response
        .location
        .unwrap()
        .trim_end_matches("/edit")
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
async fn publish(client: &mut Client, id: i64, publish: bool) {
    let action = if publish { "publish" } else { "unpublish" };
    let response = client
        .post(
            &format!("/admin/articles/{id}/{action}"),
            json!({"version":row(id).await.version}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
}
async fn create_term(client: &mut Client, kind: &str, slug: &str) -> i64 {
    let response = client
        .post(
            &format!("/admin/taxonomy/{kind}"),
            json!({"version":0,"slug":slug,"name":slug,"active":true}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    term::Entity::find()
        .filter(term::Column::Kind.eq(kind))
        .filter(term::Column::Slug.eq(slug))
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap()
        .id
}
async fn upload(client: &mut Client) -> String {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(3, 2)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let response = client
        .raw_request(
            "POST",
            "/admin/articles/media",
            bytes.into_inner(),
            "image/png",
            true,
        )
        .await;
    assert_eq!(response.status, 201, "{}", response.body);
    serde_json::from_str::<Value>(&response.body).unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
async fn audit_count() -> u64 {
    audit::Entity::find()
        .count(DB::connection().unwrap().inner())
        .await
        .unwrap()
}

// A real XML parser establishes syntax, entities, namespaces and URL extraction.
fn parse_xml(body: &str) -> Value {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new("python3").args(["-c", "import sys,json,xml.etree.ElementTree as E; r=E.fromstring(sys.stdin.buffer.read()); print(json.dumps({'root':r.tag,'locations':[e.text for e in r.iter() if e.tag.endswith('}loc')],'titles':[e.text for e in r.iter('title')],'items':len(list(r.iter('item')))}))"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "XML parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

async fn sitemap_paths(guest: &mut Client) -> Vec<String> {
    let index = guest.get("/sitemap.xml").await;
    assert_eq!(index.status, 200, "{}", index.body);
    assert!(
        index.headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("application/xml")
    );
    let root = parse_xml(&index.body);
    assert!(root["root"].as_str().unwrap().ends_with("}sitemapindex"));
    let origin = directory::config::site::origin().unwrap();
    let mut all = Vec::new();
    for location in root["locations"].as_array().unwrap() {
        let location = location.as_str().unwrap();
        let path = location
            .strip_prefix(&origin)
            .expect("Sitemap host must be configured, not request-derived");
        let response = guest.get(path).await;
        assert_eq!(response.status, 200, "{path}: {}", response.body);
        let page = parse_xml(&response.body);
        let locations = page["locations"].as_array().unwrap();
        assert!(locations.len() <= 100);
        for location in locations {
            all.push(
                location
                    .as_str()
                    .unwrap()
                    .strip_prefix(&origin)
                    .unwrap()
                    .to_owned(),
            );
        }
    }
    let mut unique = all.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        all.len(),
        "Sitemap pages must not duplicate content"
    );
    all
}

#[tokio::test(flavor = "current_thread")]
async fn editorial_workflows_contract() {
    let _mail = common::setup().await;
    let admin = account("admin", true).await;
    let member = account("member", true).await;
    let moderator = account("moderator", true).await;
    let editor = account("editor", true).await;
    let unverified = account("unverified", false).await;
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    for (user, permission) in [
        (&moderator, directory::listings::MODERATE_PERMISSION),
        (&editor, articles::EDIT_PERMISSION),
        (&unverified, articles::EDIT_PERMISSION),
    ] {
        for capability in ["admin.access", permission] {
            suprnova::rbac::give_permission_to_model(
                "directory.user",
                &user.id.to_string(),
                capability,
            )
            .await
            .unwrap();
        }
    }
    let mut admin_http = login(&admin).await;
    let mut editor_http = login(&editor).await;
    let mut denied = login(&moderator).await;
    let mut ordinary = login(&member).await;
    let mut unverified_http = login(&unverified).await;
    let mut guest = Client::new();
    assert_eq!(
        guest.get("/admin/articles").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        ordinary.get("/admin/articles").await.location.as_deref(),
        Some("/dashboard")
    );
    for path in [
        "/admin/articles",
        "/admin/articles/create",
        "/admin/taxonomy",
    ] {
        assert_eq!(denied.get(path).await.status, 403);
    }
    assert_eq!(
        unverified_http.get("/admin/articles/create").await.status,
        403
    );
    assert_eq!(editor_http.get("/admin/taxonomy").await.status, 403);
    assert_eq!(
        denied
            .post("/admin/articles", input(0, "denied", "Denied", &[], None))
            .await
            .status,
        403
    );
    assert_eq!(
        admin_http
            .request(
                "POST",
                "/admin/articles",
                Some(input(0, "csrf", "CSRF", &[], None)),
                false
            )
            .await
            .status,
        419
    );
    let category = create_term(&mut admin_http, "category", "guides").await;
    let tag = create_term(&mut admin_http, "tag", "practical").await;
    let private_term = create_term(&mut admin_http, "tag", "private-only").await;
    let image = upload(&mut admin_http).await;
    let private_image = upload(&mut admin_http).await;
    assert_eq!(
        denied
            .raw_request("POST", "/admin/articles/media", vec![1], "image/png", true)
            .await
            .status,
        403
    );
    assert_eq!(
        admin_http
            .raw_request(
                "POST",
                "/admin/articles/media",
                b"<svg onload='alert(1)'/>".to_vec(),
                "image/svg+xml",
                true
            )
            .await
            .status,
        422
    );
    assert_eq!(
        admin_http
            .raw_request(
                "POST",
                "/admin/articles/media",
                vec![0; 5 * 1024 * 1024 + 1],
                "image/png",
                true
            )
            .await
            .status,
        413
    );
    let id = create(
        &mut editor_http,
        input(
            0,
            "public-proof",
            "Public proof & guide",
            &[category, tag],
            Some(&image),
        ),
    )
    .await;
    let private = create(
        &mut editor_http,
        input(
            0,
            "hidden-proof",
            "Hidden draft marker",
            &[private_term],
            None,
        ),
    )
    .await;
    assert_eq!(guest.get("/articles/public-proof").await.status, 404);
    assert_eq!(
        guest
            .get(&format!("/media/articles/public-proof/{image}"))
            .await
            .status,
        404
    );
    assert_eq!(
        denied
            .get(&format!("/admin/articles/media/{image}"))
            .await
            .status,
        403
    );
    assert_eq!(
        denied
            .get(&format!("/dashboard/listings/media/{image}"))
            .await
            .status,
        404,
        "Listing moderation must not expose editorial media"
    );
    assert_eq!(
        denied
            .get(&format!("/admin/articles/{id}/preview"))
            .await
            .status,
        403
    );
    assert_eq!(
        editor_http
            .get(&format!("/admin/articles/{id}/preview"))
            .await
            .status,
        200
    );
    assert_eq!(
        denied
            .post(
                &format!("/admin/articles/{id}/publish"),
                json!({"version":1})
            )
            .await
            .status,
        403
    );
    assert!(queries::terms(true).await.unwrap().is_empty());
    let before = audit_count().await;
    let mut invalid = input(1, "public-proof", "Invalid terms", &[i64::MAX], None);
    assert_eq!(
        editor_http
            .post(&format!("/admin/articles/{id}"), invalid.clone())
            .await
            .status,
        422
    );
    assert_eq!(row(id).await.version, 1);
    assert_eq!(audit_count().await, before);
    invalid["term_ids"] = json!([]);
    invalid["slug"] = json!("../escape");
    assert_eq!(
        editor_http
            .post(&format!("/admin/articles/{id}"), invalid)
            .await
            .status,
        422
    );
    publish(&mut editor_http, id, true).await;
    let public = props(&mut guest, "/articles/public-proof").await;
    assert_eq!(public["article"]["title"], "Public proof & guide");
    assert!(
        !public["article"]["body_html"]
            .as_str()
            .unwrap()
            .contains("<script")
    );
    assert!(
        !public["article"]["body_html"]
            .as_str()
            .unwrap()
            .contains("javascript:")
    );
    let html = guest.get("/articles/public-proof").await;
    assert_eq!(html.status, 200, "{}", html.body);
    assert!(
        html.body.contains("<strong>Visible proof</strong>"),
        "Initial HTML must contain visible rendered prose"
    );
    assert!(html.body.contains("property=\"og:title\""));
    assert!(html.body.contains("rel=\"canonical\""));
    assert_eq!(
        public["seo"]["canonical"],
        format!(
            "{}/articles/public-proof",
            directory::config::site::origin().unwrap()
        )
    );
    assert!(
        serde_json::from_str::<Value>(public["seo"]["structured_data"].as_str().unwrap())
            .unwrap()
            .is_array()
    );
    assert_eq!(
        guest
            .get(&format!("/media/articles/public-proof/{image}"))
            .await
            .status,
        200
    );
    assert_eq!(
        props(&mut guest, "/articles?category=guides&tag=practical").await["pagination"]["total"],
        1
    );
    assert_eq!(
        props(&mut guest, "/articles?q=Hidden").await["pagination"]["total"],
        0
    );
    assert_eq!(
        props(&mut guest, "/articles?tag=private-only").await["pagination"]["total"],
        0
    );
    assert!(
        !queries::terms(true)
            .await
            .unwrap()
            .iter()
            .any(|t| t.id == private_term)
    );
    assert_eq!(guest.get("/articles?per_page=101").await.status, 422);
    assert_eq!(guest.get("/articles?page=0").await.status, 422);
    let old = row(id).await;
    let response = editor_http
        .post(
            &format!("/admin/articles/{id}"),
            input(
                old.version,
                "revised-proof",
                "Unpublished revision marker",
                &[category, tag],
                Some(&private_image),
            ),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    assert_eq!(
        props(&mut guest, "/articles/public-proof").await["article"]["title"],
        "Public proof & guide"
    );
    assert_eq!(guest.get("/articles/revised-proof").await.status, 404);
    assert_eq!(
        guest
            .get(&format!("/media/articles/public-proof/{private_image}"))
            .await
            .status,
        404
    );
    assert_eq!(
        props(&mut editor_http, &format!("/admin/articles/{id}/preview")).await["article"]["current"]
            ["title"],
        "Unpublished revision marker"
    );
    assert_eq!(
        editor_http
            .post(
                &format!("/admin/articles/{id}"),
                input(old.version, "public-proof", "Stale overwrite", &[], None)
            )
            .await
            .status,
        422
    );
    let version = row(id).await.version;
    let mut left = editor_http.clone();
    let mut right = editor_http.clone();
    let path = format!("/admin/articles/{id}");
    let (a, b) = tokio::join!(
        left.post(
            &path,
            input(
                version,
                "revised-proof",
                "Concurrent revision A",
                &[category],
                Some(&private_image)
            )
        ),
        right.post(
            &path,
            input(
                version,
                "revised-proof",
                "Concurrent revision B",
                &[category],
                Some(&private_image)
            )
        )
    );
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [302, 422]);
    let before = row(id).await;
    let audits = audit_count().await;
    DB::connection().unwrap().inner().execute_unprepared("CREATE TRIGGER reject_article_audit BEFORE INSERT ON administrative_audit WHEN NEW.target_type = 'article' AND NEW.action = 'published' BEGIN SELECT RAISE(FAIL, 'controlled audit interruption'); END").await.unwrap();
    assert_eq!(
        editor_http
            .post(
                &format!("/admin/articles/{id}/publish"),
                json!({"version":before.version})
            )
            .await
            .status,
        500
    );
    assert_eq!(row(id).await, before);
    assert_eq!(audit_count().await, audits);
    DB::connection()
        .unwrap()
        .inner()
        .execute_unprepared("DROP TRIGGER reject_article_audit")
        .await
        .unwrap();
    publish(&mut editor_http, id, true).await;
    let old_url = guest.get("/articles/public-proof").await;
    assert_eq!(old_url.status, 301);
    assert_eq!(old_url.location.as_deref(), Some("/articles/revised-proof"));
    assert_eq!(guest.get("/articles/revised-proof").await.status, 200);
    assert_eq!(
        editor_http
            .post(
                &format!("/admin/articles/{private}"),
                input(
                    row(private).await.version,
                    "public-proof",
                    "Steal old URL",
                    &[],
                    None
                )
            )
            .await
            .status,
        422
    );
    assert_eq!(
        admin_http
            .post(
                &format!("/admin/taxonomy/category/{category}/remove"),
                json!({"version":1})
            )
            .await
            .status,
        422
    );
    assert_eq!(
        admin_http
            .post(
                &format!("/admin/taxonomy/tag/{category}"),
                json!({"version":1,"slug":"guides","name":"Wrong kind","active":true})
            )
            .await
            .status,
        404
    );
    assert_eq!(
        admin_http
            .post(
                &format!("/admin/taxonomy/category/{category}"),
                json!({"version":1,"slug":"guides","name":"Renamed guides","active":true})
            )
            .await
            .status,
        302
    );
    assert_eq!(
        props(&mut guest, "/articles/revised-proof").await["article"]["terms"][0]["name"],
        "Renamed guides"
    );
    assert_eq!(
        admin_http
            .post(
                &format!("/admin/taxonomy/category/{category}"),
                json!({"version":1,"slug":"guides","name":"Stale term","active":false})
            )
            .await
            .status,
        422
    );
    assert_eq!(
        admin_http
            .post(
                &format!("/admin/taxonomy/category/{category}"),
                json!({"version":2,"slug":"guides","name":"Renamed guides","active":false})
            )
            .await
            .status,
        302
    );
    assert_eq!(guest.get("/articles/revised-proof").await.status, 200);
    assert_eq!(
        props(&mut guest, "/articles?category=guides").await["pagination"]["total"],
        0
    );
    publish(&mut editor_http, id, false).await;
    for url in [
        "/articles/public-proof",
        "/articles/revised-proof",
        "/articles/hidden-proof",
    ] {
        assert_eq!(guest.get(url).await.status, 404);
    }
    assert_eq!(parse_xml(&guest.get("/feed.xml").await.body)["items"], 0);
    assert!(
        !sitemap_paths(&mut guest)
            .await
            .iter()
            .any(|path| path.starts_with("/articles/"))
    );
    for n in 0..105 {
        let data: SaveArticle = serde_json::from_value(input(
            0,
            &format!("feed-{n}"),
            &format!("Feed & proof {n}\u{fffe}"),
            &[],
            None,
        ))
        .unwrap();
        let id = workflow::save(admin.id, None, data).await.unwrap();
        workflow::publish(admin.id, id, 1, true).await.unwrap();
    }
    let feed = guest.get("/feed.xml").await;
    assert_eq!(feed.status, 200);
    assert!(
        feed.headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("application/rss+xml")
    );
    let parsed = parse_xml(&feed.body);
    assert_eq!(parsed["items"], 50);
    assert_eq!(parsed["titles"][1], "Feed & proof 104");
    let sitemap = sitemap_paths(&mut guest).await;
    assert_eq!(
        sitemap
            .iter()
            .filter(|path| path.starts_with("/articles/feed-"))
            .count(),
        105
    );
    assert!(!sitemap.iter().any(|path| path.contains("hidden-proof")
        || path.contains("revised-proof")
        || path.contains("private-only")
        || path.contains("/admin")));
    listing_outputs(&admin, &mut guest).await;
    let robots = guest.get("/robots.txt").await;
    assert_eq!(robots.status, 200);
    assert!(robots.body.contains(&format!(
        "Sitemap: {}/sitemap.xml",
        directory::config::site::origin().unwrap()
    )));
    assert!(!robots.body.contains("Disallow: /media/articles"));
    assert_eq!(guest.get("/sitemaps/articles/0.xml").await.status, 404);
    assert_eq!(guest.get("/sitemaps/articles/999.xml").await.status, 404);
    assert!(
        audit::Entity::find()
            .filter(audit::Column::Action.eq("published"))
            .count(DB::connection().unwrap().inner())
            .await
            .unwrap()
            > 0
    );
    assert_eq!(
        revision::Entity::find()
            .filter(revision::Column::ArticleId.eq(id))
            .count(DB::connection().unwrap().inner())
            .await
            .unwrap(),
        3
    );
    println!(
        "Editorial HTTP, publication, taxonomy, initial HTML, XML and private media contracts passed."
    );
}

async fn listing_outputs(admin: &User, guest: &mut Client) {
    use directory::listings::{
        entities::{entitlement, listing},
        queries as directory_queries, workflow as listing_workflow,
    };
    listing_workflow::seed_categories().await.unwrap();
    let category = directory_queries::categories().await.unwrap()[0].id;
    let now = chrono::Utc::now().timestamp();
    let mut allowed_slug = String::new();
    let mut denied_slugs = Vec::new();
    for (index, (mode, status, end, suspended)) in [
        ("live", "active", None, false),
        ("test", "active", None, false),
        ("live", "refunded", None, false),
        ("live", "active", Some(now - 1), false),
        ("free", "active", None, true),
    ]
    .into_iter()
    .enumerate()
    {
        let input=serde_json::from_value(json!({"version":0,"title":format!("Sitemap listing {index}"),"summary":"A listing output fixture","description":"**Listing visible proof**","url":"https://example.test/resource","category_ids":[category],"media_id":null,"media_alt":""})).unwrap();
        let id = listing_workflow::save(admin.id, None, input).await.unwrap();
        listing_workflow::submit(admin.id, id, 1).await.unwrap();
        let row = listing::Entity::find_by_id(id)
            .one(DB::connection().unwrap().inner())
            .await
            .unwrap()
            .unwrap();
        listing_workflow::decide(admin.id,id,serde_json::from_value(json!({"version":row.version,"revision_id":row.current_revision_id,"decision":"approve","reason":""})).unwrap()).await.unwrap();
        entitlement::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            listing_id: Set(id),
            mode: Set(mode.into()),
            status: Set(status.into()),
            valid_from: Set(now - 60),
            valid_until: Set(end),
            updated_at: Set(now),
        }
        .insert(DB::connection().unwrap().inner())
        .await
        .unwrap();
        if suspended {
            let mut changed: listing::ActiveModel = row.clone().into();
            changed.suspended = Set(true);
            changed
                .update(DB::connection().unwrap().inner())
                .await
                .unwrap();
        }
        if index == 0 {
            allowed_slug = row.slug;
        } else {
            denied_slugs.push(row.slug);
        }
    }
    let paths = sitemap_paths(guest).await;
    assert!(paths.contains(&format!("/listings/{allowed_slug}")));
    for slug in denied_slugs {
        assert!(!paths.contains(&format!("/listings/{slug}")));
    }
    let html = guest.get(&format!("/listings/{allowed_slug}")).await;
    assert_eq!(html.status, 200, "{}", html.body);
    assert!(html.body.contains("<strong>Listing visible proof</strong>"));
    let public = props(guest, &format!("/listings/{allowed_slug}")).await;
    assert_eq!(
        public["seo"]["canonical"],
        format!(
            "{}/listings/{allowed_slug}",
            directory::config::site::origin().unwrap()
        )
    );
}
