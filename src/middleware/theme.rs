use axum::{
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Middleware to add Accept-CH header for Client Hints support
/// This enables browsers to send Sec-CH-Prefers-Color-Scheme header
pub async fn add_client_hints(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;

    // Add Client Hints acceptance header for color scheme preferences
    response.headers_mut().insert(
        "Accept-CH",
        HeaderValue::from_static("Sec-CH-Prefers-Color-Scheme"),
    );

    response
}
