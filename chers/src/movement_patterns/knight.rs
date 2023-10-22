use crate::{can_be_moved_to_given, Board, Color, Coordinate};

/// Computes the movement patterns of a [piece_color] [crate::Figure::Knight]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, player: Color) -> Vec<Coordinate> {
    // Knights move in "L"-shapes. We simply pre-compute those here and check
    // every possible combination.
    let possible = [
        from.up(2).and_then(|m| m.left(1)),
        from.up(1).and_then(|m| m.left(2)),
        from.down(1).and_then(|m| m.left(2)),
        from.down(2).and_then(|m| m.left(1)),
        from.up(2).and_then(|m| m.right(1)),
        from.up(1).and_then(|m| m.right(2)),
        from.down(1).and_then(|m| m.right(2)),
        from.down(2).and_then(|m| m.right(1)),
    ];

    let mut moves = Vec::new();
    for cell in possible.into_iter().flatten() {
        if can_be_moved_to_given(cell, player, board) {
            moves.push(cell);
        }
    }

    moves
}
