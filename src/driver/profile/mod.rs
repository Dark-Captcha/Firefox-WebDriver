//! Firefox profile management and configuration.
//!
//! This module handles the creation and configuration of Firefox profiles,
//! including:
//!
//! - Creating temporary profiles with automatic cleanup
//! - Using existing profile directories
//! - Writing preferences (`user.js`)
//! - Installing extensions
//!
//! # Example
//!
//! ```no_run
//! use firefox_webdriver::driver::profile::{Profile, ExtensionSource};
//!
//! # fn example() -> firefox_webdriver::Result<()> {
//! let profile = Profile::new_temp()?;
//!
//! // Write default preferences
//! let prefs = Profile::default_prefs();
//! profile.write_prefs(&prefs)?;
//!
//! // Install extension
//! let ext = ExtensionSource::unpacked("./extension");
//! profile.install_extension(&ext)?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64Standard;
use serde_json::{Value, from_str};
use tempfile::TempDir;
use tracing::debug;
use zip::ZipArchive;

use crate::error::{Error, Result};

// ============================================================================
// Submodules
// ============================================================================

/// Extension installation and management.
pub mod extensions;

/// Firefox preference definitions and serialization.
pub mod preferences;

// ============================================================================
// Re-exports
// ============================================================================

pub use extensions::ExtensionSource;
pub use preferences::{FirefoxPreference, PreferenceValue};

// ============================================================================
// Constants
// ============================================================================

/// Header comment for `user.js` file.
const USER_JS_HEADER: &str = "// Firefox WebDriver user.js\n\
                              // Auto-generated preferences for automation\n\n";

// ============================================================================
// Profile
// ============================================================================

/// A Firefox profile directory.
///
/// Manages a Firefox profile, which contains settings, extensions, and state.
/// Profiles can be temporary (auto-cleanup) or persistent (user-managed).
///
/// # Temporary Profiles
///
/// Created with [`Profile::new_temp()`], these are automatically deleted
/// when the `Profile` is dropped.
///
/// # Persistent Profiles
///
/// Created with [`Profile::from_path()`], these persist after the program exits.
pub struct Profile {
    /// Optional temporary directory handle (keeps temp dir alive).
    _temp_dir: Option<TempDir>,

    /// Path to the profile directory.
    path: PathBuf,
}

// ============================================================================
// Profile - Constructors
// ============================================================================

impl Profile {
    /// Creates a new temporary profile.
    ///
    /// The profile directory is created in the system temp directory with
    /// a unique name. It is automatically deleted when the Profile is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use firefox_webdriver::driver::profile::Profile;
    ///
    /// # fn example() -> firefox_webdriver::Result<()> {
    /// let profile = Profile::new_temp()?;
    /// println!("Profile at: {}", profile.path().display());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_temp() -> Result<Self> {
        let temp_dir = TempDir::with_prefix("firefox-webdriver-")
            .map_err(|e| Error::profile(format!("Failed to create temp profile: {}", e)))?;

        let path = temp_dir.path().to_path_buf();
        debug!(path = %path.display(), "Created temporary profile");

        Ok(Self {
            _temp_dir: Some(temp_dir),
            path,
        })
    }

    /// Uses an existing profile directory.
    ///
    /// If the directory doesn't exist, it is created.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the profile directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use firefox_webdriver::driver::profile::Profile;
    ///
    /// # fn example() -> firefox_webdriver::Result<()> {
    /// let profile = Profile::from_path("./my_profile")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| {
                Error::profile(format!(
                    "Failed to create profile directory at {}: {}",
                    path.display(),
                    e
                ))
            })?;
            debug!(path = %path.display(), "Created profile directory");
        } else {
            debug!(path = %path.display(), "Using existing profile directory");
        }

        Ok(Self {
            _temp_dir: None,
            path,
        })
    }
}

// ============================================================================
// Profile - Accessors
// ============================================================================

