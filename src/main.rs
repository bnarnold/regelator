use ammonia::clean;
use axum::{
    extract::{FromRef, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sql_query,
    sqlite::SqliteConnection,
    RunQueryDsl,
};
use minijinja::{Environment, Value};
use pulldown_cmark::{html, Event, Parser, Tag};
use std::collections::HashMap;
use std::sync::Arc;

mod handlers;
mod models;
mod repository;
mod schema;

use repository::RuleRepository;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Convert markdown text to safe HTML with custom link rewriting
fn markdown_filter(
    markdown: &Value,
    link_context: Option<&Value>,
) -> Result<String, minijinja::Error> {
    let markdown_str = markdown.as_str().ok_or_else(|| {
        minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "markdown filter requires a string value",
        )
    })?;

    // Parse link context if provided
    let link_map: HashMap<String, String> = if let Some(context) = link_context {
        if let Some(obj) = context.as_object() {
            if let Some(iter) = obj.try_iter() {
                iter.filter_map(|key| {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(value) = context.get_item(&Value::from(key_str)) {
                            if let Some(value_str) = value.as_str() {
                                return Some((key_str.to_string(), value_str.to_string()));
                            }
                        }
                    }
                    None
                })
                .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    let parser = Parser::new(markdown_str);

    // Process events to rewrite custom link schemes
    let processed_events = parser.map(|event| match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let new_dest = rewrite_custom_link(&dest_url, &link_map);
            Event::Start(Tag::Link {
                link_type,
                dest_url: new_dest.into(),
                title,
                id,
            })
        }
        _ => event,
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, processed_events);
    Ok(clean(&html_output).to_string())
}

/// Rewrite custom link schemes like "definition:slug" to full URLs
fn rewrite_custom_link(dest_url: &str, link_map: &HashMap<String, String>) -> String {
    if let Some((scheme, slug)) = dest_url.split_once(':') {
        if let Some(prefix) = link_map.get(scheme) {
            return format!("{}{}", prefix, slug);
        }
    }

    // Return original URL if not a custom scheme or scheme not found
    dest_url.to_string()
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
        .route(
            "/quiz",
            get(|| async { axum::response::Redirect::to("/en/quiz/wfdf-ultimate") }),
        )
        .route(
            "/{language}/quiz/{rule_set_slug}",
            get(handlers::quiz_landing),
        )
        .route(
            "/{language}/quiz/{rule_set_slug}/start",
            post(handlers::start_quiz_session),
        )
        .route(
            "/{language}/quiz/{rule_set_slug}/question",
            post(handlers::random_quiz_question),
        )
        .route(
            "/{language}/quiz/{rule_set_slug}/submit",
            post(handlers::submit_quiz_answer),
        )
        .route(
            "/{language}/quiz/{rule_set_slug}/session/{session_id}/clear",
            get(handlers::clear_session_data),
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
        use minijinja::{context, Environment};

        let mut env = Environment::new();
        env.add_filter("markdown", markdown_filter);

        // Test headers
        let ctx = context! { content => "# Header 1" };
        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        assert_eq!(tmpl.render(&ctx).unwrap(), "<h1>Header 1</h1>\n");

        // Test emphasis
        let ctx = context! { content => "*italic*" };
        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        assert_eq!(tmpl.render(&ctx).unwrap(), "<p><em>italic</em></p>\n");

        // Test links (ammonia adds rel attributes for security)
        let ctx = context! { content => "[link text](https://example.com)" };
        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        let result = tmpl.render(&ctx).unwrap();
        assert!(result
            .contains("<a href=\"https://example.com\" rel=\"noopener noreferrer\">link text</a>"));
    }

    #[test]
    fn test_markdown_filter_xss_protection() {
        use minijinja::{context, Environment};

        let mut env = Environment::new();
        env.add_filter("markdown", markdown_filter);

        // Test that script tags are completely removed by ammonia
        let ctx = context! { content => "<script>alert('xss')</script>" };
        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        assert_eq!(tmpl.render(&ctx).unwrap(), "");

        // Test that dangerous links are sanitized (href removed, rel added)
        let ctx = context! { content => "[dangerous](javascript:alert('xss'))" };
        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        let result = tmpl.render(&ctx).unwrap();
        assert!(result.contains("<a rel=\"noopener noreferrer\">dangerous</a>"));
    }

    #[test]
    fn test_markdown_filter_safe_html() {
        use minijinja::{context, Environment};

        let mut env = Environment::new();
        env.add_filter("markdown", markdown_filter);

        // Test that basic HTML tags are preserved when safe
        let ctx = context! { content => "Normal **bold** text" };
        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        assert_eq!(
            tmpl.render(&ctx).unwrap(),
            "<p>Normal <strong>bold</strong> text</p>\n"
        );
    }

    #[test]
    fn test_markdown_filter_link_rewriting() {
        use minijinja::{context, Environment};

        let mut env = Environment::new();
        env.add_filter("markdown", markdown_filter);

        let mut link_map = std::collections::HashMap::new();
        link_map.insert(
            "definition".to_string(),
            "/en/rules/wfdf-ultimate/definitions#".to_string(),
        );
        link_map.insert("rule".to_string(), "#".to_string());

        let ctx = context! {
            link_map => link_map,
            content => "See [the throw](definition:throw) and [rule 16.3](rule:handling-contested-calls)"
        };

        let tmpl = env
            .template_from_str("{{ content | markdown(link_map) }}")
            .unwrap();
        let result = tmpl.render(&ctx).unwrap();

        assert!(result.contains("href=\"/en/rules/wfdf-ultimate/definitions#throw\""));
        assert!(result.contains("href=\"#handling-contested-calls\""));
    }

    #[test]
    fn test_markdown_filter_no_link_context() {
        use minijinja::{context, Environment};

        let mut env = Environment::new();
        env.add_filter("markdown", markdown_filter);

        let ctx = context! {
            content => "See [the throw](definition:throw) and [regular link](https://example.com)"
        };

        let tmpl = env.template_from_str("{{ content | markdown }}").unwrap();
        let result = tmpl.render(&ctx).unwrap();

        // Without link_map, custom schemes get stripped by ammonia (this is correct security behavior)
        assert!(result.contains("rel=\"noopener noreferrer\">the throw</a>"));
        assert!(!result.contains("href=\"definition:throw\""));
        // Regular links should work normally
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_markdown_filter_mixed_links() {
        use minijinja::{context, Environment};

        let mut env = Environment::new();
        env.add_filter("markdown", markdown_filter);

        let mut link_map = std::collections::HashMap::new();
        link_map.insert(
            "definition".to_string(),
            "/en/rules/wfdf-ultimate/definitions#".to_string(),
        );

        let ctx = context! {
            link_map => link_map,
            content => "See [term](definition:throw), [external](https://example.com), and [unknown](unknown:test)"
        };

        let tmpl = env
            .template_from_str("{{ content | markdown(link_map) }}")
            .unwrap();
        let result = tmpl.render(&ctx).unwrap();

        // Known scheme should be rewritten
        assert!(result.contains("href=\"/en/rules/wfdf-ultimate/definitions#throw\""));
        // Regular URL should remain unchanged
        assert!(result.contains("href=\"https://example.com\""));
        // Unknown scheme gets stripped by ammonia (this is correct security behavior)
        assert!(result.contains("rel=\"noopener noreferrer\">unknown</a>"));
        assert!(!result.contains("href=\"unknown:test\""));
    }
}
