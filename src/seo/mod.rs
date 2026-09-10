//! Revision-bound metadata and operator SEO controls.
pub mod discovery;
pub mod entities;
pub mod markdown;
pub mod middleware;
pub mod not_found;
pub mod redirects;
pub mod report;
pub mod settings;

use serde::{Deserialize, Serialize};
use suprnova::FrameworkError;

pub const MANAGE_PERMISSION: &str = "seo.manage";

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Overrides {
    pub title: String,
    pub description: String,
    pub image: String,
    pub noindex: bool,
}

impl Overrides {
    pub fn validate(mut self) -> Result<Self, FrameworkError> {
        self.title = text(&self.title, "seo.title", 160)?;
        self.description = text(&self.description, "seo.description", 320)?;
        self.image = image_url(&self.image, "seo.image")?;
        Ok(self)
    }

    pub fn decode(value: &str) -> Result<Self, FrameworkError> {
        suprnova::serde_json::from_str(value)
            .map_err(|_| FrameworkError::internal("Stored SEO metadata is invalid."))
    }

    pub fn encode(&self) -> Result<String, FrameworkError> {
        suprnova::serde_json::to_string(self)
            .map_err(|_| FrameworkError::internal("Could not encode SEO metadata."))
    }
}

pub fn text(value: &str, field: &str, limit: usize) -> Result<String, FrameworkError> {
    let value = value.trim();
    if value.chars().count() > limit || value.chars().any(char::is_control) {
        return Err(crate::listings::invalid(
            field,
            &format!("Use at most {limit} characters without control characters."),
        ));
    }
    Ok(value.into())
}

/// Images are referenced, never fetched by the server. Private media paths are not valid social images.
pub fn image_url(value: &str, field: &str) -> Result<String, FrameworkError> {
    let value = text(value, field, 2048)?;
    if value.is_empty() {
        return Ok(value);
    }
    let origin = crate::config::site::origin()?;
    let absolute = if value.starts_with('/') {
        format!("{origin}{value}")
    } else {
        value.clone()
    };
    let parsed = url::Url::parse(&absolute).map_err(|_| {
        crate::listings::invalid(field, "Use an HTTP(S) image URL or public site path.")
    })?;
    if value.starts_with("//")
        || value.contains('\\')
        || !matches!(parsed.scheme(), "https" | "http")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
        || parsed.query().is_some()
        || (parsed.origin().ascii_serialization() == origin
            && ["/admin", "/dashboard", "/billing"]
                .iter()
                .any(|p| parsed.path().starts_with(p)))
    {
        return Err(crate::listings::invalid(
            field,
            "Use a public HTTP(S) image without credentials, query or fragment.",
        ));
    }
    Ok(value)
}

pub fn absolute_image(value: &str, origin: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else if value.starts_with('/') {
        Some(format!("{origin}{value}"))
    } else {
        Some(value.into())
    }
}

#[derive(Serialize)]
pub struct Preview {
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub canonical: String,
    pub noindex: bool,
}

/// Public metadata and operator previews resolve the same stored values.
pub fn preview(
    site: &crate::config::site::Site,
    defaults: &settings::Defaults,
    overrides: &Overrides,
    title: &str,
    description: &str,
    image: Option<&str>,
    path: &str,
) -> Preview {
    let title = if overrides.title.is_empty() {
        title
    } else {
        &overrides.title
    };
    // Expand placeholders in the format only, never placeholders present in content.
    let formatted = defaults
        .title_format
        .split("{title}")
        .map(|part| part.replace("{site}", &site.name))
        .collect::<Vec<_>>()
        .join(title);
    let description = if !overrides.description.is_empty() {
        &overrides.description
    } else if !description.is_empty() {
        description
    } else if !defaults.description.is_empty() {
        &defaults.description
    } else {
        &site.description
    };
    let image = if !overrides.image.is_empty() {
        Some(overrides.image.as_str())
    } else {
        image.or_else(|| (!defaults.image.is_empty()).then_some(defaults.image.as_str()))
    };
    Preview {
        title: formatted,
        description: description.into(),
        image: image.and_then(|image| absolute_image(image, &site.origin)),
        canonical: format!("{}{path}", site.origin),
        noindex: defaults.noindex || overrides.noindex,
    }
}
