use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use charming::theme::Theme as CharmingTheme;

/// Theme preference extracted from browser Client Hints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl From<Theme> for CharmingTheme {
    fn from(val: Theme) -> Self {
        match val {
            Theme::Light => CharmingTheme::Default,
            Theme::Dark => CharmingTheme::Dark,
        }
    }
}

impl<S> FromRequestParts<S> for Theme
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get theme from Sec-CH-Prefers-Color-Scheme Client Hints header
        if let Some(color_scheme_header) = parts.headers.get("sec-ch-prefers-color-scheme")
            && let Ok("dark") = color_scheme_header.to_str()
        {
            Ok(Theme::Dark)
        } else {
            Ok(Theme::Light)
        }
    }
}
