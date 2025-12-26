//! Element locator strategies.
//!
//! Provides Selenium-like `By` selectors for finding elements.
//!
//! # Example
//!
//! ```ignore
//! use firefox_webdriver::By;
//!
//! // CSS selector (default)
//! let btn = tab.find_element(By::Css("#submit")).await?;
//!
//! // By ID (shorthand for CSS #id)
//! let form = tab.find_element(By::Id("login-form")).await?;
//!
//! // By text content
//! let link = tab.find_element(By::Text("Click here")).await?;
//!
//! // By partial text
//! let link = tab.find_element(By::PartialText("Click")).await?;
//!
//! // By XPath
//! let btn = tab.find_element(By::XPath("//button[@type='submit']")).await?;
//!
//! // By tag name
//! let inputs = tab.find_elements(By::Tag("input")).await?;
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// By Enum
// ============================================================================

/// Element locator strategy (like Selenium's `By`).
///
/// Supports multiple strategies for finding elements in the DOM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "strategy", content = "value")]
pub enum By {
    /// CSS selector (most common).
    ///
    /// # Example
    /// ```ignore
    /// By::Css("#login-button")
    /// By::Css("button.primary")
    /// By::Css("[data-testid='submit']")
    /// ```
    #[serde(rename = "css")]
    Css(String),

    /// XPath expression.
    ///
    /// # Example
    /// ```ignore
    /// By::XPath("//button[@type='submit']")
    /// By::XPath("//div[contains(@class, 'modal')]")
    /// By::XPath("//a[text()='Login']")
    /// ```
    #[serde(rename = "xpath")]
    XPath(String),

    /// Exact text content match.
    ///
    /// Finds element where `textContent.trim() === value`.
    ///
    /// # Example
    /// ```ignore
    /// By::Text("Submit")
    /// By::Text("Click here to continue")
    /// ```
    #[serde(rename = "text")]
    Text(String),

    /// Partial text content match.
    ///
    /// Finds element where `textContent.includes(value)`.
    ///
    /// # Example
    /// ```ignore
    /// By::PartialText("Submit")
    /// By::PartialText("continue")
    /// ```
    #[serde(rename = "partialText")]
    PartialText(String),

    /// Element ID (shorthand for `#id` CSS selector).
    ///
    /// # Example
    /// ```ignore
    /// By::Id("username")  // equivalent to By::Css("#username")
    /// ```
    #[serde(rename = "id")]
    Id(String),

    /// Tag name.
    ///
    /// # Example
    /// ```ignore
    /// By::Tag("button")
    /// By::Tag("input")
    /// ```
    #[serde(rename = "tag")]
    Tag(String),

    /// Name attribute.
    ///
    /// # Example
    /// ```ignore
    /// By::Name("email")  // equivalent to By::Css("[name='email']")
    /// ```
    #[serde(rename = "name")]
    Name(String),

    /// Class name (single class).
    ///
    /// # Example
    /// ```ignore
    /// By::Class("btn-primary")  // equivalent to By::Css(".btn-primary")
    /// ```
    #[serde(rename = "class")]
    Class(String),

    /// Link text (for `<a>` elements).
    ///
    /// # Example
    /// ```ignore
    /// By::LinkText("Home")
    /// ```
    #[serde(rename = "linkText")]
    LinkText(String),

    /// Partial link text (for `<a>` elements).
    ///
    /// # Example
    /// ```ignore
    /// By::PartialLinkText("Read more")
    /// ```
    #[serde(rename = "partialLinkText")]
    PartialLinkText(String),
}

impl By {
    /// Creates a CSS selector.
    #[inline]
    pub fn css(selector: impl Into<String>) -> Self {
        Self::Css(selector.into())
    }

    /// Creates an XPath selector.
    #[inline]
    pub fn xpath(expr: impl Into<String>) -> Self {
        Self::XPath(expr.into())
    }

    /// Creates a text content selector.
    #[inline]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Creates a partial text content selector.
    #[inline]
    pub fn partial_text(text: impl Into<String>) -> Self {
        Self::PartialText(text.into())
    }

    /// Creates an ID selector.
    #[inline]
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    /// Creates a tag name selector.
    #[inline]
    pub fn tag(tag: impl Into<String>) -> Self {
        Self::Tag(tag.into())
    }

    /// Creates a name attribute selector.
    #[inline]
    pub fn name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// Creates a class name selector.
    #[inline]
    pub fn class(class: impl Into<String>) -> Self {
        Self::Class(class.into())
    }

    /// Creates a link text selector.
    #[inline]
    pub fn link_text(text: impl Into<String>) -> Self {
        Self::LinkText(text.into())
    }

    /// Creates a partial link text selector.
    #[inline]
    pub fn partial_link_text(text: impl Into<String>) -> Self {
        Self::PartialLinkText(text.into())
    }

    /// Returns the strategy name for the protocol.
    #[must_use]
    pub fn strategy(&self) -> &'static str {
        match self {
            Self::Css(_) => "css",
            Self::XPath(_) => "xpath",
            Self::Text(_) => "text",
            Self::PartialText(_) => "partialText",
            Self::Id(_) => "id",
            Self::Tag(_) => "tag",
            Self::Name(_) => "name",
            Self::Class(_) => "class",
            Self::LinkText(_) => "linkText",
            Self::PartialLinkText(_) => "partialLinkText",
        }
    }

    /// Returns the selector value.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::Css(v)
            | Self::XPath(v)
            | Self::Text(v)
            | Self::PartialText(v)
            | Self::Id(v)
            | Self::Tag(v)
            | Self::Name(v)
            | Self::Class(v)
            | Self::LinkText(v)
            | Self::PartialLinkText(v) => v,
        }
    }
}

// ============================================================================
// From implementations for ergonomics
// ============================================================================

impl From<&str> for By {
    /// Converts a string to CSS selector (default).
    fn from(s: &str) -> Self {
        Self::Css(s.to_string())
    }
}

impl From<String> for By {
    /// Converts a string to CSS selector (default).
    fn from(s: String) -> Self {
        Self::Css(s)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_by_css() {
        let by = By::Css("#login".to_string());
        assert_eq!(by.strategy(), "css");
        assert_eq!(by.value(), "#login");
    }

    #[test]
    fn test_by_id() {
        let by = By::Id("username".to_string());
        assert_eq!(by.strategy(), "id");
        assert_eq!(by.value(), "username");
    }

    #[test]
    fn test_by_xpath() {
        let by = By::XPath("//button".to_string());
        assert_eq!(by.strategy(), "xpath");
        assert_eq!(by.value(), "//button");
    }

    #[test]
    fn test_by_text() {
        let by = By::Text("Submit".to_string());
        assert_eq!(by.strategy(), "text");
        assert_eq!(by.value(), "Submit");
    }

    #[test]
    fn test_from_str() {
        let by: By = "#login".into();
        assert!(matches!(by, By::Css(_)));
    }

    #[test]
    fn test_builder_methods() {
        assert!(matches!(By::css("#id"), By::Css(_)));
        assert!(matches!(By::xpath("//div"), By::XPath(_)));
        assert!(matches!(By::text("hello"), By::Text(_)));
        assert!(matches!(By::id("myid"), By::Id(_)));
    }
}
