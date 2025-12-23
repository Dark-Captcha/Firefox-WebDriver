//! Browser window management and control.
//!
//! Each [`Window`] owns:
//! - One Firefox process (child process)
//! - One WebSocket connection (unique port)
//! - One profile directory (temporary or persistent)
//!
//! # Example
//!
//! ```no_run
//! use firefox_webdriver::Driver;
//!
//! # async fn example() -> firefox_webdriver::Result<()> {
//! let driver = Driver::builder()
//!     .binary("/usr/bin/firefox")
//!     .extension("./extension")
//!     .build()?;
//!
//! let window = driver.window()
//!     .headless()
//!     .window_size(1920, 1080)
//!     .spawn()
//!     .await?;
//!
//! let tab = window.tab();
//! tab.goto("https://example.com").await?;
//!
//! window.close().await?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use serde_json::Value;
use tokio::process::Child;
use tracing::{debug, info};
use uuid::Uuid;

use crate::driver::{Driver, FirefoxOptions, Profile};
use crate::error::{Error, Result};
use crate::identifiers::{FrameId, SessionId, TabId};
use crate::protocol::{
    BrowsingContextCommand, Command, ProxyCommand, Request, Response, SessionCommand,
};
use crate::transport::Connection;

use super::Tab;
use super::proxy::ProxyConfig;

// ============================================================================
// ProcessGuard
// ============================================================================

/// Guards a child process and ensures it is killed when dropped.
struct ProcessGuard {
    /// The child process handle.
    child: Option<Child>,
    /// Process ID for logging.
    pid: u32,
}

impl ProcessGuard {
    /// Creates a new process guard.
    fn new(child: Child) -> Self {
        let pid = child.id().unwrap_or(0);
        debug!(pid, "Process guard created");
        Self {
            child: Some(child),
            pid,
        }
    }

    /// Kills the process and waits for it to exit.
    async fn kill(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            debug!(pid = self.pid, "Killing Firefox process");
            if let Err(e) = child.kill().await {
                debug!(pid = self.pid, error = %e, "Failed to kill process");
            }
            if let Err(e) = child.wait().await {
                debug!(pid = self.pid, error = %e, "Failed to wait for process");
            }
            info!(pid = self.pid, "Process terminated");
        }
        Ok(())
    }

    /// Returns the process ID.
    #[inline]
    fn pid(&self) -> u32 {
        self.pid
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take()
            && let Err(e) = child.start_kill()
        {
            debug!(pid = self.pid, error = %e, "Failed to send kill signal in Drop");
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Internal shared state for a window.
pub(crate) struct WindowInner {
    /// Unique identifier for this window.
    pub uuid: Uuid,
    /// Session ID.
    pub session_id: SessionId,
    /// Protected process handle.
    process: Mutex<ProcessGuard>,
    /// WebSocket connection.
    pub connection: Connection,
    /// Profile directory.
    #[allow(dead_code)]
    profile: Profile,
    /// WebSocket port number.
    pub port: u16,
    /// All tabs in this window.
    tabs: Mutex<FxHashMap<TabId, Tab>>,
    /// The initial tab created when Firefox opens.
    pub initial_tab_id: TabId,
}

// ============================================================================
// Window
// ============================================================================

/// A handle to a Firefox browser window.
///
/// The window owns a Firefox process, WebSocket connection, and profile.
/// When dropped, the process is automatically killed.
///
/// # Example
///
/// ```no_run
/// # use firefox_webdriver::Driver;
/// # async fn example() -> firefox_webdriver::Result<()> {
/// # let driver = Driver::builder().binary("/usr/bin/firefox").extension("./ext").build()?;
/// let window = driver.window().headless().spawn().await?;
///
/// // Get the initial tab
/// let tab = window.tab();
///
/// // Create a new tab
/// let new_tab = window.new_tab().await?;
///
/// // Close the window
/// window.close().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Window {
    /// Shared inner state.
    pub(crate) inner: Arc<WindowInner>,
}

// ============================================================================
// Window - Display
// ============================================================================

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("uuid", &self.inner.uuid)
            .field("session_id", &self.inner.session_id)
            .field("port", &self.inner.port)
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Window - Constructor
// ============================================================================

impl Window {
    /// Creates a new window handle.
    pub(crate) fn new(
        connection: Connection,
        process: Child,
        profile: Profile,
        port: u16,
        session_id: SessionId,
        initial_tab_id: TabId,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let initial_tab = Tab::new(initial_tab_id, FrameId::main(), session_id, None);
        let mut tabs = FxHashMap::default();
        tabs.insert(initial_tab_id, initial_tab);

        debug!(uuid = %uuid, session_id = %session_id, tab_id = %initial_tab_id, port, "Window created");

        Self {
            inner: Arc::new(WindowInner {
                uuid,
                session_id,
                process: Mutex::new(ProcessGuard::new(process)),
                connection,
                profile,
                port,
                tabs: Mutex::new(tabs),
                initial_tab_id,
            }),
        }
    }
}

// ============================================================================
// Window - Accessors
// ============================================================================

impl Window {
    /// Returns the session ID.
    #[inline]
    #[must_use]
    pub fn session_id(&self) -> SessionId {
        self.inner.session_id
    }

    /// Returns the Rust-side unique UUID.
    #[inline]
    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.inner.uuid
    }

    /// Returns the WebSocket port for this window.
    #[inline]
    #[must_use]
    pub fn port(&self) -> u16 {
        self.inner.port
    }

    /// Returns the Firefox process ID.
    #[inline]
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.inner.process.lock().pid()
    }
}

