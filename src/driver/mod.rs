//! Firefox WebDriver driver module.
//!
//! This module provides the main entry point for browser automation.
//!
//! # Components
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Driver`] | Factory for creating browser windows |
//! | [`DriverBuilder`] | Fluent configuration builder |
//! | [`FirefoxOptions`] | Browser launch options |
//! | [`Profile`] | Firefox profile management |
//! | [`ExtensionSource`] | Extension installation source |
//!
//! # Example
//!
//! ```no_run
//! use firefox_webdriver::{Driver, Result};
//!
//! # async fn example() -> Result<()> {
//! let driver = Driver::builder()
//!     .binary("/usr/bin/firefox")
//!     .extension("./extension")
//!     .build()
//!     .await?;
//!
//! let window = driver.window().headless().spawn().await?;
//! let tab = window.tab();
//!
//! tab.goto("https://example.com").await?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Submodules
// ============================================================================

/// Static assets and HTML templates for driver initialization.
pub mod assets;

/// Fluent builder pattern for driver configuration.
pub mod builder;

/// Core driver implementation.
pub mod core;

/// Firefox browser options and preferences.
pub mod options;

/// Firefox profile management.
pub mod profile;

// ============================================================================
// Re-exports
// ============================================================================

pub use builder::DriverBuilder;
pub use core::Driver;
pub use options::FirefoxOptions;
pub use profile::{ExtensionSource, Profile};
