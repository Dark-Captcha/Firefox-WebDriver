//! JavaScript execution methods.

use serde_json::Value;
use tracing::debug;

use crate::error::Result;
use crate::protocol::{Command, ScriptCommand};

use super::Tab;

// ============================================================================
// Tab - Script Execution
// ============================================================================

impl Tab {
    /// Executes synchronous JavaScript in the page context.
    ///
    /// The script should use `return` to return a value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let title = tab.execute_script("return document.title").await?;
    /// ```
    pub async fn execute_script(&self, script: &str) -> Result<Value> {
        debug!(tab_id = %self.inner.tab_id, script_len = script.len(), "Executing script");

        let command = Command::Script(ScriptCommand::Evaluate {
            script: script.to_string(),
            args: vec![],
        });

        let response = self.send_command(command).await?;

        let value = response
            .result
            .as_ref()
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(Value::Null);

        debug!(tab_id = %self.inner.tab_id, "Script executed");
        Ok(value)
    }

    /// Executes asynchronous JavaScript in the page context.
    ///
    /// The script should return a Promise or use async/await.
    pub async fn execute_async_script(&self, script: &str) -> Result<Value> {
        debug!(tab_id = %self.inner.tab_id, script_len = script.len(), "Executing async script");

        let command = Command::Script(ScriptCommand::EvaluateAsync {
            script: script.to_string(),
            args: vec![],
        });

        let response = self.send_command(command).await?;

        let value = response
            .result
            .as_ref()
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(Value::Null);

        debug!(tab_id = %self.inner.tab_id, "Async script executed");
        Ok(value)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Escapes a string for safe use in JavaScript.
pub(crate) fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
}
