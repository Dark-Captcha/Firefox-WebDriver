//! Firefox command-line options and configuration.
//!
//! Provides a type-safe interface for configuring Firefox process options
//! such as headless mode, window size, and other command-line arguments.
//!
//! # Example
//!
//! ```ignore
//! use firefox_webdriver::FirefoxOptions;
//!
//! let options = FirefoxOptions::new()
//!     .with_headless()
//!     .with_window_size(1920, 1080)
//!     .with_private();
//!
//! let args = options.to_args();
//! // ["--headless", "--window-size", "1920,1080", "--private-window"]
//! ```

// ============================================================================
// FirefoxOptions
// ============================================================================

/// Firefox process configuration options.
///
/// Controls how Firefox is launched, including display mode, window dimensions,
/// and additional command-line arguments.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FirefoxOptions {
    /// Run Firefox without a GUI (headless mode).
    pub headless: bool,

    /// Window dimensions in pixels (width, height).
    pub window_size: Option<(u32, u32)>,

    /// Enable kiosk mode (fullscreen with restricted UI).
    pub kiosk: bool,

    /// Open Developer Tools on startup.
    pub devtools: bool,

    /// Open a private browsing window.
    pub private: bool,

    /// Additional custom command-line arguments.
    pub extra_args: Vec<String>,
}

// ============================================================================
// Constructors
// ============================================================================

impl FirefoxOptions {
    /// Creates a new options instance with default settings.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            headless: false,
            window_size: None,
            kiosk: false,
            devtools: false,
            private: false,
            extra_args: Vec::new(),
        }
    }

    /// Creates options configured for headless mode.
    #[inline]
    #[must_use]
    pub fn headless() -> Self {
        Self {
            headless: true,
            ..Default::default()
        }
    }
}

// ============================================================================
// Builder Methods
// ============================================================================

impl FirefoxOptions {
    /// Enables headless mode.
    #[inline]
    #[must_use]
    pub fn with_headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// Sets window size in pixels.
    #[inline]
    #[must_use]
    pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
        self.window_size = Some((width, height));
        self
    }

    /// Enables kiosk mode.
    #[inline]
    #[must_use]
    pub fn with_kiosk(mut self) -> Self {
        self.kiosk = true;
        self
    }

    /// Enables developer tools on startup.
    #[inline]
    #[must_use]
    pub fn with_devtools(mut self) -> Self {
        self.devtools = true;
        self
    }

    /// Enables private browsing mode.
    #[inline]
    #[must_use]
    pub fn with_private(mut self) -> Self {
        self.private = true;
        self
    }

    /// Adds a custom command-line argument.
    #[inline]
    #[must_use]
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Adds multiple custom command-line arguments.
    #[inline]
    #[must_use]
    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extra_args.extend(args.into_iter().map(Into::into));
        self
    }
}

// ============================================================================
// Conversion Methods
// ============================================================================

impl FirefoxOptions {
    /// Converts options to Firefox command-line arguments.
    #[must_use]
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(8 + self.extra_args.len());

        if self.headless {
            args.push("--headless".to_string());
        }

        if let Some((width, height)) = self.window_size {
            args.push("--window-size".to_string());
            args.push(format!("{width},{height}"));
        }

        if self.kiosk {
            args.push("--kiosk".to_string());
        }

        if self.devtools {
            args.push("--devtools".to_string());
        }

        if self.private {
            args.push("--private-window".to_string());
        }

        args.extend(self.extra_args.clone());
        args
    }

    /// Validates the options configuration.
    ///
    /// # Errors
    ///
    /// Returns error message if validation fails.
    pub fn validate(&self) -> Result<(), String> {
        if let Some((width, height)) = self.window_size
            && (width == 0 || height == 0)
        {
            return Err("Window dimensions must be greater than zero".to_string());
        }
        Ok(())
    }

    /// Returns `true` if headless mode is enabled.
    #[inline]
    #[must_use]
    pub const fn is_headless(&self) -> bool {
        self.headless
    }

    /// Returns `true` if private browsing is enabled.
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        self.private
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_default() {
        let options = FirefoxOptions::new();
        assert!(!options.headless);
        assert!(options.window_size.is_none());
        assert!(!options.kiosk);
        assert!(!options.devtools);
        assert!(!options.private);
        assert!(options.extra_args.is_empty());
    }

    #[test]
    fn test_headless_constructor() {
        let options = FirefoxOptions::headless();
        assert!(options.headless);
        assert!(options.is_headless());
    }

    #[test]
    fn test_builder_chain() {
        let options = FirefoxOptions::new()
            .with_headless()
            .with_window_size(1920, 1080)
            .with_devtools()
            .with_private();

        assert!(options.headless);
        assert_eq!(options.window_size, Some((1920, 1080)));
        assert!(options.devtools);
        assert!(options.private);
    }

    #[test]
    fn test_to_args_headless() {
        let options = FirefoxOptions::new().with_headless();
        let args = options.to_args();
        assert!(args.contains(&"--headless".to_string()));
    }

    #[test]
    fn test_to_args_window_size() {
        let options = FirefoxOptions::new().with_window_size(800, 600);
        let args = options.to_args();
        assert!(args.contains(&"--window-size".to_string()));
        assert!(args.contains(&"800,600".to_string()));
    }

    #[test]
    fn test_to_args_all_options() {
        let options = FirefoxOptions::new()
            .with_headless()
            .with_window_size(1024, 768)
            .with_kiosk()
            .with_devtools()
            .with_private()
            .with_arg("--custom");

        let args = options.to_args();
        assert!(args.contains(&"--headless".to_string()));
        assert!(args.contains(&"--kiosk".to_string()));
        assert!(args.contains(&"--devtools".to_string()));
        assert!(args.contains(&"--private-window".to_string()));
        assert!(args.contains(&"--custom".to_string()));
    }

    #[test]
    fn test_with_args_multiple() {
        let options = FirefoxOptions::new().with_args(["--arg1", "--arg2"]);
        assert_eq!(options.extra_args.len(), 2);
    }

    #[test]
    fn test_validate_valid() {
        let options = FirefoxOptions::new().with_window_size(800, 600);
        assert!(options.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_width() {
        let options = FirefoxOptions::new().with_window_size(0, 600);
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_validate_zero_height() {
        let options = FirefoxOptions::new().with_window_size(800, 0);
        assert!(options.validate().is_err());
    }
}
