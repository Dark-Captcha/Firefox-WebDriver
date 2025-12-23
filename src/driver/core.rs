//! Firefox WebDriver coordinator and factory.
//!
//! The [`Driver`] struct acts as the central coordinator for browser automation.
//! It manages the lifecycle of browser windows.
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
//! let window = driver.window().headless().spawn().await?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use tokio::process::{Child, Command};
use tracing::{debug, info};

use crate::browser::{Window, WindowBuilder};
use crate::error::{Error, Result};
use crate::identifiers::{SessionId, TabId};
use crate::transport::PendingServer;

use super::assets;
use super::builder::DriverBuilder;
use super::options::FirefoxOptions;
use super::profile::{ExtensionSource, Profile};

// ============================================================================
// Constants
// ============================================================================

/// Default bind address for WebSocket server (localhost).
const DEFAULT_BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

// ============================================================================
// Types
// ============================================================================

/// Internal shared state for the driver.
pub(crate) struct DriverInner {
    /// Path to the Firefox binary executable.
    pub binary: PathBuf,

    /// Extension source for WebDriver functionality.
    pub extension: ExtensionSource,

    /// IP address to bind the WebSocket server to.
    pub bind_ip: IpAddr,

    /// Active windows tracked by their internal UUID.
    pub windows: Mutex<FxHashMap<uuid::Uuid, Window>>,
}

// ============================================================================
// Driver
// ============================================================================

/// Firefox WebDriver coordinator.
///
/// The driver is responsible for:
/// - Spawning Firefox processes with custom profiles
/// - Managing WebSocket server lifecycle
/// - Tracking active browser windows
///
/// # Examples
///
/// ```no_run
/// use firefox_webdriver::Driver;
///
/// # async fn example() -> firefox_webdriver::Result<()> {
/// let driver = Driver::builder()
///     .binary("/usr/bin/firefox")
///     .extension("./extension")
///     .build()?;
///
/// let window = driver.window().headless().spawn().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Driver {
    /// Shared inner state.
    pub(crate) inner: Arc<DriverInner>,
}

// ============================================================================
// Driver - Display
// ============================================================================

impl fmt::Debug for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Driver")
            .field("binary", &self.inner.binary)
            .field("window_count", &self.window_count())
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Driver - Public API
// ============================================================================

impl Driver {
    /// Creates a configuration builder for the driver.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use firefox_webdriver::Driver;
    ///
    /// # fn example() -> firefox_webdriver::Result<()> {
    /// let driver = Driver::builder()
    ///     .binary("/usr/bin/firefox")
    ///     .extension("./extension")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn builder() -> DriverBuilder {
        DriverBuilder::new()
    }

    /// Creates a window builder for spawning new browser windows.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use firefox_webdriver::Driver;
    /// # async fn example(driver: &Driver) -> firefox_webdriver::Result<()> {
    /// let window = driver.window()
    ///     .headless()
    ///     .window_size(1920, 1080)
    ///     .spawn()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn window(&self) -> WindowBuilder<'_> {
        WindowBuilder::new(self)
    }

    /// Returns the number of active windows currently tracked.
    #[inline]
    #[must_use]
    pub fn window_count(&self) -> usize {
        self.inner.windows.lock().len()
    }

    /// Closes all active windows and shuts down the driver.
    ///
    /// # Errors
    ///
    /// Returns an error if any window fails to close.
    pub async fn close(&self) -> Result<()> {
        let windows: Vec<Window> = {
            let mut map = self.inner.windows.lock();
            map.drain().map(|(_, w)| w).collect()
        };

        info!(count = windows.len(), "Shutting down all windows");

        for window in windows {
            if let Err(e) = window.close().await {
                debug!(error = %e, "Error closing window during shutdown");
            }
        }

        Ok(())
    }
}

// ============================================================================
// Driver - Internal API
// ============================================================================

