//! Message definitions for the WebSocket multiplayer chess protocol.
//!
//! These messages are used for communication between clients and the server
//! during multiplayer chess games. The module is organized into:
//! - [`client`] - Messages sent from clients to the server
//! - [`server`] - Messages sent from the server to clients
//!
//! Events are categorized by visibility:
//! - **Public events**: Visible to all players and spectators
//! - **Private events**: Sent only to specific players
//!
//! ## Type Annotations
//!
//! This crate uses `ts-rs` for TypeScript generation. Some types from the `chers`
//! crate (like `Coordinate`, `Color`, `Game`) are defined using `tsify` for WASM
//! bindings and don't implement the `TS` trait from `ts-rs`.
//!
//! To handle this, we use the `#[ts(type = "TypeName")]` attribute to tell
//! ts-rs to reference these external types by name in the generated TypeScript:
//!
//! ```rust,ignore
//! #[ts(type = "Coordinate")]
//! from: Coordinate,
//! ```
//!
//! This ensures the generated TypeScript references `Coordinate` (which is exported
//! by the chers WASM package) rather than trying to inline the type definition.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// Re-export types from chers for convenience
pub use chers::{Color, Coordinate, Game, PromotedFigure};

/// The piece a pawn can be promoted to.
///
/// This enum uses single-letter notation (standard in chess) that maps to TypeScript union type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PromotionPiece {
    /// Queen
    #[serde(rename = "Q")]
    Q,
    /// Rook
    #[serde(rename = "R")]
    R,
    /// Bishop  
    #[serde(rename = "B")]
    B,
    /// Knight
    #[serde(rename = "N")]
    N,
}

impl From<PromotionPiece> for PromotedFigure {
    fn from(piece: PromotionPiece) -> Self {
        match piece {
            PromotionPiece::Q => PromotedFigure::Queen,
            PromotionPiece::R => PromotedFigure::Rook,
            PromotionPiece::B => PromotedFigure::Bishop,
            PromotionPiece::N => PromotedFigure::Knight,
        }
    }
}

impl From<&PromotionPiece> for PromotedFigure {
    fn from(piece: &PromotionPiece) -> Self {
        (*piece).into()
    }
}

/// Messages sent from clients to the server.
///
/// These represent player actions and requests during a game session.
pub mod client {
    use super::*;

    /// All possible messages a client can send to the server.
    #[derive(Debug, Serialize, Deserialize, TS)]
    #[serde(tag = "kind")]
    #[ts(export)]
    pub enum ClientMessage {
        /// Authenticate with a token to claim a player slot.
        ///
        /// Sent immediately after WebSocket connection is established.
        /// The server responds with either:
        /// - `PrivateEvent::Authenticated` with the assigned player and token
        /// - `PrivateEvent::AuthenticationFailed` if the token is invalid
        Authenticate {
            /// Secret token that identifies this player.
            token: String,
        },

        /// Make a chess move using structured coordinates.
        ///
        /// The server validates that:
        /// 1. It's the requesting player's turn
        /// 2. The move is legal according to chess rules
        /// 3. If promotion is specified, it's valid for that position
        ///
        /// On success, a `PublicEvent::MoveMade` is broadcast to all.
        /// On failure, a `PrivateEvent::MoveRejected` is sent to the requesting player.
        MakeMove {
            /// The starting coordinate of the piece being moved.
            #[ts(type = "Coordinate")]
            from: Coordinate,
            /// The destination coordinate.
            #[ts(type = "Coordinate")]
            to: Coordinate,
            /// The piece to promote to, if this is a pawn promotion move.
            /// Valid values: "Q" | "R" | "B" | "N"
            promotion: Option<PromotionPiece>,
        },

        /// Request the current game state.
        ///
        /// Used by spectators joining mid-game or players reconnecting
        /// to synchronize their local state with the server.
        ///
        /// The server responds with `PrivateEvent::StateSync`.
        RequestSync,

        /// Offer a draw to the opponent.
        ///
        /// The server forwards this to the opponent as `PrivateEvent::DrawOffered`.
        OfferDraw,

        /// Accept a draw offer from the opponent.
        ///
        /// Ends the game with a draw result.
        AcceptDraw,

