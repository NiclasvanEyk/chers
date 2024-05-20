use crate::{
    check::checking_pieces_of_opponent, force_move_piece, movement_patterns, piece_at, Move,
    Player, PromotedFigure,
};

use super::{Coordinate, State};

/// Returns all *legal* moves.
pub fn autocomplete_to(state: &State, from: Coordinate) -> Vec<Coordinate> {
    let possible = dbg!(possible_moves(state, from));
    let without_chk = dbg!(without_checks(state, from, possible));

    without_chk
}

/// Returns all possible moves, also including ones that are not legal, e.g.
/// because they would lead the current player to check themselves.
pub fn possible_moves(state: &State, from: Coordinate) -> Vec<Coordinate> {
    possible_moves_by(state, state.player, from)
}

pub fn possible_moves_by(state: &State, player: Player, from: Coordinate) -> Vec<Coordinate> {
    let Some(piece) = piece_at(from, &state.board) else {
        return Vec::new();
    };

    if player != piece.color {
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

        let could_lead_to_check = match force_move_piece(state, the_move) {
            Ok((resulting_state, _)) => would_check_opponent(resulting_state),
            Err(err) => match err {
                crate::CantMovePiece::RequiresPromotion => {
                    would_check_opponent_after_promotion(state, the_move)
                }
                // Uninteresting cases we want to skip. Most of these should
                // not happen anyways and we could (or even should) even panic.
                // E.g. when we have no piece to move, something is definitely
                // wrong here!
                crate::CantMovePiece::NoPieceToMove => false,
                crate::CantMovePiece::ItBelongsToOtherPlayer => false,
                crate::CantMovePiece::IllegalMove { .. } => false,
            },
        };

        if could_lead_to_check {
            continue;
        }

        valid_targets.push(target);
    }

    valid_targets
}

fn would_check_opponent(state: State) -> bool {
    let next_turn_state = state.reversed();
    let has_checking_pieces = checking_pieces_of_opponent(&next_turn_state).is_empty();
    !has_checking_pieces
}

fn would_check_opponent_after_promotion(previous_state: &State, the_move: Move) -> bool {
    // While we could check all cases, promoting to queen does also check for
    // all moves of a bishop and rook, so we skip those. So queen and knight
    // should be sufficient if there are no edge cases that I overlook here.
    for figure in [PromotedFigure::Queen, PromotedFigure::Knight] {
        let the_actual_move = the_move.with_promotion_to(figure);
        let Ok((resulting_state, _)) = force_move_piece(previous_state, the_actual_move) else {
            continue;
        };

        if would_check_opponent(resulting_state) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::{fen::parse_state, fmt_coordinates, Cell};

    use super::*;

    #[test]
    fn queen_movement() {
        let notation = "rnbqkbnr/pppp3p/5pp1/4P3/3Q4/8/PPP1PPPP/RNB1KBNR w KQkq - 0 4";
        let state = parse_state(notation).unwrap();

        let moves = autocomplete_to(&state, Cell::D4);

        for expected in [
            // Up/Down
            Cell::D1,
            Cell::D2,
            Cell::D3,
            Cell::D5,
            Cell::D6,
            Cell::D7, // This one would even capture the pawn!
            // Left/Right
            Cell::A4,
            Cell::B4,
            Cell::C4,
            Cell::E4,
            Cell::F4,
            Cell::G4,
            Cell::H4,
            // Upleft/Downright
            Cell::A7, // This one would even capture the pawn!
            Cell::B6,
            Cell::C5,
            Cell::E3,
            // Upright/Downleft
            Cell::C3,
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
    fn pawn_can_promote() {
        let notation = "rnbqkbnr/pP1ppppp/8/8/8/8/1pPPPPPP/RNBQKBNR w KQkq - 0 5";
        let state = parse_state(notation).unwrap();

        let moves = autocomplete_to(&state, Cell::B7);

        for expected in [Cell::A8, Cell::C8] {
            assert!(
                moves.contains(&expected),
                "{} not in {}",
                expected,
                fmt_coordinates(&moves)
            );
        }

        assert_eq!(2, moves.len());
    }

    // #[test]
    // fn king_cant_move_if_result_still_checks() {
    //     let notation = "rnb1kbnr/pppp1ppp/8/4P3/7q/8/PPPPP1PP/RNBQKBNR w KQkq - 0 1";
    //     let state = parse_state(notation).unwrap();
    //     let available_moves = autocomplete_to(&state, Cell::e1);
    //
    //     assert!(
    //         !is_checked_by_opponent(&state).is_empty(),
    //         "check was not even detected"
    //     );
    //     assert!(
    //         available_moves.is_empty(),
    //         "king can still move, even though it should not be able to"
    //     );
    // }
}
