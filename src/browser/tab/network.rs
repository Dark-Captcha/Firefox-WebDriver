//! Network interception and blocking methods.

use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64Standard;
use serde_json::Value;
use tracing::debug;

use crate::browser::network::{
    BodyAction, HeadersAction, InterceptedRequest, InterceptedRequestBody,
    InterceptedRequestHeaders, InterceptedResponse, InterceptedResponseBody, RequestAction,
    RequestBody,
};
use crate::error::{Error, Result};
use crate::identifiers::InterceptId;
use crate::protocol::{Command, Event, EventReply, NetworkCommand, Response};

use super::Tab;

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
        debug!(tab_id = %self.inner.tab_id, "Clearing block rules");
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
// Helper Functions
// ============================================================================

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
        let mut map = HashMap::new();
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
