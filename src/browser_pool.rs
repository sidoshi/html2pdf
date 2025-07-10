use anyhow::Result;
use chromiumoxide::{
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::page::PrintToPdfParamsBuilder,
};
use futures::StreamExt;

pub struct BrowserPool {
    pub browser: Browser,
}

impl BrowserPool {
    pub async fn new() -> Result<Self> {
        // Initialize the browser pool here if needed
        let config = BrowserConfig::builder()
            .viewport(None) // Set viewport to None for headless mode
            .build()
            .map_err(|e| {
                eprintln!("Failed to create browser config: {}", e);
                anyhow::anyhow!("Browser config error")
            })?;

        let (browser, mut handler) = Browser::launch(config).await?;

        // Spawn handler properly - this is crucial for chromiumoxide to work
        // Don't break on errors as some WebSocket deserialization errors are normal
        tokio::task::spawn(async move {
            while let Some(_) = handler.next().await {
                // Continue processing regardless of errors
                // WebSocket deserialization errors are common and shouldn't stop the handler
            }
        });

        Ok(BrowserPool { browser })
    }

    pub async fn print_to_pdf(&self, html: &str) -> Result<Vec<u8>> {
        let page = self.browser.new_page("about:blank").await?;

        // Set the HTML content
        page.set_content(html).await?;

        // Prepare PDF parameters
        let params = PrintToPdfParamsBuilder::default().build();

        // Generate PDF
        let pdf_result = page.pdf(params).await?;

        Ok(pdf_result)
    }
}
