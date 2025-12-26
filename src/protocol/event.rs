//! Event message types.
//!
//! Events are notifications sent from the remote end (extension) to the
//! local end (Rust) when browser activity occurs.
//!
//! See ARCHITECTURE.md Section 2.4-2.5 and Section 5 for specification.
//!
//! # Event Types
//!
//! | Module | Events |
//! |--------|--------|
//! | `browsingContext` | `load`, `domContentLoaded`, `navigationStarted`, `navigationFailed` |
//! | `element` | `added`, `removed`, `attributeChanged` |
//! | `network` | `beforeRequestSent`, `responseStarted`, `responseCompleted` |

// ============================================================================
// Imports
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::identifiers::RequestId;

// ============================================================================
// Event
// ============================================================================

/// An event notification from remote end to local end.
///
/// # Format
///
/// ```json
/// {
///   "id": "event-uuid",
///   "type": "event",
///   "method": "module.eventName",
///   "params": { ... }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    /// Unique identifier for EventReply correlation.
    pub id: RequestId,

    /// Event type marker (always "event").
    #[serde(rename = "type")]
    pub event_type: String,

    /// Event name in `module.eventName` format.
    pub method: String,

    /// Event-specific data.
    pub params: Value,
}

impl Event {
    /// Returns the module name from the method.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let event = Event { method: "browsingContext.load".into(), .. };
    /// assert_eq!(event.module(), "browsingContext");
    /// ```
    #[inline]
    #[must_use]
    pub fn module(&self) -> &str {
        self.method.split('.').next().unwrap_or_default()
    }

    /// Returns the event name from the method.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let event = Event { method: "browsingContext.load".into(), .. };
    /// assert_eq!(event.event_name(), "load");
    /// ```
    #[inline]
    #[must_use]
    pub fn event_name(&self) -> &str {
        self.method.split('.').nth(1).unwrap_or_default()
    }

    /// Parses the event into a typed variant.
    #[must_use]
    pub fn parse(&self) -> ParsedEvent {
        self.parse_internal()
    }
}

// ============================================================================
// EventReply
// ============================================================================

/// A reply from local end to remote end for events requiring a decision.
///
/// Used for network interception to allow/block/redirect requests.
///
/// # Format
///
/// ```json
/// {
///   "id": "event-uuid",
///   "replyTo": "network.beforeRequestSent",
///   "result": { "action": "block" }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct EventReply {
    /// Matches the event's ID.
    pub id: RequestId,

    /// Event method being replied to.
    #[serde(rename = "replyTo")]
    pub reply_to: String,

    /// Decision/action to take.
    pub result: Value,
}

impl EventReply {
    /// Creates a new event reply.
    #[inline]
    #[must_use]
    pub fn new(id: RequestId, reply_to: impl Into<String>, result: Value) -> Self {
        Self {
            id,
            reply_to: reply_to.into(),
            result,
        }
    }

    /// Creates an "allow" reply for network events.
    #[inline]
    #[must_use]
    pub fn allow(id: RequestId, reply_to: impl Into<String>) -> Self {
        Self::new(id, reply_to, json!({ "action": "allow" }))
    }

    /// Creates a "block" reply for network events.
    #[inline]
    #[must_use]
    pub fn block(id: RequestId, reply_to: impl Into<String>) -> Self {
        Self::new(id, reply_to, json!({ "action": "block" }))
    }

    /// Creates a "redirect" reply for network events.
    #[inline]
    #[must_use]
    pub fn redirect(id: RequestId, reply_to: impl Into<String>, url: impl Into<String>) -> Self {
        Self::new(
            id,
            reply_to,
            json!({ "action": "redirect", "url": url.into() }),
        )
    }
}

// ============================================================================
// ParsedEvent
// ============================================================================

/// Parsed event types for type-safe handling.
#[derive(Debug, Clone)]
pub enum ParsedEvent {
    /// Navigation started.
    BrowsingContextNavigationStarted {
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
        /// Page URL.
        url: String,
    },

