//! Tab navigation methods.

use tracing::debug;

use crate::error::Result;
use crate::protocol::{BrowsingContextCommand, Command};

use super::Tab;

// ============================================================================
// Tab - Navigation
// ============================================================================

impl Tab {
    /// Navigates to a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to navigate to
    ///
    /// # Errors
    ///
    /// Returns an error if navigation fails.
    pub async fn goto(&self, url: &str) -> Result<()> {
        debug!(url = %url, tab_id = %self.inner.tab_id, "Navigating");

        let command = Command::BrowsingContext(BrowsingContextCommand::Navigate {
            url: url.to_string(),
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Loads HTML content directly into the page.
    ///
    /// Useful for testing with inline HTML without needing a server.
    ///
    /// # Arguments
    ///
    /// * `html` - HTML content to load
    ///
    /// # Example
    ///
    /// ```ignore
    /// tab.load_html("<html><body><h1>Test</h1></body></html>").await?;
    /// ```
    pub async fn load_html(&self, html: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, html_len = html.len(), "Loading HTML content");

        let escaped_html = html
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace("${", "\\${");

        let script = format!(
            r#"(function() {{
                const html = `{}`;
                const parser = new DOMParser();
                const doc = parser.parseFromString(html, 'text/html');
                const newTitle = doc.querySelector('title');
                if (newTitle) {{ document.title = newTitle.textContent; }}
                const newBody = doc.body;
                if (newBody) {{
                    document.body.innerHTML = newBody.innerHTML;
                    for (const attr of newBody.attributes) {{
                        document.body.setAttribute(attr.name, attr.value);
                    }}
                }}
                const newHead = doc.head;
                if (newHead) {{
                    for (const child of newHead.children) {{
                        if (child.tagName !== 'TITLE') {{
                            document.head.appendChild(child.cloneNode(true));
                        }}
                    }}
                }}
            }})();"#,
            escaped_html
        );

        self.execute_script(&script).await?;
        Ok(())
    }

    /// Reloads the current page.
    pub async fn reload(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Reloading page");
        let command = Command::BrowsingContext(BrowsingContextCommand::Reload);
        self.send_command(command).await?;
        Ok(())
    }

    /// Navigates back in history.
    pub async fn back(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Navigating back");
        let command = Command::BrowsingContext(BrowsingContextCommand::GoBack);
        self.send_command(command).await?;
        Ok(())
    }

    /// Navigates forward in history.
    pub async fn forward(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Navigating forward");
        let command = Command::BrowsingContext(BrowsingContextCommand::GoForward);
        self.send_command(command).await?;
        Ok(())
    }

    /// Gets the current page title.
    pub async fn get_title(&self) -> Result<String> {
        debug!(tab_id = %self.inner.tab_id, "Getting page title");
        let command = Command::BrowsingContext(BrowsingContextCommand::GetTitle);
        let response = self.send_command(command).await?;

        let title = response
            .result
            .as_ref()
            .and_then(|v| v.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        debug!(tab_id = %self.inner.tab_id, title = %title, "Got page title");
        Ok(title)
    }

    /// Gets the current URL.
    pub async fn get_url(&self) -> Result<String> {
        debug!(tab_id = %self.inner.tab_id, "Getting page URL");
        let command = Command::BrowsingContext(BrowsingContextCommand::GetUrl);
        let response = self.send_command(command).await?;

        let url = response
            .result
            .as_ref()
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        debug!(tab_id = %self.inner.tab_id, url = %url, "Got page URL");
        Ok(url)
    }

    /// Focuses this tab (makes it active).
    pub async fn focus(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Focusing tab");
        let command = Command::BrowsingContext(BrowsingContextCommand::FocusTab);
        self.send_command(command).await?;
        Ok(())
    }

    /// Focuses the window containing this tab.
    pub async fn focus_window(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Focusing window");
        let command = Command::BrowsingContext(BrowsingContextCommand::FocusWindow);
        self.send_command(command).await?;
        Ok(())
    }

    /// Closes this tab.
    pub async fn close(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Closing tab");
        let command = Command::BrowsingContext(BrowsingContextCommand::CloseTab);
        self.send_command(command).await?;
        Ok(())
    }

    /// Gets the page source HTML.
    pub async fn get_page_source(&self) -> Result<String> {
        debug!(tab_id = %self.inner.tab_id, "Getting page source");
        let result = self
            .execute_script("return document.documentElement.outerHTML")
            .await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }
}
