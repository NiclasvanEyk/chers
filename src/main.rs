enum Color {
    White,
    Black,
}

impl Color {
    fn for_coordinate(coordinate: Coordinate) -> Color {
        // Even rows have even numbers black, odd rows have odd ones
        if coordinate.y % 2 == coordinate.x % 2 {
            return Color::Black;
        } else {
            return Color::White;
        }
    }
}

enum Figure {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl ToString for Coordinate {
    fn to_string(&self) -> String {
        let letter = format!("{:?}", self.x as u8 as char);
        return format!("{}{}", letter, self.y + 1,);
    }
}

impl Coordinate {
    pub fn checked_from_board_index(row: usize, column: usize) -> Option<Coordinate> {
        return Coordinate::checked(8 - row, 8 - column);
    }

    pub fn checked(x: usize, y: usize) -> Option<Coordinate> {
        if x > 8 || y > 8 {
            return Option::None;
        }

        return Option::Some(Coordinate { x: x, y: y });
    }
}

struct Piece {
    pub color: Color,
    pub figure: Figure,
}

impl ToString for Piece {
    fn to_string(&self) -> String {
        return match self.color {
            Color::White => match self.figure {
                Figure::King => "♔",
                Figure::Queen => "♕",
                Figure::Rook => "♖",
                Figure::Bishop => "♗",
                Figure::Knight => "♘",
                Figure::Pawn => "♙",
            },
            Color::Black => match self.figure {
                Figure::King => "♚",
                Figure::Queen => "♛",
                Figure::Rook => "♜",
                Figure::Bishop => "♝",
                Figure::Knight => "♞",
                Figure::Pawn => "♟︎",
            },
        }
        .to_string();
    }
}

struct Move {
    pub piece: Piece,
    pub from: Coordinate,
    pub to: Coordinate,
}

type Board = [[Option<Piece>; 8]; 8];

const INITIAL_BOARD: Board = [
    [
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Rook,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Knight,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Bishop,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Queen,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::King,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Bishop,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Knight,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Rook,
        }),
    ],
    [
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::Black,
            figure: Figure::Pawn,
        }),
    ],
    [
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
    ],
    [
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
    ],
    [
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
    ],
    [
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
        Option::None,
    ],
    [
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Pawn,
        }),
    ],
    [
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Rook,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Knight,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Bishop,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Queen,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::King,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Bishop,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Knight,
        }),
        Option::Some(Piece {
            color: Color::White,
            figure: Figure::Rook,
        }),
    ],
];

fn to_terminal_string(coordinate: Coordinate, piece: &Option<Piece>) -> String {
    let prefix = match Color::for_coordinate(coordinate) {
        Color::White => "\x1b[47m",
        Color::Black => "\x1b[40m",
    };

    let piece_str = match piece {
        Some(p) => p.to_string(),
        None => " ".to_string(),
    };

    return format!("{}{}\x1b[0m", prefix, piece_str);
}

fn show_board(board: Board) -> String {
    return board
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            row.iter()
                .enumerate()
                .map(|(cell_index, piece)| {
                    let coordinate =
                        Coordinate::checked_from_board_index(cell_index, row_index).unwrap();

                    return to_terminal_string(coordinate, piece);
                })
                .collect::<Vec<String>>()
                .join("")
        })
        .collect::<Vec<String>>()
        .join("\n")
        .to_string();
}

fn main() {
    println!("{}\n\n", show_board(INITIAL_BOARD));
}
