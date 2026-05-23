//! Version 2 of the chess server protocol.
//!
//! This module contains the event and command definitions used by the
//! next-generation chess server (another-chess-server).
//!
//! ## Organization
//!
//! - [`types`] - Core types like [`User`], [`UserId`], and [`RoomId`]
//! - [`events`] - Events broadcast by room actors to WebSocket handlers
//! - [`commands`] - Commands sent by WebSocket handlers to room actors
//! - [`sync`] - State synchronization types for reconnection
//!
//! ## Communication Patterns
//!
//! The system uses two distinct patterns:
//!
//! 1. **Event Bus (pub/sub)** - Events are broadcast to all subscribers.
//!    Multiple WebSocket handlers can subscribe to events for a room.
//!
//! 2. **Command Bus (request/reply)** - Commands are sent point-to-point
//!    and receive a single response from the room actor.

pub mod commands;
pub mod events;
pub mod sync;
pub mod types;

// Re-export for convenience
pub use commands::{Command, CommandResponse, CommandResult};
pub use events::Event;
pub use sync::RoomStateMirror;
pub use types::{RoomId, User, UserId};
