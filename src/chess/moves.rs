use super::{Color, Coordinate, Piece, State, BOARD_SIZE};

enum Axis {
    Straight,
    Diagonal,
}

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
            let mut potential_moves = [
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
            for potential_move in potential_moves {
                let Some(potential_move) = potential_move else {
                    continue;
                };

                if potential_move.can_be_moved_to_by(state, &piece) {
                    moves.push(potential_move);
                }
            }

            moves
        }

        super::Figure::Queen => todo!(),
        super::Figure::Rook => todo!(),
        super::Figure::Bishop => todo!(),
        super::Figure::Knight => todo!(),
    }
}

fn expand_until_collision(from: Coordinate, state: &State, axis: Axis) -> Vec<Coordinate> {
    match axis {
        Axis::Straight => todo!(),
        Axis::Diagonal => todo!(),
    }
}

fn resides_on_pawn_rank(from: Coordinate, color: Color) -> bool {
    match color {
        Color::White => from.y == 6,
        Color::Black => from.y == 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::chess::{Engine, Figure::Pawn};

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
