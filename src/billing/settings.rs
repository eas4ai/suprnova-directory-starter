use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, sea_query::Expr};
use serde::{Deserialize, Serialize};
use suprnova::DB;

use super::{
    BillingError, MappingView, Mode, ProfileView, Provider, SaveSettings, SettingsView, adapters,
    entity,
    secrets::{self, Credentials},
    validation::{self, PriceMapping},
};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredProfile {
    pub enabled: bool,
    pub public_key: String,
    pub secrets: Option<String>,
}

impl StoredProfile {
    pub(super) fn credentials(
        &self,
        mode: Mode,
        provider: Provider,
    ) -> Result<Option<Credentials>, BillingError> {
        self.secrets
            .as_deref()
            .map(|wire| secrets::decrypt(mode, provider, wire))
            .transpose()
    }

    fn view(self) -> ProfileView {
        ProfileView {
            enabled: self.enabled,
            public_key: self.public_key,
            has_secrets: self.secrets.is_some(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredSettings {
    pub version: u8,
    pub stripe: StoredProfile,
    pub paddle: StoredProfile,
    pub default_provider: Option<Provider>,
    pub mappings: BTreeMap<String, PriceMapping>,
}

impl StoredSettings {
    pub(super) fn profile(&self, provider: Provider) -> &StoredProfile {
        match provider {
            Provider::Stripe => &self.stripe,
            Provider::Paddle => &self.paddle,
        }
    }
}

pub(super) async fn read(mode: Mode) -> Result<(i64, StoredSettings), BillingError> {
    let db = DB::connection()
        .map_err(|_| BillingError::Unavailable("The billing database is unavailable."))?;
    read_from(db.inner(), mode).await
}

pub(super) async fn read_from(
    connection: &impl ConnectionTrait,
    mode: Mode,
) -> Result<(i64, StoredSettings), BillingError> {
    let row = entity::Entity::find_by_id(mode.as_str())
        .one(connection)
        .await?
        .ok_or(BillingError::Unavailable(
            "Billing settings are missing. Run the application migrations.",
        ))?;
    let settings: StoredSettings = suprnova::serde_json::from_str(&row.payload).map_err(|_| {
        BillingError::Unavailable(
            "Stored billing settings are invalid. Restore an intact database before editing.",
        )
    })?;
    if settings.version != 1 || row.revision < 0 {
        return Err(BillingError::Unavailable(
            "The stored billing settings version is not supported by this application.",
        ));
    }
    Ok((row.revision, settings))
}

pub async fn load(mode: Mode) -> Result<SettingsView, BillingError> {
    secrets::require_key()?;
    let (revision, settings) = read(mode).await?;
    // Do not offer an editable form over unreadable secrets.
    settings.stripe.credentials(mode, Provider::Stripe)?;
    settings.paddle.credentials(mode, Provider::Paddle)?;
    Ok(SettingsView {
        mode,
        revision,
        default_provider: settings.default_provider,
        stripe: settings.stripe.view(),
        paddle: settings.paddle.view(),
        mappings: settings
            .mappings
            .into_iter()
            .map(|(plan, prices)| MappingView {
                plan,
                stripe: prices.stripe,
                paddle: prices.paddle,
            })
            .collect(),
    })
}

pub async fn save(mode: Mode, input: SaveSettings) -> Result<(), BillingError> {
    save_with(mode, input, adapters::validate_construction).await
}

async fn save_with(
    mode: Mode,
    input: SaveSettings,
    construct: impl Fn(Mode, Provider, &str, &Credentials) -> Result<(), BillingError>,
) -> Result<(), BillingError> {
    secrets::require_key()?;
    let (revision, old) = read(mode).await?;
    if revision != input.revision {
        return Err(BillingError::Conflict);
    }
    let next_revision = revision.checked_add(1).ok_or(BillingError::Unavailable(
        "Billing settings revision limit reached.",
    ))?;
    let mappings = validation::mappings(input.mappings)?;
    let (mut stripe, stripe_credentials) =
        validation::merge_profile(mode, Provider::Stripe, input.stripe, &old.stripe)?;
    let (mut paddle, paddle_credentials) =
        validation::merge_profile(mode, Provider::Paddle, input.paddle, &old.paddle)?;
    for (provider, profile, credentials) in [
        (Provider::Stripe, &mut stripe, stripe_credentials),
        (Provider::Paddle, &mut paddle, paddle_credentials),
    ] {
        if let Some(credentials) = credentials {
            construct(mode, provider, &profile.public_key, &credentials)?;
            profile.secrets = Some(secrets::encrypt(mode, provider, &credentials)?);
        }
    }
    // Disabling the selected default atomically clears the selection.
    let default_provider = input.default_provider.filter(|provider| match provider {
        Provider::Stripe => stripe.enabled,
        Provider::Paddle => paddle.enabled,
    });
    let settings = StoredSettings {
        version: 1,
        stripe,
        paddle,
        default_provider,
        mappings,
    };
    let payload = suprnova::serde_json::to_string(&settings)
        .map_err(|_| BillingError::Unavailable("Could not encode billing settings."))?;
    let db = DB::connection()
        .map_err(|_| BillingError::Unavailable("The billing database is unavailable."))?;
    // A single conditional UPDATE is the transaction boundary: no partial profile,
    // default or mapping writes, and no process-local mutex across deployments.
    let result = entity::Entity::update_many()
        .col_expr(entity::Column::Payload, Expr::value(payload))
        .col_expr(entity::Column::Revision, Expr::value(next_revision))
        .filter(entity::Column::Mode.eq(mode.as_str()))
        .filter(entity::Column::Revision.eq(revision))
        .exec(db.inner())
        .await?;
    if result.rows_affected != 1 {
        return Err(BillingError::Conflict);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::MigratorTrait;

    #[tokio::test(flavor = "current_thread")]
    async fn constructor_failure_preserves_configuration() {
        assert_eq!(std::env::var("APP_ENV").as_deref(), Ok("test"));
        crate::config::register_all();
        suprnova::Crypt::init(suprnova::EncryptionKey::from_env().unwrap());
        crate::bootstrap::register().await;
        crate::migrations::Migrator::up(DB::connection().unwrap().inner(), None)
            .await
            .unwrap();
        let input = || {
            suprnova::serde_json::from_value(suprnova::serde_json::json!({
            "revision":0,"default_provider":"stripe",
            "stripe":{"enabled":true,"public_key":"pk_test_fixture","api_key":"sk_test_fixture","webhook_key":"whsec_fixture","clear_secrets":false},
            "paddle":{"enabled":false,"public_key":"","api_key":"","webhook_key":"","clear_secrets":false},"mappings":[]
        })).unwrap()
        };
        let result = save_with(Mode::Test, input(), |_, _, _, _| {
            Err(BillingError::Unavailable("Controlled constructor failure"))
        })
        .await;
        assert!(matches!(
            result,
            Err(BillingError::Unavailable("Controlled constructor failure"))
        ));
        let unchanged = load(Mode::Test).await.unwrap();
        assert_eq!(unchanged.revision, 0);
        assert!(
            !unchanged.stripe.enabled
                && !unchanged.stripe.has_secrets
                && unchanged.default_provider.is_none()
        );
        save(Mode::Test, input()).await.unwrap();
        assert_eq!(load(Mode::Test).await.unwrap().revision, 1);
    }
}