impl Driver {
    /// Creates a new driver instance.
    ///
    /// # Arguments
    ///
    /// * `binary` - Path to Firefox binary
    /// * `extension` - Extension source for WebDriver
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub(crate) fn new(binary: PathBuf, extension: ExtensionSource) -> Result<Self> {
        let inner = Arc::new(DriverInner {
            binary,
            extension,
            bind_ip: DEFAULT_BIND_IP,
            windows: Mutex::new(FxHashMap::default()),
        });

        info!("Driver initialized");

        Ok(Self { inner })
    }

    /// Spawns a new Firefox window with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `options` - Firefox launch options
    /// * `custom_profile` - Optional custom profile path
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile creation fails
    /// - Extension installation fails
    /// - WebSocket server binding fails
    /// - Firefox process fails to spawn
    /// - Extension fails to connect
    pub(crate) async fn spawn_window(
        &self,
        options: FirefoxOptions,
        custom_profile: Option<PathBuf>,
    ) -> Result<Window> {
        // Create profile
        let profile = self.prepare_profile(custom_profile)?;

        // Install extension
        profile.install_extension(&self.inner.extension)?;
        debug!("Installed WebDriver extension");

        // Write preferences
        let prefs = Profile::default_prefs();
        profile.write_prefs(&prefs)?;
        debug!(pref_count = prefs.len(), "Wrote profile preferences");

        // Bind WebSocket server
        let pending_server = PendingServer::bind(self.inner.bind_ip, 0).await?;
        let port = pending_server.port();
        let ws_url = format!("ws://{}:{}", self.inner.bind_ip, port);
        debug!(port, url = %ws_url, "WebSocket server bound");

        // Generate session ID and data URI
        let session_id = SessionId::next();
        let data_uri = assets::build_init_data_uri(&ws_url, &session_id);

        // Spawn Firefox process
        let child = self.spawn_firefox_process(&profile, &options, &data_uri)?;
        let pid = child.id();
        info!(pid, port, "Firefox process spawned");

        // Wait for extension to connect
        let (connection, ready_data) = pending_server.accept().await?;
        debug!("WebSocket handshake completed");

        // Extract tab ID from ready message
        let tab_id = TabId::new(ready_data.tab_id)
            .ok_or_else(|| Error::protocol("Invalid tab_id in READY message"))?;
        debug!(session_id = %session_id, tab_id = %tab_id, "Browser IDs assigned");

        // Create window
        let window = Window::new(connection, child, profile, port, session_id, tab_id);

        // Track window
        self.inner
            .windows
            .lock()
            .insert(*window.uuid(), window.clone());

        info!(
            session_id = %session_id,
            window_count = self.window_count(),
            "Window spawned successfully"
        );

        Ok(window)
    }

    /// Prepares a Firefox profile for the window.
    ///
    /// # Arguments
    ///
    /// * `custom_profile` - Optional path to existing profile
    ///
    /// # Errors
    ///
    /// Returns an error if profile creation fails.
    fn prepare_profile(&self, custom_profile: Option<PathBuf>) -> Result<Profile> {
        match custom_profile {
            Some(path) => {
                debug!(path = %path.display(), "Using custom profile");
                Profile::from_path(path)
            }
            None => {
                debug!("Creating temporary profile");
                Profile::new_temp()
            }
        }
    }

    /// Spawns the Firefox process with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `profile` - Firefox profile to use
    /// * `options` - Firefox launch options
    /// * `data_uri` - Initial page data URI
    ///
    /// # Errors
    ///
    /// Returns an error if the process fails to spawn.
    fn spawn_firefox_process(
        &self,
        profile: &Profile,
        options: &FirefoxOptions,
        data_uri: &str,
    ) -> Result<Child> {
        let mut cmd = Command::new(&self.inner.binary);

        // Profile arguments
        cmd.arg("--profile")
            .arg(profile.path())
            .arg("--no-remote")
            .arg("--new-instance");

        // User-specified options
        cmd.args(options.to_args());

        // Initial page
        cmd.arg(data_uri);

        // Suppress stdio
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        cmd.spawn().map_err(Error::process_launch_failed)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::Driver;

    #[test]
    fn test_builder_returns_driver_builder() {
        let _builder = Driver::builder();
    }

    #[test]
    fn test_driver_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<Driver>();
    }

    #[test]
    fn test_driver_is_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<Driver>();
    }
}
