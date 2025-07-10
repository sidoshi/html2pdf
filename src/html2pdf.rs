use axum::{Json, http::StatusCode};
use base64::{Engine as _, engine::general_purpose};
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::page::PrintToPdfParamsBuilder,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Html2PdfRequest {
    pub blob: String,
}

#[derive(Serialize)]
pub struct Html2PdfResponse {
    pub pdf_base64: String,
}

pub async fn html2pdf(
    Json(payload): Json<Html2PdfRequest>,
) -> Result<Json<Html2PdfResponse>, StatusCode> {
    // Validate input
    if payload.blob.is_empty() {
        eprintln!("Empty HTML blob received");
        return Err(StatusCode::BAD_REQUEST);
    }

    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .viewport(None)
            .build()
            .map_err(|e| {
                eprintln!("Failed to create browser config: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
    )
    .await
    .map_err(|e| {
        eprintln!("Failed to launch browser: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Spawn handler properly - this is crucial for chromiumoxide to work
    // Don't break on errors as some WebSocket deserialization errors are normal
    let _handle = tokio::task::spawn(async move {
        while let Some(_) = handler.next().await {
            // Continue processing regardless of errors
            // WebSocket deserialization errors are common and shouldn't stop the handler
        }
    });

    let page = browser.new_page("about:blank").await.map_err(|e| {
        eprintln!("Failed to create new page: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Wait a bit for the page to be ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Set the HTML content
    page.set_content(&payload.blob).await.map_err(|e| {
        eprintln!("Failed to set content: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate PDF using the correct method
    let pdf_data = page
        .save_pdf(PrintToPdfParamsBuilder::default().build(), "ht.pdf")
        .await
        .map_err(|e| {
            eprintln!("Failed to generate PDF: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Close the browser to clean up resources
    let _ = browser.close().await;

    let pdf_base64 = general_purpose::STANDARD.encode(pdf_data);

    Ok(Json(Html2PdfResponse { pdf_base64 }))
}
