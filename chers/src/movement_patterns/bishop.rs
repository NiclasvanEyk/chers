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

#[cfg(test)]
mod tests {
    use crate::{fen, fmt_coordinates};

    use super::*;

    #[test]
    fn bishops_can_move_dioagnoal_across_4_axes() {
        // 8
        // 7    ♙                       ♟
        // 6
        // 5
        // 4                ♗
        // 3
        // 2
        // 1
        //      a   b   c   d   e   f   g
        let state = fen::parse_state("8/P5p1/8/8/3B4/8/8/8 w - - 0 1").unwrap();
        let from = Coordinate::algebraic("d4").unwrap();
        let targets = moves(&state.board, from, state.player);

        let expected = [
            "c5", "b6", // Top Left
            "e5", "f6", "g7", // Top Right
            "e3", "f2", "g1", // Bottom Left
            "c3", "b2", "a1", // Bottom Right
        ];

        for notation in expected.iter() {
            let coordinate = &Coordinate::algebraic(notation).unwrap();
            assert!(
                targets.contains(coordinate),
                "The bishop should be able to move to {notation}, but it is missing in {:?}",
                fmt_coordinates(&targets)
            );
        }

        // Nothing else
        assert_eq!(expected.len(), targets.len());
    }
}
