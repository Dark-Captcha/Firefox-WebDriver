//! Error types for Firefox WebDriver.
//!
//! This module defines all error types used throughout the crate.
//! Error codes follow ARCHITECTURE.md Section 6.2.
//!
//! # Usage
//!
//! All fallible operations return [`Result<T>`] which uses [`Error`]:
//!
//! ```ignore
//! use firefox_webdriver::{Result, Error};
//!
//! async fn example(tab: &Tab) -> Result<()> {
//!     let element = tab.find_element("#submit").await?;
//!     element.click().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Error Categories
//!
//! | Category | Variants |
//! |----------|----------|
//! | Configuration | [`Error::Config`], [`Error::Profile`] |
//! | Connection | [`Error::Connection`], [`Error::ConnectionTimeout`], [`Error::ConnectionClosed`] |
//! | Protocol | [`Error::UnknownCommand`], [`Error::InvalidArgument`], [`Error::Protocol`] |
//! | Element | [`Error::ElementNotFound`], [`Error::StaleElement`] |
//! | Navigation | [`Error::FrameNotFound`], [`Error::TabNotFound`] |
//! | Execution | [`Error::ScriptError`], [`Error::Timeout`], [`Error::RequestTimeout`] |
//! | External | [`Error::Io`], [`Error::Json`], [`Error::WebSocket`] |

// ============================================================================
// Imports
// ============================================================================

use std::io::Error as IoError;
use std::path::PathBuf;
use std::result::Result as StdResult;

use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;
use tokio_tungstenite::tungstenite::Error as WsError;

use crate::identifiers::{ElementId, FrameId, RequestId, TabId};

// ============================================================================
// Result Alias
// ============================================================================

/// Result type alias using crate [`enum@Error`].
///
/// All fallible operations in this crate return this type.
pub type Result<T> = StdResult<T, Error>;

// ============================================================================
// Error Enum
// ============================================================================

/// Main error type for the crate.
///
/// Each variant includes relevant context for debugging.
/// Error codes match ARCHITECTURE.md Section 6.2.
#[derive(Error, Debug)]
pub enum Error {
    // ========================================================================
    // Configuration Errors
    // ========================================================================
    /// Configuration error.
    ///
    /// Returned when driver configuration is invalid.
    #[error("Configuration error: {message}")]
    Config {
        /// Description of the configuration error.
        message: String,
    },

    /// Profile error.
    ///
    /// Returned when Firefox profile creation or setup fails.
    #[error("Profile error: {message}")]
    Profile {
        /// Description of the profile error.
        message: String,
    },

    /// Firefox binary not found at path.
    ///
    /// Returned when the specified Firefox binary does not exist.
    #[error("Firefox not found at: {path}")]
    FirefoxNotFound {
        /// Path where Firefox was expected.
        path: PathBuf,
    },

    /// Failed to launch Firefox process.
    ///
    /// Returned when Firefox process fails to start.
    #[error("Failed to launch Firefox: {message}")]
    ProcessLaunchFailed {
        /// Description of the launch failure.
        message: String,
    },

    // ========================================================================
    // Connection Errors
    // ========================================================================
    /// WebSocket connection failed.
    ///
    /// Returned when WebSocket connection cannot be established.
    #[error("Connection failed: {message}")]
    Connection {
        /// Description of the connection error.
        message: String,
    },

    /// Connection timeout waiting for extension.
    ///
    /// Returned when extension does not connect within timeout period.
    #[error("Connection timeout after {timeout_ms}ms")]
    ConnectionTimeout {
        /// Milliseconds waited before timeout.
        timeout_ms: u64,
    },

    /// WebSocket connection closed unexpectedly.
    ///
    /// Returned when connection is lost during operation.
    #[error("Connection closed")]
    ConnectionClosed,

    // ========================================================================
    // Protocol Errors
    // ========================================================================
    /// Unknown command method.
    ///
    /// Returned when extension receives unrecognized command.
    #[error("Unknown command: {command}")]
    UnknownCommand {
        /// The unrecognized command method.
        command: String,
    },

