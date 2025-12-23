//! Static assets and HTML templates for driver initialization.
//!
//! This module generates the HTML page that Firefox loads on startup to
//! establish the WebSocket connection between the browser extension and
//! the Rust driver.
//!
//! # Connection Flow
//!
//! 1. Firefox opens data URI as initial page
//! 2. Page posts `WEBDRIVER_INIT` message to window
//! 3. Content script receives message, validates localhost URL
//! 4. Content script forwards to background script
//! 5. Background script connects to WebSocket server

// ============================================================================
// Imports
// ============================================================================

use serde_json::json;

use crate::identifiers::SessionId;

// ============================================================================
// Public Functions
// ============================================================================

/// Generates the initialization HTML page as a data URI.
///
/// This page is loaded as the first tab when Firefox starts. It contains
/// JavaScript that posts a message to the content script, triggering the
/// WebSocket handshake.
///
/// # Arguments
///
/// * `ws_url` - WebSocket server URL (e.g., "ws://127.0.0.1:12345")
/// * `session_id` - Session identifier for this window
///
/// # Returns
///
/// A `data:text/html,...` URI that can be passed to Firefox.
#[must_use]
pub fn build_init_data_uri(ws_url: &str, session_id: &SessionId) -> String {
    let config_json = build_config_json(ws_url, session_id);
    let html = build_init_html(ws_url, session_id, &config_json);

    format!("data:text/html,{}", urlencoding::encode(&html))
}

// ============================================================================
// Internal Functions
// ============================================================================

/// Builds the JSON configuration object for the extension.
fn build_config_json(ws_url: &str, session_id: &SessionId) -> String {
    let config = json!({
        "type": "WEBDRIVER_INIT",
        "wsUrl": ws_url,
        "sessionId": session_id.as_u32(),
    });

    config.to_string()
}

/// Builds the initialization HTML page content.
fn build_init_html(ws_url: &str, session_id: &SessionId, config_json: &str) -> String {
    let session_id_str = session_id.as_u32().to_string();

    INIT_HTML_TEMPLATE
        .replace("$WS_URL", ws_url)
        .replace("$SESSION_ID", &session_id_str)
        .replace("$CONFIG_JSON", config_json)
}

// ============================================================================
// Constants
// ============================================================================

/// HTML template for the initialization page.
///
/// This page displays connection info and posts `WEBDRIVER_INIT` message
/// for the content script to forward to the background script.
const INIT_HTML_TEMPLATE: &str = r##"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>WebDriver Init</title>
    <style>
        body {
            background: #1a1a2e;
            color: #ccc;
            font-family: monospace;
            padding: 40px;
            line-height: 1.6;
        }
        h1 { color: #e94560; margin-bottom: 20px; }
        .key { color: #4ade80; font-weight: bold; }
        .val { color: #fff; word-break: break-all; }
        hr { border: 0; border-top: 1px dashed #333; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Firefox WebDriver</h1>
    <div><span class="key">WS_URL:</span> <span class="val">$WS_URL</span></div>
    <div><span class="key">SESSION:</span> <span class="val">$SESSION_ID</span></div>
    <hr>
    <div style="color: #4ade80;">> Initializing connection...</div>
    <script>window.postMessage($CONFIG_JSON, '*');</script>
</body>
</html>"##;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_build_init_data_uri_format() {
        let session_id = SessionId::next();
        let uri = build_init_data_uri("ws://127.0.0.1:12345", &session_id);

        assert!(uri.starts_with("data:text/html,"));
        assert!(uri.len() > 100); // Should have substantial content
    }

    #[test]
    fn test_build_config_json_structure() {
        let session_id = SessionId::next();
        let json_str = build_config_json("ws://127.0.0.1:12345", &session_id);

        let parsed: Value = serde_json::from_str(&json_str).expect("valid json");
        assert_eq!(parsed["type"], "WEBDRIVER_INIT");
        assert_eq!(parsed["wsUrl"], "ws://127.0.0.1:12345");
        assert!(parsed["sessionId"].is_number());
    }

    #[test]
    fn test_build_init_html_contains_required_elements() {
        let session_id = SessionId::next();
        let config_json = build_config_json("ws://127.0.0.1:12345", &session_id);
        let html = build_init_html("ws://127.0.0.1:12345", &session_id, &config_json);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>WebDriver Init</title>"));
        assert!(html.contains("window.postMessage"));
        assert!(html.contains("WEBDRIVER_INIT"));
        assert!(html.contains("ws://127.0.0.1:12345"));
    }

    #[test]
    fn test_data_uri_is_url_encoded() {
        let session_id = SessionId::next();
        let uri = build_init_data_uri("ws://127.0.0.1:12345", &session_id);

        // URL encoding should escape special characters
        assert!(!uri.contains('<'));
        assert!(!uri.contains('>'));
    }
}
