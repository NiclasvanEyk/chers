use crate::{Board, Coordinate, Piece};

use super::expansion::expand_until_collides;

/// Computes the movement patterns of a [piece_color] [crate::Figure::Rook]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, piece: Piece) -> Vec<Coordinate> {
    expand_until_collides(
        board,
        from,
        piece.color,
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
    use crate::{fen, fmt_coordinates, piece_at, Cell};

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
        let from = Cell::D4;
        let targets = moves(&state.board, from, piece_at(from, &state.board).unwrap());

        let expected = [
            Cell::D5,
            Cell::D6, // Top
            Cell::E4,
            Cell::F4,
            Cell::G4, // Right
            Cell::D3,
            Cell::D2,
            Cell::D1, // Bottom
            Cell::C4,
            Cell::B4,
            Cell::A4, // Left
        ];

        for cell in expected.iter() {
            assert!(
                targets.contains(cell),
                "The rook should be able to move to {cell}, but it is missing in {:?}",
                fmt_coordinates(&targets)
            );
        }

        // Nothing else
        assert_eq!(expected.len(), targets.len());
    }
}