        /// Decline a draw offer from the opponent.
        ///
        /// The opponent is notified via `PrivateEvent::DrawDeclined`.
        DeclineDraw,

        /// Resign from the current game.
        ///
        /// Ends the game with the opponent as the winner.
        Resign,

        /// Send a heartbeat ping to keep the connection alive.
        ///
        /// Should be sent every 30 seconds. The server responds with
        /// `PrivateEvent::HeartbeatAck` and updates the last seen timestamp.
        Heartbeat,
    }
}

/// Messages sent from the server to clients.
///
/// These are organized into public events (visible to everyone including spectators)
/// and private events (visible only to specific players).
pub mod server {
    use super::*;

    /// Events visible to all players and spectators.
    ///
    /// These events are broadcast to everyone watching the game.
    /// Spectators only receive public events.
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[serde(tag = "kind")]
    #[ts(export)]
    pub enum PublicEvent {
        /// The game has started with both players connected.
        ///
        /// Contains the initial game state and player information.
        /// Sent when the second player joins and the game begins.
        GameStarted {
            /// The initial game state (starting position).
            #[ts(type = "Game")]
            game_state: Game,
            /// Information about the white player.
            white_player: PlayerInfo,
            /// Information about the black player.
            black_player: PlayerInfo,
        },

        /// A move was made on the board.
        ///
        /// Broadcast after a successful move, containing the move details,
        /// new state, and check/checkmate status.
        MoveMade {
            /// The player who made the move.
            #[ts(type = "Color")]
            author: Color,
            /// The starting coordinate of the piece that was moved.
            #[ts(type = "Coordinate")]
            from: Coordinate,
            /// The destination coordinate.
            #[ts(type = "Coordinate")]
            to: Coordinate,
            /// The piece the pawn was promoted to, if applicable.
            promotion: Option<PromotionPiece>,
            /// The new game state after the move.
            #[ts(type = "Game")]
            new_state: Game,
            /// Whether the move puts the opponent in check.
            is_check: bool,
            /// Whether the move results in checkmate.
            is_checkmate: bool,
        },

        /// The game has ended.
        ///
        /// Sent when the game concludes due to checkmate, resignation,
        /// draw agreement, timeout, or other termination conditions.
        GameOver {
            /// The final result of the game.
            result: GameResult,
            /// The reason the game ended (machine-readable, frontend should provide human-readable message).
            reason: GameEndReason,
        },

        /// A player's connection status changed.
        ///
        /// Sent when a player connects, disconnects, or reconnects.
        /// This allows the UI to show "Waiting for opponent" or similar states.
        PlayerStatusChanged {
            /// Which player's status changed.
            #[ts(type = "Color")]
            player: Color,
            /// The new connection status.
            status: PlayerConnectionStatus,
        },

        /// A player offered a draw.
        ///
        /// Broadcast so spectators can see that a draw was offered
        /// (though only the opponent can accept).
        DrawOffered {
            #[ts(type = "Color")]
            player: Color,
        },

        /// A player declined a draw offer.
        DrawDeclined {
            #[ts(type = "Color")]
            player: Color,
        },
    }

    /// Events sent only to specific players.
    ///
    /// These contain sensitive information or player-specific
    /// feedback that spectators should not see.
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[serde(tag = "kind")]
    #[ts(export)]
    pub enum PrivateEvent {
        /// Authentication was successful.
        ///
        /// Sent after a client sends `ClientMessage::Authenticate`.
        /// Contains the player assignment and token for reconnection.
        Authenticated {
            /// Whether this client is playing as White or Black.
            #[ts(type = "Color")]
            player: Color,
        },

        /// Authentication failed.
        ///
        /// Sent when the provided token is invalid or the match is full.
        AuthenticationFailed {
            /// The reason authentication was rejected.
            reason: AuthFailureReason,
        },

        /// A move was rejected.
        ///
        /// Sent when a move is invalid or it's not the player's turn.
        /// The move was not applied to the game state.
        MoveRejected {
            /// The reason the move was rejected.
            reason: MoveRejectionReason,
        },

        /// Heartbeat acknowledgment.
        ///
        /// Sent in response to `ClientMessage::Heartbeat` to confirm
        /// the connection is still alive.
        HeartbeatAck,

