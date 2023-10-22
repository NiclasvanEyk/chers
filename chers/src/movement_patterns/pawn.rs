use crate::{is_free, piece_at, Board, Color, Coordinate};

/// Computes the movement patterns of a [piece_color] [crate::Figure::Pawn]
/// residing on [from], given that [player] owns and wants to move it.
///
/// If an [en_passant_target] exists, it will be properly handled, but more
/// complex invariants, such as not being able to move due to a resulting check
/// (also known as pinning) are out of scope.
pub fn moves(
    board: &Board,
    from: Coordinate,
    player: Color,
    en_passant_target: Option<Coordinate>,
) -> Vec<Coordinate> {
    let mut moves = Vec::new();
    let piece_color = player;

    // The most common move for a pawn is forward. We can safely assume that
    // the next cell exists on the board, since pawns get promoted to another
    // piece, once they reach the end of the board.
    let single_step = from.forward(piece_color, 1).unwrap();
    let single_step_is_free = is_free(single_step, board);
    if single_step_is_free {
        moves.push(single_step)
    }

    // If a pawn has not been moved yet it can actually move ahead _two_ cells,
    // given no pieces stand in the way. Again we can safely assume that the
    // cell is still on the board, since unmoved pawns are far away from the
    // other end of the board.
    if single_step_is_free && has_not_been_moved(from, piece_color) {
        let double_step = from.forward(piece_color, 2).unwrap();
        if is_free(double_step, board) {
            moves.push(double_step);
        }
    }

    // Pawns can also capture diagonally, given there is a piece to capture
    // and the cell is still on the board.
    let diagonals = capture_moves(single_step);
    for capture_move in diagonals.iter() {
        if let Some(piece) = piece_at(*capture_move, board) {
            if piece.color != player {
                moves.push(*capture_move)
            }
        }
    }

    // We will also handle the special case, where a pawn can capture another
    // one "in passing" (french "en passant"), given that the other one has
    // previously moved two cells at once. In this case, en_passant_target
    // contains the cell to which our pawn can be moved to, to capture the
    // passing piece.
    let Some(target) = en_passant_target else {
        return moves;
    };

    for capture_move in diagonals {
        if target == capture_move {
            moves.push(capture_move)
        }
    }

    moves
}

fn has_not_been_moved(from: Coordinate, color: Color) -> bool {
    match color {
        Color::White => from.y == 6,
        Color::Black => from.y == 1,
    }
}

fn capture_moves(forward: Coordinate) -> Vec<Coordinate> {
    let mut capture_moves = Vec::new();

    if let Some(m) = forward.left(1) {
        capture_moves.push(m);
    }

    if let Some(m) = forward.right(1) {
        capture_moves.push(m);
    }

    capture_moves
}

#[cfg(test)]
mod tests {
    use crate::{fen, Engine};

    use super::*;

    #[test]
    fn pawns_can_move_forward_once_and_twice_at_the_beginning() {
        let state = Engine {}.start();
        let from = Coordinate::algebraic("a2").unwrap();
        let targets = moves(&state.board, from, state.player, state.en_passant_target);

        println!("{:?}", targets);
        assert!(targets.contains(&Coordinate::algebraic("a3").unwrap()));
        assert!(targets.contains(&Coordinate::algebraic("a4").unwrap()));
    }

    #[test]
    fn pawns_cant_move_forward_if_somethings_in_the_way() {
        // 4    ♟
        // 3
        // 2    ♙
        // 1
        //      a
        let state = fen::parse_state("8/8/8/8/p7/8/P7/8 w - - 0 1").unwrap();
        let from = Coordinate::algebraic("a2").unwrap();
        let targets = moves(&state.board, from, state.player, state.en_passant_target);

        println!("{:?}", targets);
        assert_eq!(1, targets.len());
        assert!(targets.contains(&Coordinate::algebraic("a3").unwrap()));
        assert!(!targets.contains(&Coordinate::algebraic("a4").unwrap()));

        // 4
        // 3    ♟
        // 2    ♙
        // 1
        //      a
        let state = fen::parse_state("8/8/8/8/8/p7/P7/8 w - - 0 1").unwrap();
        let from = Coordinate::algebraic("a2").unwrap();
        let targets = moves(&state.board, from, state.player, state.en_passant_target);

        println!("{:?}", targets);
        assert!(!targets.contains(&Coordinate::algebraic("a3").unwrap()));
        assert!(!targets.contains(&Coordinate::algebraic("a4").unwrap()));
    }

    #[test]
    fn pawns_can_capture_diagonally() {
        // 7    ♟                   7
        // 6                        6
        // 5         ♙       -->    5    ♟    ♙
        //
        //      a    b                   a    b
        let state = fen::parse_state("7k/8/8/pP6/8/8/8/7K w - a6 0 2").unwrap();
        let from = Coordinate::algebraic("b5").unwrap();
        let targets = moves(&state.board, from, state.player, state.en_passant_target);

        println!("{:?}", targets);
        assert!(targets.contains(&Coordinate::algebraic("a3").unwrap()));
        assert!(targets.contains(&Coordinate::algebraic("c3").unwrap()));
    }
}
