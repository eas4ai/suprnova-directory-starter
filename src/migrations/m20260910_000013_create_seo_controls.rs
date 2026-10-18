use sea_orm_migration::prelude::*;
pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260910_000013_create_seo_controls"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "listing_revisions",
            "article_revisions",
            "listing_categories",
            "article_terms",
        ] {
            m.alter_table(
                Table::alter()
                    .table(Alias::new(table))
                    .add_column(
                        ColumnDef::new(Alias::new("seo"))
                            .text()
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await?;
        }
        m.create_table(
            Table::create()
                .table(Alias::new("seo_settings"))
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .big_integer()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("version"))
                        .big_integer()
                        .not_null(),
                )
                .col(ColumnDef::new(Alias::new("value")).text().not_null())
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .big_integer()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;
        m.get_connection()
            .execute_unprepared(
                "INSERT INTO seo_settings (id, version, value, updated_at) VALUES (1, 0, '{}', 0)",
            )
            .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("seo_redirects"))
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("source"))
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("destination"))
                        .string()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("version"))
                        .big_integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .big_integer()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("seo_not_found"))
                .col(
                    ColumnDef::new(Alias::new("path"))
                        .string()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(Alias::new("hits")).big_integer().not_null())
                .col(
                    ColumnDef::new(Alias::new("first_seen"))
                        .big_integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("last_seen"))
                        .big_integer()
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;
        m.create_index(
            Index::create()
                .name("seo_not_found_seen")
                .table(Alias::new("seo_not_found"))
                .col(Alias::new("last_seen"))
                .to_owned(),
        )
        .await?;
        crate::accounts::roles::seed(m.get_connection()).await?;
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for table in ["seo_not_found", "seo_redirects", "seo_settings"] {
            m.drop_table(Table::drop().table(Alias::new(table)).to_owned())
                .await?;
        }
        for table in [
            "listing_revisions",
            "article_revisions",
            "listing_categories",
            "article_terms",
        ] {
            m.alter_table(
                Table::alter()
                    .table(Alias::new(table))
                    .drop_column(Alias::new("seo"))
                    .to_owned(),
            )
            .await?;
        }
        // Preserve operator RBAC grants on rollback, as other starter migrations do.
        Ok(())
    }
}
