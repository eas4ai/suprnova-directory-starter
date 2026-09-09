//! Owner drafts, exact-revision moderation and shared public visibility.

pub mod entities;
pub mod media;
pub mod queries;
mod validation;
pub mod workflow;

pub use validation::SaveListing;
pub const MODERATE_PERMISSION: &str = "listings.moderate";

use suprnova::{FrameworkError, ValidationErrors};

pub(crate) fn invalid(field: &str, message: &str) -> FrameworkError {
    let mut errors = ValidationErrors::new();
    errors.add(field, message);
    FrameworkError::Validation(errors)
}

pub(crate) fn conflict() -> FrameworkError {
    invalid(
        "version",
        "This listing changed after you opened it. Reload the saved version before trying again.",
    )
}

pub(crate) fn database_error(error: sea_orm::DbErr) -> FrameworkError {
    tracing::error!(error = %error, "Listing database operation failed");
    FrameworkError::database("The listing could not be loaded or saved. Try again.")
}

pub(crate) fn missing() -> FrameworkError {
    suprnova::AppError::not_found("Listing not found.").into()
}
