//! DOM element interaction and manipulation.
//!
//! Elements are identified by UUID and stored in the content script's
//! internal `Map<UUID, Element>`.
//!
//! # Example
//!
//! ```ignore
//! let element = tab.find_element("#submit-button").await?;
//!
//! // Get properties
//! let text = element.get_text().await?;
//! let value = element.get_value().await?;
//!
//! // Interact
//! element.click().await?;
//! element.type_text("Hello, World!").await?;
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::fmt;
use std::sync::Arc;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::identifiers::{ElementId, FrameId, SessionId, TabId};
use crate::protocol::{Command, ElementCommand, InputCommand, Request, Response};

use super::Window;

// ============================================================================
// Types
// ============================================================================

/// Internal shared state for an element.
pub(crate) struct ElementInner {
    /// This element's unique ID.
    pub id: ElementId,

    /// Tab ID where this element exists.
    pub tab_id: TabId,

    /// Frame ID where this element exists.
    pub frame_id: FrameId,

    /// Session ID.
    pub session_id: SessionId,

    /// Parent window.
    pub window: Option<Window>,
}

// ============================================================================
// Element
// ============================================================================

/// A handle to a DOM element in a browser tab.
///
/// Elements are identified by a UUID stored in the extension's content script.
/// Operations use generic dynamic property access (`element[method]()`).
///
/// # Example
///
/// ```ignore
/// let element = tab.find_element("input[name='email']").await?;
///
/// // Set value and submit
/// element.set_value("user@example.com").await?;
/// element.type_text("\n").await?; // Press Enter
/// ```
#[derive(Clone)]
pub struct Element {
    /// Shared inner state.
    pub(crate) inner: Arc<ElementInner>,
}

// ============================================================================
// Element - Display
// ============================================================================

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("id", &self.inner.id)
            .field("tab_id", &self.inner.tab_id)
            .field("frame_id", &self.inner.frame_id)
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Element - Constructor
// ============================================================================

impl Element {
    /// Creates a new element handle.
    pub(crate) fn new(
        id: ElementId,
        tab_id: TabId,
        frame_id: FrameId,
        session_id: SessionId,
        window: Option<Window>,
    ) -> Self {
        Self {
            inner: Arc::new(ElementInner {
                id,
                tab_id,
                frame_id,
                session_id,
                window,
            }),
        }
    }
}

// ============================================================================
// Element - Accessors
// ============================================================================

impl Element {
    /// Returns this element's ID.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &ElementId {
        &self.inner.id
    }

    /// Returns the tab ID where this element exists.
    #[inline]
    #[must_use]
    pub fn tab_id(&self) -> TabId {
        self.inner.tab_id
    }

    /// Returns the frame ID where this element exists.
    #[inline]
    #[must_use]
    pub fn frame_id(&self) -> FrameId {
        self.inner.frame_id
    }
}

// ============================================================================
// Element - Actions
// ============================================================================

impl Element {
    /// Clicks the element.
    ///
    /// Uses `element.click()` internally.
    pub async fn click(&self) -> Result<()> {
        self.call_method("click", vec![]).await?;
        Ok(())
    }

    /// Focuses the element.
    pub async fn focus(&self) -> Result<()> {
        self.call_method("focus", vec![]).await?;
        Ok(())
    }

    /// Blurs (unfocuses) the element.
    pub async fn blur(&self) -> Result<()> {
        self.call_method("blur", vec![]).await?;
        Ok(())
    }

    /// Clears the element's value.
    ///
    /// Sets `element.value = ""`.
    pub async fn clear(&self) -> Result<()> {
        self.set_property("value", Value::String(String::new()))
            .await
    }
}

// ============================================================================
// Element - Properties
// ============================================================================

impl Element {
    /// Gets the element's text content.
    pub async fn get_text(&self) -> Result<String> {
        let value = self.get_property("textContent").await?;
        Ok(value.as_str().unwrap_or("").to_string())
    }

