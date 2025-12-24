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
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     ConnectionPool                          │
//! │                     (single port)                           │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │ SessionId=1 → Connection 1 ──► Firefox 1            │   │
//! │  │ SessionId=2 → Connection 2 ──► Firefox 2            │   │
//! │  │ SessionId=3 → Connection 3 ──► Firefox 3            │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Connection Lifecycle
//!
//! 1. `ConnectionPool::new` - Bind to localhost with random port
//! 2. Launch Firefox with extension and WebSocket URL
//! 3. Pool accepts connection, waits for READY with SessionId
//! 4. `Connection` - Send commands, receive responses/events
//! 5. `ConnectionPool::remove` - Clean up on window close
//!
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | `connection` | WebSocket connection and event loop |
//! | `pool` | Connection pool for multiplexed connections |

// ============================================================================
// Submodules
// ============================================================================

/// WebSocket connection and event loop.
pub mod connection;

/// Connection pool for multiplexed WebSocket connections.
pub mod pool;

// ============================================================================
// Re-exports
// ============================================================================

pub use connection::{Connection, EventHandler, ReadyData};
pub use pool::ConnectionPool;
