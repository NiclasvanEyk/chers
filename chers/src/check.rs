use crate::{
    fmt_coordinates, move_piece, moves_available::autocomplete_to, pieces, Board, Coordinate,
    Figure, Move, Piece, Player, State,
};

/// Computes if any of the current player's pieces can capture their opponents king.
pub fn checks(state: &State) -> Vec<(Coordinate, Piece)> {
    let opponent = state.player.other();
    let king = find_king_of(&state.board, opponent);
    println!("{opponent:?}'s king on {king}");

    let mut checking_pieces = Vec::new();
    for (coordinate, piece) in pieces(&state.board) {
        if piece.color == opponent {
            continue;
        }

        let moves = autocomplete_to(state, coordinate);
        if piece.figure == Figure::Queen {
            println!(
                "queen at {coordinate} can move to {}",
                fmt_coordinates(&moves)
            );
        }

        for maybe_checking_move in moves {
            if maybe_checking_move == king {
                checking_pieces.push((coordinate, piece))
            }
        }
    }

    checking_pieces
}

/// Computes if the state represents checkmate.
pub fn mates(state: &State) -> bool {
    let opponent = state.player.other();
    let king = find_king_of(&state.board, opponent);

    // If all potential moves of the king still lead to a ceck, we have mate
    let escapes = autocomplete_to(state, king);
    if escapes.is_empty() {
        return true;
    }

    escapes.iter().any(|potential_escape| {
        match move_piece(
            state,
            Move {
                from: king,
                to: *potential_escape,
                promotion: None,
            },
        ) {
            Err(err) => panic!("{:?}", err),
            Ok((resulting_state, _)) => {
                let checking_moves = checks(&resulting_state);
                println!("Still checking: {checking_moves:?}",);
                checking_moves.is_empty()
            }
        }
    })
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

        assert!(checks(&initial_state).is_empty());
    }

    #[test]
    fn it_detects_mate() {
        // This is theoretically invalid state, but it allows us to test the
        // functionality in isoloation
        let notation = "rnbqkbnr/ppppp2p/8/5ppQ/5P2/4P3/PPPP2PP/RNB1KBNR w KQkq - 1 3";
        let parsed = parse_state(notation).unwrap();

        let checks = checks(&parsed);

        assert!(checks.contains(&(Coordinate::algebraic("h5").unwrap(), Piece::white(Queen))));
        assert!(mates(&parsed));
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

        assert!(mates(&new_state));
        assert!(events.contains(&crate::Event::Mate));
    }
}