    /// Invalid argument in command params.
    ///
    /// Returned when command parameters are invalid.
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        /// Description of the invalid argument.
        message: String,
    },

    /// Protocol violation or unexpected response.
    ///
    /// Returned when protocol message format is invalid.
    #[error("Protocol error: {message}")]
    Protocol {
        /// Description of the protocol violation.
        message: String,
    },

    // ========================================================================
    // Element Errors
    // ========================================================================
    /// Element not found by selector.
    ///
    /// Returned when CSS selector matches no elements.
    #[error("Element not found: selector={selector}, tab={tab_id}, frame={frame_id}")]
    ElementNotFound {
        /// CSS selector used.
        selector: String,
        /// Tab where search was performed.
        tab_id: TabId,
        /// Frame where search was performed.
        frame_id: FrameId,
    },

    /// Element is stale (no longer in DOM).
    ///
    /// Returned when element reference is no longer valid.
    #[error("Stale element: {element_id}")]
    StaleElement {
        /// The stale element's ID.
        element_id: ElementId,
    },

    // ========================================================================
    // Navigation Errors
    // ========================================================================
    /// Frame not found.
    ///
    /// Returned when frame ID does not exist.
    #[error("Frame not found: {frame_id}")]
    FrameNotFound {
        /// The missing frame ID.
        frame_id: FrameId,
    },

    /// Tab not found.
    ///
    /// Returned when tab ID does not exist.
    #[error("Tab not found: {tab_id}")]
    TabNotFound {
        /// The missing tab ID.
        tab_id: TabId,
    },

    // ========================================================================
    // Execution Errors
    // ========================================================================
    /// JavaScript execution error.
    ///
    /// Returned when script execution fails in browser.
    #[error("Script error: {message}")]
    ScriptError {
        /// Error message from script execution.
        message: String,
    },

    /// Operation timeout.
    ///
    /// Returned when operation exceeds timeout duration.
    #[error("Timeout after {timeout_ms}ms: {operation}")]
    Timeout {
        /// Description of the operation that timed out.
        operation: String,
        /// Milliseconds waited before timeout.
        timeout_ms: u64,
    },

    /// Command request timeout.
    ///
    /// Returned when WebSocket request times out.
    #[error("Request {request_id} timed out after {timeout_ms}ms")]
    RequestTimeout {
        /// The request ID that timed out.
        request_id: RequestId,
        /// Milliseconds waited before timeout.
        timeout_ms: u64,
    },

    // ========================================================================
    // Network Errors
    // ========================================================================
    /// Network intercept not found.
    ///
    /// Returned when intercept ID does not exist.
    #[error("Intercept not found: {intercept_id}")]
    InterceptNotFound {
        /// The missing intercept ID.
        intercept_id: String,
    },

    // ========================================================================
    // External Errors
    // ========================================================================
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] IoError),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WsError),

    /// Channel receive error.
    #[error("Channel closed")]
    ChannelClosed(#[from] RecvError),
}

// ============================================================================
// Error Constructors
// ============================================================================

impl Error {
    /// Creates a configuration error.
    #[inline]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Creates a profile error.
    #[inline]
    pub fn profile(message: impl Into<String>) -> Self {
        Self::Profile {
            message: message.into(),
        }
    }

