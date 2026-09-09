use sea_orm_migration::prelude::*;
pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000011_create_owner_notifications"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let mut table = Table::create();
        table.table(Alias::new("owner_notifications"));
        for name in ["id", "event_key", "title", "body", "path", "status"] {
            let mut col = ColumnDef::new(Alias::new(name));
            col.text().not_null();
            if name == "id" {
                col.primary_key();
            }
            table.col(&mut col);
        }
        for name in [
            "owner_id",
            "listing_id",
            "attempts",
            "next_attempt_at",
            "created_at",
        ] {
            table.col(ColumnDef::new(Alias::new(name)).big_integer().not_null());
        }
        for name in ["purchase_id", "lease_token", "last_error"] {
            table.col(ColumnDef::new(Alias::new(name)).text().null());
        }
        for name in ["lease_until", "delivered_at"] {
            table.col(ColumnDef::new(Alias::new(name)).big_integer().null());
        }
        m.create_table(table.to_owned()).await?;
        for (name, columns, unique) in [
            ("notification_event_unique", vec!["event_key"], true),
            (
                "notification_delivery_due",
                vec!["status", "next_attempt_at"],
                false,
            ),
            (
                "notification_owner_history",
                vec!["owner_id", "listing_id", "created_at"],
                false,
            ),
        ] {
            let mut index = Index::create();
            index.name(name).table(Alias::new("owner_notifications"));
            for column in columns {
                index.col(Alias::new(column));
            }
            if unique {
                index.unique();
            }
            m.create_index(index.to_owned()).await?;
        }
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(
            Table::drop()
                .table(Alias::new("owner_notifications"))
                .to_owned(),
        )
        .await
    }
}
