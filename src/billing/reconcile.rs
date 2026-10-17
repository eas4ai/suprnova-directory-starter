//! Bounded replay of authenticated events and read-only purchase recovery.
use std::time::Duration;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    Set, sea_query::Expr,
};
use serde::Serialize;
use suprnova::{
    DB, FrameworkError,
    serde_json::{self, json},
};

use super::{
    Mode,
    collect::{self, CollectError},
    events, fulfillment,
    gateway::{self, Gateway},
    lifecycle_entities::{event, purchase},
    settings,
};
use crate::listings::{database_error, missing};

pub const MAX_ATTEMPTS: i32 = 8;
pub const ITEM_TIMEOUT: Duration = Duration::from_secs(90);
pub const BATCH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Serialize)]
pub struct ResultRow {
    pub event_id: String,
    pub status: String,
    pub error_code: Option<String>,
}

/// Replay does not reverify the old timestamped signature. Only accept() can
/// create provider records; its authenticated exact bytes remain durable.
pub async fn process_event(
    id: &str,
    gateway: &dyn Gateway,
    retry_failed: bool,
    use_current_credentials: bool,
) -> Result<ResultRow, FrameworkError> {
    let db = DB::connection()?;
    let now = chrono::Utc::now().timestamp();
    let Some(row) = event::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?
    else {
        return Err(missing());
    };
    let skipped = || ResultRow {
        event_id: row.id.clone(),
        status: row.status.clone(),
        error_code: row.error_code.clone(),
    };
    if row.status == "applied"
        || row.lease_until.is_some_and(|until| until > now)
        || (!retry_failed && (row.attempts >= MAX_ATTEMPTS || row.next_attempt_at > now))
    {
        return Ok(skipped());
    }
    let token = uuid::Uuid::new_v4().to_string();
    let attempt = if retry_failed && row.attempts >= MAX_ATTEMPTS {
        1
    } else {
        row.attempts + 1
    };
    let changed = event::Entity::update_many()
        .col_expr(event::Column::Status, Expr::value("processing"))
        .col_expr(event::Column::LeaseToken, Expr::value(&token))
        .col_expr(event::Column::LeaseUntil, Expr::value(now + 120))
        .col_expr(event::Column::Attempts, Expr::value(attempt))
        .filter(event::Column::Id.eq(id))
        .filter(event::Column::Attempts.eq(row.attempts))
        .filter(event::Column::Status.ne("applied"))
        .filter(
            Condition::any()
                .add(event::Column::LeaseUntil.is_null())
                .add(event::Column::LeaseUntil.lte(now)),
        )
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    if changed.rows_affected == 0 {
        return Ok(skipped());
    }
    let result = tokio::time::timeout(
        ITEM_TIMEOUT,
        prepare_and_apply(&row, &token, gateway, use_current_credentials),
    )
    .await;
    let error = match result {
        Ok(Ok(true)) => {
            return Ok(ResultRow {
                event_id: id.into(),
                status: "applied".into(),
                error_code: None,
            });
        }
        Ok(Ok(false)) => {
            return Ok(ResultRow {
                event_id: id.into(),
                status: "lease_replaced".into(),
                error_code: None,
            });
        }
        Ok(Err(CollectError::Evidence(code))) => code,
        Ok(Err(CollectError::Database(error))) => {
            tracing::warn!(event_id = id, error = %error, "Payment fulfillment transaction failed");
            "fulfillment_transaction_failed"
        }
        Err(_) => "reconciliation_timeout",
    };
    let status = if attempt >= MAX_ATTEMPTS {
        "failed"
    } else {
        "deferred"
    };
    let delay = std::cmp::min(30_i64 * (1_i64 << std::cmp::min(attempt, 7)), 3600);
    event::Entity::update_many()
        .col_expr(event::Column::Status, Expr::value(status))
        .col_expr(event::Column::ErrorCode, Expr::value(error))
        .col_expr(event::Column::LeaseToken, Expr::value(None::<String>))
        .col_expr(event::Column::LeaseUntil, Expr::value(None::<i64>))
        .col_expr(
            event::Column::NextAttemptAt,
            Expr::value(chrono::Utc::now().timestamp() + delay),
        )
        .filter(event::Column::Id.eq(id))
        .filter(event::Column::LeaseToken.eq(&token))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    Ok(ResultRow {
        event_id: id.into(),
        status: status.into(),
        error_code: Some(error.into()),
    })
}

