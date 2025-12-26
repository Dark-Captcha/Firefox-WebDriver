//! Screenshot capture methods.

use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64Standard;
use tracing::debug;

use crate::error::{Error, Result};
use crate::protocol::command::{BrowsingContextCommand, Command};

use super::Tab;

// ============================================================================
// Types
// ============================================================================

/// Image format for screenshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    /// PNG format (lossless, larger file size).
    #[default]
    Png,
    /// JPEG format with quality (0-100).
    Jpeg(u8),
}

impl ImageFormat {
    /// Creates PNG format.
    #[inline]
    #[must_use]
    pub fn png() -> Self {
        Self::Png
    }

    /// Creates JPEG format with quality (0-100).
    #[inline]
    #[must_use]
    pub fn jpeg(quality: u8) -> Self {
        Self::Jpeg(quality.min(100))
    }

    /// Returns the MIME type for this format.
    #[must_use]
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg(_) => "image/jpeg",
        }
    }

    /// Returns the file extension for this format.
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg(_) => "jpg",
        }
    }

    /// Returns the format string for the protocol.
    fn format_str(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg(_) => "jpeg",
        }
    }

    /// Returns the quality value if JPEG.
    fn quality(&self) -> Option<u8> {
        match self {
            Self::Png => None,
            Self::Jpeg(q) => Some(*q),
        }
    }
}

// ============================================================================
// ScreenshotBuilder
// ============================================================================

/// Builder for configuring and capturing screenshots.
///
/// Uses the browser's native screenshot API (`browser.tabs.captureVisibleTab`)
/// for accurate pixel capture without JavaScript limitations.
///
/// # Example
///
/// ```ignore
/// // Capture as PNG base64
/// let png_data = tab.screenshot().png().capture().await?;
///
/// // Capture as JPEG and save to file
/// tab.screenshot().jpeg(80).save("page.jpg").await?;
/// ```
pub struct ScreenshotBuilder<'a> {
    tab: &'a Tab,
    format: ImageFormat,
}

impl<'a> ScreenshotBuilder<'a> {
    /// Creates a new screenshot builder.
    pub(crate) fn new(tab: &'a Tab) -> Self {
        Self {
            tab,
            format: ImageFormat::Png,
        }
    }

    /// Sets PNG format (default).
    #[must_use]
    pub fn png(mut self) -> Self {
        self.format = ImageFormat::Png;
        self
    }

    /// Sets JPEG format with quality (0-100).
    #[must_use]
    pub fn jpeg(mut self, quality: u8) -> Self {
        self.format = ImageFormat::Jpeg(quality.min(100));
        self
    }

    /// Sets the image format.
    #[must_use]
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }

    /// Captures the screenshot and returns base64-encoded data.
    ///
    /// Uses the browser's native `captureVisibleTab` API for accurate capture.
    pub async fn capture(&self) -> Result<String> {
        debug!(
            tab_id = %self.tab.inner.tab_id,
            format = ?self.format,
            "Capturing screenshot via browser API"
        );

        let command = Command::BrowsingContext(BrowsingContextCommand::CaptureScreenshot {
            format: self.format.format_str().to_string(),
            quality: self.format.quality(),
        });

        let response = self.tab.send_command(command).await?;

        debug!(response = ?response, "Screenshot response");

        let data = response
            .result
            .as_ref()
            .and_then(|v| v.get("data"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                let result_str = response
                    .result
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string());
                Error::script_error(format!(
                    "Screenshot response missing data field. Got: {}",
                    result_str
                ))
            })?;

        Ok(data.to_string())
    }

    /// Captures the screenshot and returns raw bytes.
    pub async fn capture_bytes(&self) -> Result<Vec<u8>> {
        let base64_data = self.capture().await?;
        Base64Standard
            .decode(&base64_data)
            .map_err(|e| Error::script_error(format!("Failed to decode base64: {}", e)))
    }

    /// Captures the screenshot and saves to a file.
    ///
    /// The format is determined by the builder settings, not the file extension.
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.capture_bytes().await?;
        std::fs::write(path.as_ref(), bytes).map_err(Error::Io)?;
        Ok(())
    }
}

// ============================================================================
// Tab - Screenshot
// ============================================================================

impl Tab {
    /// Creates a screenshot builder for capturing page screenshots.
    ///
    /// Uses the browser's native screenshot API for accurate pixel capture.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // PNG screenshot as base64
    /// let data = tab.screenshot().png().capture().await?;
    ///
    /// // JPEG screenshot saved to file
    /// tab.screenshot().jpeg(85).save("page.jpg").await?;
    /// ```
    #[must_use]
    pub fn screenshot(&self) -> ScreenshotBuilder<'_> {
        ScreenshotBuilder::new(self)
    }

    /// Captures a PNG screenshot and returns base64-encoded data.
    ///
    /// Shorthand for `tab.screenshot().png().capture().await`.
    pub async fn capture_screenshot(&self) -> Result<String> {
        self.screenshot().png().capture().await
    }

    /// Captures a screenshot and saves to a file.
    ///
    /// Format is determined by file extension (.png or .jpg/.jpeg).
    pub async fn save_screenshot(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase();

        let builder = self.screenshot();
        let builder = match ext.as_str() {
            "jpg" | "jpeg" => builder.jpeg(85),
            _ => builder.png(),
        };

        builder.save(path).await
    }
}
