use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use tracing::debug;

const QUIZ_SESSION_COOKIE_NAME: &str = "quiz_session";

/// QuizSession provides quiz session context to handlers
/// Simple wrapper around session ID for consistent access
#[derive(Debug)]
pub struct QuizSession {
    /// The unique session identifier
    pub session_id: String,
}

impl QuizSession {
    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Create a new quiz session with generated UUID
    pub fn new() -> Self {
        Self {
            session_id: uuid::Uuid::now_v7().to_string(),
        }
    }

    /// Create from existing session ID
    fn from_session_id(session_id: String) -> Self {
        Self { session_id }
    }
}

impl Default for QuizSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a simple session cookie (no encryption needed - just tracking)
pub fn create_quiz_session_cookie(session_id: String) -> Cookie<'static> {
    Cookie::build((QUIZ_SESSION_COOKIE_NAME, session_id))
        .http_only(true)
        .secure(false) // Allow HTTP for development
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(24)) // 24 hour session
        .path("/")
        .build()
}

/// Clear the quiz session cookie
pub fn clear_quiz_session_cookie() -> Cookie<'static> {
    Cookie::build((QUIZ_SESSION_COOKIE_NAME, ""))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(0))
        .path("/")
        .build()
}

impl<S> FromRequestParts<S> for QuizSession
where
    S: Send + Sync,
{
    type Rejection = (); // Never reject - always provide a session

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract cookie jar
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(_) => return Ok(QuizSession::new()), // Create new session if no jar
        };

        // Get existing session or create new one
        if let Some(cookie) = jar.get(QUIZ_SESSION_COOKIE_NAME) {
            let session_id = cookie.value().to_string();
            if !session_id.is_empty() && uuid::Uuid::parse_str(&session_id).is_ok() {
                debug!("Found existing quiz session: {}", session_id);
                Ok(QuizSession::from_session_id(session_id))
            } else {
                debug!("Invalid session ID, creating new session");
                Ok(QuizSession::new()) // Invalid session ID, create new one
            }
        } else {
            debug!("No session cookie found, creating new session");
            Ok(QuizSession::new()) // No session cookie, create new one
        }
    }
}

/// Simple middleware function compatible with `from_fn`
/// Automatically sets session cookies for quiz routes if not present
pub async fn quiz_session_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract cookie jar from request
    let (mut parts, body) = req.into_parts();
    let jar = match CookieJar::from_request_parts(&mut parts, &()).await {
        Ok(jar) => jar,
        Err(_) => {
            debug!("Could not extract cookie jar, proceeding without session");
            let req = axum::extract::Request::from_parts(parts, body);
            return next.run(req).await;
        }
    };

    // Check if quiz session cookie already exists and is valid
    let needs_cookie = if let Some(cookie) = jar.get(QUIZ_SESSION_COOKIE_NAME) {
        let session_id = cookie.value();
        session_id.is_empty() || uuid::Uuid::parse_str(session_id).is_err()
    } else {
        true
    };

    // Reconstruct request and call next handler
    let req = axum::extract::Request::from_parts(parts, body);
    let mut response = next.run(req).await;

    // Add session cookie to response if needed
    if needs_cookie {
        let new_session_id = uuid::Uuid::now_v7().to_string();
        let cookie = create_quiz_session_cookie(new_session_id.clone());

        debug!("Setting new quiz session cookie: {}", new_session_id);

        // Convert cookie to header value and add to response
        if let Ok(cookie_header) = cookie.to_string().parse::<axum::http::HeaderValue>() {
            response
                .headers_mut()
                .append(axum::http::header::SET_COOKIE, cookie_header);
        }
    }

    response
}
