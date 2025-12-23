//! Firefox WebDriver - Undetectable browser automation library.
//!
//! This library provides a high-level API for automating Firefox browser
//! using a custom WebExtension-based architecture.
//!
//! # Architecture
//!
//! The driver follows a client-server model:
//!
//! - **Local End (Rust)**: Sends commands, receives events via WebSocket
//! - **Remote End (Extension)**: Executes commands in Firefox, emits events
//!
//! Key design principles:
//!
//! - Each [`Window`] owns: Firefox process + WebSocket connection + event loop
//! - Protocol uses `module.methodName` format (BiDi-inspired)
//! - Elements stored by reference in content script `Map` (undetectable)
//! - Event-driven architecture (no polling)
//!
//! # Quick Start
//!
//! ```no_run
//! use firefox_webdriver::{Driver, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Build driver with Firefox binary and extension paths
//!     let driver = Driver::builder()
//!         .binary("/path/to/firefox")
//!         .extension("/path/to/extension")
//!         .build()?;
//!
//!     // Spawn a headless browser window
//!     let window = driver.window().headless().spawn().await?;
//!     let tab = window.tab();
//!
//!     // Navigate and interact
//!     tab.goto("https://example.com").await?;
//!     let title = tab.get_title().await?;
//!     println!("Page title: {}", title);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`browser`] | Browser entities: [`Window`], [`Tab`], [`Element`] |
//! | [`driver`] | Driver factory and configuration |
//! | [`error`] | Error types and [`Result`] alias |
//! | [`identifiers`] | Type-safe ID wrappers |
//! | [`protocol`] | WebSocket message types (internal) |
//! | [`transport`] | WebSocket transport layer (internal) |
//!
//! # Features
//!
//! - **Undetectable**: No `navigator.webdriver` flag, no detectable globals
//! - **Event-driven**: DOM mutations, network events push to client
//! - **CSP bypass**: Script execution via `browser.scripting` API
//! - **Parallel automation**: 300+ concurrent windows supported

// ============================================================================
// Modules
// ============================================================================

/// Browser entities: Window, Tab, Element.
///
/// This module contains the core types for browser automation:
///
/// - [`Window`] - Browser window (owns Firefox process)
/// - [`Tab`] - Browser tab with frame context
/// - [`Element`] - DOM element reference
pub mod browser;

/// Driver factory and configuration.
///
/// Use [`Driver::builder()`] to create a configured driver instance.
pub mod driver;

/// Error types and result aliases.
///
/// All fallible operations return [`Result<T>`] which uses [`Error`].
pub mod error;

/// Type-safe identifiers for browser entities.
///
/// Newtype wrappers prevent mixing incompatible IDs at compile time.
pub mod identifiers;

/// WebSocket protocol message types.
///
/// Internal module defining command/response/event structures.
pub mod protocol;

/// WebSocket transport layer.
///
/// Internal module handling WebSocket server and connection management.
pub mod transport;

// ============================================================================
// Re-exports
// ============================================================================

// Browser types
pub use browser::{
    BodyAction, Cookie, Element, FrameInfo, HeadersAction, InterceptedRequest,
    InterceptedRequestBody, InterceptedRequestHeaders, InterceptedResponse,
    InterceptedResponseBody, ProxyConfig, ProxyType, RequestAction, RequestBody, ResponseAction,
    Tab, Window,
};

// Driver types
pub use driver::{Driver, DriverBuilder, ExtensionSource, FirefoxOptions, Profile};

// Error types
pub use error::{Error, Result};

// Identifier types
pub use identifiers::{
    ElementId, FrameId, InterceptId, RequestId, ScriptId, SessionId, SubscriptionId, TabId,
};
