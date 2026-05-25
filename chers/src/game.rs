use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::PromotedFigure;

use super::{CastlingRights, Color::White, Coordinate, Piece, State, INITIAL_BOARD};

#[derive(Tsify, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Move {
        piece: Piece,
        from: Coordinate,
        to: Coordinate,
    },
    Capture {
        at: Coordinate,
        captured: Piece,
        by: Piece,
    },
    Promotion {
        to: PromotedFigure,
    },
    Check {
        by: Vec<(Coordinate, Piece)>,
    },
    Mate,
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
