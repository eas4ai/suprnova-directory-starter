use sea_orm_migration::prelude::*;

pub struct Migration;
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260909_000008_create_articles"
    }
}

fn text(name: &str) -> ColumnDef {
    let mut c = ColumnDef::new(Alias::new(name));
    c.text().not_null();
    c
}
fn integer(name: &str) -> ColumnDef {
    let mut c = ColumnDef::new(Alias::new(name));
    c.big_integer().not_null();
    c
}
fn nullable(mut c: ColumnDef) -> ColumnDef {
    c.null();
    c
}
fn id() -> ColumnDef {
    let mut c = ColumnDef::new(Alias::new("id"));
    c.integer().not_null().auto_increment().primary_key();
    c
}
fn foreign(table: &str, field: &str, target: &str, key: &str) -> ForeignKeyCreateStatement {
    ForeignKey::create()
        .from(Alias::new(table), Alias::new(field))
        .to(Alias::new(target), Alias::new(key))
        .on_delete(ForeignKeyAction::Restrict)
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("listing_categories"))
                .add_column(integer("version").default(0))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("article_media"))
                .col(text("id").primary_key())
                .col(&mut integer("owner_id"))
                .col(&mut text("storage_key"))
                .col(&mut integer("width"))
                .col(&mut integer("height"))
                .col(&mut integer("created_at"))
                .foreign_key(&mut foreign("article_media", "owner_id", "users", "id"))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("articles"))
                .col(&mut id())
                .col(&mut integer("author_id"))
                .col(&mut text("slug"))
                .col(&mut integer("version"))
                .col(&mut nullable(integer("current_revision_id")))
                .col(&mut nullable(integer("published_revision_id")))
                .col(&mut nullable(integer("published_at")))
                .col(&mut integer("created_at"))
                .col(&mut integer("updated_at"))
                .foreign_key(&mut foreign("articles", "author_id", "users", "id"))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("article_revisions"))
                .col(&mut id())
                .col(&mut integer("article_id"))
                .col(&mut text("slug"))
                .col(&mut text("title"))
                .col(&mut text("summary"))
                .col(&mut text("search_text"))
                .col(&mut text("body"))
                .col(&mut nullable(text("media_id")))
                .col(&mut text("media_alt"))
                .col(&mut integer("created_at"))
                .foreign_key(&mut foreign(
                    "article_revisions",
                    "article_id",
                    "articles",
                    "id",
                ))
                .foreign_key(&mut foreign(
                    "article_revisions",
                    "media_id",
                    "article_media",
                    "id",
                ))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("article_slugs"))
                .col(text("slug").primary_key())
                .col(&mut integer("article_id"))
                .foreign_key(&mut foreign(
                    "article_slugs",
                    "article_id",
                    "articles",
                    "id",
                ))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("article_terms"))
                .col(&mut id())
                .col(&mut text("kind"))
                .col(&mut text("slug"))
                .col(&mut text("name"))
                .col(ColumnDef::new(Alias::new("active")).boolean().not_null())
                .col(&mut integer("version"))
                .index(
                    Index::create()
                        .unique()
                        .col(Alias::new("kind"))
                        .col(Alias::new("slug")),
                )
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(Alias::new("article_revision_terms"))
                .col(&mut integer("revision_id"))
                .col(&mut integer("term_id"))
                .primary_key(
                    Index::create()
                        .col(Alias::new("revision_id"))
                        .col(Alias::new("term_id")),
                )
                .foreign_key(&mut foreign(
                    "article_revision_terms",
                    "revision_id",
                    "article_revisions",
                    "id",
                ))
                .foreign_key(&mut foreign(
                    "article_revision_terms",
                    "term_id",
                    "article_terms",
                    "id",
                ))
                .to_owned(),
        )
        .await?;
        // One row serializes bounded taxonomy changes, including capacity checks.
        m.create_table(
            Table::create()
                .table(Alias::new("taxonomy_write_lock"))
                .col(integer("id").primary_key())
                .col(&mut integer("version"))
                .to_owned(),
        )
        .await?;
        m.get_connection()
            .execute_unprepared("INSERT INTO taxonomy_write_lock (id, version) VALUES (1, 0)")
            .await?;
        for (name, table, columns) in [
            (
                "articles_public_order",
                "articles",
                vec!["published_at", "id"],
            ),
            (
                "article_revisions_article",
                "article_revisions",
                vec!["article_id", "id"],
            ),
            ("article_slugs_article", "article_slugs", vec!["article_id"]),
            (
                "article_terms_lookup",
                "article_revision_terms",
                vec!["term_id", "revision_id"],
            ),
        ] {
            let mut index = Index::create();
            index.name(name).table(Alias::new(table));
            for column in columns {
                index.col(Alias::new(column));
            }
            m.create_index(index.to_owned()).await?;
        }
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "taxonomy_write_lock",
            "article_revision_terms",
            "article_terms",
            "article_slugs",
            "article_revisions",
            "articles",
            "article_media",
        ] {
            m.drop_table(Table::drop().table(Alias::new(table)).to_owned())
                .await?;
        }
        m.alter_table(
            Table::alter()
                .table(Alias::new("listing_categories"))
                .drop_column(Alias::new("version"))
                .to_owned(),
        )
        .await?;
        Ok(())
    }
}
