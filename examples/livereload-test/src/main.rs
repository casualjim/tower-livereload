use axum::{response::Html, routing::get, Router};
use tower_livereload::LiveReloadLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let livereload = LiveReloadLayer::new();
    let app = Router::new()
        .route("/", get(|| async { Html("<html><head><title>Test</title></head><body><h1>Live Reload Test - With Fix</h1><p>This page uses the fixed tower-livereload that prevents infinite reload loops.</p></body></html>") }))
        .layer(livereload);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await?;
    println!("Server running on http://localhost:3030");
    axum::serve(listener, app).await?;

    Ok(())
}
