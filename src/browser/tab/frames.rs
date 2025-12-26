//! Frame switching methods.

use serde_json::Value;
use tracing::debug;

use crate::error::{Error, Result};
use crate::identifiers::FrameId;
use crate::protocol::{BrowsingContextCommand, Command, Response};

use super::{FrameInfo, Tab};

// ============================================================================
// Tab - Frame Switching
// ============================================================================

impl Tab {
    /// Switches to a frame by iframe element.
    ///
    /// Returns a new Tab handle with the updated frame context.
    ///
    /// # Arguments
    ///
    /// * `iframe` - Element reference to an iframe
    ///
    /// # Example
    ///
    /// ```ignore
    /// let iframe = tab.find_element("iframe#content").await?;
    /// let frame_tab = tab.switch_to_frame(&iframe).await?;
    /// ```
    pub async fn switch_to_frame(&self, iframe: &crate::browser::Element) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, element_id = %iframe.id(), "Switching to frame");

        let command = Command::BrowsingContext(BrowsingContextCommand::SwitchToFrame {
            element_id: iframe.id().clone(),
        });
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to a frame by index (0-based).
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the frame
    pub async fn switch_to_frame_by_index(&self, index: usize) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, index, "Switching to frame by index");

        let command =
            Command::BrowsingContext(BrowsingContextCommand::SwitchToFrameByIndex { index });
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to a frame by URL pattern.
    ///
    /// Supports wildcards (`*` for any characters, `?` for single character).
    ///
    /// # Arguments
    ///
    /// * `url_pattern` - URL pattern with optional wildcards
    pub async fn switch_to_frame_by_url(&self, url_pattern: &str) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, url_pattern, "Switching to frame by URL");

        let command = Command::BrowsingContext(BrowsingContextCommand::SwitchToFrameByUrl {
            url_pattern: url_pattern.to_string(),
        });
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to the parent frame.
    pub async fn switch_to_parent_frame(&self) -> Result<Tab> {
        debug!(tab_id = %self.inner.tab_id, "Switching to parent frame");

        let command = Command::BrowsingContext(BrowsingContextCommand::SwitchToParentFrame);
        let response = self.send_command(command).await?;

        let frame_id = extract_frame_id(&response)?;

        Ok(Tab::new(
            self.inner.tab_id,
            FrameId::new(frame_id),
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Switches to the main (top-level) frame.
    #[must_use]
    pub fn switch_to_main_frame(&self) -> Tab {
        debug!(tab_id = %self.inner.tab_id, "Switching to main frame");

        Tab::new(
            self.inner.tab_id,
            FrameId::main(),
            self.inner.session_id,
            self.inner.window.clone(),
        )
    }

    /// Gets the count of direct child frames.
    pub async fn get_frame_count(&self) -> Result<usize> {
        debug!(tab_id = %self.inner.tab_id, "Getting frame count");
        let command = Command::BrowsingContext(BrowsingContextCommand::GetFrameCount);
        let response = self.send_command(command).await?;

        let count = response
            .result
            .as_ref()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| Error::protocol("No count in response"))?;

        debug!(tab_id = %self.inner.tab_id, count = count, "Got frame count");
        Ok(count as usize)
    }

    /// Gets information about all frames in the tab.
    pub async fn get_all_frames(&self) -> Result<Vec<FrameInfo>> {
        debug!(tab_id = %self.inner.tab_id, "Getting all frames");
        let command = Command::BrowsingContext(BrowsingContextCommand::GetAllFrames);
        let response = self.send_command(command).await?;

        let frames: Vec<FrameInfo> = response
            .result
            .as_ref()
            .and_then(|v| v.get("frames"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(parse_frame_info).collect())
            .unwrap_or_default();

        debug!(tab_id = %self.inner.tab_id, count = frames.len(), "Got all frames");
        Ok(frames)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extracts frame ID from response.
fn extract_frame_id(response: &Response) -> Result<u64> {
    response
        .result
        .as_ref()
        .and_then(|v| v.get("frameId"))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Error::protocol("No frameId in response"))
}

/// Parses frame info from JSON value.
fn parse_frame_info(v: &Value) -> Option<FrameInfo> {
    Some(FrameInfo {
        frame_id: FrameId::new(v.get("frameId")?.as_u64()?),
        parent_frame_id: v
            .get("parentFrameId")
            .and_then(|p| p.as_i64())
            .and_then(|p| {
                if p < 0 {
                    None
                } else {
                    Some(FrameId::new(p as u64))
                }
            }),
        url: v.get("url")?.as_str()?.to_string(),
    })
}