    /// DOM content loaded.
    BrowsingContextDomContentLoaded {
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
        /// Page URL.
        url: String,
    },

    /// Page load complete.
    BrowsingContextLoad {
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
        /// Page URL.
        url: String,
    },

    /// Navigation failed.
    BrowsingContextNavigationFailed {
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
        /// Page URL.
        url: String,
        /// Error message.
        error: String,
    },

    /// Element added to DOM.
    ElementAdded {
        /// Selector strategy (css, xpath, text, etc.).
        strategy: String,
        /// Selector value.
        value: String,
        /// Element ID.
        element_id: String,
        /// Subscription ID.
        subscription_id: String,
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
    },

    /// Element removed from DOM.
    ElementRemoved {
        /// Element ID.
        element_id: String,
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
    },

    /// Element attribute changed.
    ElementAttributeChanged {
        /// Element ID.
        element_id: String,
        /// Attribute name.
        attribute_name: String,
        /// Old value.
        old_value: Option<String>,
        /// New value.
        new_value: Option<String>,
        /// Tab ID.
        tab_id: u32,
        /// Frame ID.
        frame_id: u64,
    },

    /// Network request about to be sent.
    NetworkBeforeRequestSent {
        /// Request ID.
        request_id: String,
        /// Request URL.
        url: String,
        /// HTTP method.
        method: String,
        /// Resource type.
        resource_type: String,
    },

    /// Network response headers received.
    NetworkResponseStarted {
        /// Request ID.
        request_id: String,
        /// Request URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// HTTP status text.
        status_text: String,
    },

    /// Network response completed.
    NetworkResponseCompleted {
        /// Request ID.
        request_id: String,
        /// Request URL.
        url: String,
        /// HTTP status code.
        status: u16,
    },

    /// Unknown event type.
    Unknown {
        /// Event method.
        method: String,
        /// Event params.
        params: Value,
    },
}

// ============================================================================
// Event Parsing Implementation
// ============================================================================

impl Event {
    /// Internal parsing implementation.
    fn parse_internal(&self) -> ParsedEvent {
        match self.method.as_str() {
            "browsingContext.navigationStarted" => ParsedEvent::BrowsingContextNavigationStarted {
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
                url: self.get_string("url"),
            },

            "browsingContext.domContentLoaded" => ParsedEvent::BrowsingContextDomContentLoaded {
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
                url: self.get_string("url"),
            },

            "browsingContext.load" => ParsedEvent::BrowsingContextLoad {
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
                url: self.get_string("url"),
            },

            "browsingContext.navigationFailed" => ParsedEvent::BrowsingContextNavigationFailed {
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
                url: self.get_string("url"),
                error: self.get_string("error"),
            },

            "element.added" => ParsedEvent::ElementAdded {
                strategy: self.get_string("strategy"),
                value: self.get_string("value"),
                element_id: self.get_string("elementId"),
                subscription_id: self.get_string("subscriptionId"),
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
            },

            "element.removed" => ParsedEvent::ElementRemoved {
                element_id: self.get_string("elementId"),
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
            },

            "element.attributeChanged" => ParsedEvent::ElementAttributeChanged {
                element_id: self.get_string("elementId"),
                attribute_name: self.get_string("attributeName"),
                old_value: self.get_optional_string("oldValue"),
                new_value: self.get_optional_string("newValue"),
                tab_id: self.get_u32("tabId"),
                frame_id: self.get_u64("frameId"),
            },

            "network.beforeRequestSent" => ParsedEvent::NetworkBeforeRequestSent {
                request_id: self.get_string("requestId"),
                url: self.get_string("url"),
                method: self.get_string_or("method", "GET"),
                resource_type: self.get_string_or("resourceType", "other"),
            },

            "network.responseStarted" => ParsedEvent::NetworkResponseStarted {
                request_id: self.get_string("requestId"),
                url: self.get_string("url"),
                status: self.get_u16("status"),
                status_text: self.get_string("statusText"),
            },

            "network.responseCompleted" => ParsedEvent::NetworkResponseCompleted {
                request_id: self.get_string("requestId"),
                url: self.get_string("url"),
                status: self.get_u16("status"),
            },

            _ => ParsedEvent::Unknown {
                method: self.method.clone(),
                params: self.params.clone(),
            },
        }
    }

