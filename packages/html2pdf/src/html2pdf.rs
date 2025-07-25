use axum::{Json, extract::State};
use base64::{Engine as _, engine::general_purpose};
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use serde::{Deserialize, Serialize};

use crate::{AppState, error::HttpError};

#[derive(Deserialize)]
pub struct Html2PdfRequest {
    pub blob: String,
    #[serde(rename = "printParams")]
    pub print_params: Option<PrintToPdfParams>,
}

#[derive(Serialize)]
pub struct Html2PdfResponse {
    #[serde(rename = "pdfBase64")]
    pub pdf_base64: String,
}

pub async fn html2pdf(
    State(app_state): State<AppState>,
    Json(payload): Json<Html2PdfRequest>,
) -> Result<Json<Html2PdfResponse>, HttpError> {
    tracing::debug!("Received HTML2PDF request");

    if payload.blob.is_empty() {
        return Err(HttpError::BadRequest(anyhow::anyhow!("Empty HTML content")));
    }

    let pdf_bytes = app_state
        .browser_pool
        .print_to_pdf(&payload.blob, payload.print_params)
        .await?;
    let pdf_base64 = general_purpose::STANDARD.encode(pdf_bytes);

    Ok(Json(Html2PdfResponse { pdf_base64 }))
}
