//! Durable provider administration. Provider protocols remain in Suprnova.

mod adapters;
pub mod cancellation;
pub mod checkout;
mod collect;
mod entity;
pub mod events;
pub mod evidence;
pub mod fulfillment;
pub mod gateway;
pub mod lifecycle_entities;
pub mod paddle_evidence;
pub mod plans;
pub mod reconcile;
mod secrets;
mod settings;
pub mod stripe_evidence;
mod validation;

#[cfg(test)]
mod stripe_evidence_tests;

pub use adapters::{ConfiguredProvider, resolve};
pub use settings::{load, save};
pub use validation::{Mode, Provider, SaveSettings};

use serde::Serialize;
use suprnova::{FrameworkError, ValidationErrors};

pub const BILLING_PERMISSION: &str = "billing.configure";
pub const ADMIN_PERMISSION: &str = "admin.access";

/// Messages deliberately exclude submitted values and provider SDK errors.
pub enum BillingError {
    Invalid(ValidationErrors),
    Conflict,
    Unavailable(&'static str),
    Database(sea_orm::DbErr),
}

impl From<sea_orm::DbErr> for BillingError {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::Database(error)
    }
}

impl std::fmt::Debug for BillingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(_) => f.write_str("Invalid billing configuration"),
            Self::Conflict => f.write_str("Billing configuration changed"),
            Self::Unavailable(message) => f.write_str(message),
            Self::Database(_) => f.write_str("Billing database operation failed"),
        }
    }
}

impl BillingError {
    pub fn into_framework(self) -> FrameworkError {
        match self {
            Self::Invalid(errors) => FrameworkError::Validation(errors),
            Self::Conflict => invalid(
                "configuration",
                "Another administrator saved changes. Reload this page before saving again.",
            ),
            Self::Unavailable(message) => invalid("configuration", message),
            Self::Database(error) => {
                tracing::error!(error = %error, "Billing database operation failed");
                FrameworkError::internal(
                    "Billing settings could not be saved or loaded. Try again or contact the operator.",
                )
            }
        }
    }
}

pub(crate) fn invalid(field: &str, message: &str) -> FrameworkError {
    let mut errors = ValidationErrors::new();
    errors.add(field, message);
    FrameworkError::Validation(errors)
}

#[derive(Serialize)]
pub struct SettingsView {
    pub mode: Mode,
    pub revision: i64,
    pub default_provider: Option<Provider>,
    pub stripe: ProfileView,
    pub paddle: ProfileView,
    pub mappings: Vec<MappingView>,
}

#[derive(Serialize)]
pub struct ProfileView {
    pub enabled: bool,
    pub public_key: String,
    pub has_secrets: bool,
}

#[derive(Serialize)]
pub struct MappingView {
    pub plan: String,
    pub stripe: Option<String>,
    pub paddle: Option<String>,
}
