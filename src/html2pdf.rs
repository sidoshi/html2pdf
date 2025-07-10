use std::sync::Arc;

use axum::{Json, extract::State};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

use crate::{browser_pool::BrowserPool, error::HttpError};

#[derive(Deserialize)]
pub struct Html2PdfRequest {
    pub blob: String,
}

#[derive(Serialize)]
pub struct Html2PdfResponse {
    pub pdf_base64: String,
}

pub async fn html2pdf(
    State(browser_pool): State<Arc<BrowserPool>>,
    Json(payload): Json<Html2PdfRequest>,
) -> Result<Json<Html2PdfResponse>, HttpError> {
    if payload.blob.is_empty() {
        return Err(HttpError::BadRequest(anyhow::anyhow!("Empty HTML content")));
    }

    let pdf_bytes = browser_pool.print_to_pdf(&payload.blob).await?;
    let pdf_base64 = general_purpose::STANDARD.encode(pdf_bytes);

    Ok(Json(Html2PdfResponse { pdf_base64 }))
}
