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
            (0, -1), // Up
            (0, 1),  // Down
            (-1, 0), // Left
            (1, 0),  // Right
        ],
    )
}
#[cfg(test)]
mod tests {
    use crate::{fen, fmt_coordinates};

    use super::*;

    #[test]
    fn rooks_can_move_straight_across_4_axes() {
        // 8
        // 7                ♟
        // 6                ♟
        // 5
        // 4                ♖             ♙
        // 3
        // 2
        // 1
        // a    b     c     d             h
        let state = fen::parse_state("8/3p4/3p4/8/3R3P/8/8/8 w - - 0 1").unwrap();
        let from = Coordinate::algebraic("d4").unwrap();
        let targets = moves(&state.board, from, state.player);

        let expected = [
            "d5", "d6", // Top
            "e4", "f4", "g4", // Right
            "d3", "d2", "d1", // Bottom
            "c4", "b4", "a4", // Left
        ];

        for notation in expected.iter() {
            let coordinate = &Coordinate::algebraic(notation).unwrap();
            assert!(
                targets.contains(coordinate),
                "The rook should be able to move to {notation}, but it is missing in {:?}",
                fmt_coordinates(&targets)
            );
        }

        // Nothing else
        assert_eq!(expected.len(), targets.len());
    }
}
