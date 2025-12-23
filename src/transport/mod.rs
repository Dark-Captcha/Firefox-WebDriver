//! WebSocket transport layer.
//!
//! This module handles communication between local end (Rust) and
//! remote end (Extension) via WebSocket.
//!
//! See ARCHITECTURE.md Section 3 for transport specification.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐                              ┌─────────────────┐
//! │  Window (Rust)  │                              │  Extension      │
//! │                 │         WebSocket            │  (Background)   │
//! │  PendingServer  │◄────────────────────────────►│                 │
//! │  → Connection   │      localhost:PORT          │  WebSocket      │
//! │                 │                              │  Client         │
//! └─────────────────┘                              └─────────────────┘
//! ```
//!
//! # Connection Lifecycle
//!
//! 1. `PendingServer::bind` - Bind to localhost with random port
//! 2. Launch Firefox with extension and WebSocket URL
//! 3. `PendingServer::accept` - Wait for extension to connect
//! 4. `Connection` - Send commands, receive responses/events
//! 5. `Connection::shutdown` - Close connection on drop
//!
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | `connection` | WebSocket connection and event loop |
//! | `server` | WebSocket server binding and acceptance |

// ============================================================================
// Submodules
// ============================================================================

/// WebSocket connection and event loop.
pub mod connection;

/// WebSocket server for Firefox communication.
pub mod server;

// ============================================================================
// Re-exports
// ============================================================================

pub use connection::{Connection, EventHandler, ReadyData};
pub use server::PendingServer;
