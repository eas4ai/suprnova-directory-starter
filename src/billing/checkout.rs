use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
    sea_query::{Expr, OnConflict},
};
use serde::{Deserialize, Serialize};
use suprnova::{
    DB, FrameworkError,
    payments::{CreateCustomerRequest, SessionMode, SessionPayload, StartSessionRequest},
    serde_json::{self, json},
};

use super::{
    Mode, Provider,
    gateway::{Gateway, GatewayError},
    invalid,
    lifecycle_entities::{payment, plan, purchase, reference, slot},
    settings,
};
use crate::{
    listings::{
        database_error,
        entities::{entitlement, listing, revision},
        missing,
        workflow::require_verified,
    },
    models::user,
};

pub fn checkout_mode() -> Result<Mode, FrameworkError> {
    Mode::parse(&std::env::var("BILLING_CHECKOUT_MODE").unwrap_or_default()).map_err(|_| {
        invalid(
            "configuration",
            "Set BILLING_CHECKOUT_MODE to test or live on the application host.",
        )
    })
}

fn origin() -> Result<String, FrameworkError> {
    let value = std::env::var("APP_URL").unwrap_or_default();
    let parsed = url::Url::parse(&value)
        .map_err(|_| invalid("configuration", "Configure an absolute HTTP(S) APP_URL."))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.path() != "/"
    {
        return Err(invalid(
            "configuration",
            "APP_URL must be an HTTP(S) origin without credentials, path, query or fragment.",
        ));
    }
    Ok(parsed.origin().ascii_serialization())
}

#[derive(Serialize)]
pub struct ListingSummary {
    pub id: i64,
    pub title: String,
    pub slug: String,
}
#[derive(Serialize)]
pub struct ProviderOption {
    pub id: Provider,
    pub label: &'static str,
    pub default: bool,
}
#[derive(Serialize)]
pub struct Offer {
    pub key: String,
    pub name: String,
    pub description: String,
    pub billing_type: String,
    pub amount: i64,
    pub currency: String,
    pub providers: Vec<ProviderOption>,
}
#[derive(Serialize)]
pub struct Offers {
    pub listing: ListingSummary,
    pub plans: Vec<Offer>,
    pub mode: Mode,
    pub purchase_id: Option<String>,
}

pub(crate) async fn owned(actor: i64, id: &str) -> Result<purchase::Model, FrameworkError> {
    let db = DB::connection()?;
    purchase::Entity::find_by_id(id)
        .filter(purchase::Column::OwnerId.eq(actor))
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)
}

async fn approved_listing(
    actor: i64,
    id: i64,
) -> Result<(listing::Model, revision::Model), FrameworkError> {
    require_verified(actor).await?;
    let db = DB::connection()?;
    let listing = listing::Entity::find_by_id(id)
        .filter(listing::Column::OwnerId.eq(actor))
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if listing.archived || listing.suspended {
        return Err(invalid(
            "listing",
            "This listing cannot start a purchase while archived or suspended.",
        ));
    }
    let revision = revision::Entity::find_by_id(listing.approved_revision_id.ok_or_else(|| {
        invalid(
            "listing",
            "Submit your listing and wait for approval before choosing a plan.",
        )
    })?)
    .filter(revision::Column::ListingId.eq(id))
    .filter(revision::Column::Status.eq("approved"))
    .one(db.inner())
    .await
    .map_err(database_error)?
    .ok_or_else(missing)?;
    Ok((listing, revision))
}

