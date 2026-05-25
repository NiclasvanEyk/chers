use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::PromotedFigure;

use super::{
    moves_available::autocomplete_to, CastlingRights, Color::White, Coordinate, Move, Piece, State,
    INITIAL_BOARD,
};

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

pub fn available_moves(state: &State, from: Coordinate) -> Vec<Coordinate> {
    autocomplete_to(state, from)
}

/// Checks if a move is legal for the given state.
///
/// Returns `true` if the move is valid, `false` otherwise.
/// This can be used to pre-validate moves before attempting to execute them.
pub fn is_valid_move(state: &State, r#move: Move) -> bool {
    let legal_moves = available_moves(state, r#move.from);
    legal_moves.contains(&r#move.to)
}
