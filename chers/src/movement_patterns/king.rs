use crate::{can_be_moved_to_given, Board, Color, Coordinate};

/// Computes the movement patterns of a [piece_color] [crate::Figure::King]
/// residing on [from], given that [player] owns and wants to move it.
///
/// TODO: Castling will also be implemented here.
pub fn moves(board: &Board, from: Coordinate, player: Color) -> Vec<Coordinate> {
    let potential_moves = [
        from.up(1),
        from.right(1),
        from.down(1),
        from.left(1),
        from.diagonal(1, 1),
        from.diagonal(-1, 1),
        from.diagonal(1, -1),
        from.diagonal(-1, -1),
    ];

    let mut moves = Vec::new();
    for potential_move in potential_moves.into_iter().flatten() {
        if can_be_moved_to_given(potential_move, player, board) {
            moves.push(potential_move);
        }
    }

    moves
}
