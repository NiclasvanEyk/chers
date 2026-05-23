use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{
    move_execution::{move_piece, CantMovePiece},
    PromotedFigure,
};

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

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Game {}

impl Game {
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

    /// Checks if a move is legal for the given state.
    ///
    /// Returns `true` if the move is valid, `false` otherwise.
    /// This can be used to pre-validate moves before attempting to execute them.
    pub fn is_valid_move(&self, state: &State, r#move: Move) -> bool {
        let legal_moves = self.available_moves(state, r#move.from);
        legal_moves.contains(&r#move.to)
    }

    pub fn move_piece(
        &self,
        state: &State,
        r#move: Move,
    ) -> Result<(State, Vec<Event>), CantMovePiece> {
        move_piece(state, r#move)
    }
}
