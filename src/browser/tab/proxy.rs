//! Tab-level proxy configuration.

use tracing::debug;

use crate::browser::proxy::ProxyConfig;
use crate::error::Result;
use crate::protocol::{Command, ProxyCommand};

use super::Tab;

// ============================================================================
// Tab - Proxy
// ============================================================================

impl Tab {
    /// Sets a proxy for this tab.
    ///
    /// Tab-level proxy overrides window-level proxy for this tab only.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::ProxyConfig;
    ///
    /// tab.set_proxy(ProxyConfig::http("proxy.example.com", 8080)).await?;
    /// ```
    pub async fn set_proxy(&self, config: ProxyConfig) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, proxy_type = %config.proxy_type.as_str(), "Setting proxy");

        let command = Command::Proxy(ProxyCommand::SetTabProxy {
            proxy_type: config.proxy_type.as_str().to_string(),
            host: config.host,
            port: config.port,
            username: config.username,
            password: config.password,
            proxy_dns: config.proxy_dns,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Clears the proxy for this tab.
    pub async fn clear_proxy(&self) -> Result<()> {
        let command = Command::Proxy(ProxyCommand::ClearTabProxy);
        self.send_command(command).await?;
        Ok(())
    }
}