async fn prepare_and_apply(
    row: &event::Model,
    token: &str,
    gateway: &dyn Gateway,
    use_current: bool,
) -> Result<bool, CollectError> {
    let marker = chrono::Utc::now().timestamp_micros();
    if !collect::supported(&row.provider, &row.event_type) {
        return Ok(
            fulfillment::apply(&row.id, token, None, &[fulfillment::Fact::Ignored], marker).await?,
        );
    }
    let body = serde_json::from_slice(&row.raw_body)
        .map_err(|_| CollectError::Evidence("retained_event_invalid"))?;
    let provider = gateway::provider(&row.provider)?;
    let mode = Mode::parse(&row.mode).map_err(|_| CollectError::Evidence("invalid_mode"))?;
    let p = events::find_purchase(provider, mode, &body)
        .await?
        .ok_or(CollectError::Evidence("purchase_not_correlated"))?;
    if row
        .purchase_id
        .as_deref()
        .is_some_and(|known| known != p.id)
    {
        return Err(CollectError::Evidence("purchase_correlation_changed"));
    }
    let read_purchase = if use_current || row.credential_source == "operator_current" {
        current_credentials(&p).await?
    } else {
        p.clone()
    };
    let facts = collect::collect(
        &read_purchase,
        &row.event_type,
        events::data(&row.provider, &body),
        gateway,
        marker / 1_000_000,
    )
    .await?;
    Ok(fulfillment::apply(&row.id, token, Some(&p.id), &facts, marker).await?)
}

/// Explicit operator recovery after API-key revocation. The original encrypted
/// snapshot is never overwritten, and this copy is passed to GET operations only.
async fn current_credentials(p: &purchase::Model) -> Result<purchase::Model, CollectError> {
    let mode = Mode::parse(&p.mode).map_err(|_| CollectError::Evidence("invalid_mode"))?;
    let provider = gateway::provider(&p.provider)?;
    let (_, settings) = settings::read(mode)
        .await
        .map_err(|_| CollectError::Evidence("current_credentials_unreadable"))?;
    let profile = settings.profile(provider);
    let credentials = profile
        .credentials(mode, provider)
        .map_err(|_| CollectError::Evidence("current_credentials_unreadable"))?
        .ok_or(CollectError::Evidence("missing_current_credentials"))?;
    let mut copy = p.clone();
    copy.credentials = Some(
        super::secrets::encrypt(mode, provider, &credentials)
            .map_err(|_| CollectError::Evidence("current_credentials_unreadable"))?,
    );
    copy.public_key = Some(profile.public_key.clone());
    Ok(copy)
}

pub async fn pending(limit: u64, gateway: &dyn Gateway) -> Result<Vec<ResultRow>, FrameworkError> {
    if !(1..=100).contains(&limit) {
        return Err(FrameworkError::bad_request(
            "Choose a batch size from 1 to 100.",
        ));
    }
    let db = DB::connection()?;
    let now = chrono::Utc::now().timestamp();
    let rows = event::Entity::find()
        .filter(event::Column::Status.is_in(["pending", "deferred", "processing"]))
        .filter(event::Column::Attempts.lt(MAX_ATTEMPTS))
        .filter(event::Column::NextAttemptAt.lte(now))
        .filter(
            Condition::any()
                .add(event::Column::LeaseUntil.is_null())
                .add(event::Column::LeaseUntil.lte(now)),
        )
        .order_by_asc(event::Column::NextAttemptAt)
        .order_by_asc(event::Column::Id)
        .limit(limit)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let started = tokio::time::Instant::now();
    let mut result = Vec::new();
    for row in rows {
        let Some(remaining) = BATCH_TIMEOUT.checked_sub(started.elapsed()) else {
            break;
        };
        match tokio::time::timeout(remaining, process_event(&row.id, gateway, false, false)).await {
            Ok(value) => result.push(value?),
            Err(_) => break, // Interrupted leases remain eligible after their deadline.
        }
    }
    Ok(result)
}

