//! Firefox preference serialization for `user.js`.
//!
//! Firefox preferences are written as JavaScript function calls:
//!
//! ```javascript
//! user_pref("preference.name", value);
//! ```
//!
//! # Example
//!
//! ```
//! use firefox_webdriver::driver::profile::{FirefoxPreference, PreferenceValue};
//!
//! let pref = FirefoxPreference::new("browser.startup.page", PreferenceValue::Int(0))
//!     .with_comment("Start on blank page");
//!
//! assert!(pref.to_user_pref_line().contains("user_pref"));
//! ```

// ============================================================================
// PreferenceValue
// ============================================================================

/// A preference value in `user.js`.
///
/// Firefox preferences can be booleans, integers, or strings.
///
/// # Examples
///
/// ```
/// use firefox_webdriver::driver::profile::PreferenceValue;
///
/// let bool_val = PreferenceValue::Bool(true);
/// let int_val = PreferenceValue::Int(42);
/// let str_val = PreferenceValue::String("value".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreferenceValue {
    /// Boolean value (true/false).
    Bool(bool),

    /// Integer value.
    Int(i32),

    /// String value.
    String(String),
}

// ============================================================================
// PreferenceValue - Methods
// ============================================================================

impl PreferenceValue {
    /// Formats the value for `user.js`.
    ///
    /// - Booleans: `true` or `false`
    /// - Integers: numeric literal
    /// - Strings: quoted and escaped
    #[must_use]
    pub fn to_js_string(&self) -> String {
        match self {
            Self::Bool(b) => b.to_string(),
            Self::Int(i) => i.to_string(),
            Self::String(s) => format!("\"{}\"", escape_js_string(s)),
        }
    }
}

// ============================================================================
// PreferenceValue - Trait Implementations
// ============================================================================

impl From<bool> for PreferenceValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for PreferenceValue {
    #[inline]
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<String> for PreferenceValue {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for PreferenceValue {
    #[inline]
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

// ============================================================================
// FirefoxPreference
// ============================================================================

/// A Firefox preference with a name and value.
///
/// # Examples
///
/// ```
/// use firefox_webdriver::driver::profile::{FirefoxPreference, PreferenceValue};
///
/// // Simple preference
/// let pref = FirefoxPreference::new("browser.startup.page", PreferenceValue::Int(0));
///
/// // With comment
/// let pref = FirefoxPreference::new("app.update.enabled", false)
///     .with_comment("Disable auto-updates");
/// ```
#[derive(Debug, Clone)]
pub struct FirefoxPreference {
    /// Preference name (e.g., "browser.startup.page").
    pub key: String,

    /// Preference value.
    pub value: PreferenceValue,

    /// Optional comment explaining the preference.
    pub comment: Option<String>,
}

// ============================================================================
// FirefoxPreference - Implementation
// ============================================================================

impl FirefoxPreference {
    /// Creates a new preference.
    ///
    /// # Arguments
    ///
    /// * `key` - Preference name (e.g., "browser.startup.page")
    /// * `value` - Preference value
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<PreferenceValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            comment: None,
        }
    }

    /// Adds a comment to the preference.
    ///
    /// Comments appear as `// comment` above the preference line.
    ///
    /// # Arguments
    ///
    /// * `comment` - Comment text
    #[must_use]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Generates the `user_pref("key", value);` line.
    ///
    /// If a comment is set, it appears on the line above.
    ///
    /// # Example Output
    ///
    /// ```text
    /// // Disable auto-updates
    /// user_pref("app.update.enabled", false);
    /// ```
    #[must_use]
    pub fn to_user_pref_line(&self) -> String {
        let mut output = String::new();

        if let Some(comment) = &self.comment {
            output.push_str("// ");
            output.push_str(comment);
            output.push('\n');
        }

        output.push_str(&format!(
            "user_pref(\"{}\", {});",
            self.key,
            self.value.to_js_string()
        ));

        output
    }
}

// ============================================================================
// Private Helpers
// ============================================================================

/// Escapes special characters for JavaScript strings.
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{FirefoxPreference, PreferenceValue, escape_js_string};

