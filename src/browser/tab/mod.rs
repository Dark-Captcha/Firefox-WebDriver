//! Browser tab automation and control.
//!
//! Each [`Tab`] represents a browser tab with a specific frame context.
//!
//! # Module Structure
//!
//! | Module | Description |
//! |--------|-------------|
//! | `core` | Tab struct and accessors |
//! | `navigation` | URL navigation, history |
//! | `frames` | Frame switching |
//! | `script` | JavaScript execution |
//! | `elements` | Element search and observation |
//! | `network` | Request interception, blocking |
//! | `storage` | Cookies, localStorage, sessionStorage |
//! | `proxy` | Tab-level proxy |
//! | `screenshot` | Page and element screenshots |
//! | `scroll` | Scroll control |
//!
//! # Example
//!
//! ```ignore
//! let tab = window.tab();
//!
//! // Navigate
//! tab.goto("https://example.com").await?;
//!
//! // Find elements
//! let button = tab.find_element("#submit").await?;
//! button.click().await?;
//!
//! // Screenshot
//! let png = tab.screenshot().png().capture().await?;
//! tab.screenshot().jpeg(80).save("page.jpg").await?;
//!
//! // Scroll
//! tab.scroll_by(0, 500).await?;
//! ```

// ============================================================================
// Submodules
// ============================================================================

mod core;
mod elements;
mod frames;
mod navigation;
mod network;
mod proxy;
mod screenshot;
mod script;
mod scroll;
mod storage;

// ============================================================================
// Re-exports
// ============================================================================

pub use core::{FrameInfo, Tab};
pub use screenshot::{ImageFormat, ScreenshotBuilder};
