use crate::{
    movement_patterns::bishop::moves as bishop_moves, movement_patterns::rook::moves as rook_moves,
    Board, Coordinate, Piece,
};

/// Computes the movement patterns of a [piece_color] [crate::Figure::Queen]
/// residing on [from], given that [player] owns and wants to move it.
pub fn moves(board: &Board, from: Coordinate, piece: Piece) -> Vec<Coordinate> {
    let mut moves = Vec::new();

    // A queen can move diagonally like a bishop and straight like a rook,
    // so we just re-use those functions.
    moves.append(&mut rook_moves(board, from, piece));
    moves.append(&mut bishop_moves(board, from, piece));

    // There might be some duplication, which we want to avoid
    // TODO: Not sure if necessary?
    moves.sort_by_key(|a| format!("{},{}", a.x, a.y));
    moves.dedup();

    moves
}

#[cfg(test)]
mod tests {
    use crate::{fen, fmt_coordinates, piece_at};

    use super::*;

    #[test]
    fn queen_can_move_to_almost_everything() {
        // 8
        // 7    ♙
        // 6
        // 5                    ♟
        // 4                ♕           ♟
        // 3
        // 2
        // 1                ♟
        //      a   b   c   d   e   f   g   h
        let state = fen::parse_state("8/P7/8/4p3/3Q2p1/8/8/3p4 w - - 0 1").unwrap();
        let from = Coordinate::algebraic("d4").unwrap();
        let targets = moves(&state.board, from, piece_at(from, &state.board).unwrap());

        let expected = [
            // Straight
            "d5", "d6", "d7", "d8", // The way up is free
            "c4", "b4", "a4", // So is the way left
            "d3", "d2", "d1", // Downwards can be moved to the end, capturing an enemy pawn
            "e4", "f4", "g4", // Same goes for the right
            // Diagonal
            "c5", "b6", // Top left is blocked by White's pawn
            "e5", //Top right immediately captures a pawn
            "e3", "f2", "g1", // Bottom right is free
            "c3", "b2", "a1", // So is bottom left is free
        ];

        for notation in expected.iter() {
            let coordinate = &Coordinate::algebraic(notation).unwrap();
            assert!(
                targets.contains(coordinate),
                "The queen should be able to move to {notation}, but it is missing in {:?}",
                fmt_coordinates(&targets)
            );
        }

        // Nothing else
        assert_eq!(expected.len(), targets.len());
    }

    #[test]
    fn queen_regression_test_1() {
        let state =
            fen::parse_state("rnbqkbnr/ppp2ppp/3p4/4p3/3P4/3Q4/PPP1PPPP/RNB1KBNR w KQkq - 0 1")
                .unwrap();
        let from = Coordinate::algebraic("d3").unwrap();
        let targets = moves(&state.board, from, piece_at(from, &state.board).unwrap());

        let coordinate = &Coordinate::algebraic("b5").unwrap();
        assert!(
            targets.contains(coordinate),
            "The queen should be able to move to b5, but it is missing in {:?}",
            fmt_coordinates(&targets)
        );
    }
}
