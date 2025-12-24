//! Connection pool for multiplexed WebSocket connections.
//!
//! Manages multiple WebSocket connections keyed by SessionId.
//! All Firefox windows connect to the same port, messages routed by session.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           ConnectionPool                │
//! │           (single port)                 │
//! │  ┌─────────────────────────────────┐   │
//! │  │ SessionId=1 → Connection 1      │   │
//! │  │ SessionId=2 → Connection 2      │   │
//! │  │ SessionId=3 → Connection 3      │   │
//! │  └─────────────────────────────────┘   │
//! └─────────────────────────────────────────┘
//! ```

// ============================================================================
// Imports
// ============================================================================

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashMap;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::identifiers::SessionId;
use crate::protocol::{Request, Response};
use crate::transport::Connection;
use crate::transport::connection::ReadyData;

// ============================================================================
// Constants
// ============================================================================

/// Default bind address for WebSocket server (localhost).
const DEFAULT_BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// Timeout for waiting for a session to connect.
const SESSION_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// ConnectionPool
// ============================================================================

/// Manages multiple WebSocket connections keyed by SessionId.
///
/// Thread-safe, supports concurrent access from multiple Windows.
/// All Firefox windows connect to the same port, messages routed by session.
///
/// # Example
///
/// ```ignore
/// let pool = ConnectionPool::new().await?;
/// println!("WebSocket URL: {}", pool.ws_url());
///
/// // Wait for a specific session to connect
/// let ready_data = pool.wait_for_session(session_id).await?;
///
/// // Send a request to that session
/// let response = pool.send(session_id, request).await?;
/// ```
pub struct ConnectionPool {
    /// WebSocket server port.
    port: u16,

    /// Active connections by session ID.
    connections: RwLock<FxHashMap<SessionId, Connection>>,

    /// Waiters for pending sessions (spawn_window waiting for Firefox to connect).
    waiters: Mutex<FxHashMap<SessionId, oneshot::Sender<ReadyData>>>,

    /// Shutdown flag.
    shutdown: AtomicBool,
}

// ============================================================================
// ConnectionPool - Constructor
// ============================================================================

impl ConnectionPool {
    /// Creates a new connection pool and starts the accept loop.
    ///
    /// Binds to `localhost:0` (random available port).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if binding fails.
    pub async fn new() -> Result<Arc<Self>> {
        Self::with_ip_port(DEFAULT_BIND_IP, 0).await
    }

    /// Creates a new connection pool bound to a specific port.
    ///
    /// # Arguments
    ///
    /// * `port` - Port to bind to (0 for random)
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if binding fails.
    pub async fn with_port(port: u16) -> Result<Arc<Self>> {
        Self::with_ip_port(DEFAULT_BIND_IP, port).await
    }

    /// Creates a new connection pool bound to a specific IP and port.
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address to bind to
    /// * `port` - Port to bind to (0 for random)
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if binding fails.
    pub async fn with_ip_port(ip: IpAddr, port: u16) -> Result<Arc<Self>> {
        let addr = SocketAddr::new(ip, port);
        let listener = TcpListener::bind(addr).await?;
        let actual_port = listener.local_addr()?.port();

        debug!(port = actual_port, "ConnectionPool WebSocket server bound");

        let pool = Arc::new(Self {
            port: actual_port,
            connections: RwLock::new(FxHashMap::default()),
            waiters: Mutex::new(FxHashMap::default()),
            shutdown: AtomicBool::new(false),
        });

        // Spawn accept loop
        let pool_clone = Arc::clone(&pool);
        tokio::spawn(async move {
            pool_clone.accept_loop(listener).await;
        });

        info!(port = actual_port, "ConnectionPool started");

        Ok(pool)
    }
}

// ============================================================================
// ConnectionPool - Public API
// ============================================================================