    /// Gets a string from params.
    #[inline]
    fn get_string(&self, key: &str) -> String {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Gets a string from params with default.
    #[inline]
    fn get_string_or(&self, key: &str, default: &str) -> String {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    /// Gets an optional string from params.
    #[inline]
    fn get_optional_string(&self, key: &str) -> Option<String> {
        self.params
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Gets a u32 from params.
    #[inline]
    fn get_u32(&self, key: &str) -> u32 {
        self.params
            .get(key)
            .and_then(|v| v.as_u64())
            .unwrap_or_default() as u32
    }

    /// Gets a u64 from params.
    #[inline]
    fn get_u64(&self, key: &str) -> u64 {
        self.params
            .get(key)
            .and_then(|v| v.as_u64())
            .unwrap_or_default()
    }

    /// Gets a u16 from params.
    #[inline]
    fn get_u16(&self, key: &str) -> u16 {
        self.params
            .get(key)
            .and_then(|v| v.as_u64())
            .unwrap_or_default() as u16
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_parsing() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "event",
            "method": "browsingContext.load",
            "params": {
                "tabId": 1,
                "frameId": 0,
                "url": "https://example.com"
            }
        }"#;

        let event: Event = serde_json::from_str(json_str).expect("parse event");
        assert_eq!(event.module(), "browsingContext");
        assert_eq!(event.event_name(), "load");

        let parsed = event.parse();
        match parsed {
            ParsedEvent::BrowsingContextLoad {
                tab_id,
                frame_id,
                url,
            } => {
                assert_eq!(tab_id, 1);
                assert_eq!(frame_id, 0);
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("unexpected parsed event type"),
        }
    }

    #[test]
    fn test_event_reply_allow() {
        let id = RequestId::generate();
        let reply = EventReply::allow(id, "network.beforeRequestSent");
        let json = serde_json::to_string(&reply).expect("serialize");

        assert!(json.contains("replyTo"));
        assert!(json.contains("allow"));
    }

    #[test]
    fn test_event_reply_block() {
        let id = RequestId::generate();
        let reply = EventReply::block(id, "network.beforeRequestSent");
        let json = serde_json::to_string(&reply).expect("serialize");

        assert!(json.contains("block"));
    }

    #[test]
    fn test_event_reply_redirect() {
        let id = RequestId::generate();
        let reply = EventReply::redirect(id, "network.beforeRequestSent", "https://other.com");
        let json = serde_json::to_string(&reply).expect("serialize");

        assert!(json.contains("redirect"));
        assert!(json.contains("https://other.com"));
    }

    #[test]
    fn test_element_added_parsing() {
        let json_str = r##"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "event",
            "method": "element.added",
            "params": {
                "strategy": "css",
                "value": "#login-form",
                "elementId": "elem-123",
                "subscriptionId": "sub-456",
                "tabId": 1,
                "frameId": 0
            }
        }"##;

        let event: Event = serde_json::from_str(json_str).expect("parse event");
        let parsed = event.parse();

        match parsed {
            ParsedEvent::ElementAdded {
                strategy,
                value,
                element_id,
                ..
            } => {
                assert_eq!(strategy, "css");
                assert_eq!(value, "#login-form");
                assert_eq!(element_id, "elem-123");
            }
            _ => panic!("unexpected parsed event type"),
        }
    }

    #[test]
    fn test_unknown_event() {
        let json_str = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "event",
            "method": "custom.unknownEvent",
            "params": { "foo": "bar" }
        }"#;

        let event: Event = serde_json::from_str(json_str).expect("parse event");
        let parsed = event.parse();

        match parsed {
            ParsedEvent::Unknown { method, .. } => {
                assert_eq!(method, "custom.unknownEvent");
            }
            _ => panic!("expected Unknown variant"),
        }
    }
}
