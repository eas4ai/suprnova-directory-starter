use serde::Serialize;
use suprnova::FrameworkError;

#[derive(Clone, Serialize)]
pub struct Site {
    pub name: String,
    pub description: String,
    pub origin: String,
    pub logo_url: Option<String>,
    pub accent: String,
}

fn invalid(field: &str, instruction: &str) -> FrameworkError {
    FrameworkError::internal(format!("Site configuration: {field} {instruction}"))
}

pub fn origin() -> Result<String, FrameworkError> {
    let value = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8765".into());
    if value.len() > 2048 {
        return Err(invalid("APP_URL", "must be at most 2048 bytes."));
    }
    let url = url::Url::parse(&value)
        .map_err(|_| invalid("APP_URL", "must be an absolute HTTP(S) origin."))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid(
            "APP_URL",
            "must contain an HTTP(S) host without credentials, path, query or fragment.",
        ));
    }
    Ok(url.origin().ascii_serialization())
}

pub fn read() -> Result<Site, FrameworkError> {
    let name = std::env::var("APP_NAME")
        .unwrap_or_else(|_| "Directory".into())
        .trim()
        .to_owned();
    let description = std::env::var("SITE_DESCRIPTION")
        .unwrap_or_else(|_| "Discover useful tools, ideas and communities.".into())
        .trim()
        .to_owned();
    for (field, value, max) in [
        ("APP_NAME", &name, 80),
        ("SITE_DESCRIPTION", &description, 320),
    ] {
        if value.is_empty() || value.chars().count() > max || value.chars().any(char::is_control) {
            return Err(invalid(
                field,
                &format!("must contain 1 to {max} characters without control characters."),
            ));
        }
    }
    let origin = origin()?;
    let logo = std::env::var("SITE_LOGO_URL").unwrap_or_default();
    let logo_url = if logo.trim().is_empty() {
        None
    } else {
        let logo = logo.trim();
        if logo.len() > 2048
            || logo.chars().any(char::is_control)
            || logo.starts_with("//")
            || logo.contains('\\')
        {
            return Err(invalid(
                "SITE_LOGO_URL",
                "must be an HTTP(S) URL or a path beginning with one slash.",
            ));
        }
        let url = if logo.starts_with('/') {
            url::Url::parse(&format!("{origin}{logo}"))
        } else {
            url::Url::parse(logo)
        }
        .map_err(|_| {
            invalid(
                "SITE_LOGO_URL",
                "must be an HTTP(S) URL or an absolute site path.",
            )
        })?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(invalid(
                "SITE_LOGO_URL",
                "must use HTTP(S) without credentials.",
            ));
        }
        Some(url.to_string())
    };
    let accent = std::env::var("SITE_ACCENT").unwrap_or_else(|_| "#146b56".into());
    if accent.len() != 7
        || !accent.starts_with('#')
        || !accent[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid(
            "SITE_ACCENT",
            "must use six hexadecimal digits, such as #146b56.",
        ));
    }
    let rgb = [1, 3, 5].map(|start| {
        u8::from_str_radix(&accent[start..start + 2], 16).unwrap_or(255) as f64 / 255.0
    });
    let linear = rgb.map(|c| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let luminance = 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
    if 1.05 / (luminance + 0.05) < 4.5 {
        return Err(invalid(
            "SITE_ACCENT",
            "must be dark enough for white button text (contrast ratio at least 4.5).",
        ));
    }
    Ok(Site {
        name,
        description,
        origin,
        logo_url,
        accent,
    })
}