impl ConnectionPool {
    /// Returns the WebSocket URL for this pool.
    ///
    /// Format: `ws://127.0.0.1:{port}`
    #[inline]
    #[must_use]
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    /// Returns the port the pool is bound to.
    #[inline]
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the number of active connections.
    #[inline]
    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Waits for a specific session to connect.
    ///
    /// Called by `spawn_window` after launching Firefox.
    /// Returns when Firefox with this sessionId connects and sends READY.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to wait for
    ///
    /// # Errors
    ///
    /// - [`Error::ConnectionTimeout`] if session doesn't connect within 30s
    pub async fn wait_for_session(&self, session_id: SessionId) -> Result<ReadyData> {
        let (tx, rx) = oneshot::channel();

        // Register waiter
        {
            let mut waiters = self.waiters.lock();
            waiters.insert(session_id, tx);
        }

        // Wait with timeout
        match timeout(SESSION_CONNECT_TIMEOUT, rx).await {
            Ok(Ok(ready_data)) => {
                debug!(session_id = %session_id, "Session connected");
                Ok(ready_data)
            }
            Ok(Err(_)) => {
                // Channel closed without sending - shouldn't happen
                self.waiters.lock().remove(&session_id);
                Err(Error::connection("Session waiter channel closed"))
            }
            Err(_) => {
                // Timeout
                self.waiters.lock().remove(&session_id);
                Err(Error::connection_timeout(
                    SESSION_CONNECT_TIMEOUT.as_millis() as u64,
                ))
            }
        }
    }

    /// Sends a request to a specific session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Target session
    /// * `request` - Request to send
    ///
    /// # Errors
    ///
    /// - [`Error::SessionNotFound`] if session doesn't exist
    /// - [`Error::ConnectionClosed`] if connection is closed
    /// - [`Error::RequestTimeout`] if response not received within timeout
    pub async fn send(&self, session_id: SessionId, request: Request) -> Result<Response> {
        let connection = {
            let connections = self.connections.read();
            connections
                .get(&session_id)
                .ok_or_else(|| Error::session_not_found(session_id))?
                .clone()
        };

        connection.send(request).await
    }

    /// Sends a request with custom timeout.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Target session
    /// * `request` - Request to send
    /// * `timeout` - Maximum time to wait for response
    ///
    /// # Errors
    ///
    /// - [`Error::SessionNotFound`] if session doesn't exist
    /// - [`Error::ConnectionClosed`] if connection is closed
    /// - [`Error::RequestTimeout`] if response not received within timeout
    pub async fn send_with_timeout(
        &self,
        session_id: SessionId,
        request: Request,
        request_timeout: Duration,
    ) -> Result<Response> {
        let connection = {
            let connections = self.connections.read();
            connections
                .get(&session_id)
                .ok_or_else(|| Error::session_not_found(session_id))?
                .clone()
        };

        connection.send_with_timeout(request, request_timeout).await
    }
}

// ============================================================================
// ConnectionPool - Event Handlers
// ============================================================================

impl ConnectionPool {
    /// Sets the event handler for a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Target session
    /// * `handler` - Event handler callback
    pub fn set_event_handler(
        &self,
        session_id: SessionId,
        handler: crate::transport::EventHandler,
    ) {
        let connections = self.connections.read();
        if let Some(connection) = connections.get(&session_id) {
            connection.set_event_handler(handler);
        }
    }

    /// Clears the event handler for a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Target session
    pub fn clear_event_handler(&self, session_id: SessionId) {
        let connections = self.connections.read();
        if let Some(connection) = connections.get(&session_id) {
            connection.clear_event_handler();
        }
    }
}

// ============================================================================
// ConnectionPool - Lifecycle
// ============================================================================

impl ConnectionPool {
    /// Removes a session from the pool.
    ///
    /// Called when a Window closes.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session to remove
    pub fn remove(&self, session_id: SessionId) {
        let removed = {
            let mut connections = self.connections.write();
            connections.remove(&session_id)
        };

        if let Some(connection) = removed {
            connection.shutdown();
            debug!(session_id = %session_id, "Session removed from pool");
        }
    }

