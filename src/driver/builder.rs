//! Builder pattern for driver configuration.
//!
//! Provides a fluent API for configuring and creating [`Driver`] instances.
//!
//! # Example
//!
//! ```no_run
//! use firefox_webdriver::Driver;
//!
//! # fn example() -> firefox_webdriver::Result<()> {
//! let driver = Driver::builder()
//!     .binary("/usr/bin/firefox")
//!     .extension("./extension")
//!     .build()?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::path::PathBuf;

use crate::error::{Error, Result};

use super::core::Driver;
use super::profile::ExtensionSource;

// ============================================================================
// DriverBuilder
// ============================================================================

/// Builder for configuring a [`Driver`] instance.
///
/// Use [`Driver::builder()`] to create a new builder.
#[derive(Debug, Default, Clone)]
pub struct DriverBuilder {
    /// Path to Firefox binary.
    binary: Option<PathBuf>,
    /// Extension source.
    extension: Option<ExtensionSource>,
}

// ============================================================================
// DriverBuilder Implementation
// ============================================================================

impl DriverBuilder {
    /// Creates a new driver builder with no configuration.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the path to the Firefox binary executable.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to Firefox binary (e.g., "/usr/bin/firefox")
    #[inline]
    #[must_use]
    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary = Some(path.into());
        self
    }

    /// Sets the path to the WebDriver extension.
    ///
    /// Automatically detects whether the path is a directory (unpacked)
    /// or file (packed .xpi).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to extension directory or .xpi file
    #[inline]
    #[must_use]
    pub fn extension(mut self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        self.extension = Some(ExtensionSource::from(path));
        self
    }

    /// Sets the extension from a base64-encoded string.
    ///
    /// Useful for embedding the extension in the binary.
    ///
    /// # Arguments
    ///
    /// * `data` - Base64-encoded .xpi content
    #[inline]
    #[must_use]
    pub fn extension_base64(mut self, data: impl Into<String>) -> Self {
        self.extension = Some(ExtensionSource::base64(data));
        self
    }

    /// Sets the extension source directly.
    ///
    /// # Arguments
    ///
    /// * `source` - Extension source variant
    #[inline]
    #[must_use]
    pub fn extension_source(mut self, source: ExtensionSource) -> Self {
        self.extension = Some(source);
        self
    }

    /// Builds the driver with validation.
    ///
    /// # Errors
    ///
    /// - [`Error::Config`] if binary or extension not set
    /// - [`Error::FirefoxNotFound`] if binary path doesn't exist
    /// - [`Error::Config`] if extension path doesn't exist
    pub fn build(self) -> Result<Driver> {
        let binary = self.validate_binary()?;
        let extension = self.validate_extension()?;

        Driver::new(binary, extension)
    }
}

// ============================================================================
// Validation
// ============================================================================

impl DriverBuilder {
    /// Validates the binary path configuration.
    fn validate_binary(&self) -> Result<PathBuf> {
        let binary = self.binary.clone().ok_or_else(|| {
            Error::config(
                "Firefox binary path is required. Use .binary() to set it.\n\
                 Example: Driver::builder().binary(\"/usr/bin/firefox\")",
            )
        })?;

        if !binary.exists() {
            return Err(Error::firefox_not_found(&binary));
        }

        Ok(binary)
    }

    /// Validates the extension configuration.
    fn validate_extension(&self) -> Result<ExtensionSource> {
        let extension = self.extension.clone().ok_or_else(|| {
            Error::config(
                "Extension is required. Use .extension() or .extension_base64() to set it.\n\
                 Example: Driver::builder().extension(\"./extension\")",
            )
        })?;

        // Validate file-based extensions exist
        if let Some(path) = extension.path()
            && !path.exists()
        {
            return Err(Error::config(format!(
                "Extension not found at: {}\n\
                 Ensure the extension directory or .xpi file exists.",
                path.display()
            )));
        }

        Ok(extension)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_builder() {
        let builder = DriverBuilder::new();
        assert!(builder.binary.is_none());
        assert!(builder.extension.is_none());
    }

    #[test]
    fn test_default_creates_empty_builder() {
        let builder = DriverBuilder::default();
        assert!(builder.binary.is_none());
        assert!(builder.extension.is_none());
    }

    #[test]
    fn test_binary_sets_path() {
        let builder = DriverBuilder::new().binary("/usr/bin/firefox");
        assert_eq!(builder.binary, Some(PathBuf::from("/usr/bin/firefox")));
    }

    #[test]
    fn test_extension_sets_source() {
        let builder = DriverBuilder::new().extension("./extension");
        assert!(builder.extension.is_some());
    }

    #[test]
    fn test_extension_base64_sets_source() {
        let builder = DriverBuilder::new().extension_base64("UEsDBBQ...");
        assert!(builder.extension.is_some());

        if let Some(ExtensionSource::Base64(data)) = builder.extension {
            assert_eq!(data, "UEsDBBQ...");
        } else {
            panic!("Expected Base64 extension source");
        }
    }

    #[test]
    fn test_extension_source_sets_directly() {
        let source = ExtensionSource::packed("./ext.xpi");
        let builder = DriverBuilder::new().extension_source(source.clone());
        assert_eq!(builder.extension, Some(source));
    }

    #[test]
    fn test_build_fails_without_binary() {
        let result = DriverBuilder::new().extension("./extension").build();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("binary"));
    }

    #[test]
    fn test_build_fails_without_extension() {
        let result = DriverBuilder::new().binary("/bin/sh").build();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("Extension"));
    }

    #[test]
    fn test_build_fails_with_nonexistent_binary() {
        let result = DriverBuilder::new()
            .binary("/nonexistent/firefox")
            .extension_base64("data")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_is_clone() {
        let builder = DriverBuilder::new().binary("/usr/bin/firefox");
        let cloned = builder.clone();
        assert_eq!(builder.binary, cloned.binary);
    }
}
