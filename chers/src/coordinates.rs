use std::fmt::Display;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::Player;

use super::{Board, Color, Piece, BOARD_SIZE};

// An offset relative to the top left (0,0) from white's view
//
// Due to the [`Board`] being layed out as an array
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
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
