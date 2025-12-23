//! Network interception types.
//!
//! Types for request/response interception callbacks.
//!
//! # Request Interception
//!
//! ```ignore
//! use firefox_webdriver::RequestAction;
//!
//! let intercept_id = tab.intercept_request(|req| {
//!     if req.url.contains("ads") {
//!         RequestAction::block()
//!     } else {
//!         RequestAction::allow()
//!     }
//! }).await?;
//! ```
//!
//! # Response Interception
//!
//! ```ignore
//! use firefox_webdriver::BodyAction;
//!
//! let intercept_id = tab.intercept_response_body(|res| {
//!     if res.url.contains("config.json") {
//!         BodyAction::modify_body(r#"{"modified": true}"#)
//!     } else {
//!         BodyAction::allow()
//!     }
//! }).await?;
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::collections::HashMap;

// ============================================================================
// InterceptedRequest
// ============================================================================

/// Data about an intercepted network request.
#[derive(Debug, Clone)]
pub struct InterceptedRequest {
    /// Unique request ID.
    pub request_id: String,

    /// Request URL.
    pub url: String,

    /// HTTP method (GET, POST, etc.).
    pub method: String,

    /// Resource type (document, script, xhr, etc.).
    pub resource_type: String,

    /// Tab ID where request originated.
    pub tab_id: u32,

    /// Frame ID where request originated.
    pub frame_id: u64,

    /// Request body (if available).
    pub body: Option<RequestBody>,
}

// ============================================================================
// RequestBody
// ============================================================================

/// Request body data.
#[derive(Debug, Clone)]
pub enum RequestBody {
    /// Form data (application/x-www-form-urlencoded or multipart/form-data).
    FormData(HashMap<String, Vec<String>>),

    /// Raw bytes (base64 encoded).
    Raw(Vec<u8>),

    /// Error reading body.
    Error(String),
}

// ============================================================================
// RequestAction
// ============================================================================

/// Action to take for an intercepted request.
#[derive(Debug, Clone)]
pub enum RequestAction {
    /// Allow the request to proceed.
    Allow,

    /// Block/cancel the request.
    Block,

    /// Redirect to a different URL.
    Redirect(String),
}

// ============================================================================
// RequestAction - Constructors
// ============================================================================

impl RequestAction {
    /// Creates an Allow action.
    #[inline]
    #[must_use]
    pub fn allow() -> Self {
        Self::Allow
    }

    /// Creates a Block action.
    #[inline]
    #[must_use]
    pub fn block() -> Self {
        Self::Block
    }

    /// Creates a Redirect action.
    #[inline]
    #[must_use]
    pub fn redirect(url: impl Into<String>) -> Self {
        Self::Redirect(url.into())
    }
}

// ============================================================================
// InterceptedRequestBody
// ============================================================================

/// Data about an intercepted request body (read-only, cannot be modified).
///
/// Browser limitation: request body cannot be modified, only inspected.
#[derive(Debug, Clone)]
pub struct InterceptedRequestBody {
    /// Unique request ID.
    pub request_id: String,

    /// Request URL.
    pub url: String,

    /// HTTP method.
    pub method: String,

    /// Resource type (document, script, xhr, etc.).
    pub resource_type: String,

    /// Tab ID.
    pub tab_id: u32,

    /// Frame ID.
    pub frame_id: u64,

    /// Request body (if available).
    pub body: Option<RequestBody>,
}

// ============================================================================
// InterceptedRequestHeaders
// ============================================================================

/// Data about intercepted request headers.
#[derive(Debug, Clone)]
pub struct InterceptedRequestHeaders {
    /// Unique request ID.
    pub request_id: String,

    /// Request URL.
    pub url: String,

    /// HTTP method.
    pub method: String,

    /// Request headers.
    pub headers: HashMap<String, String>,

    /// Tab ID.
    pub tab_id: u32,

    /// Frame ID.
    pub frame_id: u64,
}

// ============================================================================
// HeadersAction
// ============================================================================

/// Action to take for intercepted headers.
#[derive(Debug, Clone)]
pub enum HeadersAction {
    /// Allow headers to proceed unchanged.
    Allow,