        /// Full state synchronization.
        ///
        /// Sent in response to `ClientMessage::RequestSync` or when
        /// a player reconnects. Contains everything needed to restore
        /// the client's view of the game.
        StateSync {
            /// The current game/board state.
            #[ts(type = "Game")]
            game_state: Game,
            /// Which player's turn it is.
            #[ts(type = "Color")]
            current_turn: Color,
            /// Information about the white player.
            white_player: PlayerInfo,
            /// Information about the black player.
            black_player: PlayerInfo,
            /// Which player offered a draw, if any (pending draw offer).
            #[ts(type = "Color | null")]
            draw_offered_by: Option<Color>,
            /// Number of moves made so far.
            move_count: u32,
        },

        /// Your opponent offered a draw.
        ///
        /// Sent when the opponent sends `ClientMessage::OfferDraw`.
        /// The recipient can accept or decline.
        DrawOffered,

        /// Your opponent declined your draw offer.
        ///
        /// Sent when the opponent sends `ClientMessage::DeclineDraw`.
        DrawDeclined,
    }

    /// A server message containing either a public or private event.
    ///
    /// This is the top-level message type sent from server to clients.
    /// Players receive both public and private events, while spectators
    /// only receive public events.
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[serde(tag = "visibility", content = "event")]
    #[ts(export)]
    pub enum ServerMessage {
        /// An event visible to all (players and spectators).
        Public(PublicEvent),
        /// An event visible only to a specific player.
        Private(PrivateEvent),
    }

    /// Information about a player.
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub struct PlayerInfo {
        /// The player's display name.
        pub name: String,
        /// Whether this player is currently connected.
        pub connected: bool,
    }

    /// The result of a completed game.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub enum GameResult {
        /// White won the game.
        WhiteWins,
        /// Black won the game.
        BlackWins,
        /// The game ended in a draw.
        Draw,
    }

    /// The reason a game ended.
    ///
    /// These are machine-readable codes that the frontend should translate
    /// into human-readable messages (allowing for i18n and customization).
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub enum GameEndReason {
        /// Standard checkmate - one player's king is in check and has no legal moves.
        #[serde(rename = "checkmate")]
        Checkmate,
        /// Stalemate - player to move has no legal moves but their king is not in check.
        #[serde(rename = "stalemate")]
        Stalemate,
        /// A player resigned.
        #[serde(rename = "resignation")]
        Resignation,
        /// Both players agreed to a draw.
        #[serde(rename = "draw_agreement")]
        DrawAgreement,
        /// Threefold repetition - same position occurred 3 times.
        #[serde(rename = "threefold_repetition")]
        ThreefoldRepetition,
        /// 50-move rule - 50 moves without pawn move or capture.
        #[serde(rename = "fifty_move_rule")]
        FiftyMoveRule,
        /// Insufficient material to checkmate (e.g., king vs king).
        #[serde(rename = "insufficient_material")]
        InsufficientMaterial,
        /// Game ended due to timeout.
        #[serde(rename = "timeout")]
        Timeout,
        /// A player disconnected and didn't reconnect in time.
        #[serde(rename = "abandoned")]
        Abandoned,
    }

    /// A player's connection status.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub enum PlayerConnectionStatus {
        /// Player is currently connected and active.
        Connected,
        /// Player disconnected but can reconnect within the timeout period.
        Disconnected,
        /// Player is reconnecting (connection in progress).
        Reconnecting,
    }

    /// Reasons why authentication might fail.
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub enum AuthFailureReason {
        /// The provided token is invalid or expired.
        InvalidToken,
        /// The match already has two players and no vacant slots.
        MatchFull,
        /// The specified match does not exist.
        MatchNotFound,
    }

    /// Reasons why a move might be rejected.
    #[derive(Debug, Clone, Serialize, Deserialize, TS)]
    #[ts(export)]
    pub enum MoveRejectionReason {
        /// The move notation could not be parsed.
        InvalidNotation,
        /// The move is not legal for the current board state.
        IllegalMove,
        /// It's not this player's turn to move.
        NotYourTurn,
        /// The game is already over.
        GameOver,
    }
}

// Re-export the main types for convenience
pub use client::ClientMessage;
pub use server::{PrivateEvent, PublicEvent, ServerMessage};
