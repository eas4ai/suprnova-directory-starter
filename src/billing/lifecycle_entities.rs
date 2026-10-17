//! Application purchase terms and authenticated fulfillment records.

pub mod plan {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_plans")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub key: String,
        pub name: String,
        pub description: String,
        pub enabled: bool,
        pub billing_type: String,
        pub amount: i64,
        pub currency: String,
        pub version: i64,
        pub created_at: i64,
        pub updated_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod purchase {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_purchases")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub listing_id: i64,
        pub owner_id: i64,
        pub approved_revision_id: i64,
        pub plan_key: String,
        pub plan_name: String,
        pub billing_type: String,
        pub amount: i64,
        pub currency: String,
        pub provider: String,
        pub mode: String,
        pub price_id: Option<String>,
        pub public_key: Option<String>,
        pub credentials: Option<String>,
        pub profile_revision: Option<i64>,
        pub return_origin: String,
        pub customer_ref: Option<String>,
        pub session_ref: Option<String>,
        pub subscription_ref: Option<String>,
        pub checkout_payload: Option<String>,
        pub state: String,
        pub error_code: Option<String>,
        pub version: i64,
        pub lease_until: Option<i64>,
        pub cancel_requested: bool,
        pub cancel_at: Option<i64>,
        pub cancel_event_at: i64,
        pub created_at: i64,
        pub updated_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod slot {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_slots")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub listing_id: i64,
        pub purchase_id: String,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod payment {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_payments")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub purchase_id: String,
        pub provider_payment_id: String,
        pub amount_total: i64,
        pub period_start: i64,
        pub period_end: Option<i64>,
        pub status: String,
        pub paid_at: i64,
        pub refund_event_at: i64,
        pub dispute_event_at: i64,
        pub adverse: String,
        pub created_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod reference {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_references")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub provider: String,
        pub mode: String,
        pub kind: String,
        pub reference: String,
        pub purchase_id: String,
        pub payment_id: Option<String>,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod event {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_events")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub provider: String,
        pub mode: String,
        pub provider_event_id: String,
        pub event_type: String,
        pub raw_body: Vec<u8>,
        pub signature: String,
        pub occurred_at: i64,
        pub received_at: i64,
        pub purchase_id: Option<String>,
        pub credential_source: String,
        pub status: String,
        pub attempts: i32,
        pub next_attempt_at: i64,
        pub error_code: Option<String>,
        pub lease_token: Option<String>,
        pub lease_until: Option<i64>,
        pub enrichment: Option<String>,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod receipt {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "publishing_receipts")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub event_record_id: String,
        pub purchase_id: Option<String>,
        pub outcome: String,
        pub applied_at: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
