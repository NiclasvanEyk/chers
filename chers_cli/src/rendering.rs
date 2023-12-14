use chers::{Board, Color, Coordinate, Figure, Piece};

pub struct TerminalRenderer {}

impl TerminalRenderer {
    pub fn render(&self, board: &Board) {
        clear_terminal();
        println!("{}", show_board(board));
    }
}

// ============================================================================

fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

// ============================================================================

fn show_board(board: &Board) -> String {
    return board
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            row.iter()
                .enumerate()
                .map(|(cell_index, piece)| {
                    let coordinate = Coordinate {
                        x: 8 - cell_index,
                        y: 8 - row_index,
                    };

                    to_terminal_string(coordinate, *piece)
                })
                .collect::<Vec<String>>()
                .join("")
                + format!(" {}", 8 - row_index).as_str()
        })
        .collect::<Vec<String>>()
        .join("\n")
        + "\na b c d e f g h\n";
}

fn to_terminal_string(coordinate: Coordinate, piece: Option<Piece>) -> String {
    let background = match color_for_coordinate(coordinate) {
        Color::White => "\x1b[48;5;216m",
        Color::Black => "\x1b[48;5;173m",
    };
    let foreground = match piece {
        Some(p) => match p.color {
            Color::White => "\x1b[38;2;255;255;255m",
            Color::Black => "\x1b[38;2;0;0;0m",
        },
        None => "",
    };

    let piece_str = match piece {
        Some(p) => piece_to_string(&p),
        None => " ".to_string(),
    };

    format!("{background}{}{} \x1b[0m", foreground, piece_str)
}

fn color_for_coordinate(coordinate: Coordinate) -> Color {
    // Even rows have even numbers black, odd rows have odd ones
    if coordinate.y % 2 == coordinate.x % 2 {
        Color::White
    } else {
        Color::Black
    }
}

fn piece_to_string(piece: &Piece) -> String {
    match piece.color {
        Color::White => match piece.figure {
            Figure::King => "♔",
            Figure::Queen => "♕",
            Figure::Rook => "♜",
            Figure::Bishop => "♗",
            Figure::Knight => "♞",
            Figure::Pawn => "♙",
        },
        Color::Black => match piece.figure {
            Figure::King => "♚",
            Figure::Queen => "♛",
            Figure::Rook => "♜",
            Figure::Bishop => "♝",
            Figure::Knight => "♞",
            Figure::Pawn => "♙",
        },
    }
    .to_string()
}