    /// Modify headers.
    ModifyHeaders(HashMap<String, String>),
}

// ============================================================================
// HeadersAction - Constructors
// ============================================================================

impl HeadersAction {
    /// Creates an Allow action.
    #[inline]
    #[must_use]
    pub fn allow() -> Self {
        Self::Allow
    }

    /// Creates a ModifyHeaders action.
    #[inline]
    #[must_use]
    pub fn modify_headers(headers: HashMap<String, String>) -> Self {
        Self::ModifyHeaders(headers)
    }
}

// ============================================================================
// InterceptedResponse
// ============================================================================

/// Data about an intercepted network response.
#[derive(Debug, Clone)]
pub struct InterceptedResponse {
    /// Unique request ID.
    pub request_id: String,

    /// Request URL.
    pub url: String,

    /// HTTP status code.
    pub status: u16,

    /// HTTP status text.
    pub status_text: String,

    /// Response headers.
    pub headers: HashMap<String, String>,

    /// Tab ID where request originated.
    pub tab_id: u32,

    /// Frame ID where request originated.
    pub frame_id: u64,
}

// ============================================================================
// ResponseAction
// ============================================================================

/// Action to take for intercepted response headers.
///
/// Alias for [`HeadersAction`].
pub type ResponseAction = HeadersAction;

// ============================================================================
// InterceptedResponseBody
// ============================================================================

/// Data about an intercepted response body.
#[derive(Debug, Clone)]
pub struct InterceptedResponseBody {
    /// Unique request ID.
    pub request_id: String,

    /// Request URL.
    pub url: String,

    /// Tab ID.
    pub tab_id: u32,

    /// Frame ID.
    pub frame_id: u64,

    /// Response body as string.
    pub body: String,

    /// Content length.
    pub content_length: usize,
}

// ============================================================================
// BodyAction
// ============================================================================

/// Action to take for intercepted response body.
#[derive(Debug, Clone)]
pub enum BodyAction {
    /// Allow body to proceed unchanged.
    Allow,

    /// Modify body content.
    ModifyBody(String),
}

// ============================================================================
// BodyAction - Constructors
// ============================================================================

impl BodyAction {
    /// Creates an Allow action.
    #[inline]
    #[must_use]
    pub fn allow() -> Self {
        Self::Allow
    }

    /// Creates a ModifyBody action.
    #[inline]
    #[must_use]
    pub fn modify_body(body: impl Into<String>) -> Self {
        Self::ModifyBody(body.into())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{BodyAction, HeadersAction, RequestAction};

    use std::collections::HashMap;

    #[test]
    fn test_request_action_allow() {
        let action = RequestAction::allow();
        assert!(matches!(action, RequestAction::Allow));
    }

    #[test]
    fn test_request_action_block() {
        let action = RequestAction::block();
        assert!(matches!(action, RequestAction::Block));
    }

    #[test]
    fn test_request_action_redirect() {
        let action = RequestAction::redirect("https://example.com");
        if let RequestAction::Redirect(url) = action {
            assert_eq!(url, "https://example.com");
        } else {
            panic!("Expected Redirect action");
        }
    }

    #[test]
    fn test_headers_action_allow() {
        let action = HeadersAction::allow();
        assert!(matches!(action, HeadersAction::Allow));
    }

    #[test]
    fn test_headers_action_modify() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());

        let action = HeadersAction::modify_headers(headers);
        if let HeadersAction::ModifyHeaders(h) = action {
            assert_eq!(h.get("X-Custom"), Some(&"value".to_string()));
        } else {
            panic!("Expected ModifyHeaders action");
        }
    }

    #[test]
    fn test_body_action_allow() {
        let action = BodyAction::allow();
        assert!(matches!(action, BodyAction::Allow));
    }

    #[test]
    fn test_body_action_modify() {
        let action = BodyAction::modify_body("new body");
        if let BodyAction::ModifyBody(body) = action {
            assert_eq!(body, "new body");
        } else {
            panic!("Expected ModifyBody action");
        }
    }
}
