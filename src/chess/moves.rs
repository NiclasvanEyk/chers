use super::{Color, Coordinate, Piece, State, BOARD_SIZE};

pub fn autocomplete_to(state: &State, from: Coordinate, piece: Piece) -> Vec<Coordinate> {
    if state.player != piece.color {
        return Vec::new();
    }

    match piece.figure {
        super::Figure::Pawn => {
            let mut moves = Vec::new();

            let single_step = from.forward(piece.color, 1);
            println!("{:?}", single_step);
            let single_step_is_free = single_step.is_free(&state.board);
            if single_step_is_free {
                moves.push(single_step)
            }

            if single_step_is_free && resides_on_home_rank(from, piece.color) {
                let double_step = from.forward(piece.color, 2);
                if double_step.is_free(&state.board) {
                    moves.push(double_step);
                }
            }

            let mut capture_moves = Vec::new();
            if from.x > 0 {
                capture_moves.push(single_step.sideways_left(1));
            }
            if from.y < BOARD_SIZE - 1 {
                capture_moves.push(single_step.sideways_right(1));
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

        super::Figure::King => todo!(),
        super::Figure::Queen => todo!(),
        super::Figure::Rook => todo!(),
        super::Figure::Bishop => todo!(),
        super::Figure::Knight => todo!(),
    }
}

fn resides_on_home_rank(from: Coordinate, color: Color) -> bool {
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
