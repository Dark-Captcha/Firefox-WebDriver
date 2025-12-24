//! Browser tab automation and control.
//!
//! Each [`Tab`] represents a browser tab with a specific frame context.
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
//! // Execute JavaScript
//! let result = tab.execute_script("return document.title").await?;
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64Standard;
use parking_lot::Mutex as ParkingMutex;
use serde_json::Value;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::debug;

use crate::error::{Error, Result};
use crate::identifiers::{ElementId, FrameId, InterceptId, SessionId, SubscriptionId, TabId};
use crate::protocol::event::ParsedEvent;
use crate::protocol::{
    BrowsingContextCommand, Command, Cookie, ElementCommand, Event, EventReply, NetworkCommand,
    ProxyCommand, Request, Response, ScriptCommand, StorageCommand,
};

use super::network::{
    BodyAction, HeadersAction, InterceptedRequest, InterceptedRequestBody,
    InterceptedRequestHeaders, InterceptedResponse, InterceptedResponseBody, RequestAction,
    RequestBody,
};
use super::proxy::ProxyConfig;
use super::{Element, Window};

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for wait_for_element (30 seconds).
const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// Types
// ============================================================================

/// Information about a frame in the tab.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// Frame ID.
    pub frame_id: FrameId,
    /// Parent frame ID (None for main frame).
    pub parent_frame_id: Option<FrameId>,
    /// Frame URL.
    pub url: String,
}

/// Internal shared state for a tab.
pub(crate) struct TabInner {
    /// Tab ID.
    pub tab_id: TabId,
    /// Current frame ID.
    pub frame_id: FrameId,
    /// Session ID.
    pub session_id: SessionId,
    /// Parent window (optional for standalone tab references).
    pub window: Option<Window>,
}

// ============================================================================
// Tab
// ============================================================================

/// A handle to a browser tab.
///
/// Tabs provide methods for navigation, scripting, and element interaction.
#[derive(Clone)]
pub struct Tab {
    pub(crate) inner: Arc<TabInner>,
}

impl fmt::Debug for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tab")
            .field("tab_id", &self.inner.tab_id)
            .field("frame_id", &self.inner.frame_id)
            .field("session_id", &self.inner.session_id)
            .finish_non_exhaustive()
    }
}

impl Tab {
    /// Creates a new tab handle.
    pub(crate) fn new(
        tab_id: TabId,
        frame_id: FrameId,
        session_id: SessionId,
        window: Option<Window>,
    ) -> Self {
        Self {
            inner: Arc::new(TabInner {
                tab_id,
                frame_id,
                session_id,
                window,
            }),
        }
    }
}

// ============================================================================
// Tab - Accessors
// ============================================================================

impl Tab {
    /// Returns the tab ID.
    #[inline]
    #[must_use]
    pub fn tab_id(&self) -> TabId {
        self.inner.tab_id
    }

    /// Returns the current frame ID.
    #[inline]
    #[must_use]
    pub fn frame_id(&self) -> FrameId {
        self.inner.frame_id
    }

    /// Returns the session ID.
    #[inline]
    #[must_use]
    pub fn session_id(&self) -> SessionId {
        self.inner.session_id
    }
}

// ============================================================================
// Tab - Navigation
// ============================================================================

impl Tab {
    /// Navigates to a URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to navigate to
    ///
    /// # Errors
    ///
    /// Returns an error if navigation fails.
    pub async fn goto(&self, url: &str) -> Result<()> {
        debug!(url = %url, tab_id = %self.inner.tab_id, "Navigating");

        let command = Command::BrowsingContext(BrowsingContextCommand::Navigate {
            url: url.to_string(),
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Alias for [`goto`](Self::goto).
    pub async fn navigate(&self, url: &str) -> Result<()> {
        self.goto(url).await
    }

    /// Loads HTML content directly into the page.
    ///
    /// Useful for testing with inline HTML without needing a server.
    ///
    /// # Arguments
    ///
    /// * `html` - HTML content to load
    ///
    /// # Example
    ///
    /// ```ignore
    /// tab.load_html("<html><body><h1>Test</h1></body></html>").await?;
    /// ```
    pub async fn load_html(&self, html: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, html_len = html.len(), "Loading HTML content");

        let escaped_html = html
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace("${", "\\${");

        let script = format!(
            r#"(function() {{
                const html = `{}`;
                const parser = new DOMParser();
                const doc = parser.parseFromString(html, 'text/html');
                const newTitle = doc.querySelector('title');
                if (newTitle) {{ document.title = newTitle.textContent; }}
                const newBody = doc.body;
                if (newBody) {{
                    document.body.innerHTML = newBody.innerHTML;
                    for (const attr of newBody.attributes) {{
                        document.body.setAttribute(attr.name, attr.value);
                    }}
                }}
                const newHead = doc.head;
                if (newHead) {{
                    for (const child of newHead.children) {{
                        if (child.tagName !== 'TITLE') {{
                            document.head.appendChild(child.cloneNode(true));
                        }}
                    }}
                }}
            }})();"#,
            escaped_html
        );

