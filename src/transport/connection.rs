//! WebSocket connection and event loop.
//!
//! This module handles the WebSocket connection to Firefox extension,
//! including request/response correlation and event routing.
//!
//! See ARCHITECTURE.md Section 3.5-3.6 for event loop specification.
//!
//! # Event Loop
//!
//! The connection spawns a tokio task that handles:
//!
//! - Incoming messages from extension (responses, events)
//! - Outgoing commands from Rust API
//! - Request/response correlation by UUID
//! - Event handler callbacks

// ============================================================================
// Imports
// ============================================================================

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use serde_json::{from_str, to_string};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, trace, warn};

use crate::error::{Error, Result};
use crate::identifiers::RequestId;
use crate::protocol::{Event, EventReply, Request, Response};

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for command execution (30s per spec).
const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum pending requests before rejecting new ones.
const MAX_PENDING_REQUESTS: usize = 100;

/// Timeout for READY handshake.
const READY_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// Types
// ============================================================================

/// Map of request IDs to response channels.
type CorrelationMap = FxHashMap<RequestId, oneshot::Sender<Result<Response>>>;

/// Event handler callback type.
///
/// Called for each event received from the extension.
/// Return `Some(EventReply)` to send a reply (for network interception).
pub type EventHandler = Box<dyn Fn(Event) -> Option<EventReply> + Send + Sync>;

// ============================================================================
// ReadyData
// ============================================================================

/// Data received in the READY handshake message.
///
/// The extension sends this immediately after connecting to provide
/// initial tab and session information.
#[derive(Debug, Clone)]
pub struct ReadyData {
    /// Initial tab ID from Firefox.
    pub tab_id: u32,
    /// Session ID.
    pub session_id: u32,
}

// ============================================================================
// ConnectionCommand
// ============================================================================

/// Internal commands for the event loop.
enum ConnectionCommand {
    /// Send a request and wait for response.
    Send {
        request: Request,
        response_tx: oneshot::Sender<Result<Response>>,
    },
    /// Remove a timed-out correlation entry.
    RemoveCorrelation(RequestId),
    /// Shutdown the connection.
    Shutdown,
}

// ============================================================================
// Connection
// ============================================================================

/// WebSocket connection to Firefox extension.
///
/// Handles request/response correlation and event routing.
/// The connection spawns an internal event loop task.
///
/// # Thread Safety
///
/// `Connection` is `Send + Sync` and can be shared across tasks.
/// All operations are non-blocking.
pub struct Connection {
    /// Channel for sending commands to the event loop.
    command_tx: mpsc::UnboundedSender<ConnectionCommand>,
    /// Correlation map (shared with event loop).
    correlation: Arc<Mutex<CorrelationMap>>,
    /// Event handler (shared with event loop).
    event_handler: Arc<Mutex<Option<EventHandler>>>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
            correlation: Arc::clone(&self.correlation),
            event_handler: Arc::clone(&self.event_handler),
        }
    }
}

impl Connection {
    /// Creates a new connection from a WebSocket stream.
    ///
    /// Spawns the event loop task internally.
    pub(crate) fn new(ws_stream: WebSocketStream<TcpStream>) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let correlation = Arc::new(Mutex::new(CorrelationMap::default()));
        let event_handler: Arc<Mutex<Option<EventHandler>>> = Arc::new(Mutex::new(None));

        // Spawn event loop task
        let correlation_clone = Arc::clone(&correlation);
        let event_handler_clone = Arc::clone(&event_handler);

        tokio::spawn(Self::run_event_loop(
            ws_stream,
            command_rx,
            correlation_clone,
            event_handler_clone,
        ));

