use crate::listings::invalid;
use serde::Deserialize;
use suprnova::FrameworkError;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveArticle {
    pub version: i64,
    #[serde(default)]
    pub seo: crate::seo::Overrides,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub media_id: Option<String>,
    pub media_alt: String,
    pub term_ids: Vec<i64>,
}

pub fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 120
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

impl SaveArticle {
    pub(super) fn validate(mut self) -> Result<Self, FrameworkError> {
        self.slug = self.slug.trim().to_owned();
        self.seo = self.seo.validate()?;
        self.title = self.title.trim().to_owned();
        self.summary = self.summary.trim().to_owned();
        self.body = self.body.trim().to_owned();
        self.media_alt = self.media_alt.trim().to_owned();
        if !valid_slug(&self.slug) {
            return Err(invalid(
                "slug",
                "Use 1 to 120 lowercase letters, digits and hyphens, starting and ending with a letter or digit.",
            ));
        }
        for (field, value, max) in [
            ("title", &self.title, 160),
            ("summary", &self.summary, 320),
            ("body", &self.body, 50_000),
        ] {
            if value.is_empty() || value.chars().count() > max || value.contains('\0') {
                return Err(invalid(
                    field,
                    &format!("Enter between 1 and {max} characters."),
                ));
            }
        }
        if self.version < 0 {
            return Err(invalid("version", "Reload the article before saving."));
        }
        self.term_ids.sort_unstable();
        self.term_ids.dedup();
        if self.term_ids.len() > 15 || self.term_ids.iter().any(|id| *id < 1) {
            return Err(invalid(
                "term_ids",
                "Choose at most fifteen categories and tags.",
            ));
        }
        if self.media_alt.chars().count() > 280
            || self.media_alt.contains('\0')
            || (self.media_id.is_some() && self.media_alt.is_empty())
        {
            return Err(invalid(
                "media_alt",
                "Describe the image in 1 to 280 characters.",
            ));
        }
        if let Some(id) = &self.media_id {
            uuid::Uuid::parse_str(id)
                .map_err(|_| invalid("media_id", "Upload a valid article image."))?;
        }
        Ok(self)
    }
}
