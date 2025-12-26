//! Cookie and web storage methods.

use serde_json::Value;
use tracing::debug;

use crate::error::Result;
use crate::protocol::{Command, Cookie, StorageCommand};

use super::Tab;
use super::script::json_string;

// ============================================================================
// Tab - Storage (Cookies)
// ============================================================================

impl Tab {
    /// Gets a cookie by name.
    pub async fn get_cookie(&self, name: &str) -> Result<Option<Cookie>> {
        debug!(tab_id = %self.inner.tab_id, name = %name, "Getting cookie");

        let command = Command::Storage(StorageCommand::GetCookie {
            name: name.to_string(),
            url: None,
        });

        let response = self.send_command(command).await?;

        let cookie = response
            .result
            .as_ref()
            .and_then(|v| v.get("cookie"))
            .and_then(|v| serde_json::from_value::<Cookie>(v.clone()).ok());

        debug!(tab_id = %self.inner.tab_id, name = %name, found = cookie.is_some(), "Got cookie");
        Ok(cookie)
    }

    /// Sets a cookie.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::Cookie;
    ///
    /// tab.set_cookie(Cookie::new("session", "abc123")).await?;
    /// ```
    pub async fn set_cookie(&self, cookie: Cookie) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, name = %cookie.name, "Setting cookie");

        let command = Command::Storage(StorageCommand::SetCookie { cookie, url: None });
        self.send_command(command).await?;
        Ok(())
    }

    /// Deletes a cookie by name.
    pub async fn delete_cookie(&self, name: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, name = %name, "Deleting cookie");

        let command = Command::Storage(StorageCommand::DeleteCookie {
            name: name.to_string(),
            url: None,
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Gets all cookies for the current page.
    pub async fn get_all_cookies(&self) -> Result<Vec<Cookie>> {
        debug!(tab_id = %self.inner.tab_id, "Getting all cookies");

        let command = Command::Storage(StorageCommand::GetAllCookies { url: None });
        let response = self.send_command(command).await?;

        let cookies: Vec<Cookie> = response
            .result
            .as_ref()
            .and_then(|v| v.get("cookies"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<Cookie>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        debug!(tab_id = %self.inner.tab_id, count = cookies.len(), "Got all cookies");
        Ok(cookies)
    }
}

// ============================================================================
// Tab - Storage (localStorage)
// ============================================================================

impl Tab {
    /// Gets a value from localStorage.
    pub async fn local_storage_get(&self, key: &str) -> Result<Option<String>> {
        debug!(tab_id = %self.inner.tab_id, key = %key, "Getting localStorage");

        let script = format!("return localStorage.getItem({});", json_string(key));
        let value = self.execute_script(&script).await?;

        let result = match value {
            Value::Null => None,
            Value::String(s) => Some(s),
            _ => value.as_str().map(|s| s.to_string()),
        };

        debug!(tab_id = %self.inner.tab_id, key = %key, found = result.is_some(), "Got localStorage");
        Ok(result)
    }

    /// Sets a value in localStorage.
    pub async fn local_storage_set(&self, key: &str, value: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, key = %key, value_len = value.len(), "Setting localStorage");

        let script = format!(
            "localStorage.setItem({}, {});",
            json_string(key),
            json_string(value)
        );

        self.execute_script(&script).await?;
        Ok(())
    }

    /// Deletes a key from localStorage.
    pub async fn local_storage_delete(&self, key: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, key = %key, "Deleting localStorage");

        let script = format!("localStorage.removeItem({});", json_string(key));
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Clears all localStorage.
    pub async fn local_storage_clear(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Clearing localStorage");

        self.execute_script("localStorage.clear();").await?;
        Ok(())
    }
}

// ============================================================================
// Tab - Storage (sessionStorage)
// ============================================================================

impl Tab {
    /// Gets a value from sessionStorage.
    pub async fn session_storage_get(&self, key: &str) -> Result<Option<String>> {
        debug!(tab_id = %self.inner.tab_id, key = %key, "Getting sessionStorage");

        let script = format!("return sessionStorage.getItem({});", json_string(key));
        let value = self.execute_script(&script).await?;

        let result = match value {
            Value::Null => None,
            Value::String(s) => Some(s),
            _ => value.as_str().map(|s| s.to_string()),
        };

        debug!(tab_id = %self.inner.tab_id, key = %key, found = result.is_some(), "Got sessionStorage");
        Ok(result)
    }

    /// Sets a value in sessionStorage.
    pub async fn session_storage_set(&self, key: &str, value: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, key = %key, value_len = value.len(), "Setting sessionStorage");

        let script = format!(
            "sessionStorage.setItem({}, {});",
            json_string(key),
            json_string(value)
        );

        self.execute_script(&script).await?;
        Ok(())
    }

    /// Deletes a key from sessionStorage.
    pub async fn session_storage_delete(&self, key: &str) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, key = %key, "Deleting sessionStorage");

        let script = format!("sessionStorage.removeItem({});", json_string(key));
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Clears all sessionStorage.
    pub async fn session_storage_clear(&self) -> Result<()> {
        debug!(tab_id = %self.inner.tab_id, "Clearing sessionStorage");

        self.execute_script("sessionStorage.clear();").await?;
        Ok(())
    }
}
