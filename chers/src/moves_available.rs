use super::{Color, Coordinate, State};

pub fn autocomplete_to(state: &State, from: Coordinate) -> Vec<Coordinate> {
    let Some(piece) = from.piece(&state.board) else {
        return Vec::new();
    };

    if state.player != piece.color {
        return Vec::new();
    }

    // TODO: Afterwards we still need to check whether the move won't lead to
    //       the king being taken!

    match piece.figure {
        super::Figure::Pawn => {
            let mut moves = Vec::new();

            // Safe to unwrap here, pawns can always move forward, since they
            // are promoted when they reach the end of the board.
            let single_step = from.forward(piece.color, 1).unwrap();
            let single_step_is_free = single_step.is_free(&state.board);
            if single_step_is_free {
                moves.push(single_step)
            }

            if single_step_is_free && resides_on_pawn_rank(from, piece.color) {
                // Here we can again unwrap safely, since paws can only move
                // two steps, when they reside on the pawn rank.
                let double_step = from.forward(piece.color, 2).unwrap();
                if double_step.is_free(&state.board) {
                    moves.push(double_step);
                }
            }

            let mut capture_moves = Vec::new();
            if let Some(m) = single_step.left(1) {
                capture_moves.push(m);
            }
            if let Some(m) = single_step.right(1) {
                capture_moves.push(m);
            }

            let en_passant_target_origin = state
                .en_passant_target
                .and_then(|target| target.backward(state.player.other(), 1));

            for capture_move in capture_moves {
                if let Some(piece) = capture_move.piece(&state.board) {
                    if piece.color != state.player {
                        moves.push(capture_move)
                    }
                } else if let Some(en_passant_capture_move) = en_passant_target_origin {
                    if en_passant_capture_move == capture_move {
                        moves.push(capture_move)
                    }
                }
            }

            moves
        }

        super::Figure::King => {
            let potential_moves = [
                from.up(1),
                from.right(1),
                from.down(1),
                from.left(1),
                from.diagonal(1, 1),
                from.diagonal(-1, 1),
                from.diagonal(1, -1),
                from.diagonal(-1, -1),
            ];

            let mut moves = Vec::new();
            for potential_move in potential_moves.into_iter().flatten() {
                if potential_move.can_be_moved_to_given(state) {
                    moves.push(potential_move);
                }
            }

            moves
        }

        super::Figure::Rook => expand_straight_until_collides(state, from),

        super::Figure::Bishop => expand_diagonally_until_collides(state, from),

        super::Figure::Queen => {
            let mut moves = Vec::new();

            moves.append(&mut expand_straight_until_collides(state, from));
            moves.append(&mut expand_diagonally_until_collides(state, from));

            moves.sort_by_key(|a| format!("{},{}", a.x, a.y));
            moves.dedup();

            moves
        }

        super::Figure::Knight => {
            let possible = [
                from.up(2).and_then(|m| m.left(1)),
                from.up(1).and_then(|m| m.left(2)),
                from.down(1).and_then(|m| m.left(2)),
                from.down(2).and_then(|m| m.left(1)),
                from.up(2).and_then(|m| m.right(1)),
                from.up(1).and_then(|m| m.right(2)),
                from.down(1).and_then(|m| m.right(2)),
                from.down(2).and_then(|m| m.right(1)),
            ];

            let mut moves = Vec::new();
            for cell in possible.into_iter().flatten() {
                if cell.can_be_moved_to_given(state) {
                    moves.push(cell);
                }
            }

            moves
        }
    }
}

fn expand_straight_until_collides(state: &State, from: Coordinate) -> Vec<Coordinate> {
    expand_until_collides(state, from, [(0, 1), (0, -1), (1, 0), (-1, 0)])
}

fn expand_diagonally_until_collides(state: &State, from: Coordinate) -> Vec<Coordinate> {
    expand_until_collides(state, from, [(1, 1), (1, -1), (-1, 1), (-1, -1)])
}

fn expand_until_collides(
    state: &State,
    from: Coordinate,
    mut into: [(isize, isize); 4],
) -> Vec<Coordinate> {
    let mut cells = Vec::new();

    for direction in into.iter_mut() {
        loop {
            let Some(cell_on_board) = from.diagonal(direction.0, direction.1) else {
                break;
            };

            let Some(collided_piece) = cell_on_board.piece(&state.board) else {
                // If we do not hit a piece, we can advance
                if direction.0 > 0 {
                    direction.0 += 1;
                } else if direction.0 < 0 {
                    direction.0 -= 1;
                }

                if direction.1 > 0 {
                    direction.1 += 1;
                } else if direction.1 < 0 {
                    direction.1 -= 1;
                }

                cells.push(cell_on_board);
                continue;
            };

            // If we hit a piece, we can move there if it belongd to the other player.
            if collided_piece.belongs_to(state.opponent()) {
                cells.push(cell_on_board);
            }

            // In any case, we need to stop iterating after hitting a piece.
            break;
        }
    }

    cells
}

fn resides_on_pawn_rank(from: Coordinate, color: Color) -> bool {
    match color {
        Color::White => from.y == 6,
        Color::Black => from.y == 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::{fen::parse_state, fmt_coordinates, Engine};

    use super::*;

    #[test]
    fn white_pawn_can_move_twice_at_the_beginning() {
        let targets = autocomplete_to(&Engine {}.start(), Coordinate::algebraic("a2").unwrap());

        println!("{:?}", targets);
        assert_eq!(2, targets.len());
        assert!(targets.contains(&Coordinate::algebraic("a3").unwrap()));
        assert!(targets.contains(&Coordinate::algebraic("a4").unwrap()));
    }

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
}
