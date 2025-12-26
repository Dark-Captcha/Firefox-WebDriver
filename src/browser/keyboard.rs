//! Keyboard key definitions and utilities.
//!
//! Provides ergonomic key constants for common navigation and control keys.
//!
//! # Example
//!
//! ```ignore
//! use firefox_webdriver::Key;
//!
//! // Navigation keys
//! element.press(Key::Enter).await?;
//! element.press(Key::Tab).await?;
//! element.press(Key::Escape).await?;
//!
//! // For typing text, use type_text instead:
//! element.type_text("Hello, World!").await?;
//! ```

// ============================================================================
// Key Enum
// ============================================================================

/// Common keyboard keys for navigation and control.
///
/// This enum provides ergonomic constants for frequently used keys.
/// For typing text/letters, use `element.type_text()` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // ========================================================================
    // Navigation & Control
    // ========================================================================
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Escape key
    Escape,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Space bar
    Space,

    // ========================================================================
    // Arrow Keys
    // ========================================================================
    /// Arrow Up
    ArrowUp,
    /// Arrow Down
    ArrowDown,
    /// Arrow Left
    ArrowLeft,
    /// Arrow Right
    ArrowRight,

    // ========================================================================
    // Page Navigation
    // ========================================================================
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
}

impl Key {
    /// Returns the key properties: (key, code, keyCode, printable).
    #[must_use]
    pub fn properties(self) -> (&'static str, &'static str, u32, bool) {
        match self {
            Key::Enter => ("Enter", "Enter", 13, false),
            Key::Tab => ("Tab", "Tab", 9, false),
            Key::Escape => ("Escape", "Escape", 27, false),
            Key::Backspace => ("Backspace", "Backspace", 8, false),
            Key::Delete => ("Delete", "Delete", 46, false),
            Key::Space => (" ", "Space", 32, true),
            Key::ArrowUp => ("ArrowUp", "ArrowUp", 38, false),
            Key::ArrowDown => ("ArrowDown", "ArrowDown", 40, false),
            Key::ArrowLeft => ("ArrowLeft", "ArrowLeft", 37, false),
            Key::ArrowRight => ("ArrowRight", "ArrowRight", 39, false),
            Key::Home => ("Home", "Home", 36, false),
            Key::End => ("End", "End", 35, false),
            Key::PageUp => ("PageUp", "PageUp", 33, false),
            Key::PageDown => ("PageDown", "PageDown", 34, false),
        }
    }

    /// Returns the key value string.
    #[inline]
    #[must_use]
    pub fn key(self) -> &'static str {
        self.properties().0
    }

    /// Returns the code string.
    #[inline]
    #[must_use]
    pub fn code(self) -> &'static str {
        self.properties().1
    }

    /// Returns the legacy keyCode.
    #[inline]
    #[must_use]
    pub fn key_code(self) -> u32 {
        self.properties().2
    }

    /// Returns whether this key produces printable output.
    #[inline]
    #[must_use]
    pub fn is_printable(self) -> bool {
        self.properties().3
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_properties() {
        let (key, code, key_code, printable) = Key::Enter.properties();
        assert_eq!(key, "Enter");
        assert_eq!(code, "Enter");
        assert_eq!(key_code, 13);
        assert!(!printable);
    }

    #[test]
    fn test_space_is_printable() {
        assert!(Key::Space.is_printable());
        assert!(!Key::Enter.is_printable());
    }
}
