#[allow(dead_code)]
mod common;

use common::Client;
use directory::{
    accounts::{self, SaveAccess, queries},
    commands::admin_access::{ADMIN_PERMISSIONS, AccessAction, change_access},
    listings::{
        entities::{audit, category, entitlement, listing},
        queries::Page,
        workflow,
    },
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

const PASSWORD: &str = "fixture-password-123";

async fn account(name: &str, verified: bool) -> User {
    let mut user = User::create(
        name,
        &format!("administration-{name}@example.test"),
        PASSWORD,
    )
    .await
    .unwrap();
    if verified {
        user.email_verified_at = Some(chrono::Utc::now());
        user.save().await.unwrap();
    }
    user
}

async fn login(user: &User, remember: bool) -> Client {
    let mut client = Client::new();
    assert_eq!(client.get("/login").await.status, 200);
    let response = client
        .post(
            "/login",
            json!({"email":user.email,"password":PASSWORD,"remember":remember}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    client
}

async fn props(client: &mut Client, path: &str) -> Value {
    let response = client.inertia_get(path).await;
    assert_eq!(response.status, 200, "{path}: {}", response.body);
    serde_json::from_str::<Value>(&response.body).unwrap()["props"].clone()
}

async fn count_audits() -> u64 {
    audit::Entity::find()
        .count(DB::connection().unwrap().inner())
        .await
        .unwrap()
}

async fn state(actor: i64, target: i64) -> Value {
    serde_json::to_value(queries::detail(actor, target).await.unwrap()).unwrap()
}

fn access(version: i64, roles: &[&str], suspended: bool) -> Value {
    json!({"version":version,"roles":roles,"suspended":suspended})
}

async fn permission(user: &User, capability: &str) {
    suprnova::rbac::give_permission_to_model("directory.user", &user.id.to_string(), capability)
        .await
        .unwrap();
}

fn listing_input(version: i64, category: i64) -> Value {
    json!({"version":version,"title":"Administration publication proof","summary":"Account access proof",
        "description":"Visible while its owner can publish.","url":"https://example.test/resource",
        "category_ids":[category],"media_id":null,"media_alt":""})
}

async fn administrative_decision_audits(actor: i64, client: &mut Client) {
    use directory::{
        articles::entities::{article, term},
        billing::lifecycle_entities::plan,
    };
    const MARKER: &str = "administration-audit-private-description-marker";
    let db = DB::connection().unwrap();
    let response = client.post("/admin/taxonomy/category", json!({
        "version":0,"slug":"administration-audit-category","name":"Audit category","active":true
    })).await;
    assert_eq!(response.status, 302, "{}", response.body);
    let category = term::Entity::find()
        .filter(term::Column::Slug.eq("administration-audit-category"))
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    let response = client.post("/admin/articles", json!({
        "version":0,"slug":"administration-audit-article","title":"Audit article","summary":"Audit proof",
        "body":MARKER,"term_ids":[category.id],"media_id":null,"media_alt":""
    })).await;
    assert_eq!(response.status, 302, "{}", response.body);
    let article_id: i64 = response
        .location
        .unwrap()
        .trim_end_matches("/edit")
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let draft = article::Entity::find_by_id(article_id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    let response = client
        .post(
            &format!("/admin/articles/{article_id}/publish"),
            json!({"version":draft.version}),
        )
        .await;
    assert_eq!(response.status, 302, "{}", response.body);
    let published = article::Entity::find_by_id(article_id)
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    assert!(published.published_revision_id.is_some());
    let mut plan_input = json!({"version":0,"key":"administration-audit-plan","name":"Audit plan",
        "description":MARKER,"enabled":true,"billing_type":"free","amount":0,"currency":"USD"});
    let response = client.post("/admin/plans", plan_input.clone()).await;
    assert_eq!(response.status, 302, "{}", response.body);
    let saved_plan = plan::Entity::find_by_id("administration-audit-plan")
        .one(db.inner())
        .await
        .unwrap()
        .unwrap();
    for (target, id, action) in [
        ("article_term", category.id.to_string(), "taxonomy_saved"),
        ("article", article_id.to_string(), "published"),
        ("publishing_plan", saved_plan.key.clone(), "created"),
    ] {
        let entries = audit::Entity::find()
            .filter(audit::Column::TargetType.eq(target))
            .filter(audit::Column::TargetId.eq(&id))
            .filter(audit::Column::Action.eq(action))
            .all(db.inner())
            .await
            .unwrap();
        assert_eq!(
            entries.len(),
            1,
            "one attributable audit for {target}/{id}/{action}"
        );
        let entry = &entries[0];
        assert_eq!(entry.actor_id, actor);
        assert_eq!(entry.actor_type, "user");
        assert!(entry.created_at > 0);
        assert!(!entry.summary.is_empty());
        assert!(!entry.summary.contains(MARKER));
    }
    let log = props(client, "/admin/audit?per_page=100").await;
    assert!(!log.to_string().contains(MARKER));
    let audits = count_audits().await;
    // Failure must occur after the domain writes, at the audit insertion boundary.
    db.inner().execute_unprepared(
        "CREATE TRIGGER reject_decision_audit BEFORE INSERT ON administrative_audit WHEN NEW.target_type IN ('article_term', 'article', 'publishing_plan') BEGIN SELECT RAISE(FAIL, 'controlled decision audit interruption'); END"
    ).await.unwrap();
    for target in ["article_term", "article", "publishing_plan"] {
        let response = match target {
            "article_term" => client.post(&format!("/admin/taxonomy/category/{}",category.id),json!({
                "version":category.version,"slug":category.slug,"name":"Uncommitted rename","active":false
            })).await,
            "article" => client.post(&format!("/admin/articles/{article_id}/unpublish"),json!({"version":published.version})).await,
            _ => {
                plan_input["version"] = json!(saved_plan.version);
                plan_input["name"] = json!("Uncommitted plan name");
                plan_input["enabled"] = json!(false);
                client.post("/admin/plans/administration-audit-plan",plan_input.clone()).await
            }
        };
        assert_eq!(response.status, 500, "{target}: {}", response.body);
        assert_eq!(
            term::Entity::find_by_id(category.id)
                .one(db.inner())
                .await
                .unwrap()
                .unwrap(),
            category
        );
        assert_eq!(
            article::Entity::find_by_id(article_id)
                .one(db.inner())
                .await
                .unwrap()
                .unwrap(),
            published
        );
        assert_eq!(
            plan::Entity::find_by_id(saved_plan.key.clone())
                .one(db.inner())
                .await
                .unwrap()
                .unwrap(),
            saved_plan
        );
        assert_eq!(count_audits().await, audits);
    }
    db.inner()
        .execute_unprepared("DROP TRIGGER reject_decision_audit")
        .await
        .unwrap();
    let remaining: i64 = DB::scalar("SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name = 'reject_decision_audit'", vec![]).await.unwrap();
    assert_eq!(remaining, 0, "controlled audit failure was removed");
}

#[tokio::test(flavor = "current_thread")]
async fn administration_workflows_contract() {
    let _mail = common::setup().await;
    let admin = account("operator", true).await;
    let member = account("member", true).await;
    let unverified = account("unverified", false).await;
    // Migration must have seeded these before the host provisioning command runs.
    for name in accounts::roles::PREDEFINED {
        use suprnova::rbac::entity::{RoleColumn, RoleEntity};
        assert!(
            RoleEntity::find()
                .filter(RoleColumn::Name.eq(name))
                .filter(RoleColumn::GuardName.eq("web"))
                .one(DB::connection().unwrap().inner())
                .await
                .unwrap()
                .is_some(),
            "missing seeded role {name}"
        );
    }
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    let mut admin_http = login(&admin, false).await;
    let mut member_http = login(&member, false).await;
    let mut guest = Client::new();
    assert_eq!(
        guest.get("/admin/accounts").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        member_http.get("/admin/accounts").await.location.as_deref(),
        Some("/dashboard")
    );

    // Each account has exactly the entry permission and one independent capability.
    let matrix = [
        ("moderation", "listings.moderate", "/admin/listings"),
        ("editorial", "articles.manage", "/admin/articles"),
        ("taxonomy", "taxonomy.manage", "/admin/taxonomy"),
        ("accounts", "accounts.manage", "/admin/accounts"),
        ("billing", "billing.configure", "/admin/billing"),
        ("audit", "audit.view", "/admin/audit"),
    ];
    let mut delegated = Vec::new();
    for (name, capability, allowed) in matrix {
        let user = account(name, true).await;
        permission(&user, "admin.access").await;
        permission(&user, capability).await;
        let mut client = login(&user, false).await;
        for (_, _, path) in matrix {
            let response = client.get(path).await;
            assert_eq!(
                response.status,
                if path == allowed { 200 } else { 403 },
                "{capability} at {path}: {}",
                response.body
            );
        }
        let mutation_matrix = [
            (
                "listings.moderate",
                "/admin/listings/99999/decision",
                json!({"version":0,"revision_id":1,"decision":"approve","reason":""}),
            ),
            (
                "articles.manage",
                "/admin/articles",
                json!({"version":0,"slug":"matrix-denied","title":"Denied","summary":"Denied","body":"Denied","term_ids":[],"media_id":null,"media_alt":""}),
            ),
            (
                "taxonomy.manage",
                "/admin/taxonomy/category",
                json!({"version":0,"slug":"matrix-denied","name":"Denied","active":true}),
            ),
            ("billing.configure", "/admin/billing", json!({})),
        ];
        let audit_count = count_audits().await;
        for (required, path, input) in mutation_matrix {
            if capability != required {
                let response = client.post(path, input).await;
                assert_eq!(
                    response.status, 403,
                    "{capability} mutation at {path}: {}",
                    response.body
                );
                assert_eq!(count_audits().await, audit_count);
            }
        }
        if capability != "accounts.manage" {
            assert_eq!(
                client
                    .post(
                        &format!("/admin/accounts/{}", member.id),
                        access(0, &["administrator"], false)
                    )
                    .await
                    .status,
                403
            );
        }
        delegated.push(user);
    }
    let before = state(admin.id, member.id).await;
    assert!(
        accounts::save(
            member.id,
            member.id,
            SaveAccess {
                version: 0,
                roles: vec!["administrator".into()],
                suspended: false
            }
        )
        .await
        .is_err()
    );
    assert_eq!(state(admin.id, member.id).await, before);
    permission(&unverified, "admin.access").await;
    permission(&unverified, "accounts.manage").await;
    let mut unverified_http = login(&unverified, false).await;
    assert_eq!(unverified_http.get("/admin/accounts").await.status, 403);
    assert_eq!(
        unverified_http
            .post(
                &format!("/admin/accounts/{}", member.id),
                access(0, &[], true)
            )
            .await
            .status,
        403
    );
    assert_eq!(
        admin_http
            .post(
                &format!("/admin/accounts/{}", unverified.id),
                access(0, &["editor"], false)
            )
            .await
            .status,
        422
    );
    assert!(
        change_access(unverified.id, AccessAction::Grant)
            .await
            .is_err()
    );

    // Predefined roles grant only their seeded permissions, and role changes are live.
    let path = format!("/admin/accounts/{}", member.id);
    for (version, role, allowed) in [
        (0, "moderator", "/admin/listings"),
        (1, "editor", "/admin/articles"),
    ] {
        assert_eq!(
            admin_http
                .post(&path, access(version, &[role], false))
                .await
                .status,
            302
        );
        assert_eq!(member_http.get(allowed).await.status, 200);
        assert_eq!(member_http.get("/admin/accounts").await.status, 403);
        assert_eq!(member_http.get("/admin/billing").await.status, 403);
        assert_eq!(member_http.get("/admin/taxonomy").await.status, 403);
    }
    let before = state(admin.id, member.id).await;
    let audits = count_audits().await;
    for data in [
        access(0, &[], true),
        access(2, &["invented-superuser"], false),
        json!({"version":2,"roles":[],"suspended":false,"permissions":["accounts.manage"]}),
        json!({"version":2,"roles":[],"suspended":false,"email_verified_at":"2026-01-01"}),
    ] {
        assert_eq!(admin_http.post(&path, data).await.status, 422);
        assert_eq!(state(admin.id, member.id).await, before);
        assert_eq!(count_audits().await, audits);
    }

    // Prove a failing audit insert rolls back access and role assignments together.
    DB::connection().unwrap().inner().execute_unprepared("CREATE TRIGGER reject_account_audit BEFORE INSERT ON administrative_audit WHEN NEW.target_type = 'account' BEGIN SELECT RAISE(FAIL, 'controlled account audit interruption'); END").await.unwrap();
    assert_eq!(
        admin_http
            .post(&path, access(2, &["moderator"], true))
            .await
            .status,
        500
    );
    assert_eq!(state(admin.id, member.id).await, before);
    assert_eq!(count_audits().await, audits);
    DB::connection()
        .unwrap()
        .inner()
        .execute_unprepared("DROP TRIGGER reject_account_audit")
        .await
        .unwrap();

    // An approved, entitled listing is public before account suspension.
    let category = category::ActiveModel {
        slug: Set("administration-proof".into()),
        name: Set("Administration proof".into()),
        active: Set(true),
        version: Set(1),
        ..Default::default()
    }
    .insert(DB::connection().unwrap().inner())
    .await
    .unwrap()
    .id;
    let id = workflow::save(
        member.id,
        None,
        serde_json::from_value(listing_input(0, category)).unwrap(),
    )
    .await
    .unwrap();
    workflow::submit(member.id, id, 1).await.unwrap();
    let row = listing::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap();
    workflow::decide(admin.id,id,serde_json::from_value(json!({"version":row.version,"revision_id":row.current_revision_id,"decision":"approve","reason":""})).unwrap()).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    entitlement::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        listing_id: Set(id),
        mode: Set("free".into()),
        status: Set("active".into()),
        valid_from: Set(now - 60),
        valid_until: Set(None),
        updated_at: Set(now),
    }
    .insert(DB::connection().unwrap().inner())
    .await
    .unwrap();
    let listing_path = format!("/listings/{}", row.slug);
    assert_eq!(guest.get(&listing_path).await.status, 200);
    let mut existing = login(&member, false).await;
    let mut remembered = login(&member, true).await;
    remembered.forget_session_cookie();
    // Establish the carrier works before suspension, then keep a second stale carrier.
    assert_eq!(remembered.get("/dashboard").await.status, 200);
    let mut stale_remembered = login(&member, true).await;
    stale_remembered.forget_session_cookie();
    assert_eq!(
        admin_http
            .post(&path, access(2, &["editor"], true))
            .await
            .status,
        302
    );
    assert_eq!(
        existing.get("/dashboard").await.location.as_deref(),
        Some("/login")
    );
    assert_eq!(
        stale_remembered.get("/dashboard").await.location.as_deref(),
        Some("/login")
    );
    let listing_before = listing::Entity::find_by_id(id)
        .one(DB::connection().unwrap().inner())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        member_http
            .post(
                &format!("/dashboard/listings/{id}"),
                listing_input(listing_before.version, category)
            )
            .await
            .location
            .as_deref(),
        Some("/login")
    );
    assert!(
        workflow::save(
            member.id,
            Some(id),
            serde_json::from_value(listing_input(listing_before.version, category)).unwrap()
        )
        .await
        .is_err()
    );
    assert_eq!(
        listing::Entity::find_by_id(id)
            .one(DB::connection().unwrap().inner())
            .await
            .unwrap()
            .unwrap(),
        listing_before
    );
    let mut rejected = Client::new();
    assert_eq!(rejected.get("/login").await.status, 200);
    assert_eq!(
        rejected
            .post(
                "/login",
                json!({"email":member.email,"password":PASSWORD,"remember":true})
            )
            .await
            .status,
        422
    );
    assert_eq!(guest.get(&listing_path).await.status, 404);
    assert_eq!(
        admin_http.post(&path, access(3, &[], false)).await.status,
        302
    );
    assert_eq!(state(admin.id, member.id).await["verified"], true);
    assert_eq!(
        User::find_by_email(&member.email)
            .await
            .unwrap()
            .unwrap()
            .email_verified_at,
        member.email_verified_at
    );
    assert_eq!(guest.get(&listing_path).await.status, 200);
    // Reinstatement preserves independent moderation and entitlement checks.
    let mut moderated: listing::ActiveModel = listing_before.clone().into();
    moderated.suspended = Set(true);
    moderated
        .update(DB::connection().unwrap().inner())
        .await
        .unwrap();
    assert_eq!(
        admin_http.post(&path, access(4, &[], true)).await.status,
        302
    );
    assert_eq!(
        admin_http.post(&path, access(5, &[], false)).await.status,
        302
    );
    assert_eq!(guest.get(&listing_path).await.status, 404);

    // Search and audit return bounded DTOs with no credential values or fields.
    let secret = "administration-remember-secret-marker";
    member
        .update_remember_token(Some(secret.into()))
        .await
        .unwrap();
    for suffix in ["?page=0", "?per_page=101", "?per_page=0"] {
        assert_eq!(
            admin_http
                .get(&format!("/admin/accounts{suffix}"))
                .await
                .status,
            422
        );
        assert_eq!(
            admin_http
                .get(&format!("/admin/audit{suffix}"))
                .await
                .status,
            422
        );
    }
    assert_eq!(
        admin_http
            .get(&format!("/admin/accounts?q={}", "x".repeat(201)))
            .await
            .status,
        422
    );
    let search = props(&mut admin_http, "/admin/accounts?per_page=2").await;
    assert_eq!(search["accounts"].as_array().unwrap().len(), 2);
    assert_eq!(search["pagination"]["per_page"], 2);
    let found = props(
        &mut admin_http,
        "/admin/accounts?q=administration-member%40example.test",
    )
    .await;
    assert_eq!(found["accounts"].as_array().unwrap().len(), 1);
    assert_eq!(found["accounts"][0]["id"], member.id);
    let detail = props(&mut admin_http, &path).await;
    let log = props(&mut admin_http, "/admin/audit?per_page=2").await;
    assert_eq!(log["entries"].as_array().unwrap().len(), 2);
    for payload in [&search, &found, &detail, &log] {
        let text = payload.to_string();
        for marker in [
            PASSWORD,
            member.password.as_str(),
            secret,
            "remember_token",
            "\"password\"",
        ] {
            assert!(
                !text.contains(marker),
                "credential leaked in administrative output"
            );
        }
    }
    assert!(
        queries::search(member.id, "", Page { number: 1, size: 2 })
            .await
            .is_err()
    );
    assert!(
        queries::audit(member.id, Page { number: 1, size: 2 })
            .await
            .is_err()
    );
    let (entries, _) = queries::audit(
        admin.id,
        Page {
            number: 1,
            size: 100,
        },
    )
    .await
    .unwrap();
    assert!(entries.iter().any(|e| e.target_type == "account"
        && e.target_id == member.id.to_string()
        && e.actor_id == admin.id
        && e.actor_type == "user"
        && e.created_at > 0
        && e.summary.contains("role")));
    assert!(
        entries
            .iter()
            .any(|e| e.actor_type == "operator" && e.action == "operator_granted")
    );

    administrative_decision_audits(admin.id, &mut admin_http).await;

    // Leave only a newly provisioned administrator; HTTP cannot remove the last one.
    let recovery = account("recovery", true).await;
    change_access(recovery.id, AccessAction::Grant)
        .await
        .unwrap();
    change_access(admin.id, AccessAction::Revoke).await.unwrap();
    for user in &delegated {
        change_access(user.id, AccessAction::Revoke).await.unwrap();
    }
    let mut recovery_http = login(&recovery, false).await;
    let recovery_path = format!("/admin/accounts/{}", recovery.id);
    let before = state(recovery.id, recovery.id).await;
    let audits = count_audits().await;
    for data in [access(1, &[], false), access(1, &["administrator"], true)] {
        assert_eq!(recovery_http.post(&recovery_path, data).await.status, 422);
        assert_eq!(state(recovery.id, recovery.id).await, before);
        assert_eq!(count_audits().await, audits);
    }
    // Suspend the earlier verified administrator, then recover it through the host API.
    let version = state(recovery.id, admin.id).await["version"]
        .as_i64()
        .unwrap();
    assert_eq!(
        recovery_http
            .post(
                &format!("/admin/accounts/{}", admin.id),
                access(version, &[], true)
            )
            .await
            .status,
        302
    );
    assert!(!accounts::is_active(admin.id).await.unwrap());
    // A user snapshot loaded before suspension must not overwrite access state.
    // Framework verification/reset save this same user row independently.
    admin.save().await.unwrap();
    assert!(!accounts::is_active(admin.id).await.unwrap());
    {
        use suprnova::auth::UserProvider;
        let provider = accounts::provider::AccountUserProvider::new();
        provider
            .set_password(&admin.id.to_string(), &admin.password)
            .await
            .unwrap();
        assert!(!accounts::is_active(admin.id).await.unwrap());
        assert!(
            provider
                .retrieve_by_id(&admin.id.to_string())
                .await
                .unwrap()
                .is_none()
        );
    }
    change_access(admin.id, AccessAction::Grant).await.unwrap();
    assert!(accounts::is_active(admin.id).await.unwrap());
    for capability in ADMIN_PERMISSIONS {
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
    let mut restored = login(&admin, false).await;
    assert_eq!(restored.get("/admin/accounts").await.status, 200);
    assert_eq!(
        User::find_by_email(&admin.email)
            .await
            .unwrap()
            .unwrap()
            .email_verified_at,
        admin.email_verified_at
    );

    // Both administrators race to demote themselves. The shared access lock must
    // let one commit and make the other observe the last-administrator boundary.
    assert_eq!(
        queries::administrator_count(DB::connection().unwrap().inner())
            .await
            .unwrap(),
        2
    );
    let admin_version = state(admin.id, admin.id).await["version"].as_i64().unwrap();
    let recovery_version = state(admin.id, recovery.id).await["version"]
        .as_i64()
        .unwrap();
    let audit_count = count_audits().await;
    let (left, right) = tokio::join!(
        accounts::save(
            admin.id,
            admin.id,
            SaveAccess {
                version: admin_version,
                roles: vec![],
                suspended: false
            }
        ),
        accounts::save(
            recovery.id,
            recovery.id,
            SaveAccess {
                version: recovery_version,
                roles: vec![],
                suspended: false
            }
        ),
    );
    assert_eq!(usize::from(left.is_ok()) + usize::from(right.is_ok()), 1);
    let rejected = if let Err(error) = left {
        error
    } else {
        right.unwrap_err()
    };
    assert!(
        matches!(rejected, suprnova::FrameworkError::Validation(_)),
        "losing demotion must fail the last-admin rule: {rejected}"
    );
    assert_eq!(
        queries::administrator_count(DB::connection().unwrap().inner())
            .await
            .unwrap(),
        1
    );
    assert_eq!(count_audits().await, audit_count + 1);
    change_access(recovery.id, AccessAction::Grant)
        .await
        .unwrap();

    // Stable fixtures for the subsequent browser journey in this same test database.
    let browser_admin = account("browser-admin", true).await;
    account("browser-member", true).await;
    change_access(browser_admin.id, AccessAction::Grant)
        .await
        .unwrap();
}
