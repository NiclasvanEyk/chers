use std::io::Write;

enum Color {
    White,
    Black,
}

impl Color {
    fn for_coordinate(coordinate: Coordinate) -> Color {
        // Even rows have even numbers black, odd rows have odd ones
        if coordinate.y % 2 == coordinate.x % 2 {
            return Color::White;
        } else {
            return Color::Black;
        }
    }

    pub fn switch(self) -> Color {
        return match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
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

#[derive(Debug)]
enum CoordinateParserError {
    Empty,
    MissinColumn,
    InvalidColumn,
    MissingRow,
    InvalidRow,
}

impl Coordinate {
    pub fn checked_from_board_index(row: usize, column: usize) -> Option<Coordinate> {
        return Coordinate::checked(8 - row, 8 - column);
    }

    pub fn checked(x: usize, y: usize) -> Option<Coordinate> {
        if x > 8 || y > 8 {
            return Option::None;
        }

        return Option::Some(Coordinate { x, y });
    }

    pub fn parse(input: &str) -> Result<Coordinate, CoordinateParserError> {
        let normalized = input.trim();
        if normalized.is_empty() {
            return Err(CoordinateParserError::Empty);
        }

        let characters: Vec<char> = normalized.chars().take(2).collect();

        let x = match characters.get(0) {
            Some(column) => match column.to_ascii_lowercase() {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                'f' => 5,
                'g' => 6,
                'h' => 7,
                _ => return Err(CoordinateParserError::InvalidColumn),
            },
            None => return Err(CoordinateParserError::MissinColumn),
        };

        let y = match characters.get(1) {
            Some(row) => match row.to_ascii_lowercase() {
                '1' => 7,
                '2' => 6,
                '3' => 5,
                '4' => 4,
                '5' => 3,
                '6' => 2,
                '7' => 1,
                '8' => 0,
                _ => return Err(CoordinateParserError::InvalidRow),
            },
            None => return Err(CoordinateParserError::MissingRow),
        };

        return Ok(Coordinate { x, y });
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
                Figure::Rook => "♜",
                Figure::Bishop => "♗",
                Figure::Knight => "♞",
                Figure::Pawn => "♙",
            },
            Color::Black => match self.figure {
                Figure::King => "♚",
                Figure::Queen => "♛",
                Figure::Rook => "♜",
                Figure::Bishop => "♝",
                Figure::Knight => "♞",
                Figure::Pawn => "♙",
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

#[derive(Debug)]
enum ReadMoveError {
    Empty,
    InvalidFrom(CoordinateParserError),
    InvalidTo(CoordinateParserError),
}

fn parse_move(input: &str) -> Result<(Coordinate, Coordinate), ReadMoveError> {
    let words: Vec<&str> = input.trim().split(' ').take(2).collect();

    let from = match words.get(0) {
        Some(word) => match Coordinate::parse(word) {
            Ok(coordinate) => coordinate,
            Err(error) => return Err(ReadMoveError::InvalidFrom(error)),
        },
        None => return Err(ReadMoveError::InvalidFrom(CoordinateParserError::Empty)),
    };

    let to = match words.get(0) {
        Some(word) => match Coordinate::parse(word) {
            Ok(coordinate) => coordinate,
            Err(error) => return Err(ReadMoveError::InvalidTo(error)),
        },
        None => return Err(ReadMoveError::InvalidFrom(CoordinateParserError::Empty)),
    };

    return Ok((from, to));
}

fn prompt_for_move() -> (Coordinate, Coordinate) {
    let mut x = String::new();

    loop {
        print!("Your move: ");
        std::io::stdout().flush();

        std::io::stdin().read_line(&mut x);
        match parse_move(&x) {
            Ok(from_to) => return from_to,
            Err(err) => println!("{:?}", err),
        };
    }
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
    let background = match Color::for_coordinate(coordinate) {
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
        Some(p) => p.to_string(),
        None => " ".to_string(),
    };

    return format!("{}{}{} \x1b[0m", background, foreground, piece_str);
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

struct Match {
    board: Board,
}

enum AdvanceError {}

impl Match {
    pub fn new() -> Match {
        return Match {
            board: INITIAL_BOARD,
        };
    }

    pub fn advance(&self, from: Coordinate, to: Coordinate) {}

    pub fn checkmate(&self) -> bool {
        return false;
    }
}

fn main() {
    let game = Match::new();
    let mut player = Color::White;

    while !game.checkmate() {
        println!("{}\n", show_board(INITIAL_BOARD));

        let (from, to) = prompt_for_move();
        game.advance(from, to);

        player = player.switch()
    }
}
