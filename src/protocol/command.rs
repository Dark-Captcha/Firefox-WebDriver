//! Command definitions organized by module.
//!
//! Commands follow `module.methodName` format per ARCHITECTURE.md Section 2.2.
//!
//! # Command Modules
//!
//! | Module | Commands |
//! |--------|----------|
//! | `browsingContext` | Navigation, tabs, frames |
//! | `element` | Find, properties, methods |
//! | `script` | JavaScript execution |
//! | `input` | Keyboard and mouse |
//! | `network` | Interception, blocking |
//! | `proxy` | Proxy configuration |
//! | `storage` | Cookies |
//! | `session` | Status, subscriptions |

// ============================================================================
// Imports
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::identifiers::{ElementId, InterceptId};

// ============================================================================
// Command Wrapper
// ============================================================================

/// All protocol commands organized by module.
///
/// This enum wraps module-specific command enums for unified serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Command {
    /// BrowsingContext module commands.
    BrowsingContext(BrowsingContextCommand),
    /// Element module commands.
    Element(ElementCommand),
    /// Session module commands.
    Session(SessionCommand),
    /// Script module commands.
    Script(ScriptCommand),
    /// Input module commands.
    Input(InputCommand),
    /// Network module commands.
    Network(NetworkCommand),
    /// Proxy module commands.
    Proxy(ProxyCommand),
    /// Storage module commands.
    Storage(StorageCommand),
}

// ============================================================================
// BrowsingContext Commands
// ============================================================================

/// BrowsingContext module commands for navigation and tab management.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum BrowsingContextCommand {
    /// Navigate to URL.
    #[serde(rename = "browsingContext.navigate")]
    Navigate {
        /// URL to navigate to.
        url: String,
    },

    /// Reload current page.
    #[serde(rename = "browsingContext.reload")]
    Reload,

    /// Navigate back in history.
    #[serde(rename = "browsingContext.goBack")]
    GoBack,

    /// Navigate forward in history.
    #[serde(rename = "browsingContext.goForward")]
    GoForward,

    /// Get page title.
    #[serde(rename = "browsingContext.getTitle")]
    GetTitle,

    /// Get current URL.
    #[serde(rename = "browsingContext.getUrl")]
    GetUrl,

    /// Create new tab.
    #[serde(rename = "browsingContext.newTab")]
    NewTab,

    /// Close current tab.
    #[serde(rename = "browsingContext.closeTab")]
    CloseTab,

    /// Focus tab (make active).
    #[serde(rename = "browsingContext.focusTab")]
    FocusTab,

    /// Focus window (bring to front).
    #[serde(rename = "browsingContext.focusWindow")]
    FocusWindow,

    /// Switch to frame by element reference.
    #[serde(rename = "browsingContext.switchToFrame")]
    SwitchToFrame {
        /// Element ID of iframe.
        #[serde(rename = "elementId")]
        element_id: ElementId,
    },

    /// Switch to frame by index.
    #[serde(rename = "browsingContext.switchToFrameByIndex")]
    SwitchToFrameByIndex {
        /// Zero-based frame index.
        index: usize,
    },

    /// Switch to frame by URL pattern.
    #[serde(rename = "browsingContext.switchToFrameByUrl")]
    SwitchToFrameByUrl {
        /// URL pattern with wildcards.
        #[serde(rename = "urlPattern")]
        url_pattern: String,
    },

    /// Switch to parent frame.
    #[serde(rename = "browsingContext.switchToParentFrame")]
    SwitchToParentFrame,

    /// Get child frame count.
    #[serde(rename = "browsingContext.getFrameCount")]
    GetFrameCount,

    /// Get all frames info.
    #[serde(rename = "browsingContext.getAllFrames")]
    GetAllFrames,
}

// ============================================================================
// Element Commands
// ============================================================================

