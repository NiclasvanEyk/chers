//! Events broadcast by room actors.
//!
//! Events are published via the [`EventBus`](crate::bus::EventBus) trait
//! and delivered to all subscribers (WebSocket handlers). They represent
//! state changes that all connected clients should be aware of.
//!
//! Events are organized by phase:
//! - [`LobbyEvent`] - Events during the lobby phase (before game starts)
//! - [`GameEvent`] - Events during active gameplay
//! - [`PostGameEvent`] - Events after the game has ended
//! - [`SystemEvent`] - General system events not tied to a phase

use crate::v2::types::User;

/// Top-level event type, grouping events by phase.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Lobby(LobbyEvent),
    Game(GameEvent),
    PostGame(PostGameEvent),
    System(SystemEvent),
}

/// Events that are not tied to a specific phase.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SystemEvent {
    CommandRejected {
        user: User,
        reason: String,
    },

    /// A new connection has superseded an existing one for the same user.
    ///
    /// The recipient whose `connection_id` matches `old_connection_id`
    /// should close its WebSocket connection gracefully.
    ConnectionSuperseded {
        user: User,
        old_connection_id: String,
    },
}

/// All events published during the lobby phase.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum LobbyEvent {
    PlayerJoined { user: User },
    PlayerNameChanged { user: User },
    PlayerReadyChanged { user: User, is_ready: bool },
    PlayerLeft { user: User },
    JoinRejected { user: User, reason: String },
}

pub use chers::Move;

/// All events published while playing a game of chess.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum GameEvent {
    PlayerReconnected {
        user: User,
    },
    PlayerLeft {
        user: User,
    },
    /// The game has started with players assigned to colors.
    ///
    /// The `white` and `black` fields indicate which player controls each color.
    /// These assignments are random (50/50 chance for each player).
    GameStarted {
        white: User,
        black: User,
    },
    /// A move was made on the board.
    ///
    /// Contains the player who made the move and the move details.
    Turn {
        author: User,
        move_: Move,
    },
    /// The game has ended.
    ///
    /// The winner is `None` for draws.
    GameEnded {
        winner: Option<User>,
        reason: GameEndReason,
    },
    MoveRejected {
        author: User,
        reason: String,
    },
}

/// The reason a game ended.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GameEndReason {
    /// Standard checkmate - one player's king is in check and has no legal moves.
    Checkmate,
    /// Stalemate - player to move has no legal moves but their king is not in check.
    Stalemate,
    /// A player resigned.
    Resignation,
    /// Both players agreed to a draw.
    DrawAgreement,
    /// 50-move rule - 50 moves without pawn move or capture.
    FiftyMoveRule,
    /// Insufficient material to checkmate (e.g., king vs king).
    InsufficientMaterial,
    /// Threefold repetition - same position occurred 3 times.
    ThreefoldRepetition,
    /// Game ended due to timeout.
    Timeout,
    /// A player disconnected and didn't reconnect in time.
    Abandoned,
}

/// All events published after the winner has been decided.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PostGameEvent {
    PlayerReconnected { user: User },
    PlayerLeft { user: User },
    RematchOffered { by: User },
    RematchAccepted { by: User },
    RematchDeclined { by: User },
}
