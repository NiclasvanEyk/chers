use crate::{
    movement_patterns::bishop::moves as bishop_moves, movement_patterns::rook::moves as rook_moves,
    Board, Color, Coordinate,
};

/// Computes the movement patterns of a [piece_color] [crate::Figure::Queen]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, player: Color) -> Vec<Coordinate> {
    let mut moves = Vec::new();

    // A queen can move diagonally like a bishop and straight like a rook,
    // so we just re-use those functions.
    moves.append(&mut rook_moves(board, from, player));
    moves.append(&mut bishop_moves(board, from, player));

    // There might be some duplication, which we want to avoid
    // TODO: Not sure if necessary?
    moves.sort_by_key(|a| format!("{},{}", a.x, a.y));
    moves.dedup();

    moves
}
