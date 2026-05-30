//! State synchronization types.
//!
//! Provides a personalized mirror of the room state for clients.

use chers::{Color, Move, State};

use crate::v2::{events::GameEndReason, types::User};

/// Personalized room state - mirrors what's stored but tailored to the requesting user.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RoomStateMirror {
    /// Lobby phase - waiting for players to ready up
    Lobby {
        /// The requesting player
        you: User,
        /// The opponent (if joined)
        opponent: Option<User>,
        /// Whether you are ready
        you_are_ready: bool,
        /// Whether opponent is ready
        opponent_is_ready: bool,
    },

    /// Game phase - actively playing
    Game {
        /// The requesting player
        you: User,
        /// Your assigned color (White or Black)
        your_color: Color,
        /// The opponent
        opponent: User,
        /// Opponent's color
        opponent_color: Color,
        /// Current board state
        board_state: State,
        /// Whether it's your turn to move
        is_your_turn: bool,
        /// The sequence of moves played so far
        move_history: Vec<Move>,
    },

    /// PostGame phase - game has ended
    PostGame {
        /// The requesting player
        you: User,
        /// Your assigned color (White or Black)
        your_color: Color,
        /// The opponent
        opponent: User,
        /// Opponent's color
        opponent_color: Color,
        /// Winner's color (None for draw)
        winner: Option<Color>,
        /// Whether you won
        you_won: bool,
        /// Reason the game ended
        reason: GameEndReason,
        /// Final board state
        final_board: State,
        /// The complete sequence of moves from the game
        move_history: Vec<Move>,
    },
}
