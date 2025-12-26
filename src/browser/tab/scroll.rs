//! Scroll control methods.

use tracing::debug;

use crate::error::Result;

use super::Tab;

// ============================================================================
// Tab - Scroll
// ============================================================================

impl Tab {
    /// Scrolls the page by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal scroll amount in pixels (positive = right)
    /// * `y` - Vertical scroll amount in pixels (positive = down)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Scroll down 500 pixels
    /// tab.scroll_by(0, 500).await?;
    ///
    /// // Scroll right 200 pixels
    /// tab.scroll_by(200, 0).await?;
    /// ```
    pub async fn scroll_by(&self, x: i32, y: i32) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, x = x, y = y, "Scrolling by");

        let script = format!("window.scrollBy({}, {});", x, y);
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Scrolls the page to the specified position.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in pixels from left
    /// * `y` - Vertical position in pixels from top
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Scroll to top of page
    /// tab.scroll_to(0, 0).await?;
    ///
    /// // Scroll to position (100, 500)
    /// tab.scroll_to(100, 500).await?;
    /// ```
    pub async fn scroll_to(&self, x: i32, y: i32) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, x = x, y = y, "Scrolling to");

        let script = format!("window.scrollTo({}, {});", x, y);
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Scrolls to the top of the page.
    pub async fn scroll_to_top(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Scrolling to top");
        self.scroll_to(0, 0).await
    }

    /// Scrolls to the bottom of the page.
    pub async fn scroll_to_bottom(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Scrolling to bottom");

        self.execute_script("window.scrollTo(0, document.body.scrollHeight);")
            .await?;
        Ok(())
    }

    /// Gets the current scroll position.
    ///
    /// # Returns
    ///
    /// Tuple of (x, y) scroll position in pixels.
    pub async fn get_scroll_position(&self) -> Result<(i32, i32)> {
        let result = self
            .execute_script("return { x: window.scrollX, y: window.scrollY };")
            .await?;

        let x = result.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = result.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        debug!(tab_id = %self.inner.tab_id, x = x, y = y, "Got scroll position");
        Ok((x, y))
    }

    /// Gets the page dimensions (scrollable area).
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) in pixels.
    pub async fn get_page_size(&self) -> Result<(i32, i32)> {
        let result = self
            .execute_script(
                r#"
                const body = document.body;
                const html = document.documentElement;
                return {
                    width: Math.max(body.scrollWidth, body.offsetWidth, html.clientWidth, html.scrollWidth, html.offsetWidth),
                    height: Math.max(body.scrollHeight, body.offsetHeight, html.clientHeight, html.scrollHeight, html.offsetHeight)
                };
                "#,
            )
            .await?;

        let width = result.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let height = result.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        debug!(tab_id = %self.inner.tab_id, width = width, height = height, "Got page size");
        Ok((width, height))
    }

    /// Gets the viewport dimensions.
    ///
    /// # Returns
    ///
    /// Tuple of (width, height) in pixels.
    pub async fn get_viewport_size(&self) -> Result<(i32, i32)> {
        let result = self
            .execute_script("return { width: window.innerWidth, height: window.innerHeight };")
            .await?;

        let width = result.get("width").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let height = result.get("height").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        debug!(tab_id = %self.inner.tab_id, width = width, height = height, "Got viewport size");
        Ok((width, height))
    }
}
