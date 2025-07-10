mod browser_pool;
mod error;
mod html2pdf;

use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};

use browser_pool::BrowserPool;
use html2pdf::html2pdf;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let browser_pool = Arc::new(BrowserPool::new().await?);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/html2pdf", post(html2pdf))
        .with_state(browser_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn healthz() -> &'static str {
    "Pong"
}
