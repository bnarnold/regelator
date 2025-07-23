use axum::{
    extract::{FromRef, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    sql_query,
    sqlite::SqliteConnection,
    RunQueryDsl,
};
use minijinja::Environment;
use std::sync::Arc;

mod handlers;
mod models;
mod repository;

use repository::RuleRepository;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;
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
