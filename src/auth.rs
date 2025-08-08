use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const COOKIE_NAME: &str = "admin_session";

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims {
    pub admin_id: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

impl AdminClaims {
    pub fn new(admin_id: String, username: String, session_duration: chrono::Duration) -> Self {
        let now = Utc::now();
        Self {
            admin_id,
            username,
            exp: (now + session_duration).timestamp(),
            iat: now.timestamp(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// Create a signed JWT cookie for admin authentication
/// Works on localhost (secure cookies allowed over HTTP on localhost)
pub fn create_admin_cookie(
    admin_id: String,
    username: String,
    jwt_secret: &str,
    session_duration: chrono::Duration,
) -> Result<Cookie<'static>, jsonwebtoken::errors::Error> {
    let claims = AdminClaims::new(admin_id, username, session_duration);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )?;

    let cookie = Cookie::build((COOKIE_NAME, token))
        .http_only(true)
        .secure(true) // Works on localhost per MDN documentation
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(session_duration.num_seconds()))
        .path("/admin")
        .build();

    Ok(cookie)
}

/// Verify admin cookie and extract claims
pub fn verify_admin_cookie(
    cookie_value: &str,
    jwt_secret: &str,
) -> Result<AdminClaims, AdminAuthError> {
    let token_data = decode::<AdminClaims>(
        cookie_value,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    let claims = token_data.claims;

    if claims.is_expired() {
        return Err(AdminAuthError::Expired);
    }

    Ok(claims)
}

/// Create a cookie that clears the admin session
pub fn clear_admin_cookie() -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(0))
        .path("/admin")
        .build()
}

pub const ADMIN_COOKIE_NAME: &str = COOKIE_NAME;

#[derive(Debug)]
pub enum AdminAuthError {
    InvalidToken(jsonwebtoken::errors::Error),
    Expired,
    Missing,
}

impl From<jsonwebtoken::errors::Error> for AdminAuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AdminAuthError::InvalidToken(err)
    }
}

impl std::fmt::Display for AdminAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminAuthError::InvalidToken(e) => write!(f, "Invalid token: {}", e),
            AdminAuthError::Expired => write!(f, "Token expired"),
            AdminAuthError::Missing => write!(f, "No authentication token"),
        }
    }
}

impl std::error::Error for AdminAuthError {}

impl IntoResponse for AdminAuthError {
    fn into_response(self) -> Response {
        // For admin authentication failures, redirect to login page
        Redirect::to("/admin/login").into_response()
    }
}

/// AdminToken provides authenticated admin context to handlers
/// Newtype wrapper around AdminClaims that can only be constructed via authentication
#[derive(Debug)]
pub struct AdminToken(AdminClaims);

impl AdminToken {
    /// Get the admin ID
    pub fn admin_id(&self) -> &str {
        &self.0.admin_id
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.0.username
    }

    /// Get the expiration timestamp
    pub fn exp(&self) -> i64 {
        self.0.exp
    }

    /// Get the issued at timestamp
    pub fn iat(&self) -> i64 {
        self.0.iat
    }

    /// Internal constructor - only used by the extractor
    fn from_verified_claims(claims: AdminClaims) -> Self {
        Self(claims)
    }
}

impl<S> FromRequestParts<S> for AdminToken
where
    S: Send + Sync,
    crate::config::Config: axum::extract::FromRef<S>,
{
    type Rejection = AdminAuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract configuration to get JWT secret
        let config = crate::config::Config::from_ref(state);

        // Extract cookie jar
        let jar = axum_extra::extract::CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AdminAuthError::Missing)?;

        // Get admin cookie
        let cookie = jar.get(ADMIN_COOKIE_NAME).ok_or(AdminAuthError::Missing)?;

        // Verify cookie and get claims
        let claims = verify_admin_cookie(cookie.value(), &config.security.jwt_secret)?;

        Ok(AdminToken::from_verified_claims(claims))
    }
}
