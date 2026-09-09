use sea_orm_migration::prelude::*;
pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000009_create_account_access"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Alias::new("account_access"))
                .col(
                    ColumnDef::new(Alias::new("user_id"))
                        .big_integer()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("version"))
                        .big_integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(Alias::new("suspended"))
                        .boolean()
                        .not_null()
                        .default(false),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(Alias::new("account_access"), Alias::new("user_id"))
                        .to(Alias::new("users"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Restrict),
                )
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("account_write_lock"))
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("version"))
                        .big_integer()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;
        m.get_connection()
            .execute_unprepared("INSERT INTO account_write_lock (id, version) VALUES (1, 0)")
            .await?;
        m.alter_table(
            Table::alter()
                .table(Alias::new("administrative_audit"))
                .add_column(
                    ColumnDef::new(Alias::new("actor_type"))
                        .string()
                        .not_null()
                        .default("user"),
                )
                .to_owned(),
        )
        .await?;
        crate::accounts::roles::seed(m.get_connection()).await?;
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("administrative_audit"))
                .drop_column(Alias::new("actor_type"))
                .to_owned(),
        )
        .await?;
        for table in ["account_write_lock", "account_access"] {
            m.drop_table(Table::drop().table(Alias::new(table)).to_owned())
                .await?;
        }
        // RBAC definitions may be in operator use; never discard grants on rollback.
        Ok(())
    }
}
