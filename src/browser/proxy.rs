//! Proxy configuration types.
//!
//! Types for configuring per-tab and per-window proxy settings.
//!
//! # Example
//!
//! ```
//! use firefox_webdriver::ProxyConfig;
//!
//! // HTTP proxy without auth
//! let proxy = ProxyConfig::http("proxy.example.com", 8080);
//!
//! // SOCKS5 proxy with auth
//! let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
//!     .with_credentials("user", "pass")
//!     .with_proxy_dns(true);
//! ```

// ============================================================================
// Imports
// ============================================================================

use serde::{Deserialize, Serialize};

// ============================================================================
// ProxyType
// ============================================================================

/// Proxy protocol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    /// HTTP proxy (or SSL CONNECT for HTTPS).
    Http,

    /// HTTP proxying over TLS connection to proxy.
    Https,

    /// SOCKS v4 proxy.
    Socks4,

    /// SOCKS v5 proxy.
    #[serde(rename = "socks")]
    Socks5,

    /// Direct connection (no proxy).
    #[default]
    Direct,
}

// ============================================================================
// ProxyType - Implementation
// ============================================================================

impl ProxyType {
    /// Returns the string representation for the extension.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks4 => "socks4",
            Self::Socks5 => "socks",
            Self::Direct => "direct",
        }
    }
}

// ============================================================================
// ProxyConfig
// ============================================================================

/// Proxy configuration.
///
/// # Example
///
/// ```
/// use firefox_webdriver::ProxyConfig;
///
/// // HTTP proxy without auth
/// let proxy = ProxyConfig::http("proxy.example.com", 8080);
///
/// // SOCKS5 proxy with auth
/// let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
///     .with_credentials("user", "pass")
///     .with_proxy_dns(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy type.
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,

    /// Proxy hostname.
    pub host: String,

    /// Proxy port.
    pub port: u16,

    /// Username for authentication (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for authentication (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Whether to proxy DNS queries (SOCKS4/SOCKS5 only).
    #[serde(rename = "proxyDns", default)]
    pub proxy_dns: bool,
}

// ============================================================================
// ProxyConfig - Constructors
// ============================================================================

impl ProxyConfig {
    /// Creates a new proxy configuration.
    ///
    /// # Arguments
    ///
    /// * `proxy_type` - Proxy protocol type
    /// * `host` - Proxy hostname
    /// * `port` - Proxy port
    #[must_use]
    pub fn new(proxy_type: ProxyType, host: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type,
            host: host.into(),
            port,
            username: None,
            password: None,
            proxy_dns: false,
        }
    }

    /// Creates an HTTP proxy configuration.
    #[inline]
    #[must_use]
    pub fn http(host: impl Into<String>, port: u16) -> Self {
        Self::new(ProxyType::Http, host, port)
    }

    /// Creates an HTTPS proxy configuration.
    #[inline]
    #[must_use]
    pub fn https(host: impl Into<String>, port: u16) -> Self {
        Self::new(ProxyType::Https, host, port)
    }

    /// Creates a SOCKS4 proxy configuration.
    #[inline]
    #[must_use]
    pub fn socks4(host: impl Into<String>, port: u16) -> Self {
        Self::new(ProxyType::Socks4, host, port)
    }

    /// Creates a SOCKS5 proxy configuration.
    #[inline]
    #[must_use]
    pub fn socks5(host: impl Into<String>, port: u16) -> Self {
        Self::new(ProxyType::Socks5, host, port)
    }

    /// Creates a direct (no proxy) configuration.
    #[inline]
    #[must_use]
    pub fn direct() -> Self {
        Self {
            proxy_type: ProxyType::Direct,
            host: String::new(),
            port: 0,
            username: None,
            password: None,
            proxy_dns: false,
        }
    }
}

// ============================================================================
// ProxyConfig - Builder Methods
// ============================================================================

impl ProxyConfig {
    /// Sets authentication credentials.
    ///
    /// # Arguments
    ///
    /// * `username` - Proxy username
    /// * `password` - Proxy password
    #[must_use]
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Enables DNS proxying (SOCKS4/SOCKS5 only).
    ///
    /// When enabled, DNS queries are sent through the proxy.
    #[must_use]
    pub fn with_proxy_dns(mut self, proxy_dns: bool) -> Self {
        self.proxy_dns = proxy_dns;
        self
    }
}

// ============================================================================
// ProxyConfig - Predicates
// ============================================================================