        self.execute_script(&script).await?;
        Ok(())
    }

    /// Reloads the current page.
    pub async fn reload(&self) -> Result<()> {
        let command = Command::BrowsingContext(BrowsingContextCommand::Reload);
        self.send_command(command).await?;
        Ok(())
    }

    /// Navigates back in history.
    pub async fn back(&self) -> Result<()> {
        let command = Command::BrowsingContext(BrowsingContextCommand::GoBack);
        self.send_command(command).await?;
        Ok(())
    }

    /// Navigates forward in history.
    pub async fn forward(&self) -> Result<()> {
        let command = Command::BrowsingContext(BrowsingContextCommand::GoForward);
        self.send_command(command).await?;
        Ok(())
    }

    /// Gets the current page title.
    pub async fn get_title(&self) -> Result<String> {
        let command = Command::BrowsingContext(BrowsingContextCommand::GetTitle);
        let response = self.send_command(command).await?;

        let title = response
            .result
            .as_ref()
            .and_then(|v| v.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(title)
    }

    /// Gets the current URL.
    pub async fn get_url(&self) -> Result<String> {
        let command = Command::BrowsingContext(BrowsingContextCommand::GetUrl);
        let response = self.send_command(command).await?;

        let url = response
            .result
            .as_ref()
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(url)
    }

    /// Focuses this tab (makes it active).
    pub async fn focus(&self) -> Result<()> {
        let command = Command::BrowsingContext(BrowsingContextCommand::FocusTab);
        self.send_command(command).await?;
        Ok(())
    }

    /// Focuses the window containing this tab.
    pub async fn focus_window(&self) -> Result<()> {
        let command = Command::BrowsingContext(BrowsingContextCommand::FocusWindow);
        self.send_command(command).await?;
        Ok(())
    }

    /// Closes this tab.
    pub async fn close(&self) -> Result<()> {
        let command = Command::BrowsingContext(BrowsingContextCommand::CloseTab);
        self.send_command(command).await?;
        Ok(())
    }
}

// ============================================================================
// Tab - Frame Switching
// ============================================================================

impl Tab {
    /// Switches to a frame by iframe element.
    ///
    /// Returns a new Tab handle with the updated frame context.
    ///
    /// # Arguments
    ///
    /// * `iframe` - Element reference to an iframe
    ///
    /// # Example
    ///
    /// ```ignore
    /// let iframe = tab.find_element("iframe#content").await?;
    /// let frame_tab = tab.switch_to_frame(&iframe).await?;
    /// ```
    pub async fn switch_to_frame(&self, iframe: &Element) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, element_id = %iframe.id(), "Switching to frame");

        let command = Command::BrowsingContext(BrowsingContextCommand::SwitchToFrame {
            element_id: iframe.id().clone(),
        });
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to a frame by index (0-based).
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the frame
    pub async fn switch_to_frame_by_index(&self, index: usize) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, index, "Switching to frame by index");

        let command =
            Command::BrowsingContext(BrowsingContextCommand::SwitchToFrameByIndex { index });
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to a frame by URL pattern.
    ///
    /// Supports wildcards (`*` for any characters, `?` for single character).
    ///
    /// # Arguments
    ///
    /// * `url_pattern` - URL pattern with optional wildcards
    pub async fn switch_to_frame_by_url(&self, url_pattern: &str) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, url_pattern, "Switching to frame by URL");

        let command = Command::BrowsingContext(BrowsingContextCommand::SwitchToFrameByUrl {
            url_pattern: url_pattern.to_string(),
        });
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to the parent frame.
    pub async fn switch_to_parent_frame(&self) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, "Switching to parent frame");

        let command = Command::BrowsingContext(BrowsingContextCommand::SwitchToParentFrame);
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to the main (top-level) frame.
    #[must_use]
    pub fn switch_to_main_frame(&self) -> Tab {
        debug!(tab_id = %self.inner.tab_id, "Switching to main frame");

        Tab::new(
            self.inner.tab_id,
            FrameId::main(),
            self.inner.session_id,
            self.inner.window.clone(),
        )
    }

    /// Gets the count of direct child frames.
    pub async fn get_frame_count(&self) -> Result<usize> {
        let command = Command::BrowsingContext(BrowsingContextCommand::GetFrameCount);
        let response = self.send_command(command).await?;

        let count = response
            .result
            .as_ref()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| Error::protocol("No count in response"))?;

        Ok(count as usize)
    }

    /// Gets information about all frames in the tab.
    pub async fn get_all_frames(&self) -> Result<Vec<FrameInfo>> {
        let command = Command::BrowsingContext(BrowsingContextCommand::GetAllFrames);
        let response = self.send_command(command).await?;

        let frames = response
            .result
            .as_ref()
            .and_then(|v| v.get("frames"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_frame_info).collect())
            .unwrap_or_default();

        Ok(frames)
    }

    /// Checks if currently in the main frame.
    #[inline]
    #[must_use]
    pub fn is_main_frame(&self) -> bool {
        self.inner.frame_id.is_main()
    }
}

