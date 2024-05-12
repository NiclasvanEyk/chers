use std::fmt::Display;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

use crate::Player;

use super::{Board, Color, Piece, BOARD_SIZE};

// An offset relative to the top left (0,0) from white's view
//
// Due to the [`Board`] being layed out as an array
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

pub struct Cell {}

impl Cell {
    pub const A1: Coordinate = Coordinate { x: 0, y: 0 };
    pub const A2: Coordinate = Coordinate { x: 0, y: 1 };
    pub const A3: Coordinate = Coordinate { x: 0, y: 2 };
    pub const A4: Coordinate = Coordinate { x: 0, y: 3 };
    pub const A5: Coordinate = Coordinate { x: 0, y: 4 };
    pub const A6: Coordinate = Coordinate { x: 0, y: 5 };
    pub const A7: Coordinate = Coordinate { x: 0, y: 6 };
    pub const A8: Coordinate = Coordinate { x: 0, y: 7 };
    pub const B1: Coordinate = Coordinate { x: 1, y: 0 };
    pub const B2: Coordinate = Coordinate { x: 1, y: 1 };
    pub const B3: Coordinate = Coordinate { x: 1, y: 2 };
    pub const B4: Coordinate = Coordinate { x: 1, y: 3 };
    pub const B5: Coordinate = Coordinate { x: 1, y: 4 };
    pub const B6: Coordinate = Coordinate { x: 1, y: 5 };
    pub const B7: Coordinate = Coordinate { x: 1, y: 6 };
    pub const B8: Coordinate = Coordinate { x: 1, y: 7 };
    pub const C1: Coordinate = Coordinate { x: 2, y: 0 };
    pub const C2: Coordinate = Coordinate { x: 2, y: 1 };
    pub const C3: Coordinate = Coordinate { x: 2, y: 2 };
    pub const C4: Coordinate = Coordinate { x: 2, y: 3 };
    pub const C5: Coordinate = Coordinate { x: 2, y: 4 };
    pub const C6: Coordinate = Coordinate { x: 2, y: 5 };
    pub const C7: Coordinate = Coordinate { x: 2, y: 6 };
    pub const C8: Coordinate = Coordinate { x: 2, y: 7 };
    pub const D1: Coordinate = Coordinate { x: 3, y: 0 };
    pub const D2: Coordinate = Coordinate { x: 3, y: 1 };
    pub const D3: Coordinate = Coordinate { x: 3, y: 2 };
    pub const D4: Coordinate = Coordinate { x: 3, y: 3 };
    pub const D5: Coordinate = Coordinate { x: 3, y: 4 };
    pub const D6: Coordinate = Coordinate { x: 3, y: 5 };
    pub const D7: Coordinate = Coordinate { x: 3, y: 6 };
    pub const D8: Coordinate = Coordinate { x: 3, y: 7 };
    pub const E1: Coordinate = Coordinate { x: 4, y: 0 };
    pub const E2: Coordinate = Coordinate { x: 4, y: 1 };
    pub const E3: Coordinate = Coordinate { x: 4, y: 2 };
    pub const E4: Coordinate = Coordinate { x: 4, y: 3 };
    pub const E5: Coordinate = Coordinate { x: 4, y: 4 };
    pub const E6: Coordinate = Coordinate { x: 4, y: 5 };
    pub const E7: Coordinate = Coordinate { x: 4, y: 6 };
    pub const E8: Coordinate = Coordinate { x: 4, y: 7 };
    pub const F1: Coordinate = Coordinate { x: 5, y: 0 };
    pub const F2: Coordinate = Coordinate { x: 5, y: 1 };
    pub const F3: Coordinate = Coordinate { x: 5, y: 2 };
    pub const F4: Coordinate = Coordinate { x: 5, y: 3 };
    pub const F5: Coordinate = Coordinate { x: 5, y: 4 };
    pub const F6: Coordinate = Coordinate { x: 5, y: 5 };
    pub const F7: Coordinate = Coordinate { x: 5, y: 6 };
    pub const F8: Coordinate = Coordinate { x: 5, y: 7 };
    pub const G1: Coordinate = Coordinate { x: 6, y: 0 };
    pub const G2: Coordinate = Coordinate { x: 6, y: 1 };
    pub const G3: Coordinate = Coordinate { x: 6, y: 2 };
    pub const G4: Coordinate = Coordinate { x: 6, y: 3 };
    pub const G5: Coordinate = Coordinate { x: 6, y: 4 };
    pub const G6: Coordinate = Coordinate { x: 6, y: 5 };
    pub const G7: Coordinate = Coordinate { x: 6, y: 6 };
    pub const G8: Coordinate = Coordinate { x: 6, y: 7 };
    pub const H1: Coordinate = Coordinate { x: 7, y: 0 };
    pub const H2: Coordinate = Coordinate { x: 7, y: 1 };
    pub const H3: Coordinate = Coordinate { x: 7, y: 2 };
    pub const H4: Coordinate = Coordinate { x: 7, y: 3 };
    pub const H5: Coordinate = Coordinate { x: 7, y: 4 };
    pub const H6: Coordinate = Coordinate { x: 7, y: 5 };
    pub const H7: Coordinate = Coordinate { x: 7, y: 6 };
    pub const H8: Coordinate = Coordinate { x: 7, y: 7 };
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

