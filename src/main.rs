mod html2pdf;

use axum::{
    Router,
    routing::{get, post},
};

use html2pdf::html2pdf;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/html2pdf", post(html2pdf));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> &'static str {
    "Pong"
}
