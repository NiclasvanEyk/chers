use crate::{Board, Color, Coordinate};

use super::expansion::expand_until_collides;

/// Computes the movement patterns of a [piece_color] [crate::Figure::Bishop]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, player: Color) -> Vec<Coordinate> {
    expand_until_collides(
        board,
        from,
        player,
        [
            (1, 1),   // North West
            (1, -1),  // North East
            (-1, 1),  // South West
            (-1, -1), // South East
        ],
    )
}
