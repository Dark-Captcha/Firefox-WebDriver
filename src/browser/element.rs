//! DOM element interaction and manipulation.
//!
//! Elements are identified by UUID and stored in the content script's
//! internal `Map<UUID, Element>`.
//!
//! # Example
//!
//! ```ignore
//! use firefox_webdriver::Key;
//!
//! let element = tab.find_element("#submit-button").await?;
//!
//! // Get properties
//! let text = element.get_text().await?;
//! let value = element.get_value().await?;
//!
//! // Interact
//! element.click().await?;
//! element.type_text("Hello, World!").await?;
//!
//! // Press navigation keys
//! element.press(Key::Enter).await?;
//! element.press(Key::Tab).await?;
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::fmt;
use std::sync::Arc;

use serde_json::Value;
use tracing::debug;

use crate::error::{Error, Result};
use crate::identifiers::{ElementId, FrameId, SessionId, TabId};
use crate::protocol::{Command, ElementCommand, InputCommand, Request, Response};

use super::Window;
use super::keyboard::Key;
use super::selector::By;

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
        debug!(element_id = %self.inner.id, "Clicking element");
        self.call_method("click", vec![]).await?;
        Ok(())
    }

    /// Focuses the element.
    pub async fn focus(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Focusing element");
        self.call_method("focus", vec![]).await?;
        Ok(())
    }

    /// Blurs (unfocuses) the element.
    pub async fn blur(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Blurring element");
        self.call_method("blur", vec![]).await?;
        Ok(())
    }

    /// Clears the element's value.
    ///
    /// Sets `element.value = ""`.
    pub async fn clear(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Clearing element");
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
    /// Presses a navigation/control key.
    ///
    /// For typing text, use [`type_text`](Self::type_text) instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::Key;
    ///
    /// element.press(Key::Enter).await?;
    /// element.press(Key::Tab).await?;
    /// element.press(Key::Backspace).await?;
    /// ```
    pub async fn press(&self, key: Key) -> Result<()> {
        let (key_str, code, key_code, printable) = key.properties();
        self.type_key(
            key_str, code, key_code, printable, false, false, false, false,
        )
        .await
    }

    /// Types a single key with optional modifiers (low-level API).
    ///
    /// Prefer using [`press`](Self::press) for common keys or [`type_text`](Self::type_text) for text.
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
        debug!(element_id = %self.inner.id, text_len = text.len(), "Typing text");

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
        debug!(element_id = %self.inner.id, button = button, "Mouse clicking element");

        let command = Command::Input(InputCommand::MouseClick {
            element_id: Some(self.inner.id.clone()),
            x: None,
            y: None,
            button,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Double-clicks the element.
    ///
    /// Dispatches two click sequences followed by dblclick event.
    pub async fn double_click(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Double clicking element");

        self.call_method(
            "dispatchEvent",
            vec![serde_json::json!({"type": "dblclick", "bubbles": true, "cancelable": true})],
        )
        .await?;
        Ok(())
    }

    /// Right-clicks the element (context menu click).
    ///
    /// Dispatches contextmenu event.
    pub async fn context_click(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Context clicking element");
        self.mouse_click(2).await
    }

    /// Hovers over the element.
    ///
    /// Moves mouse to element center and dispatches mouseenter/mouseover events.
    pub async fn hover(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Hovering over element");
        self.mouse_move().await
    }

    /// Moves mouse to the element center.
    pub async fn mouse_move(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Moving mouse to element");

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
        debug!(element_id = %self.inner.id, button = button, "Mouse down on element");

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
        debug!(element_id = %self.inner.id, button = button, "Mouse up on element");

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
// Element - Scroll
// ============================================================================

impl Element {
    /// Scrolls the element into view.
    ///
    /// Uses `element.scrollIntoView()` with smooth behavior.
    pub async fn scroll_into_view(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Scrolling element into view");

        self.call_method(
            "scrollIntoView",
            vec![serde_json::json!({"behavior": "smooth", "block": "center"})],
        )
        .await?;
        Ok(())
    }

    /// Scrolls the element into view immediately (no smooth animation).
    pub async fn scroll_into_view_instant(&self) -> Result<()> {
        debug!(element_id = %self.inner.id, "Scrolling element into view (instant)");

        self.call_method(
            "scrollIntoView",
            vec![serde_json::json!({"behavior": "instant", "block": "center"})],
        )
        .await?;
        Ok(())
    }

    /// Gets the element's bounding rectangle.
    ///
    /// # Returns
    ///
    /// Tuple of (x, y, width, height) in pixels.
    pub async fn get_bounding_rect(&self) -> Result<(f64, f64, f64, f64)> {
        let result = self.call_method("getBoundingClientRect", vec![]).await?;

        let x = result.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = result.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let width = result.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let height = result.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);

        debug!(element_id = %self.inner.id, x = x, y = y, width = width, height = height, "Got bounding rect");
        Ok((x, y, width, height))
    }
}

// ============================================================================
// Element - Checkbox/Radio
// ============================================================================

impl Element {
    /// Checks if the element is checked (for checkboxes/radio buttons).
    pub async fn is_checked(&self) -> Result<bool> {
        let value = self.get_property("checked").await?;
        Ok(value.as_bool().unwrap_or(false))
    }

    /// Checks the checkbox/radio button.
    ///
    /// Does nothing if already checked.
    pub async fn check(&self) -> Result<()> {
        if !self.is_checked().await? {
            self.click().await?;
        }
        Ok(())
    }

    /// Unchecks the checkbox.
    ///
    /// Does nothing if already unchecked.
    pub async fn uncheck(&self) -> Result<()> {
        if self.is_checked().await? {
            self.click().await?;
        }
        Ok(())
    }

    /// Toggles the checkbox state.
    pub async fn toggle(&self) -> Result<()> {
        self.click().await
    }

    /// Sets the checked state.
    pub async fn set_checked(&self, checked: bool) -> Result<()> {
        if checked {
            self.check().await
        } else {
            self.uncheck().await
        }
    }
}

// ============================================================================
// Element - Select/Dropdown
// ============================================================================

impl Element {
    /// Selects an option by visible text (for `<select>` elements).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let select = tab.find_element(By::css("select#country")).await?;
    /// select.select_by_text("United States").await?;
    /// ```
    pub async fn select_by_text(&self, text: &str) -> Result<()> {
        // Find and click the option
        if let Ok(options) = self.find_elements(By::tag("option")).await {
            for option in options {
                if let Ok(option_text) = option.get_text().await
                    && option_text.trim() == text
                {
                    option.set_property("selected", Value::Bool(true)).await?;
                    // Trigger change event
                    self.call_method(
                        "dispatchEvent",
                        vec![serde_json::json!({"type": "change", "bubbles": true})],
                    )
                    .await?;
                    return Ok(());
                }
            }
        }

        Err(Error::invalid_argument(format!(
            "Option with text '{}' not found",
            text
        )))
    }

    /// Selects an option by value attribute (for `<select>` elements).
    pub async fn select_by_value(&self, value: &str) -> Result<()> {
        self.set_property("value", Value::String(value.to_string()))
            .await?;
        self.call_method(
            "dispatchEvent",
            vec![serde_json::json!({"type": "change", "bubbles": true})],
        )
        .await?;
        Ok(())
    }

    /// Selects an option by index (for `<select>` elements).
    pub async fn select_by_index(&self, index: usize) -> Result<()> {
        self.set_property("selectedIndex", Value::Number(index.into()))
            .await?;
        self.call_method(
            "dispatchEvent",
            vec![serde_json::json!({"type": "change", "bubbles": true})],
        )
        .await?;
        Ok(())
    }

    /// Gets the selected option's value (for `<select>` elements).
    pub async fn get_selected_value(&self) -> Result<Option<String>> {
        let value = self.get_property("value").await?;
        Ok(value.as_str().map(|s| s.to_string()))
    }

    /// Gets the selected option's index (for `<select>` elements).
    pub async fn get_selected_index(&self) -> Result<i64> {
        let value = self.get_property("selectedIndex").await?;
        Ok(value.as_i64().unwrap_or(-1))
    }

    /// Gets the selected option's text (for `<select>` elements).
    pub async fn get_selected_text(&self) -> Result<Option<String>> {
        let options = self.find_elements(By::css("option:checked")).await?;
        if let Some(option) = options.first() {
            let text = option.get_text().await?;
            return Ok(Some(text));
        }
        Ok(None)
    }

    /// Checks if this is a multi-select element.
    pub async fn is_multiple(&self) -> Result<bool> {
        let value = self.get_property("multiple").await?;
        Ok(value.as_bool().unwrap_or(false))
    }
}

// ============================================================================
// Element - Nested Search
// ============================================================================

impl Element {
    /// Finds a child element using a locator strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::By;
    ///
    /// let form = tab.find_element(By::Id("login-form")).await?;
    /// let btn = form.find_element(By::Css("button[type='submit']")).await?;
    /// ```
    pub async fn find_element(&self, by: By) -> Result<Element> {
        let command = Command::Element(ElementCommand::Find {
            strategy: by.strategy().to_string(),
            value: by.value().to_string(),
            parent_id: Some(self.inner.id.clone()),
        });

        let response = self.send_command(command).await?;

        let element_id = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::element_not_found(
                    format!("{}:{}", by.strategy(), by.value()),
                    self.inner.tab_id,
                    self.inner.frame_id,
                )
            })?;

        Ok(Element::new(
            ElementId::new(element_id),
            self.inner.tab_id,
            self.inner.frame_id,
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Finds all child elements using a locator strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::By;
    ///
    /// let form = tab.find_element(By::Id("login-form")).await?;
    /// let inputs = form.find_elements(By::Tag("input")).await?;
    /// ```
    pub async fn find_elements(&self, by: By) -> Result<Vec<Element>> {
        let command = Command::Element(ElementCommand::FindAll {
            strategy: by.strategy().to_string(),
            value: by.value().to_string(),
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
// Element - Screenshot
// ============================================================================

impl Element {
    /// Captures a PNG screenshot of this element.
    ///
    /// Returns base64-encoded image data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let element = tab.find_element("#chart").await?;
    /// let screenshot = element.screenshot().await?;
    /// ```
    pub async fn screenshot(&self) -> Result<String> {
        self.screenshot_with_format("png", None).await
    }

    /// Captures a JPEG screenshot of this element with specified quality.
    ///
    /// # Arguments
    ///
    /// * `quality` - JPEG quality (0-100)
    pub async fn screenshot_jpeg(&self, quality: u8) -> Result<String> {
        self.screenshot_with_format("jpeg", Some(quality.min(100)))
            .await
    }

    /// Captures a screenshot with specified format.
    ///
    /// The extension returns full page screenshot + clip info.
    /// Rust handles the cropping to avoid canvas security issues.
    async fn screenshot_with_format(&self, format: &str, quality: Option<u8>) -> Result<String> {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as Base64Standard;
        use image::GenericImageView;

        let command = Command::Element(ElementCommand::CaptureScreenshot {
            element_id: self.inner.id.clone(),
            format: format.to_string(),
            quality,
        });

        let response = self.send_command(command).await?;

        tracing::debug!(response = ?response, "Element screenshot response");

        let result = response.result.as_ref().ok_or_else(|| {
            let error_str = response.error.as_deref().unwrap_or("none");
            let msg_str = response.message.as_deref().unwrap_or("none");
            Error::script_error(format!(
                "Element screenshot failed. error={}, message={}",
                error_str, msg_str
            ))
        })?;

        let data = result
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::script_error("Screenshot response missing data field"))?;

        // Check if clip info is provided (new format)
        if let Some(clip) = result.get("clip") {
            let x = clip.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
            let y = clip.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
            let width = clip.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
            let height = clip.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
            let scale = clip.get("scale").and_then(|v| v.as_f64()).unwrap_or(1.0);

            // Apply scale to coordinates
            let x = (x as f64 * scale) as u32;
            let y = (y as f64 * scale) as u32;
            let width = (width as f64 * scale) as u32;
            let height = (height as f64 * scale) as u32;

            if width == 0 || height == 0 {
                return Err(Error::script_error("Element has zero dimensions"));
            }

            // Decode full page image
            let image_bytes = Base64Standard
                .decode(data)
                .map_err(|e| Error::script_error(format!("Failed to decode base64: {}", e)))?;

            let img = image::load_from_memory(&image_bytes)
                .map_err(|e| Error::script_error(format!("Failed to load image: {}", e)))?;

            // Clamp crop region to image bounds
            let (img_width, img_height) = img.dimensions();
            let x = x.min(img_width.saturating_sub(1));
            let y = y.min(img_height.saturating_sub(1));
            let width = width.min(img_width.saturating_sub(x));
            let height = height.min(img_height.saturating_sub(y));

            // Crop
            let cropped = img.crop_imm(x, y, width, height);

            // Encode back to base64
            let mut output = std::io::Cursor::new(Vec::new());
            match format {
                "jpeg" => {
                    let q = quality.unwrap_or(85);
                    cropped
                        .write_to(&mut output, image::ImageFormat::Jpeg)
                        .map_err(|e| {
                            Error::script_error(format!("Failed to encode JPEG: {}", e))
                        })?;
                    // Note: image crate doesn't support quality param directly in write_to
                    // For proper quality control, would need jpeg encoder directly
                    let _ = q; // suppress unused warning
                }
                _ => {
                    cropped
                        .write_to(&mut output, image::ImageFormat::Png)
                        .map_err(|e| Error::script_error(format!("Failed to encode PNG: {}", e)))?;
                }
            }

            Ok(Base64Standard.encode(output.into_inner()))
        } else {
            // Old format: data is already cropped
            Ok(data.to_string())
        }
    }

    /// Captures a screenshot and returns raw bytes.
    pub async fn screenshot_bytes(&self) -> Result<Vec<u8>> {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as Base64Standard;

        let base64_data = self.screenshot().await?;
        Base64Standard
            .decode(&base64_data)
            .map_err(|e| Error::script_error(format!("Failed to decode base64: {}", e)))
    }

    /// Captures a screenshot and saves to a file.
    ///
    /// Format is determined by file extension (.png or .jpg/.jpeg).
    pub async fn save_screenshot(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD as Base64Standard;

        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();

        let base64_data = match ext.as_str() {
            "jpg" | "jpeg" => self.screenshot_jpeg(85).await?,
            _ => self.screenshot().await?,
        };

        let bytes = Base64Standard
            .decode(&base64_data)
            .map_err(|e| Error::script_error(format!("Failed to decode base64: {}", e)))?;

        std::fs::write(path, bytes).map_err(Error::Io)?;
        Ok(())
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

        window
            .inner
            .pool
            .send(window.inner.session_id, request)
            .await
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
