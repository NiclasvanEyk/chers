use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    move_execution::{move_piece, CantMovePiece},
    PromotedFigure,
};

use super::{
    moves_available::autocomplete_to, CastlingRights, Color::White, Coordinate, Move, Piece, State,
    INITIAL_BOARD,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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

#[derive(Default)]
pub struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> State {
        State {
            player: White,
            board: INITIAL_BOARD,
            castling_rights: CastlingRights::all(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn available_moves(&self, state: &State, from: Coordinate) -> Vec<Coordinate> {
        autocomplete_to(state, from)
    }

    pub fn move_piece(
        &self,
        state: &State,
        r#move: Move,
    ) -> Result<(State, Vec<Event>), CantMovePiece> {
        move_piece(state, r#move)
    }
}
