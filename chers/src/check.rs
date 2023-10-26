use crate::{
    force_move_piece, moves_available::possible_moves, pieces, Board, Coordinate, Figure, Move,
    Piece, Player, State,
};

/// Computes if any of the current player's pieces can capture their opponents king.
pub fn checking_pieces_of_opponent(state: &State) -> Vec<(Coordinate, Piece)> {
    let king = find_king_of(&state.board, state.player);
    let opponents_turn = state.reversed();

    let mut checking_pieces = Vec::new();
    for (coordinate, piece) in pieces(&state.board) {
        if state.player.owns(piece) {
            continue;
        }

        let moves = possible_moves(&opponents_turn, coordinate);
        for target in moves {
            if target == king {
                checking_pieces.push((coordinate, piece))
            }
        }
    }

    checking_pieces
}

/// Computes if the state represents checkmate.
///
/// If all potential moves all pieces still lead to a check, we have mate.
pub fn check_by_opponent_is_mate(state: &State) -> bool {
    for (from, piece) in pieces(&state.board) {
        if !state.player.owns(piece) {
            continue;
        }

        println!(
            "Checking moves of {piece}, since {:?} owns it",
            state.player
        );
        for to in possible_moves(state, from) {
            let (resulting_state, _) = force_move_piece(state, Move::simple(from, to)).unwrap();
            let checking = checking_pieces_of_opponent(&resulting_state.reversed());
            if checking.is_empty() {
                println!(
                    "[info] The {} could move from {} to {} in order to escape the check",
                    piece, from, to
                );
                return false;
            } else {
                println!(
                    "Moving the {} from {} to {} still results in the king being checked",
                    piece, from, to
                );
            }
        }
    }

    println!("We are checking");
    true
}

/// Finds the king of the given [player].
fn find_king_of(board: &Board, player: Player) -> Coordinate {
    for (coordinate, piece) in pieces(board) {
        if piece.color == player && piece.figure == Figure::King {
            return coordinate;
        }
    }

    panic!("Board does not contain a king!");
}

#[cfg(test)]
mod tests {
    use crate::{fen::parse_state, Engine};

    use super::*;

    #[test]
    fn it_finds_the_king() {
        let initial_state = Engine::new().start();

        assert_eq!(
            Coordinate::algebraic("e1").unwrap(),
            find_king_of(&initial_state.board, Player::White),
        );
        assert_eq!(
            Coordinate::algebraic("e8").unwrap(),
            find_king_of(&initial_state.board, Player::Black),
        );
    }

    #[test]
    fn at_the_beginning_no_mate_exist() {
        // This is theoretically invalid state, but it allows us to test the
        // functionality in isoloation
        let initial_state = Engine::new().start();

        assert!(checking_pieces_of_opponent(&initial_state).is_empty());
    }

    #[test]
    fn it_detects_simple_rook_check() {
        // R   K
        //
        // k
        let notation = "8/8/8/8/8/r1k5/8/K7 w - - 0 1";
        let state = parse_state(notation).unwrap();

        assert!(is_checked_by_opponent(&state), "check was not detected");
    }

    #[test]
    fn it_detects_simple_rook_mate() {
        // R R K
        //
        // k
        // a
        let notation = "8/8/8/8/8/rrk5/8/K7 w - - 0 1";
        let state = parse_state(notation).unwrap();

        assert!(check_by_opponent_is_mate(&state), "mate was not detected");
    }

    #[test]
    fn king_cant_move_if_result_still_checks() {
        let notation = "rnb1kbnr/pppp1ppp/8/4P3/7q/8/PPPPP1PP/RNBQKBNR w KQkq - 0 1";
        let state = parse_state(notation).unwrap();

        assert!(
            !checking_pieces_of_opponent(&state).is_empty(),
            "check was not even detected"
        );
    }
}
