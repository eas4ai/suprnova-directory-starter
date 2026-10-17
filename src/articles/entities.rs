//! Article drafts and public revisions are separate records.
pub mod article {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "articles")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub author_id: i64,
        pub slug: String,
        pub version: i64,
        pub current_revision_id: Option<i64>,
        pub published_revision_id: Option<i64>,
        pub published_at: Option<i64>,
        pub created_at: i64,
        pub updated_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod revision {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "article_revisions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub article_id: i64,
        pub slug: String,
        pub title: String,
        pub summary: String,
        pub search_text: String,
        pub seo: String,
        pub body: String,
        pub media_id: Option<String>,
        pub media_alt: String,
        pub created_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod slug {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "article_slugs")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub slug: String,
        pub article_id: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod term {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, serde::Serialize)]
    #[sea_orm(table_name = "article_terms")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub kind: String,
        pub slug: String,
        pub name: String,
        pub active: bool,
        pub version: i64,
        pub seo: String,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod revision_term {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "article_revision_terms")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub revision_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub term_id: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod media {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "article_media")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub owner_id: i64,
        pub storage_key: String,
        pub width: i32,
        pub height: i32,
        pub created_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