/// Element module commands for DOM interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum ElementCommand {
    /// Find single element by CSS selector.
    #[serde(rename = "element.find")]
    Find {
        /// CSS selector.
        selector: String,
        /// Parent element ID (optional).
        #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
        parent_id: Option<ElementId>,
    },

    /// Find all elements by CSS selector.
    #[serde(rename = "element.findAll")]
    FindAll {
        /// CSS selector.
        selector: String,
        /// Parent element ID (optional).
        #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
        parent_id: Option<ElementId>,
    },

    /// Get property via `element[name]`.
    #[serde(rename = "element.getProperty")]
    GetProperty {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
        /// Property name.
        name: String,
    },

    /// Set property via `element[name] = value`.
    #[serde(rename = "element.setProperty")]
    SetProperty {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
        /// Property name.
        name: String,
        /// Property value.
        value: Value,
    },

    /// Call method via `element[name](...args)`.
    #[serde(rename = "element.callMethod")]
    CallMethod {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
        /// Method name.
        name: String,
        /// Method arguments.
        #[serde(default)]
        args: Vec<Value>,
    },

    /// Subscribe to element appearance.
    #[serde(rename = "element.subscribe")]
    Subscribe {
        /// CSS selector to watch.
        selector: String,
        /// Auto-unsubscribe after first match.
        #[serde(rename = "oneShot")]
        one_shot: bool,
    },

    /// Unsubscribe from element observation.
    #[serde(rename = "element.unsubscribe")]
    Unsubscribe {
        /// Subscription ID.
        #[serde(rename = "subscriptionId")]
        subscription_id: String,
    },

    /// Watch for element removal.
    #[serde(rename = "element.watchRemoval")]
    WatchRemoval {
        /// Element ID to watch.
        #[serde(rename = "elementId")]
        element_id: ElementId,
    },

    /// Stop watching for element removal.
    #[serde(rename = "element.unwatchRemoval")]
    UnwatchRemoval {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
    },

    /// Watch for attribute changes.
    #[serde(rename = "element.watchAttribute")]
    WatchAttribute {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
        /// Specific attribute (optional).
        #[serde(rename = "attributeName", skip_serializing_if = "Option::is_none")]
        attribute_name: Option<String>,
    },

    /// Stop watching for attribute changes.
    #[serde(rename = "element.unwatchAttribute")]
    UnwatchAttribute {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
    },
}

// ============================================================================
// Session Commands
// ============================================================================

/// Session module commands for connection management.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum SessionCommand {
    /// Get session status.
    #[serde(rename = "session.status")]
    Status,

    /// Get and clear extension logs.
    #[serde(rename = "session.stealLogs")]
    StealLogs,

    /// Subscribe to events.
    #[serde(rename = "session.subscribe")]
    Subscribe {
        /// Event names to subscribe to.
        events: Vec<String>,
        /// CSS selectors for element events.
        #[serde(skip_serializing_if = "Option::is_none")]
        selectors: Option<Vec<String>>,
    },

    /// Unsubscribe from events.
    #[serde(rename = "session.unsubscribe")]
    Unsubscribe {
        /// Subscription ID.
        subscription_id: String,
    },
}

// ============================================================================
// Script Commands
// ============================================================================

/// Script module commands for JavaScript execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum ScriptCommand {
    /// Execute synchronous script.
    #[serde(rename = "script.evaluate")]
    Evaluate {
        /// JavaScript code.
        script: String,
        /// Script arguments.
        #[serde(default)]
        args: Vec<Value>,
    },

    /// Execute async script.
    #[serde(rename = "script.evaluateAsync")]
    EvaluateAsync {
        /// JavaScript code.
        script: String,
        /// Script arguments.
        #[serde(default)]
        args: Vec<Value>,
    },

    /// Add preload script.
    #[serde(rename = "script.addPreloadScript")]
    AddPreloadScript {
        /// Script to run before page load.
        script: String,
    },

    /// Remove preload script.
    #[serde(rename = "script.removePreloadScript")]
    RemovePreloadScript {
        /// Script ID.
        script_id: String,
    },
}

// ============================================================================
// Input Commands
// ============================================================================