        Self {
            command_tx,
            correlation,
            event_handler,
        }
    }

    /// Waits for the READY handshake message.
    ///
    /// Must be called after connection is established.
    /// The extension sends READY with nil UUID immediately after connecting.
    ///
    /// # Errors
    ///
    /// - [`Error::ConnectionTimeout`] if READY not received within 30s
    /// - [`Error::ConnectionClosed`] if connection closes before READY
    pub async fn wait_ready(&self) -> Result<ReadyData> {
        let (tx, rx) = oneshot::channel();

        // Register correlation for READY (nil UUID)
        {
            let mut correlation = self.correlation.lock();
            correlation.insert(RequestId::ready(), tx);
        }

        // Wait for READY with timeout
        let response = timeout(READY_TIMEOUT, rx)
            .await
            .map_err(|_| Error::connection_timeout(READY_TIMEOUT.as_millis() as u64))??;

        let response = response?;

        // Extract data from READY response using helper methods
        let tab_id = response.get_u64("tabId").max(1) as u32;
        let session_id = response.get_u64("sessionId").max(1) as u32;

        debug!(tab_id, session_id, "READY handshake completed");

        Ok(ReadyData { tab_id, session_id })
    }

    /// Sets the event handler callback.
    ///
    /// The handler is called for each event received from the extension.
    /// Return `Some(EventReply)` to send a reply back.
    pub fn set_event_handler(&self, handler: EventHandler) {
        let mut guard = self.event_handler.lock();
        *guard = Some(handler);
    }

    /// Clears the event handler.
    pub fn clear_event_handler(&self) {
        let mut guard = self.event_handler.lock();
        *guard = None;
    }

    /// Sends a request and waits for response with default timeout (30s).
    ///
    /// # Errors
    ///
    /// - [`Error::ConnectionClosed`] if connection is closed
    /// - [`Error::RequestTimeout`] if response not received within timeout
    /// - [`Error::Protocol`] if too many pending requests
    pub async fn send(&self, request: Request) -> Result<Response> {
        self.send_with_timeout(request, DEFAULT_COMMAND_TIMEOUT)
            .await
    }

    /// Sends a request and waits for response with custom timeout.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to send
    /// * `request_timeout` - Maximum time to wait for response
    ///
    /// # Errors
    ///
    /// - [`Error::ConnectionClosed`] if connection is closed
    /// - [`Error::RequestTimeout`] if response not received within timeout
    /// - [`Error::Protocol`] if too many pending requests
    pub async fn send_with_timeout(
        &self,
        request: Request,
        request_timeout: Duration,
    ) -> Result<Response> {
        let request_id = request.id;

        // Check pending request limit
        {
            let correlation = self.correlation.lock();
            if correlation.len() >= MAX_PENDING_REQUESTS {
                warn!(
                    pending = correlation.len(),
                    max = MAX_PENDING_REQUESTS,
                    "Too many pending requests"
                );
                return Err(Error::protocol(format!(
                    "Too many pending requests: {}/{}",
                    correlation.len(),
                    MAX_PENDING_REQUESTS
                )));
            }
        }

        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();

        // Send command to event loop
        self.command_tx
            .send(ConnectionCommand::Send {
                request,
                response_tx,
            })
            .map_err(|_| Error::ConnectionClosed)?;

        // Wait for response with timeout
        match timeout(request_timeout, response_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(Error::ConnectionClosed),
            Err(_) => {
                // Timeout - clean up correlation entry
                let _ = self
                    .command_tx
                    .send(ConnectionCommand::RemoveCorrelation(request_id));

                Err(Error::request_timeout(
                    request_id,
                    request_timeout.as_millis() as u64,
                ))
            }
        }
    }

    /// Returns the number of pending requests.
    #[inline]
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.correlation.lock().len()
    }

    /// Shuts down the connection gracefully.
    ///
    /// This is called automatically on drop.
    pub fn shutdown(&self) {
        let _ = self.command_tx.send(ConnectionCommand::Shutdown);
    }

    /// Event loop that handles WebSocket I/O.
    async fn run_event_loop(
        ws_stream: WebSocketStream<TcpStream>,
        mut command_rx: mpsc::UnboundedReceiver<ConnectionCommand>,
        correlation: Arc<Mutex<CorrelationMap>>,
        event_handler: Arc<Mutex<Option<EventHandler>>>,
    ) {
        let (mut ws_write, mut ws_read) = ws_stream.split();

        loop {
            tokio::select! {
                // Incoming messages from extension
                message = ws_read.next() => {
                    match message {
                        Some(Ok(Message::Text(text))) => {
                            let reply = Self::handle_incoming_message(
                                &text,
                                &correlation,
                                &event_handler,
                            );

                            // Send event reply if needed
                            if let Some(reply) = reply
                                && let Ok(json) = to_string(&reply)
                                && let Err(e) = ws_write.send(Message::Text(json.into())).await
                            {
                                warn!(error = %e, "Failed to send event reply");
                            }
                        }

                        Some(Ok(Message::Close(_))) => {
                            debug!("WebSocket closed by remote");
                            break;
                        }

                        Some(Err(e)) => {
                            error!(error = %e, "WebSocket error");
                            break;
                        }

                        None => {
                            debug!("WebSocket stream ended");
                            break;
                        }

                        // Ignore Binary, Ping, Pong
                        _ => {}
                    }
                }

                // Commands from Rust API
                command = command_rx.recv() => {
                    match command {
                        Some(ConnectionCommand::Send { request, response_tx }) => {
                            Self::handle_send_command(
                                request,
                                response_tx,
                                &mut ws_write,
                                &correlation,
                            ).await;
                        }

                        Some(ConnectionCommand::RemoveCorrelation(request_id)) => {
                            correlation.lock().remove(&request_id);
                            debug!(?request_id, "Removed timed-out correlation");
                        }

                        Some(ConnectionCommand::Shutdown) => {
                            debug!("Shutdown command received");
                            let _ = ws_write.close().await;
                            break;
                        }

                        None => {
                            debug!("Command channel closed");
                            break;
                        }
                    }
                }
            }
        }

        // Fail all pending requests on shutdown
        Self::fail_pending_requests(&correlation);

        debug!("Event loop terminated");
    }

    /// Handles an incoming text message from the extension.
    fn handle_incoming_message(
        text: &str,
        correlation: &Arc<Mutex<CorrelationMap>>,
        event_handler: &Arc<Mutex<Option<EventHandler>>>,
    ) -> Option<EventReply> {
        // Try to parse as Response first
        if let Ok(response) = from_str::<Response>(text) {
            let tx = correlation.lock().remove(&response.id);

            if let Some(tx) = tx {
                let _ = tx.send(Ok(response));
            } else {
                warn!(id = %response.id, "Response for unknown request");
            }

            return None;
        }

        // Try to parse as Event
        if let Ok(event) = from_str::<Event>(text) {
            let handler = event_handler.lock();
            if let Some(ref handler) = *handler {
                return handler(event);
            }
            return None;
        }

        warn!(text = %text, "Failed to parse incoming message");
        None
    }

    /// Handles a send command from the Rust API.
    async fn handle_send_command(
        request: Request,
        response_tx: oneshot::Sender<Result<Response>>,
        ws_write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        correlation: &Arc<Mutex<CorrelationMap>>,
    ) {
        let request_id = request.id;

        // Serialize request
        let json = match to_string(&request) {
            Ok(j) => j,
            Err(e) => {
                let _ = response_tx.send(Err(Error::Json(e)));
                return;
            }
        };

        // Store correlation before sending
        correlation.lock().insert(request_id, response_tx);

        // Send over WebSocket
        if let Err(e) = ws_write.send(Message::Text(json.into())).await {
            // Remove correlation and notify caller
            if let Some(tx) = correlation.lock().remove(&request_id) {
                let _ = tx.send(Err(Error::connection(e.to_string())));
            }
        }

        trace!(?request_id, "Request sent");
    }

    /// Fails all pending requests with ConnectionClosed error.
    fn fail_pending_requests(correlation: &Arc<Mutex<CorrelationMap>>) {
        let pending: Vec<_> = correlation.lock().drain().collect();
        let count = pending.len();

        for (_, tx) in pending {
            let _ = tx.send(Err(Error::ConnectionClosed));
        }

        if count > 0 {
            debug!(count, "Failed pending requests on shutdown");
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Only shutdown if this is the last reference
        // Since command_tx is cloned, we can check if we're the only sender
        // Actually, we can't easily check this, so we should NOT auto-shutdown on drop
        // The pool.remove() will explicitly call shutdown()
        //
        // DO NOT call shutdown here - it breaks cloned connections!
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_COMMAND_TIMEOUT.as_secs(), 30);
        assert_eq!(MAX_PENDING_REQUESTS, 100);
        assert_eq!(READY_TIMEOUT.as_secs(), 30);
    }

    #[test]
    fn test_ready_data() {
        let data = ReadyData {
            tab_id: 1,
            session_id: 2,
        };
        assert_eq!(data.tab_id, 1);
        assert_eq!(data.session_id, 2);
    }
}
