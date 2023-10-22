use crate::{Board, Color, Coordinate};

use super::expansion::expand_until_collides;

/// Computes the movement patterns of a [piece_color] [crate::Figure::Rook]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, player: Color) -> Vec<Coordinate> {
    expand_until_collides(
        board,
        from,
        player,
        [
            (0, 1),  // Up
            (0, -1), // Down
            (1, 0),  // Left
            (-1, 0), // Right
        ],
    )
}
