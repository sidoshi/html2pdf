use anyhow::Result;
use chromiumoxide::{
    Page,
    browser::{Browser, BrowserConfig},
    cdp::browser_protocol::page::{PrintToPdfParams, PrintToPdfParamsBuilder},
};
use futures::StreamExt;
use tokio::sync::{Mutex, Semaphore};

pub struct BrowserPool {
    browser: Browser,
    page_pool: Mutex<Vec<Page>>,
    semaphore: Semaphore,
    max_pool_size: usize,
}

impl BrowserPool {
    pub async fn new() -> Result<Self> {
        Self::new_with_pool_size(10).await
    }

    pub async fn new_with_pool_size(max_concurrent_tabs: usize) -> Result<Self> {
        // Initialize the browser
        let config = BrowserConfig::builder().build().map_err(|e| {
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

        Ok(BrowserPool {
            browser,
            page_pool: Mutex::new(Vec::new()),
            semaphore: Semaphore::new(max_concurrent_tabs),
            max_pool_size: max_concurrent_tabs,
        })
    }

    pub async fn print_to_pdf(
        &self,
        html: &str,
        custom_params: Option<PrintToPdfParams>,
    ) -> Result<Vec<u8>> {
        // Acquire semaphore permit to limit concurrent usage
        //
        let _permit = self.semaphore.acquire().await?;

        // Try to get a page from the pool, or create a new one
        let page = self.get_or_create_page().await?;

        // Set the HTML content
        page.set_content(html).await?;

        // Create default params with spread-like syntax
        let default_params = PrintToPdfParamsBuilder::default()
            .print_background(true)
            .build();

        let params = custom_params.map_or(default_params, |custom| PrintToPdfParams {
            print_background: custom.print_background.or(Some(true)),
            ..custom
        });

        // Generate PDF
        let pdf_result = page.pdf(params).await?;

        // Return the page to the pool instead of closing it
        self.return_page_to_pool(page).await;

        Ok(pdf_result)
    }

    async fn get_or_create_page(&self) -> Result<Page> {
        // Try to get a page from the pool first
        {
            let mut pool = self.page_pool.lock().await;
            if let Some(page) = pool.pop() {
                return Ok(page);
            }
        }

        // Create a new page if pool is empty
        let page = self.browser.new_page("about:blank").await?;
        Ok(page)
    }

    async fn return_page_to_pool(&self, page: Page) {
        let mut pool = self.page_pool.lock().await;
        if pool.len() < self.max_pool_size {
            // Clear any existing content before returning to pool
            let _ = page.goto("about:blank").await;
            pool.push(page);
        } else {
            // If pool is full, close the page
            let _ = page.close().await;
        }
    }

    /// Get the number of available permits in the semaphore
    #[allow(dead_code)]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get the current number of pages in the pool
    #[allow(dead_code)]
    pub async fn pool_size(&self) -> usize {
        let pool = self.page_pool.lock().await;
        pool.len()
    }

    /// Cleanup all pages in the pool (useful for shutdown)
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        let mut pool = self.page_pool.lock().await;
        for page in pool.drain(..) {
            let _ = page.close().await;
        }
    }
}
