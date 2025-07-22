#[tokio::main]
async fn main() {
    let app = axum::Router::new().route("/health", axum::routing::get(|| async { "OK" }));
    let listener = tokio::net::TcpListener::bind("[::]:8000")
        .await
        .expect("Bind socket");

    axum::serve(listener, app).await.expect("Serve app");
}