        write!(f, "{}{}", x, y)
    }
}

#[derive(Debug)]
pub enum CoordinateParseError {
    UnknownLetter(char),
    MissingXCoordinate,
    MissingYCoordinate,
    NonDigitYCoordinate(char),
}

impl Display for CoordinateParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            CoordinateParseError::UnknownLetter(letter) => format!("Unknown letter: '{letter}'"),
            CoordinateParseError::MissingXCoordinate => String::from("No x coordinate specified"),
            CoordinateParseError::MissingYCoordinate => String::from("No y coordinate specified"),
            CoordinateParseError::NonDigitYCoordinate(letter) => {
                format!("y must represent a digit. '{letter}' passed.")
            }
        };

        write!(f, "{}", message)
    }
}

impl Coordinate {
    pub fn parse(string: &str) -> Result<Self, CoordinateParseError> {
        let mut characters = string.chars();

        let Some(x_raw) = characters.next() else {
            return Err(CoordinateParseError::MissingXCoordinate);
        };

        let x = match x_raw {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            letter => return Err(CoordinateParseError::UnknownLetter(letter)),
        };

        let Some(y_raw) = characters.next() else {
            return Err(CoordinateParseError::MissingYCoordinate);
        };

        let Some(y) = y_raw.to_digit(10) else {
            return Err(CoordinateParseError::NonDigitYCoordinate(y_raw));
        };

        Ok(Coordinate { x, y: y as usize })
    }
}

#[derive(Debug)]
#[wasm_bindgen]
pub enum CoordinateParserError {
    Empty,
    MissingColumn,
    InvalidColumn,
    MissingRow,
    InvalidRow,
}

#[wasm_bindgen]
impl Coordinate {
    #[wasm_bindgen(constructor)]
    pub fn new(x: usize, y: usize) -> Coordinate {
        Coordinate { x, y }
    }

    pub fn horizontal(&self, amount: isize) -> Option<Coordinate> {
        let Some(x) = self.x.checked_add_signed(amount) else {
            return None;
        };

        if x >= BOARD_SIZE {
            return None;
        }

        Some(Coordinate { x, y: self.y })
    }

    pub fn vertical(&self, amount: isize) -> Option<Coordinate> {
        let Some(y) = self.y.checked_add_signed(amount) else {
            return None;
        };

        if y >= BOARD_SIZE {
            return None;
        }

        Some(Coordinate { x: self.x, y })
    }

    pub fn diagonal(&self, horizontal: isize, vertical: isize) -> Option<Coordinate> {
        let Some(first) = self.horizontal(horizontal) else {
            return None;
        };

        first.vertical(vertical)
    }

    pub fn forward(&self, color: Color, amount: isize) -> Option<Coordinate> {
        self.vertical(match color {
            super::Color::White => -amount,
            super::Color::Black => amount,
        })
    }

    pub fn backward(&self, color: Color, amount: isize) -> Option<Coordinate> {
        self.forward(color, -amount)
    }

    pub fn up(&self, amount: isize) -> Option<Coordinate> {
        self.vertical(amount)
    }

    pub fn down(&self, amount: isize) -> Option<Coordinate> {
        self.vertical(-amount)
    }

    pub fn right(&self, amount: isize) -> Option<Coordinate> {
        self.horizontal(amount)
    }

    pub fn left(&self, amount: isize) -> Option<Coordinate> {
        self.horizontal(-amount)
    }

    // TODO: Those two don't really fit in here. Maybe move somewhere else?
}

impl Coordinate {
    /// Parses a coordinate in algebraic notation
    pub fn algebraic(input: &str) -> Result<Coordinate, CoordinateParserError> {
        let normalized = input.trim();
        if normalized.is_empty() {
            return Err(CoordinateParserError::Empty);
        }

        let characters: Vec<char> = normalized.chars().take(2).collect();

        let x = match characters.first() {
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

        Ok(Coordinate { x, y })
    }
}

pub fn is_free(coord: Coordinate, board: &Board) -> bool {
    piece_at(coord, board).is_none()
}

pub fn piece_at(coord: Coordinate, board: &Board) -> Option<Piece> {
    board[coord.y][coord.x]
}

pub fn can_be_moved_to_given(to: Coordinate, by: Player, board: &Board) -> bool {
    match piece_at(to, board) {
        None => true,
        Some(piece) => piece.color != by,
    }
}

pub fn fmt_coordinates(coordinates: &[Coordinate]) -> String {
    let mut s = String::from("[");

    s += coordinates
        .iter()
        .map(|coordinate| coordinate.to_string())
        .collect::<Vec<String>>()
        .join(", ")
        .as_str();

    s += "]";
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_and_parsing() {
        assert_eq!(
            Coordinate { x: 0, y: 0 },
            Coordinate::algebraic("a8").unwrap(),
        );
        assert_eq!(
            Coordinate { x: 0, y: 7 },
            Coordinate::algebraic("a1").unwrap(),
        );

        let coord = Coordinate { x: 3, y: 4 };
        assert_eq!(
            Coordinate::algebraic(coord.to_string().as_str()).unwrap(),
            coord
        );
    }
}
