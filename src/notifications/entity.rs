use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "owner_notifications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub event_key: String,
    pub owner_id: i64,
    pub listing_id: i64,
    pub purchase_id: Option<String>,
    pub title: String,
    pub body: String,
    pub path: String,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: i64,
    pub lease_token: Option<String>,
    pub lease_until: Option<i64>,
    pub delivered_at: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
