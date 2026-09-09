pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;
mod m20240101_000002_create_sessions_table;
mod m20240101_000003_create_remember_tokens_table;
mod m20240101_000004_create_auth_flow_tokens_table;
mod m20260909_000005_create_billing_settings;
mod m20260909_000006_create_directory;
mod m20260909_000007_create_publishing;
mod m20260909_000008_create_articles;

mod m20260909_000009_create_account_access;

mod m20260909_000010_preserve_moderation_reason;

mod m20260909_000011_create_owner_notifications;
mod m20260909_000012_create_demo_seed;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240101_000002_create_sessions_table::Migration),
            Box::new(m20240101_000003_create_remember_tokens_table::Migration),
            Box::new(m20240101_000004_create_auth_flow_tokens_table::Migration),
            Box::new(suprnova::rbac::migrations::CreateRbacTables),
            Box::new(m20260909_000005_create_billing_settings::Migration),
            Box::new(m20260909_000006_create_directory::Migration),
            Box::new(m20260909_000007_create_publishing::Migration),
            Box::new(m20260909_000008_create_articles::Migration),
            Box::new(m20260909_000009_create_account_access::Migration),
            Box::new(m20260909_000010_preserve_moderation_reason::Migration),
            Box::new(m20260909_000011_create_owner_notifications::Migration),
            Box::new(m20260909_000012_create_demo_seed::Migration),
        ]
    }
}
