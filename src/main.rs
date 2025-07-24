use axum::{
    extract::{FromRef, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sql_query,
    sqlite::SqliteConnection,
    RunQueryDsl,
};
use minijinja::{Environment, Value};
use pulldown_cmark::{html, Parser};
use ammonia::clean;
use std::sync::Arc;

mod handlers;
mod models;
mod repository;
mod schema;

use repository::RuleRepository;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Convert markdown text to safe HTML for use in minijinja templates
fn markdown_filter(value: &Value) -> Result<String, minijinja::Error> {
    let markdown = value.as_str().ok_or_else(|| {
        minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "markdown filter requires a string value"
        )
    })?;
    
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    Ok(clean(&html_output).to_string())
}
use tower::Layer;
use tower_http::{
    compression::CompressionLayer, services::ServeDir, set_header::SetResponseHeaderLayer,
};

#[derive(Clone, FromRef)]
struct AppState {
    templates: Arc<Environment<'static>>,
    db: DbPool,
    rule_repository: RuleRepository,
}

impl AppState {
    fn new() -> Result<Self, eyre::Error> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("src/templates"));
        
        // Register custom filters
        env.add_filter("markdown", markdown_filter);

        // TODO: Read from configuration in future story
        let database_url = "db/regelator.db";
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder().build(manager)?;

        Ok(AppState {
            templates: Arc::new(env),
            db: pool.clone(),
            rule_repository: RuleRepository::new(pool),
        })
    }
}

struct AppError(eyre::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        eprintln!("Application error: {:?}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn health(State(state): State<AppState>) -> Result<&'static str, AppError> {
    let mut conn = state.db.get()?;
    sql_query("SELECT 1=1").execute(&mut conn)?;
    Ok("OK")
}

#[tokio::main]
async fn main() {
    let state = AppState::new().expect("Failed to initialize application state");

    let app = Router::new()
        .route(
            "/",
            get(|| async { axum::response::Redirect::to("/en/rules") }),
        )
        .route("/health", get(health))
        .route("/{language}/rules", get(handlers::list_rule_sets))
        .route("/{language}/rules/{rule_set}", get(handlers::list_rules))
        .route(
            "/{language}/rules/{rule_set}/definitions",
            get(handlers::definitions_page),
        )
        .route(
            "/{language}/rules/{rule_set}/{rule_slug}",
            get(handlers::show_rule),
        )
        .nest_service(
            "/static",
            SetResponseHeaderLayer::if_not_present(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            )
            .layer(ServeDir::new("static")),
        )
        .layer(CompressionLayer::new())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("[::]:8000")
        .await
        .expect("Bind socket");

    axum::serve(listener, app).await.expect("Serve app");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_filter_basic_features() {
        use minijinja::Value;
        
        // Test headers
        assert_eq!(markdown_filter(&Value::from("# Header 1")).unwrap(), "<h1>Header 1</h1>\n");
        assert_eq!(markdown_filter(&Value::from("## Header 2")).unwrap(), "<h2>Header 2</h2>\n");
        
        // Test emphasis
        assert_eq!(markdown_filter(&Value::from("*italic*")).unwrap(), "<p><em>italic</em></p>\n");
        assert_eq!(markdown_filter(&Value::from("**bold**")).unwrap(), "<p><strong>bold</strong></p>\n");
        
        // Test links (ammonia adds rel attributes for security)
        assert_eq!(
            markdown_filter(&Value::from("[link text](https://example.com)")).unwrap(),
            "<p><a href=\"https://example.com\" rel=\"noopener noreferrer\">link text</a></p>\n"
        );
        
        // Test lists
        assert_eq!(
            markdown_filter(&Value::from("- Item 1\n- Item 2")).unwrap(),
            "<ul>\n<li>Item 1</li>\n<li>Item 2</li>\n</ul>\n"
        );
        
        // Test ordered lists
        assert_eq!(
            markdown_filter(&Value::from("1. First\n2. Second")).unwrap(),
            "<ol>\n<li>First</li>\n<li>Second</li>\n</ol>\n"
        );
    }

    #[test]
    fn test_markdown_filter_xss_protection() {
        use minijinja::Value;
        
        // Test that script tags are completely removed by ammonia
        assert_eq!(
            markdown_filter(&Value::from("<script>alert('xss')</script>")).unwrap(),
            ""
        );
        
        // Test that raw HTML is sanitized by ammonia (script tags removed)
        assert_eq!(
            markdown_filter(&Value::from("Plain text with <script>")).unwrap(),
            "<p>Plain text with </p>"
        );
        
        // Test that dangerous links are sanitized (href removed, rel added)
        assert_eq!(
            markdown_filter(&Value::from("[dangerous](javascript:alert('xss'))")).unwrap(),
            "<p><a rel=\"noopener noreferrer\">dangerous</a></p>\n"
        );
    }

    #[test]
    fn test_markdown_filter_safe_html() {
        use minijinja::Value;
        
        // Test that basic HTML tags are preserved when safe
        assert_eq!(
            markdown_filter(&Value::from("Normal **bold** text")).unwrap(),
            "<p>Normal <strong>bold</strong> text</p>\n"
        );
        
        // Test paragraphs
        assert_eq!(
            markdown_filter(&Value::from("First paragraph\n\nSecond paragraph")).unwrap(),
            "<p>First paragraph</p>\n<p>Second paragraph</p>\n"
        );
    }

    #[test]
    fn test_markdown_filter_error_handling() {
        use minijinja::Value;
        
        // Test non-string input
        let result = markdown_filter(&Value::from(42));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("markdown filter requires a string value"));
    }
}