// ============================================================================
// Tab - Network
// ============================================================================

impl Tab {
    /// Sets URL patterns to block.
    ///
    /// Patterns support wildcards (`*`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// tab.set_block_rules(&["*ads*", "*tracking*"]).await?;
    /// ```
    pub async fn set_block_rules(&self, patterns: &[&str]) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, pattern_count = patterns.len(), "Setting block rules");

        let command = Command::Network(NetworkCommand::SetBlockRules {
            patterns: patterns.iter().map(|s| (*s).to_string()).collect(),
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Clears all URL block rules.
    pub async fn clear_block_rules(&self) -> Result<()> {
        let command = Command::Network(NetworkCommand::ClearBlockRules);
        self.send_command(command).await?;
        Ok(())
    }

    /// Intercepts network requests with a callback.
    ///
    /// # Returns
    ///
    /// An `InterceptId` that can be used to stop this intercept.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::RequestAction;
    ///
    /// let id = tab.intercept_request(|req| {
    ///     if req.url.contains("ads") {
    ///         RequestAction::block()
    ///     } else {
    ///         RequestAction::allow()
    ///     }
    /// }).await?;
    /// ```
    pub async fn intercept_request<F>(&self, callback: F) -> Result<InterceptId>
    where
        F: Fn(InterceptedRequest) -> RequestAction + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, "Enabling request interception");

        let window = self.get_window()?;
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "network.beforeRequestSent" {
                    return None;
                }

                let request = parse_intercepted_request(&event);
                let action = callback(request);
                let result = request_action_to_json(&action);

                Some(EventReply::new(
                    event.id,
                    "network.beforeRequestSent",
                    result,
                ))
            }),
        );

        let command = Command::Network(NetworkCommand::AddIntercept {
            intercept_requests: true,
            intercept_request_headers: false,
            intercept_request_body: false,
            intercept_responses: false,
            intercept_response_body: false,
        });

        let response = self.send_command(command).await?;
        extract_intercept_id(&response)
    }

    /// Intercepts request headers with a callback.
    pub async fn intercept_request_headers<F>(&self, callback: F) -> Result<InterceptId>
    where
        F: Fn(InterceptedRequestHeaders) -> HeadersAction + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, "Enabling request headers interception");

        let window = self.get_window()?;
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "network.requestHeaders" {
                    return None;
                }

                let headers_data = parse_intercepted_request_headers(&event);
                let action = callback(headers_data);
                let result = headers_action_to_json(&action);

                Some(EventReply::new(event.id, "network.requestHeaders", result))
            }),
        );

        let command = Command::Network(NetworkCommand::AddIntercept {
            intercept_requests: false,
            intercept_request_headers: true,
            intercept_request_body: false,
            intercept_responses: false,
            intercept_response_body: false,
        });

        let response = self.send_command(command).await?;
        extract_intercept_id(&response)
    }

    /// Intercepts request body for logging (read-only).
    pub async fn intercept_request_body<F>(&self, callback: F) -> Result<InterceptId>
    where
        F: Fn(InterceptedRequestBody) + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, "Enabling request body interception");

        let window = self.get_window()?;
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "network.requestBody" {
                    return None;
                }

                let body_data = parse_intercepted_request_body(&event);
                callback(body_data);

                Some(EventReply::new(
                    event.id,
                    "network.requestBody",
                    serde_json::json!({ "action": "allow" }),
                ))
            }),
        );

        let command = Command::Network(NetworkCommand::AddIntercept {
            intercept_requests: false,
            intercept_request_headers: false,
            intercept_request_body: true,
            intercept_responses: false,
            intercept_response_body: false,
        });

        let response = self.send_command(command).await?;
        extract_intercept_id(&response)
    }

    /// Intercepts response headers with a callback.
    pub async fn intercept_response<F>(&self, callback: F) -> Result<InterceptId>
    where
        F: Fn(InterceptedResponse) -> HeadersAction + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, "Enabling response interception");

        let window = self.get_window()?;
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "network.responseHeaders" {
                    return None;
                }

                let resp = parse_intercepted_response(&event);
                let action = callback(resp);
                let result = headers_action_to_json(&action);

                Some(EventReply::new(event.id, "network.responseHeaders", result))
            }),
        );

        let command = Command::Network(NetworkCommand::AddIntercept {
            intercept_requests: false,
            intercept_request_headers: false,
            intercept_request_body: false,
            intercept_responses: true,
            intercept_response_body: false,
        });

        let response = self.send_command(command).await?;
        extract_intercept_id(&response)
    }

    /// Intercepts response body with a callback.
    pub async fn intercept_response_body<F>(&self, callback: F) -> Result<InterceptId>
    where
        F: Fn(InterceptedResponseBody) -> BodyAction + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, "Enabling response body interception");

        let window = self.get_window()?;
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "network.responseBody" {
                    return None;
                }

                let body_data = parse_intercepted_response_body(&event);
                let action = callback(body_data);
                let result = body_action_to_json(&action);

                Some(EventReply::new(event.id, "network.responseBody", result))
            }),
        );

        let command = Command::Network(NetworkCommand::AddIntercept {
            intercept_requests: false,
            intercept_request_headers: false,
            intercept_request_body: false,
            intercept_responses: false,
            intercept_response_body: true,
        });

        let response = self.send_command(command).await?;
        extract_intercept_id(&response)
    }

    /// Stops network interception.
    ///
    /// # Arguments
    ///
    /// * `intercept_id` - The intercept ID returned from intercept methods
    pub async fn stop_intercept(&self, intercept_id: &InterceptId) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, %intercept_id, "Stopping interception");

        let window = self.get_window()?;
        window
            .inner
            .pool
            .clear_event_handler(window.inner.session_id);

        let command = Command::Network(NetworkCommand::RemoveIntercept {
            intercept_id: intercept_id.clone(),
        });

        self.send_command(command).await?;
        Ok(())
    }
}

