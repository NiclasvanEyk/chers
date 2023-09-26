use std::{fmt::Display, io::Write};

use crate::chess::{Board, Color, Coordinate, Engine, Figure, Move, Piece, State};

pub struct TerminalChersMatch {
    engine: Engine,
    state: State,
}

impl TerminalChersMatch {
    pub fn new(engine: Engine) -> Self {
        let initial_state = engine.start();

        return Self {
            engine,
            state: initial_state,
        };
    }

    pub fn run(&mut self) {
        loop {
            clear_terminal();
            println!("{}", show_board(self.state.board));

            let r#move = prompt_for_move(&self.state.player);

            match self.engine.move_piece(&self.state, r#move) {
                Err(error) => {
                    println!("{:?}", error);
                }
                Ok((new_state, events)) => {
                    self.state = new_state;
                    for event in events {
                        println!("{:?}", event);
                    }
                }
            };
        }
    }
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

#[derive(Debug)]
pub enum CoordinateParserError {
    Empty,
    MissingColumn,
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
            None => return Err(CoordinateParserError::MissingColumn),
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

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self.x {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => '?',
        };

        let y = 8 - self.y;

        return write!(f, "{}{}", x.to_uppercase(), y);
    }
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

#[derive(Debug)]
enum ReadMoveError {
    Empty,
    InvalidFrom(CoordinateParserError),
    InvalidTo(CoordinateParserError),
}

fn parse_move(input: &str) -> Result<Move, ReadMoveError> {
    let words: Vec<&str> = input.trim().split(' ').take(2).collect();

    let from = match words.get(0) {
        Some(word) => match Coordinate::parse(word) {
            Ok(coordinate) => coordinate,
            Err(error) => return Err(ReadMoveError::InvalidFrom(error)),
        },
        None => return Err(ReadMoveError::InvalidFrom(CoordinateParserError::Empty)),
    };

    let to = match words.get(1) {
        Some(word) => match Coordinate::parse(word) {
            Ok(coordinate) => coordinate,
            Err(error) => return Err(ReadMoveError::InvalidTo(error)),
        },
        None => return Err(ReadMoveError::InvalidFrom(CoordinateParserError::Empty)),
    };

    return Ok(Move { from, to });
}

fn prompt_for_move(player: &Color) -> Move {
    loop {
        let mut x = String::new();
        print!("{:?}'s turn: ", player);
        std::io::stdout().flush().expect("Could not flush stdout");

        std::io::stdin()
            .read_line(&mut x)
            .expect("Could not read input");

        match parse_move(&x) {
            Ok(from_to) => return from_to,
            Err(err) => println!("{:?}", err),
        };
    }
}

fn to_terminal_string(coordinate: Coordinate, piece: Option<Piece>) -> String {
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

pub fn show_board(board: Board) -> String {
    return board
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            row.iter()
                .enumerate()
                .map(|(cell_index, piece)| {
                    let coordinate =
                        Coordinate::checked_from_board_index(cell_index, row_index).unwrap();

                    return to_terminal_string(coordinate, *piece);
                })
                .collect::<Vec<String>>()
                .join("")
        })
        .collect::<Vec<String>>()
        .join("\n");
}

pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}