    /// Gets the element's inner HTML.
    pub async fn get_inner_html(&self) -> Result<String> {
        let value = self.get_property("innerHTML").await?;
        Ok(value.as_str().unwrap_or("").to_string())
    }

    /// Gets the element's value (for input elements).
    pub async fn get_value(&self) -> Result<String> {
        let value = self.get_property("value").await?;
        Ok(value.as_str().unwrap_or("").to_string())
    }

    /// Sets the element's value (for input elements).
    pub async fn set_value(&self, value: &str) -> Result<()> {
        self.set_property("value", Value::String(value.to_string()))
            .await
    }

    /// Gets an attribute value.
    ///
    /// Returns `None` if the attribute doesn't exist.
    pub async fn get_attribute(&self, name: &str) -> Result<Option<String>> {
        let result = self
            .call_method("getAttribute", vec![Value::String(name.to_string())])
            .await?;
        Ok(result.as_str().map(|s| s.to_string()))
    }

    /// Checks if the element is displayed.
    ///
    /// Returns `false` if `offsetParent` is null (element is hidden).
    pub async fn is_displayed(&self) -> Result<bool> {
        let offset_parent = self.get_property("offsetParent").await?;
        Ok(!offset_parent.is_null())
    }

    /// Checks if the element is enabled.
    ///
    /// Returns `true` if `disabled` property is false or absent.
    pub async fn is_enabled(&self) -> Result<bool> {
        let disabled = self.get_property("disabled").await?;
        Ok(!disabled.as_bool().unwrap_or(false))
    }
}

// ============================================================================
// Element - Keyboard Input
// ============================================================================