/// Input module commands for keyboard and mouse simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum InputCommand {
    /// Type single key with modifiers.
    #[serde(rename = "input.typeKey")]
    TypeKey {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
        /// Key value (e.g., "a", "Enter").
        key: String,
        /// Key code (e.g., "KeyA", "Enter").
        code: String,
        /// Legacy keyCode number.
        #[serde(rename = "keyCode")]
        key_code: u32,
        /// Is printable character.
        printable: bool,
        /// Ctrl modifier.
        #[serde(default)]
        ctrl: bool,
        /// Shift modifier.
        #[serde(default)]
        shift: bool,
        /// Alt modifier.
        #[serde(default)]
        alt: bool,
        /// Meta modifier.
        #[serde(default)]
        meta: bool,
    },

    /// Type text string character by character.
    #[serde(rename = "input.typeText")]
    TypeText {
        /// Element ID.
        #[serde(rename = "elementId")]
        element_id: ElementId,
        /// Text to type.
        text: String,
    },

    /// Mouse click.
    #[serde(rename = "input.mouseClick")]
    MouseClick {
        /// Element ID (optional).
        #[serde(rename = "elementId", skip_serializing_if = "Option::is_none")]
        element_id: Option<ElementId>,
        /// X coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        x: Option<i32>,
        /// Y coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        y: Option<i32>,
        /// Mouse button (0=left, 1=middle, 2=right).
        #[serde(default)]
        button: u8,
    },

    /// Mouse move.
    #[serde(rename = "input.mouseMove")]
    MouseMove {
        /// Element ID (optional).
        #[serde(rename = "elementId", skip_serializing_if = "Option::is_none")]
        element_id: Option<ElementId>,
        /// X coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        x: Option<i32>,
        /// Y coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        y: Option<i32>,
    },

    /// Mouse button down.
    #[serde(rename = "input.mouseDown")]
    MouseDown {
        /// Element ID (optional).
        #[serde(rename = "elementId", skip_serializing_if = "Option::is_none")]
        element_id: Option<ElementId>,
        /// X coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        x: Option<i32>,
        /// Y coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        y: Option<i32>,
        /// Mouse button.
        #[serde(default)]
        button: u8,
    },

    /// Mouse button up.
    #[serde(rename = "input.mouseUp")]
    MouseUp {
        /// Element ID (optional).
        #[serde(rename = "elementId", skip_serializing_if = "Option::is_none")]
        element_id: Option<ElementId>,
        /// X coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        x: Option<i32>,
        /// Y coordinate.
        #[serde(skip_serializing_if = "Option::is_none")]
        y: Option<i32>,
        /// Mouse button.
        #[serde(default)]
        button: u8,
    },
}

// ============================================================================
// Network Commands
// ============================================================================

/// Network module commands for request interception.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum NetworkCommand {
    /// Add network intercept.
    #[serde(rename = "network.addIntercept")]
    AddIntercept {
        /// Intercept requests.
        #[serde(default, rename = "interceptRequests")]
        intercept_requests: bool,
        /// Intercept request headers.
        #[serde(default, rename = "interceptRequestHeaders")]
        intercept_request_headers: bool,
        /// Intercept request body (read-only).
        #[serde(default, rename = "interceptRequestBody")]
        intercept_request_body: bool,
        /// Intercept response headers.
        #[serde(default, rename = "interceptResponses")]
        intercept_responses: bool,
        /// Intercept response body.
        #[serde(default, rename = "interceptResponseBody")]
        intercept_response_body: bool,
    },

    /// Remove network intercept.
    #[serde(rename = "network.removeIntercept")]
    RemoveIntercept {
        /// Intercept ID.
        #[serde(rename = "interceptId")]
        intercept_id: InterceptId,
    },

    /// Set URL block rules.
    #[serde(rename = "network.setBlockRules")]
    SetBlockRules {
        /// URL patterns to block.
        patterns: Vec<String>,
    },

    /// Clear all block rules.
    #[serde(rename = "network.clearBlockRules")]
    ClearBlockRules,
}

// ============================================================================
// Proxy Commands
// ============================================================================