// ============================================================================
// Window - Lifecycle
// ============================================================================

impl Window {
    /// Closes the window and kills the Firefox process.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be killed.
    #[allow(clippy::await_holding_lock)]
    pub async fn close(&self) -> Result<()> {
        debug!(uuid = %self.inner.uuid, "Closing window");
        self.inner.connection.shutdown();
        let mut guard = self.inner.process.lock();
        guard.kill().await?;
        info!(uuid = %self.inner.uuid, "Window closed");
        Ok(())
    }
}

// ============================================================================
// Window - Tab Management
// ============================================================================

impl Window {
    /// Returns the initial tab for this window.
    #[must_use]
    pub fn tab(&self) -> Tab {
        Tab::new(
            self.inner.initial_tab_id,
            FrameId::main(),
            self.inner.session_id,
            Some(self.clone()),
        )
    }

    /// Creates a new tab in this window.
    ///
    /// # Errors
    ///
    /// Returns an error if tab creation fails.
    pub async fn new_tab(&self) -> Result<Tab> {
        let command = Command::BrowsingContext(BrowsingContextCommand::NewTab);
        let response = self.send_command(command).await?;

        let tab_id_u32 = response
            .result
            .as_ref()
            .and_then(|v| v.get("tabId"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| Error::protocol("Expected tabId in NewTab response"))?;

        let new_tab_id = TabId::new(tab_id_u32 as u32)
            .ok_or_else(|| Error::protocol("Invalid tabId in NewTab response"))?;

        let tab = Tab::new(
            new_tab_id,
            FrameId::main(),
            self.inner.session_id,
            Some(self.clone()),
        );

        self.inner.tabs.lock().insert(new_tab_id, tab.clone());
        debug!(session_id = %self.inner.session_id, tab_id = %new_tab_id, "New tab created");
        Ok(tab)
    }

    /// Returns the number of tabs in this window.
    #[inline]
    #[must_use]
    pub fn tab_count(&self) -> usize {
        self.inner.tabs.lock().len()
    }

    /// Steals logs from extension (returns and clears).
    ///
    /// Useful for debugging extension issues.
    pub async fn steal_logs(&self) -> Result<Vec<Value>> {
        let command = Command::Session(SessionCommand::StealLogs);
        let response = self.send_command(command).await?;
        let logs = response
            .result
            .as_ref()
            .and_then(|v| v.get("logs"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(logs)
    }
}

// ============================================================================
// Window - Proxy
// ============================================================================

impl Window {
    /// Sets a proxy for all tabs in this window.
    ///
    /// Window-level proxy applies to all tabs unless overridden by tab-level proxy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::ProxyConfig;
    ///
    /// // HTTP proxy for all tabs
    /// window.set_proxy(ProxyConfig::http("proxy.example.com", 8080)).await?;
    ///
    /// // SOCKS5 proxy with auth
    /// window.set_proxy(
    ///     ProxyConfig::socks5("proxy.example.com", 1080)
    ///         .with_credentials("user", "pass")
    ///         .with_proxy_dns(true)
    /// ).await?;
    /// ```
    pub async fn set_proxy(&self, config: ProxyConfig) -> Result<()> {
        debug!(
            session_id = %self.inner.session_id,
            proxy_type = %config.proxy_type.as_str(),
            host = %config.host,
            port = config.port,
            "Setting window proxy"
        );

        let command = Command::Proxy(ProxyCommand::SetWindowProxy {
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

    /// Clears the proxy for this window.
    ///
    /// After clearing, all tabs use direct connection (unless they have tab-level proxy).
    pub async fn clear_proxy(&self) -> Result<()> {
        debug!(session_id = %self.inner.session_id, "Clearing window proxy");
        let command = Command::Proxy(ProxyCommand::ClearWindowProxy);
        self.send_command(command).await?;
        Ok(())
    }
}

// ============================================================================
// Window - Internal
// ============================================================================

impl Window {
    /// Sends a command via WebSocket and waits for the response.
    pub(crate) async fn send_command(&self, command: Command) -> Result<Response> {
        let request = Request::new(self.inner.initial_tab_id, FrameId::main(), command);
        self.inner.connection.send(request).await
    }
}

// ============================================================================
// WindowBuilder
// ============================================================================

/// Builder for spawning browser windows.
///
/// # Example
///
/// ```no_run
/// # use firefox_webdriver::Driver;
/// # async fn example() -> firefox_webdriver::Result<()> {
/// # let driver = Driver::builder().binary("/usr/bin/firefox").extension("./ext").build()?;
/// let window = driver.window()
///     .headless()
///     .window_size(1920, 1080)
///     .profile("./my_profile")
///     .spawn()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct WindowBuilder<'a> {
    /// Reference to the driver.
    driver: &'a Driver,
    /// Firefox launch options.
    options: FirefoxOptions,
    /// Optional custom profile path.
    profile: Option<PathBuf>,
}

// ============================================================================
// WindowBuilder - Implementation
// ============================================================================

impl<'a> WindowBuilder<'a> {
    /// Creates a new window builder.
    pub(crate) fn new(driver: &'a Driver) -> Self {
        Self {
            driver,
            options: FirefoxOptions::new(),
            profile: None,
        }
    }

    /// Enables headless mode.
    ///
    /// Firefox runs without a visible window.
    #[must_use]
    pub fn headless(mut self) -> Self {
        self.options = self.options.with_headless();
        self
    }

    /// Sets the window size.
    ///
    /// # Arguments
    ///
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    #[must_use]
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.options = self.options.with_window_size(width, height);
        self
    }

    /// Uses a custom profile directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to profile directory
    #[must_use]
    pub fn profile(mut self, path: impl Into<PathBuf>) -> Self {
        self.profile = Some(path.into());
        self
    }

    /// Spawns the window.
    ///
    /// # Errors
    ///
    /// Returns an error if window creation fails.
    pub async fn spawn(self) -> Result<Window> {
        self.driver.spawn_window(self.options, self.profile).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::Window;

    #[test]
    fn test_window_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<Window>();
    }

    #[test]
    fn test_window_is_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<Window>();
    }
}