pub async fn offers(actor: i64, id: i64) -> Result<Offers, FrameworkError> {
    let (listing, approved) = approved_listing(actor, id).await?;
    let mode = checkout_mode()?;
    let db = DB::connection()?;
    let existing = slot::Entity::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(database_error)?;
    let (_, settings) = settings::read(mode)
        .await
        .map_err(super::BillingError::into_framework)?;
    let plans = plan::Entity::find()
        .filter(plan::Column::Enabled.eq(true))
        .order_by_asc(plan::Column::Amount)
        .order_by_asc(plan::Column::Key)
        .limit(100)
        .all(db.inner())
        .await
        .map_err(database_error)?;
    let mut choices = Vec::new();
    if existing.is_none() {
        for plan in plans {
            let mut providers = Vec::new();
            if plan.billing_type != "free" {
                for (provider, label) in
                    [(Provider::Stripe, "Stripe"), (Provider::Paddle, "Paddle")]
                {
                    let profile = settings.profile(provider);
                    let mapped = settings
                        .mappings
                        .get(&plan.key)
                        .and_then(|mapping| match provider {
                            Provider::Stripe => mapping.stripe.as_ref(),
                            Provider::Paddle => mapping.paddle.as_ref(),
                        })
                        .is_some();
                    if profile.enabled
                        && mapped
                        && profile
                            .credentials(mode, provider)
                            .map_err(super::BillingError::into_framework)?
                            .is_some()
                    {
                        providers.push(ProviderOption {
                            id: provider,
                            label,
                            default: settings.default_provider == Some(provider),
                        });
                    }
                }
                if providers.is_empty() {
                    continue;
                }
            }
            choices.push(Offer {
                key: plan.key,
                name: plan.name,
                description: plan.description,
                billing_type: plan.billing_type,
                amount: plan.amount,
                currency: plan.currency,
                providers,
            });
        }
    }
    Ok(Offers {
        listing: ListingSummary {
            id,
            title: approved.title,
            slug: listing.slug,
        },
        plans: choices,
        mode,
        purchase_id: existing.map(|slot| slot.purchase_id),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartPurchase {
    pub plan_key: String,
    pub provider: Option<Provider>,
}

/// Reserve a purchase under a listing write lock, then perform I/O after commit.
pub async fn start(
    actor: i64,
    id: i64,
    input: StartPurchase,
    gateway: &dyn Gateway,
) -> Result<String, FrameworkError> {
    require_verified(actor).await?;
    let mode = checkout_mode()?;
    let return_origin = origin()?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    let locked = listing::Entity::update_many()
        .col_expr(
            listing::Column::Version,
            Expr::col(listing::Column::Version),
        )
        .filter(listing::Column::Id.eq(id))
        .filter(listing::Column::OwnerId.eq(actor))
        .filter(listing::Column::Archived.eq(false))
        .filter(listing::Column::Suspended.eq(false))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    if locked.rows_affected != 1 {
        return Err(missing());
    }
    let listing = listing::Entity::find_by_id(id)
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    let approved_id = listing
        .approved_revision_id
        .ok_or_else(|| invalid("listing", "Approval is required before checkout."))?;
    if revision::Entity::find_by_id(approved_id)
        .filter(revision::Column::ListingId.eq(id))
        .filter(revision::Column::Status.eq("approved"))
        .one(&tx)
        .await
        .map_err(database_error)?
        .is_none()
    {
        return Err(invalid("listing", "Approval is required before checkout."));
    }
    if let Some(slot) = slot::Entity::find_by_id(id)
        .one(&tx)
        .await
        .map_err(database_error)?
    {
        let existing = purchase::Entity::find_by_id(&slot.purchase_id)
            .one(&tx)
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        if existing.plan_key != input.plan_key
            || (existing.provider != "free"
                && input.provider.map(Provider::as_str) != Some(existing.provider.as_str()))
        {
            return Err(invalid(
                "plan_key",
                "Continue the existing purchase before choosing a different plan.",
            ));
        }
        tx.commit().await.map_err(database_error)?;
        continue_purchase(actor, &existing.id, gateway).await?;
        return Ok(existing.id);
    }
    let plan = plan::Entity::find_by_id(&input.plan_key)
        .filter(plan::Column::Enabled.eq(true))
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(|| invalid("plan_key", "Choose an available publishing plan."))?;
    // Lock the plan so an administrator cannot disable or edit it between read and snapshot.
    plan::Entity::update_many()
        .col_expr(plan::Column::Version, Expr::col(plan::Column::Version))
        .filter(plan::Column::Key.eq(&plan.key))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    let plan = plan::Entity::find_by_id(&input.plan_key)
        .filter(plan::Column::Enabled.eq(true))
        .one(&tx)
        .await
        .map_err(database_error)?
        .ok_or_else(|| {
            invalid(
                "plan_key",
                "This plan was disabled. Reload available plans.",
            )
        })?;
    let now = chrono::Utc::now().timestamp();
    let id_string = uuid::Uuid::new_v4().to_string();
    let mut row = purchase::ActiveModel {
        id: Set(id_string.clone()),
        listing_id: Set(id),
        owner_id: Set(actor),
        approved_revision_id: Set(approved_id),
        plan_key: Set(plan.key.clone()),
        plan_name: Set(plan.name),
        billing_type: Set(plan.billing_type.clone()),
        amount: Set(plan.amount),
        currency: Set(plan.currency),
        provider: Set("free".to_owned()),
        mode: Set("free".to_owned()),
        price_id: Set(None),
        public_key: Set(None),
        credentials: Set(None),
        profile_revision: Set(None),
        return_origin: Set(return_origin),
        customer_ref: Set(None),
        session_ref: Set(None),
        subscription_ref: Set(None),
        checkout_payload: Set(None),
        state: Set("active".to_owned()),
        error_code: Set(None),
        version: Set(1),
        lease_until: Set(None),
        cancel_requested: Set(false),
        cancel_at: Set(None),
        cancel_event_at: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if plan.billing_type != "free" {
        let provider = input
            .provider
            .ok_or_else(|| invalid("provider", "Choose a configured payment provider."))?;
        // Read committed configuration, then lock and compare that exact revision.
        let (version, settings) = settings::read_from(&tx, mode)
            .await
            .map_err(super::BillingError::into_framework)?;
        let locked = super::entity::Entity::update_many()
            .col_expr(
                super::entity::Column::Revision,
                Expr::col(super::entity::Column::Revision),
            )
            .filter(super::entity::Column::Mode.eq(mode.as_str()))
            .filter(super::entity::Column::Revision.eq(version))
            .exec(&tx)
            .await
            .map_err(database_error)?;
        if locked.rows_affected != 1 {
            return Err(invalid(
                "provider",
                "Provider settings changed. Reload the available plans.",
            ));
        }
        let profile = settings.profile(provider);
        if !profile.enabled {
            return Err(invalid("provider", "This provider is disabled."));
        }
        let price = settings
            .mappings
            .get(&plan.key)
            .and_then(|mapping| match provider {
                Provider::Stripe => mapping.stripe.as_ref(),
                Provider::Paddle => mapping.paddle.as_ref(),
            })
            .ok_or_else(|| {
                invalid(
                    "provider",
                    "This plan has no price for the selected provider and mode.",
                )
            })?;
        let credentials = profile
            .credentials(mode, provider)
            .map_err(super::BillingError::into_framework)?
            .ok_or_else(|| invalid("provider", "This provider has no usable credentials."))?;
        super::adapters::validate_construction(mode, provider, &profile.public_key, &credentials)
            .map_err(super::BillingError::into_framework)?;
        row.provider = Set(provider.as_str().to_owned());
        row.mode = Set(mode.as_str().to_owned());
        row.price_id = Set(Some(price.clone()));
        row.public_key = Set(Some(profile.public_key.clone()));
        row.credentials = Set(profile.secrets.clone());
        row.profile_revision = Set(Some(version));
        row.state = Set("reserved".to_owned());
    } else if input.provider.is_some() {
        return Err(invalid(
            "provider",
            "A free plan does not use a payment provider.",
        ));
    }
    row.insert(&tx).await.map_err(database_error)?;
    slot::ActiveModel {
        listing_id: Set(id),
        purchase_id: Set(id_string.clone()),
    }
    .insert(&tx)
    .await
    .map_err(database_error)?;
    if plan.billing_type == "free" {
        entitlement::ActiveModel {
            id: Set(id_string.clone()),
            listing_id: Set(id),
            mode: Set("free".to_owned()),
            status: Set("active".to_owned()),
            valid_from: Set(now),
            valid_until: Set(None),
            updated_at: Set(now),
        }
        .insert(&tx)
        .await
        .map_err(database_error)?;
    }
    tx.commit().await.map_err(database_error)?;
    continue_purchase(actor, &id_string, gateway).await?;
    Ok(id_string)
}

fn reference_valid(id: &str) -> bool {
    !id.is_empty() && id.len() <= 255 && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

pub(crate) async fn link_reference(
    tx: &DatabaseTransaction,
    row: &purchase::Model,
    kind: &str,
    value: &str,
    payment_id: Option<&str>,
) -> Result<(), FrameworkError> {
    if !reference_valid(value) {
        return Err(invalid(
            "payment",
            "The provider returned an invalid reference.",
        ));
    }
    let id = format!("{}:{}:{kind}:{value}", row.provider, row.mode);
    reference::Entity::insert(reference::ActiveModel {
        id: Set(id.clone()),
        provider: Set(row.provider.clone()),
        mode: Set(row.mode.clone()),
        kind: Set(kind.to_owned()),
        reference: Set(value.to_owned()),
        purchase_id: Set(row.id.clone()),
        payment_id: Set(payment_id.map(str::to_owned)),
    })
    .on_conflict(
        OnConflict::column(reference::Column::Id)
            .do_nothing()
            .to_owned(),
    )
    .try_insert()
    .exec(tx)
    .await
    .map_err(database_error)?;
    let existing = reference::Entity::find_by_id(&id)
        .one(tx)
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?;
    if existing.purchase_id != row.id
        || (existing.payment_id.is_some()
            && payment_id.is_some()
            && existing.payment_id.as_deref() != payment_id)
    {
        return Err(invalid(
            "payment",
            "This provider reference belongs to another purchase or payment.",
        ));
    }
    if existing.payment_id.is_none() && payment_id.is_some() {
        reference::Entity::update_many()
            .col_expr(reference::Column::PaymentId, Expr::value(payment_id))
            .filter(reference::Column::Id.eq(id))
            .exec(tx)
            .await
            .map_err(database_error)?;
    }
    Ok(())
}

async fn claim(
    row: &purchase::Model,
    state: &str,
) -> Result<Option<purchase::Model>, FrameworkError> {
    let db = DB::connection()?;
    let now = chrono::Utc::now().timestamp();
    let changed = purchase::Entity::update_many()
        .col_expr(purchase::Column::State, Expr::value(state))
        .col_expr(purchase::Column::Version, Expr::value(row.version + 1))
        .col_expr(purchase::Column::LeaseUntil, Expr::value(now + 30))
        .col_expr(purchase::Column::UpdatedAt, Expr::value(now))
        .filter(purchase::Column::Id.eq(&row.id))
        .filter(purchase::Column::Version.eq(row.version))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    if changed.rows_affected == 0 {
        return Ok(None);
    }
    owned(row.owner_id, &row.id).await.map(Some)
}

async fn failed(
    row: &purchase::Model,
    state: &str,
    error: GatewayError,
) -> Result<(), FrameworkError> {
    let db = DB::connection()?;
    purchase::Entity::update_many()
        .col_expr(purchase::Column::State, Expr::value(state))
        .col_expr(purchase::Column::ErrorCode, Expr::value(error.0))
        .col_expr(
            purchase::Column::LeaseUntil,
            Expr::value(chrono::Utc::now().timestamp() + 30),
        )
        .filter(purchase::Column::Id.eq(&row.id))
        .filter(purchase::Column::Version.eq(row.version))
        .exec(db.inner())
        .await
        .map_err(database_error)?;
    Ok(())
}

pub async fn continue_purchase(
    actor: i64,
    id: &str,
    gateway: &dyn Gateway,
) -> Result<(), FrameworkError> {
    require_verified(actor).await?;
    let mut row = owned(actor, id).await?;
    if row.provider == "free" {
        return Ok(());
    }
    let now = chrono::Utc::now().timestamp();
    if row.lease_until.is_some_and(|until| until > now) {
        return Ok(());
    }
    if row.customer_ref.is_none()
        && matches!(
            row.state.as_str(),
            "reserved" | "customer_unknown" | "customer_creating"
        )
    {
        let Some(claimed) = claim(&row, "customer_creating").await? else {
            return Ok(());
        };
        row = claimed;
        let db = DB::connection()?;
        let user = user::Entity::find_by_id(actor)
            .one(db.inner())
            .await
            .map_err(database_error)?
            .ok_or_else(missing)?;
        let request = CreateCustomerRequest {
            user_id: actor.to_string(),
            email: user.email,
            name: Some(user.name),
            metadata: Some(json!({"purchase_id": row.id})),
        };
        let customer = match gateway.create_customer(&row, request).await {
            Ok(customer) if reference_valid(&customer) => customer,
            Ok(_) => {
                failed(
                    &row,
                    "customer_unknown",
                    GatewayError("provider_reference_invalid"),
                )
                .await?;
                return Ok(());
            }
            Err(error) => {
                failed(&row, "customer_unknown", error).await?;
                return Ok(());
            }
        };
        let tx = db.inner().begin().await.map_err(database_error)?;
        let changed = purchase::Entity::update_many()
            .col_expr(purchase::Column::CustomerRef, Expr::value(&customer))
            .col_expr(purchase::Column::State, Expr::value("reserved"))
            .col_expr(purchase::Column::LeaseUntil, Expr::value(None::<i64>))
            .filter(purchase::Column::Id.eq(id))
            .filter(purchase::Column::Version.eq(row.version))
            .exec(&tx)
            .await
            .map_err(database_error)?;
        if changed.rows_affected == 0 {
            return Ok(());
        }
        link_reference(&tx, &row, "customer", &customer, None).await?;
        tx.commit().await.map_err(database_error)?;
        row = owned(actor, id).await?;
    }
    // Stripe explicitly supports the same key; never retry it after the provider's
    // retention window. Paddle ambiguous creates require evidence/reconciliation.
    let stripe_retry = row.provider == "stripe"
        && row.state == "checkout_creating"
        && now - row.created_at < 23 * 3600;
    if row.customer_ref.is_none() || !(row.state == "reserved" || stripe_retry) {
        return Ok(());
    }
    let Some(row) = claim(&row, "checkout_creating").await? else {
        return Ok(());
    };
    let base = &row.return_origin;
    let request = StartSessionRequest {
        mode: if row.billing_type == "one_time" {
            SessionMode::OneOff
        } else {
            SessionMode::Subscription
        },
        customer_ref: row.customer_ref.clone().ok_or_else(missing)?,
        price_refs: vec![row.price_id.clone().ok_or_else(missing)?],
        success_return_url: format!("{base}/dashboard/purchases/{}", row.id),
        cancel_return_url: format!("{base}/dashboard/purchases/{}", row.id),
        amount_hint: None,
        idempotency_key: (row.provider == "stripe").then(|| row.id.clone()),
        metadata: Some(
            json!({"purchase_id": row.id, "listing_id": row.listing_id.to_string(), "plan_key": row.plan_key}),
        ),
    };
    let payload = match gateway.start_session(&row, request).await {
        Ok(payload) => payload,
        Err(error) => {
            failed(&row, "checkout_creating", error).await?;
            return Ok(());
        }
    };
    let session = match &payload {
        SessionPayload::StripeCheckoutRedirect {
            url,
            provider_session_id,
        } if row.provider == "stripe"
            && reference_valid(provider_session_id)
            && provider_session_id.starts_with("cs_") =>
        {
            let valid = url::Url::parse(url).is_ok_and(|url| {
                url.scheme() == "https"
                    && url.host_str() == Some("checkout.stripe.com")
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.port_or_known_default() == Some(443)
            });
            if !valid {
                failed(
                    &row,
                    "checkout_creating",
                    GatewayError("provider_checkout_invalid"),
                )
                .await?;
                return Ok(());
            }
            provider_session_id.clone()
        }
        SessionPayload::PaddleInline {
            transaction_id,
            client_token,
            customer_token,
        } if row.provider == "paddle"
            && reference_valid(transaction_id)
            && transaction_id.starts_with("txn_")
            && row.public_key.as_deref() == Some(client_token)
            && customer_token.is_none() =>
        {
            transaction_id.clone()
        }
        _ => {
            failed(
                &row,
                "checkout_creating",
                GatewayError("provider_checkout_invalid"),
            )
            .await?;
            return Ok(());
        }
    };
    let encoded = serde_json::to_string(&payload)
        .map_err(|_| FrameworkError::internal("Could not retain checkout details."))?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    let changed = purchase::Entity::update_many()
        .col_expr(purchase::Column::SessionRef, Expr::value(&session))
        .col_expr(purchase::Column::CheckoutPayload, Expr::value(encoded))
        .col_expr(purchase::Column::State, Expr::value("open"))
        .col_expr(purchase::Column::LeaseUntil, Expr::value(None::<i64>))
        .col_expr(purchase::Column::ErrorCode, Expr::value(None::<String>))
        .filter(purchase::Column::Id.eq(id))
        .filter(purchase::Column::Version.eq(row.version))
        .filter(purchase::Column::State.eq("checkout_creating"))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    if changed.rows_affected == 1 {
        link_reference(&tx, &row, "session", &session, None).await?;
    }
    tx.commit().await.map_err(database_error)
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CheckoutView {
    Stripe {
        url: String,
    },
    Paddle {
        transaction_id: String,
        client_token: String,
        environment: &'static str,
    },
}
#[derive(Serialize)]
pub struct PurchaseView {
    pub id: String,
    pub listing_id: i64,
    pub listing_title: String,
    pub plan_name: String,
    pub amount: i64,
    pub currency: String,
    pub billing_type: String,
    pub provider: String,
    pub mode: String,
    pub state: String,
    pub error: Option<&'static str>,
    pub payment_status: String,
    pub paid_through: Option<i64>,
    pub cancel_requested: bool,
    pub cancel_at: Option<i64>,
    pub can_cancel: bool,
    pub can_retry: bool,
    pub checkout: Option<CheckoutView>,
}

pub async fn view(actor: i64, id: &str) -> Result<PurchaseView, FrameworkError> {
    let row = owned(actor, id).await?;
    let db = DB::connection()?;
    let title = revision::Entity::find_by_id(row.approved_revision_id)
        .one(db.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(missing)?
        .title;
    let latest = payment::Entity::find()
        .filter(payment::Column::PurchaseId.eq(id))
        .order_by_desc(payment::Column::PeriodStart)
        .order_by_desc(payment::Column::PaidAt)
        .limit(1)
        .one(db.inner())
        .await
        .map_err(database_error)?;
    let entitlement_id = latest
        .as_ref()
        .map(|payment| payment.id.as_str())
        .unwrap_or(id);
    let entitlement = entitlement::Entity::find_by_id(entitlement_id)
        .one(db.inner())
        .await
        .map_err(database_error)?;
    let now = chrono::Utc::now().timestamp();
    let payment_status = entitlement
        .as_ref()
        .map(|e| {
            if e.status != "active" {
                e.status.as_str()
            } else if e.valid_until.is_some_and(|end| end <= now) {
                "expired"
            } else if e.mode == "test" {
                "test"
            } else if e.mode == "free" {
                "free"
            } else {
                "paid"
            }
        })
        .unwrap_or("none")
        .to_owned();
    let checkout = if row.state == "open" {
        row.checkout_payload
            .as_ref()
            .and_then(|payload| serde_json::from_str::<SessionPayload>(payload).ok())
            .and_then(|payload| match payload {
                SessionPayload::StripeCheckoutRedirect { url, .. } => {
                    Some(CheckoutView::Stripe { url })
                }
                SessionPayload::PaddleInline {
                    transaction_id,
                    client_token,
                    ..
                } => Some(CheckoutView::Paddle {
                    transaction_id,
                    client_token,
                    environment: if row.mode == "test" {
                        "sandbox"
                    } else {
                        "production"
                    },
                }),
                _ => None,
            })
    } else {
        None
    };
    let can_retry = row.lease_until.is_none_or(|until| until <= now)
        && (matches!(
            row.state.as_str(),
            "reserved" | "customer_unknown" | "customer_creating"
        ) || (row.provider == "stripe"
            && row.state == "checkout_creating"
            && now - row.created_at < 23 * 3600));
    let can_cancel = row.subscription_ref.is_some()
        && row.cancel_at.is_none()
        && row.state != "canceled"
        && row.lease_until.is_none_or(|until| until <= now)
        && (!row.cancel_requested || row.error_code.is_some());
    Ok(PurchaseView { id: row.id, listing_id: row.listing_id, listing_title: title, plan_name: row.plan_name, amount: row.amount, currency: row.currency, billing_type: row.billing_type,
        provider: row.provider, mode: row.mode, state: row.state, error: row.error_code.map(|_| "Payment confirmation needs attention. No additional charge has been created. Retry when available or contact the directory operator."),
          payment_status, paid_through: entitlement.and_then(|e| e.valid_until), cancel_requested: row.cancel_requested, cancel_at: row.cancel_at, can_cancel, can_retry, checkout })
}
