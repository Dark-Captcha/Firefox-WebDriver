//! Request and Response message types.
//!
//! Defines the message format for command requests and responses
//! between local end (Rust) and remote end (Extension).
//!
//! See ARCHITECTURE.md Section 2.2-2.3 for specification.

// ============================================================================
// Imports
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::identifiers::{FrameId, RequestId, TabId};

use super::Command;

// ============================================================================
// Request
// ============================================================================

/// A command request from local end to remote end.
///
/// # Format
///
/// ```json
/// {
///   "id": "uuid",
///   "method": "module.methodName",
///   "tabId": 1,
///   "frameId": 0,
///   "params": { ... }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Request {
    /// Unique identifier for request/response correlation.
    pub id: RequestId,

    /// Target tab ID.
    #[serde(rename = "tabId")]
    pub tab_id: TabId,

    /// Target frame ID (0 = main frame).
    #[serde(rename = "frameId")]
    pub frame_id: FrameId,

    /// Command with method and params.
    #[serde(flatten)]
    pub command: Command,
}

impl Request {
    /// Creates a new request with auto-generated ID.
    #[inline]
    #[must_use]
    pub fn new(tab_id: TabId, frame_id: FrameId, command: Command) -> Self {
        Self {
            id: RequestId::generate(),
            tab_id,
            frame_id,
            command,
        }
    }

    /// Creates a new request with specific ID.
    #[inline]
    #[must_use]
    pub fn with_id(id: RequestId, tab_id: TabId, frame_id: FrameId, command: Command) -> Self {
        Self {
            id,
            tab_id,
            frame_id,
            command,
        }
    }
}

// ============================================================================
// Response
// ============================================================================

/// A response from remote end to local end.
///
/// # Format
///
/// Success:
/// ```json
/// {
///   "id": "uuid",
///   "type": "success",
///   "result": { ... }
/// }
/// ```
///
/// Error:
/// ```json
/// {
///   "id": "uuid",
///   "type": "error",
///   "error": "error code",
///   "message": "error message"
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    /// Matches the command `id`.
    pub id: RequestId,

    /// Response type.
    #[serde(rename = "type")]
    pub response_type: ResponseType,

    /// Result data (if success).
    #[serde(default)]
    pub result: Option<Value>,

    /// Error code (if error).
    #[serde(default)]
    pub error: Option<String>,

    /// Error message (if error).
    #[serde(default)]
    pub message: Option<String>,
}

impl Response {
    /// Returns `true` if this is a success response.
    #[inline]
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.response_type == ResponseType::Success
    }

    /// Returns `true` if this is an error response.
    #[inline]
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.response_type == ResponseType::Error
    }

    /// Extracts the result value, returning error if response was error.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Protocol`] if the response was an error.
    pub fn into_result(self) -> Result<Value> {
        match self.response_type {
            ResponseType::Success => Ok(self.result.unwrap_or(Value::Null)),
            ResponseType::Error => {
                let error_code = self.error.unwrap_or_else(|| "unknown error".to_string());
                let message = self.message.unwrap_or_else(|| error_code.clone());
                Err(Error::protocol(message))
            }
        }
    }

    /// Gets a string value from the result.
    ///
    /// Returns empty string if key not found or not a string.
    #[inline]
    #[must_use]
    pub fn get_string(&self, key: &str) -> String {
        self.result
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Gets a u64 value from the result.
    ///
    /// Returns 0 if key not found or not a number.
    #[inline]
    #[must_use]
    pub fn get_u64(&self, key: &str) -> u64 {
        self.result
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_u64())
            .unwrap_or_default()
    }

    /// Gets a boolean value from the result.
    ///
    /// Returns false if key not found or not a boolean.
    #[inline]
    #[must_use]
    pub fn get_bool(&self, key: &str) -> bool {
        self.result
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or_default()
    }
}

// ============================================================================
// ResponseType
// ============================================================================

/// Response type discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    /// Successful response.
    Success,
    /// Error response.
    Error,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::BrowsingContextCommand;

    #[test]
    fn test_request_serialization() {
        let tab_id = TabId::new(1).expect("valid tab id");
        let frame_id = FrameId::main();
        let command = Command::BrowsingContext(BrowsingContextCommand::Navigate {
            url: "https://example.com".to_string(),
        });

        let request = Request::new(tab_id, frame_id, command);
        let json = serde_json::to_string(&request).expect("serialize");

        assert!(json.contains("browsingContext.navigate"));
        assert!(json.contains("tabId"));
        assert!(json.contains("frameId"));
    }

    #[test]
    fn test_request_with_id() {
        let id = RequestId::generate();
        let tab_id = TabId::new(1).expect("valid tab id");
        let frame_id = FrameId::main();
        let command = Command::BrowsingContext(BrowsingContextCommand::GetTitle);

        let request = Request::with_id(id, tab_id, frame_id, command);
        assert_eq!(request.id, id);
    }

    #[test]
    fn test_success_response() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "success",
            "result": {"title": "Example"}
        }"#;

        let response: Response = serde_json::from_str(json_str).expect("parse");
        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.get_string("title"), "Example");
    }

    #[test]
    fn test_error_response() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "error",
            "error": "no such element",
            "message": "Element not found"
        }"#;

        let response: Response = serde_json::from_str(json_str).expect("parse");
        assert!(response.is_error());
        assert!(!response.is_success());
        assert_eq!(response.error, Some("no such element".to_string()));
    }

    #[test]
    fn test_into_result_success() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "success",
            "result": {"value": 42}
        }"#;

        let response: Response = serde_json::from_str(json_str).expect("parse");
        let result = response.into_result().expect("should succeed");
        assert_eq!(result.get("value").and_then(|v| v.as_u64()), Some(42));
    }

    #[test]
    fn test_into_result_error() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "error",
            "error": "timeout",
            "message": "Operation timed out"
        }"#;

        let response: Response = serde_json::from_str(json_str).expect("parse");
        let result = response.into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_response_get_helpers() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "success",
            "result": {
                "name": "test",
                "count": 42,
                "enabled": true
            }
        }"#;

        let response: Response = serde_json::from_str(json_str).expect("parse");
        assert_eq!(response.get_string("name"), "test");
        assert_eq!(response.get_u64("count"), 42);
        assert!(response.get_bool("enabled"));

        // Missing keys return defaults
        assert_eq!(response.get_string("missing"), "");
        assert_eq!(response.get_u64("missing"), 0);
        assert!(!response.get_bool("missing"));
    }
}
