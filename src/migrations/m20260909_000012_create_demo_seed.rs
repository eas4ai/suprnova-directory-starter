use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Alias::new("demo_seed_state"))
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
                .col(ColumnDef::new(Alias::new("completed")).boolean().not_null())
                .to_owned(),
        )
        .await?;
        m.get_connection()
            .execute_unprepared(
                "INSERT INTO demo_seed_state (id, version, completed) VALUES (1, 0, FALSE)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(
            Table::drop()
                .table(Alias::new("demo_seed_state"))
                .to_owned(),
        )
        .await
    }
}
