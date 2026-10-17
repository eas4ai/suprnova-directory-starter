use super::{MANAGE_PERMISSION, entities::settings as entity, image_url, text};
use crate::listings::{conflict, database_error, invalid};
use sea_orm::{
    ColumnTrait, EntityTrait, ExprTrait, QueryFilter, TransactionTrait, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use suprnova::{DB, FrameworkError};

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Defaults {
    pub title_format: String,
    pub description: String,
    pub image: String,
    pub publisher: String,
    pub profiles: Vec<String>,
    pub google_verification: String,
    pub bing_verification: String,
    pub noindex: bool,
}
impl Default for Defaults {
    fn default() -> Self {
        Self {
            title_format: "{title} | {site}".into(),
            description: String::new(),
            image: String::new(),
            publisher: String::new(),
            profiles: Vec::new(),
            google_verification: String::new(),
            bing_verification: String::new(),
            noindex: false,
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: i64,
    pub defaults: Defaults,
}

impl Defaults {
    fn validate(mut self) -> Result<Self, FrameworkError> {
        self.title_format = text(&self.title_format, "title_format", 160)?;
        if !self.title_format.contains("{title}")
            || self.title_format.matches("{title}").count() != 1
            || self.title_format.matches("{site}").count() > 1
            || self
                .title_format
                .replace("{title}", "")
                .replace("{site}", "")
                .contains(['{', '}'])
        {
            return Err(invalid(
                "title_format",
                "Include {title} once and optionally {site}; no other placeholders are supported.",
            ));
        }
        self.description = text(&self.description, "description", 320)?;
        self.image = image_url(&self.image, "image")?;
        self.publisher = text(&self.publisher, "publisher", 120)?;
        for (field, value) in [
            ("google_verification", &mut self.google_verification),
            ("bing_verification", &mut self.bing_verification),
        ] {
            *value = text(value, field, 256)?;
            if !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'='))
            {
                return Err(invalid(
                    field,
                    "Paste only the verification value, not an HTML tag.",
                ));
            }
        }
        if self.profiles.len() > 10 {
            return Err(invalid(
                "profiles",
                "Use at most ten public social-profile URLs.",
            ));
        }
        for profile in &mut self.profiles {
            *profile = image_url(profile, "profiles")?;
            if !profile.starts_with("https://") {
                return Err(invalid(
                    "profiles",
                    "Use absolute HTTPS public social-profile URLs.",
                ));
            }
        }
        self.profiles.sort();
        self.profiles.dedup();
        Ok(self)
    }
}

pub async fn load() -> Result<Settings, FrameworkError> {
    let row = entity::Entity::find_by_id(1)
        .one(DB::connection()?.inner())
        .await
        .map_err(database_error)?
        .ok_or_else(|| {
            FrameworkError::internal("SEO settings are missing; run the documented migrations.")
        })?;
    Ok(Settings {
        version: row.version,
        defaults: suprnova::serde_json::from_str(&row.value)
            .map_err(|_| FrameworkError::internal("Stored SEO defaults are invalid."))?,
    })
}

pub async fn save(actor: i64, input: Settings) -> Result<(), FrameworkError> {
    crate::articles::require_permission(actor, MANAGE_PERMISSION).await?;
    let defaults = input.defaults.validate()?;
    let value = suprnova::serde_json::to_string(&defaults)
        .map_err(|_| FrameworkError::internal("Could not save SEO defaults."))?;
    let db = DB::connection()?;
    let tx = db.inner().begin().await.map_err(database_error)?;
    crate::accounts::guard_permission(&tx, actor, MANAGE_PERMISSION).await?;
    let updated = entity::Entity::update_many()
        .col_expr(entity::Column::Value, Expr::value(value))
        .col_expr(
            entity::Column::Version,
            Expr::col(entity::Column::Version).add(1),
        )
        .col_expr(
            entity::Column::UpdatedAt,
            Expr::value(chrono::Utc::now().timestamp()),
        )
        .filter(entity::Column::Id.eq(1))
        .filter(entity::Column::Version.eq(input.version))
        .exec(&tx)
        .await
        .map_err(database_error)?;
    if updated.rows_affected != 1 {
        return Err(conflict());
    }
    crate::audit::record(
        &tx,
        actor,
        "seo_settings",
        "1".into(),
        "seo_settings_saved",
        "Changed site metadata defaults, publisher identity, verification and indexing settings.",
    )
    .await?;
    tx.commit().await.map_err(database_error)
}
