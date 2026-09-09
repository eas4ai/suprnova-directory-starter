use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000005_create_billing_settings"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("billing_settings"))
                    .col(
                        ColumnDef::new(Alias::new("mode"))
                            .string_len(4)
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("revision"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("payload")).text().not_null())
                    .to_owned(),
            )
            .await?;
        let empty = r#"{"version":1,"stripe":{"enabled":false,"public_key":"","secrets":null},"paddle":{"enabled":false,"public_key":"","secrets":null},"default_provider":null,"mappings":{}}"#;
        let insert = Query::insert()
            .into_table(Alias::new("billing_settings"))
            .columns([
                Alias::new("mode"),
                Alias::new("revision"),
                Alias::new("payload"),
            ])
            .values_panic(["test".into(), 0_i64.into(), empty.into()])
            .values_panic(["live".into(), 0_i64.into(), empty.into()])
            .to_owned();
        manager.get_connection().execute(&insert).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("billing_settings"))
                    .to_owned(),
            )
            .await
    }
}
