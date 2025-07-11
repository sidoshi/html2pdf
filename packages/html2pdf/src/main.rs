mod browser_pool;
mod cnfg;
mod error;
mod html2pdf;

use std::sync::Arc;

use anyhow::Result;
use axum::http::Method;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use browser_pool::BrowserPool;
use html2pdf::html2pdf;

#[tokio::main]
async fn main() -> Result<()> {
    let config = cnfg::get();
    tracing_subscriber::fmt::init();

    let browser_pool = Arc::new(BrowserPool::new().await?);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_credentials(false);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/html2pdf", post(html2pdf))
        .with_state(browser_pool)
        .layer(cors);

    let port = config.port;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn healthz() -> &'static str {
    "Pong"
}
