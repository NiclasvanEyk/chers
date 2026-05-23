//! Core types shared across the chess server protocol.
//!
//! These types are used in both events and commands.

/// A match-level unique identifier for users.
pub type UserId = String;

/// A unique identifier for a room.
pub type RoomId = String;

/// A user (player) in a chess match.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct User {
    /// A match-level unique identifier used by various components in the server.
    pub id: UserId,

    /// A human-readable name chosen by the user.
    pub name: String,

    /// Identifies the active WebSocket connection for this user.
    ///
    /// Set by the server when a player joins or reconnects. Commands are
    /// validated against this ID — if a command arrives with a different
    /// `connection_id`, it is rejected (the connection has been superseded).
    pub connection_id: String,
}
