use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use suprnova::{DB, FrameworkError};

use super::{ADMIN_PERMISSION, BILLING_PERMISSION, invalid, lifecycle_entities::plan};
use crate::listings::{database_error, entities::audit, workflow::require_verified};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavePlan {
    pub key: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub billing_type: String,
    pub amount: i64,
    pub currency: String,
    pub version: i64,
}

#[derive(Serialize)]
pub struct Plan {
    pub key: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub billing_type: String,
    pub amount: i64,
    pub currency: String,
    pub version: i64,
}

impl From<plan::Model> for Plan {
    fn from(row: plan::Model) -> Self {
        Self {
            key: row.key,
            name: row.name,
            description: row.description,
            enabled: row.enabled,
            billing_type: row.billing_type,
            amount: row.amount,
            currency: row.currency,
            version: row.version,
        }
    }
}

pub(crate) async fn require_billing(actor_id: i64) -> Result<(), FrameworkError> {
    require_verified(actor_id).await?;
    for permission in [ADMIN_PERMISSION, BILLING_PERMISSION] {
        if !suprnova::rbac::has_permission_for_model(
            "directory.user",
            &actor_id.to_string(),
            permission,
        )
        .await?
        {
            return Err(suprnova::AppError::forbidden(
                "Billing administration permission is required.",
            )
            .into());
        }
    }
    Ok(())
}

impl SavePlan {
    fn validate(mut self) -> Result<Self, FrameworkError> {
        self.name = self.name.trim().to_owned();
        self.description = self.description.trim().to_owned();
        self.currency = self.currency.trim().to_ascii_uppercase();
        if self.key.is_empty()
            || self.key.len() > 64
            || !self
                .key
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
        {
            return Err(invalid(
                "key",
                "Use 1 to 64 lowercase letters, numbers, hyphens or underscores.",
            ));
        }
        if self.name.is_empty() || self.name.chars().count() > 120 {
            return Err(invalid("name", "Enter a plan name in 1 to 120 characters."));
        }
        if self.description.chars().count() > 2000 {
            return Err(invalid("description", "Use at most 2000 characters."));
        }
        if !matches!(
            self.billing_type.as_str(),
            "free" | "one_time" | "monthly" | "annual"
        ) {
            return Err(invalid(
                "billing_type",
                "Choose free, one-time, monthly or annual billing.",
            ));
        }
        if !(0..=1_000_000_000).contains(&self.amount)
            || (self.billing_type == "free") != (self.amount == 0)
        {
            return Err(invalid(
                "amount",
                "Free plans require zero; paid plans require 1 to 1000000000 minor units.",
            ));
        }
        if self.currency.len() != 3
            || suprnova::payments::Currency::from_code(&self.currency).is_none()
        {
            return Err(invalid(
                "currency",
                "Enter a supported three-letter currency code.",
            ));
        }
        if self.version < 0 || self.version == i64::MAX {
            return Err(invalid("version", "Reload the plan before saving."));
        }
        Ok(self)
    }
}

pub async fn list(actor_id: i64) -> Result<Vec<Plan>, FrameworkError> {
    require_billing(actor_id).await?;
    let db = DB::connection()?;
    Ok(plan::Entity::find()
        .order_by_asc(plan::Column::Key)
        .limit(100)
        .all(db.inner())
        .await
        .map_err(database_error)?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn save(actor_id: i64, key: Option<&str>, input: SavePlan) -> Result<(), FrameworkError> {
    require_billing(actor_id).await?;
    let input = input.validate()?;
    if key.is_some_and(|key| key != input.key) {
        return Err(invalid(
            "key",
            "A saved plan key cannot be changed. Disable it and create another plan.",
        ));
    }
    let db = DB::connection()?;
    let transaction = db.inner().begin().await.map_err(database_error)?;
    let now = chrono::Utc::now().timestamp();
    if let Some(key) = key {
        let changed = plan::Entity::update_many()
            .col_expr(plan::Column::Name, Expr::value(input.name))
            .col_expr(plan::Column::Description, Expr::value(input.description))
            .col_expr(plan::Column::Enabled, Expr::value(input.enabled))
            .col_expr(plan::Column::BillingType, Expr::value(input.billing_type))
            .col_expr(plan::Column::Amount, Expr::value(input.amount))
            .col_expr(plan::Column::Currency, Expr::value(input.currency))
            .col_expr(plan::Column::Version, Expr::value(input.version + 1))
            .col_expr(plan::Column::UpdatedAt, Expr::value(now))
            .filter(plan::Column::Key.eq(key))
            .filter(plan::Column::Version.eq(input.version))
            .exec(&transaction)
            .await
            .map_err(database_error)?;
        if changed.rows_affected != 1 {
            return Err(invalid(
                "version",
                "This plan changed or no longer exists. Reload before saving.",
            ));
        }
    } else {
        if input.version != 0 {
            return Err(invalid("version", "A new plan starts at version zero."));
        }
        // Serialize creation under an existing row; the mapping/UI contract is
        // bounded to 100 local plans, including disabled historical plans.
        super::entity::Entity::update_many()
            .col_expr(
                super::entity::Column::Revision,
                Expr::col(super::entity::Column::Revision),
            )
            .filter(super::entity::Column::Mode.eq("live"))
            .exec(&transaction)
            .await
            .map_err(database_error)?;
        if plan::Entity::find()
            .count(&transaction)
            .await
            .map_err(database_error)?
            >= 100
        {
            return Err(invalid(
                "key",
                "This installation already has 100 plans. Edit an existing plan.",
            ));
        }
        let result = plan::ActiveModel {
            key: Set(input.key.clone()),
            name: Set(input.name),
            description: Set(input.description),
            enabled: Set(input.enabled),
            billing_type: Set(input.billing_type),
            amount: Set(input.amount),
            currency: Set(input.currency),
            version: Set(1),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&transaction)
        .await;
        if let Err(error) = result {
            if matches!(
                error.sql_err(),
                Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
            ) {
                return Err(invalid("key", "A plan already uses this key."));
            }
            return Err(database_error(error));
        }
    }
    audit::ActiveModel {
        actor_id: Set(actor_id), target_type: Set("publishing_plan".to_owned()), target_id: Set(input.key),
        action: Set(if key.is_some() { "updated" } else { "created" }.to_owned()),
        summary: Set("Plan name, description, enablement, billing type, amount and currency saved; existing purchase terms preserved.".to_owned()),
        created_at: Set(now), ..Default::default()
    }.insert(&transaction).await.map_err(database_error)?;
    transaction.commit().await.map_err(database_error)
}
