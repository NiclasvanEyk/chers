use crate::{pieces, Color, State};

/// Computes a score for each player roughly based on
/// https://www.pi.infn.it/%7Ecarosi/chess/shannon.txt
pub fn shannon_value(state: &State) -> (u8, u8) {
    let mut white = 0;
    let mut black = 0;

    for (_, piece) in pieces(&state.board) {
        let score = match piece.figure {
            crate::Figure::King => 200,
            crate::Figure::Queen => 9,
            crate::Figure::Rook => 5,
            crate::Figure::Bishop => 3,
            crate::Figure::Knight => 3,
            crate::Figure::Pawn => 1,
        };

        if piece.color == Color::White {
            white += score
        } else {
            black += score
        }
    }

    (white, black)
}