/// Retain a request to inspect an existing resource. Neither this function nor
/// its processor calls create_customer/start_session/cancel.
pub async fn recover(
    purchase_id: &str,
    resource: Option<&str>,
    gateway: &dyn Gateway,
    use_current_credentials: bool,
) -> Result<ResultRow, FrameworkError> {
    let id = enqueue_recovery(purchase_id, resource, use_current_credentials).await?;
    process_event(&id, gateway, false, use_current_credentials).await
}

pub(crate) async fn enqueue_recovery(
    purchase_id: &str,
    resource: Option<&str>,
    use_current_credentials: bool,
) -> Result<String, FrameworkError> {
    let db = DB::connection()?;
    let p = purchase::Entity::find_by_id(purchase_id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if p.provider == "free" {
        return Err(FrameworkError::bad_request(
            "Free publication has no provider payment to reconcile.",
        ));
    }
    let reference = resource.or(p.subscription_ref.as_deref()).or(p.session_ref.as_deref())
        .ok_or_else(|| FrameworkError::bad_request("The create response was lost. Supply the existing provider session or transaction ID with --resource; obtain it from the provider dashboard using this purchase's correlation ID."))?;
    let kind = if reference.starts_with("sub_") {
        "reconcile.subscription"
    } else if p.provider == "stripe" && reference.starts_with("in_") {
        "reconcile.invoice"
    } else if (p.provider == "stripe" && reference.starts_with("cs_"))
        || (p.provider == "paddle" && reference.starts_with("txn_"))
    {
        "reconcile.checkout"
    } else {
        return Err(FrameworkError::bad_request(
            "Use an existing session, transaction, invoice or subscription ID for this purchase's provider.",
        ));
    };
    if reference.len() > 255
        || !reference
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(FrameworkError::bad_request("Invalid provider resource ID."));
    }
    let now = chrono::Utc::now().timestamp();
    let id = uuid::Uuid::new_v4().to_string();
    let data = json!({"id": reference, "metadata": {"purchase_id": p.id}, "custom_data": {"purchase_id": p.id}});
    let body = if p.provider == "stripe" {
        json!({"data": {"object": data}})
    } else {
        json!({"data": data})
    };
    event::ActiveModel {
        id: Set(id.clone()),
        provider: Set(p.provider),
        mode: Set(p.mode),
        provider_event_id: Set(format!("operator_{id}")),
        event_type: Set(kind.into()),
        raw_body: Set(serde_json::to_vec(&body)
            .map_err(|_| FrameworkError::internal("Could not retain recovery request."))?),
        signature: Set(String::new()),
        occurred_at: Set(now),
        received_at: Set(now),
        purchase_id: Set(Some(p.id)),
        credential_source: Set(if use_current_credentials {
            "operator_current"
        } else {
            "operator_snapshot"
        }
        .into()),
        status: Set("pending".into()),
        attempts: Set(0),
        next_attempt_at: Set(now),
        error_code: Set(None),
        lease_token: Set(None),
        lease_until: Set(None),
        enrichment: Set(None),
    }
    .insert(db.inner())
    .await
    .map_err(database_error)?;
    Ok(id)
}

#[derive(Serialize)]
pub struct EventStatus {
    pub id: String,
    pub purchase_id: Option<String>,
    pub provider: String,
    pub mode: String,
    pub status: String,
    pub attempts: i32,
    pub error_code: Option<String>,
}

pub async fn status(limit: u64) -> Result<Vec<EventStatus>, FrameworkError> {
    if !(1..=100).contains(&limit) {
        return Err(FrameworkError::bad_request(
            "Choose a batch size from 1 to 100.",
        ));
    }
    let db = DB::connection()?;
    Ok(event::Entity::find()
        .filter(event::Column::Status.ne("applied"))
        .order_by_asc(event::Column::NextAttemptAt)
        .order_by_asc(event::Column::Id)
        .limit(limit)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(|row| EventStatus {
            id: row.id,
            purchase_id: row.purchase_id,
            provider: row.provider,
            mode: row.mode,
            status: row.status,
            attempts: row.attempts,
            error_code: row.error_code,
        })
        .collect())
}