// ============================================================================
// Tab - Proxy
// ============================================================================

impl Tab {
    /// Sets a proxy for this tab.
    ///
    /// Tab-level proxy overrides window-level proxy for this tab only.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::ProxyConfig;
    ///
    /// tab.set_proxy(ProxyConfig::http("proxy.example.com", 8080)).await?;
    /// ```
    pub async fn set_proxy(&self, config: ProxyConfig) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, proxy_type = %config.proxy_type.as_str(), "Setting proxy");

        let command = Command::Proxy(ProxyCommand::SetTabProxy {
            proxy_type: config.proxy_type.as_str().to_string(),
            host: config.host,
            port: config.port,
            username: config.username,
            password: config.password,
            proxy_dns: config.proxy_dns,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Clears the proxy for this tab.
    pub async fn clear_proxy(&self) -> Result<()> {
        let command = Command::Proxy(ProxyCommand::ClearTabProxy);
        self.send_command(command).await?;
        Ok(())
    }
}

// ============================================================================
// Tab - Storage (Cookies)
// ============================================================================

impl Tab {
    /// Gets a cookie by name.
    pub async fn get_cookie(&self, name: &str) -> Result<Option<Cookie>> {
        let command = Command::Storage(StorageCommand::GetCookie {
            name: name.to_string(),
            url: None,
        });

        let response = self.send_command(command).await?;

        let cookie = response
            .result
            .as_ref()
            .and_then(|v| v.get("cookie"))
            .and_then(|v| serde_json::from_value::<Cookie>(v.clone()).ok());

        Ok(cookie)
    }

    /// Sets a cookie.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::Cookie;
    ///
    /// tab.set_cookie(Cookie::new("session", "abc123")).await?;
    /// ```
    pub async fn set_cookie(&self, cookie: Cookie) -> Result<()> {
        let command = Command::Storage(StorageCommand::SetCookie { cookie, url: None });
        self.send_command(command).await?;
        Ok(())
    }

    /// Deletes a cookie by name.
    pub async fn delete_cookie(&self, name: &str) -> Result<()> {
        let command = Command::Storage(StorageCommand::DeleteCookie {
            name: name.to_string(),
            url: None,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Gets all cookies for the current page.
    pub async fn get_all_cookies(&self) -> Result<Vec<Cookie>> {
        let command = Command::Storage(StorageCommand::GetAllCookies { url: None });
        let response = self.send_command(command).await?;

        let cookies = response
            .result
            .as_ref()
            .and_then(|v| v.get("cookies"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<Cookie>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(cookies)
    }
}

// ============================================================================
// Tab - Storage (localStorage)
// ============================================================================

impl Tab {
    /// Gets a value from localStorage.
    pub async fn local_storage_get(&self, key: &str) -> Result<Option<String>> {
        let script = format!("return localStorage.getItem({});", json_string(key));
        let value = self.execute_script(&script).await?;

        match value {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s)),
            _ => Ok(value.as_str().map(|s| s.to_string())),
        }
    }

    /// Sets a value in localStorage.
    pub async fn local_storage_set(&self, key: &str, value: &str) -> Result<()> {
        let script = format!(
            "localStorage.setItem({}, {});",
            json_string(key),
            json_string(value)
        );

        self.execute_script(&script).await?;
        Ok(())
    }

    /// Deletes a key from localStorage.
    pub async fn local_storage_delete(&self, key: &str) -> Result<()> {
        let script = format!("localStorage.removeItem({});", json_string(key));
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Clears all localStorage.
    pub async fn local_storage_clear(&self) -> Result<()> {
        self.execute_script("localStorage.clear();").await?;
        Ok(())
    }
}

// ============================================================================
// Tab - Storage (sessionStorage)
// ============================================================================

impl Tab {
    /// Gets a value from sessionStorage.
    pub async fn session_storage_get(&self, key: &str) -> Result<Option<String>> {
        let script = format!("return sessionStorage.getItem({});", json_string(key));
        let value = self.execute_script(&script).await?;

        match value {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s)),
            _ => Ok(value.as_str().map(|s| s.to_string())),
        }
    }

    /// Sets a value in sessionStorage.
    pub async fn session_storage_set(&self, key: &str, value: &str) -> Result<()> {
        let script = format!(
            "sessionStorage.setItem({}, {});",
            json_string(key),
            json_string(value)
        );

        self.execute_script(&script).await?;
        Ok(())
    }

    /// Deletes a key from sessionStorage.
    pub async fn session_storage_delete(&self, key: &str) -> Result<()> {
        let script = format!("sessionStorage.removeItem({});", json_string(key));
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Clears all sessionStorage.
    pub async fn session_storage_clear(&self) -> Result<()> {
        self.execute_script("sessionStorage.clear();").await?;
        Ok(())
    }
}

// ============================================================================
// Tab - Script Execution
// ============================================================================

impl Tab {
    /// Executes synchronous JavaScript in the page context.
    ///
    /// The script should use `return` to return a value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let title = tab.execute_script("return document.title").await?;
    /// ```
    pub async fn execute_script(&self, script: &str) -> Result<Value> {
        let command = Command::Script(ScriptCommand::Evaluate {
            script: script.to_string(),
            args: vec![],
        });

        let response = self.send_command(command).await?;

        let value = response
            .result
            .as_ref()
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(Value::Null);

        Ok(value)
    }

    /// Executes asynchronous JavaScript in the page context.
    ///
    /// The script should return a Promise or use async/await.
    pub async fn execute_async_script(&self, script: &str) -> Result<Value> {
        let command = Command::Script(ScriptCommand::EvaluateAsync {
            script: script.to_string(),
            args: vec![],
        });

        let response = self.send_command(command).await?;

        let value = response
            .result
            .as_ref()
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(Value::Null);

        Ok(value)
    }
}

// ============================================================================
// Tab - Element Search
// ============================================================================

impl Tab {
    /// Finds a single element by CSS selector.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ElementNotFound`] if no matching element exists.
    pub async fn find_element(&self, selector: &str) -> Result<Element> {
        let command = Command::Element(ElementCommand::Find {
            selector: selector.to_string(),
            parent_id: None,
        });

        let response = self.send_command(command).await?;

        let element_id = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::element_not_found(selector, self.inner.tab_id, self.inner.frame_id)
            })?;

        Ok(Element::new(
            ElementId::new(element_id),
            self.inner.tab_id,
            self.inner.frame_id,
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Finds all elements matching a CSS selector.
    pub async fn find_elements(&self, selector: &str) -> Result<Vec<Element>> {
        let command = Command::Element(ElementCommand::FindAll {
            selector: selector.to_string(),
            parent_id: None,
        });

        let response = self.send_command(command).await?;

        let elements = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementIds"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|id| {
                        Element::new(
                            ElementId::new(id),
                            self.inner.tab_id,
                            self.inner.frame_id,
                            self.inner.session_id,
                            self.inner.window.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(elements)
    }
}

// ============================================================================
// Tab - Element Observation
// ============================================================================

impl Tab {
    /// Waits for an element matching the selector to appear.
    ///
    /// Uses MutationObserver (no polling). Times out after 30 seconds.
    ///
    /// # Errors
    ///
    /// Returns `Timeout` if element doesn't appear within 30 seconds.
    pub async fn wait_for_element(&self, selector: &str) -> Result<Element> {
        self.wait_for_element_timeout(selector, DEFAULT_WAIT_TIMEOUT)
            .await
    }

    /// Waits for an element with a custom timeout.
    ///
    /// # Arguments
    ///
    /// * `selector` - CSS selector to watch for
    /// * `timeout_duration` - Maximum time to wait
    pub async fn wait_for_element_timeout(
        &self,
        selector: &str,
        timeout_duration: Duration,
    ) -> Result<Element> {
        debug!(
            tab_id = %self.inner.tab_id,
            selector,
            timeout_ms = timeout_duration.as_millis(),
            "Waiting for element"
        );

        let window = self.get_window()?;

        let (tx, rx) = oneshot::channel::<Result<Element>>();
        let tx = Arc::new(ParkingMutex::new(Some(tx)));
        let selector_clone = selector.to_string();
        let tab_id = self.inner.tab_id;
        let frame_id = self.inner.frame_id;
        let session_id = self.inner.session_id;
        let window_clone = self.inner.window.clone();
        let tx_clone = Arc::clone(&tx);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "element.added" {
                    return None;
                }

                let parsed = event.parse();
                if let ParsedEvent::ElementAdded {
                    selector: event_selector,
                    element_id,
                    ..
                } = parsed
                    && event_selector == selector_clone
                {
                    let element = Element::new(
                        ElementId::new(&element_id),
                        tab_id,
                        frame_id,
                        session_id,
                        window_clone.clone(),
                    );

                    if let Some(tx) = tx_clone.lock().take() {
                        let _ = tx.send(Ok(element));
                    }
                }

                None
            }),
        );

        let command = Command::Element(ElementCommand::Subscribe {
            selector: selector.to_string(),
            one_shot: true,
        });
        let response = self.send_command(command).await?;

        // Check if element already exists
        if let Some(element_id) = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementId"))
            .and_then(|v| v.as_str())
        {
            window
                .inner
                .pool
                .clear_event_handler(window.inner.session_id);

            return Ok(Element::new(
                ElementId::new(element_id),
                self.inner.tab_id,
                self.inner.frame_id,
                self.inner.session_id,
                self.inner.window.clone(),
            ));
        }

        let result = timeout(timeout_duration, rx).await;

        window
            .inner
            .pool
            .clear_event_handler(window.inner.session_id);

        match result {
            Ok(Ok(element)) => element,
            Ok(Err(_)) => Err(Error::protocol("Channel closed unexpectedly")),
            Err(_) => Err(Error::Timeout {
                operation: format!("wait_for_element({})", selector),
                timeout_ms: timeout_duration.as_millis() as u64,
            }),
        }
    }

    /// Registers a callback for when elements matching the selector appear.
    ///
    /// # Returns
    ///
    /// Subscription ID for later unsubscription.
    pub async fn on_element_added<F>(&self, selector: &str, callback: F) -> Result<SubscriptionId>
    where
        F: Fn(Element) + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, selector, "Subscribing to element.added");

        let window = self.get_window()?;

        let selector_clone = selector.to_string();
        let tab_id = self.inner.tab_id;
        let frame_id = self.inner.frame_id;
        let session_id = self.inner.session_id;
        let window_clone = self.inner.window.clone();
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "element.added" {
                    return None;
                }

                let parsed = event.parse();
                if let ParsedEvent::ElementAdded {
                    selector: event_selector,
                    element_id,
                    ..
                } = parsed
                    && event_selector == selector_clone
                {
                    let element = Element::new(
                        ElementId::new(&element_id),
                        tab_id,
                        frame_id,
                        session_id,
                        window_clone.clone(),
                    );
                    callback(element);
                }

                None
            }),
        );

        let command = Command::Element(ElementCommand::Subscribe {
            selector: selector.to_string(),
            one_shot: false,
        });

        let response = self.send_command(command).await?;

        let subscription_id = response
            .result
            .as_ref()
            .and_then(|v| v.get("subscriptionId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::protocol("No subscriptionId in response"))?;

        Ok(SubscriptionId::new(subscription_id))
    }

    /// Registers a callback for when a specific element is removed.
    pub async fn on_element_removed<F>(&self, element_id: &ElementId, callback: F) -> Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, %element_id, "Watching for element removal");

        let window = self.get_window()?;

        let element_id_clone = element_id.as_str().to_string();
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "element.removed" {
                    return None;
                }

                let parsed = event.parse();
                if let ParsedEvent::ElementRemoved {
                    element_id: removed_id,
                    ..
                } = parsed
                    && removed_id == element_id_clone
                {
                    callback();
                }

                None
            }),
        );

        let command = Command::Element(ElementCommand::WatchRemoval {
            element_id: element_id.clone(),
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Unsubscribes from element observation.
    pub async fn unsubscribe(&self, subscription_id: &SubscriptionId) -> Result<()> {
        let command = Command::Element(ElementCommand::Unsubscribe {
            subscription_id: subscription_id.as_str().to_string(),
        });

        self.send_command(command).await?;

        if let Some(window) = &self.inner.window {
            window
                .inner
                .pool
                .clear_event_handler(window.inner.session_id);
        }

        Ok(())
    }
}

// ============================================================================
// Tab - Internal
// ============================================================================

impl Tab {
    /// Sends a command and returns the response.
    pub(crate) async fn send_command(&self, command: Command) -> Result<Response> {
        let window = self.get_window()?;
        let request = Request::new(self.inner.tab_id, self.inner.frame_id, command);
        window
            .inner
            .pool
            .send(window.inner.session_id, request)
            .await
    }

    /// Gets the window reference or returns an error.
    fn get_window(&self) -> Result<&Window> {
        self.inner
            .window
            .as_ref()
            .ok_or_else(|| Error::protocol("Tab has no associated window"))
    }
}

// ============================================================================
// Private Helpers
// ============================================================================

/// Escapes a string for safe use in JavaScript.
fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
}

