use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::Environment;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    templates: Arc<Environment<'static>>,
}

impl AppState {
    fn new() -> Self {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("src/templates"));
        
        AppState {
            templates: Arc::new(env),
        }
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

async fn home(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let tmpl = state.templates.get_template("home.html")?;
    let rendered = tmpl.render(())?;
    Ok(Html(rendered))
}

#[tokio::main]
async fn main() {
    let state = AppState::new();
    
    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(|| async { "OK" }))
        .with_state(state);
        
    let listener = tokio::net::TcpListener::bind("[::]:8000")
        .await
        .expect("Bind socket");

    axum::serve(listener, app).await.expect("Serve app");
}
