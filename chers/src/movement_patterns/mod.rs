use crate::{Coordinate, Figure, Piece, State};

mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rook;

mod expansion;

/// Computes a list of possible moves for the piece, given the current [Player]
/// owns and wants to move it.
pub fn of(state: &State, from: Coordinate, piece: Piece) -> Vec<Coordinate> {
    match piece.figure {
        Figure::Pawn => pawn::moves(&state.board, from, state.player, state.en_passant_target),
        Figure::King => king::moves(&state.board, from, state.player),
        Figure::Rook => rook::moves(&state.board, from, state.player),
        Figure::Bishop => bishop::moves(&state.board, from, state.player),
        Figure::Queen => queen::moves(&state.board, from, state.player),
        Figure::Knight => knight::moves(&state.board, from, state.player),
    }
}
