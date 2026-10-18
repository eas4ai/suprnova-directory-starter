use std::collections::BTreeMap;

use directory::{
    commands::admin_access::{ADMIN_PERMISSIONS, AccessAction, change_access},
    migrations::Migrator,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::{MigrationTrait, MigratorTrait, SchemaManager};
use suprnova::session::{SessionStore, driver::DatabaseSessionDriver};

#[path = "fixtures/m20260909_000009_create_account_access_3_2_1.rs"]
mod old_access;
#[path = "fixtures/rbac_3_2_1.rs"]
mod old_rbac;
#[path = "fixtures/m20260910_000013_create_seo_controls_3_2_1.rs"]
mod old_seo;

struct LegacyMigrator;

#[async_trait::async_trait]
impl MigratorTrait for LegacyMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // The application migrations are unchanged since starter 3d568ec.
        // Freeze the framework migration separately: using today's constructor
        // here would silently turn this into a fresh-install test.
        Migrator::migrations()
            .into_iter()
            .filter_map(|migration| match migration.name() {
                "m20261005_000007_rbac_spatie_layout"
                | "m20261005_000004_create_sessions_table" => None,
                "m20260610_000001_create_rbac_tables" => {
                    Some(Box::new(old_rbac::Migration) as Box<dyn MigrationTrait>)
                }
                "m20260909_000009_create_account_access" => {
                    Some(Box::new(old_access::Migration) as Box<dyn MigrationTrait>)
                }
                "m20260910_000013_create_seo_controls" => {
                    Some(Box::new(old_seo::Migration) as Box<dyn MigrationTrait>)
                }
                _ => Some(migration),
            })
            .collect()
    }
}

async fn execute(db: &DatabaseConnection, sql: &str) {
    db.execute_unprepared(sql).await.unwrap();
}

async fn strings(db: &DatabaseConnection, sql: &str) -> Vec<String> {
    db.query_all_raw(Statement::from_string(DbBackend::Sqlite, sql))
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.try_get_by_index(0).unwrap())
        .collect()
}

async fn snapshot(db: &DatabaseConnection) -> BTreeMap<String, Vec<String>> {
    let mut result = BTreeMap::new();
    result.insert("schema".into(), strings(db,
        "SELECT type || ':' || name || ':' || coalesce(sql, '') FROM sqlite_master ORDER BY type, name").await);
    for table in strings(
        db,
        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
    )
    .await
    {
        let columns = strings(
            db,
            &format!("SELECT name FROM pragma_table_info('{table}') ORDER BY cid"),
        )
        .await;
        // quote preserves NULL, numeric values and binary payment-event bodies.
        let expression = columns
            .iter()
            .map(|column| format!("quote(\"{column}\")"))
            .collect::<Vec<_>>()
            .join(" || '|' || ");
        result.insert(
            table.clone(),
            strings(
                db,
                &format!("SELECT {expression} FROM \"{table}\" ORDER BY 1"),
            )
            .await,
        );
    }
    result
}

async fn permitted(id: i64, permission: &str) -> bool {
    suprnova::rbac::has_permission_for_model("directory.user", &id.to_string(), permission)
        .await
        .unwrap()
}

async fn assert_layout(db: &DatabaseConnection) {
    let manager = SchemaManager::new(db);
    for table in [
        "model_has_roles",
        "model_has_permissions",
        "role_has_permissions",
    ] {
        assert!(manager.has_table(table).await.unwrap(), "missing {table}");
    }
    for table in ["model_roles", "model_permissions", "role_permissions"] {
        assert!(
            !manager.has_table(table).await.unwrap(),
            "old {table} remains"
        );
    }
    let columns = strings(
        db,
        "SELECT name FROM pragma_table_info('sessions') ORDER BY cid",
    )
    .await;
    assert!(!columns.contains(&"csrf_token".to_owned()));
    for column in [
        "id",
        "user_id",
        "payload",
        "last_activity",
        "ip_address",
        "user_agent",
    ] {
        assert!(
            columns.contains(&column.to_owned()),
            "missing sessions.{column}"
        );
    }
    assert_eq!(
        strings(
            db,
            "SELECT upper(type) FROM pragma_table_info('sessions') WHERE name = 'user_id'"
        )
        .await,
        ["INTEGER"]
    );
}

