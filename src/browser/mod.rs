//! Browser entities module.
//!
//! This module provides the core browser automation types:
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Window`] | Browser window (owns Firefox process + WebSocket) |
//! | [`Tab`] | Browser tab (frame context) |
//! | [`Element`] | DOM element reference |
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
//!     .build()?;
//!
//! let window = driver.window().headless().spawn().await?;
//! let tab = window.tab();
//!
//! tab.goto("https://example.com").await?;
//! let element = tab.find_element("h1").await?;
//! let text = element.get_text().await?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Submodules
// ============================================================================

/// DOM element interaction.
pub mod element;

/// Network interception types.
pub mod network;

/// Proxy configuration types.
pub mod proxy;

/// Browser tab automation.
pub mod tab;

/// Browser window management.
pub mod window;

// ============================================================================
// Re-exports
// ============================================================================

pub use element::Element;
pub use network::{
    BodyAction, HeadersAction, InterceptedRequest, InterceptedRequestBody,
    InterceptedRequestHeaders, InterceptedResponse, InterceptedResponseBody, RequestAction,
    RequestBody, ResponseAction,
};
pub use proxy::{ProxyConfig, ProxyType};
pub use tab::{FrameInfo, Tab};
pub use window::{Window, WindowBuilder};

// Re-export Cookie from protocol for convenience
pub use crate::protocol::Cookie;
