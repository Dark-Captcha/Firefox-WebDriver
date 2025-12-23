//! Firefox extension installation and management.
//!
//! Extensions can be provided in three formats:
//!
//! | Format | Description |
//! |--------|-------------|
//! | Unpacked | Directory containing `manifest.json` |
//! | Packed | `.xpi` or `.zip` archive |
//! | Base64 | Base64-encoded `.xpi` content |
//!
//! # Example
//!
//! ```
//! use firefox_webdriver::driver::profile::ExtensionSource;
//!
//! // Unpacked directory
//! let unpacked = ExtensionSource::unpacked("./extension");
//!
//! // Packed .xpi file
//! let packed = ExtensionSource::packed("./extension.xpi");
//!
//! // Base64-encoded (useful for embedding)
//! let base64 = ExtensionSource::base64("UEsDBBQ...");
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::path::PathBuf;

// ============================================================================
// ExtensionSource
// ============================================================================

/// Source location for a Firefox extension.
///
/// Extensions can be provided as unpacked directories, packed archives,
/// or base64-encoded content.
///
/// # Examples
///
/// ```
/// use firefox_webdriver::driver::profile::ExtensionSource;
///
/// // Unpacked directory
/// let unpacked = ExtensionSource::unpacked("./extension");
///
/// // Packed .xpi file
/// let packed = ExtensionSource::packed("./extension.xpi");
///
/// // Base64-encoded
/// let base64 = ExtensionSource::base64("UEsDBBQ...");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtensionSource {
    /// Path to an unpacked extension directory.
    Unpacked(PathBuf),

    /// Path to a packed extension archive (.xpi or .zip).
    Packed(PathBuf),

    /// Base64-encoded extension content.
    Base64(String),
}

// ============================================================================
// ExtensionSource - Constructors
// ============================================================================

impl ExtensionSource {
    /// Creates an unpacked extension source.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to directory containing `manifest.json`
    #[inline]
    #[must_use]
    pub fn unpacked(path: impl Into<PathBuf>) -> Self {
        Self::Unpacked(path.into())
    }

    /// Creates a packed extension source.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to `.xpi` or `.zip` file
    #[inline]
    #[must_use]
    pub fn packed(path: impl Into<PathBuf>) -> Self {
        Self::Packed(path.into())
    }

    /// Creates a base64-encoded extension source.
    ///
    /// Useful for embedding extensions in the binary.
    ///
    /// # Arguments
    ///
    /// * `data` - Base64-encoded `.xpi` content
    #[inline]
    #[must_use]
    pub fn base64(data: impl Into<String>) -> Self {
        Self::Base64(data.into())
    }
}

// ============================================================================
// ExtensionSource - Accessors
// ============================================================================

impl ExtensionSource {
    /// Returns the path if this is a file-based source.
    ///
    /// Returns `None` for base64-encoded sources.
    #[inline]
    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Unpacked(path) | Self::Packed(path) => Some(path),
            Self::Base64(_) => None,
        }
    }

    /// Returns `true` if this is an unpacked extension.
    #[inline]
    #[must_use]
    pub fn is_unpacked(&self) -> bool {
        matches!(self, Self::Unpacked(_))
    }

    /// Returns `true` if this is a packed extension.
    #[inline]
    #[must_use]
    pub fn is_packed(&self) -> bool {
        matches!(self, Self::Packed(_))
    }

    /// Returns `true` if this is a base64-encoded extension.
    #[inline]
    #[must_use]
    pub fn is_base64(&self) -> bool {
        matches!(self, Self::Base64(_))
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl From<PathBuf> for ExtensionSource {
    /// Automatically determines extension type based on path.
    ///
    /// - Directories become [`ExtensionSource::Unpacked`]
    /// - Files become [`ExtensionSource::Packed`]
    fn from(path: PathBuf) -> Self {
        if path.is_dir() {
            Self::Unpacked(path)
        } else {
            Self::Packed(path)
        }
    }
}

impl From<&str> for ExtensionSource {
    fn from(path: &str) -> Self {
        Self::from(PathBuf::from(path))
    }
}

impl From<String> for ExtensionSource {
    fn from(path: String) -> Self {
        Self::from(PathBuf::from(path))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::ExtensionSource;

    use std::path::PathBuf;

    #[test]
    fn test_unpacked_constructor() {
        let source = ExtensionSource::unpacked("./extension");
        assert!(source.is_unpacked());
        assert!(!source.is_packed());
        assert!(!source.is_base64());
    }

    #[test]
    fn test_packed_constructor() {
        let source = ExtensionSource::packed("./extension.xpi");
        assert!(source.is_packed());
        assert!(!source.is_unpacked());
        assert!(!source.is_base64());
    }

    #[test]
    fn test_base64_constructor() {
        let source = ExtensionSource::base64("UEsDBBQ...");
        assert!(source.is_base64());
        assert!(!source.is_unpacked());
        assert!(!source.is_packed());
        assert!(source.path().is_none());
    }

    #[test]
    fn test_path_accessor() {
        let unpacked = ExtensionSource::unpacked("./ext");
        assert_eq!(unpacked.path(), Some(&PathBuf::from("./ext")));

        let packed = ExtensionSource::packed("./ext.xpi");
        assert_eq!(packed.path(), Some(&PathBuf::from("./ext.xpi")));

        let base64 = ExtensionSource::base64("data");
        assert_eq!(base64.path(), None);
    }

    #[test]
    fn test_from_pathbuf_directory() {
        // Current directory is always a directory
        let source = ExtensionSource::from(PathBuf::from("."));
        assert!(source.is_unpacked());
    }

    #[test]
    fn test_from_pathbuf_file() {
        // Non-existent path treated as file
        let source = ExtensionSource::from(PathBuf::from("./nonexistent.xpi"));
        assert!(source.is_packed());
    }

    #[test]
    fn test_from_str() {
        let source = ExtensionSource::from("./extension");
        assert!(source.path().is_some());
    }

    #[test]
    fn test_from_string() {
        let source = ExtensionSource::from(String::from("./extension"));
        assert!(source.path().is_some());
    }

    #[test]
    fn test_clone() {
        let source = ExtensionSource::unpacked("./ext");
        let cloned = source.clone();
        assert_eq!(source, cloned);
    }

    #[test]
    fn test_debug() {
        let source = ExtensionSource::unpacked("./ext");
        let debug_str = format!("{:?}", source);
        assert!(debug_str.contains("Unpacked"));
    }
}