    // ------------------------------------------------------------------------
    // PreferenceValue Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bool_to_js_string() {
        assert_eq!(PreferenceValue::Bool(true).to_js_string(), "true");
        assert_eq!(PreferenceValue::Bool(false).to_js_string(), "false");
    }

    #[test]
    fn test_int_to_js_string() {
        assert_eq!(PreferenceValue::Int(42).to_js_string(), "42");
        assert_eq!(PreferenceValue::Int(-10).to_js_string(), "-10");
        assert_eq!(PreferenceValue::Int(0).to_js_string(), "0");
    }

    #[test]
    fn test_string_to_js_string() {
        assert_eq!(
            PreferenceValue::String("test".to_string()).to_js_string(),
            "\"test\""
        );
        assert_eq!(
            PreferenceValue::String(String::new()).to_js_string(),
            "\"\""
        );
    }

    #[test]
    fn test_from_bool() {
        let val: PreferenceValue = true.into();
        assert_eq!(val, PreferenceValue::Bool(true));
    }

    #[test]
    fn test_from_i32() {
        let val: PreferenceValue = 42.into();
        assert_eq!(val, PreferenceValue::Int(42));
    }

    #[test]
    fn test_from_string() {
        let val: PreferenceValue = String::from("test").into();
        assert_eq!(val, PreferenceValue::String("test".to_string()));
    }

    #[test]
    fn test_from_str() {
        let val: PreferenceValue = "test".into();
        assert_eq!(val, PreferenceValue::String("test".to_string()));
    }

    // ------------------------------------------------------------------------
    // escape_js_string Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_escape_backslash() {
        assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape_js_string("with\"quotes"), "with\\\"quotes");
    }

    #[test]
    fn test_escape_newline() {
        assert_eq!(escape_js_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_escape_carriage_return() {
        assert_eq!(escape_js_string("line1\rline2"), "line1\\rline2");
    }

    #[test]
    fn test_escape_tab() {
        assert_eq!(escape_js_string("col1\tcol2"), "col1\\tcol2");
    }

    #[test]
    fn test_escape_combined() {
        assert_eq!(
            escape_js_string("path\\to\n\"file\""),
            "path\\\\to\\n\\\"file\\\""
        );
    }

    // ------------------------------------------------------------------------
    // FirefoxPreference Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_new_preference() {
        let pref = FirefoxPreference::new("test.pref", PreferenceValue::Bool(true));
        assert_eq!(pref.key, "test.pref");
        assert_eq!(pref.value, PreferenceValue::Bool(true));
        assert!(pref.comment.is_none());
    }

    #[test]
    fn test_with_comment() {
        let pref = FirefoxPreference::new("test.pref", PreferenceValue::Int(42))
            .with_comment("Test comment");

        assert_eq!(pref.comment, Some("Test comment".to_string()));
    }

    #[test]
    fn test_to_user_pref_line_bool() {
        let pref = FirefoxPreference::new("test.pref", PreferenceValue::Bool(true));
        assert_eq!(pref.to_user_pref_line(), "user_pref(\"test.pref\", true);");
    }

    #[test]
    fn test_to_user_pref_line_int() {
        let pref = FirefoxPreference::new("test.pref", PreferenceValue::Int(42));
        assert_eq!(pref.to_user_pref_line(), "user_pref(\"test.pref\", 42);");
    }

    #[test]
    fn test_to_user_pref_line_string() {
        let pref = FirefoxPreference::new("test.pref", PreferenceValue::String("value".into()));
        assert_eq!(
            pref.to_user_pref_line(),
            "user_pref(\"test.pref\", \"value\");"
        );
    }

    #[test]
    fn test_to_user_pref_line_with_comment() {
        let pref = FirefoxPreference::new("test.pref", PreferenceValue::Int(42))
            .with_comment("Test preference");

        let line = pref.to_user_pref_line();
        assert!(line.starts_with("// Test preference\n"));
        assert!(line.ends_with("user_pref(\"test.pref\", 42);"));
    }

    #[test]
    fn test_new_with_into_value() {
        // Test that Into<PreferenceValue> works
        let pref = FirefoxPreference::new("test.bool", false);
        assert_eq!(pref.value, PreferenceValue::Bool(false));

        let pref = FirefoxPreference::new("test.int", 123);
        assert_eq!(pref.value, PreferenceValue::Int(123));
    }

    #[test]
    fn test_preference_clone() {
        let pref = FirefoxPreference::new("test.pref", true).with_comment("comment");
        let cloned = pref.clone();

        assert_eq!(pref.key, cloned.key);
        assert_eq!(pref.value, cloned.value);
        assert_eq!(pref.comment, cloned.comment);
    }
}