#[tokio::test]
async fn empty_and_populated_3_2_1_databases_upgrade_without_data_or_permission_loss() {
    assert_eq!(std::env::var("APP_ENV").as_deref(), Ok("test"));
    directory::config::register_all();
    directory::bootstrap::register().await;
    let connection = suprnova::DB::connection().unwrap();
    let db = connection.inner();

    // The fresh-install case has its own isolated database.
    let fresh = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&fresh, None).await.unwrap();
    assert_layout(&fresh).await;
    let first = snapshot(&fresh).await;
    Migrator::up(&fresh, None).await.unwrap();
    assert_eq!(first, snapshot(&fresh).await);

    LegacyMigrator::up(db, None).await.unwrap();
    assert!(
        SchemaManager::new(db)
            .has_column("roles", "display_name")
            .await
            .unwrap()
    );
    assert!(
        SchemaManager::new(db)
            .has_column("sessions", "csrf_token")
            .await
            .unwrap()
    );
    execute(db, include_str!("fixtures/upgrade_3_2_1.sql")).await;
    let before = snapshot(db).await;
    Migrator::up(db, None).await.unwrap();
    assert_layout(db).await;
    let after = snapshot(db).await;
    for table in [
        "users",
        "listings",
        "listing_revisions",
        "publishing_plans",
        "publishing_purchases",
        "publishing_payments",
        "publishing_events",
        "publishing_receipts",
        "publication_entitlements",
    ] {
        assert!(!before[table].is_empty(), "fixture must exercise {table}");
        assert_eq!(before[table], after[table], "migration changed {table}");
    }
    assert_eq!(
        strings(
            db,
            "SELECT display_name FROM suprnova_role_details WHERE role_id = 31"
        )
        .await,
        ["Legacy moderator"]
    );
    assert_eq!(
        strings(
            db,
            "SELECT display_name FROM suprnova_permission_details WHERE permission_id = 41"
        )
        .await,
        ["Enter administration"]
    );
    assert_eq!(
        strings(
            db,
            "SELECT CAST(user_id AS TEXT) FROM sessions WHERE id = 'legacy-session'"
        )
        .await,
        ["9000000001"]
    );
    let store = DatabaseSessionDriver::new(std::time::Duration::from_secs(100 * 365 * 86400));
    let session = store
        .read("legacy-session")
        .await
        .unwrap()
        .expect("migrated session");
    assert_eq!(session.user_id.as_deref(), Some("9000000001"));
    assert_eq!(session.csrf_token, "legacy-csrf");
    assert_eq!(session.data["draft"], "Keep this draft");
    assert!(permitted(9000000001, "admin.access").await);
    assert!(permitted(9000000001, "listings.moderate").await);
    assert!(permitted(9000000002, "articles.manage").await);
    assert!(!permitted(9000000002, "listings.moderate").await);
    for permission in ADMIN_PERMISSIONS {
        assert!(
            !permitted(9000000003, permission).await,
            "unrelated role gained {permission}"
        );
    }
    Migrator::up(db, None).await.unwrap();
    assert_eq!(
        after,
        snapshot(db).await,
        "repeated migration changed schema or data"
    );

    for id in [9000000001, 9000000002, 9000000003] {
        change_access(id, AccessAction::Grant).await.unwrap();
        change_access(id, AccessAction::Grant).await.unwrap();
        for permission in ADMIN_PERMISSIONS {
            assert!(permitted(id, permission).await);
        }
        change_access(id, AccessAction::Revoke).await.unwrap();
        change_access(id, AccessAction::Revoke).await.unwrap();
        for permission in ADMIN_PERMISSIONS {
            assert!(!permitted(id, permission).await);
        }
    }
    assert!(
        permitted(9000000003, "community.read").await,
        "revocation removed unrelated access"
    );
}