/// Extracts frame ID from response.
fn extract_frame_id(response: &Response) -> Result<u64> {
    response
        .result
        .as_ref()
        .and_then(|v| v.get("frameId"))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Error::protocol("No frameId in response"))
}

/// Extracts intercept ID from response.
fn extract_intercept_id(response: &Response) -> Result<InterceptId> {
    let id = response
        .result
        .as_ref()
        .and_then(|v| v.get("interceptId"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::protocol("No interceptId in response"))?;

    Ok(InterceptId::new(id))
}

/// Parses frame info from JSON value.
fn parse_frame_info(v: &Value) -> Option<FrameInfo> {
    Some(FrameInfo {
        frame_id: FrameId::new(v.get("frameId")?.as_u64()?),
        parent_frame_id: v
            .get("parentFrameId")
            .and_then(|p| p.as_i64())
            .and_then(|p| {
                if p < 0 {
                    None
                } else {
                    Some(FrameId::new(p as u64))
                }
            }),
        url: v.get("url")?.as_str()?.to_string(),
    })
}

/// Parses intercepted request from event.
fn parse_intercepted_request(event: &Event) -> InterceptedRequest {
    InterceptedRequest {
        request_id: event
            .params
            .get("requestId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: event
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        method: event
            .params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string(),
        resource_type: event
            .params
            .get("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("other")
            .to_string(),
        tab_id: event
            .params
            .get("tabId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        frame_id: event
            .params
            .get("frameId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        body: None,
    }
}

/// Parses intercepted request headers from event.
fn parse_intercepted_request_headers(event: &Event) -> InterceptedRequestHeaders {
    InterceptedRequestHeaders {
        request_id: event
            .params
            .get("requestId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: event
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        method: event
            .params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string(),
        headers: event
            .params
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default(),
        tab_id: event
            .params
            .get("tabId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        frame_id: event
            .params
            .get("frameId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    }
}

/// Parses intercepted request body from event.
fn parse_intercepted_request_body(event: &Event) -> InterceptedRequestBody {
    InterceptedRequestBody {
        request_id: event
            .params
            .get("requestId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: event
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        method: event
            .params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_string(),
        resource_type: event
            .params
            .get("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("other")
            .to_string(),
        tab_id: event
            .params
            .get("tabId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        frame_id: event
            .params
            .get("frameId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        body: event.params.as_object().and_then(parse_request_body),
    }
}

/// Parses intercepted response from event.
fn parse_intercepted_response(event: &Event) -> InterceptedResponse {
    InterceptedResponse {
        request_id: event
            .params
            .get("requestId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: event
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        status: event
            .params
            .get("status")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16,
        status_text: event
            .params
            .get("statusText")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        headers: event
            .params
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default(),
        tab_id: event
            .params
            .get("tabId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        frame_id: event
            .params
            .get("frameId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    }
}

/// Parses intercepted response body from event.
fn parse_intercepted_response_body(event: &Event) -> InterceptedResponseBody {
    InterceptedResponseBody {
        request_id: event
            .params
            .get("requestId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        url: event
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        tab_id: event
            .params
            .get("tabId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        frame_id: event
            .params
            .get("frameId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        body: event
            .params
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        content_length: event
            .params
            .get("contentLength")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
    }
}

/// Parses request body from event params.
fn parse_request_body(params: &serde_json::Map<String, Value>) -> Option<RequestBody> {
    let body = params.get("body")?;
    let body_obj = body.as_object()?;

    if let Some(error) = body_obj.get("error").and_then(|v| v.as_str()) {
        return Some(RequestBody::Error(error.to_string()));
    }

    if let Some(form_data) = body_obj.get("data").and_then(|v| v.as_object())
        && body_obj.get("type").and_then(|v| v.as_str()) == Some("formData")
    {
        let mut map = std::collections::HashMap::new();
        for (key, value) in form_data {
            if let Some(arr) = value.as_array() {
                let values: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                map.insert(key.clone(), values);
            }
        }
        return Some(RequestBody::FormData(map));
    }

    if let Some(raw_data) = body_obj.get("data").and_then(|v| v.as_array())
        && body_obj.get("type").and_then(|v| v.as_str()) == Some("raw")
    {
        let mut bytes = Vec::new();
        for item in raw_data {
            if let Some(obj) = item.as_object()
                && let Some(b64) = obj.get("data").and_then(|v| v.as_str())
                && let Ok(decoded) = Base64Standard.decode(b64)
            {
                bytes.extend(decoded);
            }
        }
        if !bytes.is_empty() {
            return Some(RequestBody::Raw(bytes));
        }
    }

    None
}

/// Converts request action to JSON.
fn request_action_to_json(action: &RequestAction) -> Value {
    match action {
        RequestAction::Allow => serde_json::json!({ "action": "allow" }),
        RequestAction::Block => serde_json::json!({ "action": "block" }),
        RequestAction::Redirect(url) => serde_json::json!({ "action": "redirect", "url": url }),
    }
}

/// Converts headers action to JSON.
fn headers_action_to_json(action: &HeadersAction) -> Value {
    match action {
        HeadersAction::Allow => serde_json::json!({ "action": "allow" }),
        HeadersAction::ModifyHeaders(h) => {
            serde_json::json!({ "action": "modifyHeaders", "headers": h })
        }
    }
}

/// Converts body action to JSON.
fn body_action_to_json(action: &BodyAction) -> Value {
    match action {
        BodyAction::Allow => serde_json::json!({ "action": "allow" }),
        BodyAction::ModifyBody(b) => serde_json::json!({ "action": "modifyBody", "body": b }),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::Tab;

    #[test]
    fn test_tab_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<Tab>();
    }

    #[test]
    fn test_tab_is_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<Tab>();
    }
}
