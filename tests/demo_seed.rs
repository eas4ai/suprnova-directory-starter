//! Exercise the distributed console binary against disposable, migrated databases.
use directory::{
    articles::entities::article,
    listings::entities::{category, entitlement, listing, revision},
    models::user,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;
use std::{path::PathBuf, process::Output};

struct Install {
    path: PathBuf,
    url: String,
    db: DatabaseConnection,
}
impl Install {
    async fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("directory-demo-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let url = format!(
            "sqlite://{}?mode=rwc",
            path.join("database.sqlite").display()
        );
        let db = Database::connect(&url).await.unwrap();
        directory::migrations::Migrator::up(&db, None)
            .await
            .unwrap();
        Self { path, url, db }
    }
    async fn run(&self, environment: &str, override_production: bool) -> Output {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_console"));
        command
            .current_dir(&self.path)
            .env_clear()
            .env("APP_ENV", environment)
            .env("APP_NAME", "Demo test")
            .env("APP_URL", "https://demo.example.test")
            .env("DATABASE_URL", &self.url)
            .env("MAIL_MAILER", "log")
            .arg("directory:demo");
        if override_production {
            command.arg("--allow-production");
        }
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            command.kill_on_drop(true).output(),
        )
        .await
        .unwrap()
        .unwrap()
    }
    async fn scalar(&self, sql: &str) -> i64 {
        self.db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                sql,
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get_by_index(0)
            .unwrap()
    }
    async fn operator(&self, email: &str) -> <user::Entity as EntityTrait>::Model {
        let now = chrono::Utc::now();
        user::ActiveModel {
            name: Set("Existing operator".into()),
            email: Set(email.into()),
            password: Set("operator-password-must-stay-byte-identical".into()),
            remember_token: Set(Some("operator-session-token".into())),
            email_verified_at: Set(None),
            created_at: Set(now.to_rfc3339()),
            updated_at: Set(now.to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .unwrap()
    }
}
impl Drop for Install {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn success(result: &Output) {
    assert!(
        result.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
fn refused(result: &Output, reason: &str) {
    assert!(!result.status.success());
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.contains(reason), "{output}");
}

#[tokio::test]
async fn console_seed_is_repeatable_and_preserves_operator_and_demo_credentials() {
    let install = Install::new().await;
    let operator = install.operator("operator@example.test").await;
    success(&install.run("test", false).await);
    let owner = user::Entity::find()
        .filter(user::Column::Email.eq(directory::demo::OWNER_EMAIL))
        .one(&install.db)
        .await
        .unwrap()
        .unwrap();
    assert!(owner.email_verified_at.is_none());
    assert!(owner.remember_token.is_none());
    assert!(!owner.password.contains("operator-password"));
    assert!(!suprnova::hashing::verify("password", &owner.password).unwrap());
    assert_eq!(
        category::Entity::find().count(&install.db).await.unwrap(),
        2
    );
    assert_eq!(listing::Entity::find().count(&install.db).await.unwrap(), 4);
    for status in ["approved", "submitted", "rejected", "draft"] {
        assert_eq!(
            revision::Entity::find()
                .filter(revision::Column::Status.eq(status))
                .count(&install.db)
                .await
                .unwrap(),
            1
        );
    }
    assert_eq!(article::Entity::find().count(&install.db).await.unwrap(), 2);
    assert_eq!(
        article::Entity::find()
            .filter(article::Column::PublishedRevisionId.is_not_null())
            .count(&install.db)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        listing::Entity::find()
            .filter(directory::listings::queries::eligible(
                chrono::Utc::now().timestamp()
            ))
            .count(&install.db)
            .await
            .unwrap(),
        1
    );
    let grant = entitlement::Entity::find()
        .one(&install.db)
        .await
        .unwrap()
        .unwrap();
    assert!(grant.id.starts_with("demo:synthetic:"));
    assert_eq!(grant.mode, "free");
    success(&install.run("test", false).await);
    assert_eq!(
        user::Entity::find_by_id(owner.id)
            .one(&install.db)
            .await
            .unwrap()
            .unwrap(),
        owner
    );
    assert_eq!(
        user::Entity::find_by_id(operator.id)
            .one(&install.db)
            .await
            .unwrap()
            .unwrap(),
        operator
    );
    assert_eq!(listing::Entity::find().count(&install.db).await.unwrap(), 4);
    assert_eq!(
        revision::Entity::find().count(&install.db).await.unwrap(),
        4
    );
    assert_eq!(article::Entity::find().count(&install.db).await.unwrap(), 2);
    for table in [
        "publishing_purchases",
        "owner_notifications",
        "model_roles",
        "model_permissions",
    ] {
        assert_eq!(
            install
                .scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .await,
            0
        );
    }
}

#[tokio::test]
async fn production_requires_override_even_after_successful_seed() {
    let install = Install::new().await;
    for environment in ["production", "PRODUCTION", "prod", ""] {
        refused(&install.run(environment, false).await, "--allow-production");
        assert_eq!(user::Entity::find().count(&install.db).await.unwrap(), 0);
    }
    success(&install.run("production", true).await);
    refused(
        &install.run("production", false).await,
        "--allow-production",
    );
    success(&install.run("production", true).await);
    assert_eq!(listing::Entity::find().count(&install.db).await.unwrap(), 4);
}

#[tokio::test]
async fn reserved_account_and_category_collisions_refuse_without_partial_writes() {
    let install = Install::new().await;
    let existing = install.operator(directory::demo::OWNER_EMAIL).await;
    refused(&install.run("test", false).await, "reserved demo");
    assert_eq!(
        user::Entity::find_by_id(existing.id)
            .one(&install.db)
            .await
            .unwrap()
            .unwrap(),
        existing
    );
    assert_eq!(
        category::Entity::find().count(&install.db).await.unwrap(),
        0
    );
    assert_eq!(listing::Entity::find().count(&install.db).await.unwrap(), 0);
    assert_eq!(
        install
            .scalar("SELECT completed FROM demo_seed_state")
            .await,
        0
    );
    let install = Install::new().await;
    let existing = category::ActiveModel {
        slug: Set("demo-learning".into()),
        name: Set("Operator category".into()),
        active: Set(false),
        version: Set(7),
        ..Default::default()
    }
    .insert(&install.db)
    .await
    .unwrap();
    refused(&install.run("test", false).await, "reserved demo");
    assert_eq!(
        category::Entity::find_by_id(existing.id)
            .one(&install.db)
            .await
            .unwrap()
            .unwrap(),
        existing
    );
    assert_eq!(user::Entity::find().count(&install.db).await.unwrap(), 0);
    assert_eq!(article::Entity::find().count(&install.db).await.unwrap(), 0);
}

#[tokio::test]
async fn late_write_failure_rolls_back_and_retry_succeeds() {
    let install = Install::new().await;
    install.db.execute_unprepared("CREATE TRIGGER reject_demo_article BEFORE INSERT ON articles BEGIN SELECT RAISE(ABORT, 'controlled demo failure'); END").await.unwrap();
    refused(&install.run("test", false).await, "Demo seed failed");
    assert_eq!(user::Entity::find().count(&install.db).await.unwrap(), 0);
    assert_eq!(
        category::Entity::find().count(&install.db).await.unwrap(),
        0
    );
    assert_eq!(listing::Entity::find().count(&install.db).await.unwrap(), 0);
    assert_eq!(
        entitlement::Entity::find()
            .count(&install.db)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        install
            .scalar("SELECT completed FROM demo_seed_state")
            .await,
        0
    );
    install
        .db
        .execute_unprepared("DROP TRIGGER reject_demo_article")
        .await
        .unwrap();
    success(&install.run("test", false).await);
}

#[tokio::test]
async fn concurrent_console_seeds_share_one_fixture_set() {
    let install = Install::new().await;
    let (first, second) = tokio::join!(install.run("test", false), install.run("test", false));
    success(&first);
    success(&second);
    assert_eq!(user::Entity::find().count(&install.db).await.unwrap(), 1);
    assert_eq!(listing::Entity::find().count(&install.db).await.unwrap(), 4);
    assert_eq!(article::Entity::find().count(&install.db).await.unwrap(), 2);
}
