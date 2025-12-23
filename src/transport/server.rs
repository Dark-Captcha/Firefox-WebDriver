//! WebSocket server for Firefox communication.
//!
//! This module provides the WebSocket server that Firefox extension connects to.
//!
//! See ARCHITECTURE.md Section 3.1-3.4 for connection model.
//!
//! # Connection Flow
//!
//! 1. Rust binds WebSocket server to `localhost:0` (random port)
//! 2. Firefox launches with extension and data URI containing WebSocket URL
//! 3. Extension connects to WebSocket server
//! 4. Extension sends READY message with initial tab/session IDs
//! 5. Connection established, ready for commands

// ============================================================================
// Imports
// ============================================================================

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::time::timeout;
use tracing::{debug, info};

use crate::error::{Error, Result};

use super::Connection;
use super::connection::ReadyData;

// ============================================================================
// Constants
// ============================================================================

/// Timeout for waiting for Firefox to connect (30s per spec).
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// PendingServer
// ============================================================================

/// A WebSocket server that is bound but not yet connected.
///
/// Represents the state between binding to a port and accepting
/// the Firefox extension's connection.
///
/// # Example
///
/// ```ignore
/// use std::net::{IpAddr, Ipv4Addr};
/// use firefox_webdriver::transport::PendingServer;
///
/// let server = PendingServer::bind(IpAddr::V4(Ipv4Addr::LOCALHOST), 0).await?;
/// let ws_url = server.ws_url();
///
/// // Launch Firefox with ws_url...
///
/// let (connection, ready_data) = server.accept().await?;
/// ```
pub struct PendingServer {
    /// TCP listener for incoming connections.
    listener: TcpListener,
    /// Port the server is bound to.
    port: u16,
}

impl PendingServer {
    /// Binds a WebSocket server to the specified address and port.
    ///
    /// Use port 0 to let the OS assign a random available port.
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address to bind to (typically localhost)
    /// * `port` - Port to bind to (0 for random)
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if binding fails.
    pub async fn bind(ip: IpAddr, port: u16) -> Result<Self> {
        let addr = SocketAddr::new(ip, port);
        let listener = TcpListener::bind(addr).await?;
        let actual_port = listener.local_addr()?.port();

        debug!(port = actual_port, "WebSocket server bound");

        Ok(Self {
            listener,
            port: actual_port,
        })
    }

    /// Returns the port the server is bound to.
    #[inline]
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the WebSocket URL for this server.
    ///
    /// Format: `ws://127.0.0.1:{port}`
    #[inline]
    #[must_use]
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Returns the local socket address.
    #[inline]
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), self.port)
    }

    /// Accepts a connection from Firefox and completes the handshake.
    ///
    /// This method:
    /// 1. Waits for TCP connection (with timeout)
    /// 2. Upgrades to WebSocket
    /// 3. Waits for READY handshake message
    ///
    /// # Returns
    ///
    /// Tuple of ([`Connection`], [`ReadyData`]) on success.
    ///
    /// # Errors
    ///
    /// - [`Error::ConnectionTimeout`] if Firefox doesn't connect within 30s
    /// - [`Error::Connection`] if WebSocket upgrade fails
    /// - [`Error::Protocol`] if READY handshake fails
    pub async fn accept(self) -> Result<(Connection, ReadyData)> {
        // Wait for Firefox to connect with timeout
        let accept_result = timeout(CONNECTION_TIMEOUT, self.listener.accept()).await;

        let (stream, addr) = accept_result
            .map_err(|_| Error::connection_timeout(CONNECTION_TIMEOUT.as_millis() as u64))??;

        debug!(?addr, "TCP connection accepted");

        // Upgrade to WebSocket
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .map_err(|e| Error::connection(format!("WebSocket upgrade failed: {e}")))?;

        info!(port = self.port, "WebSocket connection established");

        // Create connection and wait for READY handshake
        let connection = Connection::new(ws_stream);
        let ready_data = connection.wait_ready().await?;

        Ok((connection, ready_data))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_server_bind_random_port() {
        let server = PendingServer::bind(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
            .await
            .expect("bind should succeed");

        assert!(server.port() > 0);
        assert!(server.ws_url().starts_with("ws://127.0.0.1:"));
    }

    #[tokio::test]
    async fn test_server_ws_url_format() {
        let server = PendingServer::bind(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
            .await
            .expect("bind should succeed");

        let url = server.ws_url();
        let expected = format!("ws://127.0.0.1:{}", server.port());
        assert_eq!(url, expected);
    }

    #[tokio::test]
    async fn test_server_local_addr() {
        let server = PendingServer::bind(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
            .await
            .expect("bind should succeed");

        let addr = server.local_addr();
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(addr.port(), server.port());
    }

    #[tokio::test]
    async fn test_bind_specific_port() {
        // Find an available port first
        let temp_server = PendingServer::bind(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
            .await
            .expect("bind should succeed");
        let port = temp_server.port();
        drop(temp_server);

        // Try to bind to that specific port
        let result = PendingServer::bind(IpAddr::V4(Ipv4Addr::LOCALHOST), port).await;
        // May or may not succeed depending on OS port reuse timing
        // Just verify it doesn't panic
        let _ = result;
    }
}