impl Profile {
    /// Returns the path to the profile directory.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the extensions directory, creating it if necessary.
    fn extensions_dir(&self) -> PathBuf {
        let dir = self.path.join("extensions");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    }
}

// ============================================================================
// Profile - Preferences
// ============================================================================

impl Profile {
    /// Writes preferences to `user.js`.
    ///
    /// # Arguments
    ///
    /// * `prefs` - Slice of preferences to write
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_prefs(&self, prefs: &[FirefoxPreference]) -> Result<()> {
        let file_path = self.path.join("user.js");

        let mut content = String::from(USER_JS_HEADER);
        for pref in prefs {
            content.push_str(&pref.to_user_pref_line());
            content.push('\n');
        }

        fs::write(&file_path, content).map_err(|e| {
            Error::profile(format!(
                "Failed to write user.js at {}: {}",
                file_path.display(),
                e
            ))
        })?;

        debug!(
            path = %file_path.display(),
            pref_count = prefs.len(),
            "Wrote preferences to user.js"
        );

        Ok(())
    }

    /// Returns the default preferences for WebDriver automation.
    ///
    /// These preferences configure Firefox for automation:
    /// - Allow unsigned extensions
    /// - Disable telemetry
    /// - Disable updates
    /// - Enable fingerprint randomization
    #[must_use]
    pub fn default_prefs() -> Vec<FirefoxPreference> {
        use preferences::{FirefoxPreference as Pref, PreferenceValue as Val};

        vec![
            // Critical: WebDriver Mechanics
            Pref::new("xpinstall.signatures.required", Val::Bool(false))
                .with_comment("Allow unsigned extensions"),
            Pref::new("extensions.autoDisableScopes", Val::Int(0))
                .with_comment("Allow extensions in all scopes"),
            Pref::new(
                "extensions.webextensions.restrictedDomains",
                Val::String(String::new()),
            )
            .with_comment("Allow extensions on all domains"),
            Pref::new(
                "security.data_uri.block_toplevel_data_uri_navigations",
                Val::Bool(false),
            )
            .with_comment("Allow data URI navigation"),
            // Stability: Startup & UI
            Pref::new("browser.startup.page", Val::Int(0)).with_comment("Start on blank page"),
            Pref::new("browser.shell.checkDefaultBrowser", Val::Bool(false)),
            // Silence: Telemetry
            Pref::new("toolkit.telemetry.unified", Val::Bool(false)),
            Pref::new("toolkit.telemetry.enabled", Val::Bool(false)),
            Pref::new("toolkit.telemetry.server", Val::String(String::new())),
            Pref::new(
                "datareporting.policy.dataSubmissionEnabled",
                Val::Bool(false),
            ),
            Pref::new("datareporting.healthreport.uploadEnabled", Val::Bool(false)),
            Pref::new("app.normandy.enabled", Val::Bool(false)),
            Pref::new("browser.safebrowsing.malware.enabled", Val::Bool(false)),
            Pref::new("browser.safebrowsing.phishing.enabled", Val::Bool(false)),
            Pref::new("network.captive-portal-service.enabled", Val::Bool(false)),
            Pref::new("network.connectivity-service.enabled", Val::Bool(false)),
            // Updates: Disable
            Pref::new("app.update.auto", Val::Bool(false)),
            Pref::new("app.update.enabled", Val::Bool(false)),
            // Canvas Fingerprint Randomization
            Pref::new("privacy.resistFingerprinting", Val::Bool(true))
                .with_comment("Enable fingerprint randomization (canvas, etc.)"),
        ]
    }
}

// ============================================================================
// Profile - Extensions
// ============================================================================

impl Profile {
    /// Installs an extension into the profile.
    ///
    /// # Arguments
    ///
    /// * `source` - Extension source (unpacked, packed, or base64)
    ///
    /// # Errors
    ///
    /// Returns an error if installation fails.
    pub fn install_extension(&self, source: &ExtensionSource) -> Result<()> {
        match source {
            ExtensionSource::Unpacked(path) => {
                debug!(path = %path.display(), "Installing unpacked extension");
                self.install_unpacked(path)
            }
            ExtensionSource::Packed(path) => {
                debug!(path = %path.display(), "Installing packed extension");
                self.install_packed(path)
            }
            ExtensionSource::Base64(data) => {
                debug!("Installing base64 extension");
                self.install_base64(data)
            }
        }
    }

