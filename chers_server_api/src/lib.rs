use std::fmt::Debug;

use chers::{Color, Move};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// All messages sent from the server to one or both players before the game
/// starts.
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum LobbyCommand {
    /// A player has defined a name for themselves.
    SetName {
        name: String,
    },

    Ready {},
}

/// All messages sent from the server to one or both players before the game
/// starts.
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum LobbyAnnouncement {
    /// Someone tried to join a match that does not exist.
    /// Ideally we could respond with a 404 to the initial websocket request,
    /// but there is no "nice" way of doing this using the browser WebSocket
    /// API, so we use this message instead.
    MatchDoesNotExist {},

    /// A new player joined.
    PlayerJoined {},

    /// A player has defined a name for themselves.
    PlayerIdentified {},

    Ready {},

    /// Both players agreed to start the match, switching the matches state to
    /// "progressing".
    MatchStarted {},

    Error {
        message: String,
    },
}

/// All messages sent from a player and accepted by the server while the match
/// is in progress.
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum ProgressingMatchCommand {
    /// A player wants to move a piece.
    AttemptMove { the_move: Move },

    /// A player offers the other a draw.
    OfferToDraw {},

    /// A player accepts the previously offered draw.
    AgreeToDraw {},

    /// A player resigns.
    Resign {},
}

/// All messages sent from the server to one or both players while the match is
/// in progress.
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum ProgressingMatchAnnouncement {
    /// Broadcasted when a legal move was made. Contains the updated board
    /// state, as well as any events that happened.
    Move { the_move: Move },

    /// Send to one of the players, after they attemted an illegal move.
    IllegalMove { the_move: Move },

    /// One players offers the other a draw. This is only sent to the
    /// other player.
    OfferToDraw {},

    /// One of the players resigned.
    Resignation { who: Color },

    /// The other player agreed to end the match in a draw.
    AgreementToDraw {},

    /// One of the players disconnected.
    PlayerDisconnected { who: Color },

    /// One of the players reconnected.
    PlayerReconnected { who: Color },

    /// Broadcasted when there is no activity for a certain amount of time, or
    /// if the match just keeps going for too long.
    MatchTimedOut {},
}
