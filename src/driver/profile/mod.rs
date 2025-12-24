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
            // ================================================================
            // SECTION 1: WebDriver Extension Support
            // Source: modules/libpref/init/all.js, browser/app/profile/firefox.js
            // ================================================================
            // Only Firefox requires add-on signatures (firefox.js)
            Pref::new("xpinstall.signatures.required", Val::Bool(false))
                .with_comment("Only Firefox requires add-on signatures"),
            // Disable add-ons not installed by user in all scopes (firefox.js)
            Pref::new("extensions.autoDisableScopes", Val::Int(0))
                .with_comment("Disable add-ons that are not installed by the user in all scopes"),
            // Restricted domains for webextensions (all.js)
            Pref::new(
                "extensions.webextensions.restrictedDomains",
                Val::String(String::new()),
            )
            .with_comment("Restricted domains for webextensions"),
            Pref::new(
                "security.data_uri.block_toplevel_data_uri_navigations",
                Val::Bool(false),
            )
            .with_comment("Block toplevel data URI navigations"),
            // ================================================================
            // SECTION 2: Fast Startup (Skip UI prompts)
            // Source: browser/app/profile/firefox.js
            // ================================================================
            // 0 = blank, 1 = home, 2 = last visited page, 3 = resume previous session
            Pref::new("browser.startup.page", Val::Int(0))
                .with_comment("0 = blank, 1 = home, 2 = last visited, 3 = resume session"),
            // At startup, check if we're the default browser and prompt user if not
            Pref::new("browser.shell.checkDefaultBrowser", Val::Bool(false))
                .with_comment("At startup, check if we're the default browser"),
            Pref::new(
                "browser.startup.homepage_override.mstone",
                Val::String("ignore".into()),
            )
            .with_comment("Used to display upgrade page after version upgrade"),
            Pref::new("browser.sessionstore.resume_from_crash", Val::Bool(false))
                .with_comment("Whether to resume session after crash"),
            // Number of crashes that can occur before about:sessionrestore is displayed
            Pref::new("toolkit.startup.max_resumed_crashes", Val::Int(-1))
                .with_comment("Number of crashes before about:sessionrestore is displayed"),
            Pref::new("browser.tabs.warnOnClose", Val::Bool(false))
                .with_comment("Warn when closing multiple tabs"),
            Pref::new("browser.tabs.warnOnCloseOtherTabs", Val::Bool(false))
                .with_comment("Warn when closing other tabs"),
            // browser.warnOnQuit == false will override all other possible prompts
            Pref::new("browser.warnOnQuit", Val::Bool(false))
                .with_comment("Override all other possible prompts when quitting"),
            Pref::new("browser.pagethumbnails.capturing_disabled", Val::Bool(true))
                .with_comment("Disable page thumbnail capturing"),
            Pref::new("browser.aboutConfig.showWarning", Val::Bool(false))
                .with_comment("Show warning when accessing about:config"),
            Pref::new(
                "browser.bookmarks.restore_default_bookmarks",
                Val::Bool(false),
            )
            .with_comment("Restore default bookmarks"),
            Pref::new("browser.disableResetPrompt", Val::Bool(true))
                .with_comment("Disable reset prompt"),
            // This records whether or not the panel has been shown at least once
            Pref::new("browser.download.panel.shown", Val::Bool(true))
                .with_comment("Records whether download panel has been shown"),
            Pref::new("browser.feeds.showFirstRunUI", Val::Bool(false))
                .with_comment("Show first run UI for feeds"),
            Pref::new(
                "browser.messaging-system.whatsNewPanel.enabled",
                Val::Bool(false),
            )
            .with_comment("Enable What's New panel"),
            Pref::new("browser.rights.3.shown", Val::Bool(true))
                .with_comment("Rights notification shown"),
            Pref::new("browser.slowStartup.notificationDisabled", Val::Bool(true))
                .with_comment("Disable slow startup notification"),
            Pref::new("browser.slowStartup.maxSamples", Val::Int(0))
                .with_comment("Max samples for slow startup detection"),
            // UI tour experience (firefox.js)
            Pref::new("browser.uitour.enabled", Val::Bool(false))
                .with_comment("UI tour experience"),
            Pref::new("startup.homepage_welcome_url", Val::String(String::new()))
                .with_comment("Welcome page URL"),
            Pref::new(
                "startup.homepage_welcome_url.additional",
                Val::String(String::new()),
            )
            .with_comment("Additional welcome page URL"),
            Pref::new("startup.homepage_override_url", Val::String(String::new()))
                .with_comment("Homepage override URL"),
            // ================================================================
            // SECTION 3: Disable Telemetry & Data Collection
            // Source: modules/libpref/init/all.js
            // ================================================================
            // Whether to use the unified telemetry behavior, requires a restart
            Pref::new("toolkit.telemetry.unified", Val::Bool(false))
                .with_comment("Whether to use unified telemetry behavior"),
            Pref::new("toolkit.telemetry.enabled", Val::Bool(false))
                .with_comment("Enable telemetry"),
            // Server to submit telemetry pings to
            Pref::new("toolkit.telemetry.server", Val::String(String::new()))
                .with_comment("Server to submit telemetry pings to"),
            Pref::new("toolkit.telemetry.archive.enabled", Val::Bool(false))
                .with_comment("Enable telemetry archive"),
            Pref::new("toolkit.telemetry.newProfilePing.enabled", Val::Bool(false))
                .with_comment("Enable new profile ping"),
            Pref::new(
                "toolkit.telemetry.shutdownPingSender.enabled",
                Val::Bool(false),
            )
            .with_comment("Enable shutdown ping sender"),
            Pref::new("toolkit.telemetry.updatePing.enabled", Val::Bool(false))
                .with_comment("Enable update ping"),
            Pref::new("toolkit.telemetry.bhrPing.enabled", Val::Bool(false))
                .with_comment("Enable BHR (Background Hang Reporter) ping"),
            Pref::new(
                "toolkit.telemetry.firstShutdownPing.enabled",
                Val::Bool(false),
            )
            .with_comment("Enable first shutdown ping"),
            Pref::new(
                "toolkit.telemetry.reportingpolicy.firstRun",
                Val::Bool(false),
            )
            .with_comment("First run reporting policy"),
            Pref::new(
                "datareporting.policy.dataSubmissionEnabled",
                Val::Bool(false),
            )
            .with_comment("Enable data submission"),
            Pref::new("datareporting.healthreport.uploadEnabled", Val::Bool(false))
                .with_comment("Enable health report upload"),
            Pref::new(
                "browser.newtabpage.activity-stream.feeds.telemetry",
                Val::Bool(false),
            )
            .with_comment("Activity stream feeds telemetry"),
            Pref::new(
                "browser.newtabpage.activity-stream.telemetry",
                Val::Bool(false),
            )
            .with_comment("Activity stream telemetry"),
            Pref::new("browser.ping-centre.telemetry", Val::Bool(false))
                .with_comment("Ping centre telemetry"),
            // ================================================================
            // SECTION 4: Disable Auto-Updates
            // Source: browser/app/profile/firefox.js
            // ================================================================
            // If set to true, the Update Service will apply updates in the background
            Pref::new("app.update.staging.enabled", Val::Bool(false))
                .with_comment("Apply updates in the background when finished downloading"),
            // Whether or not to attempt using the service for updates
            Pref::new("app.update.service.enabled", Val::Bool(false))
                .with_comment("Whether to attempt using the service for updates"),
            Pref::new("extensions.update.enabled", Val::Bool(false))
                .with_comment("Check for updates to Extensions and Themes"),
            Pref::new("extensions.getAddons.cache.enabled", Val::Bool(false))
                .with_comment("Enable add-ons cache"),
            Pref::new("browser.search.update", Val::Bool(false))
                .with_comment("Enable search engine updates"),
            // ================================================================
            // SECTION 5: Disable Background Services
            // ================================================================
            Pref::new("app.normandy.enabled", Val::Bool(false))
                .with_comment("Enable Normandy/Shield studies"),
            Pref::new("app.normandy.api_url", Val::String(String::new()))
                .with_comment("Normandy API URL"),
            Pref::new("browser.safebrowsing.malware.enabled", Val::Bool(false))
                .with_comment("Enable Safe Browsing malware checks"),
            Pref::new("browser.safebrowsing.phishing.enabled", Val::Bool(false))
                .with_comment("Enable Safe Browsing phishing checks"),
            Pref::new("browser.safebrowsing.downloads.enabled", Val::Bool(false))
                .with_comment("Enable Safe Browsing download checks"),
            Pref::new("browser.safebrowsing.blockedURIs.enabled", Val::Bool(false))
                .with_comment("Enable Safe Browsing blocked URIs"),
            // Enable captive portal detection (firefox.js)
            Pref::new("network.captive-portal-service.enabled", Val::Bool(false))
                .with_comment("Enable captive portal detection"),
            Pref::new("network.connectivity-service.enabled", Val::Bool(false))
                .with_comment("Enable connectivity service"),
            // ================================================================
            // SECTION 6: Privacy - DNS Leak Prevention
            // Source: modules/libpref/init/all.js
            // ================================================================
            Pref::new("network.dns.disableIPv6", Val::Bool(true))
                .with_comment("Disable IPv6 DNS lookups"),
            Pref::new("network.proxy.socks_remote_dns", Val::Bool(true))
                .with_comment("Force DNS through SOCKS proxy"),
            // 0=off, 1=reserved, 2=TRR first, 3=TRR only, 4=reserved, 5=off by choice
            Pref::new("network.trr.mode", Val::Int(3))
                .with_comment("TRR mode: 0=off, 2=TRR first, 3=TRR only"),
            Pref::new(
                "network.trr.uri",
                Val::String("https://cloudflare-dns.com/dns-query".into()),
            )
            .with_comment("DNS-over-HTTPS server URI"),
            Pref::new("network.trr.bootstrapAddr", Val::String("1.1.1.1".into()))
                .with_comment("Bootstrap address for TRR"),
            Pref::new("network.dns.echconfig.enabled", Val::Bool(false))
                .with_comment("Enable ECH (Encrypted Client Hello)"),
            Pref::new("network.dns.http3_echconfig.enabled", Val::Bool(false))
                .with_comment("Enable HTTP/3 ECH"),
            Pref::new("security.OCSP.enabled", Val::Int(0))
                .with_comment("OCSP: 0=disabled, 1=enabled, 2=enabled for EV only"),
            Pref::new("security.ssl.enable_ocsp_stapling", Val::Bool(false))
                .with_comment("Enable OCSP stapling"),
            Pref::new("security.ssl.enable_ocsp_must_staple", Val::Bool(false))
                .with_comment("Enable OCSP must-staple"),
            // ================================================================
            // SECTION 7: Privacy - Disable Prefetching & Speculative Connections
            // Source: modules/libpref/init/all.js
            // ================================================================
            Pref::new("network.dns.disablePrefetch", Val::Bool(true))
                .with_comment("Disable DNS prefetching"),
            Pref::new("network.dns.disablePrefetchFromHTTPS", Val::Bool(true))
                .with_comment("Disable DNS prefetch from HTTPS pages"),
            // Enables the prefetch service (prefetching of <link rel=\"next\">)
            Pref::new("network.prefetch-next", Val::Bool(false))
                .with_comment("Enable prefetch service for link rel=next/prefetch"),
            // The maximum number of current global half open sockets for speculative connections
            Pref::new("network.http.speculative-parallel-limit", Val::Int(0))
                .with_comment("Max global half open sockets for speculative connections"),
            Pref::new("network.predictor.enabled", Val::Bool(false))
                .with_comment("Enable network predictor"),
            Pref::new("network.predictor.enable-prefetch", Val::Bool(false))
                .with_comment("Enable network predictor prefetch"),
            // Whether to warm up network connections for autofill or search results
            Pref::new(
                "browser.urlbar.speculativeConnect.enabled",
                Val::Bool(false),
            )
            .with_comment("Warm up network connections for autofill/search results"),
            // Whether to warm up network connections for places
            Pref::new(
                "browser.places.speculativeConnect.enabled",
                Val::Bool(false),
            )
            .with_comment("Warm up network connections for places"),
            Pref::new("browser.urlbar.suggest.searches", Val::Bool(false))
                .with_comment("Suggest searches in URL bar"),
            // ================================================================
            // SECTION 8: Privacy - WebRTC Leak Prevention
            // Source: modules/libpref/init/all.js (MOZ_WEBRTC section)
            // ================================================================
            Pref::new("media.peerconnection.enabled", Val::Bool(false))
                .with_comment("Enable WebRTC peer connections"),
            Pref::new(
                "media.peerconnection.ice.default_address_only",
                Val::Bool(true),
            )
            .with_comment("Only use default address for ICE candidates"),
            Pref::new("media.peerconnection.ice.no_host", Val::Bool(true))
                .with_comment("Don't include host candidates in ICE"),
            Pref::new(
                "media.peerconnection.ice.proxy_only_if_behind_proxy",
                Val::Bool(true),
            )
            .with_comment("Only use proxy for ICE if behind proxy"),
            // ================================================================
            // SECTION 9: Privacy - Disable Other Leak Vectors
            // Source: modules/libpref/init/all.js
            // ================================================================
            Pref::new("dom.push.enabled", Val::Bool(false)).with_comment("Enable Push API"),
            // Is the network connection allowed to be up?
            Pref::new("dom.push.connection.enabled", Val::Bool(false))
                .with_comment("Enable Push API network connection"),
            Pref::new("beacon.enabled", Val::Bool(false)).with_comment("Enable Beacon API"),
            // ================================================================
            // SECTION 10: Fingerprint Protection
            // Source: modules/libpref/init/all.js
            // ================================================================
            Pref::new("privacy.resistFingerprinting", Val::Bool(false))
                .with_comment("Enable fingerprinting resistance"),
            // ================================================================
            // SECTION 11: Performance
            // Source: modules/libpref/init/all.js
            // ================================================================
            // Enable multi by default (all.js)
            Pref::new("dom.ipc.processCount", Val::Int(1))
                .with_comment("Number of content processes"),
            Pref::new("browser.tabs.remote.autostart", Val::Bool(true))
                .with_comment("Enable multi-process tabs"),
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
