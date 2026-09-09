use std::sync::Arc;

use suprnova::payments::PaymentProvider;
use suprnova_payments_paddle::{PaddleEnvironment, PaddleProvider};
use suprnova_payments_stripe::StripeProvider;

use super::{BillingError, Mode, Provider, secrets::Credentials, settings, validation};

pub struct ConfiguredProvider {
    pub mode: Mode,
    pub provider: Arc<dyn PaymentProvider>,
    pub price_id: String,
}

fn construct(
    mode: Mode,
    provider: Provider,
    public_key: &str,
    credentials: &Credentials,
) -> Result<Arc<dyn PaymentProvider>, BillingError> {
    validation::validate_credentials(mode, provider, public_key, credentials)?;
    // Both SDKs enter the same Rustls dependency graph. Select a provider when
    // the host has not done so; a concurrent installer winning is also success.
    if rustls::crypto::CryptoProvider::get_default().is_none()
        && rustls::crypto::ring::default_provider()
            .install_default()
            .is_err()
        && rustls::crypto::CryptoProvider::get_default().is_none()
    {
        return Err(BillingError::Unavailable(
            "The TLS provider could not initialize billing adapters.",
        ));
    }
    match provider {
        Provider::Stripe => Ok(Arc::new(StripeProvider::new(
            &credentials.api_key,
            public_key,
            &credentials.webhook_key,
        ))),
        Provider::Paddle => {
            let environment = match mode {
                Mode::Test => PaddleEnvironment::Sandbox,
                Mode::Live => PaddleEnvironment::Production,
            };
            PaddleProvider::new(&credentials.api_key, &credentials.webhook_key, public_key, environment)
                .map(|provider| Arc::new(provider) as Arc<dyn PaymentProvider>)
                .map_err(|_| BillingError::Unavailable("Paddle could not initialize this configuration. Check the credential format; the previous configuration is unchanged."))
        }
    }
}

pub(super) fn validate_construction(
    mode: Mode,
    provider: Provider,
    public_key: &str,
    credentials: &Credentials,
) -> Result<(), BillingError> {
    construct(mode, provider, public_key, credentials).map(|_| ())
}

/// Resolve only the requested provider, mode and plan; there is no fallback.
/// This constructs an adapter locally and does not create a checkout or call a provider.
pub async fn resolve(
    mode: Mode,
    provider: Provider,
    plan: &str,
) -> Result<ConfiguredProvider, BillingError> {
    super::secrets::require_key()?;
    let (_, settings) = settings::read(mode).await?;
    let profile = settings.profile(provider);
    if !profile.enabled {
        return Err(BillingError::Unavailable(
            "This provider is disabled for the selected mode.",
        ));
    }
    let mapping = settings
        .mappings
        .get(plan)
        .ok_or(BillingError::Unavailable(
            "This plan has no price mapping for the selected provider and mode.",
        ))?;
    let price_id = match provider {
        Provider::Stripe => &mapping.stripe,
        Provider::Paddle => &mapping.paddle,
    }
    .clone()
    .ok_or(BillingError::Unavailable(
        "This plan has no price mapping for the selected provider and mode.",
    ))?;
    let credentials = profile
        .credentials(mode, provider)?
        .ok_or(BillingError::Unavailable(
            "This provider has no credentials.",
        ))?;
    Ok(ConfiguredProvider {
        mode,
        provider: construct(mode, provider, &profile.public_key, &credentials)?,
        price_id,
    })
}
