//! Browser entities module.
//!
//! This module provides the core browser automation types:
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Window`] | Browser window (owns Firefox process, references shared pool) |
//! | [`Tab`] | Browser tab (frame context) |
//! | [`Element`] | DOM element reference |
//! | [`Key`] | Keyboard key constants |
//! | [`By`] | Element locator strategies |
//!
//! # Example
//!
//! ```no_run
//! use firefox_webdriver::{Driver, Result, Key, By};
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
//!
//! // Find with By selector
//! let element = tab.find_element(By::tag("h1")).await?;
//!
//! // Find by text
//! let btn = tab.find_element(By::text("Submit")).await?;
//!
//! // Press keys
//! element.press(Key::Enter).await?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Submodules
// ============================================================================

/// DOM element interaction.
pub mod element;

/// Keyboard key definitions.
pub mod keyboard;

/// Network interception types.
pub mod network;

/// Proxy configuration types.
pub mod proxy;

/// Element locator strategies.
pub mod selector;

/// Browser tab automation.
pub mod tab;

/// Browser window management.
pub mod window;

// ============================================================================
// Re-exports
// ============================================================================

pub use element::Element;
pub use keyboard::Key;
pub use network::{
    BodyAction, HeadersAction, InterceptedRequest, InterceptedRequestBody,
    InterceptedRequestHeaders, InterceptedResponse, InterceptedResponseBody, RequestAction,
    RequestBody, ResponseAction,
};
pub use proxy::{ProxyConfig, ProxyType};
pub use selector::By;
pub use tab::{FrameInfo, ImageFormat, ScreenshotBuilder, Tab};
pub use window::{Window, WindowBuilder};

// Re-export Cookie from protocol for convenience
pub use crate::protocol::Cookie;