    /// Installs an unpacked extension directory.
    fn install_unpacked(&self, src: &Path) -> Result<()> {
        let extension_id = self.read_manifest_id(src)?;
        let dest = self.extensions_dir().join(&extension_id);

        copy_dir_recursive(src, &dest)?;

        debug!(
            extension_id = %extension_id,
            dest = %dest.display(),
            "Installed unpacked extension"
        );

        Ok(())
    }

    /// Installs a packed extension (.xpi or .zip).
    fn install_packed(&self, src: &Path) -> Result<()> {
        let file = fs::File::open(src).map_err(Error::Io)?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| Error::profile(format!("Invalid extension archive: {}", e)))?;

        let temp_extract = TempDir::new().map_err(Error::Io)?;
        archive
            .extract(temp_extract.path())
            .map_err(|e| Error::profile(format!("Failed to extract extension: {}", e)))?;

        self.install_unpacked(temp_extract.path())
    }

    /// Installs a base64-encoded extension.
    fn install_base64(&self, data: &str) -> Result<()> {
        // Decode base64
        let bytes = Base64Standard
            .decode(data)
            .map_err(|e| Error::profile(format!("Invalid base64 extension data: {}", e)))?;

        // Write to temp file
        let temp_dir = TempDir::new().map_err(Error::Io)?;
        let temp_xpi = temp_dir.path().join("extension.xpi");
        fs::write(&temp_xpi, bytes).map_err(Error::Io)?;

        // Install as packed
        self.install_packed(&temp_xpi)
    }

    /// Reads the extension ID from manifest.json.
    fn read_manifest_id(&self, dir: &Path) -> Result<String> {
        let manifest_path = dir.join("manifest.json");
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            Error::profile(format!(
                "Extension manifest not found at {}: {}",
                manifest_path.display(),
                e
            ))
        })?;

        let json: Value = from_str(&content)
            .map_err(|e| Error::profile(format!("Invalid manifest.json: {}", e)))?;

        // Try standard WebExtension ID locations
        if let Some(id) = json.pointer("/browser_specific_settings/gecko/id")
            && let Some(id_str) = id.as_str()
        {
            return Ok(id_str.to_string());
        }

        if let Some(id) = json.pointer("/applications/gecko/id")
            && let Some(id_str) = id.as_str()
        {
            return Ok(id_str.to_string());
        }

        Err(Error::profile(
            "Extension manifest missing 'gecko.id' field".to_string(),
        ))
    }
}

// ============================================================================
// Private Helpers
// ============================================================================

/// Recursively copies a directory and all its contents.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(Error::Io)?;
    }

    for entry in fs::read_dir(src).map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let file_type = entry.file_type().map_err(Error::Io)?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(Error::Io)?;
        }
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::Profile;

    #[test]
    fn test_new_temp_creates_directory() {
        let profile = Profile::new_temp().expect("create temp profile");
        assert!(profile.path().exists());
        assert!(profile.path().is_dir());
    }

    #[test]
    fn test_temp_profile_cleanup_on_drop() {
        let path = {
            let profile = Profile::new_temp().expect("create temp profile");
            let path = profile.path().to_path_buf();
            assert!(path.exists());
            path
        };
        assert!(!path.exists());
    }

    #[test]
    fn test_default_prefs_not_empty() {
        let prefs = Profile::default_prefs();
        assert!(!prefs.is_empty());
    }

    #[test]
    fn test_from_path_creates_directory() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let profile_path = temp.path().join("test_profile");

        assert!(!profile_path.exists());
        let profile = Profile::from_path(&profile_path).expect("create profile");
        assert!(profile.path().exists());
    }
}
