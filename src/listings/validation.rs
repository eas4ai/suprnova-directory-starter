use serde::Deserialize;
use suprnova::FrameworkError;

use super::invalid;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveListing {
    pub version: i64,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub url: String,
    pub category_ids: Vec<i64>,
    pub media_id: Option<String>,
    pub media_alt: String,
}

impl SaveListing {
    pub(super) fn validate(mut self) -> Result<Self, FrameworkError> {
        self.title = self.title.trim().to_owned();
        self.summary = self.summary.trim().to_owned();
        self.description = self.description.trim().to_owned();
        self.url = self.url.trim().to_owned();
        self.media_alt = self.media_alt.trim().to_owned();
        for (field, value, maximum) in [
            ("title", self.title.as_str(), 120),
            ("summary", self.summary.as_str(), 280),
            ("description", self.description.as_str(), 20_000),
            ("url", self.url.as_str(), 2_048),
        ] {
            if value.is_empty() || value.chars().count() > maximum || value.contains('\0') {
                return Err(invalid(
                    field,
                    &format!("Enter between 1 and {maximum} characters."),
                ));
            }
        }
        let url = url::Url::parse(&self.url)
            .map_err(|_| invalid("url", "Enter an absolute HTTP or HTTPS URL."))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(invalid(
                "url",
                "Use an HTTP or HTTPS URL without embedded credentials.",
            ));
        }
        self.category_ids.sort_unstable();
        self.category_ids.dedup();
        if self.category_ids.is_empty()
            || self.category_ids.len() > 5
            || self.category_ids.iter().any(|id| *id < 1)
        {
            return Err(invalid(
                "category_ids",
                "Choose between one and five active categories.",
            ));
        }
        if self.version < 0 {
            return Err(invalid("version", "Reload the listing before saving."));
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
                .map_err(|_| invalid("media_id", "Upload a valid listing image."))?;
        }
        Ok(self)
    }
}
