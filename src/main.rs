use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};

use serde::{Deserialize, Serialize};

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

async fn html2pdf(Json(payload): Json<Html2PdfRequest>) -> (StatusCode, Json<Html2PdfResponse>) {
    (
        StatusCode::OK,
        Json(Html2PdfResponse {
            pdf_base64: format!("PDF_BASE64_CONTENT_FOR_{}", payload.blob),
        }),
    )
}

#[derive(Deserialize)]
struct Html2PdfRequest {
    blob: String,
}

#[derive(Serialize)]
struct Html2PdfResponse {
    pdf_base64: String,
}
