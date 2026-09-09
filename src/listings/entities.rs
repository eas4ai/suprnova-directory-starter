//! Relational records for listing content and publication. Provider evidence is separate.

pub mod listing {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "listings")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub owner_id: i64,
        pub slug: String,
        pub version: i64,
        pub current_revision_id: Option<i64>,
        pub approved_revision_id: Option<i64>,
        pub archived: bool,
        pub suspended: bool,
        pub created_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod revision {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "listing_revisions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub listing_id: i64,
        pub title: String,
        pub summary: String,
        pub search_text: String,
        pub description: String,
        pub url: String,
        pub media_id: Option<String>,
        pub media_alt: String,
        pub status: String,
        pub reason: Option<String>,
        pub decided_by: Option<i64>,
        pub decided_at: Option<i64>,
        pub created_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod category {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, serde::Serialize)]
    #[sea_orm(table_name = "listing_categories")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub slug: String,
        pub name: String,
        pub active: bool,
        pub version: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod revision_category {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "listing_revision_categories")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub revision_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub category_id: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod media {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "listing_media")]
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

pub mod entitlement {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publication_entitlements")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub listing_id: i64,
        pub mode: String,
        pub status: String,
        pub valid_from: i64,
        pub valid_until: Option<i64>,
        pub updated_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod audit {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, serde::Serialize)]
    #[sea_orm(table_name = "administrative_audit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub actor_id: i64,
        pub actor_type: String,
        pub target_type: String,
        pub target_id: String,
        pub action: String,
        pub summary: String,
        pub created_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
