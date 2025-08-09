use ammonia::clean;
use axum::{
    extract::{FromRef, State},
    http::{header, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use color_eyre::eyre::Context;
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
use tracing::{info, instrument, level_filters::LevelFilter};

mod handlers;
mod models;
mod quiz_session;
mod repository;
mod schema;

use regelator::config::{Config, LoggingConfig};
use repository::RuleRepository;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Round float to 1 decimal place for display  
fn round1_filter(value: Value) -> Result<String, minijinja::Error> {
    // Use try_into to convert to f64
    let f: f64 = value.try_into().map_err(|_| {
        minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "round1 filter requires a number",
        )
    })?;

    Ok(format!("{f:.1}"))
}

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
            return format!("{prefix}{slug}");
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
    config: Config,
}

impl AppState {
    fn new() -> Result<Self, color_eyre::eyre::Error> {
        let config = Config::load()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to load configuration: {}", e))?;

        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("src/templates"));

        // Register custom filters
        env.add_filter("markdown", markdown_filter);
        env.add_filter("round1", round1_filter);

        let manager = ConnectionManager::<SqliteConnection>::new(&config.database.url);
        let pool = Pool::builder().build(manager)?;

        Ok(AppState {
            templates: Arc::new(env),
            db: pool.clone(),
            rule_repository: RuleRepository::new(pool),
            config,
        })
    }
}

struct AppError(color_eyre::eyre::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("Application error: {:?}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<color_eyre::eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[instrument(skip(state))]
async fn health(State(state): State<AppState>) -> Result<&'static str, AppError> {
    let mut conn = state.db.get()?;
    sql_query("SELECT 1=1").execute(&mut conn)?;
    Ok("OK")
}

/// Initialize tracing subscriber based on configuration
fn init_tracing(config: &LoggingConfig) -> color_eyre::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::from_level(config.level).into())
        .from_env()
        .context("Set up env filter")?;

    // Create ErrorLayer for better error tracing
    let error_layer = tracing_error::ErrorLayer::default();

    match config.format.as_str() {
        "tree" => {
            let tree = tracing_tree::HierarchicalLayer::new(2)
                .with_targets(true)
                .with_ansi(config.enable_colors)
                .with_bracketed_fields(true);

            Registry::default()
                .with(env_filter)
                .with(error_layer)
                .with(tree)
                .init();
        }
        "json" => {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true)
                .with_timer(tracing_subscriber::fmt::time::SystemTime);

            Registry::default()
                .with(env_filter)
                .with(error_layer)
                .with(json_layer)
                .init();
        }
        _ => {
            // Default to compact format
            let fmt_layer = tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(config.enable_colors)
                .with_timer(tracing_subscriber::fmt::time::SystemTime);

            Registry::default()
                .with(env_filter)
                .with(error_layer)
                .with(fmt_layer)
                .init();
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize error reporting
    color_eyre::install().expect("Failed to install color-eyre");

    let state = AppState::new().expect("Failed to initialize application state");

    // Initialize tracing
    init_tracing(&state.config.logging).expect("Failed to initialize tracing");

    let bind_address = state.config.bind_address();

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
        // Quiz routes with session middleware
        .merge(
            Router::new()
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
                    "/{language}/quiz/{rule_set_slug}/clear",
                    get(handlers::clear_session_data),
                )
                .layer(middleware::from_fn(quiz_session::quiz_session_middleware)),
        )
        // Admin routes
        .route(
            "/admin",
            get(|| async { axum::response::Redirect::to("/admin/login") }),
        )
        .route("/admin/login", get(handlers::admin_login_form))
        .route("/admin/login", post(handlers::admin_login_submit))
        .route("/admin/dashboard", get(handlers::admin_dashboard))
        .route(
            "/admin/change-password",
            get(handlers::admin_change_password_form),
        )
        .route(
            "/admin/change-password",
            post(handlers::admin_change_password_submit),
        )
        .route("/admin/logout", get(handlers::admin_logout))
        // Admin statistics routes
        .route("/admin/stats", get(handlers::admin::admin_stats_dashboard))
        .route(
            "/admin/stats/question/{question_id}",
            get(handlers::admin::admin_question_detail_stats),
        )
        // Admin question management routes
        .route("/admin/questions", get(handlers::admin::questions_list))
        .route(
            "/admin/questions/new",
            get(handlers::admin::new_question_form),
        )
        .route(
            "/admin/questions/new",
            post(handlers::admin::create_question),
        )
        .route(
            "/admin/questions/{question_id}/edit",
            get(handlers::admin::edit_question_form),
        )
        .route(
            "/admin/questions/{question_id}/edit",
            post(handlers::admin::update_question),
        )
        .route(
            "/admin/questions/{question_id}/preview",
            get(handlers::admin::preview_question),
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

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("Bind socket");

    let actual_address = listener.local_addr().expect("Get local address");
    info!("Server listening on {}", actual_address);

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
