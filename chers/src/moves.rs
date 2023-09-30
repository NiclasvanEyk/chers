use super::{Color, Coordinate, Piece, State};

pub fn autocomplete_to(state: &State, from: Coordinate, piece: Piece) -> Vec<Coordinate> {
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

            for capture_move in capture_moves {
                if let Some(piece) = capture_move.piece(&state.board) {
                    if piece.color != state.player {
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
    let mut cells = Vec::new();

    let mut directions = [1, -1, 1, -1];
    for (index, direction) in directions.iter_mut().enumerate() {
        loop {
            if index % 2 == 0 {
                match from.horizontal(*direction) {
                    None => break,
                    Some(cell) => {
                        if !cell.can_be_moved_to_given(state) {
                            break;
                        }

                        cells.push(cell)
                    }
                }
            } else {
                match from.vertical(*direction) {
                    None => break,
                    Some(cell) => {
                        if !cell.can_be_moved_to_given(state) {
                            break;
                        }

                        cells.push(cell)
                    }
                }
            }

            if *direction > 0 {
                *direction += 1;
            } else {
                *direction -= 1;
            }
        }
    }

    cells
}

fn expand_diagonally_until_collides(state: &State, from: Coordinate) -> Vec<Coordinate> {
    let mut cells = Vec::new();

    let mut vectors = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    for vector in vectors.iter_mut() {
        loop {
            match from.diagonal(vector.0, vector.1) {
                None => break,
                Some(cell) => {
                    if !cell.can_be_moved_to_given(state) {
                        break;
                    }

                    cells.push(cell)
                }
            }

            if vector.0 > 0 {
                vector.0 += 1;
            } else {
                vector.0 -= 1;
            }

            if vector.1 > 0 {
                vector.1 += 1;
            } else {
                vector.1 -= 1;
            }
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
    use crate::{Engine, Figure::Pawn};

    use super::*;

    #[test]
    fn white_pawn_can_move_twice_at_the_beginning() {
        let targets = autocomplete_to(
            &Engine {}.start(),
            Coordinate::algebraic("a2").unwrap(),
            Piece::white(Pawn),
        );

        println!("{:?}", targets);
        assert_eq!(2, targets.len());
        assert!(targets.contains(&Coordinate::algebraic("a3").unwrap()));
        assert!(targets.contains(&Coordinate::algebraic("a4").unwrap()));
    }
}
