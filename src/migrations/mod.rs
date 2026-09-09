pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;
mod m20240101_000002_create_sessions_table;
mod m20240101_000003_create_remember_tokens_table;
mod m20240101_000004_create_auth_flow_tokens_table;
mod m20260909_000005_create_billing_settings;
mod m20260909_000006_create_directory;

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
        ]
    }
}
