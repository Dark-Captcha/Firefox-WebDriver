//! Core Tab struct and accessors.

use std::fmt;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::identifiers::{FrameId, SessionId, TabId};
use crate::protocol::{Command, Request, Response};

use crate::browser::Window;

// ============================================================================
// Types
// ============================================================================

/// Information about a frame in the tab.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// Frame ID.
    pub frame_id: FrameId,
    /// Parent frame ID (None for main frame).
    pub parent_frame_id: Option<FrameId>,
    /// Frame URL.
    pub url: String,
}

/// Internal shared state for a tab.
pub(crate) struct TabInner {
    /// Tab ID.
    pub tab_id: TabId,
    /// Current frame ID.
    pub frame_id: FrameId,
    /// Session ID.
    pub session_id: SessionId,
    /// Parent window (optional for standalone tab references).
    pub window: Option<Window>,
}

// ============================================================================
// Tab
// ============================================================================

/// A handle to a browser tab.
///
/// Tabs provide methods for navigation, scripting, and element interaction.
#[derive(Clone)]
pub struct Tab {
    pub(crate) inner: Arc<TabInner>,
}

impl fmt::Debug for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tab")
            .field("tab_id", &self.inner.tab_id)
            .field("frame_id", &self.inner.frame_id)
            .field("session_id", &self.inner.session_id)
            .finish_non_exhaustive()
    }
}

impl Tab {
    /// Creates a new tab handle.
    pub(crate) fn new(
        tab_id: TabId,
        frame_id: FrameId,
        session_id: SessionId,
        window: Option<Window>,
    ) -> Self {
        Self {
            inner: Arc::new(TabInner {
                tab_id,
                frame_id,
                session_id,
                window,
            }),
        }
    }
}

// ============================================================================
// Tab - Accessors
// ============================================================================

impl Tab {
    /// Returns the tab ID.
    #[inline]
    #[must_use]
    pub fn tab_id(&self) -> TabId {
        self.inner.tab_id
    }

    /// Returns the current frame ID.
    #[inline]
    #[must_use]
    pub fn frame_id(&self) -> FrameId {
        self.inner.frame_id
    }

    /// Returns the session ID.
    #[inline]
    #[must_use]
    pub fn session_id(&self) -> SessionId {
        self.inner.session_id
    }

    /// Checks if currently in the main frame.
    #[inline]
    #[must_use]
    pub fn is_main_frame(&self) -> bool {
        self.inner.frame_id.is_main()
    }
}

// ============================================================================
// Tab - Internal
// ============================================================================

impl Tab {
    /// Sends a command and returns the response.
    pub(crate) async fn send_command(&self, command: Command) -> Result<Response> {
        let window = self.get_window()?;
        let request = Request::new(self.inner.tab_id, self.inner.frame_id, command);
        window
            .inner
            .pool
            .send(window.inner.session_id, request)
            .await
    }

    /// Gets the window reference or returns an error.
    pub(crate) fn get_window(&self) -> Result<&Window> {
        self.inner
            .window
            .as_ref()
            .ok_or_else(|| Error::protocol("Tab has no associated window"))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::Tab;

    #[test]
    fn test_tab_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<Tab>();
    }

    #[test]
    fn test_tab_is_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<Tab>();
    }
}
