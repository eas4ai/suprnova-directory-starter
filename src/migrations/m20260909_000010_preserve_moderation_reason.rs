use sea_orm_migration::prelude::*;
pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000010_preserve_moderation_reason"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("administrative_audit"))
                .add_column(ColumnDef::new(Alias::new("private_reason")).text().null())
                .to_owned(),
        )
        .await?;
        // Preserve old audit text; broad audit pages already exclude listing free text.
        m.get_connection().execute_unprepared("UPDATE administrative_audit SET private_reason = summary WHERE target_type = 'listing'").await?;
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Keep reasons recorded after this migration before dropping the detail column.
        m.get_connection().execute_unprepared("UPDATE administrative_audit SET summary = private_reason WHERE target_type = 'listing' AND private_reason IS NOT NULL").await?;
        m.alter_table(
            Table::alter()
                .table(Alias::new("administrative_audit"))
                .drop_column(Alias::new("private_reason"))
                .to_owned(),
        )
        .await
    }
}