impl ProxyConfig {
    /// Returns `true` if this proxy has authentication configured.
    #[inline]
    #[must_use]
    pub fn has_auth(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }

    /// Returns `true` if this is a SOCKS proxy.
    #[inline]
    #[must_use]
    pub fn is_socks(&self) -> bool {
        matches!(self.proxy_type, ProxyType::Socks4 | ProxyType::Socks5)
    }

    /// Returns `true` if this is an HTTP/HTTPS proxy.
    #[inline]
    #[must_use]
    pub fn is_http(&self) -> bool {
        matches!(self.proxy_type, ProxyType::Http | ProxyType::Https)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{ProxyConfig, ProxyType};

    // ------------------------------------------------------------------------
    // ProxyType Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_proxy_type_as_str() {
        assert_eq!(ProxyType::Http.as_str(), "http");
        assert_eq!(ProxyType::Https.as_str(), "https");
        assert_eq!(ProxyType::Socks4.as_str(), "socks4");
        assert_eq!(ProxyType::Socks5.as_str(), "socks");
        assert_eq!(ProxyType::Direct.as_str(), "direct");
    }

    #[test]
    fn test_proxy_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ProxyType::Http).unwrap(),
            r#""http""#
        );
        assert_eq!(
            serde_json::to_string(&ProxyType::Https).unwrap(),
            r#""https""#
        );
        assert_eq!(
            serde_json::to_string(&ProxyType::Socks4).unwrap(),
            r#""socks4""#
        );
        assert_eq!(
            serde_json::to_string(&ProxyType::Socks5).unwrap(),
            r#""socks""#
        );
        assert_eq!(
            serde_json::to_string(&ProxyType::Direct).unwrap(),
            r#""direct""#
        );
    }

    #[test]
    fn test_proxy_type_default() {
        assert_eq!(ProxyType::default(), ProxyType::Direct);
    }

    // ------------------------------------------------------------------------
    // ProxyConfig Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_proxy_config_http() {
        let proxy = ProxyConfig::http("proxy.example.com", 8080);
        assert_eq!(proxy.proxy_type, ProxyType::Http);
        assert_eq!(proxy.host, "proxy.example.com");
        assert_eq!(proxy.port, 8080);
        assert!(!proxy.has_auth());
        assert!(proxy.is_http());
        assert!(!proxy.is_socks());
    }

    #[test]
    fn test_proxy_config_https() {
        let proxy = ProxyConfig::https("proxy.example.com", 8443);
        assert_eq!(proxy.proxy_type, ProxyType::Https);
        assert!(proxy.is_http());
    }

    #[test]
    fn test_proxy_config_socks4() {
        let proxy = ProxyConfig::socks4("proxy.example.com", 1080);
        assert_eq!(proxy.proxy_type, ProxyType::Socks4);
        assert!(proxy.is_socks());
        assert!(!proxy.is_http());
    }

    #[test]
    fn test_proxy_config_socks5() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080);
        assert_eq!(proxy.proxy_type, ProxyType::Socks5);
        assert!(proxy.is_socks());
    }

    #[test]
    fn test_proxy_config_direct() {
        let proxy = ProxyConfig::direct();
        assert_eq!(proxy.proxy_type, ProxyType::Direct);
        assert!(!proxy.is_http());
        assert!(!proxy.is_socks());
    }

    #[test]
    fn test_proxy_config_with_auth() {
        let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
            .with_credentials("user", "pass")
            .with_proxy_dns(true);

        assert_eq!(proxy.proxy_type, ProxyType::Socks5);
        assert!(proxy.has_auth());
        assert!(proxy.is_socks());
        assert!(proxy.proxy_dns);
        assert_eq!(proxy.username.as_deref(), Some("user"));
        assert_eq!(proxy.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_proxy_config_serialization() {
        let proxy = ProxyConfig::http("proxy.example.com", 8080).with_credentials("user", "pass");

        let json = serde_json::to_string(&proxy).unwrap();
        assert!(json.contains(r#""type":"http""#));
        assert!(json.contains(r#""host":"proxy.example.com""#));
        assert!(json.contains(r#""port":8080"#));
        assert!(json.contains(r#""username":"user""#));
    }

    #[test]
    fn test_proxy_config_clone() {
        let proxy = ProxyConfig::http("proxy.example.com", 8080);
        let cloned = proxy.clone();
        assert_eq!(proxy.host, cloned.host);
        assert_eq!(proxy.port, cloned.port);
    }
}