/// Proxy module commands for proxy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum ProxyCommand {
    /// Set window-level proxy.
    #[serde(rename = "proxy.setWindowProxy")]
    SetWindowProxy {
        /// Proxy type: http, https, socks4, socks5.
        #[serde(rename = "type")]
        proxy_type: String,
        /// Proxy host.
        host: String,
        /// Proxy port.
        port: u16,
        /// Username (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        /// Password (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        /// Proxy DNS (SOCKS only).
        #[serde(rename = "proxyDns", default)]
        proxy_dns: bool,
    },

    /// Clear window-level proxy.
    #[serde(rename = "proxy.clearWindowProxy")]
    ClearWindowProxy,

    /// Set tab-level proxy.
    #[serde(rename = "proxy.setTabProxy")]
    SetTabProxy {
        /// Proxy type.
        #[serde(rename = "type")]
        proxy_type: String,
        /// Proxy host.
        host: String,
        /// Proxy port.
        port: u16,
        /// Username (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        /// Password (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        /// Proxy DNS (SOCKS only).
        #[serde(rename = "proxyDns", default)]
        proxy_dns: bool,
    },

    /// Clear tab-level proxy.
    #[serde(rename = "proxy.clearTabProxy")]
    ClearTabProxy,
}

// ============================================================================
// Storage Commands
// ============================================================================

/// Storage module commands for cookie management.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum StorageCommand {
    /// Get cookie by name.
    #[serde(rename = "storage.getCookie")]
    GetCookie {
        /// Cookie name.
        name: String,
        /// URL (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// Set cookie.
    #[serde(rename = "storage.setCookie")]
    SetCookie {
        /// Cookie data.
        cookie: Cookie,
        /// URL (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// Delete cookie by name.
    #[serde(rename = "storage.deleteCookie")]
    DeleteCookie {
        /// Cookie name.
        name: String,
        /// URL (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },

    /// Get all cookies.
    #[serde(rename = "storage.getAllCookies")]
    GetAllCookies {
        /// URL (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
}

// ============================================================================
// Cookie
// ============================================================================

/// Browser cookie with standard properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Secure flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<bool>,
    /// HttpOnly flag.
    #[serde(rename = "httpOnly", skip_serializing_if = "Option::is_none")]
    pub http_only: Option<bool>,
    /// SameSite attribute.
    #[serde(rename = "sameSite", skip_serializing_if = "Option::is_none")]
    pub same_site: Option<String>,
    /// Expiration timestamp (seconds).
    #[serde(rename = "expirationDate", skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<f64>,
}

impl Cookie {
    /// Creates a new cookie with name and value.
    #[inline]
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
            expiration_date: None,
        }
    }

    /// Sets the domain.
    #[inline]
    #[must_use]
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the path.
    #[inline]
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets the secure flag.
    #[inline]
    #[must_use]
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = Some(secure);
        self
    }

    /// Sets the httpOnly flag.
    #[inline]
    #[must_use]
    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only = Some(http_only);
        self
    }

    /// Sets the sameSite attribute.
    #[inline]
    #[must_use]
    pub fn with_same_site(mut self, same_site: impl Into<String>) -> Self {
        self.same_site = Some(same_site.into());
        self
    }

    /// Sets the expiration date.
    #[inline]
    #[must_use]
    pub fn with_expiration_date(mut self, expiration_date: f64) -> Self {
        self.expiration_date = Some(expiration_date);
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browsing_context_navigate() {
        let cmd = BrowsingContextCommand::Navigate {
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("browsingContext.navigate"));
        assert!(json.contains("https://example.com"));
    }

    #[test]
    fn test_element_find() {
        let cmd = ElementCommand::Find {
            selector: "button.submit".to_string(),
            parent_id: None,
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("element.find"));
        assert!(json.contains("button.submit"));
    }

    #[test]
    fn test_element_get_property() {
        let cmd = ElementCommand::GetProperty {
            element_id: ElementId::new("test-uuid"),
            name: "textContent".to_string(),
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("element.getProperty"));
        assert!(json.contains("test-uuid"));
        assert!(json.contains("textContent"));
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = Cookie::new("session", "abc123")
            .with_domain(".example.com")
            .with_path("/")
            .with_secure(true)
            .with_http_only(true)
            .with_same_site("strict");

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some(".example.com".to_string()));
        assert_eq!(cookie.secure, Some(true));
    }

    #[test]
    fn test_network_add_intercept() {
        let cmd = NetworkCommand::AddIntercept {
            intercept_requests: true,
            intercept_request_headers: false,
            intercept_request_body: false,
            intercept_responses: false,
            intercept_response_body: false,
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("network.addIntercept"));
    }
}