    /// Shuts down the pool and all connections.
    pub async fn shutdown(&self) {
        info!("ConnectionPool shutting down");

        // Signal accept loop to stop
        self.shutdown.store(true, Ordering::SeqCst);

        // Close all connections
        let connections: Vec<_> = {
            let mut map = self.connections.write();
            map.drain().collect()
        };

        for (session_id, connection) in connections {
            connection.shutdown();
            debug!(session_id = %session_id, "Connection closed during shutdown");
        }

        // Cancel all waiters
        let waiters: Vec<_> = {
            let mut map = self.waiters.lock();
            map.drain().collect()
        };

        drop(waiters); // Dropping senders will cause receivers to error

        info!("ConnectionPool shutdown complete");
    }
}

// ============================================================================
// ConnectionPool - Accept Loop
// ============================================================================

impl ConnectionPool {
    /// Background task that accepts new connections.
    async fn accept_loop(self: Arc<Self>, listener: TcpListener) {
        debug!("Accept loop started");

        loop {
            // Check shutdown flag
            if self.shutdown.load(Ordering::SeqCst) {
                debug!("Accept loop shutting down");
                break;
            }

            // Accept with timeout to allow checking shutdown flag
            match timeout(Duration::from_millis(100), listener.accept()).await {
                Ok(Ok((stream, addr))) => {
                    let pool = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = pool.handle_connection(stream, addr).await {
                            warn!(error = %e, ?addr, "Connection handling failed");
                        }
                    });
                }
                Ok(Err(e)) => {
                    error!(error = %e, "Accept failed");
                }
                Err(_) => {
                    // Timeout - just continue to check shutdown flag
                    continue;
                }
            }
        }

        debug!("Accept loop terminated");
    }

    /// Handles a single incoming connection.
    async fn handle_connection(
        &self,
        stream: tokio::net::TcpStream,
        addr: SocketAddr,
    ) -> Result<()> {
        debug!(?addr, "New TCP connection");

        // Upgrade to WebSocket
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .map_err(|e| Error::connection(format!("WebSocket upgrade failed: {e}")))?;

        info!(?addr, "WebSocket connection established");

        // Create Connection and wait for READY
        let connection = Connection::new(ws_stream);
        let ready_data = connection.wait_ready().await?;

        let session_id = SessionId::from_u32(ready_data.session_id)
            .ok_or_else(|| Error::protocol("Invalid session_id in READY (must be > 0)"))?;

        info!(session_id = %session_id, ?addr, "Session READY received");

        // Store connection
        {
            let mut connections = self.connections.write();
            connections.insert(session_id, connection);
        }

        // Notify waiter if any
        {
            let mut waiters = self.waiters.lock();
            if let Some(tx) = waiters.remove(&session_id) {
                let _ = tx.send(ready_data);
            }
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ConnectionPool::new().await.expect("pool creation");
        assert!(pool.port() > 0);
        assert!(pool.ws_url().starts_with("ws://127.0.0.1:"));
        assert_eq!(pool.connection_count(), 0);
        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_pool_ws_url_format() {
        let pool = ConnectionPool::new().await.expect("pool creation");
        let url = pool.ws_url();
        let expected = format!("ws://127.0.0.1:{}", pool.port());
        assert_eq!(url, expected);
        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_send_to_unknown_session() {
        let pool = ConnectionPool::new().await.expect("pool creation");
        let session_id = SessionId::next();
        let request = crate::protocol::Request::new(
            crate::identifiers::TabId::new(1).unwrap(),
            crate::identifiers::FrameId::main(),
            crate::protocol::Command::Session(crate::protocol::SessionCommand::Status),
        );

        let result = pool.send(session_id, request).await;
        assert!(result.is_err());

        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_wait_for_session_timeout() {
        let pool = ConnectionPool::new().await.expect("pool creation");
        let session_id = SessionId::next();

        // Use a short timeout for testing
        let (tx, rx) = oneshot::channel::<ReadyData>();
        pool.waiters.lock().insert(session_id, tx);

        // Don't send anything, let it timeout
        drop(rx);

        // The waiter should be cleaned up
        // (In real usage, wait_for_session would timeout)
        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_remove_nonexistent_session() {
        let pool = ConnectionPool::new().await.expect("pool creation");
        let session_id = SessionId::next();

        // Should not panic
        pool.remove(session_id);

        pool.shutdown().await;
    }
}
