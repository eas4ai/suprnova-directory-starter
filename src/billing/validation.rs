use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use suprnova::ValidationErrors;

use super::{BillingError, secrets::Credentials, settings::StoredProfile};

const MAX_CREDENTIAL_LENGTH: usize = 512;
pub const MAX_MAPPINGS: usize = 100;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Test,
    Live,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Live => "live",
        }
    }

    pub fn parse(value: &str) -> Result<Self, BillingError> {
        match value {
            "test" => Ok(Self::Test),
            "live" => Ok(Self::Live),
            _ => Err(BillingError::Unavailable("Choose test or live mode.")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Stripe,
    Paddle,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stripe => "stripe",
            Self::Paddle => "paddle",
        }
    }
}

// No Debug or Serialize: input credentials must never become response props or logs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveSettings {
    pub revision: i64,
    pub default_provider: Option<Provider>,
    pub stripe: ProfileInput,
    pub paddle: ProfileInput,
    pub mappings: Vec<MappingInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileInput {
    pub enabled: bool,
    pub public_key: String,
    pub api_key: String,
    pub webhook_key: String,
    pub clear_secrets: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingInput {
    pub plan: String,
    pub stripe: String,
    pub paddle: String,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PriceMapping {
    pub stripe: Option<String>,
    pub paddle: Option<String>,
}

fn field_error(field: &str, message: &str) -> BillingError {
    let mut errors = ValidationErrors::new();
    errors.add(field, message);
    BillingError::Invalid(errors)
}

fn token(value: &str, prefix: &str) -> bool {
    value.len() > prefix.len()
        && value.len() <= MAX_CREDENTIAL_LENGTH
        && value.starts_with(prefix)
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub(super) fn validate_credentials(
    mode: Mode,
    provider: Provider,
    public_key: &str,
    credentials: &Credentials,
) -> Result<(), BillingError> {
    let (api_prefix, public_prefix, webhook_prefix) = match (provider, mode) {
        (Provider::Stripe, Mode::Test) => ("sk_test_", "pk_test_", "whsec_"),
        (Provider::Stripe, Mode::Live) => ("sk_live_", "pk_live_", "whsec_"),
        (Provider::Paddle, Mode::Test) => ("pdl_sdbx_apikey_", "test_", "pdl_ntfset_"),
        (Provider::Paddle, Mode::Live) => ("pdl_live_apikey_", "live_", "pdl_ntfset_"),
    };
    for (name, value, prefix) in [
        ("api_key", credentials.api_key.as_str(), api_prefix),
        ("public_key", public_key, public_prefix),
        (
            "webhook_key",
            credentials.webhook_key.as_str(),
            webhook_prefix,
        ),
    ] {
        if !token(value, prefix) {
            return Err(field_error(
                &format!("{}.{}", provider.as_str(), name),
                &format!(
                    "Enter a valid {} {} beginning with {prefix} for {} mode.",
                    provider.as_str(),
                    name.replace('_', " "),
                    mode.as_str()
                ),
            ));
        }
    }
    Ok(())
}

pub(super) fn merge_profile(
    mode: Mode,
    provider: Provider,
    input: ProfileInput,
    stored: &StoredProfile,
) -> Result<(StoredProfile, Option<Credentials>), BillingError> {
    // Always decrypt the old value before accepting changes, even a clear request.
    // A wrong deployment key must not silently destroy recoverable credentials.
    let old = stored.credentials(mode, provider)?;
    if input.clear_secrets
        && (input.enabled || !input.api_key.is_empty() || !input.webhook_key.is_empty())
    {
        return Err(field_error(
            &format!("{}.clear_secrets", provider.as_str()),
            "Disable this profile and leave replacement secrets blank before clearing credentials.",
        ));
    }
    let credentials = if input.clear_secrets {
        None
    } else {
        let previous = old.unwrap_or_default();
        let api_key = if input.api_key.is_empty() {
            previous.api_key
        } else {
            input.api_key
        };
        let webhook_key = if input.webhook_key.is_empty() {
            previous.webhook_key
        } else {
            input.webhook_key
        };
        if api_key.is_empty() && webhook_key.is_empty() {
            None
        } else {
            Some(Credentials {
                api_key,
                webhook_key,
            })
        }
    };
    let public_key = if input.clear_secrets {
        String::new()
    } else {
        input.public_key
    };
    if let Some(ref credentials) = credentials {
        validate_credentials(mode, provider, &public_key, credentials)?;
    } else if input.enabled || !public_key.is_empty() {
        return Err(field_error(
            &format!("{}.api_key", provider.as_str()),
            "Enter all credentials together, or leave the disabled profile empty.",
        ));
    }
    let profile = StoredProfile {
        enabled: input.enabled,
        public_key,
        secrets: None,
    };
    Ok((profile, credentials))
}

pub(super) fn mappings(
    inputs: Vec<MappingInput>,
) -> Result<BTreeMap<String, PriceMapping>, BillingError> {
    if inputs.len() > MAX_MAPPINGS {
        return Err(field_error(
            "mappings",
            "Use at most 100 plan mappings per mode.",
        ));
    }
    let mut result = BTreeMap::new();
    for input in inputs {
        if input.plan.is_empty()
            || input.plan.len() > 64
            || !input
                .plan
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        {
            return Err(field_error(
                "mappings",
                "Plan identifiers need 1–64 lowercase letters, digits, hyphens or underscores.",
            ));
        }
        let stripe = optional_price(&input.stripe, "price_")?;
        let paddle = optional_price(&input.paddle, "pri_")?;
        if stripe.is_none() && paddle.is_none() {
            return Err(field_error(
                "mappings",
                "Give each plan at least one provider price, or remove its row.",
            ));
        }
        if result
            .insert(input.plan, PriceMapping { stripe, paddle })
            .is_some()
        {
            return Err(field_error(
                "mappings",
                "Each plan may appear only once in this mode.",
            ));
        }
    }
    Ok(result)
}

fn optional_price(value: &str, prefix: &str) -> Result<Option<String>, BillingError> {
    if value.is_empty() {
        return Ok(None);
    }
    if !token(value, prefix) {
        return Err(field_error(
            "mappings",
            &format!("Enter a price identifier beginning with {prefix}, or leave it blank."),
        ));
    }
    Ok(Some(value.to_owned()))
}
