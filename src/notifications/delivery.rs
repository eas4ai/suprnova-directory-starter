use super::entity::{self, Column};
use crate::listings::{database_error, missing};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, ExprTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use suprnova::{DB, FrameworkError, Mail, mail::Mailable};
pub const MAX_ATTEMPTS: i32 = 8;
#[derive(Clone, Serialize, Deserialize)]
struct OwnerMail {
    subject: String,
    body: String,
    url: String,
}
#[async_trait::async_trait]
impl Mailable for OwnerMail {
    fn mailable_name() -> &'static str {
        "DirectoryOwnerNotification"
    }
    fn subject(&self) -> String {
        self.subject.clone()
    }
    fn text_template_source(&self) -> Option<String> {
        Some("{{ body }}\n\n{{ url }}".into())
    }
}
#[derive(Serialize)]
pub struct DeliveryStatus {
    pub id: String,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: i64,
    pub last_error: Option<String>,
}
impl From<entity::Model> for DeliveryStatus {
    fn from(n: entity::Model) -> Self {
        Self {
            id: n.id,
            status: n.status,
            attempts: n.attempts,
            next_attempt_at: n.next_attempt_at,
            last_error: n.last_error,
        }
    }
}
fn available(now: i64) -> Condition {
    Condition::any()
        .add(Column::LeaseUntil.is_null())
        .add(Column::LeaseUntil.lte(now))
}
fn due(now: i64) -> Condition {
    Condition::all()
        .add(Column::Status.ne("sent"))
        .add(Column::Attempts.lt(MAX_ATTEMPTS))
        .add(Column::NextAttemptAt.lte(now))
        .add(available(now))
}
fn bounded(limit: u64) -> Result<(), FrameworkError> {
    if !(1..=100).contains(&limit) {
        return Err(FrameworkError::bad_request(
            "Notification limit must be between 1 and 100.",
        ));
    }
    Ok(())
}
pub async fn status(limit: u64) -> Result<Vec<DeliveryStatus>, FrameworkError> {
    bounded(limit)?;
    let db = DB::connection()?;
    Ok(entity::Entity::find()
        .filter(Column::Status.ne("sent"))
        .order_by_asc(Column::NextAttemptAt)
        .order_by_asc(Column::Id)
        .limit(limit)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(Into::into)
        .collect())
}
pub async fn retry(id: &str) -> Result<DeliveryStatus, FrameworkError> {
    let db = DB::connection()?;
    let now = chrono::Utc::now().timestamp();
    let changed = entity::Entity::update_many()
        .col_expr(Column::Attempts, Expr::value(0))
        .col_expr(Column::NextAttemptAt, Expr::value(now))
        .col_expr(Column::Status, Expr::value("pending"))
        .filter(Column::Id.eq(id))
        .filter(Column::Status.ne("sent"))
        .filter(available(now))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    if changed.rows_affected == 0 {
        let row = entity::Entity::find_by_id(id)
            .one(db.inner())
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        if row.status == "sent" {
            return Ok(row.into());
        }
        return Err(FrameworkError::bad_request(
            "This notification is being delivered. Wait for its lease to expire.",
        ));
    }
    process(id).await
}
pub async fn pending(limit: u64) -> Result<Vec<DeliveryStatus>, FrameworkError> {
    bounded(limit)?;
    let db = DB::connection()?;
    let rows = entity::Entity::find()
        .filter(due(chrono::Utc::now().timestamp()))
        .order_by_asc(Column::NextAttemptAt)
        .order_by_asc(Column::Id)
        .limit(limit)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut output = Vec::new();
    for row in rows {
        if Instant::now() + Duration::from_secs(20) >= deadline {
            break;
        }
        output.push(process(&row.id).await?);
    }
    Ok(output)
}
async fn send(row: &entity::Model) -> Result<(), FrameworkError> {
    let db = DB::connection()?;
    let owner = crate::models::user::Entity::find_by_id(row.owner_id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    let site = crate::config::site::read()?;
    Mail::to(owner.email)
        .send(OwnerMail {
            subject: format!("{} — {}", site.name, row.title),
            body: row.body.clone(),
            url: format!("{}{}", site.origin, row.path),
        })
        .await
}
pub async fn process(id: &str) -> Result<DeliveryStatus, FrameworkError> {
    let db = DB::connection()?;
    let now = chrono::Utc::now().timestamp();
    let token = uuid::Uuid::new_v4().to_string();
    let changed = entity::Entity::update_many()
        .col_expr(Column::Status, Expr::value("delivering"))
        .col_expr(Column::Attempts, Expr::col(Column::Attempts).add(1))
        .col_expr(Column::LeaseToken, Expr::value(&token))
        .col_expr(Column::LeaseUntil, Expr::value(now + 60))
        .filter(Column::Id.eq(id))
        .filter(due(now))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    let row = entity::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if changed.rows_affected == 0 || row.lease_token.as_deref() != Some(token.as_str()) {
        return Ok(row.into());
    }
    let outcome = tokio::time::timeout(Duration::from_secs(15), send(&row)).await;
    let error = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(_)) => Some("mail_delivery_failed"),
        Err(_) => Some("mail_delivery_timeout"),
    };
    let done = chrono::Utc::now().timestamp();
    // A replaced lease cannot acknowledge another worker's delivery. Losing an
    // acknowledgment leaves a retry; SMTP may have accepted the earlier message.
    entity::Entity::update_many()
        .col_expr(
            Column::Status,
            Expr::value(if error.is_none() { "sent" } else { "failed" }),
        )
        .col_expr(
            Column::DeliveredAt,
            Expr::value(error.is_none().then_some(done)),
        )
        .col_expr(Column::LastError, Expr::value(error))
        .col_expr(
            Column::NextAttemptAt,
            Expr::value(done + 30 * 2_i64.pow(std::cmp::min(row.attempts, 8) as u32)),
        )
        .col_expr(Column::LeaseToken, Expr::value(None::<String>))
        .col_expr(Column::LeaseUntil, Expr::value(None::<i64>))
        .filter(Column::Id.eq(id))
        .filter(Column::LeaseToken.eq(token))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    Ok(entity::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?
        .into())
}
