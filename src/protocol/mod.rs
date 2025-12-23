//! WebSocket protocol message types.
//!
//! This module defines the message format for communication between
//! local end (Rust) and remote end (Extension).
//!
//! # Protocol Overview
//!
//! From ARCHITECTURE.md Section 2:
//!
//! | Message Type | Direction | Purpose |
//! |--------------|-----------|---------|
//! | `Request` | Local → Remote | Command request |
//! | `Response` | Remote → Local | Command response |
//! | `Event` | Remote → Local | Browser notification |
//! | `EventReply` | Local → Remote | Event decision |
//!
//! # Command Naming
//!
//! Commands follow `module.methodName` format:
//!
//! - `browsingContext.navigate`
//! - `element.find`
//! - `network.addIntercept`
//!
//! # Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | `command` | Command definitions by domain |
//! | `event` | Event and EventReply types |
//! | `request` | Request and Response types |

// ============================================================================
// Submodules
// ============================================================================

/// Command definitions organized by module.
pub mod command;

/// Event message types.
pub mod event;

/// Request and Response message types.
pub mod request;

// ============================================================================
// Re-exports
// ============================================================================

pub use command::{
    BrowsingContextCommand, Command, Cookie, ElementCommand, InputCommand, NetworkCommand,
    ProxyCommand, ScriptCommand, SessionCommand, StorageCommand,
};
pub use event::{Event, EventReply, ParsedEvent};
pub use request::{Request, Response, ResponseType};
