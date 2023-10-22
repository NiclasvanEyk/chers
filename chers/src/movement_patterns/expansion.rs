use crate::{piece_at, Board, Coordinate, Player};

/// Expand in direction of the movement vectors given by [into] until you
/// hit a piece.
///
/// The cell where a piece is hit will be included in the returned coordinate
/// set, since rooks, queens and bishops can capture the piece in the direction
/// they move to.
pub fn expand_until_collides(
    board: &Board,
    from: Coordinate,
    player: Player,
    mut into: [(isize, isize); 4],
) -> Vec<Coordinate> {
    let mut cells = Vec::new();

    for direction in into.iter_mut() {
        loop {
            let Some(cell_on_board) = from.diagonal(direction.0, direction.1) else {
                break;
            };

            let Some(collided_piece) = piece_at(cell_on_board, board) else {
                // If we do not hit a piece, we can advance
                match direction.0.cmp(&0) {
                    std::cmp::Ordering::Less => {
                        direction.0 -= 1;
                    }
                    std::cmp::Ordering::Equal => {}
                    std::cmp::Ordering::Greater => {
                        direction.0 += 1;
                    }
                }

                match direction.1.cmp(&0) {
                    std::cmp::Ordering::Less => {
                        direction.1 -= 1;
                    }
                    std::cmp::Ordering::Equal => {}
                    std::cmp::Ordering::Greater => {
                        direction.1 += 1;
                    }
                }

                cells.push(cell_on_board);
                continue;
            };

            // If we hit a piece, we can move there if it belongd to the other player.
            if collided_piece.belongs_to(player.other()) {
                cells.push(cell_on_board);
            }

            // In any case, we need to stop iterating after hitting a piece.
            break;
        }
    }

    cells
}