    /// Creates a Firefox not found error.
    #[inline]
    pub fn firefox_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FirefoxNotFound { path: path.into() }
    }

    /// Creates a process launch failed error.
    #[inline]
    pub fn process_launch_failed(err: IoError) -> Self {
        Self::ProcessLaunchFailed {
            message: err.to_string(),
        }
    }

    /// Creates a connection error.
    #[inline]
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Creates a connection timeout error.
    #[inline]
    pub fn connection_timeout(timeout_ms: u64) -> Self {
        Self::ConnectionTimeout { timeout_ms }
    }

    /// Creates a protocol error.
    #[inline]
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    /// Creates an invalid argument error.
    #[inline]
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument {
            message: message.into(),
        }
    }

    /// Creates an element not found error.
    #[inline]
    pub fn element_not_found(
        selector: impl Into<String>,
        tab_id: TabId,
        frame_id: FrameId,
    ) -> Self {
        Self::ElementNotFound {
            selector: selector.into(),
            tab_id,
            frame_id,
        }
    }

    /// Creates a stale element error.
    #[inline]
    pub fn stale_element(element_id: ElementId) -> Self {
        Self::StaleElement { element_id }
    }

    /// Creates a frame not found error.
    #[inline]
    pub fn frame_not_found(frame_id: FrameId) -> Self {
        Self::FrameNotFound { frame_id }
    }

    /// Creates a tab not found error.
    #[inline]
    pub fn tab_not_found(tab_id: TabId) -> Self {
        Self::TabNotFound { tab_id }
    }

    /// Creates a script error.
    #[inline]
    pub fn script_error(message: impl Into<String>) -> Self {
        Self::ScriptError {
            message: message.into(),
        }
    }

    /// Creates a timeout error.
    #[inline]
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Creates a request timeout error.
    #[inline]
    pub fn request_timeout(request_id: RequestId, timeout_ms: u64) -> Self {
        Self::RequestTimeout {
            request_id,
            timeout_ms,
        }
    }

    /// Creates an intercept not found error.
    #[inline]
    pub fn intercept_not_found(intercept_id: impl Into<String>) -> Self {
        Self::InterceptNotFound {
            intercept_id: intercept_id.into(),
        }
    }
}

// ============================================================================
// Error Predicates
// ============================================================================

impl Error {
    /// Returns `true` if this is a timeout error.
    #[inline]
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout { .. } | Self::Timeout { .. } | Self::RequestTimeout { .. }
        )
    }

    /// Returns `true` if this is an element error.
    #[inline]
    #[must_use]
    pub fn is_element_error(&self) -> bool {
        matches!(
            self,
            Self::ElementNotFound { .. } | Self::StaleElement { .. }
        )
    }

    /// Returns `true` if this is a connection error.
    #[inline]
    #[must_use]
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            Self::Connection { .. }
                | Self::ConnectionTimeout { .. }
                | Self::ConnectionClosed
                | Self::WebSocket(_)
        )
    }

    /// Returns `true` if this error is recoverable.
    ///
    /// Recoverable errors may succeed on retry.
    #[inline]
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout { .. }
                | Self::Timeout { .. }
                | Self::RequestTimeout { .. }
                | Self::StaleElement { .. }
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::ErrorKind;

    #[test]
    fn test_error_display() {
        let err = Error::connection("failed to connect");
        assert_eq!(err.to_string(), "Connection failed: failed to connect");
    }

    #[test]
    fn test_config_error() {
        let err = Error::config("missing binary path");
        assert_eq!(err.to_string(), "Configuration error: missing binary path");
    }

    #[test]
    fn test_is_timeout() {
        let timeout_err = Error::ConnectionTimeout { timeout_ms: 5000 };
        let other_err = Error::connection("test");

        assert!(timeout_err.is_timeout());
        assert!(!other_err.is_timeout());
    }

    #[test]
    fn test_is_connection_error() {
        let conn_err = Error::connection("test");
        let timeout_err = Error::ConnectionTimeout { timeout_ms: 1000 };
        let closed_err = Error::ConnectionClosed;
        let other_err = Error::config("test");

        assert!(conn_err.is_connection_error());
        assert!(timeout_err.is_connection_error());
        assert!(closed_err.is_connection_error());
        assert!(!other_err.is_connection_error());
    }

    #[test]
    fn test_is_recoverable() {
        let timeout_err = Error::Timeout {
            operation: "test".into(),
            timeout_ms: 1000,
        };
        let config_err = Error::config("test");

        assert!(timeout_err.is_recoverable());
        assert!(!config_err.is_recoverable());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Json(_)));
    }
}
