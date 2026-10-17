use serde::{Deserialize, Serialize};
use suprnova::{Crypt, EncryptionKey, crypto::CryptPurpose};

use super::{BillingError, Mode, Provider};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Credentials {
    pub api_key: String,
    pub webhook_key: String,
}

pub(super) fn require_key() -> Result<(), BillingError> {
    let key = EncryptionKey::from_env().map_err(|_| BillingError::Unavailable(
        "The operator must set a valid APP_KEY and restart the application before configuring billing."))?;
    if key.as_bytes().iter().all(|byte| *byte == key.as_bytes()[0]) || !Crypt::is_initialized() {
        return Err(BillingError::Unavailable(
            "Billing requires a generated APP_KEY; default keys are not accepted.",
        ));
    }
    Ok(())
}

fn context(mode: Mode, provider: Provider) -> String {
    format!(
        "directory.billing.v1.{}.{}",
        mode.as_str(),
        provider.as_str()
    )
}

pub(super) fn encrypt(
    mode: Mode,
    provider: Provider,
    credentials: &Credentials,
) -> Result<String, BillingError> {
    require_key()?;
    let plain = suprnova::serde_json::to_string(credentials)
        .map_err(|_| BillingError::Unavailable("Could not encode billing credentials."))?;
    Crypt::encrypt_string_for(CryptPurpose::Cast, &context(mode, provider), &plain).map_err(|_| {
        BillingError::Unavailable(
            "Could not encrypt billing credentials. The previous configuration is unchanged.",
        )
    })
}

pub(super) fn decrypt(
    mode: Mode,
    provider: Provider,
    wire: &str,
) -> Result<Credentials, BillingError> {
    require_key()?;
    let plain = Crypt::decrypt_string_for(CryptPurpose::Cast, &context(mode, provider), wire)
        .map_err(|_| BillingError::Unavailable("Stored billing credentials cannot be decrypted. Restore the correct APP_KEY and intact database before editing."))?;
    suprnova::serde_json::from_str(&plain).map_err(|_| {
        BillingError::Unavailable(
            "Stored billing credentials are invalid. Restore an intact database before editing.",
        )
    })
}
