use crate::{check::is_checked_by_opponent, force_move_piece, movement_patterns, piece_at, Move};

use super::{Coordinate, State};

/// Returns all *legal* moves.
pub fn autocomplete_to(state: &State, from: Coordinate) -> Vec<Coordinate> {
    possible_moves(state, from)
}

/// Returns all possible moves, also including ones that are not legal, e.g.
/// because they would lead the current player to check themselves.
pub fn possible_moves(state: &State, from: Coordinate) -> Vec<Coordinate> {
    let Some(piece) = piece_at(from, &state.board) else {
        return Vec::new();
    };

    if state.player != piece.color {
        return Vec::new();
    }

    movement_patterns::of(state, from, piece)
}

/// Returns all moves without the ones allowing the opponent to directly take
/// their king the next turn.
fn without_checks(state: &State, from: Coordinate, targets: Vec<Coordinate>) -> Vec<Coordinate> {
    let mut valid_targets = Vec::new();

    for target in targets {
        let the_move = Move {
            from,
            to: target,
            promotion: None,
        };

        let Ok((resulting_state, _)) = force_move_piece(state, the_move) else {
            continue;
        };

        // only allow moves, that do not lead into a self-check
        if !is_checked_by_opponent(&resulting_state).is_empty() {
            continue;
        }

        valid_targets.push(target)
    }

    valid_targets
}

#[cfg(test)]
mod tests {
    use crate::{fen::parse_state, fmt_coordinates};

    use super::*;

    #[test]
    fn queen_movement() {
        let notation = "rnbqkbnr/pppp3p/5pp1/4P3/3Q4/8/PPP1PPPP/RNB1KBNR w KQkq - 0 4";
        let state = parse_state(notation).unwrap();

        let moves = autocomplete_to(&state, Coordinate::algebraic("d4").unwrap());

        for expected in [
            // Up/Down
            Coordinate::algebraic("d1").unwrap(),
            Coordinate::algebraic("d2").unwrap(),
            Coordinate::algebraic("d3").unwrap(),
            Coordinate::algebraic("d5").unwrap(),
            Coordinate::algebraic("d6").unwrap(),
            Coordinate::algebraic("d7").unwrap(), // This one would even capture the pawn!
            // Left/Right
            Coordinate::algebraic("a4").unwrap(),
            Coordinate::algebraic("b4").unwrap(),
            Coordinate::algebraic("c4").unwrap(),
            Coordinate::algebraic("e4").unwrap(),
            Coordinate::algebraic("f4").unwrap(),
            Coordinate::algebraic("g4").unwrap(),
            Coordinate::algebraic("h4").unwrap(),
            // Upleft/Downright
            Coordinate::algebraic("a7").unwrap(), // This one would even capture the pawn!
            Coordinate::algebraic("b6").unwrap(),
            Coordinate::algebraic("c5").unwrap(),
            Coordinate::algebraic("e3").unwrap(),
            // Upright/Downleft
            Coordinate::algebraic("c3").unwrap(),
        ] {
            assert!(
                moves.contains(&expected),
                "{} not in {}",
                expected,
                fmt_coordinates(&moves)
            );
        }

        assert_eq!(18, moves.len());
    }

    #[test]
    fn king_cant_move_if_result_still_checks() {
        let notation = "rnb1kbnr/pppp1ppp/8/4P3/7q/8/PPPPP1PP/RNBQKBNR w KQkq - 0 1";
        let state = parse_state(notation).unwrap();
        let available_moves = autocomplete_to(&state, Coordinate::algebraic("e1").unwrap());

        assert!(
            !is_checked_by_opponent(&state).is_empty(),
            "check was not even detected"
        );
        assert!(
            available_moves.is_empty(),
            "king can still move, even though it should not be able to"
        );
    }
}
