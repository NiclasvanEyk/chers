use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::PromotedFigure;

use super::{CastlingRights, Color::White, Coordinate, Piece, State, INITIAL_BOARD};

#[derive(Tsify, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// This is always present in the events vector after a move.
    Move {
        piece: Piece,
        from: Coordinate,
        to: Coordinate,
    },

    /// The move captured a piece of the opponent.
    Capture {
        at: Coordinate,
        captured: Piece,
        by: Piece,
    },

    /// A pawn moved to the opposite end of the board and was promoted.
    ///
    /// NOTE: We could also place this as an Optional<PromotedFigure> onto
    /// the Move event, not sure what makes more sense here.
    Promotion { to: PromotedFigure },

    /// The move checks the opponents king.
    Check { by: Vec<(Coordinate, Piece)> },

    /// The move checks the opponents king in a way where it is unable to move out
    /// of the check. This ends the game.
    CheckMate,
}

pub fn initial_state() -> State {
    State {
        player: White,
        board: INITIAL_BOARD,
        castling_rights: CastlingRights::all(),
        en_passant_target: None,
        halfmove_clock: 0,
        fullmove_number: 1,
    }
}
