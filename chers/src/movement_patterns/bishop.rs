use crate::{Board, Coordinate, Piece};

use super::expansion::expand_until_collides;

/// Computes the movement patterns of a [piece_color] [crate::Figure::Bishop]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, piece: Piece) -> Vec<Coordinate> {
    expand_until_collides(
        board,
        from,
        piece.color,
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
    use crate::{fen, fmt_coordinates, piece_at, Cell};

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
        let from = Cell::D4;
        let targets = moves(&state.board, from, piece_at(from, &state.board).unwrap());

        let expected = [
            // Top Left
            Cell::C5,
            Cell::B6,
            // Top Right
            Cell::E5,
            Cell::F6,
            Cell::G7,
            // Bottom Left
            Cell::C3,
            Cell::E3,
            Cell::F2,
            Cell::G1,
            // Bottom Right
            Cell::B2,
            Cell::A1,
        ];

        for cell in expected.iter() {
            assert!(
                targets.contains(cell),
                "The bishop should be able to move to {cell}, but it is missing in {:?}",
                fmt_coordinates(&targets)
            );
        }

        // Nothing else
        assert_eq!(expected.len(), targets.len());
    }
}
