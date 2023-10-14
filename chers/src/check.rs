use crate::{
    move_piece, moves_available::possible_moves, pieces, Board, Coordinate, Figure, Move, Piece,
    Player, State,
};

/// Computes if any of the current player's pieces can capture their opponents king.
pub fn is_checked_by_opponent(state: &State) -> Vec<(Coordinate, Piece)> {
    let king = find_king_of(&state.board, state.player);
    let mut other = state.clone();
    other.player = state.opponent();

    let mut checking_pieces = Vec::new();
    for (coordinate, piece) in pieces(&state.board) {
        if state.player.owns(piece) {
            continue;
        }

        let moves = possible_moves(&other, coordinate);
        println!("{:?}", moves);
        for maybe_checking_move in moves {
            if maybe_checking_move == king {
                checking_pieces.push((coordinate, piece))
            }
        }
    }

    checking_pieces
}

/// Computes if the state represents checkmate.
///
/// Note that state is the new state after the resulting move of the opponent.
pub fn check_is_mate(state: &State) -> bool {
    // If all potential moves all pieces still lead to a check, we have mate
    for (from, piece) in pieces(&state.board) {
        if !state.player.owns(piece) {
            continue;
        }

        let escapes = possible_moves(state, from);
        if escapes.is_empty() {
            continue;
        }

        for to in escapes {
            match move_piece(
                state,
                Move {
                    from,
                    to,
                    promotion: None,
                },
            ) {
                Err(err) => panic!("{:?}", err),
                Ok((resulting_state, _)) => {
                    if is_checked_by_opponent(&resulting_state).is_empty() {
                        return false;
                    }
                }
            }
        }
    }

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
    use crate::{fen::parse_state, Engine, Figure::Queen};

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

        assert!(is_checked_by_opponent(&initial_state).is_empty());
    }

    #[test]
    fn it_detects_mate() {
        // This is theoretically invalid state, but it allows us to test the
        // functionality in isoloation
        let notation = "rnbqkbnr/ppppp2p/8/5ppQ/5P2/4P3/PPPP2PP/RNB1KBNR w KQkq - 1 3";
        let parsed = parse_state(notation).unwrap();

        let checks = is_checked_by_opponent(&parsed);
        println!("{:?}", checks);

        assert!(checks.contains(&(Coordinate::algebraic("h5").unwrap(), Piece::white(Queen))));
        assert!(check_is_mate(&parsed));
    }

    #[test]
    fn it_emits_mate_event() {
        let notation = "rnbqkbnr/ppppp2p/8/5pp1/4PP2/8/PPPP2PP/RNBQKBNR w KQkq - 0 1";
        let parsed = parse_state(notation).unwrap();

        let (new_state, events) = move_piece(
            &parsed,
            Move {
                from: Coordinate::algebraic("d1").unwrap(),
                to: Coordinate::algebraic("h5").unwrap(),
                promotion: None,
            },
        )
        .unwrap();

        assert!(check_is_mate(&new_state));
        assert!(events.contains(&crate::Event::Mate));
    }

    #[test]
    fn king_cant_move_if_result_still_checks() {
        let notation = "rnb1kbnr/pppp1ppp/8/4P3/7q/8/PPPPP1PP/RNBQKBNR w KQkq - 0 1";
        let state = parse_state(notation).unwrap();

        assert!(
            !is_checked_by_opponent(&state).is_empty(),
            "check was not even detected"
        );
    }
}