impl Element {
    /// Types a single key with optional modifiers.
    ///
    /// Dispatches full keyboard event sequence: keydown → input → keypress → keyup.
    ///
    /// # Arguments
    ///
    /// * `key` - Key value (e.g., "a", "Enter")
    /// * `code` - Key code (e.g., "KeyA", "Enter")
    /// * `key_code` - Legacy keyCode number
    /// * `printable` - Whether key produces visible output
    /// * `ctrl` - Ctrl modifier
    /// * `shift` - Shift modifier
    /// * `alt` - Alt modifier
    /// * `meta` - Meta modifier
    #[allow(clippy::too_many_arguments)]
    pub async fn type_key(
        &self,
        key: &str,
        code: &str,
        key_code: u32,
        printable: bool,
        ctrl: bool,
        shift: bool,
        alt: bool,
        meta: bool,
    ) -> Result<()> {
        let command = Command::Input(InputCommand::TypeKey {
            element_id: self.inner.id.clone(),
            key: key.to_string(),
            code: code.to_string(),
            key_code,
            printable,
            ctrl,
            shift,
            alt,
            meta,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Types a character with default key properties.
    ///
    /// Convenience method that uses `type_text` internally for reliability.
    pub async fn type_char(&self, c: char) -> Result<()> {
        self.type_text(&c.to_string()).await
    }

    /// Types a text string character by character.
    ///
    /// Each character goes through full keyboard event sequence.
    /// This is slower but more realistic than `set_value`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// element.type_text("Hello, World!").await?;
    /// ```
    pub async fn type_text(&self, text: &str) -> Result<()> {
        let command = Command::Input(InputCommand::TypeText {
            element_id: self.inner.id.clone(),
            text: text.to_string(),
        });

        self.send_command(command).await?;
        Ok(())
    }
}

// ============================================================================
// Element - Mouse Input
// ============================================================================

impl Element {
    /// Clicks the element using mouse events.
    ///
    /// Dispatches: mousemove → mousedown → mouseup → click.
    /// This is more realistic than `click()` which uses `element.click()`.
    ///
    /// # Arguments
    ///
    /// * `button` - Mouse button (0=left, 1=middle, 2=right)
    pub async fn mouse_click(&self, button: u8) -> Result<()> {
        let command = Command::Input(InputCommand::MouseClick {
            element_id: Some(self.inner.id.clone()),
            x: None,
            y: None,
            button,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Moves mouse to the element center.
    pub async fn mouse_move(&self) -> Result<()> {
        let command = Command::Input(InputCommand::MouseMove {
            element_id: Some(self.inner.id.clone()),
            x: None,
            y: None,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Presses mouse button down on the element (without release).
    ///
    /// Dispatches only mousedown event.
    /// Use with `mouse_up()` for drag operations.
    ///
    /// # Arguments
    ///
    /// * `button` - Mouse button (0=left, 1=middle, 2=right)
    pub async fn mouse_down(&self, button: u8) -> Result<()> {
        let command = Command::Input(InputCommand::MouseDown {
            element_id: Some(self.inner.id.clone()),
            x: None,
            y: None,
            button,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Releases mouse button on the element.
    ///
    /// Dispatches only mouseup event.
    /// Use with `mouse_down()` for drag operations.
    ///
    /// # Arguments
    ///
    /// * `button` - Mouse button (0=left, 1=middle, 2=right)
    pub async fn mouse_up(&self, button: u8) -> Result<()> {
        let command = Command::Input(InputCommand::MouseUp {
            element_id: Some(self.inner.id.clone()),
            x: None,
            y: None,
            button,
        });

        self.send_command(command).await?;
        Ok(())
    }
}

// ============================================================================
// Element - Nested Search
// ============================================================================

impl Element {
    /// Finds a child element by CSS selector.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ElementNotFound`] if no matching element exists.
    pub async fn find_element(&self, selector: &str) -> Result<Element> {
        let command = Command::Element(ElementCommand::Find {
            selector: selector.to_string(),
            parent_id: Some(self.inner.id.clone()),
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

    /// Finds all child elements matching a CSS selector.
    ///
    /// Returns an empty vector if no elements match.
    pub async fn find_elements(&self, selector: &str) -> Result<Vec<Element>> {
        let command = Command::Element(ElementCommand::FindAll {
            selector: selector.to_string(),
            parent_id: Some(self.inner.id.clone()),
        });

        let response = self.send_command(command).await?;

        let element_ids = response
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

        Ok(element_ids)
    }
}

// ============================================================================
// Element - Generic Property Access
// ============================================================================

impl Element {
    /// Gets a property value via `element[name]`.
    ///
    /// # Arguments
    ///
    /// * `name` - Property name (e.g., "value", "textContent")
    pub async fn get_property(&self, name: &str) -> Result<Value> {
        let command = Command::Element(ElementCommand::GetProperty {
            element_id: self.inner.id.clone(),
            name: name.to_string(),
        });

        let response = self.send_command(command).await?;

        Ok(response
            .result
            .and_then(|v| v.get("value").cloned())
            .unwrap_or(Value::Null))
    }

    /// Sets a property value via `element[name] = value`.
    ///
    /// # Arguments
    ///
    /// * `name` - Property name
    /// * `value` - Value to set
    pub async fn set_property(&self, name: &str, value: Value) -> Result<()> {
        let command = Command::Element(ElementCommand::SetProperty {
            element_id: self.inner.id.clone(),
            name: name.to_string(),
            value,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Calls a method via `element[name](...args)`.
    ///
    /// # Arguments
    ///
    /// * `name` - Method name
    /// * `args` - Method arguments
    pub async fn call_method(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        let command = Command::Element(ElementCommand::CallMethod {
            element_id: self.inner.id.clone(),
            name: name.to_string(),
            args,
        });

        let response = self.send_command(command).await?;

        Ok(response
            .result
            .and_then(|v| v.get("value").cloned())
            .unwrap_or(Value::Null))
    }
}

// ============================================================================
// Element - Internal
// ============================================================================

impl Element {
    /// Sends a command and returns the response.
    async fn send_command(&self, command: Command) -> Result<Response> {
        let window = self
            .inner
            .window
            .as_ref()
            .ok_or_else(|| Error::protocol("Element has no associated window"))?;

        let request = Request::new(self.inner.tab_id, self.inner.frame_id, command);

        window.inner.connection.send(request).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::Element;

    #[test]
    fn test_element_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<Element>();
    }

    #[test]
    fn test_element_is_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<Element>();
    }
}
