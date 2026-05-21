//! Commands sent to room actors.
//!
//! Commands are sent point-to-point via the [`CommandBus`](crate::command::CommandBus)
//! trait. Unlike events, commands expect a response and only the room actor
//! processes them.
//!
//! Commands are organized by phase:
//! - [`LobbyCommand`] - Commands during the lobby phase
//! - [`GameCommand`] - Commands during active gameplay
//! - [`PostGameCommand`] - Commands after the game has ended

pub use chers::Move;

use crate::v2::{sync::RoomStateMirror, types::User};

/// Top-level command type, grouping commands by phase.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Lobby(LobbyCommand),
    Game(GameCommand),
    PostGame(PostGameCommand),

    /// Request the current room state.
    ///
    /// Universal command that works in any phase.
    /// Returns a personalized [`RoomStateMirror`] response.
    RequestState {
        user: User,
    },

    /// Leave the room from any phase.
    ///
    /// Universal command — routed to the current phase's handler.
    /// The `connection_id` is validated against the user's active connection.
    Leave {
        user: User,
        connection_id: String,
    },
}

/// Commands available during the lobby phase.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum LobbyCommand {
    Join {
        secret: String,
        name: String,
        /// The connection ID assigned by the server for this WebSocket.
        connection_id: String,
    },
    Leave {
        user: User,
        /// The connection ID to validate against the user's active connection.
        connection_id: String,
    },
    ChangeName { user: User, new_name: String },
    ChangeReady { user: User, is_ready: bool },
}

/// Commands available during active gameplay.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum GameCommand {
    Reconnect {
        secret: String,
        /// The connection ID assigned by the server for this WebSocket.
        connection_id: String,
    },
    Leave {
        user: User,
        /// The connection ID to validate against the user's active connection.
        connection_id: String,
    },
    /// Make a chess move.
    ///
    /// The move contains the from/to coordinates and optional promotion piece.
    MakeMove {
        user: User,
        move_: Move,
    },
    /// Resign from the current game.
    ///
    /// The opponent is declared the winner immediately.
    Resign {
        user: User,
    },
}

/// Commands available after the game has ended.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PostGameCommand {
    Reconnect {
        secret: String,
        /// The connection ID assigned by the server for this WebSocket.
        connection_id: String,
    },
    Leave {
        user: User,
        /// The connection ID to validate against the user's active connection.
        connection_id: String,
    },
    OfferRematch { user: User },
    AcceptRematch { user: User },
    DeclineRematch { user: User },
}

/// The outcome of a command sent to an actor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CommandResponse {
    Accepted,
    Rejected {
        reason: String,
    },
    /// Current room state, personalized for the requesting user.
    State(RoomStateMirror),
}

/// The combined output of handling a command.
///
/// A command handler returns both a direct response to the sender
/// and any events to broadcast to all subscribers.
#[derive(Clone, Debug)]
pub struct CommandResult {
    pub response: CommandResponse,
    pub events: Vec<crate::v2::events::Event>,
}

impl CommandResult {
    /// Create a result indicating the command was accepted with an event.
    pub fn accepted(event: crate::v2::events::Event) -> Self {
        Self {
            response: CommandResponse::Accepted,
            events: vec![event],
        }
    }

    /// Create a result indicating the command was rejected.
    pub fn rejected(reason: impl Into<String>) -> Self {
        Self {
            response: CommandResponse::Rejected {
                reason: reason.into(),
            },
            events: vec![],
        }
    }
}
