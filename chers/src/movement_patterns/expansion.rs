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
            println!("Checking {direction:?}");
            let Some(cell_on_board) = from.diagonal(direction.0, direction.1) else {
                println!("Moved out of the board! Stopping iteration in direction");
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

            println!("Hit a piece! Stopping iteration in direction");

            // If we hit a piece, we can move there if it belongd to the other player.
            if collided_piece.belongs_to(player.other()) {
                println!("Adding piece to possible moves");
                cells.push(cell_on_board);
            } else {
                println!("Hit our own piece, cant move there");
            }

            // In any case, we need to stop iterating after hitting a piece.
            break;
        }
    }

    cells
}
