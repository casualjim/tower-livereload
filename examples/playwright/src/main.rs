use axum::{
    response::Html,
    routing::{get, post},
    Router,
};
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();
    let app = Router::new()
        .route("/", get(|| async { Html("<h1>Playwright!</h1>") }))
        .route(
            "/reload",
            post(|| async move {
                reloader.reload();
            }),
        )
        .layer(livereload);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
